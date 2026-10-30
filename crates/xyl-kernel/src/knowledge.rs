//! Knowledge: what a people knows, how it learns, and what that lets it do
//! There is no tech tree.
//!
//! Knowledge is a stock per people in four domains (energy, materials,
//! biology, information), in knowledge units (KU: the free thought of a
//! thousand unit minds for a year). Mastery is the logarithmic form
//! mastery = 1 - 1 / (1 + ln(1 + K / U)), U the experience of the first
//! doubling (`wright`). Mastery depends on the
//! power a people commands, not its headcount, and never reaches one.
//!
//! - Thought is power: a people thinks with the power it commands, its
//!   bodies' metabolism (growth.rs, at its chemistry's pace) plus what its
//!   works turn to thought. A watt at temperature T erases Landauer's
//!   1 / (k T ln 2) bits; kinetic speed is already in the power, so only
//!   Landauer's T_unit / T is applied. Free thought is the share of bodies
//!   food no longer needs (1 - N / N_cap) plus the works' thought; every member
//!   also adds a lived share.
//! - Discovery: the frontier yields at the rate of the thought spent on it,
//!   dK/dt = E - lam K, solved in closed form (`advance`).
//! - Forgetting follows the people's own life: knowers die once a
//!   generation (growth.rs), and what they knew goes with them; a collective
//!   forgets less (`pooled_keep`).
//!
//! Thought is directed by the people's needs (pulls): hunger to biology, a
//! spent crust to materials, a power shortfall to energy, trade to
//! information, over a floor of curiosity, plus any focus its rulers set.

use crate::cosmic::{People, Shape, Strategy, Traits};
use crate::growth::{self, Rulers};
use xyl_core::dyadic::Dy;
use xyl_cosmos::worlds::{t_unit, Ctx, Solvent, Where};

fn n(x: i64) -> Dy {
    Dy::int(x)
}

fn r(a: i64, b: i64) -> Dy {
    Dy::ratio(a, b)
}

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

    /// Position of the domain in a [`ByDomain`] array (0 energy, 1 materials, 2 biology, 3 information).
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

// ---------------------------------------------------------------------- constants

/// The experience of the first doubling, in KU (KNOWLEDGE_UNIT =
/// 10: ten thousand unit minds' year). Declared ruler.
pub fn knowledge_unit() -> Dy {
    n(10)
}

/// Unit minds a knowledge unit is the year's free thought of
/// (a thousand minds' year). Declared ruler.
pub const KU_MINDS: i64 = 1000;

/// The share of every member's thought its living adds to what its people
/// knows. Declared ruler.
pub fn lived_share() -> Dy {
    r(1, 20)
}

/// The floor of undirected inquiry, as a share of one domain's pull
/// (its curiosity). Declared ruler.
pub fn curiosity() -> Dy {
    r(1, 4)
}

/// How much of a collective's forgetting its pooling spares. Declared ruler.
pub fn pooled_keep() -> Dy {
    r(1, 2)
}

// ---------------------------------------------------------------------- the laws

/// Cost multiplier after `experience` (in KU): 1 with none, falling one
/// e-fold for each e-fold of experience, 1 / (1 + ln(1 + x / U)).
pub fn wright(experience: Dy) -> Dy {
    Dy::ONE / (Dy::ONE + (Dy::ONE + experience.max(Dy::ZERO) / knowledge_unit()).log())
}

/// How far experience has carried a capability, 0 .. 1 (laws.mastery).
pub fn mastery(experience: Dy) -> Dy {
    // zero experience is exactly zero mastery (avoids rounding at zero)
    if experience.signum() <= 0 {
        return Dy::ZERO;
    }
    Dy::ONE - wright(experience)
}

/// Bits a joule erases at `t` relative to the unit temperature (Landauer).
pub fn landauer(t: Dy) -> Dy {
    t_unit() / t.max(Dy::ONE)
}

/// A unit body's metabolism (W): Kleiber at the mind-bearing body's mass and
/// the unit pace.
pub fn unit_body_w(_rul: &Rulers) -> Dy {
    crate::growth::unit_metabolism_w()
}

/// A knowledge unit's power (W): the free thought of KU_MINDS unit bodies.
fn ku_body_w(rul: &Rulers) -> Dy {
    n(KU_MINDS) * unit_body_w(rul)
}

// ---------------------------------------------------------------------- mind.profile

