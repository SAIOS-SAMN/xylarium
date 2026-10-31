//! Building ley lines: carrying the network through time event by event
//! ([`Network::step`]), and where a builder sites its next line by the worth
//! law.

use super::{best_spec, cruise_speed, home_escape, kpc_m, kpc_per_gyr, passage_j, tithe, year_s};
use super::{Handling, Ley, Network, LINES_PER_READING};
use crate::cosmic::{Cosmic, People};
use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use xyl_core::dyadic::Dy;
use xyl_core::Q;
use xyl_cosmos::chain::{units, Place};

/// A house able to build: its people's index, its cruise speed out of its
/// home well (m/s), its spare power (W) and its drive's exhaust speed (m/s).
type Builder = (usize, Dy, Dy, Dy);

/// Each joined star by galactic core, as (orbit radius in kpc, sample
/// number), sorted by radius.
type JoinedByCore = BTreeMap<u32, Vec<(Dy, u64)>>;

/// Each star in play's galactic core and orbit radius (kpc), by sample number.
type CoreAndRadius = BTreeMap<u64, (u32, Dy)>;

/// The best line found: (arc in kpc, near mouth, far mouth, worth in J, the
/// passer's body mass in kg).
type Best = (Dy, u64, u64, Dy, Dy);

impl Network {
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
        let houses = builders(cosmic);
        let in_tune = in_tune_stars(cosmic, discord);
        // Starts of idle builders and dues of works are taken in one time
        // order, so `current` only ever reads the constellations forward.
        let mut idle = self.idle_builders(cosmic, &houses, from, to);
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
                out.push(opening_entry(cosmic, key, &done));
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
        self.read_quarantine(cosmic, discord, to, &mut out);
        out
    }

    /// The builders with no line under way and the time each begins: after
    /// its last line opened, never before its dawn or the last reading
    /// (`from`), and only those beginning by `to`.
    fn idle_builders(
        &self,
        cosmic: &Cosmic,
        houses: &[Builder],
        from: Dy,
        to: Dy,
    ) -> Vec<(Dy, (usize, bool))> {
        let mut idle = Vec::new();
        for &(i, _, _, _) in houses {
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
        idle
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
        let wanted: Option<BTreeSet<usize>> =
            want.map(|w| w.iter().filter_map(|s| self.of.get(s).copied()).collect());
        let Some(p) = cosmic.peoples.get(i) else {
            return;
        };
        let Some(&home) = self.of.get(&p.star) else {
            return;
        };
        let joined = self.joined(home);
        let by_n: CoreAndRadius = self
            .sample
            .iter()
            .map(|s| (s.star.n, (s.star.core, s.star.orbit.radius)))
            .collect();
        let mine = self.joined_by_core(&joined, &by_n);
        let placer = Placer::new(self, start);
        // A house has discovered the sky its craft could have crossed since
        // its dawn, kpc_per_gyr(cruise) (start - dawn); constellations beyond
        // that are not candidates.
        let seen = kpc_per_gyr(cruise) * (start - p.dawn).max(Dy::ZERO);
        let per = self.candidates(&placer, &joined, wanted.as_ref(), &by_n, &mine, seen);
        let siting = Siting::new(self, cosmic, p, key.1, v_e);
        let Some((arc, m, k, _, body)) = siting.best(&placer, per, &by_n, &mine, seen) else {
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

    /// The joined stars by galactic core, each core's sorted by orbit radius.
    fn joined_by_core(&self, joined: &[bool], by_n: &CoreAndRadius) -> JoinedByCore {
        let mut mine: JoinedByCore = BTreeMap::new();
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
        mine
    }

    /// One candidate per discovered, unjoined constellation (restricted to
    /// `wanted` when given): its star nearest the joined stars by a lower
    /// bound on the arc - within a galaxy the difference of orbit radii,
    /// across galaxies the arc between the cores. Returns (lower bound in
    /// kpc, star) by constellation.
    fn candidates(
        &self,
        placer: &Placer<'_>,
        joined: &[bool],
        wanted: Option<&BTreeSet<usize>>,
        by_n: &CoreAndRadius,
        mine: &JoinedByCore,
        seen: Dy,
    ) -> BTreeMap<usize, (Dy, u64)> {
        let cores_at = placer.cores();
        let mut between: BTreeMap<u32, Dy> = BTreeMap::new();
        let mut per: BTreeMap<usize, (Dy, u64)> = BTreeMap::new();
        for (&k, &c) in &self.of {
            if joined[c] || wanted.is_some_and(|w| !w.contains(&c)) {
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
                None => core_lower_bound(&mut between, &cores_at, mine, core, &placer.sphere),
            };
            if lb > seen {
                continue;
            }
            let e = per.entry(c).or_insert((lb, k));
            if lb < e.0 {
                *e = (lb, k);
            }
        }
        per
    }
}

/// The houses that can build: living ancient peoples whose swiftest drive
/// climbs out of their home well and whose world has spare power.
fn builders(cosmic: &Cosmic) -> Vec<Builder> {
    let ratio = Dy::int(crate::propulsion::MASS_RATIO);
    cosmic
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
        .collect()
}

/// The progenitors' targets: the stars of living non-ancient peoples whose
/// discord is under a half.
fn in_tune_stars(cosmic: &Cosmic, discord: &BTreeMap<usize, Dy>) -> Vec<u64> {
    cosmic
        .peoples
        .iter()
        .enumerate()
        .filter(|(i, p)| {
            !p.ancient
                && p.population.signum() > 0
                && discord.get(i).is_none_or(|d| *d < Dy::ratio(1, 2))
        })
        .map(|(_, p)| p.star)
        .collect()
}

/// The annals entry for a line opening.
fn opening_entry(cosmic: &Cosmic, key: (usize, bool), done: &Ley) -> (Dy, &'static str, String) {
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
    (
        done.built,
        "LEY",
        format!(
            "The {who} open a ley line from star {} to star {} ({} J held open)",
            done.from, done.to, done.cost_j
        ),
    )
}

/// A lower bound on the arc (kpc) from the joined stars to any star of
/// galaxy `core`: the least arc between `core`'s centre and a joined
/// galaxy's centre, memoised in `between`; 10^12 kpc where a centre is not
/// placed.
fn core_lower_bound(
    between: &mut BTreeMap<u32, Dy>,
    cores_at: &BTreeMap<u32, Place>,
    mine: &JoinedByCore,
    core: u32,
    sphere: &Q,
) -> Dy {
    *between.entry(core).or_insert_with(|| {
        let Some(there) = cores_at.get(&core) else {
            return Dy::sci(1, 12);
        };
        mine.keys()
            .filter_map(|g| cores_at.get(g))
            .map(|here| here.arc_kpc(there, sphere))
            .min()
            .unwrap_or(Dy::sci(1, 12))
    })
}

/// Stars in play placed at one time, each placed once and cached.
struct Placer<'a> {
    net: &'a Network,
    start: Dy,
    sphere: Q,
    placed: RefCell<BTreeMap<u64, Place>>,
}

impl<'a> Placer<'a> {
    fn new(net: &'a Network, start: Dy) -> Self {
        Placer {
            net,
            start,
            sphere: xyl_cosmos::chain::radius(),
            placed: RefCell::default(),
        }
    }

    /// Star `n` placed at `start`; None if it is not in play.
    fn place(&self, n: u64) -> Option<Place> {
        if let Some(p) = self.placed.borrow().get(&n) {
            return Some(p.clone());
        }
        let p = self
            .net
            .sample
            .get(*self.net.index.get(&n)?)?
            .star
            .place(self.start, &self.sphere);
        self.placed.borrow_mut().insert(n, p.clone());
        Some(p)
    }

    /// The arc between stars `a` and `b` at `start` (kpc).
    fn arc(&self, a: u64, b: u64) -> Option<Dy> {
        Some(self.place(a)?.arc_kpc_dy(&self.place(b)?, &self.sphere))
    }

    /// Each galaxy's centre at `start`, read from the first of its stars in
    /// play that carries it. The arc between two centres is a lower bound on
    /// every arc between their stars.
    fn cores(&self) -> BTreeMap<u32, Place> {
        let mut cores_at: BTreeMap<u32, Place> = BTreeMap::new();
        for s in &self.net.sample {
            if let std::collections::btree_map::Entry::Vacant(e) = cores_at.entry(s.star.core) {
                if let Some(cp) = self.place(s.star.n).and_then(|x| x.core.map(|b| *b)) {
                    e.insert(cp);
                }
            }
        }
        cores_at
    }
}

/// The siting law. A line is worth its passer's mass times what it saves per
/// kg against normal transit, less its tithe: the two wells' remainders at
/// their mouths, c^2 (delta_A(r_m) + delta_B(r_m)), plus the voyage across
/// the span within the passer's generation, c min(S / T, c); the tithe is
/// `passage_j` for every mouth's width spanned. A line is built only where
/// the worth is positive.
struct Siting<'a> {
    net: &'a Network,
    cosmic: &'a Cosmic<'a>,
    builder: &'a People,
    progenitors: bool,
    v_e: Dy,
    rulers: crate::growth::Rulers,
    c: Dy,
    rg_per_kpc: Dy,
    hole_suns: Dy,
}

