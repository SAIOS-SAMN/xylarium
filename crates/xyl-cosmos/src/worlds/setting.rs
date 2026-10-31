//! A world's setting (orbit path, heartbeat) and its state at an age (`build`).

use super::*;

/// What a world circles and how: its host (kg), its orbit (m), its
/// eccentricity's history, and its spin axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Path {
    /// The host's mass (kg): the star for a planet, the giant for a moon.
    pub host_kg: Dy,
    /// Semi-major axis about the host (m).
    pub a_m: Dy,
    /// Eccentricity at the star's birth, 0 to 1.
    pub e0: Dy,
    /// Eccentricity forced by the neighbours, the value `e0` relaxes toward.
    pub e_forced: Dy,
    /// The eccentricity's damping time (Gyr); zero or less means it sits at `e_forced`.
    pub tau_e: Dy,
    /// The cosine of the spin axis's tilt before lock; once locked, the axis
    /// is upright (cosine 1).
    pub cos_obliquity: Dy,
    /// The age at which the host's tides lock the spin (Gyr).
    pub lock_gyr: Dy,
    /// Its spin at birth (rad/s); zero for a moon locked from the start.
    pub spin0: Dy,
}

/// A world's day and year at an age, from its spin and orbit (bodies.rs
/// `Orbit::at`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Heartbeat {
    /// Its day (s); a locked world's day is its year.
    pub day_s: Dy,
    /// Its year (s): one orbit at the turning law's mean motion.
    pub year_s: Dy,
    /// Whether its spin is locked to its orbit at this age.
    pub locked: bool,
}

impl Path {
    /// The path of a drawn orbit (bodies.rs [`Orbit`]), converted to dyadics.
    pub fn of(o: &Orbit) -> Path {
        Path {
            host_kg: Dy::from_q(&o.host_kg),
            a_m: Dy::from_q(&o.a_m),
            e0: Dy::from_q(&o.e0),
            e_forced: Dy::from_q(&o.e_forced),
            tau_e: Dy::from_q(&o.tau_e),
            cos_obliquity: Dy::from_q(&o.cos_obliquity),
            lock_gyr: Dy::from_q(&o.lock_gyr),
            spin0: Dy::from_q(&o.spin0),
        }
    }

    /// Its day and year at age `t` (Gyr): the turning law sets the year; the
    /// spin brakes linearly from the birth spin toward the orbital rate until
    /// lock (bodies.rs `Orbit::at`). A path with no birth spin is locked.
    pub fn heartbeat(&self, t: Dy) -> Heartbeat {
        let mean = Dy::from_q(&crate::turning::rad_s(&crate::turning::turning_si(
            &self.host_kg.to_q(),
            &self.a_m.to_q(),
        )));
        let locked = t >= self.lock_gyr || self.spin0.signum() <= 0;
        let w = if locked {
            mean
        } else {
            let frac = t / self.lock_gyr.max(Dy::sci(1, -12));
            self.spin0 - (self.spin0 - mean) * frac
        };
        let two_pi = Dy::int(2) * Dy::PI;
        Heartbeat {
            day_s: two_pi / w.max(Dy::sci(1, -12)),
            year_s: two_pi / mean.max(Dy::sci(1, -30)),
            locked,
        }
    }

    /// Its eccentricity at age `t` (Gyr), relaxed from `e0` toward `e_forced`
    /// with time constant `tau_e`.
    pub fn e(&self, t: Dy) -> Dy {
        bodies::eccentricity(self.e0, self.e_forced, self.tau_e, t)
    }

    /// The tide's heating (W) of a world of `mass` and `radius` on this path,
    /// as modes in age (Gyr). The coefficient is `bodies::tidal_w` at e = 1,
    /// taken to multiply e^2; the eccentricity relaxes from `e0` toward
    /// `e_forced` exponentially, so the heating is three modes (the forced
    /// part, the cross term and the free part) at rates 0, -1/tau_e and
    /// -2/tau_e. With `tau_e <= 0` only the forced part remains.
    pub fn tide(&self, mass: Dy, radius: Dy, icy: bool) -> Modes {
        let per_e2 = bodies::tidal_w(self.host_kg, self.a_m, Dy::ONE, mass, radius, icy).to_q();
        if per_e2.is_zero() {
            return Modes::default();
        }
        let (ef, free) = (self.e_forced.to_q(), (self.e0 - self.e_forced).to_q());
        if self.tau_e.signum() <= 0 {
            return Modes::of(vec![Boost::e(&per_e2 * &(&ef * &ef), Q::ZERO)]);
        }
        let k = Q::ONE.checked_div(&self.tau_e.to_q()).unwrap_or(Q::ZERO);
        Modes::of(vec![
            Boost::e(&per_e2 * &(&ef * &ef), Q::ZERO),
            Boost::e(&(&per_e2 * &Q::int(2)) * &(&ef * &free), -k.clone()),
            Boost::e(&per_e2 * &(&free * &free), -(&k * &Q::int(2))),
        ])
    }

