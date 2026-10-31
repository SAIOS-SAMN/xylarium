//! Tests of orbits and spins.

use super::*;

fn rocky_world() -> World {
    let elements = crate::elements::table().unwrap();
    let mut el = vec![Q::ZERO; elements.len()];
    // a two-thirds mantle carrying U, Th and K
    let mantle = Q::ratio(2, 3);
    let set = |el: &mut Vec<Q>, s: &str, share: &str| {
        let i = elements.iter().position(|e| e.sym == s).unwrap();
        el[i] = &mantle * &crate::elements::decimal(share).unwrap_or(Q::ZERO);
    };
    set(&mut el, "U", "1e-8");
    set(&mut el, "Th", "1e-7");
    set(&mut el, "K", "1e-4");
    World {
        a: Q::ONE,
        mass: Q::ONE,
        radius: Q::ONE,
        giant: false,
        host: None,
        a_host_radii: None,
        layers: [Q::ratio(1, 3), Q::ratio(2, 3), Q::ZERO, Q::ZERO],
        elements: el,
        mantle: Vec::new(),
        ices: Vec::new(),
        e_forced: None,
        swing: Q::ZERO,
        phase: Q::ZERO,
        merged_at: None,
    }
}

#[test]
fn closer_world_locks_sooner_with_shorter_year() {
    let w = rocky_world();
    let sun = Si::star_mass_kg();
    let mut rng = Stream::derive(1, &["orbits".into()]);
    let close = World {
        a: Q::ratio(3, 10),
        ..w.clone()
    };
    let far = World {
        a: Q::ONE,
        ..w.clone()
    };
    let os = orbits(&sun, &[&close, &far], &mut rng);
    assert!(
        os[0].lock_gyr < os[1].lock_gyr,
        "the closer world locks sooner"
    );
    assert!(
        os[0].e0 <= Q::ratio(9, 10)
            && os[0].cos_obliquity >= -Q::ONE
            && os[0].cos_obliquity <= Q::ONE
    );
    let now = os[1].at(&Q::int(3));
    assert!(now.day > Q::ZERO, "a world that turns has a day");
    // the year is one turn at the turning law's rate, and is longer for the
    // farther world
    let law = (&Q::int(360) * &Si::gyr_s())
        .checked_div(&crate::turning::turning_si(&sun, &os[1].a_m))
        .unwrap();
    assert!(
        (&now.year_s - &law).abs() <= &law * &Q::ratio(1, 1_000_000_000),
        "year {} against the law's {}",
        now.year_s,
        law
    );
    assert!(
        os[0].at(&Q::int(3)).year_s < now.year_s,
        "the closer world's year is shorter"
    );
}
