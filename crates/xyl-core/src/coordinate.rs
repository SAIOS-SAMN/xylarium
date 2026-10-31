//! The coordinate (docs/network.md): a body's exact reading on one of the four
//! lanes {S, G, T, H} of TUniversal, written in degrees, minutes, seconds and
//! three finer sexagesimal places of arc: the sector of a protractor of
//! `360 x 60^5` marks that holds it, with the whole turns kept.
//!
//! Inputs: an exact rational value in turns ([`Q`]) on a [`Lane`]: S, the
//! body's phase on its anchor's ring; G, its holding as a share of its
//! anchor's; T, the body's beat, whole rings plus a phase; H, not yet
//! modelled. `direction` uses the enclosed cosine and sine of `certified`.
//!
//! Outputs: [`Reading`] (`of` keeps the whole turns, `rings`, and the share of
//! the turn, `share`), with `mark` (the sector index `m`,
//! `m / MARKS <= share < (m + 1) / MARKS`), `places` (degrees 0-359, then
//! minutes, seconds and three finer places 0-59), `written`
//! (`DDD:MM:SS:XX:XX:XX`, prefixed `N/` with the whole turns on the T lane),
//! `value` (rings plus share) and `within` (whether two readings share their
//! whole turns and a prefix of places on the same lane); and `direction`, the
//! sector of the in-plane direction (x, y), read with no arctangent.
//!
//! Invariants: the value stays exact; places are decided by exact comparison,
//! never by rounding, and a prefix of places names a whole coarser sector.
//! A coordinate is computed from the body's values, with no index or hash.
//! `direction` returns `None` for the zero direction and where the 2^-60
//! enclosures cannot decide the sector.
//!
//! Terms: TUniversal (docs/glossary.md).

use crate::Q;

/// Degrees in a turn.
pub const DEGREES: i64 = 360;
/// Marks on a lane's protractor: 360 degrees of 60^5 marks, the fifth
/// sexagesimal place of arc.
pub const MARKS: i64 = DEGREES * 60 * 60 * 60 * 60 * 60;
/// Places of a coordinate: degrees, minutes, seconds and three finer places.
pub const PLACES: usize = 6;

/// The four lanes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lane {
    /// The body's phase on its anchor's ring, in turns.
    S,
    /// The body's holding as a share of its anchor's.
    G,
    /// The body's beat: the ring and phase of its pulse.
    T,
    /// The fourth lane; no reading is modelled on it yet.
    H,
}

/// A lane's reading: whole turns and an exact share of the turn, in [0, 1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    /// The lane the reading is taken on.
    pub lane: Lane,
    /// The whole turns (on the T lane, the rings).
    pub rings: Q,
    /// The exact share of the turn, in \[0, 1).
    pub share: Q,
}

impl Reading {
    /// A reading of `value` turns on `lane`: its whole turns and its share.
    pub fn of(lane: Lane, value: &Q) -> Reading {
        let share = value.frac();
        Reading {
            lane,
            rings: value - &share,
            share,
        }
    }

    /// The value read: the whole turns plus the share.
    pub fn value(&self) -> Q {
        &self.rings + &self.share
    }

    /// The sector of the protractor the reading lies in, decided exactly: the
    /// mark `m` with `m / MARKS <= share < (m + 1) / MARKS`. Returns `None`
    /// only if the mark does not fit an `i64`.
    pub fn mark(&self) -> Option<i64> {
        let m = (&self.share * &Q::int(MARKS)).floor_i128()?;
        i64::try_from(m).ok()
    }

    /// The reading's six places, coarsest first: degrees (0-359), then
    /// minutes, seconds and three finer places (0-59).
    pub fn places(&self) -> Option<[u16; PLACES]> {
        let mut c = self.mark()?;
        let mut out = [0u16; PLACES];
        for p in (1..PLACES).rev() {
            out[p] = u16::try_from(c % 60).ok()?;
            c /= 60;
        }
        out[0] = u16::try_from(c).ok()?;
        Some(out)
    }

    /// Written `DDD:MM:SS:XX:XX:XX`; on the T lane prefixed with the rings,
    /// `N/DDD:MM:SS:XX:XX:XX`.
    pub fn written(&self) -> Option<String> {
        let p = self.places()?;
        let arc = std::iter::once(format!("{:03}", p[0]))
            .chain(p[1..].iter().map(|d| format!("{d:02}")))
            .collect::<Vec<_>>()
            .join(":");
        Some(if self.lane == Lane::T {
            format!("{}/{arc}", self.rings)
        } else {
            arc
        })
    }

    /// Whether the reading lies on the lane of `other`, in the same whole
    /// turns and in the sector named by the first `depth` places of `other`
    /// (a coarser sector holds every finer one it contains).
    pub fn within(&self, other: &Reading, depth: usize) -> Option<bool> {
        let (a, b) = (self.places()?, other.places()?);
        Some(
            self.lane == other.lane
                && self.rings == other.rings
                && a[..depth.min(PLACES)] == b[..depth.min(PLACES)],
        )
    }
}

