//! Hydromagnetics: a
//! world's core dynamo, its field, and the particles that reach its surface.
//!
//! A core convects when it loses heat faster than its iron conducts along its
//! adiabat (bodies.rs). In a conductor that convection builds and holds a
//! field, which shapes what reaches the world (its star's wind and the
//! galaxy's cosmic rays) and so what its surface keeps.
//!
//!   Dynamo    runs while q_cmb > q_ad; the share of the core's heat flow
//!             available to it is 1 - q_ad / q_cmb.
//!   Field     the dipole moment is the share 2x / (1 + x) of the convective
//!             power (excess flux x core radius) against the unit world's,
//!             times the core radius ratio; units are the unit world's
//!             (worlds::unit, a ruler).
//!   Standoff  where the field's pressure meets the wind (Chapman-Ferraro):
//!             10.1 R times the share of B against P, scaled to the unit
//!             world's measured magnetopause.
//!   Exposure  the share of the sky the wind reaches: R / r_mp, all of it
//!             with no field.
//!   Dose      the star's particles and the galaxy's cosmic rays above the
//!             field's cutoff (Stormer; the spectrum falls as
//!             rigidity^-1.7), both absorbed by the air's column.

use crate::worlds::{unit, State};
use xyl_core::dyadic::Dy;

fn n(x: i64) -> Dy {
    Dy::int(x)
}

fn r(a: i64, b: i64) -> Dy {
    Dy::ratio(a, b)
}

/// The unit world's magnetopause in its radii (subsolar, quiet wind).
fn unit_world_standoff() -> Dy {
    r(101, 10)
}

fn power(w: &State) -> Dy {
    (w.q_cmb - w.q_ad).max(Dy::ZERO) * w.r_core
}

/// The share 2x / (1 + x): 1 at x = 1, rising and saturating at 2. Used here
/// in place of power laws.
fn share2(x: Dy) -> Dy {
    let x = x.max(Dy::ZERO);
    Dy::int(2) * x / (Dy::ONE + x)
}

/// The world's dipole moment in units of the unit world's: the share of its
/// convective power against the unit's, times its core radius over the
/// unit's.
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

/// The dipole field at the surface, in the unit world's.
pub fn surface_field(w: &State) -> Dy {
    moment(w) / w.radius.max(Dy::sci(1, -6))
}

/// Where the field holds off the wind, in the world's radii (0: no field).
pub fn standoff(w: &State, pressure: Dy) -> Dy {
    let b = surface_field(w);
    if b.signum() <= 0 {
        return Dy::ZERO;
    }
    unit_world_standoff() * share2(b / pressure.max(Dy::sci(1, -12)))
}

/// The share of the world's sky its star's wind reaches.
pub fn exposure(w: &State, pressure: Dy) -> Dy {
    let rr = standoff(w, pressure);
    if rr <= Dy::ONE {
        Dy::ONE
    } else {
        Dy::ONE / rr
    }
}

/// The share of the galaxy's cosmic rays the field lets through: above its
/// cutoff rigidity (6 GV at the unit world, rising with the moment over r^2),
/// the integral spectrum falling as rigidity^-1.7 about a 4 GV median.
fn gcr_through_field(w: &State) -> Dy {
    let radius = w.radius.max(Dy::sci(1, -6));
    let cutoff = n(6) * moment(w) / radius;
    Dy::ONE / (Dy::ONE + r(17, 10) * cutoff / n(4))
}

/// What passes the air's column (1033 g/cm^2 per bar at unit gravity,
/// attenuated over 185 g/cm^2 with its cascade).
fn through_air(w: &State) -> Dy {
    let g = if w.g.signum() > 0 { w.g } else { Dy::ONE }.max(Dy::sci(1, -6));
    let column = n(1033) * w.p_bar / g;
    (-(column / n(185))).exp()
}

/// The particles' power reaching a world's surface (W/m^2): its star's wind
/// (2.7e-4 W/m^2 through the unit orbit) through the share of sky its field
/// leaves open, the galaxy's cosmic rays (1.2e-5 W/m^2 at R0) above its
/// cutoff - both through its air.
pub fn particle_power(w: &State) -> Dy {
    let stellar = if w.sw.signum() > 0 {
        Dy::sci(27, -5) * w.sw * exposure(w, w.sw)
    } else {
        Dy::ZERO
    };
    let cosmic = Dy::sci(12, -6) * w.cr * gcr_through_field(w);
    (stellar + cosmic) * through_air(w)
}

/// Chemical energy the particles store in a world's surface ices and seas
/// (flux units): 3% of the absorbed energy stored as oxidants (G ~ 2 per
/// 100 eV at ~1.5 eV each), over the 340 W/m^2 flux unit.
pub fn radiolysis(w: &State) -> Dy {
    r(3, 100) * particle_power(w) / n(340)
}

fn dose_raw(w: &State, pressure: Dy, galactic: Dy) -> Dy {
    let stellar = pressure * exposure(w, pressure);
    let cosmic = galactic * gcr_through_field(w);
    (stellar + cosmic) * through_air(w)
}

/// Particles reaching the surface, in the unit world's dose under one bar of
/// its own air. `galactic`: the cosmic-ray flux where it circles.
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
    fn the_standoff_follows_chapman_ferraro_and_the_wind_reaches_a_sliver() {
        let u = unit();
        // the standoff is the field's share against the wind's pressure: a harder
        // wind holds it nearer (never (B^2 / P)^(1/6))
        let (r1, r64) = (standoff(u, Dy::ONE), standoff(u, Dy::int(64)));
        assert!(r64 < r1 && r64.signum() > 0, "{r1} against {r64}");
        // the exposure is R / r_mp, the share of its directions the wind
        // reaches (never squared): a field holds most of the sky off
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
