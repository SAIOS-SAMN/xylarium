//! Rotors: turns on named planes, stated in degrees, composed exactly as
//! the even versors of the geometric algebra (`crate::cga`).
//!
//! Inputs: a [`Turn`], a plane named by its two generators (base axes counted
//! from 1, as `cga::base_axis`) and an angle in degrees ([`Q`]); or a drawn
//! half-angle direction `(c, s)` at any scale on a named plane, or a
//! [`Direction`], one drawn and refused once if zero; vectors to
//! turn as `certified::Encl` arrays of any dimension.
//!
//! Outputs:
//! - [`Rotor`] of `N` base axes, an even versor `c - s e_from e_toward` and the products of
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
//! - [`Winding`], a turn split into its whole turns and its phase in
//!   degrees (E019, #149).
//! - [`pivot`], the relative rotor between two exact unit directions, its
//!   one plane by generators and its turn in degrees, certified.
//!
//! Invariants: a turn's direction is its half-angle, `degrees / 720` of a
//! turn, cut toward zero to the grid of `2^-bits` once; after that every rotor
//! and every turned exact vector is exact, at any scale of the versor (the
//! sandwich is divided by `V V~`, a positive scalar for a product of turns).
//! A turn whose plane is not two distinct axes within `cga::MAX_N`, and the
//! zero direction `(0, 0)`, which holds no angle, are refused (`None`). A
//! rotor holds its dimension `N` in its type: its planes lie within its `N`
//! axes, so it turns an `N`-vector without refusal.

use crate::certified::{cos_sin_turns, toward_zero, Encl};
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
/// the direction is drawn ([`Direction::drawn`]), so a rotor built from it
/// ([`Rotor::on`]) needs none.
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
    reason = "a const block: a plane written outside the rotor fails the build, never a run"
)]
const fn written_plane(from: usize, toward: usize, n: usize) -> (usize, usize) {
    match (base_axis(from), base_axis(toward)) {
        (Some(a), Some(b)) if from != toward && from <= n && toward <= n => (a, b),
        _ => panic!("a rotor's plane is two distinct axes of the rotor"),
    }
}

/// A rotor of `N` base axes: an even versor of `cga`, the product of turns
/// on planes within those axes, held at any scale with the reciprocal of
/// its scale `V V~`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rotor<const N: usize> {
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
    (toward_zero(&c.mid(), bits), toward_zero(&s.mid(), bits))
}

impl<const N: usize> Rotor<N> {
    /// The identity rotor: no turn.
    pub fn identity() -> Rotor<N> {
        Rotor {
            v: Mv::scalar(Q::ONE),
            inv: Q::ONE,
        }
    }

    /// The rotor of one turn, its half-angle on the grid of `2^-bits`.
    /// `None` for a plane that is not two distinct axes of the rotor's `N`
    /// (within `cga::MAX_N`), and where the gridded half-angle is the zero
    /// direction.
    pub fn of(turn: &Turn, bits: u32) -> Option<Rotor<N>> {
        let (c, s) = half_direction(&turn.degrees, bits);
        Rotor::drawn(turn.from, turn.toward, &c, &s)
    }

    /// The rotor of a drawn half-angle direction `(c, s)` at any scale on
    /// the plane turning `from` toward `toward`: the versor
    /// `c - s e_from e_toward`, of scale `c^2 + s^2`. `None` for the zero
    /// direction and for a plane that is not two distinct axes of the
    /// rotor's `N` (within `cga::MAX_N`).
    pub fn drawn(from: usize, toward: usize, c: &Q, s: &Q) -> Option<Rotor<N>> {
        let b = plane(from, toward, N)?;
        let inv = (&(c * c) + &(s * s)).recip()?;
        Some(Rotor {
            v: Mv::scalar(c.clone()).sub(&b.scale(s)),
            inv,
        })
    }

