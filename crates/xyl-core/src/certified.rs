//! Certified reals: exact rational intervals proven to contain a real value,
//! for the roots and transcendentals (sqrt, exp, ln, pi, cos/sin, asin,
//! rational powers) that a rational cannot hold.
//!
//! Inputs: exact rationals ([`Q`]) and a precision in bits; no other module.
//!
//! Outputs: [`Encl`], an interval `lo <= x <= hi` of exact rationals, with
//! interval arithmetic (`add`, `sub`, `mul`, `recip`, `outward`); the functions
//! `sqrt`, `exp`, `exp_of`, `ln`, `ln2`, `pi`, `powq`, `cos_sin_turns` (angle
//! in turns), `asin_small` and `asin_any`; `fixed_bounds`, a rational in
//! 128-bit units of `2^-FIXED` (`FIXED` = 60) for the fixed-point series of
//! the circular functions, refused (`None`) where a bound does not fit
//! `i128`; and `down` / `up`, rounding to the grid of `2^-bits`.
//!
//! Invariants:
//! - every series is summed with an explicit bound on its remainder;
//! - intermediates are rounded outward to a dyadic grid (`lo` down, `hi` up),
//!   which bounds the size of the rationals without losing containment;
//! - an operation on enclosures (sum, product, monotone map) encloses its
//!   result;
//! - out-of-domain input is refused with `None`: `sqrt` of a negative, `ln`
//!   and `powq` of a non-positive value (except `powq` with `p = 0`), `recip`
//!   of an interval holding zero, `fixed_bounds` of a value past `i128`;
//! - no floating point is used, so the same inputs give the same enclosure on
//!   every machine.
//!
//! Two overlapping enclosures are not ordered: a caller decides only what the
//! bounds decide, and asks for more bits when they do not.
//!
//! Terms: `Encl` (docs/glossary.md).

use crate::q::dyadic;
use crate::Q;
use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::One;

/// An exact rational interval proven to contain a real value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Encl {
    /// Lower bound, at or below the true value.
    pub lo: Q,
    /// Upper bound, at or above the true value.
    pub hi: Q,
}

/// `2^bits`, exact.
pub(crate) fn pow2(bits: u32) -> BigInt {
    BigInt::one() << bits
}

/// `q` rounded down to the grid of `2^-bits`: one floor division of the
/// numerator, shifted left by `bits`, by the denominator.
pub fn down(q: &Q, bits: u32) -> Q {
    dyadic((q.numer() << bits).div_floor(&q.denom()), u64::from(bits))
}

/// `q` cut to the grid of `2^-bits` toward zero: `sgn(q) floor(|q| 2^bits) /
/// 2^bits`. Mirror-symmetric, `toward_zero(-q) = -toward_zero(q)`, and
/// never larger in magnitude than `q`, so a cut adds no spin, turn or
/// matter. The cut for held values; [`down`] and [`up`] bound enclosures.
pub fn toward_zero(q: &Q, bits: u32) -> Q {
    if q.signum() < 0 {
        up(q, bits)
    } else {
        down(q, bits)
    }
}

/// `q` rounded up to the grid of `2^-bits`.
pub fn up(q: &Q, bits: u32) -> Q {
    dyadic((q.numer() << bits).div_ceil(&q.denom()), u64::from(bits))
}

impl Encl {
    /// A rational, known exactly.
    pub fn exact(q: Q) -> Encl {
        Encl {
            lo: q.clone(),
            hi: q,
        }
    }

    /// The exact midpoint, for display or as a representative value.
    pub fn mid(&self) -> Q {
        // The sum is halved by multiplying by 1/2, so a dyadic sum stays on
        // the gcd-free path of `Q`.
        &(&self.lo + &self.hi) * &Q::Small { n: 1, d: 2 }
    }

    /// Whether `q` lies in the closed interval `[lo, hi]`.
    pub fn contains(&self, q: &Q) -> bool {
        &self.lo <= q && q <= &self.hi
    }

    /// The sum: `[lo + o.lo, hi + o.hi]`.
    pub fn add(&self, o: &Encl) -> Encl {
        Encl {
            lo: &self.lo + &o.lo,
            hi: &self.hi + &o.hi,
        }
    }

