//! The turning law (docs/model.md): the rate at which a body turns about
//! its anchor, from the deficit of the mass inside its radius.
//!
//! Units: G = c = M_core = 1. Lengths are in a tier's unit
//! (`xyl_core::units::Tier`) and converted to r_g of the universal core by
//! `Tier::in_rg`; masses are in core masses (the universal core is 1); the
//! bell is `circuit::anchor().bell`, in units of r_g / c; rates are radians
//! per ring (one ring one bell).
//!
//! - Deficit: `xyl_core::shares::deficit` = M(<r) / (r + M(<r)), M and r in
//!   r_g, exact, at most 1: gravity's convergent face (`shares::faces`),
//!   whose antipole, the divergent face r / (r + M), is the rate at which
//!   the well's clocks run.
//! - Turning ([`turning`] exact, [`turning_dy`] on readings): `omega =
//!   delta(<r) bell / r` radians per ring, with `r` in r_g and
//!   `delta = M / (r + M)`. With c = 1 the speed `omega r / bell = delta`
//!   stays below light; the same law applies at every tier. Far from a point
//!   mass `delta` goes as M / r. These are the law's two forms, one per
//!   carrier; every other rate reads one of them.
//! - Twist: once around, a carried direction turns by delta of a turn
//!   (`twist_per_turn`, degrees).
//!
//! Boundary (removed as the system, body and world layers are converted):
//! `turning_si`, `year`, `twist_per_turn`, `roundness` and `rigidity` take
//! kilograms and metres or suns and AU and convert them to core masses and
//! r_g through the SI seam (`package::Si`, `package::core_kg`), then evaluate the
//! same law; a rate in radians per SI second is `package::Si::rad_s` of
//! `turning_si`. `span_in_rings` converts rulers declared in Gyr for the
//! chemistry through `circuit::rings_of_gyr` until it is converted.
//! `roundness` / `rigidity` are provisional (measured rock and ice boundary
//! bodies).
//! Non-positive radii and zero denominators give zero, and so does every
//! rate before a founding (the ring is the founding's bell).
//! Terms: turning law, deficit (docs/glossary.md).

use crate::elements::decimal;
use crate::package::Si;
use xyl_core::dyadic::Dy;
use xyl_core::shares::{deficit, deficit_dy};
use xyl_core::units::Tier;
use xyl_core::Q;

/// The home galaxy's mass in core masses: the stellar mass of the
/// founding's lit rings (`galaxy::stellar_mass`) plus the universal core, 1.
pub fn home_mass() -> Q {
    &crate::galaxy::stellar_mass() + &Q::ONE
}

/// The turning law in r_g: `omega = delta bell / r` radians per ring about
/// `inside` core masses at `r_rg` r_g, `delta = M / (r + M)` (the deficit
/// share), `bell` the ring in units of r_g / c (`circuit::anchor`). A
/// non-positive radius, or no founding, gives zero.
fn turning_rg(inside: &Q, r_rg: &Q) -> Q {
    if r_rg.signum() <= 0 {
        return Q::ZERO;
    }
    let delta = deficit(&inside.clone().max(Q::ZERO), r_rg).unwrap_or(Q::ZERO);
    (&delta * &crate::circuit::anchor().bell)
        .checked_div(r_rg)
        .unwrap_or(Q::ZERO)
}

/// The turning law, exact: the rate in radians per ring of a body at `r`
/// units of `tier` about `inside` core masses, `omega = delta bell / r` with
/// `r` read in r_g (`r Tier::in_rg`), `delta = M / (r + M)` the deficit
/// share and `bell` the ring in r_g / c (`circuit::anchor().bell`). A
/// non-positive radius, or no founding, gives zero.
///
/// # Examples
///
/// No body turns at a non-positive radius:
///
/// ```
/// use xyl_core::units::Tier;
/// use xyl_core::Q;
/// assert_eq!(xyl_cosmos::turning::turning(&Q::ONE, &Q::ZERO, Tier::Galaxy), Q::ZERO);
/// ```
pub fn turning(inside: &Q, r: &Q, tier: Tier) -> Q {
    turning_rg(inside, &(r * &tier.in_rg()))
}

/// The turning law in dyadics, for per-pair evaluation: radians per ring at
/// `r` units of `tier` about `inside` core masses, `omega = delta bell / r`
/// as in [`turning`], the deficit read by `xyl_core::shares::deficit_dy`. A
/// non-positive radius, or no founding, gives zero.
pub fn turning_dy(inside: Dy, r: Dy, tier: Tier) -> Dy {
    // The bell in r_g / c, kept once the founding has set it.
    static BELL: std::sync::OnceLock<Dy> = std::sync::OnceLock::new();
    let bell = match BELL.get() {
        Some(b) => *b,
        None => {
            let b = Dy::from_q(&crate::circuit::anchor().bell);
            if b.signum() > 0 {
                let _ = BELL.set(b);
            }
            b
        }
    };
    if r.signum() <= 0 {
        return Dy::ZERO;
    }
    let r_rg = r * Dy::from_q(&tier.in_rg());
    (bell * deficit_dy(inside, r_rg))
        .div(&r_rg)
        .unwrap_or_default()
}

