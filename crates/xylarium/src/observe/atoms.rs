//! Law readings of the atoms' bands: `seals Z`, `push N` and `bonding N`.

use super::Options;
use crate::run::show;
use std::collections::BTreeMap;
use xyl_core::Q;
use xyl_cosmos::bang::BigBang;

/// Z's hold on band `n` of atom `a`: tau_b = (Z - electrons of every
/// shallower turning) / n.
fn band_hold(a: &xyl_cosmos::atoms::Atom, z: u32, n: u32) -> Q {
    let shallower: u32 = a
        .bands
        .iter()
        .filter(|c| c.n < n)
        .map(|c| c.electrons)
        .sum();
    Q::ratio(i64::from(z) - i64::from(shallower), i64::from(n))
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
pub(super) fn bonding(top: usize) -> String {
    const CONTACT: u32 = 8;
    let mut out = String::from(
        "Z: Q, Q_Z -> form (contacts c, grip G, conduction sigma, locked surface frames)\n",
    );
    let mut tally: BTreeMap<&str, Vec<u32>> = BTreeMap::new();
    let mut flowing: Vec<(Q, u32)> = Vec::new();
    for z in 1..=u32::try_from(top).unwrap_or(150) {
        let Some(a) = xyl_cosmos::atoms::atom(z) else {
            continue;
        };
        // per open frame: tau_b and the band's free room 1 - omega_b
        let mut frames: Vec<(Q, Q)> = Vec::new();
        for b in a.bands.iter().filter(|b| b.unpaired() > 0) {
            let tau = band_hold(&a, z, b.n);
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
pub(super) fn push_reading(top: usize) -> String {
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
            let tau = band_hold(&a, z, b.n);
            let (held, open) = (3 * b.electrons - b.unpaired(), b.unpaired());
            a_z = &a_z + &(&tau * &Q::int(i64::from(held)));
            q_z = &q_z + &(&tau * &Q::int(i64::from(open)));
        }
        // the built push: 1/2 open / hold, open = compound::open(z) (the
        // electrons' open frames), hold = formation::weight(z)
        let (open, hold) = (
            Q::int(i64::from(xyl_cosmos::compound::open(z))),
            xyl_cosmos::formation::weight(z),
        );
        let half = Q::ratio(1, 2);
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
        concat!(
            "p_Z / p built over open kinds: min {} p50 {} p90 {} max {}; ",
            "the largest p_Z {} at Z{}\n"
        ),
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

/// The quantum capacity under each kind (3Q from the 3Z formulas), read on
/// each atom's own bands: 3Z its filled frames, 3Q three times the room of
/// every shape of every turning it reaches (the quantum state 3Z lives on),
/// R_Q = 6Z - 3Q, rho_Q = (2Z - Q_cap) / Q_cap; the kinds that seal (every
/// winding full and every shape of the outermost turning full: rho_Q = +1)
/// are the convergence points of the table.
pub(super) fn seals_read(o: &Options, top: usize) -> String {
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
        // a seam: unpaired(Z - 1) + 1 - unpaired(Z) < 0, so a single-nucleon
        // peel releases
        if z > 1 {
            let unpaired = |z: u32| i64::from(xyl_cosmos::atoms::unpaired(z));
            if unpaired(z - 1) + 1 - unpaired(z) < 0 {
                seams.push(z);
            }
        }
        // Q_cap = sum over turnings n of sum over shapes l < n of 2 d(l)
        let cap: u32 = (1..=a.outer_n)
            .map(|n| {
                (0..n)
                    .map(|l| 2 * xyl_cosmos::atoms::orientations(l))
                    .sum::<u32>()
            })
            .sum();
        let rho = Q::ratio(2 * i64::from(z) - i64::from(cap), i64::from(cap.max(1)));
        if a.sealed() {
            sealed.push((z, a.outer_n, cap));
        }
        if [1, 2, 28, 115, 208, 1000, 1050, 1092, 1100, 1200, 1600].contains(&z) {
            rows.push_str(&format!(
                concat!(
                    "  Z{:>5}: turning {} capacity {} -> 3Z {} 3Q {} R_Q {} rho_Q {} ",
                    "(open frames {}, sealed {})\n"
                ),
                z,
                a.outer_n,
                cap,
                3 * z,
                3 * cap,
                6 * i64::from(z) - 3 * i64::from(cap),
                show(&rho),
                a.unpaired,
                a.sealed()
            ));
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
            concat!(
                "The vessel (the firmament's band): Z{}..Z{}, {} kinds; ",
                "founded whole from 2^{} hydrogen at the bounce's share {}\n"
            ),
            lo,
            hi,
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
