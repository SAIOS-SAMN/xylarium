//! The forecast of minds: for every galaxy's sample stars, which worlds
//! raise a mind, when, and the expected number of minds by each epoch.
//!
//! Inputs: `galaxy::star` (the sample star, Gyr and kpc), `systems::system`
//! (its worlds), and `life` (`system_life` for each world's chemistries and
//! lineage, `climb`, `world_at`, `foragers`, `viable`), under the
//! `worlds::Ctx` of the universe's `laws::Scales`. All quantities are `Dy`.
//!
//! A world raises a mind where a complete ladder (`life::Ladder`) is climbed
//! to its last rung before its window closes and the world at that epoch
//! feeds at least `life::viable` foragers. The climb is the expectation in
//! closed form: a rung of mean `m` under impacts at rate `mu` completes in
//! `(e^(mu m) - 1) / mu`, and a sterilising pulse falls at the reciprocal of
//! its hazard. No random draw enters, so a mind's epoch is a function of its
//! world and system alone; the expectation curve steps from 0 to 1 at it.
//!
//! - [`schedule`]: the [`Schedule`] (every [`Mind`] ordered by epoch, star
//!   and world; the curve over [`curve_points`] steps of [`curve_step`] Gyr;
//!   the [`Limits`] counting the first gate each world fails) and the
//!   [`Kept`] cost. Stars outside the [`Sight`] stay compact. Work is spread
//!   over threads; the merged result does not depend on the thread count.
//!   With a cache folder, each star's share is stored under a key of the
//!   star's inputs and the source digest ([`life_digest`]) and reused while
//!   both are unchanged.
//! - [`ancients`]: the core-born minds (within `galaxy::core_kpc` of their
//!   galactic core, [`in_core`]) that wake within [`ancient_window`] of that
//!   core's first core-born mind ([`first_core_dawn`]).
//! - [`Schedule::encode`] and [`Schedule::decode`]: exact bytes via
//!   `xyl_core::codec`; decoding refuses damaged or trailing input (`None`).
//!
//! The forecast describes the undisturbed galaxy; the running universe
//! records what minds do as annals in xyl-kernel and does not re-derive it.
//!
//! Terms: ancient, forecast (of minds), sight (docs/glossary.md).

use crate::galaxy::{sample, star};
use crate::life::{self, first_age, foragers, horizon_gyr, system_life, viable, world_at, Lineage};
use crate::systems::system;
use crate::worlds::{Ctx, Galaxy, Region};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use xyl_core::dyadic::Dy;

mod ancients;
mod kept;

pub use ancients::*;
pub use kept::*;

/// The interval between points of the expectation curve (Gyr).
pub fn curve_step() -> Dy {
    Dy::ratio(1, 4)
}

/// The number of curve points: 0 Gyr to the 30 Gyr horizon in curve steps.
pub fn curve_points() -> usize {
    121
}

/// A mind the galaxy raises, with the record of its lineage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mind {
    /// The star's index, as `galaxy::star` takes it.
    pub star: u64,
    /// The index of the mind's world within the star's system.
    pub w: usize,
    /// When the mind wakes (Gyr after the Big Bang): the last rung's time.
    pub epoch: Dy,
    /// The region of the chemistry the mind arose in.
    pub region: Region,
    /// The star's mass (suns).
    pub star_mass: Dy,
    /// The star's distance from the galaxy's core when the mind woke (kpc,
    /// in three dimensions).
    pub r_kpc: Dy,
    /// When each rung was reached (Gyr after the Big Bang).
    pub steps: Vec<Dy>,
    /// The number of impacts that set the lineage back.
    pub setbacks: u32,
    /// Whether the first rung arrived on a neighbour's ejecta.
    pub seeded: bool,
    /// Whether the world feeds a viable people on the media it offers at any
    /// probe age (`life::Living::media`), as opposed to the media at the
    /// mind's epoch.
    pub fed_settled: bool,
}

