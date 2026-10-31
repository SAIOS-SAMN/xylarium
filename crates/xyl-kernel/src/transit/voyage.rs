//! Places, voyages and drives: the distance between two places
//! ([`separation_m`]), a drive's voyage out of a well ([`voyage`]), the
//! transfer window within a system ([`window_years`]) and the drive a people
//! flies ([`best_spec`], [`blended`]).

use super::{kpc_m, year_s};
use crate::cosmic::Cosmic;
use crate::propulsion::{delta_v, propellant_per_kg, Spec};
use xyl_core::dyadic::Dy;
use xyl_core::units::Tier;
use xyl_core::Q;
use xyl_cosmos::chain::units;
use xyl_cosmos::manifold::{gap, manifolds, Manifold};
use xyl_cosmos::package::Si;
use xyl_cosmos::systems::World;

/// A place: a world of a star's system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Place {
    /// The star, by its sample number.
    pub star: u64,
    /// The world, by its index in the star's system.
    pub world: usize,
}

/// A world's escape speed (m/s): the turning law at its surface (turning.rs),
/// its turning there times its radius, i.e. c delta with delta = M / (R + M)
/// in the white hole's units (a deficit share, not sqrt(2 G M / R)).
pub fn escape_speed(w: &World) -> Dy {
    let m = &w.mass * &Si::world_mass_kg();
    let r = &w.radius * &Si::world_radius_m();
    Dy::from_q(&(&xyl_cosmos::turning::rad_s(&xyl_cosmos::turning::turning_si(&m, &r)) * &r))
}

/// The gap between two worlds of one system at its star's `age` (Gyr), in
/// metres, read on the ocean (manifold.rs `gap`): the turn between where
/// their orbits stand now - each by its phase, plane, shape and twist; a
/// moon about its giant - swept at the inner one's reach, then the climb
/// between them. No squared length or root is taken.
pub fn orbit_gap_m(all: &[Manifold], wa: usize, wb: usize, age: &Q) -> Option<Dy> {
    let g = gap(all, wa, wb, age)?;
    Some(Dy::from_q(
        &(&(&g * &units().r_g_m) * &Tier::System.in_rg()),
    ))
}

/// The distance between two places now (m): within a system by where their
/// orbits stand now, between systems by the arc between their stars (both
/// witnessed).
pub fn separation_m(cosmic: &Cosmic, a: Place, b: Place) -> Option<Dy> {
    if a.star == b.star {
        let wit = cosmic.systems.get(&a.star)?;
        let all = manifolds(cosmic.gal.seed, &[a.star], &wit.system, &wit.star, &units());
        let age = (cosmic.now - Dy::from_q(&wit.star.born))
            .max(Dy::ZERO)
            .to_q();
        return orbit_gap_m(&all, a.world, b.world, &age);
    }
    Some(cosmic.distance(a.star, b.star)? * kpc_m())
}

/// A voyage, as one drive makes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Voyage {
    /// Travel time at the coasting speed, in years.
    pub years: Dy,
    /// Its coasting speed (m/s).
    pub cruise: Dy,
    /// The fuel's energy it burns per kg carried (J/kg).
    pub energy_per_kg: Dy,
}

/// The speed a drive coasts at out of a well of this escape speed (m/s): half
/// of what its mass ratio buys beyond the climb out (the other half brakes).
pub fn cruise_speed(spec: &Spec, ratio: Dy, escape: Dy) -> Dy {
    ((delta_v(spec.v_e, ratio) - escape) / Dy::int(2)).max(Dy::ZERO)
}

/// A voyage of `dist_m` out of a well of `escape` (m/s); None if the drive
/// cannot climb out of it.
pub fn voyage(spec: &Spec, ratio: Dy, escape: Dy, dist_m: Dy) -> Option<Voyage> {
    let cruise = cruise_speed(spec, ratio, escape);
    if cruise.signum() <= 0 {
        return None;
    }
    let years = dist_m.max(Dy::ZERO) / cruise / year_s();
    Some(Voyage {
        years,
        cruise,
        energy_per_kg: propellant_per_kg(ratio) * spec.energy_per_kg,
    })
}

