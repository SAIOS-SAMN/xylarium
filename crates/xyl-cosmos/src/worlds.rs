//! A world's air, seas and climate, and the chemistries they allow.
//!
//! The atmosphere is built like the planet: what was gathered when the world
//! formed (its ices, systems.rs), plus what its interior outgasses over time
//! (bodies.rs), less what escapes to space. From that follow which volatiles
//! are liquid at the surface pressure and temperature (solvents), how much
//! chemistry those conditions allow, what energy reaches it, and its pace.
//! No world's values are built in; the unit world is one possible input.
//!
//!   Temperature  absorbed light read linearly: flux falls as L / a (not
//!                L / a^2), and warmth is the absorbed share times the unit
//!                temperature (no T^4 law), with albedo from the surface
//!                cover. The star brightens along its main sequence (Gough
//!                1981); over an eccentric orbit the mean of 1 / r is 1 / a.
//!   Outgassing   a freezing magma ocean releases most of its volatiles (in
//!                proportion to the mantle share accretion melted); the
//!                convecting mantle releases what its heat has processed since.
//!   Escape       Jeans: a gas of molecular mass mu is held if lambda =
//!                G M mu m_H / (k T_exo R) is large; the exosphere is hotter
//!                under a young or small star's ultraviolet and with no field.
//!                Ultraviolet photolysis converts NH3 to N2 and CH4 to CO2; the
//!                stellar wind breaks molecules high in the air and fragments
//!                escape with a few eV (photochemical escape), applied in four
//!                passes with the climate re-solved between them.
//!   Greenhouse   surface warming 1 + 3/4 h with h = tau / (1 + tau); each gas
//!                weighted by its cluster's hold on outgoing light (bond law,
//!                in water's units; homonuclear pairs hold none). The scale
//!                `greenhouse_tau` is provisional (one fit across three reference
//!                worlds). Water vapour sits at saturation; the climate is
//!                the warmest stable balance, and a steam atmosphere rains out
//!                unless absorbed flux exceeds the runaway limit (Nakajima et
//!                al. 1992). Weathering on open water draws CO2 into carbonate
//!                until it balances outgassing. Cold poles trap condensables
//!                (seasonal polar caps).
//!   Solvents     liquid between the triple point and the boiling curve
//!                (Clausius-Clapeyron) up to the critical point, supercritical
//!                above both; a subsurface ocean where internal heat keeps the
//!                ice shell thinner than the ice layer; silicate melt above
//!                ~1400 K; a giant's deep plasma.
//!   Chemistry    a solvent, the bonds it allows (strong bonds that outlast a
//!                structure, weak bonds at the solvent's cohesion), the
//!                available elements, the energy supply (chemically active
//!                starlight, internal heat, radiolysis fuel) and the pace
//!                (Eyring, activation scaled to the solvent's range).
//!
//! Temperatures and pressures scale with the universe's laws (laws.rs
//! `Scales`). Arithmetic is dyadic (xyl-core `dyadic`): exact and
//! platform-independent.

use crate::bang::BigBang;
use crate::bodies::{self, Orbit};
use crate::chem::Chemistry;
use crate::elements::Element;
use crate::galaxy::Star;
use crate::hydro;
use crate::laws::Scales;
use crate::systems::{System, World};
use std::sync::OnceLock;
use xyl_core::certified::Encl;
use xyl_core::dyadic::Dy;
use xyl_core::motion::{Boost, Modes};
use xyl_core::Q;

fn n(x: i64) -> Dy {
    Dy::int(x)
}

fn r(a: i64, b: i64) -> Dy {
    Dy::ratio(a, b)
}

fn sci(m: i64, e: i32) -> Dy {
    Dy::sci(m, e)
}

// ---------------------------------------------------------------------- the volatiles

/// A volatile: what a world's air, seas and ices are made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Species {
    /// Water.
    H2O,
    /// Ammonia.
    NH3,
    /// Carbon dioxide.
    CO2,
    /// Methane.
    CH4,
    /// Molecular nitrogen.
    N2,
    /// Carbon monoxide.
    CO,
}

/// (species, its atoms by Z (the cluster the bond law closes), nebular
/// condensation K, phase diagram: triple K, triple bar, normal boiling K,
/// critical K, critical bar, heat of vaporisation kJ/mol, each as (mantissa,
/// power of ten); lightest fragment light breaks it into (proton masses);
/// share of a comet's ices by mass (Mumma & Charnley 2011)). Its mass and
/// how it holds light are read from its atoms (atoms.rs, elements.rs); the
/// rest are provisional catalogue values.
type Row = (
    Species,
    &'static [u32],
    i64,
    [(i64, i32); 6],
    i64,
    (i64, i32),
);

const TABLE: [Row; 6] = [
    (
        Species::H2O,
        &[8, 1, 1],
        182,
        [
            (27316, -2),
            (61, -4),
            (3731, -1),
            (6471, -1),
            (2206, -1),
            (407, -1),
        ],
        1,
        (1, 0),
    ),
    (
        Species::NH3,
        &[7, 1, 1, 1],
        131,
        [
            (1954, -1),
            (608, -4),
            (2398, -1),
            (4055, -1),
            (1135, -1),
            (233, -1),
        ],
        1,
        (1, -2),
    ),
    (
        Species::CO2,
        &[6, 8, 8],
        72,
        [
            (2166, -1),
            (518, -2),
            (1947, -1),
            (3041, -1),
            (738, -1),
            (167, -1),
        ],
        16,
        (1, -1),
    ),
    (
        Species::CH4,
        &[6, 1, 1, 1, 1],
        41,
        [
            (907, -1),
            (117, -3),
            (1117, -1),
            (1906, -1),
            (460, -1),
            (82, -1),
        ],
        1,
        (1, -2),
    ),
    (
        Species::N2,
        &[7, 7],
        22,
        [
            (632, -1),
            (125, -3),
            (774, -1),
            (1262, -1),
            (340, -1),
            (56, -1),
        ],
        14,
        (5, -3),
    ),
    (
        Species::CO,
        &[6, 8],
        25,
        [
            (681, -1),
            (154, -3),
            (816, -1),
            (1329, -1),
            (350, -1),
            (60, -1),
        ],
        14,
        (1, -1),
    ),
];

impl Species {
    /// Every species, in table order (the order [`Mix`] stores amounts in).
    pub const ALL: [Species; 6] = [
        Species::H2O,
        Species::NH3,
        Species::CO2,
        Species::CH4,
        Species::N2,
        Species::CO,
    ];

    fn idx(self) -> usize {
        self as usize
    }

    fn row(self) -> &'static Row {
        &TABLE[self.idx()]
    }

    /// The species' chemical formula, as used in world ice lists (`"H2O"`, `"NH3"`, ...).
    pub fn name(self) -> &'static str {
        match self {
            Species::H2O => "H2O",
            Species::NH3 => "NH3",
            Species::CO2 => "CO2",
            Species::CH4 => "CH4",
            Species::N2 => "N2",
            Species::CO => "CO",
        }
    }

    /// The species with chemical formula `name`.
    ///
    /// Returns `None` when `name` is not one of the six formulas of [`Species::name`].
    pub fn of(name: &str) -> Option<Species> {
        Species::ALL.iter().copied().find(|s| s.name() == name)
    }

    /// Its atoms by Z.
    pub fn atoms(self) -> &'static [u32] {
        self.row().1
    }

    /// Its molecular mass (proton masses): its atoms' weights summed.
    pub fn mu(self) -> Dy {
        static MU: OnceLock<[Dy; 6]> = OnceLock::new();
        MU.get_or_init(|| {
            let table = crate::elements::table().unwrap_or_default();
            Species::ALL.map(|s| {
                s.atoms()
                    .iter()
                    .map(|&z| {
                        table
                            .iter()
                            .find(|e| e.z == z)
                            .map(|e| Dy::from_q(&e.a))
                            .unwrap_or(Dy::ZERO)
                    })
                    .sum()
            })
        })[self.idx()]
    }

    /// How strongly it holds outgoing light: its cluster's hold by the bond
    /// law (atoms.rs `hold`) relative to water's; zero for a homonuclear pair.
    pub fn hold(self) -> Dy {
        static HOLD: OnceLock<[Dy; 6]> = OnceLock::new();
        HOLD.get_or_init(|| {
            let water = crate::atoms::hold(Species::H2O.atoms());
            Species::ALL.map(|s| {
                Dy::from_q(
                    &crate::atoms::hold(s.atoms())
                        .checked_div(&water)
                        .unwrap_or(Q::ZERO),
                )
            })
        })[self.idx()]
    }

    /// The lightest fragment light or the stellar wind breaks it into (proton masses):
    /// the mass that must escape for the species to be lost.
    pub fn fragment(self) -> Dy {
        n(self.row().4)
    }

    /// Its share of a comet's ices by mass, 0 to 1 (catalogue value).
    pub fn comet(self) -> Dy {
        let (m, e) = self.row().5;
        sci(m, e)
    }
}

