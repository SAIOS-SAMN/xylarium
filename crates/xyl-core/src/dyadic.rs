//! Dyadic rationals for the heavy physical models: `m x 2^e` with a 62-bit
//! integer mantissa, every operation done in integers and rounded toward
//! zero.
//!
//! Each value is an exact rational (a dyadic one), the same on every machine
//! and thread count. It is not a hardware float: no platform rounding mode,
//! no fused operations, no library `exp` whose last bit varies. It is what a
//! float would be if it were exactly specified. A model whose quantities span
//! 10^-12 to 10^30 (the heavy physical readings) keeps 62 bits of precision
//! at every scale, at integer speed.
//!
//! Precision: + - x / round toward zero to 62 bits (relative error below
//! 2^-61). `exp`, `ln`, `sqrt` and `pow` are evaluated in 62-bit fixed point
//! after range reduction, accurate to within 2^-55 relative. Where a decision
//! must be proven rather than computed, use `certified` enclosures instead.

use crate::Q;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};
use std::cmp::Ordering;

const MANT: u32 = 62;
const TOP: i128 = 1 << MANT;
const HALF_TOP: i128 = 1 << (MANT - 1);

/// `m x 2^e`, normalised: `m == 0`, or `2^61 <= |m| < 2^62`.
///
/// ```
/// use xyl_core::Q;
/// use xyl_core::dyadic::Dy;
///
/// let quarter = Dy::ratio(1, 4);
/// assert_eq!(quarter.to_q(), Q::new(1, 4).unwrap_or_default()); // exact
/// assert!(Dy::ratio(1, 3) < Dy::ratio(1, 2));
/// // 1/3 is rounded toward zero, so three of them stay below one.
/// assert!(Dy::ratio(1, 3) * Dy::int(3) < Dy::ONE);
/// assert_eq!(Dy::ratio(1, 0), Dy::ZERO);
/// assert_eq!(Dy::int(2).div(&Dy::ZERO), None);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Dy {
    m: i64,
    e: i32,
}

fn norm(m: i128, e: i64) -> Dy {
    if m == 0 {
        return Dy::ZERO;
    }
    let neg = m < 0;
    let mut a = m.unsigned_abs();
    let mut e = e;
    while a >= TOP as u128 {
        a >>= 1;
        e += 1;
    }
    while a < HALF_TOP as u128 {
        a <<= 1;
        e -= 1;
    }
    let mi = i64::try_from(a).unwrap_or(i64::MAX);
    let ei = i32::try_from(e.clamp(i64::from(i32::MIN), i64::from(i32::MAX))).unwrap_or(0);
    Dy {
        m: if neg { -mi } else { mi },
        e: ei,
    }
}

impl Dy {
    /// Zero, in its one normal form (`m = 0`, `e = 0`).
    pub const ZERO: Dy = Dy { m: 0, e: 0 };

    /// The integer `n`; exact when `|n| < 2^62`, otherwise rounded toward
    /// zero to 62 bits.
    pub fn int(n: i64) -> Dy {
        norm(i128::from(n), 0)
    }

    /// Whether the value is zero.
    pub fn is_zero(&self) -> bool {
        self.m == 0
    }

    /// The sign of the value: `-1`, `0` or `1`.
    pub fn signum(&self) -> i8 {
        match self.m.cmp(&0) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }

    /// The nearest dyadic toward zero to a rational.
    pub fn from_q(q: &Q) -> Dy {
        let r = q.to_big();
        if r.is_zero() {
            return Dy::ZERO;
        }
        let (n, d) = (r.numer().clone(), r.denom().clone());
        // choose a shift so n * 2^s / d has about 62 bits
        let bits = |x: &BigInt| i64::try_from(x.abs().bits()).unwrap_or(0);
        let s = i64::from(MANT) + 1 - (bits(&n) - bits(&d));
        let scaled = if s >= 0 {
            (n << usize::try_from(s).unwrap_or(0)) / d
        } else {
            (n >> usize::try_from(-s).unwrap_or(0)) / d
        };
        norm(scaled.to_i128().unwrap_or(0), -s)
    }

