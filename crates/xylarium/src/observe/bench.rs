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
//!
//! The Q face, readings only (#148; Mycelium #239): the span `3Q = 3cap`
//! and two derivations of its held count, (a) `W_Q = 3cap - U` (the open
//! states open on both faces) and (b) `W_Q = 3cap - E` (the room the Q
//! face's open count); per derivation the closed share `W_Q / 3Q`, the pole
//! `open / W_Q`, the separation `W_Q - W` from the Z face, and the bits the
//! chain needs to the vessel's top under `W_Q / 3Q`. Then LAW-COUPLING
//! v2's draft (Mycelium #241): `3p`, `3n`, `p / w`, `t = p / n` and the push
//! `p (1 - g) / w` at the capacity face's held share `g = 2P / Z`.

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

/// The capacity of the bands kind `z` has entered: twice the orientations
/// of every band holding at least one electron, leaving out the room of
/// shapes and turnings the atom reaches but has not entered.
fn entered_capacity(z: u32) -> u32 {
    band_occupancy(z)
        .unwrap_or_default()
        .iter()
        .filter(|(_, o)| o.occupied() > 0)
        .map(|(_, o)| o.capacity())
        .sum()
}

/// The candidate closed shares the bench reads beside the built law, by
/// name: held over the capacity (#170), held over the entered capacity, and
/// occupancy over the entered capacity. Readings only; the founding reads
/// the built law (`elements::closed_share`).
const CANDIDATES: [&str; 3] = ["2P/cap", "2P/cap_in", "Z/cap_in"];

fn candidates(z: u32, o: &Occupancy) -> [Q; 3] {
    let cap_in = i64::from(entered_capacity(z));
    let of = |a: u32, b: i64| Q::new(i64::from(a), b).unwrap_or(Q::ZERO);
    [
        of(o.held(), i64::from(o.capacity())),
        of(o.held(), cap_in),
        of(z, cap_in),
    ]
}

/// The Q face's two derivations the bench reads beside the built Z face
/// (#148, #183; Mycelium #239), by name: (a) the open states `U` open on
/// both faces, `W_Q = 3cap - U`; (b) the room `E` the Q face's open count,
/// `W_Q = 3cap - E`. Readings only; the founding reads the Z face.
const Q_FACES: [&str; 2] = ["a: open U", "b: open E"];

/// A kind's Q face: the span `3Q = 3cap`, and per derivation its open count
/// and held count `W_Q = 3Q - open`.
struct QFace {
    span: u32,
    open: [u32; 2],
    held: [u32; 2],
}

fn q_face(o: &Occupancy) -> QFace {
    let span = 3 * o.capacity();
    let open = [o.open, o.room];
    QFace {
        span,
        open,
        held: open.map(|x| span - x),
    }
}

/// The nucleon dipole of LAW-COUPLING v2's draft (Mycelium #241, 78.181
/// section 2): `3p` and `3n` from the body (`n = A - Z`), the axis share
/// `p / w` and `t = p / n`, and the push `pi = p (1 - g) / w` with its
/// antipole `1 - pi`. Xylarium has no `g` of its own; the bench reads it as
/// the capacity face's held share `g = 2P / Z`, under which
/// `pi = U / w` exactly. Readings only.
struct Nucleons {
    p: u32,
    n: u32,
    pi: Q,
}

fn nucleon_dipole(z: u32, mass: u32, o: &Occupancy) -> Option<Nucleons> {
    let n = mass.checked_sub(z)?;
    let w = i64::from(mass);
    let g = Q::new(i64::from(o.held()), i64::from(z))?;
    let pi = &Q::new(i64::from(z), w)? * &(&Q::ONE - &g);
    Some(Nucleons { p: z, n, pi })
}

