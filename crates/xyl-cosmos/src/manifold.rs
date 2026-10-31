//! The system and inner manifolds: each planet's orbit about its star and
//! each moon's about its giant, as closed-form motions read at any age, with
//! their reaches, caps and the gaps between worlds (docs/model.md).
//!
//! Reads `systems::System`, `galaxy::Star`, `bodies::orbits` (the same drawn
//! orbits the worlds' settings use, keyed by `path`), `turning::turning_si`
//! and `twist_per_turn`, `package::Si`, and `xyl_core::{measure, motion,
//! rotor, units}`. A planet's vectors are in the system tier (60 r_g), a
//! moon's in the planet tier (r_g / 60^2); `dilation` moves between tiers
//! by an exact power of sixty. Ages are Gyr since the star's birth.
//!
//! Outputs:
//! - `manifolds` -> one `Manifold` per world: rate in degrees per Gyr from
//!   the turning law, eccentricity relaxing toward its forced value as a
//!   boost (`Manifold::e`), plane as an exact half-angle frame, twist per
//!   turn equal to the anchor's deficit. A giant's moons sit on a 2:1 chain:
//!   each rate is the innermost's halved k times, k the nearest power of two
//!   to the turning-law ratio.
//! - `Manifold::stand` and `bearing`: the vector and the bearing
//!   (direction, distance a (1 - e cos M)) at an age, as enclosures.
//! - `reach`, `Manifold::reach`: the arc `2 pi a m / (M + m)`.
//! - `cap_degrees`, `star_cap_degrees`: the share of the anchor's S^n a
//!   reach fills (`measure::share_turns`), times 360. `share`: belonging, 1
//!   at the centre, 1/2 at the reach, falling beyond.
//! - `gap`, `out_to_star`, `galaxy_place`: distances composed up the chain;
//!   turns between directions use a dyadic `acos`. `gap` and `out_to_star`
//!   return None for a world not in the list.
//!
//! Terms: manifold (docs/glossary.md).

use crate::bodies;
use crate::galaxy::Star;
use crate::package::Si;
use crate::systems::System;
use xyl_core::certified::{cos_sin_turns, pi, Encl};
use xyl_core::measure;
use xyl_core::motion::{Boost, Modes};
use xyl_core::rotor::{Half, Rotor};
use xyl_core::units::{Tier, Units};
use xyl_core::{Stream, Q};

const BITS: u32 = 80;

fn e(q: Q) -> Encl {
    Encl::exact(q)
}

/// 1 / (2 pi), enclosed once.
fn per_turn() -> &'static Encl {
    static K: std::sync::OnceLock<Encl> = std::sync::OnceLock::new();
    K.get_or_init(|| {
        let p = pi(100);
        let two = Q::int(2);
        Encl {
            lo: Q::ONE.checked_div(&(&p.hi * &two)).unwrap_or(Q::ZERO),
            hi: Q::ONE.checked_div(&(&p.lo * &two)).unwrap_or(Q::ZERO),
        }
        .outward(100)
    })
}

/// The scale between two tiers: a vector in `from`'s unit read in `to`'s,
/// an exact power of sixty.
pub fn dilation(from: Tier, to: Tier) -> Q {
    from.in_rg().checked_div(&to.in_rg()).unwrap_or(Q::ZERO)
}

/// A manifold's anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    /// The system's star.
    Star,
    /// A giant of the same system, by its index in the system's worlds.
    World(usize),
}

/// A world's manifold from its anchor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifold {
    /// Index of the world in the system's worlds.
    pub world: usize,
    /// What the manifold extends from: the star or a giant world.
    pub anchor: Anchor,
    /// The tier whose unit `a` and the manifold's vectors are in.
    pub tier: Tier,
    /// The semi-major axis in its tier's unit, exact.
    pub a: Q,
    /// Eccentricity relaxing from `e0` toward `e_forced` over `tau_e` Gyr
    /// (no relaxation when `tau_e` is 0).
    pub e0: Q,
    /// Forced eccentricity the orbit relaxes toward.
    pub e_forced: Q,
    /// Relaxation time of the eccentricity (Gyr); 0 for none.
    pub tau_e: Q,
    /// Turning rate (degrees per Gyr), exact, from the turning law.
    pub rate: Q,
    /// Mean anomaly at the star's birth (turns, an exact draw).
    pub m0: Q,
    /// The orbital plane by half-angles: periapsis about its axis,
    /// inclination about the line of nodes, node about the anchor's axis,
    /// composed into one exact frame.
    pub frame: Rotor,
    /// The anchor's deficit (degrees): the twist per turn.
    pub twist_deg: Q,
    /// Mass over the anchor's mass (the Hill scale ratio).
    pub mass_ratio: Q,
    /// On a captured chain: the innermost moon's index and this moon's step
    /// count along the chain.
    pub chain: Option<(usize, u32)>,
}

