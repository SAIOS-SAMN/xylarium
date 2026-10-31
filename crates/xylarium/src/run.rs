//! The `genesis` and `run` commands and the shared universe builder, on
//! the event engine in `xyl_kernel::universe` (the clock is the bell's
//! count).
//!
//!   xylarium genesis --home DIR [--seed 7] [--stars 400]
//!                    [--threads 2] [--sight census]
//!       Founds a new home: the Big Bang, its galaxies and systems and the
//!       forecast of minds, written as `DIR/genesis`, with the first
//!       checkpoint at the first mind's dawn. Returns `Err` for a home that
//!       already holds a genesis. Local files only; nothing is posted.
//!
//!   xylarium run [--home DIR] [--for N] [--save-every K] [--velocity V]
//!                \[same options\]
//!       Live play, one ring of the bell per advance with every event inside
//!       it at its own count, the checkpoint written every K rings (default
//!       1) and on stop. `--velocity` caps rings per second (0: no cap).
//!       An uplink runs the same loop with a [`Link`] beside it
//!       ([`run_with`]).
//!
//! [`with`] builds the universe for every command: it regenerates the Big
//! Bang, chemistry, elements and galaxy from the seed, reads or computes
//! the forecast and the heavy-star list (caches in the home, keyed by the
//! laws' digest), then either restores the checkpoint
//! (`Checkpoint::restore`, [`Source::Checkpoint`]) or, with none, carries a
//! fresh universe to 14 Gyr ([`Source::Fresh`]). The checkpoint is written
//! whole or not at all (temporary file, then rename), after committing the
//! keepers' chain with the link's last final clock height.
//!
//! Terms: home, sight, uplink (docs/glossary.md).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use xyl_core::dyadic::Dy;
use xyl_core::Q;
use xyl_cosmos::bang::BigBang;
use xyl_cosmos::chem::Chemistry;
use xyl_cosmos::worlds::{Ctx, Galaxy};
use xyl_cosmos::Laws;
use xyl_kernel::checkpoint::Checkpoint;
use xyl_kernel::cosmic::Cosmic;
use xyl_kernel::universe::Universe;

/// The command-line options shared by `genesis`, `run` and `observe`, as
/// parsed by [`crate::options`].
pub struct Options {
    /// The universe's seed: every stream is derived from it (`--seed`, default 7).
    pub seed: u64,
    /// The number of sample stars whose forecast of minds is worked out (`--stars`, default 400).
    pub stars: u64,
    /// Worker threads for the forecast (`--threads`, default 2).
    pub threads: usize,
    /// The universe's home directory holding genesis, checkpoint and forecasts (`--home`).
    pub home: PathBuf,
    /// Rings of the bell between checkpoints in live play (`--save-every`).
    pub save_every: u64,
    /// Rings of the bell to play before stopping (`--for`); `None` plays
    /// until interrupted.
    pub until: Option<u64>,
    /// Rings per wall-clock second (`--velocity`): a throttle on the runner,
    /// never on the physics; 0 runs as fast as the machine allows.
    pub velocity: u64,
    /// Unroll every sample star (a census), not only what the cores see.
    pub census: bool,
}

/// Where the universe's held state came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// Carried to a checkpoint read from the home, with its records laid back.
    Checkpoint,
    /// No checkpoint in the home: a fresh genesis carried to 14 Gyr.
    Fresh,
}

pub(crate) fn show(q: &Q) -> String {
    Dy::from_q(q).to_string()
}

fn checkpoint_path(home: &Path) -> PathBuf {
    home.join("checkpoint")
}

