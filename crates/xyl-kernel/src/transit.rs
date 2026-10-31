//! Transit: distances between places, voyage times, the ley network with
//! its constellations, realms and quarantine, folding, and toll quotes
//! (docs/civilizations.md).
//!
//! Inputs: the [`Cosmic`] world (witnessed systems, star places), drives
//! from propulsion.rs, stars from `xyl_cosmos::galaxy`, orbits from
//! `xyl_cosmos::manifold`, the turning law (`xyl_cosmos::turning`) and the
//! SI seam (`package::Si`). Distances are in m or kpc, speeds in m/s,
//! energies in J, times in years or Gyr as each item states.
//!
//! - Distance ([`separation_m`]): within a system, the gap between where
//!   the two orbits stand now ([`orbit_gap_m`]); between systems, the arc
//!   between the stars ([`Cosmic::distance`]).
//! - Voyage ([`voyage`]): out of the well at [`escape_speed`], coasting at
//!   half the speed change left ([`cruise_speed`]) and braking with the
//!   other half; fuel is payload x (R - 1) at the fuel's energy per kg.
//!   [`window_years`] is the wait for a transfer window within a system.
//! - Constellations ([`constellations`]): stars bound by their mutual
//!   turning tighter than the galaxy's shear across their span; the unit of
//!   passage. Pairs are re-measured only when they could have crossed.
//! - [`Network`]: ley lines ([`Ley`]) built by ancient houses and their
//!   progenitors, each sited where a passage is worth most and opened at
//!   its due time (an event of the universe); a crossing takes a mouth's
//!   light time ([`crossing_years`]). Realms are connected sets of
//!   constellations joined by open lines. A people's [`Discord`] (reaching
//!   off its constellation while forgetting, plus its breach from folding
//!   space) closes its constellation by the quarantine share.
//! - Tolls ([`quote`]): floor = unrecovered cost over last year's traffic;
//!   ceiling = what the lane saves the passer; toll = half way between,
//!   never above the ceiling; an open line asks only its floor. [`tithe`]
//!   is a passage's energy per lane length spanned. Tolls are quoted, not
//!   collected: no traffic is carried yet.
//!
//! Lines built, stewards' handling and peoples' folds are held in the
//! checkpoint; constellations and quarantine are re-read from the laws.
//!
//! Terms: constellation, ley line, realm, quarantine, discord, fold,
//! handling, tithe, worth law (docs/glossary.md).

use crate::cosmic::Cosmic;
use crate::propulsion::{delta_v, propellant_per_kg, Spec};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};
use xyl_core::dyadic::Dy;
use xyl_core::units::Tier;
use xyl_core::Q;
use xyl_cosmos::chain::units;
use xyl_cosmos::galaxy::Star;
use xyl_cosmos::manifold::{gap, manifolds, Manifold};
use xyl_cosmos::package::Si;
use xyl_cosmos::systems::World;

fn n(x: i64) -> Dy {
    Dy::int(x)
}

/// An astronomical unit (m).
#[cfg(test)]
fn au_m() -> Dy {
    Dy::from_q(&Si::au())
}

/// Kiloparsecs per Gyr in one km/s: a thousand metres a second over a Gyr
/// in seconds, per kiloparsec (the one SI seam, crate::package::Si).
pub fn kms_to_kpc_gyr() -> Dy {
    Dy::from_q(
        &(&Q::int(1000) * &Si::gyr_s())
            .checked_div(&Si::kpc())
            .unwrap_or(Q::ZERO),
    )
}

/// A kiloparsec (m), the one SI seam (package::Si).
pub fn kpc_m() -> Dy {
    Dy::from_q(&Si::kpc())
}

/// A standard year (s).
pub fn year_s() -> Dy {
    Dy::from_q(&Si::gyr_s()) / Dy::sci(1, 9)
}

/// A speed in m/s as kpc per Gyr.
pub fn kpc_per_gyr(m_s: Dy) -> Dy {
    m_s * Dy::from_q(&Si::gyr_s()) / kpc_m()
}

/// A passage's energy (J), set by the builder's capability rather than a
/// constant: a throat carries a body of `body_kg` across at the pulse's
/// speed, and the hole supplies only what the builder's drive (exhaust `v_e`)
/// does not: body c (c - v_e). At v_e -> c a passage costs nothing; a people
/// that cannot leave its well pays the body's whole energy.
pub fn passage_j(body_kg: Dy, v_e: Dy) -> Dy {
    let c = crate::propulsion::c_m_s();
    body_kg * c * (c - v_e.clamp(Dy::ZERO, c))
}

/// A place: a world of a star's system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Place {
    /// The star, by its sample number.
    pub star: u64,
    /// The world, by its index in the star's system.
    pub world: usize,
}

/// A world's escape speed (m/s): the turning law at its surface (turning.rs),
/// its turning there times its radius, i.e. c delta with delta = M / (R + M)
/// in the white hole's units (a deficit share, not sqrt(2 G M / R)).
pub fn escape_speed(w: &World) -> Dy {
    let m = &w.mass * &Si::world_mass_kg();
    let r = &w.radius * &Si::world_radius_m();
    Dy::from_q(&(&xyl_cosmos::turning::rad_s(&xyl_cosmos::turning::turning_si(&m, &r)) * &r))
}

/// The gap between two worlds of one system at its star's `age` (Gyr), in
/// metres, read on the ocean (manifold.rs `gap`): the turn between where
/// their orbits stand now - each by its phase, plane, shape and twist; a
/// moon about its giant - swept at the inner one's reach, then the climb
/// between them. No squared length or root is taken.
pub fn orbit_gap_m(all: &[Manifold], wa: usize, wb: usize, age: &Q) -> Option<Dy> {
    let g = gap(all, wa, wb, age)?;
    Some(Dy::from_q(
        &(&(&g * &units().r_g_m) * &Tier::System.in_rg()),
    ))
}

/// The distance between two places now (m): within a system by where their
/// orbits stand now, between systems by the arc between their stars (both
/// witnessed).
pub fn separation_m(cosmic: &Cosmic, a: Place, b: Place) -> Option<Dy> {
    if a.star == b.star {
        let wit = cosmic.systems.get(&a.star)?;
        let all = manifolds(cosmic.gal.seed, &[a.star], &wit.system, &wit.star, &units());
        let age = (cosmic.now - Dy::from_q(&wit.star.born))
            .max(Dy::ZERO)
            .to_q();
        return orbit_gap_m(&all, a.world, b.world, &age);
    }
    Some(cosmic.distance(a.star, b.star)? * kpc_m())
}

/// A voyage, as one drive makes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Voyage {
    /// Travel time at the coasting speed, in years.
    pub years: Dy,
    /// Its coasting speed (m/s).
    pub cruise: Dy,
    /// The fuel's energy it burns per kg carried (J/kg).
    pub energy_per_kg: Dy,
}

/// The speed a drive coasts at out of a well of this escape speed (m/s): half
/// of what its mass ratio buys beyond the climb out (the other half brakes).
pub fn cruise_speed(spec: &Spec, ratio: Dy, escape: Dy) -> Dy {
    ((delta_v(spec.v_e, ratio) - escape) / n(2)).max(Dy::ZERO)
}

/// A voyage of `dist_m` out of a well of `escape` (m/s); None if the drive
/// cannot climb out of it.
pub fn voyage(spec: &Spec, ratio: Dy, escape: Dy, dist_m: Dy) -> Option<Voyage> {
    let cruise = cruise_speed(spec, ratio, escape);
    if cruise.signum() <= 0 {
        return None;
    }
    let years = dist_m.max(Dy::ZERO) / cruise / year_s();
    Some(Voyage {
        years,
        cruise,
        energy_per_kg: propellant_per_kg(ratio) * spec.energy_per_kg,
    })
}

/// The farthest a drive carries within `years` out of this well (kpc).
pub fn reach_kpc(spec: &Spec, ratio: Dy, escape: Dy, years: Dy) -> Dy {
    cruise_speed(spec, ratio, escape) * years * year_s() / kpc_m()
}

// ---------------------------------------------------------------------- the toll law

/// How a passage's holder handles it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Handling {
    /// Only what it cost: the toll asks its floor.
    Open,
    /// By the toll law: half way from floor to ceiling. The default for a
    /// line a steward has not set.
    #[default]
    Tolled,
    /// Its builder's house and the peoples of its realm only.
    Guarded,
    /// Carries no one.
    Closed,
}

/// A held passage's books (J).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lane {
    /// What holding it open cost.
    pub cost_j: Dy,
    /// What it has collected.
    pub collected_j: Dy,
    /// The passages it carried last year.
    pub last_year: u64,
}

