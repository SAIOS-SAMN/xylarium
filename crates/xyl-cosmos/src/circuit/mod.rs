//! The substrate (docs/model.md): the founding and the sections above it
//! (core, ocean, galaxies) as closed-form functions of time `t` in rings
//! (one ring one bell, the engine's unit of time; anchor.rs), evaluated on
//! demand rather than stored as frames. Rates are per ring.
//!
//! Inputs: the parent's spin (through `origin::torsion_share`), the founded
//! elements (`elements::Element`), `formation` for the cascade and the
//! lighting, `atoms::vessel`, `compound` for each kind's frames, and
//! `package` for the core's mass and the SI seam. Amounts are in held
//! frames, universe units (the beads' states sum to the founding unit);
//! reaches are in core rings about the universal throat, read in the
//! cluster tier (`xyl_core::units::Tier::Cluster`).
//!
//! - found.rs: `Substrate::found` throws every kind to its reach, holds the
//!   vessel's kinds at the throat as the firmament (scaled to mass 1) and
//!   beads the other shells at their pinches, each `Bead` with its reach,
//!   span, state, phase and recipe. cache.rs keeps the result in the home.
//! - anchor.rs: `Anchor`, the rim, bell (in core time r_g / c), ring length
//!   in seconds, tier units, first stars (rings) and spin the founding fixes
//!   for the process; `rings_of_gyr` and `gyr_of_rings`, the temporary
//!   boundary conversions where Gyr and rings meet.
//! - core.rs: the bell's clock, each firmament kind's `Reverberation`, the
//!   outflow `m0 (1 - exp(-rate (t - onset)))`, each kind's landing on the
//!   galaxies by pull times coherence, a galaxy's composition at `t`
//!   (recipe plus intake), the books, the waveform and each galaxy's `Push`.
//! - poles.rs, axes.rs: the pole axis and the pulse's planes as versors.
//! - weights.rs: proton and neutron counts and the electron phase state.
//!
//! A bead counts as a galaxy only where its lit holding is at least one
//! star's mass on the chain (`Substrate::holds_a_star`) and its core is no
//! heavier than the universal core (`Substrate::galaxies`).
//! Terms: substrate (docs/glossary.md).

use std::collections::BTreeMap;
use xyl_core::Q;

/// A mix of kinds: amount (held frames) by kind.
pub type Mix = BTreeMap<u32, Q>;

mod anchor;
mod axes;
mod cache;
mod core;
mod found;
mod grain;
mod poles;
mod profile;
mod weights;

pub use self::core::*;
pub use anchor::{
    anchor, gyr_of_rings, gyr_of_rings_dy, register_spin, rings_of_gyr, rings_of_gyr_dy,
    set as set_anchor, Anchor,
};
pub use axes::*;
pub use cache::cache_in;
pub use found::*;
pub(crate) use grain::*;
pub use poles::*;
pub(crate) use profile::*;
pub use weights::*;