/// The sciences a life's senses touch first (weights
/// per sense, each row summing to 2; energy, materials, biology, information).
/// Declared rulers.
fn sense_weights(sense: &str) -> ByDomain {
    match sense {
        "chemistry" => [r(1, 8), r(3, 4), Dy::ONE, r(1, 8)],
        "heat and pressure" => [r(3, 4), r(3, 4), r(3, 8), r(1, 8)],
        "heat" => [Dy::ONE, r(3, 4), r(1, 8), r(1, 8)],
        "fields" => [Dy::ONE, r(1, 8), r(1, 8), r(3, 4)],
        _ => [r(1, 2), r(1, 4), r(1, 4), Dy::ONE],
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
    /// How readable its own heredity is.
    pub legible: Dy,
    /// Whether it lives in water or the dark (swimming or drifting form, dark habitat, or under ice): no fire, so energy science comes late.
    pub aquatic: bool,
    /// The sciences its life touches first, averaging 1.
    pub affinity: ByDomain,
}

/// A people's profile from its traits (mind.profile). Declared weights:
/// collective = social x (1.5 superorganism, 0.75 distributed, 0.5
/// society); distributed 0.6 for a colony or a spread mind, 0.4 more for a
/// swimmer's or many-limbed body; plastic 0.5 aquatic (else 0.1), 0.3 more for
/// an r-line, 0.2 x distributed; legible = ln(b / eps0) / ln(I_MIND), its
/// chemistry's copying fidelity at its body's temperature (life.rs).
pub fn profile(t: &Traits, ctx: &Ctx) -> Profile {
    let aquatic = matches!(t.form, "swimming" | "drifting") || t.dark || t.at == Where::UnderIce;
    let shape_w = match t.shape {
        Shape::Superorganism => r(3, 2),
        Shape::Distributed => r(3, 4),
        Shape::Society => r(1, 2),
    };
    let collective = (t.social * shape_w).min(Dy::ONE);
    let mut distributed = Dy::ZERO;
    if matches!(t.shape, Shape::Distributed | Shape::Superorganism) {
        distributed += r(3, 5);
    }
    if matches!(t.form, "swimming" | "drifting" | "low and many-limbed") {
        distributed += r(2, 5);
    }
    let distributed = distributed.min(Dy::ONE);
    let plastic = ((if aquatic { r(1, 2) } else { r(1, 10) })
        + (if t.strategy == Strategy::R {
            r(3, 10)
        } else {
            Dy::ZERO
        })
        + r(1, 5) * distributed)
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
    let affinity = w.map(|x| n(4) * x / total.max(Dy::sci(1, -9)));
    Profile {
        collective,
        distributed,
        plastic,
        legible,
        aquatic,
        affinity,
    }
}

// ---------------------------------------------------------------------- life and forgetting

/// How long a knower lives (standard years): one of its people's
/// generations (growth.rs).
pub fn knower_life_yr(traits: &Traits, rul: &Rulers) -> Dy {
    growth::generation_yr(traits, rul)
}

/// The yearly share of a people's knowledge that is lost
/// (knowledge.forgetting): knowers dying at 1 / life, reduced for a
/// collective.
pub fn forgetting(life_yr: Dy, collective: Dy) -> Dy {
    let keep = (-(Dy::ONE / life_yr.max(r(1, 1000)))).exp();
    let pooled = Dy::ONE - pooled_keep() * collective.clamp(Dy::ZERO, Dy::ONE);
    ((Dy::ONE - keep) * pooled).clamp(Dy::ZERO, Dy::ONE)
}

// ---------------------------------------------------------------------- where its mind goes

/// What a people's troubles ask of its thinkers (knowledge.pulls), each >= 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Needs {
    /// Hunger: its numbers over what its world feeds (1 - N_cap / N, above).
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
    /// What a people's numbers alone say: hunger when it outgrows its world.
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

    /// The pull on each domain.
    pub fn pulls(&self) -> ByDomain {
        [
            self.power_short,
            self.crust_spent,
            self.hunger + self.births,
            self.trade.min(Dy::ONE),
        ]
    }
}

// ---------------------------------------------------------------------- effort

