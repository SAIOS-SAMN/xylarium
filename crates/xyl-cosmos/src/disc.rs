//! A star's disc: the star tier's cut of a lit bead's infall, the worlds its
//! manifold holds against the galaxy core's twist, and each world's
//! planetary core.
//!
//! Reads `circuit::Bead` and the formation helpers (`cooling`, `lights`,
//! `hold_at_heat`, `throat_push`, `vacuum_halving`). All quantities are exact
//! `Q`: reaches and holdings are shares of the infall or of the star's
//! manifold, extents are in galaxy rings, heat in hydrogen units.
//!
//! Outputs:
//! - `star_cut` -> `Option<StarCut>`, read in three states of the infall.
//!   Collapse: the dark cut holds `s0` within the rim `R0` (system-tier
//!   units), and the star's compactness is `delta0 = s0 c / (R0 60 + s0 c)`
//!   (`rim_share`). Ignition: the star holds `delta0` of the turning it
//!   receives per echo, its burn, which adds the over-pressure
//!   `p0 = 1 + burn`. Settled: the burning cut gives the star's held share
//!   and rim. Also the tide `H / (R + H)`, the disc's agitation `a_disc` and
//!   the infall's shear `H w / (R (R + w))`; `None` where the bead does not
//!   light.
//! - `rim_share`: a held body's deficit at its rim, `m / (R 60 + m)`.
//! - `manifold_worlds`: reads the manifold outside the star's share in 360
//!   equal degrees, each adding `1 - a` of its line; a world closes where
//!   the gathered matter's deficit `m / (star + m)` reaches the twist
//!   `tide x`. Returns each world's (reach, holding) and the belt share.
//! - `star_disc` -> worlds (`World`: reach, holding, heat, kept share,
//!   lightest held kind) and belts. Every fragment takes the disc's mix; a
//!   per-reach composition is not yet modelled.
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
    mass.checked_div(&(&rim_rg + mass)).unwrap_or(Q::ONE)
}

/// A world of a star's disc: its reach and holding (shares of the star's
/// manifold), the heat at its place, and its planetary core's cut: the share
/// of the world's matter the core holds and the lightest kind held (rocky,
/// giant and intermediate worlds follow from these; there are no type
/// labels).
#[derive(Clone, Debug)]
pub struct World {
    /// Reach from the star (share of the manifold's reach).
    pub reach: Q,
    /// Holding (share of the manifold's matter).
    pub holding: Q,
    /// Heat at the world's reach, a(r) = delta_s(r) a_disc, where
    /// delta_s(r) = star / (r + star) is the star's light share at that reach
    /// and the vacuum takes the rest.
    pub heat: Q,
    /// The share of the world's matter its core holds; the rest rejoins the
    /// disc's gas.
    pub kept: Q,
    /// The lightest kind held (`None`: nothing held).
    pub lightest: Option<u32>,
}

/// A lit bead's infall and the star's cut of it, read on the chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StarCut {
    /// The share of the infall the star holds.
    pub star: Q,
    /// The galaxy's tide at the pinch, `H / (R + H)`: the core-star tie.
    pub tide: Q,
    /// The disc's agitation, at most 1: the pulse's turning at the star per
    /// echo, times the inbound echo's swing and the disc's grip, over what
    /// the disc sheds per ring (the released kinds' open frames and the
    /// vacuum's halving).
    pub a_disc: Q,
    /// The infall's extent (galaxy rings).
    pub infall: Q,
    /// The infall's holding (share of the galaxy's grains).
    pub cloud: Q,
    /// The turning the infall carries into the cut: the galaxy's shear across
    /// the infall's extent, `H w / (R (R + w))`.
    pub spin: Q,
    /// The heat the cut was read at: the pulse's turning at the star per
    /// echo, in hydrogen units (`atoms::unpaired_hot`).
    pub heat: Q,
    /// The over-pressure the star's burn adds, `1 + burn per echo`.
    pub p0: Q,
    /// The held core's rim (system-tier units, `60 r_g`): the reach of the
    /// loosest kind the burning star still holds (`formation::Hold::reach`).
    /// The cut reads reaches one tier below the piece's, as the founding
    /// reads the cluster tier and the pieces the galaxy tier.
    pub rim: Q,
    /// The collapsed star's compactness before it burns: its deficit at the
    /// dark cut's rim, `m0 / (R0 + m0)` with `m0 = s0 c` (core masses, equal
    /// to `r_g`) and `R0` the dark rim in `r_g`. Its burn per echo is this
    /// share of the received turning.
    pub compactness: Q,
}

