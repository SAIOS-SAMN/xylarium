//! Places, voyages and drives: the distance between two places
//! ([`separation_m`]), a drive's voyage out of a well ([`voyage`]), the
//! transfer window within a system ([`window_years`]) and the drive a people
//! uses ([`best_spec`], [`blended`]).

use super::{kpc_m, tier_m, year_s};
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
    /// The star, by sample number.
    pub star: u64,
    /// The world, by its index in the star's system.
    pub world: usize,
}

/// Returns a world's escape speed (m/s): the turning law at the world's
/// surface times its radius, `c delta` with `delta = M / (R + M)` the
/// deficit share (`xyl_core::shares::deficit`), the mass read as a length
/// `G M / c^2`, in place of sqrt(2 G M / R).
pub fn escape_speed(w: &World) -> Dy {
    // The world's mass as a length, G m / c^2, against its radius.
    let c = Si::c();
    let m = (&(&Si::g() * &w.mass) * &Si::world_mass_kg())
        .checked_div(&(&c * &c))
        .unwrap_or(Q::ZERO);
    let r = &w.radius * &Si::world_radius_m();
    Dy::from_q(&(&c * &xyl_core::shares::deficit(&m, &r).unwrap_or(Q::ZERO)))
}

/// Returns the gap (m) between worlds `wa` and `wb` of one system at the
/// star's `age` (rings), read on the ocean (manifold.rs `gap`): the turn
/// between the current positions of the two orbits (each given by phase,
/// plane, shape and twist; a moon orbits its giant), swept at the inner
/// world's reach, plus the climb between them. No squared length or root is
/// taken. Returns None if `gap` refuses.
pub fn orbit_gap_m(all: &[Manifold], wa: usize, wb: usize, age: &Q) -> Option<Dy> {
    let g = gap(all, wa, wb, age)?;
    Some(Dy::from_q(&(&g * &tier_m(Tier::System))))
}

/// Returns the distance (m) between two places at count `t` (rings): within
/// a system, the gap between the positions of the orbits then
/// ([`orbit_gap_m`]); between systems, the arc between the stars. Returns
/// None if a needed star is not witnessed.
pub fn separation_m(cosmic: &Cosmic, a: Place, b: Place, t: Dy) -> Option<Dy> {
    if a.star == b.star {
        let wit = cosmic.systems.get(&a.star)?;
        let all = manifolds(cosmic.gal.seed, &[a.star], &wit.system, &wit.star, &units());
        let age = (t - Dy::from_q(&wit.star.born)).max(Dy::ZERO).to_q();
        return orbit_gap_m(&all, a.world, b.world, &age);
    }
    Some(cosmic.distance(a.star, b.star, t)? * kpc_m())
}

/// A voyage made by one drive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Voyage {
    /// Travel time at the coasting speed, in years.
    pub years: Dy,
    /// The coasting speed (m/s).
    pub cruise: Dy,
    /// The fuel energy transduced per kg carried (J/kg): (R - 1) x the fuel's
    /// energy per kg.
    pub energy_per_kg: Dy,
}

/// Returns the coasting speed (m/s) of a drive leaving a well of escape
/// speed `escape`: half of the speed change bought by the mass ratio beyond
/// the escape speed (the other half brakes), and zero if none remains.
pub fn cruise_speed(spec: &Spec, ratio: Dy, escape: Dy) -> Dy {
    ((delta_v(spec.v_e, ratio) - escape)
        .div(&(Dy::int(2)))
        .unwrap_or_default())
    .max(Dy::ZERO)
}

/// Returns a voyage of `dist_m` metres out of a well of escape speed
/// `escape` (m/s), or None if the drive cannot climb out of it.
pub fn voyage(spec: &Spec, ratio: Dy, escape: Dy, dist_m: Dy) -> Option<Voyage> {
    let cruise = cruise_speed(spec, ratio, escape);
    if cruise.signum() <= 0 {
        return None;
    }
    let years = dist_m
        .max(Dy::ZERO)
        .div(&cruise)
        .unwrap_or_default()
        .div(&year_s())
        .unwrap_or_default();
    Some(Voyage {
        years,
        cruise,
        energy_per_kg: propellant_per_kg(ratio) * spec.energy_per_kg,
    })
}

