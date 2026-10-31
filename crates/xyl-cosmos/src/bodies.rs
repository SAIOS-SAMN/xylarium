//! A world's body: its orbit and spin at any age, and its interior's heat
//! budget, temperatures and dynamo.
//!
//! Inputs: `systems::World` (mass and radius in world units, axis in AU,
//! layers, element masses, and the settling's swing and phase from
//! `systems::settle_about`), the host's mass in kg, `laws::Scales`, and SI
//! constants from `package::Si`. Mean motion and break-up spin come from the
//! turning law (`turning::turning_si`); the tidal stretch across a body is
//! its host's deficit difference `delta(a - R) - delta(a + R)`.
//!
//! - [`orbits`]: per world an [`Orbit`]. Eccentricity is the settled swing,
//!   capped at half the fractional gap to the nearest neighbour; inclination
//!   is `swing / (1 + swing)` of a quarter turn; node and periapsis are the
//!   settled phase; the forced eccentricity is the Laplace-Lagrange leading
//!   term `5/4 x axis ratio x neighbour e0`. Obliquity cosine (`2u - 1`) and
//!   birth spin (0.1 to 0.5 of break-up) are drawn from the caller's stream.
//!   Tidal circularisation ([`circularise_gyr`]) and spin lock ([`lock_gyr`])
//!   use Love number `k2` ([`love`]) and `Q = 100`.
//! - [`Orbit::at`]: eccentricity relaxed toward the forced value, spin braked
//!   linearly to lock, day and year lengths.
//! - [`interior`]: the [`Interior`] at an age in closed form. Heat is a sum
//!   of boosts (`xyl_core::motion::Modes`: U238, U235, Th232, K40 at
//!   half-lives scaled by `Scales`, plus the tide's modes); the mantle sits
//!   at the temperature where Arrhenius-viscosity convection carries the
//!   heat made (a bracketed Newton root), plus its lag and formation
//!   transient; the core follows the mantle's base; the dynamo's strength is
//!   the core-boundary flux over the adiabatic flux, less one, clamped to
//!   0..1. [`heat_sources`] and [`kept`] give each isotope's inventory at
//!   the star's birth relative to the Sun's.
//!
//! Orbit quantities are `Q` rounded down to a 2^-32 grid, transcendentals
//! read from certified enclosures; interior quantities are `Dy`. Laboratory
//! rock and iron constants are declared rulers (`Rock`, the `Lab` table).

use crate::elements::{decimal, Element};
use crate::laws::Scales;
use crate::package::Si;
use crate::systems::World;
use xyl_core::certified::{down, exp, pi};
use xyl_core::dyadic::Dy;
use xyl_core::motion::{Boost, Modes};
use xyl_core::{Stream, Q};

const BITS: u32 = 56;
const GRID: u32 = 32;

fn d(s: &str) -> Q {
    decimal(s).unwrap_or(Q::ZERO)
}

fn q(n: i64, den: i64) -> Q {
    Q::new(n, den).unwrap_or(Q::ZERO)
}

fn g(x: Q) -> Q {
    down(&x, GRID)
}

fn ex(x: &Q) -> Q {
    g(exp(x, BITS).mid())
}

/// A body's turning at its own surface (rad/s): the turning law c delta / R,
/// its break-up spin.
fn surface_turning(m_kg: &Q, r_m: &Q) -> Q {
    crate::turning::rad_s(&crate::turning::turning_si(m_kg, r_m))
}

fn pi_q() -> Q {
    pi(BITS).mid()
}

fn unit(rng: &mut Stream) -> Q {
    let x = i64::try_from(rng.next_u64() >> 11).unwrap_or(0);
    q(x, 1 << 53)
}

/// A rocky body's measured constants.
pub struct Rock;
impl Rock {
    /// The unit world's self-gravity pressure rho g R, 3.4e11 Pa: the unit of a body's
    /// own (`self_pressure`).
    pub fn world_pressure() -> Q {
        d("3.4e11")
    }
    /// Shear rigidity of rock, 5.0e10 Pa.
    pub fn mu_rock() -> Q {
        d("5.0e10")
    }
    /// Shear rigidity of ice, 4.0e9 Pa (used for bodies more than a fifth ice).
    pub fn mu_ice() -> Q {
        d("4.0e9")
    }
    /// Tidal dissipation quality factor Q, dimensionless (100).
    pub fn tidal_q() -> Q {
        d("100")
    }
    /// Moment-of-inertia factor I / (M R^2), dimensionless (0.33).
    pub fn inertia() -> Q {
        d("0.33")
    }
}

