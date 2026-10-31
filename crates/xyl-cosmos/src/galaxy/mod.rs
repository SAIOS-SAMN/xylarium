//! The home galaxy and its homologues: their structure read from the
//! founding, and their sample stars as bodies on orbits about their cores.
//!
//! Inputs: `circuit::anchor` (rim, bell) and the founding's beads and
//! lighting (`cores::substrate_for`), `cores::shape` for each galactic core,
//! `chem::Chemistry::birth_along`, the turning law (`turning`),
//! `manifold::reach`, `disc::star_cut`, and `xyl_core::measure`. Lengths are
//! galaxy rings (the galaxy tier's unit, 60^5 r_g, `Tier::Galaxy`), masses
//! core masses (the universal core is 1), times rings after the Big Bang
//! (one ring one bell, `circuit::anchor`), rates radians per ring.
//!
//! - [`Structure`]: each bead is a ring of the galaxy, its span in galaxy
//!   rings, its state the ring's share of the galaxy's holding. The core's
//!   reach ([`core_reach`]) is one galaxy ring, where its deficit
//!   `1 / (r + 1)` is one half; the nucleus ([`nucleus_reach`]) is the
//!   innermost ring; the reference ring ([`r0`]) holds half the holding.
//!   [`density`] (crowding against `r0`), [`inside_mass`] and the crowded
//!   layer's half-thickness ([`layer`]) are read from it. Built once the
//!   universe is founded.
//! - [`star`]: sample star `n`, at most [`SAMPLE_CAP`] per galaxy plus
//!   [`NUCLEUS_SAMPLE`] from [`NUCLEUS_BASE`], the galactic core in the high
//!   bits ([`core_of`], [`in_galaxy`]). Placement uses exact irrational
//!   rotations of the index (golden, silver, plastic, sqrt 3 - 1 turns), the
//!   phyllotaxis pattern of magnetised drops (Douady and Couder): radius along the rings'
//!   cumulative holding, phase, and for bulge and nucleus a tilt holding
//!   that share of S^360's measure. Birth (first light, then along the
//!   ring's star formation; `None` where the ring never lights), mass
//!   ([`imf`]: the share `x` of the piece's star mass `m_p = s c`, the
//!   share of the piece's infall the star tier's cut holds, drawn by the
//!   deficit form `x / (x + b)`, `b = d / (1 + d)` from the ring's
//!   crowding), companion and vertical swing come from the star's `Stream`. The turning rate is the
//!   turning law over the mass inside; a homologue's lengths scale by its
//!   shape's scale and its masses by its hole ratio.
//! - [`Star::bearing`] and [`Star::place`]: position at `t` from the
//!   phase (turns) and half-angle plane frame; places are on the universe's
//!   3-sphere (`chain::Place`). [`disc_frame`] gives the [`DiscFrame`] of a
//!   star's disc; [`vertical`] its swing through the plane and exposure,
//!   derived from the turning law, the crowding and the ring structure.
//!
//! Boundary (removed as the layers above are converted): [`kpc_of_rings`],
//! [`rings_of_kpc`] and their dyadic forms convert galaxy rings for the
//! layers still in kpc; [`suns_of_core_masses`] and [`core_masses_of_suns`]
//! convert masses for the layers that read star masses in suns (systems,
//! worlds, life, the kernel); [`rim_kpc`] and [`radius_kpc`] read the anchor in kpc for
//! `chain` and the display.
//!
//! Terms: galaxy ring, manifold (docs/glossary.md).

mod frame;
mod star;
#[cfg(test)]
mod tests;
mod vertical;

pub use frame::{
    disc_frame, first_star_compactness, first_star_mass, piece_rim, piece_star_mass, DiscFrame,
};
pub(crate) use star::{golden_turns, node_of, plastic_turns, tilt_of};
pub use star::{imf, star, Class, Orbit, Star};
pub use vertical::{layer, vertical, vertical_frequency, Vertical};