    /// The rotor of a drawn direction on the plane turning axis `FROM`
    /// toward axis `TOWARD`, both written in the code and checked when it is
    /// compiled: the versor `c - s e_FROM e_TOWARD`, of scale `c^2 + s^2`.
    pub fn on<const FROM: usize, const TOWARD: usize>(d: &Direction) -> Rotor<N> {
        let (a, b) = const { written_plane(FROM, TOWARD, N) };
        let (c, s) = (d.c.to_q(), d.s.to_q());
        let plane = Mv::blade(a, Q::ONE).geo(&Mv::blade(b, Q::ONE));
        // A `Direction` is never `(0, 0)`, so `c^2 + s^2 > 0` and the
        // reciprocal is always taken.
        let inv = (&(&c * &c) + &(&s * &s)).recip().map_or(Q::ONE, |r| r);
        Rotor {
            v: Mv::scalar(c).sub(&plane.scale(&s)),
            inv,
        }
    }

    /// `self` after `o`: the product turns by `o` first, then by `self`; its
    /// scale is the product of the two scales.
    pub fn then_after(&self, o: &Rotor<N>) -> Rotor<N> {
        Rotor {
            v: self.v.geo(&o.v),
            inv: &self.inv * &o.inv,
        }
    }

    /// The inverse rotor: the reverse `V~`, of the same scale. Composed with
    /// the rotor it is the scalar `V V~`, the identity at that scale,
    /// exactly.
    pub fn inverse(&self) -> Rotor<N> {
        Rotor {
            v: self.v.reverse(),
            inv: self.inv.clone(),
        }
    }

    /// The rotor's versor in `cga`.
    pub fn versor(&self) -> &Mv {
        &self.v
    }

    /// A multivector turned by the rotor: `V x V~ / (V V~)`, exact. A
    /// bivector stays a bivector of the same magnitude.
    pub fn conjugate(&self, x: &Mv) -> Mv {
        self.v.sandwich(x).scale(&self.inv)
    }

    /// An exact vector turned: `V v V~ / (V V~)`. The rotor's planes lie
    /// within its `N` axes, so the turned vector does too; an axis past
    /// `cga::MAX_N` lies on no plane and keeps its component.
    fn turn(&self, v: &[Q; N]) -> [Q; N] {
        let turned = self.v.sandwich(&crate::cga::base_vector(v));
        std::array::from_fn(|i| {
            base_axis(i + 1).map_or(v[i].clone(), |a| &turned.coeff(a) * &self.inv)
        })
    }

    /// An enclosed vector turned by the rotor: its exact midpoint turned,
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

/// A rotor read by its dimensions (model.md, Dimensional orthogonality of a
/// rotor): the turning it carries on each plane of base axes (its bivector
/// part, by generators `(i, j)`, `i < j`), the volumes it carries on four or
/// more axes (its parts of grade 4 and above, which only turnings on
/// orthogonal planes make), and its dimensional capacity, the number of
/// mutually orthogonal planes it turns. Never one angle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dimensions {
    /// The turning on each plane, `((i, j), coefficient)`, at the rotor's
    /// scale, in the order of the planes' generators.
    pub planes: Vec<((usize, usize), Q)>,
    /// The volumes: the axes of each part of grade 4 and above, with its
    /// coefficient at the rotor's scale.
    pub volumes: Vec<(Vec<usize>, Q)>,
    /// The number of mutually orthogonal planes the turning spans: half the
    /// rank of its antisymmetric matrix over the base axes, exact. A plane
    /// turned by exactly a half turn leaves no bivector part and shows only
    /// in a volume.
    pub capacity: usize,
}

/// Two planes of base axes are orthogonal when they share no axis: their
/// turnings commute and compose with a volume. Planes that share an axis do
/// not commute, and their composite turns the third plane between them.
pub fn orthogonal(a: (usize, usize), b: (usize, usize)) -> bool {
    a.0 != b.0 && a.0 != b.1 && a.1 != b.0 && a.1 != b.1
}

/// The base axes (counted from 1) of a blade's mask: bits 0 to 2 are axes
/// 1 to 3, bits 3 and 4 the conformal pair (none), and bit `b >= 5` axis
/// `b - 1` (`cga::base_axis`).
fn axes_of(mask: usize) -> Option<Vec<usize>> {
    let mut axes = Vec::new();
    for bit in 0..usize::BITS as usize {
        if mask & (1 << bit) == 0 {
            continue;
        }
        match bit {
            0..=2 => axes.push(bit + 1),
            3 | 4 => return None,
            _ => axes.push(bit - 1),
        }
    }
    Some(axes)
}