    /// The difference: `[lo - o.hi, hi - o.lo]`.
    pub fn sub(&self, o: &Encl) -> Encl {
        Encl {
            lo: &self.lo - &o.hi,
            hi: &self.hi - &o.lo,
        }
    }

    /// The negation: `[-hi, -lo]`.
    pub fn neg(&self) -> Encl {
        Encl {
            lo: -self.hi.clone(),
            hi: -self.lo.clone(),
        }
    }

    /// The product: the extremes of the four corner products.
    pub fn mul(&self, o: &Encl) -> Encl {
        // An exact factor scales the bounds of the other enclosure; the
        // bounds are swapped if the factor is negative.
        if self.lo == self.hi || o.lo == o.hi {
            let (k, e) = if self.lo == self.hi {
                (&self.lo, o)
            } else {
                (&o.lo, self)
            };
            let (a, b) = (k * &e.lo, k * &e.hi);
            return if k.signum() < 0 {
                Encl { lo: b, hi: a }
            } else {
                Encl { lo: a, hi: b }
            };
        }
        let (a, b, c, d) = (
            &self.lo * &o.lo,
            &self.lo * &o.hi,
            &self.hi * &o.lo,
            &self.hi * &o.hi,
        );
        Encl {
            lo: a.clone().min(b.clone()).min(c.clone()).min(d.clone()),
            hi: a.max(b).max(c).max(d),
        }
    }

    /// The reciprocal of an enclosure that excludes zero; `None` otherwise.
    pub fn recip(&self) -> Option<Encl> {
        if self.lo.signum() > 0 || self.hi.signum() < 0 {
            Some(Encl {
                lo: self.hi.recip()?,
                hi: self.lo.recip()?,
            })
        } else {
            None
        }
    }

    /// The enclosure rounded outward to the grid of `2^-bits`: it still
    /// contains the value, and its bounds have smaller numerators and
    /// denominators.
    pub fn outward(&self, bits: u32) -> Encl {
        Encl {
            lo: down(&self.lo, bits),
            hi: up(&self.hi, bits),
        }
    }
}

/// `sqrt(x)` for `x >= 0`, to within `2^-bits`: the integer square root of
/// the scaled value bounds it from both sides. Returns `None` for negative
/// `x`.
pub fn sqrt(x: &Q, bits: u32) -> Option<Encl> {
    if x.signum() < 0 {
        return None;
    }
    // `sqrt(n/d) = sqrt(n d) / d`; scaled by 2^bits, the lower bound is
    // `floor(sqrt(n d 4^bits)) / (d 2^bits)` and the upper bound adds one unit.
    let (n, d) = (x.numer(), x.denom());
    let s = pow2(bits);
    let radicand = &n * &d * &s * &s;
    let r = radicand.sqrt();
    let exact = &r * &r == radicand;
    let den = &d * &s;
    let lo = Q::from_big(BigRational::new(r.clone(), den.clone()));
    let hi = if exact {
        lo.clone()
    } else {
        Q::from_big(BigRational::new(r + BigInt::one(), den))
    };
    Some(Encl { lo, hi })
}

