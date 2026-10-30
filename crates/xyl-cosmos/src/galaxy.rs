//! The home galaxy: its stars as bodies on orbits about the core.
//!
//! The core is the universal core itself (centre.rs): the home galaxy is the
//! core's own matter, so its structure is the founding's cascade read one
//! tier down ([`Structure`]). Each bead of the founding is a ring of the
//! galaxy, its reach and span in the galaxy tier's unit (60^5 r_g) and its
//! state the ring's share of the galaxy's holding; the rim is the
//! founding's, where the galaxy's matter ends and the first bead's begins;
//! the core's reach is one galaxy unit, where its deficit 1 / (r + 1) is one
//! half; the nucleus is the innermost ring; the reference ring is the reach
//! holding half the galaxy's holding, against which crowding ([`density`])
//! and chemistry are read. In suns the profile hangs from the one declared
//! bridge, the core's mass ([`hole_mass`]). Nothing of a bulge's or a disc's
//! scale is declared.
//!
//! Every ball fills by the universe's measure (xyl-core measure.rs): a galaxy
//! sits in S^360, so the share of a ball within radius r is share_360 of the
//! angle whose deficit share is r / (r + a), with no hand-chosen exponent.
//! Stars are placed by magnetic repulsion rather than independent draws:
//! equal moments settle as the golden-angle spiral (phyllotaxis, as in
//! Douady and Couder's magnetised drops), each star pivoting the golden angle
//! from the last, its share of its ball's measure pivoting by other
//! irrational turns.
//!
//! The sample: a galaxy's stars are a population of sample stars, each
//! standing for an equal share of its stellar mass: at most [`SAMPLE_CAP`]
//! (65,536, a resolution choice) scaled by the Big Bang's star-forming
//! share, with the nucleus sampled separately and more finely
//! ([`NUCLEUS_SAMPLE`]). Component, position, orbital phases, birth time and
//! mass all come from the star's own stream; a star is identified by its
//! index. Its position is wherever its orbit has carried it at the requested
//! time, and distances are arcs on the universe's closed 3-sphere (curvature
//! radius the founding's rim over pi).
//!
//! - A star is born when its ring's piece of the founding's gas lights
//!   ([`birth_gyr`]); where it never lights, no star stands.
//! - Its mass comes from the IMF ([`imf`]), whose break moves heavier in
//!   crowded rings: the core makes heavy stars, many ending as black holes.
//!
//! Declared rulers that remain here: the IMF's bounds and break, and the
//! vertical orbit's scales (the pull at the reference ring, the 20 km/s
//! spread at 10 Gyr, and the crowded layer's half-thickness
//! [`layer_kpc`]).

use crate::bang::BigBang;
use crate::chain::Place;
use crate::package::hole_mass;
use xyl_core::certified::{cos_sin_turns, down, pi, Encl};
use xyl_core::dyadic::Dy;
use xyl_core::measure;
use xyl_core::{Stream, Q};

/// The grid a drawn value is rounded to (as in the Big Bang).
const GRID_BITS: u32 = 40;

fn q(n: i64, d: i64) -> Q {
    Q::new(n, d).unwrap_or(Q::ZERO)
}

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
    // kept only once the universe is founded (its rim set): a reading taken
    // before the founding builds an empty structure, never caches it
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
        let half = q(1, 2);
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
            .checked_div(&st.lambda0.clone().max(q(1, 1_000_000_000)))
            .unwrap_or(Q::ZERO),
    )
}

/// The chain's frame for a star's disc at reach `r` (kpc) of the home
/// galaxy: its piece of the founding, the infall's extent there (kpc), the
/// share of the infall the star holds, the galaxy's tide at the pinch (the
/// core-star tie) and the disc's agitation (`disc::star_cut`). None
/// where the piece never lights. Read once per piece and held.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscFrame {
    /// The piece's index among the founding's beads.
    pub piece: usize,
    /// The infall's extent (kpc): the star-forming cloud's.
    pub infall_kpc: Q,
    /// The star's own gathering extent (kpc): its mass over the infall's line
    /// density, never more than the infall itself.
    pub extent_kpc: Q,
    /// The star's manifold reach (kpc), its worlds' outer edge: the arc of
    /// its circle about the core its own pull holds (`manifold::reach`, the
    /// deficit share `m / (M(<r) + m)` of `2 pi r`).
    pub reach_kpc: Q,
    /// The star's cut of the infall (`disc::star_cut`).
    pub cut: crate::disc::StarCut,
    /// The share of its manifold the star holds: its gathering over its
    /// reach, at most 1 (a star whose gathering fills its manifold has no
    /// matter left for worlds).
    pub star_share: Q,
    /// The galaxy's tide at the pinch, `H / (R + H)`.
    pub tide: Q,
    /// The disc's agitation, the face the galaxy leaves unlocked at the star.
    pub a_disc: Q,
}

