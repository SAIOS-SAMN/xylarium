//! Galaxy formation: the founding cascade that sorts the kinds into shells,
//! and the home galaxy's gas cooled, collapsed and cut into stars and discs.
//!
//! Inputs: the founding's beads (`circuit::Bead`), the founding unit, the
//! held share and the inbound echo's swing `1 + A_in` (`circuit::found`);
//! nucleus holds and open frames from `compound` and masses from
//! `circuit::nucleus_weights`; `measure::share_turns` for the infall's
//! measure. Lengths are in galaxy rings, times in whole rings; in these units
//! the core's reach is one ring and its hold `H = 1 + held`.
//!
//! - Founding shells (`founding_shells`, `equitable_cascade`, `cascade`):
//!   each kind's reach `R_z = (K + s_i) / w_z / H` and whether the throat
//!   holds it, `R_z (1 + s_final) <= 1`; the latter two are diagnostics.
//! - Gas (`lighting`, `of`): each bead is a piece of the home galaxy's gas,
//!   its state over the unit spread over its span, its kinds at its pinch
//!   with agitation `a = (p - delta) / p`, `p` the throat's push
//!   (`circuit::throat_push`, 1/2) and `delta = H / (R + H)`. Outside one
//!   galaxy ring the gas is a circle in the plane (n = 1), within it the
//!   bulge's ball (n = 2). Gas inflow from the ocean is not modelled.
//! - Cooling: two channels per ring, the open frames' emission
//!   `e = open / weight` (one halving every `ceil(1 / e)` rings) and the
//!   vacuum's halving (`(r / (r + h))^k <= 1/2`).
//! - Collapse: an infall of extent `w` at reach `R` gathers
//!   `m(w) = (lambda + mu) W share_n(theta)`, `theta` the deficit angle
//!   `w / (w + R)` of the half turn, and grips it by `m / (w + m)`. It holds
//!   where the grip exceeds its agitation, the shear `H w / (R (R + w))` and
//!   the pulse's heat (`1 / (pi (R + 1))` times the swing and the grip, over
//!   what the piece sheds per ring). Extents are read on a ladder of halvings
//!   of the piece's width; the piece cools until its residue is under the
//!   best extent's spare, then fragments at the largest extent that holds.
//! - The star tier's cut (`hold_at_heat`): each kind carried to
//!   `l_z = p P_0 (e_z / e_mean)(w_mean / m_z) + add` stands at
//!   `R_z = l_z / h`; the held set is the largest fixed point of
//!   `h / (R_z + h) >= p`, iterated from all held. The rest goes to the disc
//!   (`disc::star_disc`).
//!
//! Outputs: `Lighting` (rings to first light, lit share, per-piece record;
//! fed into the founding), `Formation` with one `Piece` per bead (read by
//! `xylarium observe`), `Hold`, `Kind`, `Stack`, `EquitableReading`.
//! Invariants: every value is an exact `Q`, rounded down to the 2^-64 grid
//! where noted, with `pi` and `ln 2` as certified midpoints; empty beads,
//! spans and mixes yield `None` or empty results; results are deterministic
//! (the parallel meetings in `founding_shells` are collected in order).
//!
//! Terms: cascade, lighting, piece (docs/glossary.md).
//!
//! Layout: `gas` (cooling, lighting, collapse), `shells` (the founding
//! cascade and shells), `hold` (the star tier's cut); this file holds the
//! nucleus weights, the kinds and the formation read from them.

mod gas;
mod hold;
mod shells;

pub(crate) use gas::{cooling, lights, vacuum_halving, Cooling};
pub use gas::{lighting, Lighting};
pub use hold::{hold_at_heat, Hold};
pub use shells::{cascade, equitable_cascade, founding_shells, EquitableReading, Stack};

use crate::circuit::Bead;
use crate::disc::{star_disc, World};
use crate::elements::Element;
use xyl_core::Q;

/// `x / 2`.
pub(crate) fn half(x: &Q) -> Q {
    x.checked_div(&Q::int(2)).unwrap_or(Q::ZERO)
}