/// The manifolds of a star's planets and moons, from the same orbits used
/// for the worlds' settings (`bodies::orbits`, keyed by `path`), with each
/// world's mean anomaly at the star's birth drawn from its own stream.
pub fn manifolds(
    seed: u64,
    path: &[u64],
    sys: &System,
    star: &Star,
    units: &Units,
) -> Vec<Manifold> {
    let star_kg = &sys.spec.mass * &Si::star_mass_kg();
    let planets: Vec<&crate::systems::World> =
        sys.worlds.iter().filter(|w| w.host.is_none()).collect();
    let mut parts: Vec<xyl_core::rng::Part> = vec!["orbits".into()];
    parts.extend(path.iter().map(|&p| p.into()));
    let mut rng = Stream::derive(seed, &parts);
    let orbits = bodies::orbits(&star_kg, &planets, &mut rng);
    let in_tier = |m: &Q, tier: Tier| units.in_tier(m, tier);
    let mut planet = 0usize;
    let mut out = Vec::with_capacity(sys.worlds.len());
    // the innermost moon of each giant, for its chain
    let innermost = |host: usize| {
        sys.worlds
            .iter()
            .enumerate()
            .filter(|(_, w)| w.host == Some(host))
            .min_by(|a, b| a.1.a_host_radii.cmp(&b.1.a_host_radii))
            .map(|(i, _)| i)
    };
    for (i, w) in sys.worlds.iter().enumerate() {
        let mut draw = Stream::derive(seed, &["anomaly".into(), star.n.into(), (i as u64).into()]);
        let m0 = xyl_core::dyadic::Dy::unit(draw.next_u64()).to_q();
        let mass_kg = &w.mass * &Si::world_mass_kg();
        match (w.host, &w.a_host_radii) {
            (Some(h), Some(a_r)) => {
                let Some(g) = sys.worlds.get(h) else { continue };
                let host_kg = &g.mass * &Si::world_mass_kg();
                let a_m = &(a_r * &g.radius) * &Si::world_radius_m();
                let ef = w.e_forced.clone().unwrap_or(Q::ZERO);
                let chain = innermost(h).and_then(|first| {
                    let a_in = sys.worlds[first].a_host_radii.clone()?;
                    // steps along the 2:1 chain: the innermost's turning over
                    // this moon's is 2^k (turning law at each, no squared
                    // distance)
                    let a_in_m = &(&a_in * &g.radius) * &Si::world_radius_m();
                    let ratio = crate::turning::turning_si(&host_kg, &a_in_m)
                        .checked_div(&crate::turning::turning_si(&host_kg, &a_m))?;
                    let k = (0..24u32).min_by_key(|&k| {
                        let two = (0..k).fold(Q::ONE, |acc, _| &acc * &Q::int(2));
                        (&ratio - &two).abs()
                    })?;
                    Some((first, k))
                });
                out.push(Manifold {
                    world: i,
                    anchor: Anchor::World(h),
                    tier: Tier::Planet,
                    a: in_tier(&a_m, Tier::Planet),
                    e0: ef.clone(),
                    e_forced: ef,
                    tau_e: Q::ZERO,
                    rate: crate::turning::turning_si(&host_kg, &a_m),
                    m0,
                    frame: Rotor::identity(),
                    twist_deg: crate::turning::twist_per_turn(&host_kg, &a_m),
                    mass_ratio: mass_kg.checked_div(&host_kg).unwrap_or(Q::ZERO),
                    chain,
                });
            }
            _ => {
                let Some(o) = orbits.get(planet) else {
                    continue;
                };
                planet += 1;
                out.push(Manifold {
                    world: i,
                    anchor: Anchor::Star,
                    tier: Tier::System,
                    a: in_tier(&o.a_m, Tier::System),
                    e0: o.e0.clone(),
                    e_forced: o.e_forced.clone(),
                    tau_e: o.tau_e.clone(),
                    rate: crate::turning::turning_si(&star_kg, &o.a_m),
                    m0,
                    frame: frame_of(&o.inc, &o.node, &o.peri),
                    twist_deg: crate::turning::twist_per_turn(&star_kg, &o.a_m),
                    mass_ratio: mass_kg.checked_div(&star_kg).unwrap_or(Q::ZERO),
                    chain: None,
                });
            }
        }
    }
    out
}

