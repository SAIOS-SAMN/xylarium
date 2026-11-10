//! The share laws every tier composes (docs/model.md, The founding as flow):
//! the deficit share, the dilation and the flow law.
//!
//! Inputs: exact rationals ([`Q`]) or dyadic readings ([`Dy`]); a hold and a
//! reach in one unit, or a push (a share of the turn of what reaches a hold).
//!
//! - [`faces`]: gravity's two faces about a hold `m` at reach `r` (docs/model.md,
//!   Gravity: the deformation about a mass invariant): the convergent face
//!   `c = m / (r + m)` and the divergent face `d = r / (r + m)`, `c + d = 1`.
//!   With the pole coordinate `t = r / m`, `c = 1 / (1 + t)` and
//!   `d = t / (1 + t)`; the antipole, the inversion `t -> 1 / t`, exchanges
//!   them; their imbalance `p = c - d` is a push with `D(p) = c / d = m / r`;
//!   at the equator `r = m`, the inversion's fixed circle, they balance at
//!   one half.
//! - [`deficit`]: the convergent face, `m / (r + m)`: the share a hold keeps
//!   of what reaches it.
//! - [`dilation`]: the dilation per echo of a push `p`, `D(p) = (1 + p) / (1 -
//!   p)`, for `-1 < p < 1`; `D(-p) = 1 / D(p)`.
//! - [`flow`]: the composition of two pushes, `x (+) y = (x + y) / (1 + x
//!   y)` (two opposed pushes compose as `x (+) (-y)`). The dilation turns
//!   the flow law into a product, `D(x (+) y) = D(x) D(y)`.
//!
//! Invariants: every value is exact on `Q`; a push outside `(-1, 1)` and a
//! deficit with a negative hold or reach, or with both zero, are refused
//! (`None`). The `Dy` forms ([`deficit_dy`], [`dilation_dy`]) read the same
//! laws on readings: a non-positive hold holds nothing, and a negative reach
//! is read at the throat.
//!
//! Terms: deficit share, dilation, flow law (docs/glossary.md).

use crate::dyadic::Dy;
use crate::Q;

/// Gravity's two faces about a mass invariant: the convergent (held) face and
/// the divergent (passed) face, antipoles under the inversion `t -> 1 / t`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Faces {
    /// The convergent face, `m / (r + m)`: the deficit.
    pub convergent: Q,
    /// The divergent face, `r / (r + m)`.
    pub divergent: Q,
}

impl Faces {
    /// The pole coordinate `t = r / m = divergent / convergent`; `None` where
    /// the convergent face is zero (no hold).
    pub fn pole(&self) -> Option<Q> {
        self.divergent.checked_div(&self.convergent)
    }

    /// The faces' imbalance `p = c - d = (1 - t) / (1 + t)`, a push in
    /// `[-1, 1]`: positive within the equator `r = m`, zero on it, negative
    /// beyond.
    /// Its dilation is `c / d = m / r`.
    pub fn push(&self) -> Q {
        &self.convergent - &self.divergent
    }

    /// The antipole: the inversion `t -> 1 / t`, which exchanges the faces.
    pub fn antipole(&self) -> Faces {
        Faces {
            convergent: self.divergent.clone(),
            divergent: self.convergent.clone(),
        }
    }
}

/// Gravity's two faces about a hold `m` at reach `r` (one calibre for both):
/// `m / (r + m)` and `r / (r + m)`. `None` for a negative hold or reach, or
/// where both are zero.
pub fn faces(m: &Q, r: &Q) -> Option<Faces> {
    if m.signum() < 0 || r.signum() < 0 {
        return None;
    }
    let whole = r + m;
    Some(Faces {
        convergent: m.checked_div(&whole)?,
        divergent: r.checked_div(&whole)?,
    })
}

