//! Exact rationals, held in two machine words while numerator and denominator
//! fit `i64` and as a heap `BigRational` otherwise.
//!
//! Inputs: integers (`int`, `new`, `from_i128`) or big ratios (`from_big`,
//! `from_reduced`); no other module.
//!
//! Outputs: [`Q`], a ratio in lowest terms with a positive denominator, with
//! `+ - *` and negation on references, ordering, `floor`, `frac`, `recip`,
//! [`Q::checked_div`], `min`/`max`, and [`Q::put_bytes`], its canonical byte
//! form for fingerprints and codecs.
//!
//! Representation: while numerator and denominator fit in `i64`, a `Q` is two
//! machine words with no allocation, and every operation is computed in `i128`,
//! where a product or cross-sum of two `i64` ratios cannot overflow; the
//! result is reduced and demoted back to `i64` when it fits. Only a result
//! that does not fit is promoted to a heap `BigRational`, and a big result that
//! fits again is demoted. Dyadic big operands (power-of-two denominators) take
//! a path with no gcd.
//!
//! Invariants: the form is canonical (one value, one representation), so
//! equality and hashing are structural; no operation wraps. There is no
//! division operator: dividing by zero is a `None` the caller handles
//! (`checked_div`, `recip`, `new`), not a panic.
//!
//! Terms: `Q` (docs/glossary.md).

use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

/// An exact rational number in canonical form: lowest terms, positive
/// denominator, held in machine words while it fits `i64` and on the heap
/// otherwise.
///
/// ```
/// use xyl_core::Q;
///
/// let half = Q::new(1, 2).unwrap_or_default();
/// let third = Q::new(2, 6).unwrap_or_default(); // Reduced to 1/3.
/// assert_eq!(third, Q::new(1, 3).unwrap_or_default());
/// assert_eq!(&half + &third, Q::new(5, 6).unwrap_or_default());
/// assert_eq!(&half * &third, Q::new(1, 6).unwrap_or_default());
/// assert_eq!(half.checked_div(&third), Q::new(3, 2));
/// assert_eq!(half.checked_div(&Q::ZERO), None);
/// assert!(third < half);
/// assert_eq!(half.to_string(), "1/2");
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Q {
    /// Lowest terms, `d > 0`.
    Small {
        /// Numerator, carrying the sign.
        n: i64,
        /// Denominator, always positive and coprime to `n`.
        d: i64,
    },
    /// Lowest terms, positive denominator, for a value that does not fit
    /// `Small`.
    Big(Box<BigRational>),
}

impl Default for Q {
    fn default() -> Self {
        Q::ZERO
    }
}

