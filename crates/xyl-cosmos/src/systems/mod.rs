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
//! - Star (`Spec`, `luminosity`): `L = M delta`, `delta = 1 / (r + 1)` the
//!   vacuum's share at its reach about the universal core, in suns.
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
//!   gathers moons on its own manifold, `2 pi a m / (M + m)`, by the same
//!   region and settling laws one tier down.
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
    // below the resolution: e^x under 2^-bits is enclosed by [0, 2^-bits]
    // (x <= -(bits + 1), since e^-(bits + 1) < 2^-bits), never computed in
    // ever-larger rationals (a far world's growth time made one such
    // argument run for minutes)
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

/// A star's mean luminosity in suns: its light is its burn (stars.rs), the
/// pulse it absorbs at its reach `r` about the universal core in proportion
/// to its mass, `L = M / (r + 1)`: a one-sun star at the core's reach shines
/// one sun, the unit; one in a galaxy further out, dimmer by the vacuum's
/// share there. Every main sequence at a reach lasts the same span and every
/// disc has the same ratio of rock line to outer reach.
pub fn luminosity(mass: &Q, reach: Dy) -> Q {
    mass * &crate::circuit::vacuum_share(reach, Dy::ONE).to_q()
}

/// A star's spec: its mass and its light (suns).
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
    /// Its orbit about its star, its reach a (AU; a moon shares its giant's).
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
    /// Layer shares of its mass: core, mantle, ices, envelope.
    pub layers: [Q; 4],
    /// Each element's mass in it (world masses), indexed as the element table.
    pub elements: Vec<Q>,
    /// Each element's mass in its silicate mantle (world masses); empty where
    /// not apportioned.
    pub mantle: Vec<Q>,
    /// The volatiles it gathered, each per unit of its rock (its ices:
    /// what condensed where it formed, scaled by the gas's
    /// metals, or what comets brought inside the snow line). A moon has its
    /// giant's.
    pub ices: Vec<(&'static str, Q)>,
    /// A moon's eccentricity, forced for ever by its neighbours in the chain.
    pub e_forced: Option<Q>,
    /// The greatest swing its neighbours gave it at the settled age, as a
    /// share of its reach: its starting eccentricity, and its tilt as the
    /// swing's deficit angle (bodies::orbits).
    pub swing: Q,
    /// Its settled phase about its anchor (turns): its node and periapsis.
    pub phase: Q,
    /// The age (Gyr) at which it merges into its heavier neighbour and ends;
    /// None for a world that stands to the horizon.
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
    /// The comet cloud its giants flung out: (mass, elements).
    pub comets: Option<(Q, Vec<Q>)>,
}

/// Whether a body is round: the turn its own curvature gives at its surface
/// meets its material's rigidity angle (turning.rs).
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
/// star circling at `r` kpc. Provisional: stellar encounters are not yet
/// modelled (a column-density reading stripped every world), so every world
/// keeps its star.
pub fn survival(_a: &Q, _r: &Q, _gyr: &Q) -> Q {
    Q::ONE
}

/// The settled age at which systems are read (Gyr).
pub fn settled_gyr() -> Q {
    Q::int(3)
}

/// The system of `star`, at galactocentric radius `r`, in the universe whose
/// chemistry is `chem`: `path` keys its draws (seed, cell, slot).
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
    // the birth gas: the ring chemistry at the star's place and birth,
    // weighted by its galaxy's composition at that time (founding recipe plus
    // what it took from the core); the home galaxy and the others alike
    let weights = crate::cores::weights_at(seed, bang, star.core, &star.born, elements)?;
    let (gas, z) = (
        Gas::at(chem, elements, &star.r, &star.born, &lump, &weights),
        chem.metals(&star.r, &star.born)?,
    );
    if z.signum() <= 0 {
        return Some(empty(spec));
    }
    let env = density(&star.r).mid();
    // the star's manifold about the core, gathering its inner share; each
    // world's region is cut against the core's tide at the star
    // (disc::manifold_worlds), so the core-star-planet chain is one law
    let Some(frame) = crate::galaxy::disc_frame(&star.r, &spec.mass) else {
        return Some(empty(spec));
    };
    let (regions, belt_share) =
        crate::disc::manifold_worlds(&frame.star_share, &frame.a_disc, &frame.tide);
    // the worlds' outer edge is the star's manifold reach (AU); a world's
    // reach is its share of it
    let a_out = held(
        &(&frame.reach_kpc * &crate::package::Si::kpc())
            .checked_div(&crate::package::Si::au())
            .unwrap_or(Q::ZERO),
    );
    // the infall's solids (world masses): the star's mass over its share of
    // the infall, times the gas's metal mass fraction (its metallicity
    // against the Sun's birth gas times the Sun's,
    // Rulers::reference_metallicity, a ruler)
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
    // the disc's ungathered solids: the belts', less what worlds take as moons
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

/// A world's moon budget and the star's tide at it. Each world anchors its
/// own manifold, reach 2 pi a m / (M + m); its moons gather there from the
/// disc's unheld matter (`moon_pool` at most), against the star's tide at
/// the world, s / (x + s) in the star's manifold units.
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

/// The disc's solid surface density at its mean reach (g/cm^2), which sets
/// a core's gas accretion time: `infall_solids` (world masses) over
/// pi `a_out`^2 (AU).
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

/// The disc's embryos: each region of the manifold inside the rock line
/// becomes a body at its share of `a_out`, with its holding of the infall's
/// solids enriched past each ice line, at phase k times the golden turn.
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
/// The core is the embryo's matter. A core past ten world masses gathers an
/// envelope in the share its growth beats the disc's dispersal; the growth
/// time is its turning period over the period at 1 AU,
/// a (a + M) / (1 + M) with M the star as a length in AU (G M / c^2 per
/// sun), over the surface density `sigma1`. Inside the snow line its ices
/// arrive late, by comet.
fn planet(
    e: &Embryo,
    spec: &Spec,
    gas: &Gas,
    z: &Q,
    env: &Q,
    sigma1: &Q,
    r_kpc: &Q,
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
    // a world is lost when its survival share is below one half
    let lost = survival(&oq, r_kpc, &settled_gyr()) < Q::ratio(1, 2);
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
