//! The founding's release: the universal core's bounces (T17) release its
//! open kinds cohort by cohort, and each released layer is read at its
//! cohort's pinch: its pole reach on the axis plane, its place about the
//! throat by the reach rate, and its rest point at the far face (see the
//! parent module's header for the formulas).

use xyl_core::dyadic::Dy;
use xyl_core::shares::{FacePairs, Parcel};
use xyl_core::Q;

/// A founded kind on the core at the bounce, before it leaves: its hold in
/// the founding's units (the founded hold is 1), its pole point's
/// logarithm `ln t` with `t = U / W_e` (zero for a closed kind, which has
/// none), and its axis polarity `A`.
#[derive(Clone, Debug)]
struct Layer {
    z: u32,
    hold: Q,
    ln_t: Dy,
    polarity: Dy,
}

/// One cohort of the core's spin orbit (T17).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cohort {
    /// Echoes after the bounce at which it leaves.
    pub echoes: Dy,
    /// Whether it leaves by the far face.
    pub far: bool,
    /// Its hold, in the founding's units (the founded hold is 1).
    pub hold: Q,
    /// Its share of the turning against its share of the hold: its
    /// separation over the throat's, `rho_cohort / rho_hold`.
    pub share: Q,
    /// The core's spin once it has left.
    pub spin: Q,
    /// The pinch interval it leaves in, `N* = 3 (4 + a) / (2a)` echoes at
    /// the core's spin before it leaves.
    pub interval: Dy,
    /// Its layers: each kind, its hold and its pole reach's logarithm
    /// `ln r` on the axis plane as it leaves (at least zero: past the
    /// throat on the near face, inverted on the far).
    pub layers: Vec<(u32, Q, Dy)>,
}

/// The core's spin orbit (T17): its cohorts, what it keeps and its spin
/// once they have left. The pair closes on both faces where the spin is at
/// most one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpinOrbit {
    /// The cohorts, in the order they leave.
    pub cohorts: Vec<Cohort>,
    /// The kinds the core keeps once the last cohort has left, with their
    /// holds: the closed kinds and any open kind that never leaves.
    pub kept: Vec<(u32, Q)>,
    /// The spin once the last cohort has left.
    pub spin: Q,
    /// The founded hold `M_0`, in the founding's units (1).
    pub founded: Q,
    /// The hold once the last cohort has left.
    pub held: Q,
    /// The turning once the last cohort has left, `a M^2`.
    pub turning: Q,
    /// The remanence (T19): the turning held beyond the founded turning per
    /// unit hold, `J - (a_0 M_0) M`.
    pub remanence: Q,
}

/// A founding shell: a kind's matter as it stands at its cohort's pinch
/// (or, kept, at the throat): its amount (held count), its reach about the
/// throat (founding units, the core's hold 1), its swing (the band spreads
/// `reach (1 +/- swing)`), and whether the core keeps it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shell {
    /// The kind's atomic number.
    pub z: u32,
    /// Its amount in the held count, as in the mix.
    pub amount: Q,
    /// Its reach about the throat, founding units.
    pub reach: Q,
    /// Its swing, the deficit `H / (R + H)` of the hold inside it at its
    /// reach: the reverberation's radial excursion.
    pub swing: Q,
    /// Whether the core keeps it (the firmament).
    pub kept: bool,
}

/// A kind's mass at capture per unit of its held count: `Z / W_e`, its
/// protons over its electron hold. The bounce captures charge first
/// (`Q0 = 0`); the neutrons are the rest point, read after the release.
fn capture_mass(z: u32) -> Option<Q> {
    let w_e = crate::compound::electron_hold(z);
    (w_e > 0).then(|| Q::new(i64::from(z), i64::from(w_e)))?
}