    /// The cosine of the spin axis's tilt at age `t` (Gyr): the birth value,
    /// or 1 (upright) at or after `lock_gyr`.
    pub fn cos_obliquity_at(&self, t: Dy) -> Dy {
        if t >= self.lock_gyr {
            Dy::ONE
        } else {
            self.cos_obliquity
        }
    }
}

/// Everything a world's physics is read from.
#[derive(Clone, Debug)]
pub struct Setting<'a> {
    /// The world being read.
    pub world: &'a World,
    /// Its star's mass (solar masses).
    pub star_mass: Dy,
    /// Its star's luminosity (solar) at the age read; the caller supplies the
    /// raised luminosity of a star that has left the main sequence.
    pub lum: Dy,
    /// Its distance from its star (AU): a moon's is its giant's.
    pub a_au: Dy,
    /// Its birth gas's metals (solar = 1).
    pub z: Dy,
    /// Its star's distance from the galaxy's centre (kpc).
    pub r_kpc: Dy,
    /// Its isotopes (kg in the mantle at the star's birth, from
    /// `bodies::heat_sources`).
    pub sources: [Q; 4],
    /// Its orbit and spin; `None` when not drawn (the world then reads as
    /// circular, untided and at a 45 degree tilt).
    pub path: Option<Path>,
}

/// A world as physics makes it at an age.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Age (Gyr) since its star's birth.
    pub age: Dy,
    /// Whether it is a giant: an envelope with no surface, whose interior,
    /// atmosphere and escape are not solved.
    pub giant: bool,
    /// Mass (world masses).
    pub mass: Dy,
    /// Radius (world radii).
    pub radius: Dy,
    /// The year's mean light at the world's distance, `L / a` (flux units of
    /// 340 W/m^2), with L the star's mean luminosity over the bell.
    pub flux: Dy,
    /// Temperature (K) of the light where it formed: the unit temperature times
    /// luminosity over distance.
    pub t_form: Dy,
    /// Surface temperature (K): the climate's warmest stable balance.
    pub t_surf: Dy,
    /// Surface pressure (bar): the sum of the air's partial pressures.
    pub p_bar: Dy,
    /// The air: each gas's partial pressure (bar).
    pub air: Mix,
    /// The seas and surface ices: each condensed volatile, as the partial
    /// pressure (bar) it would add if it were all vapour.
    pub seas: Mix,
    /// The share of each sea's surface held frozen where the world sits at
    /// that sea's melting point (ice and water together, 0 to 1).
    pub frozen: Mix,
    /// The volatiles it gathered when it formed, each per unit of its rock mass.
    pub ices: Mix,
    /// Its whole outflowing heat in units of a unit world's (0.087 W/m^2 over
    /// a unit-world-radius sphere).
    pub heat: Dy,
    /// Heat flowing out per m^2 of surface (W/m^2), all sources.
    pub q: Dy,
    /// The tidal part of `q` (W/m^2).
    pub q_tide: Dy,
    /// Mantle temperature (K).
    pub t_int: Dy,
    /// Heat the core gives the mantle per m^2 of the core's surface (W/m^2).
    pub q_cmb: Dy,
    /// Heat the core's iron conducts along the core's adiabat (W/m^2); a
    /// dynamo runs while `q_cmb` exceeds it.
    pub q_ad: Dy,
    /// The core's radius (m).
    pub r_core: Dy,
    /// The dynamo's strength, 0 to 1.
    pub field: Dy,
    /// Share of the mantle its accretion melted, 0 to 1 (the magma ocean).
    pub melted: Dy,
    /// The heat passed out of the mantle over the heat that raises the mantle
    /// to its melting point (`bodies::Interior::processed`).
    pub processed: Dy,
    /// Share of its gathered volatiles released to the surface, 0 to 1.
    pub degassed: Dy,
    /// The stellar wind's stripping budget over the world's life, as a mass
    /// fraction of the world (the unit of the gathered volatiles):
    /// `strip_rate` x flux (solar units) x activity dose (Gyr x solar
    /// activity) / world mass (world masses).
    pub wind: Dy,
    /// Exosphere temperature (K): hotter under an active star, and by half again
    /// with no magnetic field.
    pub t_exo: Dy,
    /// Share of its released volatiles (those Jeans escape left it) that the
    /// stellar wind then took, 0 to 1.
    pub escaped: Dy,
    /// Share of its standing air the wind took, 0 to 1.
    pub air_escaped: Dy,
    /// The stellar crowding of its galactic neighbourhood relative to the
    /// reference ring: `galaxy::density` at the star's distance from the
    /// galaxy's centre.
    pub env: Dy,
    /// Its star's surface temperature (K).
    pub star_t: Dy,
    /// Bond albedo, 0 to 1: 0.3 bare, rising with surface ices.
    pub albedo: Dy,
    /// Orbital eccentricity at its age.
    pub e: Dy,
    /// Surface gravity (world gravities): mass over radius squared.
    pub g: Dy,
    /// Cosmic-ray flux where its system circles, relative to the disc's at R0
    /// (see [`cosmic_rays`]).
    pub cr: Dy,
    /// Stellar-wind exposure: its star's activity relative to the Sun's at
    /// 4.57 Gyr, over its distance (AU).
    pub sw: Dy,
    /// Light reaching its surface (flux units).
    pub light: Dy,
    /// The universe's scales it was built under.
    pub sc: Scales,
}

