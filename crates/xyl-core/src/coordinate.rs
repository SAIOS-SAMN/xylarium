//! The coordinate (docs/network.md): TUniversal's lanes {S, G, T, H}, and
//! on each lane an exact rational, the body's own reading on that lane
//! (S its phase on its anchor's ring, G its holding as a share of its
//! anchor's, T its own beat's ring and phase; H not yet modelled), written
//! as the sector of a 60^6-mark protractor it lies in: six sexagesimal places,
//! `XX:XX:XX:XX:XX:XX`, place 1 the coarsest. A prefix of places is a whole
//! coarser sector. The places are decided by exact comparison - never a
//! rounding: the value stays exact, the places only say which sector holds
//! it. No index, no hash: a coordinate is read off the body's content.

use crate::Q;

/// Marks on a lane's protractor: 60^6.
pub const MARKS: i64 = 46_656_000_000;
/// Places of a coordinate.
pub const PLACES: usize = 6;

/// The four lanes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lane {
    /// The body's phase on its anchor's ring, in turns.
    S,
    /// The body's holding as a share of its anchor's.
    G,
    /// The body's own beat: its ring and phase.
    T,
    /// The fourth lane; no reading is modelled on it yet.
    H,
}

/// A lane's reading: an exact share of the turn, in [0, 1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    /// The lane the reading is taken on.
    pub lane: Lane,
    /// The exact share of the turn, in \[0, 1).
    pub share: Q,
}

impl Reading {
    /// A reading of `value` turns on `lane` (its whole turns set aside).
    pub fn of(lane: Lane, value: &Q) -> Reading {
        Reading {
            lane,
            share: value.frac(),
        }
    }

    /// The sector of the protractor it lies in, decided exactly: the mark
    /// `m` with `m / 60^6 <= share < (m + 1) / 60^6`.
    pub fn mark(&self) -> Option<i64> {
        let m = (&self.share * &Q::int(MARKS)).floor_i128()?;
        i64::try_from(m).ok()
    }

    /// Its six places, coarsest first.
    pub fn places(&self) -> Option<[u8; PLACES]> {
        let mut c = self.mark()?;
        let mut out = [0u8; PLACES];
        for p in (0..PLACES).rev() {
            out[p] = u8::try_from(c % 60).ok()?;
            c /= 60;
        }
        Some(out)
    }

    /// Written `XX:XX:XX:XX:XX:XX`.
    pub fn written(&self) -> Option<String> {
        Some(
            self.places()?
                .iter()
                .map(|d| format!("{d:02}"))
                .collect::<Vec<_>>()
                .join(":"),
        )
    }

    /// Whether it lies in the sector a prefix of `depth` places of `other`
    /// names (a coarser sector holds every finer one it contains).
    pub fn within(&self, other: &Reading, depth: usize) -> Option<bool> {
        let (a, b) = (self.places()?, other.places()?);
        Some(self.lane == other.lane && a[..depth.min(PLACES)] == b[..depth.min(PLACES)])
    }
}

/// The turn of a direction (the flows' phase reading): the sector of the
/// 60^6 protractor the in-plane direction (x, y) lies in, decided exactly
/// with no arctangent and no rounding. A rational direction's turn is
/// irrational (Niven: its tangent is rational only at whole eighths), so its
/// share here is its sector's mark, the value itself never needed. The quarter-turn is read from the
/// signs, the eighth from |x| against |y|, both exact; inside, a turn lies
/// at or past mark m where the cross product of the mark's direction with
/// (x, y), cos(m) y - sin(m) x, is not negative - read on certified
/// enclosures of cos and sin (one crossing over the marks). None for the
/// zero direction, or where the enclosure cannot decide (a direction within
/// 2^-60 of a mark, the fixed-point grid the enclosures are read on,
/// `certified::FIXED`).
pub fn direction(lane: Lane, x: &Q, y: &Q) -> Option<Reading> {
    if x.is_zero() && y.is_zero() {
        return None;
    }
    let quarter = MARKS / 4;
    // the quarter-turn, exactly, and the direction turned back into [0, 1/4)
    let (q, u, v) = match (x.signum(), y.signum()) {
        (1, s) if s >= 0 => (0, x.clone(), y.clone()),
        (s, 1) if s <= 0 => (1, y.clone(), -x.clone()),
        (-1, s) if s <= 0 => (2, -x.clone(), -y.clone()),
        _ => (3, -y.clone(), x.clone()),
    };
    // at or past mark m of the quarter (m in [0, quarter))
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
        share: Q::new(lo + q * quarter, MARKS)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reading_keeps_its_value_and_names_its_sector_exactly() {
        let r = Reading::of(Lane::S, &Q::new(7, 3).unwrap());
        // the whole turns set aside, the share exact
        assert_eq!(r.share, Q::new(1, 3).unwrap());
        // 1/3 of 60^6 marks is exactly 15_552_000_000: the sector 20:00:00:00:00:00
        assert_eq!(r.written().unwrap(), "20:00:00:00:00:00");
        // a value just below a mark lies in the sector before it, never rounded up
        let below = Reading::of(
            Lane::S,
            &(&Q::new(1, 3).unwrap() - &Q::new(1, 10i64.pow(15)).unwrap()),
        );
        assert_eq!(below.written().unwrap(), "19:59:59:59:59:59");
        assert_eq!(below.within(&r, 1), Some(false));
        assert_eq!(
            below.within(&Reading::of(Lane::S, &Q::new(1, 4).unwrap()), 0),
            Some(true)
        );
    }

    #[test]
    fn a_direction_reads_its_sector_exactly() {
        let q = |n, d| Q::new(n, d).unwrap();
        // the axes and the diagonal are exact marks
        assert_eq!(
            direction(Lane::S, &q(3, 1), &q(0, 1))
                .unwrap()
                .written()
                .unwrap(),
            "00:00:00:00:00:00"
        );
        assert_eq!(
            direction(Lane::S, &q(0, 1), &q(2, 1))
                .unwrap()
                .written()
                .unwrap(),
            "15:00:00:00:00:00"
        );
        assert_eq!(
            direction(Lane::S, &q(1, 1), &q(1, 1))
                .unwrap()
                .written()
                .unwrap(),
            "07:30:00:00:00:00"
        );
        assert_eq!(
            direction(Lane::S, &q(-1, 1), &q(-1, 1))
                .unwrap()
                .written()
                .unwrap(),
            "37:30:00:00:00:00"
        );
        // a direction just past the diagonal lies in the sector after it, never rounded back
        let past = direction(Lane::S, &q(1_000_000, 1), &q(1_000_001, 1)).unwrap();
        assert!(past.mark().unwrap() >= MARKS / 8);
        assert!(direction(Lane::S, &q(0, 1), &q(0, 1)).is_none());
    }
}
