//! Readings of the galaxy: `galaxy`, `chemistry` and the law reading
//! `tiers S`.

use super::Options;
use crate::run::show;
use xyl_core::dyadic::Dy;
use xyl_core::Q;
use xyl_cosmos::chain::{kpc, radius, units};
use xyl_cosmos::chem::Chemistry;
use xyl_cosmos::origin::Origin;
use xyl_cosmos::package::Si;
use xyl_kernel::universe::Universe;

/// The `galaxy` reading: the units from the white hole, the bells, the
/// stars' fuelling at the core's reach, and each people's star on its
/// galactic manifold.
pub(super) fn galaxy(u: &Universe<'_>) -> String {
    let r = radius();
    let mut out = String::from("Units from the white hole (G = c = M_core = 1)\n");
    out.push_str(&format!(
        "  r_g = {} m; the galaxy tier's unit 60^5 r_g; 1 kpc = {} of it\n",
        show(&units().r_g_m),
        show(&kpc())
    ));
    out.push_str(&format!(
        "  the universe's radius = {} galaxy units\n",
        show(&r)
    ));
    out.push_str(&format!("Bells: {}\n", u.field.bells()));
    let t = xyl_cosmos::circuit::rings_of_gyr_dy(u.cosmic.now);
    // The end of the first and of the last step of the founding's first lit
    // star's sequence at reach 0, the core's reach, counted in rings and
    // printed in Gyr.
    let delta = xyl_cosmos::galaxy::first_star_compactness();
    let seq = xyl_cosmos::galaxy::first_star_sequence();
    let ends = xyl_cosmos::stars::ends(&seq, delta, Dy::ZERO);
    let (main, last) = (
        xyl_cosmos::circuit::gyr_of_rings(&ends.first().cloned().unwrap_or(Q::ZERO)),
        xyl_cosmos::circuit::gyr_of_rings(&ends.last().cloned().unwrap_or(Q::ZERO)),
    );
    out.push_str(&format!(
        concat!(
            "Stars fuelled by the pulse: the first lit star's compactness {} ",
            "of the ring per bell {} ",
            "at the core's reach; pole point {}, {} steps, throat after {} echoes; ",
            "main sequence {} Gyr, last step's end {} Gyr there; ",
            "remnant keeps {} of its hold; ",
            "the radiation's swing within a bell now {} of its mean\n"
        ),
        delta,
        xyl_cosmos::stars::ring_per_bell(Dy::ZERO),
        show(&seq.pole_point),
        seq.steps.len(),
        seq.throat_echoes
            .map_or_else(|| "no".to_string(), |n| n.to_string()),
        show(&main),
        show(&last),
        show(&xyl_cosmos::stars::remnant_share(&seq)),
        xyl_cosmos::stars::flicker(t)
    ));
    out.push_str(concat!(
        "Stars on their manifolds now (galaxy units; their way from their own core ",
        "- reach and climb - its height over its plane, phase; ",
        "their reach about the universal core, life and light)\n"
    ));
    for (i, p) in u.cosmic.peoples.iter().enumerate() {
        let Some(w) = u.cosmic.systems.get(&p.star) else {
            continue;
        };
        out.push_str(&star_on_manifold(u, i, &p.name, w));
    }
    out
}

