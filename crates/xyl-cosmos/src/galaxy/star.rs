//! Sample stars: their orbits about their galactic cores, placement by
//! irrational rotations of the index, birth, mass ([`imf`], core masses)
//! and place at any `t`. Times are rings after the Big Bang, rates per ring.

use super::{
    birth, core_of, core_reach, density, inside_mass, nucleus_mass, nucleus_reach, reach_at,
    sample, stellar_mass, vertical, vertical_frequency, Pop, Vertical, CORE_SHIFT, GRID_BITS,
    NUCLEUS_BASE, NUCLEUS_SAMPLE,
};
use crate::bang::BigBang;
use crate::chain::Place;
use xyl_core::certified::{cos_sin_turns, down, pi, Encl};
use xyl_core::dyadic::Dy;
use xyl_core::measure;
use xyl_core::units::Tier;
use xyl_core::{Stream, Q};

/// 1 / (2 pi), enclosed once: converts the turning law's and the vertical
/// swing's radians per ring to turns.
fn inv_two_pi() -> &'static Encl {
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

/// A star's galactic manifold from the core (`chain`): its radius, turning
/// rate and plane. The plane's angles are held as exact half-angles, and the
/// rate comes from the turning law, exact in radians per ring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Orbit {
    /// Orbit radius (galaxy rings), scaled by the galaxy's length scale.
    pub radius: Dy,
    /// Turning rate (radians per ring), from the turning law.
    pub rate: Dy,
    /// Orbital phase at the Big Bang (turns, exact).
    pub turns0: Dy,
    /// Tilt of the plane about the line of nodes, as an exact half-angle
    /// (c, s); (1, 0) for a disc star, whose plane is the disc's.
    pub tilt: (Dy, Dy),
    /// Node of the plane about the core's axis, as a half-angle (c, s); (1, 0)
    /// for a disc star.
    pub node: (Dy, Dy),
    /// A disc star's vertical oscillation: the draw that sets its speed and
    /// its phase (turns, exact); `None` for bulge and nucleus stars.
    pub swing: Option<(Dy, Dy)>,
}

impl Orbit {
    /// The plane's frame: the tilt about x, then the node about the core's
    /// axis, composed exactly by half-angles.
    pub fn frame(&self) -> xyl_core::rotor::Rotor {
        use xyl_core::rotor::{Half, Rotor};
        let h = |p: (Dy, Dy)| Half {
            c: p.0.to_q(),
            s: p.1.to_q(),
        };
        Rotor::about(2, &h(self.node)).then_after(&Rotor::about(0, &h(self.tilt)))
    }

    /// Orbital phase at `t` rings (turns): the initial phase plus the
    /// turning, `turns0 + omega t / (2 pi)`.
    pub fn turns_at(&self, t: Dy) -> Dy {
        self.turns0 + self.rate * t * Dy::from_q(&inv_two_pi().mid())
    }
}

/// A sample star, as drawn at the Big Bang.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Star {
    /// Index in the galaxy's sample.
    pub n: u64,
    /// Home-equivalent distance from the core (galaxy rings), exact; sets the
    /// gas conditions the star formed in.
    pub r: Q,
    /// Birth time (rings after the Big Bang), rounded to the grid.
    pub born: Q,
    /// Mass (core masses), rounded down to the 2^-64 grid ([`imf`]).
    pub mass: Q,
    /// A binary's companion mass (core masses); zero for a single star.
    pub companion: Q,
    /// The star's galactic manifold: radius, turning, phase and plane.
    pub orbit: Orbit,
    /// Population.
    pub pop: Pop,
    /// Stellar mass this sample star represents (core masses), scaled by
    /// the hole ratio for a homologue.
    pub stands_for: Dy,
    /// The star's galactic core index (0 for the home galaxy). `r` above is
    /// the home-equivalent radius used for gas and chemistry; `orbit.radius`
    /// is the actual radius.
    pub core: u32,
    /// The galaxy's length scale over the home galaxy's (1 for the home galaxy).
    pub scale: Dy,
    /// The galaxy's core shape: its bearing on the universe's sphere, its
    /// plane and its circulation about the throat.
    pub galaxy: crate::cores::Shape,
    /// The galaxy's hole mass over the home's; scales the turning masses.
    pub mass_ratio: Dy,
}

impl Star {
    /// The star's reach about its galactic core (galaxy rings). A star relates
    /// to the core it is anchored on, in the lowest common tier (the
    /// containment chain), so its reach is the orbit radius in its galaxy
    /// (`stars::reach_rings`); the galaxy's reach about the universal core is
    /// one tier up and is not added to it.
    pub fn reach_rings(&self) -> Dy {
        self.orbit.radius.max(Dy::ZERO)
    }