/// A world's orbital elements and spin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Orbit {
    /// The host's mass, in kg.
    pub host_kg: Q,
    /// Semi-major axis, in m.
    pub a_m: Q,
    /// Eccentricity at birth, in \[0, 0.9\], capped at half the gap to the
    /// nearest neighbour.
    pub e0: Q,
    /// Eccentricity forced by the neighbours (Laplace-Lagrange leading term),
    /// toward which `e0` damps; at most `e0` when `e0` is positive.
    pub e_forced: Q,
    /// Inclination, in radians, in \[0, pi/2\].
    pub inc: Q,
    /// Longitude of the ascending node, as a fraction of a turn in \[0, 1).
    pub node: Q,
    /// Argument of periapsis, as a fraction of a turn in \[0, 1).
    pub peri: Q,
    /// The cosine of its spin axis's tilt.
    pub cos_obliquity: Q,
    /// Spin at birth, in rad/s (a share 0.1 to 0.5 of break-up spin).
    pub spin0: Q,
    /// Age at which the host's tides lock the spin, in Gyr.
    pub lock_gyr: Q,
    /// Tidal damping time of eccentricity, in Gyr.
    pub tau_e: Q,
}

/// What an orbit is at an age.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Now {
    /// Eccentricity at this age, damped from `e0` toward the forced value.
    pub e: Q,
    /// Whether the host's tides have locked the spin (age at or past `lock_gyr`).
    pub locked: bool,
    /// Length of the day, in standard days of 86 400 s.
    pub day: Q,
    /// Length of the year (orbital period), in s.
    pub year_s: Q,
    /// Cosine of the spin axis's tilt, in \[-1, 1\]; 1 once locked.
    pub cos_obliquity: Q,
}

/// A body's self-gravity pressure (Pa): the unit world's rho g R (3.4e11 Pa) times its
/// mass over its radius in world units.
fn self_pressure(mass: &Q, radius: &Q) -> Q {
    (&Rock::world_pressure() * mass)
        .checked_div(radius)
        .unwrap_or(Q::ZERO)
}

/// k2 of a uniform elastic body: 1.5 / (1 + 19 mu / (2 P)), P its
/// self-gravity pressure.
pub fn love(mass: &Q, radius: &Q, icy: bool) -> Q {
    let mu = if icy { Rock::mu_ice() } else { Rock::mu_rock() };
    let den = &Q::ONE
        + &(&Q::int(19) * &mu)
            .checked_div(&(&Q::int(2) * &self_pressure(mass, radius)))
            .unwrap_or(Q::ZERO);
    g(q(3, 2).checked_div(&den).unwrap_or(Q::ZERO))
}

/// The tidal stretch across a body: the change of its host's deficit across
/// it, delta(a - R) - delta(a + R), with delta(x) = M / (x + M) and M the
/// host's mass as a length (G M / c^2).
fn stretch(host_kg: &Q, a_m: &Q, r_m: &Q) -> Q {
    // the host's mass as a length: r_g times its mass over the core's
    let core_kg = &crate::package::hole_mass() * &Si::star_mass_kg();
    let m_len = &crate::chain::units().r_g_m * &host_kg.checked_div(&core_kg).unwrap_or(Q::ZERO);
    let delta = |x: &Q| m_len.checked_div(&(x + &m_len)).unwrap_or(Q::ZERO);
    (&delta(&(a_m - r_m).max(Q::ONE)) - &delta(&(a_m + r_m))).max(Q::ZERO)
}

/// A planet's mean motion about its star (rad/s), from the turning law.
fn mean_motion(host_kg: &Q, a_m: &Q) -> Q {
    crate::turning::rad_s(&crate::turning::turning_si(host_kg, a_m))
}

/// Tidal damping time of eccentricity (Gyr):
/// 4/63 (Q / k2) (m / M) / (n stretch).
pub fn circularise_gyr(host_kg: &Q, a_m: &Q, mass: &Q, radius: &Q, icy: bool) -> Q {
    let (m, r) = (mass * &Si::world_mass_kg(), radius * &Si::world_radius_m());
    let n = mean_motion(host_kg, a_m);
    let k2 = love(mass, radius, icy);
    let t = &(&q(4, 63) * &Rock::tidal_q().checked_div(&k2).unwrap_or(Q::ZERO))
        * &m.checked_div(host_kg).unwrap_or(Q::ZERO);
    let rate = &n * &stretch(host_kg, a_m, &r);
    g(t.checked_div(&rate)
        .unwrap_or(Q::ZERO)
        .checked_div(&Si::gyr_s())
        .unwrap_or(Q::ZERO))
}

/// Time for a host's tides to brake a spin `spin0` (rad/s) to lock (Gyr):
/// (spin0 / n) (I Q / (3 k2 stretch)) / n.
pub fn lock_gyr(host_kg: &Q, a_m: &Q, mass: &Q, radius: &Q, icy: bool, spin0: &Q) -> Q {
    let r = radius * &Si::world_radius_m();
    let k2 = love(mass, radius, icy);
    let n = mean_motion(host_kg, a_m).max(q(1, 1 << 60));
    let rigid = (&Rock::inertia() * &Rock::tidal_q())
        .checked_div(&(&(&Q::int(3) * &k2) * &stretch(host_kg, a_m, &r)))
        .unwrap_or(Q::ZERO);
    let secs = (&spin0.checked_div(&n).unwrap_or(Q::ZERO) * &rigid)
        .checked_div(&n)
        .unwrap_or(Q::ZERO);
    g(secs.checked_div(&Si::gyr_s()).unwrap_or(Q::ZERO))
}

