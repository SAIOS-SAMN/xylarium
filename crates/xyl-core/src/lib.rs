//! Exact numerics for Xylarium: the number types, geometry, motion and
//! deterministic randomness every other crate builds on.
//!
//! Every held quantity is exact: rationals ([`Q`]), certified enclosures
//! ([`certified::Encl`]) for roots and transcendentals, and dyadic rationals
//! ([`dyadic::Dy`]) for bounded intermediate readings. The modules:
//!
//! - [`q`]: the rational type; [`certified`]: enclosures; [`dyadic`]: `Dy`;
//! - [`cga`]: the conformal geometric algebra `Cl(n+1, 1)` over `Q`;
//!   [`rotor`]: rational half-angles and quaternion rotors;
//! - [`units`]: the unit ladder anchored at the universal core
//!   (`G = c = M_core = 1`, tiers a power of sixty of `r_g` apart);
//! - [`shares`]: the deficit share, the dilation and the flow law;
//! - [`measure`]: the share of an n-sphere within an angle of an axis;
//! - [`motion`]: closed-form orbits of exponential modes; [`event`]: the
//!   certified crossing solver; [`coordinate`]: the sexagesimal coordinate;
//! - [`codec`]: the snapshot codec; [`watch`]: the process monitor;
//! - [`rng`]: deterministic random streams.
//!
//! Every random draw comes from a [`Stream`] derived from the seed and a
//! stable path of labels, so the same seed gives the same universe in any
//! process and on any thread count.

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
pub mod shares;
pub mod units;
pub mod watch;

pub use q::Q;
pub use rng::Stream;