/// A set of amounts by species (an air, its seas, a world's gas): `None` is
/// absent, which differs from present at zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Mix([Option<Dy>; 6]);

impl Mix {
    /// The amount of species `s`; `None` when it is absent.
    pub fn get(&self, s: Species) -> Option<Dy> {
        self.0[s.idx()]
    }

    /// The amount of species `s`, zero when absent.
    pub fn amount(&self, s: Species) -> Dy {
        self.get(s).unwrap_or(Dy::ZERO)
    }

    /// Whether species `s` is present (at any amount, including zero).
    pub fn has(&self, s: Species) -> bool {
        self.0[s.idx()].is_some()
    }

    /// Sets the amount of species `s`, marking it present.
    pub fn set(&mut self, s: Species, v: Dy) {
        self.0[s.idx()] = Some(v);
    }

    /// Removes species `s`, returning its amount; `None` when it was absent.
    pub fn remove(&mut self, s: Species) -> Option<Dy> {
        self.0[s.idx()].take()
    }

    /// Whether no species is present.
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(Option::is_none)
    }

    /// The sum of the amounts of every present species.
    pub fn total(&self) -> Dy {
        self.iter().map(|(_, v)| v).sum()
    }

    /// The present species and their amounts, in [`Species::ALL`] order.
    pub fn iter(&self) -> impl Iterator<Item = (Species, Dy)> + '_ {
        Species::ALL
            .iter()
            .filter_map(move |&s| self.get(s).map(|v| (s, v)))
    }
}

/// A volatile's phase diagram under a universe's laws: temperatures and the
/// heat of vaporisation scale with chemistry's energy, pressures with
/// condensed matter's.
#[derive(Clone, Copy, Debug)]
pub struct Diagram {
    /// Triple-point temperature (K).
    pub tt: Dy,
    /// Triple-point pressure (bar).
    pub pt: Dy,
    /// Normal boiling temperature (K) at the pressure scale `pref`.
    pub tb: Dy,
    /// Critical temperature (K).
    pub tc: Dy,
    /// Critical pressure (bar).
    pub pc: Dy,
    /// Heat of vaporisation (kJ/mol).
    pub l_kj: Dy,
    /// The pressure scale (1 bar under the reference scales): the boiling point's reference.
    pub pref: Dy,
}

/// What phase a volatile is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Phase {
    /// Below the triple temperature.
    Solid,
    /// Between the triple temperature and the boiling curve, at or above the triple pressure.
    Liquid,
    /// Above the boiling curve or below the triple pressure.
    Gas,
    /// At or above both the critical temperature and the critical pressure.
    Supercritical,
}

/// The constants the models run on, as dyadics, and a universe's scales and
/// phase diagrams.
pub struct Ctx {
    /// The universe's scales the diagrams and models are read under.
    pub sc: Scales,
    diagrams: [Diagram; 6],
}

impl Ctx {
    /// The context for a universe's scales: each species' catalogue phase
    /// diagram scaled by `sc` (temperatures and heats by `sc.chem`, pressures by
    /// `sc.pressure`).
    pub fn new(sc: Scales) -> Ctx {
        let diagrams = Species::ALL.map(|s| {
            let p = s.row().3.map(|(m, e)| sci(m, e));
            Diagram {
                tt: p[0] * sc.chem,
                pt: p[1] * sc.pressure,
                tb: p[2] * sc.chem,
                tc: p[3] * sc.chem,
                pc: p[4] * sc.pressure,
                l_kj: p[5] * sc.chem,
                pref: sc.pressure,
            }
        });
        Ctx { sc, diagrams }
    }

    /// The phase diagram of species `s` under this context's scales.
    pub fn diagram(&self, s: Species) -> &Diagram {
        &self.diagrams[s.idx()]
    }

    /// The measured points of a volatile's liquid-vapour curve, as
    /// (1/T, ln p): its triple point, its normal boiling point where it boils
    /// above its triple point (a solvent that sublimes at a bar has none),
    /// and its critical point. Clausius-Clapeyron makes ln p linear in 1/T
    /// between measured points, so the curve runs through every one of them:
    /// it boils at its triple temperature at its triple pressure, never below.
    fn liquid_curve(&self, s: Species) -> Vec<(Dy, Dy)> {
        let d = self.diagram(s);
        let mut pts = vec![(Dy::ONE / d.tt, d.pt.log())];
        if d.tb > d.tt && d.tb < d.tc {
            pts.push((Dy::ONE / d.tb, d.pref.log()));
        }
        pts.push((Dy::ONE / d.tc, d.pc.log()));
        pts
    }

    /// The boiling temperature of a volatile at `p_bar`: its liquid-vapour
    /// curve read at that pressure, the triple temperature at or below its
    /// triple pressure, capped at its critical point.
    pub fn boiling(&self, s: Species, p_bar: Dy) -> Dy {
        let d = self.diagram(s);
        if p_bar <= d.pt {
            return d.tt;
        }
        if p_bar >= d.pc {
            return d.tc;
        }
        let lp = p_bar.log();
        let pts = self.liquid_curve(s);
        for w in pts.windows(2) {
            let ((x1, l1), (x2, l2)) = (w[0], w[1]);
            if lp <= l2 {
                return Dy::ONE / (x1 + (lp - l1) * (x2 - x1) / (l2 - l1));
            }
        }
        d.tc
    }

    /// Vapour pressure over a volatile's liquid or ice at `t` (bar): over ice
    /// (below its triple point) Clausius-Clapeyron from the triple point;
    /// over liquid its liquid-vapour curve, through its measured points.
    pub fn saturation(&self, s: Species, t: Dy) -> Dy {
        let d = self.diagram(s);
        if t < d.tt {
            return d.pt
                * (d.l_kj * n(1000) / k().r_gas * (Dy::ONE / d.tt - Dy::ONE / t.max(Dy::ONE)))
                    .exp();
        }
        // above the critical point there is no liquid to cap it: the last
        // curve segment is extrapolated, so the fluid joins the envelope
        let x = Dy::ONE / t;
        let pts = self.liquid_curve(s);
        for w in pts.windows(2) {
            let ((x1, l1), (x2, l2)) = (w[0], w[1]);
            if x >= x2 || w[1] == pts[pts.len() - 1] {
                return (l1 + (x - x1) * (l2 - l1) / (x2 - x1)).exp();
            }
        }
        d.pc
    }

    /// The phase of species `s` at temperature `t` (K) and pressure `p_bar`
    /// (bar), read from its phase diagram and boiling curve.
    pub fn phase(&self, s: Species, t: Dy, p_bar: Dy) -> Phase {
        let d = self.diagram(s);
        if t >= d.tc && p_bar >= d.pc {
            return Phase::Supercritical;
        }
        if t < d.tt {
            return Phase::Solid;
        }
        if p_bar < d.pt {
            return Phase::Gas;
        }
        if t <= self.boiling(s, p_bar) {
            Phase::Liquid
        } else {
            Phase::Gas
        }
    }
}

/// The models' measured and declared constants.
struct K {
    t_unit: Dy,
    gm_over_r_unit: Dy,
    k_over_mh: Dy,
    g_unit_pa_per_massfrac: Dy,
    bar: Dy,
    kb_ev: Dy,
    r_gas: Dy,
    delivered: Dy,
    photolysis_uv: Dy,
    strip_rate: Dy,
    transport_bar: Dy,
    hot_ev: Dy,
    weather_e_k: Dy,
    weather_scale: Dy,
    jeans_hold: Dy,
    thermosphere: Dy,
    runaway_flux: Dy,
    greenhouse_tau: Dy,
    melt_k: Dy,
    giant_plasma_mass: Dy,
    solvent_barrier: Dy,
    strong_kt: Dy,
    weak_kt: Dy,
    geo_flux: Dy,
    heat_unit_w: Dy,
    giant_flux: Dy,
    plasma_window: Dy,
    photo_low: Dy,
    unit_g: Dy,
    latent_rock: Dy,
    magma_degas: Dy,
    m_h_kg: Dy,
    ev_j: Dy,
    k_ice: Dy,
    covalent_ev: Dy,
    silicate_ev: Dy,
    melt_network_ev: Dy,
    faraday: Dy,
}

