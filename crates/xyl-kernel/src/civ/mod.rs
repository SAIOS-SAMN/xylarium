//! Civilizations: every layer of a people's life, carried together on the
//! universal clock in the order each reads the others (docs/civilizations.md).
//! Every subsystem runs for every woken people, whether observed or not.
//!
//! One carry per span, in order:
//! - Mind (knowledge.rs): each people thinks with the power it commands; every
//!   member adds a lived share (a people at its land's limit still learns),
//!   and it forgets at its own life's rate. Biology mastery feeds its harvest
//!   and synthesis (housing.rs). A ruler may direct its inquiry
//!   (`focus`).
//! - World: the world at the span's end, and a fall or exodus inside the span.
//! - Numbers: the logistic toward what its world houses ([`housing`]), at one
//!   e-fold per its own generation, scaled by its radiation environment; a
//!   fall when the world no longer feeds it.
//! - Gauges: instinct, intelligence, sentience, consciousness and the combined
//!   gauge, read from the layers after every carry. Each is a multiplier, never
//!   a requirement or a latched state; consciousness scales the next span's
//!   thought (F = 1 + g_C).

pub mod history;
pub mod housing;

use crate::cosmic::{Annal, Cosmic, SystemReader};
use crate::growth::Rulers;
use crate::knowledge::{Domain, Knowledge, Stocks, Thinkers};
use history::{Eminent, Held, Turning};
use housing::houses;
use xyl_core::dyadic::Dy;
use xyl_core::Stream;

/// A people's state beyond its numbers: what its world houses, its settlers
/// and its history.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Numbers {
    /// What its world housed at its last carry (bodies).
    pub housing: Dy,
    /// Daughter peoples it has founded.
    pub daughters: u32,
    /// Bodies it has sent out as settlers, in total.
    pub emigrants: Dy,
    /// Its remembered figures, best remembered first.
    pub eminent: Vec<Eminent>,
    /// Turnings reported since its last carry, not yet named.
    pub candidates: Vec<Turning>,
}

/// A people's four gauges and their combination, each a share in [0, 1]:
/// - instinct: how far its world houses it, housing / (housing + numbers);
/// - intelligence: mean mastery of energy, materials and biology (knowledge.rs);
/// - sentience: mastery of information (the science of signals);
/// - consciousness: the share of its world's housing its numbers leave free,
///   1 - numbers / housing;
/// - gamma = [(1 + g_I)(1 + g_N)(1 + g_S)(1 + g_C) - 1] / 15.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gauges {
    /// How far its world houses it, a share in [0, 1]: housing / (housing + numbers).
    pub instinct: Dy,
    /// Mean mastery of energy, materials and biology, a share in [0, 1].
    pub intelligence: Dy,
    /// Mastery of information, a share in [0, 1].
    pub sentience: Dy,
    /// The share of its world's housing its numbers leave free, in [0, 1].
    pub consciousness: Dy,
    /// Combined gauge in [0, 1]: \[(1 + instinct)(1 + intelligence)(1 + sentience)(1 + consciousness) - 1\] / 15.
    pub gamma: Dy,
}

fn share(a: Dy, b: Dy) -> Dy {
    let both = a + b;
    if both.signum() > 0 {
        a / both
    } else {
        Dy::ZERO
    }
}

impl Gauges {
    /// Reads the gauges from a people's `numbers`, what its world houses and
    /// its knowledge stocks `k`.
    pub fn read(numbers: Dy, housing: Dy, k: &Stocks) -> Gauges {
        let instinct = share(housing, numbers);
        let intelligence =
            (k.mastery(Domain::Energy) + k.mastery(Domain::Materials) + k.mastery(Domain::Biology))
                / Dy::int(3);
        let sentience = k.mastery(Domain::Information);
        let consciousness = if housing.signum() > 0 {
            (Dy::ONE - numbers / housing).clamp(Dy::ZERO, Dy::ONE)
        } else {
            Dy::ZERO
        };
        let gamma = ((Dy::ONE + instinct)
            * (Dy::ONE + intelligence)
            * (Dy::ONE + sentience)
            * (Dy::ONE + consciousness)
            - Dy::ONE)
            / Dy::int(15);
        Gauges {
            instinct,
            intelligence,
            sentience,
            consciousness,
            gamma,
        }
    }
}

