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
use xyl_core::Q;
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

/// A ruler's direction of inquiry: an exact share of the people's free
/// thought for each domain (energy, materials, biology, information), each
/// in `[0, 1]` and together at most 1, read whole and never as a winner. All
/// zero lets the inquiry follow the people's needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Focus {
    shares: [Q; 4],
}

impl Default for Focus {
    fn default() -> Focus {
        Focus::none()
    }
}

impl Focus {
    /// The focus of `shares`, in domain order; `None` where a share lies
    /// outside `[0, 1]` or the shares sum past 1.
    pub fn of(shares: [Q; 4]) -> Option<Focus> {
        let sum = shares.iter().fold(Q::ZERO, |a, s| &a + s);
        let each = shares.iter().all(|s| s.signum() >= 0 && *s <= Q::ONE);
        (each && sum <= Q::ONE).then_some(Focus { shares })
    }

    /// No direction: the inquiry follows the people's needs.
    pub fn none() -> Focus {
        Focus {
            shares: std::array::from_fn(|_| Q::ZERO),
        }
    }

    /// The shares, in domain order.
    pub fn shares(&self) -> &[Q; 4] {
        &self.shares
    }

    /// The domains the focus directs, each with its share, in domain order
    /// (`materials 3/4, biology 1/8`); empty where it directs nothing.
    pub fn described(&self) -> String {
        Domain::ALL
            .iter()
            .zip(&self.shares)
            .filter(|(_, s)| s.signum() > 0)
            .map(|(d, s)| format!("{} {s}", d.name()))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Whether the focus directs nothing.
    pub fn is_none(&self) -> bool {
        self.shares.iter().all(|s| s.signum() == 0)
    }

    /// Written as the four shares in domain order, joined by commas.
    pub fn written(&self) -> String {
        self.shares
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    /// The focus [`Focus::written`] wrote; `None` where it does not parse or
    /// is not a focus.
    pub fn parse(text: &str) -> Option<Focus> {
        let parts: Vec<Q> = text
            .split(',')
            .map(crate::checkpoint::q_of)
            .collect::<Option<_>>()?;
        Focus::of(parts.try_into().ok()?)
    }
}

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
    xyl_core::dyadic!(1, 20)
}

/// The floor of undirected inquiry: the weight every domain receives for free
/// thought before needs and focus. Declared ruler.
pub fn curiosity() -> Dy {
    xyl_core::dyadic!(1, 4)
}

/// The share of forgetting that pooling spares a fully collective people
/// (collective = 1). Declared ruler.
pub fn pooled_keep() -> Dy {
    xyl_core::dyadic!(1, 2)
}

// ---------------------------------------------------------- Experience and cost.

/// Returns the cost multiplier after `experience` x (in KU):
/// 1 / (1 + ln(1 + x / U)) with U = [`knowledge_unit`], which is 1 with no
/// experience and falls toward 0; negative experience is read as zero.
pub fn wright(experience: Dy) -> Dy {
    (Dy::ONE)
        .div(
            &(Dy::ONE
                + (Dy::ONE
                    + experience
                        .max(Dy::ZERO)
                        .div(&knowledge_unit())
                        .unwrap_or_default())
                .log()),
        )
        .unwrap_or_default()
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
    t_unit().div(&t.max(Dy::ONE)).unwrap_or_default()
}

/// The power equivalent of one knowledge unit (W): the metabolism of
/// `KU_MINDS` unit bodies (`life::unit_metabolism_w`, xyl-cosmos).
fn ku_body_w() -> Dy {
    Dy::int(KU_MINDS) * xyl_cosmos::life::unit_metabolism_w()
}

// -------------------------------------------------- Mind profile (mind.profile).

/// The weights by which a life's primary sense directs its sciences, in
/// domain order (energy, materials, biology, information); each row sums to
/// 2. Declared rulers.
fn sense_weights(sense: &str) -> ByDomain {
    match sense {
        "chemistry" => [
            xyl_core::dyadic!(1, 8),
            xyl_core::dyadic!(3, 4),
            Dy::ONE,
            xyl_core::dyadic!(1, 8),
        ],
        "heat and pressure" => [
            xyl_core::dyadic!(3, 4),
            xyl_core::dyadic!(3, 4),
            xyl_core::dyadic!(3, 8),
            xyl_core::dyadic!(1, 8),
        ],
        "heat" => [
            Dy::ONE,
            xyl_core::dyadic!(3, 4),
            xyl_core::dyadic!(1, 8),
            xyl_core::dyadic!(1, 8),
        ],
        "fields" => [
            Dy::ONE,
            xyl_core::dyadic!(1, 8),
            xyl_core::dyadic!(1, 8),
            xyl_core::dyadic!(3, 4),
        ],
        _ => [
            xyl_core::dyadic!(1, 2),
            xyl_core::dyadic!(1, 4),
            xyl_core::dyadic!(1, 4),
            Dy::ONE,
        ],
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
        Shape::Superorganism => xyl_core::dyadic!(3, 2),
        Shape::Distributed => xyl_core::dyadic!(3, 4),
        Shape::Society => xyl_core::dyadic!(1, 2),
    };
    let collective = (t.social * shape_w).min(Dy::ONE);
    let mut distributed = Dy::ZERO;
    if matches!(t.shape, Shape::Distributed | Shape::Superorganism) {
        distributed += xyl_core::dyadic!(3, 5);
    }
    if matches!(t.form, "swimming" | "drifting" | "low and many-limbed") {
        distributed += xyl_core::dyadic!(2, 5);
    }
    let distributed = distributed.min(Dy::ONE);
    let plastic = ((if aquatic {
        xyl_core::dyadic!(1, 2)
    } else {
        xyl_core::dyadic!(1, 10)
    }) + (if t.strategy == Strategy::R {
        xyl_core::dyadic!(3, 10)
    } else {
        Dy::ZERO
    }) + xyl_core::dyadic!(1, 5) * distributed)
        .min(Dy::ONE);
    let legible = if t.solvent == Solvent::Plasma {
        Dy::ZERO
    } else {
        let eps0 = xyl_cosmos::life::fidelity(t.solvent, t.body_t, ctx);
        (((xyl_cosmos::life::bits_per_monomer())
            .div(&eps0.max(Dy::sci(1, -300)))
            .unwrap_or_default())
        .log()
        .div(&(xyl_cosmos::life::i_mind().log()))
        .unwrap_or_default())
        .clamp(Dy::ZERO, Dy::ONE)
    };
    let w = sense_weights(t.sense);
    let total: Dy = w.iter().copied().sum();
    let affinity = w.map(|x| {
        (Dy::int(4) * x)
            .div(&total.max(Dy::sci(1, -9)))
            .unwrap_or_default()
    });
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

/// Returns the yearly share of a people's knowledge that is lost
/// (knowledge.forgetting): (1 - e^(-1 / life)) x (1 - pooled_keep x
/// collective), with knowers dying at the rate 1 / life and the loss reduced
/// for a collective.
pub fn forgetting(life_yr: Dy, collective: Dy) -> Dy {
    let keep = (-((Dy::ONE)
        .div(&life_yr.max(xyl_core::dyadic!(1, 1000)))
        .unwrap_or_default()))
    .exp();
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
            Dy::ONE - p.capacity.div(&p.population).unwrap_or_default()
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
            life_yr: growth::generation_yr(&p.traits, &rul),
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
pub fn effort(t: &Thinkers) -> Effort {
    let depth = landauer(t.body_t)
        * t.impedance.clamp(Dy::ZERO, Dy::ONE)
        * (Dy::ONE + t.consciousness.clamp(Dy::ZERO, Dy::ONE));
    let per_ku = ku_body_w();
    let lived =
        (t.bodies_w.max(Dy::ZERO) * lived_share() * (Dy::ONE + t.profile.collective) * depth)
            .div(&per_ku)
            .unwrap_or_default();
    let free_share = crate::civ::housing::free_share(t.population, t.capacity);
    let free = ((t.bodies_w.max(Dy::ZERO) * free_share + t.works_w.max(Dy::ZERO)) * depth)
        .div(&per_ku)
        .unwrap_or_default();
    Effort { lived, free }
}

/// Returns how a span's thought divides among the domains, and the free
/// thought's weights: lived thought by affinity, free thought by curiosity +
/// pull + the focus's share of each domain (knowledge.quarter's weights).
pub fn directed(e: &Effort, t: &Thinkers, focus: &Focus) -> (ByDomain, ByDomain) {
    let pull = t.needs.pulls();
    let weights: ByDomain =
        std::array::from_fn(|i| curiosity() + pull[i].max(Dy::ZERO) + Dy::from_q(&focus.shares[i]));
    let wsum: Dy = weights.iter().copied().sum();
    let asum: Dy = t.profile.affinity.iter().copied().sum();
    let out = std::array::from_fn(|i| {
        (e.lived * t.profile.affinity[i])
            .div(&asum.max(Dy::sci(1, -9)))
            .unwrap_or_default()
            + (e.free * weights[i])
                .div(&wsum.max(Dy::sci(1, -9)))
                .unwrap_or_default()
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
    let settled = e.div(&lam).unwrap_or_default();
    settled + (k0 - settled) * (-(lam * years)).exp()
}

/// Returns the continuous rate lam = -ln(1 - share) of a yearly share lost,
/// with the share clamped to [0, 0.999999].
pub fn rate(share: Dy) -> Dy {
    -(Dy::ONE - share.clamp(Dy::ZERO, xyl_core::dyadic!(999_999, 1_000_000))).log()
}

// ----------------------------------------------------------------------- Stocks.

/// A people's knowledge state: the stock per domain, the focus and the
/// readings of the last span.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stocks {
    /// Knowledge held per domain (KU).
    pub stock: ByDomain,
    /// A ruler's direction of inquiry (a seat's focus decision).
    pub focus: Focus,
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

    /// Sets a ruler's direction of inquiry for people `who`: each domain's
    /// free-thought weight rises by the focus's share of it
    /// (knowledge.focus).
    pub fn focus(&mut self, who: usize, focus: Focus) {
        self.sync(who + 1);
        self.of[who].focus = focus;
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
        let u = knowledge_unit();
        {
            let s = &mut self.of[i];
            s.fluid = t.fluid.clamp(Dy::ZERO, Dy::ONE);
            let before: Dy = s.stock.iter().copied().sum();
            let e = if t.population.signum() > 0 {
                effort(t)
            } else {
                Effort::default()
            };
            let (by, weights) = directed(&e, t, &s.focus);
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
                s.growth = ((after + u).div(&(before + u)).unwrap_or_default())
                    .log()
                    .div(&years)
                    .unwrap_or_default();
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
            social: xyl_core::dyadic!(1, 2),
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
            life_yr: growth::generation_yr(t, &rul),
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
        e_ku.max(Dy::ZERO).div(&lam).unwrap_or_default()
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
            xyl_core::dyadic!(1, 10),
            Solvent::Volatile(Species::H2O),
            Shape::Society,
        );
        let big = traits(Dy::int(10), Solvent::Volatile(Species::H2O), Shape::Society);
        assert!(growth::generation_yr(&big, &rul) > growth::generation_yr(&small, &rul));
    }

    #[test]
    fn effort_grows_with_power_and_free_housing() {
        let t = traits(Dy::ONE, Solvent::Volatile(Species::H2O), Shape::Society);
        // A full world gives only lived thought.
        let full = effort(&thinkers(Dy::sci(1, 9), &t, Dy::sci(1, 9)));
        assert_eq!(full.free, Dy::ZERO);
        assert!(full.lived.signum() > 0);
        // A world feeding twice its numbers frees half its bodies' power for
        // free thought.
        let half = effort(&thinkers(Dy::sci(1, 9), &t, Dy::sci(2, 9)));
        assert!(half.free > full.lived * Dy::int(5));
        // The same numbers of bigger bodies use more power and so produce more
        // thought.
        let big = traits(Dy::int(8), Solvent::Volatile(Species::H2O), Shape::Society);
        assert!(effort(&thinkers(Dy::sci(1, 9), &big, Dy::sci(1, 9))).lived > full.lived);
        // A cold chemistry has a slower metabolism but more bits per joule
        // (Landauer), which offsets the slower pace: the same thought per
        // body of equal mass.
        let cold = traits(Dy::ONE, Solvent::Volatile(Species::CH4), Shape::Society);
        let c = effort(&thinkers(Dy::sci(1, 9), &cold, Dy::sci(1, 9))).lived;
        assert!(
            (c.div(&full.lived).unwrap_or_default() - Dy::ONE).abs() < xyl_core::dyadic!(1, 100),
            "cold/warm {}",
            c.div(&full.lived).unwrap_or_default()
        );
        // The closed form: one span and the same span in a hundred pieces
        // agree to the grid, and the stock settles at the fixed point.
        let lam = rate(xyl_core::dyadic!(15, 100));
        let coarse = advance(Dy::ZERO, Dy::sci(1, 6), lam, Dy::int(100));
        let mut fine = Dy::ZERO;
        for _ in 0..100 {
            fine = advance(fine, Dy::sci(1, 6), lam, Dy::ONE);
        }
        assert!(
            (coarse.div(&fine).unwrap_or_default() - Dy::ONE).abs() < Dy::sci(1, -9),
            "coarse {coarse} fine {fine}"
        );
        let far = advance(Dy::ZERO, Dy::sci(1, 6), lam, Dy::int(100_000));
        assert!(
            (far.div(&settled(Dy::sci(1, 6), lam)).unwrap_or_default() - Dy::ONE).abs()
                < Dy::sci(1, -9)
        );
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
        k2.focus(0, Focus::of([Q::ZERO, Q::ONE, Q::ZERO, Q::ZERO]).unwrap());
        th.needs = Needs::default();
        k2.step(&[th], Dy::int(10));
        assert!(
            k2.of[0].stock[Domain::Materials.index()] > k2.of[0].stock[Domain::Biology.index()]
        );
        assert!(k2.of[0].growth.signum() > 0);
    }

    /// A focus holds four shares, each in [0, 1] and together at most 1;
    /// anything else is refused, and the written form reads back whole.
    #[test]
    fn focus_refuses_shares_out_of_bounds_and_round_trips() {
        let f = Focus::of([
            xyl_core::ratio!(1, 2),
            Q::ZERO,
            xyl_core::ratio!(1, 3),
            xyl_core::ratio!(1, 6),
        ])
        .unwrap();
        assert_eq!(Focus::parse(&f.written()), Some(f.clone()));
        assert!(!f.is_none() && Focus::none().is_none());
        assert_eq!(
            Focus::of([
                xyl_core::ratio!(3, 4),
                xyl_core::ratio!(1, 2),
                Q::ZERO,
                Q::ZERO
            ]),
            None
        );
        assert_eq!(
            Focus::of([xyl_core::ratio!(-1, 4), Q::ZERO, Q::ZERO, Q::ZERO]),
            None
        );
        assert_eq!(Focus::parse("1/2,1/2"), None);
    }
}
