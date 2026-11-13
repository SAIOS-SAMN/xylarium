//! The Hopf carrier: the circle fibration `S^1 -> S^11 -> CP^5` on the six
//! planes of `C^6`, its weighted (Seifert) turning, the rational overlap of
//! two fibres and their linking (#183).
//!
//! Inputs: a plane index `p` in `0..PLANES` (plane 0 the axis plane, planes
//! 1 to 5 the isoclinic planes); a winding `w` in `Z^6`, one integer per
//! plane; an angle in degrees ([`Q`]); a fibre's plane shares (six
//! non-negative rationals summing to one) and its phases in degrees.
//!
//! Layout: the model numbers the twelve base axes from 0, plane `p` on axes
//! `2p` and `2p + 1`; `cga` counts base axes from 1, so plane `p` is
//! `cga` axes `2p + 1` and `2p + 2` ([`plane`]). The axis plane's first axis
//! is the pole axis, where the Z-Q stretch is read.
//!
//! Outputs:
//! - [`turn`]: the rotor `z_p -> e^{i w_p theta} z_p` on every plane, a
//!   [`Rotor`] of the twelve axes. With the common winding [`COMMON`] (one on
//!   every plane, the axis plane included) it is the Hopf action, free on
//!   `S^11`: no non-zero vector is fixed by a turn short of a whole turn.
//!   Any other winding is the weighted Seifert action of the per-plane twist
//!   (#128).
//! - [`plane_shares`]: a weight `m` in `Z^6` read as plane shares
//!   `|m_p| / l`, `l = sum |m_p|`.
//! - [`arc`]: the shorter arc of a phase difference, in `[0, 180]` degrees.
//! - [`overlap`]: two fibres' held and passed faces, `c = sum_p min(s_p(i),
//!   s_p(j)) (1 - arc_p / 180)` and `d = 1 - c`, read as
//!   [`shares::Faces`](crate::shares::Faces): exact, with no root and no
//!   arccosine (Fubini-Study's angle is not taken).
//! - [`linking`]: the axis-isoclinic linking of two fibres,
//!   `Lk = sum_{q=1..5} (w_i0 w_jq + w_j0 w_iq)`, from the plane-pair
//!   products on the 3-spheres `S^3_{0,q}` ([`pair_linking`]); the axis
//!   circle and the isoclinic 9-sphere of the join `S^11 = S^1 * S^9` link
//!   once ([`STRATA_LINKING`]).
//!
//! - The charged spin (ruled 2026-11-11, #183): the spin is the manifold's
//!   holonomy at the body's reach, `theta = 360 delta` degrees per traversal
//!   ([`holonomy`]), on the six planes in the common phase; the body's open
//!   charge `pi = (p / w) (1 - delta)` ([`open_charge`], LAW-COUPLING v2's
//!   push on xylarium's orientation, `g = delta`); the axis tilt, a turn of
//!   the pole axis toward the open charge's isoclinic plane ([`tilt`]),
//!   whose faces are read off the motion as `c = 1 - eps / 90`,
//!   `d = eps / 90` ([`tilt_faces_l1`]); its natural balance, where the tilt's
//!   faces equal the charge's, `eps* = 90 pi` degrees
//!   ([`tilt_equilibrium`]); and the charged spin
//!   `K = delta (T B_0 T~ + sum_{q=1..5} B_q)` ([`charged_spin`]), its
//!   magnitude the holonomy as a share of the turn. A complex-linear mixing
//!   of planes leaves the common phase unchanged, so only the tilt of one
//!   generator deforms it.
//!
//! Invariants: every value is exact. A plane index past five, a weight with
//! `l = 0` (one state, which overlaps itself wholly), shares that are negative
//! or do not sum to one, and a linking past `i64` are refused (`None`).
//!
//! Terms: Hopf carrier, overlap share (docs/glossary.md).

use crate::rotor::{Rotor, Turn};
use crate::shares::Faces;
use crate::Q;

/// The planes of `C^6`: the axis plane and the five isoclinic planes.
pub const PLANES: usize = 6;

/// The base axes of `C^6`, two per plane.
pub const AXES: usize = 2 * PLANES;

