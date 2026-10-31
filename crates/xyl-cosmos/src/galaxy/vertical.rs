//! A disc star's vertical orbit through the plane: speed, amplitude, period
//! and exposure to the crowded layer, in closed form.

use super::density;
use xyl_core::dyadic::Dy;

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
    /// Period of the oscillation (Myr).
    pub period_myr: Dy,
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

/// The plane's oscillation frequency at `r` (rad/Gyr): the frequency at the
/// reference ring (75.2 rad/Gyr, a declared ruler) times the crowding share
/// 2 d / (1 + d), with d the ring's crowding ([`density`]), in place of
/// sqrt(4 pi G rho).
pub fn vertical_frequency(r_kpc: Dy) -> Dy {
    let density = crowding(r_kpc);
    Dy::ratio(752, 10) * Dy::int(2) * density / (Dy::ONE + density)
}

/// A star's vertical orbit at `r` (kpc) after `age` Gyr: the uniform draw `u`
/// in [0, 1) sets the speed through an exponential distribution, and `phase`
/// is the position in the oscillation (turns).
pub fn vertical(r_kpc: Dy, age: Dy, u: Dy, phase: Dy) -> Vertical {
    let density = crowding(r_kpc);
    // `sigma = 20 km/s x 2t / (1 + t) x 2d / (1 + d)`, t = age / 10 Gyr: shares
    // of age and crowding in place of power laws; vz = -sigma ln(1 - u), an
    // exponential draw in place of a Rayleigh root.
    let t = age.max(Dy::ZERO) / Dy::int(10);
    let sigma = Dy::int(20)
        * (Dy::int(2) * t / (Dy::ONE + t))
        * (Dy::int(2) * density / (Dy::ONE + density));
    let vz = sigma * -(Dy::ONE - u).max(Dy::sci(1, -12)).log();
    let nu = vertical_frequency(r_kpc);
    // The amplitude is A = vz / (0.9778 nu) in kpc; 1 kpc/Gyr = 0.9778 km/s.
    let amp = vz / (nu * Dy::ratio(9778, 10_000)).max(Dy::sci(1, -9));
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
        period_myr: Dy::int(2) * Dy::PI / nu.max(Dy::sci(1, -9)) * Dy::int(1000),
        exposure,
        phase,
    }
}
