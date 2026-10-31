//! The cosmic world in play: the galaxy from the Big Bang, deep time from
//! the first mind's dawn, the peoples the forecast wakes, and the annals.
//!
//! Inputs: a `xyl_cosmos::worlds::Galaxy` (seed, Big Bang, chemistry,
//! elements) and the forecast of minds (`xyl_cosmos::minds::Schedule`).
//! Stars come from `xyl_cosmos::galaxy`, systems from
//! `xyl_cosmos::systems`, worlds and media from `xyl_cosmos::worlds`.
//!
//! - Stars are identified by their index in the galaxy's sample and placed
//!   on their orbits at the present ([`Cosmic::place`]); [`Cosmic::distance`]
//!   is the arc between two stars on the universe's closed space (kpc).
//! - Systems are built only when witnessed: when a mind wakes there or a
//!   people reaches it ([`Cosmic::witness`]; [`Cosmic::prepare`] builds the
//!   span's systems in parallel). Worlds are read per degree of the star's
//!   life ([`Cosmic::world_at`], [`SystemReader`]) and cached; the caches
//!   change no result.
//! - Waking ([`Cosmic::wake`]): each forecast mind due by the target time
//!   arises on its world, unless the world is held or cannot feed a viable
//!   people (then it is counted `silent`). [`select`] reads its [`Traits`]
//!   from its world: strategy (a tilted axis or a small star gives r),
//!   sense, form, sociality, mind shape and size. Its numbers start at what
//!   its world houses with nothing learned (civ/housing.rs).
//! - Ancients: a mind born in a galactic core within one core assembly time
//!   (`minds::ancient_window`) of that core's first core-born mind; each
//!   [`People`] records whether it is one.
//! - [`Cosmic::settle`] founds a daughter people; [`Annal`]s record events.
//!
//! Peoples' numbers are carried by civ/mod.rs. Feedback from peoples'
//! actions onto the forecast is not modelled. The cosmic world is
//! regenerated from the seed; peoples' numbers are held in the checkpoint.
//!
//! Terms: ancient, annal, people, forecast (docs/glossary.md).

use std::collections::BTreeMap;
use xyl_core::dyadic::Dy;
use xyl_core::Stream;

use xyl_cosmos::chain::{radius, Place};
use xyl_cosmos::galaxy::{star, Star};
use xyl_cosmos::life::{foragers, viable, Body};
use xyl_cosmos::minds::{ancient_window, in_core, Mind, Schedule};
use xyl_cosmos::systems::{system, System};
use xyl_cosmos::worlds::{
    age_of_degree, best, body_temperature, build, degree_of, media, settings, star_degree_gyr, Ctx,
    Galaxy, Medium, Region, Solvent, State, Where,
};

fn n(x: i64) -> Dy {
    Dy::int(x)
}

fn r(a: i64, b: i64) -> Dy {
    Dy::ratio(a, b)
}

/// The syllables peoples' names are spoken in: display only, never a mechanic.
pub(crate) const SYLLABLES: [&str; 16] = [
    "ka", "th", "ol", "ne", "vri", "sha", "um", "or", "el", "qua", "zi", "mar", "tu", "ish", "ren",
    "ok",
];

/// How a people's lineage bred (a variable world breeds r, a steady one K).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    /// r-strategist: the smaller, faster-turning body; bred by a variable world.
    R,
    /// K-strategist: the larger, slower-turning body; bred by a steady world.
    K,
}

/// What its mind is shaped as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// A people of many minds.
    Society,
    /// A colony that thinks as one.
    Superorganism,
    /// A mind spread slow through its medium.
    Distributed,
}

/// What a world selected in the lineage that became a mind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Traits {
    /// Its breeding strategy, r or K.
    pub strategy: Strategy,
    /// The solvent its chemistry lives in.
    pub solvent: Solvent,
    /// Whether its medium is one starlight does not reach (under ice or in a giant's deep interior).
    pub dark: bool,
    /// Where on its world its medium lies (surface, under ice, supercritical, deep).
    pub at: Where,
    /// The temperature its body keeps (K): its chemistry's best.
    pub body_t: Dy,
    /// Its leading sense: "fields", "heat and pressure", "chemistry", "heat" or "light".
    pub sense: &'static str,
    /// Its body's form: "drifting", "swimming", "low and many-limbed", "tall and slender" or "walking".
    pub form: &'static str,
    /// What it works with: "shaped fields", "grasping limbs", "tendrils", "mouths and tongues" or "jointed palps".
    pub hands: &'static str,
    /// How social it is, 0 to 1.
    pub social: Dy,
    /// The shape of its mind: society, superorganism or distributed.
    pub shape: Shape,
    /// Its body's mass over a K-strategist's on a unit world.
    pub size: Dy,
}

