//! The field law: each hole's pulse as an exact versor, its arrivals at a
//! star, and the star's exact energy books when it absorbs one
//! (docs/model.md).
//!
//! Reads `pulse` (`Node`, `wavelength_kpc`, `c_kpc_per_ring`), `centre`
//! (`horizon_share`, `efficiency`), `chain` (places, arcs, units),
//! `stars::ends` and `stars::Sequence`, `circuit::anchor` (the core's spin), `package::Si` and
//! `xyl_core::cga` in `Cl(4, 1)` over `Q`. Distances are kpc between bells
//! and places, AU inside a system; times and periods rings (one ring one
//! bell); masses suns and world masses. Energy is in world mass x AU^2 x
//! (2 pi / yr)^2 (5.29829e33 J), where an orbit's turning is M / a;
//! `star_year` is a sun's year of light in that unit and `star_ring` a sun's
//! ring of light, read through the SI seam (`package::Si::seconds_per_ring`).
//!
//! - Versors: `bend` (translator), `turn_about` (a turn in degrees on the
//!   plane dual to an exact unit axis, its half-angle an exact circle
//!   point); `carry` moves a point by any versor, odd ones by the twisted
//!   adjoint.
//! - `Bell::of`, `bells`: each node's screw. Its pitch is the core's
//!   wavelength times the extent of M / M_core (`pitch_kpc`), its period
//!   the pitch at light speed, its axis the core's or the progenitor
//!   orbit's normal, its spin the core's or a BLAKE3 draw in (0, 1), its
//!   light `mass x 32000 x horizon_share x efficiency`.
//! - `Bell::arrivals`: rings reaching a place in a span, in closed form
//!   (ring k sounds at born + k periods, arrives one light time later).
//!   `Bell::received`: the turning law, `360 h` degrees about the bell's
//!   axis with `h = sigma / (d + sigma)` the face of the hole's strain
//!   `sigma = spin r_g` against the reach, and the pitch's share `pitch h`
//!   along it (E011). `Bell::energy_at`: light x period x the system's
//!   face `a / (d + a)` (E011), no floor.
//! - `absorb` -> `Option<Absorbed>`: a ring is one breath of the manifold,
//!   its turn swinging each world about the star and back at the
//!   amplitude `Bell::received`; the star pays up to the energy in to hold
//!   them, `m M / (a + M rho) x s` each (`s` the turn's share), leaving the
//!   unpaid share as shear; of the rest, `transduced_share` (the current
//!   `transduction` step's release in the star's sequence,
//!   `stars::Step::release`) transduces out and the remainder leaves as
//!   light. A breath returns, so it leaves no drift.
//!
//! Invariant (`Absorbed::balances`): in = gravity + light + transduced out
//! exactly, every term held as a 62-bit dyadic and light the exact
//! remainder. Derived lengths and rates are fixed on a 2^-60 grid before
//! entering the algebra.

use crate::centre::{efficiency, horizon_share};
use crate::chain::Place;
use crate::elements::decimal;
use crate::package::hole_mass;
use crate::pulse::{c_kpc_per_ring, wavelength_kpc, Kind, Node};
use xyl_core::certified::{toward_zero, Encl};
use xyl_core::cga::{self, Mv, E1, E2, E3};
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// Bits of the grid every derived length and rate is rounded to before it
/// enters the algebra; everything after is exact.
const GRID: u32 = 60;

fn grid(d: Dy) -> Q {
    toward_zero(&d.to_q(), GRID)
}

/// An exact quotient; zero over zero is zero (every divisor here is a
/// positive measure, checked where it could vanish).
fn qd(a: &Q, b: &Q) -> Q {
    a.checked_div(b).unwrap_or(Q::ZERO)
}

/// A value held to 62 significant bits, as a dyadic rational: a base or
/// share rounded before a certified power or exponential is taken of it, or
/// before it is stored (the fields here, the systems' reaches and masses).
pub(crate) fn dyadic(q: &Q) -> Q {
    Dy::from_q(q).to_q()
}

/// AU per kpc, from the one SI seam (`package::Si`).
pub fn au_per_kpc() -> Q {
    crate::package::Si::kpc()
        .checked_div(&crate::package::Si::au())
        .unwrap_or(Q::ZERO)
}

/// A sun's light for a year in field energy units: 3.828e26 W x 3.15576e7 s
/// over the unit, world mass x AU^2 x (2 pi / yr)^2 = 5.29829e33 J. Every
/// energy in the field law (`Bell::energy_at`, `absorb`, `Absorbed`, the
/// kernel's `Books`) is in this unit.
pub fn star_year() -> Q {
    // Stand-in S018: The energy unit is the Sun's light over a Julian year.
    decimal("2.28002").unwrap_or(Q::ZERO)
}