/// The founded kinds of `mix` as layers on the core at the bounce, each
/// hold its capture mass over the founded total (so the founded hold is
/// 1). The polarity is read on the axis plane, filled from both sides at
/// the kind's capture phase in the common phase (#201).
fn layers(mix: &[(u32, Q)], share: &Q) -> Vec<Layer> {
    let massed: Vec<(u32, Q)> = mix
        .iter()
        .filter_map(|(z, a)| Some((*z, a * &capture_mass(*z)?)))
        .filter(|(_, m)| m.signum() > 0)
        .collect();
    let total = massed.iter().fold(Q::ZERO, |s, (_, m)| &s + m);
    massed
        .into_iter()
        .filter_map(|(z, m)| {
            let hold = m.checked_div(&total)?;
            let (u, w_e) = (crate::compound::open(z), crate::compound::electron_hold(z));
            if u == 0 {
                return Some(Layer {
                    z,
                    hold,
                    ln_t: Dy::ZERO,
                    polarity: Dy::ZERO,
                });
            }
            let t = Dy::int(i64::from(u)).div(&Dy::int(i64::from(w_e)))?;
            let from_q = crate::circuit::fill_shares(z, share, &xyl_core::hopf::COMMON);
            let [axis, ..] = crate::atoms::plane_polarities(z, &from_q)?;
            Some(Layer {
                z,
                hold,
                ln_t: t.ln()?,
                polarity: Dy::from_q(&axis),
            })
        })
        .collect()
}

