//! The field in play: delivers the bells' pulses to the stars that peoples
//! inhabit, keeps each star's absorbed-energy books, moves the star's worlds
//! and the star itself, and reads the radiation and environment of each
//! people.
//!
//! Inputs: the [`Cosmic`] world (peoples' stars, witnessed systems, star
//! places) and the bells from `xyl_cosmos::pulse` (the core and the holes
//! among the first sample stars) with their laws in `xyl_cosmos::field`
//! (`path`, `absorb`, `burning`).
//!
//! - Delivery ([`Field::step`]): only stars with peoples are reached. For
//!   each bell the arrivals in (from, to] Gyr are counted in closed form;
//!   each ring is absorbed where the worlds stand, and once a ring leaves
//!   them unchanged the remaining rings are counted rather than repeated,
//!   so the result does not depend on how the span is cut. Stars are
//!   computed in parallel.
//! - Books ([`Books`]): energy in = gravity + light + burned out, exact in
//!   `Q` ([`Books::balances`]).
//! - Anchors and drift: each star's worlds as anchors from its G point
//!   (AU), and the star's offset from its galactic orbit (AU, summed per
//!   axis), which shifts the arrival times of later pulses.
//! - Radiation ([`Field::radiation`]): the star's absorbed energy per year
//!   over the world's orbital radius.
//! - Environment ([`Field::environment`]): 1 at or below the people's tuned
//!   home radiation, home / radiation above it; the tuning follows the
//!   current radiation at 1 - e^(-years / generation).
//!
//! Books, anchors and drift are regenerated from the seed; each people's
//! tuned home radiation is held in the checkpoint.
//!
//! Terms: bell, books (docs/glossary.md).

use crate::cosmic::Cosmic;
use std::collections::BTreeMap;
use xyl_core::dyadic::Dy;
use xyl_core::Q;
use xyl_cosmos::chain::radius;
use xyl_cosmos::field::{absorb, au_per_kpc, bells, burning, path, Absorbed, Anchor, Bell};
use xyl_cosmos::pulse::{nodes, nodes_among, Node};

/// Number of sample stars searched for bells: the bells are the core plus the
/// holes among the first `SAMPLE` stars and all holes of the nucleus.
const SAMPLE: u64 = 64;

/// The cumulative energy books of one star: every pulse absorbed and how its
/// energy was divided.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Books {
    /// Energy absorbed from all pulses since the first, in the field's energy
    /// units (xyl-cosmos field.rs).
    pub energy_in: Q,
    /// Part of `energy_in` spent re-locking the star's worlds on their orbits.
    pub gravity: Q,
    /// Part of `energy_in` given out as light: the remainder after gravity and
    /// burn-out.
    pub light: Q,
    /// Part of `energy_in` converted by the star's burning step: the star's
    /// mass-defect share of what gravity left.
    pub burned_out: Q,
    /// Number of pulses absorbed.
    pub pulses: u64,
    /// Time of the first absorbed pulse, in Gyr; `None` before any.
    pub since: Option<Dy>,
}

impl Books {
    fn add(&mut self, a: &Absorbed, k: u64, at: Dy) {
        let n = Q::int(i64::try_from(k).unwrap_or(i64::MAX));
        self.energy_in = &self.energy_in + &(&a.energy_in * &n);
        self.gravity = &self.gravity + &(&a.gravity * &n);
        self.light = &self.light + &(&a.light * &n);
        self.burned_out = &self.burned_out + &(&a.burned_out * &n);
        self.pulses += k;
        self.since.get_or_insert(at);
    }

    /// Returns whether `energy_in = gravity + light + burned_out` holds exactly.
    pub fn balances(&self) -> bool {
        &(&self.gravity + &self.light) + &self.burned_out == self.energy_in
    }

    /// Returns the absorbed energy per year since the first pulse, at `now`
    /// Gyr (the factor 10^9 converts Gyr to years); zero before the first
    /// pulse or when `now` is not later than it.
    pub fn per_year(&self, now: Dy) -> Dy {
        match self.since {
            Some(t) if now > t => Dy::from_q(&self.energy_in) / ((now - t) * Dy::sci(1, 9)),
            _ => Dy::ZERO,
        }
    }
}

/// The field state of the universe in play: the bells and each reached
/// star's books, anchors and drift.
#[derive(Clone, Debug, Default)]
pub struct Field {
    bells: Vec<(Node, Bell)>,
    /// Each reached star's books, by star.
    pub books: BTreeMap<u64, Books>,
    /// Each people's tuned home radiation, the level the people's repair is
    /// adapted to (indexed like `cosmic.peoples`; `None` until the people
    /// first receives nonzero radiation).
    pub home: Vec<Option<Dy>>,
    /// Each reached star's world anchors, in AU from the star's G point.
    pub anchors: BTreeMap<u64, Vec<Anchor>>,
    /// Each reached star's offset from its galactic orbit caused by the
    /// pulses, in AU per axis.
    pub drift: BTreeMap<u64, [Q; 3]>,
}

