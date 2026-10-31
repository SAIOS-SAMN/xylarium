//! A star's life in closed form: its burn, light, main-sequence life, giant
//! phase and remnant, read at any age without time stepping (docs/model.md).
//!
//! Reads `circuit::anchor` (galaxy ring in kpc), `circuit::vacuum_share`,
//! `circuit::bell` and `echo` (one and two rings), the substrate's firmament
//! outflow rate (`cores::substrate_for`, per ring), `compound::hold` and,
//! for the star's rim share, the sun's mass and radius from `package::Si`
//! (open: see `delta_star`). Reach `r` is in galaxy rings (`reach_rings`
//! converts from kpc); times are rings of time after the Big Bang (one ring
//! one bell); masses are core masses and light is power in core units, core
//! masses (times `c^2`, `c = 1`) per ring.
//!
//! - Burn (`burn_per_bell`): the share of a star's hold the pulse transduces
//!   per bell, `delta_star x ring_per_bell(r)`. The ring per bell is
//!   `1 / (pi (r + 1))`; `delta_star = M r_g / (R + M r_g)` is the turning
//!   law's deficit at the star's rim, the same for every star because the
//!   radius is taken linear in the mass.
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

/// A star's deficit at its rim, `M r_g / (R + M r_g)`: the star's
/// gravitational length over its radius plus that length, evaluated for the
/// sun. The radius is taken linear in the mass, so both scale together and
/// the share is the same for every star. Open: it reads the sun's mass and
/// radius (`package::Si`); a derivation from the founding needs the star's
/// rim from its cut (`disc::star_cut`), whose over-pressure reads this burn.
pub fn delta_star() -> Q {
    let r_g = crate::systems::star_length_m();
    r_g.checked_div(&(&crate::package::Si::star_radius_m() + &r_g))
        .unwrap_or(Q::ZERO)
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

/// The share of a star's hold the pulse transduces per bell at reach `r`,
/// at the echo's mean: `delta_star` times the ring per bell.
pub fn burn_per_bell(reach: Dy) -> Dy {
    Dy::from_q(&delta_star()) * ring_per_bell(reach)
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

/// A star's main-sequence life (rings) at reach `r`: the hydrogen step's
/// share over the burn per bell, a count of bells, each one ring. Zero
/// before a founding.
pub fn life(reach: Dy) -> Q {
    let burn = burn_per_bell(reach);
    if burn.signum() <= 0 || crate::circuit::anchor().rim.signum() <= 0 {
        return Q::ZERO;
    }
    (Dy::from_q(&transduced_hydrogen()) / burn * crate::circuit::bell()).to_q()
}

/// A star's mean power at reach `r` (core masses per ring, `c = 1`): the
/// hold the burn transduces per ring, `P = m burn_per_bell(r) / bell`, with
/// `m` the star's mass (core masses). Zero for a non-positive mass.
pub fn power(mass: &Q, reach: Dy) -> Q {
    let share = burn_per_bell(reach) / crate::circuit::bell();
    (&mass.clone().max(Q::ZERO) * &share.to_q()).max(Q::ZERO)
}

/// A power (core masses per ring) as a luminosity in suns: the power over
/// that of a one-sun star (`galaxy::core_masses_of_suns`) at reach 0, so a
/// star of `M` suns at reach `r` reads `M / (r + 1)`. Boundary for the
/// system and world layers still in suns; zero before a founding.
pub fn suns_of_power(p: &Q) -> Q {
    let sun = power(&crate::galaxy::core_masses_of_suns(&Q::ONE), Dy::ZERO);
    p.checked_div(&sun).unwrap_or(Q::ZERO)
}

/// A star's light (core masses per ring, `c = 1`) at `age` rings after its
/// birth at `born`, at reach `r`: the mean power ([`power`]) of a star of
/// `mass` core masses times the vessel's drift factor `1 + pi bell q(t)`,
/// with q read at `born + age`.
pub fn light(mass: &Q, reach: Dy, born: Dy, age: Dy) -> Q {
    let mean = power(mass, reach);
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

/// The share of a star's hydrogen step transduced by `age` rings at reach `r`,
/// exact: `age / life`; zero when the life is zero.
pub fn burned(reach: Dy, age: &Q) -> Q {
    age.checked_div(&life(reach)).unwrap_or(Q::ZERO)
}

/// When a star leaves the main sequence and when its giant's helium is spent
/// (rings) at reach `r`: the main-sequence life, and that life times the ratio
/// of the helium step's cumulative share to the hydrogen step's.
pub fn ends(reach: Dy) -> (Q, Q) {
    let main = life(reach);
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
        // The star's rim share lies in (0, 1).
        let d = delta_star();
        assert!(d.signum() > 0 && d < Q::ONE, "{d}");
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
        // The power is linear in the mass and is the share the burn
        // transduces per ring; it falls outward with the vacuum's share.
        let p = power(&m, r);
        assert!(p.signum() > 0);
        assert_eq!(power(&(&m * &Q::int(2)), r), &p * &Q::int(2));
        assert!(power(&m, Dy::ZERO) > p);
        assert_eq!(power(&Q::ZERO, r), Q::ZERO);
        // The light starts from the power and only adds the drift.
        assert!(light(&m, r, Dy::int(10), Dy::ZERO) >= p);
        // The life is the hydrogen step's share over the burn, in rings, and
        // the giant ends after the main sequence.
        let (main, giant) = ends(r);
        assert!(main.signum() > 0 && giant >= main);
        let burned_by = &main * &burn_per_bell(r).to_q();
        let gap = (&burned_by - &transduced_hydrogen()).abs();
        assert!(gap < Q::ratio(1, 1_000_000), "{burned_by}");
        // The boundary reads a one-sun star at reach 0 as one sun.
        let sun = crate::galaxy::core_masses_of_suns(&Q::ONE);
        let one = suns_of_power(&power(&sun, Dy::ZERO));
        assert!((&one - &Q::ONE).abs() < Q::ratio(1, 1_000_000), "{one}");
    }
}
