//! The founding shells: each kind's capture share, the stacks of the mass
//! profile, each kind's reach about the throat and its rest point there
//! (see the parent module's header for the formulas).

use super::half;
use num_bigint::BigInt;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use xyl_core::shares::{FacePairs, Parcel};
use xyl_core::Q;

/// A kind on the cascade: (atomic number, amount, mass per nucleus).
type Weighed = (u32, Q, Q);

/// A released shell: (atomic number, amount, reach, the stack's push that
/// carried it), the reach and the push as face pairs.
type Released = (u32, Q, FacePairs, FacePairs);

/// `x` cut toward zero to the nucleus grain, 2^-232.
fn nucleus_grain(x: &Q) -> Q {
    xyl_core::certified::toward_zero(x, 232)
}

/// A kind's mass per nucleus at capture: its protons, `Z`. The bounce
/// captures charge first (`Q0 = 0`); the neutrons are the conversion's rest
/// point after the reach is read ([`antipoles`]).
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

/// The share a run of kinds holds: their amounts over the total `amount`,
/// exact. For the whole chain it is 1, so `H = 2` and `s_final = 1/6`.
fn held(kinds: &[Weighed], amount: &Q) -> Q {
    let held = kinds.iter().fold(Q::ZERO, |acc, (_, a, _)| &acc + a);
    held.checked_div(amount).unwrap_or(Q::ZERO)
}

/// The face `1/2 held / (held + hold)` of a share `held` against a hold:
/// the throat's final share `s_final` for the chain's whole hold, a stack's
/// share `s_i` for the stack's. A stack's share is the one face of what it
/// holds, never a sum of its kinds' faces.
fn face(held: &Q, hold: &Q) -> Q {
    half(&held.checked_div(&(held + hold)).unwrap_or(Q::ZERO))
}

/// What a push carries kind `z` to about `hold`, as face pairs: the push's
/// dilation `D(p)` composed against the kind's resistance `r_m`,
/// `p_net = p (+) (-r_m)`, so `R = H D(p) / D(r_m)`, the pairs
/// `[(H), D(p), (3, W_e)]` with `D(r_m) = W_e / 3`, `W_e = 3Z - U` the
/// electron hold. `None` for a kind with no electron hold.
fn carry(push: &FacePairs, z: u32, hold: &Q) -> Option<FacePairs> {
    let w_e = BigInt::from(crate::compound::electron_hold(z));
    let resist = FacePairs::pair(BigInt::from(3), w_e)?;
    Some(FacePairs::of(hold)?.compose(push).compose(&resist))
}

/// Each parcel at its rest point: a parcel of kind `z` at reach `R` about
/// the hold `H` rests at `Q* / Z = H / R`, the antipole (`t -> 1 / t`) of
/// its reach ([`Parcel::antipole`]), exact. Gathered per kind, unsummed:
/// a kind is the distribution of its parcels' rest points, and a reader
/// reads it at the reader's precision (`circuit::rest_neutrons`,
/// `circuit::mass_number`). A kind with no parcel has no entry (`Q* = 0`,
/// its capture state).
fn antipoles(parcels: &[(u32, Parcel)], hold: &Q) -> BTreeMap<u32, Vec<Parcel>> {
    let mut table: BTreeMap<u32, Vec<Parcel>> = BTreeMap::new();
    for (z, p) in parcels.iter().filter(|(_, p)| p.amount.signum() > 0) {
        if let Some(rest) = p.antipole(hold) {
            table.entry(*z).or_default().push(rest);
        }
    }
    table
}

/// The founding shells: each kind's reach about the throat at capture,
/// `R_z = H D(s_final (+) s_i) / D(r_m)` with `D(r_m) = W_e / 3`, after
/// the filter below. Returns the final share s_final (each shell spreads
/// R (1 +/- s_final)), per kind (z, amount, reach), and the universal
/// table: each kind's parcels at the conversion's rest point, `Q* / Z`
/// as face pairs (read by `circuit::rest_neutrons`). What the throat holds is not read
/// here: it is the closed kinds, held on both faces of the paired throat
/// (T16).
pub fn founding_shells(mix: &[(u32, Q)]) -> (Q, Vec<(u32, Q, Q)>, BTreeMap<u32, Vec<Parcel>>) {
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
    let held = held(&kinds, &amount);
    let hold = &Q::ONE + &held;
    let s_final = face(&held, &hold);
    let released = release(&kinds, &amount, &s_final, &hold);
    let takes = catches(&released, &hold, &s_final);
    let parcels = transfer(&released, takes, &hold);
    let neutrons = antipoles(&parcels, &hold);
    let out = parcels
        .into_iter()
        .filter(|(_, p)| p.amount.signum() > 0)
        .map(|(z, p)| {
            let a = p.amount.checked_div(&to_mass(z)).unwrap_or(Q::ZERO);
            (z, a, p.pairs.value())
        })
        .collect();
    (s_final, out, neutrons)
}

