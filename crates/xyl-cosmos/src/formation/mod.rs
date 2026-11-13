//! Galaxy formation: the founding shells that sort the kinds about the
//! throat, and the home galaxy's gas cooled, collapsed and lit.
//!
//! Inputs: the founding's beads (`circuit::Bead`), the founding unit, the
//! held share and the inbound echo's swing `1 + A_in`
//! (`circuit::Substrate::echo_swing`); nucleus holds and open states from
//! `compound` and masses from `circuit::nucleus_weights`;
//! `measure::share_turns` for the infall's measure. Lengths are in galaxy
//! rings, times in whole rings; in these units the core's reach is one ring
//! and its hold `H = 1 + held`.
//!
//! - Release (`spin_orbit`, T17): the core's bounces release its open
//!   kinds cohort by cohort, each climbing `ln t += N* ln(D(s) D(n)^A)` per
//!   pinch interval `N* = 3 (4 + a) / (2a)` until it crosses, the kinds
//!   pressed by the near face inverted out by the far face; the closed
//!   kinds are kept on both faces (T16).
//! - Founding shells (`founding_shells`): each released layer leaves from
//!   the kept hold's horizon `2H` and stands after its cohort's pinch
//!   interval at the reach rate's closed form,
//!   `(r - r0) + 2m ln(r / r0) = 2 (h - m) N*`, swinging by `h / (r + h)`;
//!   the universal table holds every kind's rest point at the bounce,
//!   `Q* / Z = D(R)` from its electron phase at its capture phase (E018,
//!   `circuit::neutrons_at`).
//! - Gas (`lighting`): each bead is a piece of the home galaxy's gas,
//!   its state over the unit spread over its span, its kinds at its pinch
//!   with agitation `a = (p - delta) / p`, `p` the throat's push
//!   (`circuit::throat_push`, 1/2) and `delta = H / (R + H)`. Outside one
//!   galaxy unit the gas is a circle in the plane (n = 1), within it the
//!   bulge's ball (n = 2). Gas inflow from the ocean is not modelled.
//! - Cooling: two channels per ring, the open states' emission
//!   `e = open / hold` (one halving every `ceil(1 / e)` rings) and the
//!   vacuum's halving (`(r / (r + h))^k <= 1/2`).
//! - Collapse: an infall of extent `w` at reach `R` gathers
//!   `m(w) = (lambda + mu) W share_n(theta)`, `theta` the deficit angle
//!   `w / (w + R)` of the half turn, and grips it by `m / (w + m)`. It holds
//!   where the grip exceeds its agitation, the shear `H w / (R (R + w))` and
//!   the pulse's heat (`pulse_heat`: `1 / (pi (R + 1))` times the swing and
//!   the grip, over what the piece sheds per ring). Extents are read on a
//!   ladder of halvings of the piece's width; the piece cools until its
//!   residue is under the best extent's spare, then fragments at the largest
//!   extent that holds.
//! - The star tier's cut (`hold_at`): each kind carried to
//!   `l_z = p P_0 (e_z / e_mean)(w_mean / m_z) + add` stands at
//!   `R_z = l_z / h`; the held set is the largest fixed point of
//!   `h / (R_z + h) >= p`, iterated from all held. The star's cut of a lit
//!   piece is `disc::star_hold`.
//!
//! Outputs: `Lighting` (rings to first light, lit share, per-piece record;
//! fed into the founding), `Hold`, `Kind`.
//! Invariants: every value is an exact `Q`, cut toward zero to the 2^-64 grid
//! where noted (the release's logarithms are dyadic readings, the reaches
//! and rest points cut toward zero at 2^-64), with `pi` and
//! `ln 2` as certified midpoints; empty beads, spans and mixes yield `None`
//! or empty results; results are deterministic.
//!
//! Terms: lighting, piece (docs/glossary.md).
//!
//! Layout: `gas` (cooling, lighting, collapse), `shells` (the core's
//! release and the founding shells), `hold` (the star tier's cut); this file holds the nucleus
//! masses, emissions and the kinds.

mod gas;
mod hold;
mod shells;

pub(crate) use gas::{cooling, lights, pulse_heat, vacuum_halving};
pub use gas::{lighting, Lighting};
pub use hold::{hold_at, Hold};
pub use shells::{founding_shells, spin_orbit, Cohort, Shell, SpinOrbit};

use crate::elements::Element;
use xyl_core::Q;

/// `x / 2`.
pub(crate) fn half(x: &Q) -> Q {
    x.checked_div(&Q::int(2)).unwrap_or(Q::ZERO)
}

/// A nucleus's mass in nucleons, Z + Q (weights.rs). Gravity and inertia use
/// this, independent of kind; chemistry uses the hold (`compound::electron_hold`, the
/// electrons' states), which is also the unit amounts are counted in.
pub fn mass_weight(z: u32) -> Q {
    crate::circuit::nucleus_weights(z).mass()
}

/// What a nucleus radiates per ring: its open states over its hold.
pub fn emission(z: u32) -> Q {
    Q::int(i64::from(crate::compound::open(z)))
        .checked_div(&Q::int(i64::from(crate::compound::electron_hold(z))))
        .unwrap_or(Q::ZERO)
}

/// One element's share of the galaxy, by count and by holding.
#[derive(Clone, Debug)]
pub struct Kind {
    /// Atomic number of the element.
    pub z: u32,
    /// Hold of one nucleus, 3Z - unpaired (`compound::electron_hold`).
    pub weight: Q,
    /// Share of the galaxy's grains' holding (shares sum to 1).
    pub share: Q,
}

/// The kinds of matter in a galaxy's manifest, by count from the element
/// table, weighted by the atom tier, heaviest first.
pub fn kinds(elements: &[Element]) -> Vec<Kind> {
    let mut ks: Vec<(u32, Q, Q)> = elements
        .iter()
        .filter(|e| e.abundance.signum() > 0)
        .map(|e| {
            (
                e.z,
                Q::int(i64::from(crate::compound::electron_hold(e.z))),
                e.abundance.clone(),
            )
        })
        .collect();
    let total = ks.iter().fold(Q::ZERO, |a, (_, w, n)| &a + &(w * n));
    ks.sort_by(|a, b| b.1.cmp(&a.1));
    ks.into_iter()
        .map(|(z, w, n)| Kind {
            z,
            share: (&w * &n).checked_div(&total).unwrap_or(Q::ZERO),
            weight: w,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece_holdings_sum_to_the_galaxy_gas() {
        let origin = crate::genesis::origin();
        let s = crate::cores::substrate(origin).unwrap();
        let (unit, held) = (s.unit(), s.held_share());
        let h = &Q::ONE + &held;
        // Each piece holds its bead's state over the unit, so the holdings
        // sum to 1; the held share lies in [0, 1] and the lit share too.
        let gas = s
            .beads
            .iter()
            .filter_map(|b| cooling(b, &unit, &h, &Q::ONE).map(|c| (b, c)))
            .fold(Q::ZERO, |a, (b, c)| {
                &a + &(&c.lambda * &(&b.span.1 - &b.span.0))
            });
        assert!(
            (&gas - &Q::ONE).abs() < Q::new(1, 1_000_000).unwrap(),
            "the gas {gas}"
        );
        assert!(held >= Q::ZERO && held <= Q::ONE);
        let l = &s.lighting;
        assert!(l.lit >= Q::ZERO && l.lit <= Q::ONE, "lit {}", l.lit);
    }
}