    /// The galaxy's turning rate (radians per ring) at radius `own` (galaxy
    /// rings) in the star's galaxy: the turning law over the home-equivalent
    /// mass inside times the hole ratio (core masses), as for the star's
    /// orbital rate (used for constellation shear).
    pub fn rate_at_dy(&self, own: Dy) -> Dy {
        let scale = self.scale.max(Dy::sci(1, -12));
        let inside = super::inside_mass_dy(own / scale) * self.mass_ratio;
        crate::turning::turning_dy(inside, own, Tier::Galaxy)
    }

    /// Whether the star's home-equivalent radius is within the core's reach.
    pub fn in_core(&self) -> bool {
        self.r <= core_reach()
    }

    /// Bearing from the star's galactic core at `t` rings (galaxy rings,
    /// relative to the core's place): reach (orbit radius), unit direction
    /// (the plane's frame applied to the phase; exact half-angles), and climb
    /// off the plane (a disc star's vertical swing, zero otherwise).
    pub fn bearing(&self, t: Dy) -> (Q, [Q; 3], Q) {
        let o = &self.orbit;
        let (c, s) = cos_sin_turns(&Encl::exact(o.turns_at(t).to_q()));
        let dir = o
            .frame()
            .apply(&[c, s, Encl::exact(Q::ZERO)])
            .map(|x| x.mid());
        let climb = match o.swing {
            Some((u, phase)) => {
                // `z = A sin(phase + nu t / (2 pi) turns)`, evaluated at the
                // home-equivalent radius, A scaled by the galaxy's length scale.
                let born = Dy::from_q(&self.born);
                let age = (t - born).max(Dy::ZERO);
                let r_home = Dy::from_q(&self.r);
                let v = vertical(r_home, born, age, u, phase);
                let turned = Encl::exact((vertical_frequency(r_home) * t).to_q()).mul(inv_two_pi());
                let swing = Encl::exact(phase.to_q()).add(&turned);
                Encl::exact((v.amp * self.scale).to_q())
                    .mul(&cos_sin_turns(&swing).1)
                    .mid()
            }
            None => Q::ZERO,
        };
        (o.radius.to_q(), dir, climb)
    }

    /// The star's place at `t` rings on its galaxy's sphere, with its core on
    /// the universe's sphere of radius `r`.
    pub fn place(&self, t: Dy, r: &Q) -> Place {
        let (reach, dir, climb) = self.bearing(t);
        let core = self.galaxy.place_at(t, r);
        Place::in_galaxy(
            reach,
            dir,
            climb,
            self.core,
            &crate::chain::galaxy_radius(&self.scale.to_q()),
            core,
        )
    }

    /// The vertical orbit at `age` rings, evaluated at the home-equivalent
    /// radius as in [`Star::bearing`] (`None` for a bulge or nucleus star,
    /// which does not orbit in the disc's plane). Lengths are home-equivalent
    /// galaxy rings; the star's galaxy scales them by `scale`.
    pub fn vertical_at(&self, age: Dy) -> Option<Vertical> {
        let (r, born) = (Dy::from_q(&self.r), Dy::from_q(&self.born));
        self.orbit
            .swing
            .map(|(u, phase)| vertical(r, born, age, u, phase))
    }

    /// The share of the vertical oscillation spent inside the crowded layer
    /// at `age` rings (1 for a star without a vertical orbit).
    pub fn exposure(&self, age: Dy) -> Dy {
        self.vertical_at(age).map(|v| v.exposure).unwrap_or(Dy::ONE)
    }
}

/// The grid star masses are rounded down to (core masses): 2^-64.
const MASS_BITS: u32 = 64;

/// A sample star's place in its galaxy before scaling: home-equivalent
/// radius (galaxy rings), population, tilt and node half-angles and initial phase.
struct Placement {
    radius: Dy,
    pop: Pop,
    tilt: (Dy, Dy),
    node: (Dy, Dy),
    turns0: Dy,
}