use crate::bang::BigBang;
use xyl_core::certified::Encl;
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// The grid a drawn value is rounded to (as in the Big Bang).
const GRID_BITS: u32 = 40;

/// The home galaxy's structure, read from the founding. The home galaxy is
/// the core's matter, so its structure is the founding's cascade read one
/// tier down: its rings are the beads, with reaches and spans in the galaxy
/// tier's unit (60^5 r_g) instead of the cluster's, and each ring's holding
/// is the bead's state. The core holds the universal state, 1, so the
/// galaxy's whole holding is the core's and the core's deficit at reach r is
/// 1 / (r + 1). Masses are core masses: a ring holds its state over the
/// whole holding. Cached once per process after the universe is founded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Structure {
    /// Each ring, inner first: inner and outer edge (galaxy rings), state
    /// (the ring's share of the galaxy's holding), state per galaxy ring of
    /// span, and the core's deficit at the ring's reach.
    pub rings: Vec<(Q, Q, Q, Q, Q)>,
    /// The holding of all rings: the founding unit, equal to the core's state.
    pub holding: Q,
    /// The innermost ring's outer edge (galaxy rings): the nucleus.
    pub nucleus: Q,
    /// The reference ring (galaxy rings): the reach holding half the
    /// galaxy's holding.
    pub r0: Q,
    /// State per galaxy ring at the reference ring.
    pub lambda0: Q,
}

fn structure() -> std::sync::Arc<Structure> {
    static S: std::sync::OnceLock<std::sync::Arc<Structure>> = std::sync::OnceLock::new();
    // Cached only once the universe is founded (rim > 0): a structure built
    // before the founding is empty and must not be kept.
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
        let rings: Vec<(Q, Q, Q, Q, Q)> = beads
            .iter()
            .map(|(from, to, state, reach)| {
                let lambda = state.checked_div(&(to - from)).unwrap_or(Q::ZERO);
                let tide = Q::ONE.checked_div(&(reach + &Q::ONE)).unwrap_or(Q::ONE);
                (from.clone(), to.clone(), state.clone(), lambda, tide)
            })
            .collect();
        let holding = rings.iter().fold(Q::ZERO, |a, r| &a + &r.2);
        let nucleus = rings.first().map(|r| r.1.clone()).unwrap_or(Q::ZERO);
        // The reference ring: where the cumulative holding crosses one half,
        // interpolated linearly within the crossing ring.
        let target = &holding * &Q::ratio(1, 2);
        let mut cum = Q::ZERO;
        let mut r0 = rings.last().map(|r| r.1.clone()).unwrap_or(Q::ONE);
        let mut lambda0 = rings.last().map(|r| r.3.clone()).unwrap_or(Q::ONE);
        for r in &rings {
            let next = &cum + &r.2;
            if next >= target {
                let within = (&target - &cum).checked_div(&r.2).unwrap_or(Q::ZERO);
                r0 = &r.0 + &(&(&r.1 - &r.0) * &within);
                lambda0 = r.3.clone();
                break;
            }
            cum = next;
        }
        std::sync::Arc::new(Structure {
            rings,
            holding,
            nucleus,
            r0,
            lambda0: lambda0.max(Q::ZERO),
        })
    };
    if crate::circuit::anchor().rim.signum() > 0 {
        let _ = S.set(built.clone());
    }
    built
}

/// The ring of the structure containing reach `r` (galaxy rings): beyond the
/// rim the outermost ring, inside the innermost ring's inner edge the
/// innermost. `None` only for an empty structure.
fn ring_at<'a>(st: &'a Structure, r: &Q) -> Option<&'a (Q, Q, Q, Q, Q)> {
    st.rings
        .iter()
        .find(|k| &k.0 <= r && r < &k.1)
        .or_else(|| st.rings.last().filter(|k| r >= &k.1))
        .or_else(|| st.rings.first())
}