/// `exp(x)`, enclosed to a relative width of about `2^-bits`.
///
/// The argument is halved until it is at most 1/2, then enclosed on a dyadic
/// grid of `2^-g`. The Taylor series of `exp(y)` for `y >= 0` has only
/// positive terms, so each term is carried as a dyadic pair rounded outward
/// (lower bound down, upper bound up), which keeps the rationals small. The
/// series stops at the first term below `2^-(g - 8)`, and the remainder is
/// bounded by twice that term (`y <= 1/2`). The enclosure is then squared
/// once per halving, rounding outward. A negative argument is evaluated as
/// `1 / exp(-x)`.
pub fn exp(x: &Q, bits: u32) -> Encl {
    if x.signum() < 0 {
        // The exponential of a positive argument is at least one (its series
        // starts at one and every term is positive), so the enclosure
        // excludes zero and its reciprocal reads
        // (`tests::exp_of_a_non_negative_argument_is_at_least_one`).
        let Some(r) = exp(&-x.clone(), bits + 2).recip() else {
            return Encl::exact(Q::ZERO);
        };
        return r.outward(bits + 16);
    }
    let half = crate::ratio!(1, 2);
    let mut halvings = 0u32;
    let mut y = x.clone();
    while y > half {
        y = y.over(crate::nonzero!(2));
        halvings += 1;
    }
    let g = bits + 24 + halvings;
    let (ylo, yhi) = (down(&y, g), up(&y, g));
    // The stopping threshold is 8 bits coarser than the rounding grid: an
    // upper bound rounded up is never below one grid step, so a threshold at
    // the grid would never be met.
    let tiny = Q::from_big(BigRational::new(BigInt::one(), pow2(g - 8)));
    let (mut slo, mut shi) = (Q::ZERO, Q::ZERO);
    let (mut tlo, mut thi) = (Q::ONE, Q::ONE);
    // The term index counts from one, so each divisor is nonzero by
    // construction.
    for i in (1..).filter_map(std::num::NonZeroI64::new) {
        slo = down(&(&slo + &tlo), g);
        shi = up(&(&shi + &thi), g);
        tlo = down(&(&tlo * &ylo).over(i), g);
        thi = up(&(&thi * &yhi).over(i), g);
        if thi < tiny {
            break;
        }
    }
    // The remainder after the kept terms is at most twice the first omitted
    // term for y <= 1/2.
    let mut e = Encl {
        lo: slo,
        hi: &shi + &thi.scale(2),
    };
    for _ in 0..halvings {
        e = e.mul(&e).outward(g);
    }
    e.outward(bits + 16)
}

/// `atanh(z)` for `0 <= z <= 1/3`, from a dyadic enclosure of `z`: the
/// positive series `sum z^(2k+1) / (2k+1)` carried outward, with the
/// remainder bounded by the geometric tail `t / (1 - z^2)` of the first
/// omitted term `t`. The function is odd: a negative `z` is evaluated as
/// `-atanh(-z)`.
fn atanh(z: &Q, bits: u32) -> Encl {
    if z.signum() < 0 {
        return atanh(&-z.clone(), bits).neg();
    }
    let g = bits + 16;
    let (zlo, zhi) = (down(z, g), up(z, g));
    let (z2lo, z2hi) = (down(&(&zlo * &zlo), g), up(&(&zhi * &zhi), g));
    // The stopping threshold is 8 bits coarser than the rounding grid (see
    // `exp`).
    let tiny = Q::from_big(BigRational::new(BigInt::one(), pow2(g - 8)));
    let (mut plo, mut phi) = (zlo, zhi);
    let (mut slo, mut shi) = (Q::ZERO, Q::ZERO);
    // The odd divisors 2k + 1 are nonzero by construction.
    for odd in (1..).step_by(2).filter_map(std::num::NonZeroI64::new) {
        let tlo = down(&plo.over(odd), g);
        let thi = up(&phi.over(odd), g);
        if thi < tiny {
            // For `z <= 1/3` the bound on `z^2` is below one, so the tail's
            // divisor `1 - z^2` is positive
            // (`tests::atanh_tail_ratio_is_positive_on_the_domain`).
            let tail = match thi.checked_div(&(&Q::ONE - &z2hi)) {
                Some(t) => up(&t, g),
                None => up(&thi, g),
            };
            return Encl {
                lo: slo,
                hi: &shi + &tail,
            }
            .outward(bits);
        }
        slo = down(&(&slo + &tlo), g);
        shi = up(&(&shi + &thi), g);
        plo = down(&(&plo * &z2lo), g);
        phi = up(&(&phi * &z2hi), g);
    }
    // The odd numbers do not run out before the terms fall below `tiny`.
    Encl {
        lo: Q::ZERO,
        hi: Q::ONE,
    }
}

/// `atan(x)` for `|x| <= 1/5`: the alternating series
/// `sum (-1)^k x^(2k+1) / (2k+1)`, whose remainder is bounded by the first
/// omitted term.
fn atan_small(x: &Q, bits: u32) -> Encl {
    let tiny = Q::from_big(BigRational::new(BigInt::one(), pow2(bits)));
    let x2 = x * x;
    let mut power = x.clone();
    let mut sum = Q::ZERO;
    let mut add = true;
    // The odd divisors 2k + 1 are nonzero by construction.
    for odd in (1..).step_by(2).filter_map(std::num::NonZeroI64::new) {
        let t = power.over(odd);
        if t.abs() < tiny {
            let tail = t.abs();
            return Encl {
                lo: &sum - &tail,
                hi: &sum + &tail,
            }
            .outward(bits);
        }
        sum = if add { &sum + &t } else { &sum - &t };
        add = !add;
        power = &power * &x2;
    }
    // The odd numbers do not run out before the terms fall below `tiny`;
    // `|atan x| <= |x|` encloses the value in any case.
    Encl {
        lo: -x.abs(),
        hi: x.abs(),
    }
}

