//! A star's disc: the star tier's cut of a lit bead's infall and the worlds
//! its manifold holds against the galaxy core's twist.
//!
//! Reads `circuit::Bead`, `circuit::throat_push` and the formation helpers
//! (`cooling`, `lights`, `hold_at`, `pulse_heat`, `vacuum_halving`).
//! All quantities are exact `Q`: reaches and holdings are shares of the
//! infall or of the star's manifold, extents are in galaxy units, heat in
//! hydrogen units.
//!
//! Outputs:
//! - `star_hold` -> `Option<StarHold>`, read in three states of the infall.
//!   Collapse: the dark cut holds `s0` within the rim `R0` (system-tier
//!   units), and the star's compactness is `delta0 = s0 c / (R0 60 + s0 c)`
//!   (`rim_share`). Ignition: the star holds `delta0` of the turning it
//!   receives per echo, its transduction, which adds the over-pressure
//!   `p0 = 1 + transduced`. Settled: the transduction cut gives the star's held share
//!   and rim. The dark cut's held kinds, renormalised, are the star's held
//!   mixture (`StarHold::held`), read with the cut's heat for the star's
//!   transduction sequence (`stars::Sequence`). Also the tide `H / (R + H)`, the
//!   disc's agitation `a_disc` and the infall's shear `H w / (R (R + w))`;
//!   `None` where the bead does not light.
//! - `rim_share`: a held body's deficit at its rim, `m / (R 60 + m)`.
//! - `manifold_worlds`: reads the manifold outside the star's share in 360
//!   equal degrees, each adding `1 - a` of its line; a world closes where
//!   the gathered matter's deficit `m / (star + m)` reaches the twist
//!   `tide x`. Returns each world's (reach, holding) and the belt share.
//!
//! Invariant: worlds and belts together hold `1 - star`. This file is not
//! among the founding digest's sources (build.rs), so editing it leaves the
//! founding cache valid. Terms: belt (docs/glossary.md).

use crate::circuit::Bead;
use crate::formation::*;
use xyl_core::Q;

/// A held body's deficit at its rim, `m / (R + m)`, with `m` its mass in
/// core masses (equal to its gravitational length in `r_g`) and `R` its rim
/// in system-tier units (`60 r_g`, `xyl_core::units::Tier::System`): the
/// share of what reaches the body that it holds. Zero for a non-positive
/// mass; one for a body with no extent.
pub fn rim_share(mass: &Q, rim: &Q) -> Q {
    if mass.signum() <= 0 {
        return Q::ZERO;
    }
    let rim_rg = &rim.clone().max(Q::ZERO) * &xyl_core::units::Tier::System.in_rg();
    xyl_core::shares::deficit(mass, &rim_rg).unwrap_or(Q::ONE)
}

/// A lit bead's infall and the star's cut of it, read on the chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StarHold {
    /// The share of the infall the star holds.
    pub star: Q,
    /// The galaxy's tide at the pinch, `H / (R + H)`: the core-star tie.
    pub tide: Q,
    /// The disc's agitation, at most 1: the pulse's turning at the star per
    /// echo, times the inbound echo's swing and the disc's grip, over what
    /// the disc sheds per ring (the released kinds' open frames and the
    /// vacuum's halving).
    pub a_disc: Q,
    /// The infall's extent (galaxy units).
    pub infall: Q,
    /// The infall's holding (share of the galaxy's grains).
    pub cloud: Q,
    /// The turning the infall carries into the cut: the galaxy's shear across
    /// the infall's extent, `H w / (R (R + w))`.
    pub spin: Q,
    /// The turning the cut was read at, per set of planes
    /// (`atoms::Received`): on the isoclinic planes the pulse's turning at
    /// the star per echo, in hydrogen units (`atoms::unpaired_hot`), and on
    /// the axis plane the star's compactness.
    pub received: crate::atoms::Received,
    /// The over-pressure the star's transduction adds, `1 + transduced per echo`.
    pub p0: Q,
    /// The held core's rim (system-tier units, `60 r_g`): the reach of the
    /// loosest kind the transduction star still holds (`formation::Hold::reach`).
    /// The cut reads reaches one tier below the piece's, as the founding
    /// reads the cluster tier and the pieces the galaxy tier.
    pub rim: Q,
    /// The collapsed star's compactness before it transduces: its deficit at the
    /// dark cut's rim, `m0 / (R0 + m0)` with `m0 = s0 c` (core masses, equal
    /// to `r_g`) and `R0` the dark rim in `r_g`. Its transduction per echo is this
    /// share of the received turning.
    pub compactness: Q,
    /// The collapsed star's held mixture: the kinds of the infall's mix the
    /// dark cut holds (those not in its released part, `Hold::rest`), each
    /// with its share of the mix renormalised over the held kinds. The
    /// star's transduction sequence is read from it at `heat`
    /// (`stars::Sequence::of`).
    pub held: Vec<(u32, Q)>,
}