/// The disc's frame at reach `r` (kpc) of the home galaxy for a star of
/// `mass` suns (see [`DiscFrame`]). The cut is read once per piece and held;
/// the star's gathering extent is its own.
pub fn disc_frame(r: &Q, mass: &Q) -> Option<DiscFrame> {
    static HELD: std::sync::OnceLock<
        std::sync::Mutex<std::collections::BTreeMap<usize, Option<crate::disc::StarCut>>>,
    > = std::sync::OnceLock::new();
    let anchor = crate::circuit::anchor();
    let sub = crate::cores::substrate_for(&anchor.spin)?;
    let kpc_per_ring = anchor
        .home_rim_kpc
        .checked_div(&anchor.rim)
        .unwrap_or(Q::ONE)
        .max(q(1, 1_000_000_000));
    let rings = r.checked_div(&kpc_per_ring).unwrap_or(Q::ZERO);
    let (piece, record) = sub.lighting.piece_at(&rings)?;
    record.2?;
    let map = HELD.get_or_init(Default::default);
    let cut = match map.lock().ok().and_then(|m| m.get(&piece).cloned()) {
        Some(c) => c,
        None => {
            let bead = sub.beads.get(piece)?;
            let swing = &Q::ONE + &sub.inbound_mean(sub.echo()).to_q();
            // the pulse's turning at the star per echo (hydrogen units), and
            // the star's burn per echo as over-pressure, at the star's reach
            // about the universal core
            let reach = crate::stars::reach_rings(Dy::from_q(r));
            let turning = (crate::stars::ring_per_bell(reach) * Dy::int(2)).to_q();
            let burn = (crate::stars::burn_per_bell(reach) * Dy::int(2)).to_q();
            let c = crate::disc::star_cut(
                bead,
                &sub.unit(),
                &sub.held_share(),
                &swing,
                &turning,
                &burn,
            );
            if let Ok(mut m) = map.lock() {
                m.insert(piece, c.clone());
            }
            c
        }
    };
    let cut = cut?;
    // the star's gathering: its mass in holding units (the galaxy's grains
    // hold one unit, stellar_mass suns) over the infall's holding per ring
    let m_s = mass.checked_div(&stellar_mass())?;
    let lambda = cut
        .cloud
        .checked_div(&cut.infall.clone().max(q(1, 1_000_000_000)))?;
    let extent = m_s
        .checked_div(&lambda.max(q(1, 1_000_000_000_000)))?
        .min(cut.infall.clone());
    let extent_kpc = &extent * &kpc_per_ring;
    // the star's manifold: the arc of its circle about the core (galaxy
    // rings) its own pull holds against the mass inside its orbit
    let inside = inside_mass(r);
    let ratio = mass.checked_div(&inside.max(q(1, 1_000_000)))?;
    let reach = crate::manifold::reach(&Encl::exact(rings.clone()), &ratio).mid();
    let star_share = extent
        .checked_div(&reach.clone().max(q(1, 1_000_000_000_000)))
        .unwrap_or(Q::ONE)
        .min(Q::ONE);
    Some(DiscFrame {
        piece,
        infall_kpc: &record.3 * &kpc_per_ring,
        extent_kpc,
        reach_kpc: &reach * &kpc_per_ring,
        star_share,
        tide: cut.tide.clone(),
        a_disc: cut.a_disc.clone(),
        cut,
    })
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
        .max(q(1, 1_000_000_000));
    let rings = r.checked_div(&kpc_per_ring).unwrap_or(Q::ZERO);
    let bells = sub.lighting.cools_in_at(&rings)?;
    Some(&Q::int(bells.max(1)) * &anchor.bell_gyr.to_q())
}

