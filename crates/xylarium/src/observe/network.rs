//! The ley network reading: `ley K`.

use std::collections::{BTreeMap, BTreeSet};
use xyl_core::dyadic::Dy;
use xyl_kernel::transit::{best_spec, home_escape, Place};
use xyl_kernel::universe::Universe;

/// The `ley K` reading: the constellations and ley lines, passage times from
/// the ancient houses, the star map by degree cells, and the discord and
/// quarantine.
pub(super) fn ley(u: &Universe<'_>, k: usize) -> String {
    let mut out = lines(u, k);
    out.push_str(&passages(u, k));
    out.push_str(&cells(u, k));
    out.push_str(&discord(u, k));
    out
}

/// The constellations' sizes and mouth reach, the lines by builder, the
/// first `k` lines built and the first `k` under way.
fn lines(u: &Universe<'_>, k: usize) -> String {
    let net = &u.network;
    let cs = &net.constellations;
    let mut sizes: Vec<usize> = cs.iter().map(|c| c.stars.len()).collect();
    sizes.sort_unstable();
    let lone = sizes.iter().filter(|&&n| n == 1).count();
    let mut out = format!(
        concat!(
            "The ley network at {} Gyr: {} stars in play ({} witnessed) in {} constellations ",
            "({} lone, largest {} stars, median {}); {} ley lines built, {} under way\n"
        ),
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
    let mut by: BTreeMap<String, usize> = BTreeMap::new();
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
    out
}

/// Passage times from the first three living ancient houses to their `k`
/// nearest living peoples, at the house's best drive and home escape speed.
fn passages(u: &Universe<'_>, k: usize) -> String {
    let net = &u.network;
    let mut out = String::new();
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
    out
}

/// The star map by degree cells: each star in play's cell on its galaxy's
/// sphere, the number of cells occupied, and the cell and constellation of
/// the first `k` living peoples' stars.
fn cells(u: &Universe<'_>, k: usize) -> String {
    let net = &u.network;
    let sphere = xyl_cosmos::chain::radius();
    let now = xyl_cosmos::circuit::rings_of_gyr_dy(u.gyr());
    let places: BTreeMap<u64, xyl_cosmos::chain::Place> = net
        .sample
        .iter()
        .map(|s| (s.star.n, s.star.place(now, &sphere)))
        .collect();
    let mut cells: BTreeMap<xyl_cosmos::chain::Cell, usize> = BTreeMap::new();
    for p in places.values() {
        *cells.entry(p.cell()).or_default() += 1;
    }
    let galaxies: BTreeSet<Option<u32>> = cells.keys().map(|c| c.galaxy).collect();
    let mut out = format!(
        "  degree cells: {} occupied by {} stars in play across {} galaxies\n",
        cells.len(),
        places.len(),
        galaxies.len()
    );
    let mut seen = BTreeSet::new();
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
            concat!(
                "    star {} ({}): galaxy {} cell az {} deg, polar {} deg; ",
                "constellation of star {} ({} stars)\n"
            ),
            p.star,
            p.name,
            c.galaxy
                .map(|g| g.to_string())
                .unwrap_or_else(|| "-".into()),
            c.azimuth,
            c.polar,
            con.map(|c| c.key().to_string())
                .unwrap_or_else(|| "-".into()),
            con.map(|c| c.stars.len()).unwrap_or(0)
        ));
    }
    out
}

/// The discord of the peoples outside the ancient houses, its terms for the
/// `k` most discordant, and the `k` constellations held most closed.
fn discord(u: &Universe<'_>, k: usize) -> String {
    let net = &u.network;
    let parts = u.discord_parts();
    let mut d: Vec<(Dy, usize)> = parts.iter().map(|(&i, x)| (x.discord, i)).collect();
    d.sort();
    d.reverse();
    let (joined, still) = (
        parts.values().filter(|x| x.joined).count(),
        parts.values().filter(|x| x.cruise.signum() <= 0).count(),
    );
    let mut out = format!(
        concat!(
            "  discord read on {} peoples (no ancient house): {} joined to the network, ",
            "{} whose drives cannot leave their well; quarantine on {} constellations\n"
        ),
        parts.len(),
        joined,
        still,
        net.quarantine
            .iter()
            .filter(|(_, q)| q.signum() > 0)
            .count()
    );
    for (x, i) in d.iter().take(k) {
        let p = &u.cosmic.peoples[*i];
        let q = &parts[i];
        // `d = reach / (reach + mouth) x forget x (1 - joined)`.
        out.push_str(&format!(
            concat!(
                "    {:<14} d {} = reach {} kpc / (reach + mouth {} kpc) x forget {} ",
                "x (1 - joined {}); v_e {} m/s, cruise {} m/s, escape {} m/s (star {})\n"
            ),
            p.name,
            x,
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
