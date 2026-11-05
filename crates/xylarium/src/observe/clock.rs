//! The clock reading: `clock`.

use crate::run::show;
use xyl_core::Q;
use xyl_cosmos::circuit::{bell, gyr_of_rings, gyr_of_rings_dy};
use xyl_kernel::checkpoint::hex;
use xyl_kernel::chronometry::Chronometry;
use xyl_kernel::universe::Universe;

/// The `clock` reading: the bell and the next event, each keeper from the
/// epicenter to the rims read now, the keepers' chain, the bridge (the
/// dilation of the universe's rings against the network's heights, segment
/// by segment) and ring 0's point.
pub(super) fn clock(u: &Universe<'_>) -> String {
    let next = u
        .next_event()
        .map(|t| format!("count {} ({} Gyr)", show(&t), show(&gyr_of_rings(&t))))
        .unwrap_or_else(|| "none queued".into());
    let mut out = format!(
        concat!(
            "The bell: one ring {} Gyr; now count {} (exact: {}); ",
            "the next event {}\n"
        ),
        gyr_of_rings_dy(bell()),
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
        let head = c.heads.get(k).map(|h| hex(&h[..6])).unwrap_or_default();
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
    out.push_str(&bridge(c));
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
