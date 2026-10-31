//! Certified reals: exact rational intervals proven to contain a real value,
//! for the roots and transcendentals (sqrt, exp, ln, pi, cos/sin, asin,
//! rational powers) that a rational cannot hold.
//!
//! Inputs: exact rationals ([`Q`]) and a precision in bits; no other module.
//!
//! Outputs: [`Encl`], an interval `lo <= x <= hi` of exact rationals, with
//! interval arithmetic (`add`, `sub`, `mul`, `recip`, `outward`); the functions
//! `sqrt`, `exp`, `exp_of`, `ln`, `ln2`, `pi`, `powq`, `cos_sin_turns` (angle
//! in turns), `asin_small` and `asin_any`; the 128-bit fixed-point path
//! `exp_fixed` / `fixed_bounds` in units of `2^-FIXED` (`FIXED` = 60); and
//! `down` / `up`, rounding to the grid of `2^-bits`.
//!
//! Invariants:
//! - every series is summed with an explicit bound on its remainder;
//! - intermediates are rounded outward to a dyadic grid (`lo` down, `hi` up),
//!   which bounds the size of the rationals without losing containment;
//! - an operation on enclosures (sum, product, monotone map) encloses its
//!   result;
//! - out-of-domain input is refused with `None`: `sqrt` of a negative, `ln`
//!   and `powq` of a non-positive value, `recip` of an interval holding zero;
//! - no floating point is used, so the same inputs give the same enclosure on
//!   every machine.
//!
//! Two overlapping enclosures are not ordered: a caller decides only what the
//! bounds decide, and asks for more bits when they do not.
//!
//! Terms: `Encl` (docs/glossary.md).

use crate::Q;
use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Zero};

/// An exact rational interval proven to contain a real value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Encl {
    /// Lower bound, at or below the true value.
    pub lo: Q,
    /// Upper bound, at or above the true value.
    pub hi: Q,
}

fn pow2(bits: u32) -> BigInt {
    BigInt::one() << bits
}

/// `k / 2^bits` in lowest terms without a gcd: the power of two's common
/// factor is the numerator's trailing zero bits.
fn dyadic(k: BigInt, bits: u32) -> Q {
    // zero has no trailing zeros to count: it is the one canonical zero
    if k.is_zero() {
        return Q::ZERO;
    }
    let tz = u32::try_from(k.trailing_zeros().unwrap_or(0))
        .unwrap_or(0)
        .min(bits);
    Q::from_reduced(BigRational::new_raw(k >> tz, pow2(bits - tz)))
}

/// `q` rounded down to the grid of `2^-bits`: one floor division of the
/// numerator, shifted, by the denominator.
pub fn down(q: &Q, bits: u32) -> Q {
    dyadic((q.numer() << bits).div_floor(&q.denom()), bits)
}

/// `q` rounded up to the grid of `2^-bits`.
pub fn up(q: &Q, bits: u32) -> Q {
    dyadic((q.numer() << bits).div_ceil(&q.denom()), bits)
}

impl Encl {
    /// A rational, known exactly.
    pub fn exact(q: Q) -> Encl {
        Encl {
            lo: q.clone(),
            hi: q,
        }
    }

