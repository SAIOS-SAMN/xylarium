//! The coordinate (docs/network.md; LAW-COORDINATE v1, Mycelium proposal
//! 78.173): a body's exact value on one of the four lanes of TUniversal,
//! read in sectors of base 60.
//!
//! Inputs: an exact rational value of either sign ([`Q`]) on a [`Lane`],
//! in the unit the caller names (turns, rings or degrees). The lanes carry
//! the network's meanings: S spatial (boost and dilation, the reach), G
//! gauge (change of kind), T temporal (null flux: the beat and the
//! passages), H holonomic (turnings between planes).
//!
//! Outputs: [`Reading`] (`of`, `sector`, `within`); [`Sector`], the
//! half-open interval `[m 60^-d, (m + 1) 60^-d)` that holds the value at
//! depth `d`, `m` fixed by exact comparison, with `lo`, `hi`, `places` and
//! `written` (`2;20`, `-1;40`); [`Side`], where a reading lies against
//! another's sector; [`beat`], the T lane written as whole rings and the
//! ring's phase in degrees; [`CoordinateRefusal`].
//!
//! Invariants: the value stays exact and is never held on a grid; sectors
//! nest (the sector at depth `d + 1` lies in the one at depth `d` that holds
//! the same value); `within` decides on the sector's exact bounds; readings
//! on different lanes are refused, never compared; places and their writing
//! are display only. A coordinate is computed from the body's values, with
//! no index or hash.
//!
//! Terms: TUniversal (docs/glossary.md).

use crate::Q;

/// Degrees in a turn: the unit a turn is written in (seconds are degrees).
pub const DEGREES: i64 = 360;

/// The base of the places.
const BASE: i64 = 60;

/// The four lanes, in the network's meanings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lane {
    /// Spatial: boost and dilation, the reach. A body's holding as a share
    /// of its anchor's is read here, by its dilation.
    S,
    /// Gauge: change of kind (planned: pairing and the transductions).
    G,
    /// Temporal: null flux, the beat and the passages; the universe's
    /// count in rings.
    T,
    /// Holonomic: turnings between planes. A body's phase on its anchor's
    /// ring is the turning of one plane.
    H,
}

/// Why a reading is refused, with the coordinates that caused it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoordinateRefusal {
    /// Two readings on different lanes: the lane held and the lane asked.
    LanesDiffer {
        /// The lane of the reading held.
        held: Lane,
        /// The lane of the reading asked against.
        asked: Lane,
    },
    /// A sector index past a machine word at this depth.
    Unbounded {
        /// The depth asked.
        depth: u32,
    },
}

/// Where a reading lies against a sector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Below the sector's lower bound.
    Below,
    /// In the sector, its lower bound included.
    Within,
    /// At or above the sector's upper bound.
    Above,
}

/// A lane's reading: its exact value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    /// The lane the reading is taken on.
    pub lane: Lane,
    /// The exact value, of either sign, in the caller's unit.
    pub value: Q,
}

/// The sector of depth `depth` and index `index`:
/// `[index 60^-depth, (index + 1) 60^-depth)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sector {
    /// The depth: how many places of base 60 the sector is read to.
    pub depth: u32,
    /// The index `m`, the integer the exact comparison fixes.
    pub index: i128,
}

/// `60^depth`, or `None` past a machine word.
fn scale(depth: u32) -> Option<i128> {
    i128::from(BASE).checked_pow(depth)
}

impl Sector {
    /// The sector at `depth` that holds `value`.
    pub fn of(value: &Q, depth: u32) -> Result<Sector, CoordinateRefusal> {
        let index = scale(depth)
            .and_then(|s| (value * &Q::from_i128(s, 1)?).floor_i128())
            .ok_or(CoordinateRefusal::Unbounded { depth })?;
        Ok(Sector { depth, index })
    }

    /// The lower bound, `index 60^-depth`, held in the sector.
    pub fn lo(&self) -> Option<Q> {
        Q::from_i128(self.index, scale(self.depth)?)
    }

