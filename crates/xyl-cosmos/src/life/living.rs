//! A world's chemistries and lineages, and the life of a system's worlds.

use super::*;

/// A chemistry that a world's physics allows and that can copy itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chemistry {
    /// The region of the world this chemistry lives in.
    pub region: Region,
    /// The chemistry's ladder to a mind, from its region's best medium.
    pub ladder: Ladder,
    /// The chance that the chemistry is never sterilised before its window
    /// closes, `exp(-hazard x window)`.
    pub live: Dy,
    /// The sterilising hazard (per Gyr); the expected time to a sterilising
    /// pulse is its reciprocal.
    pub hazard: Dy,
    /// When the chemistry's window closes (Gyr after the Big Bang): at 0.9 of
    /// the star's lifetime for a lit medium, or at the world's merger if
    /// earlier; `None` for a dark medium on a world that does not merge.
    pub end: Option<Dy>,
    /// Impacts per Gyr from the system's debris (see [`strikes_per_gyr`]).
    pub mu: Dy,
    /// When the medium first exists on the world (Gyr after the Big Bang):
    /// the start of the chemistry's climb.
    pub start: Dy,
}

/// The setting of a star's worlds: the star's birth, place, exposure to the
/// crowded plane, mass and luminosity, and the system's debris share.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Home {
    /// The star's birth (Gyr after the Big Bang).
    pub born: Dy,
    /// The star's distance from its galaxy's core (kpc).
    pub r_kpc: Dy,
    /// The share of the star's orbit spent inside the crowded plane layer at
    /// the settled age, in \[0, 1\].
    pub exposure: Dy,
    /// The star's mass (suns).
    pub star_mass: Dy,
    /// The star's luminosity (suns).
    pub lum: Dy,
    /// The share of the system's solids left as debris: belts and comet
    /// cloud over everything the disc gathered, in \[0, 1\].
    pub debris: Dy,
}

impl Home {
    /// The home of `star`'s worlds: the star's birth, place and exposure (at
    /// the settled age), and the stellar mass, luminosity and debris share of
    /// `sys`. `_gal` is not read.
    pub fn of(_gal: &Galaxy, star: &Star, sys: &System) -> Home {
        Home {
            born: crate::circuit::gyr_of_rings_dy(Dy::from_q(&star.born)),
            r_kpc: crate::galaxy::kpc_of_rings_dy(Dy::from_q(&star.r)),
            exposure: star.exposure(crate::circuit::rings_of_gyr_dy(settled_gyr())),
            star_mass: Dy::from_q(&sys.spec.mass),
            lum: Dy::from_q(&sys.spec.lum),
            debris: debris_share(sys),
        }
    }
}

/// The share of a system's solids left as debris: the mass of the belts and
/// comet cloud over that mass plus the worlds' mass; zero for an empty
/// system.
pub fn debris_share(sys: &System) -> Dy {
    let left = sys
        .belts
        .iter()
        .fold(xyl_core::Q::ZERO, |a, b| &a + &b.mass)
        + sys
            .comets
            .as_ref()
            .map(|(m, _)| m.clone())
            .unwrap_or(xyl_core::Q::ZERO);
    let gathered = sys
        .worlds
        .iter()
        .fold(xyl_core::Q::ZERO, |a, w| &a + &w.mass);
    let all = &left + &gathered;
    if all.signum() > 0 {
        Dy::from_q(&left.checked_div(&all).unwrap_or(xyl_core::Q::ZERO))
    } else {
        Dy::ZERO
    }
}

/// Each chemistry that a world's media allow: per region, the best medium by
/// the same rule that later feeds a people (`worlds::best`: richness x
/// energy capped at a flux unit), with its ladder, its chance never to be
/// sterilised, the end of its window and its impact rate. A region whose
/// ladder is `None` or has no rungs is omitted.
pub fn chemistries(ms: &[Medium], body: &Body, home: &Home, ctx: &Ctx) -> Vec<Chemistry> {
    let mut regions: Vec<Region> = Vec::new();
    for m in ms {
        if !regions.contains(&m.region) {
            regions.push(m.region);
        }
    }
    regions.sort_by_key(|rg| rg.name());
    let by: Vec<(Region, &Medium)> = regions
        .into_iter()
        .filter_map(|rg| best(ms, Some(rg)).map(|m| (rg, m)))
        .collect();
    // A lit medium's window closes at 0.9 of the star's lifetime, as the
    // star's rising luminosity on the giant branch evaporates the seas.
    let star_end = home.born + lifetime(home.r_kpc, ctx) * Dy::ratio(9, 10);
    let crowd = worlds::cosmic_rays(home.r_kpc) * home.exposure;
    let area = body.area_m2();
    let mut out = Vec::new();
    for (region, m) in by {
        let Some(lad) = ladder(m, area * m.band, ctx) else {
            continue;
        };
        if lad.steps.is_empty() {
            continue;
        }
        let dark = matches!(m.at, Where::UnderIce | Where::Deep);
        // The window closes at the star's end (lit media) or at the world's
        // merger into its neighbour (any medium), whichever is earlier.
        let end = match (dark, body.merged_at) {
            (true, None) => None,
            (true, Some(merged)) => Some(merged),
            (false, None) => Some(star_end),
            (false, Some(merged)) => Some(star_end.min(merged)),
        };
        let window = end
            .map(|e| e.min(horizon_gyr()))
            .unwrap_or_else(horizon_gyr)
            - home.born;
        // Impacts come from the system's debris (strikes_per_gyr);
        // sterilisation comes from nearby dying stars (sterilise_per_gyr).
        let rate = sterilise_per_gyr(crowd, dark);
        let hazard = rate * window;
        out.push(Chemistry {
            region,
            ladder: lad,
            live: (-hazard).exp(),
            hazard: rate,
            end,
            mu: strikes_per_gyr(home.debris, body),
            start: home.born,
        });
    }
    out
}