    /// This dyadic as the exact rational it is.
    pub fn to_q(&self) -> Q {
        if self.m == 0 {
            return Q::ZERO;
        }
        let m = BigInt::from(self.m);
        let r = if self.e >= 0 {
            BigRational::from_integer(m << usize::try_from(self.e).unwrap_or(0))
        } else {
            BigRational::new(m, BigInt::one() << usize::try_from(-self.e).unwrap_or(0))
        };
        Q::from_big(r)
    }

    /// The negation, exact.
    pub fn neg(&self) -> Dy {
        Dy {
            m: -self.m,
            e: self.e,
        }
    }

    /// The absolute value, exact.
    pub fn abs(&self) -> Dy {
        Dy {
            m: self.m.abs(),
            e: self.e,
        }
    }

    /// The sum, rounded toward zero to 62 bits; an addend more than 126
    /// binary orders below the other is dropped.
    pub fn add(&self, o: &Dy) -> Dy {
        if self.m == 0 {
            return *o;
        }
        if o.m == 0 {
            return *self;
        }
        let (hi, lo) = if self.e >= o.e { (self, o) } else { (o, self) };
        let shift = i64::from(hi.e) - i64::from(lo.e);
        if shift > 126 {
            return *hi;
        }
        // align at the lower exponent, with the higher scaled up as far as is safe
        let up = shift.min(62);
        let a = i128::from(hi.m) << up;
        let b = i128::from(lo.m) >> (shift - up);
        norm(a + b, i64::from(hi.e) - up)
    }

    /// The difference, rounded toward zero to 62 bits.
    pub fn sub(&self, o: &Dy) -> Dy {
        self.add(&o.neg())
    }

    /// The product, rounded toward zero to 62 bits.
    pub fn mul(&self, o: &Dy) -> Dy {
        norm(
            i128::from(self.m) * i128::from(o.m),
            i64::from(self.e) + i64::from(o.e),
        )
    }

    /// The quotient, rounded toward zero to 62 bits.
    ///
    /// Returns `None` when `o` is zero.
    pub fn div(&self, o: &Dy) -> Option<Dy> {
        if o.m == 0 {
            return None;
        }
        let num = i128::from(self.m) << MANT;
        Some(norm(
            num / i128::from(o.m),
            i64::from(self.e) - i64::from(o.e) - i64::from(MANT),
        ))
    }

    /// The larger of the two values.
    pub fn max(self, o: Dy) -> Dy {
        if self >= o {
            self
        } else {
            o
        }
    }

    /// The smaller of the two values.
    pub fn min(self, o: Dy) -> Dy {
        if self <= o {
            self
        } else {
            o
        }
    }

    /// `(mantissa, exponent)` with the value `m / 2^62 x 2^(e + 62)`: m in [2^61, 2^62).
    fn parts(&self) -> (i128, i64) {
        (i128::from(self.m), i64::from(self.e))
    }

    /// The square root of a non-negative value; `None` for a negative one.
    pub fn sqrt(&self) -> Option<Dy> {
        if self.m < 0 {
            return None;
        }
        if self.m == 0 {
            return Some(Dy::ZERO);
        }
        let (m, e) = self.parts();
        // make the exponent even, then sqrt(m * 2^62) * 2^((e - 62)/2)
        let (m, e) = if e.rem_euclid(2) != 0 {
            (m << 1, e - 1)
        } else {
            (m, e)
        };
        let r = num_integer::Roots::sqrt(&(m << MANT));
        Some(norm(r, (e - i64::from(MANT)) / 2))
    }