/// The throat's push as an exact rational: the vacuum's share where reach
/// and hold coincide (circuit::throat_push).
pub(crate) fn throat_push() -> Q {
    crate::circuit::throat_push().to_q()
}

/// A nucleus's weight (hold) from the atom tier: 3Z - unpaired; for a
/// cluster, 3 charge - open from its parts (compound.rs).
pub fn weight(z: u32) -> Q {
    Q::int(i64::from(crate::compound::hold(z)))
}

/// A nucleus's mass in nucleons, Z + Q (weights.rs). Gravity and inertia use
/// this, independent of kind; chemistry uses the hold (`weight`, the
/// electrons' frames), which is also the unit amounts are counted in.
pub fn mass_weight(z: u32) -> Q {
    crate::circuit::nucleus_weights(z).mass()
}

/// What a nucleus radiates per ring: its open frames over its weight.
pub fn emission(z: u32) -> Q {
    Q::int(i64::from(crate::compound::open(z)))
        .checked_div(&weight(z))
        .unwrap_or(Q::ZERO)
}

/// One element's share of the galaxy, by count and by holding.
#[derive(Clone, Debug)]
pub struct Kind {
    /// Atomic number of the element.
    pub z: u32,
    /// Hold of one nucleus, 3Z - unpaired (see [`weight`]).
    pub weight: Q,
    /// Share of the galaxy's grains' holding (shares sum to 1).
    pub share: Q,
}

/// A piece of the gas, a bead read one tier down, and what it forms.
#[derive(Clone, Debug)]
pub struct Piece {
    /// Inner edge of the piece (galaxy rings): its bead's span.
    pub from: Q,
    /// Outer edge of the piece (galaxy rings).
    pub to: Q,
    /// The pinch the piece's kinds stand at (galaxy rings): its bead's reach.
    pub reach: Q,
    /// Holding per light-ring of reach.
    pub lambda: Q,
    /// Holding-weighted mean agitation of the shells covering the piece.
    pub agitation: Q,
    /// Holding-weighted mean emission of the shells covering the piece (per
    /// ring).
    pub emission: Q,
    /// Rings until it cools to collapse (None: it never does).
    pub cools_in: Option<i64>,
    /// Infall width (the smallest holding width, a cloud), each cloud's
    /// holding, and how many fit.
    pub infall: Q,
    /// Holding of each cloud: `lambda` times the infall width.
    pub cloud: Q,
    /// Number of clouds that fit in the piece (at least 1 once it collapses;
    /// 0 when it never does).
    pub clouds: i64,
    /// Agitation at collapse (cooled by both channels).
    pub cooled: Q,
    /// The galaxy's tide at the piece, delta_gal = H / (R + H): the vacuum's
    /// share there, the push the pulse delivers.
    pub tide: Q,
    /// The infall's agitation at ignition: cooled residue plus the pulse's
    /// heat, the ring the bell brings at the reach taken by the infall's
    /// grip over what it sheds per ring, swung by the inbound echo.
    pub heated: Q,
    /// The star tier's cut: the share of each infall its core holds (the
    /// star), and the open share of the infall and of its disc.
    pub star_share: Q,
    /// Open share of the infall's matter.
    pub infall_open: Q,
    /// Open share of the matter released to the disc.
    pub disc_open: Q,
    /// The disc's worlds, each with its planetary core's cut, and the belts'
    /// share.
    pub worlds: Vec<World>,
    /// Share of the disc left as belts rather than worlds.
    pub belts: Q,
    /// The stars the clouds make: each one's holding and the count (all
    /// clouds together).
    pub star: Q,
    /// Number of stars: the cloud count when a cloud's star holds anything,
    /// else 0.
    pub stars: Q,
}

/// A galaxy's formation.
#[derive(Clone, Debug)]
pub struct Formation {
    /// The push share at the cut, the throat's.
    pub push: Q,
    /// Held share of the founded matter: the firmament's over the whole.
    pub held: Q,
    /// The pieces of gas, the beads one tier down, in bead order.
    pub pieces: Vec<Piece>,
}

