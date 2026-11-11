//! Frames: turns on named planes, stated in degrees, composed exactly as
//! the even versors of the geometric algebra (`crate::cga`).
//!
//! Inputs: a [`Turn`], a plane named by its two generators (base axes counted
//! from 1, as `cga::base_axis`) and an angle in degrees ([`Q`]); or a drawn
//! half-angle direction `(c, s)` at any scale on a named plane, or a
//! [`Direction`], one drawn and refused once if zero; vectors to
//! turn as `certified::Encl` arrays of any dimension.
//!
//! Outputs:
//! - [`Frame`] of `N` base axes, an even versor `c - s e_from e_toward` and the products of
//!   such versors, on any planes of base axes up to `cga::MAX_N` (the axis
//!   plane and the five isoclinic planes among them): `of` turns `from`
//!   toward `toward` by a turn's degrees, `drawn` by a drawn half-angle
//!   direction, `on` by a [`Direction`] on a plane written in the code
//!   (checked when compiled), `then_after` composes by the geometric product, `inverse` is
//!   the reverse, and `apply` turns an enclosed `N`-vector by the sandwich
//!   `V v V~ / (V V~)`, with no matrix.
//! - [`circle_point`], the cosine and sine of an angle in degrees as the
//!   exact rational point of the circle the turn carries its first
//!   generator to.
//! - [`angle_between`], the certified angle between two exact directions.
//!
//! Invariants: a turn's direction is its half-angle, `degrees / 720` of a
//! turn, rounded down to the grid of `2^-bits` once; after that every frame
//! and every turned exact vector is exact, at any scale of the versor (the
//! sandwich is divided by `V V~`, a positive scalar for a product of turns).
//! A turn whose plane is not two distinct axes within `cga::MAX_N`, and the
//! zero direction `(0, 0)`, which holds no angle, are refused (`None`). A
//! frame holds its dimension `N` in its type: its planes lie within its `N`
//! axes, so it turns an `N`-vector without refusal.

use crate::certified::{cos_sin_turns, down, Encl};
use crate::cga::{base_axis, Mv};
use crate::dyadic::Dy;
use crate::Q;

/// A turn: the plane it turns, named by its generators (base axes counted
/// from 1), turning `from` toward `toward`, by `degrees`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Turn {
    /// The generator the turn carries toward `toward`.
    pub from: usize,
    /// The generator `from` is carried toward.
    pub toward: usize,
    /// The angle of the turn, in degrees.
    pub degrees: Q,
}

/// A drawn half-angle direction `(c, s)` at any scale, never the zero
/// direction `(0, 0)`, which holds no angle: the refusal is taken once, where
/// the direction is drawn ([`Direction::drawn`]), so a frame built from it
/// ([`Frame::on`]) needs none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Direction {
    c: Dy,
    s: Dy,
}

impl Direction {
    /// The zero angle, the direction `(1, 0)`.
    pub const ZERO_ANGLE: Direction = Direction {
        c: Dy::ONE,
        s: Dy::ZERO,
    };

    /// The direction `(c, s)`. `None` for `(0, 0)`.
    pub fn drawn(c: Dy, s: Dy) -> Option<Direction> {
        if c.is_zero() && s.is_zero() {
            return None;
        }
        Some(Direction { c, s })
    }

    /// The direction's first component.
    pub fn c(&self) -> Dy {
        self.c
    }

    /// The direction's second component, at the scale of the first.
    pub fn s(&self) -> Dy {
        self.s
    }
}

/// The masks of the generators of a plane written in the code, checked when
/// the code is compiled: two distinct axes of `n`, within `cga::MAX_N`.
#[expect(
    clippy::panic,
    reason = "a const block: a plane written outside the frame fails the build, never a run"
)]
const fn written_plane(from: usize, toward: usize, n: usize) -> (usize, usize) {
    match (base_axis(from), base_axis(toward)) {
        (Some(a), Some(b)) if from != toward && from <= n && toward <= n => (a, b),
        _ => panic!("a frame's plane is two distinct axes of the frame"),
    }
}

/// A frame of `N` base axes: an even versor of `cga`, the product of turns
/// on planes within those axes, held at any scale with the reciprocal of
/// its scale `V V~`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame<const N: usize> {
    v: Mv,
    inv: Q,
}

/// The plane `e_from e_toward` as a bivector, with the sign of the product's
/// order. `None` for one axis twice or an axis outside `1..=n`.
fn plane(from: usize, toward: usize, n: usize) -> Option<Mv> {
    if from == toward || from > n || toward > n {
        return None;
    }
    let (a, b) = (base_axis(from)?, base_axis(toward)?);
    Some(Mv::blade(a, Q::ONE).geo(&Mv::blade(b, Q::ONE)))
}