/// The lineage of one chemistry: the rungs it climbed and when. A world's
/// life is the furthest of its chemistries' lineages (see `lineage`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lineage {
    /// The region of the chemistry this lineage belongs to.
    pub region: Region,
    /// When each rung was reached (Gyr after the Big Bang).
    pub steps: Vec<Dy>,
    /// The number of rungs in the chemistry's ladder.
    pub n: usize,
    /// Whether the ladder's last rung holds a mind's blueprint.
    pub complete: bool,
    /// The number of impacts that cost the lineage progress.
    pub setbacks: u32,
    /// Whether the surface was sterilised before the window closed.
    pub sterilised: bool,
    /// The impact rate (per Gyr) the lineage climbed under.
    pub mu: Dy,
    /// The chance that the lineage is never sterilised before its window
    /// closes, in \[0, 1\].
    pub live: Dy,
    /// When the window closes (Gyr after the Big Bang); `None` for a dark
    /// medium on a world that does not merge.
    pub end: Option<Dy>,
    /// The per-monomer miscopy probability of the bare chemistry, `eps0`.
    pub eps0: Dy,
    /// The mean wait of each rung of the ladder (Gyr).
    pub means: Vec<Dy>,
    /// Whether the first rung arrived on a neighbour's ejecta.
    pub seeded: bool,
}

impl Lineage {
    /// Whether the lineage raised a mind (every rung of a complete ladder
    /// climbed).
    pub fn mind(&self) -> bool {
        self.complete && self.steps.len() == self.n
    }
}

/// A world's life in closed form: for each chemistry, the rungs it climbs
/// and when, and whether it was sterilised. The world's lineage is the one
/// ranked first by raising a mind, then by the share of rungs climbed, then
/// by the earliest last rung. `seeded` gives, per region, when life of that
/// chemistry could arrive from a neighbour. `key` enters no computation.
/// Returns `None` where no chemistry starts within the horizon.
pub fn lineage(key: &[u64], chems: &[Chemistry], seeded: &[(Region, Dy)]) -> Option<Lineage> {
    let _ = key;
    let mut best: Option<((bool, Dy, Dy), Lineage)> = None;
    for ch in chems {
        let t0 = ch.start;
        if t0 >= horizon_gyr() {
            continue;
        }
        let horizon = ch
            .end
            .map(|e| e.min(horizon_gyr()))
            .unwrap_or_else(horizon_gyr);
        // The sterilising pulse comes at its expected time, the hazard's
        // reciprocal after the start.
        let steril = if ch.hazard.signum() > 0 && horizon > t0 {
            let at = t0 + Dy::ONE / ch.hazard;
            (at < horizon).then_some(at)
        } else {
            None
        };
        let means: Vec<Dy> = ch.ladder.steps.iter().map(|s| s.mean_gyr).collect();
        let arrives = seeded
            .iter()
            .find(|(rg, _)| *rg == ch.region)
            .map(|(_, t)| *t);
        let got = climb(&means, t0, horizon, steril, ch.mu, arrives);
        let rec = Lineage {
            region: ch.region,
            n: means.len(),
            complete: ch.ladder.complete,
            setbacks: got.setbacks,
            sterilised: steril.is_some_and(|s| s < horizon),
            mu: ch.mu,
            live: ch.live,
            end: ch.end,
            eps0: ch.ladder.eps0,
            means,
            seeded: arrives.is_some_and(|a| got.times.first() == Some(&a.max(t0))),
            steps: got.times,
        };
        let share = Dy::int(i64::try_from(rec.steps.len()).unwrap_or(0))
            / Dy::int(i64::try_from(rec.n.max(1)).unwrap_or(1));
        let k = (
            rec.mind(),
            share,
            -rec.steps.last().copied().unwrap_or(Dy::ZERO),
        );
        if best.as_ref().is_none_or(|(bk, _)| k > *bk) {
            best = Some((k, rec));
        }
    }
    best.map(|(_, l)| l)
}

/// A world of a system, ready for its life to be read: its setting, its
/// state at the last probe, its media, its body and its chemistries.
pub struct Living<'a> {
    /// The world's place in its system and around its star.
    pub setting: Setting<'a>,
    /// The world's state at its last probe (see [`probe_ages`]).
    pub state: State,
    /// Every distinct medium (by solvent and place) the world offers at any
    /// probe.
    pub media: Vec<Medium>,
    /// The world's body as life reads it.
    pub body: Body,
    /// The chemistries the world's media allow (see [`chemistries`]).
    pub chems: Vec<Chemistry>,
}

