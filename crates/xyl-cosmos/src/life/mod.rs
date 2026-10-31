//! Life: how far a world's chemistry can climb toward a mind, written as
//! information copied against its decay, and when each rung is reached.
//!
//! Inputs: `worlds` (a world's `Setting`, `State` and `Medium`s, bond
//! energies, `Ctx` with `laws::Scales`), `galaxy::Star`, `systems::System`,
//! `stars::life` for the star's lifetime, `turning::year_yr` for orbital
//! periods, and SI constants from `package::Si`. Times are Gyr after the Big
//! Bang, all quantities `Dy`. No per-world parameter sets how common or fast
//! life is; it follows from these formulas under each universe's scales.
//!
//! - Fidelity ([`fidelity`], [`field_fidelity`]): a monomer is miscopied
//!   with `eps0 = exp(-d E / kT)`, `d = 1/2` of the solvent's weak bond
//!   (Boltzmann). A plasma copies patterns held in its field, with `E` the
//!   field energy per carrier. `eps0 >= 1/2` copies nothing.
//! - Ladder ([`ladder`]): rung `s` holds `b / eps^(s+1)` bits (Eigen's
//!   ceiling, raised a rung at a time by kinetic proofreading), with `eps`
//!   the bare error plus decay ([`decay_rate`]) over one generation; the
//!   ladder is complete where a rung reaches `I_MIND` ([`i_mind`]), at most
//!   [`MAX_RUNGS`]. Each [`Step`]'s mean wait follows from Haldane's cost of
//!   selection capped at one bit per generation ([`bits_per_generation`]),
//!   the copy rate `kT/h e^-barrier` ([`copy_rate`]), a generation of
//!   `t_cell (1 + log2(body / cell))`, and the population the medium's power
//!   feeds.
//! - Climb ([`climb`]): the expected completion of each rung under impacts
//!   at rate `mu` ([`strikes_per_gyr`]), `(e^(mu m) - 1) / mu`; the first
//!   rung is never undone. A surface is sterilised at `1 / hazard`
//!   ([`sterilise_per_gyr`], scaled by cosmic rays and plane exposure); media
//!   under ice or deep are shielded. A surface chemistry's window closes at
//!   0.9 of the star's lifetime; every window closes when the world merges.
//! - Transfer ([`system_life`]): life reaches a sibling world after
//!   [`transfer_gyr`] (its year over the share `R / (2 pi a)` of its orbit
//!   its body covers), divided by the dormant lineage's survival in transit.
//!   Transfer is modelled within a system only.
//! - Feeding ([`foragers`]): living area, growing season ([`growing_share`])
//!   and energy through one trophic link, over each body's metabolism;
//!   [`viable`] is the 500 of Franklin's 50/500 rule.
//!
//! Every function is deterministic: no random draw enters.
//!
//! Terms: rung (docs/glossary.md).

use crate::galaxy::Star;
use crate::systems::System;
use crate::worlds::{
    self, best, build, media, Ctx, Galaxy, Medium, Region, Setting, Solvent, State, Where,
};
use std::sync::OnceLock;
use xyl_core::dyadic::Dy;

mod body;
mod ladder;
mod living;
mod rates;

pub use body::*;
pub use ladder::*;
pub use living::*;
pub use rates::*;

/// Life's constants: physical (measured) and declared.
struct Constants {
    kb_j: Dy,
    h_j: Dy,
    ev_j: Dy,
    gyr_s: Dy,
    flux_w: Dy,
    bits_per_monomer: Dy,
    discrimination: Dy,
    i_mind: Dy,
    body_per_monomer: Dy,
    m_mind_kg: Dy,
    amu_kg: Dy,
    bits_per_generation: Dy,
    reproductive_excess: Dy,
    sterilise_at_r0: Dy,
}

fn constants() -> &'static Constants {
    static C: OnceLock<Constants> = OnceLock::new();
    C.get_or_init(|| Constants {
        kb_j: Dy::from_q(&crate::package::Si::kb_j()),
        h_j: Dy::from_q(&crate::package::Si::h_j()),
        ev_j: Dy::from_q(&crate::package::Si::ev_j()),
        gyr_s: Dy::from_q(&crate::package::Si::gyr_s()),
        flux_w: Dy::int(340),
        // an alphabet of four, the fewest letters that pair two ways: 2 bits
        // (a counting argument, not chemistry)
        bits_per_monomer: Dy::int(2),
        // discrimination d: a partner is told apart by one contact, half the
        // solvent's cohesion per molecule (declared)
        discrimination: Dy::ratio(1, 2),
        // open rulers, to be derived from each world's own chemistry once
        // planets are stable: the information a mind's blueprint holds, the
        // body built per monomer of genome, and the mass of a mind-bearing body
        i_mind: Dy::sci(1, 9),
        body_per_monomer: Dy::sci(1, 3),
        m_mind_kg: Dy::int(10),
        amu_kg: Dy::from_q(&crate::package::Si::amu_kg()),
        bits_per_generation: Dy::ONE,
        reproductive_excess: Dy::ratio(1, 2),
        // open ruler: the chance a dying star's pulse sterilises an open
        // surface over 10 Gyr where the cosmic-ray crowd is 1
        sterilise_at_r0: Dy::ratio(1, 10),
    })
}

/// A mind's blueprint (bits): I_MIND, for the kernel's readings of a mind
/// (how legible its heredity is: xyl-kernel knowledge.rs).
pub fn i_mind() -> Dy {
    constants().i_mind
}

/// Bits a monomer carries (the alphabet's).
pub fn bits_per_monomer() -> Dy {
    constants().bits_per_monomer
}

/// A chemistry needing more rungs than this never reaches a mind within any
/// star's life.
pub const MAX_RUNGS: usize = 12;

/// Deep time the forecast looks ahead to (Gyr).
pub fn horizon_gyr() -> Dy {
    Dy::int(30)
}

/// The earliest age a world is read at (Gyr after its star's birth): one
/// degree of its own star's life (`worlds::star_degree_gyr`), at the star's
/// distance from the core `r_kpc`.
pub fn first_age(r_kpc: Dy, ctx: &Ctx) -> Dy {
    worlds::star_degree_gyr(r_kpc, ctx)
}

/// The ages a world's media are probed at (Gyr after its star's birth): one
/// degree of its star's life and each doubling after, while its star's birth
/// plus the age lies within the horizon.
pub fn probe_ages(home: &Home, ctx: &Ctx) -> Vec<Dy> {
    let mut out = Vec::new();
    let mut a = first_age(home.r_kpc, ctx);
    while home.born + a < horizon_gyr() && out.len() < 64 {
        out.push(a);
        a = a * Dy::int(2);
    }
    out
}

/// The settled age a world's chemistry is read at (Gyr).
pub fn settled_gyr() -> Dy {
    Dy::int(3)
}

/// Franklin's 50/500: the number that keeps a people's evolutionary potential.
pub fn viable() -> Dy {
    Dy::int(500)
}

#[cfg(test)]
mod tests;
