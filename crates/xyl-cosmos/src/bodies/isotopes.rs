//! The radiogenic isotopes: half-lives under a universe's laws and each
//! world's inventory at its star's birth.

use super::*;

/// The isotopes: (name, element, half-life Gyr, heat W/kg of the isotope,
/// its share of the element today).
pub fn isotopes() -> [(&'static str, &'static str, Q, Q, Q); 4] {
    [
        (
            "U238",
            "U",
            Q::ratio(4468, 1000),
            Q::ratio(946, 10_000_000),
            Q::ratio(9928, 10_000),
        ),
        (
            "U235",
            "U",
            Q::ratio(704, 1000),
            Q::ratio(569, 1_000_000),
            Q::ratio(72, 10_000),
        ),
        (
            "Th232",
            "Th",
            Q::ratio(1405, 100),
            Q::ratio(264, 10_000_000),
            Q::ONE,
        ),
        (
            "K40",
            "K",
            Q::ratio(1248, 1000),
            Q::ratio(292, 10_000_000),
            Q::ratio(117, 1_000_000),
        ),
    ]
}

/// Each isotope's Gamow barrier (ln of half-life x attempt rate): an alpha
/// decay's half-life scales as exp(gamow (alpha - 1)) under other laws;
/// potassium's beta decay has none.
pub(super) fn gamow(i: usize) -> Option<Dy> {
    [
        Some(Dy::sci(878, -1)),
        Some(Dy::int(86)),
        Some(Dy::sci(889, -1)),
        None,
    ]
    .get(i)
    .copied()
    .flatten()
}

/// Each isotope's half-life (Gyr) under a universe's laws.
pub fn half_life(i: usize, sc: &Scales) -> Dy {
    let t = isotopes()
        .get(i)
        .map(|x| Dy::from_q(&x.2))
        .unwrap_or(Dy::ONE);
    gamow(i).map(|g| t * sc.half_life(g)).unwrap_or(t)
}

/// A world's heat sources: each isotope's mass (kg) at its star's birth.
/// `kept` is, per isotope, the share made and still unspent at the star's
/// birth over the Sun's (see [`kept`]).
pub fn heat_sources(world: &World, elements: &[Element], kept: &[Q; 4]) -> [Q; 4] {
    let mut out: [Q; 4] = Default::default();
    let solar_age = Q::ratio(457, 100);
    for (i, (_, el, half, _, today)) in isotopes().iter().enumerate() {
        let Some(ei) = elements.iter().position(|e| &e.sym == el) else {
            continue;
        };
        // the element's whole mass: U, Th and K are lithophile, so it sits in
        // the mantle
        let mass_kg = &world.elements.get(ei).cloned().unwrap_or(Q::ZERO) * &Si::world_mass_kg();
        let tau = half
            .checked_div(&xyl_core::certified::ln2(BITS).mid())
            .unwrap_or(Q::ONE);
        let at_sun_birth = today * &ex(&solar_age.checked_div(&tau).unwrap_or(Q::ZERO));
        out[i] = on_grid(&(&mass_kg * &at_sun_birth) * &kept[i]);
    }
    out
}

/// Of an isotope made at a rate falling as exp(-t / tau_f) over `span` Gyr,
/// the share still present (decay time `tau`).
pub fn made_and_kept(tau: &Q, tau_f: &Q, span: &Q) -> Q {
    if span.signum() <= 0 {
        return Q::ONE;
    }
    let (lf, li) = (
        tau_f.recip().unwrap_or(Q::ONE),
        tau.recip().unwrap_or(Q::ONE),
    );
    let made = (&Q::ONE - &ex(&-(&lf * span)))
        .checked_div(&lf)
        .unwrap_or(Q::ONE);
    let kept = if (&li - &lf).abs() < Q::ratio(1, 1_000_000_000) {
        span * &ex(&-(&li * span))
    } else {
        (&ex(&-(&lf * span)) - &ex(&-(&li * span)))
            .checked_div(&(&li - &lf))
            .unwrap_or(Q::ZERO)
    };
    on_grid(
        kept.checked_div(&made.max(Q::ratio(1, 1_000_000_000_000)))
            .unwrap_or(Q::ZERO),
    )
}

/// Per isotope, what a star born at `born` Gyr where its ring formed stars
/// on `tau_f` Gyr (since the first stars at `t1`) kept of what its ring made,
/// over what the Sun's did (its ring's 6.997 Gyr infall, born at 9.2 Gyr after
/// first stars at 0.3 Gyr: the reference the isotope abundances are read against).
/// Decay runs at the universe's own half-lives.
pub fn kept(tau_f: &Q, t1: &Q, born: &Q, sc: &Scales) -> [Q; 4] {
    let ln2 = xyl_core::certified::ln2(BITS).mid();
    let mut out: [Q; 4] = Default::default();
    for (i, (_, _, half, _, _)) in isotopes().iter().enumerate() {
        let tau0 = half.checked_div(&ln2).unwrap_or(Q::ONE);
        let sun = made_and_kept(&tau0, &Q::ratio(6997, 1000), &Q::ratio(89, 10));
        let tau = half_life(i, sc).to_q().checked_div(&ln2).unwrap_or(Q::ONE);
        let here = made_and_kept(&tau, tau_f, &(born - t1));
        out[i] = on_grid(
            here.checked_div(&sun.max(Q::ratio(1, 1_000_000_000_000)))
                .unwrap_or(Q::ZERO),
        );
    }
    out
}
