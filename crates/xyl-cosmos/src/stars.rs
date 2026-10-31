//! A star's life in closed form: its burn, light, main-sequence life, giant
//! phase and remnant, read at any age without time stepping (docs/model.md).
//!
//! Reads `circuit::anchor` (galaxy ring in kpc), `circuit::vacuum_share`,
//! `circuit::bell` and `echo` (one and two rings), the substrate's firmament
//! outflow rate (`cores::substrate_for`, per ring) and `compound::hold`.
//! Reach `r` is in galaxy rings (`reach_rings` converts from kpc); times are
//! rings of time after the Big Bang (one ring one bell); masses are core
//! masses and light is power in core units, core masses (times `c^2`,
//! `c = 1`) per ring. No constant of the Sun enters.
//!
//! - Compactness `delta`: a star's deficit at its rim, `m / (R 60 + m)`
//!   (`disc::rim_share`), with `R` the rim of the held core its cloud's cut
//!   gives (`galaxy::piece_rim`, from the atoms' open frames and masses in
//!   `formation::hold_at_heat`). Every star of a cloud shares the rim, so a
//!   heavier star is more compact (`galaxy::Star::compactness`).
//! - Burn (`burn_per_bell`): the share of the ring it receives that a star
//!   holds and transduces per bell, `delta x ring_per_bell(r)`, with the
//!   ring per bell `1 / (pi (r + 1))`.
//! - Power (`power`): the hold the burn transduces per ring,
//!   `P = m burn_per_bell(r) / bell`, the mass `m` (core masses) times `c^2`
//!   (`c = 1`) times the share transduced per bell, one ring.
//! - Light (`light`): `P (1 + pi bell q(t))`, where `q(t)` is the
//!   firmament's leaving rate (the vessel's drift over the ring per bell).
//!   `flicker` is the swing within a bell, `(pi / 2) |cos(2 pi t / echo)|`,
//!   a reading of the light only.
//! - Boundary (removed when layer C is converted): `suns_of_power` reads a
//!   power as the luminosity in suns the system and world layers still use,
//!   the power over that of a one-sun star (`galaxy::core_masses_of_suns`)
//!   at `r = 0`, so a star of `M` suns at reach `r` reads `M / (r + 1)`.
//! - Life (`life`, `ends`, `burned`): the hydrogen step's transduced share
//!   `1 - hold(He) / (4 hold(H))` over the burn, counted in rings (one bell
//!   per ring), the same for every mass at a reach because the burn is a
//!   share of the hold; the giant ends
//!   when the helium step's further share `1 - hold(C) / (3 hold(He))` is
//!   transduced. `remnant_share` is the hold neither step transduced.
//! - Not yet modelled: ejecta returned to the galaxy's gas, and stellar mixes
//!   other than hydrogen burning to helium.
//!
//! Before a founding the burn, life and drift read zero; divisions use
//! `checked_div` with explicit fallbacks. Terms: burn (docs/glossary.md).

use crate::circuit::vacuum_share;
use crate::compound::hold;
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// The hold of a kind as a rational.
fn hold_q(k: u32) -> Q {
    Q::int(i64::from(hold(k)))
}

/// A home-galaxy star's reach about the universal core (galaxy rings) from
/// its distance from the core (kpc), in the galaxy tier's ring, the unit in
/// which the star's piece is lit, so the vacuum's share `1 / (r + 1)` gives
/// the same hold at the star as the cooling gave at its piece. Zero before a
/// founding; never negative.
pub fn reach_rings(r_kpc: Dy) -> Dy {
    let kpc = Dy::from_q(&crate::circuit::anchor().galaxy_kpc);
    if kpc.signum() <= 0 {
        return Dy::ZERO;
    }
    (r_kpc / kpc).max(Dy::ZERO)
}

/// The pulse's ring delivered at reach `r` over one bell: the waveform's
/// magnitude integrated over half an echo, `1 / pi`, times the vacuum's share
/// `1 / (r + 1)`.
pub fn ring_per_bell(reach: Dy) -> Dy {
    vacuum_share(reach, Dy::ONE) / Dy::PI
}

