//! A world's interior in closed form: mantle thermostat, core and dynamo,
//! and tidal heating.

use super::*;

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
pub(super) struct Lab {
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

pub(super) fn viscosity(t: Dy, sc: &Scales) -> Dy {
    let l = Lab::get();
    let x =
        l.creep_ea * sc.chem / l.r_gas * (Dy::ONE / t.max(Dy::ONE) - Dy::ONE / (l.t_ref * sc.chem));
    l.eta_ref * x.min(Dy::int(200)).exp()
}

/// The convective boundary layer's thickness (m); declared, the unit world's
/// lithosphere.
pub(super) fn boundary_m() -> Dy {
    Dy::sci(1, 5)
}

/// A layer's mobility, 1 + eta_ref / eta: near one at the reference
/// viscosity, rising steeply as it warms.
pub(super) fn mobility(layer: Dy, sc: &Scales) -> Dy {
    Dy::ONE + Lab::get().eta_ref / viscosity(layer, sc).max(Dy::sci(1, -30))
}

/// Heat flux (W/m^2) a convecting layer at `hot` passes to `cold`:
/// k dT (g / g_E) mobility / d.
pub(super) fn conv_flux(hot: Dy, cold: Dy, grav: Dy, layer: Dy, sc: &Scales) -> Dy {
    let l = Lab::get();
    let dt = hot - cold;
    if dt.signum() <= 0 || grav.signum() <= 0 {
        return Dy::ZERO;
    }
    l.k_rock * dt * (grav / l.unit_world_gravity()) * mobility(layer, sc) / boundary_m()
}

/// The mantle's convective loss and its thermostat for one world.
struct Thermostat<'s> {
    /// Surface temperature (K).
    t_surf: Dy,
    /// The hottest mantle allowed (K): 6/5 of rock's melting point.
    t_top: Dy,
    /// Surface gravity (m/s^2).
    grav: Dy,
    /// Surface area (m^2).
    area: Dy,
    sc: &'s Scales,
}

impl Thermostat<'_> {
    /// What convection carries out at mantle temperature `t_int` (W).
    fn q_out(&self, t_int: Dy) -> Dy {
        conv_flux(t_int, self.t_surf, self.grav, t_int, self.sc) * self.area
    }

    /// dq/dT at T: k (g / g_E) / d (area) [mobility
    /// + dT (mobility - 1) (E_a / R / T) / T].
    fn dq(&self, t_int: Dy) -> Dy {
        let l = Lab::get();
        let ea = l.creep_ea * self.sc.chem / l.r_gas;
        let mob = mobility(t_int, self.sc);
        let dt = (t_int - self.t_surf).max(Dy::ZERO);
        l.k_rock * (self.grav / l.unit_world_gravity()) / boundary_m()
            * self.area
            * (mob + dt * (mob - Dy::ONE) * (ea / t_int.max(Dy::ONE)) / t_int.max(Dy::ONE))
    }

    /// The T in [t_surf, t_top] with q_out(T) = `h`. q_out is monotone, so
    /// the root is unique; safeguarded Newton from the hot end (bisection
    /// when a step leaves the bracket), to a millikelvin.
    fn regulated(&self, h: Dy) -> Dy {
        if self.q_out(self.t_top) <= h {
            return self.t_top;
        }
        let (mut lo, mut hi, mut t) = (self.t_surf, self.t_top, self.t_top);
        for _ in 0..60 {
            let miss = self.q_out(t) - h;
            if miss.signum() > 0 {
                hi = t;
            } else {
                lo = t;
            }
            let newton = t - miss / self.dq(t).max(Dy::sci(1, -30));
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
    }
}

/// The heat a world's isotopes make (W) as boosts: each isotope's decay
/// energy, scaled by the nuclear scale, halving at the universe's own
/// half-life.
fn radiogenic(sources: &[Q; 4], sc: &Scales) -> Modes {
    Modes::of(
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
    )
}

/// The core's radius (m) from its mass `m_core` (kg) by systems.rs
/// `extent`, its iron denser than rock by rho_core / rho_rock.
fn core_radius(m_core: Dy, sc: &Scales) -> Dy {
    let l = Lab::get();
    let rock_equiv =
        m_core / l.world_mass_kg * l.rho_rock / (l.rho_core * sc.density).max(Dy::sci(1, -9));
    Dy::from_q(&crate::elements::extent(&rock_equiv.to_q())) * l.world_radius_m
}

