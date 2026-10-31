//! The Xylarium physical model in exact arithmetic: a universe's founding
//! and the home galaxy's stars, worlds, life and minds, derived from a seed.
//!
//! The founding: a universe begins as the white hole of a parent black hole
//! ([`bounce`]); its Big Bang ([`bang`]) draws the parent's spin from the
//! seed. The elements ([`elements`], [`atoms`], [`compound`], [`fibres`])
//! are thrown by the founding cascade ([`formation`], [`circuit`]): the
//! throat holds the firmament, which carries the universal state, and the
//! rest beads into galaxies ([`cores`]). [`package`] holds what the
//! founding takes that the laws do not derive. [`laws`] holds the four
//! dimensionless constants; [`turning`] the turning law.
//!
//! Above the founding: the home galaxy and its stars ([`galaxy`],
//! [`stars`], [`chem`], [`chain`], [`centre`]), the bells and their pulses
//! ([`pulse`], [`field`]), each star's disc and worlds on its manifold
//! ([`disc`], [`systems`], [`manifold`], [`bodies`], [`worlds`],
//! [`hydro`]), and life and the forecast of minds ([`life`], [`minds`]). A
//! change above the founding leaves the founding digest, and so a cached
//! founding, unchanged.
//!
//! Units are `G = c = M_core = 1` (`xyl_core::units`); SI enters only
//! through `package::Si`. Geometry uses the conformal algebra
//! ([`xyl_core::cga`]) and certified enclosures ([`xyl_core::certified`])
//! wherever a law is transcendental. [`Universe::create`] returns None
//! where the Big Bang cannot be founded; the same seed gives the same
//! universe on every machine.

pub mod atoms;
pub mod bang;
pub mod bodies;
pub mod bounce;
pub mod centre;
pub mod chain;
pub mod chem;
pub mod circuit;
pub mod compound;
pub mod cores;
pub mod disc;
pub mod elements;
pub mod fibres;
pub mod field;
pub mod formation;
pub mod galaxy;
#[cfg(test)]
pub(crate) mod genesis;
pub mod hydro;
pub mod laws;
pub mod life;
pub mod manifold;
pub mod minds;
pub mod package;
pub mod pulse;
pub mod stars;
pub mod systems;
pub mod turning;
pub mod worlds;

pub use bang::BigBang;
pub use bounce::Bounce;
pub use laws::Laws;

/// A universe: one origin (its seed, its laws, its Big Bang). Everything
/// else in it is derived.
#[derive(Clone, Debug)]
pub struct Universe {
    /// The seed every draw in this universe derives from.
    pub seed: u64,
    /// The physical laws (constants and their bounds) this universe runs under.
    pub laws: Laws,
    /// The Big Bang drawn from `seed`.
    pub bang: BigBang,
}

impl Universe {
    /// The universe of `seed` under `laws`. The same seed under other laws
    /// starts from the same initial conditions (its Big Bang is drawn from
    /// the seed alone).
    pub fn create(seed: u64, laws: Laws) -> Option<Universe> {
        Some(Universe {
            seed,
            laws,
            bang: BigBang::of(seed)?,
        })
    }
}
