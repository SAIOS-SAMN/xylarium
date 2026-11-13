//! The coordinate (FORMS-COORDINATE v4, Mycelium proposal 78.173,
//! reconstructed from its section 1): the coordinate a kernel addresses its
//! bodies by, `origin / h : S:G:T:H : K`.
//!
//! - `keys`: axes (`S1`, the plane units `+k`, `-k`), keys and generators,
//!   the turning planes H stands on, vectors (a lane's signed components,
//!   keyed, never summed), the lanes S, G, T, H and the ruled openings.
//! - `body`: the origin (genesis hash, epoch, enclosures written out), the
//!   coordinate, its writing and canonical order, `within` on the caller's
//!   intervals, and the bodies held in order with a range on one component
//!   found by two bisections and a filter within it.
//! - `motion`: the advance (composes; no backward advance), an event's bell
//!   in first degree, merger with offsets and fission its inverse, and the
//!   re-origin across a throat (LAW-PASSAGE v3).
//! - Display only: the sexagesimal sector at a depth (`Sector`) and the T
//!   lane's beat (`beat`); nothing is compared, stored or decided by them.
//!
//! Invariants: every value is exact; origins are never compared across;
//! every refusal is named ([`CoordinateRefusal`]). The fixtures of 78.173
//! section 5 are reproduced in the tests from these definitions alone.
//!
//! Terms: TUniversal, coordinate (docs/glossary.md).

mod body;
#[cfg(test)]
mod fixtures;
mod keys;
mod motion;

pub use body::{written_value, Bodies, Bounds, Coordinate, Interval, Local, Origin, Range, Sides};
pub use keys::{Axis, Key, Lane, Lanes, Openings, Plane, Vector};
pub use motion::{Crossing, EventBell, Merger, PolePlane, Poles};

use crate::Q;

/// Degrees in a turn: the unit a turn is written in (seconds are degrees).
pub const DEGREES: i64 = 360;

/// [`DEGREES`] as a nonzero divisor.
const DEGREES_NONZERO: std::num::NonZeroI64 = crate::nonzero!(360);

/// A bell as an exact rational.
fn bell_q(h: u64) -> Q {
    Q::from_big(num_rational::BigRational::from_integer(
        num_bigint::BigInt::from(h),
    ))
}

/// The base of the places.
const BASE: i64 = 60;

