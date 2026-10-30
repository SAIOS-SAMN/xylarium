//! A people's history: turnings reported by the carry and the network, and
//! the figures it remembers them by (docs/civilizations.md, History). A
//! figure's role is read from the people at its turning, never from a table.

use crate::knowledge::Domain;
use xyl_core::dyadic::Dy;

/// Memory below this is gone: three e-folds of fading.
pub fn forgotten() -> Dy {
    Dy::int(-3).exp()
}

/// A turning of a people's history: the act that made it (`act`: exodus,
/// settlement, network, quarantine, biology, information or fold), its
/// title and time, named at the people's next carry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Turning {
    /// The act that made it.
    pub act: &'static str,
    /// What it is remembered for.
    pub title: String,
    /// When it happened (standard years).
    pub at_yr: Dy,
}

/// How a people holds a figure of its history, read from the shape of its
/// mind (`cosmic::Shape`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Held {
    /// A person among many minds.
    Person,
    /// The colony itself, thinking as one.
    Colony,
    /// A locus of a mind spread through its medium.
    Locus,
}

/// A figure a people remembers a turning by: the domain its effort led in
/// at the turning and that domain's share of its effort, how the people
/// holds it, and how well it is remembered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Eminent {
    /// A placeholder name drawn from the seed, until a mind of the people
    /// names it.
    pub name: String,
    /// The domain the people's effort led in at the turning.
    pub domain: Domain,
    /// That domain's share of the people's effort, in \[0, 1\].
    pub lead: Dy,
    /// How the people holds it.
    pub held: Held,
    /// What it is remembered for.
    pub title: String,
    /// The turning's time (standard years).
    pub born_yr: Dy,
    /// One of its people's generations after the turning (standard years).
    pub dies_yr: Dy,
    /// How well it is remembered, in \[0, 1\]: fades at the people's
    /// forgetting share a year; below [`forgotten`] it is dropped.
    pub memory: Dy,
    /// Whether its span has passed.
    pub dead: bool,
}
