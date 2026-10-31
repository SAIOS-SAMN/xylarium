//! A disc star's vertical orbit through the plane: speed, amplitude, period
//! and exposure to the crowded layer, in closed form. Ages and periods are
//! in rings, the frequency in radians per ring.

use super::density;
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// A star's vertical orbit through the disc's plane: the speed through the
/// plane from the age-velocity relation at the star's ring (20 km/s scale at
/// 10 Gyr, a declared ruler), the amplitude from the plane's restoring pull
/// ([`vertical_frequency`]; provisional until derived from the turning law),
/// the period, and the exposure: the share of the oscillation spent inside
/// the crowded layer ([`layer_kpc`]), in closed form. The crowded layer is
/// where comet clouds are perturbed and supernovae occur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vertical {
    /// Amplitude of the swing through the plane (kpc).
    pub amp_kpc: Dy,
    /// Speed through the plane (km/s).
    pub vz_kms: Dy,
    /// Period of the oscillation (rings).
    pub period: Dy,
    /// Share of the oscillation spent inside the crowded layer, in [0, 1].
    pub exposure: Dy,
    /// Position in the oscillation (turns).
    pub phase: Dy,
}

/// The crowding of the ring containing `r` (kpc), as a dyadic: the ring's
/// holding per kpc over that of the reference ring ([`density`]), 1 at the
/// reference ring and never negative.
fn crowding(r_kpc: Dy) -> Dy {
    Dy::from_q(&density(&r_kpc.to_q()).mid()).max(Dy::ZERO)
}

/// The crowded layer's half-thickness (kpc): a declared ruler (0.3). The
/// structure read from the founding gives reaches and spans along the
/// plane, not the plane's height, so no derived scale replaces it yet.
pub fn layer_kpc() -> Dy {
    Dy::ratio(3, 10)
}

/// The plane's oscillation frequency at the reference ring (radians per
/// ring): 75.2 rad/Gyr, a declared ruler, converted once to the ring
/// (`turning::rate_per_ring`). Zero before a founding.
fn reference_frequency() -> Dy {
    Dy::from_q(&crate::turning::rate_per_ring(&Q::ratio(752, 10)))
}

/// The age-velocity relation's age scale (rings): 10 Gyr, a declared ruler,
/// converted once to the ring (`turning::span_in_rings`).
fn heating_age() -> Dy {
    Dy::from_q(&crate::turning::span_in_rings(&Q::int(10)))
}

/// The ring in the age-velocity relation's units: kpc per ring over km/s.
/// The declared 1 kpc/Gyr = 0.9778 km/s, read per ring
/// (`turning::span_in_rings`). Zero before a founding.
fn kpc_per_ring_per_kms() -> Dy {
    let per_ring = Dy::from_q(&crate::turning::span_in_rings(&Q::ONE));
    if per_ring.signum() <= 0 {
        return Dy::ZERO;
    }
    Dy::ONE / (Dy::ratio(9778, 10_000) * per_ring)
}

/// The plane's oscillation frequency at `r` (radians per ring): the
/// frequency at the reference ring (75.2 rad/Gyr, per ring) times the
/// crowding share 2 d / (1 + d), with d the ring's crowding ([`density`]),
/// in place of sqrt(4 pi G rho).
pub fn vertical_frequency(r_kpc: Dy) -> Dy {
    let density = crowding(r_kpc);
    reference_frequency() * Dy::int(2) * density / (Dy::ONE + density)
}

/// A star's vertical orbit at `r` (kpc) after `age` rings: the uniform draw `u`
/// in [0, 1) sets the speed through an exponential distribution, and `phase`
/// is the position in the oscillation (turns).
pub fn vertical(r_kpc: Dy, age: Dy, u: Dy, phase: Dy) -> Vertical {
    let density = crowding(r_kpc);
    // `sigma = 20 km/s x 2t / (1 + t) x 2d / (1 + d)`, t = age over the
    // heating age (10 Gyr in rings): shares of age and crowding in place of
    // power laws; vz = -sigma ln(1 - u), an exponential draw in place of a
    // Rayleigh root.
    let t = age.max(Dy::ZERO) / heating_age().max(Dy::sci(1, -30));
    let sigma = Dy::int(20)
        * (Dy::int(2) * t / (Dy::ONE + t))
        * (Dy::int(2) * density / (Dy::ONE + density));
    let vz = sigma * -(Dy::ONE - u).max(Dy::sci(1, -12)).log();
    let nu = vertical_frequency(r_kpc);
    // The amplitude is A = vz / nu in kpc, vz read in kpc per ring.
    // The frequency is at least 1e-9 rad/Gyr, read per ring.
    let floor = Dy::from_q(&crate::turning::rate_per_ring(&Q::ratio(1, 1_000_000_000)))
        .max(Dy::sci(1, -60));
    let amp = vz * kpc_per_ring_per_kms() / nu.max(floor);
    // Exposure: the share of the oscillation inside the layer |z| < h,
    // linear in angle; 1 when the amplitude is within the layer.
    let h = layer_kpc();
    let exposure = if amp <= h {
        Dy::ONE
    } else {
        // The share is `atan(h / A)` over the quarter turn, avoiding the root
        // of 1 - x^2 in asin(h / A).
        (h / amp).atan() * Dy::int(2) / Dy::PI
    };
    Vertical {
        amp_kpc: amp,
        vz_kms: vz,
        period: Dy::int(2) * Dy::PI / nu.max(floor),
        exposure,
        phase,
    }
}
