//! Knowledge: the stock a people holds in four domains, how it grows and
//! decays over a span, and the mastery it yields. There is no tech tree.
//!
//! Inputs: a people's traits and numbers (`cosmic::People`), its bodies'
//! power (W, `growth::Power` via `cosmic::Cosmic::power`), its consciousness
//! gauge (`civ::Gauges`), and `xyl_cosmos::worlds` / `life` for temperature
//! and copying fidelity.
//!
//! - Stock: per [`Domain`] (energy, materials, biology, information), in
//!   knowledge units (KU: the free thought of [`KU_MINDS`] unit minds for a
//!   year). [`mastery`] = 1 - 1 / (1 + ln(1 + K / U)), U =
//!   [`knowledge_unit`]; it is 0 at K = 0 and rises toward 1 with K.
//! - Thought ([`effort`]): power x Landauer's T_unit / T x impedance x
//!   (1 + g_C), over a knowledge unit's power. Lived thought is a share
//!   ([`lived_share`], raised by 1 + collective) of all bodies' power; free
//!   thought is the bodies' power times 1 - N / N_cap, plus works
//!   (`works_w`, zero until modelled).
//! - Direction ([`directed`]): lived thought by the profile's affinity; free
//!   thought by curiosity + the needs' pulls + a ruler's focus. Only hunger
//!   is computed from a people's state ([`Needs::of`]); the other pulls are
//!   zero until their layers exist.
//! - Carry ([`advance`]): dK/dt = E - lam K in closed form, lam from
//!   [`forgetting`] (knowers die at a rate of one per generation; a
//!   collective's pooling spares the share [`pooled_keep`] x collective of
//!   that loss), so the result does not depend on step size.
//!
//! [`Knowledge`] holds every people's [`Stocks`], indexed like
//! `cosmic.peoples`; stocks and focus are held in the checkpoint. Declared
//! rulers: [`knowledge_unit`], [`lived_share`], [`curiosity`],
//! [`pooled_keep`] and the sense weights.

use crate::cosmic::{People, Shape, Strategy, Traits};
use crate::growth::{self, Rulers};
use xyl_core::dyadic::Dy;
use xyl_cosmos::worlds::{t_unit, Ctx, Solvent, Where};

/// A domain of knowledge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Domain {
    /// Energy: power capture and drives; pulled by a power shortfall.
    Energy,
    /// Materials: ore, building and digging; pulled by a spent crust.
    Materials,
    /// Biology: harvest and medicine; pulled by hunger and failing births.
    Biology,
    /// Information: the science of signals and sensing; pulled by trade.
    Information,
}

impl Domain {
    /// Every domain, in index order.
    pub const ALL: [Domain; 4] = [
        Domain::Energy,
        Domain::Materials,
        Domain::Biology,
        Domain::Information,
    ];

    /// Position of the domain in a [`ByDomain`] array (0 energy, 1 materials,
    /// 2 biology, 3 information).
    pub fn index(self) -> usize {
        match self {
            Domain::Energy => 0,
            Domain::Materials => 1,
            Domain::Biology => 2,
            Domain::Information => 3,
        }
    }

    /// Lower-case name of the domain, as used in reports and annals.
    pub fn name(self) -> &'static str {
        match self {
            Domain::Energy => "energy",
            Domain::Materials => "materials",
            Domain::Biology => "biology",
            Domain::Information => "information",
        }
    }
}

/// A value per domain, indexed by `Domain::index`.
pub type ByDomain = [Dy; 4];

// -------------------------------------------------------------------- Constants.

/// The experience scale U of [`wright`] and [`mastery`], in KU: U = 10 KU,
/// the free thought of ten thousand unit minds for one year. Declared ruler.
pub fn knowledge_unit() -> Dy {
    Dy::int(10)
}

/// Number of unit minds whose free thought for one year makes one knowledge
/// unit (KU). Declared ruler.
pub const KU_MINDS: i64 = 1000;

/// The share of every member's thought that living adds to the people's
/// knowledge (lived thought). Declared ruler.
pub fn lived_share() -> Dy {
    Dy::ratio(1, 20)
}