/// Write the checkpoint, whole or not at all, committing the keepers' chain
/// at this moment with the last final clock height a link heard (none when
/// running locally).
fn keep(u: &mut Universe<'_>, home: &Path, height: Option<u64>) -> Result<(), String> {
    u.commit_clocks(height);
    let path = checkpoint_path(home);
    let tmp = path.with_extension("tmp");
    std::fs::create_dir_all(home).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, Checkpoint::of(u).encode())
        .and_then(|_| std::fs::rename(&tmp, &path))
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// Build the universe from its seed (the laws regenerate what they give),
/// carry it event to event to its held state, and hand it to `f`.
pub fn with<T>(
    o: &Options,
    f: impl FnOnce(&mut Universe<'_>, Source, &BigBang) -> Result<T, String>,
) -> Result<T, String> {
    let t0 = std::time::Instant::now();
    xyl_core::watch::into(&o.home);
    xyl_cosmos::circuit::cache_in(&o.home);
    xyl_core::watch::heartbeat(
        std::time::Duration::from_secs(30),
        Arc::new(AtomicBool::new(false)),
    );
    xyl_core::watch::note(&format!(
        "Observer: seed {} stars {} threads {} sight {} home {}",
        o.seed,
        o.stars,
        o.threads,
        if o.census { "census" } else { "cores" },
        o.home.display()
    ));
    xyl_core::watch::stage("chemistry");
    let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
    let chem = Chemistry::of(&bang).ok_or("no chemistry")?;
    eprintln!("universe: chemistry in {:.1?}", t0.elapsed());
    xyl_core::watch::stage("forecast");
    let elements = xyl_cosmos::elements::founded(&bang.parent.share())?;
    // the package supplies the universe's laws; the reference laws until one does
    let sc = Laws::default().scales();
    let ctx = Ctx::new(sc);
    let gal = Galaxy {
        seed: o.seed,
        bang: &bang,
        chem: &chem,
        elements: &elements,
        sc,
    };
    let schedule = forecast(&gal, &ctx, &o.home, o.stars, o.threads, o.census);
    eprintln!("universe: forecast in {:.1?}", t0.elapsed());
    xyl_core::watch::stage("genesis and begin (the bells weighed)");
    let cosmic = Cosmic::genesis(gal, schedule);
    // the heavy stars, found once and kept in the home like the forecast: the
    // bells' weights are recomputed from them, never read from the file
    let heavy_path = o.home.join(format!(
        "bells-{}-{}",
        xyl_cosmos::minds::key(&cosmic.gal),
        &xyl_cosmos::minds::laws_digest()[..16]
    ));
    let heavy: Option<Vec<u64>> = std::fs::read_to_string(&heavy_path).ok().map(|t| {
        t.split_whitespace()
            .filter_map(|w| w.parse().ok())
            .collect()
    });
    let mut u = Universe::begin(cosmic, heavy.as_deref());
    u.network.seat(o.seed, &bang, o.stars);
    if heavy.is_none() {
        let list: Vec<String> = u.field.heavy().iter().map(u64::to_string).collect();
        let _ = std::fs::create_dir_all(&o.home)
            .and_then(|_| std::fs::write(&heavy_path, list.join("\n")));
    }
    eprintln!(
        "universe: begun at count {} ({} Gyr) in {:.1?}",
        show(&u.now),
        u.gyr(),
        t0.elapsed()
    );
    let source = if let Some(c) = std::fs::read_to_string(checkpoint_path(&o.home))
        .ok()
        .and_then(|t| Checkpoint::decode(&t))
    {
        let n = c.restore(&mut u);
        eprintln!(
            "universe: checkpoint at count {}: {n} peoples' records taken, {} inputs held",
            show(&c.at),
            c.inputs.len()
        );
        Source::Checkpoint
    } else {
        let to = u.bell.count(&Q::int(14));
        u.advance_to(&to, false);
        Source::Fresh
    };
    eprintln!(
        "universe: carried to count {} ({} Gyr), {} events logged, in {:.1?}",
        show(&u.now),
        u.gyr(),
        u.log.len(),
        t0.elapsed()
    );
    xyl_core::watch::stage("read");
    f(&mut u, source, &bang)
}

/// The roster: every people with its role, home and numbers - the readings by
/// role (ancients hold the ancient seats, every other people a sovereign's).
pub(crate) fn roster(u: &Universe<'_>) -> String {
    let mut out = String::new();
    let (mut anc, mut sov) = ((0usize, Dy::ZERO), (0usize, Dy::ZERO));
    for (i, p) in u.cosmic.peoples.iter().enumerate() {
        let role = if p.ancient { "ancient" } else { "sovereign" };
        if p.ancient {
            anc = (anc.0 + 1, anc.1 + p.population);
        } else {
            sov = (sov.0 + 1, sov.1 + p.population);
        }
        out.push_str(&format!(
            "run: roster {i:>2} {role:<9} {} (star {}, world {}, {:?}): {} of housing {}\n",
            p.name, p.star, p.world, p.region, p.population, p.capacity
        ));
    }
    let stewards = u.seats.stewards().count();
    out.push_str(&format!("run: roster by role: {} ancient peoples ({} in all); {} sovereign peoples ({} in all); {stewards} steward seats", anc.0, anc.1, sov.0, sov.1));
    out
}

/// The civilizations in one line: peoples living, their numbers, what the
/// most learned knows, the pulses the stars have absorbed and the radiation
/// the peoples live in.
pub(crate) fn summary(u: &Universe<'_>) -> String {
    let living: Vec<&xyl_kernel::cosmic::People> = u
        .cosmic
        .peoples
        .iter()
        .filter(|p| p.population.signum() > 0)
        .collect();
    let numbers = living.iter().fold(Dy::ZERO, |s, p| s + p.population);
    let most = u
        .civ
        .knowledge
        .of
        .iter()
        .map(|k| k.stock.iter().fold(Dy::ZERO, |s, x| s + *x))
        .max()
        .unwrap_or(Dy::ZERO);
    let pulses: u64 = u.field.books.values().map(|b| b.pulses).sum();
    let sheared = u
        .field
        .anchors
        .values()
        .flatten()
        .map(|w| w.sheared())
        .max()
        .unwrap_or(Q::ZERO);
    let worn = (0..u.cosmic.peoples.len())
        .filter(|&i| u.field.home.get(i).copied().flatten().is_some())
        .count();
    let focused = u
        .civ
        .knowledge
        .of
        .iter()
        .filter(|k| k.focus.is_some())
        .count();
    // the most numerous people, in the terms its numbers come from: what one
    // of its bodies weighs and burns, and what its world's harvest gives
    let rulers = xyl_kernel::growth::Rulers::default();
    let numerous = u
        .cosmic
        .peoples
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.population.cmp(&b.1.population))
        .map(|(i, p)| {
            let kg = xyl_kernel::growth::body_kg(&p.traits, &rulers);
            let met = xyl_kernel::growth::metabolism_w(&p.traits, &u.cosmic.ctx, &rulers);
            let mastery = u.civ.knowledge.of.get(i).map(|k| k.mastery(xyl_kernel::knowledge::Domain::Biology)).unwrap_or(Dy::ZERO);
            format!("; the most numerous, {}: {} bodies of {} kg at {} W each, housing {} (harvest {} W), biology mastery {}", p.name, p.population, kg, met, p.capacity, p.capacity * met, mastery)
        })
        .unwrap_or_default();
    format!(
        "run: civilizations at ring {} ({} Gyr): {} of {} peoples living, {} in all; the most learned knows {} KU; {} focused by their rulers; {} pulses absorbed by {} stars, the most sheared world {} AU^2 off its orbit; {} peoples tuned to their radiation{}",
        u.ring(),
        u.gyr(),
        living.len(),
        u.cosmic.peoples.len(),
        numbers,
        most,
        focused,
        pulses,
        u.field.books.len(),
        Dy::from_q(&sheared),
        worn,
        numerous
    )
}

