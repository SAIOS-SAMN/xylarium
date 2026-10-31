//! A star's system: its worlds, moons, belt and comet cloud, with every
//! body's matter apportioned among the elements by mass.
//!
//! Inputs: a `galaxy::Star` and its manifold (`galaxy::disc_frame`), the
//! regions `disc::manifold_worlds` cuts on it, the birth gas from
//! `chem::Chemistry` weighted by its galactic core's recipe
//! (`cores::weights_at`), the element table (`elements::Element`), and SI
//! constants from `package::Si`. Reaches are in AU, masses in world masses,
//! temperatures in K; the settling runs in core units (lengths in `r_g`).
//!
//! - Star (`Spec`, `luminosity`): `L = M delta` in solar luminosities, where
//!   `delta = 1 / (r + 1)` is the vacuum share at the star's reach r about
//!   the universal core.
//! - Orbits: each region at its share of the manifold's reach becomes an
//!   embryo whose solids are the star's mass over its gathering share times
//!   the gas's metal mass fraction, enriched past each ice line. Embryos
//!   form only outside the rock line (Fe, Si, Mg condensation).
//! - Settling (`settle_about`, closed form in ages): neighbours turning at
//!   `omega = [M / (A + M)] / A` kick each other once per synodic period; a
//!   pair merges at the first age its kick spans its gap (bisection to
//!   life's horizon). Merges before `settled_gyr` (3 Gyr) are applied; later
//!   ones are recorded as `World::merged_at` on the lighter body.
//! - Worlds (Kokubo & Ida; Mizuno): disc temperature `t_form = 278 L / a`;
//!   a core past ten world masses gathers an envelope in the share its
//!   growth beats the disc's dispersal; inside the snow line ices arrive by
//!   comet. Bodies that are not round (`is_world`) are dropped. Each world
//!   gathers moons on a manifold of reach `2 pi a m / (M + m)` about the
//!   world, by the same region and settling laws one tier down.
//! - Matter (`Gas::apportion_layers`): an element is in the solids only
//!   where the disc was cooler than its condensation temperature;
//!   siderophile and chalcophile elements go to the core less the share the
//!   mantle keeps (`mantle_kept`, a declared ruler); lithophiles and oxygen
//!   (three per silicon, one per kept iron) to the mantle; C, N and the
//!   remaining O as ices; an envelope is primordial H and He.
//!
//! Outputs: `system` returns a `System` (`Spec`, `World`s, `Belt`s, comets),
//! or `None` where the birth gas cannot be read. Invariants: a body's
//! element masses are exact `Q` and sum to its mass exactly; reaches and
//! masses are held on a dyadic grid; draws come from the stream keyed by
//! the seed and `["system", path]`, so a system is deterministic. Stellar
//! encounters (`survival`, always 1) and the vertical exposure that thins
//! comet clouds are not modelled; orbital elements and spins are in
//! `bodies`.
//!
//! Terms: belt, merge age, settling, swing (docs/glossary.md).

use crate::chem::Chemistry;
use crate::elements::extent;
use crate::elements::{decimal, Class, Element};
use crate::galaxy::{density, Star};
use xyl_core::certified::{down, pi, Encl};
use xyl_core::dyadic::Dy;
use xyl_core::{Stream, Q};

mod gas;
mod moons;
mod settle;

pub use gas::*;
use moons::*;
use settle::*;

const BITS: u32 = 64;
const GRID_BITS: u32 = 40;

fn fix(e: &Encl) -> Q {
    down(&e.mid(), GRID_BITS)
}

/// Rounds a base to dyadic precision before a certified power or exponential is
/// taken of it. Abundances reach 1e-67 per hydrogen, so exact denominators run
/// to hundreds of bits; the result is fixed on the grid anyway.
fn held(x: &Q) -> Q {
    Dy::from_q(x).to_q()
}

