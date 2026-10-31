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
//! - Versors: `bend` (translator), `rotate` (rotor `1 + t B`, a rational
//!   point of the circle), `twist` (screw: rotor then translation along the
//!   axis); `carry` moves a point by any versor, odd ones by the twisted
//!   adjoint.
//! - `Bell::of`, `bells`: each node's screw. Its pitch is the core's
//!   wavelength times the extent of M / M_core (`pitch_kpc`), its period
//!   the pitch at light speed, its axis the core's or the progenitor
//!   orbit's normal, its spin the core's or a BLAKE3 draw in (0, 1), its
//!   light `mass x 32000 x horizon_share x efficiency`.
//! - `Bell::arrivals`: rings reaching a place in a span, in closed form
//!   (ring k sounds at born + k periods, arrives one light time later).
//!   `Bell::received` and `energy_at`: the versor and the energy
//!   intercepted, light x period x a^2 / (4 d^2).
//! - `absorb` -> `Option<Absorbed>`: the pulse shears each world's orbit;
//!   the star pays up to the energy in to re-lock them, at
//!   `1/2 m (M / a)^2 |dx|^2` each, leaving the unpaid share as shear; of
//!   the rest, `transduced_share` (the current `transduction` step's release
//!   in the star's sequence, `stars::Step::release`) transduces out and the
//!   remainder leaves as light. The translation of the star's G point is the
//!   drift and shears no orbit.
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
use xyl_core::certified::down;
use xyl_core::cga::{self, Mv, E1, E2, E3};
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// Bits of the grid every derived length and rate is rounded to before it
/// enters the algebra; everything after is exact.
const GRID: u32 = 60;

fn grid(d: Dy) -> Q {
    down(&d.to_q(), GRID)
}

/// An exact quotient; zero over zero is zero (every divisor here is a
/// positive measure, checked where it could vanish).
fn qd(a: &Q, b: &Q) -> Q {
    a.checked_div(b).unwrap_or(Q::ZERO)
}

/// A value held to 62 significant bits, as a dyadic rational.
fn dyadic(q: &Q) -> Q {
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
    dyadic(&(&per_s * &crate::package::Si::seconds_per_ring()))
}

