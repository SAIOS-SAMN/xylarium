//! Coherence on S^11: an end-by-end process that binds arriving Z and Q
//! ends into fibres and yields the sealed charges 2, 28, 208, 1092.
//!
//! No inputs beyond a count of ends; all arithmetic is exact `u64`. Each
//! arriving end, held (Z++) or open (Q++), stands on S^11, whose independent
//! directions at harmonic degree l number:
//!
//! ```text
//! d(l) = C(l + 11, 11) - C(l + 9, 11)      (1, 12, 77, 352, ...)
//! ```
//!
//! (`directions`; the same count as `atoms::orientations`).
//!
//! - An end first claims a free direction of the open degree alone (a lone
//!   end, counted in `free`).
//! - Once every direction of the degree is claimed, the next end binds to a
//!   lone end, forming a fibre: one held and one open end on one direction.
//! - A degree closes when every direction holds a fibre, and the next degree
//!   opens. Turning n holds degrees 0..n - 1.
//!
//! Outputs: `Coherence` (with `arrive`, `kappa`, `sealed`, `whole`), `grow`
//! and `sealed_fibres`. Invariant: ends = 2 G + free after every arrival
//! (`whole`). Coherence is kappa = 2 G / ends; a body seals when kappa = 1
//! and its last degree is closed, which happens when the fibre count is:
//!
//! ```text
//! G = C(N + 11, 12) + C(N + 10, 12)        (1, 14, 104, 546, ...)
//! ```
//!
//! for N closed turnings (ends 2, 28, 208, 1092).

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
    /// The current turning, counted from 1.
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
    fn fibre_seals_match_the_atom_and_lose_no_end() {
        // The count directions(l) equals the S^11 harmonic count
        // atoms::orientations.
        for l in 0..8u32 {
            assert_eq!(
                directions(u64::from(l)),
                u64::from(crate::atoms::orientations(l))
            );
        }
        // Grown end by end to 1200, no end is lost at any arrival, and every
        // seal carries the telescoped harmonics' fibre count.
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
        // The atom fill seals at the same Z.
        for z in [2u32, 28, 208, 1092] {
            assert!(
                crate::atoms::atom(z).is_some_and(|a| a.sealed()),
                "Z {z} seals in the atom too"
            );
        }
    }
}
