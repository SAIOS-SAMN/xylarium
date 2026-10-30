//! A star's system, with every world's matter traced to the elements by mass.
//!
//! The star: its mass is drawn in the galaxy (galaxy.rs); its luminosity is the
//! pulse it absorbs, L = M in suns.
//!
//! The orbits: reaches a are held in AU. A star's worlds stand on its
//! manifold (galaxy::disc_frame): the arc of its circle about the core its
//! own pull holds, its outer edge; the star holds its gathering, the inner
//! share, and its solids are the star's mass over that share times the gas's
//! metal mass fraction. Worlds are read degree by degree across the manifold
//! (disc::manifold_worlds): a world forms where the matter gathered
//! since the last one holds its own region against the core's twist at that
//! degree, so the core-star-planet chain is one law, with no seed and no
//! partition. Worlds whose kicks over the age still swing them across their
//! gap merge until every neighbour stands (`settle`).
//!
//! The worlds (Kokubo & Ida; Mizuno): each orbit forms at disc temperature
//! `t_form = 278 L / a`. Every ice whose line lies inside the orbit adds to the
//! solids; inside the snow line, water arrives late by comet. A core past ten
//! world masses takes gas in the share its growth beat the disc's dispersal. The
//! radius follows from mass and layers. A body that is not round is rubble; a
//! giant gathers regular moons. Solids no region gathered are the belt beyond
//! the outermost world, out to the disc's edge, and the giants fling a comet
//! cloud.
//!
//! The matter: a world's solids are apportioned among all 150 elements by their
//! chemistry. An element is in the solids only where the disc was cooler than
//! its 50% condensation temperature; siderophile and chalcophile elements go to
//! the metal core, lithophile to the silicate mantle (three oxygens per silicon); carbon, nitrogen and the remaining oxygen only as ices
//! beyond their lines; an envelope is primordial hydrogen and helium. Each
//! element's weight is its mass times its abundance in the gas at the star's
//! birth (chem.rs), so ice-to-rock ratio, core share and trace elements follow
//! the galaxy's history. Masses are exact, in world masses, and sum to the
//! world exactly.
//!
//! Orbital elements and spins are computed in bodies.rs; the vertical exposure
//! that thins comet clouds is not yet modelled.

use crate::chem::Chemistry;
use crate::elements::extent;
use crate::elements::{decimal, Class, Element};
use crate::galaxy::{density, Star};
use xyl_core::certified::{down, pi, Encl};
use xyl_core::dyadic::Dy;
use xyl_core::{Stream, Q};

const BITS: u32 = 64;
const GRID_BITS: u32 = 40;

fn d(s: &str) -> Q {
    decimal(s).unwrap_or(Q::ZERO)
}

fn q(n: i64, den: i64) -> Q {
    Q::new(n, den).unwrap_or(Q::ZERO)
}

fn fix(e: &Encl) -> Q {
    down(&e.mid(), GRID_BITS)
}

/// Rounds a base to dyadic precision before a certified power or exponential is
/// taken of it. Abundances reach 1e-67 per hydrogen, so exact denominators run
/// to hundreds of bits; the result is fixed on the grid anyway.
fn held(x: &Q) -> Q {
    xyl_core::dyadic::Dy::from_q(x).to_q()
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
    q(x, 1 << 53)
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
        let mass = star.mass.clone().max(q(1, 10_000));
        let lum = luminosity(&mass, star.reach_rings());
        Spec { mass, lum }
    }

    /// The disc's temperature (K) at reach a (AU): `278 L / a`.
    pub fn t_form(&self, a: &Q) -> Q {
        (&Q::int(278) * &self.lum).checked_div(a).unwrap_or(Q::ZERO)
    }
}