/// A hole's light per sun of its mass (suns): the share of an Eddington-fed
/// flow (32 000 suns per sun) that crosses the horizon, radiated at the
/// centre's efficiency (`centre`).
pub fn hole_light_per_star_mass() -> Q {
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

/// Bend: the translation by `d`.
pub fn bend(d: &[Q; 3]) -> Mv {
    cga::translator_at(d)
}

/// Rotate about `axis` by `2 atan(t |axis|)`: the rotor `1 + t B`, a
/// rational point of the circle (unnormalized; the scale is removed when a
/// point is read back).
pub fn rotate(axis: &[Q; 3], t: &Q) -> Mv {
    Mv::scalar(Q::ONE).add(&plane_of(axis).scale(t))
}

/// Twist: a turn about `axis` (as in `rotate`) followed by the translation
/// `along`.
pub fn twist(axis: &[Q; 3], t: &Q, along: &[Q; 3]) -> Mv {
    bend(along).geo(&rotate(axis, t))
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
    /// The hole's spin axis (a direction, exact on the grid).
    pub axis: [Q; 3],
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
    let ratio = (mass / Dy::from_q(&hole_mass())).max(Dy::sci(1, -30));
    wavelength_kpc() * Dy::from_q(&crate::elements::extent(&ratio.to_q()))
}

/// A stellar hole's Kerr spin in (0, 1), drawn deterministically from the
/// seed and star index.
fn drawn_spin(seed: u64, n: u64) -> Q {
    let x =
        xyl_core::Stream::derive(seed, &["stellar hole spin".into(), n.into()]).next_u64() >> 12;
    // (x + 1) / (2^52 + 2) with x < 2^52: strictly inside (0, 1).
    Q::new(i64::try_from(x).unwrap_or(0) + 1, (1i64 << 52) + 2).unwrap_or(Q::ratio(1, 2))
}

impl Bell {
    /// A node's bell: a galactic core spins about the z axis (the disc's axis)
    /// at the universal core's spin (`circuit::anchor`); a stellar hole spins
    /// about its progenitor star's orbit normal at a drawn spin.
    pub fn of(node: &Node, seed: u64) -> Bell {
        let (axis, spin) = match &node.kind {
            Kind::Core(_) => ([Q::ZERO, Q::ZERO, Q::ONE], crate::circuit::anchor().spin),
            Kind::Stellar(s) => {
                let o = &s.orbit;
                // The orbit plane's normal: the frame's image of the z axis.
                let m = o.frame().matrix();
                (
                    [
                        grid(Dy::from_q(&m[0][2])),
                        grid(Dy::from_q(&m[1][2])),
                        grid(Dy::from_q(&m[2][2])),
                    ],
                    drawn_spin(seed, s.n),
                )
            }
        };
        let pitch = pitch_kpc(node.mass);
        let pitch_kpc = grid(pitch);
        let period = grid(pitch / c_kpc_per_ring());
        let mass = grid(node.mass);
        let light = &mass * &hole_light_per_star_mass();
        Bell {
            axis,
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
                (since / period).trunc_i64()
            }
        };
        u64::try_from(ring(to) - ring(from)).unwrap_or(0)
    }

    /// The pulse at a place `d_kpc` away: the bell's screw scaled by the
    /// strain there: a turn about its axis by `2 atan(t |axis|)` with
    /// `t = spin x r_g / d`, followed by a translation along the axis of one
    /// pitch per full turn (in AU). `|axis|` is taken in first degree.
    pub fn received(&self, d_kpc: Dy) -> Mv {
        // The hole's r_g: the core's r_g times the hole's mass over the
        // core's.
        let rg_kpc = Dy::from_q(&crate::chain::units().r_g_m)
            / Dy::from_q(&crate::package::Si::kpc())
            * Dy::from_q(&self.mass)
            / Dy::from_q(&hole_mass());
        let t = grid(Dy::from_q(&self.spin) * rg_kpc / d_kpc.max(Dy::sci(1, -9)));
        // The axis length in first degree (sum of |components|, no root).
        let norm = self
            .axis
            .iter()
            .fold(Dy::ZERO, |s, x| s + Dy::from_q(x).abs());
        let angle = Dy::int(2) * (Dy::from_q(&t) * norm).atan();
        // The screw's translation for this turn, in AU, along the axis.
        let shift = Dy::from_q(&self.pitch_kpc) * Dy::from_q(&au_per_kpc()) * angle
            / (Dy::int(2) * Dy::PI)
            / norm.max(Dy::sci(1, -30));
        let along = [
            grid(shift * Dy::from_q(&self.axis[0])),
            grid(shift * Dy::from_q(&self.axis[1])),
            grid(shift * Dy::from_q(&self.axis[2])),
        ];
        twist(&self.axis, &t, &along)
    }

    /// The energy of one ring intercepted by a system of radius `a_out_au` at
    /// `d_kpc`: one period of light times the share of the sphere the system's
    /// disc covers (pi a^2 over 4 pi d^2: a^2 / (4 d^2), exact), with d at
    /// least 1 AU; in field energy units.
    pub fn energy_at(&self, d_kpc: Dy, a_out_au: &Q) -> Q {
        let d_au = grid(d_kpc * Dy::from_q(&au_per_kpc())).max(Q::ONE);
        let share = qd(&(a_out_au * a_out_au), &(&Q::int(4) * &(&d_au * &d_au)));
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
    (d / c, d)
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
/// (`galaxy::Star::compactness`) at reach `r` (galaxy rings) is on at `age`
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

/// A world as held by its star: its native anchor (its place on its orbit,
/// from the star's G point) and its current place (sheared by pulses the star
/// could not fully re-lock), in AU; and its mass (world masses).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchor {
    /// Native place on the orbit, from the star's G point (AU).
    pub native: [Q; 3],
    /// Current place, including any shear not re-locked (AU).
    pub at: [Q; 3],
    /// World mass (world masses).
    pub mass: Q,
}

impl Anchor {
    /// A world on its native orbit of radius `a` AU.
    pub fn on_orbit(a: &Q, mass: &Q) -> Anchor {
        let native = [a.clone(), Q::ZERO, Q::ZERO];
        Anchor {
            at: native.clone(),
            native,
            mass: mass.clone(),
        }
    }

    /// Squared distance from the native place (AU^2): the shear.
    pub fn sheared(&self) -> Q {
        (0..3).fold(Q::ZERO, |s, i| {
            let d = &self.at[i] - &self.native[i];
            &s + &(&d * &d)
        })
    }
}

/// The result of one absorption: energy in, spent as gravity (re-locking
/// orbits), light and transduction-out; each world's place afterwards (with any shear
/// not re-locked), and the system's drift along the pulse (AU).
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
    /// The star's G point's drift along the pulse (AU).
    pub drift: [Q; 3],
}

impl Absorbed {
    /// The field law's books: energy in = gravity + light + transduced out,
    /// exactly.
    pub fn balances(&self) -> bool {
        &(&self.gravity + &self.light) + &self.transduced_out == self.energy_in
    }
}