/// The orbital elements and spins of a star's planets (innermost first);
/// `star_kg` is the star's mass.
pub fn orbits(star_kg: &Q, worlds: &[&World], rng: &mut Stream) -> Vec<Orbit> {
    let a_au: Vec<Q> = worlds.iter().map(|w| w.a.clone()).collect();
    let mut out: Vec<Orbit> = Vec::with_capacity(worlds.len());
    for (n, w) in worlds.iter().enumerate() {
        let a_m = &a_au[n] * &Si::au();
        let mut gaps = Vec::new();
        for k in [n.checked_sub(1), Some(n + 1)].into_iter().flatten() {
            if k < worlds.len() {
                let (a1, a2) = if a_au[n] < a_au[k] {
                    (&a_au[n], &a_au[k])
                } else {
                    (&a_au[k], &a_au[n])
                };
                gaps.push((a2 - a1).checked_div(&(a2 + a1)).unwrap_or(Q::ONE));
            }
        }
        let cap = gaps
            .iter()
            .min()
            .cloned()
            .map(|x| &x * &q(1, 2))
            .unwrap_or(d("0.9"));
        // the orbit's shape from the settling (systems::settle_about): its
        // starting eccentricity is the greatest swing its neighbours gave it
        // as a share of its reach, within its gap; its tilt is that swing's
        // deficit angle, swing / (1 + swing) of a quarter turn; its node and
        // periapsis are its settled phase
        let e0 = w.swing.clone().min(cap);
        let inc = &(&pi_q() * &q(1, 2))
            * &w.swing
                .checked_div(&(&Q::ONE + &w.swing))
                .unwrap_or(Q::ZERO);
        let node = w.phase.clone();
        let peri = w.phase.clone();
        let cos_obliquity = &(&Q::int(2) * &unit(rng)) - &Q::ONE;
        let share = &d("0.1") + &(&d("0.4") * &unit(rng));
        let icy = w.layers[2] > q(1, 5);
        let (m, r) = (
            &w.mass * &Si::world_mass_kg(),
            &w.radius * &Si::world_radius_m(),
        );
        let breakup = surface_turning(&m, &r);
        let spin0 = g(&share * &breakup);
        let lock = lock_gyr(star_kg, &a_m, &w.mass, &w.radius, icy, &spin0);
        let tau_e = circularise_gyr(star_kg, &a_m, &w.mass, &w.radius, icy);
        out.push(Orbit {
            host_kg: star_kg.clone(),
            a_m,
            e0,
            e_forced: Q::ZERO,
            inc,
            node,
            peri,
            cos_obliquity,
            spin0,
            lock_gyr: lock,
            tau_e,
        });
    }
    // what each neighbour forces (Laplace-Lagrange, leading term)
    let forced: Vec<Q> = (0..worlds.len())
        .map(|n| {
            let mut f = Q::ZERO;
            for k in [n.checked_sub(1), Some(n + 1)].into_iter().flatten() {
                if k < worlds.len() {
                    let ratio = a_au[n]
                        .clone()
                        .min(a_au[k].clone())
                        .checked_div(&a_au[n].clone().max(a_au[k].clone()))
                        .unwrap_or(Q::ZERO);
                    f = f.max(&(&q(5, 4) * &ratio) * &out[k].e0);
                }
            }
            f
        })
        .collect();
    for (o, f) in out.iter_mut().zip(forced) {
        o.e_forced = if o.e0.signum() > 0 {
            o.e0.clone().min(f)
        } else {
            f
        };
    }
    out
}

impl Orbit {
    /// The orbit and spin at an age (Gyr): eccentricity damped toward the
    /// forced, spin braked toward lock (then the day is the year and the axis
    /// upright).
    pub fn at(&self, t: &Q) -> Now {
        let e = if self.tau_e.signum() <= 0 {
            self.e_forced.clone()
        } else {
            &self.e_forced
                + &(&(&self.e0 - &self.e_forced)
                    * &ex(&-(t.checked_div(&self.tau_e).unwrap_or(Q::ZERO))))
        };
        let n = mean_motion(&self.host_kg, &self.a_m);
        let locked = t >= &self.lock_gyr;
        let (w_now, cos_obliquity) = if locked {
            (n.clone(), Q::ONE)
        } else {
            let frac = t.checked_div(&self.lock_gyr).unwrap_or(Q::ZERO);
            (
                &self.spin0 - &(&(&self.spin0 - &n) * &frac),
                self.cos_obliquity.clone(),
            )
        };
        let two_pi = &Q::int(2) * &pi_q();
        let day_s = two_pi
            .checked_div(&w_now.max(d("1e-12")))
            .unwrap_or(Q::ZERO);
        Now {
            e: g(e),
            locked,
            day: g(day_s.checked_div(&Si::day_s()).unwrap_or(Q::ZERO)),
            year_s: g(two_pi.checked_div(&n).unwrap_or(Q::ZERO)),
            cos_obliquity,
        }
    }
}