/// A star's surface temperature (K), linear in luminosity over radius:
/// `5772 L / R`, with R linear in mass (at least 0.08 of the reference mass)
/// in the universe's stellar scales (not M^0.8, not the fourth root of
/// L / R^2).
pub fn star_t(star_mass: Dy, lum: Dy, sc: &Scales) -> Dy {
    let radius = (star_mass / sc.star_mass).max(Dy::ratio(2, 25)) * sc.star_radius;
    Dy::int(5772) * lum.max(Dy::sci(1, -9)) / radius
}

/// The climate's solved state: surface temperature (K), pressure (bar), air,
/// seas, albedo and frozen shares (`Climate::solve`).
type Solved = (Dy, Dy, Mix, Mix, Dy, Mix);

/// The world as physics makes it at `age` Gyr after its star's birth: its
/// interior, atmosphere and oceans, temperature and field.
pub fn build(set: &Setting, age: Dy, ctx: &Ctx) -> State {
    let blank = radiated(set, age, ctx);
    if set.world.giant {
        // A giant's state is its cloud tops from radiation alone; no surface,
        // interior or escape is solved.
        return blank;
    }
    let c = constants();
    let sc = &ctx.sc;
    let w = set.world;
    let (mass, radius, flux, ices) = (blank.mass, blank.radius, blank.flux, blank.ices);
    let t_eq0 = blank.t_surf;
    let star_rel = set.star_mass / sc.star_mass;
    // Interior heat sources: isotopes, formation heat and tides from the
    // host.
    let ice_sum = ices.total();
    let ice_share = ice_sum / (Dy::ONE + ice_sum);
    let icy = ice_share > Dy::ratio(1, 5);
    let tide = set
        .path
        .map(|p| p.tide(mass, radius, icy))
        .unwrap_or_default();
    let inside = bodies::interior(w, &set.sources, age, t_eq0, &tide, sc);
    let heat = inside.heat_w / c.heat_unit_w;
    // Degassed share: the magma ocean releases `magma_degas` of the mantle
    // share that accretion melted, (3/5) G M / R over the sensible plus latent
    // heat of rock (capped at 1); convection then releases the rest at the
    // rate it processes the mantle: 1 - (1 - 0.9 melted) exp(-processed).
    let gm_r = c.gm_over_r_unit * mass / radius.max(Dy::sci(1, -6));
    let accreted = Dy::ratio(3, 5) * gm_r;
    let melted = (accreted
        / (bodies::cp_rock() * bodies::liquidus_k() * sc.chem + c.latent_rock * sc.chem))
        .min(Dy::ONE);
    let out = Dy::ONE - (Dy::ONE - c.magma_degas * melted) * (-inside.processed).exp();
    let field = inside.field;
    let act = activity(star_rel, age);
    // Exosphere temperature: the thermosphere factor times the equilibrium
    // temperature times 2 act / (1 + act) (1 at activity 1, approaching 2 for
    // a saturated star), times 3/2 with no magnetic field.
    let t_exo = c.thermosphere
        * t_eq0
        * (Dy::int(2) * act / (Dy::ONE + act))
        * if field.signum() > 0 {
            Dy::ONE
        } else {
            Dy::ratio(3, 2)
        };
    let mut gas = jeans_held(&ices, gm_r, t_exo, out, sc);
    photolyse(&mut gas, flux * band(blank.star_t, sc).brk * act);
    // Wind stripping: the budget is strip_rate x flux x the activity dose
    // over the world's mass. A broken fragment escapes with weight
    // exp(-E_b / E_hot), with E_b = bind_ev x the fragment's mass, bind_ev =
    // 2 m_H G M / R in eV, and E_hot the fragments' kinetic energy of a few eV.
    let wind = c.strip_rate * flux * wind_dose(star_rel, age) / mass.max(Dy::sci(1, -6));
    let bind_ev = gm_r * Dy::int(2) * c.m_h_kg * sc.proton / c.ev_j;
    let g = mass / (radius.max(Dy::sci(1, -6)) * radius.max(Dy::sci(1, -6)));
    let to_bar = c.g_unit_pa_per_massfrac * mass * g
        / (radius.max(Dy::sci(1, -6)) * radius.max(Dy::sci(1, -6)))
        / c.bar;
    let hpm = heat / mass.max(Dy::sci(1, -6));
    let hot = c.hot_ev * sc.chem;
    let cos_obl = set
        .path
        .map(|p| p.cos_obliquity_at(age))
        .unwrap_or(Dy::ratio(7071, 10000));
    let polar = polar_share(cos_obl);
    let held0 = gas.total();
    let clim = Climate {
        to_bar,
        flux,
        hpm,
        giant: false,
        polar,
    };
    let (air0, (t_surf, p_bar, air, seas, albedo, frozen)) =
        strip_by_wind(&clim, &mut gas, wind, bind_ev, hot, ctx);
    State {
        t_surf,
        p_bar,
        air,
        seas,
        frozen,
        heat,
        q: inside.q,
        q_tide: inside.q_tide,
        t_int: inside.t_int,
        q_cmb: inside.q_cmb,
        q_ad: inside.q_ad,
        r_core: inside.r_core,
        field,
        melted,
        processed: inside.processed,
        degassed: out,
        wind,
        t_exo,
        escaped: if held0.signum() > 0 {
            Dy::ONE - gas.total() / held0
        } else {
            Dy::ZERO
        },
        air_escaped: if air0.signum() > 0 {
            (Dy::ONE - p_bar / air0).max(Dy::ZERO)
        } else {
            Dy::ZERO
        },
        albedo,
        g,
        // The air column attenuates light as exp(-p / 10 bar).
        light: flux * (Dy::ONE - albedo) * (-(Dy::ratio(1, 10) * p_bar)).exp(),
        ..blank
    }
}

