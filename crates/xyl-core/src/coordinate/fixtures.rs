//! FORMS-COORDINATE v4's joint-read fixtures (Mycelium proposal 78.173,
//! section 5), reproduced from the definitions of section 1 alone, without
//! linking `forms`. `R` is the root of genesis `6a` repeated 32 times and
//! epoch `0x92`. Fixtures 2 and 3 (the sexagesimal reading) are the sector
//! tests in the parent module.

use super::*;
use crate::Q;

fn rational(n: i64, d: i64) -> Q {
    Q::new(n, d).unwrap()
}

fn root() -> Origin {
    Origin::root([0x6a; 32], 0x92)
}

fn letter(l: char) -> Key {
    Key::Axis(Axis::of(l, 1).unwrap())
}

fn turning(a: u32, b: u32) -> Key {
    Key::generator(Axis::unit(true, a).unwrap(), Axis::unit(true, b).unwrap()).unwrap()
}

/// The one-component lanes: S on `e_S1`, G on `e_G1`, T on `e_T1`, H on the
/// turning about axis 1, `e_+2+3`, in degrees.
fn lanes(s: Q, g: Q, t: Q, h: Q) -> Lanes {
    Lanes::base(
        Vector::one(letter('S'), s),
        Vector::one(letter('G'), g),
        Vector::one(letter('T'), t),
        Vector::one(turning(2, 3), h),
    )
    .unwrap()
}

/// A one-component body at `origin`, bell `h`, its `K` stated.
fn at(origin: &Origin, h: u64, (s, g, t, hh): (Q, Q, Q, Q), k: u64) -> Coordinate {
    Coordinate::with_k(origin.clone(), h, lanes(s, g, t, hh), vec![k])
}

fn zero() -> Q {
    Q::ZERO
}

fn hash() -> String {
    "6a".repeat(32)
}

/// The two-plane H body of fixtures 11 to 13: S, G, T at 0, H on `e_+2+3`
/// and `e_+3+1`, at `R`, bell 5, `K` read.
fn two(a: Q, b: Q) -> Coordinate {
    let h = Vector::of(vec![(turning(2, 3), a), (turning(3, 1), b)]).unwrap();
    let lanes = Lanes::base(
        Vector::one(letter('S'), zero()),
        Vector::one(letter('G'), zero()),
        Vector::one(letter('T'), zero()),
        h,
    )
    .unwrap();
    Coordinate::of(root(), 5, lanes).unwrap()
}

#[test]
fn fixture_1_the_order_and_the_writing() {
    let r = root();
    let enclosed = r.within(Local {
        h: 3,
        lanes: lanes(Q::int(2), zero(), zero(), zero()),
        k: vec![0],
    });
    let order = vec![
        at(&r, 5, (rational(-1, 2), zero(), zero(), zero()), 1),
        at(&r, 5, (rational(1, 3), Q::int(-1), zero(), zero()), 9),
        at(&r, 5, (rational(1, 3), zero(), zero(), zero()), 1),
        at(&r, 5, (rational(1, 3), zero(), zero(), zero()), 2),
        at(&r, 5, (rational(1, 2), zero(), zero(), zero()), 1),
        at(&r, 6, (Q::int(-7), zero(), zero(), zero()), 0),
        at(&enclosed, 0, (zero(), zero(), zero(), zero()), 0),
    ];
    let mut sorted = order.clone();
    sorted.reverse();
    sorted.sort();
    assert_eq!(sorted, order, "1a: the canonical order");
    // 1b: the writings.
    assert_eq!(
        order[2].written(),
        format!(
            "{}.00000092/0000000000000005:1/3:0/1:0/1:0/1:0000000000000001",
            hash()
        )
    );
    assert!(order[0].written().contains(":-1/2:"));
    assert_eq!(
        order[6].written(),
        format!(
            "{}.00000092/{}/{}",
            hash(),
            "0000000000000003:2/1:0/1:0/1:0/1:0000000000000000",
            "0000000000000000:0/1:0/1:0/1:0/1:0000000000000000"
        )
    );
    // 1c: TPoint's case and a local part.
    let top = Q::from_big(num_rational::BigRational::from_integer(
        num_bigint::BigInt::from(u64::MAX),
    ));
    let tpoint = at(&r, 40961, (Q::int(3), top, zero(), Q::int(17)), 812);
    assert_eq!(
        tpoint.written(),
        format!(
            "{}.00000092/000000000000a001:3/1:ffffffffffffffff/1:0/1:11/1:000000000000032c",
            hash()
        )
    );
    let local = at(
        &r,
        10,
        (rational(7, 30), zero(), rational(1, 900), Q::int(2)),
        3,
    );
    assert_eq!(
        local.local.written(),
        "000000000000000a:7/1e:0/1:1/384:2/1:0000000000000003"
    );
}

