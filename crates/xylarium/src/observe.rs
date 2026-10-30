//! `xylarium observe` - the universe seen from the observer's standpoint.
//!
//! It builds the universe from its seed and its checkpoint exactly as
//! `xylarium run` does on a start (the laws regenerate what they give, the
//! checkpoint lays back what they do not), then reads what is asked. It never
//! changes the universe: it writes only caches and its own log into the home,
//! so it can be run beside `run` at any time.
//!
//!   xylarium observe [--seed 7] [--stars 400] [--threads 2]
//!                    [--home var/universe-7] [--sight census] WHAT
//!
//!   universe          the bell's count, the peoples, the seats, the bells,
//!                     the books, the worlds formed, the cores
//!   peoples           every people, its role, home, numbers and housing
//!   people N          one people: its body, numbers, power, knowledge,
//!                     radiation, world and seats
//!   star N            one reached star: its books, drift and worlds
//!   house             the seats minds hold and the councils
//!   world S W         world W of star S read across its life: its air,
//!                     seas, surface, field, and what it houses its people
//!   system S          star S's manifolds inside its own: each planet on its
//!                     star's sphere, each moon on its giant's, their caps
//!                     in degrees and their shares of belonging, and
//!                     relations read in the lowest tier holding both ends
//!   galaxy            the units from the white hole, the bells, and sample
//!                     stars on their galactic manifolds: arc from the core,
//!                     height over the plane, phase in degrees
//!   chemistry         the galaxy's elements across the disc, then and now,
//!                     and its books (everything made is somewhere)
//!   core \[T\]          the substrate at T Gyr: the core's reverberation
//!                     kind by kind, its outflow, the ocean's
//!                     shares and every galaxy's intake, the books, Z Q R
//!   ley \[K\]           the ley network: constellations, lines, passages,
//!                     discord and quarantine, the star map by degree cells
//!   annals \[K\]        the last K annals (default 20)
//!   events \[K\]        the last K events logged, each at its count (default 20)
//!   clock             the bell, the keepers epicenter to rims, the bridge
//!   tiers S           star S and its worlds, each turn in its anchor's rings
//!   formation         each galaxy's cut, gas, cooling and stars
//!   cores             the cut one tier up: each candidate core's weight,
//!                     reach, held or medium, ring phase and period
//!   sync \[K\]          each people's sync: the forgetting share and the
//!                     quarantine it feeds
//!
//! Law readings, taken without carrying the universe:
//!
//!   seals \[Z\]         the sealed kinds, the seams and the vessel (the
//!                     firmament's band) up to Z
//!   founding          the founding's equitable cascade and its stacks
//!   founding-reach    each kind's reach about the throat at the founding
//!   worlds \[N\]        the planets of the first N stars, settled, with each
//!                     star's chain (piece, cut, manifold, worlds) and why a
//!                     star stands without worlds
//!   push \[N\]          the throat's push in Z kind by kind
//!   bonding \[N\]       the clusters the bond law closes kind by kind

use crate::run::{roster, show, summary, with, Options, Source};
use xyl_core::dyadic::Dy;
use xyl_core::Q;
use xyl_cosmos::bang::BigBang;
use xyl_cosmos::chem::Chemistry;
use xyl_kernel::knowledge::Domain;
use xyl_kernel::seats::{Holder, On, Role};
use xyl_kernel::universe::Universe;

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
pub fn observe(o: Options, what: &[String]) -> Result<bool, String> {
    xyl_cosmos::circuit::cache_in(&o.home);
    // the laws read without the universe carried
    match what.first().map(String::as_str) {
        Some("formation") => {
            println!("{}", formation(&o)?);
            return Ok(true);
        }
        Some("seals") => {
            println!(
                "{}",
                seals_read(&o, count(what, xyl_cosmos::elements::TOP_Z as usize))
            );
            return Ok(true);
        }
        Some("founding-reach") => {
            println!("{}", founding_read(&o)?);
            return Ok(true);
        }
        Some("worlds") => {
            println!("{}", worlds_read(&o, count(what, 200))?);
            return Ok(true);
        }
        Some("push") => {
            println!("{}", push_reading(count(what, 150)));
            return Ok(true);
        }
        Some("bonding") => {
            println!("{}", bonding(count(what, 150)));
            return Ok(true);
        }
        Some("core") => {
            // the substrate: the core's reverberation, the ocean's shares and
            // every galaxy's composition at T Gyr, read in closed form
            let t = what
                .get(1)
                .and_then(|v| xyl_cosmos::elements::decimal(v))
                .unwrap_or_else(|| Q::int(14));
            println!("{}", core_read(&o, t)?);
            return Ok(true);
        }
        Some("founding") => {
            let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
            let elements = xyl_cosmos::elements::founded(&bang.parent.share())?;
            let kinds = xyl_cosmos::formation::kinds(&elements);
            let mix: Vec<(u32, Q)> = kinds.iter().map(|k| (k.z, k.share.clone())).collect();
            for r in xyl_cosmos::formation::equitable_cascade(&mix) {
                println!("Equitable cascade, form {}: spin K {}, throat H {}, final share {}; held {} kinds (Z {:?}..{:?})", r.form, show(&r.k_spin), show(&r.h_final), show(&r.s_final), r.held.len(), r.held.iter().min(), r.held.iter().max());
                for (k, (zs, push, far, deep, korb)) in r.stacks.iter().enumerate() {
                    println!("  Stack {}: Z {}..{} ({} kinds), push K + s_i {}, reach lightest {} heaviest {}, orbital speed at the heaviest {}", k + 1, zs.iter().min().unwrap_or(&0), zs.iter().max().unwrap_or(&0), zs.len(), show(push), show(far), show(deep), show(korb));
                }
            }
            let (stacks, settled) = xyl_cosmos::formation::cascade(&mix);
            println!("The founding cascade (a reading): {} kinds, {} stacks; the settled bell's round trip {}", mix.len(), stacks.len(), show(&settled));
            for (k, st) in stacks.iter().enumerate() {
                println!("  Stack {}: Z {}..{} ({} kinds), amount {}, mean hold {}, bounce {}, released spin {}, settled {}, beat {}", k + 1, st.kinds.iter().min().unwrap_or(&0), st.kinds.iter().max().unwrap_or(&0), st.kinds.len(), show(&st.amount), show(&st.w_mean), show(&st.bounce), show(&st.released), show(&st.settled), show(&st.beat));
            }
            return Ok(true);
        }
        Some("cores") => {
            let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
            println!("{}", cores(o.seed, &bang));
            return Ok(true);
        }
        Some("tiers") => {
            let n = what
                .get(1)
                .and_then(|s| s.parse().ok())
                .ok_or("observe tiers S: which star")?;
            println!("{}", tiers(&o, n)?);
            return Ok(true);
        }
        _ => {}
    }
    with(&o, |u, source, _| {
        // `people N Y`: the civilizations' layer carried live Y years in
        // memory first (its gauges are read only by a live step; nothing is
        // written - the observer stays read-only on disk)
        if let (Some("people" | "sync"), Some(years)) = (
            what.first().map(String::as_str),
            what.get(2).and_then(|s| s.parse::<i64>().ok()),
        ) {
            let to = &u.now
                + &u.bell
                    .count(&Q::new(years, 1_000_000_000).ok_or("observe people N Y: years")?);
            let _ = u.advance_to(&to, true);
        }
        // a system is read straight from the seed: any star, in play or not
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
        let out = match what.first().map(String::as_str) {
            Some("universe") | None => universe(u),
            Some("peoples") => roster(u).replace("run: ", ""),
            Some("people") => people(u, what.get(1).and_then(|s| s.parse().ok())),
            Some("sync") => sync(u, count(what, 10)),
            Some("ley") => ley(u, count(what, 10)),
            Some("star") => star(u, what.get(1).and_then(|s| s.parse().ok())),
            Some("house") => house(u),
            Some("chemistry") => chemistry(u),
            Some("galaxy") => galaxy(u),
            // several stars in one read: a star of fewer than two planets in one line
            Some("system") if systems.len() > 1 => systems
                .iter()
                .map(|&n| match u.cosmic.systems.get(&n).map(|w| w.system.worlds.iter().filter(|x| x.host.is_none()).count()) {
                    Some(k) if k >= 2 => system(u, Some(n)),
                    k => format!("Star {n}: {} planets\n", k.map(|k| k.to_string()).unwrap_or_else(|| "no".into())),
                })
                .collect::<String>(),
            Some("system") => system(u, what.get(1).and_then(|s| s.parse().ok())),
            Some("world") => world(u, what.get(1).and_then(|s| s.parse().ok()), what.get(2).and_then(|s| s.parse().ok())),
            Some("annals") => annals(u, count(what, 20)),
            Some("events") => events(u, count(what, 20)),
            Some("clock") => clock(u),
            Some(w) => format!("observe: nothing called '{w}' (universe, peoples, people N, star N, house, world S W, system S, galaxy, chemistry, core [T], ley [K], annals [K], events [K], clock, tiers S, sync [K]; law readings: formation, cores, seals [Z], founding, founding-reach, worlds [N], push [N], bonding [N])"),
        };
        println!("{out}");
        Ok(true)
    })
}

fn universe(u: &Universe<'_>) -> String {
    let living = u
        .cosmic
        .peoples
        .iter()
        .filter(|p| p.population.signum() > 0)
        .count();
    let held = u
        .seats
        .list
        .iter()
        .filter(|s| matches!(s.holder, Holder::Mind(_)))
        .count();
    let pulses: u64 = u.field.books.values().map(|b| b.pulses).sum();
    let mut out = String::new();
    out.push_str(&format!(
        "count {} of the bell ({} yr a ring)  {} Gyr\n",
        show(&u.now),
        Dy::from_q(&u.bell.period_gyr) * Dy::sci(1, 9),
        u.gyr()
    ));
    out.push_str(&format!(
        "events logged {} ({} inputs held)\n",
        u.log.len(),
        u.inputs.len() + u.pending.len()
    ));
    out.push_str(&format!(
        "peoples {} ({} living, {} ancient)\n",
        u.cosmic.peoples.len(),
        living,
        u.cosmic.peoples.iter().filter(|p| p.ancient).count()
    ));
    out.push_str(&format!(
        "seats {} ({} held by minds, {} council)\n",
        u.seats.list.len(),
        held,
        u.seats.council().count()
    ));
    out.push_str(&format!(
        "bells {}  stars reached {}  pulses absorbed {}\n",
        u.field.bells(),
        u.field.books.len(),
        pulses
    ));
    out.push_str(&format!("annals {}\n", u.cosmic.annals.len()));
    out.push_str(&summary(u).replace("run: ", ""));
    out.push('\n');
    let l = &u.cosmic.forecast.limits;
    out.push_str(&format!(
        "Worlds formed about the sample stars (every galaxy): {} worlds; life began on {} ({} alive at their horizon); kept from minds: no solvent {}, could not copy {}, star died {}, world merged {}, sterilised {}, too slow {}, unfed {}; minds {}\n",
        l.worlds, l.alive + l.minds, l.alive, l.no_solvent, l.cannot_copy, l.star_died, l.merged, l.sterilised, l.too_slow, l.unfed, l.minds
    ));
    out.push_str(&cores(u.cosmic.gal.seed, u.cosmic.gal.bang));
    out
}