/// An ice: its species, condensation temperature (K), and (element, atoms).
type Ice = (&'static str, i64, &'static [(&'static str, i64)]);

/// The share of each siderophile and chalcophile element left in the silicate
/// mantle by core formation: mass in the reference world's bulk silicate over mass in the
/// whole body (McDonough 2003, Treatise on Geochemistry 2.15: BSE ppm x 0.675 /
/// bulk ppm). Declared ruler, measured on the reference world and applied to all
/// worlds; values as transcribed, to be checked against the source table. An
/// element not listed keeps none in its mantle.
const MANTLE_KEPT: [(&str, &str); 30] = [
    ("Fe", "0.1325"),
    ("Ni", "0.0726"),
    ("Co", "0.0805"),
    ("S", "0.0266"),
    ("Cu", "0.3375"),
    ("Zn", "0.928"),
    ("Ga", "0.9"),
    ("Ge", "0.106"),
    ("As", "0.0199"),
    ("Se", "0.0188"),
    ("Mo", "0.0199"),
    ("Ru", "0.0026"),
    ("Rh", "0.0025"),
    ("Pd", "0.0026"),
    ("Ag", "0.108"),
    ("Cd", "0.3375"),
    ("In", "1"),
    ("Sn", "0.351"),
    ("Sb", "0.0743"),
    ("Te", "0.027"),
    ("W", "0.115"),
    ("Re", "0.0025"),
    ("Os", "0.00255"),
    ("Ir", "0.00257"),
    ("Pt", "0.00252"),
    ("Au", "0.0042"),
    ("Hg", "0.3375"),
    ("Tl", "0.197"),
    ("Pb", "0.44"),
    ("Bi", "0.169"),
];

/// The share of a siderophile or chalcophile element its mantle keeps.
pub fn mantle_kept(sym: &str) -> Q {
    MANTLE_KEPT
        .iter()
        .find(|(s, _)| *s == sym)
        .map(|(_, v)| d(v))
        .unwrap_or(Q::ZERO)
}

const ICES: [Ice; 6] = [
    ("H2O", 182, &[("H", 2), ("O", 1)]),
    ("NH3", 131, &[("N", 1), ("H", 3)]),
    ("CO2", 72, &[("C", 1), ("O", 2)]),
    ("CH4", 41, &[("C", 1), ("H", 4)]),
    ("N2", 22, &[("N", 2)]),
    ("CO", 25, &[("C", 1), ("O", 1)]),
];
/// How a cooling nebula holds its carbon and nitrogen.
fn carbon_as(sp: &str) -> Q {
    match sp {
        "CO" => q(1, 2),
        "CO2" => q(1, 5),
        "CH4" => q(3, 10),
        _ => Q::ZERO,
    }
}
fn nitrogen_as(sp: &str) -> Q {
    match sp {
        "NH3" | "N2" => q(1, 2),
        _ => Q::ZERO,
    }
}

/// The local gas: each element's atoms per 10^6 Si, here and then.
pub struct Gas<'a> {
    /// The element table the counts are indexed by.
    pub elements: &'a [Element],
    /// Each element's atoms per 10^6 Si atoms, on a 2^-32 grid, indexed as
    /// `elements`.
    pub atoms: Vec<Q>,
    /// Primordial hydrogen mass fraction, a share in \[0, 1\].
    pub x: Q,
    /// Primordial helium-4 mass fraction, a share in \[0, 1\].
    pub y: Q,
}

impl<'a> Gas<'a> {
    /// The gas a star at `r`, born at `born`, formed from (its merger lump `lump`).
    ///
    /// `weights` is its galactic core's recipe (cores.rs): each metal's share
    /// tilted by its dial, the metals' mass kept.
    pub fn at(
        chem: &Chemistry,
        elements: &'a [Element],
        r: &Q,
        born: &Q,
        lump: &Q,
        weights: &crate::cores::Weights,
    ) -> Gas<'a> {
        // the composition is fixed on a grid (2^-32 of an atom per 10^6 Si) so
        // every world's shares divide by small numbers
        let grid = Q::int(1 << 32);
        // counts as atoms per 10^6 Si, whatever unit the founding uses (only
        // ratios matter), floored exactly
        let si = elements
            .iter()
            .find(|e| e.sym == "Si")
            .map(|e| e.abundance.clone())
            .filter(|x| x.signum() > 0);
        let per_si = si
            .and_then(|x| Q::int(1_000_000).checked_div(&x))
            .unwrap_or(Q::ONE);
        // every element's abundance read at once
        let rel = chem.relatives(r, born, lump).unwrap_or_default();
        let atoms = elements
            .iter()
            .enumerate()
            .map(|(i, e)| {
                if e.abundance.is_zero() {
                    return Q::ZERO;
                }
                let v = &(&(&(&e.abundance * &per_si) * rel.get(i).unwrap_or(&Q::ZERO))
                    * &weights.factor(&e.sym))
                    * &grid;
                (&v - &v.frac()).checked_div(&grid).unwrap_or(Q::ZERO)
            })
            .collect();
        Gas {
            elements,
            atoms,
            x: chem.light.x.clone(),
            y: chem.light.y.clone(),
        }
    }

    fn idx(&self, sym: &str) -> Option<usize> {
        self.elements.iter().position(|e| e.sym == sym)
    }

    fn n(&self, sym: &str) -> Q {
        self.idx(sym)
            .and_then(|i| self.atoms.get(i).cloned())
            .unwrap_or(Q::ZERO)
    }

    /// Per unit of rock: the core's share, and each ice (by mass) the disc holds
    /// below `t_form`, from this gas's own abundances (worlds._solids).
    pub fn solids(&self, t_form: &Q) -> (Q, Vec<(&'static str, Q)>) {
        // the metals: what core formation sank, and what the mantle kept (iron
        // as FeO, carrying its oxygen)
        let metal = |sym: &str, a: i64| &Q::int(a) * &self.n(sym);
        let (fe, ni, s) = (metal("Fe", 56), metal("Ni", 59), metal("S", 32));
        let (kfe, kni, ks) = (mantle_kept("Fe"), mantle_kept("Ni"), mantle_kept("S"));
        let sunk = |m: &Q, k: &Q| m * &(&Q::ONE - k);
        let core = &(&sunk(&fe, &kfe) + &sunk(&ni, &kni)) + &sunk(&s, &ks);
        let kept = &(&(&fe * &kfe) + &(&ni * &kni)) + &(&s * &ks);
        let feo_oxygen = &(&Q::int(16) * &self.n("Fe")) * &kfe;
        let mantle = &(&(&(&Q::int(24) * &self.n("Mg")) + &(&Q::int(76) * &self.n("Si"))) + &kept)
            + &feo_oxygen;
        let rock = &core + &mantle;
        if rock.is_zero() {
            return (Q::ZERO, Vec::new());
        }
        let c = self.n("C");
        let o_left = (&(&self.n("O") - &(&Q::int(3) * &self.n("Si")))
            - &(&c * &(&carbon_as("CO") + &(&Q::int(2) * &carbon_as("CO2")))))
            .max(Q::ZERO);
        let mass_of = |sp: &str| -> Q {
            match sp {
                "H2O" => &Q::int(18) * &o_left,
                "CO2" => &(&Q::int(44) * &c) * &carbon_as("CO2"),
                "CH4" => &(&Q::int(16) * &c) * &carbon_as("CH4"),
                "CO" => &(&Q::int(28) * &c) * &carbon_as("CO"),
                "NH3" => &(&Q::int(17) * &self.n("N")) * &nitrogen_as("NH3"),
                "N2" => (&(&Q::int(28) * &self.n("N")) * &nitrogen_as("N2"))
                    .checked_div(&Q::int(2))
                    .unwrap_or(Q::ZERO),
                _ => Q::ZERO,
            }
        };
        let ices = ICES
            .iter()
            .filter(|(_, cond, _)| t_form < &Q::int(*cond))
            .map(|(sp, _, _)| (*sp, mass_of(sp).checked_div(&rock).unwrap_or(Q::ZERO)))
            .collect();
        (core.checked_div(&rock).unwrap_or(Q::ZERO), ices)
    }

    /// A body's element masses (world masses), exact, from its layers:
    /// `core`, `mantle` and `envelope` masses, and each ice's mass, condensed
    /// where the disc was at `t_form`.
    pub fn apportion(
        &self,
        t_form: &Q,
        core: &Q,
        mantle: &Q,
        ices: &[(&str, Q)],
        envelope: &Q,
    ) -> Vec<Q> {
        self.apportion_layers(t_form, core, mantle, ices, envelope)
            .0
    }

    /// As `apportion`, with the mantle's own share of each element beside the
    /// whole (what the crust is made from).
    pub fn apportion_layers(
        &self,
        t_form: &Q,
        core: &Q,
        mantle: &Q,
        ices: &[(&str, Q)],
        envelope: &Q,
    ) -> (Vec<Q>, Vec<Q>) {
        let n = self.elements.len();
        let mut out = vec![Q::ZERO; n];
        let mut in_mantle = vec![Q::ZERO; n];
        let condensed = |e: &Element| e.tc.as_ref().is_some_and(|tc| tc > t_form);
        // a layer's mass shared among its elements by weight, exactly
        let shares = |mass: &Q, weights: Vec<(usize, Q)>| -> Vec<(usize, Q)> {
            let total = weights.iter().fold(Q::ZERO, |a, (_, w)| &a + w);
            if total.is_zero() || mass.is_zero() {
                return Vec::new();
            }
            weights
                .into_iter()
                .map(|(i, w)| (i, (mass * &w).checked_div(&total).unwrap_or(Q::ZERO)))
                .collect()
        };
        let w = |e: &Element, i: usize| &e.a * &self.atoms[i];
        let metal =
            |e: &Element| matches!(e.class, Some(Class::Sidero | Class::Chalco)) && condensed(e);
        // the metal core: what core formation sank of the siderophile and
        // chalcophile elements that condensed
        for (i, m) in shares(
            core,
            self.elements
                .iter()
                .enumerate()
                .filter(|(_, e)| metal(e))
                .map(|(i, e)| (i, &w(e, i) * &(&Q::ONE - &mantle_kept(&e.sym))))
                .collect(),
        ) {
            out[i] = &out[i] + &m;
        }
        // the silicate mantle: lithophiles that condensed, the metals it kept,
        // oxygen as three per silicon and one per iron it kept (FeO)
        let oi = self.idx("O");
        let o_weight = |e: &Element| {
            &(&(&e.a * &Q::int(3)) * &self.n("Si"))
                + &(&(&e.a * &self.n("Fe")) * &mantle_kept("Fe"))
        };
        for (i, m) in shares(
            mantle,
            self.elements
                .iter()
                .enumerate()
                .filter_map(|(i, e)| {
                    if Some(i) == oi {
                        Some((i, o_weight(e)))
                    } else if e.class == Some(Class::Litho) && condensed(e) {
                        Some((i, w(e, i)))
                    } else if metal(e) {
                        Some((i, &w(e, i) * &mantle_kept(&e.sym)))
                    } else {
                        None
                    }
                })
                .filter(|(_, wt)| !wt.is_zero())
                .collect(),
        ) {
            out[i] = &out[i] + &m;
            in_mantle[i] = &in_mantle[i] + &m;
        }
        // the ices, by their own formulas
        for (sp, mass) in ices {
            let Some((_, _, parts)) = ICES.iter().find(|(s, _, _)| s == sp) else {
                continue;
            };
            for (i, m) in shares(
                mass,
                parts
                    .iter()
                    .filter_map(|(sym, k)| {
                        self.idx(sym)
                            .map(|i| (i, &self.elements[i].a * &Q::int(*k)))
                    })
                    .collect(),
            ) {
                out[i] = &out[i] + &m;
            }
        }
        // an envelope: the Big Bang's hydrogen and helium
        if let (Some(h), Some(he)) = (self.idx("H"), self.idx("He")) {
            out[h] = &out[h] + &(envelope * &self.x);
            out[he] = &out[he] + &(envelope * &self.y);
        }
        (out, in_mantle)
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

/// A small body of a disc on its circle: its reach (AU, or a
/// giant's radii for a moon), its matter (world masses) and its phase (turns).
#[derive(Clone, Debug)]
struct Embryo {
    a: Q,
    m: Q,
    phase: Q,
    /// The greatest swing its neighbours give it at the settled age, as a
    /// share of its reach (settle_about): its starting eccentricity and tilt.
    swing: Q,
    /// The age (Gyr) at which it merges into its heavier neighbour; None for
    /// a body that stands to the horizon.
    merged_at: Option<Q>,
}

/// The white hole's units for the settling: r_g in metres and the core's mass
/// in kilograms.
fn rg_units() -> (xyl_core::dyadic::Dy, xyl_core::dyadic::Dy) {
    use xyl_core::dyadic::Dy;
    let r_g = Dy::from_q(&crate::chain::units().r_g_m);
    let core_kg = Dy::from_q(&(&crate::package::hole_mass() * &crate::package::Si::star_mass_kg()));
    (r_g, core_kg)
}

/// A planet's reach a (AU) in r_g.
fn planet_reach(a: &Q) -> xyl_core::dyadic::Dy {
    use xyl_core::dyadic::Dy;
    Dy::from_q(a) * Dy::from_q(&crate::package::Si::au()) / rg_units().0
}

/// Settles a star's disc (see [`settle_about`]): the swings at the settled
/// age, the merges out to life's horizon.
fn settle(b: Vec<Embryo>, star_suns: &Q, settled: &Q) -> Vec<Embryo> {
    let (_, core_kg) = rg_units();
    let anchor = xyl_core::dyadic::Dy::from_q(star_suns)
        * xyl_core::dyadic::Dy::from_q(&crate::package::Si::star_mass_kg())
        / core_kg;
    settle_about(
        b,
        anchor,
        &planet_reach,
        settled,
        &crate::life::horizon_gyr().to_q(),
    )
}

/// The settling about an anchor (a star, or a giant for its moons), its mass
/// and each body's reach as lengths in r_g (G = c = M_core = 1).
///
/// Each body turns at omega(A) = [M / (A + M)] / A. Neighbours meet once per
/// synodic period; each meeting turns them by
/// eps = [m / (r + m)] / r / |omega_i - omega_j|, and over an age they meet
/// N = age |omega_i - omega_j| times. The phase step per meeting,
/// s = frac(omega_i / |omega_i - omega_j|), is approximated by the smallest
/// continued-fraction convergent p/q with |s - p/q| <= 1 / (N q), so kicks
/// repeat every q meetings and a full period cancels to c = |1 - 2 frac(q psi_0)|
/// (psi_0 the pair's phase offset). The greatest swing (the kick) is
/// eps (floor(N / q) c + N mod q), a share of the inner reach.
///
/// The settling is carried as ages, in closed form: a pair's merge age is
/// the first age at which its kick times the inner reach spans the gap
/// (found by bisection out to `horizon`; never, where it does not by then).
/// The earliest pair merges first: the lighter body ends at that age (its
/// `merged_at`) and the heavier carries on with the summed mass, reach and
/// phase mass-weighted; the merged body's pairs are read again with the
/// merged mass, so a merger that now spans a further gap merges again.
/// Resonances (small q) merge, near-incommensurate pairs remain. The bodies
/// standing at `settled` are returned, and those that merge after it with
/// their merge age (a merge before the settled age is the disc's own
/// settling); each standing body's `swing` is the greatest kick its
/// neighbours give it at `settled`, as a share of its own reach, and a
/// merging body's is the gap it closes.
fn settle_about(
    mut b: Vec<Embryo>,
    anchor: xyl_core::dyadic::Dy,
    reach: &dyn Fn(&Q) -> xyl_core::dyadic::Dy,
    settled: &Q,
    horizon: &Q,
) -> Vec<Embryo> {
    use xyl_core::dyadic::Dy;
    let (r_g, core_kg) = rg_units();
    let unit_world = Dy::from_q(&crate::package::Si::world_mass_kg()) / core_kg;
    // light-time per Gyr in r_g: an age in Gyr as a span in the core's units
    let per_gyr =
        Dy::from_q(&crate::package::Si::gyr_s()) * Dy::from_q(&crate::package::Si::c()) / r_g;
    // the turning law: delta / r
    let omega = |a: Dy| (anchor / (a + anchor)) / a;
    // the smallest period q of the phase step s the age can tell apart
    let period = |s: Dy, n: Dy| -> i64 {
        let (mut x, mut h0, mut h1, mut k0, mut k1) = (s, 0i64, 1i64, 1i64, 0i64);
        for _ in 0..48 {
            let a = x.trunc_i64();
            (h0, h1) = (h1, a.saturating_mul(h1).saturating_add(h0));
            (k0, k1) = (k1, a.saturating_mul(k1).saturating_add(k0));
            if k1 > 0 && (s - Dy::int(h1) / Dy::int(k1)).abs() * n * Dy::int(k1) <= Dy::ONE {
                return k1;
            }
            let fr = x - Dy::int(a);
            if fr <= Dy::sci(1, -15) {
                return k1.max(1);
            }
            x = Dy::ONE / fr;
        }
        k1.max(1)
    };
    // a pair's kick over `gyr`: its greatest swing as a share of the inner
    // reach; a pair at one reach swings without bound
    let kick = |x: &Embryo, y: &Embryo, gyr: Dy| -> Dy {
        let (a1, a2) = (reach(&x.a), reach(&y.a));
        let r = (a2 - a1).abs();
        if r.is_zero() {
            return Dy::sci(1, 30);
        }
        let mm = (Dy::from_q(&x.m) + Dy::from_q(&y.m)) * unit_world;
        let (w1, w2) = (omega(a1), omega(a2));
        let dw = (w1 - w2).abs();
        if dw.is_zero() {
            return Dy::ZERO;
        }
        let eps = (mm / (r + mm)) / r / dw;
        let n = gyr * per_gyr * dw;
        let s = w1.max(w2) / dw;
        let s = s - Dy::int(s.trunc_i64());
        let q = period(s, n.max(Dy::ONE));
        let off = Dy::int(q) * Dy::from_q(&(&y.phase - &x.phase));
        let mut f = off - Dy::int(off.trunc_i64());
        if f.signum() < 0 {
            f += Dy::ONE;
        }
        let c = (Dy::ONE - Dy::int(2) * f).abs();
        let whole = Dy::int((n / Dy::int(q)).trunc_i64());
        let rest = n - whole * Dy::int(q);
        eps * (whole * c + rest)
    };
    // the pair's swing against its gap; at or past 1 it converges
    let score = |x: &Embryo, y: &Embryo, gyr: Dy| -> Dy {
        let (a1, a2) = (reach(&x.a), reach(&y.a));
        let r = (a2 - a1).abs();
        if r.is_zero() {
            return Dy::int(2);
        }
        kick(x, y, gyr) * a1.min(a2) / r
    };
    // the first age the pair converges by, out to the horizon
    let top = Dy::from_q(horizon);
    let merge_age = |x: &Embryo, y: &Embryo| -> Option<Dy> {
        if score(x, y, top) < Dy::ONE {
            return None;
        }
        let (mut lo, mut hi) = (Dy::ZERO, top);
        for _ in 0..48 {
            let mid = (lo + hi) / Dy::int(2);
            if score(x, y, mid) >= Dy::ONE {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        Some(hi)
    };
    // the disc's own settling: pairs that meet by the settled age merge, the
    // heavier carrying the sum; a pair that meets later stands as two bodies
    // until it does (its meeting recorded on the lighter, no mass moved), so
    // every body's mass is counted once
    let by_settled = Dy::from_q(settled);
    let mut ages: Vec<Option<Dy>> = b.windows(2).map(|w| merge_age(&w[0], &w[1])).collect();
    while let Some((i, _)) = ages
        .iter()
        .enumerate()
        .filter_map(|(i, a)| a.map(|t| (i, t)))
        .filter(|(_, t)| *t <= by_settled)
        .min_by(|x, y| x.1.cmp(&y.1))
    {
        let (x, y) = (b[i].clone(), b[i + 1].clone());
        let m = &x.m + &y.m;
        let by = |u: &Q, v: &Q| {
            held(
                &(&(u * &x.m) + &(v * &y.m))
                    .checked_div(&m)
                    .unwrap_or_else(|| u.clone()),
            )
        };
        b[i] = Embryo {
            a: by(&x.a, &y.a),
            m: held(&m),
            phase: by(&x.phase, &y.phase).frac(),
            swing: Q::ZERO,
            merged_at: None,
        };
        b.remove(i + 1);
        ages.remove(i);
        if i < ages.len() {
            ages[i] = merge_age(&b[i], &b[i + 1]);
        }
        if i > 0 {
            ages[i - 1] = merge_age(&b[i - 1], &b[i]);
        }
    }
    // each survivor's swing at the settled age: the greatest kick its
    // neighbours give it, as a share of its own reach
    let at = Dy::from_q(settled);
    let swings: Vec<Q> = (0..b.len())
        .map(|i| {
            let mut s = Dy::ZERO;
            for j in [i.checked_sub(1), Some(i + 1)].into_iter().flatten() {
                if j >= b.len() {
                    continue;
                }
                let (x, y) = if i < j {
                    (&b[i], &b[j])
                } else {
                    (&b[j], &b[i])
                };
                let inner = reach(&x.a).min(reach(&y.a));
                let own = reach(&b[i].a).max(Dy::sci(1, -30));
                s = s.max(kick(x, y, at) * inner / own);
            }
            held(&s.min(Dy::ONE).to_q())
        })
        .collect();
    for (e, sw) in b.iter_mut().zip(swings) {
        e.swing = sw;
    }
    // the meetings after the settled age: each pair's lighter body carries
    // the age it meets its neighbour, the earliest if it has two
    for (i, age) in ages.iter().enumerate() {
        let Some(t) = *age else { continue };
        let lighter = if b[i].m >= b[i + 1].m { i + 1 } else { i };
        let when = held(&t.max(Dy::ZERO).to_q());
        let e = &mut b[lighter];
        if e.merged_at.as_ref().is_none_or(|m| &when < m) {
            e.merged_at = Some(when);
        }
    }
    b
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
    // weighted by its galaxy's composition then (founding recipe plus what it
    // took from the core); the same path for the home galaxy and the others
    let weights = crate::cores::weights_at(seed, bang, star.core, &star.born, elements)?;
    let (gas, z) = (
        Gas::at(chem, elements, &star.r, &star.born, &lump, &weights),
        chem.metals(&star.r, &star.born)?,
    );
    if z.signum() <= 0 {
        return Some(System {
            spec,
            worlds: Vec::new(),
            belts: Vec::new(),
            comets: None,
        });
    }
    let env = density(&star.r).mid();
    // the chain: the star's manifold about the core, its gathering the inner
    // share; the core's twist at the star (the tide, the core-star tie) is
    // what each world must hold its region against (disc::manifold_worlds),
    // so the core-star-planet chain is read in one law
    let Some(frame) = crate::galaxy::disc_frame(&star.r, &spec.mass) else {
        return Some(System {
            spec,
            worlds: Vec::new(),
            belts: Vec::new(),
            comets: None,
        });
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
    // against the Sun's birth gas, times the Sun's: Rulers::reference_metallicity, a ruler)
    let star_earths = &spec.mass
        * &crate::package::Si::star_mass_kg()
            .checked_div(&crate::package::Si::world_mass_kg())
            .unwrap_or(Q::ZERO);
    let infall_solids = held(
        &(&star_earths
            .checked_div(&frame.star_share.clone().max(d("1e-9")))
            .unwrap_or(Q::ZERO)
            * &(&z * &crate::chem::Rulers::reference_metallicity())),
    );
    // the rock line: where the light's warmth falls to the lowest
    // condensation temperature of Fe, Si and Mg (from the element table)
    let t_rock = ["Fe", "Si", "Mg"]
        .iter()
        .filter_map(|sym| {
            elements
                .iter()
                .find(|e| e.sym == *sym)
                .and_then(|e| e.tc.clone())
        })
        .min()
        .unwrap_or(Q::int(1300));
    // the disc's solid surface density at the disc's mean reach (g/cm^2),
    // for a core's gathering of gas
    let au_cm = &crate::package::Si::au() * &Q::int(100);
    let sigma1 = (&infall_solids * &(&crate::package::Si::world_mass_kg() * &Q::int(1000)))
        .checked_div(
            &(&(&xyl_core::certified::pi(64).mid() * &a_out) * &(&a_out * &au_cm.clone() * &au_cm)),
        )
        .unwrap_or(Q::ZERO);
    let mut worlds: Vec<World> = Vec::new();
    // the disc's ungathered solids: the belts', less what worlds take as moons
    let mut moon_pool = held(&(&belt_share * &infall_solids));
    let mut giants_mass = Q::ZERO;
    let mut orbits: Vec<(Q, bool)> = Vec::new();
    if a_out.signum() > 0 && !regions.is_empty() {
        // each region is a world's holding: its solids, enriched past each
        // ice line, at its reach; its phase k times the golden turn. The
        // settling reads the regions' swings and merges what still spans a gap
        let mut embryos: Vec<Embryo> = Vec::new();
        for (k, (reach, holding)) in regions.iter().enumerate() {
            let a = held(&(&a_out * reach));
            let t_form = spec.t_form(&a);
            if t_form > t_rock {
                continue;
            }
            let (_, ices) = gas.solids(&t_form);
            let has_water = ices.iter().any(|(sp, _)| *sp == "H2O");
            let enrich = if has_water {
                ices.iter().fold(Q::ONE, |a, (_, m)| &a + &(m * &z))
            } else {
                Q::ONE
            };
            let m = held(&(&(holding * &infall_solids) * &enrich));
            if m.signum() <= 0 {
                continue;
            }
            embryos.push(Embryo {
                a,
                m,
                phase: (&Q::int(i64::try_from(k).unwrap_or(0)) * &crate::galaxy::golden_turns())
                    .frac(),
                swing: Q::ZERO,
                merged_at: None,
            });
        }
        // the settling: neighbours merge until every neighbour stands
        let settled = settle(embryos, &spec.mass, &settled_gyr());
        for e in settled {
            let (swing, phase, merged_at) = (e.swing.clone(), e.phase.clone(), e.merged_at.clone());
            let oq = e.a;
            let t_form = spec.t_form(&oq);
            let (core_share, mut ices) = gas.solids(&t_form);
            let has_water = ices.iter().any(|(s, _)| *s == "H2O");
            if !has_water {
                // inside the snow line its ices come late, by comet
                let late = &(&d("1e-4") * &env) * &z;
                ices = vec![
                    ("H2O", late.clone()),
                    ("CO2", &late * &d("0.1")),
                    ("CO", &late * &d("0.1")),
                    ("CH4", &late * &d("0.01")),
                    ("NH3", &late * &d("0.01")),
                    ("N2", &late * &d("0.005")),
                ];
            }
            let ices: Vec<(&str, Q)> = if has_water {
                ices.into_iter().map(|(s, m)| (s, &m * &z)).collect()
            } else {
                ices
            };
            // the core is the matter its settling gathered
            let core = e.m;
            let mut envelope = Q::ZERO;
            // a core past ten world masses takes gas in the share its growth
            // beat the disc's dispersal. Growth time: its turning period over
            // the period at 1 AU, a (a + M) / (1 + M) with M the star as a
            // length in AU (G M / c^2 per sun, in metres), over the surface
            // density
            let m_au = (&spec.mass * &star_length_m())
                .checked_div(&crate::package::Si::au())
                .unwrap_or(Q::ZERO);
            let period = (&oq * &(&oq + &m_au)).checked_div(&(&Q::ONE + &m_au))?;
            let grow = period
                .checked_div(&Q::int(25))?
                .checked_div(&sigma1.checked_div(&Q::int(10))?.max(d("1e-6")))?;
            let keep = fix(&exp(&-(grow.checked_div(&Q::int(3))?), BITS));
            if core >= Q::int(10) {
                let gathering = core.checked_div(&(&core + &Q::int(10)))?;
                envelope = held(
                    &(&(&(&(&core * &keep) * &gathering) * &d("5.6"))
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
                &extent(&core) * &(&Q::ONE + &(&d("0.4") * &ice_share))
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
            let icy = ice_share > q(1, 5);
            // a world is lost when its survival share is below one half
            let lost = survival(&oq, &star.r, &settled_gyr()) < q(1, 2);
            if is_world(&total, &radius, icy) && !lost {
                let giant = envelope.signum() > 0;
                if giant {
                    giants_mass = &giants_mass + &total;
                }
                let layers = [
                    c_mass.checked_div(&total)?,
                    m_mass.checked_div(&total)?,
                    (&core * &ice_share).checked_div(&total)?,
                    envelope.checked_div(&total)?,
                ];
                let (elems, mantle) =
                    gas.apportion_layers(&t_form, &c_mass, &m_mass, &ice_mass, &envelope);
                worlds.push(World {
                    a: oq.clone(),
                    mass: total.clone(),
                    radius: radius.clone(),
                    giant,
                    host: None,
                    a_host_radii: None,
                    layers,
                    elements: elems,
                    mantle,
                    ices: ices.clone(),
                    e_forced: None,
                    swing: swing.clone(),
                    phase: phase.clone(),
                    merged_at: merged_at.clone(),
                });
                orbits.push((oq.clone(), giant));
                // every world anchors its own manifold: its moons gather there
                // from the disc's unheld matter inside it, against the star's
                // tide at the world (hold over reach plus hold, in the star's
                // manifold units: s / (x + s))
                let star_earths = &spec.mass
                    * &crate::package::Si::star_mass_kg()
                        .checked_div(&crate::package::Si::world_mass_kg())
                        .unwrap_or(Q::ZERO);
                let world_reach_au = &(&(&Q::int(2) * &pi(BITS).mid()) * &oq)
                    * &total
                        .checked_div(&(&star_earths + &total))
                        .unwrap_or(Q::ZERO);
                let width = world_reach_au.checked_div(&a_out).unwrap_or(Q::ZERO);
                let budget =
                    held(&(&(&frame.a_disc * &width) * &infall_solids).min(moon_pool.clone()));
                let x = oq.checked_div(&a_out).unwrap_or(Q::ONE);
                let tide = frame
                    .star_share
                    .checked_div(&(&x + &frame.star_share))
                    .unwrap_or(Q::ZERO);
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
    }
    let (belts, comets) = belts(&spec, &gas, &moon_pool, &a_out, &orbits, &giants_mass, &env);
    Some(System {
        spec,
        worlds,
        belts,
        comets,
    })
}

/// A world's moons, on its own manifold: it anchors the reach its pull holds
/// in its star's field, 2 pi a m / (M + m), out from five of its radii. The
/// matter is `budget` (world masses, the disc's unheld matter inside that
/// reach); the world holds its own share, m / (m + budget), and moons gather
/// across the rest degree by degree where their matter holds its region
/// against the star's `tide` at the world (disc::manifold_worlds, the
/// star tier's law one tier down), then settle about the world (see
/// [`settle_about`]). Each moon's forced eccentricity is its neighbours' pull
/// over their rates' distance from a first-order resonance. Returns the
/// moons' mass placed, which leaves the belts' books.
fn moons(
    worlds: &mut Vec<World>,
    gas: &Gas,
    t_form: &Q,
    ice_share: &Q,
    star_suns: &Q,
    budget: &Q,
    tide: &Q,
) -> Q {
    use xyl_core::dyadic::Dy;
    let Some(parent) = worlds.len().checked_sub(1) else {
        return Q::ZERO;
    };
    if budget.signum() <= 0 {
        return Q::ZERO;
    }
    let (g_mass, g_a, g_ices, g_radius) = (
        worlds[parent].mass.clone(),
        worlds[parent].a.clone(),
        worlds[parent].ices.clone(),
        worlds[parent].radius.clone(),
    );
    let star_earths = star_suns
        * &crate::package::Si::star_mass_kg()
            .checked_div(&crate::package::Si::world_mass_kg())
            .unwrap_or(Q::ZERO);
    // its reach in its star's field, in its own radii: 2 pi a m / (M + m)
    let a_m = &(&g_a * &crate::package::Si::au());
    let reach_m = &(&(&Q::int(2) * &pi(BITS).mid()) * a_m)
        * &g_mass
            .checked_div(&(&star_earths + &g_mass))
            .unwrap_or(Q::ZERO);
    let r_m = &g_radius * &crate::package::Si::world_radius_m();
    let (inner, outer) = (
        Q::int(5),
        held(&reach_m.checked_div(&r_m).unwrap_or(Q::ZERO)),
    );
    if outer <= inner {
        return Q::ZERO;
    }
    // the world's manifold: it holds its own share, the moons the rest
    let all = &g_mass + budget;
    let own = g_mass.checked_div(&all).unwrap_or(Q::ONE);
    let (gathered, _) = crate::disc::manifold_worlds(&own, &Q::ZERO, tide);
    let embryos: Vec<Embryo> = gathered
        .iter()
        .enumerate()
        .map(|(k, (reach, holding))| Embryo {
            a: held(&(&outer * reach).max(inner.clone())),
            m: held(&(holding * &all)),
            phase: (&Q::int(i64::try_from(k).unwrap_or(0)) * &crate::galaxy::golden_turns()).frac(),
            swing: Q::ZERO,
            merged_at: None,
        })
        .collect();
    let (r_g, core_kg) = rg_units();
    let anchor = Dy::from_q(&g_mass) * Dy::from_q(&crate::package::Si::world_mass_kg()) / core_kg;
    let radius_rg = Dy::from_q(&r_m) / r_g;
    let settled = settle_about(
        embryos,
        anchor,
        &|a_r: &Q| Dy::from_q(a_r) * radius_rg,
        &settled_gyr(),
        &crate::life::horizon_gyr().to_q(),
    );
    // each moon's forced eccentricity: its neighbours' pull over their rates'
    // distance from the nearest first-order resonance p / (p + 1)
    let rate = |a_r: &Q| {
        let a = Dy::from_q(a_r) * radius_rg;
        (anchor / (a + anchor)) / a
    };
    let forced: Vec<Q> = (0..settled.len())
        .map(|k| {
            let mut f = Dy::ZERO;
            for j in [k.checked_sub(1), Some(k + 1)].into_iter().flatten() {
                let Some(nb) = settled.get(j) else { continue };
                let (w1, w2) = (rate(&settled[k].a), rate(&nb.a));
                let ratio = w1.min(w2) / w1.max(w2);
                let p = (ratio / (Dy::ONE - ratio).max(Dy::sci(1, -12)))
                    .trunc_i64()
                    .max(1);
                let near = [p - 1, p, p + 1]
                    .iter()
                    .filter(|&&x| x >= 1)
                    .map(|&x| (ratio - Dy::int(x) / Dy::int(x + 1)).abs())
                    .fold(Dy::ONE, Dy::min)
                    .max(Dy::sci(1, -3));
                f = f.max(Dy::from_q(&nb.m) / Dy::from_q(&g_mass) / near);
            }
            f.min(Dy::ratio(1, 10)).to_q()
        })
        .collect();
    let rho = (&Q::ONE - ice_share) + (ice_share * &d("5.5"));
    let icy = ice_share > &q(1, 5);
    let mut placed = Q::ZERO;
    for (e, e_forced) in settled.into_iter().zip(forced) {
        let m = e.m;
        if m < d("1e-4") {
            continue;
        }
        let radius = extent(&(&m * &rho.recip().unwrap_or(Q::ONE)));
        if is_world(&m, &radius, icy) {
            let share = if icy { ice_share.clone() } else { Q::ZERO };
            let (core_share, ices) = gas.solids(t_form);
            let rock = &m * &(&Q::ONE - &share);
            let ice_sum = ices.iter().fold(Q::ZERO, |a, (_, x)| &a + x);
            let ice_mass: Vec<(&str, Q)> = ices
                .iter()
                .map(|(sp, x)| {
                    (
                        *sp,
                        (&(&m * &share) * x)
                            .checked_div(&ice_sum)
                            .unwrap_or(Q::ZERO),
                    )
                })
                .collect();
            let (elems, mantle) = gas.apportion_layers(
                t_form,
                &(&rock * &core_share),
                &(&rock * &(&Q::ONE - &core_share)),
                &ice_mass,
                &Q::ZERO,
            );
            let layers = [
                &(&Q::ONE - &share) * &core_share,
                &(&Q::ONE - &share) * &(&Q::ONE - &core_share),
                share.clone(),
                Q::ZERO,
            ];
            placed = &placed + &m;
            worlds.push(World {
                a: g_a.clone(),
                mass: m,
                radius,
                giant: false,
                host: Some(parent),
                a_host_radii: Some(e.a),
                layers,
                elements: elems,
                mantle,
                ices: g_ices.clone(),
                e_forced: Some(e_forced),
                swing: e.swing.clone(),
                phase: e.phase.clone(),
                merged_at: e.merged_at.clone(),
            });
        }
    }
    placed
}

fn belts(
    spec: &Spec,
    gas: &Gas,
    ungathered: &Q,
    a_out: &Q,
    orbits: &[(Q, bool)],
    giants_mass: &Q,
    env: &Q,
) -> (Vec<Belt>, Option<(Q, Vec<Q>)>) {
    let mut out = Vec::new();
    if orbits.is_empty() {
        return (out, None);
    }
    let rock_of = |t_form: &Q, mass: &Q| -> Vec<Q> {
        let (core_share, ices) = gas.solids(t_form);
        let ice_sum = ices.iter().fold(Q::ZERO, |a, (_, x)| &a + x);
        let rock = mass.checked_div(&(&Q::ONE + &ice_sum)).unwrap_or(Q::ZERO);
        let ice_mass: Vec<(&str, Q)> = ices.iter().map(|(sp, x)| (*sp, &rock * x)).collect();
        gas.apportion(
            t_form,
            &(&rock * &core_share),
            &(&rock * &(&Q::ONE - &core_share)),
            &ice_mass,
            &Q::ZERO,
        )
    };
    // beyond the outermost world, what the regions never gathered (the
    // manifold's remainder, disc::manifold_worlds) out to its reach
    if let Some(a) = orbits.iter().map(|(a, _)| a.clone()).max() {
        if ungathered.signum() > 0 && *a_out > a {
            let mid = (&a + a_out).checked_div(&Q::int(2)).unwrap_or(Q::ONE);
            out.push(Belt {
                elements: rock_of(&spec.t_form(&mid), ungathered),
                inner_a: a,
                outer_a: a_out.clone(),
                mass: ungathered.clone(),
            });
        }
    }
    // the comet cloud: what the giants flung out, as the neighbourhood lets it keep
    let comets = (giants_mass.signum() > 0).then(|| {
        let kept = fix(&exp(&-env.clone(), BITS));
        let mass = &(&(giants_mass * &d("0.1")) * &d("0.05")) * &kept;
        let far = orbits
            .iter()
            .map(|(a, _)| a.clone())
            .max()
            .unwrap_or(Q::ONE);
        let t = spec.t_form(&(&far * &Q::int(30)));
        (mass.clone(), rock_of(&t, &mass))
    });
    (out, comets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::galaxy::star;

    #[test]
    fn worlds_on_the_manifold_close_their_books_and_hold_their_mass_exactly() {
        let bang = crate::genesis::bang();
        let chem = crate::genesis::chem();
        let elements = crate::elements::table().unwrap();
        let mut framed = 0;
        'outer: for chunk in [0u64, 4000, 9000] {
            for jj in 0..4u64 {
                for k in 0..40u64 {
                    let n = chunk + jj * 40 + k;
                    let Some(star) = star(7, bang, n) else {
                        continue;
                    };
                    // the chain at the star: a standing star's piece lights,
                    // so it has a frame, and the disc law's books close on it:
                    // the regions' holdings and the belt are the disc, the
                    // infall less the star's share
                    let frame = crate::galaxy::disc_frame(&star.r, &star.mass)
                        .expect("a standing star's frame");
                    let (regions, belt) =
                        crate::disc::manifold_worlds(&frame.star_share, &frame.a_disc, &frame.tide);
                    let gathered = regions.iter().fold(Q::ZERO, |a, (_, h)| &a + h);
                    assert_eq!(
                        &gathered + &belt,
                        &Q::ONE - &frame.star_share,
                        "the disc's books: regions {gathered} + belt {belt}"
                    );
                    for w in regions.windows(2) {
                        assert!(w[1].0 > w[0].0, "regions chain outward");
                    }
                    framed += 1;
                    let sys = system(7, bang, &[n], &star, chem, &elements).unwrap();
                    for w in &sys.worlds {
                        let sum = w.elements.iter().fold(Q::ZERO, |a, x| &a + x);
                        assert_eq!(sum, w.mass, "a world's elements sum to its mass exactly");
                        let layer_sum = w.layers.iter().fold(Q::ZERO, |a, x| &a + x);
                        assert_eq!(layer_sum, Q::ONE);
                        // the mantle holds exactly its layer, and core formation
                        // left it no more of each kind than the world holds
                        let mantle_mass = &w.mass * &w.layers[1];
                        let mantle_sum = w.mantle.iter().fold(Q::ZERO, |a, x| &a + x);
                        assert_eq!(
                            mantle_sum, mantle_mass,
                            "a mantle's elements sum to its layer exactly"
                        );
                        for (m, e) in w.mantle.iter().zip(&w.elements) {
                            assert!(m <= e, "the mantle holds no more than the world");
                        }
                    }
                    for b in &sys.belts {
                        let sum = b.elements.iter().fold(Q::ZERO, |a, x| &a + x);
                        assert_eq!(sum, b.mass, "a belt too");
                    }
                    if framed >= 4 {
                        break 'outer;
                    }
                }
            }
        }
        assert!(framed > 0, "some standing stars were framed");
    }
}
