//! The clock reading: `clock`.

use crate::run::show;
use xyl_core::dyadic::Dy;
use xyl_core::Q;
use xyl_kernel::chronometry::Pace;
use xyl_kernel::universe::Universe;

/// The `clock` reading: the bell and the next event, each keeper from the
/// epicenter to the rims read now, the keepers' chain, and the bridge (this
/// machine's pace against the network's clock).
pub(super) fn clock(u: &Universe<'_>) -> String {
    let next = u
        .next_event()
        .map(|t| format!("count {} ({} Gyr)", show(&t), u.bell.gyr_dy(&t)))
        .unwrap_or_else(|| "none queued".into());
    let mut out = format!(
        concat!(
            "The bell: one ring {} Gyr (exact: {}); now count {} (exact: {}); ",
            "the next event {}\n"
        ),
        Dy::from_q(&u.bell.period_gyr),
        u.bell.period_gyr,
        show(&u.now),
        u.now,
        next
    );
    out.push_str(concat!(
        "The keepers (epicenter to rims; rate r / (r + M), slip behind the epicenter's echo, ",
        "coherence, rungs)\n"
    ));
    let c = &u.clocks;
    let mut slips: Vec<Q> = Vec::new();
    for (k, keeper) in c.keepers.iter().enumerate() {
        let r = keeper.read(&u.now);
        // The slip relative to the keeper inside: `slip_k - slip_inside`.
        let rel = c.inside[k].map(|i| &r.slip - &slips[i]).unwrap_or(Q::ZERO);
        slips.push(r.slip.clone());
        let lag = &Q::ONE - &keeper.rate;
        let head = c
            .heads
            .get(k)
            .map(|h| {
                h[..6]
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            })
            .unwrap_or_default();
        out.push_str(&format!(
            concat!(
                "  {:<12} galaxy {:<2} {} kpc out: its ring {} bells, 1 - rate {}, ",
                "echo delay {} bells, its own count {}, slip {} turns ",
                "(against the keeper inside {}), coherence {}, rungs {}, head {}\n"
            ),
            keeper.shell.name(),
            keeper
                .galaxy
                .map(|g| g.to_string())
                .unwrap_or_else(|| "-".into()),
            show(&keeper.from_epicenter_kpc),
            show(&keeper.ring),
            show(&lag),
            show(&keeper.delay),
            show(&r.local),
            show(&r.slip),
            show(&rel),
            show(&r.coherence),
            r.rung,
            head
        ));
    }
    out.push_str(&format!(
        "  the chain: {} commits held, {} entries; the ledger cites the rims' heads\n",
        c.commits.len(),
        c.entries.len()
    ));
    out.push_str(&bridge(c.pace()));
    out
}

/// The bridge line: the rings per TUniversal final-point advance once
/// measured, or the rings over commits while no final point has been heard.
fn bridge(pace: Pace) -> String {
    match pace {
        Pace::Measured {
            rings_per_point,
            rings,
            points,
        } => format!(
            concat!(
                "The bridge: {} rings per TUniversal final-point advance ",
                "({} rings over {} points): this machine's pace\n"
            ),
            show(&rings_per_point),
            show(&rings),
            points
        ),
        Pace::Unlinked { rings, commits } => format!(
            concat!(
                "The bridge: unlinked (no final point heard): {} rings over {} commits; ",
                "the pace is measured once the house hears TUniversal\n"
            ),
            show(&rings),
            commits
        ),
    }
}