    /// The natural logarithm of a positive value; `None` otherwise.
    pub fn ln(&self) -> Option<Dy> {
        if self.m <= 0 {
            return None;
        }
        // x = f * 2^k with f = m / 2^62 in [1/2, 1): ln x = ln f + k ln 2
        let (m, e) = self.parts();
        let k = e + i64::from(MANT);
        let lnf = ln_fixed(m);
        let total = lnf + i128::from(k) * ln2_fixed();
        Some(norm(total, -i64::from(MANT)))
    }

    /// `e^x`.
    pub fn exp(&self) -> Dy {
        if self.m == 0 {
            return Dy::int(1);
        }
        // x in 2^-62 fixed point (saturating far outside the useful range)
        let fx = to_fixed(self);
        let Some(fx) = fx else {
            return if self.m > 0 {
                Dy {
                    m: i64::MAX >> 1,
                    e: i32::MAX / 2,
                }
            } else {
                Dy::ZERO
            };
        };
        let l2 = ln2_fixed();
        // x = k ln2 + r, |r| <= ln2/2
        let k = (fx + (if fx >= 0 { l2 / 2 } else { -l2 / 2 })) / l2;
        let r = fx - k * l2;
        norm(
            exp_fixed(r),
            -i64::from(MANT) + i64::try_from(k).unwrap_or(0),
        )
    }

    /// `x^y` for positive x: `exp(y ln x)`; `None` for x <= 0 (unless y == 0).
    pub fn pow(&self, y: &Dy) -> Option<Dy> {
        if y.is_zero() {
            return Some(Dy::int(1));
        }
        Some(y.mul(&self.ln()?).exp())
    }
}

impl Dy {
    /// pi, to 62 bits.
    pub const PI: Dy = Dy {
        m: 3_622_009_729_038_561_421,
        e: -60,
    };
    /// One, exact.
    pub const ONE: Dy = Dy { m: 1 << 61, e: -61 };

    /// `n / d` (zero when `d` is zero).
    pub fn ratio(n: i64, d: i64) -> Dy {
        Dy::int(n) / Dy::int(d)
    }

    /// `mant x 10^exp10`: a measured constant written as a scientific literal.
    pub fn sci(mant: i64, exp10: i32) -> Dy {
        let mut x = Dy::int(mant);
        let ten = Dy::int(10);
        for _ in 0..exp10.unsigned_abs() {
            x = if exp10 > 0 { x * ten } else { x / ten };
        }
        x
    }

    /// `x^n` for an integer power (by squaring).
    pub fn powi(&self, n: i32) -> Dy {
        let mut base = if n < 0 { Dy::ONE / *self } else { *self };
        let mut k = n.unsigned_abs();
        let mut acc = Dy::ONE;
        while k > 0 {
            if k & 1 == 1 {
                acc = acc * base;
            }
            base = base * base;
            k >>= 1;
        }
        acc
    }

    /// `x^p` for positive x, zero otherwise (the physical models' convention:
    /// a non-positive base contributes nothing).
    pub fn powf(&self, p: Dy) -> Dy {
        if self.m <= 0 {
            return Dy::ZERO;
        }
        self.pow(&p).unwrap_or(Dy::ZERO)
    }

    /// `sqrt(x)` for x >= 0, zero otherwise.
    pub fn root(&self) -> Dy {
        self.sqrt().unwrap_or(Dy::ZERO)
    }

    /// `ln(x)` for x > 0; for x <= 0 the log of the least positive dyadic the
    /// models use (2^-200), so a vanishing quantity has a very negative log.
    pub fn log(&self) -> Dy {
        self.ln().unwrap_or_else(|| Dy::int(-200) * ln2())
    }

    /// The value limited to `[lo, hi]` (`max(lo)` then `min(hi)`, so `hi`
    /// wins when `lo > hi`).
    pub fn clamp(self, lo: Dy, hi: Dy) -> Dy {
        self.max(lo).min(hi)
    }

