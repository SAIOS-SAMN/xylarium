//! Rational half-angles and rotors: frames composed exactly, with no pi and no
//! root.
//!
//! Inputs: a half-angle's tangent ([`Q`]) or an angle in turns fixed on a
//! grid of `2^-bits` (via `certified::cos_sin_turns`); vectors to turn as
//! `certified::Encl` triples.
//!
//! Outputs:
//! - [`Half`], a half-angle held as its direction `(c, s)` at any scale (its
//!   tangent `s / c`, including `c = 0`, a half-angle of a quarter turn):
//!   `compose` adds angles and `cos_sin` gives the whole angle's cosine and
//!   sine exactly (the share of a sphere within an angle is `measure`'s).
//! - [`Rotor`], a rational quaternion `(w, x, y, z)` at any scale, the even
//!   part of Cl(3): `about(k, h)` turns about axis `k`, `then_after` composes
//!   by multiplication, `inverse` is the conjugate, and `apply` turns an
//!   enclosed vector by the rotor's own action `v + (w t + u x t) / n`,
//!   `t = 2 u x v`, with no matrix.
//!
//! Invariants: composition is exact for turns about any axes (the one-axis
//! case is the tangent law `(a + b) / (1 - ab)`). `Half::of_turns` rounds the
//! direction down to its grid once; after that every frame is exact. A turn's
//! advance in time is uniform in degrees and is read as an enclosure by the
//! caller.

use crate::certified::{cos_sin_turns, down, Encl};
use crate::Q;

/// A half-angle held as its direction (c, s) at any scale; the angle it
/// represents is twice the half-angle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Half {
    /// The direction's cosine-like component (any scale).
    pub c: Q,
    /// The direction's sine-like component (same scale as `c`).
    pub s: Q,
}

impl Half {
    /// The zero angle, direction `(1, 0)`.
    pub fn zero() -> Half {
        Half {
            c: Q::ONE,
            s: Q::ZERO,
        }
    }

    /// The half-angle whose tangent is `t`.
    pub fn of_t(t: Q) -> Half {
        Half { c: Q::ONE, s: t }
    }

    /// An angle of `turns` (a drawn or declared direction, not a turning in
    /// time), held as the rational direction of its half fixed on a grid of
    /// 2^-bits: the frame is then exactly that drawn direction.
    pub fn of_turns(turns: &Q, bits: u32) -> Half {
        let half = turns.checked_div(&Q::int(2)).unwrap_or(Q::ZERO);
        let (c, s) = cos_sin_turns(&Encl::exact(half));
        Half {
            c: down(&c.mid(), bits),
            s: down(&s.mid(), bits),
        }
    }

    /// The sum of the two angles: the directions multiplied as complex
    /// numbers.
    pub fn compose(&self, o: &Half) -> Half {
        Half {
            c: &(&self.c * &o.c) - &(&self.s * &o.s),
            s: &(&self.s * &o.c) + &(&self.c * &o.s),
        }
    }

    fn norm(&self) -> Q {
        &(&self.c * &self.c) + &(&self.s * &self.s)
    }

    /// The whole angle's cosine and sine, exact:
    /// `((c^2 - s^2) / n, 2 c s / n)` with `n = c^2 + s^2`.
    pub fn cos_sin(&self) -> (Q, Q) {
        let n = self.norm();
        let cos = (&(&self.c * &self.c) - &(&self.s * &self.s))
            .checked_div(&n)
            .unwrap_or(Q::ONE);
        let sin = (&Q::int(2) * &(&self.c * &self.s))
            .checked_div(&n)
            .unwrap_or(Q::ZERO);
        (cos, sin)
    }
}

/// A rotor: a rational quaternion (w, x, y, z), any scale.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rotor {
    /// The scalar part.
    pub w: Q,
    /// The component on the bivector that turns about the x axis.
    pub x: Q,
    /// The component on the bivector that turns about the y axis.
    pub y: Q,
    /// The component on the bivector that turns about the z axis.
    pub z: Q,
}

impl Rotor {
    /// The identity rotor `(1, 0, 0, 0)`: no turn.
    pub fn identity() -> Rotor {
        Rotor {
            w: Q::ONE,
            x: Q::ZERO,
            y: Q::ZERO,
            z: Q::ZERO,
        }
    }

