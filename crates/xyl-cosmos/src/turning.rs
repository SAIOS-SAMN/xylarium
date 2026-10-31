//! The turning law (docs/model.md): the rate at which a body turns about
//! its anchor, from the deficit of the mass inside its radius.
//!
//! Units: G = c = M_core = 1. Lengths are in a tier's unit
//! (`xyl_core::units::Tier`) and converted to r_g of the universal core by
//! `Tier::in_rg`; masses are in core masses (the universal core is 1); the
//! bell is `circuit::anchor().bell`, in units of r_g / c; rates are radians
//! per ring (one ring one bell).
//!
//! - Deficit: `deficit_share` = M(<r) / (r + M(<r)), M and r in r_g, exact,
//!   at most 1; the same share by which the well slows clocks.
//! - Turning ([`turning`], [`turning_dy`]): `omega = delta(<r) bell / r`
//!   radians per ring, with `r` in r_g and `delta = M / (r + M)`. With
//!   c = 1 the speed `omega r / bell = delta` stays below light; the same law
//!   applies at every tier. Far from a point mass `delta` goes as M / r.
//! - Twist: once around, a carried direction turns by delta of a turn
//!   (`twist_per_turn`, degrees).
//!
//! Boundary (removed as the system, body and world layers are converted):
//! `turning_si`, `turning_rad_s`, `rad_s`, `year`, `twist_per_turn`,
//! `roundness` and `rigidity` take kilograms and metres or suns and AU and
//! convert them to core masses and r_g through the SI seam
//! (`package::Si`, `package::hole_mass`), then evaluate the same law;
//! `span_in_rings` and `rate_per_ring` convert rulers declared in Gyr for
//! the chemistry until it is converted. `roundness` / `rigidity` are
//! provisional (measured rock and ice boundary bodies).
//! Non-positive radii and zero denominators give zero, and so does every
//! rate before a founding (the ring is the founding's bell).
//! Terms: turning law, deficit (docs/glossary.md).

use crate::elements::decimal;
use crate::package::Si;
use xyl_core::dyadic::Dy;
use xyl_core::shares::deficit;
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
/// as in [`turning`]. A non-positive radius, or no founding, gives zero.
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
    let (m, r_rg) = (inside.max(Dy::ZERO), r * Dy::from_q(&tier.in_rg()));
    bell * (m / (r_rg + m)) / r_rg
}

/// The core's mass in kg, the SI seam's reading of one core mass
/// (boundary).
fn core_kg() -> Q {
    &crate::package::hole_mass() * &Si::star_mass_kg()
}

/// `inside_kg` kilograms and `r_m` metres in core masses and r_g (boundary).
fn core_units_of_si(inside_kg: &Q, r_m: &Q) -> (Q, Q) {
    let units = crate::chain::units();
    (
        inside_kg.checked_div(&core_kg()).unwrap_or(Q::ZERO),
        r_m.checked_div(&units.r_g_m).unwrap_or(Q::ZERO),
    )
}

/// The deficit share of `inside_kg` at `r_m` metres, after converting both to
/// the core's units (r_g). Boundary.
fn deficit_si(inside_kg: &Q, r_m: &Q) -> Q {
    let (m, r) = core_units_of_si(inside_kg, r_m);
    deficit(&m, &r).unwrap_or(Q::ZERO)
}

/// The turning law at `r_m` metres about `inside_kg` kilograms in radians
/// per SI second, `omega = c delta / r` with the deficit share read in core
/// units: the same law as [`turning`], written with the SI seam's `c` so it
/// needs no founding. Boundary, for the tidal and thermal laws not yet
/// written in core units. A non-positive radius gives zero.
pub fn turning_rad_s(inside_kg: &Q, r_m: &Q) -> Q {
    if r_m.signum() <= 0 {
        return Q::ZERO;
    }
    (&Si::c() * &deficit_si(inside_kg, r_m))
        .checked_div(r_m)
        .unwrap_or(Q::ZERO)
}

