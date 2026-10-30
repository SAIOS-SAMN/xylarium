//! The minds a galaxy is forecast to raise before the horizon, world by world,
//! and the expectation of minds by each epoch.
//!
//! The Big Bang gives each world its trajectory; its life is drawn on that
//! trajectory (life.rs). A mind is forecast where a complete ladder is climbed
//! to its last rung within the world's window and the world at that epoch can
//! feed a viable people (Franklin's 500). This is a forecast of the
//! undisturbed galaxy: the running universe departs
//! from it as minds act, and what happens is kept as the annals, not
//! re-derived. Stars are worked out in batches on threads.
//!
//! The climb is the chemistry's own expectation, in closed form: each rung
//! takes its mean, and an impact sends the lineage back to the rung's start,
//! so a rung of mean m under impacts at rate mu completes in
//! (e^(mu m) - 1) / mu, the memoryless restart's expectation. A sterilising
//! pulse falls at its hazard's expected time. No draw enters, so a mind's
//! epoch is a function of its world and its system alone, and the
//! expectation curve steps to one at that epoch where the world then feeds
//! a people.
//!
//! The first minds wake near the core, where the stars are oldest and
//! richest: they are the ancients (roles, layer 8).

use crate::galaxy::{sample, star};
use crate::life::{self, first_age, foragers, horizon_gyr, system_life, viable, world_at, Lineage};
use crate::systems::system;
use crate::worlds::{Ctx, Galaxy, Region};
use xyl_core::dyadic::Dy;

/// Gyr between points of the expectation curve.
pub fn curve_step() -> Dy {
    Dy::ratio(1, 4)
}

/// The curve's points: 0 to the horizon in curve steps.
pub fn curve_points() -> usize {
    121
}

/// A mind the galaxy raises.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mind {
    /// Its star (its index in the galaxy's sample) and its world.
    pub star: u64,
    /// Index of its world within the star's system.
    pub w: usize,
    /// When it wakes (Gyr after the Big Bang).
    pub epoch: Dy,
    /// The region of the chemistry the mind arose in.
    pub region: Region,
    /// Its star's mass (suns), and its distance from the core when it woke
    /// (kpc, in three dimensions).
    pub star_mass: Dy,
    /// Its star's distance from the galaxy's core when it woke (kpc, three-dimensional).
    pub r_kpc: Dy,
    /// When each rung was reached, and the impacts that set it back.
    pub steps: Vec<Dy>,
    /// Number of impacts that set its lineage back.
    pub setbacks: u32,
    /// Whether its first rung came on a neighbour's ejecta, and whether its
    /// world could feed a people at the settled age (the forecast's test).
    pub seeded: bool,
    /// Whether its world could feed a viable people at the settled age.
    pub fed_settled: bool,
}

/// What stops life, world by world: the first gate each world fails.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Limits {
    /// Worlds examined.
    pub worlds: u64,
    /// Worlds that offer no solvent medium at all.
    pub no_solvent: u64,
    /// Worlds with media but no chemistry that copies faithfully enough to begin.
    pub cannot_copy: u64,
    /// Lineages that fell short because their star's life ended before their
    /// ladder's mean climb.
    pub star_died: u64,
    /// Lineages that fell short because their world merged into its
    /// neighbour before their ladder's mean climb.
    pub merged: u64,
    /// Lineages that fell short because their surface was sterilised.
    pub sterilised: u64,
    /// Lineages that fell short by chance within an open window.
    pub too_slow: u64,
    /// Lineages that raised a mind but whose world could not then feed a viable
    /// people.
    pub unfed: u64,
    /// Minds forecast (raised and fed).
    pub minds: u64,
    /// Lineages that reached at least the first rung (life began).
    pub alive: u64,
    /// Lineages by region, and by their ladders' length.
    pub by_region: Vec<(Region, u64)>,
    /// Lineage counts indexed by the number of rungs their ladder has
    /// (0 to [`life::MAX_RUNGS`]).
    pub rungs: Vec<u64>,
}

/// One batch of stars' share: its minds, its expectation, and its limits.
#[derive(Clone, Debug, Default)]
pub struct BatchLife {
    /// The minds this batch's worlds raise.
    pub minds: Vec<Mind>,
    /// The expected minds by each curve point, from this batch's worlds.
    pub curve: Vec<Dy>,
    /// First gates this batch's worlds fail.
    pub limits: Limits,
}

