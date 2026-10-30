//! A star's disc, read on the chain: the star tier's cut of its piece's
//! infall (`star_cut`), the worlds its manifold holds against the core's
//! twist (`manifold_worlds`), and each fragment's planetary core. Built on
//! the founding (formation.rs) and never part of it, so a change here keeps
//! the genesis.

use crate::circuit::Bead;
use crate::formation::*;
use xyl_core::Q;

/// A world of a star's disc: its reach and holding (shares of the infall),
/// the heat at its place, and its planetary core's cut - the share of its
/// matter the core holds and the lightest kind held (rocky, giant and
/// intermediate follow from these; there are no type labels).
#[derive(Clone, Debug)]
pub struct World {
    /// Reach from the star (share of the infall).
    pub reach: Q,
    /// Holding (share of the infall).
    pub holding: Q,
    /// Heat at its place, a(r) = delta_s(r) a_disc: the star's light share at
    /// its reach, delta_s(r) = star / (r + star), the vacuum taking the rest.
    pub heat: Q,
    /// The share of its matter its core holds; the rest rejoins the disc's gas.
    pub kept: Q,
    /// The lightest kind held (None: nothing held).
    pub lightest: Option<u32>,
}

/// A lit bead's infall and its star's cut, read on the chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StarCut {
    /// The share of the infall its star holds.
    pub star: Q,
    /// The galaxy's tide at the pinch, `H / (R + H)`: the core-star tie.
    pub tide: Q,
    /// The disc's agitation: the heat the pulse leaves on the disc, the ring
    /// the bell brings at the star per echo, swung by the inbound echo and
    /// taken by the disc's grip at its rim, over what the disc sheds per
    /// ring (its released kinds' open frames and the vacuum's halving).
    pub a_disc: Q,
    /// The infall's extent (galaxy rings).
    pub infall: Q,
    /// Each infall's holding (share of the galaxy's grains).
    pub cloud: Q,
    /// The turning the infall brings into the cut: the galaxy's shear across
    /// its extent, `H w / (R (R + w))`.
    pub spin: Q,
    /// The heat the cut was read at: the pulse's turning at the star per
    /// echo, in hydrogen units (atoms::unpaired_hot).
    pub heat: Q,
    /// The over-pressure the star's burn adds, `1 + burn per echo`.
    pub p0: Q,
}

/// The star tier's cut of a lit bead's infall (`hold_at_heat` on its kinds
/// at the throat's push): the infall collapses carrying the galaxy's shear
/// across its extent as its turning (`add`), is heated by the pulse's
/// turning at the star per echo (`heat`, hydrogen units) and pushed by the
/// star's burn as over-pressure (`p0 = 1 + burn per echo`); the star holds
/// what that leaves held. `turning` is the pulse's turning per echo at the
/// star's reach about the universal core, `burn` its burn per echo (stars.rs).
/// None where the bead never lights.
pub fn star_cut(
    bead: &Bead,
    unit: &Q,
    held: &Q,
    swing: &Q,
    turning: &Q,
    burn: &Q,
) -> Option<StarCut> {
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
    let p0 = &Q::ONE + burn;
    let cut = hold_at_heat(&c.mix, &throat_push(), &spin, turning, &p0);
    let star = cut.held;
    // the disc's heat as the cooling reads a piece's: the ring per echo at
    // the star, swung, taken by the disc's grip at its rim (its holding over
    // its extent plus its holding), over what the disc sheds per ring
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
    })
}

/// A star's worlds on its manifold (the anchor chain: the star a zero point,
/// its manifold the arc its own pull holds against the core). In the
/// manifold's own units (its reach 1, line holding 1 across it), the star
/// holds the inner `star` share, its gathering; the rest is read degree by
/// degree outward to the reach, 360 degrees in all. Each degree adds the
/// matter its heat lets be held, `1 - a` of the line. A world forms where the
/// matter gathered since the last world holds its own region against the
/// core's twist at that degree: its deficit share of the circle,
/// `m / (star + m)`, at least the twist, `tide x` (twist is the deficit, and
/// the core's deficit across the star's manifold is linear in reach). There
/// is no seed and no partition: where worlds stand and how far apart they
/// are follows from the twist growing outward. What never holds its own
/// region is belt. Returns each world's (reach, holding) as shares of the
/// manifold, and the belts; the books close: the worlds and belts hold
/// `1 - star`.
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
        // the degree's outer edge and its middle, as shares of the reach
        let x = star + &(&step * &Q::int(k + 1));
        let mid = star + &(&step * &Q::new(2 * k + 1, 2).unwrap_or(Q::ZERO));
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

/// A star's disc and its worlds: the worlds its manifold holds against the
/// core's twist (`manifold_worlds`), each cut by its planetary core on the disc's
/// mix (`rest`). Provisional: every fragment takes the disc's mix; a
/// per-reach composition is not yet modelled.
pub fn star_disc(star: &Q, tide: &Q, rest: &[(u32, Q)]) -> (Vec<World>, Q) {
    let mid_disc = half(&(star + &Q::ONE));
    let rate = mid_disc.checked_div(&(&mid_disc + star)).unwrap_or(Q::ZERO);
    // the disc is heated by the face the galaxy leaves unlocked at the star
    let a_disc = &(&Q::ONE - tide) * &rate;
    let (frags, belts) = manifold_worlds(star, &a_disc, tide);
    let worlds = frags
        .into_iter()
        .map(|(reach, holding)| world(star, &a_disc, rest, reach, holding))
        .collect();
    (worlds, belts)
}

/// The planetary core of a disc fragment: in its own units it holds one ring
/// as its state, its matter its holding. The core pushes by EM repulsion on
/// open frames at full amplitude, p = 1/2. The place's heat adds to every
/// kind's carried spin as a turning: the star's light share at its reach,
/// delta_s(r) = star / (r + star), of the disc's agitation (the vacuum takes
/// 1 - delta_s).
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
