//! Discord, folding and quarantine: the parts of a people's [`Discord`], the
//! people's breach share from folding space, and the share by which a
//! constellation is held closed.

use super::{kpc_m, passage_j, year_s, Ley, Network};
use crate::cosmic::Cosmic;
use std::collections::BTreeMap;
use xyl_core::dyadic::Dy;

/// A people's discord and the parts it is computed from (universe.rs
/// `discord_parts`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Discord {
    /// The distance the people's ships cover in one generation, in kpc.
    pub reach_kpc: Dy,
    /// The reach of the people's constellation (the width of a ley mouth
    /// there), in kpc.
    pub mouth_kpc: Dy,
    /// The forgetting share, 1 - z.
    pub forget: Dy,
    /// Whether the people's constellation holds a ley mouth.
    pub joined: bool,
    /// The exhaust speed of the people's drive, in m/s.
    pub v_e: Dy,
    /// The ships' coasting speed out of the home well, in m/s.
    pub cruise: Dy,
    /// The home world's escape speed, in m/s.
    pub escape: Dy,
    /// The breach share from folding space, in \[0, 1\]: the energy the
    /// people has folded, divided by that energy plus the cost of the lawful
    /// lines of its constellation plus one passage by the people's drive (see
    /// [`Network::breach`]).
    pub breach: Dy,
    /// The discord, a share in \[0, 1\]: `1 - (1 - o f (1 - J))(1 - breach)`,
    /// where o = reach / (reach + mouth) is the off-constellation share, f
    /// the forgetting share and J is 1 when the constellation holds a ley
    /// mouth and 0 otherwise.
    pub discord: Dy,
}

impl Network {
    /// Returns people `i`'s breach share from folding space: the passage
    /// energy the people has folded, divided by that energy plus the cost of
    /// the lawful lines with a mouth in its constellation plus one passage
    /// by the people's drive (`own_j`). Returns zero for a people that has
    /// not folded.
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

    /// Carries people `i`'s folding of space (`Laws::ftl`) to time `at`
    /// (Gyr) and returns the energy folded in that interval (J). A people
    /// with drive exhaust `v_e`, body mass `body_kg` and generation `gen_yr`
    /// (years) folds when the universe's laws allow it, its constellation
    /// holds no ley mouth, and a passage out is worth building by the siting
    /// law at the people's drive: over the shortest span out, the
    /// constellation's reach S, c min(S / T, c) > c (c - v_e) per kg with T
    /// one generation. It folds once per generation over the interval since
    /// its folding was last read, each fold costing [`passage_j`].
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

    /// Returns the share, in [0, 1], by which the constellation of star
    /// `star` is held closed by quarantine (0 when the star belongs to no
    /// constellation or the constellation has no quarantine entry).
    pub fn shut_of(&self, star: u64) -> Dy {
        self.of
            .get(&star)
            .and_then(|&c| self.constellations.get(c))
            .and_then(|c| self.quarantine.get(&c.key()))
            .copied()
            .unwrap_or(Dy::ZERO)
    }

    /// Returns whether the constellation of `star` holds a ley mouth.
    pub fn holds_mouth(&self, star: u64) -> bool {
        let Some(c) = self.of.get(&star) else {
            return false;
        };
        self.lines
            .iter()
            .any(|l| self.of.get(&l.from) == Some(c) || self.of.get(&l.to) == Some(c))
    }

    /// Returns a line's open share: the product of (1 - quarantine share) of
    /// the constellations at the two mouths.
    pub fn open_share(&self, l: &Ley) -> Dy {
        (Dy::ONE - self.shut_of(l.from)) * (Dy::ONE - self.shut_of(l.to))
    }

    /// Reads each constellation's quarantine at time `to`: the share
    /// 1 - prod(1 - discord) over the constellation's peoples, recomputed at
    /// every reading rather than latched. A share that crosses one half in
    /// either direction is entered in `out`.
    pub(super) fn read_quarantine(
        &mut self,
        cosmic: &Cosmic,
        discord: &BTreeMap<usize, Dy>,
        to: Dy,
        out: &mut Vec<(Dy, &'static str, String)>,
    ) {
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
                let text = format!(
                    concat!(
                        "The ley lines about the constellation of star {} close: ",
                        "its peoples ring out of tune ({} held closed)"
                    ),
                    k, q
                );
                out.push((to, "QUARANTINE", text));
            }
        }
        for (&k, &was) in &self.quarantine {
            let q = shut.get(&k).copied().unwrap_or(Dy::ZERO);
            if was >= half && q < half {
                let text = format!(
                    concat!(
                        "The ley lines about the constellation of star {} reopen: ",
                        "its peoples ring in step ({} held closed)"
                    ),
                    k, q
                );
                out.push((to, "REOPENED", text));
            }
        }
        self.quarantine = shut;
    }
}
