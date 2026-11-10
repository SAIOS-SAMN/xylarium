//! The housing law: how many of a people's bodies its world feeds. The same
//! law is used at waking, in the carry and in readings
//! (docs/civilizations.md, Housing).
//!
//! Inputs: the people's `cosmic::Traits`, its world's `State` and media
//! (`xyl_cosmos::worlds`), its `Body` (`xyl_cosmos::life`), the
//! `growth::Rulers`, and its biology and energy masteries in [0, 1].
//!
//! Outputs, all `Dy`:
//! - [`harvest_w`]: what the medium yields (W), from the forage one trophic
//!   link up toward all that the primary producers fix, by biology mastery;
//! - [`feeds_w`]: harvest plus synthesis (W), the share of the world's
//!   starlight and heat its energy mastery captures, fixed at the primary
//!   producers' efficiency and eaten at the link biology mastery lifts;
//! - [`housing`]: fed power over a body's metabolism (bodies);
//! - [`houses`]: [`housing`] of [`feeds_w`] in the medium of the people's
//!   solvent, the world's starlight and heat read from `growth::power`;
//! - [`free_share`]: the share of the housing the numbers leave free,
//!   `1 - numbers / housing` in [0, 1] (the consciousness gauge, civ/mod.rs,
//!   and free thought, knowledge.rs).
//!
//! Invariants: pure functions with no state; no harvest where the people's
//! solvent is absent; masteries are clamped to [0, 1].

use crate::cosmic::Traits;
use crate::growth::{self, Rulers};
use xyl_core::dyadic::Dy;
use xyl_cosmos::life::Body;
use xyl_cosmos::worlds::{Ctx, Medium, State};

/// The number of bodies housed: the power fed, `capacity_w` (W), over the
/// power one body transduces, `metabolism_w` (W, floored at 1e-12).
pub fn housing(capacity_w: Dy, metabolism_w: Dy) -> Dy {
    capacity_w
        .div(&metabolism_w.max(Dy::sci(1, -12)))
        .unwrap_or_default()
}

/// What a medium yields to a people at biology `mastery` (W):
/// `forage (1 + (1 / trophic - 1) mastery)`, rising from the forage one
/// trophic link up the biosphere at mastery 0 to the whole of what the
/// primary producers fix at mastery 1.
pub fn harvest_w(med: &Medium, body: &Body, rulers: &Rulers, mastery: Dy) -> Dy {
    let forage = growth::food_w(med, body, rulers);
    let down = (Dy::ONE)
        .div(&rulers.trophic.max(Dy::sci(1, -6)))
        .unwrap_or_default()
        - Dy::ONE;
    forage * (Dy::ONE + down * mastery.clamp(Dy::ZERO, Dy::ONE))
}

/// What feeds a people (W): the harvest of its medium ([`harvest_w`] at
/// biology mastery `bio`; none where its solvent is absent) plus synthesis,
/// `energy world_w eta_primary link`. Synthesis is the share `energy` of the
/// world's starlight and heat `world_w`, fixed at the primary producers'
/// efficiency and eaten at the link `trophic + (1 - trophic) bio`. With
/// nothing learned, the result is the forage.
pub fn feeds_w(
    world_w: Dy,
    med: Option<&Medium>,
    body: &Body,
    rulers: &Rulers,
    bio: Dy,
    energy: Dy,
) -> Dy {
    let harvest = med
        .map(|m| harvest_w(m, body, rulers, bio))
        .unwrap_or(Dy::ZERO);
    let world = world_w.max(Dy::ZERO);
    let link = rulers.trophic + (Dy::ONE - rulers.trophic) * bio.clamp(Dy::ZERO, Dy::ONE);
    harvest + energy.clamp(Dy::ZERO, Dy::ONE) * world * rulers.eta_primary * link
}

/// How many of a people's bodies its world houses: [`feeds_w`] in the medium
/// of the people's solvent on that world over the metabolism of one body.
pub fn houses(
    traits: &Traits,
    state: &State,
    ms: &[Medium],
    body: &Body,
    ctx: &Ctx,
    rulers: &Rulers,
    bio: Dy,
    energy: Dy,
) -> Dy {
    let world = growth::power(state, body, traits, Dy::ZERO, ctx, rulers);
    housing(
        feeds_w(
            world.starlight_w + world.heat_w,
            traits.medium(ms),
            body,
            rulers,
            bio,
            energy,
        ),
        growth::metabolism_w(traits, ctx, rulers),
    )
}

/// The share of a world's `housing` that `numbers` leave free:
/// `1 - numbers / housing`, clamped to [0, 1]; zero where nothing is housed.
pub fn free_share(numbers: Dy, housing: Dy) -> Dy {
    if housing.signum() > 0 {
        (Dy::ONE - numbers.div(&housing).unwrap_or_default()).clamp(Dy::ZERO, Dy::ONE)
    } else {
        Dy::ZERO
    }
}
