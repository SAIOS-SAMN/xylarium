//! Events: the time at which one continuous quantity crosses another, solved
//! by certified bisection rather than by stepping through time.
//!
//! Inputs: a bracket `[lo, hi]` of times and a target `width` (all [`Q`]), and
//! a function returning a `certified::Encl` of the difference at a time.
//!
//! Outputs: `crossing`, the enclosure of the zero of that function inside the
//! bracket, narrowed to `width`.
//!
//! Invariants: `crossing` returns `None` unless the function's enclosures
//! certainly differ in sign at the two ends. The bracket is halved on the side
//! where the midpoint's sign is certain; if the midpoint's enclosure holds zero
//! the current bracket is returned. The result is the event's time when the
//! function crosses once inside the bracket, which the caller ensures (a
//! monotone piece of its law). Between events nothing is computed: a reading
//! evaluates the orbits it needs at the time of the reading.

use crate::certified::Encl;
use crate::Q;

/// Where `f` crosses zero inside `[lo, hi]`, enclosed to `width`: `f`'s
/// values are enclosures, and it must certainly differ in sign at the two
/// ends (`None` otherwise). The bracket is halved on the side the
/// midpoint's sign certainly falls; where the midpoint's enclosure holds
/// zero the crossing is within the bracket already, and the bracket is
/// returned. The bracket encloses the event's time if `f` crosses zero once
/// inside it, for example on a monotone piece of the law, which the caller
/// establishes.
pub fn crossing(lo: &Q, hi: &Q, width: &Q, f: impl Fn(&Q) -> Encl) -> Option<Encl> {
    let sign = |e: &Encl| -> i8 {
        if e.lo.signum() > 0 {
            1
        } else if e.hi.signum() < 0 {
            -1
        } else {
            0
        }
    };
    let start = sign(&f(lo));
    let end = sign(&f(hi));
    if start == 0 || end == 0 || start == end {
        return None;
    }
    let half = Q::new(1, 2)?;
    let (mut a, mut b) = (lo.clone(), hi.clone());
    while &(&b - &a) > width {
        let mid = &(&a + &b) * &half;
        match sign(&f(&mid)) {
            0 => break,
            s if s == start => a = mid,
            _ => b = mid,
        }
    }
    Some(Encl { lo: a, hi: b })
}