/// The half-angle of `degrees` as a direction `(cos, sin)` on the grid of
/// `2^-bits`.
fn half_direction(degrees: &Q, bits: u32) -> (Q, Q) {
    let (c, s) = cos_sin_turns(&Encl::exact(degrees.over(crate::nonzero!(720))));
    (down(&c.mid(), bits), down(&s.mid(), bits))
}

impl<const N: usize> Frame<N> {
    /// The identity frame: no turn.
    pub fn identity() -> Frame<N> {
        Frame {
            v: Mv::scalar(Q::ONE),
            inv: Q::ONE,
        }
    }

    /// The frame of one turn, its half-angle on the grid of `2^-bits`.
    /// `None` for a plane that is not two distinct axes of the frame's `N`
    /// (within `cga::MAX_N`), and where the gridded half-angle is the zero
    /// direction.
    pub fn of(turn: &Turn, bits: u32) -> Option<Frame<N>> {
        let (c, s) = half_direction(&turn.degrees, bits);
        Frame::drawn(turn.from, turn.toward, &c, &s)
    }

    /// The frame of a drawn half-angle direction `(c, s)` at any scale on
    /// the plane turning `from` toward `toward`: the versor
    /// `c - s e_from e_toward`, of scale `c^2 + s^2`. `None` for the zero
    /// direction and for a plane that is not two distinct axes of the
    /// frame's `N` (within `cga::MAX_N`).
    pub fn drawn(from: usize, toward: usize, c: &Q, s: &Q) -> Option<Frame<N>> {
        let b = plane(from, toward, N)?;
        let inv = (&(c * c) + &(s * s)).recip()?;
        Some(Frame {
            v: Mv::scalar(c.clone()).sub(&b.scale(s)),
            inv,
        })
    }

    /// The frame of a drawn direction on the plane turning axis `FROM`
    /// toward axis `TOWARD`, both written in the code and checked when it is
    /// compiled: the versor `c - s e_FROM e_TOWARD`, of scale `c^2 + s^2`.
    pub fn on<const FROM: usize, const TOWARD: usize>(d: &Direction) -> Frame<N> {
        let (a, b) = const { written_plane(FROM, TOWARD, N) };
        let (c, s) = (d.c.to_q(), d.s.to_q());
        let plane = Mv::blade(a, Q::ONE).geo(&Mv::blade(b, Q::ONE));
        // A `Direction` is never `(0, 0)`, so `c^2 + s^2 > 0` and the
        // reciprocal is always taken.
        let inv = (&(&c * &c) + &(&s * &s)).recip().map_or(Q::ONE, |r| r);
        Frame {
            v: Mv::scalar(c).sub(&plane.scale(&s)),
            inv,
        }
    }

    /// `self` after `o`: the product turns by `o` first, then by `self`; its
    /// scale is the product of the two scales.
    pub fn then_after(&self, o: &Frame<N>) -> Frame<N> {
        Frame {
            v: self.v.geo(&o.v),
            inv: &self.inv * &o.inv,
        }
    }

    /// The inverse frame: the reverse `V~`, of the same scale. Composed with
    /// the frame it is the scalar `V V~`, the identity at that scale,
    /// exactly.
    pub fn inverse(&self) -> Frame<N> {
        Frame {
            v: self.v.reverse(),
            inv: self.inv.clone(),
        }
    }

    /// The frame's versor in `cga`.
    pub fn versor(&self) -> &Mv {
        &self.v
    }

    /// An exact vector turned: `V v V~ / (V V~)`. The frame's planes lie
    /// within its `N` axes, so the turned vector does too; an axis past
    /// `cga::MAX_N` lies on no plane and keeps its component.
    fn turn(&self, v: &[Q; N]) -> [Q; N] {
        let turned = self.v.sandwich(&crate::cga::base_vector(v));
        std::array::from_fn(|i| {
            base_axis(i + 1).map_or(v[i].clone(), |a| &turned.coeff(a) * &self.inv)
        })
    }

    /// An enclosed vector turned by the frame: its exact midpoint turned,
    /// each component widened by the sum of the vector's radii, since a turn
    /// moves no vector by more than its length. An exact vector stays exact.
    pub fn apply(&self, v: &[Encl; N]) -> [Encl; N] {
        let mid: [Q; N] = std::array::from_fn(|i| v[i].mid());
        let rad = v
            .iter()
            .fold(Q::ZERO, |acc, e| &acc + &(&e.hi - &e.lo))
            .over(crate::nonzero!(2));
        self.turn(&mid).map(|t| Encl {
            lo: &t - &rad,
            hi: &t + &rad,
        })
    }
}