#[test]
fn fixture_4_within_on_the_callers_interval() {
    let r = root();
    let bounds = |lo: Q, hi: Q| Bounds {
        origin: root(),
        lane: Lane::S,
        intervals: vec![(letter('S'), Interval { lo, hi })],
    };
    let below = &rational(1, 3) - &Q::from_i128(1, 10i128.pow(15)).unwrap();
    let fine = &rational(1, 3) + &Q::from_i128(1, 3 * 60i128.pow(6)).unwrap();
    let fine_hi = &rational(1, 3) + &Q::from_i128(1, 46_656_000_000).unwrap();
    for (s, (lo, hi), side) in [
        (
            rational(1, 3),
            (rational(1, 3), rational(7, 20)),
            Side::Within,
        ),
        (
            rational(7, 20),
            (rational(1, 3), rational(7, 20)),
            Side::Above,
        ),
        (below, (rational(1, 3), rational(7, 20)), Side::Below),
        (fine, (rational(1, 3), fine_hi), Side::Within),
        (rational(2, 5), (zero(), Q::ONE), Side::Within),
        (
            rational(2, 5),
            (rational(1, 3), rational(7, 20)),
            Side::Above,
        ),
    ] {
        let body = at(&r, 7, (s, zero(), zero(), zero()), 4);
        let sides = body.within(&bounds(lo, hi)).unwrap();
        assert_eq!(sides.sides, vec![("e_S1".to_string(), side)]);
    }
    let body = at(&r, 7, (rational(1, 3), zero(), zero(), zero()), 4);
    let off = Bounds {
        origin: root(),
        lane: Lane::S,
        intervals: vec![(
            Key::Axis(Axis::of('S', 2).unwrap()),
            Interval {
                lo: rational(1, 3),
                hi: rational(7, 20),
            },
        )],
    };
    assert_eq!(
        body.within(&off),
        Err(CoordinateRefusal::ShapesDiffer {
            lane: Lane::S,
            held: vec!["e_S1".into()],
            asked: vec!["e_S2".into()],
        })
    );
}

#[test]
fn fixture_5_origins_are_never_compared() {
    let r = root();
    let enclosed = r.within(Local {
        h: 3,
        lanes: lanes(Q::int(2), zero(), zero(), zero()),
        k: vec![0],
    });
    let body = at(&r, 5, (rational(1, 3), zero(), zero(), zero()), 0);
    let bounds = Bounds {
        origin: enclosed.clone(),
        lane: Lane::S,
        intervals: vec![(
            letter('S'),
            Interval {
                lo: rational(1, 3),
                hi: rational(7, 20),
            },
        )],
    };
    assert_eq!(
        body.within(&bounds),
        Err(CoordinateRefusal::OriginsDiffer {
            held: r.written(),
            asked: enclosed.written(),
        })
    );
}

