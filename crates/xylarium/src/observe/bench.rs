//! The substrate bench: `bench [LO] [HI]`, the substrate read alone, with
//! no founding (docs/board.md, The substrate bench; #163).
//!
//! Reads the seed's drawn spin (`origin::drawn_spin`) and its torsion share
//! (`bounce::torsion_share`), the atoms (`atoms::occupancy`,
//! `atoms::band_occupancy`, `atoms::vessel`), the element table
//! (`elements::table`), the capture chain (`elements::closed_share`,
//! `elements::founding_bits`, `elements::founded`), the built hold and open
//! states (`compound::hold`, `compound::open`), the emission
//! (`formation::emission`) and the nucleon count (`circuit::mass_number`).
//! Nothing above the chain is built: no founding shells, no core, no cache
//! read or written.
//!
//! Per kind: the occupancy `E U 2P cap`, the built held count `W`, the
//! closed share `h`, the capture's step (the bands the Z-th electron
//! changes, open `U` and pairs `P` per band), the pole `U / W` (built) and
//! `U / 2P` (axiom 9), the emission, the chain's count per hydrogen
//! (unfloored, `prod a h`) and the founded whole count. Then the chain's
//! totals: the founding bits, the kinds founded whole, the mean nucleon
//! count by count and the founded share of each seal. Exact values are
//! shown as dyadic readings.

use crate::run::show;
use xyl_core::dyadic::Dy;
use xyl_core::Q;
use xyl_cosmos::atoms::{band_occupancy, occupancy, Band, Occupancy};

/// The capture's step into kind `z`: every band `(n, l)` whose open states
/// or pairs change from kind `z - 1`, with the change `(dU, dP)`, and the
/// capacity's growth where the atom reaches a new turning.
fn capture_step(z: u32) -> (Vec<(u32, u32, i64, i64)>, i64) {
    let bands = |z: u32| -> Vec<(Band, Occupancy)> {
        if z == 0 {
            Vec::new()
        } else {
            band_occupancy(z).unwrap_or_default()
        }
    };
    let (before, after) = (bands(z.saturating_sub(1)), bands(z));
    let find = |v: &[(Band, Occupancy)], n: u32, l: u32| {
        v.iter()
            .find(|(b, _)| b.n == n && b.l == l)
            .map(|(_, o)| *o)
            .unwrap_or_default()
    };
    // Every band either kind reaches: a seam can empty an outer band.
    let mut keys: Vec<(u32, u32)> = before
        .iter()
        .chain(&after)
        .map(|(b, _)| (b.n, b.l))
        .collect();
    keys.sort_unstable();
    keys.dedup();
    let changes = keys
        .into_iter()
        .map(|(n, l)| {
            let (p, o) = (find(&before, n, l), find(&after, n, l));
            (
                n,
                l,
                i64::from(o.open) - i64::from(p.open),
                i64::from(o.pairs) - i64::from(p.pairs),
            )
        })
        .filter(|(_, _, du, dp)| *du != 0 || *dp != 0)
        .collect();
    let cap = |v: &[(Band, Occupancy)]| v.iter().map(|(_, o)| i64::from(o.capacity())).sum::<i64>();
    (changes, cap(&after) - cap(&before))
}

/// The capture's step written `n,l U+du P+dp`, with `cap+k` where the
/// capacity grows past hydrogen's.
fn step(z: u32) -> String {
    let (changes, grown) = capture_step(z);
    let mut parts: Vec<String> = changes
        .iter()
        .map(|(n, l, du, dp)| format!("{n},{l} U{du:+} P{dp:+}"))
        .collect();
    if grown != 0 && z > 1 {
        parts.push(format!("cap{grown:+}"));
    }
    parts.join("; ")
}

/// `a / b` as a reading, `-` where `b` is zero.
fn ratio(a: u32, b: u32) -> String {
    if b == 0 {
        "-".to_string()
    } else {
        show(&Q::ratio(i64::from(a), i64::from(b)))
    }
}

