//! The field: the singularities' pulse, carried to the stars and absorbed by
//! them (the field law; docs/model.md).
//!
//! Every bell (pulse.rs: the core, and the hole every heavy star leaves)
//! emits a pulse that is a versor of Cl(4,1), exact over Q: bent (a
//! translator), rotated (a rotor through a rational point of the circle, no
//! float angle), twisted (a screw: a rotor with a translation along its axis
//! whose pitch, the translation per full turn, is its wavelength), or flipped
//! (a reflection, an odd versor). A loop of pulses that fails to close leaves
//! a translation: torsion, as holonomy.
//!
//! Each bell has its own screw: its pitch is set by its bounce (the core's
//! wavelength scaled by the extent of its mass over the core's; see
//! [`pitch_kpc`]), its rotor turns about its hole's spin axis by its Kerr
//! spin, it rings once a period (its pitch at light speed), and each ring
//! carries one period of its light.
//!
//! A star absorbs each pulse that reaches it. The pulse shears its worlds'
//! orbits (each world a 0D anchor held from the star's G point at its orbit's
//! radius); the star spends energy as gravity to re-lock them (the work to
//! undo each shear, 1/2 m (M/a)^2 |dx|^2, up to the energy received) and
//! commits the rest to burning: the mass defect of its current fusion step
//! burns out and the remainder leaves as light. The books balance exactly:
//! in = gravity + light + burned out. Whatever cannot be re-locked stays
//! sheared. The pulse's translation moves the whole system uniformly (its
//! drift) and shears no orbit.
//!
//! Arrivals are in closed form: a bell's ring k reaches a place at its ring
//! time plus the light time across the proper distance, so the pulses
//! arriving in any span are counted directly.
//!
//! Energy is in the unit world-mass AU^2 (yr / 2 pi)^-2, in which an orbit's
//! turning is M / a (suns over AU: the turning law, against a year at 1 AU
//! about one sun) and its stiffness that turning squared, so every orbital
//! quantity is an exact rational. Floats are never used for orbits.

use crate::centre::{efficiency, horizon_share};
use crate::chain::Place;
use crate::package::hole_mass;
use crate::pulse::{c_kpc_yr, wavelength_kpc, Kind, Node};
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

fn q(n: i64, d: i64) -> Q {
    Q::new(n, d).unwrap_or(Q::ZERO)
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

fn dec(s: &str) -> Q {
    crate::elements::decimal(s).unwrap_or(Q::ZERO)
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
    dec("2.28002")
}

/// A hole's light per sun of its mass (suns): the share of an Eddington-fed
/// flow (32 000 suns per sun) that crosses its horizon, radiated at its
/// efficiency (centre.rs).
pub fn hole_light_per_star_mass() -> Q {
    &(&Q::int(32_000) * &horizon_share()) * &efficiency()
}

// ---------------------------------------------------------------------- versors

/// The bivector of the plane a turn about `axis` turns in (`axis`'s dual):
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

/// Twist: a turn about `axis` combined with the translation `along`.
pub fn twist(axis: &[Q; 3], t: &Q, along: &[Q; 3]) -> Mv {
    bend(along).geo(&rotate(axis, t))
}

/// A point moved by a versor, exactly (an odd versor acts by its twisted
/// adjoint, as a reflection does).
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

// ---------------------------------------------------------------------- the bells

/// A bell's own screw and its light.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bell {
    /// The hole's spin axis (a direction, exact on the grid) and Kerr spin.
    pub axis: [Q; 3],
    /// The hole's Kerr spin parameter J / M^2, in (0, 1).
    pub spin: Q,
    /// Pitch: the wavelength set by the bounce (kpc).
    pub pitch_kpc: Q,
    /// Period: the pitch at light speed (yr), one ring per period.
    pub period_yr: Q,
    /// Mass (suns).
    pub mass: Q,
    /// Light output (suns).
    pub light: Q,
    /// Formation time (Gyr after the Big Bang).
    pub born: Dy,
}