/// The floor of undirected inquiry: the weight every domain receives for free
/// thought before needs and focus. Declared ruler.
pub fn curiosity() -> Dy {
    Dy::ratio(1, 4)
}

/// The share of forgetting that pooling spares a fully collective people
/// (collective = 1). Declared ruler.
pub fn pooled_keep() -> Dy {
    Dy::ratio(1, 2)
}

// ---------------------------------------------------------- Experience and cost.

/// Returns the cost multiplier after `experience` x (in KU):
/// 1 / (1 + ln(1 + x / U)) with U = [`knowledge_unit`], which is 1 with no
/// experience and falls toward 0; negative experience is read as zero.
pub fn wright(experience: Dy) -> Dy {
    Dy::ONE / (Dy::ONE + (Dy::ONE + experience.max(Dy::ZERO) / knowledge_unit()).log())
}

/// Returns the mastery that `experience` yields, 1 - [`wright`], in [0, 1);
/// zero for zero or negative experience (laws.mastery).
pub fn mastery(experience: Dy) -> Dy {
    // Zero experience gives exactly zero mastery, avoiding rounding at zero.
    if experience.signum() <= 0 {
        return Dy::ZERO;
    }
    Dy::ONE - wright(experience)
}

/// Returns the Landauer factor T_unit / T: bits erased per joule at
/// temperature `t` (K) relative to the unit temperature, with T floored at
/// 1 K.
pub fn landauer(t: Dy) -> Dy {
    t_unit() / t.max(Dy::ONE)
}

/// Returns a unit body's metabolism (W): Kleiber's law at the mind-bearing
/// body's mass and the unit pace (`growth::unit_metabolism_w`); the rulers
/// are not read.
pub fn unit_body_w(_rul: &Rulers) -> Dy {
    crate::growth::unit_metabolism_w()
}

/// The power equivalent of one knowledge unit (W): the metabolism of
/// `KU_MINDS` unit bodies.
fn ku_body_w(rul: &Rulers) -> Dy {
    Dy::int(KU_MINDS) * unit_body_w(rul)
}

// -------------------------------------------------- Mind profile (mind.profile).

/// The weights by which a life's primary sense directs its sciences, in
/// domain order (energy, materials, biology, information); each row sums to
/// 2. Declared rulers.
fn sense_weights(sense: &str) -> ByDomain {
    match sense {
        "chemistry" => [Dy::ratio(1, 8), Dy::ratio(3, 4), Dy::ONE, Dy::ratio(1, 8)],
        "heat and pressure" => [
            Dy::ratio(3, 4),
            Dy::ratio(3, 4),
            Dy::ratio(3, 8),
            Dy::ratio(1, 8),
        ],
        "heat" => [Dy::ONE, Dy::ratio(3, 4), Dy::ratio(1, 8), Dy::ratio(1, 8)],
        "fields" => [Dy::ONE, Dy::ratio(1, 8), Dy::ratio(1, 8), Dy::ratio(3, 4)],
        _ => [Dy::ratio(1, 2), Dy::ratio(1, 4), Dy::ratio(1, 4), Dy::ONE],
    }
}

/// What kind of mind a people is (mind.profile): each axis 0 .. 1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    /// How far it thinks as one.
    pub collective: Dy,
    /// How far its thinking lives through its body.
    pub distributed: Dy,
    /// How far it rewrites itself within a life.
    pub plastic: Dy,
    /// How readable the people's heredity is to the people.
    pub legible: Dy,
    /// Whether the people lives in water or the dark (swimming or drifting
    /// form, dark habitat, or under ice): without fire, energy science comes
    /// late.
    pub aquatic: bool,
    /// The sense weights scaled to average 1 over the four domains.
    pub affinity: ByDomain,
}

