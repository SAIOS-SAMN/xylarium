//! Dyadic rationals for the heavy physical models: `m x 2^e` with a 62-bit
//! integer mantissa, every operation done in integers and rounded toward zero.
//!
//! Inputs: integers, ratios and [`Q`] values (`int`, `ratio`, `sci`,
//! `from_q`); the constants pi and ln 2 are read once, rounded down, from
//! the enclosures of `certified`.
//!
//! Outputs: [`Dy`], normalised (`m == 0` or `2^61 <= |m| < 2^62`), with
//! arithmetic (also as `+ - * /` operators), `sqrt`, `ln`, `exp`, `pow`,
//! `powi`, `sin_cos`, `atan`, `acos`, `grid` (rounding to `2^-bits`),
//! `unit` (a uniform draw from 64 random bits), `to_q` (exact) and the codec
//! pair `to_bits` / `from_bits`.
//!
//! Invariants: each value is an exact (dyadic) rational and the same on every
//! machine and thread count. No hardware float is used: no platform rounding
//! mode, no fused operations, no library `exp` whose last bit varies. A model
//! whose quantities span 10^-12 to 10^30 keeps 62 bits of precision at every
//! scale.
//!
//! Precision: + - x and division round toward zero to 62 bits (relative
//! error below 2^-61). `exp`, `ln`, `sqrt` and `pow` are evaluated in 62-bit
//! fixed point after range reduction, accurate to within 2^-55 relative.
//! There is no `/` operator: `div` refuses a zero divisor (`None`), and
//! `over` divides by a nonzero integer (`nonzero!`, `dyadic!` for a written
//! ratio, both checked at build time). `sqrt`, `ln` and `pow` return `None`
//! outside their domain, and `trunc_i64` a whole part past `i64`; `exp`
//! saturates past `2^63` halvings. `from_bits` refuses a value not in normal
//! form.
//! Where a decision must be proven rather than computed, use `certified`
//! enclosures instead.
//!
//! Terms: `Dy` (docs/glossary.md).

use crate::Q;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};
use std::cmp::Ordering;
use std::sync::OnceLock;

const MANT: u32 = 62;
const TOP: i128 = 1 << MANT;
const HALF_TOP: i128 = 1 << (MANT - 1);

/// `m x 2^e`, normalised: `m == 0`, or `2^61 <= |m| < 2^62`.
///
/// ```
/// use xyl_core::Q;
/// use xyl_core::dyadic::Dy;
///
/// let quarter = xyl_core::dyadic!(1, 4);
/// assert_eq!(Some(quarter.to_q()), Q::new(1, 4)); // Exact.
/// assert!(xyl_core::dyadic!(1, 3) < xyl_core::dyadic!(1, 2));
/// // 1/3 is rounded toward zero, so three of them stay below one.
/// assert!(xyl_core::dyadic!(1, 3) * Dy::int(3) < Dy::ONE);
/// // A zero divisor is refused; a written zero denominator fails the build.
/// assert_eq!(Dy::int(2).div(&Dy::ZERO), None);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Dy {
    m: i64,
    e: i32,
}

/// A nonzero integer constant written in code, checked when the build
/// evaluates it: `nonzero!(10)` is a `NonZeroI64`, and a zero fails the
/// build.
#[macro_export]
macro_rules! nonzero {
    ($d:expr) => {
        const { $crate::q::written_denominator($d) }
    };
}