    /// The upper bound, `(index + 1) 60^-depth`, not held.
    pub fn hi(&self) -> Option<Q> {
        Q::from_i128(self.index.checked_add(1)?, scale(self.depth)?)
    }

    /// The whole part at or below the lower bound, and the `depth` places in
    /// `0..60` that follow it, coarsest first. Display only.
    pub fn places(&self) -> Option<(i128, Vec<u8>)> {
        let s = scale(self.depth)?;
        let whole = self.index.div_euclid(s);
        let mut rest = self.index.rem_euclid(s);
        let mut places = vec![0u8; usize::try_from(self.depth).ok()?];
        for p in places.iter_mut().rev() {
            *p = u8::try_from(rest % i128::from(BASE)).ok()?;
            rest /= i128::from(BASE);
        }
        Some((whole, places))
    }

    /// Written as the whole part, `;`, and the places of two digits each,
    /// separated by `,` (`2;20,00`, `-1;40`; `2` at depth 0). Display only.
    pub fn written(&self) -> Option<String> {
        let (whole, places) = self.places()?;
        let places: Vec<String> = places.iter().map(|p| format!("{p:02}")).collect();
        Some(if places.is_empty() {
            format!("{whole}")
        } else {
            format!("{whole};{}", places.join(","))
        })
    }
}

impl Reading {
    /// A reading of `value` on `lane`.
    pub fn of(lane: Lane, value: &Q) -> Reading {
        Reading {
            lane,
            value: value.clone(),
        }
    }

    /// The sector at `depth` that holds the value.
    pub fn sector(&self, depth: u32) -> Result<Sector, CoordinateRefusal> {
        Sector::of(&self.value, depth)
    }

    /// Where this reading lies against the sector at `depth` that holds
    /// `other`'s value, decided on the sector's exact bounds. Readings on
    /// different lanes are refused.
    pub fn within(&self, other: &Reading, depth: u32) -> Result<Side, CoordinateRefusal> {
        if self.lane != other.lane {
            return Err(CoordinateRefusal::LanesDiffer {
                held: self.lane,
                asked: other.lane,
            });
        }
        let sector = other.sector(depth)?;
        let bounds = sector.lo().zip(sector.hi());
        let (lo, hi) = bounds.ok_or(CoordinateRefusal::Unbounded { depth })?;
        Ok(if self.value < lo {
            Side::Below
        } else if self.value < hi {
            Side::Within
        } else {
            Side::Above
        })
    }
}

