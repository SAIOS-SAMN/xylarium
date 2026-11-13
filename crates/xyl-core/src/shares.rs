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
//! - [`FacePairs`]: a dilation held factored, a product of integer face
//!   pairs `prod (c_m / d_m)` ([#210](https://github.com/SAIOS-SAMN/xylarium/issues/210)).
//!   A composition appends the other's pairs and the inversion
//!   (`p -> -p`, `t -> 1 / t`) swaps each pair, so every factor keeps the
//!   counts it came from: no expansion, no gcd, no cut. Exact without
//!   evaluation: the balance (`prod c = prod d`) and a comparison, read
//!   after the factors two products share cancel. [`FacePairs::value`] is
//!   the reading, the product formed once.
//! - [`Parcel`]: an amount held at a dilation (face pairs); its antipole
//!   about a hold `H` is `H / D`, the pairs swapped and composed with `H`.
//!   [`weighted`] reads parcels' amount-weighted dilation
//!   `sum(a D) / sum(a)` as an enclosure on the `2^-bits` grid: the sum is
//!   formed only at a reading, at the reader's precision.
//!
//! Invariants: every value is exact on `Q`; a push outside `(-1, 1)` and a
//! deficit with a negative hold or reach, or with both zero, are refused
//! (`None`). The `Dy` forms ([`deficit_dy`], [`dilation_dy`]) read the same
//! laws on readings: a non-positive hold holds nothing, and a negative reach
//! is read at the throat.
//!
//! Terms: deficit share, dilation, flow law (docs/glossary.md).

use crate::certified::Encl;
use crate::dyadic::Dy;
use crate::Q;
use num_bigint::BigInt;
use std::cmp::Ordering;

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

/// A dilation held factored: the product of integer face pairs
/// `prod (c_m / d_m)`, every count positive. Each pair keeps the counts it
/// came from; nothing is expanded or reduced until [`FacePairs::value`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FacePairs {
    pairs: Vec<(BigInt, BigInt)>,
}

impl FacePairs {
    /// The empty product, the dilation 1 (no push).
    pub fn one() -> FacePairs {
        FacePairs { pairs: Vec::new() }
    }

    /// One pair `c / d`; `None` unless both counts are positive.
    pub fn pair(c: BigInt, d: BigInt) -> Option<FacePairs> {
        let zero = BigInt::from(0);
        (c > zero && d > zero).then(|| FacePairs {
            pairs: vec![(c, d)],
        })
    }

    /// A positive rational as its one pair, numerator over denominator;
    /// `None` for zero or a negative.
    pub fn of(q: &Q) -> Option<FacePairs> {
        FacePairs::pair(q.numer(), q.denom())
    }

    /// The dilation of a push `p = n / d`: the pair `(d + n, d - n)`, which
    /// is `(1 + p) / (1 - p)`. `None` for `p` outside `(-1, 1)`.
    pub fn dilation(p: &Q) -> Option<FacePairs> {
        let (n, d) = (p.numer(), p.denom());
        FacePairs::pair(&d + &n, &d - &n)
    }

    /// The composition: the other's pairs appended. For two dilations this
    /// is the flow law, `D(x (+) y) = D(x) D(y)`.
    pub fn compose(&self, other: &FacePairs) -> FacePairs {
        let mut pairs = self.pairs.clone();
        pairs.extend(other.pairs.iter().cloned());
        FacePairs { pairs }
    }

    /// The inversion `p -> -p` (`t -> 1 / t`): every pair swapped.
    pub fn inverse(&self) -> FacePairs {
        FacePairs {
            pairs: self
                .pairs
                .iter()
                .map(|(c, d)| (d.clone(), c.clone()))
                .collect(),
        }
    }

    /// The pairs, in the order they were composed.
    pub fn pairs(&self) -> &[(BigInt, BigInt)] {
        &self.pairs
    }

    /// This product against `other`: the pairs the two share cancel, and the
    /// remaining counts are compared crosswise, `prod c_a prod d_b` against
    /// `prod c_b prod d_a`. Exact; nothing is divided.
    pub fn cmp_value(&self, other: &FacePairs) -> Ordering {
        let mut theirs: Vec<Option<&(BigInt, BigInt)>> = other.pairs.iter().map(Some).collect();
        let mut mine: Vec<&(BigInt, BigInt)> = Vec::new();
        for p in &self.pairs {
            match theirs.iter_mut().find(|q| q.is_some_and(|q| q == p)) {
                Some(slot) => *slot = None,
                None => mine.push(p),
            }
        }
        let one = BigInt::from(1);
        let (mut left, mut right) = (one.clone(), one);
        for (c, d) in mine {
            left *= c;
            right *= d;
        }
        for (c, d) in theirs.into_iter().flatten() {
            left *= d;
            right *= c;
        }
        left.cmp(&right)
    }

