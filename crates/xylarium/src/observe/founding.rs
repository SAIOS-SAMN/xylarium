//! Law readings of the founding: `founding`, `founding-reach` and
//! `formation`.

use super::Options;
use crate::run::{show, Founded};
use xyl_core::Q;
use xyl_cosmos::circuit::Substrate;
use xyl_cosmos::cores::Shape;
use xyl_cosmos::elements::Element;
use xyl_cosmos::origin::Origin;

/// The founding's mix of `o`'s seed (each kind's share of the holding, as
/// `circuit::Substrate::found` reads it) and the parent's spin.
fn founding_mix(o: &Options) -> Result<(Vec<(u32, Q)>, Q), String> {
    let f = Founded::of(o.seed)?;
    let mix = xyl_cosmos::formation::kinds(&f.elements)
        .iter()
        .map(|k| (k.z, k.share.clone()))
        .collect();
    Ok((mix, f.origin.parent.spin.clone()))
}

/// The `founding` reading: the core's release on the founding's mix (its
/// cohorts, each with its layers' pole reaches) and the shells it leaves:
/// the kinds the core keeps and the released kinds' reaches and swings,
/// then the shells grouped where kinds share one reach; every line ends
/// with a newline.
pub(super) fn founding_cascade(o: &Options) -> Result<String, String> {
    let (mix, spin) = founding_mix(o)?;
    let orbit = xyl_cosmos::formation::spin_orbit(&mix, &spin);
    let (shells, _, take) = xyl_cosmos::formation::founding_shells(&mix, &spin);
    let kept: Vec<u32> = shells.iter().filter(|s| s.kept).map(|s| s.z).collect();
    let out_reach = || shells.iter().filter(|s| !s.kept).map(|s| s.reach.clone());
    let (near, far) = (out_reach().min(), out_reach().max());
    let rim = shells
        .iter()
        .map(|s| &s.reach * &(&Q::ONE + &s.swing))
        .max()
        .unwrap_or(Q::ZERO);
    let dash_or = |q: Option<&Q>| q.map(show).unwrap_or_else(|| "-".into());
    let mut out = format!(
        concat!(
            "The founding's release: {} kinds at the parent's spin {}; {} cohorts leave, ",
            "the core's spin after them {}; the core keeps {} kinds (Z {:?}..{:?}), ",
            "{} of the founded hold; the released reaches {} .. {} founding units; ",
            "rim {} founding units; the pools' take, booked to the core's charged holding, {}\n"
        ),
        mix.len(),
        show(&spin),
        orbit.cohorts.len(),
        show(&orbit.spin),
        kept.len(),
        kept.iter().min(),
        kept.iter().max(),
        show(&orbit.held),
        dash_or(near.as_ref()),
        dash_or(far.as_ref()),
        show(&rim),
        show(&take)
    );
    for (k, c) in orbit.cohorts.iter().enumerate() {
        let lo = c.layers.iter().map(|l| l.2).min();
        let hi = c.layers.iter().map(|l| l.2).max();
        out.push_str(&format!(
            concat!(
                "  Cohort {}: at {} echoes by the {} face, {} layers, hold {}, ",
                "pinch interval {} echoes, ln r on the axis plane {} .. {}, spin after {}\n"
            ),
            k + 1,
            c.echoes,
            if c.far { "far" } else { "near" },
            c.layers.len(),
            show(&c.hold),
            c.interval,
            lo.map_or("-".into(), |x| x.to_string()),
            hi.map_or("-".into(), |x| x.to_string()),
            show(&c.spin)
        ));
    }
    // Shells in order of reach, kinds standing at one reach grouped.
    let mut by_reach: Vec<&xyl_cosmos::formation::Shell> = shells.iter().collect();
    by_reach.sort_by(|a, b| a.reach.cmp(&b.reach).then(a.z.cmp(&b.z)));
    let mut groups: Vec<Vec<&xyl_cosmos::formation::Shell>> = Vec::new();
    for s in by_reach {
        match groups.last_mut() {
            Some(g) if g.first().is_some_and(|f| f.reach == s.reach) => g.push(s),
            _ => groups.push(vec![s]),
        }
    }
    for (k, g) in groups.iter().enumerate() {
        let amount = g.iter().fold(Q::ZERO, |a, s| &a + &s.amount);
        out.push_str(&format!(
            "  Shell {}: Z {}..{} ({} kinds), amount {}, reach {}, swing {}, {}\n",
            k + 1,
            g.iter().map(|s| s.z).min().unwrap_or(0),
            g.iter().map(|s| s.z).max().unwrap_or(0),
            g.len(),
            show(&amount),
            g.first().map(|s| show(&s.reach)).unwrap_or_default(),
            g.first().map(|s| show(&s.swing)).unwrap_or_default(),
            if g.iter().all(|s| s.kept) {
                "kept, the firmament's"
            } else {
                "released"
            }
        ));
    }
    Ok(out)
}