/// Why a coordinate or a reading is refused, by name, with what caused it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoordinateRefusal {
    /// A sector index past a machine word at this depth.
    Unbounded {
        /// The depth asked.
        depth: u32,
    },
    /// An axis that cannot be written: a letter not an ASCII capital, or
    /// an index of 0.
    AxisUnwritable {
        /// The letter (or the unit's sign).
        letter: char,
        /// The index.
        index: u32,
    },
    /// A generator of one axis twice.
    NotPlane {
        /// The axis.
        axis: String,
    },
    /// A key held twice in one lane, or a plane named in both orientations.
    KeyRepeated {
        /// The key.
        key: String,
    },
    /// A base lane missing from a lane set.
    LaneMissing {
        /// The lane.
        lane: Lane,
    },
    /// A lane held twice.
    LaneRepeated {
        /// The lane.
        lane: Lane,
    },
    /// A lane no ruled opening admits.
    LaneNotOpened {
        /// Its name.
        name: String,
        /// Its law version.
        version: u32,
    },
    /// A key of one axis on H, which stands only on generators.
    NotGenerator {
        /// The key.
        key: String,
    },
    /// A plane that holds no turning, on H.
    NotTurningPlane {
        /// The key.
        key: String,
        /// The plane it names.
        plane: Plane,
    },
    /// Two origins, never compared: the one held and the one asked.
    OriginsDiffer {
        /// The origin held, written.
        held: String,
        /// The origin asked, written.
        asked: String,
    },
    /// A lane keyed otherwise: its keys held and asked.
    ShapesDiffer {
        /// The lane.
        lane: Lane,
        /// The keys held.
        held: Vec<String>,
        /// The keys asked.
        asked: Vec<String>,
    },
    /// A component the body does not hold.
    NoComponent {
        /// The lane.
        lane: Lane,
        /// The key.
        key: String,
    },
    /// An advance to an earlier bell.
    Backward {
        /// The bell held.
        held: u64,
        /// The bell asked.
        asked: u64,
    },
    /// A merger across bells.
    BellsDiffer {
        /// The bell held.
        held: u64,
        /// The bell asked.
        asked: u64,
    },
    /// Holdings whose sum is zero.
    NoHolding,
    /// Offsets whose holding-weighted sum is not zero: the sum, written.
    Unbalanced {
        /// The sum, written.
        sum: String,
    },
    /// A throat state with no whole, `H + R = 0`: its span.
    NoWhole {
        /// The span `H - R`.
        span: String,
    },
    /// A body whose S is not the throat's span.
    SpanDiffers {
        /// The body's S.
        held: String,
        /// The span read.
        read: String,
    },
    /// A component's whole turns past a machine word.
    TurnsUnbounded {
        /// The key.
        key: String,
    },
    /// A pole that is not one generator at coefficient one: its terms,
    /// each blade mask and coefficient written.
    NotUnit {
        /// The terms held.
        terms: Vec<(usize, String)>,
    },
    /// A pole of the other square: `e+` named negative or `e-` positive.
    WrongSign {
        /// The generator's mask.
        generator: usize,
    },
    /// A passage state off its pole plane: the blades held off the plane's
    /// two units (none for the zero, which holds no state).
    NotOnPolePlane {
        /// The blade masks off the plane.
        blades: Vec<usize>,
    },
    /// A riding state holding both parities: the degrees it holds.
    MixedParity {
        /// The degrees, ascending.
        degrees: Vec<u32>,
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

/// A lane's reading for display: its exact value.
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

    /// FORMS-COORDINATE v4 fixture 2: the sexagesimal reading at stated depths
    /// gives the stated index, bounds and writing.
    #[test]
    fn sector_equals_law_coordinate_fixture_2() {
        let third_less = &crate::ratio!(1, 3) - &Q::from_i128(1, 10i128.pow(15)).unwrap();
        let rows: [(Q, u32, i128, Q, Q, &str); 8] = [
            (
                crate::ratio!(7, 3),
                0,
                2,
                crate::ratio!(2, 1),
                crate::ratio!(3, 1),
                "2",
            ),
            (
                crate::ratio!(7, 3),
                1,
                140,
                crate::ratio!(7, 3),
                crate::ratio!(47, 20),
                "2;20",
            ),
            (
                crate::ratio!(7, 3),
                2,
                8400,
                crate::ratio!(7, 3),
                crate::ratio!(8401, 3600),
                "2;20,00",
            ),
            (
                crate::ratio!(1, 7),
                3,
                30857,
                crate::ratio!(30857, 216000),
                crate::ratio!(5143, 36000),
                "0;08,34,17",
            ),
            (
                crate::ratio!(360, 7),
                2,
                185142,
                crate::ratio!(30857, 600),
                crate::ratio!(185143, 3600),
                "51;25,42",
            ),
            (
                crate::ratio!(-1, 3),
                1,
                -20,
                crate::ratio!(-1, 3),
                crate::ratio!(-19, 60),
                "-1;40",
            ),
            (
                crate::ratio!(-1, 7),
                1,
                -9,
                crate::ratio!(-3, 20),
                crate::ratio!(-2, 15),
                "-1;51",
            ),
            (
                third_less,
                6,
                15551999999,
                Q::from_i128(15551999999, 46656000000).unwrap(),
                crate::ratio!(1, 3),
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

    /// FORMS-COORDINATE v4 fixture 3: each sector lies in the one a depth
    /// coarser, and the coarser sector holding its lower bound is that one.
    #[test]
    fn sector_nests_in_coarser_sector() {
        for v in [
            crate::ratio!(1, 7),
            crate::ratio!(-22, 7),
            crate::ratio!(7, 3),
            crate::ratio!(0, 1),
            crate::ratio!(-1, 3),
        ] {
            for d in 0..5 {
                let (outer, inner) = (Sector::of(&v, d).unwrap(), Sector::of(&v, d + 1).unwrap());
                assert!(outer.lo().unwrap() <= inner.lo().unwrap());
                assert!(inner.hi().unwrap() <= outer.hi().unwrap());
                assert_eq!(Sector::of(&inner.lo().unwrap(), d).unwrap(), outer);
            }
        }
    }

    /// A sector index past a machine word is refused by name.
    #[test]
    fn sector_refuses_index_past_word() {
        assert_eq!(
            Sector::of(&crate::ratio!(1, 3), 40),
            Err(CoordinateRefusal::Unbounded { depth: 40 })
        );
    }

    /// The beat keeps the whole rings and writes the ring's phase in degrees.
    #[test]
    fn beat_writes_rings_and_phase_in_degrees() {
        assert_eq!(beat(&crate::ratio!(9, 4), 2).unwrap(), "2/90;00,00");
    }
}
