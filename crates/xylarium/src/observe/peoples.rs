//! Readings of the peoples: `people N` and `sync K`.

use super::universe::holder;
use xyl_core::dyadic::Dy;
use xyl_kernel::knowledge::Domain;
use xyl_kernel::seats::On;
use xyl_kernel::universe::Universe;

/// The `people N` reading: one people's home, traits, body, numbers, power,
/// knowledge, gauges, radiation, world and seats.
pub(super) fn people(u: &Universe<'_>, n: Option<usize>) -> String {
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
            concat!(
                "  gauges: instinct {}, intelligence {}, sentience {}, ",
                "consciousness {} -> gamma {}\n"
            ),
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

/// The quartiles of `v` (min / q1 / median / q3 / max) and the count read,
/// or "unread" for an empty set.
fn quartiles(mut v: Vec<Dy>) -> String {
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
}

/// The gamma gauge of people `i`, when it has gauges.
fn gamma(u: &Universe<'_>, i: usize) -> Option<Dy> {
    u.civ.gauges.get(i).map(|g| g.gamma)
}

/// One ranked line of the `sync` reading: people `i` and its value `v`.
fn sync_line(u: &Universe<'_>, i: usize, v: Dy) -> String {
    let p = &u.cosmic.peoples[i];
    format!(
        "    {i:>5} {:<14} {v}  (star {} world {}, gamma {})\n",
        p.name,
        p.star,
        p.world,
        gamma(u, i)
            .map(|g| g.to_string())
            .unwrap_or_else(|| "-".into())
    )
}

/// A people in step with its universe, at scale (xyl-kernel sync.rs):
/// every living people's readings as a distribution (quartiles), then per
/// reading the `k` most out of step - each reading ranked alone, never one
/// score.
pub(super) fn sync(u: &Universe<'_>, k: usize) -> String {
    let steps = xyl_kernel::sync::living(u);
    let living: Vec<usize> = steps.iter().map(|(i, _)| *i).collect();
    let mut out = format!(
        "A people in step with its universe: {} living of {} peoples\n",
        living.len(),
        u.cosmic.peoples.len()
    );
    // each reading: its label, its value on a step, and whether the highest
    // values are the most out of step
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
        quartiles(living.iter().filter_map(|&i| gamma(u, i)).collect())
    ));
    for (name, f, _) in &readings {
        out.push_str(&format!(
            "  {name}: {}\n",
            quartiles(steps.iter().filter_map(|(_, s)| f(s)).collect())
        ));
    }
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
            out.push_str(&sync_line(u, *i, *x));
        }
        // a two-sided reading (R) is out of step at both ends
        if !*high {
            out.push_str("  ... and by the same, highest:\n");
            for (x, i) in v.iter().rev().take(k) {
                out.push_str(&sync_line(u, *i, *x));
            }
        }
    }
    out
}