/// The deficit share `m / (r + m)` of a hold `m` at reach `r` (one unit for
/// both): gravity's convergent face ([`faces`]). `None` for a negative hold
/// or reach, or where both are zero.
pub fn deficit(m: &Q, r: &Q) -> Option<Q> {
    faces(m, r).map(|f| f.convergent)
}

/// The dilation per echo of a push `p`: `(1 + p) / (1 - p)`. `None` for `p`
/// outside `(-1, 1)`.
pub fn dilation(p: &Q) -> Option<Q> {
    if p.abs() >= Q::ONE {
        return None;
    }
    (&Q::ONE + p).checked_div(&(&Q::ONE - p))
}

/// Two pushes composed: `(x + y) / (1 + x y)`. `None` for either push
/// outside `(-1, 1)`.
pub fn flow(x: &Q, y: &Q) -> Option<Q> {
    if x.abs() >= Q::ONE || y.abs() >= Q::ONE {
        return None;
    }
    (x + y).checked_div(&(&Q::ONE + &(x * y)))
}

/// [`deficit`] on readings: `m / (r + m)`, zero for a non-positive hold, and
/// a negative reach read at the throat (zero).
pub fn deficit_dy(m: Dy, r: Dy) -> Dy {
    if m.signum() <= 0 {
        return Dy::ZERO;
    }
    m / (r.max(Dy::ZERO) + m)
}