/// The rank of an `n` by `n` matrix over Q, by exact elimination: the
/// pivot is the next row on which the column is not zero, an order of the
/// rows' own index, not a choice among them.
fn rank(mut m: Vec<Vec<Q>>, n: usize) -> usize {
    let cols = n;
    let mut r = 0;
    for c in 0..cols {
        let Some(p) = (r..m.len()).find(|&i| m[i][c].signum() != 0) else {
            continue;
        };
        m.swap(r, p);
        let pivot = m[r][c].clone();
        for i in 0..m.len() {
            if i == r || m[i][c].signum() == 0 {
                continue;
            }
            let Some(f) = m[i][c].checked_div(&pivot) else {
                continue;
            };
            let row = m[r].clone();
            for (x, y) in m[i].iter_mut().zip(&row) {
                *x = &*x - &(&f * y);
            }
        }
        r += 1;
    }
    r
}

impl<const N: usize> Rotor<N> {
    /// The rotor whose rate is `k` times this one's (11b's motion law, the
    /// rate an even generator): a simple rotor `V = c + B` has the rate
    /// `G = B / c`, and the scaled motion is `1 + k G`, exact. `None` for a
    /// half turn (`c = 0`, no rate) and for a rotor of capacity above one,
    /// whose volume no single rate carries.
    pub fn rate_scaled(&self, k: &Q) -> Option<Rotor<N>> {
        let rate = self.rate()?;
        let v = Mv::scalar(Q::ONE).add(&rate.scale(k));
        let inv = v.geo(&v.reverse()).scalar_part().recip()?;
        Some(Rotor { v, inv })
    }

    /// The rotor's rate per plane (11b): its bivector part over its scalar
    /// part, `G = B / c`, by generators, exact; each plane's rate is the
    /// tangent of half its turn there. `None` as for [`Rotor::rate_scaled`].
    pub fn rates(&self) -> Option<Vec<((usize, usize), Q)>> {
        let rate = self.rate()?;
        Some(
            rate.terms()
                .filter_map(|(mask, g)| match axes_of(mask)?.as_slice() {
                    [i, j] => Some(((*i, *j), g.clone())),
                    _ => None,
                })
                .collect(),
        )
    }

    /// The rate as a bivector, `B / c`; `None` as for [`Rotor::rate_scaled`].
    fn rate(&self) -> Option<Mv> {
        let d = self.dimensions();
        if d.capacity > 1 || !d.volumes.is_empty() {
            return None;
        }
        let c = self.v.scalar_part();
        Some(self.v.grade(2).scale(&c.recip()?))
    }
}