    /// A turn about axis `k` (0 x, 1 y, 2 z) by twice the half `h`.
    pub fn about(k: usize, h: &Half) -> Rotor {
        let mut r = Rotor {
            w: h.c.clone(),
            x: Q::ZERO,
            y: Q::ZERO,
            z: Q::ZERO,
        };
        match k {
            0 => r.x = h.s.clone(),
            1 => r.y = h.s.clone(),
            _ => r.z = h.s.clone(),
        }
        r
    }

    /// The inverse turn: the conjugate `(w, -x, -y, -z)`. A rotor is held at
    /// any scale, so the conjugate composed with the rotor is the scalar
    /// `w^2 + x^2 + y^2 + z^2`, the identity up to that scale, exactly.
    pub fn inverse(&self) -> Rotor {
        Rotor {
            w: self.w.clone(),
            x: -self.x.clone(),
            y: -self.y.clone(),
            z: -self.z.clone(),
        }
    }

    /// `self` after `o`: the product rotates by `o` first, then by `self`.
    pub fn then_after(&self, o: &Rotor) -> Rotor {
        let (a, b) = (self, o);
        Rotor {
            w: &(&(&a.w * &b.w) - &(&a.x * &b.x)) - &(&(&a.y * &b.y) + &(&a.z * &b.z)),
            x: &(&(&a.w * &b.x) + &(&a.x * &b.w)) + &(&(&a.y * &b.z) - &(&a.z * &b.y)),
            y: &(&(&a.w * &b.y) - &(&a.x * &b.z)) + &(&(&a.y * &b.w) + &(&a.z * &b.x)),
            z: &(&(&a.w * &b.z) + &(&a.x * &b.y)) - &(&(&a.y * &b.x) - &(&a.z * &b.w)),
        }
    }

    /// An exact vector turned by the rotor: `v + (w t + u x t) / n` with
    /// `t = 2 u x v`, `u` the rotor's bivector part and `n` its squared norm,
    /// so the rotor's scale does not matter (a zero rotor leaves `v`).
    fn turn(&self, v: &[Q; 3]) -> [Q; 3] {
        let cross = |a: &[Q; 3], b: &[Q; 3]| -> [Q; 3] {
            [
                &(&a[1] * &b[2]) - &(&a[2] * &b[1]),
                &(&a[2] * &b[0]) - &(&a[0] * &b[2]),
                &(&a[0] * &b[1]) - &(&a[1] * &b[0]),
            ]
        };
        let u = [self.x.clone(), self.y.clone(), self.z.clone()];
        let n = &(&(&self.w * &self.w) + &(&u[0] * &u[0])) + &(&(&u[1] * &u[1]) + &(&u[2] * &u[2]));
        let t = cross(&u, v).map(|c| c.scale(2));
        let ut = cross(&u, &t);
        std::array::from_fn(|i| {
            let k = (&(&self.w * &t[i]) + &ut[i])
                .checked_div(&n)
                .unwrap_or(Q::ZERO);
            &v[i] + &k
        })
    }

    /// An enclosed vector turned by the rotor: its exact midpoint turned
    /// (`turn`), each component widened by the sum of the vector's radii,
    /// since a turn moves no vector by more than its length. An exact
    /// vector stays exact.
    pub fn apply(&self, v: &[Encl; 3]) -> [Encl; 3] {
        let mid: [Q; 3] = std::array::from_fn(|i| v[i].mid());
        let rad = v
            .iter()
            .fold(Q::ZERO, |acc, e| &acc + &(&e.hi - &e.lo))
            .checked_div(&Q::int(2))
            .unwrap_or(Q::ZERO);
        let turned = self.turn(&mid);
        std::array::from_fn(|i| Encl {
            lo: &turned[i] - &rad,
            hi: &turned[i] + &rad,
        })
    }
}

