//! Transit: distances between places, voyage times, the ley network with
//! its constellations, realms and quarantine, folding, and toll quotes
//! (docs/civilizations.md).
//!
//! Inputs are the [`Cosmic`](crate::cosmic::Cosmic) world (witnessed
//! systems, star places), the drives from propulsion.rs, the stars from
//! `xyl_cosmos::galaxy`, the orbits from `xyl_cosmos::manifold`, the turning
//! law (`xyl_cosmos::turning`) and the SI seam (`package::Si`). Distances are
//! in m or kpc, speeds in m/s, energies in J, and times in years or Gyr as
//! each item states.
//!
//! - Distance ([`separation_m`], at a count of the bell): within a system it
//!   is the gap between the positions of the two orbits ([`orbit_gap_m`]);
//!   between systems it is the arc between the stars
//!   ([`Cosmic::distance`](crate::cosmic::Cosmic::distance), the one arc
//!   [`crate::cosmic::arc_kpc`]). [`tier_m`] is the metres in a tier's unit.
//! - Voyage ([`voyage`]): the craft leaves the well at [`escape_speed`],
//!   coasts at half the remaining speed change ([`cruise_speed`]) and brakes
//!   with the other half; the fuel is payload x (R - 1) at the fuel's energy
//!   per kg. [`window_years`] is the wait for a transfer window within a
//!   system.
//! - Constellations ([`constellations`]): a constellation is a set of stars
//!   whose mutual turning is at least the galaxy's shear across their span; it
//!   is the unit of passage. Pairs are re-measured only when they could have
//!   crossed.
//! - [`Network`]: ley lines ([`Ley`]) are built by ancient houses and their
//!   progenitors, each sited where a passage has the greatest worth and
//!   opened at its due time (an event of the universe); a crossing takes the
//!   light time across a mouth ([`crossing_years`]). Realms are connected
//!   sets of constellations joined by open lines. A people's [`Discord`]
//!   (reaching off its constellation while forgetting, plus the breach from
//!   folding space) closes its constellation by the quarantine share.
//! - Tolls ([`quote`]): the floor is the unrecovered cost over last year's
//!   traffic; the ceiling is what the lane saves the passer; the toll lies
//!   half way between them and never above the ceiling; an open line asks
//!   only its floor. [`tithe`] is a passage's energy per lane length
//!   spanned. Tolls are quoted, not collected, because no traffic is carried
//!   yet.
//!
//! The lines built, the stewards' handling and the peoples' folds are held in
//! the checkpoint; constellations and quarantine are re-read from the laws.
//!
//! Terms: constellation, ley line, realm, quarantine, discord, fold,
//! handling, tithe, worth law (docs/glossary.md).
//!
//! Submodules: `voyage` (places, voyages, drives, transfer windows),
//! `tolls` (the toll law), `constellations` (binding and the stars in play),
//! `network` (the network and its ley lines), `realms` (realms and
//! handling), `discord` (discord, folding and quarantine), `building` (where
//! and when lines are built), `passage` (passage times over the network).

mod building;
mod constellations;
mod discord;
mod network;
mod passage;
mod realms;
#[cfg(test)]
mod tests;
mod tolls;
mod voyage;

pub use constellations::{
    binding_at, binding_pairs, binding_reach, constellations, in_play, Constellation, InPlay,
};
pub use discord::Discord;
pub use network::{crossing_years, Ley, Network, LINES_PER_READING};
pub use tolls::{ceiling, floor, hop_value, quote, tithe, Handling, Lane, Quote};
pub use voyage::{
    best_spec, blended, cruise_speed, escape_speed, home_escape, orbit_gap_m, reach_kpc,
    separation_m, voyage, window_years, Place, Voyage,
};

use xyl_core::dyadic::Dy;
use xyl_cosmos::package::Si;

/// Returns one astronomical unit in metres.
#[cfg(test)]
fn au_m() -> Dy {
    Dy::from_q(&Si::au())
}

/// Returns the metres in one unit of `tier`: the white hole's `r_g`
/// (`chain::units`) times the tier's unit in `r_g` (`Tier::in_rg`).
pub fn tier_m(tier: xyl_core::units::Tier) -> xyl_core::Q {
    &xyl_cosmos::chain::units().r_g_m * &tier.in_rg()
}

/// Returns one kiloparsec in metres, read from the SI seam (`package::Si`).
pub fn kpc_m() -> Dy {
    Dy::from_q(&Si::kpc())
}

/// Returns one year in seconds: one Gyr in seconds divided by 10^9.
pub fn year_s() -> Dy {
    (Dy::from_q(&Si::gyr_s()))
        .div(&(Dy::sci(1, 9)))
        .unwrap_or_default()
}

/// Converts a speed `m_s` in m/s to kpc per Gyr.
pub fn kpc_per_gyr(m_s: Dy) -> Dy {
    (m_s * Dy::from_q(&Si::gyr_s()))
        .div(&kpc_m())
        .unwrap_or_default()
}

/// Returns a passage's energy (J), which depends on the builder's drive
/// rather than on a constant. A throat carries a body of `body_kg` across at
/// the pulse's speed `c`, and the hole supplies only the part the builder's
/// drive (exhaust speed `v_e`, clamped to [0, c]) does not:
/// `E = body_kg * c * (c - v_e)`. As v_e -> c the passage costs nothing; at
/// v_e = 0 it costs `body_kg * c^2`.
pub fn passage_j(body_kg: Dy, v_e: Dy) -> Dy {
    let c = crate::propulsion::c_m_s();
    // Stand-in S126: The exhaust is clamped to c and the line's medium is not read.
    body_kg * c * (c - v_e.clamp(Dy::ZERO, c))
}