/// `inside_kg` kilograms and `r_m` metres in core masses and r_g (boundary).
fn core_units_of_si(inside_kg: &Q, r_m: &Q) -> (Q, Q) {
    let units = crate::chain::units();
    (
        inside_kg
            .checked_div(&crate::package::core_kg())
            .unwrap_or(Q::ZERO),
        r_m.checked_div(&units.r_g_m).unwrap_or(Q::ZERO),
    )
}

/// The deficit share of `inside_kg` at `r_m` metres, after converting both to
/// the core's units (r_g). Boundary.
fn deficit_si(inside_kg: &Q, r_m: &Q) -> Q {
    let (m, r) = core_units_of_si(inside_kg, r_m);
    deficit(&m, &r).unwrap_or(Q::ZERO)
}

/// The turning law at `r_m` metres about `inside_kg` kilograms in degrees
/// per ring: [`turning`] in core units times 180 / pi, rounded down to the
/// 2^-60 grid. Boundary, for the system and body layers not yet in core
/// units; `package::Si::rad_s` reads it in radians per SI second. A
/// non-positive radius, or no founding, gives zero.
pub fn turning_si(inside_kg: &Q, r_m: &Q) -> Q {
    let (m, r) = core_units_of_si(inside_kg, r_m);
    let pi = xyl_core::certified::pi(80).mid();
    let deg_ring = (&turning_rg(&m, &r) * &Q::int(180))
        .checked_div(&pi)
        .unwrap_or(Q::ZERO);
    xyl_core::certified::down(&deg_ring, 60)
}

/// A world's year (rings): one turn (360 degrees) at `a_au` about
/// `star_suns` at the turning law's rate, read against the ring. A zero rate
/// gives zero.
pub fn year(star_suns: &Q, a_au: &Q) -> Q {
    let w = turning_si(&(star_suns * &Si::star_mass_kg()), &(a_au * &Si::au()));
    Q::int(360).checked_div(&w).unwrap_or(Q::ZERO)
}

/// A span declared in Gyr (a measured ruler), in rings
/// (`circuit::rings_of_gyr`), held to 62 bits: converted once, where the
/// ruler is declared. Zero before a founding.
pub fn span_in_rings(span: &Q) -> Q {
    Dy::from_q(&crate::circuit::rings_of_gyr(span)).to_q()
}

/// The twist in degrees: the angle by which a carried direction turns over
/// one circuit about `inside_kg` at `r_m`, equal to 360 times the deficit.
pub fn twist_per_turn(inside_kg: &Q, r_m: &Q) -> Q {
    // Stand-in S130: The turn is read on one plane, not per dipole.
    &Q::int(360) * &deficit_si(inside_kg, r_m)
}

/// Roundness as a ratio of turning rates: the surface turning rate of a body
/// of `mass_kg` and `radius_m` over the material's rigidity (the surface
/// turning rate of the boundary body between round and irregular). At or
/// above 1 the body is round. Provisional: the boundary bodies use measured
/// reference values until bond strength replaces them.
pub fn roundness(mass_kg: &Q, radius_m: &Q, icy: bool) -> Q {
    turning_si(mass_kg, radius_m)
        .checked_div(&rigidity(icy))
        .unwrap_or(Q::ZERO)
}