/// The isotopes: (name, element, half-life Gyr, heat W/kg of the isotope,
/// its share of the element today).
pub fn isotopes() -> [(&'static str, &'static str, Q, Q, Q); 4] {
    [
        ("U238", "U", d("4.468"), d("9.46e-5"), d("0.9928")),
        ("U235", "U", d("0.704"), d("5.69e-4"), d("0.0072")),
        ("Th232", "Th", d("14.05"), d("2.64e-5"), Q::ONE),
        ("K40", "K", d("1.248"), d("2.92e-5"), d("1.17e-4")),
    ]
}

/// Each isotope's Gamow barrier (ln of half-life x attempt rate): an alpha
/// decay's half-life scales as exp(gamow (alpha - 1)) under other laws;
/// potassium's beta decay has none.
fn gamow(i: usize) -> Option<Dy> {
    [
        Some(Dy::sci(878, -1)),
        Some(Dy::int(86)),
        Some(Dy::sci(889, -1)),
        None,
    ]
    .get(i)
    .copied()
    .flatten()
}

/// Each isotope's half-life (Gyr) under a universe's laws.
pub fn half_life(i: usize, sc: &Scales) -> Dy {
    let t = isotopes()
        .get(i)
        .map(|x| Dy::from_q(&x.2))
        .unwrap_or(Dy::ONE);
    gamow(i).map(|g| t * sc.half_life(g)).unwrap_or(t)
}

/// A world's heat sources: each isotope's mass (kg) at its star's birth.
/// `kept` is, per isotope, the share made and still unspent at the star's
/// birth over the Sun's (see [`kept`]).
pub fn heat_sources(world: &World, elements: &[Element], kept: &[Q; 4]) -> [Q; 4] {
    let mut out: [Q; 4] = Default::default();
    let solar_age = d("4.57");
    for (i, (_, el, half, _, today)) in isotopes().iter().enumerate() {
        let Some(ei) = elements.iter().position(|e| &e.sym == el) else {
            continue;
        };
        // the element in the mantle (its mass share of the world's lithophiles sits there)
        let mass_kg = &world.elements.get(ei).cloned().unwrap_or(Q::ZERO) * &Si::world_mass_kg();
        let tau = half
            .checked_div(&xyl_core::certified::ln2(BITS).mid())
            .unwrap_or(Q::ONE);
        let at_sun_birth = today * &ex(&solar_age.checked_div(&tau).unwrap_or(Q::ZERO));
        out[i] = g(&(&mass_kg * &at_sun_birth) * &kept[i]);
    }
    out
}

/// Of an isotope made at a rate falling as exp(-t / tau_f) over `span` Gyr,
/// the share still present (decay time `tau`).
pub fn made_and_kept(tau: &Q, tau_f: &Q, span: &Q) -> Q {
    if span.signum() <= 0 {
        return Q::ONE;
    }
    let (lf, li) = (
        tau_f.recip().unwrap_or(Q::ONE),
        tau.recip().unwrap_or(Q::ONE),
    );
    let made = (&Q::ONE - &ex(&-(&lf * span)))
        .checked_div(&lf)
        .unwrap_or(Q::ONE);
    let kept = if (&li - &lf).abs() < d("1e-9") {
        span * &ex(&-(&li * span))
    } else {
        (&ex(&-(&lf * span)) - &ex(&-(&li * span)))
            .checked_div(&(&li - &lf))
            .unwrap_or(Q::ZERO)
    };
    g(kept.checked_div(&made.max(d("1e-12"))).unwrap_or(Q::ZERO))
}

/// Per isotope, what a star born at `born` Gyr where its ring formed stars
/// on `tau_f` Gyr (since the first stars at `t1`) kept of what its ring made,
/// over what the Sun's did (its ring's 6.997 Gyr infall, born at 9.2 Gyr after
/// first stars at 0.3 Gyr: the reference the isotope abundances are read against).
/// Decay runs at the universe's own half-lives.
pub fn kept(tau_f: &Q, t1: &Q, born: &Q, sc: &Scales) -> [Q; 4] {
    let ln2 = xyl_core::certified::ln2(BITS).mid();
    let mut out: [Q; 4] = Default::default();
    for (i, (_, _, half, _, _)) in isotopes().iter().enumerate() {
        let tau0 = half.checked_div(&ln2).unwrap_or(Q::ONE);
        let sun = made_and_kept(&tau0, &d("6.997"), &d("8.9"));
        let tau = half_life(i, sc).to_q().checked_div(&ln2).unwrap_or(Q::ONE);
        let here = made_and_kept(&tau, tau_f, &(born - t1));
        out[i] = g(here.checked_div(&sun.max(d("1e-12"))).unwrap_or(Q::ZERO));
    }
    out
}

