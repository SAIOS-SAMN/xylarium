//! The substrate law reading: `universal-core [R]`.

use super::Options;
use crate::run::{show, Founded};
use xyl_core::dyadic::Dy;
use xyl_core::Q;
use xyl_cosmos::circuit::{gyr_of_rings_dy, PoleFlux, Substrate};
use xyl_cosmos::origin::Origin;

/// A rate per ring read per Gyr, for display only.
fn per_gyr(x: Dy) -> Dy {
    x.div(&gyr_of_rings_dy(Dy::ONE).max(Dy::sci(1, -300)))
        .unwrap_or_default()
}

/// The `universal-core [R]` reading, the substrate `t_rings` rings of the
/// bell after the bounce (None: at the first stars): the core's
/// firmament kind by kind (its reach, the hold inside it, its deficit, the
/// share of its band the reverberation carries past the throat, its rate of
/// leaving and what it still holds), the jetstream, the lighting, the
/// outflow, the ocean's share and delay for every galaxy and what each has
/// taken, the books, and Z, Q and R. Every value is a closed form at T; no
/// echo is stepped and no parcel counted. The time is read in rings and
/// written on the T lane; Gyr is shown beside it as a display conversion.
pub(super) fn core_read(o: &Options, t_rings: Option<Q>) -> Result<String, String> {
    let Founded {
        origin, elements, ..
    } = Founded::of(o.seed)?;
    let clock = std::time::Instant::now();
    let s = xyl_cosmos::cores::substrate(&origin).ok_or("no founding")?;
    let founded_in = clock.elapsed();
    let t_rings = t_rings.unwrap_or_else(|| xyl_cosmos::circuit::anchor().first_stars);
    let t = Dy::from_q(&t_rings);
    let at = xyl_core::coordinate::beat(&t_rings, 5).unwrap_or_else(|_| show(&t_rings));
    let mut out = format!(
        concat!(
            "The substrate at T {} ({} Gyr) (seed {}): founded in {:.1?}; the bell {} Gyr, ",
            "an echo {} Gyr after the core's bounce; {} firmament kinds, ",
            "{} beads ({} hold a sun), ",
            "rim {} cluster units\n"
        ),
        at,
        gyr_of_rings_dy(t),
        o.seed,
        founded_in,
        gyr_of_rings_dy(xyl_cosmos::circuit::bell()),
        gyr_of_rings_dy(xyl_cosmos::circuit::echo()),
        s.firmament.len(),
        s.beads.len(),
        s.galaxies().len() - 1,
        show(&s.rim)
    );
    // Each face's echo (T4, axiom 7): the near face returns after
    // bell / (1 - s), the far face after bell / s, at the founded share and
    // at the share of the spin after the core's bounce.
    let released =
        xyl_cosmos::bounce::torsion_share(&Q::ONE, &xyl_cosmos::circuit::anchor().core_spin);
    let faces = |share: &Q| {
        let far = Q::ONE
            .checked_div(share)
            .map(|x| Dy::from_q(&x))
            .unwrap_or(Dy::ZERO);
        (xyl_cosmos::circuit::echo_at(share), far)
    };
    let (near0, far0) = faces(&s.share);
    let (near1, far1) = faces(&released);
    out.push_str(&format!(
        concat!(
            "  The echo (T4): founded at share {}, near face {} rings, far face {} rings; ",
            "after the core's bounce at share {}, near face {} rings, far face {} rings\n"
        ),
        show(&s.share),
        near0,
        far0,
        show(&released),
        near1,
        far1
    ));
    out.push_str(&core_state(&s, t));
    let at_mean: Vec<Option<PoleFlux>> = (0..s.reverb.len())
        .map(|i| s.loop_flux_swung(i, t, Dy::ONE))
        .collect();
    out.push_str(&jetstream(&s, t, &at_mean));
    out.push_str(&format!(
        concat!(
            "  The lighting (the home galaxy's gas, the beads one tier down): ",
            "first lights after {} bells, the lit share {}, {} pieces of gas ({} light)\n"
        ),
        s.lighting
            .cools_in
            .map(|n| n.to_string())
            .unwrap_or_else(|| "never".into()),
        show(&s.lighting.lit),
        s.lighting.pieces.len(),
        s.lighting.pieces.iter().filter(|p| p.2.is_some()).count()
    ));
    out.push_str(&fastest_kinds(&s, t, &at_mean));
    out.push_str(&ocean(&s, t));
    out.push_str(&galaxies(&origin, &s, t));
    let w = s.two_weights(t, &elements);
    let p = s.pulse(t);
    out.push_str(&format!(
        concat!(
            "  Z, Q and R at T: protons {} neutrons {} (Q share {}); ",
            "the pulse p = {} turns, R = kappa (1 - 2p) per kind\n"
        ),
        show(&w.protons),
        show(&w.neutrons),
        w.neutron_share()
            .as_ref()
            .map(show)
            .unwrap_or_else(|| "-".into()),
        show(&p)
    ));
    out.push_str(&format!("  read in {:.1?}\n", clock.elapsed()));
    Ok(out)
}

