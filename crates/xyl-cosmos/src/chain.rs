//! The anchor chain's galactic tier: places of stars and cores held as
//! bearings on spheres, and arc lengths between them (docs/model.md).
//!
//! Reads `package::Si` and `package::hole_mass` (for `units`, the core
//! hole's `xyl_core::units::Units`), `galaxy::rim_kpc` and
//! `galaxy::radius_kpc` (from `circuit::anchor`). Lengths are in the galaxy
//! tier's unit, 60^5 r_g of the core (`Tier::Galaxy`); `kpc` gives one
//! kiloparsec in that unit.
//!
//! A star's place is its arc `reach` from its galaxy's core along the
//! galaxy's sphere (radius rim / pi, `galaxy_radius`), its unit direction
//! `dir`, its `climb` off its orbit's plane and an extra offset `out`; a
//! core's place is the same on the universe's sphere (radius `radius`).
//!
//! Outputs: `Place` (`core`, `in_galaxy`, `standing_off`, `to_anchor`,
//! `arc`, `arc_kpc`, `arc_kpc_dy`, `cell`) and `Cell`. On a shared sphere
//! the arc is the turn between directions times `R sin(inner / R)`, plus the
//! absolute differences in reach and climb and both `out`s (summed, no
//! squares or roots); between galaxies it is each place's path to its core
//! plus the arc between the cores. The turn uses a dyadic `acos` and the
//! midpoints of certified pi and sine, so `arc` returns an `Encl::exact` of
//! that reading, not a proven bound. `cell` is a degree label computed in
//! dyadics; no law reads it.

use crate::package::Si;
use xyl_core::certified::{cos_sin_turns, pi, Encl};
use xyl_core::dyadic::Dy;
use xyl_core::units::{Tier, Units};
use xyl_core::Q;

/// This galaxy's units: the core hole's scale (measured G, c and hole mass).
pub fn units() -> Units {
    Units::of(
        &Si::g(),
        &Si::c(),
        &(&crate::package::hole_mass() * &Si::star_mass_kg()),
    )
}

/// One kiloparsec in the galaxy tier's unit (exact).
pub fn kpc() -> Q {
    units().in_tier(&Si::kpc(), Tier::Galaxy)
}

/// The universe's curvature radius in the galaxy tier's unit: the founding's
/// rim over pi (circuit::anchor).
pub fn radius() -> Q {
    &crate::galaxy::radius_kpc() * &kpc()
}

/// The radius of a galaxy's sphere (each galaxy an enclosed S^360): the
/// core's pulse spreads around it and refocuses on the far side at the rim,
/// so the rim is half a great circle and the radius is rim / pi (galaxy
/// units). The home galaxy's rim is the founding's (circuit::anchor); another
/// galaxy's rim is that rim times the galaxy's `scale`.
pub fn galaxy_radius(scale: &Q) -> Q {
    let rim = &(&crate::galaxy::rim_kpc() * scale) * &kpc();
    rim.checked_div(&pi(80).mid()).unwrap_or(Q::ONE)
}

/// A place, held as a bearing on its sphere (no squared lengths):
/// - `reach`: its arc from its anchor along the sphere (a star's from its
///   galaxy's core, a core's from the universe's pole);
/// - `dir`: the unit direction its frame turns the anchor's axis to (exact,
///   unit by construction);
/// - `climb`: its offset from its orbit's plane (a disc star's swing);
/// - `out`: an additional offset from that place (e.g. a drift).
///
/// Distances between places in different galaxies run through the cores'
/// throats.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    /// Arc from the anchor along the sphere (galaxy units).
    pub reach: Q,
    /// Unit direction the frame turns the anchor's axis to (exact).
    pub dir: [Q; 3],
    /// Offset from the orbit's plane (galaxy units, signed).
    pub climb: Q,
    /// Additional non-negative offset from the place, such as a drift (galaxy units).
    pub out: Q,
    /// The radius of the sphere it lies on (galaxy units).
    pub sphere: Q,
    /// The place's galaxy (None: on the universe's sphere, i.e. a core).
    pub galaxy: Option<u32>,
    /// The galaxy's core on the universe's sphere (None for a core).
    pub core: Option<Box<Place>>,
}

/// A place's degree cell on the sphere it lies on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cell {
    /// The sphere's galaxy; `None` for the universe's sphere.
    pub galaxy: Option<u32>,
    /// Azimuth degree in [0, 360).
    pub azimuth: i64,
    /// Polar degree in [0, 180].
    pub polar: i64,
}

impl Place {
    /// A core on the universe's sphere of radius `r`: its reach from the pole
    /// (galaxy units) and its direction (its plane's frame applied to its
    /// phase at t, cores.rs Shape::place_at).
    pub fn core(reach: Q, dir: [Q; 3], r: &Q) -> Place {
        Place {
            reach,
            dir,
            climb: Q::ZERO,
            out: Q::ZERO,
            sphere: r.clone(),
            galaxy: None,
            core: None,
        }
    }

