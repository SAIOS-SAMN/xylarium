//! Tests of the schedule's encoding and the core's extent.

use super::*;

#[test]
fn schedule_encoding_round_trips_and_refuses_damage() {
    let m = Mind {
        star: 4130,
        w: 1,
        epoch: Dy::ratio(157, 100),
        region: Region::Cryogenic,
        star_mass: Dy::ratio(3, 2),
        r_kpc: Dy::ratio(17, 10),
        steps: vec![Dy::ratio(14, 10), Dy::ratio(15, 10), Dy::ratio(157, 100)],
        setbacks: 2,
        seeded: false,
        fed_settled: true,
    };
    let s = Schedule {
        minds: vec![m],
        curve: vec![Dy::ZERO, Dy::ratio(1, 64)],
        limits: Limits {
            worlds: 9,
            minds: 1,
            by_region: vec![(Region::Umbral, 4)],
            rungs: vec![0, 1, 2],
            ..Limits::default()
        },
    };
    let buf = s.encode();
    let back = Schedule::decode(&buf).expect("whole");
    assert_eq!(back.minds, s.minds);
    assert_eq!(back.curve, s.curve);
    assert_eq!(back.limits, s.limits);
    assert!(Schedule::decode(&buf[..buf.len() - 1]).is_none());
    let mut extra = buf.clone();
    extra.push(0);
    assert!(Schedule::decode(&extra).is_none());
    assert_eq!(laws_digest().len(), 16);
}

#[test]
fn core_distance_counts_height_above_the_plane() {
    let bang = crate::genesis::bang();
    let stars: Vec<crate::galaxy::Star> = (0..300)
        .filter_map(|n| crate::galaxy::star(7, bang, n))
        .collect();
    let t = Dy::int(5);
    let mut lifted = 0;
    for s in &stars {
        let d = from_centre_kpc(s, t);
        let r = crate::galaxy::kpc_of_rings_dy(Dy::from_q(&s.r));
        if s.pop == crate::galaxy::Pop::Disc {
            // A disc star is never nearer than its cylinder's radius, and is
            // farther when above the plane.
            assert!(d >= r - Dy::sci(1, -9), "disc star {}: {d} < {r}", s.n);
            if d > r + Dy::sci(1, -6) {
                lifted += 1;
            }
        } else {
            // A bulge or nucleus orbit is a tilted circle, so its radius is the
            // star's distance.
            assert!(
                (d - crate::galaxy::kpc_of_rings_dy(s.orbit.radius)).abs() < Dy::sci(1, -6),
                "star {}: {d}",
                s.n
            );
        }
    }
    assert!(lifted > 0, "some disc star swings out of the plane");
}
