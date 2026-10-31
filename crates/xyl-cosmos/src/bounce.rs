//! The bounce: a spinning black hole's horizon, its crossing point and the
//! white-hole source on the far side, built exactly in conformal geometric
//! algebra.
//!
//! Model (Einstein-Cartan): matter's spin couples to spacetime torsion, and
//! at the Cartan density torsion repels, so collapse halts and bounces with
//! no singularity. What crosses a black hole's horizon emerges from a white
//! hole on the far side; the simulated universe begins as such a white hole.
//!
//! Reads `xyl_core::cga` (`point_at`, `sphere_through`, `keeping`, `Source`),
//! `xyl_core::rotor::Half`, `circuit::axes` (the five isoclinic planes and
//! their antipole) and `origin::torsion_share`. Units are natural, G = c = 1;
//! points have `AXES` = 11 coordinates (the pole axis first), so the algebra
//! is `Cl(12, 1)` over `Q`.
//!
//! - `horizon_reach`: with held share `delta = M / (R + M)`, the horizon is
//!   where the received share `1 - delta` is twice the held one, so
//!   `delta_h = 1/3` and `R_h = M (1 - delta_h) / delta_h = 2M`, exact.
//! - `curved_reach`: a reach `r` read on the S^360 enclosure of curvature
//!   radius `rho` as `rho sin(r / rho)`, fixed on the 2^-64 grid.
//! - `horizon`: the dual sphere `sigma = C + (C . P) e_inf` about the centre
//!   through the crossing `P` (`crossing`, on the first plane at phase psi).
//! - `spin_bivector`: the Kerr parameter `a = J / M^2` as the same turn in
//!   all five planes.
//! - `Bounce::of`: the source `N = P ^ (P . K)`, verified null, fixing `P`
//!   and keeping the horizon, and the far face, `P` under the planes' half
//!   turn. Returns None unless `M > 0`, `0 < a < 1`, the enclosure is
//!   positive and every check on the source passes.
//!
//! Terms: horizon, throat (docs/glossary.md).

use xyl_core::certified::Encl;
use xyl_core::cga::{self, keeping, Mv, Source};
use xyl_core::Q;

