//! A world's air, seas and climate at an age, and the chemistries (media)
//! they allow.
//!
//! Inputs: a `systems::World` and its `System`, the orbit and spin drawn by
//! `bodies::orbits`, the interior heat, field and processed mantle from
//! `bodies::interior`, radiolysis from `hydro`, metals from
//! `chem::Chemistry`, and the universe's `laws::Scales` (temperatures and
//! pressures scale with them). Distances are in AU, masses and radii in
//! world units, temperatures in K, pressures in bar, ages in Gyr.
//!
//! The atmosphere is what the world gathered as ices (systems.rs) times the
//! share its interior outgassed, less what escapes:
//!
//!   Temperature  absorbed light read linearly: flux `L / a` (not `L / a^2`,
//!                the mean of 1 / r over an eccentric orbit is 1 / a),
//!                warmth the absorbed share times the unit temperature (no
//!                T^4 law), albedo from the surface's ice cover. `L` is the
//!                star's mean luminosity as the caller supplies it.
//!   Outgassing   a freezing magma ocean releases most of its volatiles (in
//!                proportion to the mantle share accretion melted); the
//!                convecting mantle releases what its heat has processed.
//!   Escape       Jeans: a gas of molecular mass mu is held if lambda =
//!                G M mu m_H / (k T_exo R) is large; the exosphere is hotter
//!                under an active star and with no field. Ultraviolet above
//!                a declared dose converts NH3 to N2 and CH4 and CO to CO2;
//!                the stellar wind breaks molecules and fragments escape
//!                with a few eV, applied in four passes with the climate
//!                re-solved between them.
//!   Greenhouse   surface warming 1 + 3/4 h with h = tau / (1 + tau); each gas
//!                weighted by its cluster's hold (`atoms::hold`, in water's
//!                units; homonuclear pairs hold none). `greenhouse_tau` is
//!                provisional, calibrated so the unit world reads near 288 K.
//!                Vapour sits at saturation; the climate is the warmest
//!                stable balance on a temperature ladder, and a steam
//!                atmosphere rains out unless absorbed flux exceeds the
//!                runaway limit (Nakajima et al. 1992). Weathering on open
//!                water draws CO2 down until it balances outgassing; cold
//!                poles trap condensables; ice and water at a melting point
//!                are solved for the ice share.
//!   Solvents     liquid between the triple point and the boiling curve
//!                (Clausius-Clapeyron) up to the critical point, supercritical
//!                above both; a subsurface H2O or NH3 ocean where internal
//!                heat keeps the ice shell thinner than the ice layer;
//!                silicate melt above 1400 K (scaled); a deep plasma in a
//!                giant of at least ten world masses.
//!   Chemistry    per medium: richness (available elements x strong bonds
//!                near 40 kT x weak bonds near 10 kT of the solvent's
//!                cohesion), energy (active starlight, free internal heat,
//!                radiolysis), pace (Eyring, activation scaled to the
//!                solvent's range) and its `Region`.
//!
//! Outputs: `build` returns a world's `State` at an age; `media` and `best`
//! its chemistries; `settings` the `Setting`s of a system's worlds (orbit
//! draws keyed by the seed and `["orbits", path]`); `events` the solved
//! spin-lock and dynamo-stop times (`xyl_core::event::crossing`); `unit` the
//! reference world, a ruler; `star_degree_gyr` the degree of a star's life.
//! Invariants: no world's values are built in; arithmetic is dyadic (`Dy`),
//! deterministic and platform-independent; a giant's state is its cloud
//! tops, with no surface air or seas.
//!
//! Terms: degree (of a star's life) (docs/glossary.md).

use crate::bang::BigBang;
use crate::bodies::{self, Orbit};
use crate::chem::Chemistry;
use crate::elements::Element;
use crate::galaxy::Star;
use crate::hydro;
use crate::laws::Scales;
use crate::systems::{System, World};
use std::sync::OnceLock;
use xyl_core::certified::Encl;
use xyl_core::dyadic::Dy;
use xyl_core::motion::{Boost, Modes};
use xyl_core::Q;

mod climate;
mod media;
mod setting;
mod star;
mod system;
mod unit;
mod volatiles;

pub use climate::*;
pub use media::*;
pub use setting::*;
pub use star::*;
pub use system::*;
pub use unit::*;
pub use volatiles::*;