fn k() -> &'static K {
    static C: OnceLock<K> = OnceLock::new();
    C.get_or_init(|| K {
        t_unit: n(278),
        gm_over_r_unit: sci(626, 5),
        k_over_mh: n(8254),
        g_unit_pa_per_massfrac: sci(117, 9),
        bar: sci(1, 5),
        kb_ev: Dy::from_q(&crate::package::Si::kb_ev()),
        r_gas: Dy::from_q(&crate::package::Si::r_gas()),
        delivered: sci(1, -4),
        // declared: bond-breaking light (flux units) above which NH3 and CH4 are
        // photolysed within years: 0.03 of a Sun's flux at its ~6.5% share and
        // middle-aged activity (Titan, at ~0.01, keeps its methane)
        photolysis_uv: sci(3, -2) * sci(65, -3) * r(14, 10),
        strip_rate: sci(2, -6),
        transport_bar: r(1, 10),
        hot_ev: r(5, 2),
        weather_e_k: r(23, 2),
        weather_scale: sci(1, -3),
        jeans_hold: n(15),
        thermosphere: n(4),
        runaway_flux: r(282, 340),
        // provisional: calibrated so the unit world sits near 288 K (reads 286 K)
        greenhouse_tau: r(31, 5),
        melt_k: n(1400),
        giant_plasma_mass: n(10),
        solvent_barrier: n(25),
        strong_kt: n(40),
        weak_kt: n(10),
        geo_flux: sci(87, -3) / n(340),
        heat_unit_w: sci(87, -3)
            * n(4)
            * Dy::PI
            * Dy::from_q(&crate::package::Si::world_radius_m())
            * Dy::from_q(&crate::package::Si::world_radius_m()),
        giant_flux: r(54, 10) / n(340),
        plasma_window: sci(2, -3),
        photo_low: r(2, 5),
        unit_g: sci(981, -2),
        latent_rock: sci(4, 5),
        magma_degas: r(9, 10),
        m_h_kg: Dy::from_q(&crate::package::Si::m_h_kg()),
        ev_j: Dy::from_q(&crate::package::Si::ev_j()),
        k_ice: n(567),
        covalent_ev: r(36, 10),
        silicate_ev: r(46, 10),
        melt_network_ev: Dy::ONE,
        faraday: Dy::from_q(&crate::package::Si::faraday()),
    })
}

/// Boltzmann's constant in eV/K.
pub fn kb_ev() -> Dy {
    k().kb_ev
}

/// A solvent's backbone bond (eV) under a universe's laws: silicate for a
/// melt, covalent otherwise.
pub fn strong_bond_ev(s: Solvent, sc: &Scales) -> Dy {
    (if s == Solvent::Melt {
        k().silicate_ev
    } else {
        k().covalent_ev
    }) * sc.chem
}

/// A covalent bond (eV) under a universe's laws.
pub fn covalent_ev(sc: &Scales) -> Dy {
    k().covalent_ev * sc.chem
}

/// Activation energy in units of kT at a solvent's own mid-range (declared).
pub fn solvent_barrier() -> Dy {
    k().solvent_barrier
}

/// The unit temperature (K at flux 1 and albedo 0: the flux unit's).
pub fn t_unit() -> Dy {
    k().t_unit
}

// ---------------------------------------------------------------------- the star

/// A star's ultraviolet activity relative to a quiet one: saturated while
/// young, longer for small stars (`star_mass` in the universe's stellar unit).
pub fn activity(star_mass: Dy, age: Dy) -> Dy {
    // spin-down rate linear in stellar mass
    let spin_down = star_mass.max(r(1, 20));
    Dy::ONE + n(9) * (-(age.max(Dy::ZERO) * spin_down)).exp()
}

/// The integrated stellar activity a world receives from its star's birth to
/// `age` (Gyr x activity), including the young star's saturated phase.
pub fn wind_dose(star_mass: Dy, age: Dy) -> Dy {
    let kk = star_mass.max(r(1, 20));
    let t = age.max(Dy::ZERO);
    t + n(9) * (Dy::ONE - (-(t * kk)).exp()) / kk
}

/// The share of a star's light carried above `x` kT: its light counted by
/// photons whose number grows linearly with energy under a Boltzmann factor,
/// so the share above x is (1 + x) e^(-x) (first degree in energy, not the
/// Planck cube and pi^4).
pub fn planck_above(x: Dy) -> Dy {
    if x.signum() <= 0 {
        return Dy::ONE;
    }
    if x > n(60) {
        return Dy::ZERO;
    }
    ((Dy::ONE + x) * (-x).exp()).min(Dy::ONE)
}

/// Starlight's colours against chemistry's: the share of a star's light that
/// can drive chemistry (enough to lift an electron, not enough to break a
/// bond) and the share that breaks bonds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Band {
    /// Share of the star's light, 0 to 1, between the photochemical threshold
    /// (2/5 of a covalent bond) and a covalent bond: light that drives chemistry.
    pub drive: Dy,
    /// Share of the star's light, 0 to 1, above a covalent bond's energy: light
    /// that breaks bonds.
    pub brk: Dy,
}

/// The split of a star of surface temperature `t_eff` (K) into light that
/// drives chemistry and light that breaks bonds, against a covalent bond under
/// the universe's scales `sc`.
pub fn band(t_eff: Dy, sc: &Scales) -> Band {
    let bond = k().covalent_ev * sc.chem;
    let kt = k().kb_ev * t_eff.max(Dy::ONE);
    let hi = planck_above(bond / kt);
    Band {
        drive: (planck_above(k().photo_low * bond / kt) - hi).max(Dy::ZERO),
        brk: hi,
    }
}

/// The stellar surface density at the galaxy's reference ring (suns per
/// pc^2): a declared ruler, 0.1 suns/pc^3 through a 600 pc layer. The
/// sterilising pulse's base rate (`life::sterilise_per_gyr`) is calibrated
/// against this crowding; [`cosmic_rays`] is read relative to it, so it
/// enters only through that calibration.
pub fn crowd_r0() -> Dy {
    r(1, 10) * n(600)
}

/// The galaxy's cosmic-ray flux where a system circles, relative to the
/// reference ring's ([`crowd_r0`]). Supernovae produce them, so the flux
/// follows the crowding of the ring the system sits in: its holding per kpc
/// over the reference ring's (`galaxy::density`, from the structure read off
/// the founding; no bulge or nucleus profile is declared). The Big Bang's
/// star-formation share and the orbit's exposure scale it where it is read
/// (`life`).
pub fn cosmic_rays(r_kpc: Dy) -> Dy {
    Dy::from_q(&crate::galaxy::density(&r_kpc.to_q()).mid()).max(Dy::ZERO)
}

// ---------------------------------------------------------------------- a world's setting

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
    /// Eccentricity its neighbours force for ever, the value `e0` relaxes toward.
    pub e_forced: Dy,
    /// The eccentricity's damping time (Gyr); zero or less means it sits at `e_forced`.
    pub tau_e: Dy,
    /// The cosine of its axis's tilt before it locks; a locked world stands
    /// upright to its host.
    pub cos_obliquity: Dy,
    /// The age at which its host locks its spin (Gyr).
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
    /// spin brakes linearly from its birth spin toward the orbital rate until
    /// the host locks it (bodies.rs `Orbit::at`).
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
        let two_pi = n(2) * Dy::PI;
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
    /// as an orbit in its age (Gyr): it goes as e^2, and the eccentricity
    /// relaxes from `e0` toward `e_forced` as a boost, so the heating is
    /// three modes - the forced part, the cross term and the free part - at
    /// rates 0, -1/tau_e and -2/tau_e.
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

    /// The cosine of its axis's tilt at age `t` (Gyr): its birth value, or 1
    /// (upright) once its host has locked it.
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
    /// Its star's luminosity (solar) as it shines; a swelling giant's glare
    /// is the caller's to supply.
    pub lum: Dy,
    /// Its distance from its star (AU): a moon's is its giant's.
    pub a_au: Dy,
    /// Its birth gas's metals (solar = 1).
    pub z: Dy,
    /// Its star's distance from the galaxy's centre (kpc).
    pub r_kpc: Dy,
    /// Its isotopes (kg in its mantle at its star's birth: bodies::heat_sources).
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
    /// The radiation: the year's mean light at its distance (flux units of
    /// 340 W/m^2), the bell's mean of the star's burn.
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
    /// Heat the core's iron conducts along its own adiabat (W/m^2); a dynamo
    /// runs while `q_cmb` exceeds it.
    pub q_ad: Dy,
    /// The core's radius (m).
    pub r_core: Dy,
    /// The dynamo's strength, 0 to 1.
    pub field: Dy,
    /// Share of the mantle its accretion melted, 0 to 1 (the magma ocean).
    pub melted: Dy,
    /// How many times its convecting heat could have melted its mantle.
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
    /// The stellar density of its galactic neighbourhood: the structure's
    /// crowding at its reach, galaxy::density.
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

/// A star's surface temperature (K): its light over its extent, read
/// linearly, with radius linear in mass in the universe's stellar scales
/// (not M^0.8, not the fourth root of L / R^2).
pub fn star_t(star_mass: Dy, lum: Dy, sc: &Scales) -> Dy {
    let radius = (star_mass / sc.star_mass).max(r(2, 25)) * sc.star_radius;
    n(5772) * lum.max(sci(1, -9)) / radius
}