impl Field {
    /// Returns the field of `cosmic`'s bells with empty books. `heavy`, when
    /// given, lists the sample stars already known to leave holes
    /// (`Field::heavy` of an earlier build), which skips the exact mass check
    /// of every candidate.
    pub fn of(cosmic: &Cosmic<'_>, heavy: Option<&[u64]>) -> Field {
        let ns = match heavy {
            Some(ks) => nodes_among(cosmic.gal.seed, cosmic.gal.bang, ks),
            None => nodes(cosmic.gal.seed, cosmic.gal.bang, Some(SAMPLE)),
        };
        let bs = bells(&ns, cosmic.gal.seed);
        Field {
            bells: ns.into_iter().zip(bs).collect(),
            books: BTreeMap::new(),
            home: Vec::new(),
            anchors: BTreeMap::new(),
            drift: BTreeMap::new(),
        }
    }

    /// Returns the indices of the sample stars whose holes are bells, excluding
    /// the core.
    pub fn heavy(&self) -> Vec<u64> {
        self.bells
            .iter()
            .filter_map(|(n, _)| match &n.kind {
                xyl_cosmos::pulse::Kind::Stellar(s) => Some(s.n),
                xyl_cosmos::pulse::Kind::Core(_) => None,
            })
            .collect()
    }

    /// Number of bells in the field: the core and the sampled heavy stars.
    pub fn bells(&self) -> usize {
        self.bells.len()
    }

    /// Returns the bells' nodes, for other consumers such as percepts. Sampled once
    /// here, since finding the heavy stars checks every candidate's mass
    /// exactly.
    pub fn nodes(&self) -> Vec<Node> {
        self.bells.iter().map(|(n, _)| n.clone()).collect()
    }

    /// Delivers every pulse arriving in `(from, to]` Gyr at the stars peoples
    /// inhabit; each star absorbs each ring by `xyl_cosmos::field::absorb`.
    /// Returns the number of pulses delivered.
    pub fn step(&mut self, cosmic: &Cosmic<'_>, from: Dy, to: Dy) -> u64 {
        use rayon::prelude::*;
        // Stars' books, worlds and drift do not depend on each other, so each
        // star's state is taken out, updated in parallel and stored back.
        let stars: std::collections::BTreeSet<u64> =
            cosmic.peoples.iter().map(|p| p.star).collect();
        let held: Vec<(u64, Option<Vec<Anchor>>, Option<[Q; 3]>, Option<Books>)> = stars
            .into_iter()
            .map(|k| {
                (
                    k,
                    self.anchors.remove(&k),
                    self.drift.remove(&k),
                    self.books.remove(&k),
                )
            })
            .collect();
        let (systems, bells) = (&cosmic.systems, &self.bells);
        let done: Vec<(u64, Option<Vec<Anchor>>, Option<[Q; 3]>, Option<Books>, u64)> = held
            .into_par_iter()
            .map(|(star, anchors, drift0, books0)| {
                ring_star(star, (anchors, drift0, books0), systems, bells, from, to)
            })
            .collect();
        let mut delivered = 0;
        for (star, anchors, drift, books, n) in done {
            if let Some(a) = anchors {
                self.anchors.insert(star, a);
            }
            if let Some(d) = drift {
                self.drift.insert(star, d);
            }
            if let Some(b) = books {
                self.books.insert(star, b);
            }
            delivered += n;
        }
        delivered
    }

    /// Returns the quantum radiation a people's world receives at `now`
    /// (energy per year per AU): the star's absorbed energy per year divided
    /// by the world's orbital radius in AU (floored at 10^-6), so the reading
    /// falls as 1 / a. Returns zero when the people, the star's books or the
    /// world are missing.
    pub fn radiation(&self, cosmic: &Cosmic<'_>, people: usize, now: Dy) -> Dy {
        let Some(p) = cosmic.peoples.get(people) else {
            return Dy::ZERO;
        };
        let (Some(b), Some(w)) = (self.books.get(&p.star), cosmic.systems.get(&p.star)) else {
            return Dy::ZERO;
        };
        let Some(world) = w.system.worlds.get(p.world) else {
            return Dy::ZERO;
        };
        let a = Dy::from_q(&world.a).max(Dy::sci(1, -6));
        b.per_year(now) / a
    }

    /// Returns a people's environment factor from `radiation` over `years`,
    /// with generation time `gen_yr`: 1 at or below the tuned home radiation,
    /// else home / radiation. The first nonzero radiation becomes the home
    /// radiation and returns 1. The home radiation then moves toward the
    /// current radiation with weight 1 - e^(-years / gen_yr).
    pub fn environment(&mut self, people: usize, radiation: Dy, years: Dy, gen_yr: Dy) -> Dy {
        if self.home.len() <= people {
            self.home.resize(people + 1, None);
        }
        let home = match self.home[people] {
            None if radiation.signum() > 0 => {
                self.home[people] = Some(radiation);
                return Dy::ONE;
            }
            None => return Dy::ONE,
            Some(h) => h,
        };
        let env = if radiation > home {
            home / radiation
        } else {
            Dy::ONE
        };
        let keep = (-(years / gen_yr.max(Dy::sci(1, -6)))).exp();
        self.home[people] = Some(home * keep + radiation * (Dy::ONE - keep));
        env
    }
}

