//! The substrate law reading: `core T`.

use super::Options;
use crate::run::show;
use xyl_core::dyadic::Dy;
use xyl_core::Q;
use xyl_cosmos::bang::BigBang;
use xyl_cosmos::circuit::{PoleFlux, Substrate};

/// The `core T` reading, the substrate at T Gyr after the bounce: the core's
/// firmament kind by kind (its reach, the hold inside it, its deficit, the
/// share of its band the reverberation carries past the throat, its rate of
/// leaving and what it still holds), the jetstream, the lighting, the
/// outflow, the ocean's share and delay for every galaxy and what each has
/// taken, the books, and Z, Q and R. Every value is a closed form at T; no
/// echo is stepped and no parcel counted.
pub(super) fn core_read(o: &Options, t_gyr: Q) -> Result<String, String> {
    let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
    let elements = xyl_cosmos::elements::founded(&bang.parent.share())?;
    let clock = std::time::Instant::now();
    let s = xyl_cosmos::cores::substrate(o.seed, &bang).ok_or("no founding")?;
    let founded_in = clock.elapsed();
    let t = Dy::from_q(&t_gyr);
    let mut out = format!(
        concat!(
            "The substrate at {} Gyr (seed {}): founded in {:.1?}; the bell {} Gyr, ",
            "an echo {} Gyr; {} firmament kinds, {} beads ({} hold a sun), rim {} rings\n"
        ),
        t,
        o.seed,
        founded_in,
        xyl_cosmos::circuit::bell_gyr(),
        xyl_cosmos::circuit::echo_gyr(),
        s.firmament.len(),
        s.beads.len(),
        s.galaxies().len() - 1,
        show(&s.rim)
    );
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
    out.push_str(&galaxies(o, &bang, &s, t));
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

/// The core's state at T: the kinds reaching past the throat, the
/// admission (spin, torsion share, least share, margin), the inbound echo
/// and the push it swings, and the kinds past the throat under it.
fn core_state(s: &Substrate, t: Dy) -> String {
    let rv = &s.reverb;
    let anchor = xyl_cosmos::circuit::anchor();
    let leaving = rv.iter().filter(|r| r.beyond.signum() > 0).count();
    let mut out = format!(
        concat!(
            "  The core: {} of {} firmament kinds reach past the throat over the bell's swing; ",
            "the furthest held kind at {} rings of the rim's {}\n"
        ),
        leaving,
        rv.len(),
        show(&s.far_reach()),
        show(&s.rim)
    );
    out.push_str(&format!(
        concat!(
            "  The admission: the parent's spin {} (drawn, then floored at the least ",
            "that holds the firmament whole), torsion share {}; ",
            "the least share {} at the inbound echo's peak {}; margin {}\n"
        ),
        show(&anchor.spin),
        show(&s.share),
        show(&s.least_share()),
        s.inbound_peak(),
        show(&anchor.margin)
    ));
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
                "holds {} of {} at T{}\n"
            ),
            k.z,
            r.reach,
            r.hold,
            r.deficit,
            r.beyond,
            r.rate,
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
        out.push_str(&format!(
            "    galaxy {i}: share {share}, delay {delay} Gyr, taken {took}\n"
        ));
    }
    let (q_state, q_mass) = s.rate(t);
    out.push_str(&format!(
        concat!(
            "  The force: the rate leaving the core {} a Gyr in its whole state ",
            "({} by mass); the ocean's pulse {} rings a Gyr; the waveform at the rim {}\n"
        ),
        q_state,
        q_mass,
        s.c_ocean(),
        s.amplitude(Dy::from_q(&s.rim), t)
    ));
    out
}

/// Each galaxy in the ocean at T: its reach then and now, recession, drift,
/// swing, shift, circulation, place, and the tide across its extent.
fn galaxies(o: &Options, bang: &BigBang, s: &Substrate, t: Dy) -> String {
    // A galaxy's extent in the ocean's rings: its rim (r_max x its scale,
    // converted to galaxy units) over the length one ring of its reach spans
    // (its anchor distance over its reach in rings).
    let shapes = xyl_cosmos::cores::shapes(o.seed, bang);
    let kpc = xyl_cosmos::chain::kpc();
    let mut out = String::new();
    for g in s.push(t) {
        let shape = shapes.iter().find(|sh| sh.index == g.index);
        let extent = shape.and_then(|sh| {
            let rim_gu = &(&xyl_cosmos::galaxy::rim_kpc() * &sh.scale().to_q()) * &kpc;
            let per_ring = sh.anchor().0.checked_div(&g.reach0.to_q())?;
            Some(Dy::from_q(&rim_gu.checked_div(&per_ring)?))
        });
        let tide = extent.map(|x| s.tide(g.reach0, x, t)).unwrap_or(Dy::ZERO);
        let rate = shape.map(|sh| sh.rate).unwrap_or(Dy::ZERO);
        let cell = shape.map(|sh| sh.place_at(t, &xyl_cosmos::chain::radius()).cell());
        out.push_str(&format!(
            concat!(
                "    galaxy {}: reach {} -> {} rings, recession {} rings a Gyr ",
                "(drift {}, swing +/- {} rings an echo), shift z {}; ",
                "circulates {} turns a Gyr, at azimuth {} polar {} deg; tide across it {}\n"
            ),
            g.index,
            g.reach0,
            g.reach,
            g.recession,
            g.drift,
            g.swing,
            g.shift,
            rate,
            cell.map(|c| c.azimuth.to_string())
                .unwrap_or_else(|| "-".into()),
            cell.map(|c| c.polar.to_string())
                .unwrap_or_else(|| "-".into()),
            tide
        ));
    }
    out
}