    /// The nearest integer toward zero, saturating.
    pub fn trunc_i64(&self) -> i64 {
        to_fixed(self)
            .map(|f| {
                // an arithmetic shift floors; a negative value is shifted by
                // its magnitude so the result rounds toward zero
                let whole = if f < 0 { -((-f) >> MANT) } else { f >> MANT };
                i64::try_from(whole).unwrap_or(if self.m > 0 { i64::MAX } else { i64::MIN })
            })
            .unwrap_or(if self.m > 0 { i64::MAX } else { i64::MIN })
    }

    /// Rounded to a grid of 2^-bits (toward minus infinity): for values that
    /// are kept or compared across runs at a stated resolution.
    pub fn grid(&self, bits: u32) -> Dy {
        let shift = -i64::from(bits) - i64::from(self.e);
        if shift <= 0 {
            return *self;
        }
        if shift >= 63 {
            return if self.m < 0 {
                norm(-1, -i64::from(bits))
            } else {
                Dy::ZERO
            };
        }
        let m = i128::from(self.m) >> shift;
        norm(m, -i64::from(bits))
    }
}

impl Dy {
    /// `(sin x, cos x)`: reduced by quarter turns to |r| <= pi/4, then the
    /// Taylor series in 62-bit fixed point.
    pub fn sin_cos(&self) -> (Dy, Dy) {
        let half_pi = Dy::PI * Dy::ratio(1, 2);
        let q = (*self / half_pi
            + if self.m >= 0 {
                Dy::ratio(1, 2)
            } else {
                Dy::ratio(-1, 2)
            })
        .trunc_i64();
        let r = *self - Dy::int(q) * half_pi;
        let Some(f) = to_fixed(&r) else {
            return (Dy::ZERO, Dy::ONE);
        };
        let f2 = (f * f) >> MANT;
        // sin r = r - r^3/3! + ...; cos r = 1 - r^2/2! + ...
        let (mut ts, mut s, mut tc, mut c, mut i) = (f, f, TOP, TOP, 1i128);
        while (ts != 0 || tc != 0) && i < 40 {
            ts = -((ts * f2) >> MANT) / ((2 * i) * (2 * i + 1));
            tc = -((tc * f2) >> MANT) / ((2 * i - 1) * (2 * i));
            s += ts;
            c += tc;
            i += 1;
        }
        let (sd, cd) = (norm(s, -i64::from(MANT)), norm(c, -i64::from(MANT)));
        match q.rem_euclid(4) {
            0 => (sd, cd),
            1 => (cd, -sd),
            2 => (-sd, -cd),
            _ => (-cd, sd),
        }
    }

    /// `sin x` for `x` in radians (the first half of [`Dy::sin_cos`]).
    pub fn sin(&self) -> Dy {
        self.sin_cos().0
    }

    /// `cos x` for `x` in radians (the second half of [`Dy::sin_cos`]).
    pub fn cos(&self) -> Dy {
        self.sin_cos().1
    }

    /// `atan x`: for |x| > 1 by pi/2 - atan(1/x); halved twice by
    /// atan x = 2 atan(x / (1 + sqrt(1 + x^2))) to |x| < 0.2; then the series.
    pub fn atan(&self) -> Dy {
        if self.m == 0 {
            return Dy::ZERO;
        }
        if self.abs() > Dy::ONE {
            let h = Dy::PI * Dy::ratio(1, 2);
            let inner = (Dy::ONE / *self).atan();
            return if self.m > 0 { h - inner } else { -h - inner };
        }
        let mut x = *self;
        let mut scale = 1i64;
        while x.abs() > Dy::ratio(1, 5) {
            x = x / (Dy::ONE + (Dy::ONE + x * x).root());
            scale *= 2;
        }
        let Some(f) = to_fixed(&x) else {
            return Dy::ZERO;
        };
        let f2 = (f * f) >> MANT;
        let (mut p, mut sum, mut k) = (f, 0i128, 0i128);
        while p != 0 && k < 200 {
            let term = p / (2 * k + 1);
            sum += if k % 2 == 0 { term } else { -term };
            p = (p * f2) >> MANT;
            k += 1;
        }
        norm(sum, -i64::from(MANT)) * Dy::int(scale)
    }