/// What a people thinks with this span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thinkers {
    /// Its bodies' metabolism, all of them (W; growth.rs Power.bodies_w).
    pub bodies_w: Dy,
    /// Power its works turn to thought (W; zero until it captures flows and
    /// builds machines that think; not yet modelled).
    pub works_w: Dy,
    /// Current numbers (bodies).
    pub population: Dy,
    /// Numbers its world feeds (bodies); sets the free share 1 - population / capacity.
    pub capacity: Dy,
    /// The temperature its minds run at (K).
    pub body_t: Dy,
    /// 0 .. 1 (impedance()).
    pub impedance: Dy,
    /// What its troubles ask of its thinkers, directing free thought.
    pub needs: Needs,
    /// Its kind of mind, which weights lived thought by affinity and pooling.
    pub profile: Profile,
    /// Its life expectancy (years).
    pub life_yr: Dy,
    /// Its fluid intelligence, 0 .. 1 (not yet modelled, read as 0).
    pub fluid: Dy,
    /// Its consciousness gauge as last read, 0 .. 1 (civ.rs `Gauges`): its
    /// thought's factor F = 1 + g_C.
    pub consciousness: Dy,
}

impl Thinkers {
    /// A people's thinkers from its numbers and the power its bodies turn
    /// (cosmic.rs Cosmic::power), with no works, no impedance and no fluid yet.
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
    /// What its members add by living, in the sciences their life touches.
    pub lived: Dy,
    /// Its free thought: the hands its food frees, and its works.
    pub free: Dy,
}

impl Effort {
    /// Lived plus free thought (KU a year).
    pub fn total(&self) -> Dy {
        self.lived + self.free
    }
}

/// A people's thought per year (KU): power x Landauer at its minds'
/// temperature, over a knowledge unit's power, scaled by consciousness
/// F = 1 + g_C. The bodies' free share is 1 - N / N_cap (zero when short);
/// works think at the minds' temperature, each watt against a unit mind's
/// watt (`mind_share`).
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

/// How a span's thought divides among the domains: lived by affinity, free
/// by curiosity + pull + focus (knowledge.quarter's weights).
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

// ---------------------------------------------------------------------- the frontier

/// A domain's stock carried over `years` under thought `e_ku` a year and a
/// forgetting rate `lam` (per year, continuous): dK/dt = e - lam K, the
/// frontier yielding at the rate of the thought spent on it. Solved in closed
/// form, K(t) = e / lam + (K0 - e / lam) exp(-lam t), and K0 + e t where
/// nothing is forgotten; a long span settles at e / lam.
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

/// The continuous rate of a yearly share lost.
pub fn rate(share: Dy) -> Dy {
    -(Dy::ONE - share.clamp(Dy::ZERO, r(999_999, 1_000_000))).log()
}

// ---------------------------------------------------------------------- the stocks

/// What a people knows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stocks {
    /// Knowledge held per domain (KU).
    pub stock: ByDomain,
    /// A ruler's direction of inquiry (a seat's focus decision).
    pub focus: Option<Domain>,
    /// Its fluid intelligence, as last read (not yet modelled).
    pub fluid: Dy,
    /// Its knowledge's growth rate a year.
    pub growth: Dy,
    /// The thought each domain got last span (KU).
    pub thought: ByDomain,
    /// The weights its needs, curiosity and focus gave each domain last span.
    pub weights: ByDomain,
    /// Its thought a year last span (KU).
    pub effort: Effort,
    /// Share of its knowledge lost a year last span, in [0, 1].
    pub forgetting: Dy,
}

impl Stocks {
    /// What its knowledge is worth in practice: its stock, carried further by
    /// its fluid intelligence (knowledge.experience).
    pub fn experience(&self, d: Domain) -> Dy {
        self.stock[d.index()] * (Dy::ONE + self.fluid)
    }

    /// How far it has learned what a domain lets it do (knowledge.mastery).
    pub fn mastery(&self, d: Domain) -> Dy {
        mastery(self.experience(d))
    }
}

/// Every people's knowledge, indexed like `cosmic.peoples`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Knowledge {
    /// Each people's knowledge stocks, indexed like `cosmic.peoples`.
    pub of: Vec<Stocks>,
    /// Units made, by why.
    pub made: std::collections::BTreeMap<&'static str, Dy>,
}

impl Knowledge {
    /// Room for every people (new peoples know nothing).
    pub fn sync(&mut self, peoples: usize) {
        if self.of.len() < peoples {
            self.of.resize_with(peoples, Stocks::default);
        }
    }

    /// A ruler's direction of inquiry: the domain gets a full share of pull
    /// more (knowledge.focus).
    pub fn focus(&mut self, who: usize, d: Option<Domain>) {
        self.sync(who + 1);
        self.of[who].focus = d;
    }

    /// Every people thinks for `years` (knowledge.quarter): its thought
    /// divides among the domains, the frontier yields what it yields, and it
    /// forgets at its own rate.
    pub fn step(&mut self, thinkers: &[Thinkers], years: Dy) {
        self.sync(thinkers.len());
        for (i, t) in thinkers.iter().enumerate() {
            self.step_one(i, t, years);
        }
    }

