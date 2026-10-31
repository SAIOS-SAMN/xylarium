//! Realms and handling: the constellations joined over open lines, the
//! realms' keys for steward seats, and how a line's handling admits a passer.

use super::{Handling, Lane, Ley, Network};
use crate::cosmic::Cosmic;
use std::collections::BTreeMap;
use xyl_core::dyadic::Dy;

impl Network {
    /// The constellations joined to `home` over the lines (the network is
    /// common: every signatory's lines carry every signatory).
    pub(super) fn joined(&self, home: usize) -> Vec<bool> {
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
    /// traffic is carried yet, so nothing), for the toll law [`quote`](super::quote).
    pub fn lane(&self, l: &Ley) -> Lane {
        Lane {
            cost_j: l.cost_j,
            collected_j: Dy::ZERO,
            last_year: 0,
        }
    }
}
