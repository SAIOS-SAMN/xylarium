//! The founding cascade and the founding shells: each kind's capture share,
//! the stacks of the mass profile and each kind's reach about the throat
//! (see the parent module's header for the formulas).

use super::{half, mass_weight, weight};
use rayon::prelude::*;
use std::collections::BTreeMap;
use xyl_core::Q;

/// A kind on the cascade: (atomic number, amount, mass per nucleus).
type Weighed = (u32, Q, Q);

/// `x` rounded down to the 2^-64 grid.
fn on_grid(x: &Q) -> Q {
    xyl_core::certified::down(x, 64)
}

/// `x` rounded down to the nucleus grain, 2^-232.
fn nucleus_grain(x: &Q) -> Q {
    xyl_core::certified::down(x, 232)
}

/// The kinds of `mix` with a positive amount and mass, each amount
/// multiplied by `scale(z)`: (z, amount, mass per nucleus).
fn weighed(mix: &[(u32, Q)], scale: impl Fn(u32) -> Q) -> Vec<Weighed> {
    mix.iter()
        .map(|(z, a)| (*z, a * &scale(*z), mass_weight(*z)))
        .filter(|(_, a, w)| a.signum() > 0 && w.signum() > 0)
        .collect()
}

/// The beads of a profile ordered heaviest first: half-open index ranges
/// parted at every local minimum (`prof[k] < prof[k - 1]` and
/// `prof[k] <= prof[k + 1]`), covering the whole profile.
fn profile_beads(prof: &[Q]) -> Vec<(usize, usize)> {
    let mut segs = Vec::new();
    let mut start = 0;
    for k in 1..prof.len().saturating_sub(1) {
        if prof[k] < prof[k - 1] && prof[k] <= prof[k + 1] {
            segs.push((start, k));
            start = k;
        }
    }
    segs.push((start, prof.len()));
    segs
}

/// Each kind's capture share along the chain, light to heavy (`kinds`
/// sorted by Z): with `held` the share captured so far and `H = 1 + held`,
/// a kind captures `s = 1/2 held / (held + H)`, the unlocked hemisphere, and
/// then adds its own amount over `amount` to `held`. Returns the shares by
/// Z and the final held share.
fn capture_shares(kinds: &[Weighed], amount: &Q) -> (BTreeMap<u32, Q>, Q) {
    let mut held = Q::ZERO;
    let mut shares = BTreeMap::new();
    for (z, a, _) in kinds {
        let h = &Q::ONE + &held;
        let s = on_grid(&half(&held.checked_div(&(&held + &h)).unwrap_or(Q::ZERO)));
        shares.insert(*z, s);
        held = on_grid(&(&held + &a.checked_div(amount).unwrap_or(Q::ZERO)));
    }
    (shares, held)
}

/// The throat's final share `1/2 held / (held + h_final)`.
fn final_share(held: &Q, h_final: &Q) -> Q {
    on_grid(&half(
        &held.checked_div(&(held + h_final)).unwrap_or(Q::ZERO),
    ))
}

/// A stack's own share `s_i`: the sum of its kinds' capture shares.
fn stack_share(part: &[Weighed], shares: &BTreeMap<u32, Q>) -> Q {
    part.iter().fold(Q::ZERO, |acc, (z, _, _)| {
        &acc + shares.get(z).unwrap_or(&Q::ZERO)
    })
}

/// The reach a push carries a nucleus of mass `w` to about the hold
/// `h_final`: `push / w / h_final`, on the 2^-64 grid.
fn carried_reach(push: &Q, w: &Q, h_final: &Q) -> Q {
    on_grid(
        &push
            .checked_div(w)
            .unwrap_or(Q::ZERO)
            .checked_div(h_final)
            .unwrap_or(Q::ZERO),
    )
}