impl<const N: usize> Rotor<N> {
    /// The rotor read by its dimensions ([`Dimensions`]).
    pub fn dimensions(&self) -> Dimensions {
        let mut planes = Vec::new();
        let mut volumes = Vec::new();
        let mut matrix = vec![vec![Q::ZERO; N]; N];
        for (mask, c) in self.v.terms() {
            let Some(axes) = axes_of(mask) else {
                continue;
            };
            let value = c * &self.inv;
            match axes.as_slice() {
                [i, j] => {
                    if *i <= N && *j <= N {
                        matrix[i - 1][j - 1] = value.clone();
                        matrix[j - 1][i - 1] = -&value;
                    }
                    planes.push(((*i, *j), value));
                }
                a if a.len() >= 4 => volumes.push((axes, value)),
                _ => {}
            }
        }
        Dimensions {
            planes,
            volumes,
            capacity: rank(matrix, N) / 2,
        }
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

/// A motion of rate `rate` (FORMS-TURNING v1, 78.173 11b): an even
/// generator `B`, a bivector, moves by `R = (1 + B)(1 - B)^-1`, exact in the
/// even algebra of any signature `Cl(p, q)` (the mask `NEG` of the
/// generators squaring to -1). Two motions compose as `R1 R2`; nothing is
/// composed plane by plane. Degrees are a reading only (`tan(theta / 4) =
/// b`). `None` where `1 - B` has no inverse.
pub fn motion<const NEG: usize>(rate: &Mv<NEG>) -> Option<Mv<NEG>> {
    let one = Mv::<NEG>::scalar(Q::ONE);
    Some(one.add(rate).geo(&even_inverse(&one.sub(rate))?))
}

/// The rate a motion is read back to, `B = (R - 1)(R + 1)^-1`, exact.
/// `None` where `R + 1` has no inverse.
pub fn rate_of<const NEG: usize>(motion: &Mv<NEG>) -> Option<Mv<NEG>> {
    let one = Mv::<NEG>::scalar(Q::ONE);
    Some(motion.sub(&one).geo(&even_inverse(&motion.add(&one))?))
}

/// The inverse of an even element `x`: `x x~ = s + w`, a scalar and a
/// grade-4 part whose square is a scalar `l`, inverts as
/// `(s - w) / (s^2 - l)`, so `x^-1 = x~ (x x~)^-1`. `None` where the norm
/// is not of that form or vanishes.
fn even_inverse<const NEG: usize>(x: &Mv<NEG>) -> Option<Mv<NEG>> {
    let rev = x.reverse();
    let n = x.geo(&rev);
    let s = n.scalar_part();
    let w = n.grade(4);
    if !n.sub(&Mv::scalar(s.clone())).sub(&w).is_zero() {
        return None;
    }
    let ww = w.geo(&w);
    let l = ww.scalar_part();
    if !ww.sub(&Mv::scalar(l.clone())).is_zero() {
        return None;
    }
    let det = (&(&s * &s) - &l).recip()?;
    Some(rev.geo(&Mv::scalar(s).sub(&w).scale(&det)))
}

/// A turn split into its winding and its phase (E019; the Diophantine
/// split, #149): the whole turns, in ZZ, and the degrees within the turn,
/// in `[0, 360)`, both carried and never merged. A value in radians, or
/// `frac` alone, drops the winding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Winding {
    /// The whole turns `K`, an integer held as `Q` (the floor of the turns).
    pub whole: Q,
    /// The phase: the degrees within the turn, in `[0, 360)`, exact.
    pub degrees: Q,
}

impl Winding {
    /// The split of `turns` (exact): `K = floor(turns)`, the phase
    /// `360 frac(turns)` degrees.
    pub fn of_turns(turns: &Q) -> Winding {
        let phase = turns.frac();
        Winding {
            whole: turns - &phase,
            degrees: &phase * &Q::int(360),
        }
    }

    /// The turns again, `K + degrees / 360`, exact.
    pub fn turns(&self) -> Q {
        &self.whole + &self.degrees.over(crate::nonzero!(360))
    }

    /// This winding less another, split anew: whole turns and phase of the
    /// difference, both carried.
    pub fn less(&self, other: &Winding) -> Winding {
        Winding::of_turns(&(&self.turns() - &other.turns()))
    }
}

/// The pivot between two unit directions (the pair rule, E019; E028): the
/// relative rotor carrying `a` to `b`, its plane by generators, and its turn
/// on that plane in degrees.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pivot<const N: usize> {
    /// The relative rotor `V = 1 + b a`, exact; `None` for antipodal
    /// directions, which no one plane carries into each other.
    pub rotor: Option<Rotor<N>>,
    /// The plane it turns: its bivector part by generators `(i, j)`, at the
    /// rotor's scale. Empty for one direction and for antipodal ones.
    pub planes: Vec<((usize, usize), Q)>,
    /// The turn on that plane in degrees, `[0, 180]`, certified: a pivot has
    /// capacity 1 at most, so its one angle is its whole reading.
    pub degrees: Encl,
}

