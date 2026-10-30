// The grain: whole nuclei, the 64-bit grid, turns and a mix's sums

use super::*;
use crate::formation;
use xyl_core::Q;

pub(crate) fn grid(x: &Q) -> Q {
    xyl_core::certified::down(x, 64).max(Q::ZERO)
}

/// Splits an amount of kind z into whole nuclei: returns (whole amount, count).
/// One nucleus of z holds w_z (its weight, 3Z - unpaired) nucleon units, so
/// amounts move in whole multiples of w_z * nucleon_unit(). Every amount then
/// shares one bounded denominator, and fusion conserves 3Z by whole nuclei
/// (n of z become n / 2 of 2z: n 3z = (n / 2) 3 (2z)). The 2^-64 grid is not
/// used for amounts because it would freeze rare kinds in place.
pub(crate) fn nuclei_of(x: &Q, z: u32) -> (Q, Q) {
    let unit = nucleon_unit();
    let one = &formation::weight(z) * &unit;
    // the whole count, exact (counts exceed machine integers: q - frac q)
    let n = x
        .checked_div(&one)
        .map(|q| &q - &q.frac())
        .unwrap_or(Q::ZERO)
        .max(Q::ZERO);
    (&n * &one, n)
}

static FOUNDING_BITS: std::sync::OnceLock<u32> = std::sync::OnceLock::new();

/// Names the founding's count (bits of hydrogen) the grain is cut from; the
/// first founding in the process wins.
pub(crate) fn set_founding_bits(bits: u32) {
    let _ = FOUNDING_BITS.set(bits);
}

pub(crate) fn nucleon_unit() -> Q {
    static U: std::sync::OnceLock<Q> = std::sync::OnceLock::new();
    // the grain scales with the founding's count (8 bits finer than one hydrogen
    // of the founding), so the rarest founded kind still moves in whole nuclei;
    // before the count is named the grain is not kept, so a reading taken
    // ahead of the founding never freezes the coarse grain for the process
    if let Some(u) = U.get() {
        return u.clone();
    }
    let bits = FOUNDING_BITS.get().copied();
    let u = (0..bits.unwrap_or(0) + 8)
        .fold(Q::ONE, |a, _| a.checked_div(&Q::int(2)).unwrap_or(Q::ZERO));
    if bits.is_some() {
        let _ = U.set(u.clone());
    }
    u
}

/// A turn rounded to the coordinate grid (60^-6), so phases carry no growing
/// denominators (a resolution choice, not a law).
pub(crate) fn turn(x: &Q) -> Q {
    let marks = Q::int(xyl_core::coordinate::MARKS);
    (&x.frac() * &marks)
        .floor_i128()
        .and_then(|m| i64::try_from(m).ok())
        .and_then(|m| Q::new(m, xyl_core::coordinate::MARKS))
        .unwrap_or(Q::ZERO)
}

/// A mix's mass by formation::mass_weight: held plus open frames (total +
/// magnetism), every amount at its whole 3 charge.
pub(crate) fn weigh(m: &Mix) -> Q {
    m.iter().fold(Q::ZERO, |acc, (z, a)| &acc + &mass_of(*z, a))
}

/// An amount of kind z (in held frames) as mass: the amount over one nucleus's
/// hold, times its Z + Q nucleons (not its 3Z frames).
pub(crate) fn mass_of(z: u32, a: &Q) -> Q {
    (a * &formation::mass_weight(z))
        .checked_div(&formation::weight(z))
        .unwrap_or(Q::ZERO)
}

pub(crate) fn add(m: &mut Mix, z: u32, x: &Q) {
    if x.signum() > 0 {
        let e = m.entry(z).or_insert(Q::ZERO);
        *e = &*e + x;
    }
}

pub(crate) fn deficit(r: &Q, h: &Q) -> Q {
    h.checked_div(&(r + h)).unwrap_or(Q::ZERO)
}
