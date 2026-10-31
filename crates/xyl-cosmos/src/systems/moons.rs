//! A world's moons on its own manifold, and the belts and comet cloud of a
//! system.

use super::*;

/// A world's moons, on its own manifold: it anchors the reach its pull holds
/// in its star's field, 2 pi a m / (M + m), out from five of its radii. The
/// matter is `budget` (world masses, the disc's unheld matter inside that
/// reach); the world holds its own share, m / (m + budget), and moons gather
/// across the rest degree by degree where their matter holds its region
/// against the star's `tide` at the world (disc::manifold_worlds, the
/// star tier's law one tier down), then settle about the world (see
/// [`settle_about`]). Each moon's forced eccentricity is its neighbours' pull
/// over their rates' distance from a first-order resonance. Returns the
/// moons' mass placed, which leaves the belts' books.
pub(super) fn moons(
    worlds: &mut Vec<World>,
    gas: &Gas,
    t_form: &Q,
    ice_share: &Q,
    star_suns: &Q,
    budget: &Q,
    tide: &Q,
) -> Q {
    let Some(parent) = worlds.len().checked_sub(1) else {
        return Q::ZERO;
    };
    if budget.signum() <= 0 {
        return Q::ZERO;
    }
    let (g_mass, g_a, g_ices, g_radius) = (
        worlds[parent].mass.clone(),
        worlds[parent].a.clone(),
        worlds[parent].ices.clone(),
        worlds[parent].radius.clone(),
    );
    let star_earths = star_suns
        * &crate::package::Si::star_mass_kg()
            .checked_div(&crate::package::Si::world_mass_kg())
            .unwrap_or(Q::ZERO);
    // its reach in its star's field, in its own radii: 2 pi a m / (M + m)
    let a_m = &(&g_a * &crate::package::Si::au());
    let reach_m = &(&(&Q::int(2) * &pi(BITS).mid()) * a_m)
        * &g_mass
            .checked_div(&(&star_earths + &g_mass))
            .unwrap_or(Q::ZERO);
    let r_m = &g_radius * &crate::package::Si::world_radius_m();
    let (inner, outer) = (
        Q::int(5),
        held(&reach_m.checked_div(&r_m).unwrap_or(Q::ZERO)),
    );
    if outer <= inner {
        return Q::ZERO;
    }
    // the world's manifold: it holds its own share, the moons the rest
    let all = &g_mass + budget;
    let own = g_mass.checked_div(&all).unwrap_or(Q::ONE);
    let (gathered, _) = crate::disc::manifold_worlds(&own, &Q::ZERO, tide);
    let embryos: Vec<Embryo> = gathered
        .iter()
        .enumerate()
        .map(|(k, (reach, holding))| Embryo {
            a: held(&(&outer * reach).max(inner.clone())),
            m: held(&(holding * &all)),
            phase: (&Q::int(i64::try_from(k).unwrap_or(0)) * &crate::galaxy::golden_turns()).frac(),
            swing: Q::ZERO,
            merged_at: None,
        })
        .collect();
    let (r_g, core_kg) = rg_units();
    let anchor = Dy::from_q(&g_mass) * Dy::from_q(&crate::package::Si::world_mass_kg()) / core_kg;
    let radius_rg = Dy::from_q(&r_m) / r_g;
    let settled = settle_about(
        embryos,
        anchor,
        &|a_r: &Q| Dy::from_q(a_r) * radius_rg,
        &settled_gyr(),
        &crate::life::horizon_gyr().to_q(),
    );
    let forced = forced_eccentricities(&settled, anchor, radius_rg, &g_mass);
    let rho = (&Q::ONE - ice_share) + (ice_share * &Q::ratio(55, 10));
    let icy = ice_share > &Q::ratio(1, 5);
    let mut placed = Q::ZERO;
    for (e, e_forced) in settled.into_iter().zip(forced) {
        if e.m < Q::ratio(1, 10_000) {
            continue;
        }
        let radius = extent(&(&e.m * &rho.recip().unwrap_or(Q::ONE)));
        if is_world(&e.m, &radius, icy) {
            let share = if icy { ice_share.clone() } else { Q::ZERO };
            let (layers, elements, mantle) = moon_matter(&e.m, gas, t_form, &share);
            placed = &placed + &e.m;
            worlds.push(World {
                a: g_a.clone(),
                mass: e.m,
                radius,
                giant: false,
                host: Some(parent),
                a_host_radii: Some(e.a),
                layers,
                elements,
                mantle,
                ices: g_ices.clone(),
                e_forced: Some(e_forced),
                swing: e.swing,
                phase: e.phase,
                merged_at: e.merged_at,
            });
        }
    }
    placed
}

