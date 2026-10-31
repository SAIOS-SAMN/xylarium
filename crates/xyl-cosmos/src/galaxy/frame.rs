//! A star's disc frame: the founding piece at its reach, the infall's cut
//! there (`disc::star_cut`, held per piece) and the star's manifold reach.
//! Reaches are galaxy rings, masses core masses; the star's mass arrives in
//! suns and is converted at the boundary (`core_masses_of_suns`).

use super::{core_masses_of_suns, inside_mass, stellar_mass};
use xyl_core::certified::Encl;
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// The chain's frame for a star's disc at reach `r` (galaxy rings) of the
/// home galaxy: its piece of the founding, the infall's extent there, the
/// share of the infall the star holds, the galaxy's tide at the pinch (the
/// core-star tie) and the disc's agitation (`disc::star_cut`). The cut is
/// read once per piece and cached.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscFrame {
    /// The piece's index among the founding's beads.
    pub piece: usize,
    /// The infall's extent (galaxy rings), the extent of the star-forming
    /// cloud.
    pub infall: Q,
    /// The star's gathering extent (galaxy rings): the star's mass over the
    /// infall's line density, at most the infall's extent.
    pub extent: Q,
    /// The star's manifold reach (galaxy rings), the outer edge of its
    /// worlds: the arc
    /// of the star's circle about the core held by the star's mass
    /// (`manifold::reach`, the deficit share `m / (M(<r) + m)` of `2 pi r`).
    pub reach: Q,
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

/// The disc's frame at reach `r` (galaxy rings) of the home galaxy for a
/// star of `mass` suns (see [`DiscFrame`]; the mass is read in core masses
/// through the boundary `core_masses_of_suns`). The cut is read once per piece and
/// cached; the gathering extent and manifold reach depend on the star's mass.
/// `None` where the founding is absent, no piece lies at `r` or the piece
/// does not light.
pub fn disc_frame(r: &Q, mass: &Q) -> Option<DiscFrame> {
    static HELD: std::sync::OnceLock<HeldCuts> = std::sync::OnceLock::new();
    let anchor = crate::circuit::anchor();
    let sub = crate::cores::substrate_for(&anchor.spin)?;
    let (piece, record) = sub.lighting.piece_at(r)?;
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
    let mass = core_masses_of_suns(mass);
    let m_s = mass.checked_div(&stellar_mass())?;
    let lambda = cut
        .cloud
        .checked_div(&cut.infall.clone().max(Q::ratio(1, 1_000_000_000)))?;
    let extent = m_s
        .checked_div(&lambda.max(Q::ratio(1, 1_000_000_000_000)))?
        .min(cut.infall.clone());
    // Manifold reach (galaxy rings): the deficit share m / (M(<r) + m) of
    // the circle 2 pi r about the core (`manifold::reach`).
    let inside = inside_mass(r);
    let ratio = mass.checked_div(&inside.max(Q::ratio(1, 1_000_000)))?;
    let reach = crate::manifold::reach(&Encl::exact(r.clone()), &ratio).mid();
    let star_share = extent
        .checked_div(&reach.clone().max(Q::ratio(1, 1_000_000_000_000)))
        .unwrap_or(Q::ONE)
        .min(Q::ONE);
    Some(DiscFrame {
        piece,
        infall: record.3.clone(),
        extent,
        reach,
        star_share,
        tide: cut.tide.clone(),
        a_disc: cut.a_disc.clone(),
        cut,
    })
}

/// The star cut of founding piece `piece` for a star at `r` galaxy rings: `Some(None)`
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
    let reach = Dy::from_q(r).max(Dy::ZERO);
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
