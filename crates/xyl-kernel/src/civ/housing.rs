//! What a world feeds a people: one housing law for waking, live play and
//! readings (docs/civilizations.md, Housing). Every input is the people's
//! world and its own masteries.

use crate::cosmic::Traits;
use crate::growth::{self, Rulers};
use xyl_core::dyadic::Dy;
use xyl_cosmos::life::Body;
use xyl_cosmos::worlds::{Ctx, Medium, State};

/// How many of its bodies its land houses: what it is fed (W) over what a
/// body burns (W).
pub fn housing(capacity_w: Dy, metabolism_w: Dy) -> Dy {
    capacity_w / metabolism_w.max(Dy::sci(1, -12))
}

/// What its medium yields to a people at its biology `mastery` (W): from the
/// forage one trophic link up its biosphere toward the whole of what its
/// primary producers fix.
pub fn harvest_w(med: &Medium, body: &Body, rulers: &Rulers, mastery: Dy) -> Dy {
    let forage = growth::food_w(med, body, rulers);
    let down = Dy::ONE / rulers.trophic.max(Dy::sci(1, -6)) - Dy::ONE;
    forage * (Dy::ONE + down * mastery.clamp(Dy::ZERO, Dy::ONE))
}

/// What feeds a people (W): what is fixed on its world, eaten at the link its
/// biology mastery `bio` lifts, trophic + (1 - trophic) bio - its medium's
/// natural fixing ([`harvest_w`]; none where its solvent is gone) and its own
/// producers' (synthesis: the share of its world's starlight and heat,
/// `world_w`, its energy mastery `energy` captures, fixed at the primary
/// producers' efficiency). With nothing learned, the forage.
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

/// How many of a people's bodies its world houses: what feeds it over what
/// a body eats, its medium its own solvent's on that world.
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
    housing(
        feeds_w(
            growth::world_w(state, body, rulers),
            traits.medium(ms),
            body,
            rulers,
            bio,
            energy,
        ),
        growth::metabolism_w(traits, ctx, rulers),
    )
}