    /// A place on the sphere of galaxy `galaxy`, of radius `rg`, its core at
    /// `core` on the universe's sphere.
    pub fn in_galaxy(reach: Q, dir: [Q; 3], climb: Q, galaxy: u32, rg: &Q, core: Place) -> Place {
        Place {
            reach,
            dir,
            climb,
            out: Q::ZERO,
            sphere: rg.clone(),
            galaxy: Some(galaxy),
            core: Some(Box::new(core)),
        }
    }

    /// The same place, offset a further `|out|`.
    pub fn standing_off(mut self, out: &Q) -> Place {
        self.out = &self.out + &out.abs();
        self
    }

    /// Path length to the anchor: reach + |climb| + out (summed, not a
    /// hypotenuse).
    pub fn to_anchor(&self) -> Q {
        &(&self.reach + &self.climb.abs()) + &self.out
    }

    /// The arc to another place. On a shared sphere: the turn between the
    /// directions (acos of their dot product) times the circle at the inner
    /// reach, R sin(s / R), plus the differences in reach and climb and both
    /// places' `out`. Between galaxies: each place's path to its core plus the
    /// arc between the cores on the universe's sphere of radius `r`. No
    /// squared lengths or roots.
    pub fn arc(&self, other: &Place, r: &Q) -> Encl {
        if self.galaxy == other.galaxy {
            let sphere = if self.galaxy.is_some() {
                self.sphere.clone()
            } else {
                r.clone()
            };
            let agree = self
                .dir
                .iter()
                .zip(&other.dir)
                .fold(Q::ZERO, |a, (x, y)| &a + &(x * y));
            let turn = Dy::from_q(&agree).acos().to_q();
            let inner = self.reach.clone().min(other.reach.clone()).max(Q::ZERO);
            let full = &(&Q::int(2) * &pi(80).mid()) * &sphere;
            let around = if full.signum() > 0 {
                let (_, s) =
                    cos_sin_turns(&Encl::exact(inner.checked_div(&full).unwrap_or(Q::ZERO)));
                &sphere * &s.mid()
            } else {
                inner
            };
            let climb = &(&(&self.reach - &other.reach).abs()
                + &(&self.climb - &other.climb).abs())
                + &(&self.out + &other.out);
            return Encl::exact(&(&turn * &around) + &climb);
        }
        let core_of = |p: &Place| p.core.as_deref().cloned().unwrap_or_else(|| p.clone());
        let between = core_of(self).arc(&core_of(other), r);
        between.add(&Encl::exact(&self.to_anchor() + &other.to_anchor()))
    }

    /// The place's degree cell on its sphere: its galaxy (None: the universe's
    /// sphere), azimuth degree [0, 360) and polar degree [0, 180]. A label
    /// computed in dyadics; the place itself stays exact and no law uses cells.
    pub fn cell(&self) -> Cell {
        let d = |q: &Q| Dy::from_q(q);
        let (x, y, z) = (d(&self.dir[0]), d(&self.dir[1]), d(&self.dir[2]));
        let half = Dy::PI / Dy::int(2);
        let az = if x.signum() > 0 {
            (y / x).atan()
        } else if x.signum() < 0 {
            (y / x).atan() + if y.signum() >= 0 { Dy::PI } else { -Dy::PI }
        } else if y.signum() >= 0 {
            half
        } else {
            -half
        };
        let deg = |rad: Dy| rad * Dy::int(180) / Dy::PI;
        let mut az_deg = deg(az).trunc_i64();
        if az_deg < 0 {
            az_deg += 360;
        }
        let polar = deg(z.clamp(Dy::int(-1), Dy::ONE).acos())
            .trunc_i64()
            .clamp(0, 180);
        Cell {
            galaxy: self.galaxy,
            azimuth: az_deg.rem_euclid(360),
            polar,
        }
    }

    /// The arc in kiloparsecs, computed in dyadics within one galaxy (turn x
    /// sphere x sin(inner / sphere), plus the climbs) for fast per-pair
    /// evaluation; between galaxies, the exact arc.
    pub fn arc_kpc_dy(&self, other: &Place, r: &Q) -> Dy {
        if self.galaxy != other.galaxy || self.galaxy.is_none() {
            return self.arc_kpc(other, r);
        }
        let d = |q: &Q| Dy::from_q(q);
        let agree = (0..3).fold(Dy::ZERO, |a, k| a + d(&self.dir[k]) * d(&other.dir[k]));
        let turn = agree.clamp(Dy::int(-1), Dy::ONE).acos();
        let (ra, rb, sphere) = (d(&self.reach), d(&other.reach), d(&self.sphere));
        let inner = ra.min(rb).max(Dy::ZERO);
        let around = if sphere.signum() > 0 {
            sphere * (inner / sphere).sin()
        } else {
            inner
        };
        let climb = (ra - rb).abs()
            + (d(&self.climb) - d(&other.climb)).abs()
            + d(&self.out)
            + d(&other.out);
        (turn * around + climb) / d(&kpc())
    }

    /// The exact arc converted to kiloparsecs.
    pub fn arc_kpc(&self, other: &Place, r: &Q) -> Dy {
        Dy::from_q(
            &self
                .arc(other, r)
                .mid()
                .checked_div(&kpc())
                .unwrap_or(Q::ZERO),
        )
    }
}