/// Each kind's shell at release, heaviest first: (z, amount, reach, push). The
/// stacks are the beads of the mass profile (each kind's amount, already in
/// mass); a stack's push is the throat's final share composed with the
/// stack's own, `p_stack = s_final (+) s_i`, held as its dilation
/// `D(s_final) D(s_i)` (the flow law as composed pairs), and each kind is
/// carried against its resistance ([`carry`]). A kind whose reach is
/// refused is not released.
fn release(kinds: &[Weighed], amount: &Q, s_final: &Q, hold: &Q) -> Vec<Released> {
    let mut heavy = kinds.to_vec();
    heavy.sort_by(|a, b| b.2.cmp(&a.2));
    let prof: Vec<Q> = heavy.iter().map(|(_, a, _)| a.clone()).collect();
    let mut shells = Vec::new();
    for (a0, a1) in profile_beads(&prof) {
        let part = &heavy[a0..a1];
        let s_i = face(&held(part, amount), hold);
        let Some(push) = FacePairs::dilation(s_final)
            .zip(FacePairs::dilation(&s_i))
            .map(|(f, i)| f.compose(&i))
        else {
            continue;
        };
        for (z, a, _) in part {
            if let Some(r) = carry(&push, *z, hold) {
                shells.push((*z, a.clone(), r, push.clone()));
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
fn catches(released: &[Released], hold: &Q, s_final: &Q) -> Vec<(usize, usize, Q)> {
    let reach: Vec<Q> = released.iter().map(|(_, _, r, _)| r.value()).collect();
    let state = ReleaseState {
        tight: released
            .iter()
            .map(|(z, _, _, _)| crate::compound::tightness(*z))
            .collect(),
        deficit: reach
            .iter()
            .map(|r| hold.checked_div(&(r + hold)).unwrap_or(Q::ZERO))
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
        .flat_map_iter(|(i, (zi, mi, ri, _))| meetings(released, &state, i, *zi, mi, ri))
        .collect()
}

/// Shell `i`'s meetings with every heavier shell `j < i` (see
/// [`catches`]).
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
    for (j, (zj, mj, rj_pairs, _)) in released.iter().enumerate().take(i) {
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

/// The parcels after the filter: each shell keeps its amount less what it
/// gave, at the shell's release reach; each take is carried by the
/// catcher's push against the caught kind's resistance ([`carry`]), so it stands at
/// `H D(p_catcher) / D(r_m, caught)`. Where the lighter shells ask more of
/// a shell than it holds, every take from it is scaled by holding over
/// asked, so none gives more than it has. Caught matter is not caught again
/// in the same founding.
fn transfer(released: &[Released], takes: Vec<(usize, usize, Q)>, hold: &Q) -> Vec<(u32, Parcel)> {
    let mut asked: Vec<Q> = vec![Q::ZERO; released.len()];
    for (_, j, take) in &takes {
        asked[*j] = &asked[*j] + take;
    }
    let mut given: Vec<Q> = vec![Q::ZERO; released.len()];
    let mut caught: Vec<(u32, Parcel)> = Vec::new();
    for (i, j, take) in takes {
        let (has, all) = (&released[j].1, &asked[j]);
        let take = if all > has {
            nucleus_grain(&(&take * has).checked_div(all).unwrap_or(Q::ZERO))
        } else {
            take
        };
        let z = released[j].0;
        let Some(pairs) = carry(&released[i].3, z, hold) else {
            continue;
        };
        if take.signum() <= 0 {
            continue;
        }
        given[j] = &given[j] + &take;
        caught.push((
            z,
            Parcel {
                amount: take,
                pairs,
            },
        ));
    }
    released
        .iter()
        .zip(given)
        .map(|((z, m, r, _), g)| {
            (
                *z,
                Parcel {
                    amount: m - &g,
                    pairs: r.clone(),
                },
            )
        })
        .chain(caught)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A lone kind holds the whole chain, so `H = 2` and its parcel rests at
    /// the antipolar face of its reach: `(Q* / Z) R = H`, the reach's pairs
    /// swapped, exact.
    #[test]
    fn a_lone_kind_rests_at_the_antipole_of_its_reach() {
        for z in [1u32, 2, 26, 92, 1092] {
            let (_, shells, table) = founding_shells(&[(z, Q::ONE)]);
            assert_eq!(
                &table[&z][0].pairs.value() * &shells[0].2,
                Q::int(2),
                "Z {z}"
            );
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
            assert!(table.get(z).is_some_and(|p| !p.is_empty()), "Z {z}");
        }
    }
}
