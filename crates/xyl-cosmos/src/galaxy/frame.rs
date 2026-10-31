//! A star's disc frame: the founding piece at its reach, the infall's cut
//! there (`disc::star_cut`, held per piece) and the star's manifold reach.

use super::{inside_mass, stellar_mass};
use xyl_core::certified::Encl;
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// The chain's frame for a star's disc at reach `r` (kpc) of the home
/// galaxy: its piece of the founding, the infall's extent there (kpc), the
/// share of the infall the star holds, the galaxy's tide at the pinch (the
/// core-star tie) and the disc's agitation (`disc::star_cut`). The cut is
/// read once per piece and cached.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscFrame {
    /// The piece's index among the founding's beads.
    pub piece: usize,
    /// The infall's extent (kpc), the extent of the star-forming cloud.
    pub infall_kpc: Q,
    /// The star's gathering extent (kpc): the star's mass over the infall's
    /// line density, at most the infall's extent.
    pub extent_kpc: Q,
    /// The star's manifold reach (kpc), the outer edge of its worlds: the arc
    /// of the star's circle about the core held by the star's mass
    /// (`manifold::reach`, the deficit share `m / (M(<r) + m)` of `2 pi r`).
    pub reach_kpc: Q,
    /// The star's cut of the infall (`disc::star_cut`).
    pub cut: crate::disc::StarCut,
    /// The share of the manifold the star holds: the gathering extent over
    /// the manifold reach, at most 1 (a star whose gathering fills the
    /// manifold leaves no matter for worlds).
    pub star_share: Q,
    /// The galaxy's tide at the pinch, `H / (R + H)`.
    pub tide: Q,
    /// The disc's agitation (`disc::StarCut::a_disc`).
    pub a_disc: Q,
}

/// The cuts read so far, by piece index (`None`: the piece does not light).
type HeldCuts = std::sync::Mutex<std::collections::BTreeMap<usize, Option<crate::disc::StarCut>>>;

/// The disc's frame at reach `r` (kpc) of the home galaxy for a star of
/// `mass` suns (see [`DiscFrame`]). The cut is read once per piece and
/// cached; the gathering extent and manifold reach depend on the star's mass.
/// `None` where the founding is absent, no piece lies at `r` or the piece
/// does not light.
pub fn disc_frame(r: &Q, mass: &Q) -> Option<DiscFrame> {
    static HELD: std::sync::OnceLock<HeldCuts> = std::sync::OnceLock::new();
    let anchor = crate::circuit::anchor();
    let sub = crate::cores::substrate_for(&anchor.spin)?;
    let kpc_per_ring = anchor
        .home_rim_kpc
        .checked_div(&anchor.rim)
        .unwrap_or(Q::ONE)
        .max(Q::ratio(1, 1_000_000_000));
    let rings = r.checked_div(&kpc_per_ring).unwrap_or(Q::ZERO);
    let (piece, record) = sub.lighting.piece_at(&rings)?;
    record.2?;
    let map = HELD.get_or_init(Default::default);
    let cut = match map.lock().ok().and_then(|m| m.get(&piece).cloned()) {
        Some(c) => c,
        None => {
            let c = piece_cut(&sub, piece, r)?;
            if let Ok(mut m) = map.lock() {
                m.insert(piece, c.clone());
            }
            c
        }
    };
    let cut = cut?;
    // `extent = (m / M_stars) / (cloud / infall)`, at most the infall: the
    // star's mass in holding units (the galaxy's stars hold one unit) over
    // the infall's holding per ring.
    let m_s = mass.checked_div(&stellar_mass())?;
    let lambda = cut
        .cloud
        .checked_div(&cut.infall.clone().max(Q::ratio(1, 1_000_000_000)))?;
    let extent = m_s
        .checked_div(&lambda.max(Q::ratio(1, 1_000_000_000_000)))?
        .min(cut.infall.clone());
    let extent_kpc = &extent * &kpc_per_ring;
    // Manifold reach (galaxy rings): the deficit share m / (M(<r) + m) of
    // the circle 2 pi r about the core (`manifold::reach`).
    let inside = inside_mass(r);
    let ratio = mass.checked_div(&inside.max(Q::ratio(1, 1_000_000)))?;
    let reach = crate::manifold::reach(&Encl::exact(rings.clone()), &ratio).mid();
    let star_share = extent
        .checked_div(&reach.clone().max(Q::ratio(1, 1_000_000_000_000)))
        .unwrap_or(Q::ONE)
        .min(Q::ONE);
    Some(DiscFrame {
        piece,
        infall_kpc: &record.3 * &kpc_per_ring,
        extent_kpc,
        reach_kpc: &reach * &kpc_per_ring,
        star_share,
        tide: cut.tide.clone(),
        a_disc: cut.a_disc.clone(),
        cut,
    })
}

/// The star cut of founding piece `piece` for a star at `r` kpc: `Some(None)`
/// where the piece does not light, `None` where the piece is missing.
fn piece_cut(
    sub: &crate::circuit::Substrate,
    piece: usize,
    r: &Q,
) -> Option<Option<crate::disc::StarCut>> {
    let bead = sub.beads.get(piece)?;
    let swing = &Q::ONE + &sub.inbound_mean(crate::circuit::echo()).to_q();
    // The pulse's turning and the star's burn (its over-pressure), per echo
    // (two bells) in hydrogen units, at the star's reach about the universal
    // core.
    let reach = crate::stars::reach_rings(Dy::from_q(r));
    let turning = (crate::stars::ring_per_bell(reach) * Dy::int(2)).to_q();
    let burn = (crate::stars::burn_per_bell(reach) * Dy::int(2)).to_q();
    Some(crate::disc::star_cut(
        bead,
        &sub.unit(),
        &sub.held_share(),
        &swing,
        &turning,
        &burn,
    ))
}
