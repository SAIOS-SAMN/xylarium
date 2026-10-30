//! A star's life in closed form, fuelled by the pulse (docs/model.md), read
//! at any age (no time stepping).
//!
//! - The burn: the share of its hold the pulse transduces per bell. A star
//!   at reach `r` about the universal core (core rings, cluster units)
//!   absorbs the ring the bell brings there, the ocean's waveform's
//!   magnitude over one bell, `1 / pi` of the universal state falling as the
//!   vacuum's share `1 / (r + 1)`, plus the vessel's drift `bell q(t)`; it
//!   takes its own deficit of that, `delta_star = M r_g / (R + M r_g)`, the
//!   turning law at its rim (its radius linear in its mass, so one share for
//!   every star). The burn is `delta_star A(r, t)` per bell.
//! - The light is that burn, transduced (the `3Z` hold conserved, light the
//!   flux): in suns, `L = M / (r + 1) (1 + pi bell q(t))`, so a one-sun star
//!   at the core's reach under a quiet vessel shines one sun, the unit. Its
//!   swing within a bell (the ring itself) is a separate reading
//!   ([`flicker`]), the radiation's, not the climate's.
//! - The life: the share of the hold the hydrogen step transduces,
//!   `1 - hold(He) / (4 hold(H))`, over the burn per bell, in bells; it
//!   leaves the main sequence there, and its giant's helium is spent where
//!   the further share of the helium step is transduced. No 10 Gyr, no fitted
//!   brightening, no mass cut for the remnant: every star's fuel is its
//!   kinds' hold and every ended star leaves the hold the pulse did not
//!   transduce ([`remnant_share`]).
//! - Not yet modelled: a star's own ejecta delivered to the galaxy's gas, and
//!   a star's actual mix (every star is read as hydrogen burning to helium).

use crate::circuit::vacuum_share;
use crate::compound::hold;
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// The hold of a kind as a rational.
fn hold_q(k: u32) -> Q {
    Q::int(i64::from(hold(k)))
}

/// A star's deficit at its own rim, `M r_g / (R + M r_g)`: its gravitational
/// length over its radius plus that length, read on the sun's (the radius
/// linear in the mass, both scale together: one share for every star).
pub fn delta_star() -> Q {
    let r_g = crate::systems::star_length_m();
    r_g.checked_div(&(&crate::package::Si::star_radius_m() + &r_g))
        .unwrap_or(Q::ZERO)
}

/// A home-galaxy star's reach about the universal core (galaxy rings) from
/// its distance from the core (kpc): the galaxy tier's ring, the unit its
/// piece is lit in, so the vacuum's share `1 / (r + 1)` reads the same hold
/// at the star as the cooling read at its piece.
pub fn reach_rings(r_kpc: Dy) -> Dy {
    let kpc = Dy::from_q(&crate::circuit::anchor().galaxy_kpc);
    if kpc.signum() <= 0 {
        return Dy::ZERO;
    }
    (r_kpc / kpc).max(Dy::ZERO)
}

/// The ring the bell brings to reach `r` over one bell: the waveform's
/// magnitude integrated over half an echo, `1 / pi`, falling as the vacuum's
/// share `1 / (r + 1)`.
pub fn ring_per_bell(reach: Dy) -> Dy {
    vacuum_share(reach, Dy::ONE) / Dy::PI
}

/// The vessel's drift at reach `r` over one bell at `t` Gyr: `bell q(t)`
/// falling as the vacuum's share; zero before a founding or while nothing
/// has left the firmament.
pub fn drift_per_bell(reach: Dy, t: Dy) -> Dy {
    let anchor = crate::circuit::anchor();
    let Some(s) = crate::cores::substrate_for(&anchor.spin) else {
        return Dy::ZERO;
    };
    s.bell * s.rate(t.max(Dy::ZERO)).0 * vacuum_share(reach, Dy::ONE)
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
/// hydrogen's, plus the helium's own share of what it keeps,
/// `1 - hold(C) / (3 hold(He))`.
pub fn transduced_helium() -> Q {
    let h = transduced_hydrogen();
    let he = &Q::ONE
        - &hold_q(6)
            .checked_div(&(&Q::int(3) * &hold_q(2)))
            .unwrap_or(Q::ZERO);
    &h + &(&(&Q::ONE - &h) * &he)
}

/// The share of its hold an ended star keeps as its remnant: what the pulse
/// did not transduce through both steps.
pub fn remnant_share() -> Q {
    &Q::ONE - &transduced_helium()
}

/// A star's main-sequence life (Gyr) at reach `r`: the hydrogen step's
/// share over the burn per bell, in bells. Zero before a founding.
pub fn life(reach: Dy) -> Q {
    let burn = burn_per_bell(reach);
    if burn.signum() <= 0 {
        return Q::ZERO;
    }
    (Dy::from_q(&transduced_hydrogen()) / burn * crate::circuit::bell_gyr()).to_q()
}

/// A star's light (suns) at `age` Gyr after its birth at `born`, at reach
/// `r`: its mean luminosity (systems::luminosity) brightened by the vessel's
/// drift, `1 + pi bell q(t)`.
pub fn light(mass: &Q, reach: Dy, born: Dy, age: Dy) -> Q {
    let mean = crate::systems::luminosity(mass, reach);
    let drift = drift_per_bell(reach, born + age) / ring_per_bell(reach).max(Dy::sci(1, -300));
    &mean * &(Dy::ONE + drift).to_q()
}

/// The radiation's swing within a bell at `t` Gyr: the ring's magnitude at
/// the instant over its mean across the bell, `(pi / 2) |cos(2 pi t / echo)|`.
/// A reading of the light, never of a climate: a world's temperature reads
/// the bell's mean (worlds.rs).
pub fn flicker(t: Dy) -> Dy {
    let echo = crate::circuit::echo_gyr();
    if echo.signum() <= 0 {
        return Dy::ONE;
    }
    Dy::PI / Dy::int(2) * (Dy::int(2) * Dy::PI * t / echo).cos().abs()
}

/// The share of a star's hydrogen step transduced by `age` Gyr at reach `r`,
/// exact: `age / life`.
pub fn burned(reach: Dy, age: &Q) -> Q {
    age.checked_div(&life(reach)).unwrap_or(Q::ZERO)
}

/// When a star leaves the main sequence and when its giant's helium is spent
/// (Gyr) at reach `r`: its life, and its life stretched by the helium step's
/// share over the hydrogen's.
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
    fn the_holds_set_the_steps_and_the_rim_sets_the_share() {
        // hydrogen holds 2 (3 less its one open frame), helium 6: four
        // hydrogens' 8 to one helium's 6 transduces a quarter
        assert_eq!(transduced_hydrogen(), Q::new(1, 4).unwrap());
        let he = transduced_helium();
        assert!(he > transduced_hydrogen() && he < Q::ONE, "{he}");
        assert_eq!(&remnant_share() + &he, Q::ONE);
        // the star's rim share is small and positive
        let d = delta_star();
        assert!(d.signum() > 0 && d < Q::ONE, "{d}");
        // the ring per bell falls as the vacuum's share
        assert!(ring_per_bell(Dy::ZERO) > ring_per_bell(Dy::int(10)));
        assert_eq!(ring_per_bell(Dy::ZERO), Dy::ONE / Dy::PI);
        // the flicker peaks at pi / 2 times the mean and vanishes at the node
        assert!(flicker(Dy::ZERO) <= Dy::PI / Dy::int(2) + Dy::sci(1, -12));
    }
}