/// The state set by radiation and formation alone: flux, formation and star
/// temperatures, galactic environment, gathered ices, and the equilibrium
/// temperature at albedo 0.3 as the surface's. A giant's state is this.
fn radiated(set: &Setting, age: Dy, ctx: &Ctx) -> State {
    let c = constants();
    let sc = &ctx.sc;
    let w = set.world;
    let (mass, radius) = (Dy::from_q(&w.mass), Dy::from_q(&w.radius));
    let a_au = set.a_au.max(Dy::sci(1, -6));
    let star_rel = set.star_mass / sc.star_mass;
    let star_t = star_t(set.star_mass, set.lum, sc);
    let e_now = set.path.map(|p| p.e(age)).unwrap_or(Dy::ZERO);
    // The flux is L / a, with L the star's mean light over the bell
    // (stars.rs); the variation within a bell is read by stars::flicker and
    // does not enter the climate.
    let flux = set.lum / a_au;
    let t_form = c.t_unit * set.lum / a_au;
    let env = Dy::from_q(&crate::galaxy::density(&set.r_kpc.to_q()).mid());
    let cr = cosmic_rays(set.r_kpc);
    let sw = activity(star_rel, age) / activity(Dy::ONE, Dy::sci(457, -2)) / a_au;
    let mut ices = Mix::default();
    for (name, f) in &w.ices {
        if let Some(s) = Species::of(name) {
            ices.set(s, Dy::from_q(f));
        }
    }
    let t_eq0 = c.t_unit * flux * Dy::ratio(7, 10);
    State {
        age,
        giant: w.giant,
        mass,
        radius,
        flux,
        t_form,
        t_surf: t_eq0,
        p_bar: Dy::sci(1, 3),
        air: Mix::default(),
        seas: Mix::default(),
        frozen: Mix::default(),
        ices,
        heat: Dy::ZERO,
        q: Dy::ZERO,
        q_tide: Dy::ZERO,
        t_int: Dy::ZERO,
        q_cmb: Dy::ZERO,
        q_ad: Dy::ZERO,
        r_core: Dy::ZERO,
        field: Dy::ZERO,
        melted: Dy::ZERO,
        processed: Dy::ZERO,
        degassed: Dy::ZERO,
        wind: Dy::ZERO,
        t_exo: Dy::ZERO,
        escaped: Dy::ZERO,
        air_escaped: Dy::ZERO,
        env,
        star_t,
        albedo: Dy::ratio(3, 10),
        e: e_now,
        g: Dy::ZERO,
        cr,
        sw,
        light: Dy::ZERO,
        sc: *sc,
    }
}

