//! The home galaxy and its homologues: their structure read from the
//! founding, and their sample stars as bodies on orbits about their cores.
//!
//! Inputs: `circuit::anchor` (rim, home galaxy rim in kpc, bell) and the
//! founding's beads and lighting (`cores::substrate_for`), `cores::shape`
//! for each galactic core, `chem::Chemistry::birth_along`, the turning law
//! (`turning`), `manifold::reach`, `disc::star_cut`, and
//! `xyl_core::measure`. Lengths are kpc, times Gyr after the Big Bang,
//! masses suns via the one bridge `package::hole_mass`.
//!
//! - [`Structure`]: each bead is a ring of the galaxy, its span scaled so
//!   the founding's rim is the home galaxy's rim, its state the ring's share
//!   of the galaxy's holding. The core's reach ([`core_kpc`]) is one galaxy
//!   unit, where its deficit `1 / (r + 1)` is one half; the nucleus
//!   ([`nucleus_kpc`]) is the innermost ring; the reference ring ([`r0`])
//!   holds half the holding. [`density`] (crowding against `r0`) and
//!   [`inside_mass`] are read from it. Built once the universe is founded.
//! - [`star`]: sample star `n`, at most [`SAMPLE_CAP`] per galaxy plus
//!   [`NUCLEUS_SAMPLE`] from [`NUCLEUS_BASE`], the galactic core in the high
//!   bits ([`core_of`], [`in_galaxy`]). Placement uses exact irrational
//!   pivots of the index (golden, silver, plastic, sqrt 3 - 1 turns), as
//!   magnetised drops settle (Douady and Couder): radius along the rings'
//!   cumulative holding, phase, and for bulge and nucleus a tilt holding
//!   that share of S^360's measure. Birth (first light, then along the
//!   ring's star formation; `None` where the ring never lights), mass
//!   ([`imf`], its break heavier where rings crowd), companion and vertical
//!   swing come from the star's own `Stream`. The turning rate is the
//!   turning law over the mass inside; a homologue's lengths scale by its
//!   shape's scale and its masses by its hole ratio.
//! - [`Star::bearing`] and [`Star::place`]: position at `t` from the exact
//!   phase (turns) and half-angle plane frame; places are on the universe's
//!   3-sphere (`chain::Place`). [`disc_frame`] gives the [`DiscFrame`] of a
//!   star's disc; [`vertical`] its swing through the plane and exposure.
//!
//! Declared rulers: the IMF's bounds (0.08 to 60 suns) and break scale, the
//! vertical pull at the reference ring (75.2 rad/Gyr), the 20 km/s spread at
//! 10 Gyr and the crowded layer's half-thickness ([`layer_kpc`], 0.3 kpc).
//!
//! Terms: galaxy ring, manifold (docs/glossary.md).

mod frame;
mod star;
#[cfg(test)]
mod tests;
mod vertical;

pub use frame::{disc_frame, DiscFrame};
pub(crate) use star::{golden_turns, node_of, plastic_turns, tilt_of};
pub use star::{imf, star, Orbit, Star};
pub use vertical::{layer_kpc, vertical, vertical_frequency, Vertical};

use crate::bang::BigBang;
use crate::package::hole_mass;
use xyl_core::certified::Encl;
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// The grid a drawn value is rounded to (as in the Big Bang).
const GRID_BITS: u32 = 40;

