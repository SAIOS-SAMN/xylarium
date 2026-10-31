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
    let horizon = crate::circuit::rings_of_gyr(&crate::life::horizon_gyr().to_q());
    for s in &stars {
        let first = birth(&s.r).expect("a standing star's ring lights");
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
        .unwrap_or(crate::circuit::anchor().rim);
    assert!(stars.iter().all(|s| s.orbit.radius.to_q() <= edge));
    // A star's distance from its core is its orbit radius plus its climb,
    // never beyond the swing's amplitude, and its place changes with t.
    let r = crate::chain::radius();
    let s = &stars[0];
    let (p0, p1) = (s.place(Dy::int(5), &r), s.place(Dy::sci(51, -1), &r));
    let (reach, _, climb) = s.bearing(Dy::int(5));
    let swing = s
        .vertical_at(Dy::int(5) - Dy::from_q(&s.born))
        .map(|v| v.amp * s.scale)
        .unwrap_or(Dy::ZERO);
    assert_eq!(Dy::from_q(&reach), s.orbit.radius);
    assert!(
        Dy::from_q(&climb.abs()) <= swing + Dy::sci(1, -9),
        "{climb}"
    );
    assert!(p0 != p1, "it moves");
    // A disc star stays within its swing's amplitude of the plane.
    let height = |s: &Star, t: Dy| {
        let (reach, dir, climb) = s.bearing(t);
        Dy::from_q(&(&(&dir[2] * &reach).abs() + &climb.abs()))
    };
    let disc = stars.iter().find(|s| s.pop == Pop::Disc).unwrap();
    let amp = disc
        .vertical_at((Dy::int(8) - Dy::from_q(&disc.born)).max(Dy::ZERO))
        .unwrap()
        .amp;
    assert!(height(disc, Dy::int(8)) <= amp * disc.scale + Dy::sci(1, -9));
    // Nucleus orbits are tilted, so some stars lie off the plane by more
    // than a tenth of their radius, all within the nucleus; if the nucleus
    // never lights, no nucleus star exists.
    let nuc: Vec<Star> = (0..64)
        .filter_map(|k| star(7, bang, NUCLEUS_BASE + k))
        .collect();
    if nuc.is_empty() {
        assert!(birth(&Q::ZERO).is_none(), "the nucleus never lights");
    } else {
        assert!(nuc
            .iter()
            .any(|s| height(s, Dy::int(3)) > s.orbit.radius / Dy::int(10)));
        assert!(
            nuc.iter()
                .all(|s| s.pop == Pop::Nucleus && s.orbit.radius.to_q() <= nucleus_reach()),
            "within the nucleus"
        );
    }
    // The mass inside r includes the universal core and grows outward.
    assert!(inside_mass(&Q::ratio(1, 1000)) >= Q::ONE);
    assert!(inside_mass(&crate::circuit::anchor().rim) >= inside_mass(&r0()));
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
        assert!(s.born >= bang.first_stars().lo);
    }
}

#[test]
fn masses_close_on_the_founding_holding() {
    let _ = crate::genesis::bang();
    let st = structure();
    // The rings' states sum to the holding; inside the rim the mass is the
    // universal core plus the whole holding, 2 core masses, and it never
    // falls outward.
    let sum = st.rings.iter().fold(Q::ZERO, |a, k| &a + &k.2);
    assert_eq!(sum, st.holding);
    let rim = st.rings.last().map(|k| k.1.clone()).unwrap();
    assert_eq!(inside_mass(&rim), Q::int(2));
    let mut last = inside_mass(&Q::ZERO);
    assert_eq!(last, Q::ONE);
    for k in &st.rings {
        let m = inside_mass(&k.1);
        assert!(m >= last);
        // A ring's mass is its share of the holding, at most what lies
        // inside its outer edge less the core.
        assert!(ring_mass(&k.0) <= &m - &Q::ONE);
        last = m;
    }
    // The stellar mass is the lit share of one core mass; the nucleus holds
    // part of the galaxy.
    let lit = stellar_mass();
    assert!(lit > Q::ZERO && lit <= Q::ONE, "{lit}");
    assert!(nucleus_mass() >= Q::ZERO && nucleus_mass() <= Q::ONE);
    assert_eq!(core_reach(), Q::ONE);
}

#[test]
fn vertical_swing_stays_in_shares_and_layer() {
    let bang = crate::genesis::bang();
    let stars: Vec<Star> = (0..400)
        .filter_map(|n| star(7, bang, n))
        .filter(|s| s.pop == Pop::Disc)
        .collect();
    assert!(!stars.is_empty());
    for s in &stars {
        let r = Dy::from_q(&s.r);
        // The layer's half-thickness is a deficit share of the reach.
        let h = layer(r);
        assert!(h > Dy::ZERO && h < r, "h {h} at r {r}");
        // The plane's pull is the turning rate held by the crowding share,
        // below twice the turning rate.
        let nu = vertical_frequency(r);
        let omega = s.rate_at_dy(s.orbit.radius);
        assert!(nu > Dy::ZERO);
        if s.scale == Dy::ONE {
            assert!(nu < omega * Dy::int(2) + Dy::sci(1, -30));
        }
        // The speed grows with age while the share saturates; the exposure is
        // a share; the amplitude is the speed over the frequency.
        let (young, old) = (s.vertical_at(Dy::int(1)), s.vertical_at(Dy::int(1000)));
        let (young, old) = (young.unwrap(), old.unwrap());
        assert!(young.vz <= old.vz && young.vz >= Dy::ZERO);
        for v in [young, old] {
            assert!(v.exposure >= Dy::ZERO && v.exposure <= Dy::ONE);
            assert!(v.amp >= Dy::ZERO && v.period > Dy::ZERO);
            assert_eq!(v.exposure == Dy::ONE, v.amp <= h);
        }
        // No swing at birth.
        assert_eq!(s.vertical_at(Dy::ZERO).unwrap().vz, Dy::ZERO);
    }
}
