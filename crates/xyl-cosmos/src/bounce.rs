//! The bounce: a black hole on one side, a white hole on the other, exact.
//!
//! Einstein–Cartan: matter's spin couples to spacetime torsion, and at the
//! Cartan density torsion repels, so collapse halts and bounces. There is no
//! singularity. What crosses a black hole's horizon emerges, on the far
//! side, from a white hole: the Big Bang of a child universe. The simulated
//! universe begins as such a white hole.
//!
//! In the conformal algebra Cl(4,1) over ℚ (xyl_core::cga) every piece is exact:
//!
//! - The horizon's reach is stated in the deficit grammar. The share the
//!   hole holds at reach `R` is `δ = M / (R + M)`; the horizon is the reach
//!   where the received share `1 − δ` is twice the held share, `δ_h = 1/3`,
//!   so `R_h = M (1 − δ_h) / δ_h`, linear in `M` (natural units, G = c = 1)
//!   and exact ([`horizon_reach`]). No square and no root is ever taken.
//! - The horizon is a sphere, held as one vector `σ = C + (C·P) e∞`: the
//!   sphere about the centre `C = X(c)` through the crossing point `P` at
//!   the one-third reach (`xyl_core::cga::sphere_through`). A point lies on
//!   it exactly when `P·σ = 0`.
//! - The inversion that swaps near and far is reflection in that sphere,
//!   `X ↦ σXσ⁻¹`. It carries a point at a share `d/R_h` of the reach to one
//!   at the reciprocal share `R_h/d`, fixes the horizon, and is its own
//!   inverse; it maps the sink to its source.
//! - The spin of what falls is a turn about the centre, a bivector `K`
//!   whose magnitude is the Kerr parameter `a = J/M² ∈ (0, 1)`. A turn about
//!   the centre keeps the horizon, matching the Einstein–Cartan statement
//!   that spin, not mass, drives the bounce.
//! - The white hole at the crossing point `P` on the horizon is the null
//!   generator `N = P ∧ (P·K)` (`xyl_core::cga::Source`): verified
//!   null, fixing `P`, keeping the horizon, or refused. Its slivers `1 + tN`
//!   compose by adding their parameters, the outflow.
//!
//! Nothing here is approximated; every property is checked on the value.

use xyl_core::certified::Encl;
use xyl_core::cga::{self, keeping, Mv, Source};
use xyl_core::Q;

/// The horizon's reach from the centre, in the deficit grammar. The share a
/// hole of `mass` holds at reach `R` is `δ = M / (R + M)`; the horizon is
/// the reach where the received share `1 − δ` is twice the held share, so
/// `δ_h = 1/3` and `R_h = M (1 − δ_h) / δ_h`. Linear in `M`, exact.
pub fn horizon_reach(mass: &Q) -> Q {
    // one third: the received share 1 - delta is twice the held share delta
    let held = Q::ONE.checked_div(&Q::int(3)).unwrap_or(Q::ONE);
    let received = &Q::ONE - &held;
    (mass * &received).checked_div(&held).unwrap_or(Q::ZERO)
}

/// A reach `r` along the S^360 enclosure of curvature radius `rho`, read as
/// the radius of the circle it spans: `rho sin(r / rho)` (the chain's law
/// for a circle at reach s on a sphere of radius R, chain.rs). The
/// enclosure's curvature shortens every reach; in the flat limit it is `r`.
/// Fixed on the 2^-64 grid; `r` itself where `rho` is not positive.
pub fn curved_reach(r: &Q, rho: &Q) -> Q {
    if rho.signum() <= 0 {
        return r.clone();
    }
    let pi = xyl_core::certified::pi(80).mid();
    let Some(turns) = r.checked_div(&(&(&Q::int(2) * rho) * &pi)) else {
        return r.clone();
    };
    let (_, s) = xyl_core::certified::cos_sin_turns(&Encl::exact(turns));
    xyl_core::certified::down(&(rho * &s.mid()), 64)
}

