//! A star's disc frame: the founding piece at its reach, the infall's cut
//! there (`disc::star_cut`, held per piece) and the star's manifold reach;
//! and the star mass a piece's cut gives ([`piece_star_mass`],
//! [`first_star_mass`]). Reaches are galaxy rings, masses core masses (the
//! home galaxy's rings hold one core mass together, as in `inside_mass`).

use super::inside_mass;
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

/// The cut of the home galaxy's piece at reach `r` (galaxy rings), read once
/// per piece and cached: the piece's index, its infall's extent (galaxy
/// rings) and the cut. `None` where the founding is absent, no piece lies at `r` or the
/// piece does not light.
fn held_cut(r: &Q) -> Option<(usize, Q, crate::disc::StarCut)> {
    static HELD: std::sync::OnceLock<HeldCuts> = std::sync::OnceLock::new();
    let anchor = crate::circuit::anchor();
    let sub = crate::cores::substrate_for(&anchor.spin)?;
    let (piece, record) = sub.lighting.piece_at(r)?;
    record.2?;
    let map = HELD.get_or_init(Default::default);
    let cut = match map.lock().ok().and_then(|m| m.get(&piece).cloned()) {
        Some(c) => c,
        None => {
            let c = piece_cut(&sub, record);
            if let Ok(mut m) = map.lock() {
                m.insert(piece, c.clone());
            }
            c
        }
    };
    Some((piece, record.3.clone(), cut?))
}

/// The star mass of the home galaxy's piece at reach `r` (core masses): the
/// share of the piece's infall the star holds, `m_p = s c`, with `s` the
/// cut's held share (`disc::StarCut::star`) and `c` the infall's holding
/// (`disc::StarCut::cloud`, a share of the galaxy's rings, which hold one
/// core mass together). It is the most one star of the piece holds: the
/// initial mass function ([`super::imf`]) spreads the piece's stars below
/// it. `None` where the founding is absent, no piece lies at `r` or the
/// piece does not light.
pub fn piece_star_mass(r: &Q) -> Option<Q> {
    let (_, _, cut) = held_cut(r)?;
    Some(&cut.star * &cut.cloud)
}

/// The star mass of the first piece of `sub`'s lighting (core masses): the
/// share `m_p = s c` of the infall held by a star of the piece that lights
/// first (the least rings to light; on a tie the innermost), read on `sub`
/// itself so the founding's galaxy selection
/// (`circuit::Substrate::holds_a_star`) does not read the cached anchor.
/// Cached per torsion share (`Substrate::share`), which fixes the founding.
/// `None` where no piece lights.
pub fn first_star_mass(sub: &crate::circuit::Substrate) -> Option<Q> {
    type Held = std::sync::Mutex<std::collections::BTreeMap<String, Option<Q>>>;
    static HELD: std::sync::OnceLock<Held> = std::sync::OnceLock::new();
    let key = format!("{}", sub.share);
    let map = HELD.get_or_init(Default::default);
    if let Some(m) = map.lock().ok().and_then(|m| m.get(&key).cloned()) {
        return m;
    }
    let first = sub
        .lighting
        .pieces
        .iter()
        .filter_map(|p| p.2.map(|rings| (rings, p)))
        .min_by(|x, y| x.0.cmp(&y.0).then_with(|| x.1 .0.cmp(&y.1 .0)))
        .map(|(_, p)| p);
    let mass = first
        .and_then(|p| piece_cut(sub, p))
        .map(|cut| &cut.star * &cut.cloud);
    if let Ok(mut m) = map.lock() {
        m.insert(key, mass.clone());
    }
    mass
}

/// The disc's frame at reach `r` (galaxy rings) of the home galaxy for a
/// star of `mass` core masses (see [`DiscFrame`]). The cut is read once per
/// piece and cached; the gathering extent and manifold reach depend on the
/// star's mass. `None` where the founding is absent, no piece lies at `r` or
/// the piece does not light.
pub fn disc_frame(r: &Q, mass: &Q) -> Option<DiscFrame> {
    let (piece, infall, cut) = held_cut(r)?;
    // `extent = m / (c / w)`, at most the infall's extent `w`: the star's
    // mass (core masses) over the infall's holding per galaxy ring, `c` the
    // infall's holding in core masses. A star holding the whole held share
    // `s c` gathers the share `s` of the infall's extent.
    let lambda = cut
        .cloud
        .checked_div(&cut.infall.clone().max(Q::ratio(1, 1_000_000_000)))?;
    let extent = mass
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
        infall,
        extent,
        reach,
        star_share,
        tide: cut.tide.clone(),
        a_disc: cut.a_disc.clone(),
        cut,
    })
}

/// The star cut of the founding piece `record` (inner edge, outer edge,
/// rings to light, infall extent; galaxy rings) of `sub`'s lighting: the
/// bead whose span is the piece's, cut at the bead's reach about the
/// universal core, so the cut is one per piece. `None` where no bead has the
/// piece's span or the piece does not light.
fn piece_cut(
    sub: &crate::circuit::Substrate,
    record: &(Q, Q, Option<i64>, Q),
) -> Option<crate::disc::StarCut> {
    let bead = sub
        .beads
        .iter()
        .find(|b| b.span.0 == record.0 && b.span.1 == record.1)?;
    let swing = &Q::ONE + &sub.inbound_mean(crate::circuit::echo()).to_q();
    // The pulse's turning and the star's burn (its over-pressure), per echo
    // (two bells) in hydrogen units, at the piece's reach about the universal
    // core.
    let reach = Dy::from_q(&bead.reach).max(Dy::ZERO);
    let turning = (crate::stars::ring_per_bell(reach) * Dy::int(2)).to_q();
    let burn = (crate::stars::burn_per_bell(reach) * Dy::int(2)).to_q();
    crate::disc::star_cut(
        bead,
        &sub.unit(),
        &sub.held_share(),
        &swing,
        &turning,
        &burn,
    )
}