/// What stops life, world by world: the first gate each world fails.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Limits {
    /// The worlds examined.
    pub worlds: u64,
    /// Worlds that offer no solvent medium.
    pub no_solvent: u64,
    /// Worlds with media but no lineage: no chemistry copies faithfully
    /// enough to begin within the horizon.
    pub cannot_copy: u64,
    /// Lineages that fell short because their window closed at the star's
    /// end, before the horizon and before the sum of their ladder's mean
    /// waits.
    pub star_died: u64,
    /// Lineages that fell short because their world merged into its
    /// neighbour before the sum of their ladder's mean waits.
    pub merged: u64,
    /// Lineages that fell short because their surface was sterilised.
    pub sterilised: u64,
    /// The other lineages that fell short: an incomplete ladder, or a climb
    /// delayed by impacts past a window longer than the summed mean waits.
    pub too_slow: u64,
    /// Lineages that raised a mind but whose world could not feed a viable
    /// people at the mind's epoch.
    pub unfed: u64,
    /// Minds forecast (raised and fed).
    pub minds: u64,
    /// Lineages that reached at least the first rung (life began).
    pub alive: u64,
    /// Lineage counts by region.
    pub by_region: Vec<(Region, u64)>,
    /// Lineage counts indexed by the number of rungs their ladder has
    /// (0 to [`life::MAX_RUNGS`]).
    pub rungs: Vec<u64>,
}

/// The share of the forecast from one star or batch of stars: its minds, its
/// expectation curve and its limits.
#[derive(Clone, Debug, Default)]
pub struct BatchLife {
    /// The minds this batch's worlds raise.
    pub minds: Vec<Mind>,
    /// The expected minds by each curve point, from this batch's worlds.
    pub curve: Vec<Dy>,
    /// The first gates this batch's worlds fail.
    pub limits: Limits,
}

fn bump<K: PartialEq>(v: &mut Vec<(K, u64)>, key: K, by: u64) {
    match v.iter_mut().find(|(k, _)| *k == key) {
        Some(slot) => slot.1 += by,
        None => v.push((key, by)),
    }
}

/// Adds the expectation of a world's mind to `curve`. The epoch is the
/// earliest completion among the world's complete chemistries, each climbed
/// from `t0` with the sterilising pulse at its expected time, the expected
/// delay from impacts and the close of its window, without arrivals from
/// neighbours. It counts where the world at that epoch feeds at least
/// `life::viable` foragers, the same test `star_life` applies. The curve
/// steps to one from the first point at or after the epoch, or stays at
/// zero.
fn expect(l: &life::Living, born: Dy, t0: Dy, curve: &mut [Dy], ctx: &Ctx) {
    let able: Vec<&life::Chemistry> = l.chems.iter().filter(|ch| ch.ladder.complete).collect();
    if able.is_empty() {
        return;
    }
    let mut first: Option<(Dy, Region)> = None;
    for ch in &able {
        let horizon = ch
            .end
            .map(|e| e.min(horizon_gyr()))
            .unwrap_or_else(horizon_gyr);
        let steril = if ch.hazard.signum() > 0 && horizon > t0 {
            let at = t0 + Dy::ONE / ch.hazard;
            (at < horizon).then_some(at)
        } else {
            None
        };
        let means: Vec<Dy> = ch.ladder.steps.iter().map(|s| s.mean_gyr).collect();
        let got = life::climb(&means, t0, horizon, steril, ch.mu, None);
        if got.times.len() == means.len() {
            if let Some(&t) = got.times.last() {
                if first.is_none_or(|(f, _)| t < f) {
                    first = Some((t, ch.region));
                }
            }
        }
    }
    let Some((t, region)) = first else { return };
    let then = world_at(&l.setting, born, t, ctx);
    if foragers(&then, region, &l.body, ctx) >= viable() {
        for slot in curve.iter_mut().skip(ceil_points(t)) {
            *slot = Dy::ONE;
        }
    }
}

/// The first curve point at or after `t` (Gyr): ceil(t / curve_step).
fn ceil_points(t: Dy) -> usize {
    let x = t / curve_step();
    let f = x.trunc_i64();
    let c = if Dy::int(f) < x { f + 1 } else { f };
    usize::try_from(c.max(0)).unwrap_or(0)
}

