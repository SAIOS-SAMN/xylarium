//! Life: how far a world's chemistry can climb toward a mind, written as
//! information copied against its decay, and when each rung is reached.
//!
//! Inputs: `worlds` (a world's `Setting`, `State` and `Medium`s, bond
//! energies, `Ctx` with `laws::Scales`), `galaxy::Star`, `systems::System`,
//! `stars::life` for the star's lifetime, `turning::year` (rings) for orbital
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
//!   at rate `mu` ([`strikes_per_gyr`]), `(e^(mu m) - 1) / mu` for a rung of
//!   mean wait `m`; the first rung is never undone. A surface is sterilised
//!   at `1 / hazard` ([`sterilise_per_gyr`], scaled by cosmic rays and plane
//!   exposure); media under ice or in the deep are shielded. A surface
//!   chemistry's window closes at 0.9 of the star's lifetime; every window
//!   closes when the world merges.
//! - Transfer ([`system_life`]): life reaches a sibling world after
//!   [`transfer_gyr`] (the target's year over the share `R / (2 pi a)` of
//!   the orbit that the target's body covers), divided by the dormant
//!   lineage's survival in transit.
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
        // An alphabet of four letters, the fewest that pair two ways, carries
        // 2 bits per monomer (a counting argument, not chemistry).
        bits_per_monomer: Dy::int(2),
        // Discrimination d: a partner is distinguished by one contact, half
        // the solvent's cohesion per molecule (declared).
        discrimination: Dy::ratio(1, 2),
        // Open rulers, to be derived from each world's chemistry: the
        // information a mind's blueprint holds (bits), the body built per
        // monomer of genome, and the mass of a mind-bearing body (kg).
        i_mind: Dy::sci(1, 9),
        body_per_monomer: Dy::sci(1, 3),
        m_mind_kg: Dy::int(10),
        amu_kg: Dy::from_q(&crate::package::Si::amu_kg()),
        bits_per_generation: Dy::ONE,
        reproductive_excess: Dy::ratio(1, 2),
        // Open ruler: the chance that a dying star's pulse sterilises an open
        // surface over 10 Gyr where the cosmic-ray crowd is 1.
        sterilise_at_r0: Dy::ratio(1, 10),
    })
}

/// The information in a mind's blueprint, `I_MIND` (bits). The kernel reads
/// it to rate how legible a mind's heredity is (xyl-kernel `knowledge.rs`).
pub fn i_mind() -> Dy {
    constants().i_mind
}

/// The information one monomer carries (bits), set by the alphabet's size.
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

/// The earliest age at which a world is read (Gyr after its star's birth):
/// one degree of the star's lifetime (`worlds::star_degree_gyr`) at the
/// star's distance `r_kpc` from the core, for a star of transduction sequence
/// `seq` and compactness `delta`.
pub fn first_age(seq: &crate::stars::Sequence, delta: Dy, r_kpc: Dy, ctx: &Ctx) -> Dy {
    worlds::star_degree_gyr(seq, delta, r_kpc, ctx)
}

/// The ages at which a world's media are probed (Gyr after its star's
/// birth): one degree of the star's lifetime and each doubling of it, while
/// the star's birth plus the age lies within the horizon, at most 64 ages.
pub fn probe_ages(home: &Home, ctx: &Ctx) -> Vec<Dy> {
    let mut out = Vec::new();
    let mut a = first_age(&home.sequence, home.compactness, home.r_kpc, ctx);
    while home.born + a < horizon_gyr() && out.len() < 64 {
        out.push(a);
        a = a * Dy::int(2);
    }
    out
}

/// The settled age at which a world's chemistry is read (Gyr).
pub fn settled_gyr() -> Dy {
    Dy::int(3)
}

/// The 500 of Franklin's 50/500 rule: the population size that keeps a
/// people's evolutionary potential.
pub fn viable() -> Dy {
    Dy::int(500)
}

#[cfg(test)]
mod tests;
