//! A people's numbers, from physics (docs/civilizations.md): growth is
//! derived, never read from a game table.
//!
//! - Body: m_b = M_MIND x size (from life history and the world's gravity).
//! - Metabolism: the unit body's (Kleiber) scaled by the size share
//!   2s / (1 + s), s = m_b / M_MIND, at the chemistry's pace.
//! - Food: the free energy reaching the medium over the living area, of which
//!   a biosphere fixes a share and passes a tenth up one food-chain link.
//! - Capacity: N_cap = food / B, scaled by the share of the year the world
//!   grows (a tilted world has winters, a locked one a dark side).
//! - Births and deaths: a generation is the unit generation scaled by
//!   2s / (1 + s); a people grows toward N_cap logistically at
//!   one e-fold per its own generation. The logistic is solved exactly
//!   over any span, so results do not depend on step size.

use crate::cosmic::Traits;
use xyl_core::dyadic::Dy;
use xyl_cosmos::life::{growing_share, Body};
use xyl_cosmos::worlds::{body_pace, Ctx, Medium};

fn n(x: i64) -> Dy {
    Dy::int(x)
}

/// The rulers growth is measured against. Declared rulers: each is a
/// measured value kept as a ruler and listed in the emergence register.
pub struct Rulers {
    /// A mind-bearing body (kg): life.rs's M_MIND.
    pub m_mind: Dy,
    /// The unit body's generation (years). Declared ruler, pending its
    /// derivation from the body's own life history: 14.2 yr, a measured
    /// value held as a ruler; every people's generation scales it by
    /// 2s / (1 + s).
    pub generation_yr: Dy,
    /// The share of free energy a biosphere fixes (primary production).
    pub eta_primary: Dy,
    /// What one link of a food chain passes on (Lindeman).
    pub trophic: Dy,
    /// The flux unit (W/m^2).
    pub flux_w: Dy,
}

impl Default for Rulers {
    fn default() -> Rulers {
        Rulers {
            m_mind: n(10),
            generation_yr: Dy::ratio(142, 10),
            eta_primary: Dy::ratio(15, 1000),
            trophic: Dy::ratio(1, 10),
            flux_w: n(340),
        }
    }
}

/// A people's body mass (kg).
pub fn body_kg(traits: &Traits, r: &Rulers) -> Dy {
    r.m_mind * traits.size
}

/// The unit body's metabolism (W): the mind-bearing body's, held as a ruler
/// in xyl-cosmos life.rs (19.1 W).
pub fn unit_metabolism_w() -> Dy {
    xyl_cosmos::life::unit_metabolism_w()
}

/// A body's metabolism (W): the unit body's, scaled by the size share
/// 2s / (1 + s), at its chemistry's own pace.
pub fn metabolism_w(traits: &Traits, ctx: &Ctx, r: &Rulers) -> Dy {
    let s = (body_kg(traits, r) / r.m_mind.max(Dy::sci(1, -12))).max(Dy::ZERO);
    unit_metabolism_w() * (Dy::int(2) * s / (Dy::ONE + s)) * body_pace(traits.solvent, ctx)
}

/// Food power the world offers one link up the biosphere (W), averaged over
/// the year (its growing share).
pub fn food_w(med: &Medium, body: &Body, r: &Rulers) -> Dy {
    let season = if body.locked {
        Dy::ratio(1, 2)
    } else {
        growing_share(body.tilt)
    };
    med.energy.max(Dy::ZERO)
        * r.flux_w
        * body.area_m2()
        * med.band
        * r.eta_primary
        * r.trophic
        * season
}

/// A generation (years): the unit body's (`Rulers::generation_yr`) scaled by
/// the size share 2s / (1 + s).
pub fn generation_yr(traits: &Traits, r: &Rulers) -> Dy {
    let s = (body_kg(traits, r) / r.m_mind.max(Dy::sci(1, -12))).max(Dy::ZERO);
    r.generation_yr * (Dy::int(2) * s / (Dy::ONE + s))
}