/// The world as physics makes it at `age` Gyr after its star's birth: its
/// interior, atmosphere and oceans, temperature and field.
pub fn build(set: &Setting, age: Dy, ctx: &Ctx) -> State {
    let c = k();
    let sc = &ctx.sc;
    let w = set.world;
    let (mass, radius) = (Dy::from_q(&w.mass), Dy::from_q(&w.radius));
    let a_au = set.a_au.max(sci(1, -6));
    let star_rel = set.star_mass / sc.star_mass;
    let star_t = star_t(set.star_mass, set.lum, sc);
    let e_now = set.path.map(|p| p.e(age)).unwrap_or(Dy::ZERO);
    // the radiation: the star's mean light over the bell at the world's
    // distance (stars.rs: the burn the pulse drives; its swing within a bell
    // is the light's own reading, stars::flicker, never the climate's)
    let flux = set.lum / a_au;
    let t_form = c.t_unit * set.lum / a_au;
    let env = Dy::from_q(&crate::galaxy::density(&set.r_kpc.to_q()).mid());
    let cr = cosmic_rays(set.r_kpc);
    let sw = activity(star_rel, age) / activity(Dy::ONE, sci(457, -2)) / a_au;
    let mut ices = Mix::default();
    for (name, f) in &w.ices {
        if let Some(s) = Species::of(name) {
            ices.set(s, Dy::from_q(f));
        }
    }
    let t_eq0 = c.t_unit * flux * r(7, 10);
    let blank = State {
        age,
        giant: w.giant,
        mass,
        radius,
        flux,
        t_form,
        t_surf: t_eq0,
        p_bar: sci(1, 3),
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
        albedo: r(3, 10),
        e: e_now,
        g: Dy::ZERO,
        cr,
        sw,
        light: Dy::ZERO,
        sc: *sc,
    };
    if w.giant {
        // an envelope with no surface: its cloud tops
        return blank;
    }
    // the interior: its isotopes, warmth from its formation, tides from its host
    let ice_sum = ices.total();
    let ice_share = ice_sum / (Dy::ONE + ice_sum);
    let icy = ice_share > r(1, 5);
    let tide = set
        .path
        .map(|p| p.tide(mass, radius, icy))
        .unwrap_or_default();
    let inside = bodies::interior(w, &set.sources, age, t_eq0, &tide, sc);
    let heat = inside.heat_w / c.heat_unit_w;
    // degassed: what its magma ocean gave up as it froze - the share of the
    // mantle its accretion melted, (3/5) G M / R against the heat and latent
    // heat that melt rock - then what its convecting heat has melted since
    let gm_r = c.gm_over_r_unit * mass / radius.max(sci(1, -6));
    let accreted = r(3, 5) * gm_r;
    let melted = (accreted
        / (bodies::cp_rock() * bodies::liquidus_k() * sc.chem + c.latent_rock * sc.chem))
        .min(Dy::ONE);
    let out = Dy::ONE - (Dy::ONE - c.magma_degas * melted) * (-inside.processed).exp();
    let field = inside.field;
    // escape: a hot exosphere under its star's ultraviolet
    let act = activity(star_rel, age);
    // activity heats the thermosphere by 2 act / (1 + act): 1 for a quiet
    // star, approaching 2 for a saturated one
    let t_exo = c.thermosphere
        * t_eq0
        * (n(2) * act / (Dy::ONE + act))
        * if field.signum() > 0 { Dy::ONE } else { r(3, 2) };
    let mut gas = Mix::default();
    for (s, f) in ices.iter() {
        let lam = gm_r * s.mu() / (c.k_over_mh / sc.proton * t_exo.max(Dy::ONE));
        let held = ((lam - n(2)) / (c.jeans_hold - n(2))).clamp(Dy::ZERO, Dy::ONE);
        gas.set(s, f * out * held);
    }
    let uv = flux * band(star_t, sc).brk * act;
    if uv > c.photolysis_uv {
        // starlight's ultraviolet: nitrogen ends as N2, carbon as CO2
        for (from, to, ratio) in [
            (Species::NH3, Species::N2, r(28, 34)),
            (Species::CH4, Species::CO2, r(44, 16)),
            (Species::CO, Species::CO2, r(44, 28)),
        ] {
            if let Some(v) = gas.remove(from) {
                gas.set(to, gas.amount(to) + v * ratio);
            }
        }
    }
    // the star's wind breaks molecules high in the air; a fragment escapes if
    // the few eV it flies with beat its weight's binding, mu m_H G M / R
    let wind = c.strip_rate * flux * wind_dose(star_rel, age) / mass.max(sci(1, -6));
    let bind_ev = gm_r * n(2) * c.m_h_kg * sc.proton / c.ev_j;
    let g = mass / (radius.max(sci(1, -6)) * radius.max(sci(1, -6)));
    let to_bar = c.g_unit_pa_per_massfrac * mass * g
        / (radius.max(sci(1, -6)) * radius.max(sci(1, -6)))
        / c.bar;
    let hpm = heat / mass.max(sci(1, -6));
    let hot = c.hot_ev * sc.chem;
    let mut budget = wind;
    let cos_obl = set
        .path
        .map(|p| p.cos_obliquity_at(age))
        .unwrap_or(r(7071, 10000));
    let polar = polar_share(cos_obl);
    let held0 = gas.total();
    let clim = Climate {
        to_bar,
        flux,
        hpm,
        giant: false,
        polar,
    };
    let (mut t_surf, mut p_bar, mut air, mut seas, mut albedo, mut frozen) = clim.solve(&gas, ctx);
    let air0 = air.total();
    let lightest = gas
        .iter()
        .map(|(s, _)| s.fragment())
        .min()
        .unwrap_or(Dy::ONE);
    let reach = wind * (-(bind_ev * lightest / hot)).exp();
    for share in [r(1, 4), r(1, 3), r(1, 2), Dy::ONE] {
        // the wind acts on what the climate holds aloft (a steam atmosphere
        // loses hydrogen and hence water; a thin one can be stripped entirely);
        // the climate is re-solved on what remains
        if budget.signum() <= 0 || air.is_empty() || reach < sci(1, -4) * gas.total() {
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
                    have - have.min(pp / to_bar.max(sci(1, -12))).min(broken * escape),
                );
            }
            budget -= spend;
        }
        (t_surf, p_bar, air, seas, albedo, frozen) = clim.solve(&gas, ctx);
    }
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
        // the air's column dims the light linearly in its depth
        light: flux * (Dy::ONE - albedo) * (-(r(1, 10) * p_bar)).exp(),
        ..blank
    }
}

/// A pole's year-round sunlight over the world's mean: 4 sin(obliquity) / pi,
/// with the sine taken from the angle (not sqrt(1 - cos^2)).
pub fn polar_share(cos_obliquity: Dy) -> Dy {
    let sin = cos_obliquity.clamp(-Dy::ONE, Dy::ONE).acos().sin().abs();
    (n(4) * sin / Dy::PI).min(Dy::ONE)
}

/// What a world's climate is solved from.
struct Climate {
    to_bar: Dy,
    flux: Dy,
    hpm: Dy,
    giant: bool,
    polar: Dy,
}

impl Climate {
    /// The air and seas a world would have at surface temperature `t`, with
    /// `open` the open-water share of its seas' surface (1 unless ice and
    /// water coexist at a melting point). Weathering runs on open water, so
    /// the CO2 left in the air rises as ice spreads (balance / open).
    fn state_open(&self, gas: &Mix, t: Dy, open: Dy, ctx: &Ctx) -> (Mix, Mix) {
        let (mut air, mut seas) = (Mix::default(), Mix::default());
        for (s, f) in gas.iter() {
            if s == Species::CO {
                continue;
            }
            let pp = f * self.to_bar;
            let vap = pp.min(ctx.saturation(s, t));
            if vap.signum() > 0 {
                air.set(s, vap);
            }
            if (pp - vap).signum() > 0 {
                seas.set(s, pp - vap);
            }
        }
        let p = air.total();
        let wet = seas.has(Species::H2O)
            && ctx.phase(Species::H2O, t, p.max(sci(1, -9))) == Phase::Liquid
            && !self.giant;
        if let (true, Some(co2)) = (wet, air.get(Species::CO2)) {
            // the carbonate-silicate cycle: weathering turns CO2 to carbonate
            // until it balances what the interior outgasses, faster the warmer
            let tt = ctx.diagram(Species::H2O).tt;
            let balance = k().weather_scale
                * self.hpm
                * self.hpm
                * (-(n(2) * (t - tt) / (k().weather_e_k * ctx.sc.chem))).exp();
            air.set(
                Species::CO2,
                co2.min((balance / open.max(sci(1, -9))).max(sci(1, -6))),
            );
            seas.remove(Species::CO2);
        }
        (air, seas)
    }