#[test]
fn fixture_6_a_range_on_one_component() {
    let r = root();
    let later = Origin::root([0x6a; 32], 0x93);
    let mut bodies = Bodies::new();
    let mut all = Vec::new();
    for i in 0..20 {
        all.push(at(&r, 5, (rational(i, 10), zero(), zero(), zero()), 0));
    }
    all.push(at(&r, 6, (rational(1, 2), zero(), zero(), zero()), 0));
    all.push(at(&later, 6, (rational(1, 2), zero(), zero(), zero()), 0));
    for g in [Q::int(-1), rational(1, 4), Q::int(2)] {
        all.push(at(&r, 5, (rational(1, 2), g, zero(), zero()), 0));
    }
    for b in &all {
        bodies.hold(b.clone());
    }
    let here = at(&r, 5, (zero(), zero(), zero(), zero()), 0);
    let span = Interval {
        lo: rational(1, 2),
        hi: rational(9, 10),
    };
    let range = bodies.range(&here, &Lane::S, &letter('S'), &span).unwrap();
    let got: Vec<(Q, Q)> = range
        .bodies
        .iter()
        .map(|b| {
            let s = b.component(&Lane::S, &letter('S')).unwrap().clone();
            let g = b.component(&Lane::G, &letter('G')).unwrap().clone();
            (s, g)
        })
        .collect();
    let half = rational(1, 2);
    assert_eq!(
        got,
        vec![
            (half.clone(), Q::int(-1)),
            (half.clone(), zero()),
            (half.clone(), rational(1, 4)),
            (half, Q::int(2)),
            (rational(3, 5), zero()),
            (rational(7, 10), zero()),
            (rational(4, 5), zero()),
        ]
    );
    // Each range is what a test of every body selects, by two bisections
    // of at most floor(log2 L) + 1 probes each.
    let tested: Vec<&Coordinate> = bodies
        .held()
        .iter()
        .filter(|b| b.origin == here.origin && b.local.h == here.local.h)
        .filter(|b| span.side(b.component(&Lane::S, &letter('S')).unwrap()) == Side::Within)
        .collect();
    assert_eq!(range.bodies.iter().collect::<Vec<_>>(), tested);
    let bound = |n: usize| usize::BITS - n.leading_zeros();
    let held = bodies.held().len();
    assert!(range.probes.0 <= bound(held) && range.probes.1 <= bound(held));
    assert_eq!(bound(512) - bound(128), 2);
    let at_half = at(&r, 5, (rational(1, 2), zero(), zero(), zero()), 0);
    let g = bodies
        .range(
            &at_half,
            &Lane::G,
            &letter('G'),
            &Interval {
                lo: zero(),
                hi: Q::ONE,
            },
        )
        .unwrap();
    let gs: Vec<Q> = g
        .bodies
        .iter()
        .map(|b| b.component(&Lane::G, &letter('G')).unwrap().clone())
        .collect();
    assert_eq!(gs, vec![zero(), rational(1, 4)]);
    let empty = bodies
        .range(
            &here,
            &Lane::S,
            &letter('S'),
            &Interval {
                lo: Q::ONE,
                hi: rational(1, 2),
            },
        )
        .unwrap();
    assert!(empty.bodies.is_empty());
    let s2 = Key::Axis(Axis::of('S', 2).unwrap());
    assert!(matches!(
        bodies.range(&here, &Lane::S, &s2, &span),
        Err(CoordinateRefusal::NoComponent { .. })
    ));
}