/// The released volatiles Jeans escape leaves: each gathered ice times the
/// degassed share `out`, held by lambda = G M mu m_H / (k T_exo R) ramped
/// from 2 to `jeans_hold`.
fn jeans_held(ices: &Mix, gm_r: Dy, t_exo: Dy, out: Dy, sc: &Scales) -> Mix {
    let c = constants();
    let mut gas = Mix::default();
    for (s, f) in ices.iter() {
        let lam = gm_r * s.mu() / (c.k_over_mh / sc.proton * t_exo.max(Dy::ONE));
        let held = ((lam - Dy::int(2)) / (c.jeans_hold - Dy::int(2))).clamp(Dy::ZERO, Dy::ONE);
        gas.set(s, f * out * held);
    }
    gas
}

/// Photolysis above the declared ultraviolet dose `uv`: NH3 goes to N2, CH4
/// and CO to CO2, each converted by its mass ratio.
fn photolyse(gas: &mut Mix, uv: Dy) {
    if uv <= constants().photolysis_uv {
        return;
    }
    for (from, to, ratio) in [
        (Species::NH3, Species::N2, Dy::ratio(28, 34)),
        (Species::CH4, Species::CO2, Dy::ratio(44, 16)),
        (Species::CO, Species::CO2, Dy::ratio(44, 28)),
    ] {
        if let Some(v) = gas.remove(from) {
            gas.set(to, gas.amount(to) + v * ratio);
        }
    }
}

/// The climate solved, then re-solved after each of four wind passes
/// spending 1/4, 1/3, 1/2 and all of the remaining budget `wind`; returns
/// the first solve's air pressure (bar) and the final state. The wind
/// breaks what the climate keeps aloft, so a steam atmosphere loses its
/// hydrogen and a thin one can be stripped entirely.
fn strip_by_wind(
    clim: &Climate,
    gas: &mut Mix,
    wind: Dy,
    bind_ev: Dy,
    hot: Dy,
    ctx: &Ctx,
) -> (Dy, Solved) {
    let mut solved = clim.solve(gas, ctx);
    let air0 = solved.2.total();
    let lightest = gas
        .iter()
        .map(|(s, _)| s.fragment())
        .min()
        .unwrap_or(Dy::ONE);
    let reach = wind * (-(bind_ev * lightest / hot)).exp();
    let mut budget = wind;
    for share in [Dy::ratio(1, 4), Dy::ratio(1, 3), Dy::ratio(1, 2), Dy::ONE] {
        let air = &solved.2;
        if budget.signum() <= 0 || air.is_empty() || reach < Dy::sci(1, -4) * gas.total() {
            break;
        }
        let total = air.total();
        let spend = budget * share;
        if total.signum() > 0 && spend.signum() > 0 {
            for (s, pp) in air.iter() {
                if !gas.has(s) {
                    continue;
                }
                let broken = spend * pp / total;
                let escape = (-(bind_ev * s.fragment() / hot)).exp();
                let have = gas.amount(s);
                gas.set(
                    s,
                    have - have
                        .min(pp / clim.to_bar.max(Dy::sci(1, -12)))
                        .min(broken * escape),
                );
            }
            budget -= spend;
        }
        solved = clim.solve(gas, ctx);
    }
    (air0, solved)
}
