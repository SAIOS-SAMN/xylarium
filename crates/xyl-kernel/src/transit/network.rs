//! The network: its ley lines ([`Ley`]), the stars in play and their
//! constellations as last read, and the arcs between stars at a time.

use super::{binding_pairs, constellations, in_play, kpc_m, year_s};
use super::{Constellation, Handling, InPlay};
use crate::cosmic::Cosmic;
use std::collections::BTreeMap;
use xyl_core::dyadic::Dy;

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
}
