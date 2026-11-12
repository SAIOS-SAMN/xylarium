//! The founding shells: each kind's capture share, the stacks of the mass
//! profile, each kind's reach about the throat and its rest point there
//! (see the parent module's header for the formulas).

use super::half;
use num_bigint::BigInt;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use xyl_core::shares::FacePairs;
use xyl_core::Q;

/// A kind on the cascade: (atomic number, amount, mass per nucleus).
type Weighed = (u32, Q, Q);

/// A released shell: (atomic number, amount, reach held as face pairs).
type Released = (u32, Q, FacePairs);

/// A sum's reading, `x` cut toward zero to the 2^-64 grid.
fn on_grid(x: &Q) -> Q {
    xyl_core::certified::toward_zero(x, 64)
}

/// `x` cut toward zero to the nucleus grain, 2^-232.
fn nucleus_grain(x: &Q) -> Q {
    xyl_core::certified::toward_zero(x, 232)
}

/// A kind's mass per nucleus at capture: its protons, `Z`. The bounce
/// captures charge first (`Q0 = 0`); the neutrons are the conversion's rest
/// point after the reach is read ([`rest_neutrons`]).
fn capture_mass(z: u32) -> Q {
    Q::int(i64::from(z))
}

/// The kinds of `mix` with a positive amount and a positive electron hold,
/// each amount multiplied by `scale(z)`: (z, amount, capture mass).
fn weighed(mix: &[(u32, Q)], scale: impl Fn(u32) -> Q) -> Vec<Weighed> {
    mix.iter()
        .filter(|(z, _)| crate::compound::electron_hold(*z) > 0)
        .map(|(z, a)| (*z, a * &scale(*z), capture_mass(*z)))
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

/// The share the chain holds, light to heavy (`kinds` sorted by Z): each
/// kind adds its amount over the total `amount`, on the 2^-64 grid.
fn held_share(kinds: &[Weighed], amount: &Q) -> Q {
    kinds.iter().fold(Q::ZERO, |held, (_, a, _)| {
        on_grid(&(&held + &a.checked_div(amount).unwrap_or(Q::ZERO)))
    })
}

/// The face `1/2 held / (held + h_final)` of a share `held` against the
/// throat's final hold: the throat's final share `s_final` for the chain's
/// whole hold, a stack's share `s_i` for the stack's. Exact: a ratio of
/// the hold's reading, not a sum.
fn final_share(held: &Q, h_final: &Q) -> Q {
    half(&held.checked_div(&(held + h_final)).unwrap_or(Q::ZERO))
}

/// A stack's share `s_i`: the face of the hold the stack gathers, its
/// amount over the total `amount` ([`final_share`]). The capture shares are
/// running subtotals of one enclosure, so a stack's share is the one face
/// of what it holds, never their sum.
fn stack_share(part: &[Weighed], amount: &Q, h_final: &Q) -> Q {
    let held = part.iter().fold(Q::ZERO, |acc, (_, a, _)| &acc + a);
    final_share(
        &on_grid(&held.checked_div(amount).unwrap_or(Q::ZERO)),
        h_final,
    )
}

/// The reach a stack's push carries kind `z` to about the hold `h_final`,
/// as face pairs: the push's dilation `D(p_stack)` composed against the
/// kind's resistance `r_m`, `p_net = p_stack (+) (-r_m)`, so `R = H D(p_net)
/// = H D(p_stack) / D(r_m)`, the pairs `[(H), D(p_stack), (3, W_e)]` with
/// `D(r_m) = W_e / 3`, `W_e = 3Z - U` the electron hold. `None` for a kind
/// with no electron hold.
fn carried_reach(d_stack: &FacePairs, z: u32, h_final: &Q) -> Option<FacePairs> {
    let w_e = BigInt::from(crate::compound::electron_hold(z));
    let resist = FacePairs::pair(BigInt::from(3), w_e)?;
    Some(FacePairs::of(h_final)?.compose(d_stack).compose(&resist))
}

/// The conversion's rest point at each parcel's reach, gathered per kind:
/// a parcel of kind `z` at reach `R` about the hold `H` rests at
/// `Q* / Z = H / R`, the antipolar face (`t -> 1 / t`) of its reach: the
/// reach's pairs swapped and composed with `H`, exact. The kind's neutrons
/// per nucleus are its parcels' rest points weighted by their amounts,
/// `Q_z = Z sum(a H / R) / sum(a)`, the sum read on the 2^-64 grid; a kind
/// with no parcel has no entry (`Q* = 0`, its capture state).
fn rest_neutrons(parcels: &[(BTreeMap<u32, Q>, FacePairs)], h_final: &Q) -> BTreeMap<u32, Q> {
    let mut sums: BTreeMap<u32, (Q, Q)> = BTreeMap::new();
    for (mix, r) in parcels {
        let Some(hold) = FacePairs::of(h_final) else {
            continue;
        };
        let t = hold.compose(&r.inverse()).value();
        for (z, a) in mix.iter().filter(|(_, a)| a.signum() > 0) {
            let e = sums.entry(*z).or_insert((Q::ZERO, Q::ZERO));
            e.0 = &e.0 + &(a * &t);
            e.1 = &e.1 + a;
        }
    }
    sums.into_iter()
        .filter_map(|(z, (num, den))| {
            Some((z, on_grid(&(&capture_mass(z) * &num.checked_div(&den)?))))
        })
        .collect()
}

/// The founding shells: each kind's reach about the throat at capture,
/// `R_z = H D(s_final (+) s_i) / D(r_m)` with `D(r_m) = W_e / 3`, after
/// the filter below. Returns the final share s_final (each shell spreads
/// R (1 +/- s_final)), per kind (z, amount, reach), and the universal
/// table: each kind's neutrons per nucleus at the conversion's rest point
/// (read by `circuit::rest_neutrons`). What the throat holds is not read
/// here: it is the closed kinds, held on both faces of the paired throat
/// (T16).
pub fn founding_shells(mix: &[(u32, Q)]) -> (Q, Vec<(u32, Q, Q)>, BTreeMap<u32, Q>) {
    // The founding runs on the capture mass: amounts are converted from
    // held count to mass (x Z / W_e) for the held share, the profile's beads
    // and the filter, and converted back on return.
    let to_mass = |z: u32| {
        capture_mass(z)
            .checked_div(&Q::int(i64::from(crate::compound::electron_hold(z))))
            .unwrap_or(Q::ONE)
    };
    let mut kinds = weighed(mix, to_mass);
    if kinds.is_empty() {
        return (Q::ZERO, Vec::new(), BTreeMap::new());
    }
    let amount = kinds.iter().fold(Q::ZERO, |acc, (_, a, _)| &acc + a);
    kinds.sort_by_key(|k| k.0);
    let held = held_share(&kinds, &amount);
    let h_final = &Q::ONE + &held;
    let s_final = final_share(&held, &h_final);
    let released = release_shells(&kinds, &amount, &s_final, &h_final);
    let takes = filter_takes(&released, &h_final, &s_final);
    let parcels = place_takes(&released, takes);
    let neutrons = rest_neutrons(&parcels, &h_final);
    let out = parcels
        .into_iter()
        .flat_map(|(mix, r)| {
            mix.into_iter()
                .filter(|(_, a)| a.signum() > 0)
                .map(move |(z, a)| (z, a.checked_div(&to_mass(z)).unwrap_or(Q::ZERO), r.value()))
                .collect::<Vec<_>>()
        })
        .collect();
    (s_final, out, neutrons)
}

/// Each kind's shell at release, heaviest first: (z, amount, reach). The
/// stacks are the beads of the mass profile (each kind's amount, already in
/// mass); a stack's push is the throat's final share composed with the
/// stack's own, `p_stack = s_final (+) s_i`, held as its dilation
/// `D(s_final) D(s_i)` (the flow law as composed pairs), and each kind is
/// carried against its resistance ([`carried_reach`]). A kind whose reach
/// is refused is not released.
fn release_shells(kinds: &[Weighed], amount: &Q, s_final: &Q, h_final: &Q) -> Vec<Released> {
    let mut heavy = kinds.to_vec();
    heavy.sort_by(|a, b| b.2.cmp(&a.2));
    let prof: Vec<Q> = heavy.iter().map(|(_, a, _)| a.clone()).collect();
    let mut shells = Vec::new();
    for (a0, a1) in profile_beads(&prof) {
        let part = &heavy[a0..a1];
        let s_i = stack_share(part, amount, h_final);
        let Some(push) = FacePairs::dilation(s_final)
            .zip(FacePairs::dilation(&s_i))
            .map(|(f, i)| f.compose(&i))
        else {
            continue;
        };
        for (z, a, _) in part {
            if let Some(r) = carried_reach(&push, *z, h_final) {
                shells.push((*z, a.clone(), r));
            }
        }
    }
    shells
}

/// The release state each meeting is read against: per released shell its
/// tightness, its reach read once, its deficit `H / (R + H)` and its spread
/// `R (1 -/+ s_final)`, and the kind whose meetings are traced
/// (`XYL_FILTER_TRACE`).
struct ReleaseState {
    tight: Vec<Q>,
    reach: Vec<Q>,
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
fn filter_takes(released: &[Released], h_final: &Q, s_final: &Q) -> Vec<(usize, usize, Q)> {
    let reach: Vec<Q> = released.iter().map(|(_, _, r)| r.value()).collect();
    let state = ReleaseState {
        tight: released
            .iter()
            .map(|(z, _, _)| crate::compound::tightness(*z))
            .collect(),
        deficit: reach
            .iter()
            .map(|r| h_final.checked_div(&(r + h_final)).unwrap_or(Q::ZERO))
            .collect(),
        band: reach
            .iter()
            .map(|r| (r * &(&Q::ONE - s_final), r * &(&Q::ONE + s_final)))
            .collect(),
        reach,
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
    released: &[Released],
    state: &ReleaseState,
    i: usize,
    zi: u32,
    mi: &Q,
    ri_pairs: &FacePairs,
) -> Vec<(usize, usize, Q)> {
    let mut mine = Vec::new();
    let ri = &state.reach[i];
    for (j, (zj, mj, rj_pairs)) in released.iter().enumerate().take(i) {
        let (lo_j, hi_j) = (&state.band[j].0, &state.band[j].1);
        // The reaches compare by their pairs, the shared hold and throat
        // share cancelling.
        if mj.signum() <= 0
            || mi.signum() <= 0
            || rj_pairs.cmp_value(ri_pairs) != Ordering::Less
            || ri <= lo_j
        {
            continue;
        }
        let rj = &state.reach[j];
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
fn place_takes(
    released: &[Released],
    takes: Vec<(usize, usize, Q)>,
) -> Vec<(BTreeMap<u32, Q>, FacePairs)> {
    let mut asked: Vec<Q> = vec![Q::ZERO; released.len()];
    for (_, j, take) in &takes {
        asked[*j] = &asked[*j] + take;
    }
    let mut given: Vec<Q> = vec![Q::ZERO; released.len()];
    let mut parts: Vec<Vec<(u32, Q, Q)>> = vec![Vec::new(); released.len()];
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
        parts[i].push((released[j].0, take, released[j].2.value()));
    }
    released
        .iter()
        .enumerate()
        .map(|(i, (z, m, r_pairs))| {
            let own = m - &given[i];
            let mut mix: BTreeMap<u32, Q> = BTreeMap::new();
            mix.insert(*z, own.clone());
            // A shell that caught nothing keeps its reach's pairs; one that
            // caught stands at the mean of its parts' reaches, a sum read on
            // the 2^-64 grid.
            if parts[i].is_empty() {
                return (mix, r_pairs.clone());
            }
            let r = &r_pairs.value();
            let (mut mass, mut ml) = (own.clone(), &own * r);
            for (zj, a, rj) in &parts[i] {
                let e = mix.entry(*zj).or_insert(Q::ZERO);
                *e = &*e + a;
                mass = &mass + a;
                ml = &ml + &(a * rj);
            }
            let mean = on_grid(&ml.checked_div(&mass).unwrap_or_else(|| r.clone()));
            (mix, FacePairs::of(&mean).unwrap_or_else(|| r_pairs.clone()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A lone kind holds the whole chain, so `H = 2` and its parcel rests at
    /// the antipolar face of its reach: `(Q* / Z) R = H`, the reach's pairs
    /// swapped; `Q_z` is a sum's reading on the 2^-64 grid.
    #[test]
    fn a_lone_kind_rests_at_the_antipole_of_its_reach() {
        for z in [1u32, 2, 26, 92, 1092] {
            let (_, shells, table) = founding_shells(&[(z, Q::ONE)]);
            let r = &shells[0].2;
            let t = table[&z].checked_div(&Q::int(i64::from(z))).unwrap();
            // Q_z is on the 2^-64 grid, so within a grid step at the reach.
            let step = Q::new(1, 1 << 62).unwrap();
            assert!((&(&t * r) - &Q::int(2)).abs() <= &step * r, "Z {z}");
        }
    }

    /// The resistance `D(r_m) = W_e / 3` is positive for every atom, so no
    /// kind of a mix is refused its reach or its rest point.
    #[test]
    fn every_atom_of_a_mix_has_a_rest_point() {
        let mix: Vec<(u32, Q)> = (1..=crate::elements::TOP_Z)
            .step_by(37)
            .map(|z| (z, Q::new(1, i64::from(z)).unwrap()))
            .collect();
        let (_, _, table) = founding_shells(&mix);
        for (z, _) in &mix {
            assert!(table.get(z).is_some_and(|q| q.signum() > 0), "Z {z}");
        }
    }
}