    /// A world's poles are colder by as much as its sky fails to carry heat to
    /// them; a gas whose frost point the pole is below freezes there.
    fn cold_trap(&self, air: &mut Mix, seas: &mut Mix, t: Dy, ctx: &Ctx) {
        let carry = air.total() / k().transport_bar;
        let t_pole = t * ((self.polar + carry) / (Dy::ONE + carry));
        for s in [Species::CO2, Species::N2, Species::CH4, Species::NH3] {
            if let Some(a) = air.get(s) {
                let cap = ctx.saturation(s, t_pole);
                if a > cap {
                    seas.set(s, seas.amount(s) + a - cap);
                    air.set(s, cap);
                }
            }
        }
    }

    /// The temperature the atmosphere makes at an assumed `t`, and its state.
    fn model(&self, gas: &Mix, t: Dy, ctx: &Ctx) -> (Dy, Mix, Mix, Dy) {
        self.model_mixed(gas, t, None, ctx)
    }

    /// The same at a melting point held by ice and water together: `mixed`
    /// is (the ice's share of the seas' surface, the albedo fully iced).
    fn model_mixed(
        &self,
        gas: &Mix,
        t: Dy,
        mixed: Option<(Dy, Dy)>,
        ctx: &Ctx,
    ) -> (Dy, Mix, Mix, Dy) {
        let open = mixed.map_or(Dy::ONE, |(f, _)| Dy::ONE - f);
        let (mut air, mut seas) = self.state_open(gas, t, open, ctx);
        self.cold_trap(&mut air, &mut seas, t, ctx);
        let icy: Dy = seas
            .iter()
            .filter(|(s, _)| t < ctx.diagram(*s).tt)
            .map(|(_, v)| v)
            .sum();
        let albedo = if icy.signum() > 0 {
            r(3, 10) + r(2, 5) * (icy / (icy + Dy::ONE)).clamp(Dy::ZERO, Dy::ONE)
        } else {
            r(3, 10)
        };
        let albedo = mixed.map_or(albedo, |(f, a_iced)| {
            albedo + f * (a_iced - albedo).max(Dy::ZERO)
        });
        let gh: Dy = air.iter().map(|(s, v)| s.hold() * v).sum();
        // equilibrium temperature linear in absorbed flux; optical depth tau
        // from the absorbing gases, the atomic scale and the column's share of
        // a bar; held share h = tau / (1 + tau) (bounded by 1); the surface is
        // warmer by 1 + 3/4 h
        let t_eq = k().t_unit * self.flux * (Dy::ONE - albedo);
        let air_total = air.total().max(sci(1, -9));
        let tau = k().greenhouse_tau * gh * ctx.sc.atom * (air_total / (air_total + Dy::ONE));
        let held = tau / (Dy::ONE + tau);
        (t_eq * (Dy::ONE + r(3, 4) * held), air, seas, albedo)
    }

    /// Surface temperature, pressure, air, seas and albedo: the warmest stable
    /// state at which the temperature the atmosphere makes is the one it was
    /// assumed at.
    fn solve(&self, gas: &Mix, ctx: &Ctx) -> (Dy, Dy, Mix, Mix, Dy, Mix) {
        let runaway = k().runaway_flux * ctx.sc.chem;
        let base = k().t_unit * self.flux * r(3, 10);
        let step = r(108, 100);
        let mut grid = Vec::with_capacity(40);
        let mut t = base.max(n(15));
        for _ in 0..40 {
            grid.push(t);
            t = t * step;
        }
        let vals: Vec<Dy> = grid
            .iter()
            .map(|&t| self.model(gas, t, ctx).0 - t)
            .collect();
        let mut root = None;
        let mut frozen = Mix::default();
        let mut pinned: Option<(Dy, Dy)> = None;
        for i in (1..grid.len()).rev() {
            // the warmest crossing from warming to cooling
            if vals[i - 1].signum() >= 0 && vals[i].signum() < 0 {
                let (_, _, seas_i, alb_i) = self.model(gas, grid[i], ctx);
                if gas.amount(Species::H2O).signum() > 0
                    && !seas_i.has(Species::H2O)
                    && self.flux * (Dy::ONE - alb_i) < runaway
                {
                    // below the runaway limit a steam sky rains out
                    continue;
                }
                let (mut lo, mut hi) = (grid[i - 1], grid[i]);
                for _ in 0..16 {
                    let mid = (lo + hi) * r(1, 2);
                    if (self.model(gas, mid, ctx).0 - mid).signum() >= 0 {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                // take the cooling side: at a melting-point pin the world is liquid
                root = Some(hi);
                // Pinned at a melting point: frozen, the world warms; open, it
                // cools, so ice and water coexist. The ice share f changes the
                // albedo and closes the open water weathering runs on (CO2);
                // solve for the f at which the model temperature equals the
                // melting point (0 cools, 1 warms).
                let (_, _, seas_ice, a_iced) = self.model(gas, lo, ctx);
                if let Some(s) = seas_ice.iter().map(|(s, _)| s).find(|&s| {
                    let tt = ctx.diagram(s).tt;
                    lo <= tt && tt <= hi
                }) {
                    let at = |f: Dy| self.model_mixed(gas, hi, Some((f, a_iced)), ctx).0 - hi;
                    if at(Dy::ZERO).signum() < 0 && at(Dy::ONE).signum() >= 0 {
                        let (mut f_lo, mut f_hi) = (Dy::ZERO, Dy::ONE);
                        for _ in 0..24 {
                            let mid = (f_lo + f_hi) * r(1, 2);
                            if at(mid).signum() < 0 {
                                f_lo = mid;
                            } else {
                                f_hi = mid;
                            }
                        }
                        frozen.set(s, f_hi);
                        pinned = Some((f_hi, a_iced));
                    }
                }
                break;
            }
        }
        // no crossing kept: if cooling everywhere, take the coldest state;
        // otherwise the state nearest balance (a steam atmosphere that rains
        // out leaves the world where it warms least, not at the grid's top)
        let root = root.unwrap_or_else(|| {
            if vals[0].signum() < 0 {
                return grid[0];
            }
            grid.iter()
                .zip(&vals)
                .min_by(|a, b| a.1.abs().cmp(&b.1.abs()))
                .map(|(&t, _)| t)
                .unwrap_or(grid[grid.len() - 1])
        });
        let (_, air, seas, albedo) = self.model_mixed(gas, root, pinned, ctx);
        (root, air.total(), air, seas, albedo, frozen)
    }
}

// ---------------------------------------------------------------------- chemistry from the world

/// What a chemistry runs in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Solvent {
    /// A liquid or supercritical volatile.
    Volatile(Species),
    /// Silicate melt (surface above ~1400 K, scaled by the chemistry scale).
    Melt,
    /// A giant's deep plasma.
    Plasma,
}

impl Solvent {
    /// The solvent's name: the volatile's formula, `"melt"` or `"plasma"`.
    pub fn name(self) -> &'static str {
        match self {
            Solvent::Volatile(s) => s.name(),
            Solvent::Melt => "melt",
            Solvent::Plasma => "plasma",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Where on a world a medium sits.
pub enum Where {
    /// At the surface: an open sea, or a melt.
    Surface,
    /// In a liquid ocean beneath an ice shell.
    UnderIce,
    /// In a supercritical fluid of the air.
    Supercritical,
    /// In a giant's deep interior.
    Deep,
}

/// The engine's five regions of chemistry space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Region {
    /// A lit liquid at moderate temperature (230 to 450 K under the reference scales).
    Biotic,
    /// Cold chemistry: below 230 K under the reference scales, or in methane, nitrogen or
    /// ammonia.
    Cryogenic,
    /// Hot or rock chemistry: a melt, a supercritical fluid, or above 450 K.
    Mineral,
    /// A giant's plasma.
    Plasmic,
    /// A dark medium no starlight reaches, such as water under ice.
    Umbral,
}

impl Region {
    /// The region's lower-case name.
    pub fn name(self) -> &'static str {
        match self {
            Region::Biotic => "biotic",
            Region::Cryogenic => "cryogenic",
            Region::Mineral => "mineral",
            Region::Plasmic => "plasmic",
            Region::Umbral => "umbral",
        }
    }
}

/// A medium a chemistry could run in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Medium {
    /// What it runs in.
    pub solvent: Solvent,
    /// Where on the world it sits.
    pub at: Where,
    /// Its temperature (K).
    pub t: Dy,
    /// The free energy reaching it (flux units): drive-band light, the work the
    /// world's heat can do, and radiolysis, as each applies.
    pub energy: Dy,
    /// How much of the solvent there is: a sea's open part or a supercritical
    /// gas as partial pressure (bar); 1 for a melt or plasma.
    pub amount: Dy,
    /// Whether no starlight reaches it.
    pub dark: bool,
    /// Chemical energy radiolysis supplies it (flux units).
    pub radiolysis: Dy,
    /// Its region of chemistry space.
    pub region: Region,
    /// How much chemistry it allows, 0 to 1 (see [`richness`]).
    pub richness: Dy,
    /// How fast its chemistry runs relative to the unit temperature (see [`pace`]).
    pub pace: Dy,
    /// The share of its world's area this medium lives across, in \[0, 1\]:
    /// 1 on a rocky world; on a giant, the share of its depth over which the
    /// temperature, rising linearly from its surface to its interior, lies in
    /// the solvent's own range (`living_band`).
    pub band: Dy,
    /// The energy its field holds per carrier (eV): what tells a right
    /// pattern from a wrong one in a plasma, which has no bonds; zero for a
    /// bonded solvent.
    pub field_ev: Dy,
}