/// One people's star of the `galaxy` reading.
fn star_on_manifold(
    u: &Universe<'_>,
    i: usize,
    name: &str,
    w: &xyl_kernel::cosmic::Witnessed,
) -> String {
    // The star counts time in rings; ages and the life print in Gyr.
    let t = xyl_cosmos::circuit::rings_of_gyr_dy(u.cosmic.now);
    let s = &w.star;
    // The star's bearing from its galactic core, in galaxy-tier units:
    // `way = reach + |climb|` and `height = dir_z reach + climb`.
    let (reach, dir, climb) = s.bearing(t);
    let way = &reach + &climb.abs();
    let height = &(&dir[2] * &reach) + &climb;
    let frac = s.orbit.turns_at(t).to_q().frac();
    let reach = s.reach_rings();
    let age = t - Dy::from_q(&s.born);
    let seq = s.sequence();
    let radius_kpc = xyl_cosmos::galaxy::kpc_of_rings_dy(s.orbit.radius);
    format!(
        concat!(
            "  star {:>4} of core {} ({} {}): from its core {}, height {}, phase {} deg, ",
            "radius {} kpc; reach {} rings, life {} Gyr, transduction {:?} at age {} Gyr, ",
            "light {} core masses per ring\n"
        ),
        s.n & ((1u64 << xyl_cosmos::galaxy::CORE_SHIFT) - 1),
        s.core,
        i,
        name,
        show(&way),
        show(&height),
        show(&(&frac * &Q::int(360))),
        radius_kpc,
        reach,
        show(&xyl_cosmos::circuit::gyr_of_rings(
            &xyl_cosmos::stars::life(&seq, s.compactness(), reach)
        )),
        xyl_cosmos::field::transduction(&seq, s.compactness(), reach, age),
        xyl_cosmos::circuit::gyr_of_rings_dy(age),
        show(&xyl_cosmos::stars::light(
            &s.mass,
            s.compactness(),
            reach,
            Dy::from_q(&s.born),
            age
        ))
    )
}

/// A decimal literal as a rational, zero when it does not parse.
fn rational(s: &str) -> Q {
    xyl_cosmos::elements::decimal(s).unwrap_or(Q::ZERO)
}

/// A reach given in kpc for display, in galaxy rings (the chemistry's unit
/// of reach), converted through `galaxy::rings_of_kpc`.
fn rings(kpc: &str) -> Q {
    xyl_cosmos::galaxy::rings_of_kpc(&rational(kpc))
}

/// An optional rational, printed, or "-".
fn show_or_dash(v: Option<Q>) -> String {
    v.map(|x| Dy::from_q(&x).to_string())
        .unwrap_or_else(|| "-".into())
}

/// The `chemistry` reading: elements relative to the Sun's birth gas across
/// the disc at three epochs, each ring's gas and star formation today, and
/// each galactic core's weight dials.
pub(super) fn chemistry(u: &Universe<'_>) -> String {
    let chem = u.cosmic.gal.chem;
    let mut out = String::from("relative to the Sun's birth gas (1 = as the Sun was made)\n");
    for (label, t) in [("today", "13.8"), ("at 9.2 Gyr", "9.2"), ("at 3 Gyr", "3")] {
        out.push_str(&format!("  {label}:\n"));
        for r in ["2", "4", "8", "13"] {
            // The chemistry counts time in rings; the epoch converts here.
            let t = xyl_cosmos::circuit::rings_of_gyr(&rational(t));
            let at = |s: &str| show_or_dash(chem.relative(s, &rings(r), &t, &Q::ONE));
            out.push_str(&format!(
                "    {r:>2} kpc  O {}  Fe {}  N {}  Eu {}  metals {}\n",
                at("O"),
                at("Fe"),
                at("N"),
                at("Eu"),
                show_or_dash(chem.metals(&rings(r), &t))
            ));
        }
    }
    // Each ring's closed forms at 13.8 Gyr: gas, star formation and metals.
    out.push_str(concat!(
        "The disc today (closed forms per ring: gas and star formation ",
        "in suns per pc^2 and per pc^2 Gyr)\n"
    ));
    let model = &chem.model;
    let today = xyl_cosmos::circuit::rings_of_gyr(&rational("13.8"));
    let dt = &today - &model.params.first_stars;
    for r in ["2", "4", "8", "13"] {
        let k = xyl_cosmos::chem::ring_of(&rings(r));
        if let Some(ring) = model.rings.get(k) {
            out.push_str(&format!(
                "  {r:>2} kpc  gas {}  star formation {}  metals {}\n",
                show_or_dash(Some(ring.gas.at(&dt).mid())),
                // Star formation is per ring; read per Gyr for display.
                show_or_dash(Some(xyl_cosmos::circuit::rings_of_gyr(
                    &ring.sfr.at(&dt).mid()
                ))),
                show_or_dash(chem.metals(&rings(r), &today))
            ));
        }
    }
    // Each galaxy's weight dials at 8 kpc today: the home galaxy's history
    // scaled by each core's recipe factors (no galaxy other than the home
    // has a separately modelled history).
    out.push_str(
        "Weight dials by galactic core, at 8 kpc today (relative to the Sun's birth gas)\n",
    );
    for c in xyl_cosmos::cores::founding(
        u.cosmic.gal.seed,
        u.cosmic.gal.origin,
        u.cosmic.gal.elements,
    ) {
        let line: Vec<String> = ["C", "N", "O", "Si", "Mg", "Fe", "Ni", "Eu"]
            .iter()
            .map(|sym| {
                let rel = chem
                    .relative(sym, &rings("8"), &today, &Q::ONE)
                    .unwrap_or(Q::ZERO);
                format!(
                    "{sym} {}",
                    show_or_dash(Some(&rel * &c.weights.factor(sym)))
                )
            })
            .collect();
        out.push_str(&format!(
            "  core {} ({}): {}\n",
            c.shape.index,
            c.weights.label(),
            line.join("  ")
        ));
    }
    out
}