/// A sun's light for one ring in field energy units: [`star_year`] over the
/// 3.15576e7 SI seconds it was read over, times the SI seconds of a ring
/// (`package::Si::seconds_per_ring`), held to 62 bits. Zero before a
/// founding.
pub fn star_ring() -> Q {
    let per_s = star_year()
        .checked_div(&decimal("3.15576e7").unwrap_or(Q::ONE))
        .unwrap_or(Q::ZERO);
    // Stand-in S019: A ring's energy reads the SI seconds of a ring inside the law.
    dyadic(&(&per_s * &crate::package::Si::seconds_per_ring()))
}

/// A hole's light per sun of its mass (suns): the share of an Eddington-fed
/// flow (32 000 suns per sun) that crosses the horizon, radiated at the
/// centre's efficiency (`centre`).
pub fn hole_light_per_star_mass() -> Q {
    // Stand-in S016: Hole light is mass x 32 000 x horizon share x efficiency.
    &(&Q::int(32_000) * &horizon_share()) * &efficiency()
}

// ------------------------------------------------------------------- Versors.

/// The bivector of the plane of a turn about `axis` (the dual of `axis`):
/// `x e2e3 + y e3e1 + z e1e2`.
fn plane_of(axis: &[Q; 3]) -> Mv {
    Mv::blade(E2 | E3, axis[0].clone())
        .add(&Mv::blade(E1 | E3, -&axis[1]))
        .add(&Mv::blade(E1 | E2, axis[2].clone()))
}

/// A turn of `degrees` about the exact unit `axis`, on the plane dual to
/// it (named by its generators, `plane_of`): the rotor
/// `cos(theta / 2) - sin(theta / 2) B`, its half-angle the exact circle
/// point of `theta / 2` on the 2^-64 grid (`rotor::circle_point`), so the
/// rotor is exact and of unit scale. `None` for an axis not exactly of unit
/// length.
pub fn turn_about(axis: &[Q; 3], degrees: &Q) -> Option<Mv> {
    let length = axis.iter().fold(Q::ZERO, |s, x| &s + &(x * x));
    if length != Q::ONE {
        return None;
    }
    let (c, s) = xyl_core::rotor::circle_point(&degrees.over(xyl_core::nonzero!(2)), 64)?;
    Some(Mv::scalar(c).sub(&plane_of(axis).scale(&s)))
}

/// A point moved by a versor, exactly (an odd versor acts by its twisted
/// adjoint, as a reflection does). `None` where the moved point cannot be
/// read back.
pub fn carry(v: &Mv, x: &[Q; 3]) -> Option<[Q; 3]> {
    let p = cga::point_at(x);
    let moved = if v.parity() == Some(true) {
        v.twisted(&p)
    } else {
        v.sandwich(&p)
    };
    let b = cga::base_at(&moved, 3)?;
    Some([b.first()?.clone(), b.get(1)?.clone(), b.get(2)?.clone()])
}

// ----------------------------------------------------------------- The bells.

/// A bell's screw and its light.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bell {
    /// The hole's spin axis: an exact unit direction (the orbit's rotor
    /// applied to the pole, exactly).
    pub axis: [Q; 3],
    /// The axis tilt from the orbit normal (degrees), at the balance of the
    /// progenitor's charge (`stellar_tilt`); `None` for a galactic core and
    /// where the progenitor's held mixture is not read, whose axis is then
    /// the orbit normal.
    pub tilt: Option<Q>,
    /// The hole's Kerr spin parameter J / M^2, in (0, 1).
    pub spin: Q,
    /// Pitch (kpc): the core's wavelength scaled by the hole's mass
    /// (`pitch_kpc`).
    pub pitch_kpc: Q,
    /// Period: the pitch at light speed (rings of time), one pulse ring per
    /// period.
    pub period: Q,
    /// Mass (suns).
    pub mass: Q,
    /// Light output (suns).
    pub light: Q,
    /// Formation time (rings after the origin).
    pub born: Dy,
}

/// The pitch of a hole of `mass` suns: the core's wavelength scaled by the
/// extent of M / M_core under the body measure (`elements::extent`, used in
/// place of (M / M_core)^(1/3)).
pub fn pitch_kpc(mass: Dy) -> Dy {
    let ratio = (mass.div(&(Dy::from_q(&hole_mass()))).unwrap_or_default()).max(Dy::sci(1, -30));
    // Stand-in S022: The pitch scales by the extent of M / M_core for a cube root.
    wavelength_kpc() * Dy::from_q(&crate::elements::extent(&ratio.to_q()))
}

