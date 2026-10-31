use super::*;
use crate::propulsion::Spec;
use xyl_core::units::Tier;
use xyl_cosmos::chain::units;
use xyl_cosmos::manifold::Manifold;
use xyl_cosmos::package::Si;
use xyl_cosmos::systems::World;

fn ley(from: u64, to: u64) -> Ley {
    Ley {
        from,
        to,
        builder: 0,
        tag: String::new(),
        progenitor: false,
        built: Dy::ZERO,
        cost_j: Dy::int(1),
        handling: Handling::Tolled,
    }
}

#[test]
fn realms_join_constellations_and_stewards_set_handling() {
    let c = |stars: Vec<u64>| Constellation {
        stars,
        core: 0,
        reach_kpc: Dy::ONE,
    };
    let mut net = Network {
        constellations: vec![c(vec![1, 2]), c(vec![3]), c(vec![5]), c(vec![7])],
        lines: vec![ley(1, 3), ley(3, 5)],
        ..Network::default()
    };
    net.of = [(1, 0), (2, 0), (3, 1), (5, 2), (7, 3)]
        .into_iter()
        .collect();
    // The lines form one realm of three constellations, keyed by the least
    // constellation key; the unconnected constellation forms none.
    assert_eq!(net.realms(), vec![(1, vec![1, 3, 5])]);
    assert_eq!(net.realm_of(5), Some(1));
    assert_eq!(net.realm_of(7), None);
    assert_eq!(net.note_realms(), vec![1]);
    assert!(net.note_realms().is_empty(), "a realm is noted once");
    // The steward closes one line of the realm; a line outside it is untouched.
    assert_eq!(net.handle(1, 5, 3, Handling::Closed), 1);
    assert_eq!(net.lines[1].handling, Handling::Closed);
    assert_eq!(net.handle(9, 1, 3, Handling::Guarded), 0);
    assert_eq!(net.lines[0].handling, Handling::Tolled);
    // The toll law reads a line's books.
    let q = quote(
        &net.lane(&net.lines[0]),
        None,
        Dy::int(1),
        Dy::int(10),
        Dy::int(100),
        Handling::Tolled,
    );
    assert!(q.toll <= q.ceiling && q.floor == Dy::int(1));
}
use crate::propulsion::{energy_per_kg, exhaust_speed, Drive, MASS_RATIO};
use xyl_core::Q;
use xyl_cosmos::laws::Scales;
use xyl_cosmos::manifold::Anchor;
use xyl_cosmos::worlds::{Solvent, Species};

fn spec(d: Drive) -> Spec {
    let el = xyl_cosmos::elements::table().unwrap();
    let e = energy_per_kg(d, Solvent::Volatile(Species::H2O), &Scales::default(), &el);
    Spec {
        drive: d,
        energy_per_kg: e,
        v_e: exhaust_speed(e),
    }
}

fn world(mass: i64, radius: i64, host: Option<usize>, a_host: Option<i64>) -> World {
    World {
        a: Q::ONE,
        mass: Q::int(mass),
        radius: Q::int(radius),
        giant: mass > 10,
        host,
        a_host_radii: a_host.map(Q::int),
        layers: [Q::ZERO, Q::ONE, Q::ZERO, Q::ZERO],
        elements: vec![],
        mantle: vec![],
        ices: vec![],
        e_forced: None,
        swing: Q::ZERO,
        phase: Q::ZERO,
        merged_at: None,
    }
}