/// The `tiers S` reading, each tier counted in the rings of its anchor's echo
/// (each galaxy an enclosed S^360): for star `n`, its turn about its
/// galaxy's core and each of its worlds' turns about the star, by the
/// ring-count law. A period is the unit-free count `1 + R / H` of the
/// anchor's echo rings (R / H is reach over holding in any ring unit); it is
/// converted to time only where the ring's length is known: the universe's
/// echo (its great circle, from its curvature radius) and a galaxy's echo
/// (twice its rim, at the speed of light). A star's echo needs the star's
/// radius from formation, which is not yet modelled.
///
/// Returns `Err` when the origin, the star, the chemistry or the founded
/// elements cannot be derived.
pub(super) fn tiers(o: &Options, n: u64) -> Result<String, String> {
    let origin = Origin::of(o.seed).ok_or("no origin")?;
    let star = xyl_cosmos::galaxy::star(o.seed, &origin, n).ok_or("no such star")?;
    let (c, kpc_m, yr_s, star_mass_kg) = (Si::c(), Si::kpc(), Si::yr_s(), Si::star_mass_kg());
    // `r_g = G M / c^2` (m); years = metres / (c yr_s).
    let rg = |kg: &Q| (&Si::g() * kg).checked_div(&(&c * &c)).unwrap_or(Q::ZERO);
    let years = |m: &Q| m.checked_div(&(&c * &yr_s)).unwrap_or(Q::ZERO);
    let pi = xyl_core::certified::pi(64).mid();
    let mut out = String::new();
    // The universe's echo ring is its great circle, 2 pi R_u (m).
    let r_u = &xyl_cosmos::galaxy::radius_kpc() * &kpc_m;
    let ring_u = &(&Q::int(2) * &pi) * &r_u;
    out.push_str(&format!(
        "The universe's echo: radius {} m, one ring {} yr (its own, the founding's rim over pi)\n",
        show(&r_u),
        show(&years(&ring_u))
    ));
    // The galaxy's echo ring is twice the rim, out to the far side and back
    // (m).
    let scale = star.scale.to_q();
    let rim_m = &(&xyl_cosmos::galaxy::rim_kpc() * &scale) * &kpc_m;
    let ring_g = &Q::int(2) * &rim_m;
    out.push_str(&format!(
        "Galaxy {}'s echo: rim {} kpc (the founding's, scaled by its core), one ring {} yr\n",
        star.core,
        show(&(&xyl_cosmos::galaxy::rim_kpc() * &scale)),
        show(&years(&ring_g))
    ));
    // The star about its galaxy's core: reach R and holding H (the r_g of the
    // enclosed mass), both in galaxy echo rings; the count is 1 + R / H.
    let radius_kpc = xyl_cosmos::galaxy::kpc_of_rings(&star.orbit.radius.to_q());
    let r_m = &radius_kpc * &kpc_m;
    // The mass inside is in core masses; the display reads it in kg.
    let inside_suns = xyl_cosmos::galaxy::suns_of_core_masses(
        &(&xyl_cosmos::galaxy::inside_mass(&star.r) * &xyl_cosmos::cores::ratio_of_scale(&scale)),
    );
    let inside = &inside_suns * &star_mass_kg;
    let (reach, hold) = (
        r_m.checked_div(&ring_g).unwrap_or(Q::ZERO),
        rg(&inside).checked_div(&ring_g).unwrap_or(Q::ZERO),
    );
    let count = &Q::ONE + &reach.checked_div(&hold).unwrap_or(Q::ZERO);
    out.push_str(&format!(
        concat!(
            "Star {} about its core: reach {} kpc, deficit {}, ",
            "its turn {} rings of its galaxy's echo = {} yr\n"
        ),
        n,
        Dy::from_q(&radius_kpc),
        show(&hold.checked_div(&(&reach + &hold)).unwrap_or(Q::ZERO)),
        show(&count),
        show(&years(&(&count * &ring_g)))
    ));
    // Each world about the star: the count is 1 + a / r_g(star). No time is
    // given, since the length of the star's echo needs the star's radius.
    let chem = Chemistry::of(&origin).ok_or("no chemistry")?;
    let elements = xyl_cosmos::elements::founded(&origin.parent.share())?;
    if let Some(sys) = xyl_cosmos::systems::system(o.seed, &origin, &[n], &star, &chem, &elements) {
        let all = xyl_cosmos::manifold::manifolds(o.seed, &[n], &sys, &star, &units());
        let units = units();
        let star_kg = &sys.spec.mass * &star_mass_kg;
        for m in all
            .iter()
            .filter(|m| m.anchor == xyl_cosmos::manifold::Anchor::Star)
        {
            let a_m = &(&m.a * &xyl_core::units::Tier::System.in_rg()) * &units.r_g_m;
            let count = &Q::ONE + &a_m.checked_div(&rg(&star_kg)).unwrap_or(Q::ZERO);
            out.push_str(&format!(
                concat!(
                    "  World {}: a {} AU, its turn {} rings of its star's echo ",
                    "(the ring's length not yet modelled: it needs the star's radius)\n"
                ),
                m.world,
                show(&a_m.checked_div(&Si::au()).unwrap_or(Q::ZERO)),
                show(&count)
            ));
        }
    }
    Ok(out)
}