fn bump<K: PartialEq>(v: &mut Vec<(K, u64)>, key: K, by: u64) {
    match v.iter_mut().find(|(k, _)| *k == key) {
        Some(slot) => slot.1 += by,
        None => v.push((key, by)),
    }
}

/// The expectation of a world's mind: the earliest of its complete
/// chemistries' climbs (the run's law: the sterilising pulse at its expected
/// time, impacts' expected delay, the window's close), counted where the
/// world at that waking can feed a people - the same test the run applies,
/// not the settled age. In closed form, no replays: the curve steps to one
/// at the epoch, or stays at zero.
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

fn ceil_points(t: Dy) -> usize {
    let x = t / curve_step();
    let f = x.trunc_i64();
    let c = if Dy::int(f) < x { f + 1 } else { f };
    usize::try_from(c.max(0)).unwrap_or(0)
}

/// One star's share of the forecast (its minds, its expectation, its
/// limits), added to `out`.
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
            // the expectation: the chance this world raises a mind by each epoch
            let _expect_busy = xyl_core::watch::busy(format!("expectation of star {n} world {w}"));
            expect(l, born, t0, &mut out.curve, ctx);
        }
    }
}

/// Whether a lineage that fell short ran out of its star's life rather than
/// out of luck: its ladder's mean is longer than its window.
fn gate_star(lin: &Lineage, born: Dy) -> bool {
    match lin.end {
        Some(end) if end < horizon_gyr() => lin.means.iter().copied().sum::<Dy>() > end - born,
        _ => false,
    }
}

/// The galaxy worked out: every mind in order of waking, the expectation
/// curve, and what stopped life.
#[derive(Clone, Debug, Default)]
pub struct Schedule {
    /// Every forecast mind, ordered by waking epoch, then star, then world.
    pub minds: Vec<Mind>,
    /// Expected number of minds by each curve point (one per [`curve_step`] from
    /// 0 Gyr; [`curve_points`] entries).
    pub curve: Vec<Dy>,
    /// Counts of the first gate each world fails, summed over all stars.
    pub limits: Limits,
}

/// Shares merged into one schedule: minds in order of waking, the curves
/// and limits summed.
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

/// Digest of the life-side laws: the source a star's share is worked out by
/// once its star is given (its system's formation and every law of its worlds
/// and life). A star's own inputs (its star as drawn, what the galaxy's code
/// reads of its place) are keyed separately per star, so a change to the
/// galaxy's code re-reads only the stars it moved; reading a key costs a
/// star's draw, never its system.
pub fn life_digest() -> String {
    // every source file of the laws, hashed at build time (build.rs)
    env!("XYL_SOURCE_DIGEST").to_string()
}

/// A star's own inputs to its share: its star as drawn, and what the
/// galaxy's code reads of its place (its crowding, its infall, its distance
/// from its core across its life).
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

/// Which stars are unrolled (compact until observed): a star's worlds and
/// minds are worked out only when something with sight of it observes it.
/// Unobserved stars stay compact (laws, origin and seed) and are unrolled
/// deterministically when observed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sight {
    /// The cores' sight: the ancients hold each galaxy's reference clock, so
    /// every star within its core (within the core's reach, galaxy::core_kpc)
    /// is observed.
    Cores,
    /// Every sample star unrolled: a census, asked for by name.
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

/// What a forecast read cost: stars worked out, stars kept.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Kept {
    /// Stars worked out fresh in this read.
    pub worked: u64,
    /// Stars read back from the share cache.
    pub kept: u64,
    /// Stars left compact (no one with sight of them).
    pub compact: u64,
}

