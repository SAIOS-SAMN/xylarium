//! The orbit reading (docs/model.md, The ring): an orbit read at a count of
//! its traversals as the axis dilation, held by its exponent, and the phases
//! of the five isoclinic planes, each exact in Q/Z, never one scalar.
//!
//! Inputs: an [`Orbit`]: the dilation per traversal on the axis plane (an
//! exact ratio, `D > 0`) and a rate per isoclinic plane in turns per
//! traversal (exact). [`Orbit::throat`] builds the core's orbit at the
//! throat from the spin `a` in `[0, 1]`: the torsion share
//! `s = (2 + a) / (4 + a)` on every plane (the spin is isoclinic) and the
//! dipole's dilation `D(n) = 1 + a / 2`, `n = 2s - 1 = a / (4 + a)`.
//!
//! Outputs: [`Orbit::at`], the [`Reading`] at a count `t` (exact in Q): the
//! axis exponent `t`, each plane's phase `t theta_i mod 1`, and the whole
//! traversals; [`Orbit::order`], the traversals after which a plane's phase
//! returns (the denominator of its rate); [`Orbit::twist`], the dipole's
//! twist `n`.
//!
//! Invariants: every value is exact; the axis dilation is never evaluated at
//! a count that is not whole (it is held by its exponent, [`Reading::axis`]
//! evaluates it only at a whole count); readings compose, `at(t1 + t2)` has
//! phases `at(t1) + at(t2) mod 1` and axis exponent `t1 + t2`. A duration
//! in light-time is not a quantity here.
//!
//! Terms: ring, twist, dipole (docs/glossary.md).

use crate::Q;

/// The isoclinic planes an orbit turns in.
pub const PLANES: usize = 5;

/// An orbit: its dilation per traversal on the axis plane and its rate on
/// each isoclinic plane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Orbit {
    /// The axis plane's dilation per traversal, exact, positive.
    pub dilation: Q,
    /// Each isoclinic plane's rate in turns per traversal, exact.
    pub rates: [Q; PLANES],
}

/// An orbit read at a count of its traversals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    /// The count read at, exact: the axis dilation's exponent.
    pub count: Q,
    /// The whole traversals made by the count.
    pub traversals: Q,
    /// Each isoclinic plane's phase, a share of a turn in `[0, 1)`.
    pub phases: [Q; PLANES],
}

impl Orbit {
    /// The core's orbit at the throat for the spin `a`: every plane at the
    /// torsion share `s = (2 + a) / (4 + a)`, the axis at the dipole's
    /// dilation `1 + a / 2`. `None` for a spin outside `[0, 1]`.
    pub fn throat(a: &Q) -> Option<Orbit> {
        if a.signum() < 0 || a > &Q::ONE {
            return None;
        }
        let s = (&Q::int(2) + a).checked_div(&(&Q::int(4) + a))?;
        let dilation = &Q::ONE + &a.checked_div(&Q::int(2))?;
        Some(Orbit {
            dilation,
            rates: std::array::from_fn(|_| s.clone()),
        })
    }

    /// The dipole's twist per echo, `n = (D - 1) / (D + 1)` from the axis
    /// dilation `D = (1 + n) / (1 - n)`.
    pub fn twist(&self) -> Option<Q> {
        (&self.dilation - &Q::ONE).checked_div(&(&self.dilation + &Q::ONE))
    }

    /// The reading at the count `t` (exact): the whole traversals, and each
    /// plane's phase `t theta_i mod 1`.
    pub fn at(&self, t: &Q) -> Reading {
        Reading {
            count: t.clone(),
            traversals: t - &t.frac(),
            phases: std::array::from_fn(|i| (t * &self.rates[i]).frac()),
        }
    }

    /// The traversals after which plane `i`'s phase returns: the
    /// denominator of its rate in lowest terms. `None` for a plane outside
    /// the five.
    pub fn order(&self, i: usize) -> Option<Q> {
        Some(Q::from_big(num_rational::BigRational::from_integer(
            self.rates.get(i)?.denom(),
        )))
    }
}

impl Reading {
    /// The axis dilation at a whole count, `D^t`, exact; `None` at a count
    /// that is not whole, where the dilation is held by its exponent.
    pub fn axis(&self, orbit: &Orbit) -> Option<Q> {
        if !self.count.frac().is_zero() {
            return None;
        }
        let k = i64::try_from(self.count.floor()).ok()?;
        let (base, k) = if k < 0 {
            (Q::ONE.checked_div(&orbit.dilation)?, -k)
        } else {
            (orbit.dilation.clone(), k)
        };
        Some((0..k).fold(Q::ONE, |acc, _| &acc * &base))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_throat_orbit_reads_the_twist_and_returns_by_its_rates() {
        // Spinless: every plane advances half a turn and returns after two.
        let o = Orbit::throat(&Q::ZERO).unwrap();
        assert_eq!(o.twist(), Some(Q::ZERO));
        assert_eq!(o.order(0), Some(Q::int(2)));
        assert_eq!(o.at(&Q::int(1)).phases[0], Q::ratio(1, 2));
        assert_eq!(o.at(&Q::int(2)).phases[0], Q::ZERO);
        // The extremal spin: s = 3/5, the dipole's dilation 3/2, twist 1/5.
        let x = Orbit::throat(&Q::ONE).unwrap();
        assert_eq!(x.rates[0], Q::ratio(3, 5));
        assert_eq!(x.dilation, Q::ratio(3, 2));
        assert_eq!(x.twist(), Some(Q::ratio(1, 5)));
        assert_eq!(x.order(4), Some(Q::int(5)));
        assert!(Orbit::throat(&Q::int(2)).is_none());
    }

    #[test]
    fn readings_compose_and_the_axis_is_held_by_its_exponent() {
        let o = Orbit::throat(&Q::ratio(1, 3)).unwrap();
        let (t1, t2) = (Q::ratio(7, 2), Q::ratio(19, 5));
        let (a, b, ab) = (o.at(&t1), o.at(&t2), o.at(&(&t1 + &t2)));
        for i in 0..PLANES {
            assert_eq!(ab.phases[i], (&a.phases[i] + &b.phases[i]).frac());
        }
        assert_eq!(ab.count, &t1 + &t2);
        // Not evaluated at a fractional count; exact at a whole one, and the
        // dilation at -k is the inverse of the one at k.
        assert!(a.axis(&o).is_none());
        let (up, down) = (o.at(&Q::int(3)), o.at(&Q::int(-3)));
        assert_eq!(&up.axis(&o).unwrap() * &down.axis(&o).unwrap(), Q::ONE);
        assert_eq!(up.traversals, Q::int(3));
    }
}