    /// The midpoint, for display or as a representative (exact).
    pub fn mid(&self) -> Q {
        // halved by a product, so a dyadic stays on the gcd-free path
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
        // an exact factor scales the other's bounds, swapped if negative
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
        let c = [
            &self.lo * &o.lo,
            &self.lo * &o.hi,
            &self.hi * &o.lo,
            &self.hi * &o.hi,
        ];
        let lo = c.iter().min().cloned().unwrap_or(Q::ZERO);
        let hi = c.iter().max().cloned().unwrap_or(Q::ZERO);
        Encl { lo, hi }
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

    /// Rounded outward to the grid of `2^-bits`: still an enclosure, with
    /// smaller numbers.
    pub fn outward(&self, bits: u32) -> Encl {
        Encl {
            lo: down(&self.lo, bits),
            hi: up(&self.hi, bits),
        }
    }
}

/// `√x` for `x ≥ 0`, to within `2^-bits`: integer square roots of the
/// scaled value bound it from both sides. `None` for negative `x`.
pub fn sqrt(x: &Q, bits: u32) -> Option<Encl> {
    if x.signum() < 0 {
        return None;
    }
    // √(n/d) = √(n·d)/d; scale by 2^bits: floor(√(n·d·4^bits)) / (d·2^bits)
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

/// `exp(x)`, enclosed within a relative `2^-bits` or so.
///
/// The argument is halved until it is at most 1/2, then enclosed on a dyadic
/// grid. The Taylor series of `exp(y)` for `y ≥ 0` has only positive terms,
/// so each term is carried as a dyadic pair rounded outward (lower bound
/// down, upper bound up) and the numbers never grow. The series stops at a
/// term below `2^-g`, and its remainder is bounded by twice that term
/// (`y ≤ 1/2`). The enclosure is then squared back up, rounding outward. A
/// negative argument is `1/exp(−x)`.
pub fn exp(x: &Q, bits: u32) -> Encl {
    if x.signum() < 0 {
        return exp(&-x.clone(), bits + 2)
            .recip()
            .unwrap_or_else(|| Encl::exact(Q::ZERO))
            .outward(bits + 16);
    }
    let half = Q::new(1, 2).unwrap_or(Q::ZERO);
    let mut halvings = 0u32;
    let mut y = x.clone();
    while y > half {
        y = y.checked_div(&Q::int(2)).unwrap_or(Q::ZERO);
        halvings += 1;
    }
    let g = bits + 24 + halvings;
    let (ylo, yhi) = (down(&y, g), up(&y, g));
    // stop 8 bits coarser than the rounding grid: an outward-rounded term is
    // never below one grid step, so a threshold at the grid would never be met
    let tiny = Q::from_big(BigRational::new(BigInt::one(), pow2(g - 8)));
    let (mut slo, mut shi) = (Q::ZERO, Q::ZERO);
    let (mut tlo, mut thi) = (Q::ONE, Q::ONE);
    let mut i: i64 = 0;
    loop {
        slo = down(&(&slo + &tlo), g);
        shi = up(&(&shi + &thi), g);
        i += 1;
        tlo = down(&(&tlo * &ylo).checked_div(&Q::int(i)).unwrap_or(Q::ZERO), g);
        thi = up(&(&thi * &yhi).checked_div(&Q::int(i)).unwrap_or(Q::ZERO), g);
        if thi < tiny {
            break;
        }
    }
    // the remainder after the kept terms is at most 2·(next term) for y ≤ 1/2
    let mut e = Encl {
        lo: slo,
        hi: &shi + &thi.scale(2),
    };
    for _ in 0..halvings {
        e = e.mul(&e).outward(g);
    }
    e.outward(bits + 16)
}

/// `atanh(z)` for `0 ≤ z ≤ 1/3`, from a dyadic enclosure of `z`: the
/// positive series `Σ z^(2k+1)/(2k+1)` carried outward, its remainder bounded
/// by the geometric tail `t/(1 − z²)` of the next term `t`. Odd: a negative
/// `z` is `−atanh(−z)`.
fn atanh(z: &Q, bits: u32) -> Encl {
    if z.signum() < 0 {
        return atanh(&-z.clone(), bits).neg();
    }
    let g = bits + 16;
    let (zlo, zhi) = (down(z, g), up(z, g));
    let (z2lo, z2hi) = (down(&(&zlo * &zlo), g), up(&(&zhi * &zhi), g));
    // stop 8 bits coarser than the rounding grid (see `exp`)
    let tiny = Q::from_big(BigRational::new(BigInt::one(), pow2(g - 8)));
    let (mut plo, mut phi) = (zlo, zhi);
    let (mut slo, mut shi) = (Q::ZERO, Q::ZERO);
    let mut k: i64 = 0;
    loop {
        let tlo = down(&plo.checked_div(&Q::int(2 * k + 1)).unwrap_or(Q::ZERO), g);
        let thi = up(&phi.checked_div(&Q::int(2 * k + 1)).unwrap_or(Q::ZERO), g);
        if thi < tiny {
            let tail = up(
                &thi.checked_div(&(&Q::ONE - &z2hi))
                    .unwrap_or_else(|| thi.clone()),
                g,
            );
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
        k += 1;
    }
}

/// `atan(x)` for `|x| ≤ 1/5`: the alternating series `Σ (−1)ᵏ x^(2k+1)/(2k+1)`,
/// whose remainder is bounded by its first omitted term.
fn atan_small(x: &Q, bits: u32) -> Encl {
    let tiny = Q::from_big(BigRational::new(BigInt::one(), pow2(bits)));
    let x2 = x * x;
    let mut power = x.clone();
    let mut sum = Q::ZERO;
    let mut k: i64 = 0;
    loop {
        let t = power.checked_div(&Q::int(2 * k + 1)).unwrap_or(Q::ZERO);
        if t.abs() < tiny {
            let tail = t.abs();
            return Encl {
                lo: &sum - &tail,
                hi: &sum + &tail,
            }
            .outward(bits);
        }
        sum = if k % 2 == 0 { &sum + &t } else { &sum - &t };
        power = &power * &x2;
        k += 1;
    }
}

/// `π`, by Machin: `16·atan(1/5) − 4·atan(1/239)`.
pub fn pi(bits: u32) -> Encl {
    let a = atan_small(&Q::new(1, 5).unwrap_or(Q::ZERO), bits + 8);
    let b = atan_small(&Q::new(1, 239).unwrap_or(Q::ZERO), bits + 8);
    a.mul(&Encl::exact(Q::int(16)))
        .sub(&b.mul(&Encl::exact(Q::int(4))))
        .outward(bits + 4)
}

/// `ln 2 = 2·atanh(1/3)`.
pub fn ln2(bits: u32) -> Encl {
    let a = atanh(&Q::new(1, 3).unwrap_or(Q::ZERO), bits + 4);
    a.add(&a)
}

/// `ln x` for `x > 0`: `x = 2^k·m` with `m ∈ [2/3, 4/3]`, then
/// `ln m = 2·atanh((m−1)/(m+1))` (`|z| ≤ 1/7`) and `k·ln 2`. `None` for
/// `x ≤ 0`.
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

/// `exp` of an enclosure: monotone, so the bounds map to bounds.
pub fn exp_of(x: &Encl, bits: u32) -> Encl {
    Encl {
        lo: exp(&x.lo, bits).lo,
        hi: exp(&x.hi, bits).hi,
    }
}

/// The fixed point of [`exp_fixed`]: values in units of `2^-60`.
pub const FIXED: u32 = 60;

/// `exp(a)` for `a` in `[lo, hi] x 2^-60` with `0 <= lo <= hi < 2^61`,
/// proven, in 128-bit integers: each bound's Taylor series is summed with
/// every product rounded toward its own side (the lower bound's terms
/// floored, the upper's raised), and the upper bound adds the series'
/// remainder (once a term `t_n` falls to one unit at `n >= 4 a`, the rest
/// is below `2 t_n`). An enclosure as much as `exp`'s, at integer speed.
pub fn exp_fixed(lo: i128, hi: i128) -> Encl {
    let one: i128 = 1 << FIXED;
    let series = |x: i128, up: bool| -> i128 {
        let (mut term, mut sum, mut n) = (one, one, 1i128);
        loop {
            let prod = term * x;
            let mut next = (prod >> FIXED) / n;
            if up && (prod & (one - 1) != 0 || (prod >> FIXED) % n != 0) {
                next += 1;
            }
            term = next;
            sum += term;
            if (n >= 4 && term <= 1) || n > 200 {
                return if up { sum + 2 * term + 2 } else { sum };
            }
            n += 1;
        }
    };
    let q = |v: i128| Q::from_i128(v, one).unwrap_or(Q::ZERO);
    Encl {
        lo: q(series(lo.max(0), false)),
        hi: q(series(hi.max(0), true)),
    }
}

/// `q` in units of `2^-60`, rounded down and up.
pub fn fixed_bounds(q: &Q) -> (i128, i128) {
    let n = q.numer() << FIXED;
    let d = q.denom();
    let lo = n.div_floor(&d);
    let hi = n.div_ceil(&d);
    (
        num_traits::ToPrimitive::to_i128(&lo).unwrap_or(i128::MIN),
        num_traits::ToPrimitive::to_i128(&hi).unwrap_or(i128::MAX),
    )
}

/// cos and sin of an angle given in turns (fractions of the full circle),
/// proven, in 128-bit fixed point. The quadrant is read exactly from the
/// turns (angles are turns), the rest `y` in `[0, 1/4)` turn is
/// carried to radians by an enclosure of 2 pi, and on `[0, pi/2)` sine
/// rises and cosine falls, so each is bounded by its series at the ends of
/// `y`'s enclosure: an alternating series with falling terms, wrong by less
/// than its first omitted term, plus one unit per rounded product.
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
        // straddles a quadrant's edge: the hull of both ends and the edge,
        // where cos and sin take 0 or +-1
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
        let hull = |pick: fn(&(Encl, Encl)) -> &Encl| Encl {
            lo: parts
                .iter()
                .map(|p| pick(p).lo.clone())
                .min()
                .unwrap_or(Q::int(-1)),
            hi: parts
                .iter()
                .map(|p| pick(p).hi.clone())
                .max()
                .unwrap_or(Q::ONE),
        };
        return (hull(|p| &p.0), hull(|p| &p.1));
    }
    let quad = num_traits::ToPrimitive::to_i64(&q_lo).unwrap_or(0);
    let q4 = Q::from_reduced(BigRational::new(q_lo, BigInt::from(4)));
    let two_pi = two_pi_fixed();
    let y_lo = (fixed_bounds(&(&f_lo - &q4)).0 * two_pi.0) >> FIXED;
    let y_hi = ((fixed_bounds(&(&f_hi - &q4)).1 * two_pi.1) >> FIXED) + 1;
    let (s_lo, s_hi) = (alternating(y_lo, true).0, alternating(y_hi, true).1);
    let (c_lo, c_hi) = (alternating(y_hi, false).0, alternating(y_lo, false).1);
    let one: i128 = 1 << FIXED;
    let e = |lo: i128, hi: i128| Encl {
        lo: Q::from_i128(lo.max(-one), one).unwrap_or(Q::ZERO),
        hi: Q::from_i128(hi.min(one), one).unwrap_or(Q::ZERO),
    };
    let (s, c) = (e(s_lo, s_hi), e(c_lo, c_hi));
    match quad {
        0 => (c, s),
        1 => (s.neg(), c),
        2 => (c.neg(), s.neg()),
        _ => (s, c.neg()),
    }
}

/// 2 pi's bounds in units of 2^-60.
fn two_pi_fixed() -> (i128, i128) {
    static P: std::sync::OnceLock<(i128, i128)> = std::sync::OnceLock::new();
    *P.get_or_init(|| {
        let p = pi(100);
        let two = Q::int(2);
        (
            fixed_bounds(&(&p.lo * &two)).0,
            fixed_bounds(&(&p.hi * &two)).1,
        )
    })
}

/// sin (`odd`) or cos of `x` in units of 2^-60, `0 <= x < 2`, as bounds:
/// the series summed, give or take its first omitted term and one unit for
/// each rounded product.
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

/// asin x for `0 <= x <= 1/2`, proven: `x` times its series in `x^2`,
/// `1 + x^2/6 + 3x^4/40 + ...` (all terms positive, each the last times
/// `x^2 (2n+1)^2 / ((2n+2)(2n+3))`, so the tail after a term is under a
/// third of it), summed in fixed point floored for the lower bound and
/// raised for the upper; the product keeps `x`'s own exactness however
/// small it is.
pub fn asin_small(x: &Encl) -> Encl {
    let half = Q::new(1, 2).unwrap_or(Q::ZERO);
    if x.lo.signum() < 0 || x.hi > half {
        return Encl {
            lo: Q::ZERO,
            hi: Q::int(2),
        };
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
    let (sq_lo, sq_hi) = (
        fixed_bounds(&(&x.lo * &x.lo)).0,
        fixed_bounds(&(&x.hi * &x.hi)).1,
    );
    let f = Encl {
        lo: Q::from_i128(factor(sq_lo, false), one).unwrap_or(Q::ONE),
        hi: Q::from_i128(factor(sq_hi, true), one).unwrap_or(Q::ONE),
    };
    Encl {
        lo: &x.lo * &f.lo,
        hi: &x.hi * &f.hi,
    }
}

/// asin of any `x` in [0, 1] (radians), enclosed: the small series below a
/// half, else the angle found by halving a bracket of turns on the
/// certified sine (each bound the turn certainly on its side).
pub fn asin_any(x: &Encl) -> Encl {
    let half = Q::new(1, 2).unwrap_or(Q::ZERO);
    if x.hi <= half && x.lo.signum() >= 0 {
        return asin_small(x);
    }
    let quarter = Q::new(1, 4).unwrap_or(Q::ZERO);
    let bound = |target: &Q, below: bool| -> Q {
        let (mut lo, mut hi) = (Q::ZERO, quarter.clone());
        for _ in 0..56 {
            let mid = (&lo + &hi).checked_div(&Q::int(2)).unwrap_or(Q::ZERO);
            let s = cos_sin_turns(&Encl::exact(mid.clone())).1;
            // below: the largest turn whose sine is certainly under target
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

/// `x^p` for `x > 0` and a rational power `p`: `exp(p·ln x)`, enclosed.
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
        // a decimal literal as an exact rational
        let (int, frac) = s.split_once('.').unwrap_or((s, ""));
        let den = 10i128.pow(u32::try_from(frac.len()).unwrap());
        let num: i128 = format!("{int}{frac}").parse().unwrap();
        Q::from_i128(num, den).unwrap()
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
        // negative arguments: exp(-3)·exp(3) encloses 1
        let p = exp(&Q::int(-3), 80).mul(&exp(&Q::int(3), 80));
        assert!(p.contains(&Q::ONE));
    }

    #[test]
    fn pi_by_machin_is_enclosed_tightly() {
        let p = pi(80);
        assert!(p.lo < dec("3.14159265358979324") && p.hi > dec("3.14159265358979323"));
        assert!(&p.hi - &p.lo < Q::from_big(BigRational::new(BigInt::one(), pow2(76))));
    }
}
