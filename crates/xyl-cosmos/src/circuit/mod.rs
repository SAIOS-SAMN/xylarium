//! The substrate (docs/model.md): the founding and the sections above it
//! (core, ocean, galaxies) as closed-form functions of time `t` in Gyr,
//! evaluated on demand rather than stored as frames.
//!
//! Inputs: the parent's spin (through `bang::torsion_share`), the founded
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
//! - anchor.rs: `Anchor`, the rim, bell, tier units, first stars and spin
//!   the founding fixes for the process.
//! - core.rs: the bell's clock, each firmament kind's `Reverberation`, the
//!   outflow `m0 (1 - exp(-rate (t - onset)))`, each kind's landing on the
//!   galaxies by pull times coherence, a galaxy's composition at `t`
//!   (recipe plus intake), the books, the waveform and each galaxy's `Push`.
//! - poles.rs, axes.rs: the pole axis and the pulse's planes as versors.
//! - weights.rs: proton and neutron counts and the electron phase state.
//!
//! A bead counts as a galaxy only where its lit mass is at least one sun.
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
pub use anchor::{anchor, register_spin, set as set_anchor, Anchor};
pub use axes::*;
pub use cache::cache_in;
pub use found::*;
pub(crate) use grain::*;
pub use poles::*;
pub(crate) use profile::*;
pub use weights::*;