/// A world's interior at an age.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interior {
    /// Mantle (interior) temperature, in K.
    pub t_int: Dy,
    /// Core temperature, in K.
    pub t_core: Dy,
    /// Total heat flowing out per m^2 of surface, in W/m^2.
    pub q: Dy,
    /// Radiogenic part of the surface heat flow, in W/m^2.
    pub q_rad: Dy,
    /// Secular-cooling part of the surface heat flow, in W/m^2.
    pub q_sec: Dy,
    /// Tidal-heating part of the surface heat flow, in W/m^2.
    pub q_tide: Dy,
    /// Heat the core gives the mantle per m^2 of core surface, in W/m^2; a
    /// dynamo runs while it exceeds `q_ad`.
    pub q_cmb: Dy,
    /// Heat the core's iron conducts along its own adiabat, in W/m^2.
    pub q_ad: Dy,
    /// The core's radius (m).
    pub r_core: Dy,
    /// The dynamo's strength, 0 to 1.
    pub field: Dy,
    /// How many times its heat could have melted its mantle (what it has degassed).
    pub processed: Dy,
    /// The whole heat flowing out (W).
    pub heat_w: Dy,
}

/// The rock and iron constants of the interior model (laboratory), and the
/// SI units, as dyadics.
struct Lab {
    world_mass_kg: Dy,
    world_radius_m: Dy,
    g: Dy,
    gyr_s: Dy,
    cp_rock: Dy,
    k_rock: Dy,
    alpha: Dy,
    eta_ref: Dy,
    t_ref: Dy,
    creep_ea: Dy,
    r_gas: Dy,
    liquidus: Dy,
    rho_rock: Dy,
    bulk: Dy,
    cp_iron: Dy,
    rho_core: Dy,
    iron_liquidus: Dy,
    k_core: Dy,
    alpha_core: Dy,
    l_iron: Dy,
    freeze_span: Dy,
}

impl Lab {
    /// The unit world's surface gravity (m/s^2), the reference for every body's g
    /// (this times its mass over its radius, world units).
    fn unit_world_gravity(&self) -> Dy {
        Dy::sci(981, -2)
    }

    fn get() -> &'static Lab {
        static LAB: std::sync::OnceLock<Lab> = std::sync::OnceLock::new();
        LAB.get_or_init(|| Lab {
            world_mass_kg: Dy::from_q(&Si::world_mass_kg()),
            world_radius_m: Dy::from_q(&Si::world_radius_m()),
            g: Dy::from_q(&Si::g()),
            gyr_s: Dy::from_q(&Si::gyr_s()),
            cp_rock: Dy::int(1200),
            k_rock: Dy::int(4),
            alpha: Dy::sci(3, -5),
            eta_ref: Dy::sci(1, 21),
            t_ref: Dy::int(1600),
            creep_ea: Dy::sci(3, 5),
            r_gas: Dy::sci(8314, -3),
            liquidus: Dy::int(1800),
            rho_rock: Dy::int(4000),
            bulk: Dy::sci(13, 10),
            cp_iron: Dy::int(450),
            rho_core: Dy::sci(11, 3),
            iron_liquidus: Dy::int(4000),
            k_core: Dy::int(40),
            alpha_core: Dy::sci(1, -5),
            l_iron: Dy::sci(75, 4),
            freeze_span: Dy::ratio(37, 100),
        })
    }
}

/// Rock's liquidus temperature, in K, used for magma oceans (worlds.rs).
pub fn liquidus_k() -> Dy {
    Lab::get().liquidus
}
/// Rock's specific heat, in J/(kg K).
pub fn cp_rock() -> Dy {
    Lab::get().cp_rock
}
/// Rock's thermal conductivity (W/m K).
pub fn k_rock() -> Dy {
    Lab::get().k_rock
}
/// Rock's density, in kg/m^3, for the mantle adiabat.
pub fn rho_rock() -> Dy {
    Lab::get().rho_rock
}

fn viscosity(t: Dy, sc: &Scales) -> Dy {
    let l = Lab::get();
    let x =
        l.creep_ea * sc.chem / l.r_gas * (Dy::ONE / t.max(Dy::ONE) - Dy::ONE / (l.t_ref * sc.chem));
    l.eta_ref * x.min(Dy::int(200)).exp()
}

/// The convective boundary layer's thickness (m); declared, the unit world's
/// lithosphere.
fn boundary_m() -> Dy {
    Dy::sci(1, 5)
}

/// A layer's mobility, 1 + eta_ref / eta: near one at the reference
/// viscosity, rising steeply as it warms.
fn mobility(layer: Dy, sc: &Scales) -> Dy {
    Dy::ONE + Lab::get().eta_ref / viscosity(layer, sc).max(Dy::sci(1, -30))
}

/// Heat flux (W/m^2) a convecting layer at `hot` passes to `cold`:
/// k dT (g / g_E) mobility / d.
fn conv_flux(hot: Dy, cold: Dy, grav: Dy, layer: Dy, sc: &Scales) -> Dy {
    let l = Lab::get();
    let dt = hot - cold;
    if dt.signum() <= 0 || grav.signum() <= 0 {
        return Dy::ZERO;
    }
    l.k_rock * dt * (grav / l.unit_world_gravity()) * mobility(layer, sc) / boundary_m()
}