/// The cosine and sine of `degrees`, exact: the point of the circle a turn
/// by `degrees` carries its first generator to, its half-angle on the grid
/// of `2^-bits`, so `cos^2 + sin^2 = 1` exactly. `None` where the gridded
/// half-angle is the zero direction.
pub fn circle_point(degrees: &Q, bits: u32) -> Option<(Q, Q)> {
    let (c, s) = half_direction(degrees, bits);
    let n = &(&c * &c) + &(&s * &s);
    let cos = (&(&c * &c) - &(&s * &s)).checked_div(&n)?;
    let sin = (&Q::int(2) * &(&c * &s)).checked_div(&n)?;
    Some((cos, sin))
}

/// The angle between two directions `a` and `b` (any lengths, exact), in
/// radians as a certified enclosure: its cosine is `a . b / |a| |b|` (the
/// lengths' product a certified root of `|a|^2 |b|^2`), and the angle is
/// `pi / 2 - asin(cos)` for a non-negative cosine and `pi / 2 + asin(-cos)`
/// otherwise. The directions stay exact; only the angle, which has no
/// rational form in general, is an enclosure. `None` for a zero direction.
pub fn angle_between(a: &[Q; 3], b: &[Q; 3], bits: u32) -> Option<crate::certified::Encl> {
    use crate::certified::{asin_any, pi, sqrt};
    let dot = a.iter().zip(b).fold(Q::ZERO, |acc, (x, y)| &acc + &(x * y));
    let norm2 = |v: &[Q; 3]| v.iter().fold(Q::ZERO, |acc, x| &acc + &(x * x));
    let (na, nb) = (norm2(a), norm2(b));
    if na.is_zero() || nb.is_zero() {
        return None;
    }
    let len = sqrt(&(&na * &nb), bits)?;
    let cos = Encl::exact(dot.clone()).mul(&len.recip()?);
    let quarter = pi(bits).mul(&Encl::exact(crate::ratio!(1, 2)));
    Some(if dot.signum() >= 0 {
        quarter.sub(&asin_any(&cos))
    } else {
        quarter.add(&asin_any(&cos.neg()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exact<const N: usize>(v: [i64; N]) -> [Encl; N] {
        v.map(|k| Encl::exact(Q::int(k)))
    }

    #[test]
    fn a_turn_in_degrees_carries_its_generator_exactly() {
        // A right angle turns e1 to e2 exactly: its half-angle direction has
        // equal components on any grid.
        fn right<const N: usize>(from: usize, toward: usize) -> Frame<N> {
            let turn = Turn {
                from,
                toward,
                degrees: Q::int(90),
            };
            Frame::of(&turn, 40).unwrap()
        }
        assert_eq!(right::<3>(1, 2).apply(&exact([1, 0, 0])), exact([0, 1, 0]));
        // Turning toward the other way round is the inverse.
        assert_eq!(right::<3>(2, 1).apply(&exact([0, 1, 0])), exact([1, 0, 0]));
        // A right angle in the plane (1, 2) and then one in (2, 3) carry e1
        // to e2 and then e2 to e3.
        let r = right::<3>(2, 3).then_after(&right(1, 2));
        assert_eq!(r.apply(&exact([1, 0, 0])), exact([0, 0, 1]));
        // Turns on the isoclinic planes past the third axis compose in a
        // frame that holds them, and leave a vector off their planes as it
        // is.
        let far = right::<11>(10, 11).then_after(&right(4, 5));
        let (mut v, mut w) = ([0i64; 11], [0i64; 11]);
        (v[0], v[3], v[9]) = (1, 1, 1);
        (w[0], w[4], w[10]) = (1, 1, 1);
        assert_eq!(far.apply(&exact(v)), exact(w));
        // One axis twice, an axis outside the frame and the zero direction
        // hold no plane or angle.
        let none = |from, toward| {
            let turn = Turn {
                from,
                toward,
                degrees: Q::int(30),
            };
            Frame::<3>::of(&turn, 40)
        };
        assert!(none(2, 2).is_none() && none(0, 1).is_none() && none(3, 4).is_none());
        assert!(Frame::<{ crate::cga::MAX_N + 1 }>::drawn(
            1,
            crate::cga::MAX_N + 1,
            &Q::ONE,
            &Q::ONE
        )
        .is_none());
        assert!(Frame::<3>::drawn(1, 2, &Q::ZERO, &Q::ZERO).is_none());
        assert!(Direction::drawn(Dy::ZERO, Dy::ZERO).is_none());
        // A direction on a written plane is the same frame as the drawn one.
        let d = Direction::drawn(Dy::int(3), Dy::int(-2)).unwrap();
        let drawn = Frame::<3>::drawn(2, 3, &Q::int(3), &Q::int(-2)).unwrap();
        assert_eq!(Frame::<3>::on::<2, 3>(&d), drawn);
        assert_eq!(
            Frame::<3>::on::<1, 2>(&Direction::ZERO_ANGLE),
            Frame::identity()
        );
    }

    #[test]
    fn a_frame_keeps_length_and_its_inverse_undoes_it() {
        // A frame at any scale keeps an exact vector's length, and its
        // inverse brings the vector back.
        let s = Frame::<3>::drawn(1, 2, &Q::int(2), &Q::int(5))
            .unwrap()
            .then_after(&Frame::drawn(2, 3, &Q::int(1), &crate::ratio!(-1, 3)).unwrap())
            .then_after(&Frame::drawn(1, 3, &Q::int(3), &Q::int(1)).unwrap());
        let v = exact([3, -4, 7]);
        let turned = s.apply(&v);
        let len = |v: &[Encl; 3]| v.iter().fold(Q::ZERO, |acc, c| &acc + &(&c.lo * &c.lo));
        assert_eq!(len(&turned), len(&v));
        assert_eq!(s.inverse().apply(&turned), v);
        // The frame and its inverse compose to the positive scalar V V~.
        let p = s.then_after(&s.inverse());
        assert_eq!(p.versor().grade(0), *p.versor());
        assert!(p.versor().scalar_part() > Q::ZERO);
        // An enclosed vector's turn encloses the turn of every point in it.
        let wide = [
            Encl {
                lo: crate::ratio!(29, 10),
                hi: Q::int(3),
            },
            Encl::exact(Q::int(-4)),
            Encl {
                lo: Q::int(7),
                hi: crate::ratio!(71, 10),
            },
        ];
        let out = s.apply(&wide);
        for p in [
            exact([3, -4, 7]),
            [crate::ratio!(29, 10), Q::int(-4), crate::ratio!(71, 10)].map(Encl::exact),
        ] {
            let at = s.apply(&p);
            assert!((0..3).all(|i| out[i].contains(&at[i].lo)));
        }
    }

    #[test]
    fn a_circle_point_is_on_the_circle_and_is_the_turned_generator() {
        for d in [-450, -90, 0, 17, 45, 90, 135, 180, 270, 359] {
            let degrees = Q::int(d);
            let (c, s) = circle_point(&degrees, 60).unwrap();
            assert_eq!(&(&c * &c) + &(&s * &s), Q::ONE);
            let turn = Turn {
                from: 1,
                toward: 2,
                degrees,
            };
            let image = Frame::<2>::of(&turn, 60).unwrap().apply(&exact([1, 0]));
            assert_eq!(image, [c, s].map(Encl::exact));
        }
    }

    #[test]
    fn the_angle_between_directions_is_enclosed() {
        let q = |a: i64, b: i64, c: i64| [Q::int(a), Q::int(b), Q::int(c)];
        let pi = crate::certified::pi(64);
        let half = pi.mul(&crate::certified::Encl::exact(crate::ratio!(1, 2)));
        // Parallel, perpendicular and opposed, at any lengths.
        let same = angle_between(&q(2, 0, 0), &q(5, 0, 0), 64).unwrap();
        assert!(same.contains(&Q::ZERO) || same.hi < crate::ratio!(1, 1_000_000));
        let right = angle_between(&q(1, 0, 0), &q(0, 3, 0), 64).unwrap();
        assert!(right.lo <= half.hi && half.lo <= right.hi);
        let back = angle_between(&q(1, 0, 0), &q(-4, 0, 0), 64).unwrap();
        assert!(back.lo <= pi.hi && pi.lo <= back.hi);
        // The same pair either way round, and a zero direction refused.
        let (ab, ba) = (
            angle_between(&q(3, 4, 0), &q(0, 1, 1), 64).unwrap(),
            angle_between(&q(0, 1, 1), &q(3, 4, 0), 64).unwrap(),
        );
        assert!(ab.lo <= ba.hi && ba.lo <= ab.hi);
        assert!(angle_between(&q(0, 0, 0), &q(1, 0, 0), 64).is_none());
    }
}