fn gcd_u128(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

impl Q {
    /// The rational zero, `0/1`.
    pub const ZERO: Q = Q::Small { n: 0, d: 1 };
    /// The rational one, `1/1`.
    pub const ONE: Q = Q::Small { n: 1, d: 1 };

    /// The integer `n` as a rational `n/1`.
    pub fn int(n: i64) -> Q {
        Q::Small { n, d: 1 }
    }

    /// `n / d`, or `None` when `d` is zero.
    pub fn new(n: i64, d: i64) -> Option<Q> {
        Q::from_i128(i128::from(n), i128::from(d))
    }

    /// `n / d`, zero when `d` is zero (as [`crate::dyadic::Dy::ratio`]). For
    /// constant ratios written in code; use [`Q::new`] where a zero
    /// denominator must be refused.
    pub fn ratio(n: i64, d: i64) -> Q {
        Q::new(n, d).unwrap_or(Q::ZERO)
    }

    /// `n / d` from wide parts, reduced; held as `Big` only if it does not fit
    /// `Small`. Returns `None` when `d` is zero.
    pub fn from_i128(n: i128, d: i128) -> Option<Q> {
        if d == 0 {
            return None;
        }
        let neg = (n < 0) != (d < 0);
        let (un, ud) = (n.unsigned_abs(), d.unsigned_abs());
        let g = gcd_u128(un, ud).max(1);
        let (un, ud) = (un / g, ud / g);
        // Both now fit u128; signed fit is checked separately for the
        // numerator's sign so i64::MIN stays representable.
        let small = i64::try_from(ud).ok().and_then(|d| {
            let n = if neg {
                i64::try_from(un)
                    .ok()
                    .map(|v| -v)
                    .or((un == 1u128 << 63).then_some(i64::MIN))
            } else {
                i64::try_from(un).ok()
            };
            n.map(|n| (n, d))
        });
        Some(match small {
            Some((n, d)) => Q::Small { n, d },
            None => {
                let bn = if neg {
                    -BigInt::from(un)
                } else {
                    BigInt::from(un)
                };
                Q::Big(Box::new(BigRational::new_raw(bn, BigInt::from(ud))))
            }
        })
    }

    /// The canonical form of an arbitrary big ratio: reduced, then held as
    /// `Small` if it fits.
    pub fn from_big(r: BigRational) -> Q {
        let r = r.reduced();
        if let (Some(n), Some(d)) = (r.numer().to_i64(), r.denom().to_i64()) {
            if d > 0 {
                return Q::Small { n, d };
            }
        }
        Q::Big(Box::new(r))
    }

    /// A big ratio already in lowest terms with a positive denominator,
    /// demoted if it fits (no gcd taken).
    pub fn from_reduced(r: BigRational) -> Q {
        if let (Some(n), Some(d)) = (r.numer().to_i64(), r.denom().to_i64()) {
            if d > 0 {
                return Q::Small { n, d };
            }
        }
        Q::Big(Box::new(r))
    }

    /// The value as an arbitrary-precision `BigRational` (allocates).
    pub fn to_big(&self) -> BigRational {
        match self {
            Q::Small { n, d } => BigRational::new_raw(BigInt::from(*n), BigInt::from(*d)),
            Q::Big(b) => (**b).clone(),
        }
    }

    /// Whether the value is exactly zero.
    pub fn is_zero(&self) -> bool {
        matches!(self, Q::Small { n: 0, .. })
    }

    /// The sign of the value: `-1`, `0` or `1`.
    pub fn signum(&self) -> i8 {
        match self {
            Q::Small { n, .. } => match n.cmp(&0) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            },
            Q::Big(b) => {
                if b.is_negative() {
                    -1
                } else if b.is_zero() {
                    0
                } else {
                    1
                }
            }
        }
    }

    /// The absolute value.
    pub fn abs(&self) -> Q {
        if self.signum() < 0 {
            -self
        } else {
            self.clone()
        }
    }

    /// The numerator in lowest terms, carrying the sign.
    pub fn numer(&self) -> BigInt {
        match self {
            Q::Small { n, .. } => BigInt::from(*n),
            Q::Big(b) => b.numer().clone(),
        }
    }

    /// The denominator in lowest terms, always positive.
    pub fn denom(&self) -> BigInt {
        match self {
            Q::Small { d, .. } => BigInt::from(*d),
            Q::Big(b) => b.denom().clone(),
        }
    }

    /// The largest integer not above the value.
    pub fn floor(&self) -> BigInt {
        match self {
            Q::Small { n, d } => BigInt::from(n.div_floor(d)),
            Q::Big(b) => b.floor().to_integer(),
        }
    }

    /// The fractional part, in [0, 1); for a value in turns, the position
    /// within the turn.
    pub fn frac(&self) -> Q {
        self - &Q::from_reduced(BigRational::from_integer(self.floor()))
    }

    /// The floor as an `i128`, or `None` when it does not fit.
    pub fn floor_i128(&self) -> Option<i128> {
        match self {
            Q::Small { n, d } => Some(i128::from(n.div_floor(d))),
            Q::Big(b) => b.floor().to_integer().to_i128(),
        }
    }

    /// The reciprocal `1 / self`.
    ///
    /// Returns `None` when the value is zero.
    pub fn recip(&self) -> Option<Q> {
        match self {
            Q::Small { n, d } => Q::from_i128(i128::from(*d), i128::from(*n)),
            Q::Big(b) => (!b.is_zero()).then(|| Q::from_big(b.recip())),
        }
    }

    /// The exact quotient `self / rhs`.
    ///
    /// Returns `None` when `rhs` is zero.
    pub fn checked_div(&self, rhs: &Q) -> Option<Q> {
        if rhs.is_zero() {
            return None;
        }
        match (self, rhs) {
            (Q::Small { n: a, d: b }, Q::Small { n: c, d: e }) => Q::from_i128(
                i128::from(*a) * i128::from(*e),
                i128::from(*b) * i128::from(*c),
            ),
            _ => Some(Q::from_big(self.to_big() / rhs.to_big())),
        }
    }

    /// `self * k` for an integer `k`.
    pub fn scale(&self, k: i64) -> Q {
        self * &Q::int(k)
    }

    /// The smaller of the two values (`self` on a tie).
    pub fn min(self, other: Q) -> Q {
        if other < self {
            other
        } else {
            self
        }
    }

    /// The larger of the two values (`self` on a tie).
    pub fn max(self, other: Q) -> Q {
        if other > self {
            other
        } else {
            self
        }
    }

    /// Canonical bytes: a tag, then the parts. Used by fingerprints and
    /// wire codecs; one value has exactly one encoding.
    pub fn put_bytes(&self, out: &mut Vec<u8>) {
        match self {
            Q::Small { n, d } => {
                out.push(0);
                out.extend_from_slice(&n.to_le_bytes());
                out.extend_from_slice(&d.to_le_bytes());
            }
            Q::Big(b) => {
                out.push(1);
                for part in [b.numer(), b.denom()] {
                    let bytes = part.to_signed_bytes_le();
                    out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
                    out.extend_from_slice(&bytes);
                }
            }
        }
    }
}

