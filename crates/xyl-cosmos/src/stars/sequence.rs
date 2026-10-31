//! Stellar transduction by polarity: the held collective's pole point, its
//! dilation by the star's irradiation, the fusion and fission steps its
//! kinds take and the sequence they form (docs/model.md, Polarity, fusion
//! and fission).
//!
//! Reads `elements::table` (each kind's nucleon count `A_z`, the table's `a`
//! rounded to the nearest whole) and `compound::open_hot` and
//! `compound::charge` (a kind's open frames at heat and its charge). Heat is
//! the turning the star receives per echo, in hydrogen units
//! (`atoms::unpaired_hot`); shares are shares of the star's hold; echoes are
//! counted from ignition.
//!
//! - Frames at heat: a kind `z` keeps `u_z(q)` open frames (its Q pole) and
//!   holds `w_z(q) = 3 Z - u_z(q)` (its Z pole).
//! - Pole point (`pole_point`): over the held mixture, with `n_z = share_z /
//!   w_z` nuclei of each kind, `t0 = sum n_z u_z / sum n_z w_z`, at most 1/2.
//! - Throat (`throat_echoes`): each echo dilates the pole point by
//!   `rho_s = (1 + delta) / (1 - delta)`, so it reaches the throat after
//!   `n* = ln(1 / t0) / ln rho_s` echoes.
//! - Fusion step (`fusion_step`): `k` nuclei of `z` become one of the kind
//!   at `A' = k A_z`, for the smallest `k >= 2` at which some kind releases
//!   hold, `1 - w' / (k w) > 0` (the largest release among the kinds at that
//!   `A'`).
//! - Fission step (`fission_step`): one nucleus of `z` becomes two of kinds
//!   with `A1 + A2 = A_z`, the split nearest equal halves that releases hold,
//!   `1 - (w1 + w2) / w > 0` (the largest release at that split).
//! - Sequence (`Sequence::of`): no kind, product or order is set; the
//!   sequence emerges from the held mixture. At each stage the held kind
//!   whose step has the greatest release steps (ties by the lesser nucleon
//!   count), its products join the mixture, and the next stage is read from
//!   the updated mixture, until no held kind releases. Fusion stages come
//!   first (the collective starts on the Z side, `t0 <= 1/2`); where the
//!   pole point can reach the throat (`t0 > 0`) the fission stages follow.
//!   Each step transduces its share times its release; the remnant is one
//!   less the steps' cumulative transduced share.
//!
//! Invariants: every step conserves the nucleon count (`k A_z = A'`,
//! `A1 + A2 = A_z`); cumulative transduced plus remnant is one. A cluster
//! kind (`compound::is_cluster`) enters the pole point but takes no step,
//! since the element table gives it no nucleon count. Terms: pole point of a
//! star, fusion step, fission step, sequence (docs/glossary.md).

use std::collections::BTreeMap;
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// The element table's nucleon counts: `A_z` by kind and the kinds at each
/// `A`, built once per process. Empty where the table does not build.
struct Nucleons {
    of: BTreeMap<u32, u32>,
    at: BTreeMap<u32, Vec<u32>>,
}

fn nucleons() -> &'static Nucleons {
    static INDEX: std::sync::OnceLock<Nucleons> = std::sync::OnceLock::new();
    INDEX.get_or_init(|| {
        let mut index = Nucleons {
            of: BTreeMap::new(),
            at: BTreeMap::new(),
        };
        for e in crate::elements::table().unwrap_or_default() {
            // `A_z = floor(a + 1/2)`, the nearest whole count of nucleons.
            let Some(a) = (&e.a + &Q::ratio(1, 2)).floor_i128() else {
                continue;
            };
            let Ok(a) = u32::try_from(a) else { continue };
            if a == 0 {
                continue;
            }
            index.of.insert(e.z, a);
            index.at.entry(a).or_default().push(e.z);
        }
        index
    })
}

/// A kind's nucleon count `A_z`, the element table's `a` rounded to the
/// nearest whole; `None` for a cluster or a kind outside the table.
pub fn nucleon_count(z: u32) -> Option<u32> {
    nucleons().of.get(&z).copied()
}

/// A kind's open frames at `heat` (hydrogen units), `u_z(q)`: its Q pole.
fn open_at(z: u32, heat: &Q) -> Q {
    Q::int(i64::from(crate::compound::open_hot(z, heat)))
}

