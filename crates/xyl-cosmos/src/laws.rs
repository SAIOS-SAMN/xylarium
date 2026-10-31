//! A universe's laws: four dimensionless constants given as ratios to
//! reference values, and the physical scales those ratios set.
//!
//! No inputs from other modules. The reference values (`reference`) are
//! exact rationals of the measured fine-structure constant alpha
//! (1/137.035999), electron-to-proton mass ratio beta (1/1836.15267),
//! gravitational coupling alpha_G (5.906e-39) and nuclear efficiency epsilon
//! (0.0071). `Laws` holds each constant over its reference (1 = reference)
//! and `ftl`, whether space can be folded (read by xyl-kernel's
//! `transit::Network::fold`).
//!
//! Outputs: `Laws::chemistry`, `pressure`, `density`, `atom`, `proton`,
//! `star_mass`, `star_radius` (each an `Encl` over the reference), and
//! `Laws::scales` -> `Scales`, their midpoints as `Dy` for the heavy models;
//! `Scales::half_life` scales an alpha decay's half-life as
//! `exp(gamow (alpha - 1))`.
//!
//! Each scale is a product of factors, one per constant and nominal power
//! `x^(n/d)` (e.g. chemistry `alpha^2 beta alpha_G^(1/2)`). Powers 1 and -1
//! are exact; any other power p = |n| / d is replaced by the share
//! `(1 + p) x / (1 + p x)` (its reciprocal for negative n), which is 1 at
//! x = 1 and positive for x > 0. No true powers or roots are taken, so under
//! the reference laws every scale is exactly 1.

use xyl_core::certified::Encl;
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// The reference values, measured: alpha = 1/137.035999,
/// beta = 1/1836.15267, alpha_G = 5.906e-39, epsilon = 0.0071.
pub fn reference() -> [Q; 4] {
    let alpha_g = Q::from_i128(5906, 10i128.pow(42)).unwrap_or(Q::ZERO);
    [
        Q::ratio(1_000_000, 137_035_999),
        Q::ratio(100_000, 183_615_267),
        alpha_g,
        Q::ratio(71, 10_000),
    ]
}

/// A universe's laws: each constant over the reference, and whether space
/// can be folded at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Laws {
    /// Fine-structure constant alpha as a ratio to the reference (1 = the
    /// reference).
    pub alpha: Q,
    /// Electron-to-proton mass ratio beta as a ratio to the reference (1 = the
    /// reference).
    pub beta: Q,
    /// Gravitational coupling alpha_G as a ratio to the reference (1 = the
    /// reference).
    pub alpha_g: Q,
    /// Nuclear efficiency epsilon (rest-mass share released by fusion) as a ratio
    /// to the reference (1 = the reference).
    pub nuclear: Q,
    /// Whether space can be folded at all: where false, no people can fold
    /// space (`xyl_kernel::transit::Network::fold`).
    pub ftl: bool,
}

impl Default for Laws {
    fn default() -> Self {
        Laws {
            alpha: Q::ONE,
            beta: Q::ONE,
            alpha_g: Q::ONE,
            nuclear: Q::ONE,
            ftl: true,
        }
    }
}

/// The nominal power `x^(n/d)` of a coupling ratio, as an exact enclosure:
/// `x` or `1 / x` for n/d = 1 or -1, otherwise the share below. A zero
/// denominator or reciprocal gives zero.
fn pw(x: &Q, n: i64, d: i64) -> Encl {
    // The ratio `x` itself (n/d = 1) and its reciprocal (n/d = -1) are exact; any other
    // power p = |n|/d becomes the share (1 + p) x / (1 + p x), which is 1 at
    // x = 1, positive and saturating, and its reciprocal for negative n.
    // No true powers or roots of a coupling are taken.
    if d == 1 && n == 1 {
        return Encl::exact(x.clone());
    }
    if d == 1 && n == -1 {
        return Encl::exact(x.recip().unwrap_or(Q::ZERO));
    }
    let p = Q::ratio(n.abs(), d);
    let share = (&(&Q::ONE + &p) * x)
        .checked_div(&(&Q::ONE + &(&p * x)))
        .unwrap_or(Q::ZERO);
    Encl::exact(if n < 0 {
        share.recip().unwrap_or(Q::ZERO)
    } else {
        share
    })
}

fn prod(parts: &[Encl]) -> Encl {
    parts.iter().fold(Encl::exact(Q::ONE), |a, b| a.mul(b))
}

impl Laws {
    /// The four constants' values: each reference value times its ratio.
    pub fn values(&self) -> [Q; 4] {
        let o = reference();
        [
            &o[0] * &self.alpha,
            &o[1] * &self.beta,
            &o[2] * &self.alpha_g,
            &o[3] * &self.nuclear,
        ]
    }

    /// Energies and temperatures of chemistry: `alpha^2 beta alpha_G^(1/2)`.
    pub fn chemistry(&self) -> Encl {
        prod(&[
            pw(&self.alpha, 2, 1),
            pw(&self.beta, 1, 1),
            pw(&self.alpha_g, 1, 2),
        ])
    }