/// The galaxy's forecast of minds for its first `o.stars` stars, kept in the
/// universe's home: whole (`forecast-<stars>-<key>-<laws digest>`), read back
/// when nothing moved; otherwise star by star (`forecast-stars/`), each
/// star's share kept under its own inputs, so a change to the laws re-reads
/// only the stars it moved. A cache of what the laws give, never the
/// universe's truth.
pub(crate) fn forecast(
    gal: &Galaxy,
    ctx: &Ctx,
    home: &std::path::Path,
    stars: u64,
    threads: usize,
    census: bool,
) -> xyl_cosmos::minds::Schedule {
    let sight = if census {
        xyl_cosmos::minds::Sight::Census
    } else {
        xyl_cosmos::minds::Sight::Cores
    };
    let path = home.join(format!(
        "forecast-{}-{}-{}-{}",
        stars,
        if census { "census" } else { "cores" },
        xyl_cosmos::minds::key(gal),
        &xyl_cosmos::minds::laws_digest()[..16]
    ));
    if let Some(s) = std::fs::read(&path)
        .ok()
        .and_then(|b| xyl_cosmos::minds::Schedule::decode(&b))
    {
        return s;
    }
    eprintln!("run: forecasting {stars} stars on {threads} threads, star by star");
    let (s, kept) = xyl_cosmos::minds::schedule(
        gal,
        ctx,
        Some(stars),
        threads,
        sight,
        Some(&home.join("forecast-stars")),
    );
    eprintln!(
        "run: forecast: {} stars unrolled ({} worked out, {} kept), {} left compact (unobserved)",
        kept.worked + kept.kept,
        kept.worked,
        kept.kept,
        kept.compact
    );
    let tmp = path.with_extension("tmp");
    if std::fs::create_dir_all(home)
        .and_then(|_| std::fs::write(&tmp, s.encode()))
        .and_then(|_| std::fs::rename(&tmp, &path))
        .is_err()
    {
        eprintln!("run: the forecast could not be kept at {}", path.display());
    }
    s
}