/// The core above the mantle's base temperature `base` (K): its temperature
/// (K) and the heat it passes the mantle (W). A linear boundary conductance
/// k_b carries flux k_b Delta; the formation excess `delta0` relaxes as
/// Delta0 e^(-k_b t / C), and following the base's cooling `cooling` (K/s)
/// adds C adiabat cooling / k_b. The boundary viscosity is read at the base,
/// then at the layer's midpoint.
fn core_heat(
    base: Dy,
    delta0: Dy,
    a_core: Dy,
    g_core: Dy,
    cap_core: Dy,
    adiabat: Dy,
    cooling: Dy,
    age: Dy,
    sc: &Scales,
) -> (Dy, Dy) {
    let l = Lab::get();
    let gyr = l.gyr_s;
    let excess = |layer: Dy| -> Dy {
        let k_b = a_core * l.k_rock * (g_core / l.unit_world_gravity()) * mobility(layer, sc)
            / boundary_m();
        let c = k_b * gyr / cap_core.max(Dy::ONE);
        let formation = delta0 * (-(c * age.max(Dy::ZERO))).exp();
        let following = cap_core * adiabat * cooling / k_b.max(Dy::sci(1, -30));
        formation + following
    };
    let first = excess(base);
    let delta = excess(base + first * Dy::ratio(1, 2));
    let t_core = base + delta;
    (
        t_core,
        conv_flux(t_core, base, g_core, base + delta * Dy::ratio(1, 2), sc) * a_core,
    )
}

/// The mantle's temperature at `age` (K) and its cooling rate now (K/s),
/// from the heat made at birth and now (W) and its slope H' now (W/s):
/// T_reg(H) plus the lag behind a falling thermostat, time constant C / q'
/// times its fall H' / q' (capped at the initial excess t_melt - T_reg),
/// plus the formation transient (t_melt - T_reg0) e^(-t / tau0) with
/// tau0 = C / q'(T_reg0). The cooling is the thermostat's fall H' / q' and
/// the transient's relaxation.
fn mantle_temperature(
    thermostat: &Thermostat,
    made_birth: Dy,
    made_now: Dy,
    slope: Dy,
    cap_mantle: Dy,
    t_melt: Dy,
    age: Dy,
) -> (Dy, Dy) {
    let gyr = Lab::get().gyr_s;
    let t0 = t_melt;
    let reg0 = thermostat.regulated(made_birth);
    let reg = thermostat.regulated(made_now);
    let q_slope = thermostat.dq(reg).max(Dy::sci(1, -30));
    let lag = ((cap_mantle / q_slope) * (-slope / q_slope))
        .max(Dy::ZERO)
        .min((t_melt - reg).max(Dy::ZERO));
    let tau0 = cap_mantle / thermostat.dq(reg0).max(Dy::sci(1, -30)) / gyr;
    let transient = (t0 - reg0) * (-(age.max(Dy::ZERO) / tau0.max(Dy::sci(1, -12)))).exp();
    let t_int = (reg + lag + transient)
        .min(thermostat.t_top)
        .max(thermostat.t_surf);
    let cooling = (-(slope / thermostat.dq(reg).max(Dy::sci(1, -30)))
        + transient / (tau0 * gyr).max(Dy::sci(1, -30)))
    .max(Dy::ZERO);
    (t_int, cooling)
}

/// A world's sizes for its interior, SI.
struct Shape {
    /// Mass and radius in world units.
    mass: Dy,
    radius: Dy,
    /// Radius (m).
    r_m: Dy,
    /// Surface area 4 pi R R_E (m^2), linear in radius.
    area: Dy,
    /// Surface gravity g_E m / R (m/s^2).
    grav: Dy,
    /// Core and mantle masses (kg).
    m_core: Dy,
    mantle: Dy,
    has_core: bool,
    /// Core radius (m), its area (m^2) and its surface gravity (m/s^2).
    r_core: Dy,
    a_core: Dy,
    g_core: Dy,
}

