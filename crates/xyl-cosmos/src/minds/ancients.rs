//! The first minds and the ancients: core-born minds and their window.

use super::*;

/// The core's assembly time (Gyr): its ring's star-formation timescale at the
/// centre, in this universe's assembly - the window within which a galaxy's
/// first minds count as its first.
pub fn ancient_window(gal: &Galaxy) -> Dy {
    Dy::from_q(
        &crate::galaxy::birth_gyr(&xyl_core::Q::ZERO)
            .unwrap_or_else(|| gal.bang.first_stars_gyr().mid()),
    )
}

/// A star's distance from the core at `t` Gyr, its reach and its climb: the
/// bulge is a ball, so a disc star swinging high over the core is outside it
/// however near its cylinder's axis.
///
/// Read in its own galaxy's tier: its bearing from its own galactic core, in
/// the home galaxy's equivalent kiloparsecs (distance over its galaxy's
/// scale), so the bulge is the same test in every galaxy.
pub fn from_centre_kpc(star: &crate::galaxy::Star, t: Dy) -> Dy {
    // distance out from the core: its reach plus its climb off the plane
    // (a path sum, not the hypotenuse)
    let (reach, _, climb) = star.bearing(t);
    let way = &reach + &climb.abs();
    Dy::from_q(
        &way.checked_div(&crate::chain::kpc())
            .unwrap_or(xyl_core::Q::ZERO),
    ) / star.scale.max(Dy::sci(1, -9))
}

/// Whether a mind was born in the galaxy's core: its star within the bulge's
/// radius of the core when it woke, in three dimensions.
pub fn in_core(m: &Mind) -> bool {
    m.r_kpc <= Dy::from_q(&crate::galaxy::core_kpc())
}

/// The ancients by birth: the
/// minds born in the core that wake within one core assembly time of the
/// core's own first mind, earliest first. Their number follows from the seed.
/// They hold the throats at the core and the uplink to the network.
///
/// Per galaxy (T1 ancients at each galactic core): a mind is ancient against
/// its own galactic core's first core-born mind.
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

/// The first core-born mind's dawn in galactic core `core`.
pub fn first_core_dawn(s: &Schedule, core: u32) -> Option<Dy> {
    s.minds
        .iter()
        .find(|m| in_core(m) && crate::galaxy::core_of(m.star) == core)
        .map(|m| m.epoch)
}
