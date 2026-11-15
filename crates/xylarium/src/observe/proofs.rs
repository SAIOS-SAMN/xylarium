//! The `proofs` reading: each theorem of the register (docs/model.md, Axioms
//! and theorems) that this reading witnesses, read on the seed's universe as
//! its residue, the left side less the right, at T (R rings after the
//! bounce, the first stars by default). An exact identity reads 0; a reading
//! carried on the dyadic grid reads its grid's residue, never rounded away.
//! A law the engine reads at T (the medium's carry about every galaxy) is
//! read at T, from the engine's own value; the theorems witnessed here are
//! identities, so their residues do not change with T.

use super::Options;
use crate::run::{show, Founded};
use xyl_core::dyadic::Dy;
use xyl_core::shares::{dilation, flow};
use xyl_core::Q;

/// The residues at `t_rings` (the first stars by default).
pub(super) fn proofs_read(o: &Options, t_rings: Option<Q>) -> Result<String, String> {
    let Founded { origin, .. } = Founded::of(o.seed)?;
    let sub = xyl_cosmos::cores::substrate(&origin).ok_or("no founding")?;
    let t_rings = t_rings.unwrap_or_else(|| xyl_cosmos::circuit::anchor().first_stars);
    let t = Dy::from_q(&t_rings);
    let a = xyl_cosmos::circuit::anchor().core_spin.clone();
    let s = xyl_cosmos::bounce::torsion_share(&Q::ONE, &a);
    let at = xyl_core::coordinate::beat(&t_rings, 5).unwrap_or_else(|_| show(&t_rings));
    let mut out = format!(
        "The proofs at T {at} (seed {}): the core's spin a {}, its torsion share s {}\n",
        o.seed,
        show(&a),
        show(&s)
    );
    let line = |out: &mut String, id: &str, what: &str, r: Option<Q>| {
        let r = r.map_or_else(|| "refused".to_string(), |r| show(&r));
        out.push_str(&format!("  {id}: {what}: residue {r}\n"));
    };
    let three_a = &Q::int(3) + &a;
    let n = a.checked_div(&(&Q::int(4) + &a));
    // T2: D(s) = 3 + a, and the twist n = 2s - 1.
    line(
        &mut out,
        "T2",
        "D(s) - (3 + a)",
        dilation(&s).map(|d| &d - &three_a),
    );
    line(
        &mut out,
        "T2",
        "(2s - 1) - a / (4 + a)",
        n.as_ref().map(|n| &(&(&Q::int(2) * &s) - &Q::ONE) - n),
    );
    // T4: the echo is the bell over the open share, on the dyadic grid.
    let echo = xyl_cosmos::circuit::echo_at(&sub.share);
    let open = Dy::from_q(&(&Q::ONE - &sub.share));
    line(
        &mut out,
        "T4",
        "echo (1 - s) - bell, at the founded share",
        Some((echo.mul(&open) - xyl_cosmos::circuit::bell()).to_q()),
    );
    // T16: the faces of a paired throat multiply to (3 + a)^2 at every
    // polarity; read at A = 1.
    let faces = dilation(&s)
        .zip(n.as_ref().and_then(dilation))
        .and_then(|(ds, dn)| {
            let far = ds.checked_div(&dn)?;
            Some(&(&(&ds * &dn) * &far) - &(&three_a * &three_a))
        });
    line(&mut out, "T16", "D(s) D(n) D(s) / D(n) - (3 + a)^2", faces);
    // T17: the far face's dilation.
    let far = (&Q::int(6) + &a).checked_div(&(&Q::int(2) + &a));
    line(
        &mut out,
        "T17",
        "D(1 - s) - (6 + a) / (2 + a)",
        dilation(&(&Q::ONE - &s)).zip(far).map(|(d, f)| &d - &f),
    );
    // T20: a pair sharing one hold reads its horizon at one third.
    line(
        &mut out,
        "T20",
        "delta_h(M, M) - 1/3",
        xyl_cosmos::bounce::horizon_share(&Q::ONE, &Q::ONE)
            .zip(Q::new(1, 3))
            .map(|(h, third)| &h - &third),
    );
    // T21: the horizon is axial: at the core's spin, with no vent and no
    // tilt, the axis plane balances at the pair's hold 2M and every turning
    // plane at 2M + aM; read on a unit hold, the largest residue of the six.
    let axial = xyl_cosmos::bounce::horizon_planes((&Q::ONE, &Q::ONE), &Q::ZERO, &a, &Q::ZERO).map(
        |planes| {
            let held = Q::int(2);
            let turned = &held + &a;
            planes
                .iter()
                .enumerate()
                .map(|(p, r)| (r - if p == 0 { &held } else { &turned }).abs())
                .fold(Q::ZERO, Q::max)
        },
    );
    line(
        &mut out,
        "T21",
        "the six planes' reaches - (2M on the axis, 2M + aM on the turning planes)",
        axial,
    );
    // T9 at T: the medium's carry about every galaxy carries out the pair
    // share of the throat's torsion share, s (+) -(1 - s), which is the
    // twist n = 2s - 1 composed with its third.
    let n9 = &(&Q::int(2) * &sub.share) - &Q::ONE;
    let want = n9
        .checked_div(&Q::int(3))
        .and_then(|third| flow(&n9, &third));
    for g in sub.push(t) {
        let c = sub.carry(g.reach0, t);
        // T3 at T: an orbit at the galaxy's reach about the medium held
        // inside it, H(r, t), has the apsis ratio D(delta) = 1 + R_h / r,
        // R_h = 2H the medium's horizon. H is read at T; it stays at its
        // founded hold while the firmament is closed (T8, T16: a sealed kind
        // has no pole state, so none passes the throat).
        let (h, r) = (c.hold.to_q(), g.reach0.to_q());
        let apsis = xyl_core::shares::deficit(&h, &r)
            .and_then(|delta| dilation(&delta))
            .zip(xyl_cosmos::bounce::horizon_reach(&h, &h).checked_div(&r))
            .map(|(d, rh)| &d - &(&Q::ONE + &rh));
        line(
            &mut out,
            "T3",
            &format!(
                "galaxy {} at reach {}, medium {} inside: D(delta) - (1 + R_h / r)",
                g.index, g.reach0, c.hold
            ),
            apsis,
        );
        line(
            &mut out,
            "T9",
            &format!(
                "galaxy {} at reach {}: its carry's outward share - n (+) n / 3",
                g.index, g.reach0
            ),
            want.as_ref().map(|w| &c.outward - w),
        );
    }
    // T24: every founded kind's rest point is the antipole of its axis
    // plane's induced phase, (Z - Q*) / (Z + Q*) = -R_0.
    let (mix, spin) = super::founding::founding_mix(o)?;
    let share = xyl_cosmos::bounce::torsion_share(&Q::ONE, &spin);
    let (_, table, _) = xyl_cosmos::formation::founding_shells(&mix, &spin);
    let common = xyl_core::hopf::COMMON;
    let (mut kinds, mut worst) = (0usize, Q::ZERO);
    for (z, parcels) in &table {
        let capture = xyl_cosmos::circuit::capture_phases(*z, &share, &common);
        let Some([r0, ..]) = xyl_cosmos::circuit::induced_phases(*z, &capture, &common, &share)
        else {
            continue;
        };
        for p in parcels {
            let v = p.pairs.value();
            let pole = (&Q::ONE - &v).checked_div(&(&Q::ONE + &v));
            if let Some(r) = pole.map(|pole| (&pole + &r0).abs()) {
                kinds += 1;
                worst = worst.max(r);
            }
        }
    }
    // T23: in the common winding each face's induced turn moves every
    // founded kind's capture phases only along the fibre, so their relative
    // phases (the point of CP^5) are kept; the largest residue over both
    // faces and every kind.
    use xyl_core::hopf::{induce, Sense, PLANES};
    use xyl_core::rotor::Winding;
    let relative = |x: &[Winding; PLANES]| {
        let t: Vec<Q> = x.iter().map(Winding::turns).collect();
        t.iter().map(|v| v - &t[0]).collect::<Vec<Q>>()
    };
    let (mut reduced, mut kept) = (0usize, Q::ZERO);
    for z in table.keys() {
        let capture = xyl_cosmos::circuit::capture_phases(*z, &share, &common);
        let r: [Winding; PLANES] = std::array::from_fn(|p| Winding::of_turns(&capture[p]));
        for sense in [Sense::Spin, Sense::InverseTorsion] {
            if let Some(moved) = induce(&r, &common, &share, &[sense; PLANES]) {
                reduced += 1;
                let worst = relative(&moved)
                    .iter()
                    .zip(relative(&r))
                    .map(|(a, b)| (a - &b).abs())
                    .fold(Q::ZERO, Q::max);
                kept = kept.max(worst);
            }
        }
    }
    line(
        &mut out,
        "T23",
        &format!("the largest change of a relative phase over {reduced} faces of founded kinds"),
        Some(kept),
    );
    line(
        &mut out,
        "T24",
        &format!("the largest |(Z - Q*) / (Z + Q*) + R_0| over {kinds} founded parcels"),
        Some(worst),
    );
    Ok(out)
}
