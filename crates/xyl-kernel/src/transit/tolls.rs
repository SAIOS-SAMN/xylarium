//! The toll law: a lane's floor (unrecovered cost over last year's
//! traffic), its ceiling (what it saves the passer) and the toll between
//! them ([`quote`]); a throat's [`tithe`].

use crate::propulsion::{propellant_per_kg, Spec};
use xyl_core::dyadic::Dy;

/// How a passage's holder handles it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Handling {
    /// Only what it cost: the toll asks its floor.
    Open,
    /// By the toll law: half way from floor to ceiling. The default for a
    /// line a steward has not set.
    #[default]
    Tolled,
    /// Its builder's house and the peoples of its realm only.
    Guarded,
    /// Carries no one.
    Closed,
}

/// A held passage's books (J).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lane {
    /// What holding it open cost.
    pub cost_j: Dy,
    /// What it has collected.
    pub collected_j: Dy,
    /// The passages it carried last year.
    pub last_year: u64,
}

/// What passing costs, and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quote {
    /// Cost not yet paid back over last year's traffic, in J per passage ([`floor`]).
    pub floor: Dy,
    /// What the lane saves the passer, in J ([`ceiling`]).
    pub ceiling: Dy,
    /// The toll asked, in J, never above the ceiling.
    pub toll: Dy,
}

/// The floor: its cost not yet paid back, over last year's traffic.
pub fn floor(lane: &Lane) -> Dy {
    (lane.cost_j - lane.collected_j).max(Dy::ZERO)
        / Dy::int(i64::try_from(lane.last_year.max(1)).unwrap_or(i64::MAX))
}

/// The ceiling: what the lane saves the passer - the cheapest way round less
/// the lane, in hops at the passer's hop value; with no way round, all it
/// carries and a hop.
pub fn ceiling(around_hops: Option<Dy>, lane_hops: Dy, hop_value: Dy, cargo_value: Dy) -> Dy {
    match around_hops {
        None => cargo_value + hop_value,
        Some(a) => (a - lane_hops).max(Dy::ZERO) * hop_value,
    }
}

/// The toll: half way from floor to ceiling, never above the ceiling; an open
/// passage asks only its floor (and never more than it saves).
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

/// A throat's tithe: a passage's energy for every
/// lane's length it spans (time as distance), at least one.
pub fn tithe(passage: Dy, span: Dy, lane_len: Dy) -> Dy {
    let hops = if lane_len.signum() > 0 {
        span / lane_len
    } else {
        Dy::ONE
    };
    passage * hops.max(Dy::ONE)
}

/// What a hop costs a passer: the fuel its drive
/// burns carrying `payload_kg` (J).
pub fn hop_value(spec: &Spec, ratio: Dy, payload_kg: Dy) -> Dy {
    payload_kg * propellant_per_kg(ratio) * spec.energy_per_kg
}