/// A kind's held frames at `heat`, `w_z(q) = 3 Z - u_z(q)`: its Z pole,
/// never negative.
pub fn held_frames(z: u32, heat: &Q) -> Q {
    (&Q::int(3 * i64::from(crate::compound::charge(z))) - &open_at(z, heat)).max(Q::ZERO)
}

/// The collective's pole point `t0 = sum n_z u_z / sum n_z w_z` over the held
/// mixture `held` (kind, share of the hold) at `heat`, with `n_z = share_z /
/// w_z` the count of nuclei of each kind. Its inverse is the star's polarity.
/// Zero for an empty mixture or one whose kinds hold no frames.
pub fn pole_point(held: &[(u32, Q)], heat: &Q) -> Q {
    let (mut open, mut closed) = (Q::ZERO, Q::ZERO);
    for (z, share) in held {
        let w = held_frames(*z, heat);
        let Some(n) = share.checked_div(&w) else {
            continue;
        };
        open = &open + &(&n * &open_at(*z, heat));
        closed = &closed + &(&n * &w);
    }
    open.checked_div(&closed).unwrap_or(Q::ZERO)
}

/// The echoes from ignition until a pole point `t0` reaches the throat under
/// compactness `delta`: `n* = ln(1 / t0) / ln rho_s`, `rho_s = (1 + delta) /
/// (1 - delta)`, in closed form. Zero where `t0 >= 1`; `None` where `t0 = 0`
/// (the collective never dilates) or `delta` lies outside `(0, 1)`.
pub fn throat_echoes(t0: &Q, delta: Dy) -> Option<Dy> {
    if t0.signum() <= 0 || delta.signum() <= 0 || delta >= Dy::ONE {
        return None;
    }
    let t0 = Dy::from_q(t0);
    if t0 >= Dy::ONE {
        return Some(Dy::ZERO);
    }
    let rho = ((Dy::ONE + delta) / (Dy::ONE - delta)).ln()?;
    if rho.signum() <= 0 {
        return None;
    }
    Some((Dy::ONE / t0).ln()? / rho)
}

/// One step of a star's sequence: the kind that steps, the nuclei it takes,
/// the kinds it makes, and the shares it holds and transduces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// The kind that steps.
    pub kind: u32,
    /// Nuclei of `kind` that one step takes: `k` for fusion, 1 for fission.
    pub nuclei: u32,
    /// The kinds made: one for fusion, two for fission.
    pub products: Vec<u32>,
    /// Whether the step splits nuclei (the Q side of the throat).
    pub fission: bool,
    /// The kind's share of the hold when it steps: its share of the held
    /// mixture plus the hold the earlier steps passed to it.
    pub share: Q,
    /// The hold released per hold transduced: `1 - w' / (k w)` for fusion,
    /// `1 - (w1 + w2) / w` for fission.
    pub release: Q,
    /// The share of the star's hold the step transduces, `share x release`.
    pub transduced: Q,
}

/// A star's transduction sequence (see the module header). The default is the
/// empty sequence: no step, a remnant of one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sequence {
    /// The held collective's pole point at ignition, `t0` (at most 1/2).
    pub pole_point: Q,
    /// The steps in order: the fusion stages, then the fission stages, each
    /// stage the held kind of greatest release.
    pub steps: Vec<Step>,
    /// Echoes from ignition to the throat for the star's compactness
    /// ([`throat_echoes`]); `None` until set by [`Sequence::at`] or where the
    /// collective never reaches the throat.
    pub throat_echoes: Option<Dy>,
    /// The share of the hold no step transduces: one less the steps'
    /// cumulative transduced share.
    pub remnant: Q,
}

/// The fusion step of kind `z` at `heat`: the smallest `k >= 2` for which a
/// kind lies at `A' = k A_z` and releases hold `1 - w' / (k w) > 0`, with the
/// largest release among the kinds there. Returns `(k, z', release)`, or
/// `None` where no such kind exists in the table.
pub fn fusion_step(z: u32, heat: &Q) -> Option<(u32, u32, Q)> {
    let index = nucleons();
    let a = nucleon_count(z)?;
    let w = held_frames(z, heat);
    let top = *index.at.keys().next_back()?;
    for k in 2..=(top / a) {
        let Some(kinds) = index.at.get(&(k * a)) else {
            continue;
        };
        let kw = &Q::int(i64::from(k)) * &w;
        let best = kinds
            .iter()
            .filter_map(|p| {
                let release = &Q::ONE - &held_frames(*p, heat).checked_div(&kw)?;
                (release.signum() > 0).then_some((*p, release))
            })
            .max_by(|x, y| x.1.cmp(&y.1).then_with(|| y.0.cmp(&x.0)));
        if let Some((p, release)) = best {
            return Some((k, p, release));
        }
    }
    None
}