impl<'a> Siting<'a> {
    fn new(
        net: &'a Network,
        cosmic: &'a Cosmic<'a>,
        builder: &'a People,
        progenitors: bool,
        v_e: Dy,
    ) -> Self {
        Siting {
            net,
            cosmic,
            builder,
            progenitors,
            v_e,
            rulers: crate::growth::Rulers::default(),
            c: crate::propulsion::c_m_s(),
            rg_per_kpc: kpc_m() / Dy::from_q(&units().r_g_m),
            hole_suns: Dy::from_q(&xyl_cosmos::package::hole_mass()),
        }
    }

    /// Star `n`'s well remainder at its mouth, delta = m / (r + m), with the
    /// star's mass m and its mouth's reach r both in the core's r_g.
    fn well(&self, n: u64) -> Dy {
        let Some(st) = self.net.index.get(&n).and_then(|&x| self.net.sample.get(x)) else {
            return Dy::ZERO;
        };
        let (m, r) = (
            Dy::from_q(&st.star.mass) / self.hole_suns,
            st.reach * self.rg_per_kpc,
        );
        if r + m > Dy::ZERO {
            m / (r + m)
        } else {
            Dy::ZERO
        }
    }

    /// The passer for a line to star `k`: the builder itself, or for
    /// progenitors the living non-ancient people at `k`. Returns its body
    /// mass (kg) and generation (s); None where progenitors find no people.
    fn passer(&self, k: u64) -> Option<(Dy, Dy)> {
        let who = if self.progenitors {
            self.cosmic
                .peoples
                .iter()
                .find(|q| !q.ancient && q.population.signum() > 0 && q.star == k)?
        } else {
            self.builder
        };
        Some((
            crate::growth::body_kg(&who.traits, &self.rulers),
            crate::growth::generation_yr(&who.traits, &self.rulers) * year_s(),
        ))
    }