/// One stack of the founding cascade.
#[derive(Clone, Debug)]
pub struct Stack {
    /// The stack's kinds, heaviest first.
    pub kinds: Vec<u32>,
    /// Amount (share of the founding) and mean hold (weight).
    pub amount: Q,
    /// Mean hold (weight) of the stack's kinds.
    pub w_mean: Q,
    /// The bounce that released it (its push share) and its spin at release.
    pub bounce: Q,
    /// Spin at release, before its passages.
    pub released: Q,
    /// Spin once the cascade settles (after every passage through it).
    pub settled: Q,
    /// The beat set by its release: the round trip to it, the nearest stack.
    pub beat: Q,
}

/// The founding cascade (diagnostic): the stack built by the capture chain,
/// emptied from the top by bursts.
/// - Stacks are the beads of the weight profile (each kind's amount x hold,
///   heaviest first): a bead at every local maximum, parted at the minima.
/// - Bounce k releases stack k at its push b_k, carried to
///   l = b_k w_mean / w_k (the whole stack's mean over its own), reach R = l
///   (the throat's state 1).
/// - Once released, it passes out through every earlier stack still inside
///   its reach, innermost first. Each passage is a meeting: the contrast T
///   of their deficits (delta = 1 / (R + 1)), with no grip applied, moves
///   turning outward as a conserved spin exchange (the outer gains
///   T m_inner / M of their spin contrast, the inner gives the mirror; never
///   more than it carries).
/// - What stack k loses in its passages it hands back at the throat:
///   b_(k+1) = p + that loss, so the next bounce is stronger.
/// - The beat: each release's pulse echoes off the nearest stack, a round
///   trip 2 R_k; with none left the throat echoes at 2 (its own reach, 1).
///
/// Returns the stacks and the settled beat.
pub fn cascade(mix: &[(u32, Q)]) -> (Vec<Stack>, Q) {
    let p = half(&Q::ONE);
    // inertia is the mass per nucleus; stacks are ordered heaviest first
    let mut kinds = weighed(mix, |_| Q::ONE);
    kinds.sort_by(|a, b| b.2.cmp(&a.2));
    if kinds.is_empty() {
        return (Vec::new(), Q::int(2));
    }
    let amount = kinds.iter().fold(Q::ZERO, |acc, (_, a, _)| &acc + a);
    let total_w = kinds.iter().fold(Q::ZERO, |acc, (_, a, w)| &acc + &(a * w));
    let w_all = total_w.checked_div(&amount).unwrap_or(Q::ONE);
    let prof: Vec<Q> = kinds.iter().map(|(_, a, w)| a * w).collect();
    let mut stacks: Vec<Stack> = Vec::new();
    let mut b = p.clone();
    for (a0, a1) in profile_beads(&prof) {
        let part = &kinds[a0..a1];
        let m = part.iter().fold(Q::ZERO, |acc, (_, a, _)| &acc + a);
        let mw = part.iter().fold(Q::ZERO, |acc, (_, a, w)| &acc + &(a * w));
        let w_mean = mw.checked_div(&m).unwrap_or(Q::ONE);
        // l0 = b_k w_all / w_mean
        let l0 = on_grid(&(&b * &w_all).checked_div(&w_mean).unwrap_or(Q::ZERO));
        let (l, lost) = pass_out(&mut stacks, &m, l0.clone());
        stacks.push(Stack {
            kinds: part.iter().map(|(z, _, _)| *z).collect(),
            amount: m.clone(),
            w_mean,
            bounce: b.clone(),
            released: l0.clone(),
            settled: l.clone(),
            beat: &Q::int(2) * &l0,
        });
        // b_(k+1) = p + the spin stack k gave in its passages
        b = &p + &lost;
    }
    (stacks, Q::int(2))
}