/// The fission step of kind `z` at `heat`: the split into kinds of nucleon
/// counts `A1 + A2 = A_z` nearest equal halves for which a pair releases
/// hold `1 - (w1 + w2) / w > 0`, with the largest release at that split.
/// Returns `([z1, z2], release)`, or `None` where no split releases.
pub fn fission_step(z: u32, heat: &Q) -> Option<([u32; 2], Q)> {
    let index = nucleons();
    let a = nucleon_count(z)?;
    let w = held_frames(z, heat);
    for a1 in (1..=a / 2).rev() {
        let (Some(lo), Some(hi)) = (index.at.get(&a1), index.at.get(&(a - a1))) else {
            continue;
        };
        let best = lo
            .iter()
            .flat_map(|z1| hi.iter().map(move |z2| (*z1, *z2)))
            .filter_map(|(z1, z2)| {
                let made = &held_frames(z1, heat) + &held_frames(z2, heat);
                let release = &Q::ONE - &made.checked_div(&w)?;
                (release.signum() > 0).then_some(([z1, z2], release))
            })
            .max_by(|x, y| x.1.cmp(&y.1).then_with(|| y.0.cmp(&x.0)));
        if best.is_some() {
            return best;
        }
    }
    None
}

/// The hold of the stepping kinds, keyed by `(A_z, z)` so that the order of
/// keys is the order of nucleon counts.
type Holds = BTreeMap<(u32, u32), Q>;

/// Adds `share` to kind `z`'s hold; a kind outside the table is ignored.
fn add_hold(holds: &mut Holds, z: u32, share: &Q) {
    if let Some(a) = nucleon_count(z) {
        let entry = holds.entry((a, z)).or_insert(Q::ZERO);
        *entry = &*entry + share;
    }
}

impl Default for Sequence {
    fn default() -> Sequence {
        Sequence {
            pole_point: Q::ZERO,
            steps: Vec::new(),
            throat_echoes: None,
            remnant: Q::ONE,
        }
    }
}

impl Sequence {
    /// The sequence of the held mixture `held` (kind, share; renormalised to
    /// one) at `heat` (hydrogen units): the fusion stages, each the held
    /// kind of greatest release stepping on the mixture as it stands, its
    /// products' hold `share x (1 - release)` joining the mixture; then,
    /// where `t0 > 0`, the fission stages read the same way, the products
    /// sharing the passed hold in proportion to their held frames.
    /// `throat_echoes` is left `None` ([`Sequence::at`] sets it). An empty
    /// mixture gives no step and a remnant of one.
    pub fn of(held: &[(u32, Q)], heat: &Q) -> Sequence {
        let total = held.iter().fold(Q::ZERO, |a, (_, s)| &a + s);
        let held: Vec<(u32, Q)> = held
            .iter()
            .filter_map(|(z, s)| Some((*z, s.checked_div(&total)?)))
            .filter(|(_, s)| s.signum() > 0)
            .collect();
        let pole = pole_point(&held, heat);
        let mut holds = Holds::new();
        for (z, s) in &held {
            add_hold(&mut holds, *z, s);
        }
        let mut steps = Vec::new();
        transduce(&mut holds, heat, false, &mut steps);
        if pole.signum() > 0 {
            transduce(&mut holds, heat, true, &mut steps);
        }
        let cumulative = steps.iter().fold(Q::ZERO, |a, s| &a + &s.transduced);
        Sequence {
            pole_point: pole,
            steps,
            throat_echoes: None,
            remnant: &Q::ONE - &cumulative,
        }
    }

    /// The sequence for a star of compactness `delta`: `throat_echoes` set to
    /// [`throat_echoes`] of the pole point.
    pub fn at(mut self, delta: Dy) -> Sequence {
        self.throat_echoes = throat_echoes(&self.pole_point, delta);
        self
    }

    /// The cumulative share transduced by the end of each step, in order.
    pub fn cumulative(&self) -> Vec<Q> {
        self.steps
            .iter()
            .scan(Q::ZERO, |a, s| {
                *a = &*a + &s.transduced;
                Some(a.clone())
            })
            .collect()
    }

    /// Whether some step of the held mixture releases hold: `false` for the
    /// held body that does not ignite.
    pub fn ignites(&self) -> bool {
        !self.steps.is_empty()
    }
}