/// Adds one star's share of the forecast (its minds, its expectation and its
/// limits) to `out`. A star with no worlds, or whose first reading age falls
/// past the horizon, adds nothing.
fn star_life(
    gal: &Galaxy,
    n: u64,
    star: &crate::galaxy::Star,
    sys: &crate::systems::System,
    ctx: &Ctx,
    out: &mut BatchLife,
) {
    {
        if sys.worlds.is_empty() {
            return;
        }
        let t0 = Dy::from_q(&star.born) + first_age(Dy::from_q(&star.r), ctx);
        if t0 >= horizon_gyr() {
            return;
        }
        let born = Dy::from_q(&star.born);
        let life_busy =
            xyl_core::watch::busy(format!("life of star {n} ({} worlds)", sys.worlds.len()));
        let (living, lins) = system_life(gal, star, sys, ctx);
        drop(life_busy);
        for (w, (l, lin)) in living.iter().zip(lins.iter()).enumerate() {
            let lim = &mut out.limits;
            lim.worlds += 1;
            if l.media.is_empty() {
                lim.no_solvent += 1;
            }
            let Some(lin) = lin else {
                if !l.media.is_empty() {
                    lim.cannot_copy += 1;
                }
                continue;
            };
            bump(&mut lim.by_region, lin.region, 1);
            if let Some(slot) = lim.rungs.get_mut(lin.n) {
                *slot += 1;
            }
            if !lin.steps.is_empty() {
                lim.alive += 1;
            }
            if lin.mind() {
                let epoch = lin.steps.last().copied().unwrap_or(Dy::ZERO);
                let then = world_at(&l.setting, born, epoch, ctx);
                if foragers(&then, lin.region, &l.body, ctx) >= viable() {
                    lim.minds += 1;
                    out.minds.push(Mind {
                        star: n,
                        w,
                        epoch,
                        region: lin.region,
                        star_mass: Dy::from_q(&sys.spec.mass),
                        r_kpc: from_centre_kpc(star, epoch),
                        steps: lin.steps.clone(),
                        setbacks: lin.setbacks,
                        seeded: lin.seeded,
                        fed_settled: foragers(&l.media, lin.region, &l.body, ctx) >= viable(),
                    });
                } else {
                    lim.unfed += 1;
                }
            } else if lin.sterilised {
                lim.sterilised += 1;
            } else if gate_star(lin, born) {
                if l.body.merged_at.is_some_and(|m| lin.end == Some(m)) {
                    lim.merged += 1;
                } else {
                    lim.star_died += 1;
                }
            } else {
                lim.too_slow += 1;
            }
            // The expectation: the chance that this world raises a mind by each
            // epoch.
            let _expect_busy = xyl_core::watch::busy(format!("expectation of star {n} world {w}"));
            expect(l, born, t0, &mut out.curve, ctx);
        }
    }
}

/// Whether a lineage that fell short had a window closing before the
/// horizon and shorter than the sum of its ladder's mean waits.
fn gate_star(lin: &Lineage, born: Dy) -> bool {
    match lin.end {
        Some(end) if end < horizon_gyr() => lin.means.iter().copied().sum::<Dy>() > end - born,
        _ => false,
    }
}

/// The forecast for a galaxy: every mind in order of waking, the expectation
/// curve, and the gates that stopped life.
#[derive(Clone, Debug, Default)]
pub struct Schedule {
    /// Every forecast mind, ordered by waking epoch, then star, then world.
    pub minds: Vec<Mind>,
    /// The expected number of minds by each curve point (one per
    /// [`curve_step`] from 0 Gyr; [`curve_points`] entries).
    pub curve: Vec<Dy>,
    /// Counts of the first gate each world fails, summed over all stars.
    pub limits: Limits,
}

/// Merges shares into one schedule: minds sorted by epoch, star and world,
/// and the curves and limits summed.
fn merge(shares: impl Iterator<Item = BatchLife>) -> Schedule {
    let mut out = Schedule {
        limits: Limits {
            rungs: vec![0; life::MAX_RUNGS + 1],
            ..Limits::default()
        },
        ..Schedule::default()
    };
    let mut total = vec![Dy::ZERO; curve_points()];
    for c in shares {
        out.minds.extend(c.minds);
        let (a, b) = (&mut out.limits, c.limits);
        a.worlds += b.worlds;
        a.no_solvent += b.no_solvent;
        a.cannot_copy += b.cannot_copy;
        a.star_died += b.star_died;
        a.merged += b.merged;
        a.sterilised += b.sterilised;
        a.too_slow += b.too_slow;
        a.unfed += b.unfed;
        a.minds += b.minds;
        a.alive += b.alive;
        for (rg, x) in b.by_region {
            bump(&mut a.by_region, rg, x);
        }
        for (s, x) in b.rungs.iter().enumerate() {
            if let Some(t) = a.rungs.get_mut(s) {
                *t += x;
            }
        }
        for (t, x) in total.iter_mut().zip(c.curve) {
            *t += x;
        }
    }
    out.curve = total;
    out.minds.sort_by(|a, b| {
        a.epoch
            .cmp(&b.epoch)
            .then(a.star.cmp(&b.star))
            .then(a.w.cmp(&b.w))
    });
    out
}