/// A world's interior at `age` Gyr: `sources` its isotopes (kg in its mantle
/// at birth), `t_surf` its surface temperature, `tide` its tidal heating as
/// an orbit (`tide_modes`), under a universe's scales.
///
/// Evaluated in closed form at `age`, without time-stepping. Heat is a sum of
/// boosts (each isotope halving at its own rate, the tide's three modes). The
/// mantle is a thermostat: convective loss rises steeply with temperature
/// (Arrhenius viscosity), so it sits at T_reg(t) where loss equals heat made,
/// lags that curve by C H' / q'^2 while heat falls, and its formation heat
/// relaxes onto it with time constant C / q'. The core follows the mantle's
/// base: its flux carries its cooling at the base's rate (C_core x the base's
/// cooling) plus its formation excess, which decays in its own conductance.
/// What the mantle has passed out (and so melted) is the energy balance: heat
/// made plus what both layers have cooled.
pub fn interior(
    world: &World,
    sources: &[Q; 4],
    age: Dy,
    t_surf: Dy,
    tide: &Modes,
    sc: &Scales,
) -> Interior {
    let l = Lab::get();
    let mass = Dy::from_q(&world.mass);
    let radius = Dy::from_q(&world.radius);
    let m_kg = mass * l.world_mass_kg;
    let r_m = radius * l.world_radius_m;
    let four_pi = Dy::int(4) * Dy::PI;
    // surface area 4 pi R R_E, and gravity g_E m / R (world units)
    let area = four_pi * r_m * l.world_radius_m;
    let grav = l.unit_world_gravity() * mass / radius.max(Dy::sci(1, -6));
    let envelope = Dy::from_q(&world.layers[3]);
    let core_share = Dy::from_q(&world.layers[0]) / (Dy::ONE - envelope).max(Dy::sci(1, -9));
    let m_core = m_kg * core_share.max(Dy::ZERO);
    let mantle = m_kg - m_core;
    let has_core = m_core.signum() > 0;
    // the core's radius from its mass by systems.rs `extent`, its iron denser
    // by rho_core / rho_rock
    let r_core = if has_core {
        let rock_equiv =
            m_core / l.world_mass_kg * l.rho_rock / (l.rho_core * sc.density).max(Dy::sci(1, -9));
        Dy::from_q(&crate::elements::extent(&rock_equiv.to_q())) * l.world_radius_m
    } else {
        Dy::ZERO
    };
    let a_core = four_pi * r_core * l.world_radius_m;
    let g_core = if r_core.signum() > 0 {
        l.unit_world_gravity() * (m_core / l.world_mass_kg) / (r_core / l.world_radius_m)
    } else {
        Dy::ZERO
    };
    let cap_mantle = mantle * l.cp_rock;
    // rock melting point: the liquidus times 1/2 + m / (1 + m) (one for the
    // unit world)
    let t_melt = l.liquidus * sc.chem * (Dy::ratio(1, 2) + mass / (Dy::ONE + mass));
    let t_top = t_melt * Dy::ratio(6, 5);
    // central pressure p = m / R (one for the unit world); iron melting point is the
    // iron liquidus times 2 p / (1 + p)
    let press = (mass / radius.max(Dy::sci(1, -3))).max(Dy::sci(1, -4));
    let t_liq_core = l.iron_liquidus * sc.chem * (Dy::int(2) * press / (Dy::ONE + press));
    let cap_core = m_core * (l.cp_iron + l.l_iron / (l.freeze_span * t_liq_core));
    let depth = (r_m - r_core).max(Dy::ZERO);
    let adiabat =
        (l.alpha / (Dy::ONE + l.rho_rock * grav * depth / l.bulk) * grav * depth / l.cp_rock).exp();
    // the heat it makes (W): each isotope's decay energy (scaled by the
    // nuclear force) halving at the universe's own half-life, and the tide
    let radiogenic = Modes::of(
        isotopes()
            .iter()
            .enumerate()
            .zip(sources.iter())
            .map(|((i, (_, _, half, w, _)), kg)| {
                let th = half_life(i, sc);
                let amp = Dy::from_q(kg) * Dy::from_q(w) * (Dy::from_q(half) / th) * sc.nuclear;
                Boost::halving(
                    amp.to_q(),
                    -Q::ONE.checked_div(&th.to_q()).unwrap_or(Q::ZERO),
                )
            })
            .collect(),
    );
    let heat = radiogenic.plus(tide);
    let dy = |e: xyl_core::certified::Encl| Dy::from_q(&e.mid());
    // the heat read once at the world's age, and at its birth
    let now = heat.at(&age.max(Dy::ZERO).to_q());
    let made_now = dy(now.value());
    let made_birth = dy(heat.value(&Q::ZERO));
    // what convection carries out at a mantle temperature (W)
    let q_out = |t_int: Dy| conv_flux(t_int, t_surf, grav, t_int, sc) * area;
    // dq/dT at T: k (g / g_E) / d (area) [mobility
    // + dT (mobility - 1) (E_a / R / T) / T]
    let dq = |t_int: Dy| -> Dy {
        let ea = l.creep_ea * sc.chem / l.r_gas;
        let mob = mobility(t_int, sc);
        let dt = (t_int - t_surf).max(Dy::ZERO);
        l.k_rock * (grav / l.unit_world_gravity()) / boundary_m()
            * area
            * (mob + dt * (mob - Dy::ONE) * (ea / t_int.max(Dy::ONE)) / t_int.max(Dy::ONE))
    };
    // the thermostat: solve q_out(T) = H for T in [t_surf, t_top]. q_out is
    // monotone, so the root is unique; safeguarded Newton from the hot end
    // (bisection when a step leaves the bracket), to a millikelvin
    let regulated = |h: Dy| -> Dy {
        if q_out(t_top) <= h {
            return t_top;
        }
        let (mut lo, mut hi, mut t) = (t_surf, t_top, t_top);
        for _ in 0..60 {
            let miss = q_out(t) - h;
            if miss.signum() > 0 {
                hi = t;
            } else {
                lo = t;
            }
            let newton = t - miss / dq(t).max(Dy::sci(1, -30));
            let next = if newton > lo && newton < hi {
                newton
            } else {
                (lo + hi) * Dy::ratio(1, 2)
            };
            if (next - t).abs() < Dy::sci(1, -3) || hi - lo < Dy::sci(1, -3) {
                return next;
            }
            t = next;
        }
        t
    };
    let gyr = l.gyr_s;
    let t0 = t_melt;
    let reg0 = regulated(made_birth);
    let reg = regulated(made_now);
    // H' (W/s), for the lag C H' / q'^2 (C in J/K)
    let slope = dy(now.slope()) / gyr;
    // the lag behind a falling thermostat: time constant C / q' times the
    // thermostat's fall H' / q', capped at the initial excess t_melt - T_reg
    let q_slope = dq(reg).max(Dy::sci(1, -30));
    let lag = ((cap_mantle / q_slope) * (-slope / q_slope))
        .max(Dy::ZERO)
        .min((t_melt - reg).max(Dy::ZERO));
    let tau0 = cap_mantle / dq(reg0).max(Dy::sci(1, -30)) / gyr;
    let transient = (t0 - reg0) * (-(age.max(Dy::ZERO) / tau0.max(Dy::sci(1, -12)))).exp();
    let t_int = (reg + lag + transient).min(t_top).max(t_surf);
    // the core: its temperature excess Delta over the mantle's base
    let base0 = t0 * adiabat;
    let base = t_int * adiabat;
    let delta0 = (t_liq_core - base0).max(Dy::ZERO);
    // the mantle's cooling now (K/s): along its thermostat, H' / q', and its
    // formation heat's relaxation
    let cooling = (-(slope / dq(reg).max(Dy::sci(1, -30)))
        + transient / (tau0 * gyr).max(Dy::sci(1, -30)))
    .max(Dy::ZERO);
    let (t_core, q_cmb) = if has_core && delta0.signum() > 0 {
        let excess = |layer: Dy| -> Dy {
            // linear boundary conductance: flux = k_b Delta (W). The formation
            // excess relaxes as Delta0 e^(-k_b t / C); following the base's
            // cooling adds C adiabat cooling / k_b
            let k_b = a_core * l.k_rock * (g_core / l.unit_world_gravity()) * mobility(layer, sc)
                / boundary_m();
            let c = k_b * gyr / cap_core.max(Dy::ONE);
            let formation = delta0 * (-(c * age.max(Dy::ZERO))).exp();
            let following = cap_core * adiabat * cooling / k_b.max(Dy::sci(1, -30));
            formation + following
        };
        // boundary viscosity at the layer's temperature: evaluated at the
        // base, then at the layer's midpoint
        let first = excess(base);
        let delta = excess(base + first * Dy::ratio(1, 2));
        let t_core = base + delta;
        (
            t_core,
            conv_flux(t_core, base, g_core, base + delta * Dy::ratio(1, 2), sc) * a_core,
        )
    } else {
        (base.max(t_liq_core.min(base)), Dy::ZERO)
    };
    // what has passed out of the mantle: the heat made, plus what the mantle
    // and the core have cooled since their birth
    let passed = dy(now.integral()) * gyr
        + cap_mantle * (t0 - t_int)
        + cap_core * (t_liq_core - t_core).max(Dy::ZERO);
    let span = (t_melt - t_surf).max(Dy::ONE);
    let processed = passed.max(Dy::ZERO) / (mantle * l.cp_rock * span).max(Dy::ONE);
    let h_tide = dy(tide.value(&age.max(Dy::ZERO).to_q()));
    let h_rad = made_now - h_tide;
    let q_total = if age.signum() > 0 {
        q_out(t_int)
    } else {
        Dy::ZERO
    };
    // what flows out (a world heated faster than it convects melts)
    let heat_w = q_total.max(h_rad + h_tide);
    let q_ad = l.k_core * l.alpha_core * g_core * t_core / l.cp_iron;
    let q_cmb_area = if a_core.signum() > 0 {
        q_cmb / a_core
    } else {
        Dy::ZERO
    };
    let field = if !has_core || q_ad.signum() <= 0 {
        Dy::ZERO
    } else {
        (q_cmb_area / q_ad - Dy::ONE).clamp(Dy::ZERO, Dy::ONE)
    };
    Interior {
        t_int,
        t_core,
        q: heat_w / area,
        q_rad: h_rad / area,
        q_sec: (heat_w - h_rad - h_tide).max(Dy::ZERO) / area,
        q_tide: h_tide / area,
        q_cmb: q_cmb_area,
        q_ad,
        r_core,
        field,
        processed,
        heat_w,
    }
}