/// `atan(x)` for any rational `x`, certified: odd in `x`; beyond 1 it is
/// `pi / 2 - atan(1 / x)`; within it, `atan(1/5)` is taken off by the
/// tangent's composition, `atan x - atan y = atan((x - y) / (1 + x y))`,
/// exact, until the rest lies within `1/5` (at most four steps), and the
/// series closes it.
pub fn atan(x: &Q, bits: u32) -> Encl {
    if x.signum() < 0 {
        return atan(&-x, bits).neg();
    }
    if x > &Q::ONE {
        let half = pi(bits + 4).mul(&Encl::exact(crate::ratio!(1, 2)));
        return x
            .recip()
            .map_or(half.clone(), |r| half.sub(&atan(&r, bits)));
    }
    let fifth = crate::ratio!(1, 5);
    let step = atan_small(&fifth, bits + 8);
    let mut rest = x.clone();
    let mut taken = Encl::exact(Q::ZERO);
    while rest > fifth {
        let Some(next) = (&rest - &fifth).checked_div(&(&Q::ONE + &(&rest * &fifth))) else {
            break;
        };
        rest = next;
        taken = taken.add(&step);
    }
    taken.add(&atan_small(&rest, bits + 8)).outward(bits)
}

/// `pi`, by Machin's formula `16 atan(1/5) - 4 atan(1/239)`.
pub fn pi(bits: u32) -> Encl {
    let a = atan_small(&crate::ratio!(1, 5), bits + 8);
    let b = atan_small(&crate::ratio!(1, 239), bits + 8);
    a.mul(&Encl::exact(Q::int(16)))
        .sub(&b.mul(&Encl::exact(Q::int(4))))
        .outward(bits + 4)
}

/// `ln 2`, enclosed as `2 atanh(1/3)`.
pub fn ln2(bits: u32) -> Encl {
    let a = atanh(&crate::ratio!(1, 3), bits + 4);
    a.add(&a)
}

/// `ln x` for `x > 0`: `x = 2^k m` with `m` in `[2/3, 4/3]`, then
/// `ln x = 2 atanh(z) + k ln 2` with `z = (m - 1) / (m + 1)` in
/// `[-1/5, 1/7]`. Returns `None` for `x <= 0`.
pub fn ln(x: &Q, bits: u32) -> Option<Encl> {
    if x.signum() <= 0 {
        return None;
    }
    let guard = bits + 16;
    let (lo_m, hi_m) = (Q::new(2, 3)?, Q::new(4, 3)?);
    let mut m = x.clone();
    let mut k: i64 = 0;
    while m > hi_m {
        m = m.checked_div(&Q::int(2))?;
        k += 1;
    }
    while m < lo_m {
        m = m.scale(2);
        k -= 1;
    }
    let z = (&m - &Q::ONE).checked_div(&(&m + &Q::ONE))?;
    let a = atanh(&z, guard);
    let lnm = a.add(&a);
    let l2 = ln2(guard + 8);
    let kk = Encl::exact(Q::int(k));
    Some(lnm.add(&kk.mul(&l2)).outward(guard))
}

/// `exp` of an enclosure: `exp` is increasing, so the lower bound maps to
/// the lower bound and the upper to the upper.
pub fn exp_of(x: &Encl, bits: u32) -> Encl {
    Encl {
        lo: exp(&x.lo, bits).lo,
        hi: exp(&x.hi, bits).hi,
    }
}

/// The fixed point of [`fixed_bounds`], [`cos_sin_turns`] and
/// [`asin_small`]: values in units of `2^-60`.
pub const FIXED: u32 = 60;