/// Returns a people's profile from its traits (mind.profile). Declared
/// weights: collective = social x (1.5 superorganism, 0.75 distributed, 0.5
/// society); distributed 0.6 for a colony or a spread mind, 0.4 more for a
/// swimming, drifting or many-limbed body; plastic 0.5 aquatic (else 0.1),
/// 0.3 more for an r-strategist, plus 0.2 x distributed; each capped at 1.
/// legible = ln(b / eps0) / ln(I_MIND), clamped to [0, 1], where b is the
/// bits per monomer and eps0 the chemistry's copying fidelity at the body's
/// temperature (life.rs); a plasma life has legible = 0.
pub fn profile(t: &Traits, ctx: &Ctx) -> Profile {
    let aquatic = matches!(t.form, "swimming" | "drifting") || t.dark || t.at == Where::UnderIce;
    let shape_w = match t.shape {
        Shape::Superorganism => Dy::ratio(3, 2),
        Shape::Distributed => Dy::ratio(3, 4),
        Shape::Society => Dy::ratio(1, 2),
    };
    let collective = (t.social * shape_w).min(Dy::ONE);
    let mut distributed = Dy::ZERO;
    if matches!(t.shape, Shape::Distributed | Shape::Superorganism) {
        distributed += Dy::ratio(3, 5);
    }
    if matches!(t.form, "swimming" | "drifting" | "low and many-limbed") {
        distributed += Dy::ratio(2, 5);
    }
    let distributed = distributed.min(Dy::ONE);
    let plastic = ((if aquatic {
        Dy::ratio(1, 2)
    } else {
        Dy::ratio(1, 10)
    }) + (if t.strategy == Strategy::R {
        Dy::ratio(3, 10)
    } else {
        Dy::ZERO
    }) + Dy::ratio(1, 5) * distributed)
        .min(Dy::ONE);
    let legible = if t.solvent == Solvent::Plasma {
        Dy::ZERO
    } else {
        let eps0 = xyl_cosmos::life::fidelity(t.solvent, t.body_t, ctx);
        ((xyl_cosmos::life::bits_per_monomer() / eps0.max(Dy::sci(1, -300))).log()
            / xyl_cosmos::life::i_mind().log())
        .clamp(Dy::ZERO, Dy::ONE)
    };
    let w = sense_weights(t.sense);
    let total: Dy = w.iter().copied().sum();
    let affinity = w.map(|x| Dy::int(4) * x / total.max(Dy::sci(1, -9)));
    Profile {
        collective,
        distributed,
        plastic,
        legible,
        aquatic,
        affinity,
    }
}

// ----------------------------------------------------- Life span and forgetting.

/// Returns how long a knower lives (standard years): one generation of the
/// people (growth.rs).
pub fn knower_life_yr(traits: &Traits, rul: &Rulers) -> Dy {
    growth::generation_yr(traits, rul)
}

/// Returns the yearly share of a people's knowledge that is lost
/// (knowledge.forgetting): (1 - e^(-1 / life)) x (1 - pooled_keep x
/// collective), with knowers dying at the rate 1 / life and the loss reduced
/// for a collective.
pub fn forgetting(life_yr: Dy, collective: Dy) -> Dy {
    let keep = (-(Dy::ONE / life_yr.max(Dy::ratio(1, 1000)))).exp();
    let pooled = Dy::ONE - pooled_keep() * collective.clamp(Dy::ZERO, Dy::ONE);
    ((Dy::ONE - keep) * pooled).clamp(Dy::ZERO, Dy::ONE)
}

// -------------------------------------------------------------- Needs and pulls.

/// The needs that direct a people's free thought (knowledge.pulls), each >= 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Needs {
    /// Hunger: 1 - N_cap / N when the population N exceeds the capacity
    /// N_cap, else 0.
    pub hunger: Dy,
    /// Births failing (0 .. 1).
    pub births: Dy,
    /// The share of its crust's ore it has spent (0 .. 1).
    pub crust_spent: Dy,
    /// Its power short of its needs (0 .. 1).
    pub power_short: Dy,
    /// Trade it carries on (0 .. 1).
    pub trade: Dy,
}

impl Needs {
    /// Returns the needs computed from a people's numbers: hunger when the
    /// population exceeds the capacity; the other needs are zero.
    pub fn of(p: &People) -> Needs {
        let hunger = if p.population > p.capacity && p.population.signum() > 0 {
            Dy::ONE - p.capacity / p.population
        } else {
            Dy::ZERO
        };
        Needs {
            hunger,
            ..Needs::default()
        }
    }