/// A released stack of amount `m` and spin `l` passing out through every
/// earlier stack inside its reach, innermost first. Each meeting exchanges
/// spin by the contrast `T = |d_in - d_out| / (d_in + d_out)` of their
/// deficits `d = 1 / (R + 1)`: the earlier stack gains `T m spread / M` and
/// the released one gives `T m_j spread / M` (at most its spin), `M` their
/// combined amount and `spread` their spin contrast. Returns the released
/// stack's spin after its passages and the spin it gave.
fn pass_out(stacks: &mut [Stack], m: &Q, mut l: Q) -> (Q, Q) {
    let mut lost = Q::ZERO;
    let mut order: Vec<usize> = (0..stacks.len()).collect();
    order.sort_by(|&x, &y| stacks[x].settled.cmp(&stacks[y].settled));
    for j in order {
        if stacks[j].settled >= l {
            continue;
        }
        let (r_in, r_out) = (stacks[j].settled.clone(), l.clone());
        let (d_in, d_out) = (
            Q::ONE.checked_div(&(&r_in + &Q::ONE)).unwrap_or(Q::ZERO),
            Q::ONE.checked_div(&(&r_out + &Q::ONE)).unwrap_or(Q::ZERO),
        );
        let t = (&d_in - &d_out)
            .abs()
            .checked_div(&(&d_in + &d_out))
            .unwrap_or(Q::ZERO);
        let both = m + &stacks[j].amount;
        let spread = &l - &stacks[j].settled;
        let give = on_grid(
            &(&(&t * &stacks[j].amount) * &spread)
                .checked_div(&both)
                .unwrap_or(Q::ZERO),
        )
        .min(l.clone());
        let gain = on_grid(&(&(&t * m) * &spread).checked_div(&both).unwrap_or(Q::ZERO));
        l = &l - &give;
        lost = &lost + &give;
        stacks[j].settled = &stacks[j].settled + &gain;
    }
    (l, lost)
}

/// The equitable cascade (diagnostic): bursts from the top and expansion
/// from the base combined; K is the epicentre's spin, k the orbital speed,
/// and each share is the unlocked face R / (R + H) at its own event. Holds,
/// per form, the spin K, the throat's final share, the stacks (kinds, push,
/// reaches) and the kinds the throat holds.
#[derive(Clone, Debug)]
pub struct EquitableReading {
    /// Label of the form of the cascade read.
    pub form: &'static str,
    /// Epicentre spin K: the sum of every kind's capture share.
    pub k_spin: Q,
    /// The throat's final hold, H = 1 + the held share.
    pub h_final: Q,
    /// The throat's final share, 1/2 R / (R + H) at the edge; each founding shell
    /// spreads R (1 +/- s_final).
    pub s_final: Q,
    /// Per stack: kinds (heaviest first), push K + s_i, reaches (lightest
    /// kind's, heaviest kind's), and the orbital speed at the heaviest.
    pub stacks: Vec<(Vec<u32>, Q, Q, Q, Q)>,
    /// The kinds the throat holds: those whose reach times (1 + s_final) is at most 1.
    pub held: Vec<u32>,
}