/// The vessel's drift at reach `r` over one bell at `t` rings: `bell q(t)`
/// times the vacuum's share; zero before a founding or while nothing has
/// left the firmament.
pub fn drift_per_bell(reach: Dy, t: Dy) -> Dy {
    let anchor = crate::circuit::anchor();
    let Some(s) = crate::cores::substrate_for(&anchor.spin) else {
        return Dy::ZERO;
    };
    crate::circuit::bell() * s.rate(t.max(Dy::ZERO)).0 * vacuum_share(reach, Dy::ONE)
}

/// The share of a star's hold the pulse transduces per bell at reach `r`:
/// the star holds its compactness `delta` (its deficit at its rim,
/// `disc::rim_share`) of the ring it receives, `delta / (pi (r + 1))`.
pub fn burn_per_bell(delta: Dy, reach: Dy) -> Dy {
    delta.max(Dy::ZERO) * ring_per_bell(reach)
}

/// The share of the hold the hydrogen step transduces: four hydrogens'
/// hold less the helium's, over the four, `1 - hold(He) / (4 hold(H))`.
pub fn transduced_hydrogen() -> Q {
    &Q::ONE
        - &hold_q(2)
            .checked_div(&(&Q::int(4) * &hold_q(1)))
            .unwrap_or(Q::ZERO)
}

/// The share of the hold transduced by the end of the helium step: the
/// hydrogen step's share h plus the helium step's share of the remainder,
/// `h + (1 - h) (1 - hold(C) / (3 hold(He)))`.
pub fn transduced_helium() -> Q {
    let h = transduced_hydrogen();
    let he = &Q::ONE
        - &hold_q(6)
            .checked_div(&(&Q::int(3) * &hold_q(2)))
            .unwrap_or(Q::ZERO);
    &h + &(&(&Q::ONE - &h) * &he)
}

/// The share of the hold an ended star keeps as its remnant: what the pulse
/// did not transduce in either step.
pub fn remnant_share() -> Q {
    &Q::ONE - &transduced_helium()
}

/// A star's main-sequence life (rings) at reach `r` for compactness `delta`:
/// the hydrogen step's share over the burn per bell, a count of bells, each
/// one ring. A more compact star burns a larger share and lives fewer rings.
/// Zero before a founding or for a star that holds nothing.
pub fn life(delta: Dy, reach: Dy) -> Q {
    let burn = burn_per_bell(delta, reach);
    if burn.signum() <= 0 || crate::circuit::anchor().rim.signum() <= 0 {
        return Q::ZERO;
    }
    (Dy::from_q(&transduced_hydrogen()) / burn * crate::circuit::bell()).to_q()
}

/// A star's mean power at reach `r` (core masses per ring, `c = 1`): the
/// hold the burn transduces per ring, `P = m burn_per_bell(delta, r) / bell`,
/// with `m` the star's mass (core masses) and `delta` its compactness. Zero
/// for a non-positive mass.
pub fn power(mass: &Q, delta: Dy, reach: Dy) -> Q {
    let share = burn_per_bell(delta, reach) / crate::circuit::bell();
    (&mass.clone().max(Q::ZERO) * &share.to_q()).max(Q::ZERO)
}

/// A power (core masses per ring) as a luminosity in suns: the power over
/// that of a one-sun mass of the same compactness `delta` at reach 0, so a
/// star of `M` suns at reach `r` reads `M / (r + 1)`. Boundary for the
/// system and world layers still in suns, removed when layer C is
/// converted; zero before a founding.
pub fn suns_of_power(p: &Q, delta: Dy) -> Q {
    let sun = power(
        &crate::galaxy::core_masses_of_suns(&Q::ONE),
        delta,
        Dy::ZERO,
    );
    p.checked_div(&sun).unwrap_or(Q::ZERO)
}

/// A star's light (core masses per ring, `c = 1`) at `age` rings after its
/// birth at `born`, at reach `r`: the mean power ([`power`]) of a star of
/// `mass` core masses and compactness `delta` times the vessel's drift factor
/// `1 + pi bell q(t)`, with q read at `born + age`.
pub fn light(mass: &Q, delta: Dy, reach: Dy, born: Dy, age: Dy) -> Q {
    let mean = power(mass, delta, reach);
    let drift = drift_per_bell(reach, born + age) / ring_per_bell(reach).max(Dy::sci(1, -300));
    &mean * &(Dy::ONE + drift).to_q()
}

