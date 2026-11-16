//! Chronometry's pure forms: every quantity of time as one closed form,
//! exact, with its calibre named (docs/chronometry.md, #299, #300).
//!
//! There is no background clock. A space winds the cycles inside it, its
//! windings `K` on each turning plane, while the T lane stands; T is
//! written only when a state crosses a throat (`T -> T - 2 (H - R)`,
//! `coordinate::PolePlane`) or a closure is verified. The forms here read
//! numbers only; the holds and shares they read come from the tiers above.
//!
//! - [`arrival`]: the front's arrival, the separation of two places' time
//!   coordinates, `tau = d + H` with the hold crossed (E032).
//! - [`well`]: a clock's rate in a well, its divergent face `r / (r + M)`,
//!   which is `r / tau(r)`.
//! - [`sheared_return`]: a throat's echo, the near face's return after
//!   `bell / (1 - s)` (T4); the far face's is `bell / s`.
//! - [`breath`] and [`breathing_turns`]: the bell's cycle on a manifold and
//!   the turns of an orbit breathing with it (#233).
//! - [`winding`]: the whole turns a turning of `theta` degrees has closed,
//!   `K = floor(abs(theta) / 360)`.
//! - [`keep`]: a keeper's reading at the root's winding: its local count,
//!   its slip, its coherence and its rungs.
//!
//! Every value is rational; a duration with pi in it (a period around a
//! circle) is read only through a certified enclosure at display, never
//! returned here.

use crate::certified::{cos_sin_turns, toward_zero, Encl};
use crate::dyadic::Dy;
use crate::Q;

/// The front's arrival at a place `d` away, in the unit `unit` of the
/// distance (the light time over one `unit`): `(d + H) / unit`, `H` the hold
/// the front crosses on the way, read in the distance's calibre through
/// `per_hold` (one hold unit's length). With no hold crossed it is light's
/// time; each hold crossed adds its light time; a negative hold adds
/// nothing. `None` for a unit or a calibre that is not positive.
pub fn arrival(d: &Q, held: &Q, per_hold: &Q, unit: &Q) -> Option<Q> {
    if per_hold.signum() <= 0 || unit.signum() <= 0 {
        return None;
    }
    let crossed = held.max(&Q::ZERO).checked_div(per_hold)?;
    (d + &crossed).checked_div(unit)
}

/// A clock's rate in a well of hold `m` at reach `r` (one calibre): the
/// well's divergent face `r / (r + M)` (`shares::faces`), the antipole of
/// the deficit, which is `r / tau(r)`; `None` where the faces are refused
/// (a negative part, or no reach and no hold).
pub fn well(r: &Q, m: &Q) -> Option<Q> {
    crate::shares::faces(m, r).map(|f| f.divergent)
}

/// A throat's sheared return at its torsion share `s` (T4, axiom 7): the
/// near face returns after `bell / (1 - s)`; `None` where `s` is not below
/// one (no return). The far face returns after `bell / s`
/// (`sheared_return` of `1 - s`).
pub fn sheared_return(bell: Dy, s: &Q) -> Option<Dy> {
    let open = &Q::ONE - s;
    if open.signum() <= 0 {
        return None;
    }
    bell.div(&Dy::from_q(&open))
}

/// One breath of a manifold at `turns` of its core's echo: `1 + delta
/// cos(360 turns)`, `delta` the deficit the hold swings by, the cosine read
/// through a certified enclosure and cut toward zero on the `2^-64` grid.
pub fn breath(delta: &Q, turns: &Q) -> Q {
    let (cos, _) = cos_sin_turns(&Encl::exact(turns.frac()));
    &Q::ONE + &toward_zero(&(delta * &cos.mid()), 64)
}