    /// Whether the product balances, `prod c = prod d`: the dilation 1, the
    /// throat's balance.
    pub fn balanced(&self) -> bool {
        self.cmp_value(&FacePairs::one()) == Ordering::Equal
    }

    /// The reading: the product formed once, `prod c / prod d`, exact.
    pub fn value(&self) -> Q {
        let one = BigInt::from(1);
        let (mut c, mut d) = (one.clone(), one);
        for (cm, dm) in &self.pairs {
            c *= cm;
            d *= dm;
        }
        // Every count is positive, so the denominator is never zero.
        Q::from_big(num_rational::BigRational::new(c, d))
    }
}

/// An amount held at a dilation: the dilation as face pairs, the amount in
/// the unit of whatever the parcel holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parcel {
    /// The amount held.
    pub amount: Q,
    /// The dilation it is held at.
    pub pairs: FacePairs,
}

impl Parcel {
    /// The parcel at the antipole of its dilation about `hold`: `hold / D`,
    /// the pairs swapped and composed with the hold, the amount kept.
    /// `None` for a hold that is not positive.
    pub fn antipole(&self, hold: &Q) -> Option<Parcel> {
        Some(Parcel {
            amount: self.amount.clone(),
            pairs: FacePairs::of(hold)?.compose(&self.pairs.inverse()),
        })
    }
}

