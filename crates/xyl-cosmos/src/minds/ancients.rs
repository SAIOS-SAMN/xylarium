//! The first minds and the ancients: core-born minds and their window.

use super::*;

/// The core's assembly time (Gyr): the star-formation time at the centre
/// (`galaxy::birth` at radius zero, converted from rings), or the midpoint of
/// the Big Bang's first-stars time where that is undefined. It is the window within which
/// a galaxy's core-born minds count as ancients.
pub fn ancient_window(gal: &Galaxy) -> Dy {
    let rings =
        crate::galaxy::birth(&xyl_core::Q::ZERO).unwrap_or_else(|| gal.bang.first_stars().mid());
    Dy::from_q(&crate::circuit::gyr_of_rings(&rings))
}

/// A star's distance from the core at `t` Gyr, the sum of its reach and its
/// height off the plane. The bulge is a ball, so a disc star high above the
/// core is outside it however near the axis of its orbit's cylinder.
///
/// The distance is read in the tier of the star's galaxy: the bearing from
/// that galaxy's core, in the home galaxy's equivalent kiloparsecs (distance
/// over the galaxy's scale), so the bulge is the same test in every galaxy.
pub fn from_centre_kpc(star: &crate::galaxy::Star, t: Dy) -> Dy {
    // The distance from the core is the reach plus the height off the plane
    // (a path sum, not the hypotenuse).
    let (reach, _, climb) = star.bearing(crate::circuit::rings_of_gyr_dy(t));
    let way = &reach + &climb.abs();
    Dy::from_q(
        &way.checked_div(&crate::chain::kpc())
            .unwrap_or(xyl_core::Q::ZERO),
    ) / star.scale.max(Dy::sci(1, -9))
}

/// Whether a mind was born in the galaxy's core: its star lay within
/// `galaxy::core_reach` of the core when the mind woke (see
/// `from_centre_kpc`; the reach is read in kpc through the boundary
/// `galaxy::kpc_of_rings`).
pub fn in_core(m: &Mind) -> bool {
    m.r_kpc <= Dy::from_q(&crate::galaxy::kpc_of_rings(&crate::galaxy::core_reach()))
}

/// The ancients by birth: the minds born in the core that wake within one
/// core assembly time (`ancient_window`) of that core's first core-born
/// mind, earliest first. Their number follows from the seed. They hold the
/// throats at the core and the uplink to the network.
///
/// The test is per galaxy (T1 ancients at each galactic core): a mind is
/// compared with the first core-born mind of the galactic core its star
/// belongs to.
pub fn ancients<'s>(s: &'s Schedule, gal: &Galaxy) -> Vec<&'s Mind> {
    let window = ancient_window(gal);
    s.minds
        .iter()
        .filter(|m| {
            in_core(m)
                && first_core_dawn(s, crate::galaxy::core_of(m.star))
                    .is_some_and(|d| m.epoch - d <= window)
        })
        .collect()
}

/// The epoch of the first core-born mind in galactic core `core`; `None`
/// where that core has no core-born mind.
pub fn first_core_dawn(s: &Schedule, core: u32) -> Option<Dy> {
    s.minds
        .iter()
        .find(|m| in_core(m) && crate::galaxy::core_of(m.star) == core)
        .map(|m| m.epoch)
}