#[test]
fn fixture_7_the_advance_composes() {
    let r = root();
    let body = at(&r, 10, (rational(1, 3), zero(), zero(), Q::int(90)), 0);
    let rates = lanes(rational(1, 7), zero(), rational(1, 2), Q::int(72));
    let to = body.advance(&rates, 40).unwrap();
    assert_eq!(
        to,
        at(
            &r,
            40,
            (rational(97, 21), zero(), Q::int(15), Q::int(2250)),
            6
        )
    );
    let via = body
        .advance(&rates, 22)
        .unwrap()
        .advance(&rates, 40)
        .unwrap();
    assert_eq!(via, to);
    assert_eq!(
        body.advance(&rates, 9),
        Err(CoordinateRefusal::Backward { held: 10, asked: 9 })
    );
    let other = Lanes::base(
        Vector::one(letter('S'), rational(1, 7)),
        Vector::one(letter('G'), zero()),
        Vector::one(letter('T'), rational(1, 2)),
        Vector::one(turning(3, 1), Q::int(72)),
    )
    .unwrap();
    assert_eq!(
        body.advance(&other, 40),
        Err(CoordinateRefusal::ShapesDiffer {
            lane: Lane::H,
            held: vec!["e_+2+3".into()],
            asked: vec!["e_+3+1".into()],
        })
    );
    let back = at(&r, 0, (zero(), zero(), zero(), Q::int(90)), 0)
        .advance(&lanes(zero(), zero(), zero(), Q::int(-180)), 5)
        .unwrap();
    assert_eq!(
        back.component(&Lane::H, &turning(2, 3)),
        Some(&Q::int(-810))
    );
    assert_eq!(back.local.k, vec![2]);
}

#[test]
fn fixture_8_an_events_bell() {
    let body = at(&root(), 4, (zero(), zero(), zero(), zero()), 0);
    let rates = lanes(rational(2, 3), zero(), zero(), zero());
    assert_eq!(
        body.event_bell(&rates, &Lane::S, &letter('S'), &Q::ONE),
        Ok(EventBell::At(rational(11, 2)))
    );
    assert_eq!(
        body.event_bell(&rates, &Lane::G, &letter('G'), &Q::ONE),
        Ok(EventBell::Unmoving)
    );
    assert_eq!(
        body.event_bell(&rates, &Lane::S, &letter('S'), &Q::int(-1)),
        Ok(EventBell::Behind(rational(5, 2)))
    );
    let s2 = Key::Axis(Axis::of('S', 2).unwrap());
    assert!(matches!(
        body.event_bell(&rates, &Lane::S, &s2, &Q::ONE),
        Err(CoordinateRefusal::NoComponent { .. })
    ));
}