/// What a network uplink does beside live play: after each ring it reads
/// and answers the universe; at each checkpoint it records what it ships;
/// it reports the last final clock height it heard. Local play has none.
pub trait Link {
    /// After each ring, with the universe's bells and their relays.
    fn ring(
        &mut self,
        u: &mut Universe<'_>,
        bells: &[xyl_cosmos::pulse::Node],
        relays: &[xyl_cosmos::pulse::Relay],
    );
    /// At each checkpoint, after it is written.
    fn saved(&mut self, home: &Path, u: &Universe<'_>) -> Result<(), String>;
    /// The last final clock height heard.
    fn height(&self) -> Option<u64>;
}

/// Live play, local: no link.
pub fn run(o: Options) -> Result<bool, String> {
    run_with(o, |_, _| Ok(None))
}

/// Live play: one ring of the bell per advance (`--for N` rings, `--save-every
/// K` rings), each event inside it at its own count, a checkpoint every K
/// rings; `link` builds the uplink beside it from the options and the stop
/// flag (None: local play).
pub fn run_with(
    o: Options,
    link: impl FnOnce(&Options, Arc<AtomicBool>) -> Result<Option<Box<dyn Link>>, String>,
) -> Result<bool, String> {
    let stop = Arc::new(AtomicBool::new(false));
    let s2 = stop.clone();
    ctrlc::set_handler(move || s2.store(true, Ordering::SeqCst)).map_err(|e| e.to_string())?;
    let mut link = link(&o, stop.clone())?;
    with(&o, |u, _, _bang| {
        eprintln!(
            "run: count {} at {} Gyr, {} peoples, {} seats",
            show(&u.now),
            u.gyr(),
            u.cosmic.peoples.len(),
            u.seats.list.len()
        );
        eprintln!("{}", summary(u));
        eprintln!("{}", roster(u));
        let bells = u.field.nodes();
        let pause =
            (o.velocity > 0).then(|| std::time::Duration::from_micros(1_000_000 / o.velocity));
        // The bell is the clock: the universe advances a ring at a time, every
        // event inside it (a waking, an input, a ley line falling due) at its
        // own count, never on a calendar's year
        let ring = Q::ONE;
        let mut rings = 0u64;
        let save = |u: &mut Universe<'_>, link: &mut Option<Box<dyn Link>>| -> Result<(), String> {
            keep(u, &o.home, link.as_ref().and_then(|l| l.height()))?;
            match link {
                Some(l) => l.saved(&o.home, u),
                None => Ok(()),
            }
        };
        while !stop.load(Ordering::SeqCst) && o.until.is_none_or(|n| rings < n) {
            let t = std::time::Instant::now();
            let to = &u.now + &ring;
            for e in u.advance_to(&to, true) {
                eprintln!("run: count {} {} {}", show(&e.at), e.kind, e.text);
            }
            rings += 1;
            if let Some(l) = link.as_mut() {
                let relays = xyl_cosmos::pulse::relays(&bells, u.gyr());
                l.ring(u, &bells, &relays);
            }
            if rings % 6 == 0 {
                eprintln!("{}", summary(u));
            }
            if rings % o.save_every.max(1) == 0 {
                save(u, &mut link)?;
            }
            if let Some(p) = pause {
                std::thread::sleep(p.saturating_sub(t.elapsed()));
            }
        }
        save(u, &mut link)?;
        eprintln!("{}", summary(u));
        eprintln!(
            "run: stopped after {rings} rings at count {}, kept",
            show(&u.now)
        );
        Ok(true)
    })
}

