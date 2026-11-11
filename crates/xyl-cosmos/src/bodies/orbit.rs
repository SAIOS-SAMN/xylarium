//! Orbits and spins: the drawn elements, tidal circularisation and spin
//! lock. The orbit and spin at an age are read by `worlds::Path`.

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
    /// The spin axis's tilt at birth (degrees, in \[0, 90\]): the natural
    /// balance of the world's charge ([`world_tilt`], #183); `None` where the
    /// world's elements are not read, and the axis then stands upright.
    pub obliquity: Option<Q>,
    /// Cosine of the spin axis's tilt at birth, in \[0, 1\]: the tilt's
    /// exact point of the circle (`rotor::circle_point`), 1 when upright.
    pub cos_obliquity: Q,
    /// Spin at birth, in rad/s (a share 0.1 to 0.5 of break-up spin).
    pub spin0: Q,
    /// Age at which the host's tides lock the spin, in Gyr.
    pub lock_gyr: Q,
    /// Tidal damping time of eccentricity, in Gyr.
    pub tau_e: Q,
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
    on_grid(xyl_core::ratio!(3, 2).checked_div(&den).unwrap_or(Q::ZERO))
}

/// The tidal stretch across a body: the change of its host's deficit across
/// it, delta(a - R) - delta(a + R), with delta(x) = M / (x + M) and M the
/// host's mass as a length (G M / c^2).
pub(super) fn stretch(host_kg: &Q, a_m: &Q, r_m: &Q) -> Q {
    // The host's mass as a length is r_g times the host's mass over the
    // core's mass.
    let m_len = &crate::chain::units().r_g_m
        * &host_kg
            .checked_div(&crate::package::core_kg())
            .unwrap_or(Q::ZERO);
    let delta = |x: &Q| xyl_core::shares::deficit(&m_len, x).unwrap_or(Q::ZERO);
    (&delta(&(a_m - r_m).max(Q::ONE)) - &delta(&(a_m + r_m))).max(Q::ZERO)
}

/// Tidal damping time of eccentricity (Gyr):
/// 4/63 (Q / k2) (m / M) / (n stretch).
pub fn circularise_gyr(host_kg: &Q, a_m: &Q, mass: &Q, radius: &Q, icy: bool) -> Q {
    let (m, r) = (mass * &Si::world_mass_kg(), radius * &Si::world_radius_m());
    let n = rad_s(host_kg, a_m);
    let k2 = love(mass, radius, icy);
    let t = &(&xyl_core::ratio!(4, 63) * &Rock::tidal_q().checked_div(&k2).unwrap_or(Q::ZERO))
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
    let n = rad_s(host_kg, a_m).max(xyl_core::ratio!(1, 1 << 60));
    let rigid = (&Rock::inertia() * &Rock::tidal_q())
        .checked_div(&(&(&Q::int(3) * &k2) * &stretch(host_kg, a_m, &r)))
        .unwrap_or(Q::ZERO);
    let secs = (&spin0.checked_div(&n).unwrap_or(Q::ZERO) * &rigid)
        .checked_div(&n)
        .unwrap_or(Q::ZERO);
    on_grid(secs.checked_div(&Si::gyr_s()).unwrap_or(Q::ZERO))
}

/// A world's axis tilt at the natural balance of its charge (#183):
/// `eps* = 90 pi` degrees with `pi = (p / w) (1 - delta)`, `p / w` the proton
/// share of its matter (`sum m_i Z_i / A_i` over `sum m_i`, by element) and
/// `delta` its held face at its rim, `m / (R + m)` with its mass in core
/// masses (its gravitational length in `r_g`) and its radius in `r_g`, the
/// form of a star's compactness (`disc::rim_share`). `None` for a world with
/// no element mass.
pub fn world_tilt(w: &World) -> Option<Q> {
    let table = crate::elements::table().ok()?;
    let (protons, mass) =
        w.elements
            .iter()
            .zip(&table)
            .fold((Q::ZERO, Q::ZERO), |(p, m), (share, el)| {
                let per = Q::int(i64::from(el.z))
                    .checked_div(&el.a)
                    .unwrap_or(Q::ZERO);
                (&p + &(share * &per), &m + share)
            });
    let proton_share = protons.checked_div(&mass)?;
    let m = (&w.mass * &Si::world_mass_kg()).checked_div(&crate::package::core_kg())?;
    let r = (&w.radius * &Si::world_radius_m()).checked_div(&crate::chain::units().r_g_m)?;
    let delta = xyl_core::shares::deficit(&m, &r)?;
    xyl_core::hopf::tilt_equilibrium(&xyl_core::hopf::open_charge(&proton_share, &delta)?)
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
        // Stand-in S040: A world with no neighbour takes an eccentricity cap of 9/10.
        let cap = gaps
            .iter()
            .min()
            .cloned()
            .map(|x| &x * &xyl_core::ratio!(1, 2))
            .unwrap_or(xyl_core::ratio!(9, 10));
        // The orbit's shape comes from the settling (systems::settle_about).
        // The birth eccentricity is the settled swing, capped by the gap to
        // the nearest neighbour; the inclination is the swing's deficit
        // angle, swing / (1 + swing) of a quarter turn; node and periapsis
        // are both the settled phase.
        let e0 = w.swing.clone().min(cap);
        let inc = &(&pi_q() * &xyl_core::ratio!(1, 2))
            * &w.swing
                .checked_div(&(&Q::ONE + &w.swing))
                .unwrap_or(Q::ZERO);
        let node = w.phase.clone();
        let peri = w.phase.clone();
        // The axis tilt is the balance of the world's charge (#183), no longer
        // a draw; the draw it took is still read, so every later draw of the
        // world's stream stands where it stood.
        let _ = rng.next_u64();
        let obliquity = world_tilt(w);
        // Stand-in S039: A world with no tilt read stands upright, cos(obliquity) = 1.
        let cos_obliquity = obliquity
            .as_ref()
            .and_then(|eps| xyl_core::rotor::circle_point(eps, 40))
            .map_or(Q::ONE, |(c, _)| c);
        // Stand-in S038: A world's birth spin is (0.1 + 0.4 u) of break-up.
        let share = &xyl_core::ratio!(1, 10)
            + &(&xyl_core::ratio!(4, 10) * &Dy::unit(rng.next_u64()).to_q());
        let icy = w.icy();
        let (m, r) = (
            &w.mass * &Si::world_mass_kg(),
            &w.radius * &Si::world_radius_m(),
        );
        let breakup = rad_s(&m, &r);
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
            obliquity,
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
                    f = f.max(&(&xyl_core::ratio!(5, 4) * &ratio) * &out[k].e0);
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
