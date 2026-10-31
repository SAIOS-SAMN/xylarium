//! The toll law: a lane's floor (unrecovered cost over last year's
//! traffic), its ceiling (what the lane saves the passer) and the toll
//! between them ([`quote`]), and a throat's [`tithe`].

use crate::propulsion::{propellant_per_kg, Spec};
use xyl_core::dyadic::Dy;

/// The rule by which a passage's holder admits passers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Handling {
    /// The toll asks only the floor, capped at the ceiling.
    Open,
    /// The toll lies half way from floor to ceiling. This is the default for
    /// a line whose handling no steward has set.
    #[default]
    Tolled,
    /// The line carries only its builder and the peoples of its realm.
    Guarded,
    /// The line carries no one.
    Closed,
}

/// The books of a held passage, in J.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lane {
    /// The energy that holding the passage open cost.
    pub cost_j: Dy,
    /// The energy collected in tolls.
    pub collected_j: Dy,
    /// The number of passages carried last year.
    pub last_year: u64,
}

/// A toll quote with the floor and ceiling it was derived from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quote {
    /// The cost not yet paid back over last year's traffic, in J per
    /// passage ([`floor`]).
    pub floor: Dy,
    /// The energy the lane saves the passer, in J ([`ceiling`]).
    pub ceiling: Dy,
    /// The toll asked, in J, which never exceeds the ceiling.
    pub toll: Dy,
}

/// Returns the floor (J per passage): the lane's cost not yet paid back,
/// divided by last year's passages (at least one).
pub fn floor(lane: &Lane) -> Dy {
    (lane.cost_j - lane.collected_j).max(Dy::ZERO)
        / Dy::int(i64::try_from(lane.last_year.max(1)).unwrap_or(i64::MAX))
}

/// Returns the ceiling (J): what the lane saves the passer. With a way round
/// of `around_hops` hops it is `(around_hops - lane_hops) * hop_value`, at
/// least zero; with no way round it is the cargo's value plus one hop.
pub fn ceiling(around_hops: Option<Dy>, lane_hops: Dy, hop_value: Dy, cargo_value: Dy) -> Dy {
    match around_hops {
        None => cargo_value + hop_value,
        Some(a) => (a - lane_hops).max(Dy::ZERO) * hop_value,
    }
}

/// Returns the quote for `lane`. The toll lies half way from floor to
/// ceiling and never above the ceiling; an open passage asks only its floor,
/// and never more than the ceiling.
pub fn quote(
    lane: &Lane,
    around_hops: Option<Dy>,
    lane_hops: Dy,
    hop_value: Dy,
    cargo_value: Dy,
    handling: Handling,
) -> Quote {
    let f = floor(lane);
    let c = ceiling(around_hops, lane_hops, hop_value, cargo_value);
    let toll = match handling {
        Handling::Open => f.min(c),
        _ => c.min(f + (c - f).max(Dy::ZERO) / Dy::int(2)),
    };
    Quote {
        floor: f,
        ceiling: c,
        toll,
    }
}

/// Returns a throat's tithe: the passage energy `passage` times the number
/// of lane lengths `span / lane_len` it spans (time as distance), with at
/// least one lane length counted.
pub fn tithe(passage: Dy, span: Dy, lane_len: Dy) -> Dy {
    let hops = if lane_len.signum() > 0 {
        span / lane_len
    } else {
        Dy::ONE
    };
    passage * hops.max(Dy::ONE)
}

/// Returns what a hop costs a passer (J): the energy of the fuel the drive
/// burns carrying `payload_kg`, `payload_kg * (R - 1) * energy_per_kg`.
pub fn hop_value(spec: &Spec, ratio: Dy, payload_kg: Dy) -> Dy {
    payload_kg * propellant_per_kg(ratio) * spec.energy_per_kg
}