#[test]
fn fixtures_9_and_10_merger_and_fission() {
    let r = root();
    let a = at(&r, 5, (Q::ONE, zero(), zero(), Q::int(720)), 2);
    let b = at(&r, 5, (Q::int(3), zero(), zero(), zero()), 0);
    let m = a.merge(&Q::ONE, &b, &Q::int(3)).unwrap();
    assert_eq!(
        m.body,
        at(&r, 5, (rational(5, 2), zero(), zero(), Q::int(180)), 0)
    );
    assert_eq!(
        m.offsets.0,
        lanes(rational(-3, 2), zero(), zero(), Q::int(540))
    );
    assert_eq!(
        m.offsets.1,
        lanes(rational(1, 2), zero(), zero(), Q::int(-180))
    );
    let same = a.merge(&Q::int(2), &a, &Q::int(5)).unwrap();
    assert_eq!(same.body, a);
    assert_eq!(same.offsets.0, lanes(zero(), zero(), zero(), zero()));
    let later = at(
        &Origin::root([0x6a; 32], 0x93),
        5,
        (Q::int(3), zero(), zero(), zero()),
        0,
    );
    assert!(matches!(
        a.merge(&Q::ONE, &later, &Q::int(3)),
        Err(CoordinateRefusal::OriginsDiffer { .. })
    ));
    let six = at(&r, 6, (Q::int(3), zero(), zero(), zero()), 0);
    assert_eq!(
        a.merge(&Q::ONE, &six, &Q::int(3)),
        Err(CoordinateRefusal::BellsDiffer { held: 5, asked: 6 })
    );
    assert_eq!(
        a.merge(&Q::ONE, &b, &Q::int(-1)),
        Err(CoordinateRefusal::NoHolding)
    );
    // Fission gives both back exactly.
    let (fa, fb) = m
        .body
        .fission((&Q::ONE, &Q::int(3)), (&m.offsets.0, &m.offsets.1))
        .unwrap();
    assert_eq!((fa, fb), (a.clone(), b.clone()));
    // At holdings 1 and 1 the offsets sum to (-1, 0, 0, 360), which no
    // merger leaves; holdings 2 and -2 have no holding.
    assert_eq!(
        m.body
            .fission((&Q::ONE, &Q::ONE), (&m.offsets.0, &m.offsets.1)),
        Err(CoordinateRefusal::Unbalanced {
            sum: lanes(Q::int(-1), zero(), zero(), Q::int(360)).written(),
        })
    );
    assert_eq!(
        m.body
            .fission((&Q::int(2), &Q::int(-2)), (&m.offsets.0, &m.offsets.1)),
        Err(CoordinateRefusal::NoHolding)
    );
    // Generated cases: S on two axes, H on three planes, signed components
    // of several denominators, holdings of both signs (and 0).
    let s = |x: Q, y: Q| {
        Vector::of(vec![
            (letter('S'), x),
            (Key::Axis(Axis::of('S', 2).unwrap()), y),
        ])
        .unwrap()
    };
    let h = |x: Q, y: Q, z: Q| {
        Vector::of(vec![
            (turning(2, 3), x),
            (turning(1, 2), y),
            (turning(3, 1), z),
        ])
        .unwrap()
    };
    for (i, (wa, wb)) in [
        (Q::ONE, Q::int(2)),
        (Q::int(-3), rational(1, 2)),
        (zero(), Q::int(5)),
        (rational(7, 3), Q::int(-1)),
    ]
    .into_iter()
    .enumerate()
    {
        let n = i64::try_from(i).unwrap() + 2;
        let body = |sx: Q, hx: Q| {
            let l = Lanes::base(
                s(sx.clone(), rational(-1, n)),
                Vector::one(letter('G'), rational(1, n + 1)),
                Vector::one(letter('T'), zero()),
                h(hx.clone(), rational(-7, n), Q::int(400)),
            )
            .unwrap();
            Coordinate::of(root(), 3, l).unwrap()
        };
        let (x, y) = (
            body(rational(3, n), Q::int(90)),
            body(rational(-5, n + 3), rational(-361, n)),
        );
        let merged = x.merge(&wa, &y, &wb).unwrap();
        let back = merged
            .body
            .fission((&wa, &wb), (&merged.offsets.0, &merged.offsets.1))
            .unwrap();
        assert_eq!(back, (x, y), "case {i}");
    }
}