    /// Returns the pull on each domain: power shortfall on energy, spent crust
    /// on materials, hunger plus failing births on biology, and trade (capped
    /// at 1) on information.
    pub fn pulls(&self) -> ByDomain {
        [
            self.power_short,
            self.crust_spent,
            self.hunger + self.births,
            self.trade.min(Dy::ONE),
        ]
    }
}

// ----------------------------------------------------------------------- Effort.

/// The inputs to a people's thought over a span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thinkers {
    /// Its bodies' metabolism, all of them (W; growth.rs Power.bodies_w).
    pub bodies_w: Dy,
    /// Power its works turn to thought (W; zero until it captures flows and
    /// builds machines that think; not yet modelled).
    pub works_w: Dy,
    /// Current numbers (bodies).
    pub population: Dy,
    /// Carrying capacity of the people's world (bodies); sets the free share
    /// 1 - population / capacity.
    pub capacity: Dy,
    /// The temperature its minds run at (K).
    pub body_t: Dy,
    /// Impedance factor on thought, 0 .. 1; set to 1 (no reduction) until
    /// modelled.
    pub impedance: Dy,
    /// The needs that direct free thought.
    pub needs: Needs,
    /// The people's mind profile, which weights lived thought by affinity and
    /// collectivity.
    pub profile: Profile,
    /// Its life expectancy (years).
    pub life_yr: Dy,
    /// Its fluid intelligence, 0 .. 1 (not yet modelled, read as 0).
    pub fluid: Dy,
    /// Its consciousness gauge as last read, 0 .. 1 (civ/mod.rs `Gauges`): its
    /// thought's factor F = 1 + g_C.
    pub consciousness: Dy,
}

impl Thinkers {
    /// Returns a people's thinkers from its numbers and its bodies' power
    /// (cosmic/mod.rs `Cosmic::power`), with no works, impedance 1, zero
    /// fluid intelligence and zero consciousness.
    pub fn of(p: &People, bodies_w: Dy, ctx: &Ctx) -> Thinkers {
        let rul = Rulers::default();
        Thinkers {
            bodies_w,
            works_w: Dy::ZERO,
            population: p.population,
            capacity: p.capacity,
            body_t: p.traits.body_t,
            impedance: Dy::ONE,
            needs: Needs::of(p),
            profile: profile(&p.traits, ctx),
            life_yr: knower_life_yr(&p.traits, &rul),
            fluid: Dy::ZERO,
            consciousness: Dy::ZERO,
        }
    }
}

/// A year's thought, in KU (knowledge.lived and knowledge.effort).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Effort {
    /// Lived thought: the share of members' thought that living adds,
    /// directed by the profile's affinity.
    pub lived: Dy,
    /// Free thought: the bodies' power freed by spare capacity, plus works.
    pub free: Dy,
}

impl Effort {
    /// Lived plus free thought (KU a year).
    pub fn total(&self) -> Dy {
        self.lived + self.free
    }
}

/// Returns a people's thought per year (KU). Each watt counts Landauer's
/// T_unit / T at the minds' temperature, times the impedance and the
/// consciousness factor F = 1 + g_C, over a knowledge unit's power. Lived
/// thought is the bodies' power x [`lived_share`] x (1 + collective); free
/// thought is the bodies' power x the free share 1 - N / N_cap (zero when
/// the population meets or exceeds the capacity) plus the works' power.
pub fn effort(t: &Thinkers, rul: &Rulers) -> Effort {
    let depth = landauer(t.body_t)
        * t.impedance.clamp(Dy::ZERO, Dy::ONE)
        * (Dy::ONE + t.consciousness.clamp(Dy::ZERO, Dy::ONE));
    let per_ku = ku_body_w(rul);
    let lived = t.bodies_w.max(Dy::ZERO) * lived_share() * (Dy::ONE + t.profile.collective) * depth
        / per_ku;
    let free_share = if t.capacity.signum() > 0 && t.population.signum() > 0 {
        (Dy::ONE - t.population / t.capacity).max(Dy::ZERO)
    } else {
        Dy::ZERO
    };
    let free = (t.bodies_w.max(Dy::ZERO) * free_share + t.works_w.max(Dy::ZERO)) * depth / per_ku;
    Effort { lived, free }
}