/// The common winding: one on every plane, the axis plane included; its
/// turning is the Hopf action.
pub const COMMON: [i64; PLANES] = [1; PLANES];

/// The holonomy at a reach whose held face is `delta`: `360 delta` degrees
/// per traversal, the spin's magnitude (#183). `None` outside `[0, 1]`.
pub fn holonomy(delta: &Q) -> Option<Q> {
    within_share(delta).then(|| delta * &Q::int(360))
}

/// The open charge of a body whose proton share is `p / w` (protons over
/// nucleons) and whose held face is `delta`: `pi = (p / w) (1 - delta)`,
/// LAW-COUPLING v2's push on xylarium's orientation (protons held,
/// `g = delta`). `None` for a share or `delta` outside `[0, 1]`.
pub fn open_charge(proton_share: &Q, delta: &Q) -> Option<Q> {
    if !within_share(proton_share) || !within_share(delta) {
        return None;
    }
    // Stand-in S124: The rate this share drives is not offset by the medium.
    Some(proton_share * &(&Q::ONE - delta))
}

/// The axis tilt by `degrees`: the pole axis (plane 0's first axis) turned
/// toward isoclinic plane 1's first axis, the plane the open charge pushes
/// on. A turn of one generator, not complex linear, so it deforms the
/// common phase. `None` where the gridded half-angle is the zero direction.
pub fn tilt(degrees: &Q, bits: u32) -> Option<Rotor<AXES>> {
    let (pole, _) = plane(0)?;
    let (toward, _) = plane(1)?;
    let turn = Turn {
        from: pole,
        toward,
        degrees: degrees.clone(),
    };
    Rotor::of(&turn, bits)
}

/// The tilt's faces, read off the motion, an L1 reading of the round angle
/// (named as one, LAW-COUPLING 78.175's shared degrees): the overlap of the
/// tilted axis plane with the upright one, its arc on the share circle twice the tilt
/// (`psi = 2 eps`), so `c = 1 - eps / 90` stays on the axis stratum and
/// `d = eps / 90` is carried onto the isoclinic one. `None` outside
/// `[0, 90]` degrees.
pub fn tilt_faces_l1(degrees: &Q) -> Option<Faces> {
    let quarter = Q::int(90);
    if degrees.signum() < 0 || degrees > &quarter {
        return None;
    }
    let divergent = degrees.checked_div(&quarter)?;
    Some(Faces {
        convergent: &Q::ONE - &divergent,
        divergent,
    })
}

/// The tilt's natural balance: where its faces equal the charge's, the
/// axis stratum holding the held charge `1 - pi` and the isoclinic one the
/// open charge `pi`: `eps* = 90 pi` degrees. `None` outside `[0, 1]`.
pub fn tilt_equilibrium(open: &Q) -> Option<Q> {
    // Stand-in S125: The tilt rests with no restoring face against the open charge.
    within_share(open).then(|| open * &Q::int(90))
}

/// The charged spin at a reach of held face `delta`, tilted by `degrees`:
/// `K = delta (T B_0 T~ + sum_{q=1..5} B_q)`, the six planes' blades in the
/// common phase with the axis plane's tilted ([`tilt`]), its magnitude the
/// holonomy as a share of the turn. `None` for `delta` outside `[0, 1]` or
/// a tilt whose gridded half-angle is the zero direction.
pub fn charged_spin(delta: &Q, degrees: &Q, bits: u32) -> Option<crate::cga::Mv> {
    use crate::cga::{base_axis, Mv};
    if !within_share(delta) {
        return None;
    }
    let blade = |p: usize| -> Option<Mv> {
        let (a, b) = plane(p)?;
        Some(Mv::blade(base_axis(a)?, Q::ONE).geo(&Mv::blade(base_axis(b)?, Q::ONE)))
    };
    let axis = tilt(degrees, bits)?.conjugate(&blade(0)?);
    let isoclinic = (1..PLANES).try_fold(Mv::zero(), |m, p| Some(m.add(&blade(p)?)))?;
    // Stand-in S132: One holonomy scales every plane, not one per plane.
    Some(axis.add(&isoclinic).scale(delta))
}

fn within_share(x: &Q) -> bool {
    x.signum() >= 0 && x <= &Q::ONE
}

