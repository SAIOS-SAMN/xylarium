//! Law readings of the atoms' bands: `seals Z`, `push N` and `bonding N`.

use super::Options;
use crate::run::show;
use std::collections::BTreeMap;
use xyl_core::Q;
use xyl_cosmos::origin::Origin;

/// The `bonding N` reading: how atoms of one kind bond into a body,
/// converging in Q over R, with Z incorporated into Q through the nucleus's
/// hold (without it the reading is only linear) and limited by each band's
/// orthogonal capacity. Each open band b of the atom carries u_b open states
/// and d_b = fibres::directions(l) mutually orthogonal directions on S^11 (the
/// band's capacity), and is held by
/// tau_b = (Z - the electrons of the shells inside its reach) / n:
///     Q_Z = sum u_b tau_b                 (Z in Q)
///     omega_b = u_b / d_b                 (the share of the capacity in use)
/// In a body (three planes and R, with 8 like atoms touching one, a declared
/// count) the most tightly held count take the contacts first, and the
/// body's grip is G = sum tau over the bonded states. A state beyond the
/// contacts in a band with free orthogonal room (1 - omega_b) can turn into
/// an empty orientation and conduct; one in a full band is locked on the
/// surface. The conduction is the share:
///     sigma = sum_surplus u_b (1 - omega_b) / tau_b  /  sum u_b / tau_b
/// The form follows from the contact count c: 0 inert, 1 molecule, 2 chain,
/// 3 or 4 sheet, more lattice. The reading is diagnostic; no law uses it.
pub(super) fn bonding(top: usize) -> String {
    const CONTACT: u32 = 8;
    let mut out = String::from(
        "Z: Q, Q_Z -> form (contacts c, grip G, conduction sigma, locked surface states)\n",
    );
    let mut tally: BTreeMap<&str, Vec<u32>> = BTreeMap::new();
    let mut flowing: Vec<(Q, u32)> = Vec::new();
    for z in 1..=u32::try_from(top).unwrap_or(150) {
        let Some(a) = xyl_cosmos::atoms::atom(z) else {
            continue;
        };
        // Per open state: tau_b and the band's free room 1 - omega_b.
        let mut bonded: Vec<(Q, Q)> = Vec::new();
        for b in a.bands.iter().filter(|b| b.unpaired() > 0) {
            let tau = a.winding_tightness(b);
            let d = xyl_cosmos::fibres::directions(b.l).max(1);
            let free = &Q::ONE - &Q::new(i64::from(b.unpaired()), i64::from(d)).unwrap_or(Q::ONE);
            for _ in 0..b.unpaired() {
                bonded.push((tau.clone(), free.clone()));
            }
        }
        bonded.sort_by(|x, y| y.0.cmp(&x.0));
        let q = u32::try_from(bonded.len()).unwrap_or(0);
        let q_z = bonded.iter().fold(Q::ZERO, |acc, f| &acc + &f.0);
        let c = q.min(CONTACT);
        let grip = bonded
            .iter()
            .take(c as usize)
            .fold(Q::ZERO, |acc, f| &acc + &f.0);
        let loose = |f: &(Q, Q)| Q::ONE.checked_div(&f.0).unwrap_or(Q::ZERO);
        let all = bonded.iter().fold(Q::ZERO, |acc, f| &acc + &loose(f));
        let flow = bonded
            .iter()
            .skip(c as usize)
            .fold(Q::ZERO, |acc, f| &acc + &(&loose(f) * &f.1));
        let sigma = flow.checked_div(&all).unwrap_or(Q::ZERO);
        let locked = bonded
            .iter()
            .skip(c as usize)
            .filter(|f| f.1.is_zero())
            .count();
        let form = match c {
            0 => "inert",
            1 => "molecule",
            2 => "chain",
            3 | 4 => "sheet",
            _ => "lattice",
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

/// The `push N` reading: each kind's push with Z in both poles (Z
/// incorporated into Q; Q_Z alone would lose the mass in the denominator).
/// Each band b holds 3 e_b units of the built held count, u_b of them open and 3 e_b - u_b held,
/// all under the nucleus's hold tau_b, so:
///     A_Z = sum (3 e_b - u_b) tau_b,  Q_Z = sum u_b tau_b,
///     A_Z + Q_Z = 3 sum e_b tau_b      (the mass in Z; 3Z where every tau is 1)
///     p_Z = 1/2 Q_Z / (A_Z + Q_Z)      (at most 1/2)
/// It is compared with the built push 1/2 Q / A (emission over held) and the
/// linear share 1/2 Q / (A + Q). The reading is diagnostic; no law uses it.
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
            let tau = a.winding_tightness(b);
            let (held, open) = (3 * b.electrons - b.unpaired(), b.unpaired());
            a_z = &a_z + &(&tau * &Q::int(i64::from(held)));
            q_z = &q_z + &(&tau * &Q::int(i64::from(open)));
        }
        // The built push is `1/2 open / hold`, with `open = compound::open(z)`
        // (the electrons' open states) and `hold = compound::electron_hold(z)`.
        let (open, hold) = (
            Q::int(i64::from(xyl_cosmos::compound::open(z))),
            Q::int(i64::from(xyl_cosmos::compound::electron_hold(z))),
        );
        let half = xyl_core::ratio!(1, 2);
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

/// The `seals Z` reading: the capacity under each kind up to `top`
/// (`atoms::capacity`, `atoms::balance`): 3Z the span, 3Q = 3 cap, then
/// R_Q = 6Z - 3Q and rho_Q = (2Z - cap) / cap. The kinds that seal (rho_Q =
/// +1, Z = cap) are the dipole's equator, each with its pole point
/// t = u / w. The reading lists the seams and the vessel, prints rows for
/// selected kinds, and then the founded mix's dipole against the throat's.
pub(super) fn seals_read(o: &Options, top: usize) -> String {
    let substrate =
        Origin::of(o.seed).and_then(|b| xyl_cosmos::cores::substrate_for(&b.parent.spin));
    let share = substrate.as_ref().map(|s| s.share.clone());
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
        // A seam: `unpaired(Z - 1) + 1 - unpaired(Z) < 0`, so a single-nucleon
        // peel releases.
        if z > 1 {
            let unpaired = |z: u32| i64::from(xyl_cosmos::atoms::unpaired(z));
            if unpaired(z - 1) + 1 - unpaired(z) < 0 {
                seams.push(z);
            }
        }
        let (Some(cap), Some((r_q, rho))) = (
            xyl_cosmos::atoms::capacity(z),
            xyl_cosmos::atoms::balance(z),
        ) else {
            continue;
        };
        // The built pole point t = u / w, open states over the held count.
        let t = Q::new(
            i64::from(a.unpaired),
            i64::from(xyl_cosmos::compound::electron_hold(z).max(1)),
        )
        .unwrap_or(Q::ZERO);
        let occ = xyl_cosmos::atoms::occupancy(z).unwrap_or_default();
        if a.sealed() {
            sealed.push((z, a.outer_n, cap, t.clone()));
        }
        if [1, 2, 28, 115, 208, 1000, 1050, 1092, 1100, 1200, 1600].contains(&z) {
            rows.push_str(&format!(
                concat!(
                    "  Z{:>5}: turning {} capacity {} -> 3Z {} 3Q {} R_Q {} rho_Q {} ",
                    "(room E {}, open states U {}, held states 2P {}; ",
                    "pole point t = U / W {} built, U / 2P {}; sealed {})\n"
                ),
                z,
                a.outer_n,
                cap,
                3 * z,
                3 * cap,
                r_q,
                show(&rho),
                occ.room,
                occ.open,
                occ.held(),
                show(&t),
                if occ.pairs == 0 {
                    "none (no held state)".to_string()
                } else {
                    show(&Q::new(i64::from(occ.open), i64::from(occ.held())).unwrap_or(Q::ZERO))
                },
                a.sealed()
            ));
            let from_q = xyl_cosmos::circuit::universal_fill_shares(z);
            if let (Some(sides), Some(weights)) = (
                xyl_cosmos::atoms::plane_sides(z, &from_q),
                xyl_cosmos::atoms::plane_weights(z, &from_q),
            ) {
                let side =
                    |o: &(Q, Q)| format!("U {} 2P {}", show(&o.0), show(&(&o.1 * &Q::int(2))));
                rows.push_str(&format!(
                    concat!(
                        "         axis plane: Z side ({}), neutral ({}), Q side ({}); ",
                        "weights: open (A) {}, held {}; A_p on the six planes: {}\n"
                    ),
                    side(&sides[0][0]),
                    side(&sides[0][1]),
                    side(&sides[0][2]),
                    show(&weights[0].0),
                    show(&weights[0].1),
                    super::bench::planes_shown(&weights.clone().map(|w| w.0))
                ));
            }
        }
    }
    out.push_str(&format!(
        "Sealed kinds: {}\n",
        sealed
            .iter()
            .map(|(z, n, c, t)| format!(
                "Z{z} (turning {n}, capacity {c}, pole point t {})",
                show(t)
            ))
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
    if let Some(s) = substrate.as_deref() {
        out.push_str(&dipole_read(s));
    }
    out
}

/// The founded mix's dipole against the throat's (docs/model.md, The
/// dipole; #151): the throat's pole shares `s_Q = s`, `s_Z = 1 - s` from the
/// torsion share, their ratio `1 + a / 2` and twist `n = 2s - 1`; then the
/// firmament's and the whole founding's `3Q / 3Z` and
/// `n = (3Q - 3Z) / (3Q + 3Z)`, with `3Q` read as the neutrons and as the
/// capacity, over whole nuclei (`circuit::mix_weights`,
/// `circuit::mix_capacity`). `R` is not in this comparison.
fn dipole_read(s: &xyl_cosmos::circuit::Substrate) -> String {
    let ratio = |q: &Q, z: &Q| q.checked_div(z).unwrap_or(Q::ZERO);
    let twist = |q: &Q, z: &Q| (q - z).checked_div(&(q + z)).unwrap_or(Q::ZERO);
    let s_q = s.share.clone();
    let s_z = &Q::ONE - &s_q;
    let mut out = format!(
        concat!(
            "The dipole against the throat (#151): the throat's s_Q {} s_Z {}, ",
            "s_Q / s_Z {}, twist n {}\n"
        ),
        show(&s_q),
        show(&s_z),
        show(&ratio(&s_q, &s_z)),
        show(&twist(&s_q, &s_z))
    );
    let firmament: xyl_cosmos::circuit::Mix = s
        .firmament
        .iter()
        .map(|k| (k.z, k.amount.clone()))
        .collect();
    let mut founded = firmament.clone();
    for b in &s.beads {
        for (k, a) in &b.recipe {
            let held = founded.entry(*k).or_insert(Q::ZERO);
            *held = &*held + a;
        }
    }
    for (label, mix) in [("firmament", &firmament), ("whole founding", &founded)] {
        let w = xyl_cosmos::circuit::mix_weights(mix);
        let cap = xyl_cosmos::circuit::mix_capacity(mix);
        out.push_str(&format!(
            concat!(
                "  {}: 3Z {}; as neutrons 3Q {} (3Q / 3Z {}, n {}); ",
                "as capacity 3Q {} (3Q / 3Z {}, n {})\n"
            ),
            label,
            show(&(&w.protons * &Q::int(3))),
            show(&(&w.neutrons * &Q::int(3))),
            show(&ratio(&w.neutrons, &w.protons)),
            show(&twist(&w.neutrons, &w.protons)),
            show(&(&cap * &Q::int(3))),
            show(&ratio(&cap, &w.protons)),
            show(&twist(&cap, &w.protons))
        ));
    }
    out
}

/// The `bond A B [T]` reading: the bond of atoms `A` and `B` read by its
/// planes (`atoms::Bond`). On the axis plane, each face's pole push and
/// polarity and the flow law's composition of the two pushes; on the
/// isoclinic planes, every open winding of each face with its tightness and
/// the share of it a received isoclinic turning `T` (default 0) leaves held;
/// between planes, nothing in founded matter (T18).
pub(super) fn bond_reading(a: u32, b: u32, turning: &Q) -> Result<String, String> {
    let (fa, fb) = (
        xyl_cosmos::atoms::atom(a).ok_or_else(|| format!("no atom of Z {a}"))?,
        xyl_cosmos::atoms::atom(b).ok_or_else(|| format!("no atom of Z {b}"))?,
    );
    let bond = xyl_cosmos::atoms::bond(&fa, &fb);
    let received = xyl_cosmos::atoms::Received::isoclinic(turning.clone());
    let pushes = bond.pole_pushes();
    let shares = [a, b].map(xyl_cosmos::circuit::universal_fill_shares);
    let polarity = bond.polarities([&shares[0], &shares[1]]);
    let mut out = format!(
        "The bond of Z {a} and Z {b} at an isoclinic turning of {}\n",
        crate::run::show(turning)
    );
    out.push_str(&format!(
        "  Axis plane: pole pushes {} and {}, separation {}; polarities {} and {}\n",
        crate::run::show(&pushes[0]),
        crate::run::show(&pushes[1]),
        bond.axis_separation()
            .map(|x| crate::run::show(&x))
            .unwrap_or_else(|| "-".into()),
        polarity[0]
            .as_ref()
            .map_or_else(|| "-".into(), super::bench::planes_shown),
        polarity[1]
            .as_ref()
            .map_or_else(|| "-".into(), super::bench::planes_shown),
    ));
    for (face, (z, atom)) in bond.held(&received).iter().zip([(a, &fa), (b, &fb)]) {
        out.push_str(&format!("  Isoclinic planes, Z {z}:"));
        for (w, band) in face
            .iter()
            .zip(atom.bands.iter().filter(|x| x.unpaired() > 0))
        {
            out.push_str(&format!(
                " ({}, {}) {} open, tightness {}, held {};",
                w.n,
                w.l,
                w.unpaired,
                crate::run::show(&atom.winding_tightness(band)),
                crate::run::show(&w.held)
            ));
        }
        out.push('\n');
    }
    out.push_str("  Between planes: none in founded matter (T18)\n");
    Ok(out)
}