/// The turns at `t` of an orbit whose mean rate is `rate` (turns per unit
/// of `t`) at deficit `delta` about its core, breathing with that core's
/// echo `echo`: the hold inside the orbit swings as `M (1 + delta cos(360 t
/// / echo))`, so a rate that goes as the deficit swings first-degree by
/// `delta (1 - delta)`, and its turns are the closed form
/// `rate t + rate delta (1 - delta) echo sin(360 t / echo) / (2 pi)`. The
/// mean alone where the echo is not positive; over whole echoes, the mean.
pub fn breathing_turns(rate: Dy, delta: Dy, t: Dy, echo: Dy) -> Option<Dy> {
    let mean = rate * t;
    if echo.signum() <= 0 {
        return Some(mean);
    }
    let two_pi = Dy::int(2) * Dy::pi();
    let angle = (two_pi * t).div(&echo)?;
    let swing = delta * (Dy::ONE - delta);
    Some(mean + rate * swing * echo.div(&two_pi)? * angle.sin())
}

/// The whole turns a turning of `theta` degrees has closed, `K =
/// floor(abs(theta) / 360)`: a winding advances while the T lane stands.
/// `None` past `i128`.
pub fn winding(theta: &Q) -> Option<i128> {
    theta.abs().checked_div(&Q::int(360))?.floor_i128()
}

/// A keeper's reading at the root's winding `n`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keeping {
    /// The keeper's own count, in its rings: `rate (n - delay) / ring`.
    pub local: Q,
    /// The turns it has fallen behind the root: `(1 - rate)(n - delay) /
    /// ring`.
    pub slip: Q,
    /// Its phase agreement with the root, in `[0, 1]`: `1 - 2 |slip -
    /// round(slip)|`, 1 in phase and 0 half a turn out.
    pub coherence: Q,
    /// Whole turns of slip, the slip's winding.
    pub rung: i128,
}

