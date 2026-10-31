//! Invariants of the galaxy tier's sample stars and structure.

use super::*;
use xyl_core::certified::down;

#[test]
fn stars_circle_the_core_and_the_core_holds_its_share() {
    let bang = crate::genesis::bang();
    let stars: Vec<Star> = (0..1000).filter_map(|n| star(7, bang, n)).collect();
    // A star exists only where its ring's gas lights; its birth is at or
    // after the ring's first light and at or before the horizon.
    assert!(!stars.is_empty() && stars.len() <= 1000);
    let horizon = crate::life::horizon_gyr().to_q();
    for s in &stars {
        let first = birth_gyr(&s.r).expect("a standing star's ring lights");
        assert!(
            s.born >= down(&first, GRID_BITS),
            "born after its first light"
        );
        assert!(s.born <= horizon, "born before the horizon");
    }
    // `in_core` and `Pop::Bulge` select the same stars: a star within the
    // core's reach (one galaxy unit) is a bulge star.
    let core = stars.iter().filter(|s| s.in_core()).count();
    let bulge = stars.iter().filter(|s| s.pop == Pop::Bulge).count();
    assert_eq!(core, bulge, "core {core} bulge {bulge}");
    // Every star lies within the outermost ring's outer edge.
    let edge = structure()
        .rings
        .last()
        .map(|r| r.1.clone())
        .unwrap_or(rim_kpc());
    assert!(stars.iter().all(|s| s.orbit.radius.to_q() <= edge));
    // A star's distance from its core is its orbit radius to within 0.5 kpc,
    // and its place changes with t.
    let r = crate::chain::radius();
    let s = &stars[0];
    let (p0, p1) = (s.place(Dy::int(5), &r), s.place(Dy::sci(51, -1), &r));
    let from_core = crate::minds::from_centre_kpc(s, Dy::int(5));
    assert!(
        (from_core - s.orbit.radius).abs() < Dy::ratio(1, 2),
        "{} vs {}",
        from_core,
        s.orbit.radius
    );
    assert!(p0 != p1, "it moves");
    // A disc star stays within 2 kpc of the plane.
    let height = |s: &Star, t: Dy| {
        let (reach, dir, climb) = s.bearing(t);
        Dy::from_q(
            &(&(&dir[2] * &reach).abs() + &climb.abs())
                .checked_div(&crate::chain::kpc())
                .unwrap(),
        )
    };
    let disc = stars.iter().find(|s| s.pop == Pop::Disc).unwrap();
    assert!(height(disc, Dy::int(8)) < Dy::int(2));
    // Nucleus orbits are tilted, so some stars lie off the plane by more
    // than a tenth of their radius, all within the nucleus; if the nucleus
    // never lights, no nucleus star exists.
    let nuc: Vec<Star> = (0..64)
        .filter_map(|k| star(7, bang, NUCLEUS_BASE + k))
        .collect();
    if nuc.is_empty() {
        assert!(birth_gyr(&Q::ZERO).is_none(), "the nucleus never lights");
    } else {
        assert!(nuc
            .iter()
            .any(|s| height(s, Dy::int(3)) > s.orbit.radius / Dy::int(10)));
        assert!(
            nuc.iter()
                .all(|s| s.pop == Pop::Nucleus && s.orbit.radius.to_q() <= nucleus_kpc()),
            "within the nucleus"
        );
    }
    // The mass inside r includes the hole and grows outward.
    assert!(inside_mass(&Q::ratio(1, 1000)) >= hole_mass());
    assert!(inside_mass(&rim_kpc()) >= inside_mass(&r0()));
    assert!(inside_mass(&r0()) > inside_mass(&Q::ratio(1, 1000)));
    // The arc from a place to itself is zero; the arc is symmetric and never
    // shorter than the difference between the two reaches.
    let (a, b) = (
        stars[1].place(Dy::int(4), &r),
        stars[2].place(Dy::int(4), &r),
    );
    assert!(a.arc(&a, &r).mid().abs() < Q::ratio(1, 1_000_000));
    let (ab, ba) = (a.arc(&b, &r).mid(), b.arc(&a, &r).mid());
    assert_eq!(ab, ba);
    assert!(ab >= (&a.reach - &b.reach).abs(), "arc {ab}");
}

#[test]
fn imf_heavier_where_rings_crowd_births_after_first_stars() {
    let bang = crate::genesis::bang();
    let mean = |r: Q| {
        let n = 60;
        let s = (0..n).fold(Q::ZERO, |a, k| {
            &a + &imf(&Q::ratio(k * 2 + 1, n * 2), &r).unwrap().mid()
        });
        s.checked_div(&Q::int(n)).unwrap()
    };
    // The IMF's break is heavier where the rings crowd: the mean mass at the
    // ring of greatest holding per kpc is at least that at r0. Every star has
    // positive mass and is born no earlier than the first stars.
    let densest = structure()
        .rings
        .iter()
        .max_by(|a, b| a.3.cmp(&b.3))
        .map(|r| r.0.clone())
        .unwrap_or(r0());
    assert!(mean(densest) >= mean(r0()));
    for s in (0..300).filter_map(|n| star(7, bang, n)) {
        assert!(s.mass.signum() > 0, "{}", s.mass);
        assert!(s.born >= bang.first_stars_gyr().lo);
    }
}
