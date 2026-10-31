//! The home galaxy's chemical evolution: each element's abundance in the
//! disc's gas at any radius and time, relative to the Sun's birth gas.
//!
//! Inputs: `bang::BigBang` (first stars, star efficiency, the founding's
//! light nuclei via `elements::founded`, read as [`Light`]), `galaxy`
//! (`birth` for each ring's first light, `density` for the disc profile,
//! `core_reach`), `centre::galaxy_episode` for the core's echo, and the
//! element table's channel shares (`elements::Channel`). Reaches at the
//! public interface are galaxy rings, read in kpc inside through the
//! boundary `galaxy::kpc_of_rings` (`ring_of`); the model's radii are kpc,
//! times rings after the Big Bang (one ring one bell), rates per ring,
//! surface densities suns per pc^2. The rulers measured in Gyr (the slow
//! return, the delay windows, the patch's window, the calibration epochs)
//! are converted to rings once, where they are declared
//! (`turning::span_in_rings`); the disc's speed reaches the ring through the
//! SI seam (`package::Si::seconds_per_ring`).
//!
//! The disc is [`RINGS`] rings of 0.5 kpc (a spatial grid). On each ring:
//! - gas falls in as a boost on the ring's first-light timescale, scaled to
//!   the structure's profile; a ring whose gas never lights forms no stars;
//! - stars form at `efficiency x Omega / 2 pi` of the gas, `Omega = v / r`;
//! - `prompt` (0.2) of what forms returns at once and `slow` (0.2) later,
//!   half over 0.3 Gyr and half over 3 Gyr, driven by one iterate of the
//!   star formation without it;
//! - core collapse and spallation make their elements at once; type Ia, AGB
//!   and mergers follow `t^-1` delay laws held as four log-spaced boosts
//!   after their minimum delay (e-folds rounded to 1/1024 Gyr, then read in
//!   rings);
//! - the fountain throws `loading` of what forms, gas with its metals, and
//!   lands it by a landing map (Archimedes' hat-box over one epicycle, not a
//!   diffusion); what lands inside the centre goes to the core and returns
//!   as jets from the core's reach over the galaxy's echo; fountain and jets
//!   are fed once onto each ring;
//! - metals leave the gas with the stars that lock them away.
//!
//! Each is a linear relaxation driven by boosts, so every [`Ring`] holds
//! sums of `c t^m e^(-lambda t)` with exact rates
//! (`xyl_core::motion::ExpSum`), built once per ring (in parallel) and read
//! at any time.
//!
//! Outputs: [`Model`] (the rings' closed forms), [`Chemistry`] with
//! [`Chemistry::relative`], [`Chemistry::relatives`], [`Chemistry::metals`],
//! [`Chemistry::lump`] (a birth cloud's merger share, Poisson by inverse
//! CDF) and [`Chemistry::birth_along`] (a star's birth time along its ring's
//! star formation). Constructors return `None` when the element table or the
//! calibration cannot be built.
//!
//! Calibration: yield scales are fitted once per process, on the first
//! history built, to the measured gas at `Rulers::r0` today: core collapse
//! from O, type Ia from Fe, AGB from N; mergers and spallation at scale 1.
//! Every later universe in the process uses the same scales. A galactic
//! core's recipe (`cores`) is applied to these abundances in `systems`.
//!
//! Not modelled: radial gas drift, halo winds and their return, and the
//! Toomre threshold.

mod calibration;
mod chemistry;
mod model;

pub use calibration::{reference_birth_gas, Calibration};
pub use chemistry::{Chemistry, Light};
pub use model::{Model, Ring};

use crate::bang::BigBang;
use crate::elements::{decimal, Channel};
use xyl_core::certified::{ln, Encl};
use xyl_core::dyadic::Dy;
use xyl_core::Q;

const BITS: u32 = 72;
/// Number of radial rings on the galactic disc, each [`Rulers::dr`] (0.5 kpc)
/// wide, so the disc spans 0 to 15 kpc.
pub const RINGS: usize = 30;