/// The turning law at `r_m` metres about `inside_kg` kilograms in degrees
/// per ring: [`turning`] in core units times 180 / pi, rounded down to the
/// 2^-60 grid. Boundary, for the system and body layers not yet in core
/// units. A non-positive radius, or no founding, gives zero.
pub fn turning_si(inside_kg: &Q, r_m: &Q) -> Q {
    let (m, r) = core_units_of_si(inside_kg, r_m);
    let pi = xyl_core::certified::pi(80).mid();
    let deg_ring = (&turning_rg(&m, &r) * &Q::int(180))
        .checked_div(&pi)
        .unwrap_or(Q::ZERO);
    xyl_core::certified::down(&deg_ring, 60)
}

/// Converts a rate in degrees per ring to radians per SI second, for the
/// tidal and thermal laws not yet written in angles; pi enters only here, and
/// the SI second only through `Si::seconds_per_ring`. Zero before a founding.
pub fn rad_s(deg_per_ring: &Q) -> Q {
    let pi = xyl_core::certified::pi(64).mid();
    (deg_per_ring * &pi)
        .checked_div(&(&Q::int(180) * &Si::seconds_per_ring()))
        .unwrap_or(Q::ZERO)
}

/// A world's year (rings): one turn (360 degrees) at `a_au` about
/// `star_suns` at the turning law's rate, read against the ring. A zero rate
/// gives zero.
pub fn year(star_suns: &Q, a_au: &Q) -> Q {
    let w = turning_si(&(star_suns * &Si::star_mass_kg()), &(a_au * &Si::au()));
    Q::int(360).checked_div(&w).unwrap_or(Q::ZERO)
}

/// Rings per Gyr: the SI seam's Gyr (`Si::gyr_s`) over its seconds per ring
/// (`Si::seconds_per_ring`), held to 62 bits and kept once a universe is
/// founded; zero before. The one place a ruler declared in Gyr meets the
/// ring.
fn rings_per_declared() -> Q {
    static R: std::sync::OnceLock<Q> = std::sync::OnceLock::new();
    if let Some(r) = R.get() {
        return r.clone();
    }
    let r = Si::gyr_s()
        .checked_div(&Si::seconds_per_ring())
        .map(|r| Dy::from_q(&r).to_q())
        .unwrap_or(Q::ZERO);
    if r.signum() > 0 {
        let _ = R.set(r.clone());
    }
    r
}

/// A span declared in Gyr (a measured ruler), in rings, held to 62 bits:
/// converted once, where the ruler is declared. Zero before a founding.
pub fn span_in_rings(span: &Q) -> Q {
    Dy::from_q(&(span * &rings_per_declared())).to_q()
}

/// A rate declared per Gyr (a measured ruler), per ring, held to 62 bits:
/// converted once, where the ruler is declared. Zero before a founding.
pub fn rate_per_ring(rate: &Q) -> Q {
    rate.checked_div(&rings_per_declared())
        .map(|r| Dy::from_q(&r).to_q())
        .unwrap_or(Q::ZERO)
}

/// The twist in degrees: the angle by which a carried direction turns over
/// one circuit about `inside_kg` at `r_m`, equal to 360 times the deficit.
pub fn twist_per_turn(inside_kg: &Q, r_m: &Q) -> Q {
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
    fn turning_stays_below_light_and_falls_outward() {
        let _ = crate::genesis::origin();
        let bell = crate::circuit::anchor().bell;
        assert!(bell.signum() > 0, "the founding sets the bell");
        let m = Q::ratio(3, 2);
        let mut last: Option<Q> = None;
        for k in 1..=12 {
            let r = Q::ratio(k, 4);
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
        let (m, r) = (Q::ratio(5, 4), Q::ratio(7, 3));
        let exact = turning(&m, &r, Tier::Galaxy);
        let dy = turning_dy(Dy::from_q(&m), Dy::from_q(&r), Tier::Galaxy);
        let gap = (&dy.to_q() - &exact).abs();
        assert!(gap <= &exact * &Q::ratio(1, 1 << 40), "{gap}");
        // The SI boundary reads the same law: degrees per ring over 180 / pi.
        let units = crate::chain::units();
        let kg = &m * &core_kg();
        let metres = &(&r * &Tier::Galaxy.in_rg()) * &units.r_g_m;
        let pi = xyl_core::certified::pi(80).mid();
        let si = turning_si(&kg, &metres);
        let back = (&si * &pi).checked_div(&Q::int(180)).unwrap();
        let gap = (&back - &exact).abs();
        assert!(gap <= &exact * &Q::ratio(1, 1 << 40), "{gap}");
    }
}
