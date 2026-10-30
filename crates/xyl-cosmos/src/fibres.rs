//! The fibres: Z and Q converging on S^11. S^11 is the overlap where Z (via
//! Z++) and Q (via Q++) converge into coherence; the process is a formula, not
//! a table, and loses no state.
//!
//! The two poles are bound rather than counted separately. Every arriving end,
//! held (Z++) or open (Q++), stands on S^11, whose independent directions are
//! its harmonics: degree l has
//!
//! ```text
//! d(l) = C(l + 11, 11) - C(l + 9, 11)      (1, 12, 77, 352, ...)
//! ```
//!
//! of them, mutually orthogonal (atoms.rs `orientations`, the same formula).
//!
//! - An end first claims a direction alone: arriving ends spread over the
//!   free orthogonal directions of the open degree, one end each (the open
//!   frames, Q's share of the state).
//! - Then it binds: once every direction of the degree is claimed, the next
//!   end binds to a lone end, forming a fibre - one held and one open end on
//!   one direction, counter-turning (the glue, G).
//! - A degree closes when every direction holds a fibre, and the next degree
//!   opens. Turning n holds degrees 0..n - 1.
//!
//! The books are exact: every arrived end is free or one end of a fibre,
//! ends = 2 G + free, at every step. Coherence is the bound share,
//! kappa = 2 G / ends (the bond law's 2 min / sum), 1 when every end is
//! fibred. A body converges (seals) when kappa = 1 and its last degree is
//! closed; by the telescoped harmonics this happens at
//!
//! ```text
//! G = C(N + 11, 12) + C(N + 10, 12)        (1, 14, 104, 546, ...)
//! ```
//!
//! fibres (ends 2, 28, 208, 1092), produced by the process rather than listed.

/// C(n, k), exact; zero when k exceeds n.
fn choose(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    (0..k).fold(1u64, |acc, i| acc * (n - i) / (i + 1))
}

/// The independent directions of harmonic degree `l` on S^11.
pub fn directions(l: u64) -> u64 {
    choose(l + 11, 11) - if l >= 2 { choose(l + 9, 11) } else { 0 }
}

/// The fibres a body holds when it has closed every degree of turnings 1..=n:
/// the harmonics telescoped twice, C(n + 11, 12) + C(n + 10, 12).
pub fn sealed_fibres(n: u64) -> u64 {
    choose(n + 11, 12) + choose(n + 10, 12)
}

/// A body's state on the overlap: arrived ends, fibres bound, lone ends still
/// claiming a direction, its turning and current degree, and how many of that
/// degree's directions hold a lone end or a fibre.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Coherence {
    /// Every end that has arrived (held and open alike).
    pub ends: u64,
    /// Fibres: one held end and one open end bound on a direction.
    pub fibres: u64,
    /// Lone ends: a direction claimed, not yet bound.
    pub free: u64,
    /// The turning (from 1) and the degree within it (0..turning).
    pub turning: u64,
    /// Degree within the current turning, 0 <= degree < turning.
    pub degree: u64,
    /// The current degree's directions holding a lone end, and holding a fibre.
    lone: u64,
    bound: u64,
}

impl Coherence {
    /// An empty body: no ends arrived, at turning 1, degree 0.
    pub fn new() -> Coherence {
        Coherence {
            turning: 1,
            ..Coherence::default()
        }
    }

    /// An end arrives (Z++ or Q++ alike; the pole decides which end of a fibre
    /// it becomes). It claims a free direction of the current degree if one is
    /// empty, otherwise binds to a lone end; a degree whose every direction
    /// holds a fibre closes and the next opens.
    pub fn arrive(&mut self) {
        let d = directions(self.degree);
        self.ends += 1;
        if self.lone + self.bound < d {
            self.lone += 1;
            self.free += 1;
        } else {
            self.lone -= 1;
            self.bound += 1;
            self.free -= 1;
            self.fibres += 1;
        }
        if self.bound == d {
            self.lone = 0;
            self.bound = 0;
            if self.degree + 1 < self.turning {
                self.degree += 1;
            } else {
                self.turning += 1;
                self.degree = 0;
            }
        }
    }

    /// The books: every end is free or one end of a fibre.
    pub fn whole(&self) -> bool {
        self.ends == 2 * self.fibres + self.free
    }

    /// Coherence, the bound share 2 G / ends, as (numerator, denominator).
    pub fn kappa(&self) -> (u64, u64) {
        (2 * self.fibres, self.ends.max(1))
    }

    /// Converged: every end fibred and every degree it opened closed.
    pub fn sealed(&self) -> bool {
        self.free == 0 && self.ends > 0 && self.lone == 0 && self.bound == 0 && self.degree == 0
    }
}

/// A body of `charge` ends, grown end by end from nothing.
pub fn grow(charge: u64) -> Coherence {
    let mut c = Coherence::new();
    for _ in 0..charge {
        c.arrive();
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_seals_fall_out_of_the_fibres_and_nothing_is_lost() {
        // the directions are S^11's harmonics, the formula atoms.rs fills by
        for l in 0..8u32 {
            assert_eq!(
                directions(u64::from(l)),
                u64::from(crate::atoms::orientations(l))
            );
        }
        // grown end by end to 1200: the books whole at every step, and the
        // bodies that converge are exactly the telescoped harmonics' seals
        let mut c = Coherence::new();
        let mut seals = Vec::new();
        for z in 1..=1200u64 {
            c.arrive();
            assert!(c.whole(), "lost an end at {z}");
            if c.sealed() {
                assert_eq!(c.fibres, sealed_fibres(c.turning - 1), "seal at {z}");
                assert_eq!(
                    (c.kappa().0, c.kappa().1),
                    (z, z),
                    "every end fibred at a seal"
                );
                seals.push(z);
            }
        }
        assert_eq!(seals, vec![2, 28, 208, 1092]);
        // the atom's own reading agrees: its open frames are the lone ends
        // wherever the atom fills in the same order, and its seals are these
        for z in [2u32, 28, 208, 1092] {
            assert!(
                crate::atoms::atom(z).is_some_and(|a| a.sealed()),
                "Z {z} seals in the atom too"
            );
        }
    }
}
