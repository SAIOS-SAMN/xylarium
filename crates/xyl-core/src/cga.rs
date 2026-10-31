//! Conformal geometric algebra `Cl(n+1, 1)` over exact rationals.
//!
//! Inputs: [`Q`] coefficients and coordinates; no other module.
//!
//! Outputs: the multivector [`Mv`] (geometric product `geo`, `reverse`,
//! `involute`, `grade`, `norm2`, `sandwich`, `twisted`, `parity`,
//! `wedge_vector`, `projective_eq`); the null basis `e_o`, `e_inf`; points
//! (`point_at`, `base_at`), dual spheres (`sphere_through`); the versors
//! `translator_at`, `rotator_on_blade`, `dilator`; `projective_ratio`;
//! `keeping` (the part of a generator that keeps a sphere); and [`Source`], a
//! null-generator flow `1 + t N` on a sphere.
//!
//! Layout: a multivector is a sparse sum of blades, each a bit mask over the
//! generators with a `Q` coefficient, ascending by mask. The first three base
//! axes are bits 0-2, `e+` bit 3 and `e-` bit 4 (`e+^2 = +1`, `e-^2 = -1`), and
//! base axis `i` past the third is bit `i + 1`, so the null basis keeps its
//! bits as `n` grows (up to `MAX_N`).
//!
//! The conformal model: a point of `Q^n` is `X(x) = x + 1/2 |x|^2 e_inf + e_o`
//! on the null cone; a dual sphere about `c` through `p` is
//! `X(c) + (X(c) . X(p)) e_inf` (its radius enters only through the points'
//! product); rigid and conformal motions are versors acting by the sandwich
//! `V X V~`.
//!
//! Invariants: all arithmetic is exact. Versors are kept unnormalised and
//! every reading of them is projective (equality up to a non-zero rational
//! factor), so no root or inverse is taken. Functions that can fail (a point
//! at infinity, an axis past `MAX_N`, a non-proportional ratio, a null
//! sphere) return `None`.
//!
//! ```
//! use xyl_core::Q;
//! use xyl_core::cga::{base_at, point_at, translator_at};
//!
//! let x = point_at(&[Q::int(1), Q::int(2), Q::int(3)]);
//! assert!(x.norm2().is_zero()); // conformal points are null
//!
//! let t = translator_at(&[Q::int(1), Q::int(0), Q::int(-1)]);
//! let moved = t.sandwich(&x);
//! let expected = point_at(&[Q::int(2), Q::int(2), Q::int(2)]);
//! assert!(moved.projective_eq(&expected));
//! assert!(moved.projective_eq(&expected.scale(&Q::int(5))));
//! assert!(!moved.projective_eq(&x));
//! assert_eq!(base_at(&moved, 3), Some(vec![Q::int(2), Q::int(2), Q::int(2)]));
//! ```

use crate::Q;

/// The first base axis.
pub const E1: usize = 0b00001;
/// The second base axis.
pub const E2: usize = 0b00010;
/// The third base axis.
pub const E3: usize = 0b00100;
/// `e+`, squaring to `+1`.
pub const EP: usize = 0b01000;
/// `e-`, squaring to `-1`.
pub const EM: usize = 0b10000;

/// The highest base axis a machine-word mask holds.
pub const MAX_N: usize = (usize::BITS - 3) as usize;

/// The mask of base axis `i`, counted from 1. `None` for 0 and past [`MAX_N`].
pub fn base_axis(i: usize) -> Option<usize> {
    match i {
        0 => None,
        1..=3 => Some(1 << (i - 1)),
        _ if i <= MAX_N => Some(1 << (i + 1)),
        _ => None,
    }
}

fn half() -> Q {
    Q::new(1, 2).unwrap_or(Q::ONE)
}

/// The sign of reordering the generators of `a` past those of `b`.
fn reorder_sign(a: usize, b: usize) -> bool {
    let mut hi = a >> 1;
    let mut swaps = 0u32;
    while hi != 0 {
        swaps += (hi & b).count_ones();
        hi >>= 1;
    }
    swaps & 1 == 1
}

fn grade_of(mask: usize) -> u32 {
    mask.count_ones()
}

/// An exact multivector: its non-zero terms, ascending by blade mask.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mv {
    c: Vec<(usize, Q)>,
}

