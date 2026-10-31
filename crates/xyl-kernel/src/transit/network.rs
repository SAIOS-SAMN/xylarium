//! The network: the ley lines ([`Ley`]), the stars in play and their
//! constellations as last read, and the arcs between stars at a given time.

use super::{binding_pairs, constellations, in_play, kpc_m, year_s};
use super::{Constellation, Handling, InPlay};
use crate::cosmic::Cosmic;
use std::collections::BTreeMap;
use xyl_core::dyadic::Dy;

/// A ley line: a circulation held open between two constellations, built by
/// an ancient house or its progenitors, with its mouths at two stars.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ley {
    /// The star of the first mouth, by sample number.
    pub from: u64,
    /// The star of the second mouth, by sample number.
    pub to: u64,
    /// The builder, as a people's index: an ancient house, whose
    /// progenitors built the line when `progenitor` is set.
    pub builder: usize,
    /// The builder's name tag (see [`crate::checkpoint::tag`]).
    pub tag: String,
    /// Whether the house's progenitors built it rather than the house itself.
    pub progenitor: bool,
    /// When the line opens or opened, in Gyr after the Big Bang.
    pub built: Dy,
    /// The energy that holding the line open cost, in J.
    pub cost_j: Dy,
    /// The line's handling of passers, as the realm's steward set it.
    pub handling: Handling,
}

/// The largest number of lines one house opens in one reading; the next
/// reading continues from where it stopped. This is a declared ruler that
/// bounds the work of one reading.
pub const LINES_PER_READING: usize = 1024;

/// The network: the stars in play and their positions, the ley lines built,
/// the works under way, the constellations as last read and the quarantine
/// share of each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Network {
    /// Every ley line opened, in opening order.
    pub lines: Vec<Ley>,
    /// Each builder's line under way, keyed by the house and whether its
    /// progenitors build it; the line's `built` is its due time.
    pub works: BTreeMap<(usize, bool), Ley>,
    /// The constellations as last read.
    pub constellations: Vec<Constellation>,
    /// Every realm's key (its least constellation key) in order of first
    /// appearance; a steward seat's index into this list is stable.
    pub realm_keys: Vec<u64>,
    /// Each people's folds of space: the passage energy folded (J) and when
    /// the people's folding was last read (Gyr).
    pub folds: BTreeMap<usize, (Dy, Dy)>,
    /// Each constellation's quarantine share, by constellation key, as last
    /// read.
    pub quarantine: BTreeMap<u64, Dy>,
    /// The time to which the network was carried, in Gyr.
    pub read_to: Dy,
    /// The stars in play (`in_play`), with every witnessed star added.
    pub sample: Vec<InPlay>,
    /// Each star's index into `sample`, by its sample number.
    pub index: BTreeMap<u64, usize>,
    /// The pairs of `sample` indices that can bind, with their spans (kpc),
    /// read once per sample.
    pub pairs: Vec<(usize, usize, Dy)>,
    /// Whether each pair (as `pairs`) was bound at its last measurement.
    pub bound: Vec<bool>,
    /// When each pair (as `pairs`) must be measured again, in Gyr: the gap
    /// between the pair's arc and its span divided by the sum of the two
    /// stars' speed bounds, but never sooner than one ring of the bell, so a
    /// pair is re-read only when it could have crossed.
    pub check: Vec<Dy>,
    /// Each star's constellation (index into `constellations`), as last
    /// read.
    pub of: BTreeMap<u64, usize>,
    /// When the constellations were last read, in Gyr.
    pub read_at: Dy,
    /// Until when that reading holds, in Gyr.
    pub holds_to: Dy,
}

/// Returns a passage's crossing time (years): the light time across a
/// mouth of width `reach_kpc`. The throat shortens any span to the mouth's
/// width, so no light time is charged for the span itself.
pub fn crossing_years(reach_kpc: Dy) -> Dy {
    reach_kpc * kpc_m() / crate::propulsion::c_m_s() / year_s()
}

impl Network {
    /// Seats the stars in play for a universe's forecast (`limit` per galaxy,
    /// the galaxy's sight) and resets the pairs and constellations.
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

    /// Adds every witnessed star not yet in play. Each new star stands for
    /// `stands_for` times the ratio `suns / stands_for` of a sampled star of
    /// the same core (1 when there is none). The pairs are read again only
    /// when a star joins.
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

    /// Brings the constellations up to time `t`. Only the pairs whose check
    /// time has come are measured again (all of them when `t` is earlier
    /// than the last reading), each star is placed once, and the groups are
    /// re-formed only when a pair's bond changed. The reading holds until the
    /// earliest check time.
    pub(super) fn current(&mut self, cosmic: &Cosmic, t: Dy) {
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
                    let at = xyl_cosmos::circuit::rings_of_gyr_dy(t);
                    e.insert(self.sample[x].star.place(at, &sphere));
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

    /// Returns the arc (kpc) between stars `a` and `b` at time `t`, each
    /// placed at `t`, or None if either is not in play.
    pub fn arc_at(&self, a: u64, b: u64, t: Dy) -> Option<Dy> {
        let sphere = xyl_cosmos::chain::radius();
        let (sa, sb) = (
            &self.sample.get(*self.index.get(&a)?)?.star,
            &self.sample.get(*self.index.get(&b)?)?.star,
        );
        // The cosmos places stars in rings; `t` (Gyr) converts here.
        let t = xyl_cosmos::circuit::rings_of_gyr_dy(t);
        Some(sa.place(t, &sphere).arc_kpc(&sb.place(t, &sphere), &sphere))
    }

    /// Returns the arc (kpc) between stars `a` and `b` at the current time.
    pub fn arc(&self, cosmic: &Cosmic, a: u64, b: u64) -> Option<Dy> {
        self.arc_at(a, b, cosmic.now)
    }

    /// Returns the due time (Gyr) of the next line under way, an event of the
    /// universe.
    pub fn next_due(&self) -> Option<Dy> {
        self.works.values().map(|w| w.built).min()
    }

    /// Returns the reach (kpc) of the constellation containing `star`.
    pub fn reach_of(&self, star: u64) -> Option<Dy> {
        self.of
            .get(&star)
            .and_then(|&c| self.constellations.get(c))
            .map(|c| c.reach_kpc)
    }
}