/// The certified exponential of a held argument.
fn exp(x: &Q, bits: u32) -> Encl {
    // For x <= -(bits + 1), e^x < 2^-bits, so the result is enclosed by
    // [0, 2^-bits] without evaluating the series; exact evaluation of such
    // arguments (the growth time of a distant world) took minutes.
    if x <= &-Q::int(i64::from(bits) + 1) {
        let floor = (0..bits).fold(Q::ONE, |a, _| a.checked_div(&Q::int(2)).unwrap_or(Q::ZERO));
        return Encl {
            lo: Q::ZERO,
            hi: floor,
        };
    }
    xyl_core::certified::exp(&held(x), bits)
}

/// A uniform rational in [0, 1).
fn unit(rng: &mut Stream) -> Q {
    let x = i64::try_from(rng.next_u64() >> 11).unwrap_or(0);
    Q::ratio(x, 1 << 53)
}

/// A sun's gravitational length, G M_sun / c^2 (m).
pub(crate) fn star_length_m() -> Q {
    use crate::package::Si;
    (&Si::g() * &Si::star_mass_kg())
        .checked_div(&(&Si::c() * &Si::c()))
        .unwrap_or(Q::ZERO)
}

/// A star's mean luminosity in solar luminosities, `L = M / (r + 1)`: the
/// mass M times the vacuum share at the star's reach r about the universal
/// core (`circuit::vacuum_share`; the burn itself is modelled in stars.rs). A
/// one-solar-mass star at the core's reach has L = 1; a star in a galaxy
/// further out is dimmer by the vacuum share there. Consequently every main
/// sequence at a given reach lasts the same span and every disc has the same
/// ratio of rock line to outer reach.
pub fn luminosity(mass: &Q, reach: Dy) -> Q {
    mass * &crate::circuit::vacuum_share(reach, Dy::ONE).to_q()
}

/// A star's specification: its mass and luminosity, in solar units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    /// Stellar mass, in solar masses (floored at 1/10000).
    pub mass: Q,
    /// Mean luminosity, in solar luminosities (see [`luminosity`]).
    pub lum: Q,
}

impl Spec {
    /// The spec of the galaxy star `star`: its mass, floored at 1/10000 solar
    /// masses, and the luminosity that mass gives at its reach.
    pub fn of(star: &Star) -> Spec {
        let mass = star.mass.clone().max(Q::ratio(1, 10_000));
        let lum = luminosity(&mass, star.reach_rings());
        Spec { mass, lum }
    }

    /// The disc's temperature (K) at reach a (AU): `278 L / a`.
    pub fn t_form(&self, a: &Q) -> Q {
        (&Q::int(278) * &self.lum).checked_div(a).unwrap_or(Q::ZERO)
    }
}