/// The digest of the life-side laws: the source from which a star's share
/// is computed once the star is given (its system's formation and every law
/// of its worlds and life). A star's inputs (the star as drawn and what the
/// galaxy's code reads of its place) are keyed separately per star, so a
/// change to the galaxy's code recomputes only the stars it changed;
/// computing a key costs a star's draw, never its system.
pub fn life_digest() -> String {
    // Every source file of the laws, hashed at build time (build.rs).
    env!("XYL_SOURCE_DIGEST").to_string()
}

/// The key of a star's inputs to its share (24 hex characters): the galaxy's
/// key, the star's index, the star as drawn, and what the galaxy's code reads
/// of its place (the density, the birth time and the distance from the core
/// at 2, 5, 9 and 14 Gyr).
fn star_key(gal: &Galaxy, n: u64, star: &crate::galaxy::Star) -> String {
    let mut h = blake3::Hasher::new_derive_key("xylarium 2026-10-14 star share v1");
    h.update(key(gal).as_bytes());
    h.update(&n.to_le_bytes());
    h.update(format!("{star:?}").as_bytes());
    h.update(
        format!(
            "{}|{}",
            crate::galaxy::density(&star.r).mid(),
            crate::galaxy::birth_gyr(&star.r).unwrap_or(xyl_core::Q::ZERO)
        )
        .as_bytes(),
    );
    for t in [2, 5, 9, 14] {
        h.update(format!("|{}", from_centre_kpc(star, Dy::int(t))).as_bytes());
    }
    h.finalize().to_hex().as_str()[..24].to_string()
}

/// Which stars are unrolled. A star's worlds and minds are computed only
/// when an observer with sight of it reads it; unobserved stars stay compact
/// (laws, origin and seed) and are unrolled deterministically when observed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sight {
    /// The cores' sight: the ancients hold each galaxy's reference clock, so
    /// every star within the galaxy's core (within `galaxy::core_kpc`) is
    /// observed.
    Cores,
    /// Every sample star unrolled: a census, requested explicitly.
    Census,
}

impl Sight {
    /// Whether this sight opens star `star`.
    pub fn opens(self, star: &crate::galaxy::Star) -> bool {
        match self {
            Sight::Census => true,
            Sight::Cores => star.in_core(),
        }
    }
}

/// The cost of a forecast read: stars computed, read back and left compact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Kept {
    /// Stars computed in this read.
    pub worked: u64,
    /// Stars read back from the share cache.
    pub kept: u64,
    /// Stars left compact (no observer with sight of them).
    pub compact: u64,
}

/// The forecast, star by star: for every galaxy, the first `limit` sample
/// stars and as many nucleus stars (up to the nucleus sample) that `sight`
/// opens, computed on `threads` threads and merged in order of waking. With
/// `keep`, each star's share is cached under `keep/<galaxy key>/<life
/// digest>/` by the key of the star's inputs and read back while they are
/// unchanged, so only the stars a change affected are recomputed. The cache
/// is derived data, not the universe's state.
pub fn schedule(
    gal: &Galaxy,
    ctx: &Ctx,
    limit: Option<u64>,
    threads: usize,
    sight: Sight,
    keep: Option<&std::path::Path>,
) -> (Schedule, Kept) {
    let total = limit
        .unwrap_or_else(|| sample(gal.bang))
        .min(sample(gal.bang));
    let folder = keep.map(|dir| dir.join(key(gal)).join(life_digest()));
    if let Some(f) = &folder {
        let _ = std::fs::create_dir_all(f);
    }
    // Every galaxy's first `total` sample stars and as many of its nucleus
    // sample (the core's stars, from galaxy.rs NUCLEUS_BASE, at most
    // NUCLEUS_SAMPLE): the core is where the ancients are born, and the
    // nucleus sample is placed within it.
    let nucleus = crate::galaxy::NUCLEUS_BASE
        ..crate::galaxy::NUCLEUS_BASE + total.min(crate::galaxy::NUCLEUS_SAMPLE);
    let stars: Vec<u64> = crate::cores::held(gal.seed, gal.bang)
        .into_iter()
        .flat_map(|c| {
            (0..total)
                .chain(nucleus.clone())
                .map(move |l| crate::galaxy::in_galaxy(c, l))
        })
        .collect();
    let next = AtomicUsize::new(0);
    let tally = Tally::default();
    let results: Vec<std::sync::Mutex<Option<BatchLife>>> =
        stars.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1) {
            scope.spawn(|| loop {
                let at = next.fetch_add(1, Ordering::Relaxed);
                let Some(&n) = stars.get(at) else { break };
                let _busy = xyl_core::watch::busy(format!("forecast of star {n}"));
                let share = forecast_star(gal, ctx, sight, n, folder.as_deref(), &tally);
                if let Some(slot) = results.get(at) {
                    if let Ok(mut g) = slot.lock() {
                        *g = Some(share);
                    }
                }
            });
        }
    });
    let s = merge(
        results
            .into_iter()
            .filter_map(|slot| slot.into_inner().ok().flatten()),
    );
    (
        s,
        Kept {
            worked: tally.worked.into_inner(),
            kept: tally.kept.into_inner(),
            compact: tally.compact.into_inner(),
        },
    )
}