/// The star tier's cut of a lit bead's infall (`hold_at_heat` on the bead's
/// kinds at the throat's push). The infall collapses carrying the galaxy's
/// shear across its extent as turning (`add`), is heated by the pulse's
/// turning at the star per echo (`heat`, hydrogen units) and is pushed by the
/// star's burn as over-pressure (`p0 = 1 + burn per echo`); the star holds
/// what remains held. `turning` is the pulse's turning per echo at the star's
/// reach about the universal core and `burn` the star's burn per echo
/// (`stars`). `None` where the bead does not light.
pub fn star_cut(bead: &Bead, unit: &Q, held: &Q, swing: &Q, turning: &Q) -> Option<StarCut> {
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
    let dark = hold_at_heat(&c.mix, &throat_push(), &spin, turning, &Q::ONE);
    let compactness = rim_share(&(&dark.held * &c.cloud), &dark.reach);
    // Ignition: the star holds `delta0` of the turning it receives per echo;
    // that held share is its burn, which adds the over-pressure
    // `p0 = 1 + burn` and releases the loosest kinds.
    let p0 = &Q::ONE + &(&compactness * turning);
    let cut = hold_at_heat(&c.mix, &throat_push(), &spin, turning, &p0);
    let star = cut.held;
    let rim = cut.reach.clone();
    // `a_disc = turning x swing x grip / shed`, at most 1, in the same form in
    // which the cooling reads a piece's heat: grip = D / (1 + D) with D the
    // disc's holding, shed = the released kinds' open frames plus ln 2 / k
    // for the vacuum's halving over k rings. A zero shed gives 1.
    let disc_holding = &Q::ONE - &star;
    let grip = disc_holding
        .checked_div(&(&Q::ONE + &disc_holding))
        .unwrap_or(Q::ZERO);
    let ln2 = xyl_core::certified::ln2(64).mid();
    let shed = &cut.rest_open
        + &vacuum_halving(&reach, &h)
            .map(|k| ln2.checked_div(&Q::int(k)).unwrap_or(Q::ZERO))
            .unwrap_or(Q::ZERO);
    let a_disc = if shed.signum() > 0 {
        (&(turning * swing) * &grip)
            .checked_div(&shed)
            .unwrap_or(Q::ONE)
            .min(Q::ONE)
    } else {
        Q::ONE
    };
    Some(StarCut {
        star,
        tide: c.tide,
        a_disc,
        infall: c.infall,
        cloud: c.cloud,
        spin,
        heat: turning.clone(),
        p0,
        rim,
        compactness,
    })
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
    let grid = |x: &Q| xyl_core::certified::down(x, 64);
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
            let reach = grid(&moment.checked_div(&m).unwrap_or(mid));
            taken = &taken + &m;
            worlds.push((reach, m));
            m = Q::ZERO;
            moment = Q::ZERO;
        }
    }
    (worlds, &left_all - &taken)
}

/// A star's disc and its worlds: the worlds the star's manifold holds against
/// the core's twist (`manifold_worlds`), each cut by its planetary core on
/// the disc's mix (`rest`), and the belt holding. Provisional: every
/// fragment takes the disc's mix; a per-reach composition is not yet
/// modelled.
pub fn star_disc(star: &Q, tide: &Q, rest: &[(u32, Q)]) -> (Vec<World>, Q) {
    let mid_disc = half(&(star + &Q::ONE));
    let rate = mid_disc.checked_div(&(&mid_disc + star)).unwrap_or(Q::ZERO);
    // `a_disc = (1 - tide) x mid / (mid + star)`, mid = (star + 1) / 2: the
    // share the galaxy's tide leaves unlocked at the star.
    let a_disc = &(&Q::ONE - tide) * &rate;
    let (frags, belts) = manifold_worlds(star, &a_disc, tide);
    let worlds = frags
        .into_iter()
        .map(|(reach, holding)| world(star, &a_disc, rest, reach, holding))
        .collect();
    (worlds, belts)
}

/// The planetary core of a disc fragment: in the fragment's units it holds
/// one ring as its state and its matter as its holding. The core pushes by
/// EM repulsion on open frames at the throat's push, p = 1/2 (`throat_push`). The
/// place's heat, delta_s(r) a_disc with delta_s(r) = star / (r + star) the
/// star's light share at the reach (the vacuum takes 1 - delta_s), adds to
/// every kind's carried spin as turning; the core itself is cold (heat 0,
/// no over-pressure).
fn world(star: &Q, a_disc: &Q, mix: &[(u32, Q)], reach: Q, holding: Q) -> World {
    let light = star.checked_div(&(&reach + star)).unwrap_or(Q::ZERO);
    let heat = &light * a_disc;
    let c = hold_at_heat(mix, &throat_push(), &heat, &Q::ZERO, &Q::ONE);
    World {
        reach,
        holding,
        heat,
        kept: c.held,
        lightest: c.lightest,
    }
}