/// The forecast, star by star: every galaxy's first `limit` sample stars that
/// `sight` opens, worked out on `threads` threads and merged in order of
/// waking. With `keep`, each star's share is cached under
/// `keep/<galaxy key>/<life digest>/` by its own inputs' key and read back
/// while they are unchanged, so only stars a change moved are recomputed.
/// The cache is derived data, not the universe's state.
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
    // every galaxy's first `total` sample stars and as many of its nucleus
    // sample (the core's own stars, galaxy.rs NUCLEUS_BASE): the core is
    // where the ancients are born, and its sample is placed within it
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
    let next = std::sync::atomic::AtomicUsize::new(0);
    let (worked, kept, compact) = (
        std::sync::atomic::AtomicU64::new(0),
        std::sync::atomic::AtomicU64::new(0),
        std::sync::atomic::AtomicU64::new(0),
    );
    let results: Vec<std::sync::Mutex<Option<BatchLife>>> =
        stars.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1) {
            scope.spawn(|| loop {
                let at = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(&n) = stars.get(at) else { break };
                let _busy = xyl_core::watch::busy(format!("forecast of star {n}"));
                let mut share = BatchLife { limits: Limits { rungs: vec![0; life::MAX_RUNGS + 1], ..Limits::default() }, curve: vec![Dy::ZERO; curve_points()], ..BatchLife::default() };
                if let Some(st) = star(gal.seed, gal.bang, n).filter(|st| {
                    let open = sight.opens(st);
                    if !open {
                        compact.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                    open
                }) {
                    let path = folder.as_ref().map(|f| f.join(format!("{n}-{}.share", star_key(gal, n, &st))));
                    match path.as_ref().and_then(|p| std::fs::read(p).ok()).and_then(|b| Schedule::decode(&b)) {
                        Some(s) => {
                            share = BatchLife { minds: s.minds, curve: s.curve, limits: s.limits };
                            kept.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        }
                        None => {
                            let t0 = std::time::Instant::now();
                            if let Some(sys) = system(gal.seed, gal.bang, &[n], &st, gal.chem, gal.elements) {
                                let built = t0.elapsed();
                                star_life(gal, n, &st, &sys, ctx, &mut share);
                                if t0.elapsed() >= std::time::Duration::from_secs(30) {
                                    xyl_core::watch::note(&format!("forecast: star {n} ({} worlds) took {:.1?}: system {built:.1?}, its life and expectations {:.1?}", sys.worlds.len(), t0.elapsed(), t0.elapsed() - built));
                                }
                            }
                            worked.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if let (Some(f), Some(p)) = (&folder, &path) {
                                keep_share(f, n, p, &share);
                            }
                        }
                    }
                }
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
            worked: worked.into_inner(),
            kept: kept.into_inner(),
            compact: compact.into_inner(),
        },
    )
}

/// Keep one star's share, dropping the star's shares kept under inputs it no
/// longer has.
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

// ---------------------------------------------------------------------- the first minds and the ancients

/// The core's assembly time (Gyr): its ring's star-formation timescale at the
/// centre, in this universe's assembly - the window within which a galaxy's
/// first minds count as its first.
pub fn ancient_window(gal: &Galaxy) -> Dy {
    Dy::from_q(
        &crate::galaxy::birth_gyr(&xyl_core::Q::ZERO)
            .unwrap_or_else(|| gal.bang.first_stars_gyr().mid()),
    )
}

/// A star's distance from the core at `t` Gyr, its reach and its climb: the
/// bulge is a ball, so a disc star swinging high over the core is outside it
/// however near its cylinder's axis.
///
/// Read in its own galaxy's tier: its bearing from its own galactic core, in
/// the home galaxy's equivalent kiloparsecs (distance over its galaxy's
/// scale), so the bulge is the same test in every galaxy.
pub fn from_centre_kpc(star: &crate::galaxy::Star, t: Dy) -> Dy {
    // distance out from the core: its reach plus its climb off the plane
    // (a path sum, not the hypotenuse)
    let (reach, _, climb) = star.bearing(t);
    let way = &reach + &climb.abs();
    Dy::from_q(
        &way.checked_div(&crate::chain::kpc())
            .unwrap_or(xyl_core::Q::ZERO),
    ) / star.scale.max(Dy::sci(1, -9))
}

/// Whether a mind was born in the galaxy's core: its star within the bulge's
/// radius of the core when it woke, in three dimensions.
pub fn in_core(m: &Mind) -> bool {
    m.r_kpc <= Dy::from_q(&crate::galaxy::core_kpc())
}

/// The ancients by birth: the
/// minds born in the core that wake within one core assembly time of the
/// core's own first mind, earliest first. Their number follows from the seed.
/// They hold the throats at the core and the uplink to the network.
///
/// Per galaxy (T1 ancients at each galactic core): a mind is ancient against
/// its own galactic core's first core-born mind.
pub fn ancients<'s>(s: &'s Schedule, gal: &Galaxy) -> Vec<&'s Mind> {
    let window = ancient_window(gal);
    s.minds
        .iter()
        .filter(|m| {
            in_core(m)
                && first_core_dawn(s, crate::galaxy::core_of(m.star))
                    .is_some_and(|d| m.epoch - d <= window)
        })
        .collect()
}