/// Returns the greatest distance (kpc) a drive covers within `years` after
/// leaving a well of escape speed `escape`.
pub fn reach_kpc(spec: &Spec, ratio: Dy, escape: Dy, years: Dy) -> Dy {
    (cruise_speed(spec, ratio, escape) * years * year_s())
        .div(&kpc_m())
        .unwrap_or_default()
}

/// Returns the transfer window between worlds `wa` and `wb` of one system.
/// A transfer is half a circulation, so it departs when the target's phase
/// leads the origin's by half a turn less the target's turning during the
/// leg; the phases meet again at their synodic rate. The result is the wait
/// in years from `age` (Gyr) for a leg of `leg_years`, and the origin's
/// orbital speed (m/s, turning rate times reach). Returns None when the two
/// rates are equal or the worlds do not share an anchor.
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
    // The manifolds' rates are per ring; read per Gyr at this boundary.
    let per = |m: &Manifold| xyl_cosmos::circuit::rings_of_gyr_dy(Dy::from_q(&m.rate));
    let (ra, rb) = (per(a), per(b));
    let rel = (rb - ra).div(&(Dy::int(360))).unwrap_or_default();
    if rel.signum() == 0 {
        return None;
    }
    let age = Dy::from_q(age);
    let phase = |m: &Manifold, r: Dy| {
        Dy::from_q(&m.m0) + (r * age).div(&(Dy::int(360))).unwrap_or_default()
    };
    let lead = xyl_core::dyadic!(1, 2)
        - (rb * leg_years)
            .div(&(Dy::sci(1, 9)))
            .unwrap_or_default()
            .div(&(Dy::int(360)))
            .unwrap_or_default();
    let off = (lead - (phase(b, rb) - phase(a, ra))) * Dy::int(rel.signum().into());
    let turns = off
        - Dy::int(
            off.trunc_i64()
                .unwrap_or(if off.signum() > 0 { i64::MAX } else { i64::MIN }),
        )
        + if off.signum() < 0 { Dy::ONE } else { Dy::ZERO };
    let wait = turns.max(Dy::ZERO).div(&rel.abs()).unwrap_or_default() * Dy::sci(1, 9);
    let a_m = &a.a * &tier_m(a.tier);
    let v_orb = Dy::from_q(&(&xyl_cosmos::package::Si::rad_s(&a.rate) * &a_m));
    Some((wait, v_orb))
}

/// Returns the drive between specs `lo` and `hi` at `share` (clamped to
/// [0, 1]): energy per kg and exhaust speed are linear in the share, and the
/// drive kind is `hi`'s above one half. It is the drive of a people that has
/// mastered that share of the energy domain.
pub fn blended(lo: &Spec, hi: &Spec, share: Dy) -> Spec {
    let s = share.clamp(Dy::ZERO, Dy::ONE);
    Spec {
        // Stand-in S085: The drive blends chemical to annihilation; the kind flips at 1/2.
        drive: if s > xyl_core::dyadic!(1, 2) {
            hi.drive
        } else {
            lo.drive
        },
        energy_per_kg: lo.energy_per_kg + s * (hi.energy_per_kg - lo.energy_per_kg),
        v_e: lo.v_e + s * (hi.v_e - lo.v_e),
    }
}

/// Returns the drive with the highest exhaust speed that the people's traits
/// and the galaxy's chemistry allow.
pub fn best_spec(cosmic: &Cosmic, p: &crate::cosmic::People) -> Spec {
    // Stand-in S083: The best drive is always annihilation at c^2 per kg, so v_e = c.
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

/// Returns the escape speed (m/s) of a people's home world, or zero if that
/// world is not witnessed.
pub fn home_escape(cosmic: &Cosmic, p: &crate::cosmic::People) -> Dy {
    cosmic
        .systems
        .get(&p.star)
        .and_then(|w| w.system.worlds.get(p.world))
        .map(escape_speed)
        .unwrap_or(Dy::ZERO)
}