/// A kind's step in one direction: its products, the hold each product
/// receives per hold passed on (in proportion to the products' held frames),
/// the nuclei taken and the release.
struct Found {
    products: Vec<u32>,
    nuclei: u32,
    release: Q,
}

/// The step kind `z` takes at `heat` in the given direction (`fission`
/// false: fusion), found from the element table by [`fusion_step`] or
/// [`fission_step`].
fn found_step(z: u32, heat: &Q, fission: bool) -> Option<Found> {
    if fission {
        fission_step(z, heat).map(|(pair, release)| Found {
            products: pair.to_vec(),
            nuclei: 1,
            release,
        })
    } else {
        fusion_step(z, heat).map(|(k, product, release)| Found {
            products: vec![product],
            nuclei: k,
            release,
        })
    }
}

/// The steps on `holds` in one direction, each stage read from the held
/// mixture as it stands: among the held kinds whose step releases hold, the
/// one with the greatest release steps (ties by the lesser nucleon count,
/// then the lesser kind), transducing `share x release` and passing
/// `share x (1 - release)` to its products, which join the mixture; the
/// stages end where no held kind releases. Each step moves hold strictly to
/// heavier kinds (fusion) or strictly to lighter ones (fission), so the
/// stages end; the count of stages is guarded by the square of the table's
/// kinds.
fn transduce(holds: &mut Holds, heat: &Q, fission: bool, steps: &mut Vec<Step>) {
    let mut found: BTreeMap<u32, Option<Found>> = BTreeMap::new();
    let guard = nucleons().of.len().saturating_mul(nucleons().of.len());
    for _ in 0..guard {
        let mut best: Option<((u32, u32), Q)> = None;
        for (&key, share) in holds.iter() {
            if share.signum() <= 0 {
                continue;
            }
            let step = found
                .entry(key.1)
                .or_insert_with(|| found_step(key.1, heat, fission));
            let Some(step) = step.as_ref() else { continue };
            // Keys are visited in increasing (A, z), so a later key replaces
            // the best only on a strictly greater release.
            if best.as_ref().is_none_or(|(_, r)| step.release > *r) {
                best = Some((key, step.release.clone()));
            }
        }
        let Some((key, _)) = best else { break };
        let Some(step) = found.get(&key.1).and_then(Option::as_ref) else {
            break;
        };
        let share = holds.insert(key, Q::ZERO).unwrap_or(Q::ZERO);
        let passed = &share * &(&Q::ONE - &step.release);
        let frames: Vec<Q> = step
            .products
            .iter()
            .map(|p| held_frames(*p, heat))
            .collect();
        let all = frames.iter().fold(Q::ZERO, |a, w| &a + w);
        let mut given = Q::ZERO;
        for (i, p) in step.products.iter().enumerate() {
            // The last product takes the exact remainder, so the hold passed
            // on is conserved.
            let part = if i + 1 == step.products.len() {
                &passed - &given
            } else {
                (&passed * &frames[i]).checked_div(&all).unwrap_or(Q::ZERO)
            };
            given = &given + &part;
            add_hold(holds, *p, &part);
        }
        steps.push(Step {
            kind: key.1,
            nuclei: step.nuclei,
            products: step.products.clone(),
            fission,
            transduced: &share * &step.release,
            share,
            release: step.release.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pure-hydrogen held mixture, an input of the tests only.
    fn hydrogen() -> Vec<(u32, Q)> {
        vec![(1, Q::ONE)]
    }

    /// Test mixtures: pure hydrogen, hydrogen with helium, and a spread of
    /// heavier kinds, inputs of the tests only.
    fn mixtures() -> Vec<Vec<(u32, Q)>> {
        vec![
            hydrogen(),
            vec![(1, Q::ratio(3, 4)), (2, Q::ratio(1, 4))],
            vec![
                (6, Q::ratio(1, 3)),
                (26, Q::ratio(1, 3)),
                (92, Q::ratio(1, 3)),
            ],
            vec![(8, Q::ONE)],
        ]
    }

    fn heats() -> [Q; 3] {
        [Q::ZERO, Q::ratio(1, 3), Q::ONE]
    }

    #[test]
    fn hydrogen_sequence_starts_with_the_step_the_rule_finds() {
        // The pure-hydrogen mixture at heat 0 begins with the fusion step the
        // rule finds for its one kind, on the whole hold, and its product
        // takes the next stage with the hold passed to it.
        let seq = Sequence::of(&hydrogen(), &Q::ZERO);
        let (k, product, release) = fusion_step(1, &Q::ZERO).unwrap();
        let first = &seq.steps[0];
        assert_eq!((first.kind, first.nuclei), (1, k));
        assert_eq!(first.products, vec![product]);
        assert_eq!((&first.share, &first.release), (&Q::ONE, &release));
        // At heat 0 the frames are the kinds' holds, so the release is the
        // built form `1 - hold(z') / (k hold(z))`.
        let built = &Q::ONE
            - &Q::int(i64::from(crate::compound::hold(product)))
                .checked_div(&Q::int(i64::from(k * crate::compound::hold(1))))
                .unwrap();
        assert_eq!(first.transduced, built);
        let second = &seq.steps[1];
        assert_eq!(second.kind, product);
        assert_eq!(second.share, &Q::ONE - &first.transduced);
    }

    #[test]
    fn pole_point_lies_on_the_z_side() {
        // A kind's open frames are at most its charge of its 3Z, so the
        // collective's pole point is at most 1/2.
        let half = Q::ratio(1, 2);
        for heat in heats() {
            for z in (1..=200).step_by(7) {
                let t0 = pole_point(&[(z, Q::ONE)], &heat);
                assert!(t0.signum() >= 0 && t0 <= half, "z {z}: {t0}");
            }
            for mix in mixtures() {
                assert!(pole_point(&mix, &heat) <= half);
            }
        }
    }

    #[test]
    fn throat_echoes_fall_as_compactness_grows() {
        let t0 = pole_point(&hydrogen(), &Q::ZERO);
        let deltas = [
            Dy::sci(1, -6),
            Dy::sci(1, -3),
            Dy::ratio(1, 10),
            Dy::ratio(1, 2),
        ];
        let n: Vec<Dy> = deltas
            .iter()
            .map(|d| throat_echoes(&t0, *d).unwrap())
            .collect();
        assert!(n.windows(2).all(|w| w[1] < w[0]), "{n:?}");
        assert!(throat_echoes(&Q::ZERO, Dy::ratio(1, 10)).is_none());
        assert!(throat_echoes(&t0, Dy::ZERO).is_none());
        assert_eq!(throat_echoes(&Q::ONE, Dy::ratio(1, 10)), Some(Dy::ZERO));
    }

    #[test]
    fn steps_conserve_nucleons_and_the_books_close() {
        for heat in heats() {
            for mix in mixtures() {
                let seq = Sequence::of(&mix, &heat);
                let cumulative = seq.cumulative().last().cloned().unwrap_or(Q::ZERO);
                assert_eq!(&cumulative + &seq.remnant, Q::ONE);
                for s in &seq.steps {
                    let a_in = s.nuclei * nucleon_count(s.kind).unwrap();
                    let a_out: u32 = s.products.iter().map(|p| nucleon_count(*p).unwrap()).sum();
                    assert_eq!(a_in, a_out, "{s:?}");
                    assert!(s.release.signum() > 0 && s.release < Q::ONE, "{s:?}");
                    assert!(s.share.signum() > 0 && s.share <= Q::ONE, "{s:?}");
                    assert_eq!(s.transduced, &s.share * &s.release);
                }
                // Fusion stages come before fission stages.
                let first_split = seq.steps.iter().position(|s| s.fission);
                if let Some(i) = first_split {
                    assert!(seq.steps[i..].iter().all(|s| s.fission));
                    assert!(seq.pole_point.signum() > 0);
                }
            }
        }
    }

    #[test]
    fn a_mixture_with_no_releasing_step_does_not_ignite() {
        // A cluster has no nucleon count in the element table and takes no
        // step; a held mixture of clusters alone does not ignite.
        let water = crate::compound::cluster(8, 1, 1, 2).unwrap();
        let seq = Sequence::of(&[(water, Q::ONE)], &Q::ZERO);
        assert!(!seq.ignites());
        assert_eq!(seq.remnant, Q::ONE);
        let delta = Dy::ratio(1, 10);
        let age = Dy::ONE;
        assert_eq!(
            crate::field::transduction(&seq, delta, Dy::ZERO, age),
            crate::field::Transduction::Unlit
        );
        // Every kind of the table without a step of either direction gives
        // the same.
        for z in (1..=200)
            .filter(|z| fusion_step(*z, &Q::ZERO).is_none() && fission_step(*z, &Q::ZERO).is_none())
        {
            assert!(!Sequence::of(&[(z, Q::ONE)], &Q::ZERO).ignites(), "z {z}");
        }
    }
}
