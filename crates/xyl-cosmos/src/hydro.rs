//! Hydromagnetics: a world's core dynamo, its dipole field, and the particle
//! power and dose that reach its surface.
//!
//! Reads `worlds::State` (core heat flow `q_cmb`, adiabatic flow `q_ad`, core
//! radius, radius, gravity, surface pressure, stellar wind `sw`, cosmic-ray
//! flux `cr`) and measures every field against `worlds::unit`, the unit world
//! (a ruler). Quantities are `Dy`; field, standoff and dose are in the unit
//! world's units, particle power in W/m^2 (SI rulers in the code).
//!
//!   Dynamo    runs while q_cmb > q_ad; convective power is
//!             (q_cmb - q_ad) x core radius.
//!   Field     `moment`: the share 2x / (1 + x) of the power against the
//!             unit world's, times the core radius ratio; `surface_field`
//!             divides by the world's radius.
//!   Standoff  `standoff` (Chapman-Ferraro): 10.1 world radii times the
//!             share 2x / (1 + x) of B against the wind pressure P.
//!   Exposure  `exposure`: R / r_mp, the whole sky (1) with no field.
//!   Dose      `particle_power`, `dose`: the star's wind through the exposed
//!             sky plus the cosmic rays that pass the field's cutoff
//!             rigidity (Stormer), the passing share being
//!             1 / (1 + 1.7 R_c / 4 GV); both are attenuated by the air's
//!             column over 185 g/cm^2. `radiolysis` stores 3% of the
//!             absorbed power as surface oxidants.
//!
//! Every function is total and deterministic: a world with no convective
//! power has moment 0, standoff 0 and exposure 1; divisors are floored.

use crate::worlds::{unit, State};
use xyl_core::dyadic::Dy;

/// The unit world's magnetopause distance in unit-world radii (subsolar, quiet
/// wind).
fn unit_world_standoff() -> Dy {
    Dy::ratio(101, 10)
}

fn power(w: &State) -> Dy {
    (w.q_cmb - w.q_ad).max(Dy::ZERO) * w.r_core
}

/// The share `2x / (1 + x)`: 0 at x = 0, 1 at x = 1, approaching 2 as x grows.
/// The module uses it in place of power laws.
fn share2(x: Dy) -> Dy {
    let x = x.max(Dy::ZERO);
    Dy::int(2) * x / (Dy::ONE + x)
}

/// The world's dipole moment in units of the unit world's moment: the share
/// `2x / (1 + x)` of the convective power ratio x, times the ratio of the
/// world's core radius to the unit world's. Returns 0 when either convective
/// power or core radius is not positive.
pub fn moment(w: &State) -> Dy {
    moment_against(w, unit())
}

fn moment_against(w: &State, u: &State) -> Dy {
    let (pu, ru, p, rc) = (power(u), u.r_core, power(w), w.r_core);
    if p.signum() <= 0 || rc.signum() <= 0 || pu.signum() <= 0 || ru.signum() <= 0 {
        return Dy::ZERO;
    }
    share2(p / pu) * (rc / ru)
}

/// The dipole field at the surface: the moment over the world's radius, in the
/// unit world's units.
pub fn surface_field(w: &State) -> Dy {
    moment(w) / w.radius.max(Dy::sci(1, -6))
}

/// The magnetopause standoff distance in world radii: 10.1 times the share
/// `2x / (1 + x)` of the surface field against the wind pressure. Returns 0
/// when the world has no field.
pub fn standoff(w: &State, pressure: Dy) -> Dy {
    let b = surface_field(w);
    if b.signum() <= 0 {
        return Dy::ZERO;
    }
    unit_world_standoff() * share2(b / pressure.max(Dy::sci(1, -12)))
}

/// The share of the world's sky that the stellar wind reaches: `R / r_mp`,
/// or 1 when the standoff is at most one world radius.
pub fn exposure(w: &State, pressure: Dy) -> Dy {
    let rr = standoff(w, pressure);
    if rr <= Dy::ONE {
        Dy::ONE
    } else {
        Dy::ONE / rr
    }
}

