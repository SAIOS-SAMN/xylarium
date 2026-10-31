//! A star's life in closed form: its transduction, light, main-sequence life, giant
//! phase and remnant, read at any age without time stepping (docs/model.md).
//!
//! Reads `circuit::anchor` (galaxy ring in kpc), `circuit::vacuum_share`,
//! `circuit::bell` and `echo` (one and two rings), the substrate's firmament
//! outflow rate (`cores::substrate_for`, per ring), the element table and
//! `compound` (frames at heat, in `sequence.rs`).
//! Reach `r` is in galaxy rings (`reach_rings` converts from kpc); times are
//! rings of time after the origin (one ring one bell); masses are core
//! masses and light is power in core units, core masses (times `c^2`,
//! `c = 1`) per ring. No constant of the Sun enters.
//!
//! - Compactness `delta`: a star's deficit at its rim, `m / (R 60 + m)`
//!   (`disc::rim_share`), with `R` the rim of the held core its cloud's cut
//!   gives (`galaxy::piece_rim`, from the atoms' open frames and masses in
//!   `formation::hold_at_heat`). Every star of a cloud shares the rim, so a
//!   heavier star is more compact (`galaxy::Star::compactness`).
//! - Transduction (`transduced_per_bell`): the share of the ring it receives that a star
//!   holds and transduces per bell, `delta x ring_per_bell(r)`, with the
//!   ring per bell `1 / (pi (r + 1))`.
//! - Power (`power`): the hold the transduction transduces per ring,
//!   `P = m transduced_per_bell(r) / bell`, the mass `m` (core masses) times `c^2`
//!   (`c = 1`) times the share transduced per bell, one ring.
//! - Light (`light`): `P (1 + pi bell q(t))`, where `q(t)` is the
//!   firmament's leaving rate (the vessel's drift over the ring per bell).
//!   `flicker` is the swing within a bell, `(pi / 2) |cos(2 pi t / echo)|`,
//!   a reading of the light only.
//! - Boundary (removed when layer C is converted): `suns_of_power` reads a
//!   power as the luminosity in suns the system and world layers still use,
//!   the power over that of a one-sun star (`galaxy::core_masses_of_suns`)
//!   at `r = 0`, so a star of `M` suns at reach `r` reads `M / (r + 1)`.
//! - Life (`life`, `ends`, `transduced`): each step of the star's transduction
//!   sequence (`Sequence`, from its cloud's held mixture and heat,
//!   `galaxy::Star::sequence`) lasts its transduced share over the transduction,
//!   counted in rings (one bell per ring). The main-sequence life is the
//!   first step's; the steps end one after another (`ends`), the same for
//!   every mass at a reach because the transduction is a share of the hold.
//!   `remnant_share` is the hold no step transduced.
//! - Sequence (`sequence.rs`): the held collective's pole point, its
//!   dilation to the throat, and the fusion and fission steps by polarity.
//! - Not yet modelled: ejecta returned to the galaxy's gas.
//!
//! Before a founding the transduction, life and drift read zero; divisions use
//! `checked_div` with explicit fallbacks. Terms: transduction (docs/glossary.md).

mod sequence;

pub use sequence::{
    fission_step, fusion_step, held_frames, nucleon_count, pole_point, throat_echoes, Sequence,
    Step,
};

use crate::circuit::vacuum_share;
use xyl_core::dyadic::Dy;
use xyl_core::Q;

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
pub fn transduced_per_bell(delta: Dy, reach: Dy) -> Dy {
    delta.max(Dy::ZERO) * ring_per_bell(reach)
}

/// The share of the hold a star's sequence leaves as its remnant: what no
/// step transduced (`Sequence::remnant`).
pub fn remnant_share(seq: &Sequence) -> Q {
    seq.remnant.clone()
}

