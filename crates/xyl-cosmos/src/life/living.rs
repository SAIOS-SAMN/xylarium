//! A world's chemistries and lineages, and the life of a system's worlds.

use super::*;

/// A chemistry a world's physics allows that can copy itself at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chemistry {
    /// The region of the world this chemistry lives in.
    pub region: Region,
    /// The chemistry's ladder to a mind, from its region's best medium.
    pub ladder: Ladder,
    /// The chance it is never sterilised before its window closes.
    pub live: Dy,
    /// The sterilising hazard (per Gyr): the expected time to a sterilising
    /// pulse is its reciprocal.
    pub hazard: Dy,
    /// When its window closes (Gyr after the Big Bang): its star's leaving the
    /// main sequence; never, for a sea in the dark.
    pub end: Option<Dy>,
    /// Strikes per Gyr from its system's debris (see [`strikes_per_gyr`]).
    pub mu: Dy,
    /// When its medium first exists on its world (Gyr after the Big Bang):
    /// its clock's start.
    pub start: Dy,
}

/// Where a star's worlds live: its birth, place and how much of its orbit it
/// spends in the crowded plane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Home {
    /// The star's birth (Gyr after the Big Bang).
    pub born: Dy,
    /// The star's distance from its galaxy's core (kpc).
    pub r_kpc: Dy,
    /// Share of its orbit spent inside the crowded plane layer at the settled age,
    /// in \[0, 1\].
    pub exposure: Dy,
    /// The star's mass (suns).
    pub star_mass: Dy,
    /// The star's luminosity (suns).
    pub lum: Dy,
    /// The share of its system's solids left as debris: belts and comet
    /// cloud over everything the disc gathered, in \[0, 1\].
    pub debris: Dy,
}

impl Home {
    /// The home of `star`'s worlds in `gal`, reading its birth, place and exposure
    /// (at the settled age) and its system's stellar mass and luminosity.
    pub fn of(_gal: &Galaxy, star: &Star, sys: &System) -> Home {
        Home {
            born: Dy::from_q(&star.born),
            r_kpc: Dy::from_q(&star.r),
            exposure: star.exposure(settled_gyr()),
            star_mass: Dy::from_q(&sys.spec.mass),
            lum: Dy::from_q(&sys.spec.lum),
            debris: debris_share(sys),
        }
    }
}

/// The share of a system's solids left as debris: its belts and comet cloud
/// over those plus its worlds.
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

/// Each chemistry a world's settled media allow: per region, its best medium
/// by the same rule that later feeds a people (`worlds::best`: richness x
/// energy capped at a flux unit), with its ladder, its chance never to be
/// sterilised, when its window ends and its impact rate.
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
    // a lit world's window closes at 0.9 of its star's lifetime, as the
    // star's rising luminosity on the giant branch boils its seas
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
        // the window closes at the star's end (lit media) or at the world's
        // merge into its neighbour (any medium), whichever is earlier
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
        // impacts come from the system's own debris (strikes_per_gyr);
        // sterilisation from nearby dying stars (sterilise_per_gyr)
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

/// A world's life: the furthest of its chemistries' lineages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lineage {
    /// The region of the chemistry this lineage belongs to.
    pub region: Region,
    /// When each rung was reached (Gyr after the Big Bang).
    pub steps: Vec<Dy>,
    /// Rungs its ladder has, and whether the last holds a mind.
    pub n: usize,
    /// Whether its ladder's last rung holds a mind's blueprint.
    pub complete: bool,
    /// Number of impacts that cost it progress.
    pub setbacks: u32,
    /// Whether its surface was sterilised before its window closed.
    pub sterilised: bool,
    /// Impacts beyond adaptation per Gyr it climbed under.
    pub mu: Dy,
    /// Chance it is never sterilised before its window closes, in \[0, 1\].
    pub live: Dy,
    /// When its window closes (Gyr after the Big Bang); `None` for a sea in the
    /// dark.
    pub end: Option<Dy>,
    /// Per-monomer miscopy probability of its bare chemistry, eps0.
    pub eps0: Dy,
    /// Mean wait of each rung of its ladder (Gyr).
    pub means: Vec<Dy>,
    /// Whether its first rung came on a neighbour's ejecta.
    pub seeded: bool,
}

impl Lineage {
    /// Whether it raised a mind (every rung of a complete ladder climbed).
    pub fn mind(&self) -> bool {
        self.complete && self.steps.len() == self.n
    }
}

/// A world's life as its chemistry gives it, in closed form: for each
/// chemistry, the rungs it climbs and when, and whether it was sterilised;
/// the furthest lineage (soonest, at a tie) is the world's. `seeded`: per
/// region, when life of that chemistry could arrive from a neighbour.
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
        // the sterilising pulse at its expected time, the hazard's reciprocal
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
/// settled state and media, its body and chemistries.
pub struct Living<'a> {
    /// The world's place in its system and around its star.
    pub setting: Setting<'a>,
    /// The world's state at its last probe (see [`probe_ages`]).
    pub state: State,
    /// Every medium its state offers at any probe.
    pub media: Vec<Medium>,
    /// Its body as life reads it.
    pub body: Body,
    /// The chemistries its media allow (see [`chemistries`]).
    pub chems: Vec<Chemistry>,
}

/// Every world of a system's life, with what its neighbours carry to it: rock
/// an impact throws off one world reaches the others in about a million of
/// its orbits - microbes travel, minds do not.
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
/// its medium exists at, with its ladder read from the medium as it first
/// is; `None` with no probe ages.
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
/// first lineages: a lineage's first rung plus the transit over the share of
/// dormant carriers that survive it.
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
            // survival in transit, exp(-dormant x monomers x transit), of the
            // first rung's genome; the arrival time is transit / survival
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

/// A world as it is at an epoch (Gyr after the Big Bang): its media read at
/// its star's age then, in the quarter-Gyr steps a world in play is read at -
/// for judging what a mind arising then can live on.
pub fn world_at(set: &Setting, born: Dy, epoch: Dy, ctx: &Ctx) -> Vec<Medium> {
    let quarters = ((epoch - born) * Dy::int(4) + Dy::ratio(1, 2)).trunc_i64();
    let age = (Dy::int(quarters) / Dy::int(4)).max(first_age(set.r_kpc, ctx));
    let state = build(set, age, ctx);
    media(&state, set.z, ctx)
}