/// The farthest a drive carries within `years` out of this well (kpc).
pub fn reach_kpc(spec: &Spec, ratio: Dy, escape: Dy, years: Dy) -> Dy {
    cruise_speed(spec, ratio, escape) * years * year_s() / kpc_m()
}

/// The transfer window between two worlds of one system: a transfer is half a
/// circulation, so it leaves when the target's phase leads the
/// origin's by half a turn less what the target turns on the way; the
/// phases come round on each other at their synodic rate. Years to wait
/// from `age` (Gyr) for a leg of `leg_years`, and the origin's orbital speed
/// (m/s, its turning times its reach); None where the two never part (one
/// rate) or are not of one anchor.
pub fn window_years(
    all: &[Manifold],
    wa: usize,
    wb: usize,
    age: &Q,
    leg_years: Dy,
) -> Option<(Dy, Dy)> {
    let (a, b) = (
        all.iter().find(|m| m.world == wa)?,
        all.iter().find(|m| m.world == wb)?,
    );
    if a.anchor != b.anchor {
        return None;
    }
    let (ra, rb) = (Dy::from_q(&a.rate), Dy::from_q(&b.rate));
    let rel = (rb - ra) / Dy::int(360);
    if rel.signum() == 0 {
        return None;
    }
    let age = Dy::from_q(age);
    let phase = |m: &Manifold, r: Dy| Dy::from_q(&m.m0) + r * age / Dy::int(360);
    let lead = Dy::ratio(1, 2) - rb * leg_years / Dy::sci(1, 9) / Dy::int(360);
    let off = (lead - (phase(b, rb) - phase(a, ra))) * Dy::int(rel.signum().into());
    let turns = off - Dy::int(off.trunc_i64()) + if off.signum() < 0 { Dy::ONE } else { Dy::ZERO };
    let wait = turns.max(Dy::ZERO) / rel.abs() * Dy::sci(1, 9);
    let a_m = &(&a.a * &units().r_g_m) * &a.tier.in_rg();
    let v_orb = Dy::from_q(&(&xyl_cosmos::turning::rad_s(&a.rate) * &a_m));
    Some((wait, v_orb))
}

/// A drive between two specs, by a share: what a people that has mastered a
/// share of its energy's domain flies (linear in the share).
pub fn blended(lo: &Spec, hi: &Spec, share: Dy) -> Spec {
    let s = share.clamp(Dy::ZERO, Dy::ONE);
    Spec {
        drive: if s > Dy::ratio(1, 2) {
            hi.drive
        } else {
            lo.drive
        },
        energy_per_kg: lo.energy_per_kg + s * (hi.energy_per_kg - lo.energy_per_kg),
        v_e: lo.v_e + s * (hi.v_e - lo.v_e),
    }
}

/// The swiftest drive a people's chemistry allows (an ancient house
/// understands every fuel).
pub fn best_spec(cosmic: &Cosmic, p: &crate::cosmic::People) -> Spec {
    crate::propulsion::Drive::ALL
        .iter()
        .map(|&d| Spec::of(d, &p.traits, &cosmic.gal.sc, cosmic.gal.elements))
        .max_by(|a, b| a.v_e.cmp(&b.v_e))
        .unwrap_or_else(|| {
            Spec::of(
                crate::propulsion::Drive::Chemical,
                &p.traits,
                &cosmic.gal.sc,
                cosmic.gal.elements,
            )
        })
}

/// A people's home world's escape speed (m/s).
pub fn home_escape(cosmic: &Cosmic, p: &crate::cosmic::People) -> Dy {
    cosmic
        .systems
        .get(&p.star)
        .and_then(|w| w.system.worlds.get(p.world))
        .map(escape_speed)
        .unwrap_or(Dy::ZERO)
}