/// The pitch of a hole of `mass` suns: the core's wavelength scaled by the
/// extent of M / M_core under the body measure (systems.rs `extent`, used in
/// place of (M / M_core)^(1/3)).
pub fn pitch_kpc(mass: Dy) -> Dy {
    let ratio = (mass / Dy::from_q(&hole_mass())).max(Dy::sci(1, -30));
    wavelength_kpc() * Dy::from_q(&crate::elements::extent(&ratio.to_q()))
}

/// A stellar hole's Kerr spin in (0, 1), drawn deterministically from the
/// seed and star index.
fn drawn_spin(seed: u64, n: u64) -> Q {
    let mut h = blake3::Hasher::new_derive_key("xylarium 2026-10-08 stellar hole spin");
    h.update(&seed.to_le_bytes());
    h.update(&n.to_le_bytes());
    let b = h.finalize();
    let x = u64::from_le_bytes(b.as_bytes()[..8].try_into().unwrap_or([0; 8])) >> 12;
    // (x + 1) / (2^52 + 2): strictly inside (0, 1)
    Q::new(i64::try_from(x).unwrap_or(0) + 1, (1i64 << 52) + 2).unwrap_or(q(1, 2))
}

impl Bell {
    /// A node's bell: the core spins about the disc's axis at the measured
    /// spin; a stellar hole spins about its progenitor's orbit normal at its
    /// drawn spin.
    pub fn of(node: &Node, seed: u64) -> Bell {
        let (axis, spin) = match &node.kind {
            Kind::Core(_) => ([Q::ZERO, Q::ZERO, Q::ONE], crate::circuit::anchor().spin),
            Kind::Stellar(s) => {
                let o = &s.orbit;
                // the orbit plane's normal: the core's axis carried by its frame
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
        let period_yr = grid(pitch / c_kpc_yr());
        let mass = grid(node.mass);
        let light = &mass * &hole_light_per_star_mass();
        Bell {
            axis,
            spin,
            pitch_kpc,
            period_yr,
            mass,
            light,
            born: node.born,
        }
    }

    /// The number of rings reaching a place `delay_yr` of light away within
    /// `(from_gyr, to_gyr]`: ring k sounds at born + k periods and arrives one
    /// light time later. Closed form.
    pub fn arrivals(&self, delay_yr: Dy, from_gyr: Dy, to_gyr: Dy) -> u64 {
        let period = Dy::from_q(&self.period_yr);
        if period.signum() <= 0 || to_gyr <= from_gyr {
            return 0;
        }
        let ring = |gyr: Dy| -> i64 {
            let since = gyr * Dy::sci(1, 9) - self.born * Dy::sci(1, 9) - delay_yr;
            if since.signum() < 0 {
                -1
            } else {
                (since / period).trunc_i64()
            }
        };
        u64::try_from(ring(to_gyr) - ring(from_gyr)).unwrap_or(0)
    }

    /// The pulse at a place `d_kpc` away: the bell's screw scaled by the
    /// strain there, a turn about its axis by `2 atan(t |axis|)` with
    /// `t = spin x r_g / d`, plus a translation along the axis of the pitch in
    /// proportion to the turn (in AU).
    pub fn received(&self, d_kpc: Dy) -> Mv {
        // the hole's r_g: the core's r_g scaled by mass over the core's
        let rg_kpc = Dy::from_q(&crate::chain::units().r_g_m)
            / Dy::from_q(&crate::package::Si::kpc())
            * Dy::from_q(&self.mass)
            / Dy::from_q(&hole_mass());
        let t = grid(Dy::from_q(&self.spin) * rg_kpc / d_kpc.max(Dy::sci(1, -9)));
        // the axis length in first degree (sum of |components|, no root)
        let norm = self
            .axis
            .iter()
            .fold(Dy::ZERO, |s, x| s + Dy::from_q(x).abs());
        let angle = n2() * (Dy::from_q(&t) * norm).atan();
        // the screw's translation for this turn, in AU, along the axis
        let shift = Dy::from_q(&self.pitch_kpc) * Dy::from_q(&au_per_kpc()) * angle
            / (n2() * Dy::PI)
            / norm.max(Dy::sci(1, -30));
        let along = [
            grid(shift * Dy::from_q(&self.axis[0])),
            grid(shift * Dy::from_q(&self.axis[1])),
            grid(shift * Dy::from_q(&self.axis[2])),
        ];
        twist(&self.axis, &t, &along)
    }

    /// The energy of one ring intercepted by a system of radius `a_out_au` at
    /// `d_kpc`: one period of light times the sphere share the system's disc
    /// covers (pi a^2 over 4 pi d^2: a^2 / 4 d^2, exact).
    pub fn energy_at(&self, d_kpc: Dy, a_out_au: &Q) -> Q {
        let d_au = grid(d_kpc * Dy::from_q(&au_per_kpc())).max(Q::ONE);
        let share = qd(&(a_out_au * a_out_au), &(&Q::int(4) * &(&d_au * &d_au)));
        &(&(&self.light * &self.period_yr) * &star_year()) * &share
    }
}

/// Every node's bell, in node order.
pub fn bells(nodes: &[Node], seed: u64) -> Vec<Bell> {
    nodes.iter().map(|n| Bell::of(n, seed)).collect()
}

/// The light delay (yr) and proper distance (kpc) from a bell to a place at
/// `t` Gyr.
pub fn path(node: &Node, place: &Place, t: Dy) -> (Dy, Dy) {
    let r = crate::chain::radius();
    let d = place.arc_kpc(&node.place(t, &r), &r);
    (d / c_kpc_yr(), d)
}

// ---------------------------------------------------------------------- the star burns

/// Measured binding energies (MeV) of the nuclei a star burns through (He-4,
/// C-12, O-16).
fn binding_mev(a: u32) -> Q {
    match a {
        4 => dec("28.29566"),
        12 => dec("92.16173"),
        16 => dec("127.61927"),
        _ => Q::ZERO,
    }
}
/// A nucleon's rest energy (MeV).
fn nucleon_mev() -> Q {
    dec("938.272")
}
/// The cost of converting a proton to a neutron in a fusion step:
/// m_n - m_p + m_e, less the 2 m_e returned by positron annihilation.
fn weak_mev() -> Q {
    &dec("1.29333") - &dec("0.51100")
}

/// The fusion step a star is on, by its age against its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Burning {
    /// Hydrogen to helium (main sequence).
    Hydrogen,
    /// Helium to carbon (giant core).
    Helium,
    /// Fuel spent: a remnant burns nothing.
    Remnant,
}

/// The step a star at reach `r` (core rings) is on at `age_gyr`: hydrogen
/// until it leaves the main sequence, helium until its giant core's helium
/// is spent, then remnant; both ends from the pulse's burn of its hold
/// (stars::ends).
pub fn burning(reach: Dy, age_gyr: Dy) -> Burning {
    let (main, giant) = crate::stars::ends(reach);
    let age = age_gyr.to_q();
    if age < main {
        Burning::Hydrogen
    } else if age < giant {
        Burning::Helium
    } else {
        Burning::Remnant
    }
}

/// The share of burned mass converted to energy: the current step's mass
/// defect, the binding released over the inputs' rest energy.
pub fn burn_out_share(b: Burning) -> Q {
    match b {
        // 4 H -> He-4: two protons made neutrons
        Burning::Hydrogen => qd(
            &(&binding_mev(4) - &(&Q::int(2) * &weak_mev())),
            &(&Q::int(4) * &nucleon_mev()),
        ),
        // 3 He-4 -> C-12
        Burning::Helium => qd(
            &(&binding_mev(12) - &(&Q::int(3) * &binding_mev(4))),
            &(&Q::int(12) * &nucleon_mev()),
        ),
        Burning::Remnant => Q::ZERO,
    }
}

// ---------------------------------------------------------------------- the star absorbs

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
/// orbits), light and burn-out; each world's place afterwards (with any shear
/// not re-locked), and the system's drift along the pulse (AU).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Absorbed {
    /// Energy the pulse brings, in field energy units (one unit = world mass x AU^2 x (2 pi / yr)^2 = 5.29829e33 J; a Sun-year of light is 2.28002 units), held as a 62-bit dyadic.
    pub energy_in: Q,
    /// Energy spent re-locking orbits (field energy units), at most `energy_in`.
    pub gravity: Q,
    /// Energy left as light (field energy units): the remainder after gravity
    /// and burn-out.
    pub light: Q,
    /// Energy converted by the current burning step's mass defect (field
    /// energy units).
    pub burned_out: Q,
    /// Each world's anchor after the absorption, in the input order.
    pub anchors: Vec<Anchor>,
    /// The star's G point's drift along the pulse (AU).
    pub drift: [Q; 3],
}