/// The core's state at T: the kinds reaching past the throat, the paired
/// throat's spin orbit (its cohorts and the core's spin after them), the
/// inbound echo and the push it swings, and the kinds past the throat under
/// it.
fn core_state(s: &Substrate, t: Dy) -> String {
    let rv = &s.reverb;
    let anchor = xyl_cosmos::circuit::anchor();
    let leaving = rv.iter().filter(|r| r.beyond.signum() > 0).count();
    let mut out = format!(
        concat!(
            "  The core: {} of {} firmament kinds reach past the throat over the bell's swing; ",
            "the rim at {} cluster units\n"
        ),
        leaving,
        rv.len(),
        show(&s.rim)
    );
    let Some(orbit) = s.spin_orbit(&anchor.spin) else {
        out.push_str("  The paired throat: the founded elements could not be read\n");
        return out;
    };
    out.push_str(&format!(
        concat!(
            "  The paired throat (T16, T17): the parent's spin {}, torsion share {}; ",
            "{} cohorts leave; the core's spin after them {} ({})\n"
        ),
        show(&anchor.spin),
        show(&s.share),
        orbit.cohorts.len(),
        show(&orbit.spin),
        if orbit.spin <= Q::ONE {
            "closed on both faces"
        } else {
            "open: past the Kerr bound"
        }
    ));
    let share_of = |x: &Q| show(&x.checked_div(&orbit.founded).unwrap_or(Q::ZERO));
    let pressed = orbit
        .cohorts
        .iter()
        .filter(|c| c.far)
        .fold(Q::ZERO, |m, c| &m + &c.hold);
    let faces = &(&Q::int(3) + &anchor.spin) * &(&Q::int(3) + &anchor.spin);
    out.push_str(&format!(
        concat!(
            "  The pair's holds (T16): on both faces, the closed kinds, {} of the founded hold; ",
            "on the near face only, by pressure (released by the far face), {}; on both faces by ",
            "pressure, none (the faces' product (3 + a)^2 = {})\n"
        ),
        share_of(&orbit.held),
        share_of(&pressed),
        show(&faces)
    ));
    // In the core's units the closed hold is 1, so its turning per unit hold
    // is 360 a* degrees.
    let remanent = orbit
        .remanence
        .checked_div(&orbit.turning)
        .unwrap_or(Q::ZERO);
    let per_hold = &Q::int(360) * &orbit.spin;
    out.push_str(&format!(
        concat!(
            "  The core's remanence (T19): {} of its held turning, {} of {} degrees per unit ",
            "hold in the core's units\n"
        ),
        show(&remanent),
        show(&(&remanent * &per_hold)),
        show(&per_hold)
    ));
    if let Ok(elements) = xyl_cosmos::elements::founded(&s.share) {
        let planes: Vec<[Q; xyl_core::hopf::PLANES]> = xyl_cosmos::formation::kinds(&elements)
            .iter()
            .filter_map(|k| {
                let from_q =
                    xyl_cosmos::circuit::fill_shares(k.z, &s.share, &xyl_core::hopf::COMMON);
                xyl_cosmos::atoms::plane_polarities(k.z, &from_q)
            })
            .collect();
        let a: Vec<Q> = planes.iter().map(|p| p[0].clone()).collect();
        let sheared = planes
            .iter()
            .filter(|p| p.iter().any(|x| x != &p[0]))
            .count();
        let mut values = a.clone();
        values.sort();
        values.dedup();
        out.push_str(&format!(
            concat!(
                "  Axis polarity (T7): {} kinds, {} Z side, {} Q side, {} neutral; ",
                "{} values from {} to {}; kinds whose six planes part: {}\n"
            ),
            a.len(),
            a.iter().filter(|x| x.signum() < 0).count(),
            a.iter().filter(|x| x.signum() > 0).count(),
            a.iter().filter(|x| x.signum() == 0).count(),
            values.len(),
            values.first().map_or("-".to_string(), show),
            values.last().map_or("-".to_string(), show),
            sheared
        ));
    }
    for c in &orbit.cohorts {
        out.push_str(&format!(
            concat!(
                "    cohort at {} echoes ({} face): hold {}, ",
                "turning share over hold share {}, spin after {}\n"
            ),
            c.echoes,
            if c.far { "far" } else { "near" },
            show(&c.hold),
            show(&c.share),
            show(&c.spin)
        ));
    }
    out.push_str(&format!(
        concat!(
            "  The inbound echo at T: {} (the ring plus the vessel's drift, per echo); ",
            "its mean over the first echo {}; the push at the throat {} swung to {} at T\n"
        ),
        s.inbound(t),
        s.inbound_mean(s.echo()),
        xyl_cosmos::circuit::throat_push(),
        s.push_at(Dy::ONE, Dy::ONE, t)
    ));
    let swung = (0..rv.len())
        .filter(|&i| s.beyond_at(i, t).signum() > 0)
        .count();
    out.push_str(&format!(
        "  The swing at T: {swung} of {} kinds past the throat under the inbound echo\n",
        rv.len()
    ));
    out
}