/// 1 / (2 pi), enclosed once: converts the vertical swing's radians per Gyr
/// to turns (the plane's restoring pull is not yet expressed in angles).
fn inv_two_pi() -> &'static Encl {
    static K: std::sync::OnceLock<Encl> = std::sync::OnceLock::new();
    K.get_or_init(|| {
        let p = pi(100);
        let two = Q::int(2);
        Encl {
            lo: Q::ONE.checked_div(&(&p.hi * &two)).unwrap_or(Q::ZERO),
            hi: Q::ONE.checked_div(&(&p.lo * &two)).unwrap_or(Q::ZERO),
        }
        .outward(100)
    })
}

/// A star's galactic manifold from the core (chain.rs): its stretch, turn and
/// plane. Every angle is held as an exact fraction of a turn, and the rate
/// comes from the turning law, so the phase is exact at every t.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Orbit {
    /// Orbit radius (kpc) and turning rate (degrees per Gyr, exact).
    pub radius: Dy,
    /// Turning rate (degrees per Gyr, exact), from the turning law.
    pub rate: Dy,
    /// Orbital phase at the Big Bang (turns, exact).
    pub turns0: Dy,
    /// The orbital plane as exact half-angles (c, s): the tilt about the line
    /// of nodes and the node about the core's axis; both (1, 0) for a disc
    /// star, whose plane is the disc's.
    pub tilt: (Dy, Dy),
    /// Node of the plane about the core's axis, as a half-angle (c, s).
    pub node: (Dy, Dy),
    /// A disc star's vertical oscillation: the draw that sets its speed, and
    /// its phase (turns, exact).
    pub swing: Option<(Dy, Dy)>,
}

impl Orbit {
    /// The plane's frame: the tilt about x, then the node about the core's
    /// axis, composed exactly by half-angles.
    pub fn frame(&self) -> xyl_core::rotor::Rotor {
        use xyl_core::rotor::{Half, Rotor};
        let h = |p: (Dy, Dy)| Half {
            c: p.0.to_q(),
            s: p.1.to_q(),
        };
        Rotor::about(2, &h(self.node)).then_after(&Rotor::about(0, &h(self.tilt)))
    }

    /// Orbital phase at `t` Gyr (turns): initial phase plus turning, exact.
    pub fn turns_at(&self, t: Dy) -> Dy {
        self.turns0 + self.rate * t / Dy::int(360)
    }
}

/// A sample star, as drawn at the Big Bang.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Star {
    /// Index in the galaxy's sample.
    pub n: u64,
    /// Home-equivalent distance from the core (kpc), exact; sets the gas
    /// conditions it formed in.
    pub r: Q,
    /// Birth time (Gyr after the Big Bang), rounded to the grid.
    pub born: Q,
    /// Mass (suns), rounded to the grid.
    pub mass: Q,
    /// A binary's companion mass (suns); zero for a single star.
    pub companion: Q,
    /// The star's galactic manifold: radius, turning, phase and plane.
    pub orbit: Orbit,
    /// Population, and how many suns of it this sample star represents.
    pub pop: Pop,
    /// Stellar mass this sample star represents (suns).
    pub stands_for: Dy,
    /// The star's galactic core index, and its galaxy's length scale over the
    /// home's (`r` above is the home-equivalent radius used for gas and
    /// chemistry; `orbit.radius` is the actual radius).
    pub core: u32,
    /// The galaxy's length scale over the home galaxy's (1 for the home galaxy).
    pub scale: Dy,
    /// The galaxy's core shape: its bearing on the universe's sphere, its
    /// plane and its circulation about the throat.
    pub galaxy: crate::cores::Shape,
    /// The galaxy's hole mass over the home's; scales the turning masses.
    pub mass_ratio: Dy,
}