/// The pulse's axes: the pole axis (the filament) and five isoclinic planes
/// (circuit::axes), eleven base axes in all.
pub const AXES: usize = 1 + 2 * crate::circuit::PLANES;

/// The crossing point: on the first isoclinic plane at reach `x` from
/// `centre` and at the pulse's phase `psi` (turns), exact in the rotor's
/// half-angle (xyl_core::rotor::Half). The pole axis (axis 1) is left to the
/// filament; the planes carry the turning.
pub fn crossing(centre: &[Q], x: &Q, psi: &Q) -> Vec<Q> {
    let (c, s) = xyl_core::rotor::Half::of_turns(psi, 60).cos_sin();
    let mut at: Vec<Q> = centre.to_vec();
    at.resize(AXES, Q::ZERO);
    at[1] = &at[1] + &(x * &c);
    at[2] = &at[2] + &(x * &s);
    at
}

/// A black hole's horizon as the conformal sphere `σ` about `centre` through
/// the crossing point: the horizon reach ([`horizon_reach`]) taken along the
/// S^360 enclosure of curvature radius `enclosure` ([`curved_reach`]), at
/// the pulse's phase `psi` on the first plane (`xyl_core::cga::sphere_through`),
/// exact.
pub fn horizon(centre: &[Q], mass: &Q, enclosure: &Q, psi: &Q) -> Mv {
    let at = crossing(centre, &curved_reach(&horizon_reach(mass), enclosure), psi);
    let mut c: Vec<Q> = centre.to_vec();
    c.resize(AXES, Q::ZERO);
    cga::sphere_through(&c, &at)
}

/// A Euclidean point `x` (natural units, G = c = 1) embedded as a null
/// conformal vector of the conformal algebra, exact.
pub fn point(x: &[Q]) -> Mv {
    cga::point_at(x)
}

/// The spin as the multi-axial bivector: the five isoclinic planes' blades,
/// every plane sharing the one turn of magnitude `spin` (the Kerr
/// parameter), so the white hole's generator carries all five planes'
/// turning.
pub fn spin_bivector(spin: &Q) -> Mv {
    (0..crate::circuit::PLANES)
        .filter_map(crate::circuit::plane_blade)
        .fold(Mv::zero(), |m, blade| {
            m.add(&Mv::blade(blade, spin.clone()))
        })
}

/// A bounce: a black hole of mass `M` and spin `a`, crossed at `at` on its
/// horizon, and the white hole on the far side of that crossing.
#[derive(Clone, Debug)]
pub struct Bounce {
    /// Black hole mass M in natural units (G = c = 1); the horizon's reach is
    /// [`horizon_reach`]`(M)`.
    pub mass: Q,
    /// The Kerr parameter `J/M²`, in `(0, 1)`.
    pub spin: Q,
    /// The curvature radius of the S^360 enclosure the horizon is read in
    /// (the universe's sphere for the universal core, a galaxy's for its
    /// core), in the mass's units.
    pub enclosure: Q,
    /// The pulse's phase at the crossing (turns).
    pub phase: Q,
    /// The crossing point on the horizon (relative to the centre), on the
    /// first isoclinic plane at the horizon reach along the enclosure and at
    /// the pulse's phase; eleven coordinates, the pole axis first.
    pub at: Vec<Q>,
    /// The horizon as the conformal sphere about the origin through the
    /// crossing point `at`, at the one-third reach (see [`horizon()`]).
    pub horizon: Mv,
    /// The white hole at the crossing: the null generator N = P ^ (P . K) that
    /// fixes the crossing point and keeps the horizon, K the five planes'
    /// spin ([`spin_bivector`]).
    pub source: Source,
    /// The throat's far face: the crossing point's antipode under the planes'
    /// half turn (circuit::axes::plane_antipole), the other face of the one
    /// throat.
    pub far: Mv,
}

