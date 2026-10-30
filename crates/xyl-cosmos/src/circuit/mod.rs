//! The substrate (docs/model.md): space is carried as sections, each a
//! function of time t and of the section above it, evaluated on demand rather
//! than stored as frames or parcels.
//!
//! - The founding (found.rs): the cascade throws every kind to its reach; the
//!   throat holds the vessel's kinds (the firmament); the remaining kinds form
//!   shells whose overlaps bead at their pinches - the galaxies, each with its
//!   state (share of the founding's mass) and recipe (the kinds carried in).
//! - The core (core.rs): the bell's clock in closed form; the firmament's
//!   reverberation (each kind's band swinging with the bell's torque past the
//!   throat) and the outflow, each kind emptying as m(t) = m0 exp(-nu s t).
//! - The ocean (core.rs): the outflow shared among the galaxies by their pull
//!   from the core, after the pulse's delay to their reach.
//! - A galaxy (core.rs): its composition at t, its recipe plus what it has
//!   taken; a bead is a galaxy only if it holds a sun.
//!
//! Units: amounts in held frames, universe units (the galaxies' states sum to
//! the founding unit); reaches in core rings about the universal throat.

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