/// The reference ring (galaxy rings): the reach inside which half the
/// galaxy's holding lies; chemistry and crowding are read against it.
pub fn r0() -> Q {
    structure().r0.clone()
}
/// The home galaxy's rim (kpc), from the founding (`circuit::anchor`).
/// Boundary and display: read by `chain` in kpc. In galaxy rings the rim is
/// the anchor's `rim`.
pub fn rim_kpc() -> Q {
    crate::circuit::anchor().home_rim_kpc
}
/// The universe's curvature radius (kpc): the founding's rim over pi
/// (`circuit::anchor`). Boundary and display, as [`rim_kpc`].
pub fn radius_kpc() -> Q {
    crate::circuit::anchor().radius_kpc
}
/// The core's reach (galaxy rings): out to where the core's deficit
/// `1 / (r + 1)` is at least one half, one galaxy ring; a star within it is
/// in the core.
pub fn core_reach() -> Q {
    Q::ONE
}
/// The nucleus's reach (galaxy rings): the innermost lit ring's outer edge.
pub fn nucleus_reach() -> Q {
    structure().nucleus.clone()
}

/// Kiloparsecs per galaxy ring (`circuit::anchor().galaxy_kpc`, through the
/// SI seam). Boundary.
fn kpc_per_ring() -> Q {
    crate::circuit::anchor().galaxy_kpc
}
/// A reach in galaxy rings, in kpc. Boundary for the layers still in kpc
/// (chemistry, worlds, life, minds, the kernel) and the display; zero before
/// a founding.
pub fn kpc_of_rings(rings: &Q) -> Q {
    rings * &kpc_per_ring()
}
/// A reach in kpc, in galaxy rings. Boundary, as [`kpc_of_rings`]; zero
/// before a founding.
pub fn rings_of_kpc(kpc: &Q) -> Q {
    kpc.checked_div(&kpc_per_ring()).unwrap_or(Q::ZERO)
}
/// [`kpc_of_rings`] in dyadics. Boundary.
pub fn kpc_of_rings_dy(rings: Dy) -> Dy {
    rings * Dy::from_q(&kpc_per_ring())
}
/// [`rings_of_kpc`] in dyadics. Boundary; zero before a founding.
pub fn rings_of_kpc_dy(kpc: Dy) -> Dy {
    let k = Dy::from_q(&kpc_per_ring());
    if k.signum() <= 0 {
        return Dy::ZERO;
    }
    kpc / k
}
/// A mass in core masses, in suns, through the declared core mass
/// (`package::hole_mass`). Boundary for the layers that read star masses in
/// suns.
pub fn suns_of_core_masses(m: &Q) -> Q {
    m * &crate::package::hole_mass()
}
/// A mass in suns, in core masses. Boundary, as [`suns_of_core_masses`].
pub fn core_masses_of_suns(suns: &Q) -> Q {
    suns.checked_div(&crate::package::hole_mass())
        .unwrap_or(Q::ZERO)
}
/// The sample's size (a resolution choice): each sample star stands for an
/// equal share of the galaxy's stars.
pub const SAMPLE_CAP: u64 = 65_536;

/// The galaxy's stellar mass (core masses). The founding is in the core's
/// units (the parent's mass, 1); the home galaxy holds the universal state,
/// 1, and its stars are the share whose gas lights (`Lighting::lit`), so the
/// stellar mass is the lit share of one core mass. Cached once the universe
/// is founded; zero before the founding.
pub fn stellar_mass() -> Q {
    static M: std::sync::OnceLock<Q> = std::sync::OnceLock::new();
    if let Some(m) = M.get() {
        return m.clone();
    }
    let anchor = crate::circuit::anchor();
    let Some(sub) = crate::cores::substrate_for(&anchor.spin) else {
        return Q::ZERO;
    };
    let m = sub.lighting.lit.clone();
    if anchor.rim.signum() > 0 {
        let _ = M.set(m.clone());
    }
    m
}

/// The stellar mass within the nucleus (core masses): the mass inside the
/// nucleus's reach less the universal core, 1.
pub fn nucleus_mass() -> Q {
    inside_mass(&nucleus_reach()) - &Q::ONE
}

