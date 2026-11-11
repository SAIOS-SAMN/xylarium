// The grain: amounts cut into whole nuclei, and a mix's mass. Reaches and
// phases are exact; nothing rational is rounded.
//
// Inputs: `compound::electron_hold` (3Z - unpaired states per nucleus) and
// `formation::mass_weight` (Z + Q nucleons).
//
// Outputs (crate-private): `nuclei_of` (whole amount and count of a kind;
// the nucleon unit is 2^-(bits + 8), 8 bits finer than one hydrogen of the
// founding's count, fixed once `set_founding_bits` names it),
// `weigh` and `mass_of` (held count to mass), `add` (ignores
// non-positive amounts) and `deficit` (h / (r + h), zero where undefined).
// Exact throughout.

use super::*;
use crate::formation;
use xyl_core::Q;

/// Splits an amount of kind z into whole nuclei: returns (whole amount, count).
/// One nucleus of z holds w_z (its weight, 3Z - unpaired) nucleon units, so
/// amounts move in whole multiples of w_z * nucleon_unit(). Every amount then
/// shares one bounded denominator, and fusion conserves 3Z by whole nuclei
/// (n of z become n / 2 of 2z: n 3z = (n / 2) 3 (2z)). The 2^-64 grid is not
/// used for amounts because it would freeze rare kinds in place.
pub(crate) fn nuclei_of(x: &Q, z: u32) -> (Q, Q) {
    let unit = nucleon_unit();
    let one = &Q::int(i64::from(crate::compound::electron_hold(z))) * &unit;
    // The whole count is exact (counts exceed machine integers: q - frac q).
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
    // The grain scales with the founding's count (8 bits finer than one
    // hydrogen of the founding), so the rarest founded kind still moves in
    // whole nuclei. Before the count is named the grain is not kept, so a
    // reading taken ahead of the founding does not fix the coarse grain for
    // the process.
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

/// A mix's mass: the sum of `mass_of` over its kinds (Z + Q nucleons per
/// nucleus, formation::mass_weight).
pub(crate) fn weigh(m: &Mix) -> Q {
    m.iter().fold(Q::ZERO, |acc, (z, a)| &acc + &mass_of(*z, a))
}

/// An amount of kind z (in held count) as mass: the amount over one nucleus's
/// hold, times its Z + Q nucleons (not its span 3Z).
pub(crate) fn mass_of(z: u32, a: &Q) -> Q {
    (a * &formation::mass_weight(z))
        .checked_div(&Q::int(i64::from(crate::compound::electron_hold(z))))
        .unwrap_or(Q::ZERO)
}

pub(crate) fn add(m: &mut Mix, z: u32, x: &Q) {
    if x.signum() > 0 {
        let e = m.entry(z).or_insert(Q::ZERO);
        *e = &*e + x;
    }
}