/// The loops counted by outcome: closed, held back toward Z (residual
/// below 1), carried on toward Q (residual 1 or more), and without a pole
/// state.
fn tally(loops: &[Option<PoleFlux>]) -> String {
    let (mut held_back, mut opened, mut closed, mut none) = (0, 0, 0, 0);
    for l in loops {
        match l {
            None => none += 1,
            Some(f) if f.closed => closed += 1,
            Some(f) if f.residual < Q::ONE => held_back += 1,
            Some(_) => opened += 1,
        }
    }
    format!(
        concat!(
            "{} loops close, {} are held back toward Z, {} carried on toward Q, ",
            "{} have no pole state (sealed, or past the far face)"
        ),
        closed, held_back, opened, none
    )
}

/// The jetstream: each kind's channels composed over the echo, tallied at T
/// and at the echo's mean (`at_mean`).
fn jetstream(s: &Substrate, t: Dy, at_mean: &[Option<PoleFlux>]) -> String {
    let loops: Vec<Option<PoleFlux>> = (0..s.reverb.len()).map(|i| s.loop_flux(i, t)).collect();
    format!(
        concat!(
            "  The jetstream (filament, vacuum and echo composed on each kind's pole state): ",
            "at T, with the inbound echo at {}: {}; at the echo's mean: {}\n"
        ),
        s.inbound(t),
        tally(&loops),
        tally(at_mean)
    )
}

/// A kind's loop at the echo's mean: its start, its channel factors and the
/// residual they leave.
fn loop_note(f: &PoleFlux) -> String {
    let ch = |n: &str| {
        f.channels
            .iter()
            .find(|c| c.0 == n)
            .map(|c| show(&c.1))
            .unwrap_or_else(|| "-".into())
    };
    format!(
        "; loop at the mean: t {} filament x{} vacuum x{} echo x{} -> residual {}{}",
        show(&f.t0),
        ch("filament"),
        ch("vacuum"),
        ch("echo"),
        show(&f.residual),
        if f.closed { " (closed)" } else { "" }
    )
}

/// The eight firmament kinds leaving fastest: reach, hold, deficit, share
/// beyond the throat, rate, what each still holds at T (amount - outflow),
/// and its loop at the echo's mean.
fn fastest_kinds(s: &Substrate, t: Dy, at_mean: &[Option<PoleFlux>]) -> String {
    let rv = &s.reverb;
    let outflow = s.outflow(t);
    let mut order: Vec<usize> = (0..s.firmament.len()).collect();
    order.sort_by(|&a, &b| rv[b].rate.cmp(&rv[a].rate));
    let mut out = String::new();
    for &i in order.iter().take(8) {
        let k = &s.firmament[i];
        let r = &rv[i];
        let m0 = Dy::from_q(&k.amount);
        let jet = at_mean
            .get(i)
            .and_then(|l| l.as_ref())
            .map(loop_note)
            .unwrap_or_else(|| "; loop: no pole state".into());
        out.push_str(&format!(
            concat!(
                "    Z{}: reach {} hold {} deficit {} beyond {} rate {} /Gyr; ",
                "onset {} rings; holds {} of {} at T{}\n"
            ),
            k.z,
            r.reach,
            r.hold,
            r.deficit,
            r.beyond,
            per_gyr(r.rate),
            r.onset
                .map_or_else(|| "none".to_string(), |o| o.to_string()),
            m0 - outflow[i].1,
            m0,
            jet
        ));
    }
    out
}