impl Traits {
    /// The medium this body's chemistry lives in on a world: the richest of
    /// its own solvent's, wherever the world holds it (surface, under ice,
    /// supercritical). The body keeps its chemistry at its solvent's best
    /// (growth.rs `metabolism_w`), so it follows its solvent as the world
    /// changes; a region is a reading of a medium, never a bound on a people.
    pub fn medium<'m>(&self, ms: &'m [Medium]) -> Option<&'m Medium> {
        ms.iter()
            .filter(|m| m.solvent == self.solvent)
            .max_by(|a, b| {
                let ka = a.richness * a.energy.min(Dy::ONE);
                let kb = b.richness * b.energy.min(Dy::ONE);
                ka.cmp(&kb).then_with(|| (a.at as u8).cmp(&(b.at as u8)))
            })
    }

    /// A reading of the traits for the annals (words from the measures; the
    /// measures are the traits).
    pub fn describe(&self) -> String {
        let word = if self.size < r(1, 20) {
            "tiny"
        } else if self.size < r(1, 2) {
            "small"
        } else if self.size < n(2) {
            "middling"
        } else if self.size < n(10) {
            "large"
        } else {
            "vast"
        };
        let kind = match self.shape {
            Shape::Superorganism => "colony that thinks as one",
            Shape::Distributed => "mind spread through its medium",
            Shape::Society => "people of many minds",
        };
        format!(
            "a {kind}: {word}, {}, {}-sensing, working with {}",
            self.form, self.sense, self.hands
        )
    }
}

/// A people that has woken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct People {
    /// Its display name, spoken in generated syllables.
    pub name: String,
    /// Its star (index in the galaxy's sample).
    pub star: u64,
    /// Index of its world within its star's system.
    pub world: usize,
    /// The region of chemistry space its lineage arose in.
    pub region: Region,
    /// What its world selected in its lineage.
    pub traits: Traits,
    /// When it woke (Gyr after the Big Bang).
    pub dawn: Dy,
    /// Its numbers (bodies).
    pub population: Dy,
    /// Numbers its world carries of its bodies (growth.rs).
    pub capacity: Dy,
    /// Its growth rate (per year).
    pub rate: Dy,
    /// When its numbers were last carried forward (Gyr after the Big Bang).
    pub updated: Dy,
    /// Born in the core within one core assembly time of the core's first mind.
    pub ancient: bool,
    /// When its lineage climbed each rung (Gyr after the Big Bang).
    pub steps: Vec<Dy>,
    /// Number of impacts that threw its lineage back a rung.
    pub setbacks: u32,
    /// When its world stopped feeding it, if it has fallen.
    pub fell: Option<Dy>,
}

/// An entry of the annals: what happened, when.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annal {
    /// When it happened (Gyr after the Big Bang).
    pub epoch: Dy,
    /// Category tag, such as "EMERGENCE", "DEEP_HISTORY", "FALL", "RUINS" or "FOCUS".
    pub kind: &'static str,
    /// Human-readable description.
    pub text: String,
}

/// A witnessed system: its star and the system physics made around it.
pub struct Witnessed {
    /// The star, as placed in the galaxy's sample.
    pub star: Star,
    /// The planetary system physics built around it.
    pub system: System,
}