/// A manifold's state at an age: eccentricity, mean phase (turns) and
/// vector from its anchor (its tier's unit).
#[derive(Clone, Debug)]
pub struct Stand {
    /// Eccentricity at the age.
    pub e: Encl,
    /// Mean phase at the age (turns).
    pub mean: Encl,
    /// Vector from the anchor (the tier's unit).
    pub vector: [Encl; 3],
}

impl Manifold {
    /// Turn rate (turns per Gyr, exact): the turning over 360; on a captured
    /// chain, the innermost moon's halved `k` times.
    pub fn rate(&self, all: &[Manifold]) -> Encl {
        let own = |m: &Manifold| e(m.rate.checked_div(&Q::int(360)).unwrap_or(Q::ZERO));
        match self.chain {
            Some((first, k)) if k > 0 => {
                let inner = all
                    .iter()
                    .find(|m| m.world == first)
                    .map(own)
                    .unwrap_or_else(|| own(self));
                let halves = (0..k).fold(Q::ONE, |acc, _| &acc * &Q::new(1, 2).unwrap_or(Q::ONE));
                inner.mul(&e(halves))
            }
            _ => own(self),
        }
    }

    /// Eccentricity at `age` Gyr: the relaxation as a boost.
    pub fn e(&self, age: &Q) -> Encl {
        if self.tau_e.is_zero() {
            return e(self.e_forced.clone());
        }
        let k = Q::ONE.checked_div(&self.tau_e).unwrap_or(Q::ZERO);
        Modes::of(vec![
            Boost::e(self.e_forced.clone(), Q::ZERO),
            Boost::e(&self.e0 - &self.e_forced, -k),
        ])
        .value(age)
    }

    /// State at `age` Gyr (since the star's birth).
    pub fn stand(&self, age: &Q, all: &[Manifold]) -> Stand {
        let ecc = self.e(age);
        let mean = e(self.m0.clone()).add(&self.rate(all).mul(&e(age.clone())));
        // the orbit as pivots: the main pivot at its turning rate, and the
        // stretch as a second pivot at twice the rate, radius a e / 2, about a
        // centre a 3e/2 behind the anchor - periapsis a (1 - e), apoapsis
        // a (1 + e), closed form at any age with no equation solved
        let (cm, sm) = cos_sin_turns(&mean);
        let (c2, s2) = (
            cm.mul(&cm).sub(&sm.mul(&sm)),
            e(Q::int(2)).mul(&cm).mul(&sm),
        );
        let a = e(self.a.clone());
        let half_e = ecc.mul(&e(Q::new(1, 2).unwrap_or(Q::ZERO)));
        let x = a.mul(&cm.add(&half_e.mul(&c2.sub(&e(Q::int(3))))));
        let y = a.mul(&sm.add(&half_e.mul(&s2)));
        // the twist over time (uniform in degrees), then the plane's frame
        // (half-angles, exact)
        let (cw, sw) = cos_sin_turns(&self.twist(&ecc, all).mul(&e(age.clone())));
        let (xp, yp) = (x.mul(&cw).sub(&y.mul(&sw)), x.mul(&sw).add(&y.mul(&cw)));
        let vector = self.frame.apply(&[xp, yp, e(Q::ZERO)]);
        Stand {
            e: ecc,
            mean,
            vector,
        }
    }

    /// Periapsis twist (turns per Gyr): the anchor's deficit each turn,
    /// independent of the ellipse's shape.
    fn twist(&self, _ecc: &Encl, all: &[Manifold]) -> Encl {
        self.rate(all).mul(&e(self
            .twist_deg
            .checked_div(&Q::int(360))
            .unwrap_or(Q::ZERO)))
    }

    /// Reach (its tier's unit): the arc of the anchor's circle it holds, its
    /// deficit share of the pair m / (M + m) of a turn at its distance (the
    /// turning law's form, delta = M / (r + M)). No root.
    pub fn reach(&self) -> Encl {
        reach(&e(self.a.clone()), &self.mass_ratio)
    }