/// A world (a planet, or a giant's moon).
#[derive(Clone, Debug)]
pub struct World {
    /// Semi-major axis about the star, in AU; a moon carries its giant's.
    pub a: Q,
    /// Mass, in world masses.
    pub mass: Q,
    /// Radius, in world radii.
    pub radius: Q,
    /// Whether the world is a giant (holds a gas envelope).
    pub giant: bool,
    /// For a moon, the index of the giant it circles in the system's worlds;
    /// `None` for a planet.
    pub host: Option<usize>,
    /// For a moon, its orbital distance in the giant's radii; `None` for a
    /// planet.
    pub a_host_radii: Option<Q>,
    /// Mass shares of the layers: core, mantle, ices, envelope.
    pub layers: [Q; 4],
    /// Each element's mass in the world (world masses), indexed as the
    /// element table.
    pub elements: Vec<Q>,
    /// Each element's mass in the silicate mantle (world masses); empty where
    /// not apportioned.
    pub mantle: Vec<Q>,
    /// The gathered volatiles, each as mass per unit of rock: the ices that
    /// condensed at the formation reach, scaled by the gas's metallicity, or,
    /// inside the snow line, the ices delivered by comets. A moon carries its
    /// giant's.
    pub ices: Vec<(&'static str, Q)>,
    /// A moon's forced eccentricity from its neighbours in the resonance
    /// chain; `None` for a planet.
    pub e_forced: Option<Q>,
    /// The largest swing the neighbours imparted by the settled age, as a
    /// share of the reach. `bodies::orbits` reads it as the starting
    /// eccentricity and reads the tilt as the swing's deficit angle.
    pub swing: Q,
    /// The settled phase about the anchor (turns), which sets the node and
    /// periapsis.
    pub phase: Q,
    /// The age (Gyr) at which the world merges into its heavier neighbour;
    /// `None` for a world that persists to the horizon.
    pub merged_at: Option<Q>,
}

/// A belt: the disc's solids no world gathered.
#[derive(Clone, Debug)]
pub struct Belt {
    /// Inner edge, in AU.
    pub inner_a: Q,
    /// Outer edge, in AU.
    pub outer_a: Q,
    /// Total mass, in world masses.
    pub mass: Q,
    /// Each element's mass in the belt, in world masses, indexed as the
    /// element table.
    pub elements: Vec<Q>,
}

/// A star's system.
#[derive(Clone, Debug)]
pub struct System {
    /// The host star's mass and luminosity.
    pub spec: Spec,
    /// The system's worlds: planets and giants' moons.
    pub worlds: Vec<World>,
    /// The disc's ungathered solids: at most an inner belt in the innermost
    /// giant's resonances and an outer belt beyond the outermost world.
    pub belts: Vec<Belt>,
    /// The comet cloud scattered by the giants: (mass, elements).
    pub comets: Option<(Q, Vec<Q>)>,
}

/// Whether a body is round: the turn that the body's gravity gives at its
/// surface reaches the material's rigidity angle (`turning::roundness` at
/// least 1). Returns false for a non-positive mass or radius.
pub fn is_world(mass: &Q, radius: &Q, icy: bool) -> bool {
    if mass.signum() <= 0 || radius.signum() <= 0 {
        return false;
    }
    crate::turning::roundness(
        &(mass * &crate::package::Si::world_mass_kg()),
        &(radius * &crate::package::Si::world_radius_m()),
        icy,
    ) >= Q::ONE
}

/// The share of worlds at reach `a` (AU) that keep their star for `gyr`, the
/// star circling at `r` galaxy rings. Provisional: stellar encounters are not yet
/// modelled (a column-density reading stripped every world), so every world
/// keeps its star.
pub fn survival(_a: &Q, _r: &Q, _gyr: &Q) -> Q {
    Q::ONE
}

/// The settled age at which systems are read (Gyr).
pub fn settled_gyr() -> Q {
    Q::int(3)
}

/// The system of `star` in the universe whose chemistry is `chem`; `path`
/// keys the draws (seed, cell, slot). Returns `None` where the birth gas or
/// its metallicity cannot be read or a division fails.
pub fn system(
    seed: u64,
    bang: &crate::bang::BigBang,
    path: &[u64],
    star: &Star,
    chem: &Chemistry,
    elements: &[Element],
) -> Option<System> {
    let spec = Spec::of(star);
    let mut parts: Vec<xyl_core::rng::Part> = vec!["system".into()];
    parts.extend(path.iter().map(|&p| p.into()));
    let mut rng = Stream::derive(seed, &parts);
    let lump = chem
        .lump(&star.r, &star.born, &unit(&mut rng))
        .unwrap_or(Q::ONE);
    // The birth gas is the ring chemistry at the star's radius and birth
    // time, weighted by the galaxy's composition at that time (the founding
    // recipe plus the matter drawn from the core). The home galaxy and the
    // other galaxies use the same law.
    let weights = crate::cores::weights_at(seed, bang, star.core, &star.born, elements)?;
    let (gas, z) = (
        Gas::at(chem, elements, &star.r, &star.born, &lump, &weights),
        chem.metals(&star.r, &star.born)?,
    );
    if z.signum() <= 0 {
        return Some(empty(spec));
    }
    let env = density(&star.r).mid();
    // The star's manifold about the core; each world's region is cut against
    // the core's tide at the star (disc::manifold_worlds), so the
    // core-star-planet chain follows one law.
    let Some(frame) = crate::galaxy::disc_frame(&star.r, &spec.mass) else {
        return Some(empty(spec));
    };
    let (regions, belt_share) =
        crate::disc::manifold_worlds(&frame.star_share, &frame.a_disc, &frame.tide);
    // The worlds' outer edge is the star's manifold reach (AU, from galaxy
    // rings through the boundary `galaxy::kpc_of_rings`); a world's reach is
    // its region's share of it.
    let a_out = held(
        &(&crate::galaxy::kpc_of_rings(&frame.reach) * &crate::package::Si::kpc())
            .checked_div(&crate::package::Si::au())
            .unwrap_or(Q::ZERO),
    );
    // The infall's solids (world masses): the star's mass over the star's
    // share of the infall, times the gas's metal mass fraction (the
    // metallicity relative to the Sun's birth gas times the Sun's metal
    // fraction, Rulers::reference_metallicity, a declared ruler).
    let star_earths = &spec.mass
        * &crate::package::Si::star_mass_kg()
            .checked_div(&crate::package::Si::world_mass_kg())
            .unwrap_or(Q::ZERO);
    let infall_solids = held(
        &(&star_earths
            .checked_div(&frame.star_share.clone().max(Q::ratio(1, 1_000_000_000)))
            .unwrap_or(Q::ZERO)
            * &(&z * &crate::chem::Rulers::reference_metallicity())),
    );
    let t_rock = rock_line_k(elements);
    let sigma1 = surface_density(&infall_solids, &a_out);
    let mut worlds: Vec<World> = Vec::new();
    // The disc's ungathered solids (the belts' share), less the mass that
    // worlds take as moons.
    let mut moon_pool = held(&(&belt_share * &infall_solids));
    let mut giants_mass = Q::ZERO;
    let mut orbits: Vec<(Q, bool)> = Vec::new();
    if a_out.signum() > 0 && !regions.is_empty() {
        let embryos = embryos(&regions, &a_out, &spec, &gas, &t_rock, &infall_solids, &z);
        for e in settle(embryos, &spec.mass, &settled_gyr()) {
            let Some((world, t_form, ice_share)) =
                planet(&e, &spec, &gas, &z, &env, &sigma1, &star.r)?
            else {
                continue;
            };
            let (oq, total) = (world.a.clone(), world.mass.clone());
            if world.giant {
                giants_mass = &giants_mass + &total;
            }
            orbits.push((oq.clone(), world.giant));
            worlds.push(world);
            let (budget, tide) = moon_budget(
                &oq,
                &total,
                &star_earths,
                &a_out,
                &frame,
                &infall_solids,
                &moon_pool,
            );
            let placed = moons(
                &mut worlds,
                &gas,
                &t_form,
                &ice_share,
                &spec.mass,
                &budget,
                &tide,
            );
            moon_pool = (&moon_pool - &placed).max(Q::ZERO);
        }
    }
    let (belts, comets) = belts(&spec, &gas, &moon_pool, &a_out, &orbits, &giants_mass, &env);
    Some(System {
        spec,
        worlds,
        belts,
        comets,
    })
}

/// A world's moon budget and the star's tide at the world. Each world
/// anchors a manifold of reach `2 pi a m / (M + m)`; the budget is the disc's
/// solids over that width (at most `moon_pool`), and the tide is `s / (x + s)`
/// with s the star's share and x the world's reach in the star's manifold
/// units.
fn moon_budget(
    a: &Q,
    mass: &Q,
    star_earths: &Q,
    a_out: &Q,
    frame: &crate::galaxy::DiscFrame,
    infall_solids: &Q,
    moon_pool: &Q,
) -> (Q, Q) {
    let world_reach_au = &(&(&Q::int(2) * &pi(BITS).mid()) * a)
        * &mass.checked_div(&(star_earths + mass)).unwrap_or(Q::ZERO);
    let width = world_reach_au.checked_div(a_out).unwrap_or(Q::ZERO);
    let budget = held(&(&(&frame.a_disc * &width) * infall_solids).min(moon_pool.clone()));
    let x = a.checked_div(a_out).unwrap_or(Q::ONE);
    let tide = frame
        .star_share
        .checked_div(&(&x + &frame.star_share))
        .unwrap_or(Q::ZERO);
    (budget, tide)
}

/// The rock line's temperature (K): the lowest condensation temperature of
/// Fe, Si and Mg in the element table (1300 K where none is listed).
fn rock_line_k(elements: &[Element]) -> Q {
    ["Fe", "Si", "Mg"]
        .iter()
        .filter_map(|sym| {
            elements
                .iter()
                .find(|e| e.sym == *sym)
                .and_then(|e| e.tc.clone())
        })
        .min()
        .unwrap_or(Q::int(1300))
}

/// The disc's mean solid surface density (g/cm^2), which sets a core's gas
/// accretion time: `infall_solids` over `pi a_out^2`, converted from world
/// masses and AU to grams and centimetres.
fn surface_density(infall_solids: &Q, a_out: &Q) -> Q {
    let au_cm = &crate::package::Si::au() * &Q::int(100);
    (infall_solids * &(&crate::package::Si::world_mass_kg() * &Q::int(1000)))
        .checked_div(
            &(&(&xyl_core::certified::pi(64).mid() * a_out) * &(a_out * &au_cm.clone() * &au_cm)),
        )
        .unwrap_or(Q::ZERO)
}

/// A system with no worlds, belts or comets.
fn empty(spec: Spec) -> System {
    System {
        spec,
        worlds: Vec::new(),
        belts: Vec::new(),
        comets: None,
    }
}

/// The disc's embryos: each region of the manifold outside the rock line
/// (where the disc is cooler than `t_rock`) becomes a body at the region's
/// share of `a_out`, with the region's holding of the infall's solids
/// enriched past each ice line, at phase k times the golden turn.
fn embryos(
    regions: &[(Q, Q)],
    a_out: &Q,
    spec: &Spec,
    gas: &Gas,
    t_rock: &Q,
    infall_solids: &Q,
    z: &Q,
) -> Vec<Embryo> {
    let mut embryos: Vec<Embryo> = Vec::new();
    for (k, (reach, holding)) in regions.iter().enumerate() {
        let a = held(&(a_out * reach));
        let t_form = spec.t_form(&a);
        if &t_form > t_rock {
            continue;
        }
        let (_, ices) = gas.solids(&t_form);
        let has_water = ices.iter().any(|(sp, _)| *sp == "H2O");
        let enrich = if has_water {
            ices.iter().fold(Q::ONE, |a, (_, m)| &a + &(m * z))
        } else {
            Q::ONE
        };
        let m = held(&(&(holding * infall_solids) * &enrich));
        if m.signum() <= 0 {
            continue;
        }
        embryos.push(Embryo {
            a,
            m,
            phase: (&Q::int(i64::try_from(k).unwrap_or(0)) * &crate::galaxy::golden_turns()).frac(),
            swing: Q::ZERO,
            merged_at: None,
        });
    }
    embryos
}

/// The planet a settled embryo becomes, with its disc temperature (K) and
/// ice share: `Some(None)` where it is not round or not kept, `None` where a
/// division fails (the system is then refused).
///
/// The core is the embryo's matter. A core of at least ten world masses
/// gathers an envelope in the share `exp(-grow / 3)` by which its growth
/// outpaces the disc's dispersal. The growth time is
/// `grow = [a (a + M) / (1 + M)] / 25 / (sigma1 / 10)`, where
/// `a (a + M) / (1 + M)` is the turning period over the period at 1 AU and M
/// is the star as a length in AU (G M / c^2 per solar mass). Inside the snow
/// line the ices arrive late, by comet.
fn planet(
    e: &Embryo,
    spec: &Spec,
    gas: &Gas,
    z: &Q,
    env: &Q,
    sigma1: &Q,
    reach: &Q,
) -> Option<Option<(World, Q, Q)>> {
    let oq = e.a.clone();
    let t_form = spec.t_form(&oq);
    let (core_share, mut ices) = gas.solids(&t_form);
    let has_water = ices.iter().any(|(s, _)| *s == "H2O");
    if !has_water {
        let late = &(&Q::ratio(1, 10_000) * env) * z;
        ices = vec![
            ("H2O", late.clone()),
            ("CO2", &late * &Q::ratio(1, 10)),
            ("CO", &late * &Q::ratio(1, 10)),
            ("CH4", &late * &Q::ratio(1, 100)),
            ("NH3", &late * &Q::ratio(1, 100)),
            ("N2", &late * &Q::ratio(5, 1000)),
        ];
    }
    let ices: Vec<(&str, Q)> = if has_water {
        ices.into_iter().map(|(s, m)| (s, &m * z)).collect()
    } else {
        ices
    };
    let core = e.m.clone();
    let mut envelope = Q::ZERO;
    let m_au = (&spec.mass * &star_length_m())
        .checked_div(&crate::package::Si::au())
        .unwrap_or(Q::ZERO);
    let period = (&oq * &(&oq + &m_au)).checked_div(&(&Q::ONE + &m_au))?;
    let grow = period
        .checked_div(&Q::int(25))?
        .checked_div(&sigma1.checked_div(&Q::int(10))?.max(Q::ratio(1, 1_000_000)))?;
    let keep = fix(&exp(&-(grow.checked_div(&Q::int(3))?), BITS));
    if core >= Q::int(10) {
        let gathering = core.checked_div(&(&core + &Q::int(10)))?;
        envelope = held(
            &(&(&(&(&core * &keep) * &gathering) * &Q::ratio(56, 10))
                * &spec.mass.clone().min(Q::ONE)),
        );
    }
    let total = &core + &envelope;
    let ice_sum = ices.iter().fold(Q::ZERO, |a, (_, m)| &a + m);
    let ice_share = ice_sum.checked_div(&(&Q::ONE + &ice_sum))?;
    let radius = if envelope.signum() > 0 {
        &(&Q::int(11) * &(&Q::ONE - &fix(&exp(&-(total.checked_div(&Q::int(30))?), BITS))))
            + &extent(&core)
    } else {
        &extent(&core) * &(&Q::ONE + &(&Q::ratio(4, 10) * &ice_share))
    };
    let rock = &core * &(&Q::ONE - &ice_share);
    let (c_mass, m_mass) = (&rock * &core_share, &rock * &(&Q::ONE - &core_share));
    let ice_mass: Vec<(&str, Q)> = ices
        .iter()
        .map(|(s, m)| {
            (
                *s,
                (&(&core * &ice_share) * m)
                    .checked_div(&ice_sum)
                    .unwrap_or(Q::ZERO),
            )
        })
        .collect();
    let icy = ice_share > Q::ratio(1, 5);
    // A world is lost when its survival share is below one half.
    let lost = survival(&oq, reach, &settled_gyr()) < Q::ratio(1, 2);
    if !is_world(&total, &radius, icy) || lost {
        return Some(None);
    }
    let layers = [
        c_mass.checked_div(&total)?,
        m_mass.checked_div(&total)?,
        (&core * &ice_share).checked_div(&total)?,
        envelope.checked_div(&total)?,
    ];
    let (elems, mantle) = gas.apportion_layers(&t_form, &c_mass, &m_mass, &ice_mass, &envelope);
    let world = World {
        a: oq,
        mass: total,
        radius,
        giant: envelope.signum() > 0,
        host: None,
        a_host_radii: None,
        layers,
        elements: elems,
        mantle,
        ices,
        e_forced: None,
        swing: e.swing.clone(),
        phase: e.phase.clone(),
        merged_at: e.merged_at.clone(),
    };
    Some(Some((world, t_form, ice_share)))
}

#[cfg(test)]
mod tests;
