//! The turning law (docs/model.md): the rate at which a body turns about
//! its anchor, from the deficit of the mass inside its radius.
//!
//! Units: G = c = M_core = 1, lengths in r_g of the core (`chain::units`);
//! inputs at the interfaces are SI or kpc and suns (`package::Si`,
//! `package::hole_mass`), outputs degrees per ring (one ring one bell, the
//! engine's unit of time; `circuit::anchor`). SI seconds enter only through
//! the SI seam, `package::Si::seconds_per_ring`.
//!
//! - Deficit: `deficit_share` = M(<r) / (r + M(<r)), M and r in r_g, exact,
//!   at most 1; the same share by which the well slows clocks.
//! - Turning: omega = c delta(<r) / r, so v = c delta stays below c; the
//!   same law applies at every tier. Far from a point mass v / c goes as r_g / r.
//! - Twist: once around, a carried direction turns by delta of a turn
//!   (`twist_per_turn`, degrees).
//!
//! Outputs: `turning_rad_s` (the law in radians per SI second, at the SI
//! seam), `turning_si`, `turning_kpc` (exact, rounded down once to the
//! 2^-60 grid), `turning_kpc_dy` (the same law in `Dy`), `rad_s` (pi enters
//! only in converting between turns and radians), `year` (rings per turn),
//! `home_mass`, `roundness` / `rigidity` (provisional: measured rock and ice
//! boundary bodies), and `span_in_rings` / `rate_per_ring`, which convert a
//! ruler declared in Gyr or per Gyr once, where it is declared.
//! Non-positive radii and zero denominators give zero, and so does every
//! rate before a founding (the ring is the founding's bell).
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

/// The turning law at `r_m` metres about `inside_kg` kilograms in radians
/// per SI second, omega = c delta / r: the SI seam's form, for the tidal and
/// thermal laws not yet written in angles; it needs no founding. A
/// non-positive radius gives zero.
pub fn turning_rad_s(inside_kg: &Q, r_m: &Q) -> Q {
    if r_m.signum() <= 0 {
        return Q::ZERO;
    }
    (&Si::c() * &deficit_si(inside_kg, r_m))
        .checked_div(r_m)
        .unwrap_or(Q::ZERO)
}

/// The turning law at `r_m` metres about `inside_kg` kilograms:
/// omega = c delta / r, in degrees per ring (180 / pi per radian, the SI
/// seconds of a ring from `Si::seconds_per_ring`), rounded down to the 2^-60
/// grid. A non-positive radius, or no founding, gives zero.
pub fn turning_si(inside_kg: &Q, r_m: &Q) -> Q {
    let rad_s = turning_rad_s(inside_kg, r_m);
    let pi = xyl_core::certified::pi(80).mid();
    let deg_ring = (&(&rad_s * &Si::seconds_per_ring()) * &Q::int(180))
        .checked_div(&pi)
        .unwrap_or(Q::ZERO);
    xyl_core::certified::down(&deg_ring, 60)
}

/// The turning rate in degrees per ring at `r_kpc` kiloparsecs about
/// `inside_mass` suns; `turning_si` after unit conversion.
pub fn turning_kpc(inside_mass: &Q, r_kpc: &Q) -> Q {
    turning_si(&(inside_mass * &Si::star_mass_kg()), &(r_kpc * &Si::kpc()))
}

/// The turning law in dyadics (degrees per ring) at `r_kpc` about
/// `inside_mass`: omega = c delta / r, delta = M / (r + M) in r_g. Same law as
/// `turning_kpc`, for fast per-pair evaluation. Zero before a founding.
pub fn turning_kpc_dy(inside_mass: Dy, r_kpc: Dy) -> Dy {
    // [core masses per sun, r_g per kpc, c / kpc in degrees per ring], kept
    // once the founding has set the ring.
    static K: std::sync::OnceLock<[Dy; 3]> = std::sync::OnceLock::new();
    let [per_core, rg_per_kpc, c_deg] = match K.get() {
        Some(k) => *k,
        None => {
            let units = crate::chain::units();
            let kpc_m = Si::kpc();
            let pi = xyl_core::certified::pi(64).mid();
            let c_deg = (&(&Si::c() * &Si::seconds_per_ring()) * &Q::int(180))
                .checked_div(&(&kpc_m * &pi))
                .unwrap_or(Q::ZERO);
            let k = [
                Dy::ONE / Dy::from_q(&crate::package::hole_mass()),
                Dy::from_q(&kpc_m.checked_div(&units.r_g_m).unwrap_or(Q::ZERO)),
                Dy::from_q(&c_deg),
            ];
            if c_deg.signum() > 0 {
                let _ = K.set(k);
            }
            k
        }
    };
    if r_kpc.signum() <= 0 {
        return Dy::ZERO;
    }
    let (m, r) = (inside_mass.max(Dy::ZERO) * per_core, r_kpc * rg_per_kpc);
    c_deg * (m / (r + m)) / r_kpc
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
