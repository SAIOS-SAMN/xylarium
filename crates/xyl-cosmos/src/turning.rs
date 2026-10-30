//! The turning law, one law at every scale (docs/model.md): curvature is the
//! law and the angle its reading. Units are the white hole's: G = c =
//! M_core = 1, lengths in r_g, time in r_g / c.
//!
//! - The deficit inside a circle of radius r is the share of the turn the
//!   enclosed mass withholds, the same share the well takes from the clock
//!   (clocks run at r / (r + M)): delta(<r) = M(<r) / (r + M(<r)), M and r in
//!   r_g, exact, at most 1.
//! - The turning law: a body at r turns about its anchor at
//!   omega = c delta(<r) / r, so its speed v = c delta stays below light. There
//!   is no separate ruler per tier.
//! - Twist is the deficit: once around, a carried direction returns turned by
//!   delta of a turn.
//! - Consequences: far from a point mass v / c goes as r_g / r, and a year as
//!   r (r + M) / M.
//!
//! At its interfaces the code uses SI and Gyr: the rate is converted to
//! degrees per Gyr, pi enters only when converting a speed into a turn, and
//! the rate is rounded once to a 2^-60 grid.

use crate::package::Si;
use xyl_core::dyadic::Dy;
use xyl_core::Q;

fn d(s: &str) -> Q {
    crate::elements::decimal(s).unwrap_or(Q::ZERO)
}

/// The home galaxy's mass (suns): its stars (the founding's lit rings,
/// galaxy::stellar_mass) and its hole.
pub fn home_mass() -> Q {
    &crate::galaxy::stellar_mass() + &crate::package::hole_mass()
}

/// The core's mass (kg): the unit of mass, M_core = 1.
fn core_kg() -> Q {
    &crate::package::hole_mass() * &Si::star_mass_kg()
}

/// The deficit: the share of the turn `inside` withholds at `r`, both in r_g:
/// M / (r + M), exact.
pub fn deficit_share(inside_rg: &Q, r_rg: &Q) -> Q {
    inside_rg
        .checked_div(&(r_rg + inside_rg))
        .unwrap_or(Q::ZERO)
}

/// The deficit of `inside_kg` at `r_m`, in the white hole's units.
fn deficit_si(inside_kg: &Q, r_m: &Q) -> Q {
    let units = crate::chain::units();
    let m_rg = inside_kg.checked_div(&core_kg()).unwrap_or(Q::ZERO);
    let r_rg = r_m.checked_div(&units.r_g_m).unwrap_or(Q::ZERO);
    deficit_share(&m_rg, &r_rg)
}

/// The turning law at `r_m` metres about `inside_kg` kilograms:
/// omega = c delta / r, in degrees per Gyr (180 / pi per radian), rounded to
/// the grid.
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

/// The turning in a galaxy at `r_kpc` with `inside_mass` inside.
pub fn turning_kpc(inside_mass: &Q, r_kpc: &Q) -> Q {
    turning_si(&(inside_mass * &Si::star_mass_kg()), &(r_kpc * &Si::kpc()))
}

/// The turning law in dyadics (degrees per Gyr) at `r_kpc` about
/// `inside_mass`: omega = c delta / r, delta = M / (r + M) in r_g. Same law as
/// `turning_kpc`, for fast per-pair evaluation.
pub fn turning_kpc_dy(inside_mass: Dy, r_kpc: Dy) -> Dy {
    // [suns per core mass, r_g per kpc, c / kpc in degrees per Gyr]
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

/// The same rate in radians per second, for the tidal and thermal laws not
/// yet written in angles; pi enters here.
pub fn rad_s(deg_per_gyr: &Q) -> Q {
    let pi = xyl_core::certified::pi(64).mid();
    (deg_per_gyr * &pi)
        .checked_div(&(&Q::int(180) * &Si::gyr_s()))
        .unwrap_or(Q::ZERO)
}

/// A world's year (standard years) at `a_au` about `star_suns`: one turn at
/// the law's rate.
pub fn year_yr(star_suns: &Q, a_au: &Q) -> Q {
    let w = turning_si(&(star_suns * &Si::star_mass_kg()), &(a_au * &Si::au()));
    (&Q::int(360) * &Q::int(1_000_000_000))
        .checked_div(&w)
        .unwrap_or(Q::ZERO)
}

/// Twist is the deficit: degrees a carried direction turns once around
/// `inside_kg` at `r_m`.
pub fn twist_per_turn(inside_kg: &Q, r_m: &Q) -> Q {
    &Q::int(360) * &deficit_si(inside_kg, r_m)
}

/// Roundness as an angle ratio: the turn a body's curvature gives at its
/// surface over its material's rigidity (the same turn at the material's
/// round-or-rubble boundary). At or above 1 the body is round. Provisional:
/// boundary bodies use measured reference values until bond grip replaces
/// them.
pub fn roundness(mass_kg: &Q, radius_m: &Q, icy: bool) -> Q {
    turning_si(mass_kg, radius_m)
        .checked_div(&rigidity(icy))
        .unwrap_or(Q::ZERO)
}

/// A material's rigidity (the surface turn of its boundary body): 400 km of
/// rock (8.04e20 kg) or 200 km of ice (3.35e19 kg).
pub fn rigidity(icy: bool) -> Q {
    static R: std::sync::OnceLock<[Q; 2]> = std::sync::OnceLock::new();
    let both = R.get_or_init(|| {
        [(d("8.04e20"), d("4.0e5")), (d("3.35e19"), d("2.0e5"))].map(|(m, r)| turning_si(&m, &r))
    });
    both[usize::from(icy)].clone()
}