#[test]
fn fixtures_11_to_13_vector_lanes() {
    let (a, b, c) = (
        two(rational(1, 3), rational(7, 4)),
        two(rational(1, 3), rational(-1, 2)),
        two(rational(1, 2), zero()),
    );
    let mut held = vec![c.clone(), a.clone(), b.clone()];
    held.sort();
    assert_eq!(held, vec![b.clone(), a.clone(), c.clone()], "11: the order");
    assert_eq!(a.local.lanes.get(&Lane::H).unwrap().written(), "(1/3,7/4)");
    assert_eq!(
        a.local.written(),
        "0000000000000005:0/1:0/1:0/1:(1/3,7/4):(0000000000000000,0000000000000000)"
    );
    assert_eq!(a.reading(&Lane::H, 0).unwrap(), "0 \u{d7} 1");
    assert_eq!(a.reading(&Lane::H, 1).unwrap(), "0;20 \u{d7} 1;45");
    assert_eq!(b.reading(&Lane::H, 0).unwrap(), "0 \u{d7} -1");
    assert_eq!(b.reading(&Lane::H, 1).unwrap(), "0;20 \u{d7} -1;30");
    let bounds = Bounds {
        origin: root(),
        lane: Lane::H,
        intervals: vec![
            (
                turning(2, 3),
                Interval {
                    lo: zero(),
                    hi: Q::ONE,
                },
            ),
            (
                turning(3, 1),
                Interval {
                    lo: Q::ONE,
                    hi: Q::int(2),
                },
            ),
        ],
    };
    let sides = c.within(&bounds).unwrap();
    assert_eq!(sides.whole(), Err(("e_+3+1".to_string(), Side::Below)));
    assert_eq!(a.within(&bounds).unwrap().whole(), Ok(()));
    // 12: a range on one component, then a filter.
    let d = two(rational(2, 5), rational(7, 4));
    let mut bodies = Bodies::new();
    for x in [&a, &b, &c, &d] {
        bodies.hold(x.clone());
    }
    let origin_body = two(zero(), zero());
    let range = bodies
        .range(
            &origin_body,
            &Lane::H,
            &turning(2, 3),
            &Interval {
                lo: rational(1, 3),
                hi: rational(1, 2),
            },
        )
        .unwrap();
    assert_eq!(range.bodies, &[b.clone(), a.clone(), d.clone()][..]);
    let kept = range.filter(
        &Lane::H,
        &turning(3, 1),
        &Interval {
            lo: Q::ONE,
            hi: Q::int(2),
        },
    );
    assert_eq!(kept, vec![&a, &d]);
    // 13: a merger and fission per component.
    let e = two(Q::ONE, rational(-1, 4));
    let m = a.merge(&Q::ONE, &e, &Q::int(2)).unwrap();
    assert_eq!(m.body, two(rational(7, 9), rational(5, 12)));
    assert_eq!(m.body.local.k, vec![0, 0]);
    assert_eq!(m.offsets.0.get(&Lane::H).unwrap().written(), "(-4/9,4/3)");
    assert_eq!(m.offsets.1.get(&Lane::H).unwrap().written(), "(2/9,-2/3)");
    assert_eq!(m.body.reading(&Lane::H, 1).unwrap(), "0;46 \u{d7} 0;25");
    let back = m
        .body
        .fission((&Q::ONE, &Q::int(2)), (&m.offsets.0, &m.offsets.1))
        .unwrap();
    assert_eq!(back, (a.clone(), e));
    let single = at(&root(), 5, (zero(), zero(), zero(), zero()), 0);
    assert_eq!(
        origin_body.merge(&Q::ONE, &single, &Q::ONE),
        Err(CoordinateRefusal::ShapesDiffer {
            lane: Lane::H,
            held: vec!["e_+2+3".into(), "e_+3+1".into()],
            asked: vec!["e_+2+3".into()],
        })
    );
}

#[test]
fn fixture_14_re_origin_across_a_throat() {
    let r = root();
    let body = at(&r, 1, (rational(3, 5), zero(), zero(), Q::int(45)), 0);
    let poles = Poles {
        hold: rational(4, 5),
        reach: rational(1, 5),
    };
    let into = r.within(Local {
        h: 1,
        lanes: lanes(zero(), zero(), zero(), zero()),
        k: vec![0],
    });
    let (passed, after) = body.pass(&into, &poles).unwrap();
    assert_eq!(passed.origin, into);
    assert_eq!(
        passed.local,
        at(
            &r,
            1,
            (rational(-3, 5), zero(), rational(-6, 5), Q::int(45)),
            0
        )
        .local
    );
    assert_eq!(
        after,
        Poles {
            hold: rational(1, 5),
            reach: rational(4, 5)
        }
    );
    assert_eq!(&after.hold + &after.reach, Q::ONE);
    assert_eq!(
        (poles.face(), after.face()),
        (Some(rational(4, 5)), Some(rational(1, 5)))
    );
    let (home, exchanged) = body.pass(&r, &poles).unwrap();
    assert_eq!(home, body);
    assert_eq!(exchanged.face(), Some(rational(1, 5)));
    let half = at(&r, 1, (rational(1, 2), zero(), zero(), Q::int(45)), 0);
    assert_eq!(
        half.pass(&into, &poles),
        Err(CoordinateRefusal::SpanDiffers {
            held: rational(1, 2).to_string(),
            read: rational(3, 5).to_string(),
        })
    );
    let none = Poles {
        hold: rational(1, 2),
        reach: rational(-1, 2),
    };
    assert_eq!(
        body.pass(&into, &none),
        Err(CoordinateRefusal::NoWhole {
            span: Q::ONE.to_string()
        })
    );
}