    /// Bearing from the anchor at `age` Gyr: the unit direction the plane's
    /// frame turns its phase to (mean turn plus periapsis twist; exact
    /// half-angles, unit by construction), and the distance a (1 - e cos M),
    /// linear in the elements.
    pub fn bearing(&self, age: &Q, all: &[Manifold]) -> Bearing {
        let ecc = self.e(age);
        let mean = e(self.m0.clone()).add(&self.rate(all).mul(&e(age.clone())));
        let (cm, _) = cos_sin_turns(&mean);
        let along = mean.add(&self.twist(&ecc, all).mul(&e(age.clone())));
        let (c, s) = cos_sin_turns(&along);
        Bearing {
            dir: self.frame.apply(&[c, s, e(Q::ZERO)]).map(|x| x.mid()),
            r: &self.a * &(&Q::ONE - &(&ecc.mid() * &cm.mid())),
        }
    }
}

/// A world's bearing from its anchor (`Manifold::bearing`): unit direction
/// and distance in its tier's unit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bearing {
    /// Unit direction from the anchor (exact).
    pub dir: [Q; 3],
    /// Distance from the anchor, a (1 - e cos M), in the tier's unit.
    pub r: Q,
}

/// The turn between two unit directions (radians): acos of their dot product.
pub fn turn_between(a: &[Q; 3], b: &[Q; 3]) -> Q {
    let agree = a.iter().zip(b).fold(Q::ZERO, |acc, (x, y)| &acc + &(x * y));
    xyl_core::dyadic::Dy::from_q(&agree).acos().to_q()
}

/// The gap between two worlds of one system at `age` (system tier units).
/// Within one anchor: the turn between their directions times the inner
/// distance, plus the difference in distances. A moon first adds its own
/// distance to its giant. No squared lengths or roots.
pub fn gap(all: &[Manifold], wa: usize, wb: usize, age: &Q) -> Option<Q> {
    if wa == wb {
        return Some(Q::ZERO);
    }
    let (ma, mb) = (
        all.iter().find(|m| m.world == wa)?,
        all.iter().find(|m| m.world == wb)?,
    );
    let out = |m: &Manifold| &m.bearing(age, all).r * &dilation(m.tier, Tier::System);
    if ma.anchor == mb.anchor {
        let (ba, bb) = (ma.bearing(age, all), mb.bearing(age, all));
        let inner = ba.r.clone().min(bb.r.clone());
        let sweep = &turn_between(&ba.dir, &bb.dir) * &inner;
        return Some(&(&sweep + &(&ba.r - &bb.r).abs()) * &dilation(ma.tier, Tier::System));
    }
    match (ma.anchor, mb.anchor) {
        (Anchor::World(h), _) if h == wb => Some(out(ma)),
        (_, Anchor::World(h)) if h == wa => Some(out(mb)),
        (Anchor::World(h), _) => Some(&out(ma) + &gap(all, h, wb, age)?),
        (_, Anchor::World(h)) => Some(&out(mb) + &gap(all, wa, h, age)?),
        _ => None,
    }
}

/// A child's reach at distance `a` from its anchor, with mass ratio `ratio`
/// to the anchor: the arc of the anchor's circle held by its deficit share,
/// 2 pi a ratio / (1 + ratio).
pub fn reach(a: &Encl, ratio: &Q) -> Encl {
    let held = ratio.checked_div(&(&Q::ONE + ratio)).unwrap_or(Q::ZERO);
    a.mul(&e(held)).mul(&pi(BITS).mul(&e(Q::int(2))))
}

/// A plane's frame from its drawn inclination (rad), node and periapsis
/// (turns): each as a half-angle on the grid, composed exactly as node after
/// inclination after periapsis.
fn frame_of(inc: &Q, node: &Q, peri: &Q) -> Rotor {
    let inc_turns = e(inc.clone()).mul(per_turn()).mid();
    let h = |t: &Q| Half::of_turns(t, 40);
    Rotor::about(2, &h(node))
        .then_after(&Rotor::about(0, &h(&inc_turns)))
        .then_after(&Rotor::about(2, &h(peri)))
}