/// The linking number of the join's two strata, the axis circle `S^1` and
/// the isoclinic `S^9` in `S^11` (1 + 9 + 1 = 11): they link once.
pub const STRATA_LINKING: i64 = 1;

/// Plane `p`'s generators as `cga` base axes (counted from 1): `2p + 1`
/// and `2p + 2`. `None` past the sixth plane.
pub fn plane(p: usize) -> Option<(usize, usize)> {
    (p < PLANES).then_some((2 * p + 1, 2 * p + 2))
}

/// The weighted turn `z_p -> e^{i w_p theta} z_p` by `degrees` on every
/// plane, each plane's half-angle on the grid of `2^-bits`. `None` where a
/// plane's gridded half-angle is the zero direction.
pub fn turn(w: &[i64; PLANES], degrees: &Q, bits: u32) -> Option<Rotor<AXES>> {
    w.iter()
        .enumerate()
        .try_fold(Rotor::identity(), |rotor, (p, wp)| {
            let (from, toward) = plane(p)?;
            let turn = Turn {
                from,
                toward,
                degrees: degrees * &Q::int(*wp),
            };
            Some(rotor.then_after(&Rotor::of(&turn, bits)?))
        })
}

/// A weight's plane shares, `|m_p| / l` with `l = sum |m_p|`. `None` for
/// `l = 0`: the single state of degree zero, which overlaps itself wholly
/// and has no shares to compare.
pub fn plane_shares(m: &[i64; PLANES]) -> Option<[Q; PLANES]> {
    let l = m
        .iter()
        .try_fold(0i64, |acc, x| acc.checked_add(x.checked_abs()?))?;
    let whole = Q::int(l).recip()?;
    Some(m.map(|x| &Q::int(x).abs() * &whole))
}

/// The shorter arc of a phase difference of `degrees`, in `[0, 180]`.
pub fn arc(degrees: &Q) -> Q {
    let f = degrees.over(crate::nonzero!(360)).frac();
    let near = f.clone().min(&Q::ONE - &f);
    &near * &Q::int(360)
}

/// Two fibres' faces, an L1 reading of each plane's round angle (named as
/// one, 78.175's shared degrees): the held overlap `c = sum_p min(s_p(i), s_p(j)) (1 -
/// arc_p / 180)` and the passed separation `d = 1 - c`, from their plane
/// shares `si`, `sj` and the phase difference on each plane in degrees
/// (`dr`, any value; its shorter arc is taken). Identical fibres hold
/// wholly (`c = 1`); fibres on disjoint planes, or antipodal on every
/// shared plane, pass wholly (`c = 0`). `None` for shares that are negative
/// or do not sum to one.
pub fn overlap(si: &[Q; PLANES], sj: &[Q; PLANES], dr: &[Q; PLANES]) -> Option<Faces> {
    let whole = |s: &[Q; PLANES]| {
        s.iter().all(|x| x.signum() >= 0) && s.iter().fold(Q::ZERO, |a, x| &a + x) == Q::ONE
    };
    if !whole(si) || !whole(sj) {
        return None;
    }
    let half_turn = Q::int(180).recip()?;
    let convergent = si.iter().zip(sj).zip(dr).fold(Q::ZERO, |acc, ((a, b), r)| {
        let kept = &Q::ONE - &(&arc(r) * &half_turn);
        &acc + &(&a.clone().min(b.clone()) * &kept)
    });
    let divergent = &Q::ONE - &convergent;
    Some(Faces {
        convergent,
        divergent,
    })
}

/// The linking of two fibres on the plane pair `(p, q)`, on the 3-sphere
/// `S^3_{p,q}` the two planes cut from `S^11`: the product `w_ip w_jq` of
/// their windings. `None` for a plane past the sixth or a product past
/// `i64`.
pub fn pair_linking(wi: &[i64; PLANES], wj: &[i64; PLANES], p: usize, q: usize) -> Option<i64> {
    wi.get(p)?.checked_mul(*wj.get(q)?)
}

