//! Tests of fidelity, ladders, climbs and seasons.

use super::*;
use crate::laws::Scales;

fn medium(s: Solvent, t: Dy, energy: Dy, richness: Dy) -> Medium {
    Medium {
        solvent: s,
        at: Where::Surface,
        t,
        energy,
        amount: Dy::ONE,
        dark: false,
        radiolysis: Dy::ZERO,
        region: Region::Biotic,
        richness,
        pace: Dy::ONE,
        band: Dy::ONE,
        field_ev: Dy::ZERO,
    }
}

#[test]
fn plasma_copies_only_where_field_exceeds_kt() {
    let t = Dy::sci(1, 7);
    let kt = worlds::kb_ev() * t;
    // a field energy of 20 kT per carrier gives eps0 below 1/1000
    assert!(field_fidelity(kt * Dy::int(20), t) < Dy::ratio(1, 1000));
    // a field energy far below kT gives eps0 above 1/2 (no information)
    assert!(field_fidelity(kt / Dy::int(100), t) > Dy::ratio(1, 2));
    // eps0 increases as the field energy falls relative to kT
    assert!(field_fidelity(kt * Dy::int(2), t) < field_fidelity(kt, t));
}

#[test]
fn fidelity_orders_water_methane_and_plasma() {
    let ctx = Ctx::new(Scales::default());
    let e = fidelity(Solvent::Volatile(worlds::Species::H2O), Dy::int(300), &ctx);
    assert!(e > Dy::ZERO && e < Dy::ONE, "eps0 {e}");
    // eps0 increases with temperature
    assert!(fidelity(Solvent::Volatile(worlds::Species::H2O), Dy::int(350), &ctx) > e);
    assert_eq!(fidelity(Solvent::Plasma, Dy::sci(1, 4), &ctx), Dy::ONE);
    // methane's weaker cohesion gives a larger eps0 than water's, so its
    // ladder needs more rungs
    let ch4 = fidelity(Solvent::Volatile(worlds::Species::CH4), Dy::int(100), &ctx);
    assert!(
        ch4 > e && ch4 < Dy::ratio(1, 2),
        "methane {ch4} vs water {e}"
    );
    let world_radius_m = Dy::from_q(&crate::package::Si::world_radius_m());
    let area = Dy::int(4) * Dy::PI * world_radius_m * world_radius_m;
    let rungs = |s: worlds::Species, t: Dy| {
        ladder(
            &medium(Solvent::Volatile(s), t, Dy::ratio(1, 2), Dy::ratio(3, 4)),
            area,
            &ctx,
        )
        .map(|l| l.steps.len())
        .unwrap_or(0)
    };
    assert!(rungs(worlds::Species::CH4, Dy::int(100)) > rungs(worlds::Species::H2O, Dy::int(280)));
}

#[test]
fn temperate_water_ladder_completes() {
    let ctx = Ctx::new(Scales::default());
    let m = medium(
        Solvent::Volatile(worlds::Species::H2O),
        Dy::int(288),
        Dy::ratio(1, 2),
        Dy::ratio(3, 4),
    );
    let world_radius_m = Dy::from_q(&crate::package::Si::world_radius_m());
    let area = Dy::int(4) * Dy::PI * world_radius_m * world_radius_m;
    let lad = ladder(&m, area, &ctx).expect("water copies");
    assert!(
        lad.complete,
        "water reaches a mind's blueprint: {} rungs",
        lad.steps.len()
    );
    assert!(!lad.steps.is_empty());
    let total: Dy = lad.steps.iter().map(|s| s.mean_gyr).sum();
    assert!(total > Dy::ZERO, "mean climb {total} Gyr");
    // bits increase and body mass does not decrease up the ladder
    for w in lad.steps.windows(2) {
        assert!(w[1].bits > w[0].bits && w[1].body_kg >= w[0].body_kg);
    }
    // a melt's ladder is never complete
    let melt = medium(Solvent::Melt, Dy::int(2000), Dy::ONE, Dy::ratio(1, 2));
    assert!(ladder(&melt, area, &ctx).is_none_or(|l| !l.complete));
}

#[test]
fn climb_is_deterministic_and_impacts_only_delay() {
    let means = [Dy::ratio(1, 2), Dy::ONE, Dy::ONE];
    let run = |mu: Dy| climb(&means, Dy::ZERO, Dy::int(30), None, mu, None);
    let calm = run(Dy::ZERO);
    assert_eq!(calm, run(Dy::ZERO));
    assert_eq!(calm.times.len(), 3);
    assert_eq!(calm.times[2], Dy::ratio(5, 2));
    // impacts delay every rung after the first and advance none
    let hit = run(Dy::int(2));
    assert!(hit.times.len() <= 3 && hit.times.first() == calm.times.first());
    for (a, b) in hit.times.iter().zip(calm.times.iter()).skip(1) {
        assert!(a > b);
    }
    assert!(hit.setbacks > 0);
    // sterilised before the first rung: no rungs
    let dead = climb(
        &means,
        Dy::ZERO,
        Dy::int(30),
        Some(Dy::sci(1, -9)),
        Dy::ZERO,
        None,
    );
    assert!(dead.times.is_empty());
}

#[test]
fn growing_share_and_transfer_bounds() {
    assert_eq!(growing_share(0), Dy::ONE);
    assert!(
        growing_share(23) < Dy::ONE && growing_share(23) > Dy::ratio(1, 2),
        "23: {}",
        growing_share(23)
    );
    assert!(
        growing_share(90) < growing_share(23),
        "a world on its side has polar nights"
    );
    // transfer to a small, far world takes longer than to a large, near one
    let near = Body {
        radius: Dy::int(2),
        giant: false,
        locked: false,
        tilt: 20,
        a_au: Dy::ONE,
        star_lum: Dy::ONE,
        star_mass: Dy::ONE,
        merged_at: None,
    };
    let far = Body {
        radius: Dy::ONE,
        a_au: Dy::int(5),
        ..near
    };
    assert!(transfer_gyr(&far) > transfer_gyr(&near));
}