/// The world's own heat flowing out through each unit of its surface (flux units).
pub fn geothermal(w: &State) -> Dy {
    // heat over radius (not radius squared)
    k().geo_flux * w.heat / w.radius.max(sci(1, -6))
}

/// The work the world's heat can do for a chemistry at `t_medium` (flux
/// units): its flow x Carnot's share between its interior and the medium.
pub fn free_heat(w: &State, t_medium: Dy) -> Dy {
    let t_hot = if w.t_int.signum() > 0 {
        t_medium.max(w.t_int)
    } else {
        t_medium
    };
    geothermal(w) * (Dy::ONE - t_medium / t_hot.max(Dy::ONE))
}

/// The thickness (km) of an ice shell whose base sits at its melting point,
/// conducting `q` W/m^2 to a surface at `t_surf` (conductivity 567 / T);
/// `None` with no heat flow (no shell is thin enough).
pub fn shell_km(q: Dy, t_surf: Dy, t_melt: Dy) -> Option<Dy> {
    if q.signum() <= 0 {
        return None;
    }
    Some(k().k_ice * (t_melt.max(Dy::ONE) / t_surf.min(t_melt).max(Dy::ONE)).log() / q / n(1000))
}

/// Every medium on the world a chemistry could run in, with its temperature
/// and the free energy reaching it.
fn solvents(w: &State, ctx: &Ctx) -> Vec<(Solvent, Where, Dy, Dy, Dy, bool, Dy)> {
    let c = k();
    let chem = ctx.sc.chem;
    let mut out = Vec::new();
    let (t, p) = (w.t_surf, w.p_bar.max(sci(1, -9)));
    let light = w.light * band(w.star_t, &ctx.sc).drive;
    // what the particles that reach its surface split into fuel
    let rad = hydro::radiolysis(w);
    for (s, amount) in w.seas.iter() {
        let ph = ctx.phase(s, t, p);
        if ph == Phase::Liquid || ph == Phase::Supercritical {
            // a sea partly frozen at its melting point feeds by its open water
            let open = Dy::ONE - w.frozen.amount(s);
            out.push((
                Solvent::Volatile(s),
                Where::Surface,
                t,
                (light + free_heat(w, t) + rad) * open,
                amount * open,
                false,
                rad,
            ));
        } else if ph == Phase::Solid && matches!(s, Species::H2O | Species::NH3) && !w.giant {
            // a liquid ocean under an ice shell, where the world's heat (and
            // tides) hold the shell thinner than the ice there is
            let t_under = ctx.diagram(s).tt
                + if s == Species::H2O {
                    n(5) * chem
                } else {
                    Dy::ZERO
                };
            let layer_km = amount * c.bar / (c.unit_g * w.g.max(sci(1, -6)) * n(1000)) / n(1000);
            let q = if w.q.signum() > 0 {
                w.q
            } else {
                geothermal(w) * n(340)
            };
            if shell_km(q, t, t_under).is_some_and(|d| d < layer_km) {
                out.push((
                    Solvent::Volatile(s),
                    Where::UnderIce,
                    t_under,
                    free_heat(w, t_under) + rad,
                    amount,
                    true,
                    rad,
                ));
            }
        }
    }
    for (s, pp) in w.air.iter() {
        if ctx.phase(s, t, p) == Phase::Supercritical {
            out.push((
                Solvent::Volatile(s),
                Where::Supercritical,
                t,
                light + free_heat(w, t),
                pp,
                false,
                Dy::ZERO,
            ));
        }
    }
    if !w.giant && t >= c.melt_k * chem {
        out.push((
            Solvent::Melt,
            Where::Surface,
            t,
            light + geothermal(w),
            Dy::ONE,
            false,
            Dy::ZERO,
        ));
    }
    if w.giant && w.mass >= c.giant_plasma_mass {
        let energy = c.giant_flux * (w.mass / n(318)) * (n(11) / w.radius.max(Dy::ONE));
        out.push((
            Solvent::Plasma,
            Where::Deep,
            sci(1, 4) * chem,
            energy,
            Dy::ONE,
            true,
            Dy::ZERO,
        ));
    }
    out
}

/// The middle of a solvent's own liquid range (K).
pub fn mid(s: Solvent, ctx: &Ctx) -> Dy {
    let chem = ctx.sc.chem;
    match s {
        Solvent::Melt => n(2000) * chem,
        Solvent::Plasma => sci(1, 4) * chem,
        Solvent::Volatile(v) => {
            let d = ctx.diagram(v);
            // a solvent that sublimes at a bar is liquid only under pressure
            let top = if d.tb > d.tt { d.tb } else { d.tc };
            (d.tt + top.min(d.tc)) * r(1, 2)
        }
    }
}

/// How fast a chemistry runs, relative to the unit temperature (Eyring, the
/// activation energy scaled to its solvent's own range).
pub fn pace(s: Solvent, t: Dy, ctx: &Ctx) -> Dy {
    let m = mid(s, ctx);
    let mut t = t;
    if let Solvent::Volatile(v) = s {
        let d = ctx.diagram(v);
        if t > d.tc {
            // a supercritical fluid runs at its own temperature
            return t / k().t_unit;
        }
        t = t.max(d.tt);
    }
    (t / k().t_unit) * (k().solvent_barrier * (Dy::ONE - m / t.max(Dy::ONE))).exp()
}

/// A minded body keeps itself where its chemistry works best: the middle of
/// its solvent's range; its pace there is that temperature over the unit's.
pub fn body_temperature(s: Solvent, ctx: &Ctx) -> Dy {
    mid(s, ctx)
}

/// The pace of a minded body's chemistry: its body temperature (see
/// [`body_temperature`]) over the unit temperature.
pub fn body_pace(s: Solvent, ctx: &Ctx) -> Dy {
    mid(s, ctx) / k().t_unit
}

/// The region of chemistry space a medium falls in: plasma is plasmic; a
/// melt, a supercritical fluid or a medium above 450 K (scaled) is mineral;
/// dark water under ice is umbral; below 230 K (scaled) or in methane, nitrogen
/// or ammonia is cryogenic; otherwise umbral when dark and biotic when lit.
pub fn region(s: Solvent, at: Where, t: Dy, dark: bool, ctx: &Ctx) -> Region {
    let chem = ctx.sc.chem;
    if s == Solvent::Plasma {
        return Region::Plasmic;
    }
    if s == Solvent::Melt || at == Where::Supercritical || t > n(450) * chem {
        return Region::Mineral;
    }
    if dark && at == Where::UnderIce && s == Solvent::Volatile(Species::H2O) {
        return Region::Umbral;
    }
    if t < n(230) * chem
        || matches!(
            s,
            Solvent::Volatile(Species::CH4 | Species::N2 | Species::NH3)
        )
    {
        return Region::Cryogenic;
    }
    if dark {
        Region::Umbral
    } else {
        Region::Biotic
    }
}

/// The solvent's cohesion per molecule (eV).
pub fn weak_bond_ev(s: Solvent, ctx: &Ctx) -> Dy {
    match s {
        Solvent::Melt => k().melt_network_ev * ctx.sc.chem,
        Solvent::Plasma => Dy::ZERO,
        Solvent::Volatile(v) => ctx.diagram(v).l_kj * n(1000) / k().faraday,
    }
}

/// How much chemistry a medium allows, 0 to 1: the elements that can join it
/// x strong bonds that outlast a structure (bond energy over kT above ~40) x
/// weak bonds that hold yet flex on the solvent's own cohesion (near ~10 kT).
pub fn richness(w: &State, s: Solvent, t: Dy, z: Dy, ctx: &Ctx) -> Dy {
    let c = k();
    let t = t.max(Dy::ONE);
    let has = |sp: Species| w.ices.amount(sp).signum() > 0;
    let mut elements = Dy::ONE + z.min(Dy::ONE);
    if has(Species::CO2) || has(Species::CH4) {
        elements += Dy::ONE;
    }
    if has(Species::NH3) || has(Species::N2) || w.air.has(Species::N2) {
        elements += Dy::ONE;
    }
    let share = (elements / n(4)).clamp(Dy::ZERO, Dy::ONE);
    if s == Solvent::Plasma {
        // ionised matter holds no bonds: structure in fields
        return share * c.plasma_window;
    }
    let kt = c.kb_ev * t;
    let strong_ev = if s == Solvent::Melt {
        c.silicate_ev
    } else {
        c.covalent_ev
    } * ctx.sc.chem;
    let strong = Dy::ONE / (Dy::ONE + (-((strong_ev / kt - c.strong_kt) / n(4))).exp());
    let x = ((weak_bond_ev(s, ctx) / kt).log() - c.weak_kt.log()) / r(3, 5);
    let weak = (-x.abs()).exp();
    share * strong * weak
}