/// A cosmic universe in play.
pub struct Cosmic<'g> {
    /// The galaxy: its seed, Big Bang, chemistry and element table.
    pub gal: Galaxy<'g>,
    /// The physical scales worlds are built under.
    pub ctx: Ctx,
    /// When and where each mind would wake in the undisturbed galaxy, in order of waking.
    pub forecast: Schedule,
    /// Deep time now (Gyr after the Big Bang).
    pub now: Dy,
    /// The next forecast mind not yet woken (minds are in order of waking).
    next: usize,
    /// Systems built so far, keyed by star index (built only when witnessed).
    pub systems: BTreeMap<u64, Witnessed>,
    /// Every people woken, in order of waking.
    pub peoples: Vec<People>,
    /// What has happened, in order.
    pub annals: Vec<Annal>,
    /// The minds whose worlds were taken or whose stars were gone when they
    /// would have woken.
    pub silent: u64,
    names: Vec<String>,
    /// Worlds as built, keyed by (star, world, degree of its star's life). A
    /// world is constant within a degree and built once. Pure cache: dropping
    /// it changes no result.
    world_cache: std::cell::RefCell<BTreeMap<(u64, usize, i64), (State, Vec<Medium>, Body)>>,
    heartbeat_cache:
        std::cell::RefCell<BTreeMap<(u64, usize, i64), Option<xyl_cosmos::worlds::Heartbeat>>>,
}