/// A world's [`Shape`]: the core is the core layer's share of the non-envelope
/// mass.
fn shape(world: &World, sc: &Scales) -> Shape {
    let l = Lab::get();
    let mass = Dy::from_q(&world.mass);
    let radius = Dy::from_q(&world.radius);
    let m_kg = mass * l.world_mass_kg;
    let r_m = radius * l.world_radius_m;
    let four_pi = Dy::int(4) * Dy::PI;
    let area = four_pi * r_m * l.world_radius_m;
    let grav = l.unit_world_gravity() * mass / radius.max(Dy::sci(1, -6));
    let envelope = Dy::from_q(&world.layers[3]);
    let core_share = Dy::from_q(&world.layers[0]) / (Dy::ONE - envelope).max(Dy::sci(1, -9));
    let m_core = m_kg * core_share.max(Dy::ZERO);
    let mantle = m_kg - m_core;
    let has_core = m_core.signum() > 0;
    let r_core = if has_core {
        core_radius(m_core, sc)
    } else {
        Dy::ZERO
    };
    let a_core = four_pi * r_core * l.world_radius_m;
    let g_core = if r_core.signum() > 0 {
        l.unit_world_gravity() * (m_core / l.world_mass_kg) / (r_core / l.world_radius_m)
    } else {
        Dy::ZERO
    };
    Shape {
        mass,
        radius,
        r_m,
        area,
        grav,
        m_core,
        mantle,
        has_core,
        r_core,
        a_core,
        g_core,
    }
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
    let Shape {
        mass,
        radius,
        r_m,
        area,
        grav,
        m_core,
        mantle,
        has_core,
        r_core,
        a_core,
        g_core,
    } = shape(world, sc);
    let cap_mantle = mantle * l.cp_rock;
    // rock melting point: the liquidus times 1/2 + m / (1 + m) (one for the
    // unit world)
    let t_melt = l.liquidus * sc.chem * (Dy::ratio(1, 2) + mass / (Dy::ONE + mass));
    let thermostat = Thermostat {
        t_surf,
        t_top: t_melt * Dy::ratio(6, 5),
        grav,
        area,
        sc,
    };
    // central pressure p = m / R (one for the unit world); iron melting
    // point is the iron liquidus times 2 p / (1 + p)
    let press = (mass / radius.max(Dy::sci(1, -3))).max(Dy::sci(1, -4));
    let t_liq_core = l.iron_liquidus * sc.chem * (Dy::int(2) * press / (Dy::ONE + press));
    let cap_core = m_core * (l.cp_iron + l.l_iron / (l.freeze_span * t_liq_core));
    let depth = (r_m - r_core).max(Dy::ZERO);
    let adiabat =
        (l.alpha / (Dy::ONE + l.rho_rock * grav * depth / l.bulk) * grav * depth / l.cp_rock).exp();
    let heat = radiogenic(sources, sc).plus(tide);
    let dy = |e: xyl_core::certified::Encl| Dy::from_q(&e.mid());
    // the heat read once at the world's age, and at its birth
    let now = heat.at(&age.max(Dy::ZERO).to_q());
    let made_now = dy(now.value());
    let made_birth = dy(heat.value(&Q::ZERO));
    let gyr = l.gyr_s;
    let t0 = t_melt;
    let (t_int, cooling) = mantle_temperature(
        &thermostat,
        made_birth,
        made_now,
        dy(now.slope()) / gyr,
        cap_mantle,
        t_melt,
        age,
    );
    // the core: its temperature excess Delta over the mantle's base
    let base0 = t0 * adiabat;
    let base = t_int * adiabat;
    let delta0 = (t_liq_core - base0).max(Dy::ZERO);
    let (t_core, q_cmb) = if has_core && delta0.signum() > 0 {
        core_heat(
            base, delta0, a_core, g_core, cap_core, adiabat, cooling, age, sc,
        )
    } else {
        (base.max(t_liq_core.min(base)), Dy::ZERO)
    };
    // energy balance: heat made plus the mantle's and core's cooling since
    // birth is what has passed out of the mantle
    let passed = dy(now.integral()) * gyr
        + cap_mantle * (t0 - t_int)
        + cap_core * (t_liq_core - t_core).max(Dy::ZERO);
    let span = (t_melt - t_surf).max(Dy::ONE);
    let processed = passed.max(Dy::ZERO) / (mantle * l.cp_rock * span).max(Dy::ONE);
    let h_tide = dy(tide.value(&age.max(Dy::ZERO).to_q()));
    let h_rad = made_now - h_tide;
    let q_total = if age.signum() > 0 {
        thermostat.q_out(t_int)
    } else {
        Dy::ZERO
    };
    // outflow is at least the heat made: a world heated faster than it
    // convects melts
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
pub(super) fn love_dy(mass: Dy, radius: Dy, icy: bool) -> Dy {
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
