//! Events: an event is where one continuous quantity crosses another, and
//! its time is solved, never found by stepping. Between events nothing is
//! computed; a reading evaluates the orbits it needs at its time.

use crate::certified::Encl;
use crate::Q;

/// Where `f` crosses zero inside `[lo, hi]`, enclosed to `width`: `f`'s
/// values are enclosures, and it must certainly differ in sign at the two
/// ends (`None` otherwise). The bracket is halved on the side the
/// midpoint's sign certainly falls; where the midpoint's enclosure holds
/// zero the crossing is within the bracket already, and the bracket is
/// returned. The bracket is the event's time if `f` crosses once inside it
/// (a monotone piece, the caller's knowledge of its law).
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
