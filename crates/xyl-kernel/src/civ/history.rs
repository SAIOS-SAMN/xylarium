//! A people's history: the turnings reported for it and the figures it
//! remembers them by (docs/civilizations.md, History).
//!
//! Outputs: [`Turning`] (an act, its title and its time in standard years),
//! [`Eminent`] (a figure: placeholder name, leading domain and its share of
//! effort, how it is held, memory in [0, 1]), [`Held`] (person, colony or
//! locus, from the people's mind shape) and [`forgotten`], the memory
//! threshold exp(-3) below which a figure is dropped.
//!
//! This module holds only the types; `civ::Civilizations::turning` reports
//! turnings and `civ::Civilizations::remember` names figures, reading each
//! figure's role from the people at the carry that names it rather than from
//! a table.
//! Figures are held in the checkpoint.
//!
//! Terms: eminent, turning (docs/glossary.md).

use crate::knowledge::Domain;
use xyl_core::dyadic::Dy;

/// The memory threshold exp(-3), three e-folds of fading; a figure whose
/// memory falls below it is dropped.
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

/// A figure a people remembers a turning by: the domain the people's effort
/// led in when the figure was named and that domain's share of the effort,
/// how the people holds the figure, and how well it is remembered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Eminent {
    /// A placeholder name drawn from the seed, until a mind of the people
    /// names it.
    pub name: String,
    /// The domain the people's effort led in when the figure was named.
    pub domain: Domain,
    /// That domain's share of the people's effort, in \[0, 1\].
    pub lead: Dy,
    /// How the people holds the figure.
    pub held: Held,
    /// What it is remembered for.
    pub title: String,
    /// The turning's time (standard years).
    pub born_yr: Dy,
    /// One generation of the people after the turning (standard years).
    pub dies_yr: Dy,
    /// How well the figure is remembered, in \[0, 1\]: it fades by the
    /// people's forgetting share each year; below [`forgotten`] the figure
    /// is dropped.
    pub memory: Dy,
    /// Whether `dies_yr` had passed at the last carry.
    pub dead: bool,
}
