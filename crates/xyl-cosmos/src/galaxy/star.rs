//! Sample stars: their orbits about their galactic cores, placement by
//! irrational pivots, birth, mass ([`imf`]) and place at any `t`.

use super::{
    birth_gyr, core_kpc, core_of, density, inside_mass, nucleus_kpc, nucleus_mass, reach_at,
    sample, stellar_mass, vertical, vertical_frequency, Pop, Vertical, CORE_SHIFT, GRID_BITS,
    NUCLEUS_BASE, NUCLEUS_SAMPLE,
};
use crate::bang::BigBang;
use crate::chain::Place;
use xyl_core::certified::{cos_sin_turns, down, pi, Encl};
use xyl_core::dyadic::Dy;
use xyl_core::measure;
use xyl_core::{Stream, Q};

/// 1 / (2 pi), enclosed once: converts the vertical swing's radians per Gyr
/// to turns (the plane's restoring pull is not yet expressed in angles).
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

/// A star's galactic manifold from the core (chain.rs): its stretch, turn and
/// plane. Every angle is held as an exact fraction of a turn, and the rate
/// comes from the turning law, so the phase is exact at every t.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Orbit {
    /// Orbit radius (kpc) and turning rate (degrees per Gyr, exact).
    pub radius: Dy,
    /// Turning rate (degrees per Gyr, exact), from the turning law.
    pub rate: Dy,
    /// Orbital phase at the Big Bang (turns, exact).
    pub turns0: Dy,
    /// The orbital plane as exact half-angles (c, s): the tilt about the line
    /// of nodes and the node about the core's axis; both (1, 0) for a disc
    /// star, whose plane is the disc's.
    pub tilt: (Dy, Dy),
    /// Node of the plane about the core's axis, as a half-angle (c, s).
    pub node: (Dy, Dy),
    /// A disc star's vertical oscillation: the draw that sets its speed, and
    /// its phase (turns, exact).
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

    /// Orbital phase at `t` Gyr (turns): initial phase plus turning, exact.
    pub fn turns_at(&self, t: Dy) -> Dy {
        self.turns0 + self.rate * t / Dy::int(360)
    }
}

