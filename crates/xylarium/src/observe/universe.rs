//! Readings of the universe as a whole: `universe`, `cores`, `star`,
//! `house`, `annals` and `events`.

use super::hex;
use crate::run::{show, summary};
use xyl_core::dyadic::Dy;
use xyl_cosmos::origin::Origin;
use xyl_kernel::seats::{Holder, On, Role};
use xyl_kernel::universe::Universe;

/// The `universe` reading: the bell's count, the log, the peoples, the
/// seats, the bells, the count of annals, the civilizations' summary, the
/// forecast's limits and the founding's galaxies.
pub(super) fn universe(u: &Universe<'_>) -> String {
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
        concat!(
            "Worlds formed about the sample stars (every galaxy): {} worlds; ",
            "life began on {} ({} alive at their horizon); kept from minds: ",
            "no solvent {}, could not copy {}, star died {}, world merged {}, ",
            "sterilised {}, too slow {}, unfed {}; minds {}\n"
        ),
        l.worlds,
        l.alive + l.minds,
        l.alive,
        l.no_solvent,
        l.cannot_copy,
        l.star_died,
        l.merged,
        l.sterilised,
        l.too_slow,
        l.unfed,
        l.minds
    ));
    out.push_str(&cores(u.cosmic.gal.seed, u.cosmic.gal.origin));
    out
}

/// The founding's galaxies: the home at the universe core, then every bead
/// of the cascade, each with its hole's mass in suns, the ratio of that mass
/// to the home hole's, and its place.
pub(super) fn cores(seed: u64, origin: &Origin) -> String {
    let all = xyl_cosmos::cores::shapes(seed, origin);
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

/// A seat's holder as printed: the universe, a player, or a mind by the
/// first four bytes of its key in hex.
pub(super) fn holder(h: &Holder) -> String {
    match h {
        Holder::Universe => "the universe".into(),
        Holder::Player(n) => format!("player {n}"),
        Holder::Mind(k) => format!("mind {}", hex(&k[..4])),
    }
}

/// The `star N` reading: the star's pulse books, drift, anchored worlds and
/// the peoples living at it.
pub(super) fn star(u: &Universe<'_>, n: Option<u64>) -> String {
    let Some(n) = n else {
        return "observe: star N (a star's sample index)".into();
    };
    let mut out = format!("star {n}\n");
    match u.field.books.get(&n) {
        Some(b) => out.push_str(&format!(
            concat!(
                "  books: in {} = gravity {} + light {} + transduced out {} ({}); ",
                "pulses {}; since {}\n"
            ),
            b.energy_in,
            b.gravity,
            b.light,
            b.transduced_out,
            if b.balances() {
                "balanced"
            } else {
                "not balanced"
            },
            b.pulses,
            b.since.map(|t| t.to_string()).unwrap_or_else(|| "-".into())
        )),
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

/// The `house` reading: every seat held by a mind and every ancient seat.
pub(super) fn house(u: &Universe<'_>) -> String {
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

/// The `annals K` reading: the last `k` annals, one a line.
pub(super) fn annals(u: &Universe<'_>, k: usize) -> String {
    let n = u.cosmic.annals.len();
    u.cosmic.annals[n.saturating_sub(k)..]
        .iter()
        .map(|a| format!("{} Gyr {:<8} {}", a.epoch, a.kind, a.text))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The `events K` reading: the last `k` logged events, each at its count.
pub(super) fn events(u: &Universe<'_>, k: usize) -> String {
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