/// `q` in units of `2^-60`, rounded down and up. `None` when a bound does
/// not fit `i128` (`|q| >= 2^67`).
pub fn fixed_bounds(q: &Q) -> Option<(i128, i128)> {
    let n = q.numer() << FIXED;
    let d = q.denom();
    let lo = n.div_floor(&d);
    let hi = n.div_ceil(&d);
    Some((
        num_traits::ToPrimitive::to_i128(&lo)?,
        num_traits::ToPrimitive::to_i128(&hi)?,
    ))
}

/// Enclosures of the cosine and sine of an angle given in turns (fractions
/// of the full circle), proven, in 128-bit fixed point. The quadrant is read
/// exactly from the turns; the remainder `y` in `[0, 1/4)` turn is converted
/// to radians with an enclosure of 2 pi. On `[0, pi/2)` the sine increases
/// and the cosine decreases, so each is bounded by its series at the ends of
/// the enclosure of `y`: an alternating series with decreasing terms, in
/// error by less than the first omitted term plus two units per term for
/// the rounded products. An interval that spans a whole turn gives `[-1, 1]`
/// for both; one that crosses a quadrant edge gives the hull of its ends and
/// the edge.
pub fn cos_sin_turns(t: &Encl) -> (Encl, Encl) {
    let whole = |q: &Q| Q::from_reduced(BigRational::from_integer(q.floor()));
    let base = whole(&t.lo);
    let (f_lo, f_hi) = (&t.lo - &base, &t.hi - &base);
    let unit = Encl {
        lo: Q::int(-1),
        hi: Q::ONE,
    };
    if f_hi >= Q::ONE {
        return (unit.clone(), unit);
    }
    let quarter = |f: &Q| (f * &Q::int(4)).floor();
    let (q_lo, q_hi) = (quarter(&f_lo), quarter(&f_hi));
    if q_lo != q_hi {
        // The interval crosses a quadrant edge, where cos and sin take 0 or
        // +-1: the result is the hull of both ends and the edge.
        let edge = Q::from_reduced(BigRational::new(q_hi.clone(), BigInt::from(4)));
        let parts = [
            cos_sin_turns(&Encl {
                lo: f_lo.clone(),
                hi: f_lo.clone(),
            }),
            cos_sin_turns(&Encl {
                lo: edge.clone(),
                hi: edge,
            }),
            cos_sin_turns(&Encl {
                lo: f_hi.clone(),
                hi: f_hi,
            }),
        ];
        let [a, b, c] = &parts;
        let hull = |pick: fn(&(Encl, Encl)) -> &Encl| Encl {
            lo: pick(a)
                .lo
                .clone()
                .min(pick(b).lo.clone())
                .min(pick(c).lo.clone()),
            hi: pick(a)
                .hi
                .clone()
                .max(pick(b).hi.clone())
                .max(pick(c).hi.clone()),
        };
        return (hull(|p| &p.0), hull(|p| &p.1));
    }
    let q4 = Q::from_reduced(BigRational::new(q_lo.clone(), BigInt::from(4)));
    // The remainders lie in [0, 1/4) turn and 2 pi is below 7, so every
    // fixed-point bound fits; were one not to, the unit interval still
    // encloses both values.
    let (Some(two_pi), Some((y_lo, _)), Some((_, y_hi))) = (
        two_pi_fixed(),
        fixed_bounds(&(&f_lo - &q4)),
        fixed_bounds(&(&f_hi - &q4)),
    ) else {
        return (unit.clone(), unit);
    };
    let y_lo = (y_lo * two_pi.0) >> FIXED;
    let y_hi = ((y_hi * two_pi.1) >> FIXED) + 1;
    let (s_lo, s_hi) = (alternating(y_lo, true).0, alternating(y_hi, true).1);
    let (c_lo, c_hi) = (alternating(y_hi, false).0, alternating(y_lo, false).1);
    let one: i128 = 1 << FIXED;
    let e = |lo: i128, hi: i128| Encl {
        lo: dyadic(BigInt::from(lo.max(-one)), u64::from(FIXED)),
        hi: dyadic(BigInt::from(hi.min(one)), u64::from(FIXED)),
    };
    let (s, c) = (e(s_lo, s_hi), e(c_lo, c_hi));
    // The quadrant is `floor(4 f)` for `f` in [0, 1), so 0 to 3; any other
    // reading is enclosed by the unit interval.
    match num_traits::ToPrimitive::to_i64(&q_lo) {
        Some(0) => (c, s),
        Some(1) => (s.neg(), c),
        Some(2) => (c.neg(), s.neg()),
        Some(3) => (s, c.neg()),
        _ => (unit.clone(), unit),
    }
}

