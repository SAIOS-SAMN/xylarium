//! Exact numerics for xylarium.
//!
//! Every held quantity is exact: rationals ([`Q`]), certified enclosures
//! ([`certified::Encl`]) for roots and transcendentals, and dyadic rationals
//! ([`dyadic::Dy`]) for bounded intermediate readings. [`cga`] is the
//! conformal geometric algebra `Cl(n+1,1)` over `Q`, [`rotor`] the rational
//! rotors, [`units`] the unit ladder anchored at the universal core
//! (`G = c = M_core = 1`, one sexagesimal tier per step), [`measure`] the
//! share of a sphere within an angle, [`event`] the certified crossing
//! solver, and [`coordinate`] the sexagesimal coordinate. Every random draw
//! comes from a [`Stream`] derived from the seed and a stable path of
//! labels, so the same seed gives the same universe in any process and on any
//! thread count.

pub mod certified;
pub mod cga;
pub mod codec;
pub mod coordinate;
pub mod dyadic;
pub mod event;
pub mod measure;
pub mod motion;
pub mod q;
pub mod rng;
pub mod rotor;
pub mod units;
pub mod watch;

pub use q::Q;
pub use rng::Stream;