/// A stellar hole's Kerr spin in (0, 1), drawn deterministically from the
/// seed and star index.
fn drawn_spin(seed: u64, n: u64) -> Q {
    // Stand-in S020: Each stellar hole's spin is a fresh draw.
    let x =
        xyl_core::Stream::derive(seed, &["stellar hole spin".into(), n.into()]).next_u64() >> 12;
    // (x + 1) / (2^52 + 2) with x < 2^52: strictly inside (0, 1).
    Q::new(i64::try_from(x).unwrap_or(0) + 1, (1i64 << 52) + 2).unwrap_or(xyl_core::ratio!(1, 2))
}

/// The tilt of a stellar hole's spin axis at the natural balance of its
/// charge (#183): `eps* = 90 pi` degrees with `pi = (p / w) (1 - delta)`,
/// `p / w` the proton share of its progenitor's held mixture
/// (`disc::StarHold::held`, `circuit::mix_weights`) and `delta` the
/// progenitor's held face at its rim (`Star::compactness`, the bound share
/// `g` at a star). `None` where the progenitor's disc manifold or its
/// mixture is not read.
pub fn stellar_tilt(s: &crate::galaxy::Star) -> Option<Q> {
    let held: crate::circuit::Mix = crate::galaxy::disc_manifold(&s.r, &s.mass)?
        .cut
        .held
        .into_iter()
        .collect();
    let w = crate::circuit::mix_weights(&held);
    let share = w.protons.checked_div(&(&w.protons + &w.neutrons))?;
    // Stand-in S003: The star's compactness stands for the remnant's held face.
    let delta = s.compactness().to_q();
    xyl_core::hopf::tilt_equilibrium(&xyl_core::hopf::open_charge(&share, &delta)?)
}

/// A stellar hole's spin axis, exact before the grid: the orbit's rotor
/// applied to the pole, tilted first by `tilt` degrees on the body's plane
/// carrying the pole (axis 3) toward the line of nodes (axis 1), so the axis
/// stands `tilt` degrees from the orbit normal. Upright for `None`.
pub fn stellar_axis(o: &crate::galaxy::Orbit, tilt: Option<&Q>) -> [Q; 3] {
    use xyl_core::rotor::{Rotor, Turn};
    let orbit = o.rotor();
    let rotor = tilt
        .and_then(|degrees| {
            let turn = Turn {
                from: 3,
                toward: 1,
                degrees: degrees.clone(),
            };
            Rotor::<3>::of(&turn, 40)
        })
        // Stand-in S004: An unread tilt stands upright.
        .map_or(orbit.clone(), |t| orbit.then_after(&t));
    let z = xyl_core::certified::Encl::exact(Q::ZERO);
    rotor
        .apply(&[z.clone(), z, xyl_core::certified::Encl::exact(Q::ONE)])
        .map(|x| x.mid())
}

impl Bell {
    /// A node's bell: a galactic core spins about the z axis (the disc's axis)
    /// at the universal core's spin (`circuit::anchor`); a stellar hole spins
    /// at a drawn spin about its progenitor star's orbit normal tilted by the
    /// balance of its charge (`stellar_tilt`, `stellar_axis`).
    pub fn of(node: &Node, seed: u64) -> Bell {
        let (axis, spin, tilt) = match &node.kind {
            // Stand-in S021: Every galactic core spins about z at the parent's drawn spin.
            Kind::Core(_) => (
                [Q::ZERO, Q::ZERO, Q::ONE],
                crate::circuit::anchor().spin,
                None,
            ),
            Kind::Stellar(s) => {
                let tilt = stellar_tilt(s);
                let n = stellar_axis(&s.orbit, tilt.as_ref());
                (n, drawn_spin(seed, s.n), tilt)
            }
        };
        let pitch = pitch_kpc(node.mass);
        let pitch_kpc = grid(pitch);
        let period = grid(pitch.div(&c_kpc_per_ring()).unwrap_or_default());
        let mass = grid(node.mass);
        let light = &mass * &hole_light_per_star_mass();
        Bell {
            axis,
            tilt,
            spin,
            pitch_kpc,
            period,
            mass,
            light,
            born: node.born,
        }
    }

    /// The number of pulse rings reaching a place `delay` rings of light
    /// away within `(from, to]` (rings after the origin): pulse ring k sounds
    /// at born + k periods and arrives one light time later. Closed form; zero
    /// for an empty span or a non-positive period.
    pub fn arrivals(&self, delay: Dy, from: Dy, to: Dy) -> u64 {
        let period = Dy::from_q(&self.period);
        if period.signum() <= 0 || to <= from {
            return 0;
        }
        let ring = |t: Dy| -> i64 {
            let since = t - self.born - delay;
            if since.signum() < 0 {
                -1
            } else {
                let rings = since.div(&period).unwrap_or_default();
                rings.trunc_i64().unwrap_or(if rings.signum() > 0 {
                    i64::MAX
                } else {
                    i64::MIN
                })
            }
        };
        u64::try_from(ring(to) - ring(from)).unwrap_or(0)
    }