/// The founding's galaxies: the home at the universe
/// core, then every bead of the cascade - its reach and phase about the
/// throat, its hole by its share of the founding's matter, its place.
fn cores(seed: u64, bang: &BigBang) -> String {
    let all = xyl_cosmos::cores::shapes(seed, bang);
    let mut out = format!(
        "The founding's galaxies: {} (the home and {} beads of the cascade)\n",
        all.len(),
        all.len().saturating_sub(1)
    );
    for s in &all {
        out.push_str(&format!(
            "  core {:>2}: hole {} suns, mass ratio {}, place {} {} {}{}\n",
            s.index,
            show(&s.hole_suns),
            show(&s.mass_ratio()),
            show(&s.place[0]),
            show(&s.place[1]),
            show(&s.place[2]),
            if s.index == 0 { " (home)" } else { "" }
        ));
    }
    out
}

fn people(u: &Universe<'_>, n: Option<usize>) -> String {
    let Some((i, p)) = n.and_then(|i| u.cosmic.peoples.get(i).map(|p| (i, p))) else {
        return format!("observe: no people {n:?} (0..{})", u.cosmic.peoples.len());
    };
    let rulers = xyl_kernel::growth::Rulers::default();
    let mut out = String::new();
    out.push_str(&format!(
        "people {i}: {}{}\n",
        p.name,
        if p.ancient { " (ancient)" } else { "" }
    ));
    out.push_str(&format!(
        "  home: star {}, world {}, region {:?}; woke {} Gyr\n",
        p.star, p.world, p.region, p.dawn
    ));
    out.push_str(&format!("  traits: {}\n", p.traits.describe()));
    out.push_str(&format!(
        "  body: {} kg at {} W\n",
        xyl_kernel::growth::body_kg(&p.traits, &rulers),
        xyl_kernel::growth::metabolism_w(&p.traits, &u.cosmic.ctx, &rulers)
    ));
    out.push_str(&format!(
        "  numbers {} of housing {}, rate {} a year\n",
        p.population, p.capacity, p.rate
    ));
    if let Some(pw) = u.cosmic.power(p) {
        out.push_str(&format!(
            "  power: bodies {} W, starlight {} W, heat {} W\n",
            pw.bodies_w, pw.starlight_w, pw.heat_w
        ));
    }
    if let Some(k) = u.civ.knowledge.of.get(i) {
        for d in Domain::ALL {
            out.push_str(&format!(
                "  knowledge {:<12} {} KU, mastery {}\n",
                d.name(),
                k.stock[d.index()],
                k.mastery(d)
            ));
        }
        out.push_str(&format!(
            "  focus: {}\n",
            k.focus.map(|d| d.name()).unwrap_or("none")
        ));
    }
    if let Some(g) = u.civ.gauges.get(i) {
        out.push_str(&format!(
            "  gauges: instinct {}, intelligence {}, sentience {}, consciousness {} -> gamma {}\n",
            g.instinct, g.intelligence, g.sentience, g.consciousness, g.gamma
        ));
    }
    if let Some(Some(h)) = u.field.home.get(i) {
        out.push_str(&format!("  home radiation (tuned to): {h}\n"));
    }
    if let Some((state, media, _)) = u.cosmic.world_at(p.star, p.world, u.cosmic.now) {
        out.push_str(&format!(
            "  world: surface {} K, {} bar, light {} flux units, age {} Gyr\n",
            state.t_surf, state.p_bar, state.flux, state.age
        ));
        for m in &media {
            out.push_str(&format!(
                "  medium: {} {:?} at {} K, energy {}\n",
                m.solvent.name(),
                m.at,
                m.t,
                m.energy
            ));
        }
    }
    for (s, seat) in u
        .seats
        .list
        .iter()
        .enumerate()
        .filter(|(_, s)| s.on == On::People(i))
    {
        out.push_str(&format!(
            "  seat {s}: {:?}, held by {}\n",
            seat.role,
            holder(&seat.holder)
        ));
    }
    out
}

/// The ley network: the constellations, the lines the ancient houses have
/// built, the passages they open, and the quarantine.
fn ley(u: &Universe<'_>, k: usize) -> String {
    use xyl_core::dyadic::Dy;
    use xyl_kernel::transit::{best_spec, home_escape, Place};
    let net = &u.network;
    let cs = &net.constellations;
    let mut sizes: Vec<usize> = cs.iter().map(|c| c.stars.len()).collect();
    sizes.sort_unstable();
    let lone = sizes.iter().filter(|&&n| n == 1).count();
    let mut out = format!(
        "The ley network at {} Gyr: {} stars in play ({} witnessed) in {} constellations ({} lone, largest {} stars, median {}); {} ley lines built, {} under way\n",
        u.gyr(),
        net.sample.len(),
        u.cosmic.systems.len(),
        cs.len(),
        lone,
        sizes.last().copied().unwrap_or(0),
        sizes.get(sizes.len() / 2).copied().unwrap_or(0),
        net.lines.len(),
        net.works.len()
    );
    let mut reach: Vec<Dy> = cs.iter().map(|c| c.reach_kpc).collect();
    reach.sort();
    if !reach.is_empty() {
        out.push_str(&format!(
            "  mouth reach (kpc) min / median / max: {} / {} / {}\n",
            reach[0],
            reach[reach.len() / 2],
            reach[reach.len() - 1]
        ));
    }
    let mut by: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for l in &net.lines {
        let name = u
            .cosmic
            .peoples
            .get(l.builder)
            .map(|p| p.name.clone())
            .unwrap_or_default();
        *by.entry(if l.progenitor {
            format!("progenitors of {name}")
        } else {
            name
        })
        .or_default() += 1;
    }
    for (name, n) in &by {
        out.push_str(&format!("  builder {name}: {n} lines\n"));
    }
    for l in net.lines.iter().take(k) {
        let arc = net
            .arc(&u.cosmic, l.from, l.to)
            .map(|d| d.to_string())
            .unwrap_or_else(|| "-".into());
        out.push_str(&format!(
            "    line {} -> {} opened {} Gyr, arc {arc} kpc, cost {} J, open {}\n",
            l.from,
            l.to,
            l.built,
            l.cost_j,
            net.open_share(l)
        ));
    }
    for (i, w) in net.works.iter().take(k) {
        out.push_str(&format!(
            "    under way: {}{} carrying a mouth {} -> {}, due {} Gyr\n",
            if i.1 { "progenitors of " } else { "" },
            u.cosmic
                .peoples
                .get(i.0)
                .map(|p| p.name.as_str())
                .unwrap_or("-"),
            w.from,
            w.to,
            w.built
        ));
    }
    // passages from each ancient house to the k nearest living peoples
    for p in u
        .cosmic
        .peoples
        .iter()
        .filter(|p| p.ancient && p.population.signum() > 0)
        .take(3)
    {
        let spec = best_spec(&u.cosmic, p);
        let esc = home_escape(&u.cosmic, p);
        let mut near: Vec<(Dy, &xyl_kernel::cosmic::People)> = u
            .cosmic
            .peoples
            .iter()
            .filter(|q| q.population.signum() > 0 && q.name != p.name)
            .filter_map(|q| u.cosmic.distance(p.star, q.star).map(|d| (d, q)))
            .collect();
        near.sort_by_key(|a| a.0);
        out.push_str(&format!("  passages from {} (star {}):\n", p.name, p.star));
        for (d, q) in near.into_iter().take(k) {
            let t = net.passage_years(
                &u.cosmic,
                &spec,
                Dy::int(xyl_kernel::propulsion::MASS_RATIO),
                esc,
                Place {
                    star: p.star,
                    world: p.world,
                },
                Place {
                    star: q.star,
                    world: q.world,
                },
                u.cosmic.peoples.iter().position(|x| std::ptr::eq(x, p)),
            );
            out.push_str(&format!(
                "    to {} (star {}, {d} kpc): {}\n",
                q.name,
                q.star,
                t.map(|y| format!("{y} years"))
                    .unwrap_or_else(|| "no lawful passage".into())
            ));
        }
    }
    // the star map by degree cells: every star in play's cell on its
    // galaxy's sphere, the cells each constellation spans, and where
    // the peoples' stars lie
    {
        let sphere = xyl_cosmos::chain::radius();
        let now = u.gyr();
        let places: std::collections::BTreeMap<u64, xyl_cosmos::chain::Place> = net
            .sample
            .iter()
            .map(|s| (s.star.n, s.star.place(now, &sphere)))
            .collect();
        let mut cells: std::collections::BTreeMap<xyl_cosmos::chain::Cell, usize> =
            std::collections::BTreeMap::new();
        for p in places.values() {
            *cells.entry(p.cell()).or_default() += 1;
        }
        let galaxies: std::collections::BTreeSet<Option<u32>> =
            cells.keys().map(|c| c.galaxy).collect();
        out.push_str(&format!(
            "  degree cells: {} occupied by {} stars in play across {} galaxies\n",
            cells.len(),
            places.len(),
            galaxies.len()
        ));
        let mut seen = std::collections::BTreeSet::new();
        for p in u
            .cosmic
            .peoples
            .iter()
            .filter(|p| p.population.signum() > 0)
        {
            if !seen.insert(p.star) {
                continue;
            }
            if seen.len() > k {
                break;
            }
            let Some(place) = places.get(&p.star) else {
                continue;
            };
            let c = place.cell();
            let con = net.of.get(&p.star).and_then(|&i| net.constellations.get(i));
            out.push_str(&format!(
                "    star {} ({}): galaxy {} cell az {} deg, polar {} deg; constellation of star {} ({} stars)\n",
                p.star,
                p.name,
                c.galaxy.map(|g| g.to_string()).unwrap_or_else(|| "-".into()),
                c.azimuth,
                c.polar,
                con.map(|c| c.key().to_string()).unwrap_or_else(|| "-".into()),
                con.map(|c| c.stars.len()).unwrap_or(0)
            ));
        }
    }
    let parts = u.discord_parts();
    let mut d: Vec<(Dy, usize)> = parts.iter().map(|(&i, x)| (x.discord, i)).collect();
    d.sort();
    d.reverse();
    let (joined, still) = (
        parts.values().filter(|x| x.joined).count(),
        parts.values().filter(|x| x.cruise.signum() <= 0).count(),
    );
    out.push_str(&format!(
        "  discord read on {} peoples (no ancient house): {} joined to the network, {} whose drives cannot leave their well; quarantine on {} constellations\n",
        parts.len(),
        joined,
        still,
        net.quarantine.iter().filter(|(_, q)| q.signum() > 0).count()
    ));
    for (x, i) in d.iter().take(k) {
        let p = &u.cosmic.peoples[*i];
        let q = &parts[i];
        out.push_str(&format!(
            "    {:<14} d {x} = reach {} kpc / (reach + mouth {} kpc) x forget {} x (1 - joined {}); v_e {} m/s, cruise {} m/s, escape {} m/s (star {})\n",
            p.name,
            q.reach_kpc,
            q.mouth_kpc,
            q.forget,
            u8::from(q.joined),
            q.v_e,
            q.cruise,
            q.escape,
            p.star
        ));
    }
    let mut q: Vec<(&u64, &Dy)> = net.quarantine.iter().collect();
    q.sort_by_key(|a| std::cmp::Reverse(a.1));
    for (key, share) in q.into_iter().take(k) {
        out.push_str(&format!(
            "    constellation of star {key}: held closed {share}\n"
        ));
    }
    out
}

