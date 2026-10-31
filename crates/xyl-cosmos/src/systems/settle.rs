//! The settling of a disc's bodies about their anchor, in closed form in
//! ages.

use super::*;

/// A small body of a disc on a circular orbit: the reach `a` (AU, or host
/// radii for a moon), the mass `m` (world masses) and the phase (turns).
#[derive(Clone, Debug)]
pub(super) struct Embryo {
    pub(super) a: Q,
    pub(super) m: Q,
    pub(super) phase: Q,
    /// The largest swing the neighbours impart by the settled age, as a share
    /// of the body's reach (settle_about); it sets the starting eccentricity
    /// and tilt.
    pub(super) swing: Q,
    /// The age (Gyr) at which the body merges into its heavier neighbour;
    /// `None` for a body that persists to the horizon.
    pub(super) merged_at: Option<Q>,
}

/// The core units used by the settling: r_g in metres and the core's mass in
/// kilograms.
pub(super) fn rg_units() -> (Dy, Dy) {
    let r_g = Dy::from_q(&crate::chain::units().r_g_m);
    let core_kg = Dy::from_q(&(&crate::package::hole_mass() * &crate::package::Si::star_mass_kg()));
    (r_g, core_kg)
}

/// A planet's reach a (AU) in r_g.
pub(super) fn planet_reach(a: &Q) -> Dy {
    Dy::from_q(a) * Dy::from_q(&crate::package::Si::au()) / rg_units().0
}

/// Settles a star's disc (see [`settle_about`]) with the star as anchor:
/// swings at the settled age, merge ages out to life's horizon.
pub(super) fn settle(b: Vec<Embryo>, star_suns: &Q, settled: &Q) -> Vec<Embryo> {
    let (_, core_kg) = rg_units();
    let anchor = Dy::from_q(star_suns) * Dy::from_q(&crate::package::Si::star_mass_kg()) / core_kg;
    settle_about(
        b,
        anchor,
        &planet_reach,
        settled,
        &crate::life::horizon_gyr().to_q(),
    )
}

/// The smallest period q of the phase step `s` that `n` meetings can tell
/// apart: the first continued-fraction convergent p/q of `s` with
/// |s - p/q| n q <= 1.
fn period(s: Dy, n: Dy) -> i64 {
    let (mut x, mut h0, mut h1, mut k0, mut k1) = (s, 0i64, 1i64, 1i64, 0i64);
    for _ in 0..48 {
        let a = x.trunc_i64();
        (h0, h1) = (h1, a.saturating_mul(h1).saturating_add(h0));
        (k0, k1) = (k1, a.saturating_mul(k1).saturating_add(k0));
        if k1 > 0 && (s - Dy::int(h1) / Dy::int(k1)).abs() * n * Dy::int(k1) <= Dy::ONE {
            return k1;
        }
        let fr = x - Dy::int(a);
        if fr <= Dy::sci(1, -15) {
            return k1.max(1);
        }
        x = Dy::ONE / fr;
    }
    k1.max(1)
}

/// The pair laws of one settling (see [`settle_about`]), in r_g units.
struct Pairs<'r> {
    /// The anchor's mass.
    anchor: Dy,
    /// A body's reach in r_g from its held reach.
    reach: &'r dyn Fn(&Q) -> Dy,
    /// One world mass.
    unit_world: Dy,
    /// The light-travel length of one Gyr in r_g, converting ages to spans.
    per_gyr: Dy,
    /// The horizon (Gyr) merge ages are searched out to.
    top: Dy,
}

