//! Orbits and spins: the drawn elements, tidal circularisation and spin
//! lock, and the orbit at an age.

use super::*;

/// A world's orbital elements and spin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Orbit {
    /// The host's mass, in kg.
    pub host_kg: Q,
    /// Semi-major axis, in m.
    pub a_m: Q,
    /// Eccentricity at birth, in \[0, 0.9\], capped at half the gap to the
    /// nearest neighbour.
    pub e0: Q,
    /// Eccentricity forced by the neighbours (Laplace-Lagrange leading term),
    /// toward which `e0` damps; at most `e0` when `e0` is positive.
    pub e_forced: Q,
    /// Inclination, in radians, in \[0, pi/2\].
    pub inc: Q,
    /// Longitude of the ascending node, as a fraction of a turn in \[0, 1).
    pub node: Q,
    /// Argument of periapsis, as a fraction of a turn in \[0, 1).
    pub peri: Q,
    /// Cosine of the spin axis's tilt at birth, in \[-1, 1\].
    pub cos_obliquity: Q,
    /// Spin at birth, in rad/s (a share 0.1 to 0.5 of break-up spin).
    pub spin0: Q,
    /// Age at which the host's tides lock the spin, in Gyr.
    pub lock_gyr: Q,
    /// Tidal damping time of eccentricity, in Gyr.
    pub tau_e: Q,
}

/// What an orbit is at an age.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Now {
    /// Eccentricity at this age, damped from `e0` toward the forced value.
    pub e: Q,
    /// Whether the host's tides have locked the spin (age at or past `lock_gyr`).
    pub locked: bool,
    /// Length of the day, in standard days of 86 400 s.
    pub day: Q,
    /// Length of the year (orbital period), in s.
    pub year_s: Q,
    /// Cosine of the spin axis's tilt, in \[-1, 1\]; 1 once locked.
    pub cos_obliquity: Q,
}

/// A body's self-gravity pressure (Pa): the unit world's rho g R (3.4e11 Pa)
/// times the body's mass over its radius, both in world units.
pub(super) fn self_pressure(mass: &Q, radius: &Q) -> Q {
    (&Rock::world_pressure() * mass)
        .checked_div(radius)
        .unwrap_or(Q::ZERO)
}

/// The Love number `k2` of a uniform elastic body:
/// `k2 = 1.5 / (1 + 19 mu / (2 P))`, with mu the shear rigidity of rock or
/// ice and P the body's self-gravity pressure.
pub fn love(mass: &Q, radius: &Q, icy: bool) -> Q {
    let mu = if icy { Rock::mu_ice() } else { Rock::mu_rock() };
    let den = &Q::ONE
        + &(&Q::int(19) * &mu)
            .checked_div(&(&Q::int(2) * &self_pressure(mass, radius)))
            .unwrap_or(Q::ZERO);
    on_grid(Q::ratio(3, 2).checked_div(&den).unwrap_or(Q::ZERO))
}

/// The tidal stretch across a body: the change of its host's deficit across
/// it, delta(a - R) - delta(a + R), with delta(x) = M / (x + M) and M the
/// host's mass as a length (G M / c^2).
pub(super) fn stretch(host_kg: &Q, a_m: &Q, r_m: &Q) -> Q {
    // The host's mass as a length is r_g times the host's mass over the
    // core's mass.
    let core_kg = &crate::package::hole_mass() * &Si::star_mass_kg();
    let m_len = &crate::chain::units().r_g_m * &host_kg.checked_div(&core_kg).unwrap_or(Q::ZERO);
    let delta = |x: &Q| m_len.checked_div(&(x + &m_len)).unwrap_or(Q::ZERO);
    (&delta(&(a_m - r_m).max(Q::ONE)) - &delta(&(a_m + r_m))).max(Q::ZERO)
}

/// A planet's mean motion about its star (rad/s), from the turning law.
pub(super) fn mean_motion(host_kg: &Q, a_m: &Q) -> Q {
    crate::turning::rad_s(&crate::turning::turning_si(host_kg, a_m))
}

/// Tidal damping time of eccentricity (Gyr):
/// 4/63 (Q / k2) (m / M) / (n stretch).
pub fn circularise_gyr(host_kg: &Q, a_m: &Q, mass: &Q, radius: &Q, icy: bool) -> Q {
    let (m, r) = (mass * &Si::world_mass_kg(), radius * &Si::world_radius_m());
    let n = mean_motion(host_kg, a_m);
    let k2 = love(mass, radius, icy);
    let t = &(&Q::ratio(4, 63) * &Rock::tidal_q().checked_div(&k2).unwrap_or(Q::ZERO))
        * &m.checked_div(host_kg).unwrap_or(Q::ZERO);
    let rate = &n * &stretch(host_kg, a_m, &r);
    on_grid(
        t.checked_div(&rate)
            .unwrap_or(Q::ZERO)
            .checked_div(&Si::gyr_s())
            .unwrap_or(Q::ZERO),
    )
}

/// Time for a host's tides to brake a spin `spin0` (rad/s) to lock (Gyr):
/// (spin0 / n) (I Q / (3 k2 stretch)) / n.
pub fn lock_gyr(host_kg: &Q, a_m: &Q, mass: &Q, radius: &Q, icy: bool, spin0: &Q) -> Q {
    let r = radius * &Si::world_radius_m();
    let k2 = love(mass, radius, icy);
    let n = mean_motion(host_kg, a_m).max(Q::ratio(1, 1 << 60));
    let rigid = (&Rock::inertia() * &Rock::tidal_q())
        .checked_div(&(&(&Q::int(3) * &k2) * &stretch(host_kg, a_m, &r)))
        .unwrap_or(Q::ZERO);
    let secs = (&spin0.checked_div(&n).unwrap_or(Q::ZERO) * &rigid)
        .checked_div(&n)
        .unwrap_or(Q::ZERO);
    on_grid(secs.checked_div(&Si::gyr_s()).unwrap_or(Q::ZERO))
}