impl Star {
    /// The star's reach about its own core (galaxy rings): a star relates to
    /// the core it is anchored on, in the lowest common tier (the containment
    /// chain), so its reach is its own radius in its galaxy
    /// (stars::reach_rings); a galaxy's own reach about the universal core is
    /// one tier up and is never added to it as if the two lay on one line.
    pub fn reach_rings(&self) -> Dy {
        crate::stars::reach_rings(self.orbit.radius)
    }

    /// The galaxy's turning rate (degrees per Gyr) at radius `own_kpc` in the
    /// star's galaxy: the turning law over the scaled home-equivalent mass
    /// inside, as for the star's own rate (used for constellation shear).
    pub fn rate_at_dy(&self, own_kpc: Dy) -> Dy {
        let scale = self.scale.max(Dy::sci(1, -12));
        crate::turning::turning_kpc_dy(inside_mass_dy(own_kpc / scale) * self.mass_ratio, own_kpc)
    }

    /// Whether the star is within the core radius (bulge or inner disc).
    pub fn in_core(&self) -> bool {
        self.r <= core_kpc()
    }

    /// Bearing from the star's galactic core at `t` Gyr (galaxy units,
    /// excluding the core's own place): reach (orbit radius), unit direction
    /// (the plane's frame applied to the phase; exact half-angles), and climb
    /// off the plane (a disc star's vertical swing).
    pub fn bearing(&self, t: Dy) -> (Q, [Q; 3], Q) {
        let o = &self.orbit;
        let (c, s) = cos_sin_turns(&Encl::exact(o.turns_at(t).to_q()));
        let dir = o
            .frame()
            .apply(&[c, s, Encl::exact(Q::ZERO)])
            .map(|x| x.mid());
        let climb = match o.swing {
            Some((u, phase)) => {
                // the home-equivalent star's swing, scaled to its galaxy
                let age = (t - Dy::from_q(&self.born)).max(Dy::ZERO);
                let r_home = Dy::from_q(&self.r);
                let v = vertical(r_home, age, u, phase);
                let swing = Encl::exact(phase.to_q())
                    .add(&Encl::exact((vertical_frequency(r_home) * t).to_q()).mul(inv_two_pi()));
                Encl::exact((v.amp_kpc * self.scale).to_q())
                    .mul(&cos_sin_turns(&swing).1)
                    .mid()
            }
            None => Q::ZERO,
        };
        let k = crate::chain::kpc();
        (&o.radius.to_q() * &k, dir, &climb * &k)
    }

    /// The star's place at `t` Gyr on its galaxy's sphere, with its core on
    /// the universe's sphere of radius `r`.
    pub fn place(&self, t: Dy, r: &Q) -> Place {
        let (reach, dir, climb) = self.bearing(t);
        let core = self.galaxy.place_at(t, r);
        Place::in_galaxy(
            reach,
            dir,
            climb,
            self.core,
            &crate::chain::galaxy_radius(&self.scale.to_q()),
            core,
        )
    }

    /// The vertical orbit at an age (None for a bulge or nucleus star, which
    /// has no plane of its own).
    pub fn vertical_at(&self, age: Dy) -> Option<Vertical> {
        self.orbit
            .swing
            .map(|(u, phase)| vertical(self.orbit.radius, age, u, phase))
    }