/// Parcels' amount-weighted dilation `sum(a D) / sum(a)`, enclosed on the
/// `2^-bits` grid: each term and the amount are read outward at `bits` and
/// the sum is formed on those readings, so the enclosure holds the exact
/// value without the exact sum being formed. The mean does not depend on
/// the amounts' scale, so each amount is read as its share of the parcels'
/// whole amount, formed here at the reading, and a kind held in trace reads
/// as well as one held in bulk. `None` where the parcels hold no positive
/// amount.
pub fn weighted(parcels: &[Parcel], bits: u32) -> Option<Encl> {
    let held: Vec<&Parcel> = parcels.iter().filter(|p| p.amount.signum() > 0).collect();
    let whole = held.iter().fold(Q::ZERO, |w, p| &w + &p.amount);
    let zero = Encl::exact(Q::ZERO);
    let (moment, amount) = held.iter().try_fold((zero.clone(), zero), |(m, a), p| {
        let share = p.amount.checked_div(&whole)?;
        let term = Encl::exact(&share * &p.pairs.value()).outward(bits);
        Some((m.add(&term), a.add(&Encl::exact(share).outward(bits))))
    })?;
    Some(moment.mul(&amount.recip()?).outward(bits))
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

/// [`deficit`] on readings: `m / (r + m)`, zero for a non-positive hold (a
/// hold of nothing holds no share), and a negative reach read at the throat
/// (zero).
pub fn deficit_dy(m: Dy, r: Dy) -> Dy {
    if m.signum() <= 0 {
        return Dy::ZERO;
    }
    // With m positive and the reach at least zero, `r + m` is positive.
    m.div(&(r.max(Dy::ZERO) + m)).map_or(Dy::ZERO, |d| d)
}

/// [`dilation`] on readings. `None` for `p` outside `(-1, 1)`.
pub fn dilation_dy(p: Dy) -> Option<Dy> {
    if p.abs() >= Dy::ONE {
        return None;
    }
    // Stand-in S115: Callers pass the gross push, not the net of its resistance.
    (Dy::ONE + p).div(&(Dy::ONE - p))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One composition of face pairs: its reading is the flow law's, its
    /// inversion composed with it balances, and a product compared with
    /// itself extended by one pair orders as that pair's own faces.
    #[test]
    fn face_pairs_compose_as_the_flow_law() {
        let pushes = [
            Q::new(1, 6).unwrap(),
            Q::new(-2, 7).unwrap(),
            Q::new(5, 11).unwrap(),
        ];
        let chain = pushes.iter().fold(FacePairs::one(), |f, p| {
            f.compose(&FacePairs::dilation(p).unwrap())
        });
        let pushed = pushes[1..]
            .iter()
            .fold(pushes[0].clone(), |x, y| flow(&x, y).unwrap());
        assert_eq!(chain.value(), dilation(&pushed).unwrap());
        assert!(chain.compose(&chain.inverse()).balanced());
        for (c, d) in chain.pairs() {
            let one = FacePairs::pair(c.clone(), d.clone()).unwrap();
            assert_eq!(chain.compose(&one).cmp_value(&chain), c.cmp(d));
        }
        assert!(FacePairs::dilation(&Q::ONE).is_none());
        assert!(FacePairs::of(&Q::ZERO).is_none());
    }

    /// LAW-TRAVERSAL's joint-read fixtures 1 to 4 (mycelium 78.176 section
    /// 4), read on gravity's faces with the hold `z` and the reach `q` of a
    /// plane (78.172 section 1): the antipole is the antipolar exchange.
    /// (1) Out and back is the identity on the pole. (2) The whole is kept:
    /// `(2/3, 1/3)` exchanges to `(1/3, 2/3)`, both summing to 1. (3) The
    /// asymmetry and the fraction invert, `sigma -> -sigma`, `o -> 1 - o`:
    /// at `(2/3, 1/3)`, `sigma = 1/3 -> -1/3` and `o = 1/3 -> 2/3`. (4) The
    /// throat `z = q` is the only fixed point.
    #[test]
    fn the_antipolar_traversal_fixtures_hold() {
        let third = Q::new(1, 3).unwrap();
        let two_thirds = Q::new(2, 3).unwrap();
        let pairs = [
            (two_thirds.clone(), third.clone()),
            (Q::ONE, Q::ONE),
            (Q::new(1, 7).unwrap(), Q::new(5, 2).unwrap()),
            (Q::int(3), Q::new(1, 60).unwrap()),
        ];
        for (z, q) in &pairs {
            let f = faces(z, q).unwrap();
            let back = f.antipole();
            // (1) Out and back is the identity.
            assert_eq!(back.antipole(), f);
            assert_eq!(&back.pole().unwrap() * &f.pole().unwrap(), Q::ONE);
            // (2) The whole is kept, as the exchanged pair (q, z).
            assert_eq!(faces(q, z).unwrap(), back);
            assert_eq!(&back.convergent + &back.divergent, Q::ONE);
            // (3) The asymmetry and the fraction invert.
            assert_eq!(back.push(), -f.push());
            assert_eq!(back.divergent, &Q::ONE - &f.divergent);
            // (4) Fixed only at the throat.
            assert_eq!(back == f, z == q, "z {z} q {q}");
        }
        // The share split at delta = H / (R + H) (joint read with
        // netstratum, carrier fixture 2): (1, 1) splits 1/2, 1/2; (3, 5)
        // splits 3/8, 5/8 and inverts to 5/8, 3/8.
        let half = Q::new(1, 2).unwrap();
        let even = faces(&Q::ONE, &Q::ONE).unwrap();
        assert_eq!((even.convergent, even.divergent), (half.clone(), half));
        let split = faces(&Q::int(3), &Q::int(5)).unwrap();
        assert_eq!(split.convergent, Q::new(3, 8).unwrap());
        assert_eq!(split.antipole().convergent, Q::new(5, 8).unwrap());
        let f = faces(&two_thirds, &third).unwrap();
        assert_eq!(
            (f.push(), f.divergent.clone()),
            (third.clone(), third.clone())
        );
        let back = f.antipole();
        assert_eq!((back.push(), back.divergent), (-&third, two_thirds));
    }

    /// The weighted reading does not depend on the amounts' scale: a
    /// parcel held at 2^-300 reads its dilation as one held at 1.
    #[test]
    fn a_trace_parcel_reads_as_well_as_a_bulk_one() {
        let pairs = FacePairs::pair(BigInt::from(7), BigInt::from(5)).unwrap();
        let tiny = (0..300).fold(Q::ONE, |q, _| q.checked_div(&Q::int(2)).unwrap());
        for amount in [Q::ONE, tiny] {
            let e = weighted(
                &[Parcel {
                    amount,
                    pairs: pairs.clone(),
                }],
                64,
            )
            .unwrap();
            assert!(e.lo <= Q::new(7, 5).unwrap() && Q::new(7, 5).unwrap() <= e.hi);
        }
    }

    /// A parcel's antipole times its dilation is the hold, and the
    /// weighted reading encloses the exact weighted dilation and narrows as
    /// the precision rises.
    #[test]
    fn parcels_read_their_weighted_dilation() {
        let parcels: Vec<Parcel> = [(1, 6), (-2, 7), (5, 11)]
            .iter()
            .enumerate()
            .map(|(k, (n, d))| Parcel {
                amount: Q::new(i64::try_from(k).unwrap() + 1, 3).unwrap(),
                pairs: FacePairs::dilation(&Q::new(*n, *d).unwrap()).unwrap(),
            })
            .collect();
        let hold = Q::new(5, 2).unwrap();
        for p in &parcels {
            let back = p.antipole(&hold).unwrap();
            assert_eq!(&back.pairs.value() * &p.pairs.value(), hold);
        }
        let exact = parcels
            .iter()
            .fold(Q::ZERO, |m, p| &m + &(&p.amount * &p.pairs.value()))
            .checked_div(&parcels.iter().fold(Q::ZERO, |a, p| &a + &p.amount))
            .unwrap();
        let (coarse, fine) = (
            weighted(&parcels, 16).unwrap(),
            weighted(&parcels, 64).unwrap(),
        );
        assert!(coarse.contains(&exact) && fine.contains(&exact));
        assert!(&fine.hi - &fine.lo <= &coarse.hi - &coarse.lo);
        assert!(weighted(&[], 64).is_none());
    }

    #[test]
    fn a_faces_dilation_is_linear_in_its_pole() {
        for t in [
            crate::ratio!(1, 9),
            crate::ratio!(1, 1),
            crate::ratio!(3, 2),
            crate::ratio!(7, 1),
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
            (crate::ratio!(1, 1), crate::ratio!(3, 1)),
            (crate::ratio!(5, 2), crate::ratio!(1, 7)),
            (crate::ratio!(2, 3), crate::ratio!(2, 3)),
            (crate::ratio!(9, 4), crate::ratio!(11, 5)),
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
            let l = crate::ratio!(60, 1);
            assert_eq!(faces(&(m * &l), &(r * &l)).unwrap(), f);
        }
        // On the equator r = m the faces balance and the antipole fixes them.
        let throat = faces(&crate::ratio!(3, 1), &crate::ratio!(3, 1)).unwrap();
        assert_eq!(throat.push(), Q::ZERO);
        assert_eq!(throat.antipole(), throat);
        assert_eq!(faces(&Q::ZERO, &Q::ZERO), None);
    }

    #[test]
    fn the_deficit_is_one_half_where_reach_and_hold_coincide() {
        assert_eq!(
            deficit(&crate::ratio!(3, 1), &crate::ratio!(3, 1)),
            Some(crate::ratio!(1, 2))
        );
        assert_eq!(deficit(&Q::ZERO, &crate::ratio!(4, 1)), Some(Q::ZERO));
        assert_eq!(deficit(&Q::ZERO, &Q::ZERO), None);
        assert_eq!(deficit(&crate::ratio!(-1, 1), &crate::ratio!(1, 1)), None);
    }

    #[test]
    fn the_dilation_turns_the_flow_law_into_a_product() {
        let pushes = [
            crate::ratio!(0, 1),
            crate::ratio!(1, 2),
            crate::ratio!(1, 3),
            crate::ratio!(-1, 4),
            crate::ratio!(5, 7),
            crate::ratio!(-2, 3),
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
        assert_eq!(flow(&crate::ratio!(1, 1), &crate::ratio!(1, 2)), None);
    }

    #[test]
    fn opposed_deficits_compose_to_the_hold_difference() {
        // The continuum's push against the vacuum's is the hold difference
        // over the reach and both holds.
        for (h, m, r) in [
            (
                crate::ratio!(3, 1),
                crate::ratio!(1, 1),
                crate::ratio!(5, 1),
            ),
            (
                crate::ratio!(1, 2),
                crate::ratio!(2, 1),
                crate::ratio!(7, 3),
            ),
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
        let mut lambdas = vec![
            crate::ratio!(7, 3),
            crate::ratio!(1, 11),
            crate::ratio!(1000003, 2),
        ];
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
            (crate::ratio!(1, 1), crate::ratio!(3, 1)),
            (crate::ratio!(2, 7), crate::ratio!(5, 3)),
            (crate::ratio!(9, 4), crate::ratio!(1, 60)),
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