/// Lower and upper bounds of 2 pi in units of 2^-60, computed once. `None`
/// only if they did not fit `i128`, which 2 pi below 7 does.
fn two_pi_fixed() -> Option<(i128, i128)> {
    static P: std::sync::OnceLock<Option<(i128, i128)>> = std::sync::OnceLock::new();
    *P.get_or_init(|| {
        let p = pi(100);
        let two = Q::int(2);
        Some((
            fixed_bounds(&(&p.lo * &two))?.0,
            fixed_bounds(&(&p.hi * &two))?.1,
        ))
    })
}

/// Bounds on the sine (`odd`) or cosine of `x` in units of 2^-60, for
/// `0 <= x < 2`: the series sum, widened by the first omitted term and two
/// units for each term's rounded products.
fn alternating(x: i128, odd: bool) -> (i128, i128) {
    let one: i128 = 1 << FIXED;
    let (mut term, mut n) = if odd { (x, 1i128) } else { (one, 0i128) };
    let (mut sum, mut sign, mut k) = (term, 1i128, 0i128);
    loop {
        term = ((((term * x) >> FIXED) * x) >> FIXED) / ((n + 1) * (n + 2));
        n += 2;
        k += 1;
        sign = -sign;
        if term <= 0 || k > 60 {
            let err = 2 * k + 2 + term.abs();
            return (sum - err, sum + err);
        }
        sum += sign * term;
    }
}

/// `asin x` for `0 <= x <= 1/2`, proven: `x` times the series in `x^2`
/// `1 + x^2/6 + 3x^4/40 + ...`. All terms are positive and each is the
/// previous one times `x^2 (2n+1)^2 / ((2n+2)(2n+3))`, which is below 1/4,
/// so the tail after a term is under a third of it. The factor is summed in
/// fixed point, floored for the lower bound and rounded up for the upper,
/// and multiplied by the exact bounds of `x`, which keeps the relative
/// precision for small `x`. Outside the domain the result is `[0, 2]`.
pub fn asin_small(x: &Encl) -> Encl {
    let outside = Encl {
        lo: Q::ZERO,
        hi: Q::int(2),
    };
    if x.lo.signum() < 0 || x.hi > crate::ratio!(1, 2) {
        return outside;
    }
    let one: i128 = 1 << FIXED;
    let factor = |x2: i128, up: bool| -> i128 {
        let (mut term, mut sum, mut n) = (one, one, 0i128);
        loop {
            let prod = (term * x2) >> FIXED;
            let num = prod * (2 * n + 1) * (2 * n + 1);
            let den = (2 * n + 2) * (2 * n + 3);
            term = num / den + i128::from(up);
            n += 1;
            if term <= i128::from(up) || n > 80 {
                return if up { sum + term + 1 } else { sum };
            }
            sum += term;
        }
    };
    // The squares are at most 1/4 on the domain, so their fixed-point
    // bounds fit.
    let (Some((sq_lo, _)), Some((_, sq_hi))) = (
        fixed_bounds(&(&x.lo * &x.lo)),
        fixed_bounds(&(&x.hi * &x.hi)),
    ) else {
        return outside;
    };
    let f = Encl {
        lo: dyadic(BigInt::from(factor(sq_lo, false)), u64::from(FIXED)),
        hi: dyadic(BigInt::from(factor(sq_hi, true)), u64::from(FIXED)),
    };
    Encl {
        lo: &x.lo * &f.lo,
        hi: &x.hi * &f.hi,
    }
}