    /// `acos x` for x in [-1, 1] (clamped): `atan2(sqrt(1 - x^2), x)`.
    pub fn acos(&self) -> Dy {
        let x = Dy::clamp(*self, -Dy::ONE, Dy::ONE);
        let s = (Dy::ONE - x * x).max(Dy::ZERO).root();
        if x.m == 0 {
            return Dy::PI * Dy::ratio(1, 2);
        }
        let a = (s / x).atan();
        if x.m > 0 {
            a
        } else {
            a + Dy::PI
        }
    }

    /// `cosh x`.
    pub fn cosh(&self) -> Dy {
        let e = self.exp();
        (e + Dy::ONE / e) * Dy::ratio(1, 2)
    }

    /// A uniform draw in [0, 1) from 64 random bits (53 of them, as a
    /// double's mantissa would hold: the same draws on every machine).
    pub fn unit(bits: u64) -> Dy {
        norm(i128::from(bits >> 11), -53)
    }
}

impl Dy {
    /// The exact bits: (mantissa, exponent), for codecs.
    pub fn to_bits(&self) -> (i64, i32) {
        (self.m, self.e)
    }

    /// A dyadic from its bits, refused unless in normal form (a codec must not
    /// accept two spellings of one value).
    pub fn from_bits(m: i64, e: i32) -> Option<Dy> {
        let d = norm(i128::from(m), i64::from(e));
        (d.m == m && (m != 0 || e == 0) && d.e == e).then_some(d)
    }
}

/// `ln 2` as a dyadic.
pub fn ln2() -> Dy {
    norm(ln2_fixed(), -i64::from(MANT))
}

impl std::ops::Add for Dy {
    type Output = Dy;
    fn add(self, o: Dy) -> Dy {
        Dy::add(&self, &o)
    }
}

impl std::ops::Sub for Dy {
    type Output = Dy;
    fn sub(self, o: Dy) -> Dy {
        Dy::sub(&self, &o)
    }
}

impl std::ops::Mul for Dy {
    type Output = Dy;
    fn mul(self, o: Dy) -> Dy {
        Dy::mul(&self, &o)
    }
}

/// Division; `x / 0` is 0 (the models guard every denominator they mean).
impl std::ops::Div for Dy {
    type Output = Dy;
    fn div(self, o: Dy) -> Dy {
        Dy::div(&self, &o).unwrap_or(Dy::ZERO)
    }
}

impl std::ops::Neg for Dy {
    type Output = Dy;
    fn neg(self) -> Dy {
        Dy::neg(&self)
    }
}

impl std::ops::AddAssign for Dy {
    fn add_assign(&mut self, o: Dy) {
        *self = *self + o;
    }
}

impl std::ops::SubAssign for Dy {
    fn sub_assign(&mut self, o: Dy) {
        *self = *self - o;
    }
}

impl std::iter::Sum for Dy {
    fn sum<I: Iterator<Item = Dy>>(it: I) -> Dy {
        it.fold(Dy::ZERO, |a, b| a + b)
    }
}

impl std::fmt::Display for Dy {
    /// A decimal rendering to 12 significant figures (display only).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.m == 0 {
            return write!(f, "0");
        }
        let neg = self.m < 0;
        let mut x = self.abs();
        let ten = Dy::int(10);
        let mut e10 = 0i32;
        while x >= Dy::int(10) {
            x = x / ten;
            e10 += 1;
        }
        while x < Dy::ONE {
            x = x * ten;
            e10 -= 1;
        }
        let digits = (x * Dy::sci(1, 11) + Dy::ratio(1, 2))
            .trunc_i64()
            .min(999_999_999_999);
        let s = digits.to_string();
        let (a, b) = s.split_at(1);
        write!(f, "{}{a}.{b}e{e10}", if neg { "-" } else { "" })
    }
}

