//! `xylarium observe`: prints one reading of a universe, named by a word.
//!
//! A carried reading builds the universe from its seed and checkpoint
//! exactly as `xylarium run` does on a start (`run::with`: the laws
//! regenerate what they give, the checkpoint lays back the rest), then reads
//! it. A law reading is computed from the seed without carrying the
//! universe. Neither changes the universe's state: the command writes only
//! caches and its own log into the home, so it can run beside `run`.
//!
//!   xylarium observe [--seed 7] [--stars 400] [--threads 2]
//!                    [--home var/universe-7] [--sight census] WHAT
//!
//! Carried readings:
//!
//!   universe          the bell's count, the peoples, the seats, the bells,
//!                     the books, the worlds formed, the cores (also the
//!                     reading when WHAT is empty)
//!   peoples           every people, its role, home, numbers and housing
//!   people N \[Y\]      one people: its body, numbers, power, knowledge,
//!                     radiation, world and seats; with Y, after Y years
//!                     of live play carried in memory
//!   star N            one reached star: its books, drift and worlds
//!   house             the seats minds hold and the councils
//!   world S W         world W of star S read across its life: its air,
//!                     seas, surface, field, and what it houses its people
//!   system S \[S..\]    star S's manifolds inside its own: each planet on its
//!                     star's sphere, each moon on its giant's, their caps
//!                     in degrees and their shares of belonging, and
//!                     relations read in the lowest tier holding both ends;
//!                     several stars give one line each below two planets
//!   galaxy            the units from the white hole, the bells, and sample
//!                     stars on their galactic manifolds: arc from the core,
//!                     height over the plane, phase in degrees
//!   chemistry         the galaxy's elements across the disc, then and now,
//!                     and its books (everything made is somewhere)
//!   ley \[K\]           the ley network: constellations, lines, passages,
//!                     discord and quarantine, the star map by degree cells
//!   annals \[K\]        the last K annals (default 20)
//!   events \[K\]        the last K events logged, each at its count
//!                     (default 20)
//!   clock             the bell, the keepers epicenter to rims, the bridge
//!   sync \[K\] \[Y\]      each people's sync: the forgetting share and the
//!                     quarantine it feeds (K default 10; Y as for people)
//!
//! Law readings, taken without carrying the universe:
//!
//!   core \[T\]          the substrate at T Gyr (default 14): the core's
//!                     reverberation kind by kind, its outflow, the ocean's
//!                     shares and every galaxy's intake, the books, Z Q R
//!   cores             the cut one tier up: each candidate core's weight,
//!                     reach, held or medium, ring phase and period
//!   tiers S           star S and its worlds, each turn in its anchor's rings
//!   formation         each galaxy's cut, gas, cooling and stars
//!   seals \[Z\]         the sealed kinds, the seams and the vessel (the
//!                     firmament's band) up to Z
//!   founding          the founding's equitable cascade and its stacks
//!   founding-reach    each kind's reach about the throat at the founding
//!   worlds \[N\]        the planets of the first N stars (default 200),
//!                     settled, with each star's chain (piece, cut,
//!                     manifold, worlds) and why a star stands without worlds
//!   push \[N\]          the throat's push in Z kind by kind (default 150)
//!   bonding \[N\]       the clusters the bond law closes kind by kind
//!                     (default 150)
//!
//! An unknown word prints the list of readings.
//!
//! The readings are grouped by file: `universe` (universe, cores, star,
//! house, annals, events), `peoples` (people, sync), `network` (ley),
//! `worlds` (world, system, worlds), `galaxy` (galaxy, chemistry, tiers),
//! `clock`, `atoms` (seals, push, bonding), `founding` (founding,
//! founding-reach, formation) and `substrate` (core).
//!
//! Terms: reading, books (docs/glossary.md).

mod atoms;
mod clock;
mod founding;
mod galaxy;
mod network;
mod peoples;
mod substrate;
mod universe;
mod worlds;

use crate::run::{roster, with, Options, Source};
use xyl_core::Q;
use xyl_cosmos::bang::BigBang;
use xyl_kernel::universe::Universe;

/// The list of readings printed for an unknown word.
const READINGS: &str = concat!(
    "universe, peoples, people N, star N, house, world S W, system S, galaxy, ",
    "chemistry, core [T], ley [K], annals [K], events [K], clock, tiers S, sync [K]; ",
    "law readings: formation, cores, seals [Z], founding, founding-reach, worlds [N], ",
    "push [N], bonding [N]"
);

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn count(what: &[String], default: usize) -> usize {
    what.get(1).and_then(|s| s.parse().ok()).unwrap_or(default)
}

/// Prints the reading named by `what[0]` (with its arguments in the
/// following words) for the universe described by `o`, to standard output.
/// Law readings (`formation`, `seals`, `founding-reach`, ...) are taken
/// without carrying the universe; the others carry it from its seed to its
/// checkpoint in `o.home` (or to 14 Gyr when there is none) first. An unknown
/// word prints the list of readings. Returns `Ok(true)` once a reading is
/// printed.
///
/// Returns `Err` with a message when the universe cannot be founded or
/// carried from `o` (for example a reading that cannot be founded).
///
/// # Examples
///
/// ```no_run
/// let words: Vec<String> = ["--home", "var/universe-7"].iter().map(|w| w.to_string()).collect();
/// let (o, _) = xylarium::options(&words)?;
/// let what = vec!["worlds".to_string(), "20".to_string()];
/// xylarium::observe::observe(o, &what)?;
/// # Ok::<(), String>(())
/// ```
pub fn observe(o: Options, what: &[String]) -> Result<bool, String> {
    xyl_cosmos::circuit::cache_in(&o.home);
    if let Some(out) = law_reading(&o, what)? {
        print!("{out}");
        return Ok(true);
    }
    with(&o, |u, source, _| carried(&o, u, source, what))
}