/// A people in step with its universe, at scale (xyl-kernel sync.rs):
/// every living people's readings as a distribution (quartiles), then per
/// reading the `k` most out of step - each reading ranked alone, never one
/// score.
fn sync(u: &Universe<'_>, k: usize) -> String {
    use xyl_core::dyadic::Dy;
    let steps = xyl_kernel::sync::living(u);
    let living: Vec<usize> = steps.iter().map(|(i, _)| *i).collect();
    let gamma = |i: usize| u.civ.gauges.get(i).map(|g| g.gamma);
    let mut out = format!(
        "A people in step with its universe: {} living of {} peoples\n",
        living.len(),
        u.cosmic.peoples.len()
    );
    let quart = |mut v: Vec<Dy>| -> String {
        if v.is_empty() {
            return "unread".into();
        }
        v.sort();
        let at = |q: usize| v[(v.len() - 1) * q / 4];
        format!(
            "{} / {} / {} / {} / {} ({} read)",
            at(0),
            at(1),
            at(2),
            at(3),
            at(4),
            v.len()
        )
    };
    type Reader = fn(&xyl_kernel::sync::Step) -> Option<Dy>;
    let readings: [(&str, Reader, bool); 3] = [
        (
            "poles R (learn - forget, -1 forgetting .. 1 hardening)",
            |s| s.r,
            false,
        ),
        (
            "aware but frozen (sentience x (1 - consciousness))",
            |s| s.frozen,
            true,
        ),
        ("held share z (learn / (learn + forget))", |s| s.z, true),
    ];
    out.push_str(&format!(
        "  gamma: {}\n",
        quart(living.iter().filter_map(|&i| gamma(i)).collect())
    ));
    for (name, f, _) in &readings {
        out.push_str(&format!(
            "  {name}: {}\n",
            quart(steps.iter().filter_map(|(_, s)| f(s)).collect())
        ));
    }
    let line = |i: usize, v: Dy| {
        let p = &u.cosmic.peoples[i];
        format!(
            "    {i:>5} {:<14} {v}  (star {} world {}, gamma {})\n",
            p.name,
            p.star,
            p.world,
            gamma(i)
                .map(|g| g.to_string())
                .unwrap_or_else(|| "-".into())
        )
    };
    for (name, f, high) in &readings {
        let mut v: Vec<(Dy, usize)> = steps
            .iter()
            .filter_map(|(i, s)| f(s).map(|x| (x, *i)))
            .collect();
        v.sort();
        if *high {
            v.reverse();
        }
        out.push_str(&format!(
            "  the {k} most out of step by {name}{}:\n",
            if *high { ", highest" } else { ", lowest" }
        ));
        for (x, i) in v.iter().take(k) {
            out.push_str(&line(*i, *x));
        }
        if !*high {
            out.push_str("  ... and by the same, highest:\n");
            for (x, i) in v.iter().rev().take(k) {
                out.push_str(&line(*i, *x));
            }
        }
    }
    out
}

fn holder(h: &Holder) -> String {
    match h {
        Holder::Universe => "the universe".into(),
        Holder::Player(n) => format!("player {n}"),
        Holder::Mind(k) => format!("mind {}", hex(&k[..4])),
    }
}

fn star(u: &Universe<'_>, n: Option<u64>) -> String {
    let Some(n) = n else {
        return "observe: star N (a star's sample index)".into();
    };
    let mut out = format!("star {n}\n");
    match u.field.books.get(&n) {
        Some(b) => {
            out.push_str(&format!(
            "  books: in {} = gravity {} + light {} + burned out {} ({}); pulses {}; since {}\n",
            b.energy_in,
            b.gravity,
            b.light,
            b.burned_out,
            if b.balances() { "balanced" } else { "not balanced" },
            b.pulses,
            b.since.map(|t| t.to_string()).unwrap_or_else(|| "-".into())
        ))
        }
        None => out.push_str("  no pulse has reached it\n"),
    }
    if let Some(d) = u.field.drift.get(&n) {
        out.push_str(&format!("  drift: [{}, {}, {}]\n", d[0], d[1], d[2]));
    }
    if let Some(ws) = u.field.anchors.get(&n) {
        for (i, w) in ws.iter().enumerate() {
            out.push_str(&format!(
                "  world {i}: mass {}, sheared {}\n",
                w.mass,
                w.sheared()
            ));
        }
    }
    for (i, p) in u
        .cosmic
        .peoples
        .iter()
        .enumerate()
        .filter(|(_, p)| p.star == n)
    {
        out.push_str(&format!(
            "  people {i}: {} on world {} ({})\n",
            p.name, p.world, p.population
        ));
    }
    out
}

fn house(u: &Universe<'_>) -> String {
    let mut out = String::new();
    for (s, seat) in u.seats.list.iter().enumerate() {
        let on = match seat.on {
            On::People(p) => u
                .cosmic
                .peoples
                .get(p)
                .map(|x| x.name.clone())
                .unwrap_or_default(),
            other => format!("{other:?}"),
        };
        if matches!(seat.holder, Holder::Mind(_)) || seat.role == Role::Ancient {
            out.push_str(&format!(
                "seat {s}: {:?} on {on}, held by {}\n",
                seat.role,
                holder(&seat.holder)
            ));
        }
    }
    if out.is_empty() {
        out.push_str("no council seats and no seat held by a mind\n");
    }
    out
}

