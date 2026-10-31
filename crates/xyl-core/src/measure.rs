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
//! in turns within which the sphere holds share `s` (the ladder's rung, then
//! up to four Newton steps on the slope `sin^(n-1)`).
//!
//! Invariants: the integral is evaluated once per dimension on 2^14 rungs of
//! the half turn (Simpson's rule on each rung), then cached for the process; it
//! is computed from the measure, not a table of chosen values. Results are
//! `Dy`, so they are deterministic across machines but not certified.
//!
//! In high dimension the measure crowds toward the equator: on S^360 almost
//! all of it lies within about 3 degrees of 90.

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
    rad.sin()
        .abs()
        .powi(i32::try_from(n.max(2) - 1).unwrap_or(i32::MAX))
}

/// The measure between two angles (radians), by Simpson's rule on one rung.
fn rung(n: u32, lo: Dy, hi: Dy) -> Dy {
    let mid = (lo + hi) * Dy::ratio(1, 2);
    (hi - lo) * (across(n, lo) + Dy::int(4) * across(n, mid) + across(n, hi)) / Dy::int(6)
}

/// The ladder of S^n: the measure held below each rung of the half turn,
/// the last the whole.
fn ladder(n: u32) -> Arc<Vec<Dy>> {
    static L: OnceLock<Mutex<BTreeMap<u32, Arc<Vec<Dy>>>>> = OnceLock::new();
    let map = L.get_or_init(Default::default);
    if let Some(l) = map.lock().ok().and_then(|m| m.get(&n).cloned()) {
        return l;
    }
    let step = Dy::PI / Dy::int(RUNGS as i64);
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
    let half = Dy::ratio(1, 2);
    let t = theta.clamp(Dy::ZERO, half);
    if n == 1 {
        return t * Dy::int(2);
    }
    let l = ladder(n);
    let whole = l[RUNGS];
    let rad = t * Dy::int(2) * Dy::PI;
    let step = Dy::PI / Dy::int(RUNGS as i64);
    let i = usize::try_from((rad / step).trunc_i64())
        .unwrap_or(0)
        .min(RUNGS - 1);
    let lo = step * Dy::int(i as i64);
    (l[i] + rung(n, lo, rad)) / whole
}

/// The angle (turns, 0 to 1/2) within which S^n holds `s` of itself: the
/// ladder's rung that holds it, then Newton's steps on the measure's own
/// slope, sin^(n - 1) over the whole.
pub fn angle_holding(n: u32, s: Dy) -> Dy {
    let s = s.clamp(Dy::ZERO, Dy::ONE);
    let l = ladder(n);
    let whole = l[RUNGS];
    let want = s * whole;
    let i = l.partition_point(|x| *x < want).clamp(1, RUNGS) - 1;
    let step = Dy::PI / Dy::int(RUNGS as i64);
    let (lo, hi) = (step * Dy::int(i as i64), step * Dy::int(i as i64 + 1));
    let gap = l[i + 1] - l[i];
    let mut rad = if gap.signum() > 0 {
        lo + step * (want - l[i]) / gap
    } else {
        lo
    };
    for _ in 0..4 {
        let slope = across(n, rad);
        if slope.is_zero() {
            break;
        }
        rad = (rad - (l[i] + rung(n, lo, rad) - want) / slope).clamp(lo, hi);
    }
    rad / (Dy::int(2) * Dy::PI)
}