/// `a / b` as a reading, `-` where `b` is zero.
fn ratio(a: u32, b: u32) -> String {
    if b == 0 {
        "-".to_string()
    } else {
        show(&Q::new(i64::from(a), i64::from(b)).unwrap_or(Q::ZERO))
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
            "  Z A | E U 2P cap cap_in | W h | candidates 2P/cap 2P/cap_in Z/cap_in | ",
            "step | t U/W, U/2P | emission U/W, face U/Z | count per H (a h chain) | ",
            "founded whole | Q face 3Q, W_Q a b, h_Q = W_Q/3Q a b, t_Q = open/W_Q a b, ",
            "W_Q - W a b | v2: 3p 3n, p/w, t = p/n, pi = p(1 - g)/w at g = 2P/Z, 1 - pi\n"
        ),
        seed,
        show(&spin),
        show(&share),
        bits
    );
    // The unfloored chain per hydrogen, prod_{2..=z} a h_z, read as a
    // dyadic; the founded counts are the exact floored chain.
    let mut chain = Dy::ONE;
    // The same chain under each candidate share, to the vessel's top.
    let vessel_top = xyl_cosmos::atoms::vessel().map_or(top, |(_, hi)| hi);
    let mut trials = [Dy::ONE; 3];
    // The chain under each Q face's closed share `W_Q / 3Q`.
    let mut q_trials = [Dy::ONE; 2];
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
        let cand = candidates(z, &o);
        let qf = q_face(&o);
        let q_share = qf
            .held
            .map(|w| Q::new(i64::from(w), i64::from(qf.span)).unwrap_or(Q::ZERO));
        if z > 1 && z <= vessel_top {
            for (t, h) in trials.iter_mut().zip(&cand) {
                *t = *t * a * Dy::from_q(h);
            }
            for (t, h) in q_trials.iter_mut().zip(&q_share) {
                *t = *t * a * Dy::from_q(h);
            }
        }
        if z < lo || z > hi {
            continue;
        }
        let w = xyl_cosmos::compound::hold(z);
        let u = xyl_cosmos::compound::open(z);
        out.push_str(&format!(
            concat!(
                "  Z{z} A{mass} | {} {} {} {} {} | {w} {} | {} {} {} | {} | ",
                "{}, {} | {}, {} | {} | {} | {} {} {} {} {} {} {} {} {} | {}\n"
            ),
            o.room,
            o.open,
            o.held(),
            o.capacity(),
            entered_capacity(z),
            show(&h),
            show(&cand[0]),
            show(&cand[1]),
            show(&cand[2]),
            step(z),
            ratio(u, w),
            ratio(o.open, o.held()),
            show(&xyl_cosmos::formation::emission(z)),
            ratio(o.open, z),
            chain,
            show(&count),
            qf.span,
            qf.held[0],
            qf.held[1],
            show(&q_share[0]),
            show(&q_share[1]),
            ratio(qf.open[0], qf.held[0]),
            ratio(qf.open[1], qf.held[1]),
            i64::from(qf.held[0]) - i64::from(w),
            i64::from(qf.held[1]) - i64::from(w),
            nucleon_dipole(z, mass, &o).map_or("-".to_string(), |v| format!(
                "{} {}, {}, {}, {}, {}",
                3 * v.p,
                3 * v.n,
                ratio(v.p, v.p + v.n),
                ratio(v.p, v.n),
                show(&v.pi),
                show(&(&Q::ONE - &v.pi))
            )),
            z = z,
            mass = mass,
            w = w,
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
    // The bits each chain needs to carry one nucleus of the vessel's top
    // kind: -log2 of its unfloored count per hydrogen, a reading (the
    // floored chain can need a bit more, as `elements::founding_bits` does).
    let ln2 = Dy::int(2)
        .ln()
        .unwrap_or_else(|| xyl_core::dyadic::Dy::int(-200) * xyl_core::dyadic::ln2());
    let needs: Vec<String> = CANDIDATES
        .iter()
        .zip(&trials)
        .map(|(name, t)| {
            format!(
                "{name} {}",
                -(t.ln()
                    .unwrap_or_else(|| xyl_core::dyadic::Dy::int(-200) * xyl_core::dyadic::ln2())
                    .div(&ln2)
                    .unwrap_or_default())
            )
        })
        .collect();
    out.push_str(&format!(
        "Bits to the vessel's top (Z{vessel_top}) under each closed share: built {bits}; {}\n",
        needs.join(", ")
    ));
    let q_needs: Vec<String> = Q_FACES
        .iter()
        .zip(&q_trials)
        .map(|(name, t)| {
            format!(
                "{name} {}",
                -(t.ln()
                    .unwrap_or_else(|| xyl_core::dyadic::Dy::int(-200) * xyl_core::dyadic::ln2())
                    .div(&ln2)
                    .unwrap_or_default())
            )
        })
        .collect();
    out.push_str(&format!(
        "Bits to the vessel's top under the Q face's closed share W_Q / 3Q: {}\n",
        q_needs.join(", ")
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
    fn the_entered_capacity_lies_between_the_occupancy_and_the_capacity() {
        // The entered bands hold every electron and are part of the reach,
        // so Z <= cap_in <= cap; at a seal every band is entered and full.
        for z in 1..=xyl_cosmos::elements::TOP_Z {
            let o = occupancy(z).unwrap_or_default();
            let cap_in = entered_capacity(z);
            assert!(z <= cap_in && cap_in <= o.capacity(), "Z{z}: {cap_in}");
            if o.room == 0 {
                assert_eq!(cap_in, z, "Z{z}");
            }
        }
    }

    #[test]
    fn the_q_faces_meet_the_z_face_at_the_seals_and_part_by_the_room() {
        // With 3Q = 3cap and cap = E + U + 2P against W = 3Z - U: (a) parts
        // from the Z face by 3E, (b) by 2E + U; both meet it exactly at the
        // seals (E = U = 0), and no held count exceeds its span.
        for z in 1..=xyl_cosmos::elements::TOP_Z {
            let o = occupancy(z).unwrap_or_default();
            let qf = q_face(&o);
            let w = i64::from(xyl_cosmos::compound::hold(z));
            let (e, u) = (i64::from(o.room), i64::from(o.open));
            assert_eq!(i64::from(qf.held[0]) - w, 3 * e, "Z{z}");
            assert_eq!(i64::from(qf.held[1]) - w, 2 * e + u, "Z{z}");
            let sealed = o.room == 0 && o.open == 0;
            assert_eq!(qf.held.iter().all(|h| i64::from(*h) == w), sealed, "Z{z}");
            assert!(qf.held.iter().all(|h| *h <= qf.span), "Z{z}");
        }
    }

    #[test]
    fn the_v2_push_at_the_held_share_is_the_open_states_over_the_nucleons() {
        // With g = 2P / Z, pi = p (1 - g) / w = U / w: the push the draft
        // sets apart from U reads U again under xylarium's held share.
        for z in 1..=xyl_cosmos::elements::TOP_Z {
            let o = occupancy(z).unwrap_or_default();
            let Some(mass) = xyl_cosmos::circuit::mass_number(z) else {
                continue;
            };
            let v = nucleon_dipole(z, mass, &o).unwrap();
            assert_eq!(v.p + v.n, mass, "Z{z}");
            assert_eq!(
                v.pi,
                Q::new(i64::from(o.open), i64::from(mass)).unwrap(),
                "Z{z}"
            );
        }
    }

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
