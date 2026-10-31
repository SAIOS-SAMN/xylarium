//! Readings of worlds and systems: `world S W`, `system S` and the law
//! reading `worlds N`.

use super::Options;
use crate::run::show;
use xyl_core::certified::Encl;
use xyl_core::dyadic::Dy;
use xyl_core::units::{Tier, Units};
use xyl_core::Q;
use xyl_cosmos::bang::BigBang;
use xyl_cosmos::galaxy::Star;
use xyl_cosmos::life::Body;
use xyl_cosmos::manifold::{
    cap_degrees, galaxy_place, manifolds, share, star_cap_degrees, Anchor, Manifold,
};
use xyl_cosmos::systems::System;
use xyl_cosmos::worlds::{Medium, State};
use xyl_kernel::cosmic::People;
use xyl_kernel::universe::Universe;

/// The `world S W` reading: one row per Gyr of the world's life (surface
/// temperature, pressure, field, air, seas, ices and its people's housing),
/// then the world's events.
pub(super) fn world(u: &Universe<'_>, star: Option<u64>, w: Option<usize>) -> String {
    let (Some(star), Some(w)) = (star, w) else {
        return "observe: world S W (a star's sample index and its world)".into();
    };
    let Some(wit) = u.cosmic.systems.get(&star) else {
        return format!("observe: star {star} is not in play");
    };
    let born = Dy::from_q(&wit.star.born);
    let people = u
        .cosmic
        .peoples
        .iter()
        .find(|p| p.star == star && p.world == w);
    let rulers = xyl_kernel::growth::Rulers::default();
    let mut out = format!(
        "world {w} of star {star} (born {born} Gyr){}\n",
        people
            .map(|p| format!(", home of {} (woke {} Gyr)", p.name, p.dawn))
            .unwrap_or_default()
    );
    out.push_str(concat!(
        "  Gyr      surface K   bar          field        ",
        "air / seas / ices                         housing\n"
    ));
    let mut t = born + Dy::ratio(1, 2);
    while t <= u.cosmic.now + Dy::ratio(1, 100) {
        if let Some((st, ms, body)) = u.cosmic.world_at(star, w, t) {
            let housing = people
                .and_then(|p| p.traits.medium(&ms).map(|m| (p, m)))
                .map(|(p, m)| housed(u, p, m, &st, &ms, &body, &rulers))
                .unwrap_or_else(|| unhoused(u, &st, &ms));
            out.push_str(&format!(
                "  {:<8} {:<11} {:<12} {:<12} {:<41} {}\n",
                t.to_string(),
                st.t_surf.to_string(),
                st.p_bar.to_string(),
                st.field.to_string(),
                format!(
                    "{} / {}{} / {}",
                    mix(&st.air),
                    mix(&st.seas),
                    st.frozen
                        .iter()
                        .map(|(s, f)| format!(" ({} {} frozen)", s.name(), f))
                        .collect::<String>(),
                    mix(&st.ices)
                ),
                housing
            ));
        }
        t += Dy::ONE;
    }
    // the world's events from the crossing solver (xyl_core::event), from
    // its star's birth to now
    let sets = xyl_cosmos::worlds::settings(&u.cosmic.gal, &[star], &wit.system, &wit.star);
    if let Some(set) = sets.get(w) {
        let age = u.cosmic.now - born;
        for (what, at) in xyl_cosmos::worlds::events(set, age, &u.cosmic.ctx) {
            out.push_str(&format!(
                "  Event: {what} at {} Gyr after its star's birth ({} Gyr; bracket {})\n",
                Dy::from_q(&at.mid()),
                Dy::from_q(&at.mid()) + born,
                Dy::from_q(&(&at.hi - &at.lo))
            ));
        }
    }
    out
}