/// The rings over which a star of compactness `delta` at reach `r`
/// transduces `share` of its hold: the share over the transduction per bell, a
/// count of bells, each one ring. Zero before a founding or for a star that
/// holds nothing.
fn rings_to_transduce(share: &Q, delta: Dy, reach: Dy) -> Q {
    let transduced = transduced_per_bell(delta, reach);
    if transduced.signum() <= 0 || crate::circuit::anchor().rim.signum() <= 0 {
        return Q::ZERO;
    }
    (Dy::from_q(share) / transduced * crate::circuit::bell()).to_q()
}

/// A star's main-sequence life (rings) at reach `r` for compactness `delta`:
/// its sequence's first step's transduced share over the transduction per bell. A
/// more compact star transduces a larger share and lives fewer rings. Zero before
/// a founding, for a star that holds nothing or for a sequence with no step
/// (the held body that does not ignite).
pub fn life(seq: &Sequence, delta: Dy, reach: Dy) -> Q {
    seq.steps
        .first()
        .map(|s| rings_to_transduce(&s.transduced, delta, reach))
        .unwrap_or(Q::ZERO)
}

/// A star's mean power at reach `r` (core masses per ring, `c = 1`): the
/// hold the transduction transduces per ring, `P = m transduced_per_bell(delta, r) / bell`,
/// with `m` the star's mass (core masses) and `delta` its compactness. Zero
/// for a non-positive mass.
pub fn power(mass: &Q, delta: Dy, reach: Dy) -> Q {
    let share = transduced_per_bell(delta, reach) / crate::circuit::bell();
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

/// The share of a star's first step transduced by `age` rings at reach `r`
/// for compactness `delta`, exact: `age / life`; zero when the life is zero.
pub fn transduced(seq: &Sequence, delta: Dy, reach: Dy, age: &Q) -> Q {
    age.checked_div(&life(seq, delta, reach)).unwrap_or(Q::ZERO)
}

/// When each step of a star's sequence ends (rings after its birth) at reach
/// `r` for compactness `delta`: the cumulative transduced share at each
/// step's end over the transduction per bell. The first is the main-sequence life;
/// the last is when the star becomes a remnant. Empty for a sequence with no
/// step.
pub fn ends(seq: &Sequence, delta: Dy, reach: Dy) -> Vec<Q> {
    seq.cumulative()
        .iter()
        .map(|share| rings_to_transduce(share, delta, reach))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_per_bell_and_flicker_in_bounds() {
        // The ring per bell falls with the vacuum's share and is 1 / pi at
        // r = 0.
        assert!(ring_per_bell(Dy::ZERO) > ring_per_bell(Dy::int(10)));
        assert_eq!(ring_per_bell(Dy::ZERO), Dy::ONE / Dy::PI);
        // The flicker is at most pi / 2 times the mean.
        assert!(flicker(Dy::ZERO) <= Dy::PI / Dy::int(2) + Dy::sci(1, -12));
    }

    #[test]
    fn power_is_the_transduced_hold_and_life_counts_rings() {
        let _ = crate::genesis::origin();
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
        // The life is the first step's transduced share over the transduction, in
        // rings; each later step ends after the one before; a more compact
        // star lives fewer rings.
        let seq = Sequence::of(&[(1, Q::ONE)], &Q::ZERO);
        let ends = ends(&seq, delta, r);
        let main = ends[0].clone();
        assert_eq!(life(&seq, delta, r), main);
        assert!(main.signum() > 0 && ends.windows(2).all(|w| w[1] >= w[0]));
        assert!(life(&seq, heavier, r) < main);
        let transduced_by = &main * &transduced_per_bell(delta, r).to_q();
        let gap = (&transduced_by - &seq.steps[0].transduced).abs();
        assert!(gap < Q::ratio(1, 1_000_000), "{transduced_by}");
        // The boundary reads a one-sun mass at reach 0 as one sun.
        let sun = crate::galaxy::core_masses_of_suns(&Q::ONE);
        let one = suns_of_power(&power(&sun, delta, Dy::ZERO), delta);
        assert!((&one - &Q::ONE).abs() < Q::ratio(1, 1_000_000), "{one}");
    }
}