/// Returns how a span's thought divides among the domains, and the free
/// thought's weights: lived thought by affinity, free thought by curiosity +
/// pull + 1 for the focused domain (knowledge.quarter's weights).
pub fn directed(e: &Effort, t: &Thinkers, focus: Option<Domain>) -> (ByDomain, ByDomain) {
    let pull = t.needs.pulls();
    let weights: ByDomain = std::array::from_fn(|i| {
        curiosity()
            + pull[i].max(Dy::ZERO)
            + if focus.is_some_and(|f| f.index() == i) {
                Dy::ONE
            } else {
                Dy::ZERO
            }
    });
    let wsum: Dy = weights.iter().copied().sum();
    let asum: Dy = t.profile.affinity.iter().copied().sum();
    let out = std::array::from_fn(|i| {
        e.lived * t.profile.affinity[i] / asum.max(Dy::sci(1, -9))
            + e.free * weights[i] / wsum.max(Dy::sci(1, -9))
    });
    (out, weights)
}

// ---------------------------------------------------------- Carry of the stocks.

/// Returns a domain's stock carried over `years` under thought `e_ku` a year
/// and a continuous forgetting rate `lam` (per year): dK/dt = e - lam K,
/// solved in closed form, K(t) = e / lam + (K0 - e / lam) exp(-lam t), and
/// K0 + e t where nothing is forgotten; a long span settles at e / lam.
/// Negative stock and thought are read as zero; a span of zero or fewer
/// years returns the stock unchanged.
pub fn advance(stock_ku: Dy, e_ku: Dy, lam: Dy, years: Dy) -> Dy {
    if years.signum() <= 0 {
        return stock_ku;
    }
    let (k0, e) = (stock_ku.max(Dy::ZERO), e_ku.max(Dy::ZERO));
    if lam.signum() <= 0 {
        return k0 + e * years;
    }
    let settled = e / lam;
    settled + (k0 - settled) * (-(lam * years)).exp()
}

/// Returns the continuous rate lam = -ln(1 - share) of a yearly share lost,
/// with the share clamped to [0, 0.999999].
pub fn rate(share: Dy) -> Dy {
    -(Dy::ONE - share.clamp(Dy::ZERO, Dy::ratio(999_999, 1_000_000))).log()
}

// ----------------------------------------------------------------------- Stocks.

/// A people's knowledge state: the stock per domain, the focus and the
/// readings of the last span.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stocks {
    /// Knowledge held per domain (KU).
    pub stock: ByDomain,
    /// A ruler's direction of inquiry (a seat's focus decision).
    pub focus: Option<Domain>,
    /// The people's fluid intelligence, as last read (not yet modelled).
    pub fluid: Dy,
    /// Growth rate per year of the total stock over the last span:
    /// ln((K_after + U) / (K_before + U)) / years.
    pub growth: Dy,
    /// The thought each domain got last span (KU).
    pub thought: ByDomain,
    /// The free-thought weights that needs, curiosity and focus gave each
    /// domain last span.
    pub weights: ByDomain,
    /// The people's thought per year last span (KU).
    pub effort: Effort,
    /// Share of the people's knowledge lost per year last span, in [0, 1].
    pub forgetting: Dy,
}

impl Stocks {
    /// Returns the experience in domain `d`: the stock times 1 + fluid
    /// intelligence (knowledge.experience).
    pub fn experience(&self, d: Domain) -> Dy {
        self.stock[d.index()] * (Dy::ONE + self.fluid)
    }

    /// Returns the mastery of domain `d` from its experience
    /// (knowledge.mastery).
    pub fn mastery(&self, d: Domain) -> Dy {
        mastery(self.experience(d))
    }
}

/// Every people's knowledge, indexed like `cosmic.peoples`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Knowledge {
    /// Each people's knowledge stocks, indexed like `cosmic.peoples`.
    pub of: Vec<Stocks>,
    /// Cumulative knowledge units produced, by source (`lived` and
    /// `research`).
    pub made: std::collections::BTreeMap<&'static str, Dy>,
}