/// The names of a mix's present species, comma-separated.
fn mix(m: &xyl_cosmos::worlds::Mix) -> String {
    m.iter()
        .filter(|(_, v)| v.signum() > 0)
        .map(|(s, _)| s.name())
        .collect::<Vec<_>>()
        .join(",")
}

/// The housing column where the people's solvent is present: the housing
/// `civ::housing::houses` gives at the people's current biology and energy
/// mastery, and the medium it lives in.
fn housed(
    u: &Universe<'_>,
    p: &People,
    m: &Medium,
    st: &State,
    ms: &[Medium],
    body: &Body,
    rulers: &xyl_kernel::growth::Rulers,
) -> String {
    let k = u
        .cosmic
        .peoples
        .iter()
        .position(|q| std::ptr::eq(q, p))
        .and_then(|i| u.civ.knowledge.of.get(i));
    let (bio, energy) = k
        .map(|k| {
            (
                k.mastery(xyl_kernel::knowledge::Domain::Biology),
                k.mastery(xyl_kernel::knowledge::Domain::Energy),
            )
        })
        .unwrap_or((Dy::ZERO, Dy::ZERO));
    format!(
        "{} in {} {:?}",
        xyl_kernel::civ::housing::houses(
            &p.traits,
            st,
            ms,
            body,
            &u.cosmic.ctx,
            rulers,
            bio,
            energy
        ),
        m.solvent.name(),
        m.at
    )
}