/// The measured rulers.
pub struct Rulers;
impl Rulers {
    /// Width of one disc ring, 0.5 kpc.
    pub fn dr() -> Q {
        decimal("0.5").unwrap_or(Q::ZERO)
    }
    /// The disc's total surface density at the reference radius `r0`, in solar
    /// masses per square parsec (M_sun pc^-2): 50. The profile across the
    /// rings is the structure's crowding (`galaxy::density`), scaled to it.
    pub fn sigma_r0() -> Q {
        decimal("50").unwrap_or(Q::ZERO)
    }
    /// The share of formed stellar mass returned at once (0.2); with the slow
    /// return of `Rulers::slow`, 0.4 is returned in all.
    pub fn prompt() -> Q {
        decimal("0.2").unwrap_or(Q::ZERO)
    }
    /// The share of formed stellar mass the giants' winds return later (0.2),
    /// half over 0.3 Gyr and half over 3 Gyr.
    pub fn slow() -> Q {
        decimal("0.2").unwrap_or(Q::ZERO)
    }
    /// The fountain's loading: the gas it throws per mass of stars formed
    /// (declared).
    pub fn loading() -> Q {
        decimal("1").unwrap_or(Q::ZERO)
    }
    /// The gas's mean particle mass A in proton masses (1.3), which sets the
    /// fountain's reach relative to air: x = 28 / A read as the share
    /// 2x / (1 + x), with no root.
    pub fn gas_mu() -> Q {
        decimal("1.3").unwrap_or(Q::ZERO)
    }
    /// Helium-to-metal enrichment ratio dY/dZ, dimensionless (2.0).
    pub fn dy_dz() -> Q {
        decimal("2.0").unwrap_or(Q::ZERO)
    }
    /// The Sun's metal mass fraction Z, 0.0142.
    pub fn reference_metallicity() -> Q {
        decimal("0.0142").unwrap_or(Q::ZERO)
    }
    /// The reference galaxy's age today (rings): 13.8 Gyr in rings, the
    /// calibration epoch, removed in phase 5 (docs/roadmap.md, milestone 3).
    pub fn today() -> Q {
        crate::turning::span_in_rings(&decimal("13.8").unwrap_or(Q::ZERO))
    }
    /// The reference ring (galaxy rings): the galaxy's half-holding reach
    /// (`galaxy::r0`), read from the founding's structure.
    pub fn r0() -> Q {
        crate::galaxy::r0()
    }
    /// The disc's flat rotation speed, 220 km/s.
    pub fn v_flat() -> Q {
        decimal("220").unwrap_or(Q::ZERO)
    }
    /// The gas at R0 today, in dex against solar (the Cosmic Abundance
    /// Standard); 0 for elements not listed.
    pub fn ism_today_dex(sym: &str) -> Q {
        decimal(match sym {
            "C" => "-0.10",
            "N" => "-0.04",
            "O" => "0.07",
            "Mg" => "-0.04",
            "Si" => "-0.01",
            "Fe" => "0.02",
            _ => "0",
        })
        .unwrap_or(Q::ZERO)
    }
}

/// A channel's delay window as declared (Gyr, measured rulers): its first
/// delay and its last; zero for the prompt channels.
fn delay_declared(c: Channel) -> (Q, Q) {
    match c {
        Channel::CoreCollapse | Channel::Spallation | Channel::BigBang => (Q::ZERO, Q::ZERO),
        Channel::TypeIa => (
            decimal("0.04").unwrap_or(Q::ZERO),
            decimal("14.0").unwrap_or(Q::ZERO),
        ),
        Channel::Agb => (
            decimal("0.1").unwrap_or(Q::ZERO),
            decimal("10.0").unwrap_or(Q::ZERO),
        ),
        Channel::Merger => (
            decimal("0.01").unwrap_or(Q::ZERO),
            decimal("14.0").unwrap_or(Q::ZERO),
        ),
    }
}

/// The evolved channels (the founding's share is uniform).
pub const CHANNELS: [Channel; 5] = [
    Channel::CoreCollapse,
    Channel::TypeIa,
    Channel::Agb,
    Channel::Merger,
    Channel::Spallation,
];

/// A channel's delay window (rings): its first delay and its last, the
/// declared window (`delay_declared`) read in rings; zero for the prompt
/// channels.
fn delay(c: Channel) -> (Q, Q) {
    let (lo, hi) = delay_declared(c);
    (
        crate::turning::span_in_rings(&lo),
        crate::turning::span_in_rings(&hi),
    )
}

fn channel_index(c: Channel) -> Option<usize> {
    CHANNELS.iter().position(|&x| x == c)
}

/// A delay law t^-1 over the declared window [lo, hi] (Gyr), held as four
/// boosts after `lo`: e-fold times log-spaced across the window, each
/// carrying a quarter of the mass (a log-even mixture, as t^-1 is), the
/// e-fold times rounded down to 1/1024 Gyr and at least 1/1024 Gyr
/// (declared), returned in rings.
fn delay_modes(lo: &Q, hi: &Q) -> Vec<(Q, Q)> {
    let ratio = Dy::from_q(&hi.checked_div(lo).unwrap_or(Q::ONE));
    (0..4)
        .map(|j| {
            // The e-fold times `tau_j = lo (hi / lo)^((2j + 1) / 8)` are log-even
            // across the window.
            let tau =
                Dy::from_q(lo) * (ratio.max(Dy::sci(1, -30)).log() * Dy::ratio(2 * j + 1, 8)).exp();
            let tau = Q::new((tau * Dy::int(1024)).trunc_i64().max(1), 1024).unwrap_or(Q::ONE);
            (Q::ratio(1, 4), crate::turning::span_in_rings(&tau))
        })
        .collect()
}

