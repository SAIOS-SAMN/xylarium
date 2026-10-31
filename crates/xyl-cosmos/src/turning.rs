//! The turning law (docs/model.md): the rate at which a body turns about
//! its anchor, from the deficit of the mass inside its radius.
//!
//! Units: G = c = M_core = 1, lengths in r_g of the core (`chain::units`);
//! inputs at the interfaces are SI or kpc and suns (`package::Si`,
//! `package::hole_mass`), outputs degrees per Gyr.
//!
//! - Deficit: `deficit_share` = M(<r) / (r + M(<r)), M and r in r_g, exact,
//!   at most 1; the same share by which the well slows clocks.
//! - Turning: omega = c delta(<r) / r, so v = c delta stays below c; the
//!   same law applies at every tier. Far from a point mass v / c goes as r_g / r.
//! - Twist: once around, a carried direction turns by delta of a turn
//!   (`twist_per_turn`, degrees).
//!
//! Outputs: `turning_si`, `turning_kpc` (exact, rounded down once to the
//! 2^-60 grid), `turning_kpc_dy` (the same law in `Dy`), `rad_s` (pi enters
//! only in converting between turns and radians), `year_yr`, `home_mass`,
//! and `roundness` / `rigidity` (provisional: measured rock and ice
//! boundary bodies). Non-positive radii and zero denominators give zero.
//! Terms: turning law, deficit (docs/glossary.md).

use crate::elements::decimal;
use crate::package::Si;
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// The home galaxy's mass in suns: the stellar mass of the founding's lit
/// rings (`galaxy::stellar_mass`) plus the mass of the central hole.
pub fn home_mass() -> Q {
    &crate::galaxy::stellar_mass() + &crate::package::hole_mass()
}

/// The core's mass in kg, which is the unit of mass (M_core = 1).
fn core_kg() -> Q {
    &crate::package::hole_mass() * &Si::star_mass_kg()
}

/// The deficit share `M / (r + M)` of the mass `inside_rg` at radius `r_rg`,
/// both in r_g, exact. A zero denominator gives zero.
pub fn deficit_share(inside_rg: &Q, r_rg: &Q) -> Q {
    inside_rg
        .checked_div(&(r_rg + inside_rg))
        .unwrap_or(Q::ZERO)
}

/// The deficit share of `inside_kg` at `r_m` metres, after converting both to
/// the core's units (r_g).
fn deficit_si(inside_kg: &Q, r_m: &Q) -> Q {
    let units = crate::chain::units();
    let m_rg = inside_kg.checked_div(&core_kg()).unwrap_or(Q::ZERO);
    let r_rg = r_m.checked_div(&units.r_g_m).unwrap_or(Q::ZERO);
    deficit_share(&m_rg, &r_rg)
}

/// The turning law at `r_m` metres about `inside_kg` kilograms:
/// omega = c delta / r, in degrees per Gyr (180 / pi per radian), rounded down
/// to the 2^-60 grid. A non-positive radius gives zero.
pub fn turning_si(inside_kg: &Q, r_m: &Q) -> Q {
    if r_m.signum() <= 0 {
        return Q::ZERO;
    }
    let c = Si::c();
    let rad_s = (&c * &deficit_si(inside_kg, r_m))
        .checked_div(r_m)
        .unwrap_or(Q::ZERO);
    let pi = xyl_core::certified::pi(80).mid();
    let deg_gyr = (&(&rad_s * &Si::gyr_s()) * &Q::int(180))
        .checked_div(&pi)
        .unwrap_or(Q::ZERO);
    xyl_core::certified::down(&deg_gyr, 60)
}

/// The turning rate in degrees per Gyr at `r_kpc` kiloparsecs about
/// `inside_mass` suns; `turning_si` after unit conversion.
pub fn turning_kpc(inside_mass: &Q, r_kpc: &Q) -> Q {
    turning_si(&(inside_mass * &Si::star_mass_kg()), &(r_kpc * &Si::kpc()))
}

/// The turning law in dyadics (degrees per Gyr) at `r_kpc` about
/// `inside_mass`: omega = c delta / r, delta = M / (r + M) in r_g. Same law as
/// `turning_kpc`, for fast per-pair evaluation.
pub fn turning_kpc_dy(inside_mass: Dy, r_kpc: Dy) -> Dy {
    // [core masses per sun, r_g per kpc, c / kpc in degrees per Gyr].
    static K: std::sync::OnceLock<[Dy; 3]> = std::sync::OnceLock::new();
    let [per_core, rg_per_kpc, c_deg] = *K.get_or_init(|| {
        let units = crate::chain::units();
        let kpc_m = Si::kpc();
        let pi = xyl_core::certified::pi(64).mid();
        let c_deg = (&(&Si::c() * &Si::gyr_s()) * &Q::int(180))
            .checked_div(&(&kpc_m * &pi))
            .unwrap_or(Q::ZERO);
        [
            Dy::ONE / Dy::from_q(&crate::package::hole_mass()),
            Dy::from_q(&kpc_m.checked_div(&units.r_g_m).unwrap_or(Q::ZERO)),
            Dy::from_q(&c_deg),
        ]
    });
    if r_kpc.signum() <= 0 {
        return Dy::ZERO;
    }
    let (m, r) = (inside_mass.max(Dy::ZERO) * per_core, r_kpc * rg_per_kpc);
    c_deg * (m / (r + m)) / r_kpc
}

/// Converts a rate in degrees per Gyr to radians per second, for the tidal
/// and thermal laws not yet written in angles; pi enters only here.
pub fn rad_s(deg_per_gyr: &Q) -> Q {
    let pi = xyl_core::certified::pi(64).mid();
    (deg_per_gyr * &pi)
        .checked_div(&(&Q::int(180) * &Si::gyr_s()))
        .unwrap_or(Q::ZERO)
}

/// A world's year in standard years at `a_au` about `star_suns`: the time for
/// one turn (360 degrees) at the turning law's rate. A zero rate gives zero.
pub fn year_yr(star_suns: &Q, a_au: &Q) -> Q {
    let w = turning_si(&(star_suns * &Si::star_mass_kg()), &(a_au * &Si::au()));
    (&Q::int(360) * &Q::int(1_000_000_000))
        .checked_div(&w)
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
    static R: std::sync::OnceLock<[Q; 2]> = std::sync::OnceLock::new();
    let both = R.get_or_init(|| {
        [
            (
                decimal("8.04e20").unwrap_or(Q::ZERO),
                decimal("4.0e5").unwrap_or(Q::ZERO),
            ),
            (
                decimal("3.35e19").unwrap_or(Q::ZERO),
                decimal("2.0e5").unwrap_or(Q::ZERO),
            ),
        ]
        .map(|(m, r)| turning_si(&m, &r))
    });
    both[usize::from(icy)].clone()
}
