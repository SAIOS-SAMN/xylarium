//! The clock reading: `clock`.

use crate::run::show;
use xyl_core::Q;
use xyl_cosmos::circuit::{bell, gyr_of_rings, gyr_of_rings_dy};
use xyl_kernel::checkpoint::hex;
use xyl_kernel::chronometry::Chronometry;
use xyl_kernel::universe::Universe;

/// The `clock` reading, the telemetry of time (docs/chronometry.md, #302):
/// the root's winding and the next event; the root's windings, its throat's
/// echo and the breath's phase, the T lane read apart; each keeper from the
/// epicenter to the rims read now with its windings; the keepers' chain;
/// the bridge (the universe's rings against the network's heights, segment
/// by segment); the checks as live residues; and ring 0's point.
pub(super) fn clock(u: &Universe<'_>) -> String {
    let next = u
        .next_event()
        .map(|t| format!("count {} ({} Gyr)", show(&t), show(&gyr_of_rings(&t))))
        .unwrap_or_else(|| "none queued".into());
    let mut out = format!(
        concat!(
            "The bell: one ring {} Gyr; the root's winding now {} (exact: {}); ",
            "the next event {}\n"
        ),
        gyr_of_rings_dy(bell()),
        show(&u.now),
        u.now,
        next
    );
    out.push_str(&windings_and_lane(u));
    out.push_str(concat!(
        "The keepers (epicenter to rims; rate r / (r + M), slip behind the epicenter's echo, ",
        "coherence, rungs)\n"
    ));
    let c = &u.clocks;
    let mut slips: Vec<Option<Q>> = Vec::new();
    for (k, keeper) in c.keepers.iter().enumerate() {
        let Some(r) = keeper.read(&u.now) else {
            slips.push(None);
            out.push_str(&format!(
                "  keeper {k}: no reading (its ring is not positive)\n"
            ));
            continue;
        };
        // The slip relative to the keeper inside: `slip_k - slip_inside`.
        let rel = c.inside[k]
            .and_then(|i| slips.get(i).cloned().flatten())
            .map_or(Q::ZERO, |si| &r.slip - &si);
        slips.push(Some(r.slip.clone()));
        let lag = &Q::ONE - &keeper.rate;
        let k_own = r.local.floor_i128().unwrap_or(0);
        let head = c.heads.get(k).map(|h| hex(&h[..6])).unwrap_or_default();
        out.push_str(&format!(
            concat!(
                "  {:<12} galaxy {:<2} {} kpc out: its ring {} root windings, 1 - rate {}, ",
                "delay {} root windings, its own count {} ({} windings), slip {} turns ",
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
            k_own,
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
    out.push_str(&bridge(c));
    out.push_str(&checks(u));
    out.push_str(&match &c.anchor {
        Some(p) => format!(
            concat!(
                "Ring 0 on TUniversal: the origin receipt's point, height {}, ",
                "lanes S {} G {} T {} H {}, K {}\n"
            ),
            p.h,
            show(&p.lanes[0]),
            show(&p.lanes[1]),
            show(&p.lanes[2]),
            show(&p.lanes[3]),
            p.k
        ),
        None => concat!(
            "Ring 0 on TUniversal: not anchored ",
            "(the universe is not originated on the network)\n"
        )
        .into(),
    });
    out
}

/// The bridge: the binding to the network on the height, read per segment
/// between consecutive commits that heard a point under one origin, each
/// segment's rings over its heights; the last few segments are printed.
fn bridge(c: &Chronometry) -> String {
    let segments = c.dilations();
    if segments.is_empty() {
        return format!(
            concat!(
                "The bridge: no segment yet ({} commits; a segment needs two commits ",
                "that heard points under one origin with the height advanced)\n"
            ),
            c.commits.len()
        );
    }
    let mut out = format!(
        "The bridge: {} segments, each the rings over the heights between two anchors\n",
        segments.len()
    );
    for s in segments.iter().rev().take(5).rev() {
        out.push_str(&format!(
            "  heights {} to {}: {} rings, dilation {} rings per height\n",
            s.from_h,
            s.to_h,
            show(&s.rings),
            show(&s.dilation)
        ));
    }
    out
}

/// No metronome (docs/chronometry.md): the root's winding is live play's
/// count, its whole windings `K` and the root throat's echo, near and far,
/// with the breath's phase now; the T lane is read apart, from the last
/// final point heard, since only crossings and closures write it.
fn windings_and_lane(u: &Universe<'_>) -> String {
    let root_k = xyl_core::chrono::winding(&(&u.now * &Q::int(360))).unwrap_or(0);
    let s = xyl_cosmos::bounce::torsion_share(&Q::ONE, &xyl_cosmos::circuit::anchor().core_spin);
    let near = xyl_core::chrono::sheared_return(bell(), &s);
    let far = xyl_core::chrono::sheared_return(bell(), &(&Q::ONE - &s));
    let phase = near
        .and_then(|e| xyl_core::dyadic::Dy::from_q(&u.now).div(&e))
        .map(|x| show(&x.to_q().frac()))
        .unwrap_or_else(|| "-".into());
    let lane = u
        .clocks
        .points
        .iter()
        .rev()
        .flatten()
        .next()
        .map(|p| format!("T {} at height {} (K {})", show(&p.lanes[2]), p.h, p.k))
        .unwrap_or_else(|| "no final point heard (unlinked)".into());
    format!(
        concat!(
            "The root: {} whole windings; its throat's echo {} bells on the near face, ",
            "{} on the far (torsion share {}), the breath at phase {} of the echo\n",
            "The T lane (written only by crossings and closures): {}\n"
        ),
        root_k,
        near.map(|e| e.to_string()).unwrap_or_else(|| "none".into()),
        far.map(|e| e.to_string()).unwrap_or_else(|| "none".into()),
        show(&s),
        phase,
        lane
    )
}

/// The clock's checks as live residues (docs/chronometry.md, The telemetry):
/// every keeper's rate in (0, 1]; each keeper's delay the arrival through
/// the keeper inside it, never shorter (time no thinner than reach); the
/// breath returning to its crest over whole echoes; each keeper's rungs
/// the winding of its slip.
fn checks(u: &Universe<'_>) -> String {
    let c = &u.clocks;
    let rates = c
        .keepers
        .iter()
        .filter(|k| k.rate.signum() <= 0 || k.rate > Q::ONE)
        .count();
    let thinner = c
        .keepers
        .iter()
        .enumerate()
        .filter(|(k, keeper)| {
            c.inside
                .get(*k)
                .copied()
                .flatten()
                .and_then(|i| c.keepers.get(i))
                .is_some_and(|inner| keeper.delay < inner.delay)
        })
        .count();
    let delta = xyl_core::ratio!(1, 3);
    let crest = xyl_core::chrono::breath(&delta, &Q::ZERO);
    let breath = (1..4)
        .map(|k| (&xyl_core::chrono::breath(&delta, &Q::int(k)) - &crest).abs())
        .fold(Q::ZERO, Q::max);
    let rungs = c
        .keepers
        .iter()
        .filter_map(|k| k.read(&u.now))
        .filter(|r| {
            r.rung >= 0 && xyl_core::chrono::winding(&(&r.slip * &Q::int(360))) != Some(r.rung)
        })
        .count();
    format!(
        concat!(
            "The checks (residues, zero when the clock holds): rates outside (0, 1] {}; ",
            "keepers reached before the keeper inside them {}; the breath's return {}; ",
            "rungs off their slip's winding {}\n"
        ),
        rates,
        thinner,
        show(&breath),
        rungs
    )
}