/// The turn of a direction (the flows' phase reading): the sector of the
/// protractor the in-plane direction (x, y) lies in, decided exactly
/// with no arctangent and no rounding. A rational direction's turn is
/// irrational except at whole eighths (Niven: a rational share of the turn
/// has a rational tangent only at whole eighths), so the share returned is
/// the sector's mark and the exact value is never needed. The quarter-turn is
/// read from the signs and the eighth from |x| against |y|, both exactly.
/// Within the quarter, bisection finds the last mark m at or before the
/// direction: the direction lies at or past m where the cross product of the
/// mark's direction with (x, y), `cos(m) y - sin(m) x`, is not negative,
/// evaluated on certified enclosures of cos and sin (the sign changes once
/// across the quarter). Returns `None` for the zero direction, or where the
/// enclosure cannot decide the sign (a direction within about 2^-60 of a
/// mark, the fixed-point grid of `certified::FIXED`).
pub fn direction(lane: Lane, x: &Q, y: &Q) -> Option<Reading> {
    if x.is_zero() && y.is_zero() {
        return None;
    }
    let quarter = MARKS / 4;
    // The quarter-turn `q` from the signs, and (u, v), the direction turned
    // back by q quarter turns into [0, 1/4).
    let (q, u, v) = match (x.signum(), y.signum()) {
        (1, s) if s >= 0 => (0, x.clone(), y.clone()),
        (s, 1) if s <= 0 => (1, y.clone(), -x.clone()),
        (-1, s) if s <= 0 => (2, -x.clone(), -y.clone()),
        _ => (3, -y.clone(), x.clone()),
    };
    // Whether (u, v) lies at or past mark m of the quarter (m in [0, quarter)).
    // `None` when the enclosure of the cross product holds zero.
    let at_or_past = |m: i64| -> Option<bool> {
        if m == 0 {
            return Some(true);
        }
        if m == quarter / 2 {
            return Some(v >= u);
        }
        let t = crate::certified::Encl::exact(Q::new(m, MARKS)?);
        let (c, s_) = crate::certified::cos_sin_turns(&t);
        let cross = crate::certified::Encl::exact(v.clone())
            .mul(&c)
            .sub(&crate::certified::Encl::exact(u.clone()).mul(&s_));
        if cross.lo.signum() >= 0 {
            Some(true)
        } else if cross.hi.signum() < 0 {
            Some(false)
        } else {
            None
        }
    };
    let (mut lo, mut hi) = (0i64, quarter);
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if at_or_past(mid)? {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(Reading {
        lane,
        rings: Q::ZERO,
        share: Q::new(lo + q * quarter, MARKS)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reading_keeps_its_turns_and_names_its_sector_in_degrees() {
        let r = Reading::of(Lane::S, &Q::new(7, 3).unwrap());
        // The whole turns are kept and the share stays exact.
        assert_eq!(
            (r.rings.clone(), r.share.clone()),
            (Q::int(2), Q::new(1, 3).unwrap())
        );
        assert_eq!(r.value(), Q::new(7, 3).unwrap());
        // A third of a turn is 120 degrees exactly.
        assert_eq!(r.written().unwrap(), "120:00:00:00:00:00");
        // A value just below a mark lies in the sector before it; it is not
        // rounded up.
        let below = Reading::of(
            Lane::S,
            &(&Q::new(7, 3).unwrap() - &Q::new(1, 10i64.pow(15)).unwrap()),
        );
        assert_eq!(below.written().unwrap(), "119:59:59:59:59:59");
        assert_eq!(below.within(&r, 1), Some(false));
        // On the T lane the rings are written and compared.
        let t = Reading::of(Lane::T, &Q::new(9, 4).unwrap());
        assert_eq!(t.written().unwrap(), "2/090:00:00:00:00:00");
        assert_eq!(
            t.within(&Reading::of(Lane::T, &Q::new(5, 4).unwrap()), 1),
            Some(false)
        );
        assert_eq!(
            t.within(&Reading::of(Lane::T, &Q::new(91, 40).unwrap()), 0),
            Some(true)
        );
    }

    #[test]
    fn a_direction_reads_its_sector_exactly() {
        let q = |n, d| Q::new(n, d).unwrap();
        let w = |x: i64, y: i64| {
            direction(Lane::S, &q(x, 1), &q(y, 1))
                .unwrap()
                .written()
                .unwrap()
        };
        // The axes and the diagonals are exact marks.
        assert_eq!(w(3, 0), "000:00:00:00:00:00");
        assert_eq!(w(0, 2), "090:00:00:00:00:00");
        assert_eq!(w(1, 1), "045:00:00:00:00:00");
        assert_eq!(w(-1, -1), "225:00:00:00:00:00");
        // A direction just past the diagonal lies in the sector after it; it
        // is not rounded back.
        let past = direction(Lane::S, &q(1_000_000, 1), &q(1_000_001, 1)).unwrap();
        assert!(past.mark().unwrap() >= MARKS / 8);
        assert!(direction(Lane::S, &q(0, 1), &q(0, 1)).is_none());
    }
}