/// The models' measured and declared constants.
struct Constants {
    t_unit: Dy,
    gm_over_r_unit: Dy,
    k_over_mh: Dy,
    g_unit_pa_per_massfrac: Dy,
    bar: Dy,
    kb_ev: Dy,
    r_gas: Dy,
    delivered: Dy,
    photolysis_uv: Dy,
    strip_rate: Dy,
    transport_bar: Dy,
    hot_ev: Dy,
    weather_e_k: Dy,
    weather_scale: Dy,
    jeans_hold: Dy,
    thermosphere: Dy,
    runaway_flux: Dy,
    greenhouse_tau: Dy,
    melt_k: Dy,
    giant_plasma_mass: Dy,
    solvent_barrier: Dy,
    strong_kt: Dy,
    weak_kt: Dy,
    geo_flux: Dy,
    heat_unit_w: Dy,
    giant_flux: Dy,
    plasma_window: Dy,
    photo_low: Dy,
    unit_g: Dy,
    latent_rock: Dy,
    magma_degas: Dy,
    m_h_kg: Dy,
    ev_j: Dy,
    k_ice: Dy,
    covalent_ev: Dy,
    silicate_ev: Dy,
    melt_network_ev: Dy,
    faraday: Dy,
}

fn constants() -> &'static Constants {
    static C: OnceLock<Constants> = OnceLock::new();
    C.get_or_init(|| Constants {
        t_unit: Dy::int(278),
        gm_over_r_unit: Dy::sci(626, 5),
        k_over_mh: Dy::int(8254),
        g_unit_pa_per_massfrac: Dy::sci(117, 9),
        bar: Dy::sci(1, 5),
        kb_ev: Dy::from_q(&crate::package::Si::kb_ev()),
        r_gas: Dy::from_q(&crate::package::Si::r_gas()),
        delivered: Dy::sci(1, -4),
        // declared: bond-breaking light (flux units) above which NH3 and CH4 are
        // photolysed within years: 0.03 of a Sun's flux at its ~6.5% share and
        // middle-aged activity (Titan, at ~0.01, keeps its methane)
        photolysis_uv: Dy::sci(3, -2) * Dy::sci(65, -3) * Dy::ratio(14, 10),
        strip_rate: Dy::sci(2, -6),
        transport_bar: Dy::ratio(1, 10),
        hot_ev: Dy::ratio(5, 2),
        weather_e_k: Dy::ratio(23, 2),
        weather_scale: Dy::sci(1, -3),
        jeans_hold: Dy::int(15),
        thermosphere: Dy::int(4),
        runaway_flux: Dy::ratio(282, 340),
        // provisional: calibrated so the unit world reads near 288 K (286 K)
        greenhouse_tau: Dy::ratio(31, 5),
        melt_k: Dy::int(1400),
        giant_plasma_mass: Dy::int(10),
        solvent_barrier: Dy::int(25),
        strong_kt: Dy::int(40),
        weak_kt: Dy::int(10),
        geo_flux: Dy::sci(87, -3) / Dy::int(340),
        heat_unit_w: Dy::sci(87, -3)
            * Dy::int(4)
            * Dy::PI
            * Dy::from_q(&crate::package::Si::world_radius_m())
            * Dy::from_q(&crate::package::Si::world_radius_m()),
        giant_flux: Dy::ratio(54, 10) / Dy::int(340),
        plasma_window: Dy::sci(2, -3),
        photo_low: Dy::ratio(2, 5),
        unit_g: Dy::sci(981, -2),
        latent_rock: Dy::sci(4, 5),
        magma_degas: Dy::ratio(9, 10),
        m_h_kg: Dy::from_q(&crate::package::Si::m_h_kg()),
        ev_j: Dy::from_q(&crate::package::Si::ev_j()),
        k_ice: Dy::int(567),
        covalent_ev: Dy::ratio(36, 10),
        silicate_ev: Dy::ratio(46, 10),
        melt_network_ev: Dy::ONE,
        faraday: Dy::from_q(&crate::package::Si::faraday()),
    })
}

/// Boltzmann's constant in eV/K.
pub fn kb_ev() -> Dy {
    constants().kb_ev
}

/// A solvent's backbone bond (eV) under a universe's laws: silicate for a
/// melt, covalent otherwise.
pub fn strong_bond_ev(s: Solvent, sc: &Scales) -> Dy {
    (if s == Solvent::Melt {
        constants().silicate_ev
    } else {
        constants().covalent_ev
    }) * sc.chem
}

/// A covalent bond (eV) under a universe's laws.
pub fn covalent_ev(sc: &Scales) -> Dy {
    constants().covalent_ev * sc.chem
}

/// Activation energy in units of kT at a solvent's own mid-range (declared).
pub fn solvent_barrier() -> Dy {
    constants().solvent_barrier
}

/// The unit temperature (K at flux 1 and albedo 0: the flux unit's).
pub fn t_unit() -> Dy {
    constants().t_unit
}

#[cfg(test)]
mod tests;
