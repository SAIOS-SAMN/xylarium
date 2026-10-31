// The multi-axial pulse as versors: one fixed pole axis and five planes
// turning isoclinically (11 = 1 + 2 x 5 base axes), their antipoles and
// antinodes.
//
// Inputs: `xyl_core::cga` (exact versors over `Q`) and the pole point
// X(t), t = U / W (poles.rs).
//
// Outputs: `PLANES`, `plane_blade`, `plane_antipole` (the planes' half
// turn), `axis_antipole` (unit-sphere inversion X(t) -> X(1 / t)),
// `antipole` (both; they commute), `antipode` (psi + 1/2 on a phase),
// `inverted`, `antinode(q)` (1/q for odd q, 0 for even), `light_of` (light
// as shine's antipole, D(1 / rho)) and `antipoles_hold`, which checks both
// antipoles projectively. All results are exact; out-of-domain input
// (zero coordinate, missing axis) returns None.
// Terms: filament (docs/glossary.md).

use xyl_core::cga::{self, base_axis, Mv};
use xyl_core::Q;

/// The multi-axial pulse (11 = 1 + 2 x 5): one fixed axis and five orthogonal
/// planes turning isoclinically, so every plane shares one phase. The fixed
/// axis is the pole axis (base axis 1, where `pole_point` sets X(t), t = U / W:
/// Z the origin, Q at infinity, the throat at t = 1); the planes lie on base
/// axes 2-3, 4-5, 6-7, 8-9, 10-11.
pub const PLANES: usize = 5;

/// The blade of isoclinic plane `k` (0..PLANES): base axes 2 + 2k and 3 + 2k.
pub fn plane_blade(k: usize) -> Option<usize> {
    Some(base_axis(2 + 2 * k)? | base_axis(3 + 2 * k)?)
}

/// The planes' antipole, the isoclinic half turn: the product of the five
/// planes' half-turn rotors, each the exact even versor 0 + 1 e_ab (exact in
/// the algebra, unreachable by a rational chart). Its sandwich sends every
/// point x of the planes to -x and fixes the axis, so a body's two faces are a
/// point and its antipode.
pub fn plane_antipole() -> Option<Mv> {
    (0..PLANES).try_fold(Mv::scalar(Q::ONE), |v, k| {
        Some(v.geo(&cga::rotator_on_blade(&Q::ZERO, &Q::ONE, plane_blade(k)?)))
    })
}

/// The same half turn on a phase carried as a share of the turn: psi + 1/2.
pub fn antipode(psi: &Q) -> Q {
    (psi + &Q::new(1, 2).unwrap_or(Q::ZERO)).frac()
}

/// The axis's antipole, the throat's inversion: the sandwich by the unit
/// sphere about the origin carries X(t) to X(1 / t), exchanging Z and Q and
/// fixing t = 1.
pub fn axis_antipole() -> Mv {
    cga::sphere_through(&[Q::ZERO], &[Q::ONE])
}

/// The same inversion on the pole coordinate: t -> 1 / t (None at a pole).
pub fn inverted(t: &Q) -> Option<Q> {
    Q::ONE.checked_div(t)
}

/// The antipole of the whole pulse: the planes' half turn and the axis's
/// inversion together. They commute (the throat sphere is centred on the
/// planes' fixed point), so the order does not matter.
pub fn antipole() -> Option<Mv> {
    Some(axis_antipole().geo(&plane_antipole()?))
}

/// The antinode share of harmonic q: q moves every plane by 1/q of its turn,
/// so a place and its antipode differ by q/2 turns. Even q puts the antipode
/// on the same face, the senses cancel and the result is a node (0); odd q
/// puts it on the opposite face, the senses add and the result is an antinode
/// of 1/q. Computed from the half turn, not special-cased by parity.
pub fn antinode(q: u32) -> Q {
    if q == 0 {
        return Q::ZERO;
    }
    let shift = Q::new(i64::from(q), 2).map(|x| x.frac()).unwrap_or(Q::ZERO);
    (&Q::int(2) * &shift)
        .checked_div(&Q::int(i64::from(q)))
        .unwrap_or(Q::ZERO)
}

/// Shine and light as one antipole pair. Shine is the efflux: the frames a
/// source's burn closes, the dilator D(rho) moving its pole point toward Z.
/// Light is the afflux, shine's antipole: the same flux through the axis's
/// inversion, I D(rho) I = D(1 / rho), opening the receiver's frames (toward
/// Q). The pair's books close where the two dilators compose to the identity.
/// Returns the light's dilation and whether the versor form closes exactly.
pub fn light_of(rho_shine: &Q) -> Option<(Q, bool)> {
    let rho_light = Q::ONE.checked_div(rho_shine)?;
    let i = axis_antipole();
    let through = i.geo(&cga::dilator(rho_shine)).geo(&i);
    let closes = cga::projective_ratio(&through, &cga::dilator(&rho_light)).is_some();
    Some((rho_light, closes))
}

/// Checks the antipoles on the versors at a carrier's state (t = U / W): the
/// inversion carries X(t) to X(1 / t), and the planes' half turn carries a
/// point of the planes to its negative with the axis fixed. Both must be
/// exact. Returns (inversion exact, half turn exact).
pub fn antipoles_hold(t: &Q) -> Option<(bool, bool)> {
    let one = Q::ONE;
    let x = cga::point_at(std::slice::from_ref(t));
    let inv = axis_antipole().sandwich(&x);
    let inversion = cga::projective_ratio(&inv, &cga::point_at(&[inverted(t)?])).is_some();
    // a point on the axis and on every plane: (t, 1, 0, 1, 0, ...)
    let mut p = vec![t.clone()];
    let mut m = vec![t.clone()];
    for _ in 0..PLANES {
        p.extend([one.clone(), Q::ZERO]);
        m.extend([-one.clone(), Q::ZERO]);
    }
    let turned = plane_antipole()?.sandwich(&cga::point_at(&p));
    let half_turn = cga::projective_ratio(&turned, &cga::point_at(&m)).is_some();
    Some((inversion, half_turn))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_antipoles_are_exact_versors() {
        for t in [Q::new(1, 3).unwrap(), Q::ONE, Q::new(7, 2).unwrap()] {
            assert_eq!(antipoles_hold(&t), Some((true, true)));
        }
        for rho in [Q::new(2, 5).unwrap(), Q::new(9, 4).unwrap()] {
            assert_eq!(light_of(&rho).map(|(_, ok)| ok), Some(true));
        }
        let a: Vec<Q> = (1..=4).map(antinode).collect();
        assert_eq!(a, vec![Q::ONE, Q::ZERO, Q::new(1, 3).unwrap(), Q::ZERO]);
        assert_eq!(antipode(&Q::new(3, 4).unwrap()), Q::new(1, 4).unwrap());
    }
}
