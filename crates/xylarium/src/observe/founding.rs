//! Law readings of the founding: `founding`, `founding-reach` and
//! `formation`.

use super::Options;
use crate::run::show;
use xyl_core::Q;
use xyl_cosmos::bang::BigBang;
use xyl_cosmos::circuit::Substrate;
use xyl_cosmos::cores::Shape;
use xyl_cosmos::elements::Element;
use xyl_cosmos::formation::{Formation, Piece};

/// The `founding` reading: the founding's equitable cascade form by form
/// with its stacks, then the founding cascade and its stacks; every line
/// ends with a newline.
pub(super) fn founding_cascade(o: &Options) -> Result<String, String> {
    let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
    let elements = xyl_cosmos::elements::founded(&bang.parent.share())?;
    let kinds = xyl_cosmos::formation::kinds(&elements);
    let mix: Vec<(u32, Q)> = kinds.iter().map(|k| (k.z, k.share.clone())).collect();
    let mut out = String::new();
    for r in xyl_cosmos::formation::equitable_cascade(&mix) {
        out.push_str(&format!(
            concat!(
                "Equitable cascade, form {}: spin K {}, throat H {}, final share {}; ",
                "held {} kinds (Z {:?}..{:?})\n"
            ),
            r.form,
            show(&r.k_spin),
            show(&r.h_final),
            show(&r.s_final),
            r.held.len(),
            r.held.iter().min(),
            r.held.iter().max()
        ));
        for (k, (zs, push, far, deep, korb)) in r.stacks.iter().enumerate() {
            out.push_str(&format!(
                concat!(
                    "  Stack {}: Z {}..{} ({} kinds), push K + s_i {}, ",
                    "reach lightest {} heaviest {}, orbital speed at the heaviest {}\n"
                ),
                k + 1,
                zs.iter().min().unwrap_or(&0),
                zs.iter().max().unwrap_or(&0),
                zs.len(),
                show(push),
                show(far),
                show(deep),
                show(korb)
            ));
        }
    }
    let (stacks, settled) = xyl_cosmos::formation::cascade(&mix);
    out.push_str(&format!(
        concat!(
            "The founding cascade (a reading): {} kinds, {} stacks; ",
            "the settled bell's round trip {}\n"
        ),
        mix.len(),
        stacks.len(),
        show(&settled)
    ));
    for (k, st) in stacks.iter().enumerate() {
        out.push_str(&format!(
            concat!(
                "  Stack {}: Z {}..{} ({} kinds), amount {}, mean hold {}, bounce {}, ",
                "released spin {}, settled {}, beat {}\n"
            ),
            k + 1,
            st.kinds.iter().min().unwrap_or(&0),
            st.kinds.iter().max().unwrap_or(&0),
            st.kinds.len(),
            show(&st.amount),
            show(&st.w_mean),
            show(&st.bounce),
            show(&st.released),
            show(&st.settled),
            show(&st.beat)
        ));
    }
    Ok(out)
}

/// The founding's kinds by reach (the table's ceiling at TOP_Z): every
/// founded kind's reach about the Wellspring from the founding cascade, its
/// open share Q / 3Z, and the ratio of reaches Z 115 / Z top (under the
/// linear law).
pub(super) fn founding_read(o: &Options) -> Result<String, String> {
    let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
    let elements = xyl_cosmos::elements::founded(&bang.parent.share())?;
    let mix: Vec<(u32, Q)> = elements
        .iter()
        .filter(|e| e.abundance.signum() > 0)
        .map(|e| (e.z, e.abundance.clone()))
        .collect();
    let present = mix.len();
    let top = mix.iter().map(|(z, _)| *z).max().unwrap_or(0);
    let (_, shells) = xyl_cosmos::formation::founding_shells(&mix);
    let mut out = format!(
        "The founding with the table to Z {}: {present} kinds founded (heaviest Z {top})\n",
        xyl_cosmos::elements::TOP_Z
    );
    let reach = |z: u32| shells.iter().find(|s| s.0 == z).map(|s| s.2.clone());
    for (z, _, r, _) in shells.iter().filter(|s| {
        [
            1, 2, 6, 8, 26, 79, 115, 150, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200,
        ]
        .contains(&s.0)
            || s.0 == top
    }) {
        let q = xyl_cosmos::compound::open(*z);
        out.push_str(&format!(
            "  Z{z:>5}: reach {}  Q/3Z {}\n",
            show(r),
            show(&Q::ratio(i64::from(q), 3 * i64::from(*z)))
        ));
    }
    if let (Some(r115), Some(rtop)) = (reach(115), reach(top)) {
        out.push_str(&format!(
            "reach Z115 / reach Z{top}: {}\n",
            show(&r115.checked_div(&rtop).unwrap_or(Q::ZERO))
        ));
    }
    Ok(out)
}