impl Knowledge {
    /// Extends the stocks to at least `peoples` entries; new peoples start
    /// with empty stocks.
    pub fn sync(&mut self, peoples: usize) {
        if self.of.len() < peoples {
            self.of.resize_with(peoples, Stocks::default);
        }
    }

    /// Sets a ruler's direction of inquiry for people `who`: the focused
    /// domain's free-thought weight rises by 1 (knowledge.focus).
    pub fn focus(&mut self, who: usize, d: Option<Domain>) {
        self.sync(who + 1);
        self.of[who].focus = d;
    }

    /// Advances every people's knowledge over `years` (knowledge.quarter):
    /// each people's thought is divided among the domains, and each domain's
    /// stock advances under that thought and the people's forgetting rate.
    pub fn step(&mut self, thinkers: &[Thinkers], years: Dy) {
        self.sync(thinkers.len());
        for (i, t) in thinkers.iter().enumerate() {
            self.step_one(i, t, years);
        }
    }

    /// Advances the knowledge of people `i` over `years` and records the
    /// span's readings; a people with no population produces no thought.
    pub fn step_one(&mut self, i: usize, t: &Thinkers, years: Dy) {
        self.sync(i + 1);
        let rul = Rulers::default();
        let u = knowledge_unit();
        {
            let s = &mut self.of[i];
            s.fluid = t.fluid.clamp(Dy::ZERO, Dy::ONE);
            let before: Dy = s.stock.iter().copied().sum();
            let e = if t.population.signum() > 0 {
                effort(t, &rul)
            } else {
                Effort::default()
            };
            let (by, weights) = directed(&e, t, s.focus);
            let share = forgetting(t.life_yr, t.profile.collective);
            let lam = rate(share);
            for d in Domain::ALL {
                let k = d.index();
                let was = s.stock[k];
                s.stock[k] = advance(was, by[k], lam, years);
                s.thought[k] = by[k] * years;
            }
            s.weights = weights;
            s.effort = e;
            s.forgetting = share;
            let after: Dy = s.stock.iter().copied().sum();
            if years.signum() > 0 {
                s.growth = ((after + u) / (before + u)).log() / years;
            }
            let lived = e.lived * years;
            let free = e.free * years;
            *self.made.entry("lived").or_insert(Dy::ZERO) += lived;
            *self.made.entry("research").or_insert(Dy::ZERO) += free;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xyl_cosmos::laws::Scales;
    use xyl_cosmos::worlds::{body_temperature, Species};

    fn traits(size: Dy, solvent: Solvent, shape: Shape) -> Traits {
        let ctx = Ctx::new(Scales::default());
        Traits {
            strategy: Strategy::K,
            solvent,
            dark: false,
            at: Where::Surface,
            body_t: body_temperature(solvent, &ctx),
            sense: "light",
            form: "walking",
            hands: "grasping limbs",
            social: Dy::ratio(1, 2),
            shape,
            size,
        }
    }

    fn thinkers(numbers: Dy, t: &Traits, capacity: Dy) -> Thinkers {
        let ctx = Ctx::new(Scales::default());
        let rul = Rulers::default();
        Thinkers {
            bodies_w: numbers * growth::metabolism_w(t, &ctx, &rul),
            works_w: Dy::ZERO,
            population: numbers,
            capacity,
            body_t: t.body_t,
            impedance: Dy::ONE,
            needs: Needs::default(),
            profile: profile(t, &ctx),
            life_yr: knower_life_yr(t, &rul),
            fluid: Dy::ZERO,
            consciousness: Dy::ZERO,
        }
    }

    /// The stock a domain settles at under steady thought (KU): the fixed
    /// point e = lam K of the law in `advance`, against which it is checked.
    fn settled(e_ku: Dy, lam: Dy) -> Dy {
        if lam.signum() <= 0 {
            return Dy::ZERO;
        }
        e_ku.max(Dy::ZERO) / lam
    }

    #[test]
    fn forgetting_falls_with_life_span_and_collectivity() {
        let short = forgetting(Dy::int(5), Dy::ZERO);
        let long = forgetting(Dy::int(500), Dy::ZERO);
        assert!(short > long);
        // A collective forgets less.
        assert!(forgetting(Dy::int(70), Dy::ONE) < forgetting(Dy::int(70), Dy::ZERO));
        // Bigger bodies live longer (their generations are longer).
        let rul = Rulers::default();
        let small = traits(
            Dy::ratio(1, 10),
            Solvent::Volatile(Species::H2O),
            Shape::Society,
        );
        let big = traits(Dy::int(10), Solvent::Volatile(Species::H2O), Shape::Society);
        assert!(knower_life_yr(&big, &rul) > knower_life_yr(&small, &rul));
    }

    #[test]
    fn effort_grows_with_power_and_free_housing() {
        let rul = Rulers::default();
        let t = traits(Dy::ONE, Solvent::Volatile(Species::H2O), Shape::Society);
        // A full world gives only lived thought.
        let full = effort(&thinkers(Dy::sci(1, 9), &t, Dy::sci(1, 9)), &rul);
        assert_eq!(full.free, Dy::ZERO);
        assert!(full.lived.signum() > 0);
        // A world feeding twice its numbers frees half its bodies' power for
        // free thought.
        let half = effort(&thinkers(Dy::sci(1, 9), &t, Dy::sci(2, 9)), &rul);
        assert!(half.free > full.lived * Dy::int(5));
        // The same numbers of bigger bodies use more power and so produce more
        // thought.
        let big = traits(Dy::int(8), Solvent::Volatile(Species::H2O), Shape::Society);
        assert!(effort(&thinkers(Dy::sci(1, 9), &big, Dy::sci(1, 9)), &rul).lived > full.lived);
        // A cold chemistry has a slower metabolism but more bits per joule
        // (Landauer), which offsets the slower pace: the same thought per
        // body of equal mass.
        let cold = traits(Dy::ONE, Solvent::Volatile(Species::CH4), Shape::Society);
        let c = effort(&thinkers(Dy::sci(1, 9), &cold, Dy::sci(1, 9)), &rul).lived;
        assert!(
            (c / full.lived - Dy::ONE).abs() < Dy::ratio(1, 100),
            "cold/warm {}",
            c / full.lived
        );
        // The closed form: one span and the same span in a hundred pieces
        // agree to the grid, and the stock settles at the fixed point.
        let lam = rate(Dy::ratio(15, 100));
        let coarse = advance(Dy::ZERO, Dy::sci(1, 6), lam, Dy::int(100));
        let mut fine = Dy::ZERO;
        for _ in 0..100 {
            fine = advance(fine, Dy::sci(1, 6), lam, Dy::ONE);
        }
        assert!(
            (coarse / fine - Dy::ONE).abs() < Dy::sci(1, -9),
            "coarse {coarse} fine {fine}"
        );
        let far = advance(Dy::ZERO, Dy::sci(1, 6), lam, Dy::int(100_000));
        assert!((far / settled(Dy::sci(1, 6), lam) - Dy::ONE).abs() < Dy::sci(1, -9));
    }

    #[test]
    fn learning_follows_needs_and_focus() {
        let t = traits(Dy::ONE, Solvent::Volatile(Species::H2O), Shape::Society);
        let mut th = thinkers(Dy::sci(1, 9), &t, Dy::sci(2, 9));
        th.needs.hunger = Dy::ONE;
        let mut k = Knowledge::default();
        k.step(&[th.clone()], Dy::int(10));
        let s = &k.of[0];
        assert!(s.stock[Domain::Biology.index()] > s.stock[Domain::Materials.index()]);
        // A focus moves the stock toward the focused domain.
        let mut k2 = Knowledge::default();
        k2.focus(0, Some(Domain::Materials));
        th.needs = Needs::default();
        k2.step(&[th], Dy::int(10));
        assert!(
            k2.of[0].stock[Domain::Materials.index()] > k2.of[0].stock[Domain::Biology.index()]
        );
        assert!(k2.of[0].growth.signum() > 0);
    }
}
