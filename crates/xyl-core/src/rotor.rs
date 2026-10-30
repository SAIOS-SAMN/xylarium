//! Half-angles and rotors. On a sphere the volume within an angle theta of
//! an axis is sin^2(theta / 2): the half-angle is the coordinate in which a
//! sphere's volume is rational. A [`Half`] is a half-angle held as its
//! direction (c, s) at any scale - its tangent t = s / c, never a float, a
//! right angle included (c = 0) - and a [`Rotor`] is a rational quaternion,
//! the even part of Cl(3): turns about different axes compose by
//! multiplying, exactly, with no pi and no root (the half-angle law
//! (a + b) / (1 - ab) is its one-axis case). Frames are composed here; a
//! turn's advance in time stays uniform in degrees and is read as an
//! enclosure.

use crate::certified::{cos_sin_turns, down, Encl};
use crate::Q;

/// A half-angle as its direction (c, s), any scale: the angle is twice it.
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

    /// From the half-angle's tangent t.
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

    /// The two halves added: the angles add.
    pub fn compose(&self, o: &Half) -> Half {
        Half {
            c: &(&self.c * &o.c) - &(&self.s * &o.s),
            s: &(&self.s * &o.c) + &(&self.c * &o.s),
        }
    }

    fn norm(&self) -> Q {
        &(&self.c * &self.c) + &(&self.s * &self.s)
    }

    /// The whole angle's cosine and sine, exact.
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

    /// The volume it holds on an ordinary sphere (S^2): the share within the
    /// whole angle of an axis, sin^2 of the half, exact; other dimensions
    /// read their own curve (measure.rs).
    pub fn share(&self) -> Q {
        (&self.s * &self.s)
            .checked_div(&self.norm())
            .unwrap_or(Q::ZERO)
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

    /// `self` after `o`: applying the product turns by `o`, then `self`.
    pub fn then_after(&self, o: &Rotor) -> Rotor {
        let (a, b) = (self, o);
        Rotor {
            w: &(&(&a.w * &b.w) - &(&a.x * &b.x)) - &(&(&a.y * &b.y) + &(&a.z * &b.z)),
            x: &(&(&a.w * &b.x) + &(&a.x * &b.w)) + &(&(&a.y * &b.z) - &(&a.z * &b.y)),
            y: &(&(&a.w * &b.y) - &(&a.x * &b.z)) + &(&(&a.y * &b.w) + &(&a.z * &b.x)),
            z: &(&(&a.w * &b.z) + &(&a.x * &b.y)) - &(&(&a.y * &b.x) - &(&a.z * &b.w)),
        }
    }

    /// Its rotation as an exact rational matrix.
    pub fn matrix(&self) -> [[Q; 3]; 3] {
        let (w, x, y, z) = (&self.w, &self.x, &self.y, &self.z);
        let (ww, xx, yy, zz) = (w * w, x * x, y * y, z * z);
        let n = &(&ww + &xx) + &(&yy + &zz);
        let k = |v: Q| v.checked_div(&n).unwrap_or(Q::ZERO);
        let two = Q::int(2);
        let tw = |a: &Q, b: &Q| &two * &(a * b);
        [
            [
                k(&(&ww + &xx) - &(&yy + &zz)),
                k(&tw(x, y) - &tw(w, z)),
                k(&tw(x, z) + &tw(w, y)),
            ],
            [
                k(&tw(x, y) + &tw(w, z)),
                k(&(&ww - &xx) + &(&yy - &zz)),
                k(&tw(y, z) - &tw(w, x)),
            ],
            [
                k(&tw(x, z) - &tw(w, y)),
                k(&tw(y, z) + &tw(w, x)),
                k(&(&ww - &xx) - &(&yy - &zz)),
            ],
        ]
    }

    /// A vector turned by it.
    pub fn apply(&self, v: &[Encl; 3]) -> [Encl; 3] {
        let m = self.matrix();
        std::array::from_fn(|i| {
            (0..3).fold(Encl::exact(Q::ZERO), |acc, j| {
                acc.add(&v[j].mul(&Encl::exact(m[i][j].clone())))
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halves_compose_exactly_and_hold_a_spheres_volume() {
        // two right angles make a half turn: cos -1, sin 0, the whole sphere
        let right = Half::of_t(Q::ONE);
        let half_turn = right.compose(&right);
        assert_eq!(half_turn.cos_sin(), (-Q::ONE, Q::ZERO));
        assert_eq!(half_turn.share(), Q::ONE);
        assert_eq!(right.share(), Q::new(1, 2).unwrap());
        // a quarter turn about z then x carries x to y, then y to z
        let r = Rotor::about(0, &right).then_after(&Rotor::about(2, &right));
        let m = r.matrix();
        assert_eq!(
            (m[0][0].clone(), m[1][0].clone(), m[2][0].clone()),
            (Q::ZERO, Q::ZERO, Q::ONE)
        );
    }
}
