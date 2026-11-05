//! The share laws every tier composes (docs/model.md, The founding as flow):
//! the deficit share, the dilation and the flow law.
//!
//! Inputs: exact rationals ([`Q`]) or dyadic readings ([`Dy`]); a hold and a
//! reach in one unit, or a push (a share of the turn of what reaches a hold).
//!
//! - [`deficit`]: the share a hold `m` keeps of what reaches it at reach `r`,
//!   `m / (r + m)`; at `r = m` it is one half, the throat's push.
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

/// The deficit share `m / (r + m)` of a hold `m` at reach `r` (one unit for
/// both). `None` for a negative hold or reach, or where both are zero.
pub fn deficit(m: &Q, r: &Q) -> Option<Q> {
    if m.signum() < 0 || r.signum() < 0 {
        return None;
    }
    m.checked_div(&(r + m))
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
