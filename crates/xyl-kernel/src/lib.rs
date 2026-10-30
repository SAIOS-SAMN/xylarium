//! The xylarium kernel: a universe in play and the peoples that live in it.
//!
//! Layout: [`universe`] (the event engine, advancing from event to event on
//! the bell's count), [`cosmic`] (the galaxy, its deep time, its peoples and
//! annals), [`civ`] (housing, numbers, history and settlers, carried
//! together), [`knowledge`] and [`growth`] (what peoples know and what their
//! bodies need), [`field`] (the bells' pulses at the stars peoples live by),
//! [`seats`], [`percept`] and [`lexicon`] (the seat surface for external
//! agents), [`propulsion`] and [`transit`] (drives, ley lines, realms,
//! stewards, quarantine and toll quotes), [`chronometry`] (the keepers'
//! clocks), [`sync`] (step readings), and [`checkpoint`] (state the laws do
//! not regenerate).

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