    /// The share of its orbit spent inside the crowded plane layer (1 for a
    /// star without a vertical orbit).
    pub fn exposure(&self, age: Dy) -> Dy {
        self.vertical_at(age).map(|v| v.exposure).unwrap_or(Dy::ONE)
    }
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
            .checked_div(&st.holding.clone().max(q(1, 1_000_000_000)))
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

/// Sample star `n` of `seed`'s universe, as drawn at the Big Bang.
pub fn star(seed: u64, bang: &BigBang, n: u64) -> Option<Star> {
    // the galactic core: the home galaxy, or another drawn as the home's
    // homologue scaled by its own hole (cores.rs)
    let core = core_of(n);
    let shape = crate::cores::shape(seed, bang, core)?;
    let (scale, mass_ratio) = (shape.scale(), Dy::from_q(&shape.mass_ratio()));
    let nucleus = (n & ((1u64 << CORE_SHIFT) - 1)) >= NUCLEUS_BASE;
    let mut rng = Stream::derive(
        seed,
        &[if nucleus { "nucleus" } else { "star" }.into(), n.into()],
    );
    let mut u = || Dy::unit(rng.next_u64());
    // the population is read from where the star sits: in the nucleus, in
    // the core's reach (a ball of tilted orbits), or in the disc
    let _pop_draw = u();
    // stars placed by magnetic repulsion: each sample star stands for an
    // equal share of its population, and equal repelling moments settle as
    // magnetised drops do (Douady and Couder). Each pivots the golden angle
    // from the last about the core, and its share of its ball's S^360 measure
    // (radius, tilt) pivots by other irrational turns, so neighbours stay
    // apart and every prefix of the population is evenly spread. A disc is a
    // sunflower about the core; a ball's orbit normals repel over the sphere
    // the same way. The random draws these replace are still consumed so the
    // remaining draws are unchanged.
    let local = if nucleus {
        (n & ((1u64 << CORE_SHIFT) - 1)) - NUCLEUS_BASE
    } else {
        n & ((1u64 << CORE_SHIFT) - 1)
    };
    let pivot = |by: &Q| Dy::from_q(&(&Q::int(i64::try_from(local).unwrap_or(0)) * by).frac());
    let _replaced = (u(), u(), u(), u());
    // its reach: the share of the population it stands for, read along the
    // rings' cumulative holding (the nucleus sample within the nucleus)
    let radius = if nucleus {
        reach_at(pivot(&silver_turns()), Some(&nucleus_kpc()))
    } else {
        reach_at(pivot(&silver_turns()), None)
    };
    let pop = if nucleus {
        Pop::Nucleus
    } else if radius.to_q() <= core_kpc() {
        Pop::Bulge
    } else {
        Pop::Disc
    };
    let (cos_tilt, node, turns0) = if pop == Pop::Disc {
        (
            (Dy::ONE, Dy::ZERO),
            (Dy::ONE, Dy::ZERO),
            pivot(&golden_turns()),
        )
    } else {
        (
            tilt_of(pivot(&plastic_turns())),
            node_of(pivot(&golden_turns())),
            pivot(&third_turns()),
        )
    };
    let swing = if pop == Pop::Disc {
        Some((u(), u()))
    } else {
        None
    };
    // the turning law: the deficit inside its circle over its radius, using
    // the home-equivalent mass inside times the hole ratio at the star's own
    // (compressed) radius, so a heavier galaxy turns faster
    let rate = {
        let r_q = down(&radius.max(Dy::sci(1, -6)).to_q(), 60);
        let own_r = down(&(radius * scale).max(Dy::sci(1, -6)).to_q(), 60);
        Dy::from_q(&crate::turning::turning_kpc(
            &(&inside_mass(&r_q) * &mass_ratio.to_q()),
            &own_r,
        ))
    };
    let stands_for = if nucleus {
        Dy::from_q(&nucleus_mass()) / Dy::int(i64::try_from(NUCLEUS_SAMPLE).unwrap_or(1))
    } else {
        Dy::from_q(&stellar_mass()) / Dy::int(i64::try_from(sample(bang)).unwrap_or(1))
    };
    let r = down(&radius.to_q(), if nucleus { 60 } else { GRID_BITS });
    // the birth: the ring's first light, then along the ring's own star
    // formation by the star's draw (chem::Chemistry::birth_along), so a ring
    // makes stars over its history, not all at its first light
    let u_born = u();
    let u_mass = u();
    let u_bin = u();
    let u_comp = u();
    let first = birth_gyr(&r)?;
    let born = crate::chem::Chemistry::cached(bang)
        .map(|c| c.birth_along(&r, &first, u_born))
        .unwrap_or(first);
    let born = down(&born, GRID_BITS);
    let mass = down(&imf(&u_mass.to_q(), &r)?.mid(), GRID_BITS);
    let companion = if u_bin < Dy::ratio(1, 3) {
        down(
            &(&mass * &(&q(1, 10) + &(&q(9, 10) * &u_comp.to_q()))),
            GRID_BITS,
        )
    } else {
        Q::ZERO
    };
    // the homologue's lengths scale by `scale` (compressed, cores.rs), its
    // masses by the hole ratio
    let orbit = Orbit {
        radius: radius * scale,
        rate,
        turns0,
        tilt: cos_tilt,
        node,
        swing,
    };
    Some(Star {
        n,
        r,
        born,
        mass,
        companion,
        orbit,
        pop,
        stands_for: stands_for * mass_ratio,
        core,
        scale,
        galaxy: shape.clone(),
        mass_ratio,
    })
}

/// The repulsion's pivots, each an irrational turn held exactly as a
/// convergent of its defining recurrence: the golden angle (1 / phi^2 of a
/// turn, Fibonacci F39 / F41), the silver (sqrt 2 - 1, Pell), the plastic
/// (1 / rho, Padovan P(n - 1) / P(n)) and sqrt 3 - 1 (convergents of
/// [1; 1, 2, 1, 2, ...]). They are independent over the rationals, so
/// together they never align.
pub(crate) fn golden_turns() -> Q {
    Q::new(63_245_986, 165_580_141).unwrap_or(Q::ZERO)
}
fn silver_turns() -> Q {
    Q::new(470_832, 1_136_689).unwrap_or(Q::ZERO)
}
pub(crate) fn plastic_turns() -> Q {
    Q::new(90_342_335_288_610_825, 119_678_113_856_248_465).unwrap_or(Q::ZERO)
}
fn third_turns() -> Q {
    Q::new(8_155_103_542_731_753, 11_140_078_609_864_049).unwrap_or(Q::ZERO)
}

/// A tilt holding `u` of S^360's measure (concentrated toward 90 degrees,
/// the measure's equator), returned as the half-angle direction (cos, sin),
/// rounded to the grid.
pub(crate) fn tilt_of(u: Dy) -> (Dy, Dy) {
    let theta = measure::angle_holding(measure::UNIVERSE_N, Dy::ONE - u);
    let (s, c) = (Dy::PI * theta).sin_cos();
    let fix = |x: Dy| Dy::from_q(&down(&x.max(Dy::ZERO).to_q(), GRID_BITS));
    (fix(c), fix(s))
}

/// A node at `u` of a turn about the axis, as the half-angle direction
/// rounded to the grid.
pub(crate) fn node_of(u: Dy) -> (Dy, Dy) {
    let (s, c) = (Dy::PI * u).sin_cos();
    let fix = |x: Dy| Dy::from_q(&down(&x.to_q(), GRID_BITS));
    (fix(c), fix(s))
}

/// The initial mass function, inverse-transformed at the uniform `u`:
/// a star's mass (suns) in [0.08, 60]. Its break mb moves with the cloud's
/// warmth, 2 d / (1 + d) of the local surface density d. Provisional
/// catalogue: the bounds and break scale are measured, not derived from
/// the turning law. Computed in dyadics; the caller rounds to the grid.
pub fn imf(u: &Q, r: &Q) -> Option<Encl> {
    // the IMF as a deficit share (no power laws): a cloud's stars stand above
    // the least mass by the share F(m) = (m - lo) / (m - lo + mb), with mb set
    // by the cloud's warmth, so the draw inverts in first degree:
    // m = lo + mb x / (1 - x), x = u F(hi)
    let (lo, hi) = (Dy::ratio(8, 100), Dy::int(60));
    let d = Dy::from_q(&density(r).mid()).max(Dy::ZERO);
    let warmth = Dy::int(2) * d / (Dy::ONE + d);
    let mb = (Dy::ratio(1, 2) * warmth)
        .max(lo * Dy::ratio(101, 100))
        .min(hi * Dy::ratio(99, 100));
    let f_hi = (hi - lo) / (hi - lo + mb);
    let x = Dy::from_q(u) * f_hi;
    let m = lo + mb * x / (Dy::ONE - x).max(Dy::sci(1, -12));
    Some(Encl::exact(m.clamp(lo, hi).to_q()))
}

/// A star's vertical orbit through the disc's plane: its speed through the
/// plane from the age-velocity relation at its ring (20 km/s scale at
/// 10 Gyr, a declared ruler), its amplitude from the plane's restoring pull
/// ([`vertical_frequency`]; provisional until derived from the turning law),
/// its period, and its exposure: the share of its oscillation spent inside
/// the crowded layer ([`layer_kpc`]), in closed form - where comets are
/// shaken loose and supernovae occur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vertical {
    /// Amplitude of the swing through the plane (kpc).
    pub amp_kpc: Dy,
    /// Speed through the plane (km/s).
    pub vz_kms: Dy,
    /// Period of the oscillation (Myr).
    pub period_myr: Dy,
    /// Share of the oscillation spent inside the crowded layer, in [0, 1].
    pub exposure: Dy,
    /// Position in the oscillation (turns).
    pub phase: Dy,
}