/// The first core-born mind's dawn in galactic core `core`.
pub fn first_core_dawn(s: &Schedule, core: u32) -> Option<Dy> {
    s.minds
        .iter()
        .find(|m| in_core(m) && crate::galaxy::core_of(m.star) == core)
        .map(|m| m.epoch)
}

// ---------------------------------------------------------------------- the schedule kept

/// The laws the forecast was worked out under, as a digest of their source
/// (the laws' digest): a cache kept under other laws is
/// worked out again, not reused.
pub fn laws_digest() -> String {
    // every source file of the laws, hashed at build time (build.rs)
    env!("XYL_SOURCE_DIGEST").to_string()
}

const REGIONS: [Region; 5] = [
    Region::Biotic,
    Region::Cryogenic,
    Region::Mineral,
    Region::Plasmic,
    Region::Umbral,
];

fn region_code(r: Region) -> u8 {
    u8::try_from(REGIONS.iter().position(|x| *x == r).unwrap_or(0)).unwrap_or(0)
}

fn put_dy(e: &mut xyl_core::codec::Enc, x: Dy) {
    let (m, ex) = x.to_bits();
    e.i64(m).i64(i64::from(ex));
}

fn get_dy(d: &mut xyl_core::codec::Dec) -> Option<Dy> {
    let m = d.i64()?;
    let e = i32::try_from(d.i64()?).ok()?;
    Dy::from_bits(m, e)
}

const FORECAST_FORMAT: &str = "xylarium forecast v3";

impl Schedule {
    /// The schedule as exact bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut e = xyl_core::codec::Enc::new();
        e.str(FORECAST_FORMAT).len(self.minds.len());
        for m in &self.minds {
            e.u64(m.star).u64(m.w as u64);
            put_dy(&mut e, m.epoch);
            e.u8(region_code(m.region));
            put_dy(&mut e, m.star_mass);
            put_dy(&mut e, m.r_kpc);
            e.len(m.steps.len());
            for &s in &m.steps {
                put_dy(&mut e, s);
            }
            e.u32(m.setbacks).bool(m.seeded).bool(m.fed_settled);
        }
        e.len(self.curve.len());
        for &c in &self.curve {
            put_dy(&mut e, c);
        }
        let l = &self.limits;
        for x in [
            l.worlds,
            l.no_solvent,
            l.cannot_copy,
            l.star_died,
            l.sterilised,
            l.too_slow,
            l.unfed,
            l.minds,
            l.alive,
            l.merged,
        ] {
            e.u64(x);
        }
        e.len(l.by_region.len());
        for &(r, x) in &l.by_region {
            e.u8(region_code(r)).u64(x);
        }
        e.len(l.rungs.len());
        for &x in &l.rungs {
            e.u64(x);
        }
        e.buf
    }

    /// A schedule from its bytes; `None` for anything damaged or left over.
    pub fn decode(buf: &[u8]) -> Option<Schedule> {
        let mut d = xyl_core::codec::Dec::new(buf);
        if d.str()? != FORECAST_FORMAT {
            return None;
        }
        let n = d.count()?;
        let mut minds = Vec::with_capacity(n);
        for _ in 0..n {
            let star = d.u64()?;
            let w = usize::try_from(d.u64()?).ok()?;
            let epoch = get_dy(&mut d)?;
            let region = *REGIONS.get(usize::from(d.u8()?))?;
            let star_mass = get_dy(&mut d)?;
            let r_kpc = get_dy(&mut d)?;
            let ns = d.count()?;
            let mut steps = Vec::with_capacity(ns);
            for _ in 0..ns {
                steps.push(get_dy(&mut d)?);
            }
            let (setbacks, seeded, fed_settled) = (d.u32()?, d.bool()?, d.bool()?);
            minds.push(Mind {
                star,
                w,
                epoch,
                region,
                star_mass,
                r_kpc,
                steps,
                setbacks,
                seeded,
                fed_settled,
            });
        }
        let nc = d.count()?;
        let mut curve = Vec::with_capacity(nc);
        for _ in 0..nc {
            curve.push(get_dy(&mut d)?);
        }
        let mut f = [0u64; 10];
        for x in f.iter_mut() {
            *x = d.u64()?;
        }
        let nb = d.count()?;
        let mut by_region = Vec::with_capacity(nb);
        for _ in 0..nb {
            by_region.push((*REGIONS.get(usize::from(d.u8()?))?, d.u64()?));
        }
        let nr = d.count()?;
        let mut rungs = Vec::with_capacity(nr);
        for _ in 0..nr {
            rungs.push(d.u64()?);
        }
        let limits = Limits {
            worlds: f[0],
            no_solvent: f[1],
            cannot_copy: f[2],
            star_died: f[3],
            sterilised: f[4],
            too_slow: f[5],
            unfed: f[6],
            minds: f[7],
            alive: f[8],
            merged: f[9],
            by_region,
            rungs,
        };
        d.done().then_some(Schedule {
            minds,
            curve,
            limits,
        })
    }
}