/// The axis-isoclinic linking of two fibres: `sum_{q=1..5} (w_i0 w_jq + w_j0
/// w_iq)`, the plane-pair products between the axis plane and each
/// isoclinic plane, both ways. Symmetric in the two fibres. `None` past
/// `i64`.
pub fn linking(wi: &[i64; PLANES], wj: &[i64; PLANES]) -> Option<i64> {
    (1..PLANES).try_fold(0i64, |acc, q| {
        let both = pair_linking(wi, wj, 0, q)?.checked_add(pair_linking(wj, wi, 0, q)?)?;
        acc.checked_add(both)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certified::Encl;

    fn vector(v: [i64; AXES]) -> [Encl; AXES] {
        v.map(|k| Encl::exact(Q::int(k)))
    }

    #[test]
    fn the_common_turn_fixes_no_vector_and_the_frozen_axis_plane_does() {
        let right = Q::int(90);
        let common = turn(&COMMON, &right, 40).unwrap();
        let frozen = turn(&[0, 1, 1, 1, 1, 1], &right, 40).unwrap();
        let mut axis_only = [0i64; AXES];
        axis_only[0] = 3;
        let mut mixed = [0i64; AXES];
        (mixed[0], mixed[1], mixed[4], mixed[11]) = (1, -2, 5, 7);
        // The Hopf action moves every non-zero vector, the pure axis state
        // among them; a right angle on every plane carries (x, y) to (-y, x).
        for v in [axis_only, mixed] {
            let turned = common.apply(&vector(v));
            assert_ne!(turned, vector(v));
            let expect: [i64; AXES] =
                std::array::from_fn(|k| if k % 2 == 0 { -v[k + 1] } else { v[k - 1] });
            assert_eq!(turned, vector(expect));
        }
        // Freezing the axis plane fixes its circle: the action is not free.
        assert_eq!(frozen.apply(&vector(axis_only)), vector(axis_only));
        // A whole common turn returns every point (joint read with
        // netstratum, carrier fixture 1).
        let whole = turn(&COMMON, &Q::int(360), 40).unwrap();
        for v in [axis_only, mixed] {
            assert_eq!(whole.apply(&vector(v)), vector(v));
        }
    }

    #[test]
    fn weighted_turns_on_disjoint_planes_commute_and_wind_each_plane() {
        let deg = Q::int(30);
        let a = turn(&[2, 0, 0, -1, 0, 3], &deg, 40).unwrap();
        let b = turn(&[0, 1, 0, 0, 4, 0], &deg, 40).unwrap();
        let mut v = [0i64; AXES];
        (v[0], v[2], v[6], v[8], v[10]) = (1, 1, 1, 1, 1);
        let v = vector(v);
        assert_eq!(a.then_after(&b).apply(&v), b.then_after(&a).apply(&v));
        // A winding of zero on a plane leaves it as it is.
        let turned = a.apply(&v);
        assert_eq!((&turned[2], &turned[3]), (&v[2], &v[3]));
        assert!(plane(PLANES).is_none());
    }

    #[test]
    fn plane_shares_are_whole_and_refuse_degree_zero() {
        let s = plane_shares(&[2, -1, 0, 0, 1, 0]).unwrap();
        assert_eq!(s.iter().fold(Q::ZERO, |a, x| &a + x), Q::ONE);
        assert_eq!(s[0], crate::ratio!(1, 2));
        assert!(plane_shares(&[0; PLANES]).is_none());
    }

    #[test]
    fn the_overlap_holds_identical_fibres_and_passes_disjoint_or_antipodal_ones() {
        let s = plane_shares(&[3, 1, 0, 2, 0, 0]).unwrap();
        let none = [0i64; PLANES].map(Q::int);
        let same = overlap(&s, &s, &none).unwrap();
        assert_eq!((same.convergent, same.divergent), (Q::ONE, Q::ZERO));
        // Phases are read by their shorter arc: 360 degrees apart is no
        // separation, -90 is 90.
        let whole_turn = [360i64; PLANES].map(Q::int);
        assert_eq!(overlap(&s, &s, &whole_turn).unwrap().convergent, Q::ONE);
        assert_eq!(arc(&Q::int(-90)), Q::int(90));
        assert_eq!(arc(&Q::int(540)), Q::int(180));
        // Antipodal on every plane, or on disjoint planes, they pass wholly.
        let opposed = [180i64; PLANES].map(Q::int);
        assert_eq!(overlap(&s, &s, &opposed).unwrap().convergent, Q::ZERO);
        let other = plane_shares(&[0, 0, 4, 0, 1, 0]).unwrap();
        assert_eq!(overlap(&s, &other, &none).unwrap().convergent, Q::ZERO);
        // The faces close, every value lies in [0, 1], and the reading is
        // symmetric in the two fibres.
        let t = plane_shares(&[1, 1, 1, 1, 1, 1]).unwrap();
        let dr = [10, -45, 170, 0, 90, 300].map(Q::int);
        let (ab, ba) = (overlap(&s, &t, &dr).unwrap(), overlap(&t, &s, &dr).unwrap());
        assert_eq!(&ab.convergent + &ab.divergent, Q::ONE);
        assert!(ab.convergent.signum() >= 0 && ab.divergent.signum() >= 0);
        assert_eq!(ab, ba);
        // Shares that are not whole are refused.
        let mut bad = s.clone();
        bad[0] = &bad[0] + &Q::ONE;
        assert!(overlap(&bad, &s, &none).is_none());
    }

    /// The quarter turn mixing the axis plane with isoclinic plane `q`: axis
    /// 1 toward the plane's first axis and axis 2 toward its second, so
    /// `(z_0, z_q) -> (-z_q, z_0)`, complex linear.
    fn mixing(q: usize) -> Rotor<AXES> {
        let (a, b) = plane(q).unwrap();
        let quarter = |from, toward| {
            let turn = Turn {
                from,
                toward,
                degrees: Q::int(90),
            };
            Rotor::of(&turn, 40).unwrap()
        };
        quarter(2, b).then_after(&quarter(1, a))
    }

    #[test]
    fn no_weighted_turn_moves_a_plane_share_and_a_mixing_quarter_turn_exchanges_them() {
        // A body on the axis plane (3, 4) and on isoclinic plane 2 (1, -2).
        let mut v = [0i64; AXES];
        (v[0], v[1], v[4], v[5]) = (3, 4, 1, -2);
        let body = vector(v);
        let plane_norms = |x: &[Encl; AXES]| -> [Q; PLANES] {
            std::array::from_fn(|p| {
                &(&x[2 * p].lo * &x[2 * p].lo) + &(&x[2 * p + 1].lo * &x[2 * p + 1].lo)
            })
        };
        // Every weighted turn keeps each plane's part: no winding exchanges
        // the axis stratum with the isoclinic one.
        for w in [COMMON, [2, 0, -3, 1, 0, 5], [0, 1, 1, 1, 1, 1]] {
            let turned = turn(&w, &Q::int(30), 40).unwrap().apply(&body);
            assert_eq!(plane_norms(&turned), plane_norms(&body));
        }
        // The mixing quarter turn carries the axis part onto plane 2 and
        // plane 2's onto the axis, negated: (z0, z2) -> (-z2, z0).
        let x = mixing(2);
        let mut e = [0i64; AXES];
        (e[0], e[1], e[4], e[5]) = (-1, 2, 3, 4);
        assert_eq!(x.apply(&body), vector(e));
        // Twice is the half turn on both planes, four times the identity:
        // two same-sense quarter turns are not the identity.
        let twice = x.then_after(&x);
        let mut h = [0i64; AXES];
        (h[0], h[1], h[4], h[5]) = (-3, -4, -1, 2);
        assert_eq!(twice.apply(&body), vector(h));
        assert_eq!(twice.then_after(&twice).apply(&body), body);
        // It is complex linear on the two planes, so it commutes with the
        // Hopf action and carries fibres to fibres.
        let hopf = turn(&COMMON, &Q::int(30), 40).unwrap();
        assert_eq!(
            x.then_after(&hopf).apply(&body),
            hopf.then_after(&x).apply(&body)
        );
    }

    #[test]
    fn the_charged_spin_rests_on_its_tilt_at_the_charges_balance() {
        use crate::cga::{base_axis, Mv};
        let blade = |a: usize, b: usize| {
            Mv::blade(base_axis(a).unwrap(), Q::ONE).geo(&Mv::blade(base_axis(b).unwrap(), Q::ONE))
        };
        let common = (0..PLANES).fold(Mv::zero(), |m, p| {
            let (a, b) = plane(p).unwrap();
            m.add(&blade(a, b))
        });
        let delta = crate::ratio!(1, 4);
        // Upright, the charged spin is the common phase at the holonomy's
        // share.
        assert_eq!(
            charged_spin(&delta, &Q::ZERO, 40).unwrap(),
            common.scale(&delta)
        );
        assert_eq!(holonomy(&delta), Some(Q::int(90)));
        // A complex-linear mixing of planes leaves the common phase as it
        // is: only a tilt of one generator deforms it.
        let x = mixing(2);
        assert_eq!(x.conjugate(&common), common);
        // A right tilt carries the pole axis onto e3: the axis blade e1 e2
        // becomes e3 e2.
        let right = charged_spin(&Q::ONE, &Q::int(90), 40).unwrap();
        let expect = common.sub(&blade(1, 2)).add(&blade(3, 2));
        assert_eq!(right, expect);
        // Any tilt keeps the spin's magnitude.
        let tilted = charged_spin(&delta, &Q::int(30), 40).unwrap();
        assert_eq!(
            tilted.geo(&tilted.reverse()).scalar_part(),
            common.scale(&delta).norm2().abs()
        );
        // LAW-COUPLING v2's fixture: (p, n) = (3, 4) at H = 1, R = 3, delta = 1/4:
        // pi = 9/28 and the tilt rests at 90 pi = 405/14 degrees, where its
        // faces are the charge's.
        let pi = open_charge(&crate::ratio!(3, 7), &delta).unwrap();
        assert_eq!(pi, crate::ratio!(9, 28));
        let rest = tilt_equilibrium(&pi).unwrap();
        assert_eq!(rest, crate::ratio!(405, 14));
        let faces = tilt_faces_l1(&rest).unwrap();
        assert_eq!(
            (faces.convergent, faces.divergent.clone()),
            (&Q::ONE - &pi, pi.clone())
        );
        // The balance restores: past it the isoclinic stratum holds more
        // than the open charge carries, short of it less.
        let past = tilt_faces_l1(&(&rest + &Q::ONE)).unwrap().divergent;
        let short = tilt_faces_l1(&(&rest - &Q::ONE)).unwrap().divergent;
        assert!(past > pi && short < pi);
        // Neutral or wholly held stands upright; a bare proton far from any
        // hold tilts a quarter turn, the strata exchanged; the equator is
        // half of it.
        assert_eq!(
            tilt_equilibrium(&open_charge(&Q::ONE, &Q::ONE).unwrap()),
            Some(Q::ZERO)
        );
        assert_eq!(
            tilt_equilibrium(&open_charge(&Q::ONE, &Q::ZERO).unwrap()),
            Some(Q::int(90))
        );
        assert_eq!(tilt_equilibrium(&crate::ratio!(1, 2)), Some(Q::int(45)));
        // Refusals.
        assert!(
            open_charge(&Q::int(2), &delta).is_none() && open_charge(&Q::ONE, &Q::int(2)).is_none()
        );
        assert!(tilt_faces_l1(&Q::int(91)).is_none() && tilt_faces_l1(&Q::int(-1)).is_none());
        assert!(holonomy(&Q::int(2)).is_none() && tilt_equilibrium(&Q::int(-1)).is_none());
    }

    #[test]
    fn linking_is_symmetric_integral_and_needs_an_axis_winding() {
        let (a, b) = ([2, 1, 0, -1, 0, 3], [1, 0, 2, 0, 5, 0]);
        assert_eq!(linking(&a, &b), linking(&b, &a));
        // For a, w0 = 2 against b's (0, 2, 0, 5, 0) sum 7; for b, w0 = 1 against a's
        // (1, 0, -1, 0, 3) sum 3.
        assert_eq!(linking(&a, &b), Some(2 * 7 + 3));
        // Fibres with no axis winding do not link across the strata.
        assert_eq!(linking(&[0, 1, 1, 1, 1, 1], &[0, 2, 0, 3, 0, 1]), Some(0));
        assert_eq!(pair_linking(&a, &b, 0, 2), Some(4));
        assert!(pair_linking(&a, &b, 0, PLANES).is_none());
        assert!(linking(&[i64::MAX; PLANES], &[2; PLANES]).is_none());
    }
}