/// Genesis 1-3 in a new home: the Big Bang, its galaxies and their systems,
/// the forecast of minds, and the first checkpoint at the first dawn.
pub fn genesis(o: Options) -> Result<bool, String> {
    xyl_cosmos::circuit::cache_in(&o.home);
    let record = o.home.join("genesis");
    if record.exists() {
        return Err(format!(
            "{} already holds a genesis; a home is founded once",
            o.home.display()
        ));
    }
    let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
    let chem = Chemistry::of(&bang).ok_or("no chemistry")?;
    let elements = xyl_cosmos::elements::founded(&bang.parent.share())?;
    // the package supplies the universe's laws; the reference laws until one does
    let sc = Laws::default().scales();
    let ctx = Ctx::new(sc);
    let gal = Galaxy {
        seed: o.seed,
        bang: &bang,
        chem: &chem,
        elements: &elements,
        sc,
    };
    std::fs::create_dir_all(&o.home).map_err(|e| e.to_string())?;
    let schedule = forecast(&gal, &ctx, &o.home, o.stars, o.threads, o.census);
    let founding = xyl_cosmos::cores::founding(o.seed, &bang, &elements);
    let anchor = xyl_cosmos::circuit::anchor();
    let mut out = format!(
        "xylarium genesis v2\nseed {}\nfounding {}\nlaws {}\nlife {}\nbig-bang first-stars {} bell-gyr {} rim-cluster {} home-rim-kpc {} radius-kpc {} margin {}\n",
        o.seed,
        xyl_cosmos::package::founding_digest(),
        xyl_cosmos::minds::laws_digest(),
        xyl_cosmos::minds::life_digest(),
        bang.first_stars_gyr().mid(),
        anchor.bell_gyr,
        anchor.rim,
        anchor.home_rim_kpc,
        anchor.radius_kpc,
        anchor.margin
    );
    // the template: what this genesis was seeded from, and its census - how
    // much of each kind the founding made, and what the home galaxy's stars
    // hold of it
    if let Some(sub) = xyl_cosmos::cores::substrate_for(&anchor.spin) {
        out.push_str(&format!(
            "package core-suns {} spin {} share {}\ncensus founded {} held-share {} beads {} galaxies {} lit {} stellar-suns {} sample {}\n",
            xyl_cosmos::package::hole_mass(),
            anchor.spin,
            anchor.share,
            sub.holding(),
            sub.held_share(),
            sub.beads.len(),
            sub.galaxies().len(),
            sub.lighting.lit,
            xyl_cosmos::galaxy::stellar_mass(),
            xyl_cosmos::galaxy::sample(&bang),
        ));
        let all = sub.holding();
        for (z, m) in sub.census() {
            out.push_str(&format!(
                "census Z{z:03} mass {m} share {}\n",
                m.checked_div(&all).unwrap_or(Q::ZERO)
            ));
        }
    }
    for c in &founding {
        let s = &c.shape;
        out.push_str(&format!(
            "galaxy {} hole-suns {} place {} {} {} born {} recipe {} sample {}\n",
            s.index,
            s.hole_suns,
            s.place[0],
            s.place[1],
            s.place[2],
            s.born,
            c.weights.label(),
            o.stars
        ));
    }
    let l = &schedule.limits;
    out.push_str(&format!(
        "systems worlds {} life {} minds {}\n",
        l.worlds,
        l.alive + l.minds,
        l.minds
    ));
    let cosmic = Cosmic::genesis(gal, schedule);
    let mut u = Universe::begin(cosmic, None);
    u.network.seat(o.seed, &bang, o.stars);
    out.push_str(&format!(
        "bell period-gyr {}\nfirst-dawn count {}\n",
        u.bell.period_gyr, u.now
    ));
    keep(&mut u, &o.home, None)?;
    let tmp = record.with_extension("tmp");
    std::fs::write(&tmp, out)
        .and_then(|_| std::fs::rename(&tmp, &record))
        .map_err(|e| e.to_string())?;
    println!(
        "genesis: founded {} - {} galaxies, first dawn at count {}",
        o.home.display(),
        founding.len(),
        show(&u.now)
    );
    Ok(true)
}