/// Every chemistry the world allows, with its region, richness, energy and pace.
pub fn media(w: &State, z: Dy, ctx: &Ctx) -> Vec<Medium> {
    solvents(w, ctx)
        .into_iter()
        .map(
            |(solvent, at, t, energy, amount, dark, radiolysis)| Medium {
                solvent,
                at,
                t,
                energy,
                amount,
                dark,
                radiolysis,
                region: region(solvent, at, t, dark, ctx),
                richness: richness(w, solvent, t, z, ctx),
                pace: pace(solvent, t, ctx),
                band: living_band(w, solvent, ctx),
                field_ev: if solvent == Solvent::Plasma {
                    plasma_field_ev(w)
                } else {
                    Dy::ZERO
                },
            },
        )
        .collect()
}

/// A giant's interior field energy per carrier (eV): the electron's
/// magnetic moment, 5.788e-5 eV per tesla, in its dynamo's field - its
/// strength (`State::field`, 0 to 1) times the dynamo's unit, 1e-4 T (an
/// open ruler).
pub fn plasma_field_ev(w: &State) -> Dy {
    sci(5788, -8) * w.field.max(Dy::ZERO) * sci(1, -4)
}

/// The share of a world's area a solvent lives across (`Medium::band`): 1 on
/// a rocky world; on a giant, the depth share x_hi - x_lo over which the
/// temperature T(x) = T_surf + (T_int - T_surf) x lies in the solvent's own
/// range - a volatile's from its triple point to its critical point, a
/// melt's and a plasma's from where they form to the interior.
pub fn living_band(w: &State, s: Solvent, ctx: &Ctx) -> Dy {
    if !w.giant {
        return Dy::ONE;
    }
    let chem = ctx.sc.chem;
    let (lo, hi) = match s {
        Solvent::Volatile(v) => {
            let d = ctx.diagram(v);
            (d.tt, d.tc)
        }
        Solvent::Melt => (k().melt_k * chem, w.t_int),
        Solvent::Plasma => (mid(Solvent::Plasma, ctx), w.t_int),
    };
    let span = w.t_int - w.t_surf;
    if span.signum() <= 0 {
        return if w.t_surf >= lo && w.t_surf <= hi {
            Dy::ONE
        } else {
            Dy::ZERO
        };
    }
    let at = |t: Dy| ((t - w.t_surf) / span).clamp(Dy::ZERO, Dy::ONE);
    (at(hi) - at(lo)).max(Dy::ZERO)
}

/// The richest medium (of a region, if asked): richness x energy (capped at 1).
pub fn best(ms: &[Medium], region: Option<Region>) -> Option<&Medium> {
    ms.iter()
        .filter(|m| region.is_none_or(|r| m.region == r))
        .max_by(|a, b| {
            let ka = a.richness * a.energy.min(Dy::ONE);
            let kb = b.richness * b.energy.min(Dy::ONE);
            ka.cmp(&kb)
                .then_with(|| a.solvent.name().cmp(b.solvent.name()))
        })
}

// ---------------------------------------------------------------------- a system's worlds

/// What a galaxy's worlds are read against: its seed, Big Bang, chemistry,
/// element table and laws.
#[derive(Clone, Copy)]
pub struct Galaxy<'c> {
    /// The galaxy's seed, from which every world's draws derive.
    pub seed: u64,
    /// The Big Bang the galaxy grew from.
    pub bang: &'c BigBang,
    /// The galaxy's chemical evolution (metals by place and time).
    pub chem: &'c Chemistry,
    /// The element table.
    pub elements: &'c [Element],
    /// The universe's scales.
    pub sc: Scales,
}

/// The settings of a system's worlds, in its order: each planet's orbit and
/// spin drawn (bodies::orbits, keyed by `path` as the system was), each moon
/// on its giant's chain with the eccentricity its neighbours force, and each
/// world's isotopes from the elements it was built with, kept as its ring's
/// history and the universe's half-lives allow.
pub fn settings<'a>(gal: &Galaxy, path: &[u64], sys: &'a System, star: &Star) -> Vec<Setting<'a>> {
    let (seed, bang, elements, sc) = (gal.seed, gal.bang, gal.elements, &gal.sc);
    let z = gal.chem.metals(&star.r, &star.born).unwrap_or(Q::ZERO);
    let star_kg = &sys.spec.mass * &crate::package::Si::star_mass_kg();
    let planets: Vec<&World> = sys.worlds.iter().filter(|w| w.host.is_none()).collect();
    let mut parts: Vec<xyl_core::rng::Part> = vec!["orbits".into()];
    parts.extend(path.iter().map(|&p| p.into()));
    let mut rng = xyl_core::Stream::derive(seed, &parts);
    let orbits = bodies::orbits(&star_kg, &planets, &mut rng);
    // the ring's formation time: its first light
    let tau_f = crate::galaxy::birth_gyr(&star.r).unwrap_or_else(|| bang.first_stars_gyr().mid());
    // isotopes are measured on the Sun's own matter (the reference world's bulk silicate):
    // a world's are read against the gas the Sun formed from as the model
    // gives it, so the Sun's birth gas holds exactly the measured inventory
    let mut kept = bodies::kept(&tau_f, &bang.first_stars_gyr().mid(), &star.born, sc);
    for (k, (_, el, _, _, _)) in kept.iter_mut().zip(bodies::isotopes().iter()) {
        let sun = crate::chem::reference_birth_gas(el).unwrap_or(Q::ONE);
        *k = k.checked_div(&sun).unwrap_or_else(|| k.clone());
    }
    let (star_mass, lum, z, r_kpc) = (
        Dy::from_q(&sys.spec.mass),
        Dy::from_q(&sys.spec.lum),
        Dy::from_q(&z),
        Dy::from_q(&star.r),
    );
    let mut planet = 0usize;
    let mut paths: Vec<Option<Path>> = Vec::with_capacity(sys.worlds.len());
    for w in &sys.worlds {
        let path = match (w.host, &w.a_host_radii) {
            (Some(h), Some(a_r)) => sys.worlds.get(h).map(|g| {
                let e = w.e_forced.as_ref().map(Dy::from_q).unwrap_or(Dy::ZERO);
                Path {
                    host_kg: Dy::from_q(&g.mass) * Dy::from_q(&crate::package::Si::world_mass_kg()),
                    a_m: Dy::from_q(a_r)
                        * Dy::from_q(&g.radius)
                        * Dy::from_q(&crate::package::Si::world_radius_m()),
                    e0: e,
                    e_forced: e,
                    tau_e: Dy::ZERO,
                    cos_obliquity: Dy::ONE,
                    lock_gyr: Dy::ZERO,
                    spin0: Dy::ZERO,
                }
            }),
            _ => {
                let o = orbits.get(planet).map(Path::of);
                planet += 1;
                o
            }
        };
        paths.push(path);
    }
    sys.worlds
        .iter()
        .zip(paths)
        .map(|(w, path)| Setting {
            world: w,
            star_mass,
            lum,
            a_au: Dy::from_q(&w.a),
            z,
            r_kpc,
            sources: bodies::heat_sources(w, elements, &kept),
            path,
        })
        .collect()
}

/// A world's own events over its life to `age` (Gyr since its star's
/// birth), each solved rather than stepped: its host locking its spin
/// (exact on its path), and its dynamo stopping where core-mantle heat flow
/// falls below what the core's adiabat conducts (a crossing of q_cmb - q_ad,
/// read from the world as built at each age).
pub fn events(set: &Setting, age: Dy, ctx: &Ctx) -> Vec<(&'static str, Encl)> {
    let mut out = Vec::new();
    if let Some(p) = set.path {
        if p.spin0.signum() > 0 && p.lock_gyr > Dy::ZERO && p.lock_gyr <= age {
            out.push(("its host locks its spin", Encl::exact(p.lock_gyr.to_q())));
        }
    }
    let start = crate::life::first_age(set.r_kpc, ctx);
    if !set.world.giant && age > start {
        let margin = |a: &Q| {
            let st = build(set, Dy::from_q(a), ctx);
            Encl::exact((st.q_cmb - st.q_ad).to_q())
        };
        if let Some(t) = xyl_core::event::crossing(
            &start.to_q(),
            &age.to_q(),
            &Q::new(1, 1000).unwrap_or(Q::ONE),
            margin,
        ) {
            out.push(("its dynamo stops", t));
        }
    }
    out
}

// ---------------------------------------------------------------------- the unit world