/// One star's state between deliveries: its worlds' anchors, its drift (AU
/// per axis) and its books, each `None` until first reached.
type StarState = (Option<Vec<Anchor>>, Option<[Q; 3]>, Option<Books>);

/// Delivers every bell's pulses arriving in `(from, to]` Gyr to `star` from
/// its held state. Returns the star's new state and the pulses delivered.
fn ring_star(
    star: u64,
    (anchors, drift0, books0): StarState,
    systems: &BTreeMap<u64, crate::cosmic::Witnessed>,
    bells: &[(Node, Bell)],
    from: Dy,
    to: Dy,
) -> (u64, Option<Vec<Anchor>>, Option<[Q; 3]>, Option<Books>, u64) {
    let _busy = xyl_core::watch::busy(format!("rings of star {star}"));
    let Some(w) = systems.get(&star) else {
        return (star, anchors, drift0, books0, 0);
    };
    let mass = w.system.spec.mass.clone();
    let mut worlds = anchors.unwrap_or_else(|| {
        w.system
            .worlds
            .iter()
            .map(|x| Anchor::on_orbit(&x.a, &x.mass))
            .collect()
    });
    let Some(a_out) = w.system.worlds.iter().map(|x| x.a.clone()).max() else {
        return (star, Some(worlds), drift0, books0, 0);
    };
    let step = burning(w.star.reach_rings(), to - Dy::from_q(&w.star.born));
    let mut drift = drift0.unwrap_or([Q::ZERO, Q::ZERO, Q::ZERO]);
    let mut books = books0;
    let mut delivered = 0;
    for (node, bell) in bells {
        if node.born >= to {
            continue;
        }
        // The star's place: its manifold position at `to`, stood off by the
        // drift's L1 length (the sum of the absolute per-axis offsets in AU,
        // converted to kpc), not by a Euclidean length.
        let kpc = au_per_kpc();
        let off = drift
            .iter()
            .fold(Q::ZERO, |a, d| &a + &d.abs())
            .checked_div(&kpc)
            .unwrap_or(Q::ZERO);
        let place = w
            .star
            .place(to, &radius())
            .standing_off(&(&off * &xyl_cosmos::chain::kpc()));
        let (delay, d) = path(node, &place, to);
        let k = bell.arrivals(delay, from, to);
        if k == 0 {
            continue;
        }
        let v = bell.received(d);
        let e = bell.energy_at(d, &a_out);
        // Each ring is absorbed where the worlds stand. Once a ring
        // leaves the anchors unchanged (the star re-locked them all),
        // every later ring is identical, so the rest are counted rather
        // than repeated; the result is independent of how the span is cut.
        let b = books.get_or_insert_with(Books::default);
        let mut i = 0;
        while i < k {
            let Some(a) = absorb(&mass, step, &worlds, &v, &e) else {
                break;
            };
            let settled = a.anchors == worlds;
            let n = if settled { k - i } else { 1 };
            let times = Q::int(i64::try_from(n).unwrap_or(i64::MAX));
            for (x, dx) in drift.iter_mut().zip(&a.drift) {
                *x = &*x + &(dx * &times);
            }
            b.add(&a, n, to);
            worlds = a.anchors;
            i += n;
        }
        delivered += k;
    }
    (star, Some(worlds), Some(drift), books, delivered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_tunes_to_home_and_falls_above_it() {
        let mut f = Field::default();
        // With no radiation there is nothing to tune to; the factor is 1.
        assert_eq!(f.environment(0, Dy::ZERO, Dy::ONE, Dy::int(25)), Dy::ONE);
        // The first nonzero radiation becomes the home radiation.
        assert_eq!(f.environment(0, Dy::int(4), Dy::ONE, Dy::int(25)), Dy::ONE);
        // Radiation below home gives 1; twice home gives one half.
        assert_eq!(
            f.environment(0, Dy::int(2), Dy::sci(1, -9), Dy::int(25)),
            Dy::ONE
        );
        let e = f.environment(0, Dy::int(8), Dy::sci(1, -9), Dy::int(25));
        assert!((e - Dy::ratio(1, 2)).abs() < Dy::sci(1, -6), "{e}");
        // Over many generations the home radiation moves to the new radiation.
        f.environment(0, Dy::int(8), Dy::int(10_000), Dy::int(25));
        let e = f.environment(0, Dy::int(8), Dy::sci(1, -9), Dy::int(25));
        assert!(e > Dy::ratio(99, 100), "adapted: {e}");
    }
}