/// Every people's layers, indexed like `cosmic.peoples`.
#[derive(Clone, Debug, Default)]
pub struct Civilizations {
    /// Each people's housing, settlers and history.
    pub numbers: Vec<Numbers>,
    /// Each people's knowledge stocks.
    pub knowledge: Knowledge,
    /// Each people's gauges as last read.
    pub gauges: Vec<Gauges>,
    /// Deep time (Gyr) each people's knowledge was carried to.
    pub carried_to: Vec<Dy>,
}

impl Civilizations {
    /// One carry for every people: each living people is carried once over
    /// its own span, from where it was last carried to `to` (Gyr; the cosmic
    /// clock is already there), with `environment` by people index (its
    /// radiation against its home's, field.rs; 1 where absent).
    ///
    /// Order per people: knowledge first; then its world at `to`, its reach
    /// (the unoccupied worlds of its system within four orbital places that
    /// house a viable people) and any fall inside the span; then its numbers
    /// on its world as it stands; then its gauges. World reads run in
    /// parallel.
    ///
    /// With `decide`, a people whose world fails and whose drive leaves its
    /// well moves to the nearest world of its reach (an exodus), and a people
    /// whose fork chooses expansion settles the nearest world of its reach
    /// (a daughter people). Both are applied here and returned, so the
    /// caller can record them as inputs; without `decide` (laying a
    /// checkpoint back) neither is decided, as the recorded inputs carry them.
    pub fn carry(
        &mut self,
        cosmic: &mut Cosmic<'_>,
        to: Dy,
        environment: &[Dy],
        decide: bool,
    ) -> Vec<Act> {
        let count = cosmic.peoples.len();
        self.sync(count);
        let rulers = Rulers::default();
        let viable = xyl_cosmos::life::viable();
        // each living people's span (Gyr from, years)
        let spans: Vec<Option<(Dy, Dy)>> = cosmic
            .peoples
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let from = self.carried_to[i].max(p.dawn);
                (to > from && p.population.signum() > 0)
                    .then(|| (from, (to - from) * Dy::sci(1, 9)))
            })
            .collect();
        // masteries before the span, to read which pass one half within it
        let before: Vec<(Dy, Dy)> = (0..count)
            .map(|i| {
                (
                    self.knowledge.of[i].mastery(Domain::Biology),
                    self.knowledge.of[i].mastery(Domain::Information),
                )
            })
            .collect();
        // mind first: what each people knows going into its span
        for (i, p) in cosmic.peoples.iter().enumerate() {
            let Some((_, years)) = spans[i] else { continue };
            let bodies_w = cosmic.power(p).map(|pw| pw.bodies_w).unwrap_or(Dy::ZERO);
            let mut t = Thinkers::of(p, bodies_w, &cosmic.ctx);
            t.consciousness = self.gauges[i].consciousness;
            self.knowledge.step_one(i, &t, years);
        }
        let masteries: Vec<(Dy, Dy)> = (0..count)
            .map(|i| {
                (
                    self.knowledge.of[i].mastery(Domain::Biology),
                    self.knowledge.of[i].mastery(Domain::Energy),
                )
            })
            .collect();
        let half = Dy::ratio(1, 2);
        for i in 0..count {
            if spans[i].is_none() {
                continue;
            }
            let (bio0, info0) = before[i];
            let info = self.knowledge.of[i].mastery(Domain::Information);
            if bio0 < half && masteries[i].0 >= half {
                self.turning(i, "biology", "mastering its biosphere".into(), to);
            }
            if info0 < half && info >= half {
                self.turning(
                    i,
                    "information",
                    "understanding signals and minds".into(),
                    to,
                );
            }
        }
        // its drive: chemical and best drive blended by energy mastery
        // (transit.rs); it sails where the drive escapes its home's well
        let drives: Vec<Option<(crate::propulsion::Spec, Dy)>> = cosmic
            .peoples
            .iter()
            .enumerate()
            .map(|(i, p)| {
                spans[i]?;
                let first = crate::propulsion::Spec::of(
                    crate::propulsion::Drive::Chemical,
                    &p.traits,
                    &cosmic.gal.sc,
                    cosmic.gal.elements,
                );
                let spec = crate::transit::blended(
                    &first,
                    &crate::transit::best_spec(cosmic, p),
                    masteries[i].1,
                );
                let escape = crate::transit::home_escape(cosmic, p);
                let cruise = crate::transit::cruise_speed(
                    &spec,
                    Dy::int(crate::propulsion::MASS_RATIO),
                    escape,
                );
                (cruise.signum() > 0).then_some((spec, escape))
            })
            .collect();
        // worlds already held by a living people
        let held: std::collections::BTreeSet<(u64, usize)> = cosmic
            .peoples
            .iter()
            .filter(|p| p.population.signum() > 0)
            .map(|p| (p.star, p.world))
            .collect();
        // the world at `to`, the reach, and any fall inside the span, in parallel
        let stands: Vec<Option<Stand>> = {
            use rayon::prelude::*;
            let (gal, ctx, systems, peoples) =
                (&cosmic.gal, &cosmic.ctx, &cosmic.systems, &cosmic.peoples);
            let (rl, held) = (&rulers, &held);
            (0..count)
                .into_par_iter()
                .map(|i| {
                    let (from, _) = spans[i]?;
                    let p = &peoples[i];
                    let (bio, energy) = masteries[i];
                    let Some(reader) = SystemReader::of(gal, ctx, systems, p.star) else {
                        return Some(Stand::default());
                    };
                    let houses_at = |w: usize, d: i64| -> Dy {
                        reader
                            .read(w, d)
                            .map(|(st, ms, body)| {
                                houses(&p.traits, &st, &ms, &body, ctx, rl, bio, energy)
                            })
                            .unwrap_or(Dy::ZERO)
                    };
                    let (d0, d1) = (reader.degree(from), reader.degree(to));
                    let failing = houses_at(p.world, d1) < Dy::ONE;
                    // the reach is read for a sailing people whose world fails
                    // or that has grown past the logistic's inflection (when
                    // its fork may weigh expansion)
                    let crowded = p.population * Dy::int(2) >= p.capacity;
                    let reach: Vec<(usize, Dy)> = if drives[i].is_some() && (failing || crowded) {
                        (p.world.saturating_sub(4)..(p.world + 5).min(reader.worlds()))
                            .filter(|&w| w != p.world && !held.contains(&(p.star, w)))
                            .map(|w| (w, houses_at(w, d1)))
                            .filter(|(_, h)| *h >= viable)
                            .collect()
                    } else {
                        Vec::new()
                    };
                    if !failing {
                        return Some(Stand { fell: None, reach });
                    }
                    // bisect for the first degree of the star's life at which
                    // the world houses less than one body (worlds are constant
                    // within a degree)
                    let (mut lo, mut hi) = (d0, d1);
                    if houses_at(p.world, lo) < Dy::ONE {
                        hi = lo;
                    }
                    while hi - lo > 1 {
                        let mid = lo + (hi - lo) / 2;
                        if houses_at(p.world, mid) >= Dy::ONE {
                            lo = mid;
                        } else {
                            hi = mid;
                        }
                    }
                    Some(Stand {
                        fell: Some(reader.epoch(hi).clamp(from, to)),
                        reach,
                    })
                })
                .collect()
        };
        let mut acts = Vec::new();
        // numbers on the world as it is at `to`
        for i in 0..count {
            let (Some(stand), Some((_, years))) = (&stands[i], spans[i]) else {
                continue;
            };
            self.carried_to[i] = to;
            let here = cosmic.peoples[i].world;
            let nearest = stand
                .reach
                .iter()
                .min_by_key(|(w, _)| w.abs_diff(here))
                .copied();
            if let Some(t) = stand.fell {
                match nearest.filter(|_| decide) {
                    Some((w, _)) => {
                        self.move_people(cosmic, i, w, t);
                        acts.push(Act::Move {
                            people: i,
                            world: w,
                            at: t,
                        });
                    }
                    None => {
                        let name = cosmic.peoples[i].name.clone();
                        let p = &mut cosmic.peoples[i];
                        p.population = Dy::ZERO;
                        p.capacity = Dy::ZERO;
                        p.updated = to;
                        p.fell = Some(t);
                        cosmic.annals.push(Annal {
                            epoch: t,
                            kind: "FALL",
                            text: format!("{name}'s world no longer feeds it; it falls"),
                        });
                        continue;
                    }
                }
            }
            let p = cosmic.peoples[i].clone();
            let Some((state, media, body)) = cosmic.world_at(p.star, p.world, to) else {
                continue;
            };
            let (bio, energy) = masteries[i];
            // the open worlds of its reach
            let open: Vec<(usize, Dy)> = stand
                .reach
                .iter()
                .filter(|(w, _)| *w != p.world)
                .copied()
                .collect();
            // its numbers: the logistic toward what its world houses, scaled
            // by its radiation environment, at one e-fold per its generation
            let housed = houses(
                &p.traits,
                &state,
                &media,
                &body,
                &cosmic.ctx,
                &rulers,
                bio,
                energy,
            ) * environment
                .get(i)
                .copied()
                .unwrap_or(Dy::ONE)
                .clamp(Dy::ZERO, Dy::ONE);
            let gen = crate::growth::generation_yr(&p.traits, &rulers);
            let grown = crate::growth::grow(
                p.population,
                housed,
                Dy::ONE / gen.max(Dy::sci(1, -9)),
                years,
            );
            let q = &mut cosmic.peoples[i];
            q.population = if housed >= Dy::ONE {
                grown.max(Dy::ZERO)
            } else {
                Dy::ZERO
            };
            q.capacity = housed.max(Dy::ZERO);
            q.updated = to;
            if p.population.signum() > 0 && q.population.signum() <= 0 {
                q.fell = Some(to);
                cosmic.annals.push(Annal {
                    epoch: to,
                    kind: "FALL",
                    text: format!("{}'s world no longer feeds it; it falls", p.name),
                });
            }
            self.numbers[i].housing = housed;
            self.gauges[i] =
                Gauges::read(cosmic.peoples[i].population, housed, &self.knowledge.of[i]);
            self.remember(cosmic, i, years, to);
            // settlers: a people grown past its logistic's inflection (half
            // of what its world houses, where each newcomer adds less than
            // the last) founds a daughter on the nearest open world, with half
            // of what that world feeds, at least a viable people, at most half
            // its own numbers
            let expands = cosmic.peoples[i].population * Dy::int(2) >= housed;
            if let (true, true, Some(&(w, h))) = (
                decide,
                expands,
                open.iter().min_by_key(|(w, _)| w.abs_diff(p.world)),
            ) {
                let numbers = cosmic.peoples[i].population;
                let emigrants = (h / Dy::int(2)).max(viable).min(numbers / Dy::int(2));
                if emigrants >= viable && numbers - emigrants >= viable {
                    if let Some(d) = self.settle(cosmic, i, w, emigrants, to) {
                        acts.push(Act::Settle {
                            parent: i,
                            world: w,
                            emigrants,
                            at: to,
                            daughter: d,
                        });
                    }
                }
            }
        }
        acts
    }

    /// Grows every per-people table to `count` peoples.
    fn sync(&mut self, count: usize) {
        self.knowledge.sync(count);
        self.numbers.resize_with(count, Numbers::default);
        self.gauges.resize(count, Gauges::default());
        if self.carried_to.len() < count {
            self.carried_to.resize(count, Dy::ZERO);
        }
    }

    /// Moves people `i` whole to world `w` of its star at `at` Gyr (an
    /// exodus), with its annal.
    pub fn move_people(&mut self, cosmic: &mut Cosmic<'_>, i: usize, w: usize, at: Dy) {
        let Some(p) = cosmic.peoples.get_mut(i) else {
            return;
        };
        let was = p.world;
        p.world = w;
        let name = p.name.clone();
        self.turning(
            i,
            "exodus",
            format!("the exodus from world {was} to world {w}"),
            at,
        );
        cosmic.annals.push(Annal {
            epoch: at,
            kind: "EXODUS",
            text: format!(
                "{name}'s world {was} no longer feeds it; it leaves for world {w} of its star"
            ),
        });
    }

    /// Founds a daughter people of `parent` on world `w` with `emigrants`
    /// bodies at `at` Gyr (see [`Cosmic::settle`]): it inherits its parent's
    /// knowledge, is carried from `at`, and the parent's numbers record
    /// it. Returns the daughter's index, or `None` when it cannot be founded.
    pub fn settle(
        &mut self,
        cosmic: &mut Cosmic<'_>,
        parent: usize,
        w: usize,
        emigrants: Dy,
        at: Dy,
    ) -> Option<usize> {
        let d = cosmic.settle(parent, w, emigrants, at)?;
        self.sync(cosmic.peoples.len());
        if let Some(k) = self.knowledge.of.get(parent).cloned() {
            self.knowledge.of[d] = k;
        }
        self.carried_to[d] = at;
        let dem = &mut self.numbers[parent];
        dem.daughters += 1;
        dem.emigrants += emigrants;
        self.turning(
            parent,
            "settlement",
            format!("the settling of world {w}"),
            at,
        );
        let (pn, dn, star) = (
            cosmic.peoples[parent].name.clone(),
            cosmic.peoples[d].name.clone(),
            cosmic.peoples[d].star,
        );
        cosmic.annals.push(Annal {
            epoch: at,
            kind: "SETTLE",
            text: format!("The {pn} settle world {w} of star {star}: the {dn} are founded with {emigrants} bodies"),
        });
        Some(d)
    }
}