/// A law reading, evaluated from the seed without carrying the universe,
/// with its final newline; `None` when `what` names a carried reading.
fn law_reading(o: &Options, what: &[String]) -> Result<Option<String>, String> {
    let out = match what.first().map(String::as_str) {
        Some("formation") => founding::formation(o)?,
        Some("seals") => atoms::seals_read(o, count(what, xyl_cosmos::elements::TOP_Z as usize)),
        Some("founding-reach") => founding::founding_read(o)?,
        Some("worlds") => worlds::worlds_read(o, count(what, 200))?,
        Some("push") => atoms::push_reading(count(what, 150)),
        Some("bonding") => atoms::bonding(count(what, 150)),
        Some("core") => {
            // the substrate at T Gyr, every value a closed form of T
            let t = what
                .get(1)
                .and_then(|v| xyl_cosmos::elements::decimal(v))
                .unwrap_or_else(|| Q::int(14));
            substrate::core_read(o, t)?
        }
        // the cascade report already ends each line with a newline
        Some("founding") => return founding::founding_cascade(o).map(Some),
        Some("cores") => {
            let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
            universe::cores(o.seed, &bang)
        }
        Some("tiers") => {
            let n = what
                .get(1)
                .and_then(|s| s.parse().ok())
                .ok_or("observe tiers S: which star")?;
            galaxy::tiers(o, n)?
        }
        _ => return Ok(None),
    };
    Ok(Some(format!("{out}\n")))
}

/// A carried reading on the universe `with` built: optional live play for
/// `people N Y` and `sync K Y`, then the reading printed.
fn carried(
    o: &Options,
    u: &mut Universe<'_>,
    source: Source,
    what: &[String],
) -> Result<bool, String> {
    // `people N Y` and `sync K Y` carry live play Y years in memory first,
    // since the gauges are set only by a live step; nothing is written, so
    // the home on disk is unchanged
    if let (Some("people" | "sync"), Some(years)) = (
        what.first().map(String::as_str),
        what.get(2).and_then(|s| s.parse::<i64>().ok()),
    ) {
        let to = &u.now
            + &u.bell
                .count(&Q::new(years, 1_000_000_000).ok_or("observe people N Y: years")?);
        let _ = u.advance_to(&to, true);
    }
    // witnessing a star unrolls its system from the seed, so `system` reads
    // any star, in play or not
    let systems: Vec<u64> = if what.first().map(String::as_str) == Some("system") {
        what.iter().skip(1).filter_map(|s| s.parse().ok()).collect()
    } else {
        Vec::new()
    };
    for &n in &systems {
        let _ = u.cosmic.witness(n);
    }
    let u: &Universe<'_> = u;
    if source == Source::Fresh {
        println!(
            "observe: no checkpoint in {}; this is a fresh genesis carried to 14 Gyr",
            o.home.display()
        );
    }
    println!("{}", carried_reading(u, what, &systems));
    Ok(true)
}

/// The text of the carried reading `what[0]`.
fn carried_reading(u: &Universe<'_>, what: &[String], systems: &[u64]) -> String {
    match what.first().map(String::as_str) {
        Some("universe") | None => universe::universe(u),
        Some("peoples") => roster(u).replace("run: ", ""),
        Some("people") => peoples::people(u, what.get(1).and_then(|s| s.parse().ok())),
        Some("sync") => peoples::sync(u, count(what, 10)),
        Some("ley") => network::ley(u, count(what, 10)),
        Some("star") => universe::star(u, what.get(1).and_then(|s| s.parse().ok())),
        Some("house") => universe::house(u),
        Some("chemistry") => galaxy::chemistry(u),
        Some("galaxy") => galaxy::galaxy(u),
        Some("system") if systems.len() > 1 => systems_lines(u, systems),
        Some("system") => worlds::system(u, what.get(1).and_then(|s| s.parse().ok())),
        Some("world") => worlds::world(
            u,
            what.get(1).and_then(|s| s.parse().ok()),
            what.get(2).and_then(|s| s.parse().ok()),
        ),
        Some("annals") => universe::annals(u, count(what, 20)),
        Some("events") => universe::events(u, count(what, 20)),
        Some("clock") => clock::clock(u),
        Some(w) => format!("observe: nothing called '{w}' ({READINGS})"),
    }
}

/// Several stars in one reading: the full system of each star with at
/// least two planets, one line for each star with fewer.
fn systems_lines(u: &Universe<'_>, systems: &[u64]) -> String {
    systems
        .iter()
        .map(|&n| {
            let planets = u
                .cosmic
                .systems
                .get(&n)
                .map(|w| w.system.worlds.iter().filter(|x| x.host.is_none()).count());
            match planets {
                Some(k) if k >= 2 => worlds::system(u, Some(n)),
                k => format!(
                    "Star {n}: {} planets\n",
                    k.map(|k| k.to_string()).unwrap_or_else(|| "no".into())
                ),
            }
        })
        .collect()
}