    /// A star's mass: the Chandrasekhar scale `alpha_G^(-1)` over the
    /// reference.
    pub fn star_mass(&self) -> Encl {
        pw(&self.alpha_g, -1, 1)
    }

    /// A star's radius: `1 / (alpha_G alpha^2)`.
    pub fn star_radius(&self) -> Encl {
        prod(&[pw(&self.alpha_g, -1, 1), pw(&self.alpha, -2, 1)])
    }

    /// Condensed matter's density: `alpha^3 beta^3 alpha_G^2`.
    pub fn density(&self) -> Encl {
        prod(&[
            pw(&self.alpha, 3, 1),
            pw(&self.beta, 3, 1),
            pw(&self.alpha_g, 2, 1),
        ])
    }
}

impl Laws {
    /// Condensed matter's pressures (phase diagrams):
    /// `alpha^5 beta^4 alpha_G^2`.
    pub fn pressure(&self) -> Encl {
        prod(&[
            pw(&self.alpha, 5, 1),
            pw(&self.beta, 4, 1),
            pw(&self.alpha_g, 2, 1),
        ])
    }

    /// Sizes of atoms and molecules: `1 / (alpha beta alpha_G^(1/2))`.
    pub fn atom(&self) -> Encl {
        prod(&[
            pw(&self.alpha, -1, 1),
            pw(&self.beta, -1, 1),
            pw(&self.alpha_g, -1, 2),
        ])
    }

    /// The proton's mass: `alpha_G^(1/2)` (k / m_H scales as its inverse).
    pub fn proton(&self) -> Encl {
        pw(&self.alpha_g, 1, 2)
    }

    /// The scales the heavy physical models run on, as dyadics (each the
    /// midpoint of its enclosure; exactly 1 under the reference laws).
    pub fn scales(&self) -> Scales {
        let dy = |e: Encl| Dy::from_q(&e.mid());
        Scales {
            chem: dy(self.chemistry()),
            pressure: dy(self.pressure()),
            atom: dy(self.atom()),
            proton: dy(self.proton()),
            density: dy(self.density()),
            star_mass: dy(self.star_mass()),
            star_radius: dy(self.star_radius()),
            nuclear: Dy::from_q(&self.nuclear),
            alpha: Dy::from_q(&self.alpha),
            ftl: self.ftl,
        }
    }
}

/// A universe's scales over the reference, as dyadics (from `Laws::scales`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scales {
    /// Chemical energy and temperature scale over the reference
    /// ([`Laws::chemistry`]).
    pub chem: Dy,
    /// Condensed-matter pressure scale over the reference ([`Laws::pressure`]).
    pub pressure: Dy,
    /// Atomic and molecular size scale over the reference ([`Laws::atom`]).
    pub atom: Dy,
    /// Proton mass scale over the reference ([`Laws::proton`]).
    pub proton: Dy,
    /// Condensed-matter density scale over the reference ([`Laws::density`]).
    pub density: Dy,
    /// Stellar mass scale over the reference ([`Laws::star_mass`]).
    pub star_mass: Dy,
    /// Stellar radius scale over the reference ([`Laws::star_radius`]).
    pub star_radius: Dy,
    /// Nuclear efficiency epsilon over the reference (the [`Laws::nuclear`]
    /// ratio).
    pub nuclear: Dy,
    /// Fine-structure constant alpha over the reference (the [`Laws::alpha`]
    /// ratio).
    pub alpha: Dy,
    /// Whether space can be folded at all ([`Laws::ftl`]).
    pub ftl: bool,
}

impl Default for Scales {
    fn default() -> Self {
        Laws::default().scales()
    }
}

impl Scales {
    /// An alpha decay's half-life over the reference, `exp(gamow (alpha - 1))`:
    /// the Gamow barrier (the log of half-life times attempt rate) is taken
    /// linear in alpha, with `gamow` its reference value.
    pub fn half_life(&self, gamow: Dy) -> Dy {
        (gamow * (self.alpha - Dy::ONE)).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_stay_positive_under_other_laws() {
        // Under laws far from the reference every scale stays positive (the
        // scales are shares of the couplings, not their powers).
        let l = Laws {
            alpha: Q::int(2),
            beta: Q::int(3),
            alpha_g: Q::int(5),
            nuclear: Q::int(7),
            ftl: true,
        };
        for s in [
            l.density(),
            l.chemistry(),
            l.pressure(),
            l.atom(),
            l.proton(),
        ] {
            assert!(s.lo.signum() > 0, "a scale under other laws: {:?}", s);
        }
    }

    #[test]
    fn stronger_gravity_makes_lighter_stars() {
        // Gravity four times stronger gives lighter stars (the Chandrasekhar
        // scale is the reciprocal of the coupling ratio).
        let l = Laws {
            alpha_g: Q::int(4),
            ..Laws::default()
        };
        let m = l.star_mass();
        assert!(m.lo.signum() > 0 && m.hi < Q::ONE, "star mass {:?}", m);
    }
}