impl Civilizations {
    /// Reports a turning of people `i`'s history at `at` Gyr: `act` names
    /// what made it (exodus, settlement, network, quarantine, biology,
    /// information or fold). It is named at the people's next carry (see
    /// [`Civilizations::remember`]).
    pub fn turning(&mut self, i: usize, act: &'static str, title: String, at: Dy) {
        let Some(n) = self.numbers.get_mut(i) else {
            return;
        };
        n.candidates.push(Turning {
            act,
            title,
            at_yr: at * Dy::sci(1, 9),
        });
    }

    /// The history of people `i` over a span of `years` ending at `to` Gyr.
    /// Each figure's memory fades at the people's forgetting share a year,
    /// `memory (1 - f)^years`, and a figure dies at its expected end. Each
    /// reported turning names a figure whose role is read from the people at
    /// that moment: the domain its effort led in and that domain's share of
    /// its effort, held as a person, the colony or a locus of its medium by
    /// the shape of its mind; a figure's span is one of its people's
    /// generations from the turning. Each naming writes an `EMINENT` annal. Figures below
    /// [`forgotten`] are forgotten; the rest are kept best remembered first.
    ///
    /// [`forgotten`]: history::forgotten
    pub fn remember(&mut self, cosmic: &mut Cosmic<'_>, i: usize, years: Dy, to: Dy) {
        let Some(p) = cosmic.peoples.get(i).cloned() else {
            return;
        };
        let (forget, weights) = self
            .knowledge
            .of
            .get(i)
            .map(|k| (k.forgetting, k.weights))
            .unwrap_or((Dy::ZERO, [Dy::ZERO; 4]));
        let fade = ((Dy::ONE - forget.clamp(Dy::ZERO, Dy::ratio(999, 1000))).log() * years).exp();
        let now_yr = to * Dy::sci(1, 9);
        let gen = crate::growth::generation_yr(&p.traits, &Rulers::default());
        // the domain its effort leads in, and that domain's share of it
        let total = weights.iter().fold(Dy::ZERO, |a, w| a + *w);
        let (domain, lead) = Domain::ALL
            .iter()
            .map(|d| (*d, weights[d.index()]))
            .max_by(|a, b| a.1.cmp(&b.1))
            .map(|(d, w)| {
                (
                    d,
                    if total.signum() > 0 {
                        w / total
                    } else {
                        Dy::ZERO
                    },
                )
            })
            .unwrap_or((Domain::Biology, Dy::ZERO));
        let held = match p.traits.shape {
            crate::cosmic::Shape::Society => Held::Person,
            crate::cosmic::Shape::Superorganism => Held::Colony,
            crate::cosmic::Shape::Distributed => Held::Locus,
        };
        let Some(n) = self.numbers.get_mut(i) else {
            return;
        };
        let life = n;
        for e in &mut life.eminent {
            e.memory = e.memory * fade;
            e.dead = e.dead || now_yr >= e.dies_yr;
        }
        let turnings: Vec<Turning> = life.candidates.drain(..).collect();
        for t in turnings {
            let name = match held {
                Held::Colony => p.name.clone(),
                _ => coin(cosmic.gal.seed, p.star, p.world, t.at_yr, t.act),
            };
            let born_yr = t.at_yr;
            let dies_yr = born_yr + gen;
            let who = match held {
                Held::Person => {
                    format!("{name}, foremost in {} among the {}", domain.name(), p.name)
                }
                Held::Colony => format!("The {name}, as one colony working in {}", domain.name()),
                Held::Locus => format!(
                    "{name}, a locus of the {} working in {}",
                    p.name,
                    domain.name()
                ),
            };
            cosmic.annals.push(Annal {
                epoch: t.at_yr / Dy::sci(1, 9),
                kind: "EMINENT",
                text: format!("{who}, is remembered for {}", t.title),
            });
            life.eminent.push(Eminent {
                name,
                domain,
                lead,
                held,
                title: t.title,
                born_yr,
                dies_yr,
                memory: Dy::ONE,
                dead: now_yr >= dies_yr,
            });
        }
        life.eminent.retain(|e| e.memory >= history::forgotten());
        life.eminent.sort_by_key(|e| std::cmp::Reverse(e.memory));
    }
}