/// The galaxy's parameters from its Big Bang: the first stars, each ring's
/// infall and the star efficiency.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Params {
    /// When the first stars shone, in rings after the Big Bang.
    pub first_stars: Q,
    /// Each ring's first light (rings after the Big Bang), by ring index
    /// (`galaxy::birth`); `None` where the ring's gas never lights and no
    /// stars form.
    pub infall: Vec<Option<Q>>,
    /// The star efficiency: the lit share of the gas, turned into stars each
    /// orbit (`BigBang::efficiency`).
    pub efficiency: Q,
}

impl Params {
    /// The parameters read off the Big Bang `bang`'s founding.
    pub fn of(bang: &BigBang) -> Params {
        Params {
            first_stars: bang.first_stars().mid(),
            infall: (0..RINGS)
                .map(|j| crate::galaxy::birth(&crate::galaxy::rings_of_kpc(&ring_mid(j))))
                .collect(),
            efficiency: bang.efficiency.clone(),
        }
    }
}

/// The factor converting km/s per kpc to a rate per ring: 1000 m per km
/// times the SI seconds of a ring, over the metres in a kpc (the SI seam,
/// `package::Si`), held to 62 bits. Zero before a founding.
fn km_s_per_kpc_per_ring() -> Q {
    use crate::package::Si;
    (&Q::int(1000) * &Si::seconds_per_ring())
        .checked_div(&Si::kpc())
        .map(|f| Dy::from_q(&f).to_q())
        .unwrap_or(Q::ZERO)
}

fn ring_mid(j: usize) -> Q {
    &Q::int(i64::try_from(2 * j + 1).unwrap_or(0)) * &decimal("0.25").unwrap_or(Q::ZERO)
}

/// Ring `j`'s star-formation timescale (rings): its first light, at least
/// 0.001 Gyr in rings; `None` where the ring's gas never lights.
fn infall_time(p: &Params, j: usize) -> Option<Q> {
    let floor = crate::turning::span_in_rings(&decimal("0.001").unwrap_or(Q::ZERO));
    p.infall
        .get(j)
        .cloned()
        .flatten()
        .map(|t| t.max(floor.clone()))
}

fn sigma_total(r: &Q) -> Encl {
    // The disc's profile is the galaxy's crowding (`galaxy::density`, read in
    // galaxy rings through the boundary `galaxy::rings_of_kpc`), scaled to
    // the reference ring's surface density.
    crate::galaxy::density(&crate::galaxy::rings_of_kpc(r)).mul(&Encl::exact(Rulers::sigma_r0()))
}

/// The value of ln 10: the midpoint of its 72-bit enclosure (1 if the enclosure fails).
pub fn ln10() -> Q {
    ln(&Q::int(10), BITS).map(|e| e.mid()).unwrap_or(Q::ONE)
}

/// The index of the ring holding reach `r` (galaxy rings): floor(2 r_kpc),
/// clamped to [0, RINGS - 1], with the reach read in kpc through the
/// boundary `galaxy::kpc_of_rings` until the chemistry's rings are galaxy
/// rings.
pub fn ring_of(r: &Q) -> usize {
    let r_kpc = crate::galaxy::kpc_of_rings(r);
    let j = (&r_kpc * &Q::int(2)).floor_i128().unwrap_or(0).max(0);
    usize::try_from(j).unwrap_or(0).min(RINGS - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abundances_nonnegative_and_none_before_first_stars() {
        let bang = crate::genesis::bang();
        let chem = crate::genesis::chem();
        let today = Rulers::today();
        // Oxygen's abundance at r0 today is non-negative.
        let o = chem.relative("O", &Rulers::r0(), &today, &Q::ONE).unwrap();
        assert!(o.signum() >= 0, "O at R0 today: {o}");
        // So is the metallicity.
        let made = chem.metals(&Rulers::r0(), &today).unwrap();
        assert!(made.signum() >= 0, "metals {made}");
        // At half the first stars' epoch no channel has made uranium yet.
        let before = bang.first_stars().lo.checked_div(&Q::int(2)).unwrap();
        assert_eq!(
            chem.relative("U", &Rulers::r0(), &before, &Q::ONE).unwrap(),
            Q::ZERO
        );
    }
}