/// [`dilation`] on readings. `None` for `p` outside `(-1, 1)`.
pub fn dilation_dy(p: Dy) -> Option<Dy> {
    if p.abs() >= Dy::ONE {
        return None;
    }
    Some((Dy::ONE + p) / (Dy::ONE - p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_faces_dilation_is_linear_in_its_pole() {
        for t in [
            Q::ratio(1, 9),
            Q::ratio(1, 1),
            Q::ratio(3, 2),
            Q::ratio(7, 1),
        ] {
            let f = faces(&Q::ONE, &t).unwrap();
            // D(d) = 1 + 2t, D(c) = 1 + 2 / t, D(d - c) = t: no square, no root.
            let two = Q::int(2);
            assert_eq!(dilation(&f.divergent), Some(&Q::ONE + &(&two * &t)));
            assert_eq!(
                dilation(&f.convergent),
                Some(&Q::ONE + &two.checked_div(&t).unwrap())
            );
            assert_eq!(dilation(&-f.push()), Some(t.clone()));
        }
    }

    #[test]
    fn the_faces_are_antipoles_under_the_inversion() {
        let cases = [
            (Q::ratio(1, 1), Q::ratio(3, 1)),
            (Q::ratio(5, 2), Q::ratio(1, 7)),
            (Q::ratio(2, 3), Q::ratio(2, 3)),
            (Q::ratio(9, 4), Q::ratio(11, 5)),
        ];
        for (m, r) in &cases {
            let f = faces(m, r).unwrap();
            // The faces close: c + d = 1, and c / d = m / r = 1 / t.
            assert_eq!(&f.convergent + &f.divergent, Q::ONE);
            assert_eq!(f.pole(), r.checked_div(m));
            // The inversion r -> m (m / r), t -> 1 / t, exchanges the faces.
            let image = m * &m.checked_div(r).unwrap();
            assert_eq!(faces(m, &image).unwrap(), f.antipole());
            // The imbalance is a push whose dilation is c / d, reversed by
            // the antipole.
            let p = f.push();
            assert_eq!(f.antipole().push(), -p.clone());
            if p.abs() < Q::ONE {
                assert_eq!(dilation(&p), f.convergent.checked_div(&f.divergent));
            }
            // A change of calibre dilates hold and reach together and keeps
            // both faces.
            let l = Q::ratio(60, 1);
            assert_eq!(faces(&(m * &l), &(r * &l)).unwrap(), f);
        }
        // On the equator r = m the faces balance and the antipole fixes them.
        let throat = faces(&Q::ratio(3, 1), &Q::ratio(3, 1)).unwrap();
        assert_eq!(throat.push(), Q::ZERO);
        assert_eq!(throat.antipole(), throat);
        assert_eq!(faces(&Q::ZERO, &Q::ZERO), None);
    }

    #[test]
    fn the_deficit_is_one_half_where_reach_and_hold_coincide() {
        assert_eq!(
            deficit(&Q::ratio(3, 1), &Q::ratio(3, 1)),
            Some(Q::ratio(1, 2))
        );
        assert_eq!(deficit(&Q::ZERO, &Q::ratio(4, 1)), Some(Q::ZERO));
        assert_eq!(deficit(&Q::ZERO, &Q::ZERO), None);
        assert_eq!(deficit(&Q::ratio(-1, 1), &Q::ratio(1, 1)), None);
    }

    #[test]
    fn the_dilation_turns_the_flow_law_into_a_product() {
        let pushes = [
            Q::ratio(0, 1),
            Q::ratio(1, 2),
            Q::ratio(1, 3),
            Q::ratio(-1, 4),
            Q::ratio(5, 7),
            Q::ratio(-2, 3),
        ];
        for x in &pushes {
            for y in &pushes {
                let xy = flow(x, y).unwrap();
                assert_eq!(
                    dilation(&xy),
                    Some(&dilation(x).unwrap() * &dilation(y).unwrap())
                );
            }
            assert_eq!(
                &dilation(x).unwrap() * &dilation(&-x.clone()).unwrap(),
                Q::ONE
            );
        }
        assert_eq!(dilation(&Q::ONE), None);
        assert_eq!(flow(&Q::ratio(1, 1), &Q::ratio(1, 2)), None);
    }

    #[test]
    fn opposed_deficits_compose_to_the_hold_difference() {
        // The continuum's push against the vacuum's is the hold difference
        // over the reach and both holds.
        for (h, m, r) in [
            (Q::ratio(3, 1), Q::ratio(1, 1), Q::ratio(5, 1)),
            (Q::ratio(1, 2), Q::ratio(2, 1), Q::ratio(7, 3)),
        ] {
            let (p_c, p_v) = (deficit(&h, &r).unwrap(), deficit(&m, &r).unwrap());
            let net = flow(&p_c, &-p_v).unwrap();
            assert_eq!(net, (&h - &m).checked_div(&(&(&r + &h) + &m)).unwrap());
        }
    }

    /// The automorphism of the chain at the law (docs/model.md, The
    /// automorphic, synaptic chain): the deficit is of degree zero in length,
    /// so a hold and a reach carried together by any dilation `lambda > 0`
    /// give the same share, exactly, and the pushes it gives compose the same
    /// way by the flow law. The kernel's tier ladder (`units::Tier`, a
    /// declared ruler) is one instance of `lambda`.
    #[test]
    fn deficit_is_invariant_under_dilation() {
        use crate::units::Tier;
        let mut lambdas = vec![Q::ratio(7, 3), Q::ratio(1, 11), Q::ratio(1000003, 2)];
        lambdas.extend(
            [
                Tier::Universe,
                Tier::Cluster,
                Tier::Galaxy,
                Tier::System,
                Tier::Planet,
            ]
            .map(Tier::in_rg),
        );
        let pairs = [
            (Q::ratio(1, 1), Q::ratio(3, 1)),
            (Q::ratio(2, 7), Q::ratio(5, 3)),
            (Q::ratio(9, 4), Q::ratio(1, 60)),
        ];
        for (m, r) in &pairs {
            let (held, other) = (deficit(m, r).unwrap(), deficit(r, m).unwrap());
            for k in &lambdas {
                let (mk, rk) = (m * k, r * k);
                assert_eq!(deficit(&mk, &rk).unwrap(), held);
                let carried = flow(&deficit(&mk, &rk).unwrap(), &deficit(&rk, &mk).unwrap());
                assert_eq!(carried, flow(&held, &other));
            }
        }
    }
}
