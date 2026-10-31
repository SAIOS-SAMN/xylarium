//! What a world receives from its star and galaxy: ultraviolet activity,
//! wind dose, light above a band, crowding and cosmic rays.

use super::*;

/// A star's ultraviolet activity relative to a quiet one,
/// `1 + 9 exp(-age x k)`: 10 at birth, decaying toward 1, more slowly for
/// small stars (`star_mass` in the universe's stellar unit).
pub fn activity(star_mass: Dy, age: Dy) -> Dy {
    // The spin-down rate k is linear in stellar mass, at least 1/20.
    let spin_down = star_mass.max(Dy::ratio(1, 20));
    Dy::ONE + Dy::int(9) * (-(age.max(Dy::ZERO) * spin_down)).exp()
}

/// The integrated stellar activity a world receives from its star's birth to
/// `age` (Gyr x activity), the integral of `activity`:
/// `t + 9 (1 - exp(-t k)) / k`.
pub fn wind_dose(star_mass: Dy, age: Dy) -> Dy {
    let kk = star_mass.max(Dy::ratio(1, 20));
    let t = age.max(Dy::ZERO);
    t + Dy::int(9) * (Dy::ONE - (-(t * kk)).exp()) / kk
}

/// The share of a star's light carried above `x` kT: its light counted by
/// photons whose number grows linearly with energy under a Boltzmann factor,
/// so the share above x is (1 + x) e^(-x) (first degree in energy, not the
/// Planck cube and pi^4).
pub fn planck_above(x: Dy) -> Dy {
    if x.signum() <= 0 {
        return Dy::ONE;
    }
    if x > Dy::int(60) {
        return Dy::ZERO;
    }
    ((Dy::ONE + x) * (-x).exp()).min(Dy::ONE)
}

/// Starlight's colours against chemistry's: the share of a star's light that
/// can drive chemistry (enough to lift an electron, not enough to break a
/// bond) and the share that breaks bonds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Band {
    /// Share of the star's light, 0 to 1, between the photochemical threshold
    /// (2/5 of a covalent bond) and a covalent bond: light that drives chemistry.
    pub drive: Dy,
    /// Share of the star's light, 0 to 1, above a covalent bond's energy: light
    /// that breaks bonds.
    pub brk: Dy,
}

/// The split of a star of surface temperature `t_eff` (K) into light that
/// drives chemistry and light that breaks bonds, against a covalent bond under
/// the universe's scales `sc`.
pub fn band(t_eff: Dy, sc: &Scales) -> Band {
    let bond = constants().covalent_ev * sc.chem;
    let kt = constants().kb_ev * t_eff.max(Dy::ONE);
    let hi = planck_above(bond / kt);
    Band {
        drive: (planck_above(constants().photo_low * bond / kt) - hi).max(Dy::ZERO),
        brk: hi,
    }
}

/// The stellar surface density at the galaxy's reference ring (suns per
/// pc^2): a declared ruler, 0.1 suns/pc^3 through a 600 pc layer. The
/// sterilising pulse's base rate (`life::sterilise_per_gyr`) is calibrated
/// against this crowding; [`cosmic_rays`] is read relative to it, so it
/// enters only through that calibration.
pub fn crowd_r0() -> Dy {
    Dy::ratio(1, 10) * Dy::int(600)
}

/// The galaxy's cosmic-ray flux where a system circles, relative to the
/// reference ring's ([`crowd_r0`]). Supernovae produce them, so the flux
/// follows the crowding of the ring the system sits in: its holding per kpc
/// over the reference ring's (`galaxy::density`, from the structure read off
/// the founding; no bulge or nucleus profile is declared). The Big Bang's
/// star-formation share and the orbit's exposure scale it where it is read
/// (`life`).
pub fn cosmic_rays(r_kpc: Dy) -> Dy {
    Dy::from_q(&crate::galaxy::density(&r_kpc.to_q()).mid()).max(Dy::ZERO)
}