/// The placement of sample star `local` (its index within its population),
/// after Douady and Couder's phyllotaxis: the fractional part of k times an
/// irrational turn is read as a share. The radius uses the silver turn; a
/// disc star's phase uses the golden turn; a bulge or nucleus star's tilt
/// uses the plastic turn, its node the golden turn and its phase the
/// sqrt 3 - 1 turn. Every prefix of the sample is thus evenly spread over
/// the rings and over the sphere of orbit normals.
fn placement(local: u64, nucleus: bool) -> Placement {
    let pivot = |by: &Q| Dy::from_q(&(&Q::int(i64::try_from(local).unwrap_or(0)) * by).frac());
    // Radius: the share as a fraction of the rings' cumulative holding (of
    // the nucleus's rings only, for a nucleus star).
    let radius = if nucleus {
        reach_at(pivot(&silver_turns()), Some(&nucleus_reach()))
    } else {
        reach_at(pivot(&silver_turns()), None)
    };
    let pop = if nucleus {
        Pop::Nucleus
    } else if radius.to_q() <= core_reach() {
        Pop::Bulge
    } else {
        Pop::Disc
    };
    let (tilt, node, turns0) = if pop == Pop::Disc {
        (
            (Dy::ONE, Dy::ZERO),
            (Dy::ONE, Dy::ZERO),
            pivot(&golden_turns()),
        )
    } else {
        (
            tilt_of(pivot(&plastic_turns())),
            node_of(pivot(&golden_turns())),
            pivot(&third_turns()),
        )
    };
    Placement {
        radius,
        pop,
        tilt,
        node,
        turns0,
    }
}

/// A star's turning rate (radians per ring): the turning law
/// `omega = delta bell / r` over the home-equivalent mass inside `radius`
/// (core masses) times the hole ratio, at the scaled radius
/// `radius x scale` (galaxy rings); both radii are at least 1e-6 galaxy
/// rings and rounded down to the 2^-60 grid.
fn turning_rate(radius: Dy, scale: Dy, mass_ratio: Dy) -> Dy {
    let r_q = down(&radius.max(Dy::sci(1, -6)).to_q(), 60);
    let own_r = down(&(radius * scale).max(Dy::sci(1, -6)).to_q(), 60);
    Dy::from_q(&crate::turning::turning(
        &(&inside_mass(&r_q) * &mass_ratio.to_q()),
        &own_r,
        Tier::Galaxy,
    ))
}

/// The home-galaxy stellar mass one sample star stands for (core masses):
/// the population's mass (nucleus, or the whole galaxy) over the
/// population's sample size.
fn stands_for(bang: &BigBang, nucleus: bool) -> Dy {
    if nucleus {
        Dy::from_q(&nucleus_mass()) / Dy::int(i64::try_from(NUCLEUS_SAMPLE).unwrap_or(1))
    } else {
        Dy::from_q(&stellar_mass()) / Dy::int(i64::try_from(sample(bang)).unwrap_or(1))
    }
}

/// Sample star `n` of `seed`'s universe, as drawn at the Big Bang. `None`
/// where the galactic core has no shape, the star's ring never lights or the
/// IMF refuses the draw.
pub fn star(seed: u64, bang: &BigBang, n: u64) -> Option<Star> {
    // Core 0 is the home galaxy; another core's galaxy is the home galaxy's
    // homologue, its lengths scaled by `scale` and its masses by the hole
    // ratio (`cores`).
    let core = core_of(n);
    let shape = crate::cores::shape(seed, bang, core)?;
    let (scale, mass_ratio) = (shape.scale(), Dy::from_q(&shape.mass_ratio()));
    let nucleus = (n & ((1u64 << CORE_SHIFT) - 1)) >= NUCLEUS_BASE;
    let mut rng = Stream::derive(
        seed,
        &[if nucleus { "nucleus" } else { "star" }.into(), n.into()],
    );
    let mut u = || Dy::unit(rng.next_u64());
    // Five draws (population, radius, tilt, node, phase) are not used:
    // placement follows from the index. They are still taken so the later
    // draws keep their positions in the stream.
    let _pop_draw = u();
    let local = if nucleus {
        (n & ((1u64 << CORE_SHIFT) - 1)) - NUCLEUS_BASE
    } else {
        n & ((1u64 << CORE_SHIFT) - 1)
    };
    let _replaced = (u(), u(), u(), u());
    let place = placement(local, nucleus);
    let radius = place.radius;
    let swing = if place.pop == Pop::Disc {
        Some((u(), u()))
    } else {
        None
    };
    let rate = turning_rate(radius, scale, mass_ratio);
    let stands_for = stands_for(bang, nucleus);
    let r = down(&radius.to_q(), if nucleus { 60 } else { GRID_BITS });
    // Birth: the ring's first light, then a draw along the ring's
    // star-formation history (`chem::Chemistry::birth_along`), so a ring
    // forms stars throughout its history; without chemistry, first light.
    let u_born = u();
    let u_mass = u();
    let u_bin = u();
    let u_comp = u();
    let first = birth(&r)?;
    let born = crate::chem::Chemistry::cached(bang)
        .map(|c| c.birth_along(&r, &first, u_born))
        .unwrap_or(first);
    let born = down(&born, GRID_BITS);
    // The mass is read at the home-equivalent radius: a homologue's star is
    // cut from the same piece of the chain as the home galaxy's.
    let mass = down(&imf(&u_mass.to_q(), &r)?.mid(), MASS_BITS);
    if mass.signum() <= 0 {
        return None;
    }
    // For u_bin < 1/3 the star is a binary, with companion mass
    // (0.1 + 0.9 u_comp) times the star's mass.
    let companion = if u_bin < Dy::ratio(1, 3) {
        down(
            &(&mass * &(&Q::ratio(1, 10) + &(&Q::ratio(9, 10) * &u_comp.to_q()))),
            GRID_BITS,
        )
    } else {
        Q::ZERO
    };
    let orbit = Orbit {
        radius: radius * scale,
        rate,
        turns0: place.turns0,
        tilt: place.tilt,
        node: place.node,
        swing,
    };
    Some(Star {
        n,
        r,
        born,
        mass,
        companion,
        orbit,
        pop: place.pop,
        stands_for: stands_for * mass_ratio,
        core,
        scale,
        galaxy: shape.clone(),
        mass_ratio,
    })
}