/// Its natural growth rate (per year): one e-fold per its own generation.
pub fn growth_per_yr(traits: &Traits, r: &Rulers) -> Dy {
    Dy::ONE / generation_yr(traits, r).max(Dy::sci(1, -9))
}

/// Numbers after `years` from `now`, toward `cap` (exact logistic:
/// K / (1 + (K/N - 1) e^(-r t))). Above capacity it decays back the same way.
pub fn grow(now: Dy, cap: Dy, rate: Dy, years: Dy) -> Dy {
    if cap.signum() <= 0 || now.signum() <= 0 {
        return Dy::ZERO;
    }
    let x = (rate * years).min(n(200));
    // zero span: return unchanged (the closed form would round)
    if x.signum() <= 0 {
        return now;
    }
    cap / (Dy::ONE + (cap / now - Dy::ONE) * (-x).exp())
}

/// Power a world offers a people's works, and what its bodies already turn
/// (W). Until seated minds direct capture of the world's flows, a people
/// commands only its bodies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Power {
    /// Its star's light over its world's whole surface (the year's mean flux).
    pub starlight_w: Dy,
    /// Its world's own heat flowing out.
    pub heat_w: Dy,
    /// Its bodies' metabolism, all of them.
    pub bodies_w: Dy,
}

/// A world's whole surface (m^2), not only its living area.
fn surface_m2(body: &Body) -> Dy {
    let unit_r = Dy::from_q(&xyl_cosmos::package::Si::world_radius_m());
    let r_m = body.radius * unit_r;
    // linear area rule against the unit world: 4 pi r R_unit
    n(4) * Dy::PI * r_m * unit_r
}

/// The power available to a people of `numbers` bodies on world `body` in state `state` (W): its star's light over the whole surface, the world's heat flow over it, and its bodies' total metabolism under `traits`, `ctx` and rulers `r`.
pub fn power(
    state: &xyl_cosmos::worlds::State,
    body: &Body,
    traits: &Traits,
    numbers: Dy,
    ctx: &Ctx,
    r: &Rulers,
) -> Power {
    let area = surface_m2(body);
    Power {
        starlight_w: state.flux.max(Dy::ZERO) * r.flux_w * area,
        heat_w: state.q.max(Dy::ZERO) * area,
        bodies_w: numbers.max(Dy::ZERO) * metabolism_w(traits, ctx, r),
    }
}

/// The world's starlight over its whole surface plus its internal heat flow
/// (W): the power a people's own producers may capture (civ::housing
/// `feeds_w`).
pub fn world_w(state: &xyl_cosmos::worlds::State, body: &Body, r: &Rulers) -> Dy {
    let area = surface_m2(body);
    (state.flux.max(Dy::ZERO) * r.flux_w + state.q.max(Dy::ZERO)) * area
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cosmic::{Shape, Strategy};
    use xyl_cosmos::worlds::{Solvent, Species, Where};

    fn traits(size: Dy, strategy: Strategy) -> Traits {
        Traits {
            strategy,
            solvent: Solvent::Volatile(Species::H2O),
            dark: false,
            at: Where::Surface,
            body_t: n(310),
            sense: "light",
            form: "walking",
            hands: "grasping limbs",
            social: Dy::ratio(1, 2),
            shape: Shape::Society,
            size,
        }
    }

    #[test]
    fn growth_is_logistic_toward_what_a_world_houses() {
        let r = Rulers::default();
        let t = traits(Dy::ONE, Strategy::K);
        let cap = n(1_000_000);
        // the logistic: towards the capacity, never past it, from either side
        let rate = growth_per_yr(&t, &r);
        let later = grow(n(500), cap, rate, n(1000));
        assert!(later > n(500) && later <= cap);
        assert!(grow(cap * n(2), cap, rate, n(1000)) < cap * n(2));
        assert_eq!(grow(n(500), cap, rate, Dy::ZERO), n(500));
        // one e-fold per its own generation: a larger body breeds slower
        assert!(growth_per_yr(&traits(n(8), Strategy::K), &r) < rate);
    }
}
