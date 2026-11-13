//! The anchor chain's galactic tier: places of stars and cores held as
//! bearings on spheres, and arc lengths between them (docs/model.md).
//!
//! Reads `package::Si` and `package::core_kg` (for `units`, the core
//! hole's `xyl_core::units::Units`), `galaxy::rim_kpc` and
//! `galaxy::radius_kpc` (from `circuit::anchor`). Lengths are in the galaxy
//! tier's unit, 60^5 r_g of the core (`Tier::Galaxy`), one galaxy unit;
//! `galaxy::units_of_kpc` and `galaxy::kpc_of_units` convert to and from
//! kiloparsecs.
//!
//! A star's place is its arc `reach` from its galaxy's core along the
//! galaxy's sphere (radius rim / pi, `galaxy_radius`), its unit direction
//! `dir`, its `climb` off its orbit's plane and an extra offset `out`; a
//! core's place is the same on the universe's sphere (radius `radius`).
//!
//! Outputs: `Place` (`core`, `in_galaxy`, `standing_off`, `to_anchor`,
//! `arc`, `arc_kpc`, `cell`) and `Cell`. On a shared sphere the arc is the
//! turn between directions times the circle at the inner reach,
//! `R sin(inner / R)` (`bounce::curved_reach`, the one chord on an S^360),
//! plus the absolute differences in reach and climb and both `out`s (summed,
//! no squares or roots); between galaxies it is each place's path to its
//! core plus the arc between the cores. The turn is the midpoint of the
//! pivot's certified turn between the directions (`xyl_core::rotor::pivot`,
//! the relative rotor on its one named plane) and the chord the midpoint of
//! a certified sine, so `arc` returns an `Encl::exact` of that reading, not
//! a proven bound. `cell` is a degree label computed in dyadics; no law
//! reads it.

use crate::package::Si;

/// Radians per degree, `pi / 180`, certified.
fn per_degree() -> Encl {
    pi(64).mul(&Encl::exact(xyl_core::ratio!(1, 180)))
}
use xyl_core::certified::{pi, Encl};
use xyl_core::dyadic::Dy;
use xyl_core::rotor::{Rotor, Winding};
use xyl_core::units::Units;
use xyl_core::Q;

/// This galaxy's units: the core hole's scale (measured G, c and hole mass),
/// the SI seam removed with the SI layers (#25). The package's measured
/// constants are nonzero; a refusal of `Units::of` reads as no scale, zero.
pub fn units() -> Units {
    Units::of(&Si::g(), &Si::c(), &crate::package::core_kg()).unwrap_or(Units { r_g_m: Q::ZERO })
}

/// The universe's curvature radius at `t` rings, in the galaxy tier's unit:
/// the founding's rim over pi (circuit::anchor), breathing with the bell.
/// Each echo the enclosure expands and contracts by the core's deficit at
/// its rim, `delta_rim = H / (rim + H)` with the core's hold `H = 1` (the
/// manifold's deformation about its hold, E011, #155), on the ring's own
/// swing, the inbound echo's ring term (`circuit::Substrate::inbound`):
/// `rho(t) = rho_0 (1 + delta_rim cos(360 t / echo))`. Smooth, first-degree
/// in the deficit, returning each echo (#233).
pub fn radius(t: Dy) -> Q {
    let rest = crate::galaxy::units_of_kpc(&crate::galaxy::radius_kpc());
    &rest * &breath(t, crate::circuit::echo())
}

/// One breath of an enclosure about its core at `t` rings over the core's
/// echo `echo` (rings): `1 + delta_rim cos(360 t / echo)`, the deficit at
/// the founding's rim `delta_rim = H / (rim + H)` with the core's hold
/// `H = 1`. A change of tier dilates the rim and the hold together, so the
/// deficit is the same in every galaxy's own units; the echo dilates with
/// them. One plus the whole deficit when the echo is zero.
fn breath(t: Dy, echo: Dy) -> Q {
    let rim = crate::circuit::anchor().rim;
    let deficit = Q::ONE.checked_div(&(&rim + &Q::ONE)).unwrap_or(Q::ZERO);
    let turns = t.div(&echo).unwrap_or_default().to_q().frac();
    let (cos, _) = xyl_core::certified::cos_sin_turns(&Encl::exact(turns));
    let swing = xyl_core::certified::toward_zero(&(&deficit * &cos.mid()), 64);
    &Q::ONE + &swing
}

