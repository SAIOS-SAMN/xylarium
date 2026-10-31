//! A people's bodies, food, power and numbers, derived from its traits and
//! its world rather than read from a game table (docs/civilizations.md).
//!
//! Inputs: `cosmic::Traits` (size, solvent), the world's `Body`, `State`
//! and `Medium` (`xyl_cosmos::life`, `xyl_cosmos::worlds`), and [`Rulers`],
//! the declared constants (mind-bearing body mass, unit generation, primary
//! production share, trophic share, flux unit).
//!
//! - Body ([`body_kg`]): m_b = M_MIND x size (kg).
//! - Metabolism ([`metabolism_w`]): the unit body's (W) scaled by the size
//!   share 2s / (1 + s), s = m_b / M_MIND, at the solvent's body pace.
//! - Food ([`food_w`]): the medium's free energy over the living area, of
//!   which the biosphere fixes `eta_primary` and passes `trophic` up one
//!   link, times the growing share of the year (one half when locked).
//! - Generation ([`generation_yr`]): the unit generation scaled by
//!   2s / (1 + s); [`growth_per_yr`] is one e-fold per generation.
//! - Numbers ([`grow`]): the logistic K / (1 + (K/N - 1) e^(-r t)) in
//!   closed form, so the result does not depend on step size; above
//!   capacity it decays back the same way.
//! - Power ([`power`], [`world_w`]): starlight and internal heat over the
//!   world's whole surface, and the bodies' total metabolism (W).
//!
//! What a world houses (capacity) is computed from these in civ/housing.rs.

use crate::cosmic::Traits;
use xyl_core::dyadic::Dy;
use xyl_cosmos::life::{growing_share, Body};
use xyl_cosmos::worlds::{body_pace, Ctx, Medium};

/// The rulers growth is measured against. Each is a declared ruler, a
/// measured value held until it is derived, and is listed in
/// docs/roadmap.md (Declared rulers).
pub struct Rulers {
    /// The mass of a mind-bearing body (kg), the value of `m_mind_kg` in
    /// xyl-cosmos `life`.
    pub m_mind: Dy,
    /// The unit body's generation (years), 14.2 yr. It is a declared ruler
    /// until it is derived from the unit body's life history; every
    /// people's generation scales it by 2s / (1 + s).
    pub generation_yr: Dy,
    /// The share of free energy a biosphere fixes (primary production).
    pub eta_primary: Dy,
    /// The share of energy one link of a food chain passes to the next
    /// (Lindeman's tenth).
    pub trophic: Dy,
    /// The flux unit (W/m^2).
    pub flux_w: Dy,
}

impl Default for Rulers {
    fn default() -> Rulers {
        Rulers {
            m_mind: Dy::int(10),
            generation_yr: Dy::ratio(142, 10),
            eta_primary: Dy::ratio(15, 1000),
            trophic: Dy::ratio(1, 10),
            flux_w: Dy::int(340),
        }
    }
}

/// A people's body mass (kg).
pub fn body_kg(traits: &Traits, r: &Rulers) -> Dy {
    r.m_mind * traits.size
}

/// The unit body's metabolism (W), the metabolism of the mind-bearing body
/// held as a ruler in xyl-cosmos `life` (19.1 W).
pub fn unit_metabolism_w() -> Dy {
    xyl_cosmos::life::unit_metabolism_w()
}

/// A body's metabolism (W): the unit body's metabolism scaled by the size
/// share 2s / (1 + s), s = m_b / M_MIND, and by the body pace of the
/// people's solvent (`worlds::body_pace`).
pub fn metabolism_w(traits: &Traits, ctx: &Ctx, r: &Rulers) -> Dy {
    let s = (body_kg(traits, r) / r.m_mind.max(Dy::sci(1, -12))).max(Dy::ZERO);
    unit_metabolism_w() * (Dy::int(2) * s / (Dy::ONE + s)) * body_pace(traits.solvent, ctx)
}

/// The food power the world offers one link up the biosphere (W): free
/// energy x flux unit x surface area x living band x `eta_primary` x
/// `trophic`, averaged over the year by the growing share (one half on a
/// tidally locked world).
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

/// A people's natural growth rate (per year): one e-fold per generation,
/// `r = 1 / generation_yr`.
pub fn growth_per_yr(traits: &Traits, r: &Rulers) -> Dy {
    Dy::ONE / generation_yr(traits, r).max(Dy::sci(1, -9))
}

/// Numbers after `years` from `now`, toward the capacity `cap`, by the
/// closed-form logistic K / (1 + (K/N - 1) e^(-r t)); above capacity the
/// numbers decay toward it by the same form. Returns zero when `cap` or
/// `now` is not positive. The exponent r t is capped at 200.
pub fn grow(now: Dy, cap: Dy, rate: Dy, years: Dy) -> Dy {
    if cap.signum() <= 0 || now.signum() <= 0 {
        return Dy::ZERO;
    }
    let x = (rate * years).min(Dy::int(200));
    // A span of zero returns the numbers unchanged; the closed form would
    // round them.
    if x.signum() <= 0 {
        return now;
    }
    cap / (Dy::ONE + (cap / now - Dy::ONE) * (-x).exp())
}

/// The power a world offers a people's works and the power the people's
/// bodies already convert (W). Until seated minds direct the capture of the
/// world's flows, a people commands only the power of its bodies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Power {
    /// The star's light over the world's whole surface, at the year's mean
    /// flux.
    pub starlight_w: Dy,
    /// The world's internal heat flowing out through the surface.
    pub heat_w: Dy,
    /// The total metabolism of the people's bodies.
    pub bodies_w: Dy,
}

/// A world's whole surface (m^2), not only its living area.
fn surface_m2(body: &Body) -> Dy {
    let unit_r = Dy::from_q(&xyl_cosmos::package::Si::world_radius_m());
    let r_m = body.radius * unit_r;
    // Linear area rule against the unit world: A = 4 pi r R_unit.
    Dy::int(4) * Dy::PI * r_m * unit_r
}

/// The power available to a people of `numbers` bodies on world `body` in
/// state `state` (W): the star's light over the whole surface, the world's
/// heat flow through it, and the bodies' total metabolism under `traits`,
/// `ctx` and the rulers `r`.
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
/// (W): the power a people's producers may capture (civ::housing
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
            body_t: Dy::int(310),
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
        let cap = Dy::int(1_000_000);
        // The logistic approaches the capacity from either side and does not
        // cross it.
        let rate = growth_per_yr(&t, &r);
        let later = grow(Dy::int(500), cap, rate, Dy::int(1000));
        assert!(later > Dy::int(500) && later <= cap);
        assert!(grow(cap * Dy::int(2), cap, rate, Dy::int(1000)) < cap * Dy::int(2));
        assert_eq!(grow(Dy::int(500), cap, rate, Dy::ZERO), Dy::int(500));
        // One e-fold per generation: a larger body has a longer generation
        // and grows more slowly.
        assert!(growth_per_yr(&traits(Dy::int(8), Strategy::K), &r) < rate);
    }
}