/// Formation, read (laws not yet wired into the game's stars): each
/// galaxy's cut, gas, cooling and stars, its counts in its own echo rings
/// (the ring a turning counts in: its anchor's echo), years only at the seam.
pub(super) fn formation(o: &Options) -> Result<String, String> {
    let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
    let elements = xyl_cosmos::elements::founded(&bang.parent.share())?;
    let shapes = xyl_cosmos::cores::shapes(o.seed, &bang);
    let mut out = founding_matter(&bang, &elements);
    // Units are the universe's own: time in rings, reach as a deficit share
    // of its enclosure, mass as a holding share (never years, kpc or suns).
    // The pieces are the founding's beads one tier down, heated by the pulse
    // at swing = 1 + the inbound echo's mean over the first echo, as at the
    // founding; every galaxy is read as the home galaxy's homologue.
    let sub = xyl_cosmos::cores::substrate(o.seed, &bang).ok_or("no founding")?;
    let swing = &Q::ONE + &sub.inbound_mean(sub.echo()).to_q();
    let f = xyl_cosmos::formation::of(&sub.beads, &sub.unit(), &sub.held_share(), &swing);
    out.push_str(&galaxies(&shapes, &f, &sub));
    out.push_str(&scratch_infalls(&elements));
    Ok(out)
}

/// The founding's matter: each element's count per hydrogen and the mean
/// mass number by count.
fn founding_matter(bang: &BigBang, elements: &[Element]) -> String {
    // mean mass number by count: sum(n_i A_i) / sum(n_i)
    let (count, mass) = elements
        .iter()
        .filter(|e| e.abundance.signum() > 0)
        .fold((Q::ZERO, Q::ZERO), |(c, m), e| {
            (&c + &e.abundance, &m + &(&e.abundance * &e.a))
        });
    let a_mean = mass.checked_div(&count).unwrap_or(Q::ONE);
    let hydrogen = elements
        .iter()
        .find(|e| e.z == 1)
        .map(|e| e.abundance.clone())
        .unwrap_or(Q::ONE);
    let by = |sym: &str| {
        elements
            .iter()
            .find(|e| e.sym == sym)
            .map(|e| show(&e.abundance.checked_div(&hydrogen).unwrap_or(Q::ZERO)))
            .unwrap_or_default()
    };
    format!(
        concat!(
            "The founding's matter (the bounce's spin {}; count per hydrogen, ",
            "whole nuclei per 2^224 hydrogen): He {}  C {}  N {}  O {}  Si {}  Fe {}  U {}; ",
            "mean mass number {}\n"
        ),
        show(&bang.parent.spin),
        by("He"),
        by("C"),
        by("N"),
        by("O"),
        by("Si"),
        by("Fe"),
        by("U"),
        show(&a_mean)
    )
}

/// Each galaxy's formation lines: its share of the universe, the gas and
/// its pieces, the clouds and stars, the first four forming pieces' star
/// tier and the first four pieces.
fn galaxies(shapes: &[Shape], f: &Formation, sub: &Substrate) -> String {
    let total = shapes.iter().fold(Q::ZERO, |a, s| &a + &s.mass_ratio());
    // gas = sum lambda (to - from) over the pieces
    let gas: Q = f
        .pieces
        .iter()
        .fold(Q::ZERO, |a, p| &a + &(&p.lambda * &(&p.to - &p.from)));
    let formed: Vec<&Piece> = f.pieces.iter().filter(|p| p.clouds > 0).collect();
    let clouds: i64 = formed.iter().map(|p| p.clouds).sum();
    let stars = formed.iter().fold(Q::ZERO, |a, p| &a + &p.stars);
    let in_stars = formed
        .iter()
        .fold(Q::ZERO, |a, p| &a + &(&p.star * &p.stars));
    let first = formed.iter().filter_map(|p| p.cools_in).min();
    let (small, big) = (
        formed
            .iter()
            .filter(|p| p.stars.signum() > 0)
            .map(|p| p.star.clone())
            .min(),
        formed
            .iter()
            .filter(|p| p.stars.signum() > 0)
            .map(|p| p.star.clone())
            .max(),
    );
    let dark: Q = f
        .pieces
        .iter()
        .filter(|p| p.clouds == 0)
        .fold(Q::ZERO, |a, p| &a + &(&p.lambda * &(&p.to - &p.from)));
    let dash_or = |q: Option<&Q>| q.map(show).unwrap_or_else(|| "-".into());
    let mut out = String::new();
    for sh in shapes {
        let s_g = sh.mass_ratio().checked_div(&total).unwrap_or(Q::ONE);
        out.push_str(&format!(
            concat!(
                "Galaxy {} (its share of the universe {}): push {}, held in the core {}; ",
                "gas {} in {} pieces (the beads), lit share {}\n"
            ),
            sh.index,
            show(&s_g),
            show(&f.push),
            show(&f.held),
            show(&gas),
            f.pieces.len(),
            show(&sub.lighting.lit)
        ));
        out.push_str(&format!(
            concat!(
                "  Clouds {}, first collapsing at {} rings; stars {}, holding {} of the grains; ",
                "each {} .. {} of the grains; gas that never lights: {}\n"
            ),
            clouds,
            first.map(|n| n.to_string()).unwrap_or_else(|| "-".into()),
            show(&stars),
            show(&in_stars),
            dash_or(small.as_ref()),
            dash_or(big.as_ref()),
            show(&dark)
        ));
        // the star tier (the core's cut and its disc) of each forming piece
        for pc in formed.iter().filter(|p| p.stars.signum() > 0).take(4) {
            out.push_str(&star_tier(pc));
            out.push_str(&worlds_line(&pc.worlds));
        }
        // the first pieces, lit or not
        for pc in f.pieces.iter().take(4) {
            out.push_str(&piece_line(pc));
        }
    }
    out
}

