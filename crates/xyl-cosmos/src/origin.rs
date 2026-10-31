//! The origin: a universe's initial conditions, drawn from its seed and
//! held as exact rationals.
//!
//! Inputs: the seed, through a `xyl_core::Stream` on the path "big origin";
//! `bounce::horizon_reach` and `Bounce::of` for the parent
//! ([`crate::bounce`]); `cores::substrate_for` and `circuit::anchor` for the
//! founding. Units are the parent's (unit mass).
//!
//! One value is drawn: the parent's Kerr spin, uniform on (0, 1) and fixed
//! on the 2^-40 grid ([`GRID_BITS`], [`clamp_spin`]). `cores::substrate_for`
//! floors it at the least spin whose torsion share holds the firmament
//! whole (the furthest held kind's reach over the rim, swung by the inbound
//! echo's peak). Everything else is read off the founding: the rim, the bell
//! (`circuit::anchor`), and the [`Origin`]'s lumpiness, held share,
//! assembly (bells until first light) and star efficiency (lit share).
//!
//! `bounce::torsion_share` at the universal core (`M = 1`) is
//! a = (H + J) / (2H + J) with H = 2M, J = a_Kerr M^2: 1/2 at zero spin, 3/5
//! at the extremal spin; [`spin_of_share`] inverts it
//! onto the grid. [`Origin::of`] returns None where the founding or the
//! bounce cannot be built; the same seed gives the same origin.
//! Terms: torsion share, lumpiness (docs/glossary.md).

use crate::bounce::Bounce;
use xyl_core::certified::Encl;
use xyl_core::{Stream, Q};

/// The grid drawn values are fixed on.
pub const GRID_BITS: u32 = 40;

/// A uniform rational on (-1, 1) in steps of 2^-52, from the top 53 bits of a draw.
fn unit(rng: &mut Stream) -> Q {
    let span: i64 = 1 << 53;
    let x = i64::try_from(rng.next_u64() >> 11).unwrap_or(0);
    Q::new(2 * x - span + 1, span).unwrap_or(Q::ZERO)
}

/// A drawn value fixed as one exact rational: the 2^-40 grid point at or
/// below its enclosure's midpoint (within 2^-40 of the true draw).
fn fix(e: &Encl) -> Q {
    xyl_core::certified::down(&e.mid(), GRID_BITS)
}

/// One step of the 2^-40 grid.
fn grid_step() -> Q {
    (0..GRID_BITS).fold(Q::ONE, |a, _| a.checked_div(&Q::int(2)).unwrap_or(Q::ZERO))
}

/// A spin clamped strictly inside (0, 1) on the 2^-40 grid.
pub fn clamp_spin(spin: &Q) -> Q {
    let step = grid_step();
    xyl_core::certified::down(spin, GRID_BITS)
        .max(step.clone())
        .min(&Q::ONE - &step)
}

/// A universe's initial conditions and what follows from them.
#[derive(Clone, Debug)]
pub struct Origin {
    /// The parent black hole whose white hole this universe is, at the
    /// admitted spin.
    pub parent: Bounce,
    /// The matter's lumpiness: the beads' states above the mean over every
    /// bead's state, the cascade's contrast, in [0, 1).
    pub lumpiness: Q,
    /// The held share of the founded matter: the firmament's mass over the
    /// firmament's plus the beads', in [0, 1].
    pub held_share: Q,
    /// The assembly: the bells until the home galaxy's gas first lights
    /// (None: it never does).
    pub assembly_bells: Option<i64>,
    /// The star efficiency: the lit share of the home galaxy's gas, what
    /// collapses into clouds, by holding, in [0, 1].
    pub efficiency: Q,
}

/// The parent's spin whose torsion share is `a` (`bounce::spin_of_share` at
/// the universal core's mass, 1), clamped to the open grid. A share at or
/// above three fifths (the extremal spin's) gives the extremal spin.
pub fn spin_of_share(a: &Q) -> Q {
    crate::bounce::spin_of_share(&Q::ONE, a).map_or_else(|| clamp_spin(&Q::ONE), |j| clamp_spin(&j))
}

/// A uniform rational strictly inside (0, 1) on the 2^-40 grid.
fn open_unit(rng: &mut Stream) -> Q {
    clamp_spin(&fix(&Encl::exact(
        (&unit(rng) + &Q::ONE)
            .checked_div(&Q::int(2))
            .unwrap_or(Q::ZERO),
    )))
}

impl Origin {
    /// The origin of `seed`'s universe.
    pub fn of(seed: u64) -> Option<Origin> {
        let mut rng = Stream::derive(seed, &["big origin".into()]);
        // The parent has unit mass in the parent's units and a Kerr spin drawn
        // from the seed, floored at the least spin that keeps the firmament
        // whole; its horizon is read in the S^360 enclosure (the founding's
        // radius in r_g, circuit::anchor). The founding depends only on the
        // spin, through the torsion share, so the spin is registered first.
        crate::circuit::register_spin(&open_unit(&mut rng));
        let anchor = crate::circuit::anchor();
        let substrate = crate::cores::substrate_for(&anchor.spin)?;
        // The founding bounce is read at pulse phase zero.
        let parent = Bounce::of(&Q::ONE, &anchor.spin, &anchor.radius_rg, &Q::ZERO)?;
        // Lumpiness is the beads' state above the mean, summed over beads,
        // divided by the beads' total state.
        let mean = substrate.mean_state();
        let (above, all) = substrate
            .beads
            .iter()
            .fold((Q::ZERO, Q::ZERO), |(a, t), b| {
                let over = (&b.state - &mean).max(Q::ZERO);
                (&a + &over, &t + &b.state)
            });
        let lumpiness = above.checked_div(&all).unwrap_or(Q::ZERO);
        let held_share = substrate.held_share();
        Some(Origin {
            parent,
            lumpiness,
            held_share,
            assembly_bells: substrate.lighting.cools_in,
            efficiency: substrate.lighting.lit.clone(),
        })
    }

    /// When the first stars shone (rings after the origin): where the home
    /// galaxy's gas first cools to collapse, in rings of the bell
    /// (circuit::anchor), exact.
    pub fn first_stars(&self) -> Encl {
        Encl::exact(crate::circuit::anchor().first_stars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_share_law_inverts_on_the_grid() {
        // A spinless hole carries one half and does not bounce; the extremal
        // spin carries three fifths; the inversion lands within a grid step.
        let torsion_share = |spin: &Q| crate::bounce::torsion_share(&Q::ONE, spin);
        assert_eq!(torsion_share(&Q::ZERO), Q::new(1, 2).unwrap());
        assert_eq!(torsion_share(&Q::ONE), Q::new(3, 5).unwrap());
        for n in [1i64, 7, 50, 99] {
            let spin = Q::new(n, 100).unwrap();
            let back = spin_of_share(&torsion_share(&spin));
            assert!((&back - &spin).abs() <= grid_step(), "{spin} -> {back}");
        }
        // A share no spin reaches gives the extremal spin.
        assert_eq!(spin_of_share(&Q::ONE), clamp_spin(&Q::ONE));
        assert!(spin_of_share(&Q::new(1, 3).unwrap()) >= grid_step());
    }
}
