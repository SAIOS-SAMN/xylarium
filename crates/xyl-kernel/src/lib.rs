//! The Xylarium kernel: the game layer that carries a founded universe in
//! play, with the peoples that arise in it and the seats players hold. It
//! reads the founded galaxy, systems, worlds and forecast from `xyl_cosmos`
//! and exact numbers from `xyl_core`.
//!
//! Modules:
//! - [`universe`]: the event engine, advancing from event to event on the
//!   bell's count;
//! - [`cosmic`]: the galaxy in play, deep time, peoples and annals;
//! - [`civ`]: the carry of knowledge, housing, numbers, gauges, settlers
//!   and history; [`knowledge`] and [`growth`] hold its laws;
//! - [`field`]: the bells' pulses at the stars peoples live by, and
//!   radiation;
//! - [`seats`], [`percept`] and [`lexicon`]: the seat surface for players
//!   and external minds;
//! - [`propulsion`] and [`transit`]: drives, voyages, ley lines, realms,
//!   quarantine and toll quotes;
//! - [`chronometry`]: the keepers' clocks and their hashed chain;
//! - [`sync`]: step readings;
//! - [`checkpoint`]: the state the laws do not regenerate.
//!
//! Every state is a function of the seed and the recorded inputs: what the
//! laws give is regenerated on each start, and the checkpoint lays back the
//! rest.

pub mod checkpoint;
pub mod chronometry;
pub mod civ;
pub mod cosmic;
pub mod field;
pub mod growth;
pub mod knowledge;
pub mod lexicon;
pub mod percept;
pub mod propulsion;
pub mod seats;
pub mod sync;
pub mod transit;
pub mod universe;