/// The home galaxy's structure, read from the founding: the home galaxy is
/// the core's own matter, so it is the founding's cascade read one tier
/// down. Its rings are the beads, their reaches and spans in the galaxy
/// tier's unit (60^5 r_g) instead of the cluster's, their holding their
/// states; the core holds the universal state, 1, so the galaxy's whole
/// holding is its core's and the core's deficit at reach r is 1 / (r + 1).
/// In suns, the profile hangs from the one declared bridge, the core's mass.
/// Computed once per process.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Structure {
    /// Kiloparsecs per galaxy unit.
    pub kpc_per_unit: Q,
    /// Each ring, inner first: inner and outer edge (kpc), its state (its
    /// share of the galaxy's holding), its state per kpc of span, and the
    /// core's deficit at its reach.
    pub rings: Vec<(Q, Q, Q, Q, Q)>,
    /// The holding of all rings: the founding unit, the core's own state.
    pub holding: Q,
    /// The reach the core holds at least half the turn to (kpc): one galaxy
    /// unit, where its deficit 1 / (r + 1) is one half.
    pub core_kpc: Q,
    /// The innermost ring's outer edge (kpc): the nucleus.
    pub nucleus_kpc: Q,
    /// The reference ring (kpc): the reach holding half the galaxy's holding.
    pub r0_kpc: Q,
    /// State per kpc at the reference ring.
    pub lambda0: Q,
}