/// k2 of a uniform elastic body, as a dyadic (see [`love`]).
fn love_dy(mass: Dy, radius: Dy, icy: bool) -> Dy {
    let p_self = Dy::from_q(&Rock::world_pressure()) * mass / radius.max(Dy::sci(1, -6));
    let mu = Dy::from_q(&if icy { Rock::mu_ice() } else { Rock::mu_rock() });
    Dy::ratio(3, 2) / (Dy::ONE + Dy::int(19) * mu / (Dy::int(2) * p_self))
}

/// Tidal heating of a synchronously rotating world on an eccentric orbit (W):
/// 10.5 (k2 / Q) (G M m / a) n e stretch. `host_kg` and `a_m` in SI, `mass`
/// and `radius` in world units.
pub fn tidal_w(host_kg: Dy, a_m: Dy, e: Dy, mass: Dy, radius: Dy, icy: bool) -> Dy {
    if e.signum() <= 0 || a_m.signum() <= 0 {
        return Dy::ZERO;
    }
    let l = Lab::get();
    let r = radius * l.world_radius_m;
    // mean motion from the turning law (rad/s)
    let n = Dy::from_q(&crate::turning::rad_s(&crate::turning::turning_si(
        &host_kg.to_q(),
        &a_m.to_q(),
    )));
    let k2 = love_dy(mass, radius, icy);
    let st = Dy::from_q(&stretch(&host_kg.to_q(), &a_m.to_q(), &r.to_q()));
    Dy::ratio(21, 2) * k2 / Dy::int(100)
        * (l.g * host_kg * mass * l.world_mass_kg / a_m)
        * n
        * e
        * st
}