/// A ratio written in code as a reading, `dyadic!(n, d)`: `n / d` rounded
/// toward zero to 62 bits, its denominator checked at build time
/// ([`nonzero!`](crate::nonzero)).
#[macro_export]
macro_rules! dyadic {
    ($n:expr, $d:expr $(,)?) => {
        $crate::dyadic::Dy::int($n).over($crate::nonzero!($d))
    };
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
    // The loops above leave `2^61 <= a < 2^62`, and the exponent is clamped
    // to the range of `i32`, so neither narrowing loses a bit.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "normalised: the mantissa is below 2^62 and the exponent clamped to i32"
    )]
    let (mi, ei) = (
        a as i64,
        e.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
    );
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

    /// The rational `q` rounded toward zero to 62 bits.
    pub fn from_q(q: &Q) -> Dy {
        let r = q.to_big();
        if r.is_zero() {
            return Dy::ZERO;
        }
        let (n, d) = (r.numer().clone(), r.denom().clone());
        // The shift `s` is chosen so that `n * 2^s / d` has about 62 bits:
        // with `2^(b-1) <= |x| < 2^b` for `b` bits, the quotient is below
        // `2^(b_n + s - b_d + 1) = 2^64`.
        let bits = |x: &BigInt| i128::from(x.abs().bits());
        let s = i128::from(MANT) + 1 - (bits(&n) - bits(&d));
        let scaled = if s >= 0 {
            (n << s.unsigned_abs()) / d
        } else {
            (n >> s.unsigned_abs()) / d
        };
        // A rational held in memory has fewer than 2^63 bits, so the
        // exponent fits i64 before `norm` clamps it to i32.
        #[expect(
            clippy::cast_possible_truncation,
            reason = "clamped to the range of i64 on the same line"
        )]
        let e = (-s).clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64;
        // The quotient is below 2^64 in magnitude (above), so it fits i128
        // (`tests::from_q_holds_every_rational_to_62_bits`).
        scaled.to_i128().map_or(Dy::ZERO, |m| norm(m, e))
    }

    /// This dyadic as the exact rational it is.
    pub fn to_q(&self) -> Q {
        if self.m == 0 {
            return Q::ZERO;
        }
        let m = BigInt::from(self.m);
        let shift = self.e.unsigned_abs();
        let r = if self.e >= 0 {
            BigRational::from_integer(m << shift)
        } else {
            BigRational::new(m, BigInt::one() << shift)
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

    /// The sum, rounded to 62 bits: toward zero, except that an addend more
    /// than 62 binary orders below the other is first floored, and one more
    /// than 126 orders below is dropped.
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
        // The mantissas are aligned at the lower exponent; the larger is
        // shifted up by at most 62 bits so that the sum fits an i128.
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

    /// The mantissa and exponent as wide integers: the value is `m x 2^e`,
    /// with |m| in [2^61, 2^62) unless the value is zero.
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
        // The exponent is made even; the root is then
        // sqrt(m * 2^62) * 2^((e - 62) / 2).
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
        // `x = f * 2^k` with `f = m / 2^62` in [1/2, 1), so
        // `ln x = ln f + k ln 2`.
        let (m, e) = self.parts();
        let k = e + i64::from(MANT);
        let lnf = ln_fixed(m);
        let total = lnf + i128::from(k) * ln2_fixed();
        Some(norm(total, -i64::from(MANT)))
    }

    /// `e^x`, by range reduction to `x = k ln 2 + r` and the Taylor series of
    /// `e^r` in fixed point. Past `2^63` halvings (`|x| > 2^63 ln 2`, about
    /// `6.4 x 10^18`) the result saturates to a very large value or to zero.
    pub fn exp(&self) -> Dy {
        if self.m == 0 {
            return Dy::int(1);
        }
        let saturated = if self.m > 0 {
            Dy {
                m: i64::MAX >> 1,
                e: i32::MAX / 2,
            }
        } else {
            Dy::ZERO
        };
        // The argument in 2^-62 fixed point, which holds |x| < 2^64.
        let Some(fx) = to_fixed(self) else {
            return saturated;
        };
        let l2 = ln2_fixed();
        // `x = k ln2 + r` with `k` the nearest integer, so |r| <= ln2/2.
        let k = (fx + (if fx >= 0 { l2 / 2 } else { -l2 / 2 })) / l2;
        let r = fx - k * l2;
        let Ok(k) = i64::try_from(k) else {
            return saturated;
        };
        norm(exp_fixed(r), -i64::from(MANT) + k)
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
    /// The constant pi to 62 bits, rounded down: read once from the certified enclosure
    /// `certified::pi`.
    pub fn pi() -> Dy {
        static PI: OnceLock<Dy> = OnceLock::new();
        *PI.get_or_init(|| Dy::from_q(&crate::certified::pi(80).lo))
    }
    /// One, exact.
    pub const ONE: Dy = Dy { m: 1 << 61, e: -61 };

    /// `self / d` for a nonzero integer divisor, rounded toward zero to 62
    /// bits. `Dy::int` of a nonzero integer is nonzero
    /// (`tests::over_agrees_with_div_for_every_nonzero_divisor`), so the zero
    /// branch is never taken.
    pub fn over(&self, d: std::num::NonZeroI64) -> Dy {
        self.div(&Dy::int(d.get())).map_or(Dy::ZERO, |q| q)
    }

    /// `mant x 10^exp10`: a measured constant written as a scientific literal.
    /// Each multiplication or division by ten is rounded toward zero.
    pub fn sci(mant: i64, exp10: i32) -> Dy {
        let mut x = Dy::int(mant);
        let ten = Dy::int(10);
        for _ in 0..exp10.unsigned_abs() {
            x = if exp10 > 0 {
                x * ten
            } else {
                x.over(crate::nonzero!(10))
            };
        }
        x
    }

    /// `x^n` for a natural power, by repeated squaring. A negative power is
    /// `Dy::ONE.div(&x.powi(n))`, which refuses zero.
    pub fn powi(&self, n: u32) -> Dy {
        let mut base = *self;
        let mut k = n;
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

    /// The value limited to `[lo, hi]` (`max(lo)` then `min(hi)`, so `hi`
    /// wins when `lo > hi`).
    pub fn clamp(self, lo: Dy, hi: Dy) -> Dy {
        self.max(lo).min(hi)
    }

    /// The nearest integer toward zero; `None` when it does not fit `i64`
    /// (`|x| >= 2^63`).
    pub fn trunc_i64(&self) -> Option<i64> {
        let f = to_fixed(self)?;
        // An arithmetic shift floors; a negative value is shifted by its
        // magnitude so that the result rounds toward zero.
        let whole = if f < 0 { -((-f) >> MANT) } else { f >> MANT };
        i64::try_from(whole).ok()
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
    /// `(sin x, cos x)`: reduced by the nearest whole number of quarter turns
    /// to |r| <= pi/4, then the Taylor series in 62-bit fixed point.
    pub fn sin_cos(&self) -> (Dy, Dy) {
        // The constant pi is positive (`tests::pi_is_positive`), so the quarter-turn count
        // always reads.
        let half_pi = Dy::pi() * crate::dyadic!(1, 2);
        let quarters = self.div(&half_pi).map_or(Dy::ZERO, |q| q);
        // Past 2^63 quarter turns (|x| above about 1.4 x 10^19 rad, where one
        // unit of the 62-bit mantissa is wider than a turn, so the angle no
        // longer reads) the count is held at the bound of `i64`, as the
        // reduction has always read it.
        let q = match (quarters
            + if self.m >= 0 {
                crate::dyadic!(1, 2)
            } else {
                crate::dyadic!(-1, 2)
            })
        .trunc_i64()
        {
            Some(q) => q,
            None if self.m >= 0 => i64::MAX,
            None => i64::MIN,
        };
        let r = *self - Dy::int(q) * half_pi;
        let Some(f) = to_fixed(&r) else {
            return (Dy::ZERO, Dy::ONE);
        };
        let f2 = (f * f) >> MANT;
        // `sin r = r - r^3/3! + ...` and `cos r = 1 - r^2/2! + ...`.
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

    /// `atan x`: for |x| > 1 by `+-pi/2 - atan(1/x)`; otherwise halved (at
    /// most twice) by `atan x = 2 atan(x / (1 + sqrt(1 + x^2)))` until
    /// |x| <= 1/5, then the alternating series.
    pub fn atan(&self) -> Dy {
        // The inverse refuses only zero, whose arctangent is zero.
        let Some(inverse) = Dy::ONE.div(self) else {
            return Dy::ZERO;
        };
        if self.abs() > Dy::ONE {
            let h = Dy::pi() * crate::dyadic!(1, 2);
            let inner = inverse.atan();
            return if self.m > 0 { h - inner } else { -h - inner };
        }
        let mut x = *self;
        let mut scale = 1i64;
        while x.abs() > crate::dyadic!(1, 5) {
            // The radicand `1 + x^2` is at least one, so its root reads, and
            // the half-angle divisor `1 + sqrt(1 + x^2)` is at least one.
            let Some(half) = (Dy::ONE + x * x).sqrt().and_then(|r| x.div(&(Dy::ONE + r))) else {
                break;
            };
            x = half;
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
        // The radicand is held at or above zero, so its root reads
        // (`tests::sqrt_reads_every_value_held_at_or_above_zero`); the ratio
        // refuses only x = 0, where the angle is a quarter turn.
        let Some(t) = (Dy::ONE - x * x)
            .max(Dy::ZERO)
            .sqrt()
            .and_then(|s| s.div(&x))
        else {
            return Dy::pi() * crate::dyadic!(1, 2);
        };
        let a = t.atan();
        if x.m > 0 {
            a
        } else {
            a + Dy::pi()
        }
    }

    /// A uniform draw in [0, 1) from 64 random bits: the top 53 bits, scaled
    /// by 2^-53.
    pub fn unit(bits: u64) -> Dy {
        norm(i128::from(bits >> 11), -53)
    }
}

impl Dy {
    /// The exact representation `(mantissa, exponent)`, for codecs.
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
            x = x.over(crate::nonzero!(10));
            e10 += 1;
        }
        while x < Dy::ONE {
            x = x * ten;
            e10 -= 1;
        }
        // The scaled value lies in [10^11, 10^12], which `i64` holds.
        let digits = (x * Dy::sci(1, 11) + crate::dyadic!(1, 2))
            .trunc_i64()
            .map_or(999_999_999_999, |d| d.min(999_999_999_999));
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

/// `ln 2` in 2^-62 fixed point: the floor of `ln 2 x 2^62`, read once from
/// the certified enclosure `certified::ln2`.
fn ln2_fixed() -> i128 {
    static L: OnceLock<i128> = OnceLock::new();
    // The constant ln 2 is below one, so its fixed point fits
    // (`tests::ln2_fits_the_fixed_point`).
    *L.get_or_init(|| to_fixed(&Dy::from_q(&crate::certified::ln2(80).lo)).map_or(0, |f| f))
}

/// `atanh(z)` for |z| <= 1/3 in 2^-62 fixed point: `sum z^(2k+1) / (2k+1)`.
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

/// `ln(f)` with `f = m / 2^62` for m in [2^61, 2^62), in 2^-62 fixed point:
/// `2 atanh(z)` with `z = (f - 1) / (f + 1)` in [-1/3, 0].
fn ln_fixed(m: i128) -> i128 {
    let z = ((m - TOP) << MANT) / (m + TOP);
    2 * atanh_fixed(z)
}

/// `e^r` for |r| <= ln2/2 in 2^-62 fixed point: the Taylor series.
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

    /// The constant pi reads positive, so the quarter turns of `sin_cos` always read.
    #[test]
    fn pi_is_positive() {
        assert!(Dy::pi().signum() > 0);
    }

    /// The constant ln 2 is below one, so its 2^-62 fixed point fits and
    /// `ln2_fixed` never takes its zero branch.
    #[test]
    fn ln2_fits_the_fixed_point() {
        let l = Dy::from_q(&crate::certified::ln2(80).lo);
        assert!(l > Dy::ZERO && l < Dy::ONE);
        assert!(to_fixed(&l).is_some());
        assert!(ln2_fixed() > 0);
    }

    /// The root of any value held at or above zero reads, so `acos` and
    /// `atan` never take a refusal of `sqrt`.
    #[test]
    fn sqrt_reads_every_value_held_at_or_above_zero() {
        for x in [
            -Dy::sci(1, 30),
            -Dy::ONE,
            Dy::ZERO,
            crate::dyadic!(1, 7),
            Dy::sci(3, 20),
        ] {
            assert!(x.max(Dy::ZERO).sqrt().is_some());
            assert!((Dy::ONE + x * x).sqrt().is_some());
        }
    }

    /// `exp` saturates past 2^63 halvings in either direction, and below
    /// that keeps growing with its argument.
    #[test]
    fn exp_saturates_past_the_halvings_i64_holds() {
        let top = Dy::sci(7, 18).exp();
        assert_eq!(top, Dy::sci(1, 19).exp());
        assert!(top > Dy::sci(1, 300));
        assert_eq!(Dy::sci(-7, 18).exp(), Dy::ZERO);
        assert_eq!(Dy::sci(-1, 30).exp(), Dy::ZERO);
        assert!(Dy::int(40).exp() < Dy::int(41).exp());
    }

    proptest::proptest! {
        /// Every rational, however wide its parts, is held by `from_q` to 62
        /// bits: nonzero when it is nonzero, with the same sign, and within
        /// 2^-60 of it relatively, so the quotient always fits `i128`.
        #[test]
        fn from_q_holds_every_rational_to_62_bits(
            n in proptest::num::i64::ANY,
            d in 1i64..i64::MAX,
            k in 0u32..200,
            up in proptest::bool::ANY,
        ) {
            let s = &Q::new(n, d).unwrap()
                * &if up {
                    Q::from_big(BigRational::from_integer(BigInt::one() << k))
                } else {
                    Q::from_big(BigRational::new(BigInt::one(), BigInt::one() << k))
                };
            let x = Dy::from_q(&s);
            proptest::prop_assert_eq!(x.signum(), s.signum());
            if !s.is_zero() {
                let err = (&x.to_q() - &s).abs();
                let ulp = Q::from_big(BigRational::new(BigInt::one(), BigInt::one() << 60));
                let tol = &s.abs() * &ulp;
                proptest::prop_assert!(err <= tol);
            }
        }
    }

    proptest::proptest! {
        /// Dividing by a nonzero integer agrees with `div` by its reading:
        /// `Dy::int` of a nonzero integer is nonzero, so `over` never takes
        /// its zero branch.
        #[test]
        fn over_agrees_with_div_for_every_nonzero_divisor(
            n in -1_000_000_000i64..1_000_000_000,
            d in 1i64..1_000_000_000,
            neg in proptest::bool::ANY,
        ) {
            let d = if neg { -d } else { d };
            let nz = std::num::NonZeroI64::new(d).unwrap();
            proptest::prop_assert!(!Dy::int(d).is_zero());
            proptest::prop_assert_eq!(Dy::int(n).over(nz), Dy::int(n).div(&Dy::int(d)).unwrap());
        }
    }

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
        // A product beyond `i64::MAX` is held: the exponent carries the scale.
        let big =
            Dy::from_q(&Q::from_i128(5972, 1).unwrap()).mul(&Dy::int(1_000_000_000_000_000_000));
        assert!(big > Dy::int(i64::MAX));
    }

    #[test]
    fn operators_constants_and_powers() {
        assert!(near(
            &Dy::pi(),
            "3.14159265358979323846",
            "0.000000000000000001"
        ));
        assert!(near(
            &ln2(),
            "0.69314718055994530942",
            "0.000000000000000001"
        ));
        // The constants are decided by their enclosures: both bounds round
        // down to the same 62 bits.
        let (p, l) = (crate::certified::pi(80), crate::certified::ln2(80));
        assert_eq!(Dy::from_q(&p.lo), Dy::from_q(&p.hi));
        assert_eq!(Dy::from_q(&l.lo), Dy::from_q(&l.hi));
        assert_eq!(Dy::pi(), Dy::from_q(&p.hi));
        assert_eq!(ln2(), Dy::from_q(&l.hi));
        assert_eq!(Dy::ONE.to_q(), Q::ONE);
        assert_eq!(
            Dy::sci(5972, 21).to_q(),
            Q::from_i128(5_972_000_000_000_000_000_000_000, 1).unwrap()
        );
        assert_eq!(Dy::int(3).powi(4).to_q(), Q::int(81));
        assert_eq!(
            Dy::ONE.div(&Dy::int(2).powi(3)).unwrap().to_q(),
            Q::new(1, 8).unwrap()
        );
        assert!(near(
            &Dy::int(27).pow(&crate::dyadic!(1, 3)).unwrap(),
            "3",
            "0.00000000000001"
        ));
        assert_eq!(Dy::int(1).div(&Dy::ZERO), None);
        assert_eq!(crate::dyadic!(7, 2).trunc_i64(), Some(3));
        assert_eq!(crate::dyadic!(-7, 2).trunc_i64(), Some(-3));
        // A whole part past `i64` is refused, not saturated.
        assert_eq!(Dy::sci(1, 19).trunc_i64(), None);
        assert_eq!(Dy::sci(-1, 19).trunc_i64(), None);
        assert_eq!(Dy::sci(9, 18).trunc_i64(), Some(9_000_000_000_000_000_000));
        // An angle just past -pi/4, where the quarter-turn reduction changes:
        // the sine is negative and the cosine positive.
        let (s, c) = crate::dyadic!(-63, 80).sin_cos();
        assert!(s < Dy::ZERO && c > Dy::ZERO);
        assert_eq!(format!("{}", Dy::int(-1234)), "-1.23400000000e3");
        assert_eq!(crate::dyadic!(5, 3).grid(4).to_q(), Q::new(26, 16).unwrap());
    }

    #[test]
    fn trigonometry_and_gamma() {
        let tol = "0.00000000000001";
        assert!(near(
            &crate::dyadic!(1, 2).sin(),
            "0.47942553860420300027",
            tol
        ));
        assert!(near(
            &crate::dyadic!(1, 2).cos(),
            "0.87758256189037271612",
            tol
        ));
        assert!(near(&Dy::int(3).sin(), "0.14112000805986722210", tol));
        assert!(near(&Dy::int(-5).cos(), "0.28366218546322626447", tol));
        assert!(near(&Dy::int(2).atan(), "1.10714871779409050302", tol));
        assert!(near(
            &crate::dyadic!(-1, 3).atan(),
            "-0.32175055439664219340",
            tol
        ));
        assert!(near(
            &crate::dyadic!(1, 4).acos(),
            "1.31811607165281796574",
            tol
        ));
        assert!(near(
            &crate::dyadic!(-3, 4).acos(),
            "2.41885840577637762338",
            tol
        ));
        assert_eq!(Dy::unit(0), Dy::ZERO);
        let x = crate::dyadic!(-22, 7);
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