/// The relative reach of the Earth's star in its galaxy, about 8 kpc of a
/// 15 kpc disc: the probe's default input, used by no law.
const REFERENCE_REACH: (i64, i64) = (8, 15);

/// The `star-types` reading at the universe's age: an open census of the
/// classes the stars' formulas give (`galaxy::Star::class`), then one probe
/// of the home galaxy at relative reach `share` of its rim.
///
/// The census reads the first `n` sample stars of every galaxy and counts
/// them by class: a known type's name where its condition holds, with the
/// compactness decade, so classes no observation has named are listed by
/// their quantities. The probe lists the main-sequence stars whose reach
/// over the rim lies within 0.05 of `share` and prints, beside them, the
/// Earth's star's age, life and compactness as reference values for
/// comparison only; no law reads them.
pub(super) fn star_types(u: &Universe<'_>, n: usize, share: Q) -> String {
    let t = xyl_cosmos::circuit::rings_of_gyr_dy(u.cosmic.now);
    let (seed, origin) = (u.cosmic.gal.seed, u.cosmic.gal.origin);
    let mut cores: Vec<u32> = vec![0];
    cores.extend(
        xyl_cosmos::cores::shapes(seed, origin)
            .iter()
            .map(|s| s.index),
    );
    cores.sort_unstable();
    cores.dedup();
    let mut census: std::collections::BTreeMap<(String, Option<i64>), usize> = Default::default();
    let mut read = 0;
    for &core in &cores {
        for k in xyl_cosmos::galaxy::indices_of(origin, core).take(n) {
            let Some(st) = xyl_cosmos::galaxy::star(seed, origin, k) else {
                continue;
            };
            let class = st.class(t);
            let name = class
                .name()
                .map_or_else(|| format!("{:?}", class.step), str::to_string);
            *census
                .entry((name, class.compactness_decade()))
                .or_default() += 1;
            read += 1;
        }
    }
    let mut out = format!(
        "Star classes at {} Gyr: {read} sample stars of {} galaxies\n",
        show(&u.cosmic.now.to_q()),
        cores.len()
    );
    for ((name, decade), count) in &census {
        let band = decade.map_or_else(|| "holds nothing".to_string(), |d| format!("1e{d}"));
        out.push_str(&format!(
            "  {count:>6}  {name}, compactness of order {band}\n"
        ));
    }
    out.push_str(&reference_probe(u, n, &share, t));
    out
}

