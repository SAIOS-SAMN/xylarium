//! Transit: distances between places, voyage times, the ley network with
//! its constellations, realms and quarantine, folding, and toll quotes
//! (docs/civilizations.md).
//!
//! Inputs: the [`Cosmic`](crate::cosmic::Cosmic) world (witnessed systems, star places), drives
//! from propulsion.rs, stars from `xyl_cosmos::galaxy`, orbits from
//! `xyl_cosmos::manifold`, the turning law (`xyl_cosmos::turning`) and the
//! SI seam (`package::Si`). Distances are in m or kpc, speeds in m/s,
//! energies in J, times in years or Gyr as each item states.
//!
//! - Distance ([`separation_m`]): within a system, the gap between where
//!   the two orbits stand now ([`orbit_gap_m`]); between systems, the arc
//!   between the stars ([`Cosmic::distance`](crate::cosmic::Cosmic::distance)).
//! - Voyage ([`voyage`]): out of the well at [`escape_speed`], coasting at
//!   half the speed change left ([`cruise_speed`]) and braking with the
//!   other half; fuel is payload x (R - 1) at the fuel's energy per kg.
//!   [`window_years`] is the wait for a transfer window within a system.
//! - Constellations ([`constellations`]): stars bound by their mutual
//!   turning tighter than the galaxy's shear across their span; the unit of
//!   passage. Pairs are re-measured only when they could have crossed.
//! - [`Network`]: ley lines ([`Ley`]) built by ancient houses and their
//!   progenitors, each sited where a passage is worth most and opened at
//!   its due time (an event of the universe); a crossing takes a mouth's
//!   light time ([`crossing_years`]). Realms are connected sets of
//!   constellations joined by open lines. A people's [`Discord`] (reaching
//!   off its constellation while forgetting, plus its breach from folding
//!   space) closes its constellation by the quarantine share.
//! - Tolls ([`quote`]): floor = unrecovered cost over last year's traffic;
//!   ceiling = what the lane saves the passer; toll = half way between,
//!   never above the ceiling; an open line asks only its floor. [`tithe`]
//!   is a passage's energy per lane length spanned. Tolls are quoted, not
//!   collected: no traffic is carried yet.
//!
//! Lines built, stewards' handling and peoples' folds are held in the
//! checkpoint; constellations and quarantine are re-read from the laws.
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
use xyl_core::Q;
use xyl_cosmos::package::Si;

/// An astronomical unit (m).
#[cfg(test)]
fn au_m() -> Dy {
    Dy::from_q(&Si::au())
}

/// Kiloparsecs per Gyr in one km/s: a thousand metres a second over a Gyr
/// in seconds, per kiloparsec (the one SI seam, crate::package::Si).
pub fn kms_to_kpc_gyr() -> Dy {
    Dy::from_q(
        &(&Q::int(1000) * &Si::gyr_s())
            .checked_div(&Si::kpc())
            .unwrap_or(Q::ZERO),
    )
}

/// A kiloparsec (m), the one SI seam (package::Si).
pub fn kpc_m() -> Dy {
    Dy::from_q(&Si::kpc())
}

/// A standard year (s).
pub fn year_s() -> Dy {
    Dy::from_q(&Si::gyr_s()) / Dy::sci(1, 9)
}

/// A speed in m/s as kpc per Gyr.
pub fn kpc_per_gyr(m_s: Dy) -> Dy {
    m_s * Dy::from_q(&Si::gyr_s()) / kpc_m()
}

/// A passage's energy (J), set by the builder's capability rather than a
/// constant: a throat carries a body of `body_kg` across at the pulse's
/// speed, and the hole supplies only what the builder's drive (exhaust `v_e`)
/// does not: body c (c - v_e). At v_e -> c a passage costs nothing; a people
/// that cannot leave its well pays the body's whole energy.
pub fn passage_j(body_kg: Dy, v_e: Dy) -> Dy {
    let c = crate::propulsion::c_m_s();
    body_kg * c * (c - v_e.clamp(Dy::ZERO, c))
}