/// The radiation's swing within a bell at `t` rings: the ring's magnitude at
/// the instant over its mean across the bell, `(pi / 2) |cos(2 pi t / echo)|`.
/// It is a reading of the light only; a world's temperature uses the bell's
/// mean (`worlds`). Returns 1 before a founding.
pub fn flicker(t: Dy) -> Dy {
    let echo = crate::circuit::echo();
    if echo.signum() <= 0 || crate::circuit::anchor().rim.signum() <= 0 {
        return Dy::ONE;
    }
    Dy::PI / Dy::int(2) * (Dy::int(2) * Dy::PI * t / echo).cos().abs()
}

/// The share of a star's hydrogen step transduced by `age` rings at reach `r`
/// for compactness `delta`, exact: `age / life`; zero when the life is zero.
pub fn burned(delta: Dy, reach: Dy, age: &Q) -> Q {
    age.checked_div(&life(delta, reach)).unwrap_or(Q::ZERO)
}

/// When a star of compactness `delta` leaves the main sequence and when its
/// giant's helium is spent (rings) at reach `r`: the main-sequence life, and
/// that life times the ratio of the helium step's cumulative share to the
/// hydrogen step's.
pub fn ends(delta: Dy, reach: Dy) -> (Q, Q) {
    let main = life(delta, reach);
    let giant = &main
        * &transduced_helium()
            .checked_div(&transduced_hydrogen())
            .unwrap_or(Q::ONE);
    (main, giant)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transduced_shares_and_rim_deficit_in_bounds() {
        // Hydrogen holds 2 (3 less its one open frame), helium 6: four
        // hydrogens' 8 to one helium's 6 transduces a quarter.
        assert_eq!(transduced_hydrogen(), Q::new(1, 4).unwrap());
        let he = transduced_helium();
        assert!(he > transduced_hydrogen() && he < Q::ONE, "{he}");
        assert_eq!(&remnant_share() + &he, Q::ONE);
        // The ring per bell falls with the vacuum's share and is 1 / pi at
        // r = 0.
        assert!(ring_per_bell(Dy::ZERO) > ring_per_bell(Dy::int(10)));
        assert_eq!(ring_per_bell(Dy::ZERO), Dy::ONE / Dy::PI);
        // The flicker is at most pi / 2 times the mean.
        assert!(flicker(Dy::ZERO) <= Dy::PI / Dy::int(2) + Dy::sci(1, -12));
    }

    #[test]
    fn power_is_the_burn_of_the_hold_and_life_counts_rings() {
        let _ = crate::genesis::bang();
        let (m, r) = (Q::ratio(1, 1_000_000), Dy::int(3));
        let rim = Q::ONE;
        let delta = Dy::from_q(&crate::disc::rim_share(&m, &rim));
        // The rim share lies in (0, 1); a heavier star on the same rim is
        // more compact.
        assert!(delta.signum() > 0 && delta < Dy::ONE);
        let heavier = Dy::from_q(&crate::disc::rim_share(&(&m * &Q::int(2)), &rim));
        assert!(heavier > delta);
        // The power is linear in the mass at a given compactness and falls
        // outward with the vacuum's share.
        let p = power(&m, delta, r);
        assert!(p.signum() > 0);
        assert_eq!(power(&(&m * &Q::int(2)), delta, r), &p * &Q::int(2));
        assert!(power(&m, delta, Dy::ZERO) > p);
        assert_eq!(power(&Q::ZERO, delta, r), Q::ZERO);
        // The light starts from the power and only adds the drift.
        assert!(light(&m, delta, r, Dy::int(10), Dy::ZERO) >= p);
        // The life is the hydrogen step's share over the burn, in rings; the
        // giant ends after the main sequence; a more compact star lives
        // fewer rings.
        let (main, giant) = ends(delta, r);
        assert!(main.signum() > 0 && giant >= main);
        assert!(life(heavier, r) < main);
        let burned_by = &main * &burn_per_bell(delta, r).to_q();
        let gap = (&burned_by - &transduced_hydrogen()).abs();
        assert!(gap < Q::ratio(1, 1_000_000), "{burned_by}");
        // The boundary reads a one-sun mass at reach 0 as one sun.
        let sun = crate::galaxy::core_masses_of_suns(&Q::ONE);
        let one = suns_of_power(&power(&sun, delta, Dy::ZERO), delta);
        assert!((&one - &Q::ONE).abs() < Q::ratio(1, 1_000_000), "{one}");
    }
}