/// The core's spin as a sheared orbit over its bounces (T17), from the
/// founded kinds of `mix` at the parent's `spin`.
///
/// At the bounce every founded kind is at the core, of hold `M` and
/// turning `J = a M^2`. A kind's states stand on both faces: its open
/// share `q = U / (U + W)` on the near face, which advances `s` per
/// traversal, its held share on the antipode, which advances `1 - s`.
/// Its separation is their flow-law sum, the rapidity
/// `rho = q ln D(s) - (1 - q) ln D(1 - s)`; the far face exchanges `q`
/// and `1 - q`. A cohort of hold `m` takes the turning in proportion to
/// its separation against the throat's present hold,
/// `J (m / M) (rho_cohort / rho_hold)`, so releasing everything takes
/// `J` whole (axiom 11), and the spin after it is `a' = J' / M'^2`.
///
/// A closed kind is held on both faces (T16). An open kind of axis
/// polarity `A` is dilated per echo by `D(s) D(n)^A` on the near face;
/// where that is at most one the near face holds it and the far face
/// releases it at once, inverted (`ln r = -ln t`). Every other open kind
/// climbs to the throat in closed form, `ln t += N* ln(D(s) D(n)^A)` per
/// bounce interval `N* = 3 (4 + a) / (2a)`, and leaves in the cohort of
/// the first bounce after it arrives, at the pole reach it has climbed to
/// (at the core the folds' meeting is a bounce through the far face, never
/// a pinch). The bounces are events found in closed form; the interval
/// moves with the spin. Logarithms and the separations are dyadic
/// readings, the holds exact, and each cohort's share of the turning is
/// gridded back to `Q`. A spinless core never bounces a cohort out.
pub fn spin_orbit(mix: &[(u32, Q)], spin: &Q) -> SpinOrbit {
    let share = crate::bounce::torsion_share(&Q::ONE, spin);
    let mut kinds = layers(mix, &share);
    let mut hold = kinds.iter().fold(Q::ZERO, |m, k| &m + &k.hold);
    let founded = hold.clone();
    let mut a = spin.clone();
    let mut turning = &(&a * &hold) * &hold;
    let mut cohorts = Vec::new();
    let mut echoes = Dy::ZERO;
    let open = |k: &Layer| crate::compound::open(k.z) > 0;
    while a.signum() > 0 && a <= Q::ONE {
        let s = crate::bounce::torsion_share(&Q::ONE, &a);
        let (Some(near), Some(anti)) = (
            Dy::from_q(&(&Q::int(3) + &a)).ln(),
            Dy::from_q(&(&Q::int(2) - &s).checked_div(&s).unwrap_or(Q::ZERO)).ln(),
        ) else {
            break;
        };
        let lnn = Dy::from_q(&(&Q::ONE + &(&a * &xyl_core::ratio!(1, 2))))
            .ln()
            .unwrap_or(Dy::ZERO);
        let interval = Dy::from_q(&(&Q::int(3) * &(&Q::int(4) + &a)))
            .div(&Dy::from_q(&(&Q::int(2) * &a)))
            .unwrap_or_default();
        let separation = |z: u32, far: bool| {
            let (u, w_e) = (crate::compound::open(z), crate::compound::electron_hold(z));
            let q = Dy::int(i64::from(u))
                .div(&Dy::int(i64::from(u + w_e)))
                .unwrap_or_default();
            let q = if far { Dy::ONE - q } else { q };
            q * near - (Dy::ONE - q) * anti
        };
        let spread = |members: &[Layer], far: bool| {
            members.iter().fold(Dy::ZERO, |r, k| {
                r + Dy::from_q(&k.hold) * separation(k.z, far)
            })
        };
        let rate = |k: &Layer| near + lnn * k.polarity;
        // The near face holds by pressure what the far face releases.
        let pressed = |k: &Layer| open(k) && rate(k) <= Dy::ZERO;
        let (out, far, at) = if cohorts.is_empty() && kinds.iter().any(pressed) {
            let (out, stay): (Vec<_>, Vec<_>) = kinds.drain(..).partition(pressed);
            kinds = stay;
            // The far face inverts the pole point, t -> 1 / t.
            let out = out
                .into_iter()
                .map(|k| Layer { ln_t: -k.ln_t, ..k })
                .collect::<Vec<_>>();
            (out, true, Dy::ZERO)
        } else {
            // The bounce at which each open kind arrives: the least whole
            // k with k N* ln(D(s) D(n)^A) >= -ln t.
            let arrive = |k: &Layer| {
                let r = rate(k);
                if !open(k) || r.signum() <= 0 {
                    return None;
                }
                let need = ((-k.ln_t).div(&(interval * r)).unwrap_or_default()).to_q();
                let whole = i64::try_from(-(-&need).floor_i128()?).ok()?;
                Some(whole.max(1))
            };
            let Some(first) = kinds.iter().filter_map(arrive).min() else {
                break;
            };
            echoes += interval * Dy::int(first);
            let climbed = interval * Dy::int(first);
            let (out, stay): (Vec<_>, Vec<_>) =
                kinds.drain(..).partition(|k| arrive(k) == Some(first));
            let climb = |k: Layer| {
                let ln_t = if k.hold.signum() > 0 && open(&k) {
                    k.ln_t + climbed * rate(&k)
                } else {
                    k.ln_t
                };
                Layer { ln_t, ..k }
            };
            kinds = stay.into_iter().map(climb).collect();
            (out.into_iter().map(climb).collect(), false, echoes)
        };
        let all: Vec<Layer> = out.iter().chain(kinds.iter()).cloned().collect();
        let m = out.iter().fold(Q::ZERO, |s, k| &s + &k.hold);
        // The cohort's separation over the hold's, each mass-weighted.
        let per = |sum: Dy, of: &Q| sum.div(&Dy::from_q(of)).unwrap_or_default();
        let share = (per(spread(&out, far), &m)
            .div(&per(spread(&all, false), &hold))
            .unwrap_or_default())
        .to_q();
        let taken = &(&turning * &m.checked_div(&hold).unwrap_or(Q::ZERO)) * &share;
        turning = &turning - &xyl_core::certified::toward_zero(&taken, 64);
        hold = &hold - &m;
        a = turning.checked_div(&(&hold * &hold)).unwrap_or(Q::ZERO);
        cohorts.push(Cohort {
            echoes: at,
            far,
            hold: m,
            share,
            spin: a.clone(),
            interval,
            layers: out
                .into_iter()
                .map(|k| (k.z, k.hold, k.ln_t.max(Dy::ZERO)))
                .collect(),
        });
    }
    // T19: the turning held beyond the founded turning per unit hold.
    let remanence = &turning - &(&(spin * &founded) * &hold);
    SpinOrbit {
        cohorts,
        kept: kinds.into_iter().map(|k| (k.z, k.hold)).collect(),
        spin: a,
        founded,
        held: hold,
        turning,
        remanence,
    }
}