impl Absorbed {
    /// The field law's books: energy in = gravity + light + burned out,
    /// exactly.
    pub fn balances(&self) -> bool {
        &(&self.gravity + &self.light) + &self.burned_out == self.energy_in
    }
}

/// A star of `mass` suns on `step` absorbs a pulse `v` bringing `energy_in`.
/// The pulse moves each world relative to the star's G point (which also
/// moves: the drift). The star's gravity pulls each back toward its native
/// place by the share of the work it can pay for: fully re-locking a world
/// costs `1/2 m (M/a)^2 |dx|^2`, with `a` its native orbit and `dx` its
/// displacement. The unpaid share remains as shear, exact on the grid.
pub fn absorb(
    mass: &Q,
    step: Burning,
    worlds: &[Anchor],
    v: &Mv,
    energy_in: &Q,
) -> Option<Absorbed> {
    let drift = carry(v, &[Q::ZERO, Q::ZERO, Q::ZERO])?;
    let mut moved = Vec::with_capacity(worlds.len());
    let mut need = Q::ZERO;
    for w in worlds {
        let c = carry(v, &w.at)?;
        // displacement from the native place, with the G point's drift removed
        let dx: [Q; 3] = std::array::from_fn(|i| &(&c[i] - &drift[i]) - &w.native[i]);
        let a2 = (0..3).fold(Q::ZERO, |s, i| &s + &(&w.native[i] * &w.native[i]));
        let d2 = (0..3).fold(Q::ZERO, |s, i| &s + &(&dx[i] * &dx[i]));
        // the turning M / a, squared: M^2 / a^2
        let work = if a2.signum() > 0 {
            qd(&(&qd(&(&w.mass * &(mass * mass)), &a2) * &d2), &Q::int(2))
        } else {
            Q::ZERO
        };
        need = &need + &work;
        moved.push(dx);
    }
    // every term is held as a 62-bit dyadic (Dy), so books summed over a
    // universe's life keep power-of-two denominators and stay bounded; the
    // rounding is monotone (gravity never exceeds the input) and light takes
    // the exact remainder, so the books still balance
    let energy_in = &dyadic(energy_in);
    let gravity = dyadic(if &need < energy_in { &need } else { energy_in });
    // whether the star pays in full is decided exactly, not from the rounded
    // gravity: a star that can re-lock its worlds keeps them on their orbits
    let held = if need.signum() <= 0 || &need <= energy_in {
        Q::ONE
    } else {
        qd(&gravity, &need)
    };
    let left = &Q::ONE - &held;
    let anchors = worlds
        .iter()
        .zip(moved)
        // only the remaining shear is rounded to the grid: a fully re-locked
        // world sits exactly on its native place
        .map(|(w, dx)| Anchor {
            at: std::array::from_fn(|i| &w.native[i] + &down(&(&left * &dx[i]), GRID)),
            native: w.native.clone(),
            mass: w.mass.clone(),
        })
        .collect();
    let rest = energy_in - &gravity;
    let burned_out = dyadic(&(&rest * &burn_out_share(step))).min(rest.clone());
    let light = &rest - &burned_out;
    Some(Absorbed {
        energy_in: energy_in.clone(),
        gravity,
        light,
        burned_out,
        anchors,
        drift,
    })
}