/// A keeper's reading at the root's winding `n`: a keeper at `rate` (its
/// nested wells), reached `delay` after the root, counting in rings of
/// `ring` root windings. `n - delay` is held at zero before the front
/// arrives. `None` for a ring that is not positive or a slip past `i128`.
pub fn keep(rate: &Q, delay: &Q, ring: &Q, n: &Q) -> Option<Keeping> {
    if ring.signum() <= 0 {
        return None;
    }
    let since = (n - delay).max(Q::ZERO);
    let own = since.checked_div(ring)?;
    let slip = &(&Q::ONE - rate) * &own;
    let rung = slip.floor_i128()?;
    let frac = &slip - &Q::from_i128(rung, 1)?;
    let half = crate::ratio!(1, 2);
    let off = if frac > half { &Q::ONE - &frac } else { frac };
    Some(Keeping {
        local: rate * &own,
        coherence: &Q::ONE - &(&Q::int(2) * &off),
        slip,
        rung,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The identities every quantity of time keeps: arrivals add and a hold
    /// crossed only lengthens them; a well's rate lies in (0, 1] and is r /
    /// tau(r); the near and far echoes are equal only at an even share; a
    /// breath returns over whole echoes; a winding counts whole turns either
    /// way; a keeper at rate one never slips and is always in phase.
    #[test]
    fn the_forms_of_time_keep_their_identities() {
        let (per, unit) = (Q::int(1000), crate::ratio!(3, 2));
        let at = |d: i64, h: i64| arrival(&Q::int(d), &Q::int(h), &per, &unit).unwrap();
        let (a, b) = (at(5, 100), at(7, 300));
        assert_eq!(&a + &b, at(12, 400));
        assert!(a > at(5, 0));
        assert_eq!(at(6, -9), Q::int(4));
        assert_eq!(arrival(&Q::ONE, &Q::ONE, &per, &Q::ZERO), None);
        for (r, m) in [(1, 1), (5, 2), (100, 3)] {
            let rate = well(&Q::int(r), &Q::int(m)).unwrap();
            assert!(rate.signum() > 0 && rate <= Q::ONE);
            assert_eq!(Some(rate), Q::new(r, r + m), "r / tau(r)");
        }
        let half = crate::ratio!(1, 2);
        assert_eq!(sheared_return(Dy::ONE, &half), Some(Dy::int(2)));
        let s = crate::ratio!(3, 5);
        let (near, far) = (
            sheared_return(Dy::ONE, &s),
            sheared_return(Dy::ONE, &(&Q::ONE - &s)),
        );
        assert!(near > far, "the faces part at a spinning share");
        assert_eq!(sheared_return(Dy::ONE, &Q::ONE), None);
        let crest = breath(&crate::ratio!(1, 3), &Q::ZERO);
        assert!(
            crest <= crate::ratio!(4, 3)
                && &crate::ratio!(4, 3) - &crest < Q::new(1, 1 << 56).unwrap()
        );
        assert_eq!(
            breath(&crate::ratio!(1, 3), &Q::int(5)),
            breath(&crate::ratio!(1, 3), &Q::ZERO)
        );
        let (rate, d, echo) = (Dy::ONE, Dy::from_q(&crate::ratio!(1, 3)), Dy::int(12));
        for k in 1..4 {
            let t = echo * Dy::int(k);
            assert!(
                (breathing_turns(rate, d, t, echo).unwrap() - rate * t).abs() < Dy::sci(1, -12)
            );
        }
        assert_eq!(winding(&Q::int(725)), Some(2));
        assert_eq!(winding(&Q::int(-725)), Some(2));
        assert_eq!(winding(&Q::int(359)), Some(0));
        let still = keep(&Q::ONE, &Q::int(3), &Q::ONE, &Q::int(40)).unwrap();
        assert_eq!(
            (still.slip, still.coherence, still.rung),
            (Q::ZERO, Q::ONE, 0)
        );
        assert_eq!(still.local, Q::int(37));
        let slow = keep(&crate::ratio!(3, 4), &Q::ZERO, &Q::ONE, &Q::int(6)).unwrap();
        assert_eq!((slow.slip.clone(), slow.rung), (crate::ratio!(3, 2), 1));
        assert_eq!(slow.coherence, Q::ZERO, "half a turn out");
        assert_eq!(
            keep(&half, &Q::int(9), &Q::ONE, &Q::int(4)).unwrap().local,
            Q::ZERO
        );
        assert_eq!(keep(&half, &Q::ZERO, &Q::ZERO, &Q::int(4)), None);
    }

    /// An orbit's rate breathes with its core's bell: over whole echoes its
    /// turns are the mean rate's, between them they swing about the mean by
    /// at most `rate delta (1 - delta) echo / (2 pi)`, ahead of it on the
    /// echo's first half and behind on its second; with no echo the mean
    /// alone (#233 item 5).
    #[test]
    fn an_orbits_rate_breathes() {
        let (rate, delta, echo) = (crate::dyadic!(1, 3), crate::dyadic!(1, 4), Dy::int(2));
        let bound = rate * delta * (Dy::ONE - delta) * echo.div(&(Dy::int(2) * Dy::pi())).unwrap();
        let tiny = Dy::sci(1, -12);
        for k in 0..5 {
            let t = echo * Dy::int(k);
            assert!((breathing_turns(rate, delta, t, echo).unwrap() - rate * t).abs() < tiny);
        }
        let quarter = echo.div(&Dy::int(4)).unwrap();
        let ahead = breathing_turns(rate, delta, quarter, echo).unwrap() - rate * quarter;
        assert!(ahead > Dy::ZERO && (ahead - bound).abs() < tiny);
        let late = echo * crate::dyadic!(3, 4);
        assert!(breathing_turns(rate, delta, late, echo).unwrap() < rate * late);
        for k in 0..16 {
            let t = echo * Dy::int(k).div(&Dy::int(7)).unwrap();
            assert!(
                (breathing_turns(rate, delta, t, echo).unwrap() - rate * t).abs() <= bound + tiny
            );
        }
        assert_eq!(
            breathing_turns(rate, delta, quarter, Dy::ZERO),
            Some(rate * quarter)
        );
    }
}