/// The key a universe's schedule is kept under: its seed, its kind of Big
/// Bang and its laws (a universe rebuilt under other laws is another).
pub fn key(gal: &Galaxy) -> String {
    let mut h = blake3::Hasher::new_derive_key("xylarium 2026-10-08 schedule key v1");
    h.update(&gal.seed.to_le_bytes());
    let s = &gal.sc;
    for x in [
        s.chem,
        s.pressure,
        s.atom,
        s.proton,
        s.density,
        s.star_mass,
        s.star_radius,
        s.nuclear,
        s.alpha,
    ] {
        let (m, e) = x.to_bits();
        h.update(&m.to_le_bytes());
        h.update(&e.to_le_bytes());
    }
    format!("seed{}-{}", gal.seed, &h.finalize().to_hex().as_str()[..12])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_schedule_round_trips_exactly_and_refuses_damage() {
        let m = Mind {
            star: 4130,
            w: 1,
            epoch: Dy::ratio(157, 100),
            region: Region::Cryogenic,
            star_mass: Dy::ratio(3, 2),
            r_kpc: Dy::ratio(17, 10),
            steps: vec![Dy::ratio(14, 10), Dy::ratio(15, 10), Dy::ratio(157, 100)],
            setbacks: 2,
            seeded: false,
            fed_settled: true,
        };
        let s = Schedule {
            minds: vec![m],
            curve: vec![Dy::ZERO, Dy::ratio(1, 64)],
            limits: Limits {
                worlds: 9,
                minds: 1,
                by_region: vec![(Region::Umbral, 4)],
                rungs: vec![0, 1, 2],
                ..Limits::default()
            },
        };
        let buf = s.encode();
        let back = Schedule::decode(&buf).expect("whole");
        assert_eq!(back.minds, s.minds);
        assert_eq!(back.curve, s.curve);
        assert_eq!(back.limits, s.limits);
        assert!(Schedule::decode(&buf[..buf.len() - 1]).is_none());
        let mut extra = buf.clone();
        extra.push(0);
        assert!(Schedule::decode(&extra).is_none());
        assert_eq!(laws_digest().len(), 16);
    }

    #[test]
    fn the_core_is_a_sphere_a_disc_star_high_over_it_is_outside() {
        let bang = crate::genesis::bang();
        let stars: Vec<crate::galaxy::Star> = (0..300)
            .filter_map(|n| crate::galaxy::star(7, bang, n))
            .collect();
        let t = Dy::int(5);
        let mut lifted = 0;
        for s in &stars {
            let d = from_centre_kpc(s, t);
            let r = Dy::from_q(&s.r);
            if s.pop == crate::galaxy::Pop::Disc {
                // never nearer than its cylinder's radius; above the plane, farther
                assert!(d >= r - Dy::sci(1, -9), "disc star {}: {d} < {r}", s.n);
                if d > r + Dy::sci(1, -6) {
                    lifted += 1;
                }
            } else {
                // a bulge or nucleus orbit is a tilted circle: its radius is its distance
                assert!(
                    (d - s.orbit.radius).abs() < Dy::sci(1, -6),
                    "star {}: {d}",
                    s.n
                );
            }
        }
        assert!(lifted > 0, "some disc star swings out of the plane");
    }
}