/// The unit world: the unit mass and radius at the unit star's distance and age, of
/// solar metals, with the reference world's bulk-silicate uranium, thorium and potassium -
/// the reference world (hydro.rs uses its field as the unit of fields).
/// Computed under default laws; it is a ruler, not a calibration target.
pub fn unit() -> &'static State {
    static U: OnceLock<State> = OnceLock::new();
    U.get_or_init(|| {
        // the core's share of rock from solar abundances (Lodders 2003): metal
        // (Fe, Ni, S) against silicate (Mg, Si and three O)
        let core = Q::new(63_988, 164_468).unwrap_or(Q::ZERO);
        let mantle = &Q::ONE - &core;
        // what comets bring across the snow line at unit crowding and metals
        let late = k().delivered;
        let ices: Vec<(&'static str, Q)> = Species::ALL
            .iter()
            .map(|s| (s.name(), (late * s.comet()).to_q()))
            .collect();
        // its layers as formation settles them: the comets' ices over the rock
        let ice_sum = ices.iter().fold(Q::ZERO, |a, (_, v)| &a + v);
        let ice = ice_sum
            .checked_div(&(&Q::ONE + &ice_sum))
            .unwrap_or(Q::ZERO);
        let rock = &Q::ONE - &ice;
        let layers_core = &core * &rock;
        let layers = [layers_core.clone(), &mantle * &rock, ice, Q::ZERO];
        let world = World {
            a: Q::ONE,
            mass: Q::ONE,
            radius: Q::ONE,
            giant: false,
            host: None,
            a_host_radii: None,
            layers,
            elements: Vec::new(),
            mantle: Vec::new(),
            ices,
            e_forced: None,
            swing: Q::ZERO,
            phase: Q::ZERO,
            merged_at: None,
        };
        // the reference world's bulk-silicate isotopes per kg of mantle today, read back to the Sun's birth
        let today = [sci(19856, -12), sci(144, -12), sci(80, -9), sci(2808, -11)];
        // the mantle the isotopes sit in: all but the core (bodies::interior's m_kg - m_core)
        let mantle_kg =
            (Dy::ONE - Dy::from_q(&layers_core)) * Dy::from_q(&crate::package::Si::world_mass_kg());
        let mut sources: [Q; 4] = Default::default();
        for (i, conc) in today.iter().enumerate() {
            let half = bodies::isotopes()[i].2.clone();
            let back = (sci(457, -2) * xyl_core::dyadic::ln2() / Dy::from_q(&half)).exp();
            sources[i] = (mantle_kg * *conc * back).to_q();
        }
        let set = Setting {
            world: &world,
            star_mass: Dy::ONE,
            lum: Dy::ONE,
            a_au: Dy::ONE,
            z: Dy::ONE,
            r_kpc: n(8),
            sources,
            path: None,
        };
        build(&set, sci(457, -2), &Ctx::new(Scales::default()))
    })
}

/// One degree of a star's life (Gyr): its main-sequence lifetime over 360,
/// never shorter than one core period. A world changes at its star's pace,
/// so it is treated as unchanged within a degree. The lifetime is the star's
/// own at its distance from the core (stars::life).
pub fn star_degree_gyr(r_kpc: Dy, ctx: &Ctx) -> Dy {
    let ring = crate::pulse::core_period_yr() / sci(1, 9);
    (crate::life::lifetime(r_kpc, ctx) / n(360))
        .max(ring)
        .max(sci(1, -12))
}

/// The degree of its star's life an age (Gyr) lies nearest.
pub fn degree_of(age: Dy, degree_gyr: Dy) -> i64 {
    (age / degree_gyr + r(1, 2)).trunc_i64()
}

/// The age (Gyr) at a degree of its star's life.
pub fn age_of_degree(k: i64, degree_gyr: Dy) -> Dy {
    n(k) * degree_gyr
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dy(s: &str) -> Dy {
        Dy::from_q(&crate::elements::decimal(s).unwrap())
    }

    #[test]
    fn phases_follow_their_diagrams() {
        let ctx = Ctx::new(Scales::default());
        for s in [Species::H2O, Species::CH4] {
            let d = *ctx.diagram(s);
            // at its reference pressure: solid below its triple point,
            // liquid between the triple point and the boiling point, gas
            // above the boiling point, supercritical past the critical point
            let p = d.pref;
            assert_eq!(ctx.phase(s, d.tt / n(2), p), Phase::Solid);
            assert_eq!(ctx.phase(s, (d.tt + d.tb) / n(2), p), Phase::Liquid);
            assert_eq!(ctx.phase(s, (d.tb + d.tc) / n(2), p), Phase::Gas);
            assert_eq!(ctx.phase(s, d.tc * n(2), d.pc * n(2)), Phase::Supercritical);
            // the boiling point at the reference pressure is the diagram's,
            // and falls under less pressure
            let b = ctx.boiling(s, p);
            assert!((b - d.tb).abs() < sci(1, -9), "boils at {b} vs {}", d.tb);
            assert!(ctx.boiling(s, p / n(10)) < b);
            // the saturation pressure at the triple point is the triple pressure
            assert!((ctx.saturation(s, d.tt) - d.pt).abs() < sci(1, -12));
        }
    }

    #[test]
    fn black_bodies_share_their_light_by_colour() {
        // the share above a threshold: all of it at zero, falling with the
        // threshold, none far above the peak
        assert_eq!(planck_above(Dy::ZERO), Dy::ONE);
        assert!(planck_above(Dy::ONE) > planck_above(n(10)));
        assert_eq!(planck_above(n(61)), Dy::ZERO);
        let sc = Scales::default();
        let (cool, hot) = (band(n(3000), &sc), band(n(6000), &sc));
        assert!(
            hot.brk > Dy::ZERO && hot.brk < Dy::ONE,
            "breaks {}",
            hot.brk
        );
        assert!(
            hot.drive > hot.brk,
            "more light drives chemistry than breaks it"
        );
        // a cooler star's light breaks less of what it shines
        assert!(cool.brk < hot.brk);
    }

    #[test]
    fn the_unit_world_reads_whole() {
        let u = unit();
        let ctx = Ctx::new(Scales::default());
        // the unit world's readings are a state, not a target: a core within
        // its radius, a field it either has or lacks, a warmth above nothing,
        // the radiation at its distance exactly its star's light over its
        // reach, and media that follow from them
        assert!(
            u.r_core > Dy::ZERO
                && u.r_core < u.radius * Dy::from_q(&crate::package::Si::world_radius_m())
        );
        assert!(u.field >= Dy::ZERO, "field {}", u.field);
        assert!(u.t_surf > Dy::ZERO, "t_surf {}", u.t_surf);
        assert_eq!(u.flux, Dy::ONE, "one sun's light at one unit of reach");
        let ms = media(u, Dy::ONE, &ctx);
        assert!(ms.iter().all(|m| m.t > Dy::ZERO));
    }

    #[test]
    fn a_world_far_out_freezes_and_a_close_one_bakes() {
        let ctx = Ctx::new(Scales::default());
        let u = unit();
        let world = World {
            a: Q::ONE,
            mass: Q::ONE,
            radius: Q::ONE,
            giant: false,
            host: None,
            a_host_radii: None,
            layers: [
                Q::new(2, 5).unwrap(),
                Q::new(3, 5).unwrap(),
                Q::ZERO,
                Q::ZERO,
            ],
            elements: Vec::new(),
            mantle: Vec::new(),
            ices: vec![
                ("H2O", dy("0.002").to_q()),
                ("CO2", dy("0.0002").to_q()),
                ("N2", dy("0.00002").to_q()),
            ],
            e_forced: None,
            swing: Q::ZERO,
            phase: Q::ZERO,
            merged_at: None,
        };
        let sources = [
            dy("1e17").to_q(),
            dy("1e15").to_q(),
            dy("3e17").to_q(),
            dy("5e16").to_q(),
        ];
        let at = |a: Dy| {
            let set = Setting {
                world: &world,
                star_mass: Dy::ONE,
                lum: Dy::ONE,
                a_au: a,
                z: Dy::ONE,
                r_kpc: n(8),
                sources: sources.clone(),
                path: None,
            };
            build(&set, n(3), &ctx)
        };
        let (close, far) = (at(r(2, 5)), at(n(5)));
        assert!(
            close.t_surf > far.t_surf,
            "{} vs {}",
            close.t_surf,
            far.t_surf
        );
        assert!(close.t_surf > u.t_surf && u.t_surf > far.t_surf);
        // a giant has no surface: no air or seas of its own, its plasma deep
        let giant = World {
            giant: true,
            mass: Q::int(318),
            radius: Q::int(11),
            layers: [Q::ZERO, Q::ZERO, Q::ZERO, Q::ONE],
            ..world.clone()
        };
        let set = Setting {
            world: &giant,
            star_mass: Dy::ONE,
            lum: Dy::ONE,
            a_au: n(5),
            z: Dy::ONE,
            r_kpc: n(8),
            sources,
            path: None,
        };
        let g = build(&set, n(3), &ctx);
        let ms = media(&g, Dy::ONE, &ctx);
        assert!(ms.iter().any(|m| m.region == Region::Plasmic));
    }
}