/// The exponent `e` of a denominator that is `2^e`.
fn two_power(d: &BigInt) -> Option<u64> {
    let e = d.trailing_zeros()?;
    (d == &(BigInt::one() << e)).then_some(e)
}

/// `n / 2^e` in lowest terms: the common factor is two to the count of
/// trailing zero bits of `n`.
fn dyadic(n: BigInt, e: u64) -> Q {
    if n.is_zero() {
        return Q::ZERO;
    }
    let tz = n.trailing_zeros().unwrap_or(0).min(e);
    Q::from_reduced(BigRational::new_raw(n >> tz, BigInt::one() << (e - tz)))
}

/// When both operands are dyadic (power-of-two denominators) and at least one
/// is big: their numerators and exponents, for a gcd-free result. `None`
/// otherwise.
fn dyadic_pair(a: &Q, b: &Q) -> Option<(BigInt, u64, BigInt, u64)> {
    if matches!((a, b), (Q::Small { .. }, Q::Small { .. })) {
        return None;
    }
    Some((
        a.numer(),
        two_power(&a.denom())?,
        b.numer(),
        two_power(&b.denom())?,
    ))
}

macro_rules! small_or_big {
    (
        $a:expr,
        $b:expr,
        |$an:ident, $ad:ident, $bn:ident, $bd:ident| $small:expr,
        |$x:ident, $y:ident| $big:expr
    ) => {
        match ($a, $b) {
            (Q::Small { n: $an, d: $ad }, Q::Small { n: $bn, d: $bd }) => {
                let ($an, $ad, $bn, $bd) = (
                    i128::from(*$an),
                    i128::from(*$ad),
                    i128::from(*$bn),
                    i128::from(*$bd),
                );
                $small
            }
            (l, r) => {
                let ($x, $y) = (l.to_big(), r.to_big());
                // The `num` crate's operations return lowest terms, so no
                // second gcd is taken.
                Q::from_reduced($big)
            }
        }
    };
}

impl Add for &Q {
    type Output = Q;
    fn add(self, rhs: &Q) -> Q {
        if let Some((a, ea, b, eb)) = dyadic_pair(self, rhs) {
            let e = ea.max(eb);
            return dyadic((a << (e - ea)) + (b << (e - eb)), e);
        }
        small_or_big!(
            self,
            rhs,
            |a, b, c, d| Q::from_i128(a * d + c * b, b * d).unwrap_or(Q::ZERO),
            |x, y| x + y
        )
    }
}

impl Sub for &Q {
    type Output = Q;
    fn sub(self, rhs: &Q) -> Q {
        if let Some((a, ea, b, eb)) = dyadic_pair(self, rhs) {
            let e = ea.max(eb);
            return dyadic((a << (e - ea)) - (b << (e - eb)), e);
        }
        small_or_big!(
            self,
            rhs,
            |a, b, c, d| Q::from_i128(a * d - c * b, b * d).unwrap_or(Q::ZERO),
            |x, y| x - y
        )
    }
}

impl Mul for &Q {
    type Output = Q;
    fn mul(self, rhs: &Q) -> Q {
        if let Some((a, ea, b, eb)) = dyadic_pair(self, rhs) {
            return dyadic(a * b, ea + eb);
        }
        small_or_big!(
            self,
            rhs,
            |a, b, c, d| Q::from_i128(a * c, b * d).unwrap_or(Q::ZERO),
            |x, y| x * y
        )
    }
}

impl Neg for &Q {
    type Output = Q;
    fn neg(self) -> Q {
        match self {
            Q::Small { n, d } => match n.checked_neg() {
                Some(n) => Q::Small { n, d: *d },
                None => Q::from_big(-self.to_big()),
            },
            Q::Big(b) => Q::from_big(-(**b).clone()),
        }
    }
}