impl Mv {
    fn of_terms(mut terms: Vec<(usize, Q)>) -> Mv {
        terms.sort_unstable_by_key(|(m, _)| *m);
        let mut c: Vec<(usize, Q)> = Vec::with_capacity(terms.len());
        for (m, v) in terms {
            match c.last_mut() {
                Some((last, sum)) if *last == m => *sum += &v,
                _ => c.push((m, v)),
            }
        }
        c.retain(|(_, v)| !v.is_zero());
        Mv { c }
    }

    fn map_terms(&self, f: impl Fn(usize, &Q) -> Q) -> Mv {
        Mv {
            c: self
                .c
                .iter()
                .map(|(m, v)| (*m, f(*m, v)))
                .filter(|(_, v)| !v.is_zero())
                .collect(),
        }
    }

    /// The zero multivector (no terms).
    pub fn zero() -> Mv {
        Mv { c: Vec::new() }
    }

    /// The grade-0 multivector `v`.
    pub fn scalar(v: Q) -> Mv {
        Mv::blade(0, v)
    }

    /// The single blade with generator bit mask `mask` (see [`E1`], [`EP`],
    /// [`base_axis`]) and coefficient `v`; zero when `v` is zero.
    pub fn blade(mask: usize, v: Q) -> Mv {
        Mv::of_terms(vec![(mask, v)])
    }

    /// The non-zero terms as `(blade mask, coefficient)`, ascending by mask.
    pub fn terms(&self) -> impl Iterator<Item = (usize, &Q)> {
        self.c.iter().map(|(m, v)| (*m, v))
    }

    /// The number of non-zero terms.
    pub fn term_count(&self) -> usize {
        self.c.len()
    }

    /// The coefficient of one blade (zero where absent).
    pub fn coeff(&self, mask: usize) -> Q {
        self.c
            .iter()
            .find(|(m, _)| *m == mask)
            .map(|(_, v)| v.clone())
            .unwrap_or(Q::ZERO)
    }

    /// The grade-0 coefficient.
    pub fn scalar_part(&self) -> Q {
        self.coeff(0)
    }

    /// Whether every coefficient is zero.
    pub fn is_zero(&self) -> bool {
        self.c.is_empty()
    }

    /// The sum `self + o`.
    pub fn add(&self, o: &Mv) -> Mv {
        Mv::of_terms(self.c.iter().chain(&o.c).cloned().collect())
    }

    /// The difference `self - o`.
    pub fn sub(&self, o: &Mv) -> Mv {
        self.add(&o.scale(&-Q::ONE))
    }

    /// Every coefficient multiplied by the rational `s`.
    pub fn scale(&self, s: &Q) -> Mv {
        self.map_terms(|_, v| v * s)
    }

    /// The geometric product.
    pub fn geo(&self, o: &Mv) -> Mv {
        let mut terms = Vec::with_capacity(self.c.len() * o.c.len());
        for (i, x) in &self.c {
            for (j, y) in &o.c {
                let negative = reorder_sign(*i, *j) ^ ((i & j & EM) != 0);
                let term = x * y;
                terms.push((i ^ j, if negative { -term } else { term }));
            }
        }
        Mv::of_terms(terms)
    }

    /// The reverse: a blade of grade `g` takes the sign `(-1)^(g(g-1)/2)`.
    pub fn reverse(&self) -> Mv {
        self.map_terms(|m, v| {
            if (grade_of(m) / 2) % 2 == 1 {
                -v
            } else {
                v.clone()
            }
        })
    }

    /// The grade involution: a blade of grade `g` takes the sign `(-1)^g`.
    pub fn involute(&self) -> Mv {
        self.map_terms(|m, v| if grade_of(m) % 2 == 1 { -v } else { v.clone() })
    }

    /// The part of grade `g`.
    pub fn grade(&self, g: u32) -> Mv {
        Mv {
            c: self
                .c
                .iter()
                .filter(|(m, _)| grade_of(*m) == g)
                .cloned()
                .collect(),
        }
    }

    /// `<M M>_0`: for a vector, its metric square.
    pub fn norm2(&self) -> Q {
        self.geo(self).scalar_part()
    }

    /// The versor sandwich `V x V~`.
    pub fn sandwich(&self, x: &Mv) -> Mv {
        self.geo(x).geo(&self.reverse())
    }

    /// The twisted adjoint `V^ x V~`: the sandwich for an even versor, its
    /// negation for an odd one (the action of a reflection on a vector).
    pub fn twisted(&self, x: &Mv) -> Mv {
        self.involute().geo(x).geo(&self.reverse())
    }