/// The probe of the `star-types` reading: the home galaxy's main-sequence
/// stars near relative reach `share`, with the Earth's star's values for
/// comparison.
fn reference_probe(u: &Universe<'_>, n: usize, share: &Q, t: Dy) -> String {
    let (seed, origin) = (u.cosmic.gal.seed, u.cosmic.gal.origin);
    let rim = xyl_cosmos::circuit::anchor().rim.clone();
    let mut out = format!(
        concat!(
            "Probe: the home galaxy's main-sequence stars at {} of its rim (within 0.05); ",
            "every other galaxy is a scaled copy of the home galaxy (`cores::Shape::scale`), ",
            "so its stars repeat these classes\n"
        ),
        show(share)
    );
    let mut found = 0;
    for k in xyl_cosmos::galaxy::indices_of(origin, 0).take(n) {
        let Some(st) = xyl_cosmos::galaxy::star(seed, origin, k) else {
            continue;
        };
        let at = st.r.checked_div(&rim).unwrap_or(Q::ZERO);
        let class = st.class(t);
        if (&at - share).abs() > Q::ratio(1, 20)
            || class.step != xyl_cosmos::field::Transduction::Step(0)
        {
            continue;
        }
        let age = (t - Dy::from_q(&st.born)).max(Dy::ZERO);
        out.push_str(&format!(
            concat!(
                "  star {}: reach {} of the rim, mass share {} of its cloud's star, ",
                "compactness {}, age {} Gyr, life {} Gyr\n"
            ),
            k,
            show(&at),
            class.mass_share,
            class.compactness,
            show(&xyl_cosmos::circuit::gyr_of_rings_dy(age).to_q()),
            show(&xyl_cosmos::circuit::gyr_of_rings(&class.life)),
        ));
        found += 1;
    }
    if found == 0 {
        out.push_str("  no main-sequence star at that reach among the sample read\n");
    }
    out.push_str(concat!(
        "  reference, the Earth's star (comparison only, read by no law): ",
        "reach about 0.53 of its galaxy's disc, compactness 4.2e-6, age 4.6 Gyr, ",
        "life about 10 Gyr\n"
    ));
    out
}

/// The probe's relative reach from the reading's argument (a share of the
/// rim), or the reference default.
pub(super) fn probe_share(arg: Option<&String>) -> Q {
    arg.and_then(|a| xyl_cosmos::elements::decimal(a))
        .unwrap_or_else(|| Q::ratio(REFERENCE_REACH.0, REFERENCE_REACH.1))
}