fn world(u: &Universe<'_>, star: Option<u64>, w: Option<usize>) -> String {
    use xyl_core::dyadic::Dy;
    let (Some(star), Some(w)) = (star, w) else {
        return "observe: world S W (a star's sample index and its world)".into();
    };
    let Some(wit) = u.cosmic.systems.get(&star) else {
        return format!("observe: star {star} is not in play");
    };
    let born = Dy::from_q(&wit.star.born);
    let people = u
        .cosmic
        .peoples
        .iter()
        .find(|p| p.star == star && p.world == w);
    let rulers = xyl_kernel::growth::Rulers::default();
    let mut out = format!(
        "world {w} of star {star} (born {born} Gyr){}\n",
        people
            .map(|p| format!(", home of {} (woke {} Gyr)", p.name, p.dawn))
            .unwrap_or_default()
    );
    out.push_str("  Gyr      surface K   bar          field        air / seas / ices                         housing\n");
    let mut t = born + Dy::ratio(1, 2);
    while t <= u.cosmic.now + Dy::ratio(1, 100) {
        if let Some((st, ms, body)) = u.cosmic.world_at(star, w, t) {
            let mix = |m: &xyl_cosmos::worlds::Mix| {
                m.iter()
                    .filter(|(_, v)| v.signum() > 0)
                    .map(|(s, _)| s.name())
                    .collect::<Vec<_>>()
                    .join(",")
            };
            let housing = people
                .and_then(|p| p.traits.medium(&ms).map(|m| (p, m)))
                .map(|(p, m)| {
                    // its housing by the one law at what it knows now
                    let k = u
                        .cosmic
                        .peoples
                        .iter()
                        .position(|q| std::ptr::eq(q, p))
                        .and_then(|i| u.civ.knowledge.of.get(i));
                    let (bio, energy) = k
                        .map(|k| {
                            (
                                k.mastery(xyl_kernel::knowledge::Domain::Biology),
                                k.mastery(xyl_kernel::knowledge::Domain::Energy),
                            )
                        })
                        .unwrap_or((Dy::ZERO, Dy::ZERO));
                    format!(
                        "{} in {} {:?}",
                        xyl_kernel::civ::housing::houses(
                            &p.traits,
                            &st,
                            &ms,
                            &body,
                            &u.cosmic.ctx,
                            &rulers,
                            bio,
                            energy
                        ),
                        m.solvent.name(),
                        m.at
                    )
                })
                .unwrap_or_else(|| {
                    // why not: each sea's phase where the world stands
                    let seas = st
                        .seas
                        .iter()
                        .map(|(s, amt)| {
                            format!(
                                "{} {:?} (amount {}, boils at {} K)",
                                s.name(),
                                u.cosmic.ctx.phase(s, st.t_surf, st.p_bar),
                                amt,
                                u.cosmic.ctx.boiling(s, st.p_bar)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        "none of its solvent; media: [{}]; seas: {seas}",
                        ms.iter()
                            .map(|m| format!("{} {:?} {:?}", m.solvent.name(), m.at, m.region))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                });
            out.push_str(&format!(
                "  {:<8} {:<11} {:<12} {:<12} {:<41} {}\n",
                t.to_string(),
                st.t_surf.to_string(),
                st.p_bar.to_string(),
                st.field.to_string(),
                format!(
                    "{} / {}{} / {}",
                    mix(&st.air),
                    mix(&st.seas),
                    st.frozen
                        .iter()
                        .map(|(s, f)| format!(" ({} {} frozen)", s.name(), f))
                        .collect::<String>(),
                    mix(&st.ices)
                ),
                housing
            ));
        }
        t += Dy::ONE;
    }
    // its own events, solved (xyl-core event.rs), over its life so far
    let sets = xyl_cosmos::worlds::settings(&u.cosmic.gal, &[star], &wit.system, &wit.star);
    if let Some(set) = sets.get(w) {
        let age = u.cosmic.now - born;
        for (what, at) in xyl_cosmos::worlds::events(set, age, &u.cosmic.ctx) {
            out.push_str(&format!(
                "  Event: {what} at {} Gyr after its star's birth ({} Gyr; bracket {})\n",
                Dy::from_q(&at.mid()),
                Dy::from_q(&at.mid()) + born,
                Dy::from_q(&(&at.hi - &at.lo))
            ));
        }
    }
    out
}

fn system(u: &Universe<'_>, n: Option<u64>) -> String {
    use xyl_core::dyadic::Dy;
    use xyl_core::units::Tier;
    use xyl_core::Q;
    use xyl_cosmos::manifold::{
        cap_degrees, galaxy_place, manifolds, share, star_cap_degrees, Anchor,
    };
    let Some(n) = n else {
        return "observe: system S (a star's sample index)".into();
    };
    let Some(wit) = u.cosmic.systems.get(&n) else {
        return format!("observe: star {n} is not in play");
    };
    let (star, sys) = (&wit.star, &wit.system);
    let units = xyl_cosmos::chain::units();
    let all = manifolds(u.cosmic.gal.seed, &[n], sys, star, &units);
    let t = u.cosmic.now;
    let age = (t - Dy::from_q(&star.born)).to_q();
    let show = |q: &Q| Dy::from_q(q).to_string();
    let mid = |e: &xyl_core::certified::Encl| show(&e.mid());
    let width = |e: &xyl_core::certified::Encl| show(&(&e.hi - &e.lo));
    let tier_m = |tier: Tier| &units.r_g_m * &tier.in_rg();
    let mut out = format!(
        "Star {n}: {} suns, born {} Gyr, age {} Gyr\n",
        show(&sys.spec.mass),
        show(&star.born),
        show(&age)
    );
    out.push_str(&format!(
        "  its cap on the core's sphere: {} deg of volume (width {})\n",
        mid(&star_cap_degrees(star, &sys.spec.mass)),
        width(&star_cap_degrees(star, &sys.spec.mass))
    ));
    let au_m = xyl_cosmos::package::Si::au();
    let who = |w: usize| {
        u.cosmic
            .peoples
            .iter()
            .find(|p| p.star == n && p.world == w)
            .map(|p| format!(" [{}]", p.name))
            .unwrap_or_default()
    };
    // the settling's marks on a world: its swing at the settled age and, where
    // it merges into its neighbour, the age it ends
    let settled = |w: usize| {
        sys.worlds
            .get(w)
            .map(|x| {
                format!(
                    ", swing {}{}",
                    show(&x.swing),
                    x.merged_at
                        .as_ref()
                        .map(|m| format!(", merges at {} Gyr", show(m)))
                        .unwrap_or_default()
                )
            })
            .unwrap_or_default()
    };
    for m in &all {
        let st = m.stand(&age, &all);
        // its reach from its anchor on its pivots, never a vector's length
        let d = xyl_core::certified::Encl::exact(m.bearing(&age, &all).r);
        let hill = m.reach();
        let deg = |turns: &xyl_core::certified::Encl| show(&(&turns.mid().frac() * &Q::int(360)));
        match m.anchor {
            Anchor::Star => {
                let a_au = (&m.a * &tier_m(Tier::System))
                    .checked_div(&au_m)
                    .unwrap_or(Q::ZERO);
                out.push_str(&format!(
                    "  Planet world {}{}: a {} system units ({} AU), e {}, mean anomaly {} deg, distance {} (width {}), cap on the star's sphere {} deg of volume{}\n",
                    m.world, who(m.world), show(&m.a), show(&a_au), mid(&st.e), deg(&st.mean), mid(&d), width(&d), mid(&cap_degrees(xyl_core::measure::BODY_N, &hill, &d)), settled(m.world)
                ));
            }
            Anchor::World(h) => {
                let a_km = (&m.a * &tier_m(Tier::Planet))
                    .checked_div(&Q::int(1000))
                    .unwrap_or(Q::ZERO);
                let host = all.iter().find(|x| x.world == h);
                // belonging at the moon: its giant's share against the star's
                let host_hill_planet = host.map(|x| {
                    x.reach().mul(&xyl_core::certified::Encl::exact(
                        xyl_cosmos::manifold::dilation(Tier::System, Tier::Planet),
                    ))
                });
                let under = host_hill_planet.map(|hh| share(xyl_core::measure::BODY_N, &d, &hh));
                out.push_str(&format!(
                    "    Moon world {}{} of world {h}: a {} planet units ({} km), chain step {}, rate 1/2^k of the innermost, mean anomaly {} deg, distance {} (width {}), cap on its giant's sphere {} deg of volume, belongs to its giant {} / to the star {}\n",
                    m.world,
                    who(m.world),
                    show(&m.a),
                    show(&a_km),
                    m.chain.map(|c| c.1.to_string()).unwrap_or_else(|| "-".into()),
                    deg(&st.mean),
                    mid(&d),
                    width(&d),
                    mid(&cap_degrees(xyl_core::measure::BODY_N, &hill, &d)),
                    under.as_ref().map(mid).unwrap_or_default(),
                    under.map(|s| show(&(&Q::ONE - &s.mid()))).unwrap_or_default()
                ));
            }
        }
    }
    // relations, each in the lowest tier holding both its ends
    out.push_str("Relations (in the lowest tier holding both ends)\n");
    let planets: Vec<usize> = all
        .iter()
        .filter(|m| m.anchor == Anchor::Star)
        .map(|m| m.world)
        .collect();
    if let [a, b, ..] = planets[..] {
        if let Some(g) = xyl_cosmos::manifold::gap(&all, a, b, &age) {
            out.push_str(&format!(
                "  world {a} to world {b}: system tier, {} units, the turn swept and the climb\n",
                show(&g)
            ));
        }
    }
    // the gaps voyages and perception read now, beside the orbits' radii
    // apart
    if let Some(&a) = planets.first() {
        out.push_str(&format!("Gaps from world {a}, as voyages and perception read them (AU now, then 1 and 5 years on; the orbits' radii apart in brackets)\n"));
        let a_of = |w: usize| {
            all.iter()
                .find(|m| m.world == w)
                .map(|m| &m.a * &tier_m(Tier::System))
                .unwrap_or(Q::ZERO)
        };
        let au_d = xyl_core::dyadic::Dy::from_q(&au_m);
        for &b in planets.iter().skip(1) {
            let place = |world| xyl_kernel::transit::Place { star: n, world };
            if let Some(g) = xyl_kernel::transit::separation_m(&u.cosmic, place(a), place(b)) {
                let flat = (&a_of(a) - &a_of(b))
                    .abs()
                    .checked_div(&au_m)
                    .unwrap_or(Q::ZERO);
                let later = |yr: i64| {
                    let at = &age + &Q::new(yr, 1_000_000_000).unwrap_or(Q::ZERO);
                    xyl_kernel::transit::orbit_gap_m(&all, a, b, &at)
                        .map(|x| (x / au_d).to_string())
                        .unwrap_or_default()
                };
                out.push_str(&format!(
                    "  to world {b}: {}, {}, {} ({})\n",
                    g / au_d,
                    later(1),
                    later(5),
                    show(&flat)
                ));
            }
        }
    }
    if let Some(moon) = all.iter().find(|m| m.anchor != Anchor::Star) {
        let d = xyl_core::certified::Encl::exact(moon.bearing(&age, &all).r);
        out.push_str(&format!(
            "  moon {} to its giant: planet tier, {} units (width {})\n",
            moon.world,
            mid(&d),
            width(&d)
        ));
    }
    let r = xyl_cosmos::chain::radius();
    let other = u.cosmic.peoples.iter().find(|p| p.star != n);
    if let (Some(&a), Some(o)) = (planets.first(), other) {
        if let Some(ow) = u.cosmic.systems.get(&o.star) {
            let oall = manifolds(u.cosmic.gal.seed, &[o.star], &ow.system, &ow.star, &units);
            if let (Some(pa), Some(pb)) = (
                galaxy_place(star, &all, a, t, &r),
                galaxy_place(&ow.star, &oall, o.world, t, &r),
            ) {
                let arc = pa.arc(&pb, &r);
                out.push_str(&format!(
                    "  world {a} to {} on star {}: galaxy tier, {} units (width {})\n",
                    o.name,
                    o.star,
                    mid(&arc),
                    width(&arc)
                ));
            }
        }
    }
    out
}

fn galaxy(u: &Universe<'_>) -> String {
    use xyl_core::dyadic::Dy;
    use xyl_core::Q;
    use xyl_cosmos::chain::{kpc, radius, units};
    let r = radius();
    let show = |q: &Q| Dy::from_q(q).to_string();
    let mut out = String::from("Units from the white hole (G = c = M_core = 1)\n");
    out.push_str(&format!(
        "  r_g = {} m; the galaxy tier's unit 60^5 r_g; 1 kpc = {} of it\n",
        show(&units().r_g_m),
        show(&kpc())
    ));
    out.push_str(&format!(
        "  the universe's radius = {} galaxy units\n",
        show(&r)
    ));
    out.push_str(&format!("Bells: {}\n", u.field.bells()));
    let t = u.cosmic.now;
    // the stars' fuel: the pulse's burn at the home's reach
    let (main, giant) = xyl_cosmos::stars::ends(Dy::ZERO);
    out.push_str(&format!(
        "Stars fuelled by the pulse: a star's rim share {} of the ring per bell {} at the core's reach; main sequence {} Gyr, giant's end {} Gyr there; remnant keeps {} of its hold; the radiation's swing within a bell now {} of its mean\n",
        show(&xyl_cosmos::stars::delta_star()),
        xyl_cosmos::stars::ring_per_bell(Dy::ZERO),
        show(&main),
        show(&giant),
        show(&xyl_cosmos::stars::remnant_share()),
        xyl_cosmos::stars::flicker(t)
    ));
    out.push_str("Stars on their manifolds now (galaxy units; their way from their own core - reach and climb - its height over its plane, phase; their reach about the universal core, life and light)\n");
    for (i, p) in u.cosmic.peoples.iter().enumerate() {
        let Some(w) = u.cosmic.systems.get(&p.star) else {
            continue;
        };
        let s = &w.star;
        // read in its galaxy's tier: its bearing from its own core
        let (reach, dir, climb) = s.bearing(t);
        let way = &reach + &climb.abs();
        let height = &(&dir[2] * &reach) + &climb;
        let frac = s.orbit.turns_at(t).to_q().frac();
        let reach = s.reach_rings();
        let age = t - Dy::from_q(&s.born);
        out.push_str(&format!(
            "  star {:>4} of core {} ({} {}): from its core {}, height {}, phase {} deg, radius {} kpc; reach {} rings, life {} Gyr, burning {:?} at age {} Gyr, light {} suns\n",
            s.n & ((1u64 << xyl_cosmos::galaxy::CORE_SHIFT) - 1),
            s.core,
            i,
            p.name,
            show(&way),
            show(&height),
            show(&(&frac * &Q::int(360))),
            s.orbit.radius,
            reach,
            show(&xyl_cosmos::stars::life(reach)),
            xyl_cosmos::field::burning(reach, age),
            age,
            show(&xyl_cosmos::stars::light(
                &w.system.spec.mass,
                reach,
                Dy::from_q(&s.born),
                age
            ))
        ));
    }
    out
}

/// Like atoms as a body, Z in Q, opened by orthogonal capacity: like
/// elements bond into a body, converging in Q over R, with Z incorporated
/// into Q (without it the read is only linear). Each open band b of the atom
/// carries u_b open frames, d_b = orientations(l) mutually orthogonal
/// directions on S^11 (its capacity), and Z's hold on it,
/// tau_b = (Z - the electrons of every shallower turning) / n:
///     Q_Z = sum u_b tau_b                 (Z in Q)
///     omega_b = u_b / d_b                 (its orthogonal capacity in use)
/// In a body's space (three planes and R, 8 like atoms touching one;
/// declared) the tightest-held frames take the contacts first, the body's
/// grip G = sum tau over its bonded frames; a frame past the contacts in a
/// band with free orthogonal room (1 - omega_b) can turn into an empty
/// orientation and flow, one in a full band is locked on the surface:
///     sigma = sum_surplus u_b (1 - omega_b) / tau_b  /  sum u_b / tau_b
/// its conduction, a share. A reading for the bench, never law.
fn bonding(top: usize) -> String {
    use xyl_core::Q;
    const CONTACT: u32 = 8;
    let show = |q: &Q| xyl_core::dyadic::Dy::from_q(q).to_string();
    let mut out = String::from(
        "Z: Q, Q_Z -> form (contacts c, grip G, conduction sigma, locked surface frames)\n",
    );
    let mut tally: std::collections::BTreeMap<&str, Vec<u32>> = std::collections::BTreeMap::new();
    let mut flowing: Vec<(Q, u32)> = Vec::new();
    for z in 1..=u32::try_from(top).unwrap_or(150) {
        let Some(a) = xyl_cosmos::atoms::atom(z) else {
            continue;
        };
        // each open frame: its band's hold and its band's free orthogonal room
        let mut frames: Vec<(Q, Q)> = Vec::new();
        for b in a.bands.iter().filter(|b| b.unpaired() > 0) {
            let shallower: u32 = a
                .bands
                .iter()
                .filter(|c| c.n < b.n)
                .map(|c| c.electrons)
                .sum();
            let tau =
                Q::new(i64::from(z) - i64::from(shallower), i64::from(b.n)).unwrap_or(Q::ZERO);
            let d = xyl_cosmos::atoms::orientations(b.l).max(1);
            let free = &Q::ONE - &Q::new(i64::from(b.unpaired()), i64::from(d)).unwrap_or(Q::ONE);
            for _ in 0..b.unpaired() {
                frames.push((tau.clone(), free.clone()));
            }
        }
        frames.sort_by(|x, y| y.0.cmp(&x.0));
        let q = u32::try_from(frames.len()).unwrap_or(0);
        let q_z = frames.iter().fold(Q::ZERO, |acc, f| &acc + &f.0);
        let c = q.min(CONTACT);
        let grip = frames
            .iter()
            .take(c as usize)
            .fold(Q::ZERO, |acc, f| &acc + &f.0);
        let loose = |f: &(Q, Q)| Q::ONE.checked_div(&f.0).unwrap_or(Q::ZERO);
        let all = frames.iter().fold(Q::ZERO, |acc, f| &acc + &loose(f));
        let flow = frames
            .iter()
            .skip(c as usize)
            .fold(Q::ZERO, |acc, f| &acc + &(&loose(f) * &f.1));
        let sigma = flow.checked_div(&all).unwrap_or(Q::ZERO);
        let locked = frames
            .iter()
            .skip(c as usize)
            .filter(|f| f.1.is_zero())
            .count();
        let form = match c {
            0 => "inert",
            1 => "molecule",
            2 => "chain",
            3 | 4 => "sheet",
            _ => "framework",
        };
        tally.entry(form).or_default().push(z);
        if sigma.signum() > 0 {
            flowing.push((sigma.clone(), z));
        }
        out.push_str(&format!(
            "  Z{z:>3}: Q {q}, Q_Z {} -> {form} (c {c}, G {}, sigma {}, locked {locked})\n",
            show(&q_z),
            show(&grip),
            show(&sigma)
        ));
    }
    for (form, zs) in &tally {
        out.push_str(&format!("{form}: {} kinds: {:?}\n", zs.len(), zs));
    }
    flowing.sort_by(|x, y| y.0.cmp(&x.0));
    out.push_str(&format!(
        "Conductors (sigma > 0): {} kinds; the best: {}\n",
        flowing.len(),
        flowing
            .iter()
            .take(12)
            .map(|(s, z)| format!("Z{z} {}", show(s)))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    out
}

/// A kind's push with Z in both poles (Z incorporated into Q; Q_Z alone
/// loses the mass in the denominator): each band b holds 3 e_b frames, u_b
/// of them open and 3 e_b - u_b held, all under Z's hold tau_b; so
///     A_Z = sum (3 e_b - u_b) tau_b,  Q_Z = sum u_b tau_b,
///     A_Z + Q_Z = 3 sum e_b tau_b      (the mass in Z; 3Z where every tau is 1)
///     p_Z = 1/2 Q_Z / (A_Z + Q_Z)      (at most 1/2)
/// read against the built push 1/2 Q / A (emission over held) and the linear
/// share 1/2 Q / (A + Q). A reading for the bench, never law.
fn push_reading(top: usize) -> String {
    use xyl_core::Q;
    let show = |q: &Q| xyl_core::dyadic::Dy::from_q(q).to_string();
    let mut out = String::from(
        "Z: p built (1/2 Q/A), p linear (1/2 Q/(A+Q)), p_Z (1/2 Q_Z/(A_Z+Q_Z)), p_Z / p built\n",
    );
    let mut ratios: Vec<(Q, u32)> = Vec::new();
    let mut max_pz = (Q::ZERO, 0);
    for z in 1..=u32::try_from(top).unwrap_or(150) {
        let Some(a) = xyl_cosmos::atoms::atom(z) else {
            continue;
        };
        let (mut a_z, mut q_z) = (Q::ZERO, Q::ZERO);
        for b in &a.bands {
            let shallower: u32 = a
                .bands
                .iter()
                .filter(|c| c.n < b.n)
                .map(|c| c.electrons)
                .sum();
            let tau =
                Q::new(i64::from(z) - i64::from(shallower), i64::from(b.n)).unwrap_or(Q::ZERO);
            let (held, open) = (3 * b.electrons - b.unpaired(), b.unpaired());
            a_z = &a_z + &(&tau * &Q::int(i64::from(held)));
            q_z = &q_z + &(&tau * &Q::int(i64::from(open)));
        }
        // the built push reads the electrons' open frames over the hold (R's
        // domain)
        let (open, hold) = (
            Q::int(i64::from(xyl_cosmos::compound::open(z))),
            xyl_cosmos::formation::weight(z),
        );
        let half = Q::new(1, 2).unwrap_or(Q::ZERO);
        let p_built = &half * &open.checked_div(&hold).unwrap_or(Q::ZERO);
        let p_lin = &half * &open.checked_div(&(&open + &hold)).unwrap_or(Q::ZERO);
        let p_z = &half * &q_z.checked_div(&(&a_z + &q_z)).unwrap_or(Q::ZERO);
        if p_z > max_pz.0 {
            max_pz = (p_z.clone(), z);
        }
        let ratio = p_z.checked_div(&p_built).unwrap_or(Q::ZERO);
        if p_built.signum() > 0 {
            ratios.push((ratio.clone(), z));
        }
        out.push_str(&format!(
            "  Z{z:>3}: {} {} {} ({})\n",
            show(&p_built),
            show(&p_lin),
            show(&p_z),
            show(&ratio)
        ));
    }
    ratios.sort();
    let pick = |f: usize| {
        ratios
            .get(f * ratios.len() / 10)
            .map(|(r, z)| format!("{} (Z{z})", show(r)))
            .unwrap_or_default()
    };
    out.push_str(&format!(
        "p_Z / p built over open kinds: min {} p50 {} p90 {} max {}; the largest p_Z {} at Z{}\n",
        pick(0),
        pick(5),
        pick(9),
        ratios
            .last()
            .map(|(r, z)| format!("{} (Z{z})", show(r)))
            .unwrap_or_default(),
        show(&max_pz.0),
        max_pz.1
    ));
    out
}

/// The worlds as the settling leaves them: the first `n` sample stars of the
/// home galaxy built straight from the seed - each system's planets, their
/// orbits (AU), masses (world units) and the ratio between neighbours' orbits -
/// read before any genesis.
fn worlds_read(o: &Options, n: usize) -> Result<String, String> {
    use xyl_core::dyadic::Dy;
    use xyl_core::Q;
    let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
    let elements = xyl_cosmos::elements::founded(&bang.parent.share())?;
    let chem = xyl_cosmos::chem::Chemistry::of(&bang).ok_or("no chemistry")?;
    let (mut counts, mut ratios, mut rows) = (Vec::new(), Vec::new(), String::new());
    let (mut belts, mut moons, mut giants, mut comets) = (0usize, 0usize, 0usize, 0usize);
    // why a star stands without worlds: its ring never lit (no star), no
    // frame at it, its gathering filling its manifold, no matter holding its
    // region against the core's twist, or its worlds all settled away
    let (mut unlit, mut unframed, mut filled, mut twisted, mut settled_out) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    for k in 0..u64::try_from(n).unwrap_or(0) {
        let Some(st) = xyl_cosmos::galaxy::star(o.seed, &bang, k) else {
            unlit += 1;
            continue;
        };
        let Some(sys) = xyl_cosmos::systems::system(o.seed, &bang, &[k], &st, &chem, &elements)
        else {
            continue;
        };
        let mut planets: Vec<(Q, Q, bool)> = sys
            .worlds
            .iter()
            .filter(|w| w.host.is_none())
            .map(|w| (w.a.clone(), w.mass.clone(), w.giant))
            .collect();
        planets.sort_by(|a, b| a.0.cmp(&b.0));
        if planets.is_empty() {
            match xyl_cosmos::galaxy::disc_frame(&st.r, &st.mass) {
                None => unframed += 1,
                Some(f) if f.star_share >= Q::ONE => filled += 1,
                Some(f)
                    if xyl_cosmos::disc::manifold_worlds(&f.star_share, &f.a_disc, &f.tide)
                        .0
                        .is_empty() =>
                {
                    twisted += 1
                }
                Some(_) => settled_out += 1,
            }
        }
        counts.push(planets.len());
        belts += sys.belts.len();
        moons += sys.worlds.iter().filter(|w| w.host.is_some()).count();
        giants += sys.worlds.iter().filter(|w| w.giant).count();
        comets += usize::from(sys.comets.is_some());
        for w in planets.windows(2) {
            ratios.push(Dy::from_q(&w[1].0.checked_div(&w[0].0).unwrap_or(Q::ONE)));
        }
        if k < 12 {
            // the chain's frame at the star: its piece, the infall's extent,
            // the star's share, the tide (the core-star tie) and the regions
            let frame = xyl_cosmos::galaxy::disc_frame(&st.r, &st.mass)
                .map(|f| {
                    let (regions, belt) = xyl_cosmos::disc::manifold_worlds(
                        &f.star_share,
                        &f.a_disc,
                        &f.tide,
                    );
                    let au = |kpc: &Q| Dy::from_q(&(kpc * &xyl_cosmos::package::Si::kpc()).checked_div(&xyl_cosmos::package::Si::au()).unwrap_or(Q::ZERO));
                    format!(
                        "piece {} at {} kpc: infall {} AU holding {}, the star's gathering {} AU, its manifold's reach {} AU; cut at spin {} heat {} over-pressure {}: star's share {}, tide {}, agitation {}; {} worlds on the manifold (first at {} of the reach), belt {}",
                        f.piece,
                        Dy::from_q(&st.r),
                        au(&f.infall_kpc),
                        Dy::from_q(&f.cut.cloud),
                        au(&f.extent_kpc),
                        au(&f.reach_kpc),
                        Dy::from_q(&f.cut.spin),
                        Dy::from_q(&f.cut.heat),
                        Dy::from_q(&f.cut.p0),
                        Dy::from_q(&f.star_share),
                        Dy::from_q(&f.tide),
                        Dy::from_q(&f.a_disc),
                        regions.len(),
                        regions.first().map(|r| Dy::from_q(&r.0).to_string()).unwrap_or_else(|| "-".into()),
                        Dy::from_q(&belt)
                    )
                })
                .unwrap_or_else(|| format!("no frame at {} kpc", Dy::from_q(&st.r)));
            rows.push_str(&format!("    chain: {frame}\n"));
            rows.push_str(&format!(
                "  star {k} ({} suns): {} planets: {}\n",
                Dy::from_q(&sys.spec.mass),
                planets.len(),
                planets
                    .iter()
                    .map(|(a, m, g)| format!(
                        "{} AU {} E{}",
                        Dy::from_q(a),
                        Dy::from_q(m),
                        if *g { " giant" } else { "" }
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    counts.sort_unstable();
    ratios.sort();
    let at = |v: &[usize], f: usize| v.get(f * v.len() / 10).copied().unwrap_or(0);
    let rt = |f: usize| {
        ratios
            .get(f * ratios.len() / 10)
            .map(|x| x.to_string())
            .unwrap_or_default()
    };
    let total: usize = counts.iter().sum();
    Ok(format!(
        "The worlds, the settling over {} stars: planets per star min {} p10 {} p50 {} p90 {} max {} (total {total}, {} without); giants {giants}, moons {moons}, belts {belts}, comet clouds {comets}\n  without worlds: {unlit} never lit, {unframed} unframed, {filled} gathering fills the manifold, {twisted} nothing holds against the twist, {settled_out} settled away\n  neighbours' orbit ratio: min {} p10 {} p50 {} p90 {}\n{rows}",
        counts.len(),
        counts.first().copied().unwrap_or(0),
        at(&counts, 1),
        at(&counts, 5),
        at(&counts, 9),
        counts.last().copied().unwrap_or(0),
        counts.iter().filter(|c| **c == 0).count(),
        ratios.first().map(|x| x.to_string()).unwrap_or_default(),
        rt(1),
        rt(5),
        rt(9)
    ))
}

fn chemistry(u: &Universe<'_>) -> String {
    use xyl_core::Q;
    let chem = u.cosmic.gal.chem;
    let q = |s: &str| xyl_cosmos::elements::decimal(s).unwrap_or(Q::ZERO);
    let show = |v: Option<Q>| {
        v.map(|x| xyl_core::dyadic::Dy::from_q(&x).to_string())
            .unwrap_or_else(|| "-".into())
    };
    let mut out = String::from("relative to the Sun's birth gas (1 = as the Sun was made)\n");
    for (label, t) in [("today", "13.8"), ("at 9.2 Gyr", "9.2"), ("at 3 Gyr", "3")] {
        out.push_str(&format!("  {label}:\n"));
        for r in ["2", "4", "8", "13"] {
            let at = |s: &str| show(chem.relative(s, &q(r), &q(t), &Q::ONE));
            out.push_str(&format!(
                "    {r:>2} kpc  O {}  Fe {}  N {}  Eu {}  metals {}\n",
                at("O"),
                at("Fe"),
                at("N"),
                at("Eu"),
                show(chem.metals(&q(r), &q(t)))
            ));
        }
    }
    // each ring's closed forms read today: its gas, its star formation, its metals
    out.push_str("The disc today (closed forms per ring: gas and star formation in suns per pc^2 and per pc^2 Gyr)\n");
    let model = &chem.model;
    let dt = &q("13.8") - &model.params.first_stars;
    for r in ["2", "4", "8", "13"] {
        let k = xyl_cosmos::chem::ring_of(&q(r));
        if let Some(ring) = model.rings.get(k) {
            out.push_str(&format!(
                "  {r:>2} kpc  gas {}  star formation {}  metals {}\n",
                show(Some(ring.gas.at(&dt).mid())),
                show(Some(ring.sfr.at(&dt).mid())),
                show(chem.metals(&q(r), &q("13.8")))
            ));
        }
    }
    // each galaxy's weight dials, read at 8 kpc today (the home galaxy's
    // history, tilted by each core's recipe; a galaxy's own history is not
    // yet modelled)
    out.push_str(
        "Weight dials by galactic core, at 8 kpc today (relative to the Sun's birth gas)\n",
    );
    for c in
        xyl_cosmos::cores::founding(u.cosmic.gal.seed, u.cosmic.gal.bang, u.cosmic.gal.elements)
    {
        let line: Vec<String> = ["C", "N", "O", "Si", "Mg", "Fe", "Ni", "Eu"]
            .iter()
            .map(|sym| {
                let rel = chem
                    .relative(sym, &q("8"), &q("13.8"), &Q::ONE)
                    .unwrap_or(Q::ZERO);
                format!("{sym} {}", show(Some(&rel * &c.weights.factor(sym))))
            })
            .collect();
        out.push_str(&format!(
            "  core {} ({}): {}\n",
            c.shape.index,
            c.weights.label(),
            line.join("  ")
        ));
    }
    out
}

fn annals(u: &Universe<'_>, k: usize) -> String {
    let n = u.cosmic.annals.len();
    u.cosmic.annals[n.saturating_sub(k)..]
        .iter()
        .map(|a| format!("{} Gyr {:<8} {}", a.epoch, a.kind, a.text))
        .collect::<Vec<_>>()
        .join("\n")
}

fn events(u: &Universe<'_>, k: usize) -> String {
    let from = u.log.len().saturating_sub(k);
    u.log[from..]
        .iter()
        .map(|e| {
            format!(
                "count {} ({} Gyr) {} {}\n",
                show(&e.at),
                u.bell.gyr_dy(&e.at),
                e.kind,
                e.text
            )
        })
        .collect()
}

fn clock(u: &Universe<'_>) -> String {
    let next = u
        .next_event()
        .map(|t| format!("count {} ({} Gyr)", show(&t), u.bell.gyr_dy(&t)))
        .unwrap_or_else(|| "none queued".into());
    let mut out = format!(
        "The bell: one ring {} Gyr (exact: {}); now count {} (exact: {}); the next event {next}\n",
        Dy::from_q(&u.bell.period_gyr),
        u.bell.period_gyr,
        show(&u.now),
        u.now
    );
    // the keepers, epicenter to rims, read at now
    out.push_str("The keepers (epicenter to rims; rate r / (r + M), slip behind the epicenter's echo, coherence, rungs)\n");
    let c = &u.clocks;
    let mut slips: Vec<xyl_core::Q> = Vec::new();
    for (k, keeper) in c.keepers.iter().enumerate() {
        let r = keeper.read(&u.now);
        let rel = c.inside[k]
            .map(|i| &r.slip - &slips[i])
            .unwrap_or(xyl_core::Q::ZERO);
        slips.push(r.slip.clone());
        let lag = &xyl_core::Q::ONE - &keeper.rate;
        out.push_str(&format!(
            "  {:<12} galaxy {:<2} {} kpc out: its ring {} bells, 1 - rate {}, echo delay {} bells, its own count {}, slip {} turns (against the keeper inside {}), coherence {}, rungs {}, head {}\n",
            keeper.shell.name(),
            keeper.galaxy.map(|g| g.to_string()).unwrap_or_else(|| "-".into()),
            show(&keeper.from_epicenter_kpc),
            show(&keeper.ring),
            show(&lag),
            show(&keeper.delay),
            show(&r.local),
            show(&r.slip),
            show(&rel),
            show(&r.coherence),
            r.rung,
            c.heads.get(k).map(|h| h[..6].iter().map(|b| format!("{b:02x}")).collect::<String>()).unwrap_or_default()
        ));
    }
    out.push_str(&format!(
        "  the chain: {} commits held, {} entries; the ledger cites the rims' heads\n",
        c.commits.len(),
        c.entries.len()
    ));
    // the bridge: the machine's pace against the network's clock
    out.push_str(&match c.pace() {
        xyl_kernel::chronometry::Pace::Measured { rings_per_point, rings, points } => format!("The bridge: {} rings per TUniversal final-point advance ({} rings over {points} points): this machine's pace\n", show(&rings_per_point), show(&rings)),
        xyl_kernel::chronometry::Pace::Unlinked { rings, commits } => format!("The bridge: unlinked (no final point heard): {} rings over {commits} commits; the pace is measured once the house hears TUniversal\n", show(&rings)),
    });
    out
}

/// Every tier in its own rings (each galaxy its own enclosed S^360): for
/// star `n`, its turn about its galaxy's core and each of its worlds' turns
/// about it, by the ring-count law - a period is a count, 1 + R / H rings of
/// the anchor's own echo, unit-free (R / H is reach over holding whatever
/// the ring) - read as time only where the ring's length is known: the
/// universe's echo (from its curvature) and a galaxy's echo (twice its rim
/// over light). A star's own echo needs its radius from formation (not yet
/// modelled).
fn tiers(o: &Options, n: u64) -> Result<String, String> {
    let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
    let star = xyl_cosmos::galaxy::star(o.seed, &bang, n).ok_or("no such star")?;
    use xyl_cosmos::package::Si;
    let (c, kpc_m, yr_s, star_mass_kg) = (Si::c(), Si::kpc(), Si::yr_s(), Si::star_mass_kg());
    let rg = |kg: &Q| (&Si::g() * kg).checked_div(&(&c * &c)).unwrap_or(Q::ZERO);
    let years = |m: &Q| m.checked_div(&(&c * &yr_s)).unwrap_or(Q::ZERO);
    let pi = xyl_core::certified::pi(64).mid();
    let mut out = String::new();
    // the universe's own echo: a ring around its great circle
    let r_u = &xyl_cosmos::galaxy::radius_kpc() * &kpc_m;
    let ring_u = &(&Q::int(2) * &pi) * &r_u;
    out.push_str(&format!(
        "The universe's echo: radius {} m, one ring {} yr (its own, the founding's rim over pi)\n",
        show(&r_u),
        show(&years(&ring_u))
    ));
    // the galaxy's own echo: twice its rim (the pulse out to its far side and back)
    let scale = star.scale.to_q();
    let rim_m = &(&xyl_cosmos::galaxy::rim_kpc() * &scale) * &kpc_m;
    let ring_g = &Q::int(2) * &rim_m;
    out.push_str(&format!(
        "Galaxy {}'s echo: rim {} kpc (the founding's, scaled by its core), one ring {} yr\n",
        star.core,
        show(&(&xyl_cosmos::galaxy::rim_kpc() * &scale)),
        show(&years(&ring_g))
    ));
    // the star about its galaxy's core: reach and holding in that galaxy's light-rings
    let r_m = &star.orbit.radius.to_q() * &kpc_m;
    let inside = &(&xyl_cosmos::galaxy::inside_mass(&star.r)
        * &xyl_cosmos::cores::ratio_of_scale(&scale))
        * &star_mass_kg;
    let (reach, hold) = (
        r_m.checked_div(&ring_g).unwrap_or(Q::ZERO),
        rg(&inside).checked_div(&ring_g).unwrap_or(Q::ZERO),
    );
    let count = &Q::ONE + &reach.checked_div(&hold).unwrap_or(Q::ZERO);
    out.push_str(&format!("Star {n} about its core: reach {} kpc, deficit {}, its turn {} rings of its galaxy's echo = {} yr\n", star.orbit.radius, show(&hold.checked_div(&(&reach + &hold)).unwrap_or(Q::ZERO)), show(&count), show(&years(&(&count * &ring_g)))));
    // its worlds about it: counts only (its own echo awaits its radius)
    let chem = Chemistry::of(&bang).ok_or("no chemistry")?;
    let elements = xyl_cosmos::elements::founded(&bang.parent.share())?;
    if let Some(sys) = xyl_cosmos::systems::system(o.seed, &bang, &[n], &star, &chem, &elements) {
        let all =
            xyl_cosmos::manifold::manifolds(o.seed, &[n], &sys, &star, &xyl_cosmos::chain::units());
        let units = xyl_cosmos::chain::units();
        let star_kg = &sys.spec.mass * &star_mass_kg;
        for m in all
            .iter()
            .filter(|m| m.anchor == xyl_cosmos::manifold::Anchor::Star)
        {
            let a_m = &(&m.a * &xyl_core::units::Tier::System.in_rg()) * &units.r_g_m;
            let count = &Q::ONE + &a_m.checked_div(&rg(&star_kg)).unwrap_or(Q::ZERO);
            out.push_str(&format!("  World {}: a {} AU, its turn {} rings of its star's echo (the ring's length not yet modelled: it needs the star's radius)\n", m.world, show(&a_m.checked_div(&Si::au()).unwrap_or(Q::ZERO)), show(&count)));
        }
    }
    Ok(out)
}

/// The quantum capacity under each kind (3Q from the 3Z formulas), read on
/// each atom's own bands: 3Z its filled frames, 3Q three times the room of
/// every shape of every turning it reaches (the quantum state 3Z lives on),
/// R_Q = 6Z - 3Q, rho_Q = (2Z - Q_cap) / Q_cap; the kinds that seal (every
/// winding full and every shape of the outermost turning full: rho_Q = +1)
/// are the convergence points of the table.
fn seals_read(o: &Options, top: usize) -> String {
    let share = BigBang::of(o.seed).map(|b| b.parent.share());
    let mut out = String::from(
        "The seals (rho_Q = +1, the quantum substrate filled) and rho_Q along the table\n",
    );
    let mut sealed = Vec::new();
    let mut seams = Vec::new();
    let mut rows = String::new();
    let top = u32::try_from(top).unwrap_or(xyl_cosmos::elements::TOP_Z);
    for z in 1..=top {
        let Some(a) = xyl_cosmos::atoms::atom(z) else {
            continue;
        };
        // a seam: a kind whose single-nucleon peel releases
        if z > 1 {
            let u = |z: u32| i64::from(xyl_cosmos::atoms::unpaired(z));
            if u(z - 1) + 1 - u(z) < 0 {
                seams.push(z);
            }
        }
        let cap: u32 = (1..=a.outer_n)
            .map(|n| {
                (0..n)
                    .map(|l| 2 * xyl_cosmos::atoms::orientations(l))
                    .sum::<u32>()
            })
            .sum();
        let rho =
            Q::new(2 * i64::from(z) - i64::from(cap), i64::from(cap.max(1))).unwrap_or(Q::ZERO);
        if a.sealed() {
            sealed.push((z, a.outer_n, cap));
        }
        if [1, 2, 28, 115, 208, 1000, 1050, 1092, 1100, 1200, 1600].contains(&z) {
            rows.push_str(&format!("  Z{z:>5}: turning {} capacity {cap} -> 3Z {} 3Q {} R_Q {} rho_Q {} (open frames {}, sealed {})\n", a.outer_n, 3 * z, 3 * cap, 6 * i64::from(z) - 3 * i64::from(cap), show(&rho), a.unpaired, a.sealed()));
        }
    }
    out.push_str(&format!(
        "Sealed kinds: {}\n",
        sealed
            .iter()
            .map(|(z, n, c)| format!("Z{z} (turning {n}, capacity {c})"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    out.push_str(&format!(
        "Seams (a single-nucleon peel releases): {}\n",
        seams
            .iter()
            .map(|z| format!("Z{z}"))
            .collect::<Vec<_>>()
            .join(" ")
    ));
    match xyl_cosmos::atoms::vessel() {
        Some((lo, hi)) => out.push_str(&format!(
            "The vessel (the firmament's band): Z{lo}..Z{hi}, {} kinds; founded whole from 2^{} hydrogen at the bounce's share {}\n",
            hi - lo + 1,
            share
                .as_ref()
                .map(xyl_cosmos::elements::founding_bits)
                .unwrap_or(0),
            share.map(|s| s.to_string()).unwrap_or_default()
        )),
        None => out.push_str("The vessel: none (no seal within the table)\n"),
    }
    out.push_str(&rows);
    out
}

/// The founding's kinds by reach (the table's ceiling at TOP_Z): every
/// founded kind's reach about the Wellspring from the founding cascade, its
/// open share Q / 3Z, and the ratio of reaches Z 115 / Z top (under the
/// linear law).
fn founding_read(o: &Options) -> Result<String, String> {
    let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
    let elements = xyl_cosmos::elements::founded(&bang.parent.share())?;
    let mix: Vec<(u32, Q)> = elements
        .iter()
        .filter(|e| e.abundance.signum() > 0)
        .map(|e| (e.z, e.abundance.clone()))
        .collect();
    let present = mix.len();
    let top = mix.iter().map(|(z, _)| *z).max().unwrap_or(0);
    let (_, shells) = xyl_cosmos::formation::founding_shells(&mix);
    let mut out = format!(
        "The founding with the table to Z {}: {present} kinds founded (heaviest Z {top})\n",
        xyl_cosmos::elements::TOP_Z
    );
    let reach = |z: u32| shells.iter().find(|s| s.0 == z).map(|s| s.2.clone());
    for (z, _, r, _) in shells.iter().filter(|s| {
        [
            1, 2, 6, 8, 26, 79, 115, 150, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200,
        ]
        .contains(&s.0)
            || s.0 == top
    }) {
        let q = xyl_cosmos::compound::open(*z);
        out.push_str(&format!(
            "  Z{z:>5}: reach {}  Q/3Z {}\n",
            show(r),
            show(&Q::new(i64::from(q), 3 * i64::from(*z)).unwrap_or(Q::ZERO))
        ));
    }
    if let (Some(r115), Some(rtop)) = (reach(115), reach(top)) {
        out.push_str(&format!(
            "reach Z115 / reach Z{top}: {}\n",
            show(&r115.checked_div(&rtop).unwrap_or(Q::ZERO))
        ));
    }
    Ok(out)
}

/// Formation, read (laws not yet wired into the game's stars): each
/// galaxy's cut, gas, cooling and stars, its counts in its own echo rings
/// (the ring a turning counts in: its anchor's echo), years only at the seam.
fn formation(o: &Options) -> Result<String, String> {
    let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
    let elements = xyl_cosmos::elements::founded(&bang.parent.share())?;
    let shapes = xyl_cosmos::cores::shapes(o.seed, &bang);
    let total = shapes.iter().fold(Q::ZERO, |a, s| &a + &s.mass_ratio());
    // the mean nucleus's mass number, by count
    let (count, mass) = elements
        .iter()
        .filter(|e| e.abundance.signum() > 0)
        .fold((Q::ZERO, Q::ZERO), |(c, m), e| {
            (&c + &e.abundance, &m + &(&e.abundance * &e.a))
        });
    let a_mean = mass.checked_div(&count).unwrap_or(Q::ONE);
    let mut out = String::new();
    // the founding's matter: each element's count per hydrogen
    let hydrogen = elements
        .iter()
        .find(|e| e.z == 1)
        .map(|e| e.abundance.clone())
        .unwrap_or(Q::ONE);
    let by = |sym: &str| {
        elements
            .iter()
            .find(|e| e.sym == sym)
            .map(|e| show(&e.abundance.checked_div(&hydrogen).unwrap_or(Q::ZERO)))
            .unwrap_or_default()
    };
    out.push_str(&format!(
        "The founding's matter (the bounce's spin {}; count per hydrogen, whole nuclei per 2^224 hydrogen): He {}  C {}  N {}  O {}  Si {}  Fe {}  U {}; mean mass number {}\n",
        show(&bang.parent.spin), by("He"), by("C"), by("N"), by("O"), by("Si"), by("Fe"), by("U"), show(&a_mean)
    ));
    // the universe's own measures: time in rings, reach as a deficit share
    // in its own enclosure, mass as holding share - never years,
    // kiloparsecs or suns. The pieces are the founding's beads one tier
    // down, heated by the pulse swung by the inbound echo's mean over the
    // first echo, as at the founding; every galaxy is the home's homologue
    let sub = xyl_cosmos::cores::substrate(o.seed, &bang).ok_or("no founding")?;
    let swing = &Q::ONE + &sub.inbound_mean(sub.echo()).to_q();
    let f = xyl_cosmos::formation::of(&sub.beads, &sub.unit(), &sub.held_share(), &swing);
    let gas: Q = f
        .pieces
        .iter()
        .fold(Q::ZERO, |a, p| &a + &(&p.lambda * &(&p.to - &p.from)));
    let formed: Vec<&xyl_cosmos::formation::Piece> =
        f.pieces.iter().filter(|p| p.clouds > 0).collect();
    let clouds: i64 = formed.iter().map(|p| p.clouds).sum();
    let stars = formed.iter().fold(Q::ZERO, |a, p| &a + &p.stars);
    let in_stars = formed
        .iter()
        .fold(Q::ZERO, |a, p| &a + &(&p.star * &p.stars));
    let first = formed.iter().filter_map(|p| p.cools_in).min();
    let (small, big) = (
        formed
            .iter()
            .filter(|p| p.stars.signum() > 0)
            .map(|p| p.star.clone())
            .min(),
        formed
            .iter()
            .filter(|p| p.stars.signum() > 0)
            .map(|p| p.star.clone())
            .max(),
    );
    let dark: Q = f
        .pieces
        .iter()
        .filter(|p| p.clouds == 0)
        .fold(Q::ZERO, |a, p| &a + &(&p.lambda * &(&p.to - &p.from)));
    for sh in &shapes {
        let s_g = sh.mass_ratio().checked_div(&total).unwrap_or(Q::ONE);
        out.push_str(&format!(
            "Galaxy {} (its share of the universe {}): push {}, held in the core {}; gas {} in {} pieces (the beads), lit share {}\n",
            sh.index, show(&s_g), show(&f.push), show(&f.held), show(&gas), f.pieces.len(), show(&sub.lighting.lit)
        ));
        out.push_str(&format!(
            "  Clouds {clouds}, first collapsing at {} rings; stars {}, holding {} of the grains; each {} .. {} of the grains; gas that never lights: {}\n",
            first.map(|n| n.to_string()).unwrap_or_else(|| "-".into()),
            show(&stars),
            show(&in_stars),
            small.as_ref().map(show).unwrap_or_else(|| "-".into()),
            big.as_ref().map(show).unwrap_or_else(|| "-".into()),
            show(&dark)
        ));
        // the star tier (the core's cut and its disc), each forming piece
        for pc in formed.iter().filter(|p| p.stars.signum() > 0).take(4) {
            // a world's holding over its star's (holding shares, never world units)
            let shares = |w: &Q| show(&w.checked_div(&pc.star_share).unwrap_or(Q::ZERO));
            out.push_str(&format!(
                "  Star tier at reach {}: tide {}, infall agitation {}; the star {} of its infall; open share infall {} / disc {}; {} worlds, from {} to {} of its star's holding, belts {} of the infall\n",
                show(&pc.reach),
                show(&pc.tide),
                show(&pc.heated),
                show(&pc.star_share),
                show(&pc.infall_open),
                show(&pc.disc_open),
                pc.worlds.len(),
                pc.worlds.first().map(|w| shares(&w.holding)).unwrap_or_else(|| "-".into()),
                pc.worlds.last().map(|w| shares(&w.holding)).unwrap_or_else(|| "-".into()),
                show(&pc.belts)
            ));
            out.push_str(&worlds_line(&pc.worlds));
        }
        // the first pieces, lit or not
        for pc in f.pieces.iter().take(4) {
            out.push_str(&format!(
                "  piece at reach {} ({} to {} rings): holding per ring {}, agitation {}, emission {}, cooled {}, heat {}, clouds {}, lights after {} rings\n",
                show(&pc.reach),
                show(&pc.from),
                show(&pc.to),
                show(&pc.lambda),
                show(&pc.agitation),
                show(&pc.emission),
                show(&pc.cooled),
                show(&(&pc.heated - &pc.cooled)),
                pc.clouds,
                pc.cools_in.map(|n| n.to_string()).unwrap_or_else(|| "-".into())
            ));
        }
    }
    // the planetary cores on the founding's own mix - a scratch reading of
    // the law (one infall of the founding's matter, not a formed star: no
    // star forms here until convection is modelled), at the galaxy's tide
    // inside its rim, delta_gal = 2 / (R + 2) for R = 1, 3/2, 2
    let mix: Vec<(u32, Q)> = xyl_cosmos::formation::kinds(&elements)
        .iter()
        .map(|k| (k.z, k.share.clone()))
        .collect();
    let p = Q::new(1, 2).unwrap_or(Q::ZERO);
    for (r, tide) in [
        ("1", Q::new(2, 3)),
        ("3/2", Q::new(4, 7)),
        ("2", Q::new(1, 2)),
    ] {
        let tide = tide.unwrap_or(Q::ZERO);
        let c = xyl_cosmos::formation::hold_at_heat(&mix, &p, &Q::ZERO, &Q::ZERO, &Q::ONE);
        let (worlds, belts) = xyl_cosmos::disc::star_disc(&c.held, &tide, &c.rest);
        out.push_str(&format!(
            "Scratch infall of the founding's mix at galactic reach {r} (tide {}): the star {} of it, held down to Z {}; its disc {} kinds, {} worlds, belts {}\n",
            show(&tide), show(&c.held), c.lightest.map(|z| z.to_string()).unwrap_or_else(|| "-".into()), c.rest.len(), worlds.len(), show(&belts)
        ));
        out.push_str(&worlds_line(&worlds));
    }
    Ok(out)
}

/// The worlds of a disc, each with its planetary core's cut: inner, middle
/// and outer (up to six shown).
fn worlds_line(worlds: &[xyl_cosmos::disc::World]) -> String {
    let n = worlds.len();
    let picks: Vec<usize> = if n <= 6 {
        (0..n).collect()
    } else {
        vec![0, 1, n / 3, n / 2, 2 * n / 3, n - 1]
    };
    picks
        .into_iter()
        .filter_map(|i| worlds.get(i).map(|w| (i, w)))
        .map(|(i, w)| {
            format!(
                "    world {i} at {}: holding {}, heat {}, its core keeps {} of its matter, held down to Z {}\n",
                show(&w.reach), show(&w.holding), show(&w.heat), show(&w.kept), w.lightest.map(|z| z.to_string()).unwrap_or_else(|| "-".into())
            )
        })
        .collect()
}

/// The substrate, read at T Gyr after the bounce: the core's
/// firmament kind by kind (its reach, the hold inside it, its deficit, the
/// share of its band the reverberation carries past the throat, its rate of
/// leaving and what it still holds), the outflow, the ocean's share and delay
/// for every galaxy and what each has taken, the books, and Z, Q and R. Every
/// value a closed form at T: no echo stepped, no parcel counted.
fn core_read(o: &Options, t_gyr: Q) -> Result<String, String> {
    use xyl_core::dyadic::Dy;
    let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
    let elements = xyl_cosmos::elements::founded(&bang.parent.share())?;
    let clock = std::time::Instant::now();
    let s = xyl_cosmos::cores::substrate(o.seed, &bang).ok_or("no founding")?;
    let founded_in = clock.elapsed();
    let t = Dy::from_q(&t_gyr);
    let rv = &s.reverb;
    let anchor = xyl_cosmos::circuit::anchor();
    let mut out = format!(
        "The substrate at {} Gyr (seed {}): founded in {:.1?}; the bell {} Gyr, an echo {} Gyr; {} firmament kinds, {} beads ({} hold a sun), rim {} rings\n",
        t,
        o.seed,
        founded_in,
        xyl_cosmos::circuit::bell_gyr(),
        xyl_cosmos::circuit::echo_gyr(),
        s.firmament.len(),
        s.beads.len(),
        s.galaxies().len() - 1,
        show(&s.rim)
    );
    let leaving = rv.iter().filter(|r| r.beyond.signum() > 0).count();
    out.push_str(&format!(
        "  The core: {leaving} of {} firmament kinds reach past the throat over the bell's swing; the furthest held kind at {} rings of the rim's {}\n",
        rv.len(),
        show(&s.far_reach()),
        show(&s.rim)
    ));
    out.push_str(&format!(
        "  The admission: the parent's spin {} (drawn, then floored at the least that holds the firmament whole), torsion share {}; the least share {} at the inbound echo's peak {}; margin {}\n",
        show(&anchor.spin),
        show(&s.share),
        show(&s.least_share()),
        s.inbound_peak(),
        show(&anchor.margin)
    ));
    out.push_str(&format!(
        "  The inbound echo at T: {} (the ring plus the vessel's drift, per echo); its mean over the first echo {}; the push at the throat {} swung to {} at T\n",
        s.inbound(t),
        s.inbound_mean(s.echo()),
        xyl_cosmos::circuit::throat_push(),
        s.push_at(Dy::ONE, Dy::ONE, t)
    ));
    let swung = (0..rv.len())
        .filter(|&i| s.beyond_at(i, t).signum() > 0)
        .count();
    out.push_str(&format!(
        "  The swing at T: {swung} of {} kinds past the throat under the inbound echo\n",
        rv.len()
    ));
    // the jetstream: each kind's channels composed over the echo at T, and
    // what the composition leaves (the torsion); the loop closes at one
    let tally = |loops: &[Option<xyl_cosmos::circuit::PoleFlux>]| {
        let (mut held_back, mut opened, mut closed, mut none) = (0, 0, 0, 0);
        for l in loops {
            match l {
                None => none += 1,
                Some(f) if f.closed => closed += 1,
                Some(f) if f.residual < Q::ONE => held_back += 1,
                Some(_) => opened += 1,
            }
        }
        format!("{closed} loops close, {held_back} are held back toward Z, {opened} carried on toward Q, {none} have no pole state (sealed, or past the far face)")
    };
    let loops: Vec<Option<xyl_cosmos::circuit::PoleFlux>> =
        (0..rv.len()).map(|i| s.loop_flux(i, t)).collect();
    let at_mean: Vec<Option<xyl_cosmos::circuit::PoleFlux>> = (0..rv.len())
        .map(|i| s.loop_flux_swung(i, t, Dy::ONE))
        .collect();
    out.push_str(&format!(
        "  The jetstream (filament, vacuum and echo composed on each kind's pole state): at T, with the inbound echo at {}: {}; at the echo's mean: {}\n",
        s.inbound(t),
        tally(&loops),
        tally(&at_mean)
    ));
    out.push_str(&format!(
        "  The lighting (the home galaxy's gas, the beads one tier down): first lights after {} bells, the lit share {}, {} pieces of gas ({} light)\n",
        s.lighting
            .cools_in
            .map(|n| n.to_string())
            .unwrap_or_else(|| "never".into()),
        show(&s.lighting.lit),
        s.lighting.pieces.len(),
        s.lighting.pieces.iter().filter(|p| p.2.is_some()).count()
    ));
    let outflow = s.outflow(t);
    let mut order: Vec<usize> = (0..s.firmament.len()).collect();
    order.sort_by(|&a, &b| rv[b].rate.cmp(&rv[a].rate));
    for &i in order.iter().take(8) {
        let k = &s.firmament[i];
        let r = &rv[i];
        let m0 = Dy::from_q(&k.amount);
        let jet = at_mean
            .get(i)
            .and_then(|l| l.as_ref())
            .map(|f| {
                let ch = |n: &str| {
                    f.channels
                        .iter()
                        .find(|c| c.0 == n)
                        .map(|c| show(&c.1))
                        .unwrap_or_else(|| "-".into())
                };
                format!(
                    "; loop at the mean: t {} filament x{} vacuum x{} echo x{} -> residual {}{}",
                    show(&f.t0),
                    ch("filament"),
                    ch("vacuum"),
                    ch("echo"),
                    show(&f.residual),
                    if f.closed { " (closed)" } else { "" }
                )
            })
            .unwrap_or_else(|| "; loop: no pole state".into());
        out.push_str(&format!(
            "    Z{}: reach {} hold {} deficit {} beyond {} rate {} /Gyr; holds {} of {} at T{jet}\n",
            k.z,
            r.reach,
            r.hold,
            r.deficit,
            r.beyond,
            r.rate,
            m0 - outflow[i].1,
            m0
        ));
    }
    let (held, left, taken, founded) = s.books(t);
    out.push_str(&format!("  The books (by mass): the firmament holds {held}, has shed {left}, the galaxies have taken {taken}, {} in flight; founded {founded}\n", left - taken));
    out.push_str(
        "  The ocean: each galaxy's share of the outflow, its delay, what it has taken by T\n",
    );
    let intakes = s.intakes(t);
    for (i, share, delay) in s.ocean_shares() {
        let took = intakes
            .iter()
            .find(|x| x.0 == i)
            .map(|x| x.1.iter().fold(Dy::ZERO, |a, (_, v)| a + *v))
            .unwrap_or(Dy::ZERO);
        out.push_str(&format!(
            "    galaxy {i}: share {share}, delay {delay} Gyr, taken {took}\n"
        ));
    }
    let (q_state, q_mass) = s.rate(t);
    out.push_str(&format!(
        "  The force: the rate leaving the core {q_state} a Gyr in its whole state ({q_mass} by mass); the ocean's pulse {} rings a Gyr; the waveform at the rim {}\n",
        s.c_ocean(),
        s.amplitude(Dy::from_q(&s.rim), t)
    ));
    // each galaxy's extent in the ocean's rings: its rim (r_max x its scale,
    // kpc) over the kpc one of its reach's rings spans
    let shapes = xyl_cosmos::cores::shapes(o.seed, &bang);
    let kpc = xyl_cosmos::chain::kpc();
    for g in s.push(t) {
        let shape = shapes.iter().find(|sh| sh.index == g.index);
        let extent = shape.and_then(|sh| {
            let rim_gu = &(&xyl_cosmos::galaxy::rim_kpc() * &sh.scale().to_q()) * &kpc;
            let per_ring = sh.anchor().0.checked_div(&g.reach0.to_q())?;
            Some(Dy::from_q(&rim_gu.checked_div(&per_ring)?))
        });
        let tide = extent.map(|x| s.tide(g.reach0, x, t)).unwrap_or(Dy::ZERO);
        let rate = shape.map(|sh| sh.rate).unwrap_or(Dy::ZERO);
        let cell = shape.map(|sh| sh.place_at(t, &xyl_cosmos::chain::radius()).cell());
        out.push_str(&format!(
            "    galaxy {}: reach {} -> {} rings, recession {} rings a Gyr (drift {}, swing +/- {} rings an echo), shift z {}; circulates {} turns a Gyr, at azimuth {} polar {} deg; tide across it {}\n",
            g.index,
            g.reach0,
            g.reach,
            g.recession,
            g.drift,
            g.swing,
            g.shift,
            rate,
            cell.map(|c| c.azimuth.to_string()).unwrap_or_else(|| "-".into()),
            cell.map(|c| c.polar.to_string()).unwrap_or_else(|| "-".into()),
            tide
        ));
    }
    let w = s.two_weights(t, &elements);
    let p = s.pulse(t);
    out.push_str(&format!(
        "  Z, Q and R at T: protons {} neutrons {} (Q share {}); the pulse p = {} turns, R = kappa (1 - 2p) per kind\n",
        show(&w.protons),
        show(&w.neutrons),
        w.neutron_share().as_ref().map(show).unwrap_or_else(|| "-".into()),
        show(&p)
    ));
    out.push_str(&format!("  read in {:.1?}\n", clock.elapsed()));
    Ok(out)
}