    /// The pulse's breath at a place `d_kpc` away, first-degree (E011): one
    /// ring is one breath of the manifold, expanding and contracting, so its
    /// turn about the bell's axis swings out and back and its screw's
    /// translation returns; the ring leaves no net turn and no net drift. Its
    /// amplitude is the turning law's `360 h` degrees on the plane dual to
    /// the axis, `h = sigma / (d + sigma)` the face of the hole's strain
    /// `sigma = spin x r_g` against the reach, finite at `d = 0`; this is the
    /// turn the star holds its worlds against. `None` for an axis not exactly
    /// of unit length.
    pub fn received(&self, d_kpc: Dy) -> Option<Mv> {
        // The hole's r_g: the core's r_g times the hole's mass over the
        // core's.
        let rg_kpc = ((Dy::from_q(&crate::chain::units().r_g_m))
            .div(&(Dy::from_q(&crate::package::Si::kpc())))
            .unwrap_or_default()
            * Dy::from_q(&self.mass))
        .div(&(Dy::from_q(&hole_mass())))
        .unwrap_or_default();
        let sigma = Dy::from_q(&self.spin) * rg_kpc;
        let h = grid(
            sigma
                .div(&(d_kpc.max(Dy::ZERO) + sigma))
                .unwrap_or_default(),
        );
        turn_about(&self.axis, &(&Q::int(360) * &h))
    }

    /// The energy of one ring intercepted by a system of radius `a_out_au` at
    /// `d_kpc`: one period of light times the system's share of the manifold
    /// the pulse spreads on at that reach (`formation::gathered`), the cap of
    /// S^n within the system's deficit angle `a / (d + a)` of the half turn,
    /// both in AU, `n` the manifold's dimension at the reach
    /// (`formation::dimension_at`: the disc's circle beyond one galaxy unit,
    /// the share the face itself; the bulge's ball within it, the share
    /// falling as the inverse square far out). Finite at `d = 0`, no floor; in
    /// field energy units.
    pub fn energy_at(&self, d_kpc: Dy, a_out_au: &Q) -> Q {
        let d_au = grid(d_kpc * Dy::from_q(&au_per_kpc())).max(Q::ZERO);
        let n = crate::formation::dimension_at(&crate::galaxy::units_of_kpc(&d_kpc.to_q()));
        let share = crate::formation::gathered(a_out_au, &d_au, n);
        &(&(&self.light * &self.period) * &star_ring()) * &share
    }
}

/// Every node's bell, in node order.
pub fn bells(nodes: &[Node], seed: u64) -> Vec<Bell> {
    nodes.iter().map(|n| Bell::of(n, seed)).collect()
}

/// The light delay (rings) and proper distance (kpc) from a bell to a place
/// at `t` rings.
pub fn path(node: &Node, place: &Place, t: Dy) -> (Dy, Dy) {
    let r = crate::chain::radius();
    let d = place.arc_kpc(&node.place(t, &r), &r);
    let c = c_kpc_per_ring();
    if c.signum() <= 0 {
        return (Dy::ZERO, d);
    }
    (d.div(&c).unwrap_or_default(), d)
}

// ------------------------------------------------------------ The star transduces.

/// The step a star is on, by its age against the ends of its sequence's
/// steps (`stars::ends`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transduction {
    /// No step of the held mixture releases hold: the held body that does
    /// not ignite transduces nothing.
    Unlit,
    /// Step `i` of the star's sequence (`stars::Sequence::steps`): the first
    /// is the main sequence, later fusion steps the giant phases, fission
    /// steps follow the throat.
    Step(usize),
    /// Every step has ended: a remnant transduces nothing.
    Remnant,
}

/// The step a star of sequence `seq` and compactness `delta`
/// (`galaxy::Star::compactness`) at reach `r` (galaxy units) is on at `age`
/// rings: the first step whose end (`stars::ends`, from the pulse's transduction of
/// its hold) lies after the age, `Remnant` once every step has ended, and
/// `Unlit` where the sequence has no step.
pub fn transduction(seq: &crate::stars::Sequence, delta: Dy, reach: Dy, age: Dy) -> Transduction {
    if !seq.ignites() {
        return Transduction::Unlit;
    }
    let age = age.to_q();
    crate::stars::ends(seq, delta, reach)
        .iter()
        .position(|end| age < *end)
        .map_or(Transduction::Remnant, Transduction::Step)
}

/// The share of transduced mass converted to energy on step `b` of `seq`: the
/// step's release, the hold transduced per hold it transduces
/// (`stars::Step::release`). Zero for a body that does not ignite and for a
/// remnant.
pub fn transduced_share(seq: &crate::stars::Sequence, b: Transduction) -> Q {
    match b {
        Transduction::Step(i) => seq
            .steps
            .get(i)
            .map(|s| s.release.clone())
            .unwrap_or(Q::ZERO),
        Transduction::Unlit | Transduction::Remnant => Q::ZERO,
    }
}