/// The placement's irrational turns, each held exactly as a convergent of
/// its defining recurrence: the golden angle (1 / phi^2 of a turn, Fibonacci
/// F39 / F41), the silver (sqrt 2 - 1, Pell), the plastic (1 / rho, Padovan
/// P(n - 1) / P(n)) and sqrt 3 - 1 (convergents of [1; 1, 2, 1, 2, ...]).
/// Their limits are independent over the rationals, so the four sequences
/// of fractional parts do not fall into step.
pub(crate) fn golden_turns() -> Q {
    Q::ratio(63_245_986, 165_580_141)
}
fn silver_turns() -> Q {
    Q::ratio(470_832, 1_136_689)
}
pub(crate) fn plastic_turns() -> Q {
    Q::ratio(90_342_335_288_610_825, 119_678_113_856_248_465)
}
fn third_turns() -> Q {
    Q::ratio(8_155_103_542_731_753, 11_140_078_609_864_049)
}

/// A tilt holding `u` of S^360's measure (concentrated toward 90 degrees,
/// the measure's equator), returned as the half-angle direction (cos, sin),
/// rounded to the grid.
pub(crate) fn tilt_of(u: Dy) -> (Dy, Dy) {
    let theta = measure::angle_holding(measure::UNIVERSE_N, Dy::ONE - u);
    let (s, c) = (Dy::PI * theta).sin_cos();
    let fix = |x: Dy| Dy::from_q(&down(&x.max(Dy::ZERO).to_q(), GRID_BITS));
    (fix(c), fix(s))
}

/// A node at `u` of a turn about the axis, as the half-angle direction
/// (cos, sin) rounded to the grid.
pub(crate) fn node_of(u: Dy) -> (Dy, Dy) {
    let (s, c) = (Dy::PI * u).sin_cos();
    let fix = |x: Dy| Dy::from_q(&down(&x.to_q(), GRID_BITS));
    (fix(c), fix(s))
}

/// The initial mass function, inverse-transformed at the uniform `u`: a
/// star's mass (core masses) at reach `r` (galaxy rings) of the home galaxy.
///
/// The piece at `r` gives one star mass, `m_p = s c` ([`super::piece_star_mass`]:
/// the share `s` of the piece's infall the star tier's cut holds,
/// `disc::star_cut`, times the infall's holding `c`), the most one star of
/// the piece holds. The draw spreads the piece's stars over shares `x` of
/// `m_p` by the deficit form `F(x) = x / (x + b)` on `[0, 1]`, the share of
/// stars holding at most `x m_p`, with the break `b = d / (1 + d)` the
/// deficit share of the ring's crowding `d` (`density` at `r`), so the break
/// is heavier where rings crowd. The draw inverts in first degree: with
/// `y = u F(1) = u / (1 + b)`, `x = b y / (1 - y) = b u / (1 + b - u)`, and
/// `m = x m_p`. Exact. `None` where no lit piece lies at `r`, the ring has
/// no crowding or the draw gives no mass (`u = 0`).
pub fn imf(u: &Q, r: &Q) -> Option<Encl> {
    let top = super::piece_star_mass(r)?;
    let d = density(r).mid().max(Q::ZERO);
    let b = d.checked_div(&(&Q::ONE + &d))?;
    let u = u.clone().max(Q::ZERO).min(Q::ONE);
    let x = (&b * &u).checked_div(&(&(&Q::ONE + &b) - &u))?;
    let m = &top * &x;
    (m.signum() > 0).then(|| Encl::exact(m))
}