/// A forecast's star counts, shared across the forecast's threads (see
/// [`Kept`]).
#[derive(Default)]
struct Tally {
    worked: AtomicU64,
    kept: AtomicU64,
    compact: AtomicU64,
}

/// One star's share of the forecast: read back from `folder` while the key
/// of the star's inputs is unchanged, otherwise computed from its system and
/// stored there; empty (and counted compact) where `sight` does not open the
/// star. A computation taking 30 s or more is noted on the watch.
fn forecast_star(
    gal: &Galaxy,
    ctx: &Ctx,
    sight: Sight,
    n: u64,
    folder: Option<&std::path::Path>,
    tally: &Tally,
) -> BatchLife {
    let mut share = BatchLife {
        limits: Limits {
            rungs: vec![0; life::MAX_RUNGS + 1],
            ..Limits::default()
        },
        curve: vec![Dy::ZERO; curve_points()],
        ..BatchLife::default()
    };
    let Some(st) = star(gal.seed, gal.bang, n).filter(|st| {
        let open = sight.opens(st);
        if !open {
            tally.compact.fetch_add(1, Ordering::Relaxed);
        }
        open
    }) else {
        return share;
    };
    let path = folder.map(|f| f.join(format!("{n}-{}.share", star_key(gal, n, &st))));
    let cached = path
        .as_ref()
        .and_then(|p| std::fs::read(p).ok())
        .and_then(|b| Schedule::decode(&b));
    match cached {
        Some(s) => {
            share = BatchLife {
                minds: s.minds,
                curve: s.curve,
                limits: s.limits,
            };
            tally.kept.fetch_add(1, Ordering::Relaxed);
        }
        None => {
            let t0 = std::time::Instant::now();
            if let Some(sys) = system(gal.seed, gal.bang, &[n], &st, gal.chem, gal.elements) {
                let built = t0.elapsed();
                star_life(gal, n, &st, &sys, ctx, &mut share);
                if t0.elapsed() >= std::time::Duration::from_secs(30) {
                    let msg = format!(
                        concat!(
                            "forecast: star {n} ({worlds} worlds) took {total:.1?}: ",
                            "system {built:.1?}, its life and expectations {life:.1?}"
                        ),
                        n = n,
                        worlds = sys.worlds.len(),
                        total = t0.elapsed(),
                        built = built,
                        life = t0.elapsed() - built,
                    );
                    xyl_core::watch::note(&msg);
                }
            }
            tally.worked.fetch_add(1, Ordering::Relaxed);
            if let (Some(f), Some(p)) = (folder, &path) {
                keep_share(f, n, p, &share);
            }
        }
    }
    share
}

/// Stores one star's share at `path`, removing the star's other cached shares
/// (stored under outdated keys). The bytes are written to a temporary file
/// and renamed; on failure the temporary file is removed.
fn keep_share(folder: &std::path::Path, n: u64, path: &std::path::Path, share: &BatchLife) {
    let prefix = format!("{n}-");
    if let Ok(dir) = std::fs::read_dir(folder) {
        for e in dir.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with(&prefix) && e.path() != path {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    let bytes = Schedule {
        minds: share.minds.clone(),
        curve: share.curve.clone(),
        limits: share.limits.clone(),
    }
    .encode();
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, bytes)
        .and_then(|_| std::fs::rename(&tmp, path))
        .is_err()
    {
        let _ = std::fs::remove_file(&tmp);
    }
}

#[cfg(test)]
mod tests;