/// The `founding-reach` reading: for selected founded kinds and the
/// heaviest, the reach about the throat from the founding's release, the
/// open share Q / 3Z and the rest point Q* / Z, then the ratio of reaches
/// Z 115 / Z top.
pub(super) fn founding_read(o: &Options) -> Result<String, String> {
    let (mix, spin) = founding_mix(o)?;
    let present = mix.len();
    let top = mix.iter().map(|(z, _)| *z).max().unwrap_or(0);
    let (shells, neutrons, _) = xyl_cosmos::formation::founding_shells(&mix, &spin);
    let mut out = format!(
        "The founding with the table to Z {}: {present} kinds founded (heaviest Z {top})\n",
        xyl_cosmos::elements::TOP_Z
    );
    let reach = |z: u32| shells.iter().find(|s| s.z == z).map(|s| s.reach.clone());
    for s in shells.iter().filter(|s| {
        [
            1, 2, 6, 8, 26, 79, 115, 150, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200,
        ]
        .contains(&s.z)
            || s.z == top
    }) {
        let z = s.z;
        let q = xyl_cosmos::compound::open(z);
        let rest = neutrons
            .get(&z)
            .and_then(|p| xyl_core::shares::weighted(p, 64))
            .map_or_else(|| "-".to_string(), |t| show(&t.mid()));
        out.push_str(&format!(
            "  Z{z:>5}: reach {}  Q/3Z {}  Q*/Z {rest}{}\n",
            show(&s.reach),
            show(&Q::new(i64::from(q), 3 * i64::from(z)).unwrap_or(Q::ZERO)),
            if s.kept { "  (kept)" } else { "" }
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

/// The `formation` reading: the founding's matter, then each galaxy's gas,
/// its lit pieces and the star tier's cut of the first lit pieces
/// (`galaxy::disc_manifold`, the cut `disc::star_hold` the stars are made
/// from), counted in rings of the galaxy's echo; every galaxy is read as the
/// home galaxy's homologue.
pub(super) fn formation(o: &Options) -> Result<String, String> {
    let Founded {
        origin, elements, ..
    } = Founded::of(o.seed)?;
    let shapes = xyl_cosmos::cores::shapes(&origin);
    let mut out = founding_matter(&origin, &elements);
    // Units are the universe's: time in rings, reach as a deficit share
    // of its enclosure, mass as a holding share (never years, kpc or suns).
    let sub = xyl_cosmos::cores::substrate(&origin).ok_or("no founding")?;
    out.push_str(&galaxies(&shapes, &sub));
    Ok(out)
}

/// The founding's matter: each element's count per hydrogen and the mean
/// mass number by count.
fn founding_matter(origin: &Origin, elements: &[Element]) -> String {
    // The mean mass number by count is `sum(n_i A_i) / sum(n_i)`.
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
        show(&origin.parent.spin),
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

/// Each galaxy's formation lines: its share of the universe, the gas's
/// pieces and lit share, the first four lit pieces' star tier and the first
/// four pieces.
fn galaxies(shapes: &[Shape], sub: &Substrate) -> String {
    let total = shapes.iter().fold(Q::ZERO, |a, s| &a + &s.mass_ratio());
    let l = &sub.lighting;
    let lit: Vec<&(Q, Q, Option<i64>, Q)> = l.pieces.iter().filter(|p| p.2.is_some()).collect();
    let mut out = String::new();
    for sh in shapes {
        let s_g = sh.mass_ratio().checked_div(&total).unwrap_or(Q::ONE);
        out.push_str(&format!(
            concat!(
                "Galaxy {} (its share of the universe {}): push {}, held in the core {}; ",
                "{} pieces (the beads), {} lit, first lighting at {} rings, lit share {}\n"
            ),
            sh.index,
            show(&s_g),
            xyl_cosmos::circuit::throat_push(),
            show(&sub.held_share()),
            l.pieces.len(),
            lit.len(),
            l.cools_in
                .map(|n| n.to_string())
                .unwrap_or_else(|| "-".into()),
            show(&l.lit)
        ));
        // The star tier (the star's cut and its manifold's worlds) of each
        // of the first four lit pieces.
        for p in lit.iter().take(4) {
            out.push_str(&star_tier(p));
        }
        // The first four pieces, lit or not.
        for p in l.pieces.iter().take(4) {
            out.push_str(&format!(
                "  piece {} to {} galaxy units: lights after {} rings, infall {} galaxy units\n",
                show(&p.0),
                show(&p.1),
                p.2.map(|n| n.to_string()).unwrap_or_else(|| "-".into()),
                show(&p.3)
            ));
        }
    }
    out
}

/// A lit piece's star tier, read at its midpoint for the piece's star mass
/// (`galaxy::piece_star_mass`): the cut's tide, disc agitation, held share,
/// turning, heat, over-pressure, rim and compactness, and the worlds and
/// belt its manifold holds (`disc::manifold_worlds`).
fn star_tier(p: &(Q, Q, Option<i64>, Q)) -> String {
    let mid = (&p.0 + &p.1).checked_div(&Q::int(2)).unwrap_or(Q::ZERO);
    let disc = xyl_cosmos::galaxy::piece_star_mass(&mid)
        .and_then(|m| xyl_cosmos::galaxy::disc_manifold(&mid, &m));
    let Some(f) = disc else {
        return format!("  Star tier at {} galaxy units: no cut\n", show(&mid));
    };
    let (worlds, belt) = xyl_cosmos::disc::manifold_worlds(&f.star_share, &f.a_disc, &f.tide);
    format!(
        concat!(
            "  Star tier at {} galaxy units: tide {}, disc agitation {}; ",
            "the star {} of its infall ",
            "(spin {}, heat {}, over-pressure {}, rim {}, compactness {}); ",
            "its manifold {} of the reach held, {} worlds, belt {}\n"
        ),
        show(&mid),
        show(&f.tide),
        show(&f.a_disc),
        show(&f.cut.star),
        show(&f.cut.spin),
        show(&f.cut.received.isoclinic),
        show(&f.cut.p0),
        show(&f.cut.rim),
        show(&f.cut.compactness),
        show(&f.star_share),
        worlds.len(),
        show(&belt)
    )
}