#[test]
fn fixture_15_dilation_invariance() {
    for ((hold, reach), delta, flowed) in [
        ((Q::ONE, Q::int(3)), rational(1, 4), rational(16, 19)),
        (
            (rational(2, 7), rational(5, 3)),
            rational(6, 41),
            rational(1681, 1891),
        ),
        (
            (rational(9, 4), rational(1, 60)),
            rational(135, 136),
            rational(18496, 18631),
        ),
    ] {
        for lambda in [rational(7, 3), rational(1, 11), rational(1_000_003, 2)] {
            let base = crate::shares::faces(&hold, &reach).unwrap().convergent;
            let dilated = crate::shares::faces(&(&lambda * &hold), &(&lambda * &reach))
                .unwrap()
                .convergent;
            assert_eq!((base.clone(), dilated), (delta.clone(), delta.clone()));
            let rest = &Q::ONE - &base;
            assert_eq!(crate::shares::flow(&base, &rest), Some(flowed.clone()));
        }
    }
    // A flow of the push 1 is refused (NotPush).
    assert_eq!(crate::shares::flow(&Q::ONE, &rational(1, 2)), None);
}

#[test]
fn fixture_16_keys_k_per_component_and_unopened_lanes() {
    // 16a: H held in the keys' written order, K per component.
    let h = Vector::of(vec![
        (turning(3, 1), Q::int(1080)),
        (turning(2, 3), Q::int(400)),
        (turning(1, 2), Q::int(-30)),
    ])
    .unwrap();
    assert_eq!(h.keys(), vec!["e_+1+2", "e_+2+3", "e_+3+1"]);
    assert_eq!(h.written(), "(-1e/1,190/1,438/1)");
    let l = Lanes::base(
        Vector::one(letter('S'), zero()),
        Vector::one(letter('G'), zero()),
        Vector::one(letter('T'), zero()),
        h,
    )
    .unwrap();
    let still = Lanes::base(
        Vector::one(letter('S'), zero()),
        Vector::one(letter('G'), zero()),
        Vector::one(letter('T'), zero()),
        Vector::of(vec![
            (turning(3, 1), zero()),
            (turning(2, 3), zero()),
            (turning(1, 2), zero()),
        ])
        .unwrap(),
    )
    .unwrap();
    let body = Coordinate::with_k(root(), 0, l, vec![9, 9, 9])
        .advance(&still, 0)
        .unwrap();
    assert_eq!(body.local.k, vec![0, 1, 3]);
    assert!(body
        .local
        .written()
        .ends_with(":(0000000000000000,0000000000000001,0000000000000003)"));
    // 16b: an unopened lane, and the same lane under a stated opening.
    let w = |openings: &Openings| {
        Lanes::of(
            vec![
                (
                    Lane::Opened("W".into(), 1),
                    Vector::one(letter('W'), zero()),
                ),
                (Lane::H, Vector::one(turning(2, 3), zero())),
                (Lane::T, Vector::one(letter('T'), zero())),
                (Lane::G, Vector::one(letter('G'), zero())),
                (Lane::S, Vector::one(letter('S'), zero())),
            ],
            openings,
        )
    };
    assert_eq!(
        w(&Openings::ruled()),
        Err(CoordinateRefusal::LaneNotOpened {
            name: "W".into(),
            version: 1
        })
    );
    let admitted = w(&Openings::stated(vec![("W".into(), 1)])).unwrap();
    let order: Vec<Lane> = admitted.lanes().iter().map(|(l, _)| l.clone()).collect();
    assert_eq!(
        order,
        vec![
            Lane::S,
            Lane::G,
            Lane::T,
            Lane::H,
            Lane::Opened("W".into(), 1)
        ]
    );
    // 16c: H stands only on turning planes; S may hold boosts.
    let unit = |p: bool, k: u32| Axis::unit(p, k).unwrap();
    let on_h = |k: Key| {
        Lanes::base(
            Vector::one(letter('S'), zero()),
            Vector::one(letter('G'), zero()),
            Vector::one(letter('T'), zero()),
            Vector::one(k, zero()),
        )
    };
    assert!(on_h(turning(2, 3)).is_ok());
    assert!(on_h(Key::generator(unit(false, 2), unit(false, 3)).unwrap()).is_ok());
    let plane = Key::generator(unit(true, 1), unit(false, 1)).unwrap();
    let mixed = Key::generator(unit(true, 1), unit(false, 2)).unwrap();
    let unsigned = Key::generator(Axis::of('Z', 1).unwrap(), Axis::of('Z', 2).unwrap()).unwrap();
    for (k, plane) in [
        (plane, Plane::Boost),
        (mixed, Plane::Mixed),
        (unsigned, Plane::Unsigned),
    ] {
        assert_eq!(
            on_h(k),
            Err(CoordinateRefusal::NotTurningPlane {
                key: k.written(),
                plane
            })
        );
    }
    let boosts = Vector::of(vec![(plane, rational(3, 5)), (mixed, rational(-1, 7))]).unwrap();
    assert!(Lanes::base(
        boosts,
        Vector::one(letter('G'), zero()),
        Vector::one(letter('T'), zero()),
        Vector::one(turning(2, 3), zero()),
    )
    .is_ok());
    assert_eq!(
        Axis::unit(true, 0),
        Err(CoordinateRefusal::AxisUnwritable {
            letter: '+',
            index: 0
        })
    );
    // 16d: refused by name.
    let without_h = Lanes::of(
        vec![
            (Lane::S, Vector::one(letter('S'), zero())),
            (Lane::G, Vector::one(letter('G'), zero())),
            (Lane::T, Vector::one(letter('T'), zero())),
        ],
        &Openings::ruled(),
    );
    assert_eq!(
        without_h,
        Err(CoordinateRefusal::LaneMissing { lane: Lane::H })
    );
    let twice = Lanes::of(
        vec![
            (Lane::S, Vector::one(letter('S'), zero())),
            (Lane::S, Vector::one(letter('S'), zero())),
        ],
        &Openings::ruled(),
    );
    assert_eq!(
        twice,
        Err(CoordinateRefusal::LaneRepeated { lane: Lane::S })
    );
    assert_eq!(
        on_h(Key::Axis(Axis::of('Z', 1).unwrap())),
        Err(CoordinateRefusal::NotGenerator { key: "e_Z1".into() })
    );
    assert_eq!(
        Vector::of(vec![(letter('S'), zero()), (letter('S'), Q::ONE)]),
        Err(CoordinateRefusal::KeyRepeated { key: "e_S1".into() })
    );
    assert!(matches!(
        Vector::of(vec![(turning(2, 3), zero()), (turning(3, 2), zero())]),
        Err(CoordinateRefusal::KeyRepeated { .. })
    ));
    let z1 = Axis::of('Z', 1).unwrap();
    assert_eq!(
        Key::generator(z1, z1),
        Err(CoordinateRefusal::NotPlane { axis: "Z1".into() })
    );
    assert_eq!(
        Axis::of('z', 1),
        Err(CoordinateRefusal::AxisUnwritable {
            letter: 'z',
            index: 1
        })
    );
    assert_eq!(
        Axis::of('Z', 0),
        Err(CoordinateRefusal::AxisUnwritable {
            letter: 'Z',
            index: 0
        })
    );
}