/// What passing costs, and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quote {
    /// Cost not yet paid back over last year's traffic, in J per passage ([`floor`]).
    pub floor: Dy,
    /// What the lane saves the passer, in J ([`ceiling`]).
    pub ceiling: Dy,
    /// The toll asked, in J, never above the ceiling.
    pub toll: Dy,
}

/// The floor: its cost not yet paid back, over last year's traffic.
pub fn floor(lane: &Lane) -> Dy {
    (lane.cost_j - lane.collected_j).max(Dy::ZERO)
        / n(i64::try_from(lane.last_year.max(1)).unwrap_or(i64::MAX))
}

/// The ceiling: what the lane saves the passer - the cheapest way round less
/// the lane, in hops at the passer's hop value; with no way round, all it
/// carries and a hop.
pub fn ceiling(around_hops: Option<Dy>, lane_hops: Dy, hop_value: Dy, cargo_value: Dy) -> Dy {
    match around_hops {
        None => cargo_value + hop_value,
        Some(a) => (a - lane_hops).max(Dy::ZERO) * hop_value,
    }
}

/// The toll: half way from floor to ceiling, never above the ceiling; an open
/// passage asks only its floor (and never more than it saves).
pub fn quote(
    lane: &Lane,
    around_hops: Option<Dy>,
    lane_hops: Dy,
    hop_value: Dy,
    cargo_value: Dy,
    handling: Handling,
) -> Quote {
    let f = floor(lane);
    let c = ceiling(around_hops, lane_hops, hop_value, cargo_value);
    let toll = match handling {
        Handling::Open => f.min(c),
        _ => c.min(f + (c - f).max(Dy::ZERO) / n(2)),
    };
    Quote {
        floor: f,
        ceiling: c,
        toll,
    }
}

/// A throat's tithe: a passage's energy for every
/// lane's length it spans (time as distance), at least one.
pub fn tithe(passage: Dy, span: Dy, lane_len: Dy) -> Dy {
    let hops = if lane_len.signum() > 0 {
        span / lane_len
    } else {
        Dy::ONE
    };
    passage * hops.max(Dy::ONE)
}

/// What a hop costs a passer: the fuel its drive
/// burns carrying `payload_kg` (J).
pub fn hop_value(spec: &Spec, ratio: Dy, payload_kg: Dy) -> Dy {
    payload_kg * propellant_per_kg(ratio) * spec.energy_per_kg
}

// ------------------------------------- constellations, windows and ley lines

/// The turn a body of `suns` gives a neighbour `span_kpc` away (degrees per
/// Gyr): the master equation's pair term, the turning law at their span.
fn pair_turning(suns: Dy, span_kpc: Dy) -> Dy {
    xyl_cosmos::turning::turning_kpc_dy(suns, span_kpc.max(Dy::sci(1, -9)))
}

/// The same, with the inner star's own turning `rate0` already read (by the
/// same law as the turning across the span, so the shear is the law's own
/// difference).
pub fn binding_at(a: &Star, ma: Dy, b: &Star, mb: Dy, rate0: Dy, span_kpc: Dy) -> Dy {
    if a.core != b.core {
        return Dy::ZERO;
    }
    let turn = pair_turning(ma + mb, span_kpc);
    let inner = if a.orbit.radius <= b.orbit.radius {
        a
    } else {
        b
    };
    let shear = (rate0 - inner.rate_at_dy(inner.orbit.radius + span_kpc.max(Dy::ZERO))).abs();
    let both = turn + shear;
    if both.signum() <= 0 {
        Dy::ZERO
    } else {
        turn / both
    }
}