/// The reach about the throat a parcel of hold `m` stands at after `tau`
/// echoes, released at `r0` with the hold `h` inside it: the reach rate
/// `dr/dtau = [R_h(h) - R_h(m)] r / (r + R_h(m))` with `R_h(M) = 2M`,
/// separable and exact, `(r - r0) + 2m ln(r / r0) = 2 (h - m) tau`, solved
/// for `r` (its left side rises with `r`, so the root is one, found by
/// halving to 10^-20 of `r0`). It expands where `h > m`, rests where
/// `h = m` and returns toward the throat where `m > h`. `None` for a
/// reach, hold or time that is not positive.
fn reach_after(r0: &Q, m: &Q, h: &Q, tau: Dy) -> Option<Q> {
    if r0.signum() <= 0 || m.signum() <= 0 || h.signum() <= 0 || tau.signum() <= 0 {
        return None;
    }
    let (r0d, md) = (Dy::from_q(r0), Dy::from_q(m));
    let rhs = Dy::int(2) * (Dy::from_q(h) - md) * tau;
    if rhs.signum() == 0 {
        return Some(r0.clone());
    }
    let left = |r: Dy| -> Option<Dy> { Some((r - r0d) + Dy::int(2) * md * r.div(&r0d)?.ln()?) };
    // The root lies within r0 + rhs outward (the logarithm adds), within
    // (0, r0) inward.
    let (mut lo, mut hi) = if rhs.signum() > 0 {
        (r0d, r0d + rhs)
    } else {
        (r0d * Dy::sci(1, -300), r0d)
    };
    for _ in 0..256 {
        let mid = (lo + hi).div(&Dy::int(2))?;
        if left(mid)? < rhs {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo <= r0d * Dy::sci(1, -20) {
            break;
        }
    }
    Some(xyl_core::certified::toward_zero(&lo.to_q(), 64))
}

/// A layer's rest point on the far face: the inversion of its pole reach,
/// `Q* / Z = 1 / r = exp(-ln r)`, gridded toward zero at 2^-64; `None`
/// where it grids to zero.
fn rest_point(ln_r: Dy) -> Option<FacePairs> {
    let t = xyl_core::certified::toward_zero(&(-ln_r).exp().to_q(), 64);
    FacePairs::of(&t)
}

/// The founding shells of `mix` (each kind's amount in the held count) at
/// the parent's `spin`: the core's spin orbit releases its open kinds
/// cohort by cohort ([`spin_orbit`]); a cohort leaves from the throat's
/// horizon `R_h(H) = 2H`, `H` the hold the core keeps after it, and each
/// layer stands after its cohort's pinch interval at the reach the reach
/// rate carries it to ([`reach_after`]), with the hold inside it the
/// core's and its heavier companions' (the lighter run ahead). The kinds
/// the core keeps stand at its horizon `R_h(1) = 2`. Each shell swings by
/// the deficit at its reach of the hold inside it. The universal table
/// holds each released kind's rest point at the inversion of its pole
/// reach ([`rest_point`]); a kept kind has none (`Q* = 0`, its capture
/// state). Returns the shells and the table.
pub fn founding_shells(mix: &[(u32, Q)], spin: &Q) -> (Vec<Shell>, crate::circuit::Table) {
    let orbit = spin_orbit(mix, spin);
    let amount_of = |z: u32| {
        mix.iter()
            .find(|(k, _)| *k == z)
            .map(|(_, a)| a.clone())
            .unwrap_or(Q::ZERO)
    };
    let two = Q::int(2);
    let mut shells = Vec::new();
    let mut table = crate::circuit::Table::new();
    let mut kept_hold = orbit.founded.clone();
    for c in &orbit.cohorts {
        kept_hold = &kept_hold - &c.hold;
        let r0 = &two * &kept_hold;
        let mut layers = c.layers.clone();
        layers.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
        let mut inside = kept_hold.clone();
        for (z, m, ln_r) in layers {
            let h = &inside + &m;
            inside = h.clone();
            let Some(reach) = reach_after(&r0, &m, &h, c.interval) else {
                continue;
            };
            let amount = amount_of(z);
            if let Some(pairs) = rest_point(ln_r) {
                table.entry(z).or_default().push(Parcel {
                    amount: amount.clone(),
                    pairs,
                });
            }
            let swing = h.checked_div(&(&reach + &h)).unwrap_or(Q::ZERO);
            shells.push(Shell {
                z,
                amount,
                reach,
                swing,
                kept: false,
            });
        }
    }
    let horizon = crate::bounce::horizon_reach(&Q::ONE);
    let swing = Q::ONE.checked_div(&(&horizon + &Q::ONE)).unwrap_or(Q::ZERO);
    for (z, _) in &orbit.kept {
        shells.push(Shell {
            z: *z,
            amount: amount_of(*z),
            reach: horizon.clone(),
            swing: swing.clone(),
            kept: true,
        });
    }
    (shells, table)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reach rate's closed form: a parcel lighter than its hold
    /// expands, one as heavy rests and one heavier returns, and the root
    /// satisfies `(r - r0) + 2m ln(r / r0) = 2 (h - m) tau` to the grid.
    #[test]
    fn the_reach_rate_expands_rests_and_returns() {
        let (r0, tau) = (Q::int(2), Dy::int(19));
        let (light, heavy, h) = (xyl_core::ratio!(1, 10), Q::int(2), Q::ONE);
        let out = reach_after(&r0, &light, &h, tau).unwrap();
        let back = reach_after(&r0, &heavy, &h, tau).unwrap();
        assert!(out > r0 && back < r0, "{out} {back}");
        assert_eq!(reach_after(&r0, &h, &h, tau), Some(r0.clone()));
        let r = Dy::from_q(&out);
        let left = (r - Dy::from_q(&r0))
            + Dy::int(2) * Dy::from_q(&light) * r.div(&Dy::from_q(&r0)).unwrap().ln().unwrap();
        let rhs = Dy::int(2) * Dy::from_q(&(&h - &light)) * tau;
        assert!((left - rhs).abs() < Dy::sci(1, -12), "{left} {rhs}");
        assert_eq!(reach_after(&Q::ZERO, &light, &h, tau), None);
    }

    /// The orbit's books close: the cohorts' holds and the kept holds sum
    /// to the founded hold, every closed kind is kept (T16), and every
    /// released layer leaves past the throat or inverted (`ln r >= 0`), so
    /// its rest point is at most one.
    #[test]
    fn the_orbit_releases_what_the_core_does_not_keep() {
        let mix: Vec<(u32, Q)> = (1..=120u32)
            .map(|z| (z, Q::new(1, i64::from(z)).unwrap()))
            .collect();
        let orbit = spin_orbit(&mix, &xyl_core::ratio!(1, 2));
        let left = orbit.cohorts.iter().fold(Q::ZERO, |s, c| &s + &c.hold);
        let kept = orbit.kept.iter().fold(Q::ZERO, |s, k| &s + &k.1);
        assert_eq!(&left + &kept, orbit.founded);
        assert_eq!(orbit.founded, Q::ONE);
        for c in &orbit.cohorts {
            for (z, _, ln_r) in &c.layers {
                assert!(crate::compound::open(*z) > 0, "Z {z} is open");
                assert!(ln_r.signum() >= 0, "Z {z}");
            }
        }
        for (z, _) in mix.iter().filter(|(z, _)| crate::compound::open(*z) == 0) {
            assert!(orbit.kept.iter().any(|k| k.0 == *z), "closed Z {z} is kept");
        }
        let (shells, table) = founding_shells(&mix, &xyl_core::ratio!(1, 2));
        for (z, parcels) in &table {
            for p in parcels {
                assert!(p.pairs.value() <= Q::ONE, "Z {z}");
            }
        }
        let all = shells.iter().fold(Q::ZERO, |s, k| &s + &k.amount);
        let mixed = mix.iter().fold(Q::ZERO, |s, k| &s + &k.1);
        assert_eq!(all, mixed, "every kind stands in one shell");
    }

    /// A spinless core never bounces a cohort out: everything is kept.
    #[test]
    fn a_spinless_core_keeps_everything() {
        let mix = [(1u32, Q::ONE), (26, Q::ONE)];
        let orbit = spin_orbit(&mix, &Q::ZERO);
        assert!(orbit.cohorts.is_empty());
        assert_eq!(orbit.kept.len(), 2);
    }
}