/// The radius at `t` rings of a galaxy's sphere (each galaxy an enclosed
/// S^360): the core's pulse spreads around it and refocuses on the far side
/// at the rim, so the rim is half a great circle and the rest radius is
/// rim / pi (galaxy units). The home galaxy's rim is the founding's
/// (circuit::anchor); another galaxy's rim is that rim times the galaxy's
/// `scale`. It breathes with its core's bell (`breath`): the home core's
/// echo, dilated by `scale` for a homologue (#233 item 1).
pub fn galaxy_radius(scale: &Q, t: Dy) -> Q {
    let rim = crate::galaxy::units_of_kpc(&(&crate::galaxy::rim_kpc() * scale));
    let rest = rim.checked_div(&pi(80).mid()).unwrap_or(Q::ONE);
    let echo = crate::circuit::echo() * Dy::from_q(scale);
    &rest * &breath(t, echo)
}

/// The unit direction a plane's rotor turns a phase to: the phase's exact
/// circle point on the reference plane (axes 1, 2), turned exactly, so the
/// direction is exactly of unit length.
pub fn on_plane(plane: &Rotor<3>, at: &Winding) -> [Q; 3] {
    let (c, s) = xyl_core::rotor::circle_point(&at.degrees, 64).unwrap_or((Q::ONE, Q::ZERO));
    plane
        .apply(&[c, s, Q::ZERO].map(Encl::exact))
        .map(|x| x.mid())
}

/// A place, held as a bearing on its sphere (no squared lengths):
/// - `reach`: its arc from its anchor along the sphere (a star's from its
///   galaxy's core, a core's from the universe's pole);
/// - `plane`: its orbit's plane, the rotor carrying the reference plane
///   (axes 1, 2) to it, and `winding`: its turns on that plane, whole turns
///   and phase carried apart (E019);
/// - `dir`: the unit direction they give ([`on_plane`]), exact;
/// - `climb`: its offset from its orbit's plane (a disc star's swing);
/// - `out`: an additional offset from that place (e.g. a drift).
///
/// Distances between places in different galaxies run through the cores'
/// throats.
#[derive(Clone, Debug, PartialEq, Eq)]
// Stand-in S037: A place is a point, not the vortex.
pub struct Place {
    /// Arc from the anchor along the sphere (galaxy units).
    pub reach: Q,
    /// Its orbit's plane: the rotor carrying the reference plane to it.
    pub plane: Rotor<3>,
    /// Its turns on that plane: whole turns and phase (E019).
    pub winding: Winding,
    /// Unit direction the plane's rotor turns the phase to (exact).
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

/// The separation of two places on one sphere ([`Place::separation`]),
/// every component kept apart: never summed into one distance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Separation {
    /// The pivot between their directions: its named plane and its turn.
    pub pivot: Option<xyl_core::rotor::Pivot<3>>,
    /// Their orbit planes' relative rotor `P_b^-1 P_a`, read by its
    /// dimensions: no plane where their orbits share one.
    pub relative: xyl_core::rotor::Dimensions,
    /// Each place's winding on the plane of its orbit.
    pub windings: (Winding, Winding),
    /// Their phase difference, whole turns and degrees, where the orbit
    /// planes coincide; `None` where they differ (the relative rotor holds
    /// that relation).
    pub phase: Option<Winding>,
    /// Their reaches from the anchor.
    pub reaches: (Q, Q),
    /// Their climbs off their planes.
    pub climbs: (Q, Q),
    /// Their further offsets.
    pub outs: (Q, Q),
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
    /// (galaxy units) and its direction (its plane's rotor applied to its
    /// phase at t, cores.rs Shape::place_at).
    pub fn core(reach: Q, plane: Rotor<3>, winding: Winding, r: &Q) -> Place {
        Place {
            reach,
            dir: on_plane(&plane, &winding),
            plane,
            winding,
            climb: Q::ZERO,
            out: Q::ZERO,
            sphere: r.clone(),
            galaxy: None,
            core: None,
        }
    }

