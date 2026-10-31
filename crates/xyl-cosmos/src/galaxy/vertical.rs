//! A disc star's vertical orbit through the plane: speed, amplitude, period
//! and exposure to the crowded layer, in closed form, derived from the
//! turning law, the crowding and the ring structure. Reaches are galaxy
//! rings, ages and periods rings, the frequency radians per ring and the
//! speed galaxy rings per ring.
//!
//! - Frequency ([`vertical_frequency`]): `nu(r) = omega(r) 2d / (1 + d)`,
//!   with `omega` the turning law about the mass inside `r`
//!   (`turning::turning_dy`, `inside_mass`) and `d` the ring's crowding
//!   (`density`): the plane's restoring pull is the turning rate held by the
//!   crowding share.
//! - Layer ([`layer`]): the crowded layer's half-thickness
//!   `h(r) = r m / (M(<r) + m)`, the deficit share of the ring's mass `m`
//!   (`ring_mass`) against the mass inside it (`inside_mass`), taken of the
//!   reach.
//! - Heating share: `s = t / (1 + t)`, `t` the star's age over the ring's
//!   cooling time, the rings until the ring's gas first lights (`birth`).
//! - Speed: `sigma = h nu s 2d / (1 + d)`; `vz = -sigma ln(1 - u)`, an
//!   exponential draw; amplitude `A = vz / nu`; period `2 pi / nu`;
//!   exposure the share of the swing spent inside `|z| < h`.

use super::{birth, density, inside_mass_dy, ring_mass};
use xyl_core::dyadic::Dy;
use xyl_core::units::Tier;

/// A star's vertical orbit through the disc's plane: the speed through the
/// plane, the amplitude from the plane's restoring pull
/// ([`vertical_frequency`]), the period, and the exposure: the share of the
/// oscillation spent inside the crowded layer ([`layer`]), in closed form.
/// The crowded layer is where comet clouds are perturbed and supernovae
/// occur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vertical {
    /// Amplitude of the swing through the plane (galaxy rings).
    pub amp: Dy,
    /// Speed through the plane (galaxy rings per ring).
    pub vz: Dy,
    /// Period of the oscillation (rings); zero where the plane has no pull.
    pub period: Dy,
    /// Share of the oscillation spent inside the crowded layer, in [0, 1].
    pub exposure: Dy,
    /// Position in the oscillation (turns).
    pub phase: Dy,
}

/// The crowding of the ring containing `r` (galaxy rings), as a dyadic: the
/// ring's holding per galaxy ring over that of the reference ring
/// ([`density`]), 1 at the reference ring and never negative.
fn crowding(r: Dy) -> Dy {
    Dy::from_q(&density(&r.to_q()).mid()).max(Dy::ZERO)
}

/// The crowding share `2d / (1 + d)` of crowding `d`, in [0, 2).
fn crowding_share(d: Dy) -> Dy {
    Dy::int(2) * d / (Dy::ONE + d)
}

/// The crowded layer's half-thickness at reach `r` (galaxy rings):
/// `h(r) = r m / (M(<r) + m)`, the deficit share of the ring's mass `m`
/// (`ring_mass`, core masses) against the mass inside it (`inside_mass`,
/// core masses), taken of the reach. Zero at a non-positive reach.
pub fn layer(r: Dy) -> Dy {
    if r.signum() <= 0 {
        return Dy::ZERO;
    }
    let m = Dy::from_q(&ring_mass(&r.to_q()));
    let inside = inside_mass_dy(r);
    if (inside + m).signum() <= 0 {
        return Dy::ZERO;
    }
    r * m / (inside + m)
}

/// The plane's oscillation frequency at `r` (radians per ring):
/// `nu = omega 2d / (1 + d)`, the turning law's rate about the mass inside
/// `r` (`turning::turning_dy`, galaxy tier) times the crowding share, with
/// `d` the ring's crowding ([`density`]). Zero before a founding.
pub fn vertical_frequency(r: Dy) -> Dy {
    let omega = crate::turning::turning_dy(inside_mass_dy(r), r, Tier::Galaxy);
    omega * crowding_share(crowding(r))
}

/// The heating share `s = t / (1 + t)` of a star born at `born` (rings) at
/// reach `r` after `age` rings: `t` is the star's age over the ring's
/// cooling time, the rings from the Big Bang until the ring's gas first
/// lights (`birth`, at least one ring; the star's birth where the ring never
/// lights). The share is zero at age zero, one half when the star has lived
/// one cooling time, and approaches one for older stars: the deficit share of
/// the star's age against the ring's cooling time.
fn heating_share(r: Dy, born: Dy, age: Dy) -> Dy {
    let age = age.max(Dy::ZERO);
    let cooling = birth(&r.to_q())
        .map(|b| Dy::from_q(&b))
        .unwrap_or(born)
        .max(Dy::ONE);
    age / (age + cooling)
}

/// A star's vertical orbit at `r` (galaxy rings), born at `born` and read
/// after `age` rings: the uniform draw `u` in [0, 1) sets the speed through
/// an exponential distribution, and `phase` is the position in the
/// oscillation (turns). Amplitude and period are zero where the plane has no
/// pull (before a founding).
pub fn vertical(r: Dy, born: Dy, age: Dy, u: Dy, phase: Dy) -> Vertical {
    let share = crowding_share(crowding(r));
    let nu = vertical_frequency(r);
    let h = layer(r);
    // The speed scale is `sigma = h nu s 2d / (1 + d)` (galaxy rings per
    // ring) and the draw `vz = -sigma ln(1 - u)`, an exponential in place
    // of a Rayleigh root.
    let sigma = h * nu * heating_share(r, born, age) * share;
    let vz = sigma * -(Dy::ONE - u).max(Dy::sci(1, -12)).log();
    let (amp, period) = if nu.signum() > 0 {
        (vz / nu, Dy::int(2) * Dy::PI / nu)
    } else {
        (Dy::ZERO, Dy::ZERO)
    };
    // Exposure: the share of the oscillation inside the layer |z| < h,
    // linear in angle; 1 when the amplitude is within the layer.
    let exposure = if amp <= h {
        Dy::ONE
    } else {
        // The share is `atan(h / A)` over the quarter turn, avoiding the root
        // of 1 - x^2 in asin(h / A).
        (h / amp).atan() * Dy::int(2) / Dy::PI
    };
    Vertical {
        amp,
        vz,
        period,
        exposure,
        phase,
    }
}