/// The kinds of matter in a galaxy's manifest, by count from the element
/// table, weighted by the atom tier, heaviest first.
pub fn kinds(elements: &[Element]) -> Vec<Kind> {
    let mut ks: Vec<(u32, Q, Q)> = elements
        .iter()
        .filter(|e| e.abundance.signum() > 0)
        .map(|e| (e.z, weight(e.z), e.abundance.clone()))
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

/// The galaxy's formation from the founding's beads: each piece's cooling
/// and, where it lights, the star tier's cut on its infalls and their
/// discs. `unit`, `held` and `swing` as for [`lighting`].
pub fn of(beads: &[Bead], unit: &Q, held: &Q, swing: &Q) -> Formation {
    let h = &Q::ONE + held;
    let p_core = throat_push();
    let mut pieces = Vec::new();
    for bead in beads {
        let (from, to) = (bead.span.0.clone(), bead.span.1.clone());
        let Some(c) = cooling(bead, unit, &h, swing) else {
            continue;
        };
        let lit = lights(&c);
        let Cooling {
            mix,
            lambda,
            agitation,
            emission,
            cools_in,
            infall,
            cloud,
            clouds,
            cooled,
            tide,
            heated,
        } = c;
        // the star tier's cut at the core's own push p_core and P_0 = 1: the
        // tide does not change the push, it enters the disc only as heat
        let cut = if lit {
            Some(hold_at_heat(&mix, &p_core, &Q::ZERO, &Q::ZERO, &Q::ONE))
        } else {
            None
        };
        let (star_share, infall_open, disc_open) = cut
            .as_ref()
            .map(|c| (c.held.clone(), c.open.clone(), c.rest_open.clone()))
            .unwrap_or((Q::ZERO, Q::ZERO, Q::ZERO));
        // the released remainder forms the disc, heated by the tide; each
        // world takes its own planetary core's cut (disc::star_disc)
        let (worlds, belts) = match &cut {
            Some(c) if star_share < Q::ONE => star_disc(&star_share, &tide, &c.rest),
            _ => (Vec::new(), Q::ZERO),
        };
        let star = &cloud * &star_share;
        let stars = if star.signum() > 0 {
            Q::int(clouds)
        } else {
            Q::ZERO
        };
        pieces.push(Piece {
            from,
            to,
            reach: bead.reach.clone(),
            lambda,
            agitation,
            emission,
            cools_in,
            infall,
            cloud,
            clouds,
            cooled,
            tide,
            heated,
            star_share,
            infall_open,
            disc_open,
            worlds,
            belts,
            star,
            stars,
        });
    }
    Formation {
        push: p_core,
        held: held.clone(),
        pieces,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece_holdings_sum_to_the_galaxy_gas() {
        let bang = crate::genesis::bang();
        let s = crate::cores::substrate(7, bang).unwrap();
        let f = of(&s.beads, &s.unit(), &s.held_share(), &Q::ONE);
        // each piece holds its bead's state over the unit, so the holdings
        // sum to 1; the held share and the push lie in [0, 1]
        let gas = f
            .pieces
            .iter()
            .fold(Q::ZERO, |a, p| &a + &(&p.lambda * &(&p.to - &p.from)));
        assert!(
            (&gas - &Q::ONE).abs() < Q::new(1, 1_000_000).unwrap(),
            "the gas {gas}"
        );
        assert!(f.held >= Q::ZERO && f.held <= Q::ONE && f.push < Q::ONE);
        let l = &s.lighting;
        assert!(l.lit >= Q::ZERO && l.lit <= Q::ONE, "lit {}", l.lit);
        assert_eq!(l.pieces.len(), f.pieces.len());
        // a piece has a lighting time exactly when it fragments into clouds
        for (p, piece) in l.pieces.iter().zip(&f.pieces) {
            assert_eq!(p.2.is_some(), piece.clouds > 0);
        }
    }
}