    /// A place on the sphere of galaxy `galaxy`, of radius `rg`, its core at
    /// `core` on the universe's sphere.
    pub fn in_galaxy(
        reach: Q,
        (plane, winding): (Rotor<3>, Winding),
        climb: Q,
        galaxy: u32,
        rg: &Q,
        core: Place,
    ) -> Place {
        Place {
            reach,
            dir: on_plane(&plane, &winding),
            plane,
            winding,
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

    /// The separation from another place on the same sphere, every
    /// component kept apart (the pair rule, E019): the pivot between their
    /// directions (its named plane and turn), the relative rotor between
    /// their orbit planes read by its dimensions, both windings, their phase
    /// difference with its winding where the planes coincide, and the pairs
    /// of reaches, climbs and offsets. `None` for places of different
    /// galaxies, whose separation runs through their cores.
    pub fn separation(&self, other: &Place) -> Option<Separation> {
        if self.galaxy != other.galaxy {
            return None;
        }
        let relative = other.plane.inverse().then_after(&self.plane).dimensions();
        let phase = relative
            .planes
            .is_empty()
            .then(|| self.winding.less(&other.winding));
        Some(Separation {
            pivot: xyl_core::rotor::pivot(&self.dir, &other.dir, 64),
            relative,
            windings: (self.winding.clone(), other.winding.clone()),
            phase,
            reaches: (self.reach.clone(), other.reach.clone()),
            climbs: (self.climb.clone(), other.climb.clone()),
            outs: (self.out.clone(), other.out.clone()),
        })
    }

    /// The route to another place, as one length. On a shared sphere it is
    /// the legs of their separation in turn: the pivot's turn times the
    /// circle at the inner reach, R sin(s / R) (`bounce::curved_reach`), then
    /// the differences in reach and climb and both places' `out`. Between
    /// galaxies: each place's path to its core plus the route between the
    /// cores on the universe's sphere of radius `r`. No squared lengths or
    /// roots. A route's length adds its legs; as a separation it is one
    /// scalar, which its readers take until they read the separation itself.
    pub fn arc(&self, other: &Place, r: &Q) -> Encl {
        if let Some(sep) = self.separation(other) {
            let sphere = if self.galaxy.is_some() {
                self.sphere.clone()
            } else {
                r.clone()
            };
            // Stand-in S140: The arc is the pivot's turn over one scalar, not per plane.
            let turn = sep
                .pivot
                .map(|p| p.degrees.mul(&per_degree()).mid())
                .unwrap_or(Q::ZERO);
            let inner = sep
                .reaches
                .0
                .clone()
                .min(sep.reaches.1.clone())
                .max(Q::ZERO);
            let around = crate::bounce::curved_reach(&inner, &sphere);
            let legs = &(&(&sep.reaches.0 - &sep.reaches.1).abs()
                + &(&sep.climbs.0 - &sep.climbs.1).abs())
                + &(&sep.outs.0 + &sep.outs.1);
            return Encl::exact(&(&turn * &around) + &legs);
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
        let half = (Dy::pi()).div(&(Dy::int(2))).unwrap_or_default();
        let az = if x.signum() > 0 {
            (y.div(&x).unwrap_or_default()).atan()
        } else if x.signum() < 0 {
            (y.div(&x).unwrap_or_default()).atan()
                + if y.signum() >= 0 { Dy::pi() } else { -Dy::pi() }
        } else if y.signum() >= 0 {
            half
        } else {
            -half
        };
        let deg = |rad: Dy| (rad * Dy::int(180)).div(&(Dy::pi())).unwrap_or_default();
        let az_d = deg(az);
        let mut az_deg = az_d.trunc_i64().unwrap_or(if az_d.signum() > 0 {
            i64::MAX
        } else {
            i64::MIN
        });
        if az_deg < 0 {
            az_deg += 360;
        }
        let polar_d = deg(z.clamp(Dy::int(-1), Dy::ONE).acos());
        let polar = polar_d
            .trunc_i64()
            .unwrap_or(if polar_d.signum() > 0 {
                i64::MAX
            } else {
                i64::MIN
            })
            .clamp(0, 180);
        Cell {
            galaxy: self.galaxy,
            azimuth: az_deg.rem_euclid(360),
            polar,
        }
    }

    /// The exact arc converted to kiloparsecs (`galaxy::kpc_of_units`).
    pub fn arc_kpc(&self, other: &Place, r: &Q) -> Dy {
        Dy::from_q(&crate::galaxy::kpc_of_units(&self.arc(other, r).mid()))
    }
}

#[cfg(test)]
mod separation_tests {
    use super::*;

    /// The universe's radius breathes with the bell: over an echo it swings
    /// about its rest by at most the core's deficit at its rim and returns
    /// after a whole echo.
    #[test]
    fn the_universes_radius_breathes() {
        let _ = crate::genesis::origin();
        let echo = crate::circuit::echo();
        let rest = crate::galaxy::units_of_kpc(&crate::galaxy::radius_kpc());
        let rim = crate::circuit::anchor().rim;
        let deficit = Q::ONE.checked_div(&(&rim + &Q::ONE)).unwrap();
        let bound = &rest * &deficit;
        let (open, half) = (radius(Dy::ZERO), radius(echo.div(&Dy::int(2)).unwrap()));
        assert!(open > rest && half < rest, "{open} {half}");
        for k in 0..8 {
            let t = echo * Dy::int(k).div(&Dy::int(8)).unwrap();
            assert!((&radius(t) - &rest).abs() <= bound, "t {t}");
        }
        let back = radius(echo);
        assert!((&back - &open).abs() < &bound * &xyl_core::ratio!(1, 1_000_000));
    }

    /// A galaxy's sphere breathes with its core's bell: a homologue twice
    /// the home's scale rests at twice its radius, swings by the same share
    /// of it, and returns after twice the echo.
    #[test]
    fn a_galaxys_sphere_breathes_with_its_core() {
        let _ = crate::genesis::origin();
        let echo = crate::circuit::echo();
        let (home, twice) = (Q::ONE, Q::int(2));
        let open = galaxy_radius(&home, Dy::ZERO);
        let half = galaxy_radius(&home, echo.div(&Dy::int(2)).unwrap());
        assert!(open > half, "{open} {half}");
        // Its swing is the rim's deficit: (open - half) / (open + half).
        let rim = crate::circuit::anchor().rim;
        let deficit = Q::ONE.checked_div(&(&rim + &Q::ONE)).unwrap();
        let swing = (&open - &half).checked_div(&(&open + &half)).unwrap();
        assert!((&swing - &deficit).abs() < &deficit * &xyl_core::ratio!(1, 1_000_000));
        let big = galaxy_radius(&twice, Dy::ZERO);
        assert!((&big - &(&Q::int(2) * &open)).abs() < &open * &xyl_core::ratio!(1, 1_000_000));
        let tiny = &open * &xyl_core::ratio!(1, 1_000_000);
        let after = galaxy_radius(&twice, echo * Dy::int(2));
        assert!((&after - &big).abs() < &Q::int(2) * &tiny);
        let mid = galaxy_radius(&twice, echo);
        assert!(mid < big, "{mid} {big}");
    }

    /// Two places on one plane keep every component apart: their relative
    /// rotor turns no plane, their phase difference carries its whole turns
    /// and degrees, and the pivot between their directions turns that
    /// phase on its plane. Tilting one plane makes the relative rotor turn a
    /// plane and leaves no phase difference to read. Places of different
    /// galaxies have no direct separation; theirs runs through the cores.
    #[test]
    fn a_separation_keeps_its_components_apart() {
        let q = |n: i64, d: i64| Q::new(n, d).unwrap();
        let r = Q::int(10);
        let flat = Rotor::<3>::identity();
        let at = |turns: Q| Winding::of_turns(&turns);
        let a = Place::core(Q::ONE, flat.clone(), at(q(9, 4)), &r);
        let b = Place::core(Q::int(2), flat.clone(), at(Q::ZERO), &r);
        let sep = a.separation(&b).unwrap();
        assert!(sep.relative.planes.is_empty() && sep.relative.capacity == 0);
        let phase = sep.phase.clone().unwrap();
        assert_eq!((phase.whole, phase.degrees), (Q::int(2), Q::int(90)));
        let pivot = sep.pivot.clone().unwrap();
        // The circle point's half-angle is on the 2^-64 grid, so the turn is
        // 90 degrees to that grid.
        assert!((&pivot.degrees.mid() - &Q::int(90)).abs() < q(1, 1_000_000_000_000));
        assert_eq!(
            pivot.planes.iter().map(|p| p.0).collect::<Vec<_>>(),
            vec![(1, 2)]
        );
        assert_eq!(sep.reaches, (Q::ONE, Q::int(2)));
        let tilt = Rotor::<3>::of(
            &xyl_core::rotor::Turn {
                from: 2,
                toward: 3,
                degrees: Q::int(30),
            },
            64,
        )
        .unwrap();
        let c = Place::core(Q::ONE, tilt, at(Q::ZERO), &r);
        let tilted = a.separation(&c).unwrap();
        assert_eq!(tilted.relative.capacity, 1);
        assert!(tilted.phase.is_none());
        let mut d = b.clone();
        d.galaxy = Some(3);
        assert!(a.separation(&d).is_none());
    }
}