/// A forming piece's star tier: its tide, agitation, the star's share of
/// its infall, open shares, worlds and belts.
fn star_tier(pc: &Piece) -> String {
    // a world's holding over its star's (holding shares, never world units)
    let shares = |w: &Q| show(&w.checked_div(&pc.star_share).unwrap_or(Q::ZERO));
    format!(
        concat!(
            "  Star tier at reach {}: tide {}, infall agitation {}; the star {} of its infall; ",
            "open share infall {} / disc {}; {} worlds, from {} to {} of its star's holding, ",
            "belts {} of the infall\n"
        ),
        show(&pc.reach),
        show(&pc.tide),
        show(&pc.heated),
        show(&pc.star_share),
        show(&pc.infall_open),
        show(&pc.disc_open),
        pc.worlds.len(),
        pc.worlds
            .first()
            .map(|w| shares(&w.holding))
            .unwrap_or_else(|| "-".into()),
        pc.worlds
            .last()
            .map(|w| shares(&w.holding))
            .unwrap_or_else(|| "-".into()),
        show(&pc.belts)
    )
}

/// A piece of gas: its reach and span in rings, holding, agitation,
/// emission, cooling, heat (heated - cooled), clouds and lighting time.
fn piece_line(pc: &Piece) -> String {
    format!(
        concat!(
            "  piece at reach {} ({} to {} rings): holding per ring {}, agitation {}, ",
            "emission {}, cooled {}, heat {}, clouds {}, lights after {} rings\n"
        ),
        show(&pc.reach),
        show(&pc.from),
        show(&pc.to),
        show(&pc.lambda),
        show(&pc.agitation),
        show(&pc.emission),
        show(&pc.cooled),
        show(&(&pc.heated - &pc.cooled)),
        pc.clouds,
        pc.cools_in
            .map(|n| n.to_string())
            .unwrap_or_else(|| "-".into())
    )
}

/// The planetary cores on the founding's own mix: a scratch reading of the
/// law (one infall of the founding's matter, not a formed star: no star
/// forms here until convection is modelled), at the galaxy's tide inside
/// its rim, delta_gal = 2 / (R + 2) for R = 1, 3/2, 2.
fn scratch_infalls(elements: &[Element]) -> String {
    let mix: Vec<(u32, Q)> = xyl_cosmos::formation::kinds(elements)
        .iter()
        .map(|k| (k.z, k.share.clone()))
        .collect();
    let p = Q::ratio(1, 2);
    let mut out = String::new();
    for (r, tide) in [
        ("1", Q::ratio(2, 3)),
        ("3/2", Q::ratio(4, 7)),
        ("2", Q::ratio(1, 2)),
    ] {
        let c = xyl_cosmos::formation::hold_at_heat(&mix, &p, &Q::ZERO, &Q::ZERO, &Q::ONE);
        let (worlds, belts) = xyl_cosmos::disc::star_disc(&c.held, &tide, &c.rest);
        out.push_str(&format!(
            concat!(
                "Scratch infall of the founding's mix at galactic reach {} (tide {}): ",
                "the star {} of it, held down to Z {}; its disc {} kinds, {} worlds, belts {}\n"
            ),
            r,
            show(&tide),
            show(&c.held),
            c.lightest
                .map(|z| z.to_string())
                .unwrap_or_else(|| "-".into()),
            c.rest.len(),
            worlds.len(),
            show(&belts)
        ));
        out.push_str(&worlds_line(&worlds));
    }
    out
}

/// The worlds of a disc, each with its planetary core's cut: inner, middle
/// and outer (up to six shown).
fn worlds_line(worlds: &[xyl_cosmos::disc::World]) -> String {
    let n = worlds.len();
    let picks: Vec<usize> = if n <= 6 {
        (0..n).collect()
    } else {
        vec![0, 1, n / 3, n / 2, 2 * n / 3, n - 1]
    };
    picks
        .into_iter()
        .filter_map(|i| worlds.get(i).map(|w| (i, w)))
        .map(|(i, w)| {
            format!(
                concat!(
                    "    world {} at {}: holding {}, heat {}, its core keeps {} of its matter, ",
                    "held down to Z {}\n"
                ),
                i,
                show(&w.reach),
                show(&w.holding),
                show(&w.heat),
                show(&w.kept),
                w.lightest
                    .map(|z| z.to_string())
                    .unwrap_or_else(|| "-".into())
            )
        })
        .collect()
}
