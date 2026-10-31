//! Systems and worlds read per degree of a star's life: [`SystemReader`],
//! the cache key of a world read, and [`Cosmic::prepare`], which builds a
//! span's systems and worlds in parallel.

use super::{Cosmic, Witnessed};
use std::collections::BTreeMap;
use xyl_core::dyadic::Dy;
use xyl_cosmos::galaxy::star;
use xyl_cosmos::life::Body;
use xyl_cosmos::minds::Mind;
use xyl_cosmos::systems::system;
use xyl_cosmos::worlds::{
    age_of_degree, build, degree_of, media, settings, star_degree_gyr, Ctx, Galaxy, Medium, State,
};

impl Cosmic<'_> {
    /// Precomputes, in parallel, the star systems and worlds for every
    /// unwoken mind the forecast schedules by `now` (pure functions of the
    /// seed), so the minds then arise in order from cached results.
    /// [`Cosmic::wake`] calls it once per wake rather than once per mind.
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
        // Timing diagnostics: the total build time and the five slowest
        // systems.
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
}

/// Cache key for a world read: star, world index, and the star's age in
/// degrees of its life.
pub(super) fn world_key(
    systems: &BTreeMap<u64, Witnessed>,
    ctx: &Ctx,
    star_n: u64,
    w: usize,
    epoch: Dy,
) -> Option<(u64, usize, i64)> {
    let wit = systems.get(&star_n)?;
    let born = xyl_cosmos::circuit::gyr_of_rings_dy(Dy::from_q(&wit.star.born));
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
    /// Returns `None` when the star's system has not been witnessed (is not
    /// in `systems`).
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
            born: xyl_cosmos::circuit::gyr_of_rings_dy(Dy::from_q(&wit.star.born)),
            deg,
            ctx,
        })
    }

    /// The degree of the star's life at `t` Gyr.
    pub fn degree(&self, t: Dy) -> i64 {
        degree_of(t - self.born, self.deg)
    }

    /// When (Gyr) the star's life reaches `degree`.
    pub fn epoch(&self, degree: i64) -> Dy {
        self.born + age_of_degree(degree, self.deg)
    }

    /// Number of worlds in the system.
    pub fn worlds(&self) -> usize {
        self.sets.len()
    }

    /// World `w`'s state, media and body at `degree`, with the age floored
    /// at one degree. None for a world index outside the system or a world
    /// that has merged into its neighbour by that age.
    pub fn read(&self, w: usize, degree: i64) -> Option<(State, Vec<Medium>, Body)> {
        let set = self.sets.get(w)?;
        let age = age_of_degree(degree, self.deg).max(self.deg);
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

/// A world's state, media and body at the degree in its key, a pure function
/// of the seed and the key (safe on any thread).
pub(super) fn world_read(
    gal: &Galaxy,
    ctx: &Ctx,
    systems: &BTreeMap<u64, Witnessed>,
    key: (u64, usize, i64),
) -> Option<(State, Vec<Medium>, Body)> {
    let (star_n, w, degree) = key;
    SystemReader::of(gal, ctx, systems, star_n)?.read(w, degree)
}