/// The books by mass (held, shed, taken, and in flight = shed - taken,
/// beside the founded total), each galaxy's ocean share, delay and intake,
/// and the force.
fn ocean(s: &Substrate, t: Dy) -> String {
    let (held, left, taken, founded) = s.books(t);
    let mut out = format!(
        concat!(
            "  The books (by mass): the firmament holds {}, has shed {}, ",
            "the galaxies have taken {}, {} in flight; founded {}\n"
        ),
        held,
        left,
        taken,
        left - taken,
        founded
    );
    out.push_str(
        "  The ocean: each galaxy's share of the outflow, its delay, what it has taken by T\n",
    );
    let intakes = s.intakes(t);
    for (i, share, delay) in s.ocean_shares() {
        let took = intakes
            .iter()
            .find(|x| x.0 == i)
            .map(|x| x.1.iter().fold(Dy::ZERO, |a, (_, v)| a + *v))
            .unwrap_or(Dy::ZERO);
        let delay = gyr_of_rings_dy(delay);
        out.push_str(&format!(
            "    galaxy {i}: share {share}, delay {delay} Gyr, taken {took}\n"
        ));
    }
    let (q_state, q_mass) = s.rate(t);
    out.push_str(&format!(
        concat!(
            "  The force: the rate leaving the core {} a Gyr in its whole state ",
            "({} by mass); the ocean's pulse {} cluster units a Gyr; the waveform at the rim {}\n"
        ),
        per_gyr(q_state),
        per_gyr(q_mass),
        per_gyr(s.c_ocean()),
        per_gyr(s.amplitude(Dy::from_q(&s.rim), t))
    ));
    let rim = s.carry(Dy::from_q(&s.rim), t);
    out.push_str(&format!(
        concat!(
            "  The held medium (the ocean's carry): inside the rim {} core masses at T, ",
            "carrying {} of the pulse there (about the core alone {}); ",
            "outward {} per echo (the near face's open share out, the far face's held share back)\n"
        ),
        rim.hold,
        rim.share,
        xyl_core::shares::deficit_dy(Dy::ONE, Dy::from_q(&s.rim)),
        show(&rim.outward)
    ));
    out
}

/// Each galaxy in the ocean at T: its reach then and now, recession, drift,
/// swing, shift, circulation, place, and the tide across its extent.
fn galaxies(origin: &Origin, s: &Substrate, t: Dy) -> String {
    // A galaxy's extent in the ocean's rings: its rim (r_max x its scale,
    // converted to galaxy units) over the length one ring of its reach spans
    // (its anchor distance over its reach in rings).
    let shapes = xyl_cosmos::cores::shapes(origin);
    let kpc = xyl_cosmos::galaxy::units_of_kpc(&Q::ONE);
    let mut out = String::new();
    for g in s.push(t) {
        let shape = shapes.iter().find(|sh| sh.index == g.index);
        let extent = shape.and_then(|sh| {
            let rim_gu = &(&xyl_cosmos::galaxy::rim_kpc() * &sh.scale().to_q()) * &kpc;
            let per_ring = sh.anchor().0.checked_div(&g.reach0.to_q())?;
            Some(Dy::from_q(&rim_gu.checked_div(&per_ring)?))
        });
        let tide = extent
            .map(|x| per_gyr(s.tide(g.reach0, x, t)))
            .unwrap_or(Dy::ZERO);
        let rate = shape.map(|sh| per_gyr(sh.rate)).unwrap_or(Dy::ZERO);
        let cell = shape.map(|sh| sh.place_at(t, &xyl_cosmos::chain::radius()).cell());
        let carry = s.carry(g.reach0, t);
        out.push_str(&format!(
            concat!(
                "    galaxy {}: reach {} -> {} cluster units, recession {} cluster units a Gyr ",
                "(drift {}, swing +/- {} cluster units an echo), shift z {}; ",
                "circulates {} turns a Gyr, at azimuth {} polar {} deg; tide across it {}; ",
                "the medium inside it {} core masses carries {} of the pulse, turning {} deg ",
                "a traversal on the isoclinic planes, returned after {} traversals\n"
            ),
            g.index,
            g.reach0,
            g.reach,
            per_gyr(g.recession),
            per_gyr(g.drift),
            g.swing,
            g.shift,
            rate,
            cell.map(|c| c.azimuth.to_string())
                .unwrap_or_else(|| "-".into()),
            cell.map(|c| c.polar.to_string())
                .unwrap_or_else(|| "-".into()),
            tide,
            carry.hold,
            carry.share,
            carry.turning,
            carry
                .returns
                .map_or_else(|| "no".to_string(), |n| n.to_string())
        ));
    }
    out
}