/// A star of `mass` suns absorbs a pulse `v` bringing `energy_in`, its
/// current step converting `transduced_out` of what it transduces (`transduced_share`).
/// The pulse moves each world relative to the star's G point (which also
/// moves: the drift). The star's gravity returns each world toward its
/// native place by the share of the work the energy pays for: fully
/// re-locking a world costs `1/2 m (M/a)^2 |dx|^2`, with `a` its native
/// orbit and `dx` its displacement. The unpaid share remains as shear,
/// rounded down to the 2^-60 grid. `None` where a moved point cannot be
/// read back.
pub fn absorb(
    mass: &Q,
    transduced_out: &Q,
    worlds: &[Anchor],
    v: &Mv,
    energy_in: &Q,
) -> Option<Absorbed> {
    let drift = carry(v, &[Q::ZERO, Q::ZERO, Q::ZERO])?;
    let mut moved = Vec::with_capacity(worlds.len());
    let mut need = Q::ZERO;
    for w in worlds {
        let c = carry(v, &w.at)?;
        // Displacement from the native place, with the G point's drift removed.
        let dx: [Q; 3] = std::array::from_fn(|i| &(&c[i] - &drift[i]) - &w.native[i]);
        let a2 = (0..3).fold(Q::ZERO, |s, i| &s + &(&w.native[i] * &w.native[i]));
        let d2 = (0..3).fold(Q::ZERO, |s, i| &s + &(&dx[i] * &dx[i]));
        // Work 1/2 m (M / a)^2 |dx|^2, the turning M / a squared.
        let work = if a2.signum() > 0 {
            qd(&(&qd(&(&w.mass * &(mass * mass)), &a2) * &d2), &Q::int(2))
        } else {
            Q::ZERO
        };
        need = &need + &work;
        moved.push(dx);
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
    let left = &Q::ONE - &held;
    let anchors = worlds
        .iter()
        .zip(moved)
        // Only the remaining shear is rounded to the grid: a fully re-locked
        // world sits exactly on its native place.
        .map(|(w, dx)| Anchor {
            at: std::array::from_fn(|i| &w.native[i] + &down(&(&left * &dx[i]), GRID)),
            native: w.native.clone(),
            mass: w.mass.clone(),
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
        drift,
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
                tilt: (Dy::ONE, Dy::ZERO),
                node: (Dy::ONE, Dy::ZERO),
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
        let d = [Q::ratio(1, 3), Q::ratio(-2, 1), Q::ratio(5, 7)];
        assert_eq!(carry(&bend(&d), &origin()).unwrap(), d);
        // Rotate: a quarter turn about z (t = 1: 2 atan 1) takes x to +-y.
        let zaxis = [Q::ZERO, Q::ZERO, Q::ONE];
        let r = carry(&rotate(&zaxis, &Q::ONE), &[Q::ONE, Q::ZERO, Q::ZERO]).unwrap();
        assert!(
            r[0] == Q::ZERO && r[2] == Q::ZERO && (r[1] == Q::ONE || r[1] == -Q::ONE),
            "{r:?}"
        );
        // A rational turn (t = 1/2: cos 3/5, sin 4/5) lands on a rational
        // point.
        let r = carry(
            &rotate(&zaxis, &Q::ratio(1, 2)),
            &[Q::int(5), Q::ZERO, Q::ZERO],
        )
        .unwrap();
        assert_eq!(r[0], Q::int(3));
        assert_eq!(&r[1] * &r[1], Q::int(16));
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
        // Arrivals are counted in closed form: a period's span holds one ring.
        let b = &bs[0];
        let period = Dy::from_q(&b.period);
        let delay = Dy::ratio(1, 4);
        let t0 = Dy::int(10) * period + delay + period / Dy::int(2);
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
        let v = core.received(d);
        let worlds = vec![
            Anchor::on_orbit(&Q::ONE, &Q::ONE),
            Anchor::on_orbit(&Q::ratio(52, 10), &Q::int(318)),
            Anchor::on_orbit(&Q::ratio(3, 10), &Q::ratio(1, 20)),
        ];
        let e = core.energy_at(d, &Q::ratio(52, 10));
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
        // The drift is the pulse's translation along the z axis.
        assert!(a.drift[2].signum() != 0 && a.drift[0] == Q::ZERO && a.drift[1] == Q::ZERO);
    }

    /// A pure-hydrogen held mixture at heat 0.
    fn hydrogen() -> crate::stars::Sequence {
        crate::stars::Sequence::of(&[(1, Q::ONE)], &Q::ZERO)
    }

    /// The first step's release of the pure-hydrogen sequence.
    fn h_release() -> Q {
        transduced_share(&hydrogen(), Transduction::Step(0))
    }

    #[test]
    fn transduced_share_reads_each_steps_release() {
        // Each step converts its release of what it transduces; a body that does
        // not ignite and a remnant convert nothing.
        let seq = hydrogen();
        for (i, step) in seq.steps.iter().enumerate() {
            let share = transduced_share(&seq, Transduction::Step(i));
            assert_eq!(share, step.release);
            assert!(share > Q::ZERO && share < Q::ONE, "step {i}: {share}");
        }
        assert_eq!(transduced_share(&seq, Transduction::Remnant), Q::ZERO);
        assert_eq!(transduced_share(&seq, Transduction::Unlit), Q::ZERO);
        // The step by age at r = 0 follows the ends in order, then the
        // remnant (the universe is founded first because the bell sets the
        // life).
        let _ = crate::genesis::origin();
        let delta = crate::galaxy::first_star_compactness();
        let ends = crate::stars::ends(&seq, delta, Dy::ZERO);
        assert!(ends[0].signum() > 0);
        let mut start = Q::ZERO;
        for (i, end) in ends.iter().enumerate() {
            if *end > start {
                let mid = Dy::from_q(&(&start + end)) / Dy::int(2);
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