/// The volume a cap holds: seen from a point, a reach `x` (arc over
/// distance) subtends x radians, x / 2 pi of a turn, at most half; that cone
/// holds share_n of the point's sphere of directions, n the dimension. Taken
/// from the angle (measure.rs), without cosines or roots. Enclosed.
fn cone_share(n: u32, x: &Encl) -> Encl {
    let half = Q::new(1, 2).unwrap_or(Q::ZERO);
    let turns = |q: &Q, up: bool| {
        let t = x_turns(q, up).max(Q::ZERO).min(half.clone());
        xyl_core::dyadic::Dy::from_q(&t)
    };
    let at = |q: &Q, up: bool| measure::share_turns(n, turns(q, up)).to_q();
    Encl {
        lo: at(&x.lo, false),
        hi: at(&x.hi, true),
    }
}

/// An arc over its distance as turns: x / 2 pi, bounded outward.
fn x_turns(x: &Q, up: bool) -> Q {
    let p = per_turn();
    x * if up { &p.hi } else { &p.lo }
}

/// A child's cap on its anchor's sphere of directions, in degrees as volume:
/// the share of the anchor's S^n its reach fills, times 360.
pub fn cap_degrees(n: u32, reach: &Encl, d: &Encl) -> Encl {
    let x = Encl {
        lo: reach.lo.checked_div(&d.hi).unwrap_or(Q::ZERO),
        hi: reach.hi.checked_div(&d.lo).unwrap_or(Q::ONE),
    };
    cone_share(n, &x).mul(&e(Q::int(360)))
}

/// A point's belonging to a child of `reach` at distance `d`, as a share of
/// directions of S^n. Outside the reach: the share of the point's directions
/// the child's reach fills (1/2 at the edge, falling smoothly). Inside: one
/// minus the share the distance would fill of the child's (1 at the centre).
pub fn share(n: u32, d: &Encl, reach: &Encl) -> Encl {
    let out = |num: &Encl, den: &Encl| Encl {
        lo: num.lo.checked_div(&den.hi).unwrap_or(Q::ZERO),
        hi: num.hi.checked_div(&den.lo).unwrap_or(Q::ONE),
    };
    if d.lo >= reach.hi {
        cone_share(n, &out(reach, d))
    } else if d.hi <= reach.lo {
        let s = cone_share(n, &out(d, reach));
        e(Q::ONE).sub(&s)
    } else {
        let half = Q::new(1, 2).unwrap_or(Q::ZERO);
        let (a, b) = (
            cone_share(n, &out(reach, d)),
            e(Q::ONE).sub(&cone_share(n, &out(d, reach))),
        );
        Encl {
            lo: a.lo.min(b.lo).min(half.clone()),
            hi: a.hi.max(b.hi).max(half),
        }
    }
}

/// A star's cap on its core's sphere of directions (degrees of S^360's
/// volume): its reach, the deficit share m / (M(<R) + m) of the circle at R,
/// over R, with M(<R) the mass inside its orbit (scaled to its galaxy).
pub fn star_cap_degrees(star: &Star, star_suns: &Q) -> Encl {
    let inside =
        &crate::galaxy::inside_mass(&star.r) * &crate::cores::ratio_of_scale(&star.scale.to_q());
    let ratio = star_suns.checked_div(&inside).unwrap_or(Q::ZERO);
    cap_degrees(measure::UNIVERSE_N, &reach(&e(Q::ONE), &ratio), &e(Q::ONE))
}

/// A world's distance to its star at `age` (system tier units): a planet's
/// own distance; a moon's own distance dilated from the planet tier (an
/// exact power of sixty) plus its giant's.
pub fn out_to_star(all: &[Manifold], world: usize, age: &Q) -> Option<Q> {
    let m = all.iter().find(|m| m.world == world)?;
    let own = &m.bearing(age, all).r * &dilation(m.tier, Tier::System);
    match m.anchor {
        Anchor::Star => Some(own),
        Anchor::World(h) => Some(&own + &out_to_star(all, h, age)?),
    }
}

/// A world's place at `t` Gyr on its galaxy's sphere, composed up the
/// chain: the star's bearing from its core, offset by the world's distance
/// to the star dilated to the galaxy tier; the core lies on the universe's
/// sphere of radius `r`.
pub fn galaxy_place(
    star: &Star,
    all: &[Manifold],
    world: usize,
    t: xyl_core::dyadic::Dy,
    r: &Q,
) -> Option<crate::chain::Place> {
    let age = (t - xyl_core::dyadic::Dy::from_q(&star.born)).to_q();
    let out = &out_to_star(all, world, &age)? * &dilation(Tier::System, Tier::Galaxy);
    Some(star.place(t, r).standing_off(&out))
}
