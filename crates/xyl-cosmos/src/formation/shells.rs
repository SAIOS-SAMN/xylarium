//! The founding shells: each kind's capture share, the stacks of the mass
//! profile and each kind's reach about the throat (see the parent module's
//! header for the formulas).

use super::{half, mass_weight};
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

/// The beads of a profile (kinds ordered heaviest first): half-open index ranges
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
/// then adds the kind's amount over the total `amount` to `held`. Returns the shares by
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

/// A stack's share `s_i`: the sum of its kinds' capture shares.
fn stack_share(part: &[Weighed], shares: &BTreeMap<u32, Q>) -> Q {
    part.iter().fold(Q::ZERO, |acc, (z, _, _)| {
        &acc + shares.get(z).unwrap_or(&Q::ZERO)
    })
}

/// The reach a push carries a nucleus of mass `w` to about the hold
/// `h_final`: `push / w / h_final`, on the 2^-64 grid.
fn carried_reach(push: &Q, w: &Q, h_final: &Q) -> Q {
    // Stand-in S120: The mass divides the push instead of resisting it.
    on_grid(
        &push
            .checked_div(w)
            .unwrap_or(Q::ZERO)
            .checked_div(h_final)
            .unwrap_or(Q::ZERO),
    )
}

/// The founding shells: each kind's reach about the throat,
/// R_z = (K + s_i) / m_z / H (m_z the mass weight), after the filter below. Returns the final
/// share s_final (each shell spreads R (1 +/- s_final)) and, per kind,
/// (z, amount, reach). What the throat holds is not read here: it is the
/// closed kinds, held on both faces of the paired throat (T16).
pub fn founding_shells(mix: &[(u32, Q)]) -> (Q, Vec<(u32, Q, Q)>) {
    // The founding runs on mass: amounts are converted from held count to
    // mass (x mass / hold) for the capture shares, the profile's beads and
    // the carriage's inertia, and converted back on return.
    let to_mass = |z: u32| {
        mass_weight(z)
            .checked_div(&Q::int(i64::from(crate::compound::hold(z))))
            .unwrap_or(Q::ONE)
    };
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
            mix.into_iter()
                .filter(|(_, a)| a.signum() > 0)
                .map(move |(z, a)| (z, a.checked_div(&to_mass(z)).unwrap_or(Q::ZERO), r.clone()))
                .collect::<Vec<_>>()
        })
        .collect();
    (s_final, out)
}

/// Each kind's shell at release, heaviest first: (z, amount, reach). The
/// stacks are the beads of the mass profile (each kind's amount, already in
/// mass); a stack's push is `K + s_i`, and each kind is carried by the kind's
/// mass per nucleus, `R = (K + s_i) / w / H`.
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
        // Stand-in S119: A stack push is a sum, not a composition against a hold.
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
        // The contact is 1/2 for a whole passage, else 1/2 the share of the
        // spread crossed.
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
        // Diagnostic trace (XYL_FILTER_TRACE=z_i): prints every term of one
        // lighter kind's meetings; it has no effect on results.
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
/// amount less what it gave, gains every take at its giver's release
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