// ---------------------------------------------------------- The star absorbs.

/// A world as held by its star: its reach (its orbit's radius from the
/// star's G point, AU), its native direction (where its orbit puts it now,
/// an exact unit direction, moved with the orbit by the caller), its shear
/// (the turn pulses left that the star did not re-lock, a rotor carried with
/// the orbit), and its mass (world masses). A pulse turns a world about the
/// star and keeps its reach, so the shear is a turn on a named plane, never
/// a Cartesian displacement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchor {
    /// The orbit's radius from the star's G point (AU).
    pub reach: Q,
    /// Native direction on the orbit now, from the star's G point (exact unit).
    pub native: [Q; 3],
    /// The shear: the turn from the native direction not yet re-locked.
    pub shear: xyl_core::rotor::Rotor<3>,
    /// World mass (world masses).
    pub mass: Q,
}

impl Anchor {
    /// A world on its native orbit of radius `a` AU, at its orbit's exact
    /// unit direction `dir`, unsheared.
    pub fn on_orbit(a: &Q, mass: &Q, dir: [Q; 3]) -> Anchor {
        Anchor {
            reach: a.clone(),
            native: dir,
            shear: xyl_core::rotor::Rotor::identity(),
            mass: mass.clone(),
        }
    }

    /// The same world where its orbit has carried it, `dir`, its shear kept.
    pub fn along(&self, dir: [Q; 3]) -> Anchor {
        Anchor {
            native: dir,
            ..self.clone()
        }
    }

    /// Its current direction: the native direction turned by the shear,
    /// exact.
    pub fn at(&self) -> [Q; 3] {
        self.shear
            .apply(&self.native.clone().map(Encl::exact))
            .map(|x| x.mid())
    }

    /// The shear: the turn from the native direction to the current one, in
    /// degrees, read from the pivot's rate on each plane (`turn_share`).
    pub fn sheared(&self) -> Q {
        &Q::int(360) * &turn_share(&self.native, &self.at())
    }
}

/// The share of a whole turn carrying the unit direction `from` to `to`,
/// read plane by plane from the pivot's rate (11b): a plane of rate `g`
/// turns `2 atan |g|`, so its share is `atan |g| / pi`, certified and well
/// conditioned for small turns, and the planes' shares add. 0 for one
/// direction, 1/2 for antipodal ones. Its midpoint.
fn turn_share(from: &[Q; 3], to: &[Q; 3]) -> Q {
    use xyl_core::certified::{atan, pi};
    let Some(pivot) = xyl_core::rotor::pivot(from, to, 64) else {
        return Q::ZERO;
    };
    let Some(rates) = pivot.rotor.as_ref().and_then(|r| r.rates()) else {
        return xyl_core::ratio!(1, 2);
    };
    let per = pi(64).recip().map_or(Q::ZERO, |p| p.mid());
    rates.iter().fold(Q::ZERO, |s, (_, g)| {
        &s + &(&atan(&g.abs(), 64).mid() * &per)
    })
}

/// `r_g` of one sun in AU, `G M_sun / (c^2 AU)`: the length a hold of one
/// sun reads in a system's calibre (`package::Si`).
fn sun_rg_au() -> Q {
    use crate::package::Si;
    qd(
        &(&Si::g() * &Si::star_mass_kg()),
        &(&(&Si::c() * &Si::c()) * &Si::au()),
    )
}

/// The result of one absorption: energy in, spent as gravity (re-locking
/// orbits), light and transduction-out; and each world's place afterwards
/// (with any shear not re-locked). A breath returns, so it leaves no drift.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Absorbed {
    /// Energy the pulse brings, in field energy units (one unit = world mass
    /// x AU^2 x (2 pi / yr)^2 = 5.29829e33 J; a Sun-year of light is 2.28002
    /// units), held as a 62-bit dyadic.
    pub energy_in: Q,
    /// Energy spent re-locking orbits (field energy units), at most `energy_in`.
    pub gravity: Q,
    /// Energy left as light (field energy units): the remainder after gravity
    /// and transduction-out.
    pub light: Q,
    /// Energy converted by the current transduction step's mass defect (field
    /// energy units).
    pub transduced_out: Q,
    /// Each world's anchor after the absorption, in the input order.
    pub anchors: Vec<Anchor>,
}

impl Absorbed {
    /// The field law's books: energy in = gravity + light + transduced out,
    /// exactly.
    pub fn balances(&self) -> bool {
        &(&self.gravity + &self.light) + &self.transduced_out == self.energy_in
    }
}