/// A sample star, as drawn at the Big Bang.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Star {
    /// Index in the galaxy's sample.
    pub n: u64,
    /// Home-equivalent distance from the core (kpc), exact; sets the gas
    /// conditions it formed in.
    pub r: Q,
    /// Birth time (Gyr after the Big Bang), rounded to the grid.
    pub born: Q,
    /// Mass (suns), rounded to the grid.
    pub mass: Q,
    /// A binary's companion mass (suns); zero for a single star.
    pub companion: Q,
    /// The star's galactic manifold: radius, turning, phase and plane.
    pub orbit: Orbit,
    /// Population, and how many suns of it this sample star represents.
    pub pop: Pop,
    /// Stellar mass this sample star represents (suns).
    pub stands_for: Dy,
    /// The star's galactic core index, and its galaxy's length scale over the
    /// home's (`r` above is the home-equivalent radius used for gas and
    /// chemistry; `orbit.radius` is the actual radius).
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
    /// The star's reach about its own core (galaxy rings): a star relates to
    /// the core it is anchored on, in the lowest common tier (the containment
    /// chain), so its reach is its own radius in its galaxy
    /// (stars::reach_rings); a galaxy's own reach about the universal core is
    /// one tier up and is never added to it as if the two lay on one line.
    pub fn reach_rings(&self) -> Dy {
        crate::stars::reach_rings(self.orbit.radius)
    }

    /// The galaxy's turning rate (degrees per Gyr) at radius `own_kpc` in the
    /// star's galaxy: the turning law over the scaled home-equivalent mass
    /// inside, as for the star's own rate (used for constellation shear).
    pub fn rate_at_dy(&self, own_kpc: Dy) -> Dy {
        let scale = self.scale.max(Dy::sci(1, -12));
        let inside = super::inside_mass_dy(own_kpc / scale) * self.mass_ratio;
        crate::turning::turning_kpc_dy(inside, own_kpc)
    }

    /// Whether the star is within the core radius (bulge or inner disc).
    pub fn in_core(&self) -> bool {
        self.r <= core_kpc()
    }

    /// Bearing from the star's galactic core at `t` Gyr (galaxy units,
    /// excluding the core's own place): reach (orbit radius), unit direction
    /// (the plane's frame applied to the phase; exact half-angles), and climb
    /// off the plane (a disc star's vertical swing).
    pub fn bearing(&self, t: Dy) -> (Q, [Q; 3], Q) {
        let o = &self.orbit;
        let (c, s) = cos_sin_turns(&Encl::exact(o.turns_at(t).to_q()));
        let dir = o
            .frame()
            .apply(&[c, s, Encl::exact(Q::ZERO)])
            .map(|x| x.mid());
        let climb = match o.swing {
            Some((u, phase)) => {
                // z = A sin(phase + nu t / 2 pi turns), evaluated at the
                // home-equivalent radius, A scaled by the galaxy's length scale
                let age = (t - Dy::from_q(&self.born)).max(Dy::ZERO);
                let r_home = Dy::from_q(&self.r);
                let v = vertical(r_home, age, u, phase);
                let turned = Encl::exact((vertical_frequency(r_home) * t).to_q()).mul(inv_two_pi());
                let swing = Encl::exact(phase.to_q()).add(&turned);
                Encl::exact((v.amp_kpc * self.scale).to_q())
                    .mul(&cos_sin_turns(&swing).1)
                    .mid()
            }
            None => Q::ZERO,
        };
        let k = crate::chain::kpc();
        (&o.radius.to_q() * &k, dir, &climb * &k)
    }

    /// The star's place at `t` Gyr on its galaxy's sphere, with its core on
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

    /// The vertical orbit at an age (None for a bulge or nucleus star, which
    /// has no plane of its own).
    pub fn vertical_at(&self, age: Dy) -> Option<Vertical> {
        self.orbit
            .swing
            .map(|(u, phase)| vertical(self.orbit.radius, age, u, phase))
    }

    /// The share of its orbit spent inside the crowded plane layer (1 for a
    /// star without a vertical orbit).
    pub fn exposure(&self, age: Dy) -> Dy {
        self.vertical_at(age).map(|v| v.exposure).unwrap_or(Dy::ONE)
    }
}

/// A sample star's place in its galaxy before scaling: home-equivalent
/// radius (kpc), population, tilt and node half-angles and initial phase.
struct Placement {
    radius: Dy,
    pop: Pop,
    tilt: (Dy, Dy),
    node: (Dy, Dy),
    turns0: Dy,
}