/// The pivot between the exact unit directions `a` and `b`: the versor
/// `V = 1 + b a` carries `a` to `b` by the sandwich, turning the one plane
/// `b ^ a`, with no root and no normalised dot product. Its turn is read
/// from the versor's scalar part `c` and scale `V V~`, `cos = 2 c^2 / (V V~) - 1`,
/// as `90 - (180 / pi) asin(cos)` degrees, certified. One direction turns
/// 0 degrees and antipodal directions 180, naming no plane. `None` where
/// either direction is not exactly of unit length.
pub fn pivot<const N: usize>(a: &[Q; N], b: &[Q; N], bits: u32) -> Option<Pivot<N>> {
    use crate::certified::{asin_any, pi};
    let (va, vb) = (crate::cga::base_vector(a), crate::cga::base_vector(b));
    let unit = |v: &Mv| v.geo(v).scalar_part() == Q::ONE;
    if !unit(&va) || !unit(&vb) {
        return None;
    }
    let v = Mv::scalar(Q::ONE).add(&vb.geo(&va));
    let scale = v.geo(&v.reverse()).scalar_part();
    let Some(inv) = scale.recip() else {
        return Some(Pivot {
            rotor: None,
            planes: Vec::new(),
            degrees: Encl::exact(Q::int(180)),
        });
    };
    let rotor = Rotor { v, inv };
    let planes = rotor.dimensions().planes;
    if planes.is_empty() {
        return Some(Pivot {
            rotor: Some(rotor),
            planes,
            degrees: Encl::exact(Q::ZERO),
        });
    }
    let c = rotor.v.scalar_part();
    let cos = &(&Q::int(2) * &(&(&c * &c) * &rotor.inv)) - &Q::ONE;
    let per_radian = Encl::exact(Q::int(180)).mul(&pi(bits).recip()?);
    let degrees = Encl::exact(Q::int(90)).sub(&asin_any(&Encl::exact(cos)).mul(&per_radian));
    Some(Pivot {
        rotor: Some(rotor),
        planes,
        degrees,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dimensional orthogonality, read on 11b's cases (mycelium 78.173):
    /// one plane has capacity 1 and no volume; two orthogonal planes commute,
    /// carry a volume on their four axes and have capacity 2; two planes
    /// sharing an axis do not commute, turn the third plane between them,
    /// carry no volume, and span one plane (capacity 1, as every turning
    /// of three axes does).
    #[test]
    fn a_rotor_reads_its_orthogonal_planes() {
        let q = |n: i64, d: i64| Q::new(n, d).unwrap();
        // The rate b = tan(theta / 4) gives the half-angle direction (1 - b^2, 2b).
        let on = |from, toward, b: Q| {
            Rotor::<4>::drawn(from, toward, &(&Q::ONE - &(&b * &b)), &(&Q::int(2) * &b)).unwrap()
        };
        let one = on(1, 2, q(1, 2)).dimensions();
        assert_eq!(
            (one.planes.len(), one.volumes.len(), one.capacity),
            (1, 0, 1)
        );
        let (a, b) = (on(1, 2, q(1, 2)), on(3, 4, q(1, 3)));
        assert_eq!(
            a.then_after(&b),
            b.then_after(&a),
            "orthogonal planes commute"
        );
        let both = a.then_after(&b).dimensions();
        assert_eq!(both.capacity, 2);
        assert_eq!(
            both.volumes.iter().map(|v| v.0.clone()).collect::<Vec<_>>(),
            vec![vec![1, 2, 3, 4]]
        );
        assert!(orthogonal((1, 2), (3, 4)) && !orthogonal((1, 2), (2, 3)));
        let (c, d) = (on(1, 2, q(1, 2)), on(2, 3, q(1, 3)));
        assert_ne!(
            c.then_after(&d),
            d.then_after(&c),
            "planes sharing an axis do not commute"
        );
        let shared = c.then_after(&d).dimensions();
        let turned: Vec<(usize, usize)> = shared.planes.iter().map(|p| p.0).collect();
        assert_eq!(
            turned,
            vec![(1, 2), (1, 3), (2, 3)],
            "the third plane is turned"
        );
        assert!(shared.volumes.is_empty());
        assert_eq!(shared.capacity, 1);
    }

    fn exact<const N: usize>(v: [i64; N]) -> [Encl; N] {
        v.map(|k| Encl::exact(Q::int(k)))
    }

    #[test]
    fn a_turn_in_degrees_carries_its_generator_exactly() {
        // A right angle turns e1 to e2 exactly: its half-angle direction has
        // equal components on any grid.
        fn right<const N: usize>(from: usize, toward: usize) -> Rotor<N> {
            let turn = Turn {
                from,
                toward,
                degrees: Q::int(90),
            };
            Rotor::of(&turn, 40).unwrap()
        }
        assert_eq!(right::<3>(1, 2).apply(&exact([1, 0, 0])), exact([0, 1, 0]));
        // Turning toward the other way round is the inverse.
        assert_eq!(right::<3>(2, 1).apply(&exact([0, 1, 0])), exact([1, 0, 0]));
        // A right angle in the plane (1, 2) and then one in (2, 3) carry e1
        // to e2 and then e2 to e3.
        let r = right::<3>(2, 3).then_after(&right(1, 2));
        assert_eq!(r.apply(&exact([1, 0, 0])), exact([0, 0, 1]));
        // Turns on the isoclinic planes past the third axis compose in a
        // rotor that holds them, and leave a vector off their planes as it
        // is.
        let far = right::<11>(10, 11).then_after(&right(4, 5));
        let (mut v, mut w) = ([0i64; 11], [0i64; 11]);
        (v[0], v[3], v[9]) = (1, 1, 1);
        (w[0], w[4], w[10]) = (1, 1, 1);
        assert_eq!(far.apply(&exact(v)), exact(w));
        // One axis twice, an axis outside the rotor and the zero direction
        // hold no plane or angle.
        let none = |from, toward| {
            let turn = Turn {
                from,
                toward,
                degrees: Q::int(30),
            };
            Rotor::<3>::of(&turn, 40)
        };
        assert!(none(2, 2).is_none() && none(0, 1).is_none() && none(3, 4).is_none());
        assert!(Rotor::<{ crate::cga::MAX_N + 1 }>::drawn(
            1,
            crate::cga::MAX_N + 1,
            &Q::ONE,
            &Q::ONE
        )
        .is_none());
        assert!(Rotor::<3>::drawn(1, 2, &Q::ZERO, &Q::ZERO).is_none());
        assert!(Direction::drawn(Dy::ZERO, Dy::ZERO).is_none());
        // A direction on a written plane is the same rotor as the drawn one.
        let d = Direction::drawn(Dy::int(3), Dy::int(-2)).unwrap();
        let drawn = Rotor::<3>::drawn(2, 3, &Q::int(3), &Q::int(-2)).unwrap();
        assert_eq!(Rotor::<3>::on::<2, 3>(&d), drawn);
        assert_eq!(
            Rotor::<3>::on::<1, 2>(&Direction::ZERO_ANGLE),
            Rotor::identity()
        );
    }

    #[test]
    fn a_rotor_keeps_length_and_its_inverse_undoes_it() {
        // A rotor at any scale keeps an exact vector's length, and its
        // inverse brings the vector back.
        let s = Rotor::<3>::drawn(1, 2, &Q::int(2), &Q::int(5))
            .unwrap()
            .then_after(&Rotor::drawn(2, 3, &Q::int(1), &crate::ratio!(-1, 3)).unwrap())
            .then_after(&Rotor::drawn(1, 3, &Q::int(3), &Q::int(1)).unwrap());
        let v = exact([3, -4, 7]);
        let turned = s.apply(&v);
        let len = |v: &[Encl; 3]| v.iter().fold(Q::ZERO, |acc, c| &acc + &(&c.lo * &c.lo));
        assert_eq!(len(&turned), len(&v));
        assert_eq!(s.inverse().apply(&turned), v);
        // The rotor and its inverse compose to the positive scalar V V~.
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
            let image = Rotor::<2>::of(&turn, 60).unwrap().apply(&exact([1, 0]));
            assert_eq!(image, [c, s].map(Encl::exact));
        }
    }

    /// Scaling a rotor's rate keeps its plane: no rate is the identity, the
    /// whole rate is the rotor itself, a half rate turns less on the same
    /// plane, and a half turn has no rate.
    #[test]
    fn a_rotors_rate_scales_on_its_plane() {
        let q = |a: i64, b: i64, c: i64| [Q::int(a), Q::int(b), Q::int(c)].map(Encl::exact);
        let r = Rotor::<3>::drawn(1, 2, &Q::new(3, 4).unwrap(), &Q::ONE).unwrap();
        let x = q(1, 0, 0);
        assert_eq!(r.rate_scaled(&Q::ZERO).unwrap().apply(&x), x);
        assert_eq!(r.rate_scaled(&Q::ONE).unwrap().apply(&x), r.apply(&x));
        let half = r.rate_scaled(&Q::new(1, 2).unwrap()).unwrap();
        let (full, part) = (r.apply(&x), half.apply(&x));
        assert!(part[0].mid() > full[0].mid() && part[2].mid() == Q::ZERO);
        let flip = Rotor::<3>::drawn(1, 2, &Q::ZERO, &Q::ONE).unwrap();
        assert!(flip.rate_scaled(&Q::new(1, 2).unwrap()).is_none());
        // The rate on the turn's plane is the tangent of its half-angle:
        // the drawn direction (3/4, 1) has the rate 4/3 on plane (1, 2),
        // signed by the versor's `c - s e1 e2`.
        assert_eq!(r.rates().unwrap(), vec![((1, 2), Q::new(-4, 3).unwrap())]);
    }

    /// FORMS-TURNING v1, 78.173 11b, in `Cl(3, 2)` (`e+1 e+2 e+3` square to
    /// +1, `e-1 e-2` to -1): planes sharing a unit gain the third plane,
    /// order-dependent, with no volume; disjoint planes commute and carry the
    /// volume; on one plane the rates compose by the tangent law (elliptic)
    /// or the flow law (hyperbolic). Every motion reads back to its rate and
    /// has unit norm.
    #[test]
    fn turnings_compose_as_fixture_11b() {
        const NEG: usize = 0b11000;
        type M = Mv<NEG>;
        let q = |n: i64, d: i64| Q::new(n, d).unwrap();
        let b = |mask: usize, v: Q| M::blade(mask, v);
        let (p1, p2, p3, n1, n2) = (1usize, 2, 4, 8, 16);
        let unit = |r: &M| r.geo(&r.reverse()) == M::scalar(Q::ONE);
        let check = |rate: &M| {
            let r = motion(rate).unwrap();
            assert!(unit(&r));
            assert_eq!(rate_of(&r).unwrap(), *rate);
            r
        };
        // Fixture i: planes sharing a unit.
        let r1 = check(&b(p1 | p2, q(1, 2)));
        assert_eq!(r1, M::scalar(q(3, 5)).add(&b(p1 | p2, q(4, 5))));
        let r2 = check(&b(p2 | p3, q(1, 3)));
        assert_eq!(r2, M::scalar(q(4, 5)).add(&b(p2 | p3, q(3, 5))));
        let r12 = r1.geo(&r2);
        let expect = M::scalar(q(12, 25))
            .add(&b(p1 | p2, q(16, 25)))
            .add(&b(p1 | p3, q(12, 25)))
            .add(&b(p2 | p3, q(9, 25)));
        assert_eq!(r12, expect);
        let back = b(p1 | p2, q(16, 37))
            .add(&b(p1 | p3, q(12, 37)))
            .add(&b(p2 | p3, q(9, 37)));
        assert_eq!(rate_of(&r12).unwrap(), back);
        let r21 = r2.geo(&r1);
        assert_eq!(r21.coeff(p1 | p3), q(-12, 25));
        assert_eq!(rate_of(&r21).unwrap().coeff(p1 | p3), q(-12, 37));
        assert_eq!(r12.sub(&r21), b(p1 | p3, q(24, 25)));
        assert!(r12.grade(4).is_zero());
        // Fixture ii: disjoint planes.
        let s1 = check(&b(p1 | p2, q(1, 2)));
        let s2 = check(&b(n1 | n2, q(1, 3)));
        let s12 = s1.geo(&s2);
        assert_eq!(s12, s2.geo(&s1));
        let expect = M::scalar(q(12, 25))
            .add(&b(p1 | p2, q(16, 25)))
            .add(&b(n1 | n2, q(9, 25)))
            .add(&b(p1 | p2 | n1 | n2, q(12, 25)));
        assert_eq!(s12, expect);
        assert_eq!(
            rate_of(&s12).unwrap(),
            b(p1 | p2, q(4, 7)).add(&b(n1 | n2, q(3, 7)))
        );
        // Fixture iii: one plane, elliptic and hyperbolic.
        let e = check(&b(p1 | p2, q(1, 2))).geo(&check(&b(p1 | p2, q(1, 3))));
        assert_eq!(rate_of(&e).unwrap(), b(p1 | p2, Q::ONE));
        let h = check(&b(p1 | n1, q(1, 2))).geo(&check(&b(p1 | n1, q(1, 3))));
        assert_eq!(rate_of(&h).unwrap(), b(p1 | n1, q(5, 7)));
    }

    /// A turn splits into its winding and phase and back exactly, negative
    /// turns included, and a difference is split anew.
    #[test]
    fn a_turn_splits_into_winding_and_phase() {
        let q = |n: i64, d: i64| Q::new(n, d).unwrap();
        let w = Winding::of_turns(&q(9, 4));
        assert_eq!(
            (w.whole.clone(), w.degrees.clone()),
            (Q::int(2), Q::int(90))
        );
        assert_eq!(w.turns(), q(9, 4));
        let back = Winding::of_turns(&q(-1, 3));
        assert_eq!(
            (back.whole.clone(), back.degrees.clone()),
            (Q::int(-1), Q::int(240))
        );
        let d = w.less(&back);
        assert_eq!((d.whole, d.degrees), (Q::int(2), Q::int(210)));
    }

    /// The pivot carries one unit direction to the other exactly, names
    /// its one plane and reads its turn: 0 degrees for one direction, 90
    /// for perpendicular ones on their plane, 180 for antipodal ones (no
    /// plane), the same turn either way round on the reversed plane, and a
    /// direction not of unit length refused.
    #[test]
    fn the_pivot_carries_a_direction_and_reads_its_turn() {
        let q = |a: i64, b: i64, c: i64| [Q::int(a), Q::int(b), Q::int(c)];
        let within = |e: &Encl, x: i64| e.lo <= Q::int(x) && Q::int(x) <= e.hi;
        let same = pivot(&q(1, 0, 0), &q(1, 0, 0), 64).unwrap();
        assert!(same.planes.is_empty() && same.degrees == Encl::exact(Q::ZERO));
        let right = pivot(&q(1, 0, 0), &q(0, 1, 0), 64).unwrap();
        assert!(within(&right.degrees, 90), "{:?}", right.degrees);
        assert_eq!(
            right.planes.iter().map(|p| p.0).collect::<Vec<_>>(),
            vec![(1, 2)]
        );
        let turned = right
            .rotor
            .as_ref()
            .unwrap()
            .apply(&q(1, 0, 0).map(Encl::exact));
        assert_eq!(
            turned,
            q(0, 1, 0).map(Encl::exact),
            "the pivot carries a to b"
        );
        let back = pivot(&q(1, 0, 0), &q(-1, 0, 0), 64).unwrap();
        assert!(back.rotor.is_none() && back.degrees == Encl::exact(Q::int(180)));
        // A rational unit pair (3/5, 4/5, 0) and (0, 3/5, 4/5).
        let (u, w) = (
            [Q::new(3, 5).unwrap(), Q::new(4, 5).unwrap(), Q::ZERO],
            [Q::ZERO, Q::new(3, 5).unwrap(), Q::new(4, 5).unwrap()],
        );
        let (uw, wu) = (pivot(&u, &w, 64).unwrap(), pivot(&w, &u, 64).unwrap());
        assert!(uw.degrees.lo <= wu.degrees.hi && wu.degrees.lo <= uw.degrees.hi);
        let reversed: Vec<Q> = wu.planes.iter().map(|p| -&p.1).collect();
        assert_eq!(
            uw.planes.iter().map(|p| p.1.clone()).collect::<Vec<_>>(),
            reversed
        );
        assert!(pivot(&q(2, 0, 0), &q(1, 0, 0), 64).is_none());
    }
}
