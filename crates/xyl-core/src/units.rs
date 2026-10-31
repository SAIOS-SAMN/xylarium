//! Units anchored at the universal core: `G = c = M_core = 1`, so the length
//! unit is the core's gravitational radius `r_g = G M_core / c^2`.
//!
//! Inputs: the measured `G` (m^3 / kg s^2), `c` (m/s) and the core's mass (kg)
//! as exact rationals ([`Q`]), supplied from outside (in the cosmos crate, by
//! `package::Si`).
//!
//! Outputs: [`Tier`], a level of the anchor chain by its unit's power of sixty
//! over `r_g` (Universe 60^10, Cluster 60^7, Galaxy 60^5, System 60, Planet
//! 60^-2), with `in_rg` its unit in `r_g`; [`Units`], holding metres per `r_g`
//! (`of`), with `in_tier` converting a length in metres to a tier's unit.
//!
//! Time: with `c = 1` the time unit is `r_g / c`, the time light takes to
//! cross one `r_g`, so a reach of `n` units of a tier is crossed in
//! `n` times that tier's [`Tier::in_rg`] units of `r_g / c`. The engine's
//! clock is the bell, light's round trip to the home galaxy's rim: with the
//! rim read in galaxy-tier units it is `2 rim 60^5` units of `r_g / c`,
//! exact. One bell is one ring, the unit every time above the founding is
//! counted in; its length in seconds is applied outside this crate.
//!
//! Invariants: every conversion is exact; moving between tiers multiplies by a
//! power of sixty, so each tier's readings stay within a few powers of sixty
//! of one. SI enters only through `Units::of`, converted once. A zero
//! denominator yields zero rather than a refusal.
//!
//! Terms: tier (docs/glossary.md).

use crate::Q;

/// A tier of the anchor chain, by its unit's power of sixty over `r_g`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    /// 60^10 r_g: the universe's tier, where its curvature radius reads
    /// near one.
    Universe,
    /// 60^7 r_g: the cluster's tier, where galactic cores read about their
    /// universe core, a galaxy pair a unit or two apart.
    Cluster,
    /// 60^5 r_g: the galaxy's tier, where a galaxy's reaches read from one
    /// to a hundred.
    Galaxy,
    /// 60 r_g: the system's tier, where a world's reach about its star reads.
    System,
    /// `r_g / 60^2`: the planet's tier, where a world's radius and its moons'
    /// reaches read.
    Planet,
}

impl Tier {
    fn power(self) -> i32 {
        match self {
            Tier::Universe => 10,
            Tier::Cluster => 7,
            Tier::Galaxy => 5,
            Tier::System => 1,
            Tier::Planet => -2,
        }
    }

    /// The tier's unit in `r_g`: 60 raised to the tier's power, exact.
    pub fn in_rg(self) -> Q {
        let p = self.power();
        let sixty = Q::int(60);
        let up = (0..p.unsigned_abs()).fold(Q::ONE, |acc, _| &acc * &sixty);
        if p >= 0 {
            up
        } else {
            Q::ONE.checked_div(&up).unwrap_or(Q::ZERO)
        }
    }
}

/// A universe's units: the white hole's scale in SI, held exact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Units {
    /// Metres in one `r_g`.
    pub r_g_m: Q,
}

impl Units {
    /// From the measured G (m^3 / kg s^2), c (m/s) and the core's mass (kg).
    pub fn of(g: &Q, c: &Q, core_kg: &Q) -> Units {
        Units {
            r_g_m: (g * core_kg).checked_div(&(c * c)).unwrap_or(Q::ZERO),
        }
    }

    /// A length in metres, read in a tier's unit.
    pub fn in_tier(&self, metres: &Q, tier: Tier) -> Q {
        metres
            .checked_div(&(&self.r_g_m * &tier.in_rg()))
            .unwrap_or(Q::ZERO)
    }
}
