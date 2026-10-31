//! A disc star's vertical orbit through the plane: speed, amplitude, period
//! and its exposure to the crowded layer, in closed form.

use super::density;
use xyl_core::dyadic::Dy;

/// A star's vertical orbit through the disc's plane: its speed through the
/// plane from the age-velocity relation at its ring (20 km/s scale at
/// 10 Gyr, a declared ruler), its amplitude from the plane's restoring pull
/// ([`vertical_frequency`]; provisional until derived from the turning law),
/// its period, and its exposure: the share of its oscillation spent inside
/// the crowded layer ([`layer_kpc`]), in closed form - where comets are
/// shaken loose and supernovae occur.
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

/// The crowding of the ring `r` (kpc) sits in, as a dyadic: the ring's
/// holding per kpc over the reference ring's ([`density`]), 1 there.
fn crowding(r_kpc: Dy) -> Dy {
    Dy::from_q(&density(&r_kpc.to_q()).mid()).max(Dy::ZERO)
}

/// The crowded layer's half-thickness (kpc): a declared ruler (0.3). The
/// structure read from the founding gives reaches and spans along the
/// plane, not the plane's height, so no derived scale replaces it yet.
pub fn layer_kpc() -> Dy {
    Dy::ratio(3, 10)
}

/// The plane's oscillation frequency at `r` (rad/Gyr): the pull at the
/// reference ring (75.2 rad/Gyr, a declared ruler) times the crowding share
/// there, 2 d / (1 + d) with d the ring's crowding ([`density`]), in place
/// of sqrt(4 pi G rho).
pub fn vertical_frequency(r_kpc: Dy) -> Dy {
    let density = crowding(r_kpc);
    Dy::ratio(752, 10) * Dy::int(2) * density / (Dy::ONE + density)
}

/// A star's vertical orbit at `r` after `age` Gyr: `u` draws its speed
/// (exponential), `phase` is its position in the oscillation (turns).
pub fn vertical(r_kpc: Dy, age: Dy, u: Dy, phase: Dy) -> Vertical {
    let density = crowding(r_kpc);
    // sigma = 20 km/s x 2t / (1 + t) x 2d / (1 + d), t = age / 10 Gyr: shares
    // of age and crowding in place of power laws; vz = -sigma ln(1 - u), an
    // exponential draw in place of a Rayleigh root
    let t = age.max(Dy::ZERO) / Dy::int(10);
    let sigma = Dy::int(20)
        * (Dy::int(2) * t / (Dy::ONE + t))
        * (Dy::int(2) * density / (Dy::ONE + density));
    let vz = sigma * -(Dy::ONE - u).max(Dy::sci(1, -12)).log();
    let nu = vertical_frequency(r_kpc);
    // amplitude A = vz / nu, 0.9778 converting km/s over rad/Gyr to kpc
    let amp = vz / (nu * Dy::ratio(9778, 10_000)).max(Dy::sci(1, -9));
    // exposure: the share of the oscillation inside the layer |z| < h,
    // linear in angle
    let h = layer_kpc();
    let exposure = if amp <= h {
        Dy::ONE
    } else {
        // atan(h / A) over the quarter turn, avoiding the root of 1 - x^2
        // in asin(h / A)
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