impl<'g> Cosmic<'g> {
    /// Starts the universe's clock at its first mind's waking.
    pub fn genesis(gal: Galaxy<'g>, forecast: Schedule) -> Cosmic<'g> {
        let ctx = Ctx::new(gal.sc);
        let now = forecast.minds.first().map(|m| m.epoch).unwrap_or(Dy::ZERO);
        let mut c = Cosmic {
            gal,
            ctx,
            forecast,
            now,
            next: 0,
            systems: BTreeMap::new(),
            peoples: Vec::new(),
            annals: Vec::new(),
            silent: 0,
            names: Vec::new(),
            world_cache: Default::default(),
            heartbeat_cache: Default::default(),
        };
        c.annals.push(Annal {
            epoch: Dy::ZERO,
            kind: "BIG_BANG",
            text: format!(
                "The universe begins: the white hole of a black hole in a parent universe - matter that crossed its horizon bounced at the Cartan density, and this spacetime is the far face of that core (seed {})",
                c.gal.seed
            ),
        });
        c.wake(now);
        c
    }

    /// The system of star `n`, witnessed (built once, from physics).
    pub fn witness(&mut self, n: u64) -> Option<&Witnessed> {
        if let std::collections::btree_map::Entry::Vacant(e) = self.systems.entry(n) {
            let s = star(self.gal.seed, self.gal.bang, n)?;
            let sys = system(
                self.gal.seed,
                self.gal.bang,
                &[n],
                &s,
                self.gal.chem,
                self.gal.elements,
            )?;
            e.insert(Witnessed {
                star: s,
                system: sys,
            });
        }
        self.systems.get(&n)
    }

    /// Where star `n` is now (kpc about the core).
    pub fn place(&self, n: u64) -> Option<Place> {
        self.systems
            .get(&n)
            .map(|w| w.star.place(self.now, &radius()))
    }

    /// The arc between two witnessed stars now (kpc), on the closed space.
    pub fn distance(&self, a: u64, b: u64) -> Option<Dy> {
        Some(self.place(a)?.arc_kpc(&self.place(b)?, &radius()))
    }

    /// A world's state at an epoch, read at its star's age then, quantised to
    /// degrees of the star's life.
    pub fn world_at(&self, star_n: u64, w: usize, epoch: Dy) -> Option<(State, Vec<Medium>, Body)> {
        let key = world_key(&self.systems, &self.ctx, star_n, w, epoch)?;
        if let Some(hit) = self.world_cache.borrow().get(&key) {
            return Some(hit.clone());
        }
        let out = world_read(&self.gal, &self.ctx, &self.systems, key)?;
        self.world_cache.borrow_mut().insert(key, out.clone());
        Some(out)
    }

    /// A world's heartbeat at an epoch: its day and year as it absorbs the
    /// bells' echo (worlds.rs Path::heartbeat). None for a world with no path.
    pub fn heartbeat(
        &self,
        star_n: u64,
        w: usize,
        epoch: Dy,
    ) -> Option<xyl_cosmos::worlds::Heartbeat> {
        let wit = self.systems.get(&star_n)?;
        let born = Dy::from_q(&wit.star.born);
        // a heartbeat slows over Gyr; quantised to the same degrees of the
        // star's life as the world
        let deg = star_degree_gyr(Dy::from_q(&wit.star.r), &self.ctx);
        let quarters = degree_of(epoch - born, deg);
        if let Some(hit) = self.heartbeat_cache.borrow().get(&(star_n, w, quarters)) {
            return *hit;
        }
        let sets = settings(&self.gal, &[star_n], &wit.system, &wit.star);
        let hb = sets.get(w).and_then(|s| {
            s.path
                .as_ref()
                .map(|p| p.heartbeat(age_of_degree(quarters, deg).max(Dy::ZERO)))
        });
        self.heartbeat_cache
            .borrow_mut()
            .insert((star_n, w, quarters), hb);
        hb
    }

    /// Precomputes, in parallel, the star systems and worlds for every mind the
    /// forecast wakes by `now` (pure functions of the seed), so the minds then
    /// arise in order from cached results. The event engine calls it once per
    /// carried span rather than once per waking.
    pub fn prepare(&mut self, now: Dy) {
        use rayon::prelude::*;
        let due: Vec<Mind> = self
            .forecast
            .minds
            .iter()
            .skip(self.next)
            .take_while(|m| m.epoch <= now)
            .cloned()
            .collect();
        let mut want: Vec<u64> = due
            .iter()
            .map(|m| m.star)
            .filter(|k| !self.systems.contains_key(k))
            .collect();
        want.sort_unstable();
        want.dedup();
        let gal = self.gal;
        let clock = std::time::Instant::now();
        let built: Vec<(u64, Witnessed, std::time::Duration)> = want
            .par_iter()
            .filter_map(|&k| {
                let t = std::time::Instant::now();
                let _busy = xyl_core::watch::busy(format!("system of star {k}"));
                let st = star(gal.seed, gal.bang, k)?;
                let sys = system(gal.seed, gal.bang, &[k], &st, gal.chem, gal.elements)?;
                Some((
                    k,
                    Witnessed {
                        star: st,
                        system: sys,
                    },
                    t.elapsed(),
                ))
            })
            .collect();
        // timing diagnostics: total system cost and the slowest systems
        let mut slow: Vec<(std::time::Duration, u64, usize)> = built
            .iter()
            .map(|(k, w, d)| (*d, *k, w.system.worlds.len()))
            .collect();
        slow.sort_unstable_by_key(|x| std::cmp::Reverse(x.0));
        let total: std::time::Duration = slow.iter().map(|x| x.0).sum();
        if !built.is_empty() {
            xyl_core::watch::note(&format!(
                "prepare: {} systems built in {:.1?} ({:.1?} of work); slowest: {}",
                built.len(),
                clock.elapsed(),
                total,
                slow.iter()
                    .take(5)
                    .map(|(d, k, w)| format!("star {k} {w} worlds {d:.1?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        self.systems
            .extend(built.into_iter().map(|(k, w, _)| (k, w)));
        let (ctx, systems) = (&self.ctx, &self.systems);
        let keys: Vec<(u64, usize, i64)> = due
            .iter()
            .filter_map(|m| world_key(systems, ctx, m.star, m.w, m.epoch))
            .filter(|k| !self.world_cache.borrow().contains_key(k))
            .collect();
        let clock = std::time::Instant::now();
        let reads: Vec<((u64, usize, i64), (State, Vec<Medium>, Body))> = keys
            .par_iter()
            .filter_map(|&k| {
                let _busy = xyl_core::watch::busy(format!(
                    "world {} of star {} at degree {}",
                    k.1, k.0, k.2
                ));
                world_read(&gal, ctx, systems, k).map(|v| (k, v))
            })
            .collect();
        if !reads.is_empty() {
            xyl_core::watch::note(&format!(
                "prepare: {} worlds read in {:.1?}",
                reads.len(),
                clock.elapsed()
            ));
        }
        self.world_cache.borrow_mut().extend(reads);
    }

    /// Advances deep time to `to` and wakes the minds the forecast schedules by
    /// then. Peoples' numbers are left to the caller (universe.rs, via
    /// civ). Returns the annal lines of what woke.
    pub fn wake(&mut self, to: Dy) -> Vec<String> {
        if to > self.now {
            self.now = to;
        }
        self.prepare(self.now);
        let mut lines = Vec::new();
        while let Some(m) = self.forecast.minds.get(self.next).cloned() {
            if m.epoch > self.now {
                break;
            }
            self.next += 1;
            if let Some(line) = self.arise(&m) {
                lines.push(line);
            }
        }
        lines
    }

    /// Epoch (Gyr) of the forecast's next unwoken mind: the event engine's next
    /// waking (universe.rs).
    pub fn next_wake(&self) -> Option<Dy> {
        self.forecast.minds.get(self.next).map(|m| m.epoch)
    }

    /// The power a people's world offers and its bodies turn, now (growth.rs
    /// layer 2): starlight, its world's heat, its bodies' metabolism.
    pub fn power(&self, p: &People) -> Option<crate::growth::Power> {
        let (state, _, body) = self.world_at(p.star, p.world, self.now)?;
        Some(crate::growth::power(
            &state,
            &body,
            &p.traits,
            p.population,
            &self.ctx,
            &crate::growth::Rulers::default(),
        ))
    }

    fn name(&mut self, m: &Mind) -> String {
        self.name_for(m.star, m.w)
    }

    /// A new people's name, drawn from the seed and its star and world, and
    /// unique among the universe's peoples.
    fn name_for(&mut self, star: u64, w: usize) -> String {
        let mut rng = Stream::derive(
            self.gal.seed,
            &["name".into(), star.into(), (w as u64).into()],
        );
        loop {
            let count = 2 + rng.below(2).unwrap_or(0);
            let mut s = String::new();
            for _ in 0..count {
                let i =
                    usize::try_from(rng.below(SYLLABLES.len() as u64).unwrap_or(0)).unwrap_or(0);
                s.push_str(SYLLABLES[i]);
            }
            let mut c = s.chars();
            let name = c
                .next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default();
            if !self.names.contains(&name) {
                self.names.push(name.clone());
                return name;
            }
        }
    }

    /// Founds a daughter people of `parent` on world `w` of its star at `at`
    /// Gyr, with `emigrants` bodies taken from the parent: the same traits,
    /// lineage and standing, its region read from the world's medium for
    /// those traits, its name drawn for its world. Returns its index, or
    /// `None` when the parent does not exist, the world cannot be read, or
    /// the emigrants exceed the parent's numbers.
    pub fn settle(&mut self, parent: usize, w: usize, emigrants: Dy, at: Dy) -> Option<usize> {
        let p = self.peoples.get(parent)?.clone();
        if emigrants.signum() <= 0 || emigrants > p.population {
            return None;
        }
        let (_, ms, _) = self.world_at(p.star, w, at)?;
        let region = p.traits.medium(&ms).map(|m| m.region).unwrap_or(p.region);
        let name = self.name_for(p.star, w);
        if let Some(q) = self.peoples.get_mut(parent) {
            q.population -= emigrants;
        }
        self.peoples.push(People {
            name,
            star: p.star,
            world: w,
            region,
            traits: p.traits.clone(),
            dawn: at,
            population: emigrants,
            capacity: emigrants,
            rate: p.rate,
            updated: at,
            ancient: p.ancient,
            steps: p.steps.clone(),
            setbacks: p.setbacks,
            fell: None,
        });
        Some(self.peoples.len() - 1)
    }

    /// Wakes a mind on the world its lineage climbed on, witnessing only its own system.
    fn arise(&mut self, m: &Mind) -> Option<String> {
        self.witness(m.star)?;
        if self
            .peoples
            .iter()
            .any(|p| p.star == m.star && p.world == m.w)
        {
            self.silent += 1;
            return None;
        }
        let (state, ms, body) = self.world_at(m.star, m.w, m.epoch)?;
        let fed = foragers(&ms, m.region, &body, &self.ctx);
        if fed < viable() {
            // the world as it is now cannot feed a people
            self.silent += 1;
            return None;
        }
        let strategy = if body.tilt > 23 || body.star_mass < r(1, 2) {
            Strategy::R
        } else {
            Strategy::K
        };
        let mut rng = Stream::derive(
            self.gal.seed,
            &["body".into(), m.star.into(), (m.w as u64).into()],
        );
        let traits = select(&state, &ms, m.region, strategy, &mut rng, &self.ctx)?;
        // numbers start at the world's capacity for bodies like its own
        // (growth.rs)
        let rulers = crate::growth::Rulers::default();
        // nothing learned yet: capacity from foraging alone (civ/housing.rs houses)
        let capacity = crate::civ::housing::houses(
            &traits,
            &state,
            &ms,
            &body,
            &self.ctx,
            &rulers,
            Dy::ZERO,
            Dy::ZERO,
        );
        let rate = crate::growth::growth_per_yr(&traits, &rulers);
        let name = self.name(m);
        // ancient against its own galactic core's first core-born mind (T1)
        let ancient = in_core(m)
            && xyl_cosmos::minds::first_core_dawn(
                &self.forecast,
                xyl_cosmos::galaxy::core_of(m.star),
            )
            .is_some_and(|d| m.epoch - d <= ancient_window(&self.gal));
        let r_kpc = m.r_kpc;
        if let Some(&first) = m.steps.first() {
            self.annals.push(Annal {
                epoch: m.epoch,
                kind: "DEEP_HISTORY",
                text: format!(
                    "Life began on world {} of star {} {} Gyr before the {name}; it passed {} hard steps{}",
                    m.w,
                    m.star,
                    m.epoch - first,
                    m.steps.len(),
                    if m.setbacks > 0 { format!(", thrown back {} times by the impacts of its sky", m.setbacks) } else { String::new() }
                ),
            });
        }
        let line = format!(
            "{}The {name} look up: on world {} of star {}, {} kpc from the galactic centre, in {}{} {}, a people became aware of itself - {}",
            if ancient { "[ANCIENT] " } else { "" },
            m.w,
            m.star,
            r_kpc,
            if traits.dark { "a dark " } else { "" },
            traits.solvent.name(),
            match traits.at {
                Where::Surface => "surface",
                Where::UnderIce => "sea under ice",
                Where::Supercritical => "supercritical fluid",
                Where::Deep => "deep",
            },
            traits.describe()
        );
        self.annals.push(Annal {
            epoch: m.epoch,
            kind: "EMERGENCE",
            text: line.clone(),
        });
        if capacity.signum() <= 0 {
            self.annals.push(Annal {
                epoch: m.epoch,
                kind: "FALL",
                text: format!("{name}'s world cannot feed it; it falls at its dawn"),
            });
        }
        self.peoples.push(People {
            name,
            star: m.star,
            world: m.w,
            region: m.region,
            traits,
            dawn: m.epoch,
            population: capacity,
            capacity,
            rate,
            updated: m.epoch,
            ancient,
            steps: m.steps.clone(),
            setbacks: m.setbacks,
            fell: (capacity.signum() <= 0).then_some(m.epoch),
        });
        Some(line)
    }
}

/// The traits a world selected in a lineage, from
/// its best medium for that chemistry, its gravity and its strategy.
pub fn select(
    state: &State,
    ms: &[Medium],
    region: Region,
    strategy: Strategy,
    rng: &mut Stream,
    ctx: &Ctx,
) -> Option<Traits> {
    let med = best(ms, Some(region))?;
    let dark = matches!(med.at, Where::UnderIce | Where::Deep);
    let g = state.g;
    let plasma = med.solvent == Solvent::Plasma;
    let cold_sea = matches!(
        med.solvent,
        Solvent::Volatile(
            xyl_cosmos::worlds::Species::CH4
                | xyl_cosmos::worlds::Species::N2
                | xyl_cosmos::worlds::Species::NH3
        )
    );
    let sense = if plasma {
        "fields"
    } else if dark {
        "heat and pressure"
    } else if cold_sea {
        "chemistry"
    } else if med.t > n(450) {
        "heat"
    } else {
        "light"
    };
    let form = if plasma {
        "drifting"
    } else if dark || med.at == Where::UnderIce {
        "swimming"
    } else if g > r(3, 2) {
        "low and many-limbed"
    } else if g < r(1, 2) {
        "tall and slender"
    } else {
        "walking"
    };
    let u = Dy::unit(rng.next_u64());
    let social =
        (u + if strategy == Strategy::R {
            r(3, 10)
        } else {
            Dy::ZERO
        } + if med.energy < r(1, 10) {
            r(1, 5)
        } else {
            Dy::ZERO
        } - r(1, 4))
        .clamp(Dy::ZERO, Dy::ONE);
    let shape = if social > r(3, 4) && strategy == Strategy::R {
        Shape::Superorganism
    } else if plasma || (dark && med.pace < r(1, 20)) {
        Shape::Distributed
    } else {
        Shape::Society
    };
    let hands = if plasma {
        "shaped fields"
    } else {
        [
            "grasping limbs",
            "tendrils",
            "mouths and tongues",
            "jointed palps",
        ][usize::try_from(rng.below(4).unwrap_or(0)).unwrap_or(0)]
    };
    // size: life history (the strategies' size rulers, r 2.5, K 1.2) over
    // the world's gravity, linear in g
    let ceiling = if strategy == Strategy::R {
        r(5, 2)
    } else {
        r(6, 5)
    };
    let mut size = r(6, 5) / ceiling;
    if !plasma {
        size = size / g.max(Dy::sci(1, -3));
    }
    Some(Traits {
        strategy,
        solvent: med.solvent,
        dark,
        at: med.at,
        body_t: body_temperature(med.solvent, ctx),
        sense,
        form,
        hands,
        social,
        shape,
        size,
    })
}

/// Cache key for a world read: star, world index, and the star's age in
/// degrees of its life.
fn world_key(
    systems: &BTreeMap<u64, Witnessed>,
    ctx: &Ctx,
    star_n: u64,
    w: usize,
    epoch: Dy,
) -> Option<(u64, usize, i64)> {
    let wit = systems.get(&star_n)?;
    let born = Dy::from_q(&wit.star.born);
    let deg = star_degree_gyr(Dy::from_q(&wit.star.r), ctx);
    Some((star_n, w, degree_of(epoch - born, deg)))
}

/// Reads a star's worlds at any degree of its life. The worlds' settings are
/// computed once, so any world can be read at any degree, on any thread,
/// without rebuilding the rest.
pub struct SystemReader<'a> {
    sets: Vec<xyl_cosmos::worlds::Setting<'a>>,
    born: Dy,
    deg: Dy,
    ctx: &'a Ctx,
}

impl<'a> SystemReader<'a> {
    /// A reader for star `star_n`, computing its worlds' settings once.
    /// Returns `None` when the star's system has not been witnessed (is not in `systems`).
    pub fn of(
        gal: &Galaxy,
        ctx: &'a Ctx,
        systems: &'a BTreeMap<u64, Witnessed>,
        star_n: u64,
    ) -> Option<SystemReader<'a>> {
        let wit = systems.get(&star_n)?;
        let deg = star_degree_gyr(Dy::from_q(&wit.star.r), ctx);
        Some(SystemReader {
            sets: settings(gal, &[star_n], &wit.system, &wit.star),
            born: Dy::from_q(&wit.star.born),
            deg,
            ctx,
        })
    }

    /// The degree of its star's life at `t` Gyr.
    pub fn degree(&self, t: Dy) -> i64 {
        degree_of(t - self.born, self.deg)
    }

    /// When (Gyr) its star's life reaches `degree`.
    pub fn epoch(&self, degree: i64) -> Dy {
        self.born + age_of_degree(degree, self.deg)
    }

    /// Number of worlds in the system.
    pub fn worlds(&self) -> usize {
        self.sets.len()
    }

    /// World `w`'s state, media and body at `degree`.
    pub fn read(&self, w: usize, degree: i64) -> Option<(State, Vec<Medium>, Body)> {
        let set = self.sets.get(w)?;
        let age = age_of_degree(degree, self.deg).max(self.deg);
        // a world that has merged into its neighbour by this age is gone
        if set
            .world
            .merged_at
            .as_ref()
            .is_some_and(|m| Dy::from_q(m) <= age)
        {
            return None;
        }
        let state = build(set, age, self.ctx);
        let ms = media(&state, set.z, self.ctx);
        Some((state, ms, Body::of(set)))
    }
}

/// A world's state at its key's age, a pure function of its laws (safe on any
/// thread).
fn world_read(
    gal: &Galaxy,
    ctx: &Ctx,
    systems: &BTreeMap<u64, Witnessed>,
    key: (u64, usize, i64),
) -> Option<(State, Vec<Medium>, Body)> {
    let (star_n, w, degree) = key;
    SystemReader::of(gal, ctx, systems, star_n)?.read(w, degree)
}
