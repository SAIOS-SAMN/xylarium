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
//!   from its world: strategy (an axial tilt above 23 degrees or a star
//!   mass below 1/2 gives r),
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
//!
//! Submodules: `emergence` (minds arising as peoples, their traits and
//! names, daughter peoples) and `reader` (systems and worlds read per degree
//! of a star's life, prepared in parallel).

mod emergence;
mod reader;

pub use emergence::select;
pub use reader::SystemReader;

use reader::{world_key, world_read};
use std::collections::BTreeMap;
use xyl_core::dyadic::Dy;

use xyl_cosmos::chain::{radius, Place};
use xyl_cosmos::galaxy::{star, Star};
use xyl_cosmos::life::Body;
use xyl_cosmos::minds::Schedule;
use xyl_cosmos::systems::{system, System};
use xyl_cosmos::worlds::{
    age_of_degree, degree_of, settings, star_degree_gyr, Ctx, Galaxy, Medium, Region, Solvent,
    State, Where,
};

/// The syllables peoples' names are spoken in: display only, never a mechanic.
pub(crate) const SYLLABLES: [&str; 16] = [
    "ka", "th", "ol", "ne", "vri", "sha", "um", "or", "el", "qua", "zi", "mar", "tu", "ish", "ren",
    "ok",
];

/// How a people's lineage bred: r where the world's axial tilt exceeds 23
/// degrees or its star's mass is below 1/2, K otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    /// The r-strategist: the smaller body with the faster generations.
    R,
    /// The K-strategist: the larger body with the slower generations.
    K,
}

/// The shape of a people's mind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// A people of many minds.
    Society,
    /// A colony that thinks as one.
    Superorganism,
    /// A mind spread through its medium.
    Distributed,
}

/// What a world selected in the lineage that became a mind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Traits {
    /// Its breeding strategy, r or K.
    pub strategy: Strategy,
    /// The solvent its chemistry lives in.
    pub solvent: Solvent,
    /// Whether its medium is one starlight does not reach (under ice or in a
    /// giant's deep interior).
    pub dark: bool,
    /// Where on its world its medium lies (surface, under ice, supercritical,
    /// deep).
    pub at: Where,
    /// The temperature its body keeps (K): the best temperature for its
    /// solvent (`worlds::body_temperature`).
    pub body_t: Dy,
    /// Its leading sense: "fields", "heat and pressure", "chemistry", "heat"
    /// or "light".
    pub sense: &'static str,
    /// Its body's form: "drifting", "swimming", "low and many-limbed", "tall
    /// and slender" or "walking".
    pub form: &'static str,
    /// What it works with: "shaped fields", "grasping limbs", "tendrils",
    /// "mouths and tongues" or "jointed palps".
    pub hands: &'static str,
    /// How social it is, 0 to 1.
    pub social: Dy,
    /// The shape of its mind: society, superorganism or distributed.
    pub shape: Shape,
    /// Its body's mass over a K-strategist's on a unit world.
    pub size: Dy,
}

impl Traits {
    /// The medium this body's chemistry lives in on a world: of the media of
    /// the body's solvent, wherever the world holds them (surface, under ice,
    /// supercritical, deep), the one with the largest richness x min(energy,
    /// 1). None where the world holds no medium of that solvent. The body
    /// keeps its chemistry at the solvent's best temperature (growth.rs
    /// `metabolism_w`), so it follows its solvent as the world changes; a
    /// region is a reading of a medium, never a bound on a people.
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
        let word = if self.size < Dy::ratio(1, 20) {
            "tiny"
        } else if self.size < Dy::ratio(1, 2) {
            "small"
        } else if self.size < Dy::int(2) {
            "middling"
        } else if self.size < Dy::int(10) {
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
    /// Its display name, two or three syllables of `SYLLABLES`.
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
    /// The numbers of its bodies its world houses, as last read
    /// (civ/housing.rs; scaled by the radiation environment in the carry).
    pub capacity: Dy,
    /// Its growth rate (per year).
    pub rate: Dy,
    /// When its numbers were last carried forward (Gyr after the Big Bang).
    pub updated: Dy,
    /// Whether it was born in a galactic core within one core assembly time
    /// of that core's first core-born mind.
    pub ancient: bool,
    /// When its lineage climbed each rung (Gyr after the Big Bang).
    pub steps: Vec<Dy>,
    /// Number of impacts that threw its lineage back a rung.
    pub setbacks: u32,
    /// When its world stopped feeding it (Gyr), if it has fallen.
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
    /// When and where each mind would wake in the undisturbed galaxy, in
    /// order of waking.
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
    /// Starts deep time at the epoch of the forecast's first mind (0 when
    /// the forecast is empty), records the Big Bang annal and wakes the
    /// minds due by then.
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
                concat!(
                    "The universe begins: the white hole of a black hole in a parent universe - ",
                    "matter that crossed its horizon bounced at the Cartan density, and this ",
                    "spacetime is the far face of that core (seed {})"
                ),
                c.gal.seed
            ),
        });
        c.wake(now);
        c
    }

    /// The system of star `n`, witnessed (built once, from physics). None
    /// when the star or its system cannot be built.
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

    /// Where star `n` is now (kpc about the core). None for a star not yet
    /// witnessed.
    pub fn place(&self, n: u64) -> Option<Place> {
        self.systems.get(&n).map(|w| {
            // The cosmos places stars in rings; `now` converts here.
            let t = xyl_cosmos::circuit::rings_of_gyr_dy(self.now);
            w.star.place(t, &radius())
        })
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

    /// A world's heartbeat at an epoch: its day and year (worlds.rs
    /// `Path::heartbeat`), at the star's age quantised to degrees of its
    /// life. None for an unwitnessed star or a world with no path.
    pub fn heartbeat(
        &self,
        star_n: u64,
        w: usize,
        epoch: Dy,
    ) -> Option<xyl_cosmos::worlds::Heartbeat> {
        let wit = self.systems.get(&star_n)?;
        let born = xyl_cosmos::circuit::gyr_of_rings_dy(Dy::from_q(&wit.star.born));
        // The heartbeat changes over Gyr; it is cached per degree of the
        // star's life, as world states are.
        let deg = star_degree_gyr(
            &wit.star.sequence(),
            wit.star.compactness(),
            xyl_cosmos::galaxy::kpc_of_rings_dy(Dy::from_q(&wit.star.r)),
            &self.ctx,
        );
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
}