/// Runs the equitable cascade (diagnostic) on `mix`, a list of (kind, amount)
/// pairs, and returns one [`EquitableReading`] per form. Kinds with zero amount
/// or zero mass are ignored; an empty result means no kind remained.
pub fn equitable_cascade(mix: &[(u32, Q)]) -> Vec<EquitableReading> {
    // inertia is the mass per nucleus
    let mut kinds = weighed(mix, |_| Q::ONE);
    if kinds.is_empty() {
        return Vec::new();
    }
    let amount = kinds.iter().fold(Q::ZERO, |acc, (_, a, _)| &acc + a);
    // the capture chain runs light to heavy (Z ascending)
    kinds.sort_by_key(|k| k.0);
    let mut out = Vec::new();
    // one form: each capture's share is the unlocked hemisphere,
    // 1/2 (1 - delta) = 1/2 R / (R + H), with R = held and H = 1 + held
    for form in ["the unlocked face, R = held"] {
        let (shares, held) = capture_shares(&kinds, &amount);
        let k_spin = shares.values().fold(Q::ZERO, |acc, s| &acc + s);
        let h_final = &Q::ONE + &held;
        let s_final = final_share(&held, &h_final);
        // the stacks: beads of the weight profile (amount x mass), heaviest
        // first
        let mut heavy = kinds.clone();
        heavy.sort_by(|a, b| b.2.cmp(&a.2));
        let prof: Vec<Q> = heavy.iter().map(|(_, a, w)| a * w).collect();
        let mut stacks = Vec::new();
        let mut held_z = Vec::new();
        for (a0, a1) in profile_beads(&prof) {
            let part = &heavy[a0..a1];
            // the push K + s_i acts on each kind's own mass,
            // R = (K + s_i) / w / H, with no mean-mass factor (as in
            // founding_shells)
            let push = &k_spin + &stack_share(part, &shares);
            let (w_light, w_heavy) = (
                part.iter().map(|k| k.2.clone()).min().unwrap_or(Q::ONE),
                part.iter().map(|k| k.2.clone()).max().unwrap_or(Q::ONE),
            );
            let (r_far, r_deep) = (
                carried_reach(&push, &w_light, &h_final),
                carried_reach(&push, &w_heavy, &h_final),
            );
            // the orbital speed at the heaviest kind, H / (R_deep + H)
            let k_orbit = on_grid(
                &h_final
                    .checked_div(&(&r_deep + &h_final))
                    .unwrap_or(Q::ZERO),
            );
            for (z, _, w) in part {
                if &carried_reach(&push, w, &h_final) * &(&Q::ONE + &s_final) <= Q::ONE {
                    held_z.push(*z);
                }
            }
            stacks.push((
                part.iter().map(|k| k.0).collect(),
                push,
                r_far,
                r_deep,
                k_orbit,
            ));
        }
        out.push(EquitableReading {
            form,
            k_spin,
            h_final,
            s_final,
            stacks,
            held: held_z,
        });
    }
    out
}