/// A placeholder name for a figure, drawn from the seed, the people's star
/// and world, the moment (standard years) and the act, in the peoples'
/// syllables, until a mind of the people names it.
fn coin(seed: u64, star: u64, world: usize, at_yr: Dy, act: &str) -> String {
    let mut rng = Stream::derive(
        seed,
        &[
            "eminent".into(),
            star.into(),
            (world as u64).into(),
            u64::try_from(at_yr.trunc_i64()).unwrap_or(0).into(),
            act.into(),
        ],
    );
    let syl = crate::cosmic::SYLLABLES;
    let count = 2 + rng.below(2).unwrap_or(0);
    let mut s = String::new();
    for _ in 0..count {
        let k = usize::try_from(rng.below(syl.len() as u64).unwrap_or(0)).unwrap_or(0);
        s.push_str(syl.get(k).copied().unwrap_or("a"));
    }
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

/// What the carry decided for a people, applied and returned so the caller
/// can record it as an input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    /// People `people` left its failing world for `world` at `at` Gyr.
    Move {
        /// The people's index.
        people: usize,
        /// The world it moved to, in its star's system.
        world: usize,
        /// When (Gyr).
        at: Dy,
    },
    /// People `parent` founded a daughter people on `world` at `at` Gyr.
    Settle {
        /// The parent's index.
        parent: usize,
        /// The settled world, in the parent's star's system.
        world: usize,
        /// Bodies that left with the settlers.
        emigrants: Dy,
        /// When (Gyr).
        at: Dy,
        /// The daughter's index.
        daughter: usize,
    },
}

/// Where a people stands at the end of its span: when its world stopped
/// feeding it, and the open worlds of its reach with what each houses.
#[derive(Default)]
struct Stand {
    fell: Option<Dy>,
    reach: Vec<(usize, Dy)>,
}