/// The `bench [LO] [HI]` reading for `seed`: per kind from `lo` to `hi`,
/// then the chain's totals over the whole table.
pub(super) fn bench_read(seed: u64, lo: u32, hi: u32) -> Result<String, String> {
    let clock = std::time::Instant::now();
    let spin = xyl_cosmos::origin::drawn_spin(seed);
    let share = xyl_cosmos::bounce::torsion_share(&Q::ONE, &spin);
    let top = xyl_cosmos::elements::TOP_Z;
    let (lo, hi) = (lo.max(1), hi.min(top));
    let founded = xyl_cosmos::elements::founded(&share)?;
    let bits = xyl_cosmos::elements::founding_bits(&share);
    let hydrogen = (0..bits).fold(Q::ONE, |acc, _| &acc * &Q::int(2));
    let a = Dy::from_q(&share);
    let mut out = format!(
        concat!(
            "The substrate bench (seed {}, no founding): the parent's spin {}, ",
            "torsion share a {}; the chain founds from 2^{} hydrogen\n",
            "  Z A | E U 2P cap | W h | step | t U/W, U/2P | emission | ",
            "count per H (a h chain) | founded whole\n"
        ),
        seed,
        show(&spin),
        show(&share),
        bits
    );
    // The unfloored chain per hydrogen, prod_{2..=z} a h_z, read as a
    // dyadic; the founded counts are the exact floored chain.
    let mut chain = Dy::ONE;
    let (mut nuclei, mut nucleons, mut whole_kinds) = (Q::ZERO, Q::ZERO, 0u32);
    let mut seals = Vec::new();
    for z in 1..=top {
        let h = xyl_cosmos::elements::closed_share(z);
        if z > 1 {
            chain = chain * a * Dy::from_q(&h);
        }
        let count = founded
            .iter()
            .find(|e| e.z == z)
            .map(|e| e.abundance.clone())
            .unwrap_or(Q::ZERO);
        let mass = xyl_cosmos::circuit::mass_number(z).unwrap_or(0);
        if count >= Q::ONE {
            whole_kinds += 1;
            nuclei = &nuclei + &count;
            nucleons = &nucleons + &(&count * &Q::int(i64::from(mass)));
        }
        let Some(o) = occupancy(z) else {
            continue;
        };
        if o.room == 0 && o.open == 0 {
            seals.push((z, count.clone()));
        }
        if z < lo || z > hi {
            continue;
        }
        let w = xyl_cosmos::compound::hold(z);
        let u = xyl_cosmos::compound::open(z);
        out.push_str(&format!(
            "  Z{z} A{mass} | {} {} {} {} | {w} {} | {} | {}, {} | {} | {} | {}\n",
            o.room,
            o.open,
            o.held(),
            o.capacity(),
            show(&h),
            step(z),
            ratio(u, w),
            ratio(o.open, o.held()),
            show(&xyl_cosmos::formation::emission(z)),
            chain,
            show(&count),
        ));
    }
    let mean = nucleons.checked_div(&nuclei).unwrap_or(Q::ZERO);
    out.push_str(&format!(
        "The chain: {} of {} kinds founded whole; mean nucleon count by count {}\n",
        whole_kinds,
        top,
        show(&mean)
    ));
    let seals: Vec<String> = seals
        .iter()
        .map(|(z, n)| {
            let share = n.checked_div(&hydrogen).unwrap_or(Q::ZERO);
            format!("Z{z} {}", show(&share))
        })
        .collect();
    out.push_str(&format!(
        "The seals' counts per hydrogen: {}\n",
        seals.join(", ")
    ));
    if let Some((vlo, vhi)) = xyl_cosmos::atoms::vessel() {
        out.push_str(&format!("The vessel: Z{vlo}..Z{vhi}\n"));
    }
    let ms = clock.elapsed().as_millis();
    out.push_str(&format!("Read in {}.{:03} s\n", ms / 1000, ms % 1000));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_capture_adds_one_electron() {
        // Each kind holds one electron more than the kind before it: over
        // the bands a capture changes, the open states and twice the pairs
        // grow by one.
        for z in 1..=xyl_cosmos::elements::TOP_Z {
            let (changes, grown) = capture_step(z);
            let added: i64 = changes.iter().map(|(_, _, du, dp)| du + 2 * dp).sum();
            assert_eq!(added, 1, "Z{z}: {changes:?} (capacity {grown:+})");
        }
    }
}
