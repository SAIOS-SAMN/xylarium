//! The measure of a shape: the share of an n-sphere's volume within an angle
//! theta of an axis, and its inverse.
//!
//! ```text
//! share_n(theta) = int_0^theta sin^(n-1) / int_0^pi sin^(n-1)
//! ```
//!
//! The dimension `n` sets the curve: one sine for each of the sphere's n - 1
//! directions across its axis, with no squared cosine, half-angle tangent or
//! root.
//!
//! Inputs: a dimension `n` (`UNIVERSE_N` = 360 for the universe's sphere,
//! `BODY_N` = 60 for a body's) and an angle in turns or a share, as `Dy`.
//!
//! Outputs: `share_turns(n, theta)`, the share within `theta` turns (clamped
//! to 0..1/2; for `n` = 1 the arc `2 theta`); `angle_holding(n, s)`, the angle
//! in turns within which the sphere holds share `s` (linear interpolation on
//! the ladder's rung, then up to four Newton steps on the slope `sin^(n-1)`).
//!
//! Invariants: the integral is evaluated once per dimension on 2^14 rungs of
//! the half turn (Simpson's rule on each rung), then cached for the process; it
//! is computed from the measure, not a table of chosen values. Results are
//! `Dy`, so they are deterministic across machines but not certified.
//!
//! In high dimension the measure concentrates at the equator: on S^360 the
//! angle from the axis has a spread of about 1/sqrt(359) rad, about 3 degrees,
//! about 90 degrees.

use crate::dyadic::Dy;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

/// The dimension of the universe's sphere, `S^360`.
pub const UNIVERSE_N: u32 = 360;
/// The dimension of a body's sphere, `S^60`.
pub const BODY_N: u32 = 60;

/// Rungs on the half turn.
const RUNGS: usize = 1 << 14;

/// One sine for each direction across the axis: sin^(n - 1) at `rad`.
fn across(n: u32, rad: Dy) -> Dy {
    rad.sin().abs().powi(n.max(2) - 1)
}

/// The measure between two angles (radians), by Simpson's rule on one rung.
fn rung(n: u32, lo: Dy, hi: Dy) -> Dy {
    let mid = (lo + hi) * crate::dyadic!(1, 2);
    ((hi - lo) * (across(n, lo) + Dy::int(4) * across(n, mid) + across(n, hi)))
        .over(crate::nonzero!(6))
}

/// The ladder of S^n: the measure held below each rung of the half turn,
/// the last the whole.
fn ladder(n: u32) -> Arc<Vec<Dy>> {
    static L: OnceLock<Mutex<BTreeMap<u32, Arc<Vec<Dy>>>>> = OnceLock::new();
    let map = L.get_or_init(Default::default);
    if let Some(l) = map.lock().ok().and_then(|m| m.get(&n).cloned()) {
        return l;
    }
    let step = Dy::pi().over(crate::nonzero!(RUNGS as i64));
    let mut held = Vec::with_capacity(RUNGS + 1);
    let mut sum = Dy::ZERO;
    held.push(sum);
    for i in 0..RUNGS {
        let lo = step * Dy::int(i as i64);
        sum += rung(n, lo, lo + step);
        held.push(sum);
    }
    let l = Arc::new(held);
    if let Ok(mut m) = map.lock() {
        m.insert(n, l.clone());
    }
    l
}

/// The share of S^n within `theta` turns of its axis (0 to 1/2), read from
/// the angle: for the circle (n = 1) the arc within theta either way,
/// 2 theta of the turn; for n at least 2 the measure's integral.
pub fn share_turns(n: u32, theta: Dy) -> Dy {
    let half = crate::dyadic!(1, 2);
    let t = theta.clamp(Dy::ZERO, half);
    if n == 1 {
        return t * Dy::int(2);
    }
    let l = ladder(n);
    let whole = l[RUNGS];
    let rad = t * Dy::int(2) * Dy::pi();
    let step = Dy::pi().over(crate::nonzero!(RUNGS as i64));
    // The step is pi over the rungs and the whole is the sphere's measure,
    // both positive (`tests::ladder_whole_and_step_are_positive`), so neither
    // division takes its zero branch.
    let rungs = rad.div(&step).map_or(Dy::ZERO, |r| r);
    let i = usize::try_from(rungs.trunc_i64())
        .unwrap_or(0)
        .min(RUNGS - 1);
    let lo = step * Dy::int(i as i64);
    (l[i] + rung(n, lo, rad))
        .div(&whole)
        .map_or(Dy::ZERO, |s| s)
}

/// The angle (turns, 0 to 1/2) within which S^n holds the share `s` of its
/// measure: linear interpolation on the ladder's rung that contains it, then
/// up to four Newton steps on the slope of the measure, sin^(n - 1).
pub fn angle_holding(n: u32, s: Dy) -> Dy {
    let s = s.clamp(Dy::ZERO, Dy::ONE);
    let l = ladder(n);
    let whole = l[RUNGS];
    let want = s * whole;
    let i = l.partition_point(|x| *x < want).clamp(1, RUNGS) - 1;
    let step = Dy::pi().over(crate::nonzero!(RUNGS as i64));
    let (lo, hi) = (step * Dy::int(i as i64), step * Dy::int(i as i64 + 1));
    let gap = l[i + 1] - l[i];
    // An empty rung (no gap) holds the angle at its foot.
    let mut rad = match (step * (want - l[i])).div(&gap) {
        Some(f) if gap.signum() > 0 => lo + f,
        _ => lo,
    };
    for _ in 0..4 {
        // A flat slope takes no Newton step.
        let Some(dr) = (l[i] + rung(n, lo, rad) - want).div(&across(n, rad)) else {
            break;
        };
        rad = (rad - dr).clamp(lo, hi);
    }
    // A turn is two pi, positive.
    rad.div(&(Dy::int(2) * Dy::pi())).map_or(Dy::ZERO, |t| t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ladder's whole measure and its step are positive for every
    /// sphere the laws read, so `share_turns` never takes a zero branch.
    #[test]
    fn ladder_whole_and_step_are_positive() {
        assert!(Dy::pi().over(crate::nonzero!(RUNGS as i64)).signum() > 0);
        for n in 1..=12 {
            assert!(ladder(n)[RUNGS].signum() > 0, "S^{n}");
        }
    }
}