/// The horizon's reach from the centre, in the deficit grammar. The share a
/// hole of `mass` holds at reach `R` is `delta = M / (R + M)`; the horizon is
/// the reach where the received share `1 - delta` is twice the held share, so
/// `delta_h = 1/3` and `R_h = M (1 - delta_h) / delta_h = 2M`. Linear in `M`,
/// exact.
pub fn horizon_reach(mass: &Q) -> Q {
    // The horizon's held share is delta_h = 1/3, where the received share
    // 1 - delta is twice delta.
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

/// A black hole's horizon as the conformal sphere `sigma` about `centre` through
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

/// The share a throat's jet carries at each crossing, from the hole's spin
/// by torsion. In the Einstein-Cartan bounce torsion is not an independent
/// field: it is the spin density of what fell, so the spin's share enters the
/// hold directly. The throat `H = 2M` (`horizon_reach`) holds one face
/// against the other, `delta(H, H) = 1/2`, and the jet carries the unlocked
/// face `1 - delta`; the spin's angular momentum `J = a M^2` (`a` the Kerr
/// parameter, `spin`) is a further share the throat cannot hold, so the held
/// face is `delta(H, H + J) = H / (2H + J)` and the jet carries
/// `(H + J) / (2H + J)`: one half for a spinless hole, which does not bounce,
/// and `(2 + M) / (4 + M)` at the extremal spin (three fifths for the
/// universal core, `M = 1`). One law for every throat; linear in the shares
/// and exact. Zero for a non-positive mass.
pub fn torsion_share(mass: &Q, spin: &Q) -> Q {
    if mass.signum() <= 0 {
        return Q::ZERO;
    }
    let h = horizon_reach(mass);
    let j = &spin.clone().max(Q::ZERO) * &(mass * mass);
    (&h + &j)
        .checked_div(&(&(&Q::int(2) * &h) + &j))
        .unwrap_or(Q::ZERO)
}

/// The Kerr spin whose torsion share at `mass` is `a`: `J = H (2a - 1) /
/// (1 - a)`, the share law inverted, over `M^2`. `None` for a share at or
/// above one or a non-positive mass.
pub fn spin_of_share(mass: &Q, a: &Q) -> Option<Q> {
    let one_less = &Q::ONE - a;
    if one_less.signum() <= 0 || mass.signum() <= 0 {
        return None;
    }
    let h = horizon_reach(mass);
    (&h * &(&(&Q::int(2) * a) - &Q::ONE))
        .checked_div(&one_less)?
        .checked_div(&(mass * mass))
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
    /// The Kerr parameter `J / M^2`, in `(0, 1)`.
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
    /// crossing point `at`, at the horizon reach, where the held share is 1/3
    /// (see [`horizon()`]).
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
    /// from its mass and spin by torsion ([`torsion_share`]).
    pub fn share(&self) -> Q {
        torsion_share(&self.mass, &self.spin)
    }

    /// The bounce of a black hole of `mass` and Kerr parameter `spin`, read in
    /// the S^360 enclosure of curvature radius `enclosure` and crossed on the
    /// first isoclinic plane at the [`horizon_reach`] taken along the
    /// enclosure ([`curved_reach`]) and at the pulse's `phase`; the spin turns
    /// all five planes ([`spin_bivector`]). `None` unless `mass > 0`,
    /// `0 < spin < 1` and `enclosure > 0`, or if the horizon-keeping component
    /// of the spin or the source cannot be built (the checks are made for
    /// every input).
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
        // The spin bivector is an isoclinic turn about the centre, of magnitude
        // a in each of the five planes.
        let k = spin_bivector(spin);
        // Only the component that preserves the horizon generates the source;
        // for a turn about the centre that is the whole bivector.
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

    #[test]
    fn spin_makes_the_white_hole_a_null_source() {
        let b = Bounce::of(&Q::int(1), &Q::ratio(9, 10), &Q::int(1000), &Q::ratio(1, 5)).unwrap();
        // The crossing point lies on the horizon, at the horizon reach.
        assert!(point(&b.at).geo(&b.horizon).scalar_part().is_zero());
        // The far face, the antipode under the planes' half turn, lies on the
        // horizon too, and is not the crossing point.
        assert!(b.far.geo(&b.horizon).scalar_part().is_zero());
        assert!(!b.far.projective_eq(&point(&b.at)));
        assert_eq!(horizon_reach(&Q::ratio(3, 2)), Q::int(3));
        let n = &b.source.generator;
        assert!(n.geo(n).is_zero() && !n.is_zero(), "null: N^2 = 0");
        let p = point(&b.at);
        for t in [Q::ratio(1, 7), Q::ratio(2, 5)] {
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
        // Zero spin, or a spin at the extremal bound, gives no bounce.
        assert!(Bounce::of(&Q::int(1), &Q::ZERO, &Q::int(1000), &Q::ZERO).is_none());
        assert!(Bounce::of(&Q::int(1), &Q::ONE, &Q::int(1000), &Q::ZERO).is_none());
        // Without an enclosure no horizon is read.
        assert!(Bounce::of(&Q::int(1), &Q::ratio(9, 10), &Q::ZERO, &Q::ZERO).is_none());
        // The enclosure's curvature shortens the reach: the curved reach is
        // below the flat reach, and nearer to it the larger the enclosure.
        let flat = horizon_reach(&Q::int(1));
        let (near, far) = (
            curved_reach(&flat, &Q::int(10)),
            curved_reach(&flat, &Q::int(1000)),
        );
        assert!(
            near < far && far < flat,
            "near {near} far {far} flat {flat}"
        );
    }
}