    /// `Some(true)` if every occupied grade is odd, `Some(false)` if every
    /// one is even, `None` for mixed parity or zero.
    pub fn parity(&self) -> Option<bool> {
        let mut odd = None;
        for (m, _) in &self.c {
            let o = grade_of(*m) % 2 == 1;
            match odd {
                None => odd = Some(o),
                Some(p) if p != o => return None,
                Some(_) => {}
            }
        }
        odd
    }

    /// The outer product `A ^ v` of a grade-`g` blade and a vector.
    pub fn wedge_vector(&self, g: u32, v: &Mv) -> Mv {
        self.geo(v).grade(g + 1)
    }

    /// Projective equality: `self = lambda o` for some non-zero rational.
    pub fn projective_eq(&self, o: &Mv) -> bool {
        projective_ratio(self, o).is_some_and(|l| !l.is_zero())
    }
}

/// `e_o = 1/2 (e- - e+)`, the origin.
pub fn e_o() -> Mv {
    Mv::blade(EM, half()).sub(&Mv::blade(EP, half()))
}

/// `e_inf = e- + e+`, the point at infinity.
pub fn e_inf() -> Mv {
    Mv::blade(EM, Q::ONE).add(&Mv::blade(EP, Q::ONE))
}

/// The base vector `sum x_i e_i`.
pub fn base_vector(x: &[Q]) -> Mv {
    x.iter()
        .enumerate()
        .filter_map(|(i, xi)| base_axis(i + 1).map(|a| Mv::blade(a, xi.clone())))
        .fold(Mv::zero(), |m, b| m.add(&b))
}

/// The conformal point `X(x) = x + 1/2 |x|^2 e_inf + e_o`.
pub fn point_at(x: &[Q]) -> Mv {
    let v = base_vector(x);
    let x2 = v.norm2();
    v.add(&e_inf().scale(&(&half() * &x2))).add(&e_o())
}

/// The dual sphere about `c` through the point `p`: `C + (C . P) e_inf` with
/// `C = X(c)`, `P = X(p)`, and `C . P` the scalar part of their geometric
/// product. The radius enters only through that product (for null points
/// it is the negative half of the squared distance), never as a written
/// square. A point `Y` lies on the sphere exactly when `Y . sigma = 0`.
pub fn sphere_through(c: &[Q], p: &[Q]) -> Mv {
    let centre = point_at(c);
    let through = point_at(p);
    let cp = centre.geo(&through).scalar_part();
    centre.add(&e_inf().scale(&cp))
}

/// The first `n` base coordinates of a conformal point or sphere, by its
/// weight `-(P . e_inf)`. `None` at infinity.
pub fn base_at(p: &Mv, n: usize) -> Option<Vec<Q>> {
    let alpha = -p.geo(&e_inf()).scalar_part();
    (1..=n)
        .map(|i| p.coeff(base_axis(i)?).checked_div(&alpha))
        .collect()
}

/// The translator by `d`: `1 + 1/2 e_inf d`, carrying `X(x)` to `X(x + d)`.
pub fn translator_at(d: &[Q]) -> Mv {
    Mv::scalar(Q::ONE).add(&e_inf().geo(&base_vector(d)).scale(&half()))
}

/// The unnormalised rotor `s + b B` on a bivector blade `B`.
pub fn rotator_on_blade(s: &Q, b: &Q, blade: usize) -> Mv {
    Mv::scalar(s.clone()).add(&Mv::blade(blade, b.clone()))
}

/// The dilator by `rho > 0`, unnormalised: `(rho + 1) + (rho - 1) E` with
/// `E = e_o ^ e_inf`, its orientation fixed by carrying `X(1)` to `X(rho)`.
pub fn dilator(rho: &Q) -> Mv {
    let e = e_o().geo(&e_inf()).add(&Mv::scalar(Q::ONE));
    let (sum, diff) = (rho + &Q::ONE, rho - &Q::ONE);
    let plus = Mv::scalar(sum.clone()).add(&e.scale(&diff));
    let probe = plus.sandwich(&point_at(&[Q::ONE]));
    match projective_ratio(&probe, &point_at(std::slice::from_ref(rho))) {
        Some(_) => plus,
        None => Mv::scalar(sum).sub(&e.scale(&diff)),
    }
}