/// The life of every world of a system. Each world's lineage is read first
/// alone; rock an impact ejects from one world reaches a sibling after
/// `transfer_gyr`, and a world that can receive life this way is read
/// again with those arrivals. Only the first rung transfers: microbes travel,
/// minds do not.
pub fn system_life<'a>(
    gal: &Galaxy,
    star: &Star,
    sys: &'a System,
    ctx: &Ctx,
) -> (Vec<Living<'a>>, Vec<Option<Lineage>>) {
    let home = Home::of(gal, star, sys);
    let path = [star.n];
    let ages = probe_ages(&home, ctx);
    let living: Vec<Living> = worlds::settings(gal, &path, sys, star)
        .into_iter()
        .filter_map(|setting| living_world(setting, &ages, &home, ctx))
        .collect();
    let key = |w: usize| [star.n, w as u64];
    let first: Vec<Option<Lineage>> = living
        .iter()
        .enumerate()
        .map(|(w, l)| lineage(&key(w), &l.chems, &[]))
        .collect();
    let arrivals = arrivals(&living, &first);
    let lins = first
        .into_iter()
        .enumerate()
        .map(|(w, f)| {
            if arrivals[w].is_empty() {
                f
            } else {
                lineage(&key(w), &living[w].chems, &arrivals[w])
            }
        })
        .collect();
    (living, lins)
}

/// A world read at each probe age: each chemistry starts at the first probe
/// at which its medium exists, with its ladder read from the medium at that
/// probe. Returns `None` when there are no probe ages.
fn living_world<'a>(
    setting: Setting<'a>,
    ages: &[Dy],
    home: &Home,
    ctx: &Ctx,
) -> Option<Living<'a>> {
    let body = Body::of(&setting);
    let (mut chems, mut all, mut last): (Vec<Chemistry>, Vec<Medium>, Option<State>) =
        (Vec::new(), Vec::new(), None);
    for &age in ages {
        let state = build(&setting, age, ctx);
        let ms = media(&state, setting.z, ctx);
        for ch in chemistries(&ms, &body, home, ctx) {
            if !chems.iter().any(|c| c.region == ch.region) {
                chems.push(Chemistry {
                    start: home.born + age,
                    ..ch
                });
            }
        }
        for m in ms {
            if !all
                .iter()
                .any(|x: &Medium| x.solvent == m.solvent && x.at == m.at)
            {
                all.push(m);
            }
        }
        last = Some(state);
    }
    let state = last?;
    Some(Living {
        setting,
        state,
        media: all,
        body,
        chems,
    })
}

/// When life of each region could arrive at each world from its siblings'
/// first lineages: a lineage's first rung plus the transit time over the
/// share of dormant carriers that survive the transit; the earliest arrival
/// per region is kept.
fn arrivals(living: &[Living], first: &[Option<Lineage>]) -> Vec<Vec<(Region, Dy)>> {
    let mut arrivals: Vec<Vec<(Region, Dy)>> = vec![Vec::new(); living.len()];
    for (w, lin) in first.iter().enumerate() {
        let Some(lin) = lin else { continue };
        let Some(&t1) = lin.steps.first() else {
            continue;
        };
        for (v, arr) in arrivals.iter_mut().enumerate() {
            if v == w {
                continue;
            }
            // The survival in transit of the first rung's genome is
            // exp(-dormant x monomers x transit), with the exponent capped at
            // 200; the arrival time is transit / survival.
            let transit = transfer_gyr(&living[v].body);
            let survival = living[w]
                .chems
                .iter()
                .find(|c| c.region == lin.region)
                .and_then(|c| c.ladder.steps.first().map(|st| (c.ladder.dormant, st.bits)))
                .map(|(dormant, bits)| {
                    let monomers = bits / bits_per_monomer();
                    (-(dormant * monomers * transit * constants().gyr_s).min(Dy::int(200))).exp()
                })
                .unwrap_or(Dy::ZERO);
            if survival <= Dy::ZERO {
                continue;
            }
            let when = t1 + transit / survival;
            match arr.iter_mut().find(|(rg, _)| *rg == lin.region) {
                Some(slot) => slot.1 = slot.1.min(when),
                None => arr.push((lin.region, when)),
            }
        }
    }
    arrivals
}

/// A world's media at an epoch (Gyr after the Big Bang), read at the star's
/// age then, rounded to the nearest quarter Gyr (the resolution at which a
/// world in play is read) and at least `first_age`. It is used to judge
/// what a mind arising at that epoch can live on.
pub fn world_at(set: &Setting, born: Dy, epoch: Dy, ctx: &Ctx) -> Vec<Medium> {
    let quarters = ((epoch - born) * Dy::int(4) + Dy::ratio(1, 2)).trunc_i64();
    let age = (Dy::int(quarters) / Dy::int(4)).max(first_age(set.r_kpc, ctx));
    let state = build(set, age, ctx);
    media(&state, set.z, ctx)
}