/// The share of the galactic cosmic rays that the field lets through. The
/// cutoff rigidity is `R_c = 6 GV x moment / radius` (6 GV at the unit world);
/// the passing share is `1 / (1 + 1.7 R_c / 4 GV)`.
fn gcr_through_field(w: &State) -> Dy {
    let radius = w.radius.max(Dy::sci(1, -6));
    let cutoff = Dy::int(6) * moment(w) / radius;
    Dy::ONE / (Dy::ONE + Dy::ratio(17, 10) * cutoff / Dy::int(4))
}

/// The share that passes the air's column: `exp(-column / 185 g/cm^2)`, the
/// column being 1033 g/cm^2 per bar at unit gravity, scaled by `p_bar / g`.
/// The 185 g/cm^2 attenuation length includes the secondary cascade.
fn through_air(w: &State) -> Dy {
    let g = if w.g.signum() > 0 { w.g } else { Dy::ONE }.max(Dy::sci(1, -6));
    let column = Dy::int(1033) * w.p_bar / g;
    (-(column / Dy::int(185))).exp()
}

/// The particle power reaching a world's surface (W/m^2): the stellar wind
/// (2.7e-4 W/m^2 at the unit orbit, scaled by `sw`) through the exposed share
/// of sky, plus the galactic cosmic rays (1.2e-5 W/m^2 at R0, scaled by `cr`)
/// that pass the cutoff, both attenuated by the air's column.
pub fn particle_power(w: &State) -> Dy {
    let stellar = if w.sw.signum() > 0 {
        Dy::sci(27, -5) * w.sw * exposure(w, w.sw)
    } else {
        Dy::ZERO
    };
    let cosmic = Dy::sci(12, -6) * w.cr * gcr_through_field(w);
    (stellar + cosmic) * through_air(w)
}

/// The chemical energy flux that particles store in a world's surface ices
/// and seas, in flux units: 3% of the particle power is stored as oxidants
/// (a yield of G ~ 2 per 100 eV at ~1.5 eV each), divided by the 340 W/m^2
/// flux unit.
pub fn radiolysis(w: &State) -> Dy {
    Dy::ratio(3, 100) * particle_power(w) / Dy::int(340)
}

fn dose_raw(w: &State, pressure: Dy, galactic: Dy) -> Dy {
    let stellar = pressure * exposure(w, pressure);
    let cosmic = galactic * gcr_through_field(w);
    (stellar + cosmic) * through_air(w)
}

/// The particle dose reaching the surface, relative to the unit world's dose
/// under one bar of air at unit gravity with unit wind pressure and unit
/// cosmic-ray flux. `galactic` is the cosmic-ray flux at the world's galactic
/// orbit, relative to the unit world's.
pub fn dose(w: &State, pressure: Dy, galactic: Dy) -> Dy {
    let u = State {
        p_bar: Dy::ONE,
        g: Dy::ONE,
        ..unit().clone()
    };
    dose_raw(w, pressure, galactic) / dose_raw(&u, Dy::ONE, Dy::ONE).max(Dy::sci(1, -12))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standoff_falls_with_wind_pressure_and_bounds_exposure() {
        let u = unit();
        // The standoff falls as the wind pressure rises (the share
        // 2x / (1 + x), not the power law (B^2 / P)^(1/6)).
        let (r1, r64) = (standoff(u, Dy::ONE), standoff(u, Dy::int(64)));
        assert!(r64 < r1 && r64.signum() > 0, "{r1} against {r64}");
        // The exposure is R / r_mp (not squared), the share of directions
        // the wind reaches; it is below 1 for a magnetised world.
        let e = exposure(u, Dy::ONE);
        assert!(
            (e - Dy::ONE / r1).abs() < e * Dy::sci(1, -9),
            "exposure {e}"
        );
        assert!(e > Dy::ZERO && e < Dy::ONE, "exposure {e}");
    }

    #[test]
    fn a_world_with_no_field_takes_the_whole_wind() {
        let bare = State {
            q_cmb: Dy::ZERO,
            ..unit().clone()
        };
        assert_eq!(moment(&bare), Dy::ZERO);
        assert_eq!(exposure(&bare, Dy::ONE), Dy::ONE);
        assert!(particle_power(&bare) > particle_power(unit()));
    }
}