/// A material's rigidity, the surface turning rate of its boundary body: a
/// rock body of radius 400 km (8.04e20 kg) or an ice body of radius 200 km
/// (3.35e19 kg).
pub fn rigidity(icy: bool) -> Q {
    // Kept once the founding has set the ring (before it every rate is zero).
    static R: std::sync::OnceLock<[Q; 2]> = std::sync::OnceLock::new();
    if let Some(both) = R.get() {
        return both[usize::from(icy)].clone();
    }
    let both = [
        (
            decimal("8.04e20").unwrap_or(Q::ZERO),
            decimal("4.0e5").unwrap_or(Q::ZERO),
        ),
        (
            decimal("3.35e19").unwrap_or(Q::ZERO),
            decimal("2.0e5").unwrap_or(Q::ZERO),
        ),
    ]
    .map(|(m, r)| turning_si(&m, &r));
    if both.iter().all(|w| w.signum() > 0) {
        let _ = R.set(both.clone());
    }
    both[usize::from(icy)].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t2_is_the_face_law_at_the_horizon() {
        // With the torsion share the divergent face at t_s = 1 + a / 2 (M = 1):
        // D(s) = 1 + 2 t_s = 3 + a, the twist n = d - c = a / (4 + a), and
        // D(n) = t_s = 1 + a / 2 (T2).
        for a in [
            Q::ZERO,
            xyl_core::ratio!(1, 3),
            xyl_core::ratio!(2, 5),
            Q::ONE,
        ] {
            let s = crate::bounce::torsion_share(&Q::ONE, &a);
            let f =
                xyl_core::shares::faces(&Q::ONE, &(&Q::ONE + &a.checked_div(&Q::int(2)).unwrap()))
                    .unwrap();
            assert_eq!(f.divergent, s);
            assert_eq!(xyl_core::shares::dilation(&s), Some(&Q::int(3) + &a));
            let n = -f.push();
            assert_eq!(n, a.checked_div(&(&Q::int(4) + &a)).unwrap());
            assert_eq!(xyl_core::shares::dilation(&n), f.pole());
        }
    }

    #[test]
    fn t14s_plane_push_is_a_divergent_face() {
        // T14: plane j is pushed out at J_j / (6 H_j + J_j), the divergent
        // face about the hold 6 H_j at reach J_j; its dilation 1 + J_j / (3 H_j).
        for (h, j) in [
            (xyl_core::ratio!(1, 5), xyl_core::ratio!(1, 10)),
            (xyl_core::ratio!(3, 7), Q::ONE),
        ] {
            let six_h = &Q::int(6) * &h;
            let push = j.checked_div(&(&six_h + &j)).unwrap();
            let f = xyl_core::shares::faces(&six_h, &j).unwrap();
            assert_eq!(f.divergent, push);
            assert_eq!(
                xyl_core::shares::dilation(&push),
                Some(&Q::ONE + &j.checked_div(&(&Q::int(3) * &h)).unwrap())
            );
        }
    }

    #[test]
    fn the_torsion_share_is_gravitys_divergent_face() {
        // Axiom 7, s = (H + J) / (2H + J), is the divergent face about the
        // horizon's hold H at reach H + J, its pole t = 1 + J / H.
        for (m, a) in [
            (Q::ONE, Q::ZERO),
            (Q::ONE, xyl_core::ratio!(1, 3)),
            (xyl_core::ratio!(5, 2), xyl_core::ratio!(7, 9)),
            (xyl_core::ratio!(1, 4), Q::ONE),
        ] {
            let h = crate::bounce::horizon_reach(&m);
            let j = &a * &(&m * &m);
            let f = xyl_core::shares::faces(&h, &(&h + &j)).unwrap();
            assert_eq!(crate::bounce::torsion_share(&m, &a), f.divergent);
            assert_eq!(f.pole(), Some(&Q::ONE + &j.checked_div(&h).unwrap()));
        }
    }

    #[test]
    fn turning_stays_below_light_and_falls_outward() {
        let _ = crate::genesis::origin();
        let bell = crate::circuit::anchor().bell;
        assert!(bell.signum() > 0, "the founding sets the bell");
        let m = xyl_core::ratio!(3, 2);
        let mut last: Option<Q> = None;
        for k in 1..=12 {
            let r = xyl_core::ratio!(k, 4);
            let w = turning(&m, &r, Tier::Galaxy);
            // The speed `omega r / bell` is the deficit share, below light.
            let r_rg = &r * &Tier::Galaxy.in_rg();
            let v = (&w * &r_rg).checked_div(&bell).unwrap();
            assert_eq!(Some(v.clone()), deficit(&m, &r_rg));
            assert!(v > Q::ZERO && v < Q::ONE);
            // The rate falls outward about a fixed mass.
            if let Some(prev) = last {
                assert!(w < prev);
            }
            last = Some(w);
        }
        assert_eq!(turning(&m, &Q::ZERO, Tier::Galaxy), Q::ZERO);
    }

    #[test]
    fn dyadic_and_si_forms_hold_the_core_law() {
        let _ = crate::genesis::origin();
        let (m, r) = (xyl_core::ratio!(5, 4), xyl_core::ratio!(7, 3));
        let exact = turning(&m, &r, Tier::Galaxy);
        let dy = turning_dy(Dy::from_q(&m), Dy::from_q(&r), Tier::Galaxy);
        let gap = (&dy.to_q() - &exact).abs();
        assert!(gap <= &exact * &xyl_core::ratio!(1, 1 << 40), "{gap}");
        // The SI boundary reads the same law: degrees per ring over 180 / pi.
        let units = crate::chain::units();
        let kg = &m * &crate::package::core_kg();
        let metres = &(&r * &Tier::Galaxy.in_rg()) * &units.r_g_m;
        let pi = xyl_core::certified::pi(80).mid();
        let si = turning_si(&kg, &metres);
        let back = (&si * &pi).checked_div(&Q::int(180)).unwrap();
        let gap = (&back - &exact).abs();
        assert!(gap <= &exact * &xyl_core::ratio!(1, 1 << 40), "{gap}");
    }
}