/// The placement of sample star `local` (its index within its population).
/// Douady and Couder's phyllotaxis: star k's phase is k golden angles about
/// the core, and its radius and tilt are k times other irrational turns read
/// as shares, so every prefix of the sample is evenly spread over the rings
/// and over the sphere of orbit normals.
fn placement(local: u64, nucleus: bool) -> Placement {
    let pivot = |by: &Q| Dy::from_q(&(&Q::int(i64::try_from(local).unwrap_or(0)) * by).frac());
    // radius: the pivot as a share of the rings' cumulative holding (of the
    // nucleus's rings only, for a nucleus star)
    let radius = if nucleus {
        reach_at(pivot(&silver_turns()), Some(&nucleus_kpc()))
    } else {
        reach_at(pivot(&silver_turns()), None)
    };
    let pop = if nucleus {
        Pop::Nucleus
    } else if radius.to_q() <= core_kpc() {
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

/// A star's turning rate (degrees per Gyr): the turning law over the
/// home-equivalent mass inside `radius` times the hole ratio, at the scaled
/// radius `radius x scale`, so a heavier galaxy turns faster.
fn turning_rate(radius: Dy, scale: Dy, mass_ratio: Dy) -> Dy {
    let r_q = down(&radius.max(Dy::sci(1, -6)).to_q(), 60);
    let own_r = down(&(radius * scale).max(Dy::sci(1, -6)).to_q(), 60);
    Dy::from_q(&crate::turning::turning_kpc(
        &(&inside_mass(&r_q) * &mass_ratio.to_q()),
        &own_r,
    ))
}

/// The home-galaxy stellar mass one sample star stands for (suns): its
/// population's mass over the population's sample size.
fn stands_for(bang: &BigBang, nucleus: bool) -> Dy {
    if nucleus {
        Dy::from_q(&nucleus_mass()) / Dy::int(i64::try_from(NUCLEUS_SAMPLE).unwrap_or(1))
    } else {
        Dy::from_q(&stellar_mass()) / Dy::int(i64::try_from(sample(bang)).unwrap_or(1))
    }
}

/// Sample star `n` of `seed`'s universe, as drawn at the Big Bang.
pub fn star(seed: u64, bang: &BigBang, n: u64) -> Option<Star> {
    // core 0 is the home galaxy; another core's galaxy is the home's
    // homologue, its lengths scaled by `scale` and its masses by the hole
    // ratio (cores.rs)
    let core = core_of(n);
    let shape = crate::cores::shape(seed, bang, core)?;
    let (scale, mass_ratio) = (shape.scale(), Dy::from_q(&shape.mass_ratio()));
    let nucleus = (n & ((1u64 << CORE_SHIFT) - 1)) >= NUCLEUS_BASE;
    let mut rng = Stream::derive(
        seed,
        &[if nucleus { "nucleus" } else { "star" }.into(), n.into()],
    );
    let mut u = || Dy::unit(rng.next_u64());
    // five draws (population, radius, tilt, node, phase) are no longer read:
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
    // birth: the ring's first light, then a draw along the ring's
    // star-formation history (chem::Chemistry::birth_along), so a ring forms
    // stars throughout its history
    let u_born = u();
    let u_mass = u();
    let u_bin = u();
    let u_comp = u();
    let first = birth_gyr(&r)?;
    let born = crate::chem::Chemistry::cached(bang)
        .map(|c| c.birth_along(&r, &first, u_born))
        .unwrap_or(first);
    let born = down(&born, GRID_BITS);
    let mass = down(&imf(&u_mass.to_q(), &r)?.mid(), GRID_BITS);
    // u_bin < 1/3: a binary, its companion (0.1 + 0.9 u_comp) of the mass
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

/// The repulsion's pivots, each an irrational turn held exactly as a
/// convergent of its defining recurrence: the golden angle (1 / phi^2 of a
/// turn, Fibonacci F39 / F41), the silver (sqrt 2 - 1, Pell), the plastic
/// (1 / rho, Padovan P(n - 1) / P(n)) and sqrt 3 - 1 (convergents of
/// [1; 1, 2, 1, 2, ...]). They are independent over the rationals, so
/// together they never align.
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
/// rounded to the grid.
pub(crate) fn node_of(u: Dy) -> (Dy, Dy) {
    let (s, c) = (Dy::PI * u).sin_cos();
    let fix = |x: Dy| Dy::from_q(&down(&x.to_q(), GRID_BITS));
    (fix(c), fix(s))
}

/// The initial mass function, inverse-transformed at the uniform `u`:
/// a star's mass (suns) in [0.08, 60]. Its break mb moves with the cloud's
/// warmth, 2 d / (1 + d) of the local surface density d. Provisional
/// catalogue: the bounds and break scale are measured, not derived from
/// the turning law. Computed in dyadics; the caller rounds to the grid.
pub fn imf(u: &Q, r: &Q) -> Option<Encl> {
    // the IMF as a deficit share (no power laws): the share of stars above
    // the least mass is F(m) = (m - lo) / (m - lo + mb), mb = warmth / 2, so
    // the draw inverts in first degree: m = lo + mb x / (1 - x), x = u F(hi)
    let (lo, hi) = (Dy::ratio(8, 100), Dy::int(60));
    let d = Dy::from_q(&density(r).mid()).max(Dy::ZERO);
    let warmth = Dy::int(2) * d / (Dy::ONE + d);
    let mb = (Dy::ratio(1, 2) * warmth)
        .max(lo * Dy::ratio(101, 100))
        .min(hi * Dy::ratio(99, 100));
    let f_hi = (hi - lo) / (hi - lo + mb);
    let x = Dy::from_q(u) * f_hi;
    let m = lo + mb * x / (Dy::ONE - x).max(Dy::sci(1, -12));
    Some(Encl::exact(m.clamp(lo, hi).to_q()))
}
