//! Step readings: per-people shares that show whether a people's knowledge
//! is growing or stalled, and whether it is aware but not free to act.
//!
//! Inputs: a [`Universe`]'s peoples (`cosmic.peoples`), their knowledge
//! stocks (`civ.knowledge`: last span's effort and forgetting share) and
//! their gauges (`civ.gauges`).
//!
//! Output: [`living`] returns a [`Step`] for every people with numbers
//! above zero: `z` = learn / (learn + forget) and `r` = (learn - forget) /
//! (learn + forget), learn being the yearly thought (KU) and forget the
//! forgetting share times the knowledge held; `frozen` = sentience x
//! (1 - consciousness). A reading is `None` where its layer has no data.
//!
//! Invariants: read-only and recomputed on every call from the current
//! state; nothing is stored, latched or checkpointed, and nothing here acts
//! on the universe. Read by `xylarium observe sync`.

use crate::universe::Universe;
use xyl_core::dyadic::Dy;

/// A people's readings. `None` where the reading is not yet modelled or the
/// people has no layer to read yet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Step {
    /// Poles, read on knowledge: learning per year (KU) against forgetting
    /// per year (the forgetting share of what is held):
    /// z = learn / (learn + forget), R = (learn - forget) / (learn + forget),
    /// in [-1, 1].
    pub z: Option<Dy>,
    /// Pole balance R = (learn - forget) / (learn + forget), in \[-1, 1\];
    /// positive while a people learns faster than it forgets.
    pub r: Option<Dy>,
    /// Aware but frozen: sentience x (1 - consciousness).
    pub frozen: Option<Dy>,
}

fn share(a: Dy, b: Dy) -> Option<Dy> {
    let both = a + b;
    (both.signum() > 0).then(|| a / both)
}

/// Every living people's readings now.
pub fn living(u: &Universe<'_>) -> Vec<(usize, Step)> {
    let mut out = Vec::new();
    for (i, _) in u
        .cosmic
        .peoples
        .iter()
        .enumerate()
        .filter(|(_, p)| p.population.signum() > 0)
    {
        let mut s = Step::default();
        if let Some(k) = u.civ.knowledge.of.get(i) {
            let held: Dy = k.stock.iter().copied().sum();
            let learn = k.effort.total();
            let forget = k.forgetting * held;
            s.z = share(learn, forget);
            s.r = (learn + forget > Dy::ZERO).then(|| (learn - forget) / (learn + forget));
        }
        if let Some(g) = u.civ.gauges.get(i) {
            s.frozen = Some(g.sentience * (Dy::ONE - g.consciousness));
        }
        out.push((i, s));
    }
    out
}
