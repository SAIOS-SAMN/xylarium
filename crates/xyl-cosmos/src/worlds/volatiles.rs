//! The volatiles: species, their phase diagrams and mixes, and the context
//! they are read under.

use super::*;

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
pub(super) type Row = (
    Species,
    &'static [u32],
    i64,
    [(i64, i32); 6],
    i64,
    (i64, i32),
);

pub(super) const TABLE: [Row; 6] = [
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
        Dy::int(self.row().4)
    }

    /// Its share of a comet's ices by mass, 0 to 1 (catalogue value).
    pub fn comet(self) -> Dy {
        let (m, e) = self.row().5;
        Dy::sci(m, e)
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
            let p = s.row().3.map(|(m, e)| Dy::sci(m, e));
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
                * (d.l_kj * Dy::int(1000) / constants().r_gas
                    * (Dy::ONE / d.tt - Dy::ONE / t.max(Dy::ONE)))
                .exp();
        }
        // above the critical point there is no liquid: the last curve
        // segment is extrapolated, so the pressure keeps rising with T
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