/// The nucleus: the innermost lit ring about the hole, the only strongly
/// curved region. It is sampled separately and more finely than its mass
/// share would give: its stars have indices from `NUCLEUS_BASE`, each standing
/// for an equal share of the nucleus's mass.
pub const NUCLEUS_BASE: u64 = 1 << 40;
/// Number of sample stars drawn for the nucleus, indexed from [`NUCLEUS_BASE`].
pub const NUCLEUS_SAMPLE: u64 = 512;

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

/// Crowding at reach `r` (galaxy rings): the holding per galaxy ring of the
/// ring containing `r` over that of the reference ring. The value is
/// constant within a ring; beyond the rim the outermost ring is used,
/// elsewhere the innermost.
pub fn density(r: &Q) -> Encl {
    let st = structure();
    let lambda = ring_at(&st, r).map(|k| k.3.clone()).unwrap_or(Q::ZERO);
    Encl::exact(
        lambda
            .checked_div(&st.lambda0.clone().max(Q::ratio(1, 1_000_000_000)))
            .unwrap_or(Q::ZERO),
    )
}

/// The birth time (rings after the Big Bang, a whole number) of the stars at
/// reach `r` (galaxy rings) of the home galaxy. The galaxy ring's gas is one
/// piece of the founding, cooling from the bounce, and collapses once when
/// the piece lights (`formation::Lighting::cools_in_at`, in galaxy rings), so
/// every star of a galaxy ring is born at the piece's first light, counted
/// in rings of time (at least one). `None` where the piece never lights, so
/// no star is placed there.
pub fn birth(r: &Q) -> Option<Q> {
    let anchor = crate::circuit::anchor();
    let sub = crate::cores::substrate_for(&anchor.spin)?;
    let bells = sub.lighting.cools_in_at(r)?;
    Some(Q::int(bells.max(1)))
}

/// The share of the galaxy's holding inside `r` (galaxy rings): the rings'
/// states within `r` (a ring counted by the share of its span inside) over
/// the whole holding.
fn holding_within(st: &Structure, r: &Q) -> Q {
    let r = r.clone().max(Q::ZERO);
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
    within
        .checked_div(&st.holding.clone().max(Q::ratio(1, 1_000_000_000)))
        .unwrap_or(Q::ZERO)
}

/// The home galaxy's mass inside `r` galaxy rings (core masses):
/// `M(<r) = 1 + H(<r) / H`, the universal core's mass 1 plus the rings'
/// states within `r` (a ring counted by the share of its span inside) over
/// the whole holding `H`. This mass's deficit sets a star's turning rate
/// there (`turning`).
pub fn inside_mass(r: &Q) -> Q {
    &Q::ONE + &holding_within(&structure(), r)
}

/// As `inside_mass`, in dyadics (for fast per-pair evaluation).
pub fn inside_mass_dy(r: Dy) -> Dy {
    Dy::from_q(&inside_mass(&r.to_q()))
}

/// The mass of the ring containing `r` (core masses): its state over the
/// whole holding, the ring's share of the galaxy as counted by
/// [`inside_mass`]. Zero for an empty structure.
pub fn ring_mass(r: &Q) -> Q {
    let st = structure();
    ring_at(&st, r)
        .and_then(|k| {
            k.2.checked_div(&st.holding.clone().max(Q::ratio(1, 1_000_000_000)))
        })
        .unwrap_or(Q::ZERO)
}

/// The reach (galaxy rings) at which the cumulative holding of the rings
/// reaches the share `u` of the total (`u` clamped to [0, 1]), interpolated
/// linearly within a ring. Only rings whose outer edge is at most `up_to`
/// count.
fn reach_at(u: Dy, up_to: Option<&Q>) -> Dy {
    let st = structure();
    let rings: Vec<&(Q, Q, Q, Q, Q)> = st
        .rings
        .iter()
        .filter(|k| up_to.is_none_or(|top| &k.1 <= top))
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
/// nucleus (from `NUCLEUS_BASE`).
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