macro_rules! by_value {
    ($tr:ident, $f:ident) => {
        impl $tr for Q {
            type Output = Q;
            fn $f(self, rhs: Q) -> Q {
                (&self).$f(&rhs)
            }
        }
        impl $tr<&Q> for Q {
            type Output = Q;
            fn $f(self, rhs: &Q) -> Q {
                (&self).$f(rhs)
            }
        }
    };
}
by_value!(Add, add);
by_value!(Sub, sub);
by_value!(Mul, mul);

impl Neg for Q {
    type Output = Q;
    fn neg(self) -> Q {
        -&self
    }
}

impl std::ops::AddAssign<&Q> for Q {
    fn add_assign(&mut self, rhs: &Q) {
        *self = &*self + rhs;
    }
}

impl std::ops::SubAssign<&Q> for Q {
    fn sub_assign(&mut self, rhs: &Q) {
        *self = &*self - rhs;
    }
}

impl Ord for Q {
    fn cmp(&self, other: &Q) -> Ordering {
        match (self, other) {
            (Q::Small { n: a, d: b }, Q::Small { n: c, d: e }) => {
                (i128::from(*a) * i128::from(*e)).cmp(&(i128::from(*c) * i128::from(*b)))
            }
            _ => {
                if let Some((a, ea, b, eb)) = dyadic_pair(self, other) {
                    let e = ea.max(eb);
                    return (a << (e - ea)).cmp(&(b << (e - eb)));
                }
                self.to_big().cmp(&other.to_big())
            }
        }
    }
}

impl PartialOrd for Q {
    fn partial_cmp(&self, other: &Q) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<i64> for Q {
    fn from(n: i64) -> Q {
        Q::int(n)
    }
}

impl fmt::Display for Q {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Q::Small { n, d: 1 } => write!(f, "{n}"),
            Q::Small { n, d } => write!(f, "{n}/{d}"),
            Q::Big(b) if b.denom().is_one() => write!(f, "{}", b.numer()),
            Q::Big(b) => write!(f, "{}/{}", b.numer(), b.denom()),
        }
    }
}

impl fmt::Debug for Q {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn big(n: i64, d: i64) -> BigRational {
        BigRational::new(BigInt::from(n), BigInt::from(d))
    }

    #[test]
    fn reduces_and_keeps_sign_on_the_numerator() {
        assert_eq!(Q::new(6, -4), Q::new(-3, 2));
        assert_eq!(Q::new(0, -5), Some(Q::ZERO));
        assert_eq!(Q::new(1, 0), None);
    }

    fn is_small(q: &Q) -> bool {
        matches!(q, Q::Small { .. })
    }

    #[test]
    fn overflow_promotes_instead_of_wrapping_and_demotes_back() {
        let max = Q::int(i64::MAX);
        let sum = &max + &Q::ONE;
        assert!(!is_small(&sum));
        assert_eq!(sum.to_big(), big(i64::MAX, 1) + big(1, 1));
        let back = &sum - &Q::ONE;
        assert!(is_small(&back), "a big value that fits again is demoted");
        assert_eq!(back, max);
    }

    #[test]
    fn i64_min_round_trips() {
        let m = Q::int(i64::MIN);
        assert!(is_small(&m));
        let n = -&m;
        assert!(!is_small(&n));
        assert_eq!(-&n, m);
    }

    #[test]
    fn division_by_zero_refuses() {
        assert_eq!(Q::ONE.checked_div(&Q::ZERO), None);
        assert_eq!(Q::ZERO.recip(), None);
    }

    #[test]
    fn floor_rounds_toward_negative_infinity() {
        assert_eq!(Q::new(-7, 2).and_then(|q| q.floor_i128()), Some(-4));
        assert_eq!(Q::new(7, 2).and_then(|q| q.floor_i128()), Some(3));
    }

    proptest! {
        #[test]
        fn agrees_with_bigrational(a in any::<i64>(), b in 1i64.., c in any::<i64>(), d in 1i64..) {
            let (x, y) = (Q::new(a, b).unwrap(), Q::new(c, d).unwrap());
            let (bx, by) = (big(a, b), big(c, d));
            prop_assert_eq!((&x + &y).to_big(), &bx + &by);
            prop_assert_eq!((&x - &y).to_big(), &bx - &by);
            prop_assert_eq!((&x * &y).to_big(), &bx * &by);
            prop_assert_eq!(x.cmp(&y), bx.cmp(&by));
            if c != 0 {
                prop_assert_eq!(x.checked_div(&y).unwrap().to_big(), &bx / &by);
            }
            // The form is canonical: equal values have equal forms.
            prop_assert_eq!(Q::from_big((&x + &y).to_big()), &x + &y);
        }
    }
}