/// A star's binding reach (kpc): the span at which it binds a neighbour like
/// itself by a half, found by bisection over [0, r].
pub fn binding_reach(s: &Star, m: Dy) -> Dy {
    let half = Dy::ratio(1, 2);
    let rate0 = s.rate_at_dy(s.orbit.radius);
    let (mut lo, mut hi) = (Dy::ZERO, s.orbit.radius.max(Dy::sci(1, -3)));
    if binding_at(s, m, s, m, rate0, hi) >= half {
        return hi;
    }
    for _ in 0..40 {
        let mid = (lo + hi) / n(2);
        if binding_at(s, m, s, m, rate0, mid) >= half {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// A constellation: the stars whose mutual turnings bind them tighter than
/// the galaxy's shear at their place; the unit of passage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constellation {
    /// Its stars (sample indices), least first: the least is its key.
    pub stars: Vec<u64>,
    /// Galactic core index of its least star.
    pub core: u32,
    /// Its reach (kpc): the widest binding reach of its stars, and the width of
    /// a ley line's mouth there.
    pub reach_kpc: Dy,
}

impl Constellation {
    /// Its key: its least star's sample number (0 for an empty
    /// constellation, which the network never builds).
    pub fn key(&self) -> u64 {
        self.stars.first().copied().unwrap_or(0)
    }
}

fn find(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

/// A star in play: the star as the Big Bang made it and the suns it carries
/// at the sample's resolution (its own share times the sample over the stars
/// worked out in its galaxy).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InPlay {
    /// The star as the Big Bang made it.
    pub star: Star,
    /// How many suns it carries at the sample's resolution (count of stars).
    pub suns: Dy,
    /// Its binding reach (kpc) and its own turning (degrees per Gyr), read
    /// once: neither moves with time.
    pub reach: Dy,
    /// Its own turning about its galaxy at its orbit's radius, in degrees per Gyr.
    pub rate0: Dy,
}

impl InPlay {
    fn of(star: Star, suns: Dy) -> InPlay {
        let reach = binding_reach(&star, suns);
        let rate0 = star.rate_at_dy(star.orbit.radius);
        InPlay {
            star,
            suns,
            reach,
            rate0,
        }
    }

    /// The most it moves through its galaxy a Gyr (kpc): its circulation,
    /// its turning times its reach, and its swing through the plane at its
    /// fullest (its spread read 10 Gyr on, the share's saturation); a bound
    /// on how fast any arc to it can change.
    pub fn speed_bound(&self, t: Dy) -> Dy {
        let circ = self.star.orbit.rate.abs() * Dy::PI / n(180) * self.star.orbit.radius;
        let age = (t - Dy::from_q(&self.star.born)).max(Dy::ZERO) + n(10);
        let swing = self
            .star
            .vertical_at(age)
            .map(|v| v.vz_kms.abs() * self.star.scale * kms_to_kpc_gyr())
            .unwrap_or(Dy::ZERO);
        circ + swing
    }
}

/// The stars in play: each galaxy's first `limit` sample stars that its sight
/// opens (as `minds::schedule` reads them), regenerated from the seed (orbit
/// and place only, no system), each carrying
/// m = stands_for x sample / worked in its galaxy.
pub fn in_play(seed: u64, bang: &xyl_cosmos::bang::BigBang, limit: u64) -> Vec<InPlay> {
    use xyl_cosmos::minds::Sight;
    // the network's first stars in play are the ancients' sight, their cores;
    // every people's system is admitted as it is witnessed (`admit`), so the
    // network grows with the game rather than seating the whole sample
    let sight = Sight::Cores;
    let sample = xyl_cosmos::galaxy::sample(bang);
    let total = limit.min(sample);
    let mut out = Vec::new();
    for c in xyl_cosmos::cores::held(seed, bang) {
        let stars: Vec<Star> = (0..total)
            .filter_map(|l| {
                xyl_cosmos::galaxy::star(seed, bang, xyl_cosmos::galaxy::in_galaxy(c, l))
            })
            .filter(|st| sight.opens(st))
            .collect();
        let w = Dy::int(i64::try_from(sample).unwrap_or(i64::MAX))
            / Dy::int(i64::try_from(stars.len().max(1)).unwrap_or(1));
        out.extend(stars.into_iter().map(|st| {
            let suns = st.stands_for * w;
            InPlay::of(st, suns)
        }));
    }
    out
}

/// The pairs that can bind, read once: stars sorted by (core, reach), each
/// paired outward while their binding at the reaches' difference (a lower
/// bound on their arc, fixed for circling stars) still holds a half; each
/// pair's span s_ij where its binding falls to a half, found by bisection.
/// Returns (i, j, s_ij), indices into `stars`.
pub fn binding_pairs(stars: &[InPlay]) -> Vec<(usize, usize, Dy)> {
    let half = Dy::ratio(1, 2);
    let mut order: Vec<usize> = (0..stars.len()).collect();
    order.sort_by_key(|&a| {
        (
            stars[a].star.core,
            stars[a].star.orbit.radius,
            stars[a].star.n,
        )
    });
    let mut out = Vec::new();
    for (oi, &i) in order.iter().enumerate() {
        for &j in &order[oi + 1..] {
            let (a, b) = (&stars[i], &stars[j]);
            let dr = b.star.orbit.radius - a.star.orbit.radius;
            let rate0 = if a.star.orbit.radius <= b.star.orbit.radius {
                a.rate0
            } else {
                b.rate0
            };
            let bind = |s: Dy| binding_at(&a.star, a.suns, &b.star, b.suns, rate0, s);
            if a.star.core != b.star.core || bind(dr) < half {
                break;
            }
            let mut lo = dr.max(Dy::ZERO);
            let mut hi = (lo * n(2)).max(Dy::sci(1, -6));
            while bind(hi) >= half && hi < Dy::sci(1, 3) {
                lo = hi;
                hi = hi * n(2);
            }
            for _ in 0..24 {
                let mid = (lo + hi) / n(2);
                if bind(mid) >= half {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            out.push((i, j, lo));
        }
    }
    out
}

/// The constellations from the pairs bound now: each bound pair joined.
pub fn constellations(
    stars: &[InPlay],
    pairs: &[(usize, usize, Dy)],
    bound: &[bool],
) -> Vec<Constellation> {
    let mut parent: Vec<usize> = (0..stars.len()).collect();
    for (&(i, j, _), _) in pairs.iter().zip(bound).filter(|(_, b)| **b) {
        let (x, y) = (find(&mut parent, i), find(&mut parent, j));
        parent[x.max(y)] = x.min(y);
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..stars.len() {
        let r = find(&mut parent, i);
        groups.entry(r).or_default().push(i);
    }
    let mut out: Vec<Constellation> = groups
        .into_values()
        .map(|g| {
            let mut ks: Vec<u64> = g.iter().map(|&i| stars[i].star.n).collect();
            ks.sort_unstable();
            let reach = g.iter().map(|&i| stars[i].reach).max().unwrap_or(Dy::ZERO);
            Constellation {
                stars: ks,
                core: stars[g[0]].star.core,
                reach_kpc: reach,
            }
        })
        .collect();
    out.sort_by_key(|c| c.key());
    out
}

/// The transfer window between two worlds of one system: a transfer is half a
/// circulation, so it leaves when the target's phase leads the
/// origin's by half a turn less what the target turns on the way; the
/// phases come round on each other at their synodic rate. Years to wait
/// from `age` (Gyr) for a leg of `leg_years`, and the origin's orbital speed
/// (m/s, its turning times its reach); None where the two never part (one
/// rate) or are not of one anchor.
pub fn window_years(
    all: &[Manifold],
    wa: usize,
    wb: usize,
    age: &Q,
    leg_years: Dy,
) -> Option<(Dy, Dy)> {
    let (a, b) = (
        all.iter().find(|m| m.world == wa)?,
        all.iter().find(|m| m.world == wb)?,
    );
    if a.anchor != b.anchor {
        return None;
    }
    let (ra, rb) = (Dy::from_q(&a.rate), Dy::from_q(&b.rate));
    let rel = (rb - ra) / n(360);
    if rel.signum() == 0 {
        return None;
    }
    let age = Dy::from_q(age);
    let phase = |m: &Manifold, r: Dy| Dy::from_q(&m.m0) + r * age / n(360);
    let lead = Dy::ratio(1, 2) - rb * leg_years / Dy::sci(1, 9) / n(360);
    let off = (lead - (phase(b, rb) - phase(a, ra))) * Dy::int(rel.signum().into());
    let turns = off - Dy::int(off.trunc_i64()) + if off.signum() < 0 { Dy::ONE } else { Dy::ZERO };
    let wait = turns.max(Dy::ZERO) / rel.abs() * Dy::sci(1, 9);
    let a_m = &(&a.a * &units().r_g_m) * &a.tier.in_rg();
    let v_orb = Dy::from_q(&(&xyl_cosmos::turning::rad_s(&a.rate) * &a_m));
    Some((wait, v_orb))
}

/// A ley line: a circulation held open between two constellations, built by
/// an ancient house, with its mouths at two stars.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ley {
    /// The star of its first mouth, by sample number.
    pub from: u64,
    /// The star of its second mouth, by sample number.
    pub to: u64,
    /// Its builder (a people's index: an ancient house, or its progenitors
    /// when `progenitor`) and the builder's tag.
    pub builder: usize,
    /// The builder's name tag (see [`crate::checkpoint::tag`]).
    pub tag: String,
    /// Whether the house's progenitors built it rather than the house itself.
    pub progenitor: bool,
    /// When it opened (Gyr after the Big Bang) and what holding it open cost
    /// (J): a passage's energy for every mouth's width it spans.
    pub built: Dy,
    /// What holding it open cost, in J.
    pub cost_j: Dy,
    /// How it is handled for passers, as its realm's steward set it.
    pub handling: Handling,
}

/// A people's discord in its parts (universe.rs `discord_parts`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Discord {
    /// Its ships' reach in a generation and its constellation's (kpc).
    pub reach_kpc: Dy,
    /// Its constellation's reach (the width of a ley mouth there), in kpc.
    pub mouth_kpc: Dy,
    /// Its forgetting share, 1 - z.
    pub forget: Dy,
    /// Its constellation holds a ley mouth.
    pub joined: bool,
    /// Its drive's exhaust, cruise and its home's escape (m/s).
    pub v_e: Dy,
    /// Its ships' coasting speed out of its home well, in m/s.
    pub cruise: Dy,
    /// Its home world's escape speed, in m/s.
    pub escape: Dy,
    /// Its breach share from folding space, in \[0, 1\]: the energy it has
    /// folded over that plus its constellation's lawful lines' and one
    /// passage of its own (see [`Network::breach`]).
    pub breach: Dy,
    /// Its discord, a share in \[0, 1\]: `1 - (1 - o f (1 - J))(1 - breach)`,
    /// o its off-constellation share, f its forgetting share, J whether its
    /// constellation holds a ley mouth.
    pub discord: Dy,
}

/// The most lines one house opens in one reading (the next reading carries
/// on from where it stopped). Declared ruler: a bound on one reading's work.
pub const LINES_PER_READING: usize = 1024;

/// The network: the stars in play and where they stand, the ley lines built,
/// the works under way, the constellations as last read and how far each is
/// held closed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Network {
    /// Every ley line opened, in opening order.
    pub lines: Vec<Ley>,
    /// Each builder's line under way (by its house and whether its
    /// progenitors build it), `built` its due.
    pub works: BTreeMap<(usize, bool), Ley>,
    /// The constellations as last read.
    pub constellations: Vec<Constellation>,
    /// Every realm's key (its least constellation key) in order of first
    /// appearance; a steward seat's index into this list is stable.
    pub realm_keys: Vec<u64>,
    /// Each people's folds of space: the passage energy folded (J) and when
    /// its folding was last read (Gyr).
    pub folds: BTreeMap<usize, (Dy, Dy)>,
    /// Each constellation's quarantine share (by its key), as last read.
    pub quarantine: BTreeMap<u64, Dy>,
    /// Deep time the network was carried to (Gyr).
    pub read_to: Dy,
    /// The stars in play (`in_play`), with every witnessed star added, and
    /// each one's index by its sample number.
    pub sample: Vec<InPlay>,
    /// Each star's index into `sample`, by its sample number.
    pub index: BTreeMap<u64, usize>,
    /// The pairs that can bind and their spans (read once).
    pub pairs: Vec<(usize, usize, Dy)>,
    /// Each pair bound as last measured, and when it must be measured again
    /// (Gyr): its arc's gap to its span over the two stars' speed bounds,
    /// never sooner than a ring, so a pair is re-read only when it could
    /// have crossed.
    pub bound: Vec<bool>,
    /// When each pair (as `pairs`) must be measured again, in Gyr.
    pub check: Vec<Dy>,
    /// Each star's constellation (index into `constellations`), as last
    /// read, when it was read and until when it holds (Gyr).
    pub of: BTreeMap<u64, usize>,
    /// When the constellations were last read, in Gyr.
    pub read_at: Dy,
    /// Until when that reading holds, in Gyr.
    pub holds_to: Dy,
}

/// A drive between two specs, by a share: what a people that has mastered a
/// share of its energy's domain flies (linear in the share).
pub fn blended(lo: &Spec, hi: &Spec, share: Dy) -> Spec {
    let s = share.clamp(Dy::ZERO, Dy::ONE);
    Spec {
        drive: if s > Dy::ratio(1, 2) {
            hi.drive
        } else {
            lo.drive
        },
        energy_per_kg: lo.energy_per_kg + s * (hi.energy_per_kg - lo.energy_per_kg),
        v_e: lo.v_e + s * (hi.v_e - lo.v_e),
    }
}

/// The swiftest drive a people's chemistry allows (an ancient house
/// understands every fuel).
pub fn best_spec(cosmic: &Cosmic, p: &crate::cosmic::People) -> Spec {
    crate::propulsion::Drive::ALL
        .iter()
        .map(|&d| Spec::of(d, &p.traits, &cosmic.gal.sc, cosmic.gal.elements))
        .max_by(|a, b| a.v_e.cmp(&b.v_e))
        .unwrap_or_else(|| {
            Spec::of(
                crate::propulsion::Drive::Chemical,
                &p.traits,
                &cosmic.gal.sc,
                cosmic.gal.elements,
            )
        })
}

/// A people's home world's escape speed (m/s).
pub fn home_escape(cosmic: &Cosmic, p: &crate::cosmic::People) -> Dy {
    cosmic
        .systems
        .get(&p.star)
        .and_then(|w| w.system.worlds.get(p.world))
        .map(escape_speed)
        .unwrap_or(Dy::ZERO)
}

/// The light time across a mouth (years): a passage's crossing. The throat
/// shortens any span to its mouth's width, so no light time is charged for
/// the span itself.
pub fn crossing_years(reach_kpc: Dy) -> Dy {
    reach_kpc * kpc_m() / crate::propulsion::c_m_s() / year_s()
}

impl Network {
    /// Seat the stars in play for a universe's forecast (`limit` per galaxy,
    /// its sight).
    pub fn seat(&mut self, seed: u64, bang: &xyl_cosmos::bang::BigBang, limit: u64) {
        self.sample = in_play(seed, bang, limit);
        self.reindex();
    }

    fn reindex(&mut self) {
        self.index = self
            .sample
            .iter()
            .enumerate()
            .map(|(i, s)| (s.star.n, i))
            .collect();
        self.pairs = binding_pairs(&self.sample);
        self.bound = vec![false; self.pairs.len()];
        self.check = vec![Dy::ZERO; self.pairs.len()];
        self.holds_to = Dy::ZERO;
        self.constellations.clear();
    }

    /// Every witnessed star in play, carrying its galaxy's resolution (the
    /// pairs read again only when one joins).
    fn admit(&mut self, cosmic: &Cosmic) {
        let mut grew = false;
        for (k, w) in &cosmic.systems {
            if self.index.contains_key(k) {
                continue;
            }
            let ratio = self
                .sample
                .iter()
                .find(|s| s.star.core == w.star.core)
                .map(|p| p.suns / p.star.stands_for.max(Dy::sci(1, -30)))
                .unwrap_or(Dy::ONE);
            self.sample
                .push(InPlay::of(w.star.clone(), w.star.stands_for * ratio));
            self.index.insert(*k, self.sample.len() - 1);
            grew = true;
        }
        if grew {
            self.reindex();
        }
    }

    /// The constellations current at `t`: only the pairs whose check has
    /// come are measured again (all of them when `t` is earlier than the last
    /// reading), each star placed once; the groups re-formed only when a
    /// pair's bond changed.
    fn current(&mut self, cosmic: &Cosmic, t: Dy) {
        self.admit(cosmic);
        let back = t < self.read_at;
        if !self.constellations.is_empty() && !back && t < self.holds_to {
            return;
        }
        let sphere = xyl_cosmos::chain::radius();
        let ring = Dy::from_q(&crate::universe::Bell::of().period_gyr);
        let mut placed: BTreeMap<usize, xyl_cosmos::chain::Place> = BTreeMap::new();
        let mut speeds: BTreeMap<usize, Dy> = BTreeMap::new();
        let mut changed = self.constellations.is_empty();
        for k in 0..self.pairs.len() {
            if !back && self.check[k] > t {
                continue;
            }
            let (i, j, span) = self.pairs[k];
            for x in [i, j] {
                if let std::collections::btree_map::Entry::Vacant(e) = placed.entry(x) {
                    e.insert(self.sample[x].star.place(t, &sphere));
                    speeds.insert(x, self.sample[x].speed_bound(t));
                }
            }
            let arc = placed[&i].arc_kpc_dy(&placed[&j], &sphere);
            let now = arc <= span;
            changed |= now != self.bound[k];
            self.bound[k] = now;
            let v = speeds[&i] + speeds[&j];
            let wait = if v.signum() > 0 {
                (arc - span).abs() / v
            } else {
                Dy::sci(1, 3)
            };
            self.check[k] = t + wait.max(ring);
        }
        if changed {
            self.constellations = constellations(&self.sample, &self.pairs, &self.bound);
            self.of = self
                .constellations
                .iter()
                .enumerate()
                .flat_map(|(i, c)| c.stars.iter().map(move |&s| (s, i)))
                .collect();
        }
        self.read_at = t;
        self.holds_to = self
            .check
            .iter()
            .copied()
            .min()
            .unwrap_or(t + Dy::sci(1, 3));
    }

    /// The arc between two stars in play at `t` (kpc), each placed then.
    pub fn arc_at(&self, a: u64, b: u64, t: Dy) -> Option<Dy> {
        let sphere = xyl_cosmos::chain::radius();
        let (sa, sb) = (
            &self.sample.get(*self.index.get(&a)?)?.star,
            &self.sample.get(*self.index.get(&b)?)?.star,
        );
        Some(sa.place(t, &sphere).arc_kpc(&sb.place(t, &sphere), &sphere))
    }

    /// The arc between two stars in play now (kpc).
    pub fn arc(&self, cosmic: &Cosmic, a: u64, b: u64) -> Option<Dy> {
        self.arc_at(a, b, cosmic.now)
    }

    /// The next ley line falling due (Gyr): an event of the universe.
    pub fn next_due(&self) -> Option<Dy> {
        self.works.values().map(|w| w.built).min()
    }

    /// A constellation's reach by one of its stars.
    pub fn reach_of(&self, star: u64) -> Option<Dy> {
        self.of
            .get(&star)
            .and_then(|&c| self.constellations.get(c))
            .map(|c| c.reach_kpc)
    }

    /// The constellations joined to `home` over the lines (the network is
    /// common: every signatory's lines carry every signatory).
    fn joined(&self, home: usize) -> Vec<bool> {
        let mut seen = vec![false; self.constellations.len()];
        let Some(s) = seen.get_mut(home) else {
            return seen;
        };
        *s = true;
        let mut grew = true;
        while grew {
            grew = false;
            for l in &self.lines {
                if let (Some(&a), Some(&b)) = (self.of.get(&l.from), self.of.get(&l.to)) {
                    if seen[a] != seen[b] {
                        seen[a] = true;
                        seen[b] = true;
                        grew = true;
                    }
                }
            }
        }
        seen
    }

    /// Whether line `l` carries passer `who` (a people's index; `None` an
    /// unnamed passer) by its handling.
    pub fn carries(&self, cosmic: &Cosmic, l: &Ley, who: Option<usize>) -> bool {
        match l.handling {
            Handling::Open | Handling::Tolled => true,
            Handling::Closed => false,
            Handling::Guarded => who.is_some_and(|w| {
                w == l.builder
                    || cosmic.peoples.get(w).is_some_and(|p| {
                        self.realm_of(p.star).is_some()
                            && self.realm_of(p.star) == self.realm_of(l.from)
                    })
            }),
        }
    }

    /// The realms: each a connected set of constellations joined by ley
    /// lines with a non-zero open share, keyed by its least constellation
    /// key, as (key, constellation keys).
    pub fn realms(&self) -> Vec<(u64, Vec<u64>)> {
        let mut parent: BTreeMap<u64, u64> = BTreeMap::new();
        fn root(parent: &mut BTreeMap<u64, u64>, x: u64) -> u64 {
            let mut r = x;
            while let Some(&p) = parent.get(&r) {
                if p == r {
                    break;
                }
                r = p;
            }
            parent.insert(x, r);
            r
        }
        for l in &self.lines {
            if self.open_share(l).signum() <= 0 {
                continue;
            }
            let key = |s: u64| {
                self.of
                    .get(&s)
                    .and_then(|&c| self.constellations.get(c))
                    .map(|c| c.key())
            };
            let (Some(a), Some(b)) = (key(l.from), key(l.to)) else {
                continue;
            };
            parent.entry(a).or_insert(a);
            parent.entry(b).or_insert(b);
            let (ra, rb) = (root(&mut parent, a), root(&mut parent, b));
            if ra != rb {
                let (lo, hi) = (ra.min(rb), ra.max(rb));
                parent.insert(hi, lo);
            }
        }
        let keys: Vec<u64> = parent.keys().copied().collect();
        let mut by: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
        for k in keys {
            let r = root(&mut parent, k);
            by.entry(r).or_default().push(k);
        }
        by.into_iter().collect()
    }

    /// The key of the realm star `star`'s constellation belongs to, if any.
    pub fn realm_of(&self, star: u64) -> Option<u64> {
        let c = self
            .of
            .get(&star)
            .and_then(|&c| self.constellations.get(c))?
            .key();
        self.realms()
            .into_iter()
            .find(|(_, cs)| cs.contains(&c))
            .map(|(k, _)| k)
    }

    /// Records every realm's key not yet seen, so steward seats keep their
    /// index. Returns the keys added.
    pub fn note_realms(&mut self) -> Vec<u64> {
        let mut added = Vec::new();
        for (k, _) in self.realms() {
            if !self.realm_keys.contains(&k) {
                self.realm_keys.push(k);
                added.push(k);
            }
        }
        added
    }

    /// Sets the handling of every line between stars `a` and `b` (either
    /// way) that lies in the realm keyed `realm`. Returns how many lines
    /// changed.
    pub fn handle(&mut self, realm: u64, a: u64, b: u64, handling: Handling) -> usize {
        let in_realm: Vec<bool> = self
            .lines
            .iter()
            .map(|l| {
                ((l.from, l.to) == (a, b) || (l.from, l.to) == (b, a))
                    && self.realm_of(l.from) == Some(realm)
            })
            .collect();
        let mut changed = 0;
        for (l, hit) in self.lines.iter_mut().zip(in_realm) {
            if hit && l.handling != handling {
                l.handling = handling;
                changed += 1;
            }
        }
        changed
    }

    /// Line `l`'s books as a lane: its cost, and what it has collected (no
    /// traffic is carried yet, so nothing), for the toll law [`quote`].
    pub fn lane(&self, l: &Ley) -> Lane {
        Lane {
            cost_j: l.cost_j,
            collected_j: Dy::ZERO,
            last_year: 0,
        }
    }

    /// People `i`'s breach share from folding space: the passage energy it
    /// has folded, over that plus the cost of the lawful lines with a mouth
    /// in its constellation plus one passage of its own (`own_j`). Zero for
    /// a people that has not folded.
    pub fn breach(&self, cosmic: &Cosmic, i: usize, own_j: Dy) -> Dy {
        let Some(&(folded, _)) = self.folds.get(&i) else {
            return Dy::ZERO;
        };
        let Some(p) = cosmic.peoples.get(i) else {
            return Dy::ZERO;
        };
        let mine = self.of.get(&p.star).copied();
        let lawful = self
            .lines
            .iter()
            .filter(|l| {
                mine.is_some()
                    && (self.of.get(&l.from).copied() == mine
                        || self.of.get(&l.to).copied() == mine)
            })
            .fold(Dy::ZERO, |a, l| a + l.cost_j);
        let all = folded + lawful + own_j.max(Dy::ZERO);
        if all.signum() > 0 {
            folded / all
        } else {
            Dy::ZERO
        }
    }

    /// Folding space (`Laws::ftl`): people `i`, with drive exhaust `v_e`,
    /// body mass `body_kg` and generation `gen_yr`, folds where its
    /// constellation holds no ley mouth and a passage out of it is worth
    /// building by the siting law at its own drive - the shortest span out,
    /// its constellation's reach S, gives c min(S / T, c) > c (c - v_e) per
    /// kg. It folds once a generation over the span since its folding was
    /// last read (to `at` Gyr), each fold costing [`passage_j`]. Returns the
    /// energy folded in this span (J).
    pub fn fold(
        &mut self,
        cosmic: &Cosmic,
        i: usize,
        v_e: Dy,
        body_kg: Dy,
        gen_yr: Dy,
        at: Dy,
    ) -> Dy {
        let Some(p) = cosmic.peoples.get(i) else {
            return Dy::ZERO;
        };
        let entry = self.folds.get(&i).copied().unwrap_or((Dy::ZERO, p.dawn));
        let since = entry.1.max(p.dawn);
        let reach = self.reach_of(p.star);
        let c = crate::propulsion::c_m_s();
        let worth = reach.is_some_and(|s| {
            let span_m = s * kpc_m();
            let speed = (span_m / (gen_yr * year_s()).max(Dy::ONE)).min(c);
            speed > c - v_e.clamp(Dy::ZERO, c)
        });
        if !cosmic.gal.sc.ftl || self.holds_mouth(p.star) || !worth || at <= since {
            if self.folds.contains_key(&i) {
                self.folds.insert(i, (entry.0, at.max(since)));
            }
            return Dy::ZERO;
        }
        let folds = (at - since) * Dy::sci(1, 9) / gen_yr.max(Dy::ONE);
        let energy = folds * passage_j(body_kg, v_e);
        self.folds.insert(i, (entry.0 + energy, at));
        energy
    }

    /// The share star `star`'s constellation is held closed by quarantine, in
    /// [0, 1] (0 where it belongs to no constellation).
    pub fn shut_of(&self, star: u64) -> Dy {
        self.of
            .get(&star)
            .and_then(|&c| self.constellations.get(c))
            .and_then(|c| self.quarantine.get(&c.key()))
            .copied()
            .unwrap_or(Dy::ZERO)
    }

    /// Whether a star's constellation holds a ley mouth.
    pub fn holds_mouth(&self, star: u64) -> bool {
        let Some(c) = self.of.get(&star) else {
            return false;
        };
        self.lines
            .iter()
            .any(|l| self.of.get(&l.from) == Some(c) || self.of.get(&l.to) == Some(c))
    }

    /// A line's open share: each mouth's constellation open by what its
    /// quarantine leaves.
    pub fn open_share(&self, l: &Ley) -> Dy {
        let shut = |s: u64| {
            self.of
                .get(&s)
                .and_then(|&i| self.constellations.get(i))
                .and_then(|c| self.quarantine.get(&c.key()))
                .copied()
                .unwrap_or(Dy::ZERO)
        };
        (Dy::ONE - shut(l.from)) * (Dy::ONE - shut(l.to))
    }

    /// Carry the network to `to` (Gyr), event by event. Every line falling due
    /// by then opens at its own moment, in order, and its builder begins the
    /// next there: the far mouth is the line worth the most to a constellation
    /// the network has not joined (constellations current at that moment);
    /// its due time is the later of the voyage (the swiftest drive out of the
    /// home well, along the arc) and the time the world's spare power takes to
    /// pay the line's cost. A house with no line under way begins one at once.
    /// Then each constellation's quarantine is read at `to` from its peoples'
    /// discord (by index, a share each). Returns (epoch, kind, text) for the
    /// annals.
    pub fn step(
        &mut self,
        cosmic: &Cosmic,
        discord: &BTreeMap<usize, Dy>,
        to: Dy,
    ) -> Vec<(Dy, &'static str, String)> {
        let mut out = Vec::new();
        let from = self.read_to;
        self.read_to = to.max(from);
        let ratio = n(crate::propulsion::MASS_RATIO);
        // the houses that can build, and what each has
        let houses: Vec<(usize, Dy, Dy, Dy)> = cosmic
            .peoples
            .iter()
            .enumerate()
            .filter(|(_, p)| p.ancient && p.population.signum() > 0)
            .filter_map(|(i, p)| {
                let spec = best_spec(cosmic, p);
                let cruise = cruise_speed(&spec, ratio, home_escape(cosmic, p));
                let spare = cosmic
                    .power(p)
                    .map(|w| (w.starlight_w + w.heat_w - w.bodies_w).max(Dy::ZERO))
                    .unwrap_or(Dy::ZERO);
                (cruise.signum() > 0 && spare.signum() > 0).then_some((i, cruise, spare, spec.v_e))
            })
            .collect();
        // progenitors' targets: the stars of living peoples in step with the
        // network (discord under a half); out-of-tune peoples are not targeted
        let in_tune: Vec<u64> = cosmic
            .peoples
            .iter()
            .enumerate()
            .filter(|(i, p)| {
                !p.ancient
                    && p.population.signum() > 0
                    && discord.get(i).is_none_or(|d| *d < Dy::ratio(1, 2))
            })
            .map(|(_, p)| p.star)
            .collect();
        // one clock for the builders: every builder idle by `to` begins at its
        // own moment and every line falls due at its own, all in time order,
        // so the constellations are only ever read forward
        let mut idle: Vec<(Dy, (usize, bool))> = Vec::new();
        for &(i, _, _, _) in &houses {
            for prog in [false, true] {
                if !self.works.contains_key(&(i, prog)) {
                    let p = &cosmic.peoples[i];
                    let start = self
                        .lines
                        .iter()
                        .filter(|l| l.builder == i && l.progenitor == prog)
                        .map(|l| l.built)
                        .max()
                        .unwrap_or(p.dawn)
                        .max(p.dawn)
                        .max(from.min(to));
                    if start <= to {
                        idle.push((start, (i, prog)));
                    }
                }
            }
        }
        for _ in 0..LINES_PER_READING {
            let due = self
                .works
                .iter()
                .map(|(k, w)| (w.built, *k))
                .filter(|(d, _)| *d <= to)
                .min();
            let first_idle = idle.iter().copied().min();
            let (at, key, opens) = match (due, first_idle) {
                (Some(d), Some(b)) if b.0 < d.0 => (b.0, b.1, false),
                (Some(d), _) => (d.0, d.1, true),
                (None, Some(b)) => (b.0, b.1, false),
                (None, None) => break,
            };
            if opens {
                let Some(done) = self.works.remove(&key) else {
                    break;
                };
                let name = cosmic
                    .peoples
                    .get(key.0)
                    .map(|p| p.name.clone())
                    .unwrap_or_default();
                let who = if key.1 {
                    format!("progenitors of the {name}")
                } else {
                    name
                };
                out.push((
                    done.built,
                    "LEY",
                    format!(
                        "The {who} open a ley line from star {} to star {} ({} J held open)",
                        done.from, done.to, done.cost_j
                    ),
                ));
                self.lines.push(done);
            } else {
                idle.retain(|x| x.1 != key);
            }
            if let Some(&(_, cruise, spare, v_e)) = houses.iter().find(|h| h.0 == key.0) {
                self.begin(
                    cosmic,
                    key,
                    (cruise, v_e),
                    spare,
                    at,
                    key.1.then_some(in_tune.as_slice()),
                );
            }
        }
        self.current(cosmic, to);
        // quarantine: a constellation is held closed by the share
        // 1 - product(1 - discord) over its peoples, re-read every step and
        // never latched
        let mut shut: BTreeMap<u64, Dy> = BTreeMap::new();
        for (&i, &d) in discord {
            let Some(p) = cosmic.peoples.get(i) else {
                continue;
            };
            let Some(c) = self
                .of
                .get(&p.star)
                .and_then(|&c| self.constellations.get(c))
            else {
                continue;
            };
            let e = shut.entry(c.key()).or_insert(Dy::ZERO);
            *e = Dy::ONE - (Dy::ONE - *e) * (Dy::ONE - d.clamp(Dy::ZERO, Dy::ONE));
        }
        let half = Dy::ratio(1, 2);
        for (&k, &q) in &shut {
            let was = self.quarantine.get(&k).copied().unwrap_or(Dy::ZERO);
            if was < half && q >= half {
                out.push((to, "QUARANTINE", format!("The ley lines about the constellation of star {k} close: its peoples ring out of tune ({q} held closed)")));
            }
        }
        for (&k, &was) in &self.quarantine {
            let q = shut.get(&k).copied().unwrap_or(Dy::ZERO);
            if was >= half && q < half {
                out.push((to, "REOPENED", format!("The ley lines about the constellation of star {k} reopen: its peoples ring in step ({q} held closed)")));
            }
        }
        self.quarantine = shut;
        out
    }

    /// A builder begins its next line at `start`: among the constellations it
    /// has discovered (within its craft's reach since its dawn) and the network
    /// has not joined, the line worth the most by the siting law. Each
    /// candidate's worth is bounded from its arc's lower bound and the greatest
    /// bounds are read first; arcs are read with both stars placed at `start`.
    /// An ancient house explores any unjoined constellation; its progenitors
    /// (`want`) build only to the constellations of the given stars.
    fn begin(
        &mut self,
        cosmic: &Cosmic,
        key: (usize, bool),
        drive: (Dy, Dy),
        spare_w: Dy,
        start: Dy,
        want: Option<&[u64]>,
    ) {
        let (cruise, v_e) = drive;
        let i = key.0;
        self.current(cosmic, start);
        let wanted: Option<std::collections::BTreeSet<usize>> =
            want.map(|w| w.iter().filter_map(|s| self.of.get(s).copied()).collect());
        let Some(p) = cosmic.peoples.get(i) else {
            return;
        };
        let Some(&home) = self.of.get(&p.star) else {
            return;
        };
        let joined = self.joined(home);
        let by_n: BTreeMap<u64, (u32, Dy)> = self
            .sample
            .iter()
            .map(|s| (s.star.n, (s.star.core, s.star.orbit.radius)))
            .collect();
        let mut mine: BTreeMap<u32, Vec<(Dy, u64)>> = BTreeMap::new();
        for (&k, &c) in &self.of {
            if joined[c] {
                if let Some(&(core, r)) = by_n.get(&k) {
                    mine.entry(core).or_default().push((r, k));
                }
            }
        }
        for v in mine.values_mut() {
            v.sort();
        }
        let sphere = xyl_cosmos::chain::radius();
        let placed: std::cell::RefCell<BTreeMap<u64, xyl_cosmos::chain::Place>> =
            Default::default();
        let place = |n: u64| -> Option<xyl_cosmos::chain::Place> {
            if let Some(p) = placed.borrow().get(&n) {
                return Some(p.clone());
            }
            let p = self
                .sample
                .get(*self.index.get(&n)?)?
                .star
                .place(start, &sphere);
            placed.borrow_mut().insert(n, p.clone());
            Some(p)
        };
        let arc =
            |a: u64, b: u64| -> Option<Dy> { Some(place(a)?.arc_kpc_dy(&place(b)?, &sphere)) };
        // discovery: a house knows the sky its craft could have crossed since
        // its dawn, kpc_per_gyr(cruise) (start - dawn); a constellation beyond
        // that is undiscovered and not a candidate
        let seen = kpc_per_gyr(cruise) * (start - p.dawn).max(Dy::ZERO);
        // each galaxy's core at `start`, read from any of its stars; the
        // arc between two cores bounds below every arc between their stars
        let mut cores_at: BTreeMap<u32, xyl_cosmos::chain::Place> = BTreeMap::new();
        for s in &self.sample {
            if let std::collections::btree_map::Entry::Vacant(e) = cores_at.entry(s.star.core) {
                if let Some(cp) = place(s.star.n).and_then(|x| x.core.map(|b| *b)) {
                    e.insert(cp);
                }
            }
        }
        let mut between: BTreeMap<u32, Dy> = BTreeMap::new();
        let mut core_lb = |core: u32| -> Dy {
            *between.entry(core).or_insert_with(|| {
                let Some(there) = cores_at.get(&core) else {
                    return Dy::sci(1, 12);
                };
                mine.keys()
                    .filter_map(|g| cores_at.get(g))
                    .map(|here| here.arc_kpc(there, &sphere))
                    .min()
                    .unwrap_or(Dy::sci(1, 12))
            })
        };
        // one candidate per discovered, unjoined constellation: its star
        // nearest the joined stars by a lower bound on the arc - within a
        // galaxy the reaches' difference, across galaxies the cores' arc
        let mut per: BTreeMap<usize, (Dy, u64)> = BTreeMap::new();
        for (&k, &c) in &self.of {
            if joined[c] || wanted.as_ref().is_some_and(|w| !w.contains(&c)) {
                continue;
            }
            let Some(&(core, r)) = by_n.get(&k) else {
                continue;
            };
            let lb = match mine.get(&core) {
                Some(v) => {
                    let at = v.partition_point(|x| x.0 < r);
                    [at.checked_sub(1), Some(at)]
                        .into_iter()
                        .flatten()
                        .filter_map(|j| v.get(j))
                        .map(|x| (x.0 - r).abs())
                        .min()
                        .unwrap_or(Dy::ZERO)
                }
                None => core_lb(core),
            };
            if lb > seen {
                continue;
            }
            let e = per.entry(c).or_insert((lb, k));
            if lb < e.0 {
                *e = (lb, k);
            }
        }
        // siting law: a hole is worth its passer's mass times what it saves
        // per kg against normal transit (the two wells' remainders at their
        // mouths, c^2 (delta_A(r_m) + delta_B(r_m)), plus the voyage across the
        // span within the passer's generation, c min(S / T, c)), less its
        // tithe (passage_j for every mouth's width it spans); nothing is built
        // where that is not positive
        let rulers = crate::growth::Rulers::default();
        let c = crate::propulsion::c_m_s();
        let rg_per_kpc = kpc_m() / Dy::from_q(&units().r_g_m);
        let hole_suns = Dy::from_q(&xyl_cosmos::package::hole_mass());
        let well = |n: u64| -> Dy {
            let Some(st) = self.index.get(&n).and_then(|&x| self.sample.get(x)) else {
                return Dy::ZERO;
            };
            // the star's mass and its mouth's reach, both in the core's r_g
            let (m, r) = (Dy::from_q(&st.star.mass) / hole_suns, st.reach * rg_per_kpc);
            if r + m > Dy::ZERO {
                m / (r + m)
            } else {
                Dy::ZERO
            }
        };
        let passer = |k: u64| -> Option<(Dy, Dy)> {
            let who = if key.1 {
                cosmic
                    .peoples
                    .iter()
                    .find(|q| !q.ancient && q.population.signum() > 0 && q.star == k)?
            } else {
                p
            };
            Some((
                crate::growth::body_kg(&who.traits, &rulers),
                crate::growth::generation_yr(&who.traits, &rulers) * year_s(),
            ))
        };
        let toll = |body: Dy, arc: Dy, k: u64| {
            tithe(
                passage_j(body, v_e),
                arc,
                self.reach_of(k).unwrap_or(Dy::ONE),
            )
        };
        let worth = |body: Dy, gen: Dy, arc: Dy, m: u64, k: u64| -> Dy {
            let voyage = c * (arc * kpc_m() / gen.max(Dy::ONE)).min(c);
            body * (c * c * (well(m) + well(k)) + voyage) - toll(body, arc, k)
        };
        // branch and bound: each candidate's worth is bounded above from its
        // arc's lower bound (voyage at its cap c, wells at their greatest,
        // tithe at the shortest span); candidates are taken greatest bound
        // first, stopping once no remaining bound can beat the best worth
        let w_max = mine
            .values()
            .flat_map(|v| v.iter().map(|x| well(x.1)))
            .max()
            .unwrap_or(Dy::ZERO);
        let mut bounded: Vec<(Dy, Dy, u64, Dy, Dy)> = per
            .into_values()
            .filter_map(|(lb, k)| {
                let (body, gen) = passer(k)?;
                let bound = body * (c * c * (well(k) + w_max) + c * c) - toll(body, lb, k);
                (bound.signum() > 0).then_some((bound, lb, k, body, gen))
            })
            .collect();
        bounded.sort_by_key(|b| Reverse(b.0));
        let mut best: Option<(Dy, u64, u64, Dy, Dy)> = None;
        for (bound, _, k, body, gen) in bounded {
            if best.as_ref().is_some_and(|b| b.3 >= bound) {
                break;
            }
            let Some(&(core, r_k)) = by_n.get(&k) else {
                continue;
            };
            // the joined mouths it is read from: the nearest few by reach in
            // its galaxy; across galaxies the cores' span is the same from
            // any joined star, so a few stand for all
            let pool: Vec<u64> = match mine.get(&core) {
                Some(v) => {
                    let at = v.partition_point(|x| x.0 < r_k);
                    let lo = at.saturating_sub(4);
                    v[lo..(at + 4).min(v.len())].iter().map(|x| x.1).collect()
                }
                None => mine
                    .values()
                    .flat_map(|v| v.iter().map(|x| x.1))
                    .take(8)
                    .collect(),
            };
            let near = pool
                .into_iter()
                .filter_map(|m| arc(m, k).map(|a| (a, m)))
                .min();
            let Some((arc, m)) = near else { continue };
            if arc > seen {
                continue;
            }
            let w = worth(body, gen, arc, m, k);
            if w.signum() > 0 && best.as_ref().is_none_or(|b| w > b.3) {
                best = Some((arc, m, k, w, body));
            }
        }
        let Some((arc, m, k, _, body)) = best else {
            return;
        };
        let mouth = self.reach_of(k).unwrap_or(Dy::ONE);
        let cost = tithe(passage_j(body, v_e), arc, mouth);
        let voyage = arc * kpc_m() / cruise / year_s();
        let paid = cost / spare_w / year_s();
        let due = start + voyage.max(paid) / Dy::sci(1, 9);
        self.works.insert(
            key,
            Ley {
                handling: Handling::Tolled,
                from: m,
                to: k,
                builder: i,
                tag: crate::checkpoint::tag(&p.name),
                progenitor: key.1,
                built: due,
                cost_j: cost,
            },
        );
    }

    /// The passage between two places (years), constellation by
    /// constellation: within a system, the drive's leg along the orbits plus
    /// the window wait weighted by w v_orb / (v_orb + cruise); within a
    /// constellation, the drive's leg along the arc; between constellations,
    /// only over the ley lines (each crossing a mouth's light time, scaled by
    /// its open share) with the drive's legs to and from the mouths. A line
    /// carries the passer `who` (a people's index) as its handling allows:
    /// a closed line no one, a guarded line its builder's house and the
    /// peoples of its realm. None where no passage joins them.
    pub fn passage_years(
        &self,
        cosmic: &Cosmic,
        spec: &Spec,
        ratio: Dy,
        escape: Dy,
        a: Place,
        b: Place,
        who: Option<usize>,
    ) -> Option<Dy> {
        let cruise = cruise_speed(spec, ratio, escape);
        if cruise.signum() <= 0 {
            return None;
        }
        if a.star == b.star {
            let years = voyage(spec, ratio, escape, separation_m(cosmic, a, b)?)?.years;
            let wit = cosmic.systems.get(&a.star)?;
            let all = manifolds(cosmic.gal.seed, &[a.star], &wit.system, &wit.star, &units());
            let age = (cosmic.now - Dy::from_q(&wit.star.born))
                .max(Dy::ZERO)
                .to_q();
            let wait = window_years(&all, a.world, b.world, &age, years)
                .map(|(w, v)| w * v / (v + cruise))
                .unwrap_or(Dy::ZERO);
            return Some(years + wait);
        }
        let (ca, cb) = (*self.of.get(&a.star)?, *self.of.get(&b.star)?);
        let star_leg = |x: u64, y: u64| {
            self.arc(cosmic, x, y)
                .map(|arc| arc * kpc_m() / cruise / year_s())
        };
        if ca == cb {
            return star_leg(a.star, b.star);
        }
        // the lines' mouths as the graph's nodes, a and b its ends
        let mut nodes: Vec<u64> = vec![a.star, b.star];
        for l in &self.lines {
            nodes.push(l.from);
            nodes.push(l.to);
        }
        nodes.sort_unstable();
        nodes.dedup();
        let mut edges: Vec<(u64, u64, Dy)> = Vec::new();
        for l in &self.lines {
            let open = self.open_share(l);
            if open.signum() <= 0 || !self.carries(cosmic, l, who) {
                continue;
            }
            edges.push((
                l.from,
                l.to,
                crossing_years(self.reach_of(l.to).unwrap_or(Dy::ZERO)) / open,
            ));
        }
        let mut by_c: BTreeMap<usize, Vec<u64>> = BTreeMap::new();
        for &x in &nodes {
            if let Some(&c) = self.of.get(&x) {
                by_c.entry(c).or_default().push(x);
            }
        }
        for group in by_c.values() {
            for (i, &x) in group.iter().enumerate() {
                for &y in &group[i + 1..] {
                    if let Some(t) = star_leg(x, y) {
                        edges.push((x, y, t));
                    }
                }
            }
        }
        let mut adj: BTreeMap<u64, Vec<(u64, Dy)>> = BTreeMap::new();
        for &(x, y, c) in &edges {
            adj.entry(x).or_default().push((y, c));
            adj.entry(y).or_default().push((x, c));
        }
        let mut dist: BTreeMap<u64, Dy> = BTreeMap::new();
        dist.insert(a.star, Dy::ZERO);
        let mut heap = BinaryHeap::new();
        heap.push(Reverse((Dy::ZERO, a.star)));
        while let Some(Reverse((d, s))) = heap.pop() {
            if s == b.star {
                return Some(d);
            }
            if dist.get(&s).is_some_and(|&best| d > best) {
                continue;
            }
            for &(w, c) in adj.get(&s).map(Vec::as_slice).unwrap_or(&[]) {
                let nd = d + c;
                if dist.get(&w).is_none_or(|&best| nd < best) {
                    dist.insert(w, nd);
                    heap.push(Reverse((nd, w)));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ley(from: u64, to: u64) -> Ley {
        Ley {
            from,
            to,
            builder: 0,
            tag: String::new(),
            progenitor: false,
            built: Dy::ZERO,
            cost_j: n(1),
            handling: Handling::Tolled,
        }
    }

    #[test]
    fn realms_join_constellations_and_stewards_set_handling() {
        let c = |stars: Vec<u64>| Constellation {
            stars,
            core: 0,
            reach_kpc: Dy::ONE,
        };
        let mut net = Network {
            constellations: vec![c(vec![1, 2]), c(vec![3]), c(vec![5]), c(vec![7])],
            lines: vec![ley(1, 3), ley(3, 5)],
            ..Network::default()
        };
        net.of = [(1, 0), (2, 0), (3, 1), (5, 2), (7, 3)]
            .into_iter()
            .collect();
        // one realm of three constellations, keyed by its least key; the
        // lone constellation forms none
        assert_eq!(net.realms(), vec![(1, vec![1, 3, 5])]);
        assert_eq!(net.realm_of(5), Some(1));
        assert_eq!(net.realm_of(7), None);
        assert_eq!(net.note_realms(), vec![1]);
        assert!(net.note_realms().is_empty(), "a realm is noted once");
        // the steward closes one line of its realm; a line outside it is untouched
        assert_eq!(net.handle(1, 5, 3, Handling::Closed), 1);
        assert_eq!(net.lines[1].handling, Handling::Closed);
        assert_eq!(net.handle(9, 1, 3, Handling::Guarded), 0);
        assert_eq!(net.lines[0].handling, Handling::Tolled);
        // the toll law reads a line's books
        let q = quote(
            &net.lane(&net.lines[0]),
            None,
            n(1),
            n(10),
            n(100),
            Handling::Tolled,
        );
        assert!(q.toll <= q.ceiling && q.floor == n(1));
    }
    use crate::propulsion::{energy_per_kg, exhaust_speed, Drive, MASS_RATIO};
    use xyl_core::Q;
    use xyl_cosmos::laws::Scales;
    use xyl_cosmos::manifold::Anchor;
    use xyl_cosmos::worlds::{Solvent, Species};

    fn spec(d: Drive) -> Spec {
        let el = xyl_cosmos::elements::table().unwrap();
        let e = energy_per_kg(d, Solvent::Volatile(Species::H2O), &Scales::default(), &el);
        Spec {
            drive: d,
            energy_per_kg: e,
            v_e: exhaust_speed(e),
        }
    }

    fn world(mass: i64, radius: i64, host: Option<usize>, a_host: Option<i64>) -> World {
        World {
            a: Q::ONE,
            mass: Q::int(mass),
            radius: Q::int(radius),
            giant: mass > 10,
            host,
            a_host_radii: a_host.map(Q::int),
            layers: [Q::ZERO, Q::ONE, Q::ZERO, Q::ZERO],
            elements: vec![],
            mantle: vec![],
            ices: vec![],
            e_forced: None,
            swing: Q::ZERO,
            phase: Q::ZERO,
            merged_at: None,
        }
    }

    #[test]
    fn worlds_are_as_far_as_their_orbits_and_voyages_take_what_their_drives_buy() {
        // a unit world's climb out of its well: the turning law at its
        // surface, c delta with delta = M / (R + M) in the core's units
        let unit_world = world(1, 1, None, None);
        let esc = escape_speed(&unit_world);
        let c = crate::propulsion::c_m_s();
        let (m, r) = (
            Dy::from_q(&(&Si::g() * &Si::world_mass_kg())) / (c * c),
            Dy::from_q(&Si::world_radius_m()),
        );
        assert!(
            (esc / (c * m / (r + m)) - Dy::ONE).abs() < Dy::sci(1, -9),
            "escape {esc}"
        );
        // orbits: where each stands now, read as turns and reaches - two
        // planets on circles of 1 and 5 units, in line then opposite; a moon
        // from its giant by its own orbit
        let ring = |world: usize, anchor: Anchor, tier: Tier, a: i64, m0: Q| Manifold {
            world,
            anchor,
            tier,
            a: Q::int(a),
            e0: Q::ZERO,
            e_forced: Q::ZERO,
            tau_e: Q::ZERO,
            rate: Q::int(360),
            m0,
            frame: xyl_core::rotor::Rotor::identity(),
            twist_deg: Q::ZERO,
            mass_ratio: Q::ZERO,
            chain: None,
        };
        let unit = Dy::from_q(&(&units().r_g_m * &Tier::System.in_rg()));
        let half = Q::new(1, 2).unwrap();
        let gap = |all: &[Manifold], a, b, age: &Q| orbit_gap_m(all, a, b, age).unwrap() / unit;
        let close = |x: Dy, want: i64| (x - n(want)).abs() < Dy::ratio(1, 1000);
        let inline = vec![
            ring(0, Anchor::Star, Tier::System, 1, Q::ZERO),
            ring(1, Anchor::Star, Tier::System, 5, Q::ZERO),
        ];
        assert!(close(gap(&inline, 0, 1, &Q::ZERO), 4));
        // opposite, the way round is half a turn swept at the inner reach
        // (pi) plus the climb (4), not the chord through the star
        let pi_plus_4 = Dy::from_q(&xyl_core::certified::pi(80).mid()) + n(4);
        let near = |x: Dy, want: Dy| (x - want).abs() < Dy::ratio(1, 1000);
        let opposite = vec![
            ring(0, Anchor::Star, Tier::System, 1, Q::ZERO),
            ring(1, Anchor::Star, Tier::System, 5, half.clone()),
        ];
        assert!(near(gap(&opposite, 0, 1, &Q::ZERO), pi_plus_4));
        // ...and the gap moves as they turn: half a turn on, the inner one
        // has come round to its far side of a still outer one
        let mut paced = inline.clone();
        paced[1].rate = Q::ZERO;
        assert!(near(gap(&paced, 0, 1, &half), pi_plus_4));
        let moon = vec![
            ring(0, Anchor::Star, Tier::System, 5, Q::ZERO),
            ring(1, Anchor::World(0), Tier::Planet, 60, Q::ZERO),
        ];
        let k = Dy::from_q(&xyl_cosmos::manifold::dilation(Tier::Planet, Tier::System));
        assert!((gap(&moon, 0, 1, &Q::ZERO) - n(60) * k).abs() < Dy::ratio(1, 1_000_000));
        // drive and well weigh in one law, c delta: a ten-to-one chemical
        // drive's energy (e ln R) is less than the unit world's well (G M / R), so
        // the well holds it down...
        let chem = spec(Drive::Chemical);
        let r = n(MASS_RATIO);
        assert!(voyage(&chem, r, esc, n(4) * au_m()).is_none());
        // ...while off a Moon it climbs out, and coasts for ages
        let moon_world = World {
            mass: Q::new(123, 10_000).unwrap(),
            radius: Q::new(273, 1000).unwrap(),
            ..world(1, 1, None, None)
        };
        let slow = voyage(&chem, r, escape_speed(&moon_world), n(4) * au_m())
            .expect("off a Moon it climbs out");
        assert!(slow.years > Dy::ZERO, "chemical in-system {}", slow.years);
        // fusion out of a unit world climbs out, and the same drive takes
        // longer over the longer span
        let fusion = spec(Drive::Fusion);
        let near = voyage(&fusion, r, esc, n(4) * au_m()).expect("fusion climbs out");
        let pc = kpc_m() / n(1000);
        let far = voyage(&fusion, r, esc, pc).unwrap();
        assert!(far.years > near.years, "{} vs {}", far.years, near.years);
        // and faster than the chemical drive over the same span
        assert!(near.years < slow.years);
        // a collapsed body's well - a sun's mass in ten kilometres - holds
        // even fusion down
        let collapsed = World {
            radius: Q::new(1, 637).unwrap(),
            ..world(333_000, 1, None, None)
        };
        assert!(voyage(&fusion, r, escape_speed(&collapsed), pc).is_none());
        // the fuel burned per kg carried: (R - 1) x its energy per kg
        assert_eq!(near.energy_per_kg, n(9) * fusion.energy_per_kg);
        assert!(reach_kpc(&spec(Drive::Fusion), r, esc, n(20)) > reach_kpc(&chem, r, esc, n(20)));
    }

    #[test]
    fn the_toll_sits_half_way_and_a_throat_tithes_its_span() {
        let lane = Lane {
            cost_j: n(1000),
            collected_j: n(200),
            last_year: 4,
        };
        assert_eq!(floor(&lane), n(200));
        // round is 5 hops, the lane 1: it saves 4 hops at 100 each
        let q = quote(&lane, Some(n(5)), n(1), n(100), n(10_000), Handling::Tolled);
        assert_eq!(q.ceiling, n(400));
        assert_eq!(q.toll, n(300));
        // open: only its floor; never above the ceiling
        assert_eq!(
            quote(&lane, Some(n(5)), n(1), n(100), n(10_000), Handling::Open).toll,
            n(200)
        );
        let dear = Lane {
            cost_j: n(100_000),
            collected_j: Dy::ZERO,
            last_year: 1,
        };
        assert_eq!(
            quote(&dear, Some(n(5)), n(1), n(100), n(10_000), Handling::Tolled).toll,
            n(400)
        );
        // no way round: all it carries and a hop
        assert_eq!(
            quote(&dear, None, n(1), n(100), n(10_000), Handling::Tolled).ceiling,
            n(10_100)
        );
        assert_eq!(tithe(n(7), n(30), n(3)), n(70));
        assert_eq!(tithe(n(7), n(1), n(3)), n(7));
    }
}