    /// Carries one people's knowledge across its own `years`.
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
            social: r(1, 2),
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

    /// The stock a domain settles at under steady thought (KU): e = lam K,
    /// the law's own fixed point, to check the law against.
    fn settled(e_ku: Dy, lam: Dy) -> Dy {
        if lam.signum() <= 0 {
            return Dy::ZERO;
        }
        e_ku.max(Dy::ZERO) / lam
    }

    #[test]
    fn a_long_life_holds_what_it_learns_and_a_short_one_relearns() {
        let short = forgetting(n(5), Dy::ZERO);
        let long = forgetting(n(500), Dy::ZERO);
        assert!(short > long);
        // a collective forgets less
        assert!(forgetting(n(70), Dy::ONE) < forgetting(n(70), Dy::ZERO));
        // bigger bodies live longer (their generations are longer)
        let rul = Rulers::default();
        let small = traits(r(1, 10), Solvent::Volatile(Species::H2O), Shape::Society);
        let big = traits(n(10), Solvent::Volatile(Species::H2O), Shape::Society);
        assert!(knower_life_yr(&big, &rul) > knower_life_yr(&small, &rul));
    }

    #[test]
    fn thought_is_power_and_minds_think_where_their_world_frees_them() {
        let rul = Rulers::default();
        let t = traits(Dy::ONE, Solvent::Volatile(Species::H2O), Shape::Society);
        // a full world: only the lived share
        let full = effort(&thinkers(Dy::sci(1, 9), &t, Dy::sci(1, 9)), &rul);
        assert_eq!(full.free, Dy::ZERO);
        assert!(full.lived.signum() > 0);
        // a world feeding twice its numbers frees half its hands
        let half = effort(&thinkers(Dy::sci(1, 9), &t, Dy::sci(2, 9)), &rul);
        assert!(half.free > full.lived * n(5));
        // the same numbers of bigger bodies command more power and think more
        let big = traits(n(8), Solvent::Volatile(Species::H2O), Shape::Society);
        assert!(effort(&thinkers(Dy::sci(1, 9), &big, Dy::sci(1, 9)), &rul).lived > full.lived);
        // a cold chemistry: slower metabolism, more bits a joule - Landauer
        // returns what the pace takes, so a cold people thinks as much per
        // body of its mass
        let cold = traits(Dy::ONE, Solvent::Volatile(Species::CH4), Shape::Society);
        let c = effort(&thinkers(Dy::sci(1, 9), &cold, Dy::sci(1, 9)), &rul).lived;
        assert!(
            (c / full.lived - Dy::ONE).abs() < r(1, 100),
            "cold/warm {}",
            c / full.lived
        );
        // the closed form: one span and the same span in a hundred pieces
        // agree to the grid, and the stock settles at its fixed point
        let lam = rate(r(15, 100));
        let coarse = advance(Dy::ZERO, Dy::sci(1, 6), lam, n(100));
        let mut fine = Dy::ZERO;
        for _ in 0..100 {
            fine = advance(fine, Dy::sci(1, 6), lam, Dy::ONE);
        }
        assert!(
            (coarse / fine - Dy::ONE).abs() < Dy::sci(1, -9),
            "coarse {coarse} fine {fine}"
        );
        let far = advance(Dy::ZERO, Dy::sci(1, 6), lam, n(100_000));
        assert!((far / settled(Dy::sci(1, 6), lam) - Dy::ONE).abs() < Dy::sci(1, -9));
    }

    #[test]
    fn a_people_learns_where_its_troubles_are() {
        let t = traits(Dy::ONE, Solvent::Volatile(Species::H2O), Shape::Society);
        let mut th = thinkers(Dy::sci(1, 9), &t, Dy::sci(2, 9));
        th.needs.hunger = Dy::ONE;
        let mut k = Knowledge::default();
        k.step(&[th.clone()], n(10));
        let s = &k.of[0];
        assert!(s.stock[Domain::Biology.index()] > s.stock[Domain::Materials.index()]);
        // a focus moves it
        let mut k2 = Knowledge::default();
        k2.focus(0, Some(Domain::Materials));
        th.needs = Needs::default();
        k2.step(&[th], n(10));
        assert!(
            k2.of[0].stock[Domain::Materials.index()] > k2.of[0].stock[Domain::Biology.index()]
        );
        assert!(k2.of[0].growth.signum() > 0);
    }
}