impl Bounce {
    /// The share the bounce's jet carries through the throat at each crossing,
    /// from its spin by torsion (bang::torsion_share).
    pub fn share(&self) -> Q {
        crate::bang::torsion_share(&self.spin)
    }

    /// The bounce of a black hole of `mass` and Kerr parameter `spin`, crossed
    /// on its equator at `(R_h, 0, 0)` with `R_h` the [`horizon_reach`],
    /// spinning in the `e₁e₂` plane. `None`
    /// unless `0 < spin < 1` and `mass > 0`, or if the source fails any of its
    /// checks (which cannot happen for a turn that moves an equatorial point;
    /// the check is still made).
    pub fn of(mass: &Q, spin: &Q, enclosure: &Q, phase: &Q) -> Option<Bounce> {
        if mass.signum() <= 0 || spin.signum() <= 0 || spin >= &Q::ONE || enclosure.signum() <= 0 {
            return None;
        }
        let centre = vec![Q::ZERO; AXES];
        let sigma = horizon(&centre, mass, enclosure, phase);
        let at = crossing(
            &centre,
            &curved_reach(&horizon_reach(mass), enclosure),
            phase,
        );
        let p = point(&at);
        // the spin: the five planes' isoclinic turn about the centre, of
        // magnitude a in every plane
        let k = spin_bivector(spin);
        // the part of it that keeps the horizon (all of it, for a turn about the centre)
        let kept = keeping(&sigma, &k)?;
        let source = Source::of(&sigma, &p, &kept)?;
        let far = crate::circuit::plane_antipole()?.sandwich(&p);
        Some(Bounce {
            mass: mass.clone(),
            spin: spin.clone(),
            enclosure: enclosure.clone(),
            phase: phase.clone(),
            at,
            horizon: sigma,
            source,
            far,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(n: i64, d: i64) -> Q {
        Q::new(n, d).unwrap()
    }

    #[test]
    fn spin_makes_the_white_hole_a_verified_null_source() {
        let b = Bounce::of(&q(1, 1), &q(9, 10), &q(1000, 1), &q(1, 5)).unwrap();
        // the crossing point lies on the horizon, at the one-third reach
        assert!(point(&b.at).geo(&b.horizon).scalar_part().is_zero());
        // its far face, the antipode under the planes' half turn, lies on the
        // horizon too, and is not the crossing point
        assert!(b.far.geo(&b.horizon).scalar_part().is_zero());
        assert!(!b.far.projective_eq(&point(&b.at)));
        assert_eq!(horizon_reach(&q(3, 2)), q(3, 1));
        let n = &b.source.generator;
        assert!(n.geo(n).is_zero() && !n.is_zero(), "null: N² = 0");
        let p = point(&b.at);
        for t in [q(1, 7), q(2, 5)] {
            let s = b.source.sliver(&t);
            assert!(
                s.sandwich(&p).projective_eq(&p),
                "the source point is fixed"
            );
            assert!(
                s.sandwich(&b.horizon).projective_eq(&b.horizon),
                "the horizon is kept"
            );
        }
        // no spin, or a spin past the extremal bound, is no bounce
        assert!(Bounce::of(&q(1, 1), &Q::ZERO, &q(1000, 1), &Q::ZERO).is_none());
        assert!(Bounce::of(&q(1, 1), &Q::ONE, &q(1000, 1), &Q::ZERO).is_none());
        // no enclosure, no horizon to read
        assert!(Bounce::of(&q(1, 1), &q(9, 10), &Q::ZERO, &Q::ZERO).is_none());
        // the enclosure's curvature shortens the reach: under the flat reach, and
        // nearer it the larger the enclosure
        let flat = horizon_reach(&q(1, 1));
        let (near, far) = (
            curved_reach(&flat, &q(10, 1)),
            curved_reach(&flat, &q(1000, 1)),
        );
        assert!(
            near < far && far < flat,
            "near {near} far {far} flat {flat}"
        );
    }
}