/// The T lane written: the whole rings, `/`, and the ring's phase in
/// degrees to `depth` places (`2/17;03,24`). Display only.
pub fn beat(rings: &Q, depth: u32) -> Result<String, CoordinateRefusal> {
    let phase = rings.frac();
    let whole = rings - &phase;
    let arc = Sector::of(&(&phase * &Q::int(DEGREES)), depth)?
        .written()
        .ok_or(CoordinateRefusal::Unbounded { depth })?;
    Ok(format!("{whole}/{arc}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// LAW-COORDINATE v1 fixture 2: the sexagesimal reading at stated depths
    /// gives the stated index, bounds and writing.
    #[test]
    fn sector_equals_law_coordinate_fixture_2() {
        let third_less = &Q::ratio(1, 3) - &Q::from_i128(1, 10i128.pow(15)).unwrap();
        let rows: [(Q, u32, i128, Q, Q, &str); 8] = [
            (Q::ratio(7, 3), 0, 2, Q::ratio(2, 1), Q::ratio(3, 1), "2"),
            (
                Q::ratio(7, 3),
                1,
                140,
                Q::ratio(7, 3),
                Q::ratio(47, 20),
                "2;20",
            ),
            (
                Q::ratio(7, 3),
                2,
                8400,
                Q::ratio(7, 3),
                Q::ratio(8401, 3600),
                "2;20,00",
            ),
            (
                Q::ratio(1, 7),
                3,
                30857,
                Q::ratio(30857, 216000),
                Q::ratio(5143, 36000),
                "0;08,34,17",
            ),
            (
                Q::ratio(360, 7),
                2,
                185142,
                Q::ratio(30857, 600),
                Q::ratio(185143, 3600),
                "51;25,42",
            ),
            (
                Q::ratio(-1, 3),
                1,
                -20,
                Q::ratio(-1, 3),
                Q::ratio(-19, 60),
                "-1;40",
            ),
            (
                Q::ratio(-1, 7),
                1,
                -9,
                Q::ratio(-3, 20),
                Q::ratio(-2, 15),
                "-1;51",
            ),
            (
                third_less,
                6,
                15551999999,
                Q::from_i128(15551999999, 46656000000).unwrap(),
                Q::ratio(1, 3),
                "0;19,59,59,59,59,59",
            ),
        ];
        for (v, d, m, lo, hi, w) in rows {
            let s = Sector::of(&v, d).unwrap();
            assert_eq!(s.index, m, "index of {v} at {d}");
            assert_eq!((s.lo().unwrap(), s.hi().unwrap()), (lo, hi));
            assert_eq!(s.written().unwrap(), w);
        }
    }

    /// LAW-COORDINATE v1 fixture 3: each sector lies in the one a depth
    /// coarser, and the coarser sector holding its lower bound is that one.
    #[test]
    fn sector_nests_in_coarser_sector() {
        for v in [
            Q::ratio(1, 7),
            Q::ratio(-22, 7),
            Q::ratio(7, 3),
            Q::ratio(0, 1),
            Q::ratio(-1, 3),
        ] {
            for d in 0..5 {
                let (outer, inner) = (Sector::of(&v, d).unwrap(), Sector::of(&v, d + 1).unwrap());
                assert!(outer.lo().unwrap() <= inner.lo().unwrap());
                assert!(inner.hi().unwrap() <= outer.hi().unwrap());
                assert_eq!(Sector::of(&inner.lo().unwrap(), d).unwrap(), outer);
            }
        }
    }

    /// LAW-COORDINATE v1 fixture 4: `within` reads below, within and above
    /// a sector's exact bounds.
    #[test]
    fn within_equals_law_coordinate_fixture_4() {
        let other = Reading::of(Lane::S, &Q::ratio(1, 3));
        let fine = &Q::ratio(1, 3) + &Q::from_i128(1, 3 * 60i128.pow(6)).unwrap();
        let below = &Q::ratio(1, 3) - &Q::from_i128(1, 10i128.pow(15)).unwrap();
        for (s, d, side) in [
            (Q::ratio(1, 3), 1, Side::Within),
            (Q::ratio(7, 20), 1, Side::Above),
            (below, 1, Side::Below),
            (fine, 6, Side::Within),
            (Q::ratio(2, 5), 0, Side::Within),
            (Q::ratio(2, 5), 1, Side::Above),
        ] {
            assert_eq!(Reading::of(Lane::S, &s).within(&other, d), Ok(side));
        }
    }

    /// Readings on different lanes are refused by name, never compared.
    #[test]
    fn within_refuses_lanes_that_differ() {
        let (s, t) = (
            Reading::of(Lane::S, &Q::ratio(1, 3)),
            Reading::of(Lane::T, &Q::ratio(1, 3)),
        );
        assert_eq!(
            s.within(&t, 1),
            Err(CoordinateRefusal::LanesDiffer {
                held: Lane::S,
                asked: Lane::T
            })
        );
    }

    /// A sector index past a machine word is refused by name.
    #[test]
    fn sector_refuses_index_past_word() {
        assert_eq!(
            Sector::of(&Q::ratio(1, 3), 40),
            Err(CoordinateRefusal::Unbounded { depth: 40 })
        );
    }

    /// The beat keeps the whole rings and writes the ring's phase in degrees.
    #[test]
    fn beat_writes_rings_and_phase_in_degrees() {
        assert_eq!(beat(&Q::ratio(9, 4), 2).unwrap(), "2/90;00,00");
    }
}