#[test]
fn orbit_gaps_and_voyages_follow_drives_and_wells() {
    // A unit world's escape speed is the turning law at the surface,
    // c delta with delta = M / (R + M) in the core's units.
    let unit_world = world(1, 1, None, None);
    let esc = escape_speed(&unit_world);
    let c = crate::propulsion::c_m_s();
    let (m, r) = (
        Dy::from_q(&(&Si::g() * &Si::world_mass_kg())) / (c * c),
        Dy::from_q(&Si::world_radius_m()),
    );
    assert!(
        (esc / (c * m / (r + m)) - Dy::ONE).abs() < Dy::sci(1, -9),
        "escape {esc}"
    );
    // Orbits: the current position of each, read as turns and reaches. Two
    // planets on circles of 1 and 5 units are taken in line and then
    // opposite; a moon is measured from its giant along the moon's orbit.
    let ring = |world: usize, anchor: Anchor, tier: Tier, a: i64, m0: Q| Manifold {
        world,
        anchor,
        tier,
        a: Q::int(a),
        e0: Q::ZERO,
        e_forced: Q::ZERO,
        tau_e: Q::ZERO,
        rate: Q::int(360),
        m0,
        frame: xyl_core::rotor::Rotor::identity(),
        twist_deg: Q::ZERO,
        mass_ratio: Q::ZERO,
        chain: None,
    };
    let unit = Dy::from_q(&(&units().r_g_m * &Tier::System.in_rg()));
    let half = Q::new(1, 2).unwrap();
    let gap = |all: &[Manifold], a, b, age: &Q| orbit_gap_m(all, a, b, age).unwrap() / unit;
    let close = |x: Dy, want: i64| (x - Dy::int(want)).abs() < Dy::ratio(1, 1000);
    let inline = vec![
        ring(0, Anchor::Star, Tier::System, 1, Q::ZERO),
        ring(1, Anchor::Star, Tier::System, 5, Q::ZERO),
    ];
    assert!(close(gap(&inline, 0, 1, &Q::ZERO), 4));
    // Opposite, the way round is half a turn swept at the inner reach (pi)
    // plus the climb (4), not the chord through the star.
    let pi_plus_4 = Dy::from_q(&xyl_core::certified::pi(80).mid()) + Dy::int(4);
    let near = |x: Dy, want: Dy| (x - want).abs() < Dy::ratio(1, 1000);
    let opposite = vec![
        ring(0, Anchor::Star, Tier::System, 1, Q::ZERO),
        ring(1, Anchor::Star, Tier::System, 5, half.clone()),
    ];
    assert!(near(gap(&opposite, 0, 1, &Q::ZERO), pi_plus_4));
    // The gap changes as the orbits turn: with the outer world held still,
    // half a Gyr later the inner world has turned half a turn to the far
    // side.
    let mut paced = inline.clone();
    paced[1].rate = Q::ZERO;
    assert!(near(gap(&paced, 0, 1, &half), pi_plus_4));
    let moon = vec![
        ring(0, Anchor::Star, Tier::System, 5, Q::ZERO),
        ring(1, Anchor::World(0), Tier::Planet, 60, Q::ZERO),
    ];
    let k = Dy::from_q(&xyl_cosmos::manifold::dilation(Tier::Planet, Tier::System));
    assert!((gap(&moon, 0, 1, &Q::ZERO) - Dy::int(60) * k).abs() < Dy::ratio(1, 1_000_000));
    // Drive and well are compared in one law, c delta: the speed change of a
    // chemical drive at mass ratio 10 is below the unit world's escape
    // speed, so it cannot climb out.
    let chem = spec(Drive::Chemical);
    let r = Dy::int(MASS_RATIO);
    assert!(voyage(&chem, r, esc, Dy::int(4) * au_m()).is_none());
    // From a Moon-sized world the same drive climbs out, slowly.
    let moon_world = World {
        mass: Q::new(123, 10_000).unwrap(),
        radius: Q::new(273, 1000).unwrap(),
        ..world(1, 1, None, None)
    };
    let slow = voyage(&chem, r, escape_speed(&moon_world), Dy::int(4) * au_m())
        .expect("off a Moon it climbs out");
    assert!(slow.years > Dy::ZERO, "chemical in-system {}", slow.years);
    // A fusion drive climbs out of a unit world, and takes longer over the
    // longer span.
    let fusion = spec(Drive::Fusion);
    let near = voyage(&fusion, r, esc, Dy::int(4) * au_m()).expect("fusion climbs out");
    let pc = kpc_m() / Dy::int(1000);
    let far = voyage(&fusion, r, esc, pc).unwrap();
    assert!(far.years > near.years, "{} vs {}", far.years, near.years);
    // It is also faster than the chemical drive over the same span.
    assert!(near.years < slow.years);
    // A collapsed body (a solar mass within about ten kilometres) has an
    // escape speed above what fusion reaches.
    let collapsed = World {
        radius: Q::new(1, 637).unwrap(),
        ..world(333_000, 1, None, None)
    };
    assert!(voyage(&fusion, r, escape_speed(&collapsed), pc).is_none());
    // The fuel energy burned per kg carried is (R - 1) x the fuel's energy
    // per kg.
    assert_eq!(near.energy_per_kg, Dy::int(9) * fusion.energy_per_kg);
    assert!(
        reach_kpc(&spec(Drive::Fusion), r, esc, Dy::int(20))
            > reach_kpc(&chem, r, esc, Dy::int(20))
    );
}

#[test]
fn the_toll_sits_half_way_and_a_throat_tithes_its_span() {
    let lane = Lane {
        cost_j: Dy::int(1000),
        collected_j: Dy::int(200),
        last_year: 4,
    };
    assert_eq!(floor(&lane), Dy::int(200));
    // The way round is 5 hops and the lane 1, so the lane saves 4 hops at 100
    // each.
    let q = quote(
        &lane,
        Some(Dy::int(5)),
        Dy::int(1),
        Dy::int(100),
        Dy::int(10_000),
        Handling::Tolled,
    );
    assert_eq!(q.ceiling, Dy::int(400));
    assert_eq!(q.toll, Dy::int(300));
    // An open lane asks only its floor, and never more than the ceiling.
    assert_eq!(
        quote(
            &lane,
            Some(Dy::int(5)),
            Dy::int(1),
            Dy::int(100),
            Dy::int(10_000),
            Handling::Open
        )
        .toll,
        Dy::int(200)
    );
    let dear = Lane {
        cost_j: Dy::int(100_000),
        collected_j: Dy::ZERO,
        last_year: 1,
    };
    assert_eq!(
        quote(
            &dear,
            Some(Dy::int(5)),
            Dy::int(1),
            Dy::int(100),
            Dy::int(10_000),
            Handling::Tolled
        )
        .toll,
        Dy::int(400)
    );
    // With no way round, the ceiling is the cargo's value plus one hop.
    assert_eq!(
        quote(
            &dear,
            None,
            Dy::int(1),
            Dy::int(100),
            Dy::int(10_000),
            Handling::Tolled
        )
        .ceiling,
        Dy::int(10_100)
    );
    assert_eq!(tithe(Dy::int(7), Dy::int(30), Dy::int(3)), Dy::int(70));
    assert_eq!(tithe(Dy::int(7), Dy::int(1), Dy::int(3)), Dy::int(7));
}