/// The housing column where the people's solvent is absent: the media
/// present and each sea's phase at the surface temperature and pressure.
fn unhoused(u: &Universe<'_>, st: &State, ms: &[Medium]) -> String {
    let seas = st
        .seas
        .iter()
        .map(|(s, amt)| {
            format!(
                "{} {:?} (amount {}, boils at {} K)",
                s.name(),
                u.cosmic.ctx.phase(s, st.t_surf, st.p_bar),
                amt,
                u.cosmic.ctx.boiling(s, st.p_bar)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "none of its solvent; media: [{}]; seas: {seas}",
        ms.iter()
            .map(|m| format!("{} {:?} {:?}", m.solvent.name(), m.at, m.region))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// An enclosure's midpoint, printed.
fn mid(e: &Encl) -> String {
    show(&e.mid())
}

/// An enclosure's width `hi - lo`, printed.
fn width(e: &Encl) -> String {
    show(&(&e.hi - &e.lo))
}

/// Metres in one unit of `tier`.
fn tier_m(units: &Units, tier: Tier) -> Q {
    &units.r_g_m * &tier.in_rg()
}

/// A turn count's fractional part in degrees, printed.
fn deg(turns: &Encl) -> String {
    show(&(&turns.mid().frac() * &Q::int(360)))
}

/// " [name]" of the people living on world `w` of star `n`, or empty.
fn who(u: &Universe<'_>, n: u64, w: usize) -> String {
    u.cosmic
        .peoples
        .iter()
        .find(|p| p.star == n && p.world == w)
        .map(|p| format!(" [{}]", p.name))
        .unwrap_or_default()
}

/// The settling's outputs on world `w`: its swing at the settled age and,
/// where it merges into its neighbour, the merge age.
fn settled(sys: &System, w: usize) -> String {
    sys.worlds
        .get(w)
        .map(|x| {
            format!(
                ", swing {}{}",
                show(&x.swing),
                x.merged_at
                    .as_ref()
                    .map(|m| format!(", merges at {} Gyr", show(m)))
                    .unwrap_or_default()
            )
        })
        .unwrap_or_default()
}

/// The `system S` reading: the star, each planet on the star's sphere and
/// each moon on its giant's, then the relations between them.
pub(super) fn system(u: &Universe<'_>, n: Option<u64>) -> String {
    let Some(n) = n else {
        return "observe: system S (a star's sample index)".into();
    };
    let Some(wit) = u.cosmic.systems.get(&n) else {
        return format!("observe: star {n} is not in play");
    };
    let (star, sys) = (&wit.star, &wit.system);
    let units = xyl_cosmos::chain::units();
    let all = manifolds(u.cosmic.gal.seed, &[n], sys, star, &units);
    let t = u.cosmic.now;
    let age = (t - Dy::from_q(&star.born)).to_q();
    let mut out = format!(
        "Star {n}: {} suns, born {} Gyr, age {} Gyr\n",
        show(&sys.spec.mass),
        show(&star.born),
        show(&age)
    );
    out.push_str(&format!(
        "  its cap on the core's sphere: {} deg of volume (width {})\n",
        mid(&star_cap_degrees(star, &sys.spec.mass)),
        width(&star_cap_degrees(star, &sys.spec.mass))
    ));
    for m in &all {
        out.push_str(&match m.anchor {
            Anchor::Star => planet_line(u, n, sys, m, &all, &age, &units),
            Anchor::World(h) => moon_line(u, n, m, h, &all, &age, &units),
        });
    }
    out.push_str(&relations(u, n, star, &all, &age, &units));
    out
}

/// One planet of the `system` reading: its orbit, anomaly, distance from
/// the star and cap on the star's sphere.
fn planet_line(
    u: &Universe<'_>,
    n: u64,
    sys: &System,
    m: &Manifold,
    all: &[Manifold],
    age: &Q,
    units: &Units,
) -> String {
    let st = m.stand(age, all);
    // distance from the anchor: the bearing's radius on its pivots, not a
    // Euclidean vector length
    let d = Encl::exact(m.bearing(age, all).r);
    let hill = m.reach();
    let a_au = (&m.a * &tier_m(units, Tier::System))
        .checked_div(&xyl_cosmos::package::Si::au())
        .unwrap_or(Q::ZERO);
    format!(
        concat!(
            "  Planet world {}{}: a {} system units ({} AU), e {}, mean anomaly {} deg, ",
            "distance {} (width {}), cap on the star's sphere {} deg of volume{}\n"
        ),
        m.world,
        who(u, n, m.world),
        show(&m.a),
        show(&a_au),
        mid(&st.e),
        deg(&st.mean),
        mid(&d),
        width(&d),
        mid(&cap_degrees(xyl_core::measure::BODY_N, &hill, &d)),
        settled(sys, m.world)
    )
}

/// One moon of the `system` reading, of world `h`: its orbit, chain step,
/// anomaly, distance, cap on its giant's sphere and its belonging shares.
fn moon_line(
    u: &Universe<'_>,
    n: u64,
    m: &Manifold,
    h: usize,
    all: &[Manifold],
    age: &Q,
    units: &Units,
) -> String {
    let st = m.stand(age, all);
    let d = Encl::exact(m.bearing(age, all).r);
    let hill = m.reach();
    let a_km = (&m.a * &tier_m(units, Tier::Planet))
        .checked_div(&Q::int(1000))
        .unwrap_or(Q::ZERO);
    let host = all.iter().find(|x| x.world == h);
    // belonging to the giant: share(d, giant's reach dilated from the system
    // tier to the planet tier); belonging to the star is 1 minus it
    let host_hill_planet = host.map(|x| {
        x.reach().mul(&Encl::exact(xyl_cosmos::manifold::dilation(
            Tier::System,
            Tier::Planet,
        )))
    });
    let under = host_hill_planet.map(|hh| share(xyl_core::measure::BODY_N, &d, &hh));
    format!(
        concat!(
            "    Moon world {}{} of world {}: a {} planet units ({} km), chain step {}, ",
            "rate 1/2^k of the innermost, mean anomaly {} deg, distance {} (width {}), ",
            "cap on its giant's sphere {} deg of volume, ",
            "belongs to its giant {} / to the star {}\n"
        ),
        m.world,
        who(u, n, m.world),
        h,
        show(&m.a),
        show(&a_km),
        m.chain
            .map(|c| c.1.to_string())
            .unwrap_or_else(|| "-".into()),
        deg(&st.mean),
        mid(&d),
        width(&d),
        mid(&cap_degrees(xyl_core::measure::BODY_N, &hill, &d)),
        under.as_ref().map(mid).unwrap_or_default(),
        under
            .map(|s| show(&(&Q::ONE - &s.mid())))
            .unwrap_or_default()
    )
}

/// The relations of the `system` reading, each read in the lowest tier
/// holding both its ends: the first two planets' gap (system tier), the
/// first planet's separations as transit reads them, the first moon to its
/// giant (planet tier), and the first planet to another star's people
/// (galaxy tier).
fn relations(
    u: &Universe<'_>,
    n: u64,
    star: &Star,
    all: &[Manifold],
    age: &Q,
    units: &Units,
) -> String {
    let mut out = String::from("Relations (in the lowest tier holding both ends)\n");
    let planets: Vec<usize> = all
        .iter()
        .filter(|m| m.anchor == Anchor::Star)
        .map(|m| m.world)
        .collect();
    if let [a, b, ..] = planets[..] {
        if let Some(g) = xyl_cosmos::manifold::gap(all, a, b, age) {
            out.push_str(&format!(
                "  world {a} to world {b}: system tier, {} units, the turn swept and the climb\n",
                show(&g)
            ));
        }
    }
    if let Some(&a) = planets.first() {
        out.push_str(&gaps_from(u, n, a, &planets, all, age, units));
    }
    if let Some(moon) = all.iter().find(|m| m.anchor != Anchor::Star) {
        let d = Encl::exact(moon.bearing(age, all).r);
        out.push_str(&format!(
            "  moon {} to its giant: planet tier, {} units (width {})\n",
            moon.world,
            mid(&d),
            width(&d)
        ));
    }
    let t = u.cosmic.now;
    let r = xyl_cosmos::chain::radius();
    let other = u.cosmic.peoples.iter().find(|p| p.star != n);
    if let (Some(&a), Some(o)) = (planets.first(), other) {
        if let Some(ow) = u.cosmic.systems.get(&o.star) {
            let oall = manifolds(u.cosmic.gal.seed, &[o.star], &ow.system, &ow.star, units);
            if let (Some(pa), Some(pb)) = (
                galaxy_place(star, all, a, t, &r),
                galaxy_place(&ow.star, &oall, o.world, t, &r),
            ) {
                let arc = pa.arc(&pb, &r);
                out.push_str(&format!(
                    "  world {a} to {} on star {}: galaxy tier, {} units (width {})\n",
                    o.name,
                    o.star,
                    mid(&arc),
                    width(&arc)
                ));
            }
        }
    }
    out
}

/// The separations from planet `a` to each other planet as transit reads
/// them (AU now, 1 year on and 5 years on), beside the difference of their
/// orbit radii.
fn gaps_from(
    u: &Universe<'_>,
    n: u64,
    a: usize,
    planets: &[usize],
    all: &[Manifold],
    age: &Q,
    units: &Units,
) -> String {
    let au_m = xyl_cosmos::package::Si::au();
    let mut out = format!(
        concat!(
            "Gaps from world {}, as voyages and perception read them ",
            "(AU now, then 1 and 5 years on; the orbits' radii apart in brackets)\n"
        ),
        a
    );
    let a_of = |w: usize| {
        all.iter()
            .find(|m| m.world == w)
            .map(|m| &m.a * &tier_m(units, Tier::System))
            .unwrap_or(Q::ZERO)
    };
    let au_d = Dy::from_q(&au_m);
    for &b in planets.iter().skip(1) {
        let place = |world| xyl_kernel::transit::Place { star: n, world };
        if let Some(g) = xyl_kernel::transit::separation_m(&u.cosmic, place(a), place(b)) {
            let flat = (&a_of(a) - &a_of(b))
                .abs()
                .checked_div(&au_m)
                .unwrap_or(Q::ZERO);
            let later = |yr: i64| {
                let at = age + &Q::ratio(yr, 1_000_000_000);
                xyl_kernel::transit::orbit_gap_m(all, a, b, &at)
                    .map(|x| (x / au_d).to_string())
                    .unwrap_or_default()
            };
            out.push_str(&format!(
                "  to world {b}: {}, {}, {} ({})\n",
                g / au_d,
                later(1),
                later(5),
                show(&flat)
            ));
        }
    }
    out
}

/// Why sample stars stand without planets, counted by cause.
#[derive(Default)]
struct Bare {
    /// The star's ring never lit: no star.
    unlit: usize,
    /// No disc frame at the star.
    unframed: usize,
    /// The star's gathering fills its manifold (star share >= 1).
    filled: usize,
    /// No region on the manifold holds against the core's twist.
    twisted: usize,
    /// Worlds formed and all settled away.
    settled_out: usize,
}

impl Bare {
    /// Counts the cause for a lit star `st` left without planets.
    fn count(&mut self, st: &Star) {
        match xyl_cosmos::galaxy::disc_frame(&st.r, &st.mass) {
            None => self.unframed += 1,
            Some(f) if f.star_share >= Q::ONE => self.filled += 1,
            Some(f)
                if xyl_cosmos::disc::manifold_worlds(&f.star_share, &f.a_disc, &f.tide)
                    .0
                    .is_empty() =>
            {
                self.twisted += 1
            }
            Some(_) => self.settled_out += 1,
        }
    }
}

/// The chain's frame at star `st`: its piece, the infall's extent, the
/// star's share, the tide (the core-star tie) and the regions.
fn chain_frame(st: &Star) -> String {
    xyl_cosmos::galaxy::disc_frame(&st.r, &st.mass)
        .map(|f| {
            let (regions, belt) =
                xyl_cosmos::disc::manifold_worlds(&f.star_share, &f.a_disc, &f.tide);
            let au = |kpc: &Q| {
                Dy::from_q(
                    &(kpc * &xyl_cosmos::package::Si::kpc())
                        .checked_div(&xyl_cosmos::package::Si::au())
                        .unwrap_or(Q::ZERO),
                )
            };
            format!(
                concat!(
                    "piece {} at {} kpc: infall {} AU holding {}, the star's gathering {} AU, ",
                    "its manifold's reach {} AU; cut at spin {} heat {} over-pressure {}: ",
                    "star's share {}, tide {}, agitation {}; ",
                    "{} worlds on the manifold (first at {} of the reach), belt {}"
                ),
                f.piece,
                Dy::from_q(&st.r),
                au(&f.infall_kpc),
                Dy::from_q(&f.cut.cloud),
                au(&f.extent_kpc),
                au(&f.reach_kpc),
                Dy::from_q(&f.cut.spin),
                Dy::from_q(&f.cut.heat),
                Dy::from_q(&f.cut.p0),
                Dy::from_q(&f.star_share),
                Dy::from_q(&f.tide),
                Dy::from_q(&f.a_disc),
                regions.len(),
                regions
                    .first()
                    .map(|r| Dy::from_q(&r.0).to_string())
                    .unwrap_or_else(|| "-".into()),
                Dy::from_q(&belt)
            )
        })
        .unwrap_or_else(|| format!("no frame at {} kpc", Dy::from_q(&st.r)))
}

/// One star's row of the `worlds` reading: its chain frame, then its
/// planets (orbit in AU, mass in world units, giants marked).
fn star_rows(k: u64, st: &Star, sys: &System, planets: &[(Q, Q, bool)]) -> String {
    let mut rows = format!("    chain: {}\n", chain_frame(st));
    rows.push_str(&format!(
        "  star {k} ({} suns): {} planets: {}\n",
        Dy::from_q(&sys.spec.mass),
        planets.len(),
        planets
            .iter()
            .map(|(a, m, g)| format!(
                "{} AU {} E{}",
                Dy::from_q(a),
                Dy::from_q(m),
                if *g { " giant" } else { "" }
            ))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    rows
}

/// The worlds as the settling leaves them: the first `n` sample stars of the
/// home galaxy built straight from the seed - each system's planets, their
/// orbits (AU), masses (world units) and the ratio between neighbours' orbits -
/// read before any genesis.
pub(super) fn worlds_read(o: &Options, n: usize) -> Result<String, String> {
    let bang = BigBang::of(o.seed).ok_or("no Big Bang")?;
    let elements = xyl_cosmos::elements::founded(&bang.parent.share())?;
    let chem = xyl_cosmos::chem::Chemistry::of(&bang).ok_or("no chemistry")?;
    let (mut counts, mut ratios, mut rows) = (Vec::new(), Vec::new(), String::new());
    let (mut belts, mut moons, mut giants, mut comets) = (0usize, 0usize, 0usize, 0usize);
    let mut bare = Bare::default();
    for k in 0..u64::try_from(n).unwrap_or(0) {
        let Some(st) = xyl_cosmos::galaxy::star(o.seed, &bang, k) else {
            bare.unlit += 1;
            continue;
        };
        let Some(sys) = xyl_cosmos::systems::system(o.seed, &bang, &[k], &st, &chem, &elements)
        else {
            continue;
        };
        let mut planets: Vec<(Q, Q, bool)> = sys
            .worlds
            .iter()
            .filter(|w| w.host.is_none())
            .map(|w| (w.a.clone(), w.mass.clone(), w.giant))
            .collect();
        planets.sort_by(|a, b| a.0.cmp(&b.0));
        if planets.is_empty() {
            bare.count(&st);
        }
        counts.push(planets.len());
        belts += sys.belts.len();
        moons += sys.worlds.iter().filter(|w| w.host.is_some()).count();
        giants += sys.worlds.iter().filter(|w| w.giant).count();
        comets += usize::from(sys.comets.is_some());
        for w in planets.windows(2) {
            ratios.push(Dy::from_q(&w[1].0.checked_div(&w[0].0).unwrap_or(Q::ONE)));
        }
        if k < 12 {
            rows.push_str(&star_rows(k, &st, &sys, &planets));
        }
    }
    counts.sort_unstable();
    ratios.sort();
    let at = |v: &[usize], f: usize| v.get(f * v.len() / 10).copied().unwrap_or(0);
    let rt = |f: usize| {
        ratios
            .get(f * ratios.len() / 10)
            .map(|x| x.to_string())
            .unwrap_or_default()
    };
    let total: usize = counts.iter().sum();
    let mut out = format!(
        concat!(
            "The worlds, the settling over {} stars: planets per star ",
            "min {} p10 {} p50 {} p90 {} max {} (total {}, {} without); ",
            "giants {}, moons {}, belts {}, comet clouds {}\n"
        ),
        counts.len(),
        counts.first().copied().unwrap_or(0),
        at(&counts, 1),
        at(&counts, 5),
        at(&counts, 9),
        counts.last().copied().unwrap_or(0),
        total,
        counts.iter().filter(|c| **c == 0).count(),
        giants,
        moons,
        belts,
        comets
    );
    out.push_str(&format!(
        concat!(
            "  without worlds: {} never lit, {} unframed, {} gathering fills the manifold, ",
            "{} nothing holds against the twist, {} settled away\n"
        ),
        bare.unlit, bare.unframed, bare.filled, bare.twisted, bare.settled_out
    ));
    out.push_str(&format!(
        "  neighbours' orbit ratio: min {} p10 {} p50 {} p90 {}\n{rows}",
        ratios.first().map(|x| x.to_string()).unwrap_or_default(),
        rt(1),
        rt(5),
        rt(9)
    ));
    Ok(out)
}