/// The star tier's cut of a lit bead's infall (`hold_at` on the bead's
/// kinds at the throat's push). The infall collapses carrying the galaxy's
/// shear across its extent as turning (`add`), is heated by the pulse's
/// turning at the star per echo (`heat`, hydrogen units) and is pushed by the
/// star's transduction as over-pressure (`p0 = 1 + transduced per echo`); the star holds
/// what remains held. `turning` is the pulse's turning per echo at the star's
/// reach about the universal core and `transduced` the star's transduction per echo
/// (`stars`). `None` where the bead does not light.
pub fn star_hold(bead: &Bead, unit: &Q, held: &Q, swing: &Q, turning: &Q) -> Option<StarHold> {
    let h = &Q::ONE + held;
    let c = cooling(bead, unit, &h, swing)?;
    if !lights(&c) {
        return None;
    }
    let reach = bead
        .reach
        .clone()
        .max(Q::new(1, 1_000_000_000).unwrap_or(Q::ONE));
    let spin = (&h * &c.infall)
        .checked_div(&(&reach * &(&reach + &c.infall)))
        .unwrap_or(Q::ZERO);
    // Collapse: the dark cut (no over-pressure) holds the share `s0` of the
    // infall within the rim `R0` (system-tier units). The collapsed star's
    // mass is `m0 = s0 c` core masses, which is `m0` in `r_g`, so its
    // compactness is `delta0 = m0 / (R0 60 + m0)`.
    let push = crate::circuit::throat_push().to_q();
    let isoclinic = crate::atoms::Received::isoclinic(turning.clone());
    let dark = hold_at(&c.mix, &push, &spin, &isoclinic, &Q::ONE);
    let compactness = rim_share(&(&dark.held * &c.cloud), &dark.reach);
    let held = held_mixture(&c.mix, &dark.rest);
    // Ignition: the star holds `delta0` of the turning it receives per echo;
    // that held share is its transduction, which adds the over-pressure
    // `p0 = 1 + transduced` and releases the loosest kinds.
    let p0 = &Q::ONE + &(&compactness * turning);
    let cut = hold_at(&c.mix, &push, &spin, &isoclinic, &p0);
    let star = cut.held;
    let rim = cut.reach.clone();
    // `a_disc = turning x swing x grip / shed`, at most 1, in the form in
    // which the cooling reads a piece's heat (`formation::pulse_heat`):
    // grip = D / (1 + D) with D the disc's holding, shed = the released
    // kinds' open frames plus the vacuum's halving.
    let disc_holding = &Q::ONE - &star;
    let grip = disc_holding
        .checked_div(&(&Q::ONE + &disc_holding))
        .unwrap_or(Q::ZERO);
    let a_disc = pulse_heat(
        &(&(turning * swing) * &grip),
        &cut.rest_open,
        vacuum_halving(&reach, &h),
    );
    Some(StarHold {
        star,
        tide: c.tide,
        a_disc,
        infall: c.infall,
        cloud: c.cloud,
        spin,
        received: crate::atoms::Received {
            axis: compactness.clone(),
            isoclinic: turning.clone(),
        },
        p0,
        rim,
        compactness,
        held,
    })
}

/// The kinds of `mix` (kind, share) that a cut holds, those absent from its
/// released part `rest`, with their shares renormalised to sum to one. Empty
/// where the cut holds nothing.
fn held_mixture(mix: &[(u32, Q)], rest: &[(u32, Q)]) -> Vec<(u32, Q)> {
    let kept: Vec<(u32, Q)> = mix
        .iter()
        .filter(|(z, s)| s.signum() > 0 && !rest.iter().any(|(r, _)| r == z))
        .cloned()
        .collect();
    let total = kept.iter().fold(Q::ZERO, |a, (_, s)| &a + s);
    kept.into_iter()
        .filter_map(|(z, s)| Some((z, s.checked_div(&total)?)))
        .collect()
}

/// A star's worlds on its manifold (the anchor chain: the star is a zero
/// point, its manifold the arc the star's mass holds against the core). In
/// the manifold's units (reach 1, line holding 1 across it), the star holds
/// the inner `star` share; the remaining `1 - star` is read degree by degree
/// outward to the reach, 360 degrees in all. Each degree adds the matter its
/// heat allows to be held, `1 - a` of the line. A world forms where the
/// matter gathered since the last world holds its region against the core's
/// twist at that degree: its deficit share `m / (star + m)` is at least the
/// twist `tide x` (twist is the deficit, and the core's deficit across the
/// star's manifold is linear in reach), with x the degree's outer edge. The
/// world's reach is the holding-weighted mean of its degrees' midpoints,
/// rounded down to the 2^-64 grid. There is no seed and no partition: the
/// spacing of worlds follows from the twist growing outward. Matter that
/// never forms a world is belt. Returns each world's (reach, holding) as
/// shares of the manifold, and the belt holding; worlds and belts together
/// hold `1 - star`. A non-positive `star` or `1 - star` gives no worlds.
pub fn manifold_worlds(star: &Q, a: &Q, tide: &Q) -> (Vec<(Q, Q)>, Q) {
    let left_all = &Q::ONE - star;
    if star.signum() <= 0 || left_all.signum() <= 0 {
        return (Vec::new(), left_all.max(Q::ZERO));
    }
    let held = (&Q::ONE - a).max(Q::ZERO).min(Q::ONE);
    let step = left_all.checked_div(&Q::int(360)).unwrap_or(Q::ZERO);
    let dm = &step * &held;
    let (mut worlds, mut taken) = (Vec::new(), Q::ZERO);
    let (mut m, mut moment) = (Q::ZERO, Q::ZERO);
    for k in 0..360i64 {
        // The degree's outer edge and its midpoint, as shares of the reach.
        let x = star + &(&step * &Q::int(k + 1));
        let mid = star + &(&step * &Q::ratio(2 * k + 1, 2));
        m = &m + &dm;
        moment = &moment + &(&mid * &dm);
        let own = m.checked_div(&(star + &m)).unwrap_or(Q::ZERO);
        if m.signum() > 0 && own >= tide * &x {
            let reach = moment.checked_div(&m).unwrap_or(mid);
            taken = &taken + &m;
            worlds.push((reach, m));
            m = Q::ZERO;
            moment = Q::ZERO;
        }
    }
    (worlds, &left_all - &taken)
}
