//! Units anchored at the white hole.
//!
//! The base is the core hole's own scale: G = c = M_core = 1, so the length
//! unit is its gravitational radius `r_g = G M_core / c^2`, exact from the
//! laws. Each tier of the anchor chain reads in its own unit, an exact power
//! of sixty of `r_g` (the sexagesimal ladder): moving between tiers
//! multiplies by a power of sixty, exact, and the numbers each tier reads
//! stay within a few powers of sixty of one. SI enters only where a measured
//! law is read in from outside, converted once.

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
    /// r_g / 60^2: the planet's tier, where a world's radius and its moons'
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

    /// Its unit in `r_g`: 60 to its power, exact.
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