/// `asin x` in radians for `x` in [0, 1], enclosed: the series of
/// `asin_small` when `x <= 1/2`, otherwise 56 bisections of the quarter turn
/// on the certified sine, each bound a turn whose sine certainly lies on the
/// bound's side of `x`. Bounds of `x` outside [0, 1] are clamped.
pub fn asin_any(x: &Encl) -> Encl {
    if x.hi <= crate::ratio!(1, 2) && x.lo.signum() >= 0 {
        return asin_small(x);
    }
    let quarter = crate::ratio!(1, 4);
    let bound = |target: &Q, below: bool| -> Q {
        let (mut lo, mut hi) = (Q::ZERO, quarter.clone());
        for _ in 0..56 {
            let mid = (&lo + &hi).over(crate::nonzero!(2));
            let s = cos_sin_turns(&Encl::exact(mid.clone())).1;
            // For the lower bound, `lo` moves up while the sine is certainly
            // below the target; for the upper, `hi` moves down while it is
            // certainly above.
            let past = if below {
                s.hi < *target
            } else {
                s.lo <= *target
            };
            if past {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        if below {
            lo
        } else {
            hi
        }
    };
    let (t_lo, t_hi) = (
        bound(&x.lo.clone().max(Q::ZERO), true),
        bound(&x.hi.clone().min(Q::ONE), false),
    );
    let two_pi = pi(80).mul(&Encl::exact(Q::int(2)));
    Encl {
        lo: &t_lo * &two_pi.lo,
        hi: &t_hi * &two_pi.hi,
    }
}

/// `x^p` for `x > 0` and a rational power `p`, enclosed as `exp(p ln x)`.
/// Returns exactly 1 for `p = 0`, and `None` for `x <= 0` otherwise.
pub fn powq(x: &Q, p: &Q, bits: u32) -> Option<Encl> {
    if p.is_zero() {
        return Some(Encl::exact(Q::ONE));
    }
    let l = ln(x, bits + 8)?;
    Some(exp_of(&Encl::exact(p.clone()).mul(&l), bits))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(s: &str) -> Q {
        // A decimal literal as an exact rational.
        let (int, frac) = s.split_once('.').map_or((s, ""), |p| p);
        let den = 10i128.pow(u32::try_from(frac.len()).unwrap());
        let num: i128 = format!("{int}{frac}").parse().unwrap();
        Q::from_i128(num, den).unwrap()
    }

    /// The held cut is mirror-symmetric and never grows a magnitude.
    #[test]
    fn the_cut_toward_zero_is_odd_and_never_grows() {
        for x in [
            dec("0.3"),
            dec("-0.3"),
            dec("2.71828"),
            dec("-1.41421"),
            Q::ZERO,
        ] {
            let c = toward_zero(&x, 7);
            assert_eq!(toward_zero(&(-&x), 7), -&c);
            assert!(c.abs() <= x.abs());
            assert!((&x - &c).abs() < Q::new(1, 128).unwrap());
        }
    }

    #[test]
    fn known_constants_are_enclosed_tightly() {
        let e = exp(&Q::ONE, 64);
        assert!(
            e.contains(&dec("2.718281828459045"))
                || (e.lo < dec("2.7182818284590453") && e.hi > dec("2.7182818284590451"))
        );
        assert!(&e.hi - &e.lo < Q::from_big(BigRational::new(BigInt::one(), pow2(60))));
        let l2 = ln2(64);
        assert!(l2.lo < dec("0.6931471805599454") && l2.hi > dec("0.6931471805599452"));
        let s2 = sqrt(&Q::int(2), 64).unwrap();
        assert!(s2.lo < dec("1.4142135623730951") && s2.hi > dec("1.4142135623730950"));
        assert_eq!(
            sqrt(&Q::new(9, 4).unwrap(), 32).unwrap(),
            Encl::exact(Q::new(3, 2).unwrap())
        );
    }

    #[test]
    fn ln_and_exp_invert_each_other_within_their_bounds() {
        for (n, d) in [(1, 1), (7, 3), (1, 1000), (123_456, 7), (5, 8)] {
            let x = Q::new(n, d).unwrap();
            let back = exp_of(&ln(&x, 80).unwrap(), 80);
            assert!(
                back.contains(&x),
                "exp(ln {x}) = [{}, {}]",
                back.lo,
                back.hi
            );
        }
        // Negative arguments: exp(-3) exp(3) encloses 1.
        let p = exp(&Q::int(-3), 80).mul(&exp(&Q::int(3), 80));
        assert!(p.contains(&Q::ONE));
    }

    /// The exponential of a non-negative argument is at least one, so the
    /// reciprocal `exp` takes for a negative argument always reads.
    #[test]
    fn exp_of_a_non_negative_argument_is_at_least_one() {
        for (n, d) in [(0, 1), (1, 1000), (1, 2), (3, 1), (250, 1), (4097, 3)] {
            let e = exp(&Q::new(n, d).unwrap(), 64);
            assert!(e.lo >= Q::ONE && e.recip().is_some(), "{n}/{d}");
            let back = exp(&Q::new(-n, d).unwrap(), 64);
            assert!(back.lo.signum() > 0 || n > 0);
            assert!(back.hi <= Q::ONE);
        }
    }

    /// On the domain `0 <= z <= 1/3` the bound on `z^2` stays below one at
    /// every grid `atanh` uses, so its tail divisor `1 - z^2` is positive.
    /// The certified arctangent encloses its known values: atan 1 = pi / 4,
    /// atan of a large value nears pi / 2, it is odd, and a small value reads
    /// itself to first order.
    #[test]
    fn the_arctangent_is_enclosed() {
        let quarter = pi(80).mul(&Encl::exact(crate::ratio!(1, 4)));
        let one = atan(&Q::ONE, 64);
        assert!(one.lo <= quarter.hi && quarter.lo <= one.hi);
        let big = atan(&Q::int(1_000_000), 64);
        let half = pi(80).mul(&Encl::exact(crate::ratio!(1, 2)));
        assert!(big.hi < half.lo && big.lo > half.lo.clone() - crate::ratio!(1, 999_999));
        assert_eq!(
            atan(&crate::ratio!(-1, 3), 64),
            atan(&crate::ratio!(1, 3), 64).neg()
        );
        let small = atan(&crate::ratio!(1, 1_000_000_000), 64);
        assert!(
            (&small.mid() - &crate::ratio!(1, 1_000_000_000)).abs()
                < crate::ratio!(1, 1_000_000_000_000_000_000)
        );
    }

    #[test]
    fn atanh_tail_ratio_is_positive_on_the_domain() {
        let z = crate::ratio!(1, 3);
        for bits in [16, 64, 80, 120] {
            let g = bits + 16;
            let zhi = up(&z, g);
            let z2hi = up(&(&zhi * &zhi), g);
            assert!((&Q::ONE - &z2hi).signum() > 0, "{bits}");
            // The enclosure atanh reads at the edge of its domain is ordered.
            let a = atanh(&z, bits);
            assert!(a.lo <= a.hi && a.lo.signum() > 0);
        }
    }

    /// A value whose fixed-point bounds do not fit `i128` is refused, and
    /// one that fits reads its floor and ceiling.
    #[test]
    fn fixed_bounds_refuse_what_i128_cannot_hold() {
        let one = 1i128 << FIXED;
        assert_eq!(fixed_bounds(&Q::ONE), Some((one, one)));
        let third = fixed_bounds(&crate::ratio!(1, 3)).unwrap();
        assert_eq!(third.1 - third.0, 1);
        let huge = &Q::int(i64::MAX) * &Q::int(1 << 10);
        assert_eq!(fixed_bounds(&huge), None);
    }

    /// The sine and cosine of a turn in each quadrant carry the quadrant's
    /// signs, and both lie in [-1, 1].
    #[test]
    fn cos_sin_turns_reads_every_quadrant() {
        let signs = [(1, 1), (-1, 1), (-1, -1), (1, -1)];
        for (q, (cs, ss)) in signs.into_iter().enumerate() {
            let t = Q::new(8 * i64::try_from(q).unwrap() + 1, 32).unwrap();
            let (c, s) = cos_sin_turns(&Encl::exact(t));
            assert!(c.lo.signum() == cs && c.hi.signum() == cs, "{q}");
            assert!(s.lo.signum() == ss && s.hi.signum() == ss, "{q}");
            for e in [&c, &s] {
                assert!(e.lo >= Q::int(-1) && e.hi <= Q::ONE);
            }
        }
    }

    #[test]
    fn pi_by_machin_is_enclosed_tightly() {
        let p = pi(80);
        assert!(p.lo < dec("3.14159265358979324") && p.hi > dec("3.14159265358979323"));
        assert!(&p.hi - &p.lo < Q::from_big(BigRational::new(BigInt::one(), pow2(76))));
    }
}