/// A dyadic in 2^-62 fixed point, if it fits (|x| < 2^64).
fn to_fixed(x: &Dy) -> Option<i128> {
    let (m, e) = x.parts();
    let shift = e + i64::from(MANT);
    if shift >= 0 {
        if shift > 64 {
            return None;
        }
        m.checked_shl(u32::try_from(shift).ok()?)
    } else {
        let s = u32::try_from(-shift).ok()?;
        Some(if s >= 127 { 0 } else { m >> s })
    }
}

/// `ln 2` in 2^-62 fixed point (floor of ln 2 x 2^62, computed to 80 digits).
fn ln2_fixed() -> i128 {
    3_196_577_161_300_663_914
}

/// atanh(z) for |z| <= 1/3 in 2^-62 fixed point: sum z^(2k+1)/(2k+1).
fn atanh_fixed(z: i128) -> i128 {
    let z2 = (z * z) >> MANT;
    let (mut p, mut s, mut k) = (z, 0i128, 0i128);
    while p != 0 && k < 200 {
        s += p / (2 * k + 1);
        p = (p * z2) >> MANT;
        k += 1;
    }
    s
}

/// ln(m / 2^62) for m in [2^61, 2^62), in 2^-62 fixed point:
/// 2 atanh((f - 1)/(f + 1)), |z| <= 1/3.
fn ln_fixed(m: i128) -> i128 {
    let z = ((m - TOP) << MANT) / (m + TOP);
    2 * atanh_fixed(z)
}

/// e^r for |r| <= ln2/2 in 2^-62 fixed point: the Taylor series.
fn exp_fixed(r: i128) -> i128 {
    let (mut term, mut s, mut i) = (TOP, TOP, 1i128);
    while term != 0 && i < 60 {
        term = ((term * r) >> MANT) / i;
        s += term;
        i += 1;
    }
    s
}