/// A star of `mass` suns absorbs a pulse `v` bringing `energy_in`, its
/// current step converting `transduced_out` of what it transduces
/// (`transduced_share`). The pulse is one breath: its turn swings each world
/// about the star and back, keeping its reach, and the star holds the world
/// against the breath's amplitude `v`. Re-locking a world turned from its native
/// direction by the share `s` of a whole turn (`turn_share`) costs the
/// hold's work over that share, first-degree (E011): `m M / (a + M rho) x s`, the world's
/// mass times the star's face at the orbit's reach with the hold read as a
/// length (`rho` one sun's `r_g` in AU), in field energy units. The star
/// pays what the energy covers; the unpaid share of each turn remains as
/// shear, its rate scaled by that share (`Rotor::rate_scaled`). `None`
/// where a moved point cannot be read back.
pub fn absorb(
    mass: &Q,
    transduced_out: &Q,
    worlds: &[Anchor],
    v: &Mv,
    energy_in: &Q,
) -> Option<Absorbed> {
    let hold = &(mass * &sun_rg_au());
    let mut moved = Vec::with_capacity(worlds.len());
    let mut need = Q::ZERO;
    for w in worlds {
        // The breath's turn about the star at its amplitude: the world's
        // point moved, read back as a direction at the same reach.
        let at = w.at();
        let point: [Q; 3] = std::array::from_fn(|i| &at[i] * &w.reach);
        let c = carry(v, &point)?;
        let turned: [Q; 3] = std::array::from_fn(|i| qd(&c[i], &w.reach));
        let share = dyadic(&turn_share(&w.native, &turned));
        let work = qd(&(&(&w.mass * mass) * &share), &(&w.reach + hold));
        need = &need + &work;
        moved.push(turned);
    }
    // Every term is held as a 62-bit dyadic (Dy), so books summed over a
    // universe's life keep power-of-two denominators and stay bounded; the
    // rounding is monotone (gravity never exceeds the input) and light takes
    // the exact remainder, so the books still balance.
    let energy_in = &dyadic(energy_in);
    let gravity = dyadic(if &need < energy_in { &need } else { energy_in });
    // Full payment is decided on the exact need, not the rounded gravity,
    // so a fully paid re-lock leaves no shear.
    let held = if need.signum() <= 0 || &need <= energy_in {
        Q::ONE
    } else {
        qd(&gravity, &need)
    };
    let left = toward_zero(&(&Q::ONE - &held), GRID);
    let anchors = worlds
        .iter()
        .zip(moved)
        .map(|(w, turned)| {
            // The unpaid share of the turn from the native direction stays:
            // the pivot's rate scaled by it. A fully re-locked world stands
            // exactly on its native direction.
            let shear = if left.signum() == 0 {
                xyl_core::rotor::Rotor::identity()
            } else {
                xyl_core::rotor::pivot(&w.native, &turned, 64)
                    .and_then(|p| p.rotor)
                    .and_then(|r| r.rate_scaled(&left))
                    .unwrap_or_else(xyl_core::rotor::Rotor::identity)
            };
            Anchor {
                reach: w.reach.clone(),
                native: w.native.clone(),
                shear,
                mass: w.mass.clone(),
            }
        })
        .collect();
    let rest = energy_in - &gravity;
    let transduced_out = dyadic(&(&rest * transduced_out)).min(rest.clone());
    let light = &rest - &transduced_out;
    Some(Absorbed {
        energy_in: energy_in.clone(),
        gravity,
        light,
        transduced_out,
        anchors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::galaxy::{star, NUCLEUS_BASE};
    use crate::origin::Origin;

    /// The core's node, built directly rather than by sampling the galaxy.
    fn core() -> Node {
        Node {
            kind: Kind::Core(Box::new(crate::cores::Shape {
                index: 0,
                hole_suns: hole_mass(),
                place: [Q::ZERO, Q::ZERO, Q::ZERO],
                reach: Q::ZERO,
                phase: Q::ZERO,
                born: Q::ZERO,
                tilt: xyl_core::rotor::Direction::ZERO_ANGLE,
                node: xyl_core::rotor::Direction::ZERO_ANGLE,
                rate: Dy::ZERO,
            })),
            mass: Dy::from_q(&hole_mass()),
            born: Dy::ZERO,
        }
    }
    fn stellar(origin: &Origin) -> Node {
        // A 50-sun stellar hole on the first sample star that exists (stars
        // exist only where their ring's gas lights, `galaxy::birth`).
        let s = (0..crate::galaxy::sample(origin))
            .chain(std::iter::once(NUCLEUS_BASE))
            .find_map(|n| star(7, origin, n))
            .expect("a standing star");
        Node {
            kind: Kind::Stellar(Box::new(s)),
            mass: Dy::int(50),
            born: Dy::int(1),
        }
    }

    fn origin() -> [Q; 3] {
        [Q::ZERO, Q::ZERO, Q::ZERO]
    }

    #[test]
    fn versors_bend_and_rotate_exactly() {
        // Bend: a translation by (1/3, -2, 5/7).
        let d = [
            xyl_core::ratio!(1, 3),
            xyl_core::ratio!(-2, 1),
            xyl_core::ratio!(5, 7),
        ];
        assert_eq!(carry(&cga::translator_at(&d), &origin()).unwrap(), d);
        // Turn: a half turn about z takes x to -x, a quarter turn takes x to
        // y, and a whole turn returns it, each to the half-angle's 2^-64
        // grid and within the plane dual to z; an axis not of unit length is
        // refused.
        let zaxis = [Q::ZERO, Q::ZERO, Q::ONE];
        let x = [Q::int(5), Q::ZERO, Q::ZERO];
        let tiny = xyl_core::ratio!(1, 1_000_000_000_000);
        let near =
            |p: &[Q; 3], e: [i64; 3]| p.iter().zip(e).all(|(a, b)| (a - &Q::int(b)).abs() < tiny);
        let at = |deg: i64| carry(&turn_about(&zaxis, &Q::int(deg)).unwrap(), &x).unwrap();
        assert!(near(&at(180), [-5, 0, 0]));
        let quarter = at(90);
        assert!(near(&quarter, [0, 5, 0]) || near(&quarter, [0, -5, 0]));
        assert!(near(&at(360), [5, 0, 0]));
        assert!([at(90), at(180), at(360)].iter().all(|p| p[2] == Q::ZERO));
        assert!(turn_about(&[Q::ZERO, Q::ZERO, Q::int(2)], &Q::int(90)).is_none());
    }

    #[test]
    fn bell_pitch_period_and_arrivals_in_closed_form() {
        let origin = crate::genesis::origin();
        let bs = bells(&[core(), stellar(origin)], 7);
        // The core bell's pitch is the wavefront's wavelength, its axis z and
        // its spin the anchor's.
        assert!((Dy::from_q(&bs[0].pitch_kpc) - wavelength_kpc()).abs() < Dy::sci(1, -9));
        assert_eq!(bs[0].axis, [Q::ZERO, Q::ZERO, Q::ONE]);
        assert_eq!(bs[0].spin, crate::circuit::anchor().spin);
        // A stellar hole has a shorter pitch and period, and a spin in (0, 1).
        let s = bs.get(1).expect("a stellar bell");
        assert!(s.pitch_kpc < bs[0].pitch_kpc && s.period < bs[0].period);
        assert!(s.spin.signum() > 0 && s.spin < Q::ONE);
        // Its axis stands at the balance of its progenitor's charge from the
        // orbit normal: the cosine between them is the tilt's point of the
        // circle, exactly, and the tilt lies in a quarter turn.
        let Kind::Stellar(star) = &stellar(origin).kind else {
            panic!("a stellar node")
        };
        let eps = s.tilt.clone().expect("the progenitor's charge is read");
        assert!(eps.signum() >= 0 && eps <= Q::int(90));
        let (normal, tilted) = (
            stellar_axis(&star.orbit, None),
            stellar_axis(&star.orbit, Some(&eps)),
        );
        let dot = normal
            .iter()
            .zip(&tilted)
            .fold(Q::ZERO, |a, (x, y)| &a + &(x * y));
        assert_eq!(
            Some(dot),
            xyl_core::rotor::circle_point(&eps, 40).map(|(c, _)| c)
        );
        assert!(bs[0].tilt.is_none());
        // Arrivals are counted in closed form: a period's span holds one ring.
        let b = &bs[0];
        let period = Dy::from_q(&b.period);
        let delay = xyl_core::dyadic!(1, 4);
        let t0 = Dy::int(10) * period + delay + period.div(&(Dy::int(2))).unwrap_or_default();
        let one = b.arrivals(delay, t0, t0 + period);
        assert_eq!(one, 1, "one period, one ring");
        assert_eq!(b.arrivals(delay, t0, t0 + period * Dy::int(10)), 10);
        assert_eq!(b.arrivals(delay, t0, t0), 0);
        // Nothing arrives before a bell has formed and its light has crossed.
        let late = Bell {
            born: Dy::int(12),
            ..b.clone()
        };
        assert_eq!(late.arrivals(delay, Dy::int(5), Dy::int(11)), 0);
    }

    #[test]
    fn absorption_books_balance_exactly() {
        // The bell's period and a ring of light are counted in rings, which
        // the founding sets.
        let _ = crate::genesis::origin();
        let core = Bell::of(&core(), 7);
        let d = Dy::int(8);
        let v = core.received(d).unwrap();
        let worlds = vec![
            Anchor::on_orbit(&Q::ONE, &Q::ONE, [Q::ONE, Q::ZERO, Q::ZERO]),
            Anchor::on_orbit(
                &xyl_core::ratio!(52, 10),
                &Q::int(318),
                [Q::ZERO, Q::ONE, Q::ZERO],
            ),
            Anchor::on_orbit(
                &xyl_core::ratio!(3, 10),
                &xyl_core::ratio!(1, 20),
                [xyl_core::ratio!(3, 5), xyl_core::ratio!(4, 5), Q::ZERO],
            ),
        ];
        let e = core.energy_at(d, &xyl_core::ratio!(52, 10));
        assert!(e.signum() > 0);
        let a = absorb(&Q::ONE, &h_release(), &worlds, &v, &e).unwrap();
        assert!(
            a.balances(),
            "in = gravity + light + transduced out, exactly: {a:?}"
        );
        // Every term is dyadic (power-of-two denominator), so books summed
        // over a life stay bounded.
        for x in [&a.energy_in, &a.gravity, &a.transduced_out] {
            assert_eq!(dyadic(x), *x, "{x} is not held to 62 bits");
        }
        assert!(a.gravity.signum() > 0, "the star re-locks its worlds");
        assert!(a.transduced_out.signum() > 0 && a.light > a.transduced_out);
        // A pulse too weak to pay for the re-locking leaves the orbits sheared.
        let weak = qd(&a.gravity, &Q::int(2));
        let b = absorb(&Q::ONE, &h_release(), &worlds, &v, &weak).unwrap();
        assert!(b.balances());
        assert_eq!(b.light, Q::ZERO);
        assert!(
            b.anchors.iter().any(|w| w.sheared().signum() > 0),
            "what it could not re-lock stays sheared"
        );
        // A fully paid re-lock leaves no shear.
        assert!(
            a.anchors.iter().all(|w| w.sheared() == Q::ZERO),
            "wholly re-locked"
        );
        // The next pulse acts on the sheared worlds where they are: shear
        // accumulates while the energy does not pay for the re-lock.
        let c = absorb(&Q::ONE, &h_release(), &b.anchors, &v, &weak).unwrap();
        assert!(c.balances());
        let (s1, s2): (Q, Q) = (
            b.anchors
                .iter()
                .map(Anchor::sheared)
                .fold(Q::ZERO, |x, y| &x + &y),
            c.anchors
                .iter()
                .map(Anchor::sheared)
                .fold(Q::ZERO, |x, y| &x + &y),
        );
        assert!(s2 > s1, "sheared further: {s1} -> {s2}");
    }

    /// A pure-hydrogen held mixture at heat 0.
    fn hydrogen() -> crate::stars::Sequence {
        crate::stars::Sequence::of(&[(1, Q::ONE)], &crate::atoms::Received::none())
    }

    /// The first step's release of the pure-hydrogen sequence.
    fn h_release() -> Q {
        transduced_share(&hydrogen(), Transduction::Step(0))
    }

    #[test]
    fn transduced_share_reads_each_steps_release() {
        // Each step converts its release of what it transduces; a body that does
        // not ignite and a remnant convert nothing. The steps read the
        // universal table, so the universe is founded first.
        let _ = crate::genesis::origin();
        let seq = hydrogen();
        for (i, step) in seq.steps.iter().enumerate() {
            let share = transduced_share(&seq, Transduction::Step(i));
            assert_eq!(share, step.release);
            assert!(share > Q::ZERO && share < Q::ONE, "step {i}: {share}");
        }
        assert_eq!(transduced_share(&seq, Transduction::Remnant), Q::ZERO);
        assert_eq!(transduced_share(&seq, Transduction::Unlit), Q::ZERO);
        // The step by age at r = 0 follows the ends in order, then the
        // remnant (the bell sets the life).
        let delta = crate::galaxy::first_star_compactness();
        let ends = crate::stars::ends(&seq, delta, Dy::ZERO);
        assert!(ends[0].signum() > 0);
        let mut start = Q::ZERO;
        for (i, end) in ends.iter().enumerate() {
            if *end > start {
                let mid = (Dy::from_q(&(&start + end)))
                    .div(&(Dy::int(2)))
                    .unwrap_or_default();
                assert_eq!(
                    transduction(&seq, delta, Dy::ZERO, mid),
                    Transduction::Step(i)
                );
            }
            start = end.clone();
        }
        let late = Dy::from_q(&start) * Dy::int(2);
        assert_eq!(
            transduction(&seq, delta, Dy::ZERO, late),
            Transduction::Remnant
        );
        // A mixture with no releasing step does not ignite.
        let unlit = crate::stars::Sequence::default();
        assert_eq!(
            transduction(&unlit, delta, Dy::ZERO, Dy::ONE),
            Transduction::Unlit
        );
    }
}