/// The angle between two directions `a` and `b` (any lengths, exact), in
/// radians as a certified enclosure: its cosine is `a . b / |a| |b|` (the
/// lengths' product a certified root of `|a|^2 |b|^2`), and the angle is
/// `pi / 2 - asin(cos)` for a non-negative cosine and `pi / 2 + asin(-cos)`
/// otherwise. The directions stay exact; only the angle, which has no
/// rational form in general, is an enclosure. `None` for a zero direction.
pub fn angle_between(a: &[Q; 3], b: &[Q; 3], bits: u32) -> Option<crate::certified::Encl> {
    use crate::certified::{asin_any, pi, sqrt, Encl};
    let dot = a.iter().zip(b).fold(Q::ZERO, |acc, (x, y)| &acc + &(x * y));
    let norm2 = |v: &[Q; 3]| v.iter().fold(Q::ZERO, |acc, x| &acc + &(x * x));
    let (na, nb) = (norm2(a), norm2(b));
    if na.is_zero() || nb.is_zero() {
        return None;
    }
    let len = sqrt(&(&na * &nb), bits)?;
    let cos = Encl::exact(dot.clone()).mul(&len.recip()?);
    let quarter = pi(bits).mul(&Encl::exact(Q::ratio(1, 2)));
    Some(if dot.signum() >= 0 {
        quarter.sub(&asin_any(&cos))
    } else {
        quarter.add(&asin_any(&cos.neg()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halves_and_rotors_compose_exactly() {
        // Two right angles compose to a half turn: cos -1, sin 0.
        let right = Half::of_t(Q::ONE);
        let half_turn = right.compose(&right);
        assert_eq!(half_turn.cos_sin(), (-Q::ONE, Q::ZERO));
        // A quarter turn about z, then one about x, carries x to y and then y
        // to z.
        let r = Rotor::about(0, &right).then_after(&Rotor::about(2, &right));
        let e = |a: i64, b: i64, c: i64| [a, b, c].map(|k| Encl::exact(Q::int(k)));
        assert_eq!(r.apply(&e(1, 0, 0)), e(0, 0, 1));
        // A turn keeps an exact vector's length at any scale of the rotor,
        // and its inverse brings the vector back.
        let s = Rotor {
            w: Q::int(2),
            x: Q::int(1),
            y: Q::ratio(-1, 3),
            z: Q::int(5),
        };
        let v = e(3, -4, 7);
        let turned = s.apply(&v);
        let len = |v: &[Encl; 3]| v.iter().fold(Q::ZERO, |acc, c| &acc + &(&c.lo * &c.lo));
        assert_eq!(len(&turned), len(&v));
        assert_eq!(s.inverse().apply(&turned), v);
        // An enclosed vector's turn encloses the turn of every point in it.
        let wide = [
            Encl {
                lo: Q::ratio(29, 10),
                hi: Q::int(3),
            },
            Encl::exact(Q::int(-4)),
            Encl {
                lo: Q::int(7),
                hi: Q::ratio(71, 10),
            },
        ];
        let out = s.apply(&wide);
        for p in [
            e(3, -4, 7),
            [Q::ratio(29, 10), Q::int(-4), Q::ratio(71, 10)].map(Encl::exact),
        ] {
            let at = s.apply(&p);
            assert!((0..3).all(|i| out[i].contains(&at[i].lo)));
        }
    }

    #[test]
    fn a_rotor_and_its_inverse_compose_to_its_scale() {
        let r = Rotor {
            w: Q::int(2),
            x: Q::int(1),
            y: Q::ratio(-1, 3),
            z: Q::int(5),
        };
        let n = &(&(&r.w * &r.w) + &(&r.x * &r.x)) + &(&(&r.y * &r.y) + &(&r.z * &r.z));
        for p in [r.then_after(&r.inverse()), r.inverse().then_after(&r)] {
            assert_eq!((p.w, p.x, p.y, p.z), (n.clone(), Q::ZERO, Q::ZERO, Q::ZERO));
        }
    }

    #[test]
    fn the_angle_between_directions_is_enclosed() {
        let q = |a: i64, b: i64, c: i64| [Q::int(a), Q::int(b), Q::int(c)];
        let pi = crate::certified::pi(64);
        let half = pi.mul(&crate::certified::Encl::exact(Q::ratio(1, 2)));
        // Parallel, perpendicular and opposed, at any lengths.
        let same = angle_between(&q(2, 0, 0), &q(5, 0, 0), 64).unwrap();
        assert!(same.contains(&Q::ZERO) || same.hi < Q::ratio(1, 1_000_000));
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
