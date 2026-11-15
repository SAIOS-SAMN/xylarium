//! The theorem register's identities that read no founding (docs/model.md,
//! Axioms and theorems; #249): each test asserts a theorem's statement on
//! the built laws it names, for spins across `(0, 1]`, exact. They stand
//! outside the crate's sources so they never move the founding digest.

use xyl_core::shares::{dilation, flow};
use xyl_core::Q;
use xyl_cosmos::bounce::torsion_share;
use xyl_cosmos::circuit::{bell, echo_at};

/// Spins across `(0, 1]`, the Kerr bound included.
fn spins() -> Vec<Q> {
    [(1, 64), (1, 7), (1, 3), (1, 2), (5, 7), (63, 64), (1, 1)]
        .iter()
        .filter_map(|&(n, d)| Q::new(n, d))
        .collect()
}

/// The torsion share of a unit hold at spin `a`, and its twist
/// `n = a / (4 + a)` (T2); `None` where `4 + a` is zero.
fn share_and_twist(a: &Q) -> Option<(Q, Q)> {
    let s = torsion_share(&Q::ONE, a);
    let n = a.checked_div(&(&Q::int(4) + a))?;
    Some((s, n))
}

#[test]
fn t2_the_torsion_share_dilates_to_three_plus_a() {
    for a in spins() {
        let (s, n) = share_and_twist(&a).expect("a spin in (0, 1]");
        assert_eq!(
            dilation(&s),
            Some(&Q::int(3) + &a),
            "D(s) = 3 + a at a = {a}"
        );
        // the twist is the pair's separation, 2s - 1 (T2, T9)
        assert_eq!(&(&Q::int(2) * &s) - &Q::ONE, n, "2s - 1 = n at a = {a}");
    }
}

#[test]
fn t4_the_echo_is_the_bell_over_the_open_share() {
    // the even pair returns after two bells; the echo rises with the share;
    // a share of one or more has no return
    assert_eq!(
        echo_at(&Q::new(1, 2).expect("1/2")),
        bell().mul(&xyl_core::dyadic::Dy::from_q(&Q::int(2)))
    );
    let mut last = echo_at(&Q::ZERO);
    assert_eq!(last, bell(), "no share returns in one bell");
    for a in spins() {
        let (s, _) = share_and_twist(&a).expect("a spin in (0, 1]");
        let e = echo_at(&s);
        assert!(e > last, "the echo rises with the share at a = {a}");
        last = e;
    }
    assert_eq!(
        echo_at(&Q::ONE),
        xyl_core::dyadic::Dy::ZERO,
        "a share of one has no return"
    );
}

#[test]
fn t9_the_pair_share_is_the_twist_composed_with_its_third() {
    for a in spins() {
        let (s, n) = share_and_twist(&a).expect("a spin in (0, 1]");
        let pair = flow(&s, &-(&Q::ONE - &s));
        let third = n.checked_div(&Q::int(3)).expect("3 > 0");
        assert_eq!(
            pair,
            flow(&n, &third),
            "s (+) -(1 - s) = n (+) n / 3 at a = {a}"
        );
    }
}

#[test]
fn t16_a_paired_throats_faces_multiply_to_three_plus_a_squared() {
    for a in spins() {
        let (s, n) = share_and_twist(&a).expect("a spin in (0, 1]");
        let (ds, dn) = (dilation(&s).expect("s < 1"), dilation(&n).expect("n < 1"));
        let three_a = &Q::int(3) + &a;
        for pol in [-3_i32, -1, 0, 1, 2, 5] {
            let up = (0..pol.unsigned_abs()).fold(Q::ONE, |x, _| &x * &dn);
            let (near, far) = match pol >= 0 {
                true => (&ds * &up, ds.checked_div(&up).expect("D(n) > 0")),
                false => (ds.checked_div(&up).expect("D(n) > 0"), &ds * &up),
            };
            assert_eq!(
                &near * &far,
                &three_a * &three_a,
                "product (3 + a)^2 at a = {a}, A = {pol}"
            );
            if pol != 0 {
                assert_ne!(
                    near, far,
                    "an open kind is held on one face only at A = {pol}"
                );
            }
        }
    }
}

#[test]
fn t17_the_far_face_dilates_to_six_plus_a_over_two_plus_a() {
    for a in spins() {
        let (s, _) = share_and_twist(&a).expect("a spin in (0, 1]");
        let want = (&Q::int(6) + &a)
            .checked_div(&(&Q::int(2) + &a))
            .expect("2 + a > 0");
        assert_eq!(
            dilation(&(&Q::ONE - &s)),
            Some(want),
            "D(1 - s) = (6 + a) / (2 + a) at a = {a}"
        );
    }
}

#[test]
fn t3_the_apsis_ratio_is_the_deficits_dilation_and_the_resonances_return() {
    use xyl_core::shares::deficit;
    use xyl_cosmos::bounce::horizon_reach;
    let m = Q::new(3, 2).expect("3/2");
    for (n, d) in [(1, 4), (1, 1), (5, 2), (7, 1), (100, 3)] {
        let r = &m * &Q::new(n, d).expect("a reach");
        let delta = deficit(&m, &r).expect("a hold and a reach");
        let want = &Q::ONE + &horizon_reach(&m, &m).checked_div(&r).expect("r > 0");
        assert_eq!(
            dilation(&delta),
            Some(want),
            "D(delta) = 1 + R_h / r at r = {r}"
        );
    }
    // the reach (k - 1) M advances 360 / k degrees and returns after k
    for k in 2..=12_i64 {
        let r = &m * &Q::int(k - 1);
        let advance = &Q::int(360) * &deficit(&m, &r).expect("a hold and a reach");
        assert_eq!(
            advance,
            Q::new(360, k).expect("k > 0"),
            "360 / k at k = {k}"
        );
        let turns = advance.checked_div(&Q::int(360)).expect("360 > 0");
        assert_eq!(
            turns.denom(),
            num_bigint::BigInt::from(k),
            "returns after {k}"
        );
    }
}

#[test]
fn t9_the_positron_is_the_electron_phase_reflected() {
    // R = kappa (1 - 2x): the phase reflected, x -> 1 - x (the opposed
    // hemisphere, -x mod 1), reads -R on every kind, for x inside a turn
    use xyl_cosmos::circuit::electron_phase;
    for k in 1..=40_u32 {
        for (n, d) in [(1, 7), (1, 4), (1, 3), (1, 2), (5, 8), (11, 12)] {
            let x = Q::new(n, d).expect("a phase");
            let reflected = &Q::ONE - &x;
            assert_eq!(
                electron_phase(k, &reflected),
                -electron_phase(k, &x),
                "R(1 - x) = -R(x) at Z {k}, x = {x}"
            );
        }
    }
}