impl Pairs<'_> {
    /// The turning law omega(A) = [M / (A + M)] / A.
    fn omega(&self, a: Dy) -> Dy {
        (self.anchor / (a + self.anchor)) / a
    }

    /// A pair's kick over `gyr`: `eps (floor(N / q) c + N mod q)`, the
    /// largest swing as a share of the inner reach; 1e30 for a pair at the
    /// same reach and 0 for a pair at the same turning rate.
    fn kick(&self, x: &Embryo, y: &Embryo, gyr: Dy) -> Dy {
        let (a1, a2) = ((self.reach)(&x.a), (self.reach)(&y.a));
        let r = (a2 - a1).abs();
        if r.is_zero() {
            return Dy::sci(1, 30);
        }
        let mm = (Dy::from_q(&x.m) + Dy::from_q(&y.m)) * self.unit_world;
        let (w1, w2) = (self.omega(a1), self.omega(a2));
        let dw = (w1 - w2).abs();
        if dw.is_zero() {
            return Dy::ZERO;
        }
        let eps = (mm / (r + mm)) / r / dw;
        let n = gyr * self.per_gyr * dw;
        let s = w1.max(w2) / dw;
        let s = s - Dy::int(s.trunc_i64());
        let q = period(s, n.max(Dy::ONE));
        let off = Dy::int(q) * Dy::from_q(&(&y.phase - &x.phase));
        let mut f = off - Dy::int(off.trunc_i64());
        if f.signum() < 0 {
            f += Dy::ONE;
        }
        let c = (Dy::ONE - Dy::int(2) * f).abs();
        let whole = Dy::int((n / Dy::int(q)).trunc_i64());
        let rest = n - whole * Dy::int(q);
        eps * (whole * c + rest)
    }

    /// The pair's swing (kick times inner reach) over the gap; the pair
    /// merges once this reaches 1.
    fn score(&self, x: &Embryo, y: &Embryo, gyr: Dy) -> Dy {
        let (a1, a2) = ((self.reach)(&x.a), (self.reach)(&y.a));
        let r = (a2 - a1).abs();
        if r.is_zero() {
            return Dy::int(2);
        }
        self.kick(x, y, gyr) * a1.min(a2) / r
    }

    /// The first age at which the pair merges, by 48 bisection steps out to
    /// the horizon; `None` where it does not merge by then.
    fn merge_age(&self, x: &Embryo, y: &Embryo) -> Option<Dy> {
        if self.score(x, y, self.top) < Dy::ONE {
            return None;
        }
        let (mut lo, mut hi) = (Dy::ZERO, self.top);
        for _ in 0..48 {
            let mid = (lo + hi) / Dy::int(2);
            if self.score(x, y, mid) >= Dy::ONE {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        Some(hi)
    }
}

/// The settling about an anchor (a star, or a world for its moons), with the
/// anchor's mass and each body's reach as lengths in r_g (G = c = M_core = 1).
///
/// Each body turns at omega(A) = [M / (A + M)] / A. Neighbours meet once per
/// synodic period; each meeting turns them by
/// eps = [m / (r + m)] / r / |omega_i - omega_j|, and over an age they meet
/// N = age |omega_i - omega_j| times. The phase step per meeting,
/// s = frac(omega_i / |omega_i - omega_j|), is approximated by the smallest
/// continued-fraction convergent p/q with |s - p/q| <= 1 / (N q), so kicks
/// repeat every q meetings and a full period cancels to
/// c = |1 - 2 frac(q psi_0)| (psi_0 the pair's phase offset). The greatest
/// swing (the kick) is eps (floor(N / q) c + N mod q), a share of the inner
/// reach.
///
/// The settling is carried as ages, in closed form: a pair's merge age is
/// the first age at which its kick times the inner reach spans the gap
/// (found by bisection out to `horizon`; never, where it does not by then).
/// The earliest pair merges first: the lighter body ends at that age (its
/// `merged_at`) and the heavier carries on with the summed mass, reach and
/// phase mass-weighted; the merged body's pairs are read again with the
/// merged mass, so a merger that now spans a further gap merges again.
/// Resonances (small q) merge, near-incommensurate pairs remain. Merges
/// before `settled` are applied; the bodies standing at `settled` are
/// returned, and those that merge after it carry their merge age. Each
/// body's `swing` is the largest kick its neighbours impart by `settled`, as
/// a share of the body's reach, capped at 1.
pub(super) fn settle_about(
    mut b: Vec<Embryo>,
    anchor: Dy,
    reach: &dyn Fn(&Q) -> Dy,
    settled: &Q,
    horizon: &Q,
) -> Vec<Embryo> {
    let (r_g, core_kg) = rg_units();
    let pairs = Pairs {
        anchor,
        reach,
        unit_world: Dy::from_q(&crate::package::Si::world_mass_kg()) / core_kg,
        per_gyr: Dy::from_q(&crate::package::Si::gyr_s()) * Dy::from_q(&crate::package::Si::c())
            / r_g,
        top: Dy::from_q(horizon),
    };
    let ages = merge_until(&mut b, &pairs, Dy::from_q(settled));
    set_swings(&mut b, &pairs, Dy::from_q(settled));
    // Pairs that merge after the settled age remain two bodies (no mass
    // moves); the lighter body records the age, the earlier of two when it
    // has two neighbours.
    for (i, age) in ages.iter().enumerate() {
        let Some(t) = *age else { continue };
        let lighter = if b[i].m >= b[i + 1].m { i + 1 } else { i };
        let when = held(&t.max(Dy::ZERO).to_q());
        let e = &mut b[lighter];
        if e.merged_at.as_ref().is_none_or(|m| &when < m) {
            e.merged_at = Some(when);
        }
    }
    b
}

/// Applies every merge at or before `by_settled`, earliest first: the
/// heavier body carries the summed mass and the mass-weighted reach and
/// phase, and the merged body's pairs are read again. Returns the remaining
/// pairs' merge ages, one per neighbouring pair of `b`.
fn merge_until(b: &mut Vec<Embryo>, pairs: &Pairs, by_settled: Dy) -> Vec<Option<Dy>> {
    let mut ages: Vec<Option<Dy>> = b
        .windows(2)
        .map(|w| pairs.merge_age(&w[0], &w[1]))
        .collect();
    while let Some((i, _)) = ages
        .iter()
        .enumerate()
        .filter_map(|(i, a)| a.map(|t| (i, t)))
        .filter(|(_, t)| *t <= by_settled)
        .min_by(|x, y| x.1.cmp(&y.1))
    {
        let (x, y) = (b[i].clone(), b[i + 1].clone());
        let m = &x.m + &y.m;
        let by = |u: &Q, v: &Q| {
            held(
                &(&(u * &x.m) + &(v * &y.m))
                    .checked_div(&m)
                    .unwrap_or_else(|| u.clone()),
            )
        };
        b[i] = Embryo {
            a: by(&x.a, &y.a),
            m: held(&m),
            phase: by(&x.phase, &y.phase).frac(),
            swing: Q::ZERO,
            merged_at: None,
        };
        b.remove(i + 1);
        ages.remove(i);
        if i < ages.len() {
            ages[i] = pairs.merge_age(&b[i], &b[i + 1]);
        }
        if i > 0 {
            ages[i - 1] = pairs.merge_age(&b[i - 1], &b[i]);
        }
    }
    ages
}

/// Sets each body's swing at age `at`: the largest kick its neighbours
/// impart, as a share of the body's reach, capped at 1.
fn set_swings(b: &mut [Embryo], pairs: &Pairs, at: Dy) {
    let swings: Vec<Q> = (0..b.len())
        .map(|i| {
            let mut s = Dy::ZERO;
            for j in [i.checked_sub(1), Some(i + 1)].into_iter().flatten() {
                if j >= b.len() {
                    continue;
                }
                let (x, y) = if i < j {
                    (&b[i], &b[j])
                } else {
                    (&b[j], &b[i])
                };
                let inner = (pairs.reach)(&x.a).min((pairs.reach)(&y.a));
                let own = (pairs.reach)(&b[i].a).max(Dy::sci(1, -30));
                s = s.max(pairs.kick(x, y, at) * inner / own);
            }
            held(&s.min(Dy::ONE).to_q())
        })
        .collect();
    for (e, sw) in b.iter_mut().zip(swings) {
        e.swing = sw;
    }
}