    /// The tithe (J) of a line of `arc` kpc ending at star `k`.
    fn toll(&self, body: Dy, arc: Dy, k: u64) -> Dy {
        tithe(
            passage_j(body, self.v_e),
            arc,
            self.net.reach_of(k).unwrap_or(Dy::ONE),
        )
    }

    /// The line's worth (J) from mouth `m` to `k` across `arc` kpc.
    fn worth(&self, body: Dy, gen: Dy, arc: Dy, m: u64, k: u64) -> Dy {
        let c = self.c;
        let voyage = c * (arc * kpc_m() / gen.max(Dy::ONE)).min(c);
        body * (c * c * (self.well(m) + self.well(k)) + voyage) - self.toll(body, arc, k)
    }

    /// The best line by branch and bound: each candidate's worth is bounded
    /// above from its arc's lower bound (voyage at its cap c, wells at their
    /// greatest, tithe at the shortest span); candidates are taken greatest
    /// bound first, stopping once no remaining bound can beat the best worth.
    fn best(
        &self,
        placer: &Placer<'_>,
        per: BTreeMap<usize, (Dy, u64)>,
        by_n: &CoreAndRadius,
        mine: &JoinedByCore,
        seen: Dy,
    ) -> Option<Best> {
        let c = self.c;
        let w_max = mine
            .values()
            .flat_map(|v| v.iter().map(|x| self.well(x.1)))
            .max()
            .unwrap_or(Dy::ZERO);
        let mut bounded: Vec<(Dy, Dy, u64, Dy, Dy)> = per
            .into_values()
            .filter_map(|(lb, k)| {
                let (body, gen) = self.passer(k)?;
                let bound =
                    body * (c * c * (self.well(k) + w_max) + c * c) - self.toll(body, lb, k);
                (bound.signum() > 0).then_some((bound, lb, k, body, gen))
            })
            .collect();
        bounded.sort_by_key(|b| Reverse(b.0));
        let mut best: Option<Best> = None;
        for (bound, _, k, body, gen) in bounded {
            if best.as_ref().is_some_and(|b| b.3 >= bound) {
                break;
            }
            let Some(&(core, r_k)) = by_n.get(&k) else {
                continue;
            };
            let near = mouth_pool(mine, core, r_k)
                .into_iter()
                .filter_map(|m| placer.arc(m, k).map(|a| (a, m)))
                .min();
            let Some((arc, m)) = near else { continue };
            if arc > seen {
                continue;
            }
            let w = self.worth(body, gen, arc, m, k);
            if w.signum() > 0 && best.as_ref().is_none_or(|b| w > b.3) {
                best = Some((arc, m, k, w, body));
            }
        }
        best
    }
}

/// The joined mouths a candidate at radius `r_k` of galaxy `core` is read
/// from: the nearest few by radius in its own galaxy. Across galaxies the
/// arc is dominated by the cores' span, which is the same from any joined
/// star, so the first eight joined stars stand for all.
fn mouth_pool(mine: &JoinedByCore, core: u32, r_k: Dy) -> Vec<u64> {
    match mine.get(&core) {
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
    }
}