impl PartialOrd for Dy {
    fn partial_cmp(&self, o: &Dy) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl Ord for Dy {
    fn cmp(&self, o: &Dy) -> Ordering {
        let d = self.sub(o);
        d.m.cmp(&0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: &Dy, want: &str, tol: &str) -> bool {
        let a = a.to_q();
        let want = dec(want);
        (&a - &want).abs() <= dec(tol)
    }

    fn dec(s: &str) -> Q {
        let (int, frac) = s.split_once('.').unwrap_or((s, ""));
        let neg = int.starts_with('-');
        let digits = format!("{}{frac}", int.trim_start_matches('-'));
        let n: i128 = digits.parse().unwrap();
        let q = Q::from_i128(n, 10i128.pow(u32::try_from(frac.len()).unwrap())).unwrap();
        if neg {
            -q
        } else {
            q
        }
    }

    #[test]
    fn arithmetic_is_exact_on_the_dyadic_grid() {
        let a = Dy::from_q(&Q::new(3, 4).unwrap());
        let b = Dy::from_q(&Q::new(5, 8).unwrap());
        assert_eq!(a.add(&b).to_q(), Q::new(11, 8).unwrap());
        assert_eq!(a.mul(&b).to_q(), Q::new(15, 32).unwrap());
        assert_eq!(a.sub(&b).to_q(), Q::new(1, 8).unwrap());
        assert_eq!(
            Dy::int(1_000_000).mul(&Dy::int(1_000_000)).to_q(),
            Q::int(1_000_000_000_000)
        );
        // huge and tiny keep their 62 bits
        let big =
            Dy::from_q(&Q::from_i128(5972, 1).unwrap()).mul(&Dy::int(1_000_000_000_000_000_000));
        assert!(big > Dy::int(i64::MAX));
    }

    #[test]
    fn operators_constants_and_powers() {
        assert!(near(
            &Dy::PI,
            "3.14159265358979323846",
            "0.000000000000000001"
        ));
        assert!(near(
            &ln2(),
            "0.69314718055994530942",
            "0.000000000000000001"
        ));
        assert_eq!(Dy::ONE.to_q(), Q::ONE);
        assert_eq!(
            Dy::sci(5972, 21).to_q(),
            Q::from_i128(5_972_000_000_000_000_000_000_000, 1).unwrap()
        );
        assert_eq!(Dy::int(3).powi(4).to_q(), Q::int(81));
        assert_eq!(Dy::int(2).powi(-3).to_q(), Q::new(1, 8).unwrap());
        assert!(near(
            &Dy::int(27).powf(Dy::ratio(1, 3)),
            "3",
            "0.00000000000001"
        ));
        assert_eq!(Dy::int(1) / Dy::ZERO, Dy::ZERO);
        assert_eq!(Dy::ratio(7, 2).trunc_i64(), 3);
        assert_eq!(Dy::ratio(-7, 2).trunc_i64(), -3);
        // sin and cos of an angle just past a negative half quarter turn
        let (s, c) = Dy::ratio(-63, 80).sin_cos();
        assert!(s < Dy::ZERO && c > Dy::ZERO);
        assert_eq!(format!("{}", Dy::int(-1234)), "-1.23400000000e3");
        assert_eq!(Dy::ratio(5, 3).grid(4).to_q(), Q::new(26, 16).unwrap());
    }

    #[test]
    fn trigonometry_and_gamma() {
        let tol = "0.00000000000001";
        assert!(near(&Dy::ratio(1, 2).sin(), "0.47942553860420300027", tol));
        assert!(near(&Dy::ratio(1, 2).cos(), "0.87758256189037271612", tol));
        assert!(near(&Dy::int(3).sin(), "0.14112000805986722210", tol));
        assert!(near(&Dy::int(-5).cos(), "0.28366218546322626447", tol));
        assert!(near(&Dy::int(2).atan(), "1.10714871779409050302", tol));
        assert!(near(
            &Dy::ratio(-1, 3).atan(),
            "-0.32175055439664219340",
            tol
        ));
        assert!(near(&Dy::ratio(1, 4).acos(), "1.31811607165281796574", tol));
        assert!(near(
            &Dy::ratio(-3, 4).acos(),
            "2.41885840577637762338",
            tol
        ));
        assert!(near(&Dy::int(1).cosh(), "1.54308063481524377848", tol));
        assert_eq!(Dy::unit(0), Dy::ZERO);
        let x = Dy::ratio(-22, 7);
        let (m, e) = x.to_bits();
        assert_eq!(Dy::from_bits(m, e), Some(x));
        assert_eq!(Dy::from_bits(m / 2, e + 1), None);
        assert!(Dy::unit(u64::MAX) < Dy::ONE);
    }

    #[test]
    fn transcendentals_agree_with_the_constants() {
        assert!(near(
            &Dy::int(1).exp(),
            "2.71828182845904523536",
            "0.000000000000001"
        ));
        assert!(near(
            &Dy::int(2).ln().unwrap(),
            "0.69314718055994530942",
            "0.000000000000001"
        ));
        assert!(near(
            &Dy::int(2).sqrt().unwrap(),
            "1.41421356237309504880",
            "0.000000000000001"
        ));
        assert!(near(
            &Dy::int(10)
                .pow(&Dy::from_q(&Q::new(1, 2).unwrap()))
                .unwrap(),
            "3.16227766016837933200",
            "0.00000000000001"
        ));
        assert!(near(
            &Dy::int(-3).exp(),
            "0.04978706836786394298",
            "0.000000000000001"
        ));
        let x = Dy::from_q(&Q::new(12345, 7).unwrap());
        assert!(near(&x.ln().unwrap().exp().sub(&x), "0", "0.00000000001"));
        assert!(Dy::int(0).ln().is_none() && Dy::int(-1).sqrt().is_none());
    }
}
