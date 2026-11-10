//! Tests of orbits and spins.

use super::*;

fn rocky_world() -> World {
    let elements = crate::elements::table().unwrap();
    let mut el = vec![Q::ZERO; elements.len()];
    // A mantle of two thirds of the mass carries U, Th and K.
    let mantle = xyl_core::ratio!(2, 3);
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
        layers: [
            xyl_core::ratio!(1, 3),
            xyl_core::ratio!(2, 3),
            Q::ZERO,
            Q::ZERO,
        ],
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
    // The turning law counts per ring, which the founding sets.
    let _ = crate::genesis::origin();
    let w = rocky_world();
    let sun = Si::star_mass_kg();
    let mut rng = Stream::derive(1, &["orbits".into()]);
    let close = World {
        a: xyl_core::ratio!(3, 10),
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
        os[0].e0 <= xyl_core::ratio!(9, 10)
            && os[0].cos_obliquity >= -Q::ONE
            && os[0].cos_obliquity <= Q::ONE
    );
    // The orbit at an age is read on its path (`worlds::Path`).
    let at = |o: &Orbit| crate::worlds::Path::of(o).heartbeat(Dy::int(3));
    let now = at(&os[1]);
    assert!(now.day_s > Dy::ZERO, "a world that turns has a day");
    // The year is one turn at the turning law's rate and is longer for the
    // farther world.
    let law = Dy::from_q(
        &(&Q::int(360) * &Si::seconds_per_ring())
            .checked_div(&crate::turning::turning_si(&sun, &os[1].a_m))
            .unwrap(),
    );
    assert!(
        (now.year_s - law).abs() <= law * Dy::sci(1, -9),
        "year {} against the law's {}",
        now.year_s.to_q(),
        law.to_q()
    );
    assert!(
        at(&os[0]).year_s < now.year_s,
        "the closer world's year is shorter"
    );
}