/// Each moon's forced eccentricity, capped at 1/10: its neighbours' mass
/// over the giant's, divided by their rates' distance from the nearest
/// first-order resonance p / (p + 1) (floored at 1/1000).
fn forced_eccentricities(settled: &[Embryo], anchor: Dy, radius_rg: Dy, g_mass: &Q) -> Vec<Q> {
    let rate = |a_r: &Q| {
        let a = Dy::from_q(a_r) * radius_rg;
        (anchor / (a + anchor)) / a
    };
    (0..settled.len())
        .map(|k| {
            let mut f = Dy::ZERO;
            for j in [k.checked_sub(1), Some(k + 1)].into_iter().flatten() {
                let Some(nb) = settled.get(j) else { continue };
                let (w1, w2) = (rate(&settled[k].a), rate(&nb.a));
                let ratio = w1.min(w2) / w1.max(w2);
                let p = (ratio / (Dy::ONE - ratio).max(Dy::sci(1, -12)))
                    .trunc_i64()
                    .max(1);
                let near = [p - 1, p, p + 1]
                    .iter()
                    .filter(|&&x| x >= 1)
                    .map(|&x| (ratio - Dy::int(x) / Dy::int(x + 1)).abs())
                    .fold(Dy::ONE, Dy::min)
                    .max(Dy::sci(1, -3));
                f = f.max(Dy::from_q(&nb.m) / Dy::from_q(g_mass) / near);
            }
            f.min(Dy::ratio(1, 10)).to_q()
        })
        .collect()
}

/// A moon of mass `m` with ice share `share`: its layer shares, and its
/// whole and mantle element masses apportioned exactly from the gas as it
/// condenses at `t_form`.
fn moon_matter(m: &Q, gas: &Gas, t_form: &Q, share: &Q) -> ([Q; 4], Vec<Q>, Vec<Q>) {
    let (core_share, ices) = gas.solids(t_form);
    let rock = m * &(&Q::ONE - share);
    let ice_sum = ices.iter().fold(Q::ZERO, |a, (_, x)| &a + x);
    let ice_mass: Vec<(&str, Q)> = ices
        .iter()
        .map(|(sp, x)| {
            (
                *sp,
                (&(m * share) * x).checked_div(&ice_sum).unwrap_or(Q::ZERO),
            )
        })
        .collect();
    let (elems, mantle) = gas.apportion_layers(
        t_form,
        &(&rock * &core_share),
        &(&rock * &(&Q::ONE - &core_share)),
        &ice_mass,
        &Q::ZERO,
    );
    let layers = [
        &(&Q::ONE - share) * &core_share,
        &(&Q::ONE - share) * &(&Q::ONE - &core_share),
        share.clone(),
        Q::ZERO,
    ];
    (layers, elems, mantle)
}

/// The system's belts and comet cloud: an outer belt of the `ungathered`
/// solids beyond the outermost world, and the cloud the giants flung out,
/// kept in the share exp(-env) the stellar neighbourhood allows.
pub(super) fn belts(
    spec: &Spec,
    gas: &Gas,
    ungathered: &Q,
    a_out: &Q,
    orbits: &[(Q, bool)],
    giants_mass: &Q,
    env: &Q,
) -> (Vec<Belt>, Option<(Q, Vec<Q>)>) {
    let mut out = Vec::new();
    if orbits.is_empty() {
        return (out, None);
    }
    let rock_of = |t_form: &Q, mass: &Q| -> Vec<Q> {
        let (core_share, ices) = gas.solids(t_form);
        let ice_sum = ices.iter().fold(Q::ZERO, |a, (_, x)| &a + x);
        let rock = mass.checked_div(&(&Q::ONE + &ice_sum)).unwrap_or(Q::ZERO);
        let ice_mass: Vec<(&str, Q)> = ices.iter().map(|(sp, x)| (*sp, &rock * x)).collect();
        gas.apportion(
            t_form,
            &(&rock * &core_share),
            &(&rock * &(&Q::ONE - &core_share)),
            &ice_mass,
            &Q::ZERO,
        )
    };
    // beyond the outermost world, what the regions never gathered (the
    // manifold's remainder, disc::manifold_worlds) out to its reach
    if let Some(a) = orbits.iter().map(|(a, _)| a.clone()).max() {
        if ungathered.signum() > 0 && *a_out > a {
            let mid = (&a + a_out).checked_div(&Q::int(2)).unwrap_or(Q::ONE);
            out.push(Belt {
                elements: rock_of(&spec.t_form(&mid), ungathered),
                inner_a: a,
                outer_a: a_out.clone(),
                mass: ungathered.clone(),
            });
        }
    }
    // the comet cloud: 5% of a tenth of the giants' mass, kept in exp(-env)
    let comets = (giants_mass.signum() > 0).then(|| {
        let kept = fix(&exp(&-env.clone(), BITS));
        let mass = &(&(giants_mass * &Q::ratio(1, 10)) * &Q::ratio(5, 100)) * &kept;
        let far = orbits
            .iter()
            .map(|(a, _)| a.clone())
            .max()
            .unwrap_or(Q::ONE);
        let t = spec.t_form(&(&far * &Q::int(30)));
        (mass.clone(), rock_of(&t, &mass))
    });
    (out, comets)
}