/// The crowding of the ring `r` (kpc) sits in, as a dyadic: the ring's
/// holding per kpc over the reference ring's ([`density`]), 1 there.
fn crowding(r_kpc: Dy) -> Dy {
    Dy::from_q(&density(&r_kpc.to_q()).mid()).max(Dy::ZERO)
}

/// The crowded layer's half-thickness (kpc): a declared ruler (0.3). The
/// structure read from the founding gives reaches and spans along the
/// plane, not the plane's height, so no derived scale replaces it yet.
pub fn layer_kpc() -> Dy {
    Dy::ratio(3, 10)
}

/// The plane's oscillation frequency at `r` (rad/Gyr): the pull at the
/// reference ring (75.2 rad/Gyr, a declared ruler) times the crowding share
/// there, 2 d / (1 + d) with d the ring's crowding ([`density`]), in place
/// of sqrt(4 pi G rho).
pub fn vertical_frequency(r_kpc: Dy) -> Dy {
    let density = crowding(r_kpc);
    Dy::ratio(752, 10) * Dy::int(2) * density / (Dy::ONE + density)
}

/// A star's vertical orbit at `r` after `age` Gyr: `u` draws its speed
/// (exponential), `phase` is its position in the oscillation (turns).
pub fn vertical(r_kpc: Dy, age: Dy, u: Dy, phase: Dy) -> Vertical {
    let density = crowding(r_kpc);
    // the spread grows as shares of age (against 10 Gyr) and of the plane's
    // density (no roots); the draw is exponential, first degree (no Rayleigh
    // root)
    let t = age.max(Dy::ZERO) / Dy::int(10);
    let sigma = Dy::int(20)
        * (Dy::int(2) * t / (Dy::ONE + t))
        * (Dy::int(2) * density / (Dy::ONE + density));
    let vz = sigma * -(Dy::ONE - u).max(Dy::sci(1, -12)).log();
    let nu = vertical_frequency(r_kpc);
    let amp = vz / (nu * Dy::ratio(9778, 10_000)).max(Dy::sci(1, -9));
    // exposure in closed form: the share of the oscillation spent inside the
    // crowded layer, linear in angle
    let h = layer_kpc();
    let exposure = if amp <= h {
        Dy::ONE
    } else {
        // the angle whose tangent is h / A, over the quarter turn (avoids
        // 1 - x^2 under a root)
        (h / amp).atan() * Dy::int(2) / Dy::PI
    };
    Vertical {
        amp_kpc: amp,
        vz_kms: vz,
        period_myr: Dy::int(2) * Dy::PI / nu.max(Dy::sci(1, -9)) * Dy::int(1000),
        exposure,
        phase,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stars_circle_the_core_and_the_core_holds_its_share() {
        let bang = crate::genesis::bang();
        let stars: Vec<Star> = (0..1000).filter_map(|n| star(7, bang, n)).collect();
        // a star stands only where its ring's gas lights, and is born along
        // the ring's own star formation: at or after its first light, before
        // the horizon
        assert!(!stars.is_empty() && stars.len() <= 1000);
        let horizon = crate::life::horizon_gyr().to_q();
        for s in &stars {
            let first = birth_gyr(&s.r).expect("a standing star's ring lights");
            assert!(
                s.born >= down(&first, GRID_BITS),
                "born after its first light"
            );
            assert!(s.born <= horizon, "born before the horizon");
        }
        // the core's reach is one galaxy unit, and a star within it is of the
        // core's ball; the sample follows the rings' holding, so the core holds
        // the first rings' share of it
        let core = stars.iter().filter(|s| s.in_core()).count();
        let bulge = stars.iter().filter(|s| s.pop == Pop::Bulge).count();
        assert_eq!(core, bulge, "core {core} bulge {bulge}");
        // every star within the outermost ring's edge (the rim spread by the
        // founding's final share)
        let edge = structure()
            .rings
            .last()
            .map(|r| r.1.clone())
            .unwrap_or(rim_kpc());
        assert!(stars.iter().all(|s| s.orbit.radius.to_q() <= edge));
        // a star moves: its distance from its own core is its orbit's, its place changes
        let r = crate::chain::radius();
        let s = &stars[0];
        let (p0, p1) = (s.place(Dy::int(5), &r), s.place(Dy::sci(51, -1), &r));
        let from_core = crate::minds::from_centre_kpc(s, Dy::int(5));
        assert!(
            (from_core - s.orbit.radius).abs() < Dy::ratio(1, 2),
            "{} vs {}",
            from_core,
            s.orbit.radius
        );
        assert!(p0 != p1, "it moves");
        // a disc star stays near the plane; a bulge star's orbit is tilted
        let height = |s: &Star, t: Dy| {
            let (reach, dir, climb) = s.bearing(t);
            Dy::from_q(
                &(&(&dir[2] * &reach).abs() + &climb.abs())
                    .checked_div(&crate::chain::kpc())
                    .unwrap(),
            )
        };
        let disc = stars.iter().find(|s| s.pop == Pop::Disc).unwrap();
        assert!(height(disc, Dy::int(8)) < Dy::int(2));
        // the core's ball: where the nucleus's gas lights its sample's orbits
        // are tilted, so some stand off the plane by a share of their reach,
        // all within the nucleus; where it never lights no nucleus star stands
        let nuc: Vec<Star> = (0..64)
            .filter_map(|k| star(7, bang, NUCLEUS_BASE + k))
            .collect();
        if nuc.is_empty() {
            assert!(birth_gyr(&Q::ZERO).is_none(), "the nucleus never lights");
        } else {
            assert!(nuc
                .iter()
                .any(|s| height(s, Dy::int(3)) > s.orbit.radius / Dy::int(10)));
            assert!(
                nuc.iter()
                    .all(|s| s.pop == Pop::Nucleus && s.orbit.radius.to_q() <= nucleus_kpc()),
                "within the nucleus"
            );
        }
        // the turning law: inner stars turn by the curvature inside them, and
        // the mass inside grows outward to the galaxy's whole
        assert!(inside_mass(&q(1, 1000)) >= hole_mass());
        assert!(inside_mass(&rim_kpc()) >= inside_mass(&r0()));
        assert!(inside_mass(&r0()) > inside_mass(&q(1, 1000)));
        // the arc: zero to itself, symmetric, and never shorter than the
        // difference between the two reaches
        let (a, b) = (
            stars[1].place(Dy::int(4), &r),
            stars[2].place(Dy::int(4), &r),
        );
        assert!(a.arc(&a, &r).mid().abs() < q(1, 1_000_000));
        let (ab, ba) = (a.arc(&b, &r).mid(), b.arc(&a, &r).mid());
        assert_eq!(ab, ba);
        assert!(ab >= (&a.reach - &b.reach).abs(), "arc {ab}");
    }

    #[test]
    fn the_core_makes_heavier_stars_and_every_star_is_born_after_the_first() {
        let bang = crate::genesis::bang();
        let mean = |r: Q| {
            let n = 60;
            let s = (0..n).fold(Q::ZERO, |a, k| {
                &a + &imf(&q(k * 2 + 1, n * 2), &r).unwrap().mid()
            });
            s.checked_div(&Q::int(n)).unwrap()
        };
        // the IMF's break moves heavier where the rings crowd: at the ring of
        // greatest holding per span against the reference ring
        let densest = structure()
            .rings
            .iter()
            .max_by(|a, b| a.3.cmp(&b.3))
            .map(|r| r.0.clone())
            .unwrap_or(r0());
        assert!(mean(densest) >= mean(r0()));
        for s in (0..300).filter_map(|n| star(7, bang, n)) {
            assert!(s.mass.signum() > 0, "{}", s.mass);
            assert!(s.born >= bang.first_stars_gyr().lo);
        }
    }
}