/// `Some(lambda)` where `a = lambda b`, read by cross multiplication.
pub fn projective_ratio(a: &Mv, b: &Mv) -> Option<Q> {
    if a.term_count() != b.term_count() {
        return None;
    }
    let ((pa, ak), (pb, bk)) = (a.terms().next()?, b.terms().next()?);
    if pa != pb {
        return None;
    }
    let proportional = a
        .terms()
        .zip(b.terms())
        .all(|((ma, x), (mb, y))| ma == mb && x * bk == ak * y);
    if proportional {
        ak.checked_div(bk)
    } else {
        None
    }
}

/// The part of a generator `l` that keeps the dual sphere `sigma`:
/// `1/2 (L + sigma L sigma^-1)` on its bivector part. `None` for a null
/// `sigma` (a point).
pub fn keeping(sigma: &Mv, l: &Mv) -> Option<Mv> {
    let s2 = sigma.norm2();
    let inverse = sigma.scale(&Q::ONE.checked_div(&s2)?);
    let mirrored = sigma.geo(&l.grade(2)).geo(&inverse).grade(2);
    Some(l.grade(2).add(&mirrored).scale(&half()))
}

/// A white-hole source: the point `at` on a dual sphere it emanates from and
/// its null generator `N = P ^ (P . K)`, which fixes `P` and keeps the
/// sphere.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    /// The conformal point `P` on the dual sphere that the flow fixes.
    pub at: Mv,
    /// The null bivector generator `N = P ^ (P . K)`, with `N^2 = 0`.
    pub generator: Mv,
}

impl Source {
    /// The source at `p` on `sigma` pushed by `k`, verified null, fixing `p`
    /// and keeping `sigma`. `None` where `p` is off the sphere, `k` does not
    /// move it, or a check fails.
    pub fn of(sigma: &Mv, p: &Mv, k: &Mv) -> Option<Source> {
        if !p.geo(sigma).scalar_part().is_zero() {
            return None;
        }
        let v = p.geo(k).sub(&k.geo(p)).grade(1).scale(&half());
        if v.is_zero() {
            return None;
        }
        let n = p.wedge_vector(1, &v);
        let null = !n.is_zero() && n.geo(&n).is_zero();
        let fixes = n.geo(p).sub(&p.geo(&n)).is_zero();
        let keeps = n.geo(sigma).sub(&sigma.geo(&n)).is_zero();
        (null && fixes && keeps).then(|| Source {
            at: p.clone(),
            generator: n,
        })
    }

    /// The flow's versor at parameter `t`: `1 + t N`, exact since `N^2 = 0`.
    pub fn sliver(&self, t: &Q) -> Mv {
        Mv::scalar(Q::ONE).add(&self.generator.scale(t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_are_null_and_their_products_read_distance() {
        let (a, b) = (
            point_at(&[Q::int(1), Q::int(2), Q::int(3)]),
            point_at(&[Q::int(4), Q::int(6), Q::int(3)]),
        );
        assert!(a.norm2().is_zero() && b.norm2().is_zero());
        // X(a) . X(b) = -1/2 |a - b|^2 = -12.5
        assert_eq!(a.geo(&b).scalar_part(), Q::new(-25, 2).unwrap());
        assert_eq!(
            base_at(&a, 3).unwrap(),
            vec![Q::int(1), Q::int(2), Q::int(3)]
        );
    }

    #[test]
    fn motions_carry_points() {
        let x = point_at(&[Q::int(1), Q::int(0), Q::int(0)]);
        let t = translator_at(&[Q::int(2), Q::int(-1), Q::int(5)]);
        assert!(t
            .sandwich(&x)
            .projective_eq(&point_at(&[Q::int(3), Q::int(-1), Q::int(5)])));
        // a half turn in e1e2
        let r = rotator_on_blade(&Q::ZERO, &Q::ONE, E1 | E2);
        assert!(r
            .sandwich(&x)
            .projective_eq(&point_at(&[Q::int(-1), Q::int(0), Q::int(0)])));
        let d = dilator(&Q::int(3));
        assert!(d
            .sandwich(&point_at(&[Q::int(2)]))
            .projective_eq(&point_at(&[Q::int(6)])));
        // a point lies on a sphere through it
        let s = sphere_through(
            &[Q::int(0), Q::int(0), Q::int(0)],
            &[Q::int(1), Q::int(0), Q::int(0)],
        );
        assert!(x.geo(&s).scalar_part().is_zero());
        // and another point of the same reach lies on it too
        let y = point_at(&[Q::int(0), Q::int(0), Q::int(1)]);
        assert!(y.geo(&s).scalar_part().is_zero());
    }
}