/// The orbital elements and spins of a star's planets (innermost first);
/// `star_kg` is the star's mass.
pub fn orbits(star_kg: &Q, worlds: &[&World], rng: &mut Stream) -> Vec<Orbit> {
    let a_au: Vec<Q> = worlds.iter().map(|w| w.a.clone()).collect();
    let mut out: Vec<Orbit> = Vec::with_capacity(worlds.len());
    for (n, w) in worlds.iter().enumerate() {
        let a_m = &a_au[n] * &Si::au();
        let mut gaps = Vec::new();
        for k in [n.checked_sub(1), Some(n + 1)].into_iter().flatten() {
            if k < worlds.len() {
                let (a1, a2) = if a_au[n] < a_au[k] {
                    (&a_au[n], &a_au[k])
                } else {
                    (&a_au[k], &a_au[n])
                };
                gaps.push((a2 - a1).checked_div(&(a2 + a1)).unwrap_or(Q::ONE));
            }
        }
        let cap = gaps
            .iter()
            .min()
            .cloned()
            .map(|x| &x * &Q::ratio(1, 2))
            .unwrap_or(Q::ratio(9, 10));
        // The orbit's shape comes from the settling (systems::settle_about).
        // The birth eccentricity is the settled swing, capped by the gap to
        // the nearest neighbour; the inclination is the swing's deficit
        // angle, swing / (1 + swing) of a quarter turn; node and periapsis
        // are both the settled phase.
        let e0 = w.swing.clone().min(cap);
        let inc = &(&pi_q() * &Q::ratio(1, 2))
            * &w.swing
                .checked_div(&(&Q::ONE + &w.swing))
                .unwrap_or(Q::ZERO);
        let node = w.phase.clone();
        let peri = w.phase.clone();
        let cos_obliquity = &(&Q::int(2) * &unit(rng)) - &Q::ONE;
        let share = &Q::ratio(1, 10) + &(&Q::ratio(4, 10) * &unit(rng));
        let icy = w.layers[2] > Q::ratio(1, 5);
        let (m, r) = (
            &w.mass * &Si::world_mass_kg(),
            &w.radius * &Si::world_radius_m(),
        );
        let breakup = surface_turning(&m, &r);
        let spin0 = on_grid(&share * &breakup);
        let lock = lock_gyr(star_kg, &a_m, &w.mass, &w.radius, icy, &spin0);
        let tau_e = circularise_gyr(star_kg, &a_m, &w.mass, &w.radius, icy);
        out.push(Orbit {
            host_kg: star_kg.clone(),
            a_m,
            e0,
            e_forced: Q::ZERO,
            inc,
            node,
            peri,
            cos_obliquity,
            spin0,
            lock_gyr: lock,
            tau_e,
        });
    }
    // Forced eccentricity (Laplace-Lagrange, leading term): for each
    // neighbour, 5/4 x (inner axis / outer axis) x the neighbour's e0; the
    // largest value is kept.
    let forced: Vec<Q> = (0..worlds.len())
        .map(|n| {
            let mut f = Q::ZERO;
            for k in [n.checked_sub(1), Some(n + 1)].into_iter().flatten() {
                if k < worlds.len() {
                    let ratio = a_au[n]
                        .clone()
                        .min(a_au[k].clone())
                        .checked_div(&a_au[n].clone().max(a_au[k].clone()))
                        .unwrap_or(Q::ZERO);
                    f = f.max(&(&Q::ratio(5, 4) * &ratio) * &out[k].e0);
                }
            }
            f
        })
        .collect();
    for (o, f) in out.iter_mut().zip(forced) {
        o.e_forced = if o.e0.signum() > 0 {
            o.e0.clone().min(f)
        } else {
            f
        };
    }
    out
}

impl Orbit {
    /// The orbit and spin at an age `t` (Gyr): eccentricity damped
    /// exponentially toward the forced value with time constant `tau_e`, and
    /// spin braked linearly toward the mean motion until `lock_gyr`. Once
    /// locked, the day equals the year and the obliquity cosine is 1.
    pub fn at(&self, t: &Q) -> Now {
        let e = if self.tau_e.signum() <= 0 {
            self.e_forced.clone()
        } else {
            &self.e_forced
                + &(&(&self.e0 - &self.e_forced)
                    * &ex(&-(t.checked_div(&self.tau_e).unwrap_or(Q::ZERO))))
        };
        let n = mean_motion(&self.host_kg, &self.a_m);
        let locked = t >= &self.lock_gyr;
        let (w_now, cos_obliquity) = if locked {
            (n.clone(), Q::ONE)
        } else {
            let frac = t.checked_div(&self.lock_gyr).unwrap_or(Q::ZERO);
            (
                &self.spin0 - &(&(&self.spin0 - &n) * &frac),
                self.cos_obliquity.clone(),
            )
        };
        let two_pi = &Q::int(2) * &pi_q();
        let day_s = two_pi
            .checked_div(&w_now.max(Q::ratio(1, 1_000_000_000_000)))
            .unwrap_or(Q::ZERO);
        Now {
            e: on_grid(e),
            locked,
            day: on_grid(day_s.checked_div(&Si::day_s()).unwrap_or(Q::ZERO)),
            year_s: on_grid(two_pi.checked_div(&n).unwrap_or(Q::ZERO)),
            cos_obliquity,
        }
    }
}