fn structure() -> std::sync::Arc<Structure> {
    static S: std::sync::OnceLock<std::sync::Arc<Structure>> = std::sync::OnceLock::new();
    // cached only once the universe is founded (rim > 0): a structure built
    // before the founding is empty and must not be kept
    if let Some(s) = S.get() {
        return s.clone();
    }
    let built = {
        let anchor = crate::circuit::anchor();
        let sub = crate::cores::substrate_for(&anchor.spin);
        let mut beads: Vec<(Q, Q, Q, Q)> = sub
            .as_ref()
            .map(|s| {
                s.beads
                    .iter()
                    .map(|b| {
                        (
                            b.span.0.clone(),
                            b.span.1.clone(),
                            b.state.clone(),
                            b.reach.clone(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        beads.sort_by(|x, y| x.3.cmp(&y.3));
        let kpc_per_unit = anchor
            .home_rim_kpc
            .checked_div(&anchor.rim)
            .unwrap_or(Q::ONE);
        let rings: Vec<(Q, Q, Q, Q, Q)> = beads
            .iter()
            .map(|(from, to, state, reach)| {
                let (f, t) = (from * &kpc_per_unit, to * &kpc_per_unit);
                let lambda = state.checked_div(&(&t - &f)).unwrap_or(Q::ZERO);
                let tide = Q::ONE.checked_div(&(reach + &Q::ONE)).unwrap_or(Q::ONE);
                (f, t, state.clone(), lambda, tide)
            })
            .collect();
        let holding = rings.iter().fold(Q::ZERO, |a, r| &a + &r.2);
        let half = Q::ratio(1, 2);
        let core_kpc = kpc_per_unit.clone();
        let nucleus_kpc = rings.first().map(|r| r.1.clone()).unwrap_or(Q::ZERO);
        // the reference ring: where the cumulative holding crosses one half
        let target = &holding * &half;
        let mut cum = Q::ZERO;
        let mut r0_kpc = rings.last().map(|r| r.1.clone()).unwrap_or(Q::ONE);
        let mut lambda0 = rings.last().map(|r| r.3.clone()).unwrap_or(Q::ONE);
        for r in &rings {
            let next = &cum + &r.2;
            if next >= target {
                let within = (&target - &cum).checked_div(&r.2).unwrap_or(Q::ZERO);
                r0_kpc = &r.0 + &(&(&r.1 - &r.0) * &within);
                lambda0 = r.3.clone();
                break;
            }
            cum = next;
        }
        std::sync::Arc::new(Structure {
            kpc_per_unit,
            rings,
            holding,
            core_kpc,
            nucleus_kpc,
            r0_kpc,
            lambda0: lambda0.max(Q::ZERO),
        })
    };
    if crate::circuit::anchor().rim.signum() > 0 {
        let _ = S.set(built.clone());
    }
    built
}

/// The reference ring (kpc): the reach inside which half the galaxy's
/// holding lies; chemistry and crowding are read against it.
pub fn r0() -> Q {
    structure().r0_kpc.clone()
}
/// The home galaxy's rim (kpc): where its matter ends and the first bead's
/// begins, from the founding (circuit::anchor). Its disc extends to it.
pub fn rim_kpc() -> Q {
    crate::circuit::anchor().home_rim_kpc
}
/// The universe's curvature radius (kpc): the founding's rim over pi
/// (circuit::anchor).
pub fn radius_kpc() -> Q {
    crate::circuit::anchor().radius_kpc
}
/// The core's reach (kpc): out to where the core holds at least half the
/// turn (the tide one half); a star within it is in the core.
pub fn core_kpc() -> Q {
    structure().core_kpc.clone()
}
/// The nucleus's reach (kpc): the innermost lit ring's outer edge.
pub fn nucleus_kpc() -> Q {
    structure().nucleus_kpc.clone()
}
/// The sample's size (a resolution choice): each sample star stands for an
/// equal share of the galaxy's stars.
pub const SAMPLE_CAP: u64 = 65_536;

/// The galaxy's stellar mass (suns). The founding is in the core's units
/// (the parent's mass, 1); the home galaxy is the universal state, 1 of
/// them, and its stars are the share of it whose gas lights
/// (`Lighting::lit`), so its stellar mass is the lit share of the core's
/// mass. Kept once the universe is founded; before it, zero.
pub fn stellar_mass() -> Q {
    static M: std::sync::OnceLock<Q> = std::sync::OnceLock::new();
    if let Some(m) = M.get() {
        return m.clone();
    }
    let anchor = crate::circuit::anchor();
    let Some(sub) = crate::cores::substrate_for(&anchor.spin) else {
        return Q::ZERO;
    };
    let m = &sub.lighting.lit * &hole_mass();
    if anchor.rim.signum() > 0 {
        let _ = M.set(m.clone());
    }
    m
}

/// The stellar mass within the nucleus (suns).
pub fn nucleus_mass() -> Q {
    inside_mass(&nucleus_kpc()) - &hole_mass()
}

/// The nucleus: the innermost lit ring about the hole, the only strongly
/// curved region. It is sampled separately and more finely than its mass
/// alone would give: its stars have indices from NUCLEUS_BASE, each standing
/// for an equal share of the nucleus's mass.
pub const NUCLEUS_BASE: u64 = 1 << 40;
/// Number of sample stars drawn for the nucleus, indexed from [`NUCLEUS_BASE`].
pub const NUCLEUS_SAMPLE: u64 = 512;

/// Speed of light (km/s), from the one SI seam (`package::Si`).
pub fn c_kms() -> Dy {
    Dy::from_q(&crate::package::Si::c()) / Dy::int(1000)
}

/// Which population a star belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pop {
    /// A disc star, orbiting in the disc's plane.
    Disc,
    /// A bulge star, on a tilted orbit in the bulge's ball.
    Bulge,
    /// A star of the nuclear cluster about the central hole.
    Nucleus,
}

/// The number of sample stars in a universe's galaxy: the sample's
/// resolution, [`SAMPLE_CAP`]; each stands for the stellar mass over it.
pub fn sample(_bang: &BigBang) -> u64 {
    SAMPLE_CAP
}

/// Crowding at reach `r` (kpc): the ring's holding per kpc over the
/// reference ring's, read from the structure (one within a ring).
pub fn density(r: &Q) -> Encl {
    let st = structure();
    let lambda = st
        .rings
        .iter()
        .find(|k| &k.0 <= r && r < &k.1)
        .or_else(|| st.rings.last().filter(|k| r >= &k.1))
        .or_else(|| st.rings.first())
        .map(|k| k.3.clone())
        .unwrap_or(Q::ZERO);
    Encl::exact(
        lambda
            .checked_div(&st.lambda0.clone().max(Q::ratio(1, 1_000_000_000)))
            .unwrap_or(Q::ZERO),
    )
}

/// When the stars at reach `r` (kpc) of the home galaxy are born (Gyr after
/// the Big Bang): the ring's gas is one piece of the founding, cooling from
/// the bounce, and collapses once, where its piece lights
/// (`formation::Lighting::cools_in_at`, in galaxy rings), so every star of a
/// ring is born at its piece's first light, in bells. None where the piece
/// never lights: no star stands there.
pub fn birth_gyr(r: &Q) -> Option<Q> {
    let anchor = crate::circuit::anchor();
    let sub = crate::cores::substrate_for(&anchor.spin)?;
    let kpc_per_ring = anchor
        .home_rim_kpc
        .checked_div(&anchor.rim)
        .unwrap_or(Q::ONE)
        .max(Q::ratio(1, 1_000_000_000));
    let rings = r.checked_div(&kpc_per_ring).unwrap_or(Q::ZERO);
    let bells = sub.lighting.cools_in_at(&rings)?;
    Some(&Q::int(bells.max(1)) * &anchor.bell_gyr.to_q())
}

/// The home galaxy's mass inside `r` kpc (suns): the hole plus the rings'
/// states within `r` (a ring counted by the share of its span inside) over
/// the whole holding, times the core's mass. This mass's deficit sets a
/// star's turning there (turning.rs).
pub fn inside_mass(r_kpc: &Q) -> Q {
    let st = structure();
    let r = r_kpc.clone().max(Q::ZERO);
    let within = st.rings.iter().fold(Q::ZERO, |a, k| {
        if r >= k.1 {
            &a + &k.2
        } else if r > k.0 {
            let share = (&r - &k.0).checked_div(&(&k.1 - &k.0)).unwrap_or(Q::ZERO);
            &a + &(&k.2 * &share)
        } else {
            a
        }
    });
    &hole_mass()
        + &(&within
            .checked_div(&st.holding.clone().max(Q::ratio(1, 1_000_000_000)))
            .unwrap_or(Q::ZERO)
            * &hole_mass())
}

/// As `inside_mass`, in dyadics (for fast per-pair evaluation).
pub fn inside_mass_dy(r_kpc: Dy) -> Dy {
    Dy::from_q(&inside_mass(&r_kpc.to_q()))
}

/// The reach (kpc) at which the cumulative holding of the rings reaches
/// the share `u` of the whole (`u` in [0, 1)): where a star of that share
/// of the population sits, linear within a ring.
fn reach_at(u: Dy, up_to_kpc: Option<&Q>) -> Dy {
    let st = structure();
    let rings: Vec<&(Q, Q, Q, Q, Q)> = st
        .rings
        .iter()
        .filter(|k| up_to_kpc.is_none_or(|top| &k.1 <= top))
        .collect();
    let total = rings.iter().fold(Q::ZERO, |a, k| &a + &k.2);
    if total.signum() <= 0 {
        return Dy::ZERO;
    }
    let target = &u.clamp(Dy::ZERO, Dy::ONE).to_q() * &total;
    let mut cum = Q::ZERO;
    for k in &rings {
        let next = &cum + &k.2;
        if next >= target {
            let within = (&target - &cum).checked_div(&k.2).unwrap_or(Q::ZERO);
            return Dy::from_q(&(&k.0 + &(&(&k.1 - &k.0) * &within)));
        }
        cum = next;
    }
    rings.last().map(|k| Dy::from_q(&k.1)).unwrap_or(Dy::ZERO)
}

/// The sample star indices of a home galaxy: disc and bulge first, then the
/// nucleus (from NUCLEUS_BASE).
pub fn indices(bang: &BigBang) -> impl Iterator<Item = u64> {
    (0..sample(bang)).chain(NUCLEUS_BASE..NUCLEUS_BASE + NUCLEUS_SAMPLE)
}

/// A star's index carries its galactic core in its high bits (zero for the
/// home galaxy).
pub const CORE_SHIFT: u32 = 48;

/// The galactic core index carried in the high bits of star index `n`.
pub fn core_of(n: u64) -> u32 {
    u32::try_from(n >> CORE_SHIFT).unwrap_or(0)
}

/// Star index `n` within galaxy `core`: `core` in the high bits above
/// [`CORE_SHIFT`], `n` below.
pub fn in_galaxy(core: u32, n: u64) -> u64 {
    (u64::from(core) << CORE_SHIFT) | n
}

/// The sample stars of galactic core `core`.
pub fn indices_of(bang: &BigBang, core: u32) -> impl Iterator<Item = u64> {
    indices(bang).map(move |k| in_galaxy(core, k))
}
