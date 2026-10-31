//! A world's body: its orbit and spin at any age, and its interior's heat
//! budget, temperatures and dynamo.
//!
//! Inputs: `systems::World` (mass and radius in world units, axis in AU,
//! layers, element masses, and the settling's swing and phase from
//! `systems::settle_about`), the host's mass in kg, `laws::Scales`, and SI
//! constants from `package::Si`. Mean motion and break-up spin come from the
//! turning law (`turning::turning_si`); the tidal stretch across a body is
//! its host's deficit difference `delta(a - R) - delta(a + R)`.
//!
//! - [`orbits`]: per world an [`Orbit`]. Eccentricity is the settled swing,
//!   capped at half the fractional gap to the nearest neighbour; inclination
//!   is `swing / (1 + swing)` of a quarter turn; node and periapsis are the
//!   settled phase; the forced eccentricity is the Laplace-Lagrange leading
//!   term `5/4 x axis ratio x neighbour e0`. Obliquity cosine (`2u - 1`) and
//!   birth spin (0.1 to 0.5 of break-up) are drawn from the caller's stream.
//!   Tidal circularisation ([`circularise_gyr`]) and spin lock ([`lock_gyr`])
//!   use Love number `k2` ([`love`]) and `Q = 100`.
//! - [`Orbit::at`]: eccentricity relaxed toward the forced value, spin braked
//!   linearly to lock, day and year lengths.
//! - [`interior`]: the [`Interior`] at an age in closed form. Heat is a sum
//!   of boosts (`xyl_core::motion::Modes`: U238, U235, Th232, K40 at
//!   half-lives scaled by `Scales`, plus the tide's modes); the mantle sits
//!   at the temperature where Arrhenius-viscosity convection carries the
//!   heat made (a bracketed Newton root), plus its lag and formation
//!   transient; the core follows the mantle's base; the dynamo's strength is
//!   the core-boundary flux over the adiabatic flux, less one, clamped to
//!   0..1. [`heat_sources`] and [`kept`] give each isotope's inventory at
//!   the star's birth relative to the Sun's.
//!
//! Orbit quantities are `Q` rounded down to a 2^-32 grid, transcendentals
//! read from certified enclosures; interior quantities are `Dy`. Laboratory
//! rock and iron constants are declared rulers (`Rock`, the `Lab` table).

use crate::elements::Element;
use crate::laws::Scales;
use crate::package::Si;
use crate::systems::World;
use xyl_core::certified::{down, exp, pi};
use xyl_core::dyadic::Dy;
use xyl_core::motion::{Boost, Modes};
use xyl_core::{Stream, Q};

mod interior;
mod isotopes;
mod orbit;

pub use interior::*;
pub use isotopes::*;
pub use orbit::*;

const BITS: u32 = 56;
const GRID: u32 = 32;

fn on_grid(x: Q) -> Q {
    down(&x, GRID)
}

fn ex(x: &Q) -> Q {
    on_grid(exp(x, BITS).mid())
}

/// A body's turning at its own surface (rad/s): the turning law c delta / R,
/// its break-up spin.
fn surface_turning(m_kg: &Q, r_m: &Q) -> Q {
    crate::turning::rad_s(&crate::turning::turning_si(m_kg, r_m))
}

fn pi_q() -> Q {
    pi(BITS).mid()
}

fn unit(rng: &mut Stream) -> Q {
    let x = i64::try_from(rng.next_u64() >> 11).unwrap_or(0);
    Q::ratio(x, 1 << 53)
}

/// A rocky body's measured constants.
pub struct Rock;
impl Rock {
    /// The unit world's self-gravity pressure rho g R, 3.4e11 Pa: the unit of a body's
    /// own (`self_pressure`).
    pub fn world_pressure() -> Q {
        Q::int(340_000_000_000)
    }
    /// Shear rigidity of rock, 5.0e10 Pa.
    pub fn mu_rock() -> Q {
        Q::int(50_000_000_000)
    }
    /// Shear rigidity of ice, 4.0e9 Pa (used for bodies more than a fifth ice).
    pub fn mu_ice() -> Q {
        Q::int(4_000_000_000)
    }
    /// Tidal dissipation quality factor Q, dimensionless (100).
    pub fn tidal_q() -> Q {
        Q::int(100)
    }
    /// Moment-of-inertia factor I / (M R^2), dimensionless (0.33).
    pub fn inertia() -> Q {
        Q::ratio(33, 100)
    }
}

#[cfg(test)]
mod tests;