fn n2() -> Dy {
    Dy::int(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bang::BigBang;
    use crate::galaxy::{star, NUCLEUS_BASE};

    /// The core's node, and a stellar hole's on a nucleus star - built
    /// directly, not by sampling the galaxy for heavy stars.
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
    fn stellar(bang: &BigBang) -> Node {
        // the first sample star that stands: a star stands only where its
        // ring's gas lights (galaxy::birth_gyr)
        let s = (0..crate::galaxy::sample(bang))
            .chain(std::iter::once(NUCLEUS_BASE))
            .find_map(|n| star(7, bang, n))
            .expect("a standing star");
        Node {
            kind: Kind::Stellar(Box::new(s)),
            mass: Dy::int(50),
            born: Dy::int(1),
        }
    }

    fn z() -> [Q; 3] {
        [Q::ZERO, Q::ZERO, Q::ZERO]
    }

    #[test]
    fn a_pulse_bends_turns_twists_and_flips_exactly() {
        // bent: a translation by (1/3, -2, 5/7)
        let d = [q(1, 3), q(-2, 1), q(5, 7)];
        assert_eq!(carry(&bend(&d), &z()).unwrap(), d);
        // rotated: a quarter turn about z (t = 1: 2 atan 1) takes x to y
        let zaxis = [Q::ZERO, Q::ZERO, Q::ONE];
        let r = carry(&rotate(&zaxis, &Q::ONE), &[Q::ONE, Q::ZERO, Q::ZERO]).unwrap();
        assert!(
            r[0] == Q::ZERO && r[2] == Q::ZERO && (r[1] == Q::ONE || r[1] == -Q::ONE),
            "{r:?}"
        );
        // a rational turn (t = 1/2: cos 3/5, sin 4/5) lands on a rational point
        let r = carry(&rotate(&zaxis, &q(1, 2)), &[Q::int(5), Q::ZERO, Q::ZERO]).unwrap();
        assert_eq!(r[0], Q::int(3));
        assert_eq!(&r[1] * &r[1], Q::int(16));
    }

    #[test]
    fn the_bells_ring_their_own_screws_and_arrive_by_light() {
        let bang = crate::genesis::bang();
        let bs = bells(&[core(), stellar(bang)], 7);
        // the core rings the wavefront's own wavelength and period
        assert!((Dy::from_q(&bs[0].pitch_kpc) - wavelength_kpc()).abs() < Dy::sci(1, -9));
        assert_eq!(bs[0].axis, [Q::ZERO, Q::ZERO, Q::ONE]);
        assert_eq!(bs[0].spin, crate::circuit::anchor().spin);
        // a stellar hole rings a shorter pitch, faster, about its own axis
        let s = bs.get(1).expect("a stellar bell");
        assert!(s.pitch_kpc < bs[0].pitch_kpc && s.period_yr < bs[0].period_yr);
        assert!(s.spin.signum() > 0 && s.spin < Q::ONE);
        // arrivals are counted in closed form: a period's span holds one ring
        let b = &bs[0];
        let per_gyr = Dy::from_q(&b.period_yr) / Dy::sci(1, 9);
        let t0 = Dy::int(10);
        let delay = Dy::int(26_000);
        let one = b.arrivals(delay, t0, t0 + per_gyr);
        assert_eq!(one, 1, "one period, one ring");
        assert_eq!(b.arrivals(delay, t0, t0 + per_gyr * Dy::int(10)), 10);
        assert_eq!(b.arrivals(delay, t0, t0), 0);
        // nothing arrives before a bell has formed and its light crossed
        let late = Bell {
            born: Dy::int(12),
            ..b.clone()
        };
        assert_eq!(late.arrivals(delay, Dy::int(5), Dy::int(11)), 0);
    }

    #[test]
    fn a_star_absorbs_the_pulse_and_its_books_balance_exactly() {
        let core = Bell::of(&core(), 7);
        let d = Dy::int(8);
        let v = core.received(d);
        let worlds = vec![
            Anchor::on_orbit(&Q::ONE, &Q::ONE),
            Anchor::on_orbit(&q(52, 10), &Q::int(318)),
            Anchor::on_orbit(&q(3, 10), &q(1, 20)),
        ];
        let e = core.energy_at(d, &q(52, 10));
        assert!(e.signum() > 0);
        let a = absorb(&Q::ONE, Burning::Hydrogen, &worlds, &v, &e).unwrap();
        assert!(
            a.balances(),
            "in = gravity + light + burned out, exactly: {a:?}"
        );
        // every term dyadic (power-of-two denominator): books summed over a
        // life stay bounded
        for x in [&a.energy_in, &a.gravity, &a.burned_out] {
            assert_eq!(dyadic(x), *x, "{x} is not held to 62 bits");
        }
        assert!(a.gravity.signum() > 0, "the star re-locks its worlds");
        assert!(a.burned_out.signum() > 0 && a.light > a.burned_out);
        // a pulse too weak to pay for the re-locking leaves the orbits sheared
        let weak = qd(&a.gravity, &Q::int(2));
        let b = absorb(&Q::ONE, Burning::Hydrogen, &worlds, &v, &weak).unwrap();
        assert!(b.balances());
        assert_eq!(b.light, Q::ZERO);
        assert!(
            b.anchors.iter().any(|w| w.sheared().signum() > 0),
            "what it could not re-lock stays sheared"
        );
        // a star that pays for all of it holds its worlds on their orbits
        assert!(
            a.anchors.iter().all(|w| w.sheared() == Q::ZERO),
            "wholly re-locked"
        );
        // the next pulse acts on the sheared worlds where they stand: shear
        // accumulates while the star cannot pay
        let c = absorb(&Q::ONE, Burning::Hydrogen, &b.anchors, &v, &weak).unwrap();
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
        // the drift is the pulse's translation along the axis: the whole system alike
        assert!(a.drift[2].signum() != 0 && a.drift[0] == Q::ZERO && a.drift[1] == Q::ZERO);
    }

    #[test]
    fn a_star_burns_out_its_own_steps_defect() {
        // each step converts a share of what it burns, the helium step less
        // per nucleon than the hydrogen step, a remnant nothing
        let h = burn_out_share(Burning::Hydrogen);
        let he = burn_out_share(Burning::Helium);
        assert!(h > Q::ZERO && h < Q::ONE, "H -> He: {h}");
        assert!(he > Q::ZERO && he < h, "He -> C: {he}");
        assert_eq!(burn_out_share(Burning::Remnant), Q::ZERO);
        // a star burns hydrogen through its life at its reach, then helium,
        // then nothing (the universe founded first: the bell sets the life)
        let _ = crate::genesis::bang();
        let (main, giant) = crate::stars::ends(Dy::ZERO);
        assert!(main.signum() > 0 && giant > main, "{main} {giant}");
        let half = Dy::from_q(&main) / Dy::int(2);
        assert_eq!(burning(Dy::ZERO, half), Burning::Hydrogen);
        let mid = Dy::from_q(&(&main + &giant)) / Dy::int(2);
        assert_eq!(burning(Dy::ZERO, mid), Burning::Helium);
        assert_eq!(
            burning(Dy::ZERO, Dy::from_q(&giant) * Dy::int(2)),
            Burning::Remnant
        );
    }
}