/// An orbit's eccentricity at `t` Gyr: damped from `e0` toward the forced.
pub fn eccentricity(e0: Dy, e_forced: Dy, tau_e: Dy, t: Dy) -> Dy {
    if tau_e.signum() <= 0 {
        return e_forced;
    }
    e_forced + (e0 - e_forced) * (-(t.max(Dy::ZERO) / tau_e)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rocky_world() -> World {
        let elements = crate::elements::table().unwrap();
        let mut el = vec![Q::ZERO; elements.len()];
        // a mantle of two thirds with some radiogenic kinds in it
        let mantle = q(2, 3);
        let set = |el: &mut Vec<Q>, s: &str, share: &str| {
            let i = elements.iter().position(|e| e.sym == s).unwrap();
            el[i] = &mantle * &d(share);
        };
        set(&mut el, "U", "1e-8");
        set(&mut el, "Th", "1e-7");
        set(&mut el, "K", "1e-4");
        World {
            a: Q::ONE,
            mass: Q::ONE,
            radius: Q::ONE,
            giant: false,
            host: None,
            a_host_radii: None,
            layers: [q(1, 3), q(2, 3), Q::ZERO, Q::ZERO],
            elements: el,
            mantle: Vec::new(),
            ices: Vec::new(),
            e_forced: None,
            swing: Q::ZERO,
            phase: Q::ZERO,
            merged_at: None,
        }
    }

    #[test]
    fn a_close_world_locks_and_a_far_one_spins() {
        let w = rocky_world();
        let sun = Si::star_mass_kg();
        let mut rng = Stream::derive(1, &["orbits".into()]);
        let close = World {
            a: d("0.3"),
            ..w.clone()
        };
        let far = World {
            a: Q::ONE,
            ..w.clone()
        };
        let os = orbits(&sun, &[&close, &far], &mut rng);
        assert!(
            os[0].lock_gyr < os[1].lock_gyr,
            "the closer world locks sooner"
        );
        assert!(
            os[0].e0 <= d("0.9") && os[0].cos_obliquity >= -Q::ONE && os[0].cos_obliquity <= Q::ONE
        );
        let now = os[1].at(&d("3"));
        assert!(now.day > Q::ZERO, "a world that turns has a day");
        // the year is one turn at the turning law's rate, and a farther
        // world's is longer
        let law = (&Q::int(360) * &Si::gyr_s())
            .checked_div(&crate::turning::turning_si(&sun, &os[1].a_m))
            .unwrap();
        assert!(
            (&now.year_s - &law).abs() <= &law * &d("1e-9"),
            "year {} against the law's {}",
            now.year_s,
            law
        );
        assert!(
            os[0].at(&d("3")).year_s < now.year_s,
            "the closer world's year is shorter"
        );
    }
}