/// The founding shells (the equitable cascade): each kind's reach about the
/// Wellspring, R_z = (K + s_i) / w_z / H, after the filter below, and whether
/// the Wellspring holds it (R_z (1 + s_final) <= 1). Returns the final share
/// s_final (each shell spreads R (1 +/- s_final)) and, per kind,
/// (z, amount, reach, held).
pub fn founding_shells(mix: &[(u32, Q)]) -> (Q, Vec<(u32, Q, Q, bool)>) {
    // the founding runs on mass: amounts are converted from held frames to
    // mass (x mass / hold) for the capture shares, the profile's beads and
    // the carriage's inertia, and converted back on return
    let to_mass = |z: u32| mass_weight(z).checked_div(&weight(z)).unwrap_or(Q::ONE);
    let mut kinds = weighed(mix, to_mass);
    if kinds.is_empty() {
        return (Q::ZERO, Vec::new());
    }
    let amount = kinds.iter().fold(Q::ZERO, |acc, (_, a, _)| &acc + a);
    kinds.sort_by_key(|k| k.0);
    let (shares, held) = capture_shares(&kinds, &amount);
    let k_spin = shares.values().fold(Q::ZERO, |acc, s| &acc + s);
    let h_final = &Q::ONE + &held;
    let s_final = final_share(&held, &h_final);
    let released = release_shells(&kinds, &shares, &k_spin, &h_final);
    let takes = filter_takes(&released, &h_final, &s_final);
    let out = place_takes(&released, takes)
        .into_iter()
        .flat_map(|(mix, r)| {
            let is_held = &r * &(&Q::ONE + &s_final) <= Q::ONE;
            mix.into_iter()
                .filter(|(_, a)| a.signum() > 0)
                .map(move |(z, a)| {
                    (
                        z,
                        a.checked_div(&to_mass(z)).unwrap_or(Q::ZERO),
                        r.clone(),
                        is_held,
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    (s_final, out)
}

/// Each kind's shell at release, heaviest first: (z, amount, reach). The
/// stacks are the beads of the mass profile (each kind's amount, already in
/// mass); a stack's push is `K + s_i`, and each kind is carried by its own
/// mass, `R = (K + s_i) / w / H`.
fn release_shells(
    kinds: &[Weighed],
    shares: &BTreeMap<u32, Q>,
    k_spin: &Q,
    h_final: &Q,
) -> Vec<Weighed> {
    let mut heavy = kinds.to_vec();
    heavy.sort_by(|a, b| b.2.cmp(&a.2));
    let prof: Vec<Q> = heavy.iter().map(|(_, a, _)| a.clone()).collect();
    let mut shells = Vec::new();
    for (a0, a1) in profile_beads(&prof) {
        let part = &heavy[a0..a1];
        let push = k_spin + &stack_share(part, shares);
        for (z, a, w) in part {
            shells.push((*z, a.clone(), carried_reach(&push, w, h_final)));
        }
    }
    shells
}

/// The release state each meeting is read against: per released shell its
/// tightness, its deficit `H / (R + H)` and its spread `R (1 -/+ s_final)`,
/// and the kind whose meetings are traced (`XYL_FILTER_TRACE`).
struct ReleaseState {
    tight: Vec<Q>,
    deficit: Vec<Q>,
    band: Vec<(Q, Q)>,
    trace: Option<u32>,
}

/// The filter's takes: (i catching, j giving, the take on the nucleus
/// grain). Each lighter shell, released later and pushed harder, passes out
/// through every heavier shell whose spread lies between the throat and its
/// reach. Contact, for isotropic shells: a whole passage through the other's
/// spread is 1/2, a partial one (ending inside it) 1/2 its share of that
/// spread. With s the two kinds' grip (tightness ratio),
/// D = M / (M + 1 - contact) and T the contrast of their deficits at their
/// reaches, the gripped fraction g = min(s D, T) / T is caught: the heavier
/// gives contact g m_i / (m_i + m_j) of its matter to the lighter. Every
/// passage is evaluated against the release state alone, so the result does
/// not depend on order and the lighter shells' meetings run in parallel
/// (collected in index order).
fn filter_takes(released: &[Weighed], h_final: &Q, s_final: &Q) -> Vec<(usize, usize, Q)> {
    let state = ReleaseState {
        tight: released
            .iter()
            .map(|(z, _, _)| crate::compound::tightness(*z))
            .collect(),
        deficit: released
            .iter()
            .map(|(_, _, r)| h_final.checked_div(&(r + h_final)).unwrap_or(Q::ZERO))
            .collect(),
        band: released
            .iter()
            .map(|(_, _, r)| (r * &(&Q::ONE - s_final), r * &(&Q::ONE + s_final)))
            .collect(),
        trace: std::env::var("XYL_FILTER_TRACE")
            .ok()
            .and_then(|v| v.parse::<u32>().ok()),
    };
    released
        .par_iter()
        .enumerate()
        .flat_map_iter(|(i, (zi, mi, ri))| meetings(released, &state, i, *zi, mi, ri))
        .collect()
}

/// Shell `i`'s meetings with every heavier shell `j < i` (see
/// [`filter_takes`]).
fn meetings(
    released: &[Weighed],
    state: &ReleaseState,
    i: usize,
    zi: u32,
    mi: &Q,
    ri: &Q,
) -> Vec<(usize, usize, Q)> {
    let mut mine = Vec::new();
    for (j, (zj, mj, rj)) in released.iter().enumerate().take(i) {
        let (lo_j, hi_j) = (&state.band[j].0, &state.band[j].1);
        if mj.signum() <= 0 || mi.signum() <= 0 || rj >= ri || ri <= lo_j {
            continue;
        }
        // contact: 1/2 for a whole passage, else 1/2 the share of the
        // spread crossed
        let near = if ri >= hi_j {
            half(&Q::ONE)
        } else {
            half(
                &(ri - lo_j)
                    .checked_div(&(hi_j - lo_j))
                    .unwrap_or(Q::ZERO)
                    .min(Q::ONE),
            )
        };
        let (ti, tj) = (&state.tight[i], &state.tight[j]);
        let grip_s = ti
            .clone()
            .min(tj.clone())
            .checked_div(&ti.clone().max(tj.clone()))
            .unwrap_or(Q::ZERO);
        let both = mi + mj;
        let d = both
            .checked_div(&(&both + &(&Q::ONE - &near)))
            .unwrap_or(Q::ZERO);
        let (di, dj) = (&state.deficit[i], &state.deficit[j]);
        let t = (dj - di).abs().checked_div(&(dj + di)).unwrap_or(Q::ZERO);
        if t.signum() <= 0 {
            continue;
        }
        let g = (&grip_s * &d)
            .min(t.clone())
            .checked_div(&t)
            .unwrap_or(Q::ONE);
        let frac = (&(&near * &g) * mi)
            .checked_div(&both)
            .unwrap_or(Q::ZERO)
            .min(Q::ONE);
        // diagnostic trace (XYL_FILTER_TRACE=z_i): prints every term of one
        // lighter kind's meetings; no effect on results
        if state.trace == Some(zi) {
            eprintln!(
                concat!(
                    "FILTER z_i {} meets Z{}: m_i {} m_j {} r_i {} r_j {} ",
                    "near {} s {} D {} T {} g {} frac {}"
                ),
                zi, zj, mi, mj, ri, rj, near, grip_s, d, t, g, frac
            );
        }
        let take = nucleus_grain(&(mj * &frac)).min(mj.clone());
        if take.signum() > 0 {
            mine.push((i, j, take));
        }
    }
    mine
}

/// Each shell's mix and reach after the filter. Where the lighter shells
/// ask more of a shell than it holds, every take from it is scaled by
/// holding over asked, so none gives more than it has. A shell keeps its
/// own amount less what it gave, gains every take at its giver's release
/// reach, and stands at the amount-weighted mean of its parts' reaches
/// (sum m l unchanged). Caught matter is not caught again in the same
/// founding.
fn place_takes(released: &[Weighed], takes: Vec<(usize, usize, Q)>) -> Vec<(BTreeMap<u32, Q>, Q)> {
    let mut asked: Vec<Q> = vec![Q::ZERO; released.len()];
    for (_, j, take) in &takes {
        asked[*j] = &asked[*j] + take;
    }
    let mut given: Vec<Q> = vec![Q::ZERO; released.len()];
    let mut parts: Vec<Vec<Weighed>> = vec![Vec::new(); released.len()];
    for (i, j, take) in takes {
        let (has, all) = (&released[j].1, &asked[j]);
        let take = if all > has {
            nucleus_grain(&(&take * has).checked_div(all).unwrap_or(Q::ZERO))
        } else {
            take
        };
        if take.signum() <= 0 {
            continue;
        }
        given[j] = &given[j] + &take;
        parts[i].push((released[j].0, take, released[j].2.clone()));
    }
    released
        .iter()
        .enumerate()
        .map(|(i, (z, m, r))| {
            let own = m - &given[i];
            let mut mix: BTreeMap<u32, Q> = BTreeMap::new();
            mix.insert(*z, own.clone());
            let (mut mass, mut ml) = (own.clone(), &own * r);
            for (zj, a, rj) in &parts[i] {
                let e = mix.entry(*zj).or_insert(Q::ZERO);
                *e = &*e + a;
                mass = &mass + a;
                ml = &ml + &(a * rj);
            }
            (
                mix,
                on_grid(&ml.checked_div(&mass).unwrap_or_else(|| r.clone())),
            )
        })
        .collect()
}
