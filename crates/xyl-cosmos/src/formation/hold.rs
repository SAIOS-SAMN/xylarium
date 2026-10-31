//! The star tier's cut: which kinds of an infall a core holds at its heat
//! and what it releases to the disc (see the parent module's header).

use super::{mass_weight, weight};
use xyl_core::Q;

/// What a core (a star's or a planetary core) holds at its heat: the share
/// of its matter held, and the released remainder.
///
/// The star tier's cut: the core has no bounce, only torsion; its push is
/// the electromagnetic repulsion on open frames. Each kind of the infall
/// (`mix`: element and share of the infall's holding) is carried to spin
/// l_z = p P_0 (e_z / e_mean)(w_mean / m_z) + `add` - p the core's push
/// (1/2), P_0 the core's over-pressure, e_z the kind's open share, m_z its
/// mass per nucleus, means by count, `add` a turning the matter brings (a
/// world's heat) - and stands at R_z = l_z / H about the core's hold. It is
/// held (the star) where H / (R_z + H) >= p, else carried into the disc.
/// Noble kinds (e = 0) carry no spin and fall in; an infall with no open
/// frames is all star.
#[derive(Clone, Debug)]
pub struct Hold {
    /// The share held.
    pub held: Q,
    /// The matter's mean open share, by count.
    pub open: Q,
    /// Mean open share of the released part.
    pub rest_open: Q,
    /// The released part's mix (element, share of it).
    pub rest: Vec<(u32, Q)>,
    /// The lightest kind held.
    pub lightest: Option<u32>,
    /// Each released kind's carried spin (aligned with `rest`).
    pub rest_l: Vec<Q>,
    /// The held core's extent: the reach of the loosest kind still held,
    /// R = l / H* on the held hold, in the core's units. A core holding
    /// everything is large; one that has released its light kinds is compact.
    pub reach: Q,
}

/// What a core holds at its heat. Each kind's open share is taken after the
/// heat strips its loosely held windings (atoms::unpaired_hot; a cold core
/// passes heat 0), so the push grips only what stays open. The core's light
/// adds over-pressure `p0` = P_0 = 1 + PHI / H (1 for a dark core), carrying
/// every kind out by P_0 times its push while the held threshold stays p: a
/// dark core holds its whole infall and a transduction one releases kinds, the
/// lightest first.
pub fn hold_at_heat(mix: &[(u32, Q)], p: &Q, add: &Q, heat: &Q, p0: &Q) -> Hold {
    hold_about(mix, p, add, &Q::ZERO, heat, p0)
}

/// [`hold_at_heat`] about a core that already holds `state`.
fn hold_about(mix: &[(u32, Q)], p: &Q, add: &Q, state: &Q, heat: &Q, p0: &Q) -> Hold {
    let open_of = |z: u32| {
        Q::int(i64::from(crate::compound::open_hot(z, heat)))
            .checked_div(&weight(z))
            .unwrap_or(Q::ZERO)
    };
    let kinds: Vec<(u32, Q, Q, Q)> = mix
        .iter()
        .map(|(z, share)| (*z, share.clone(), weight(*z), open_of(*z)))
        .filter(|(_, _, w, _)| w.signum() > 0)
        .collect();
    let all = |open: Q| Hold {
        held: Q::ONE,
        open,
        rest_open: Q::ZERO,
        rest: Vec::new(),
        lightest: kinds.iter().min_by(|a, b| a.2.cmp(&b.2)).map(|k| k.0),
        rest_l: Vec::new(),
        reach: Q::ZERO,
    };
    let count = kinds.iter().fold(Q::ZERO, |a, (_, s, w, _)| {
        &a + &s.checked_div(w).unwrap_or(Q::ZERO)
    });
    if count.signum() <= 0 {
        return all(Q::ZERO);
    }
    // The mean mass per nucleus w_mean (the carriage's inertia) counts nuclei
    // as share / hold, each weighed by its mass.
    let w_mean = kinds
        .iter()
        .fold(Q::ZERO, |a, (z, sh, w, _)| {
            &a + &(&sh.checked_div(w).unwrap_or(Q::ZERO) * &mass_weight(*z))
        })
        .checked_div(&count)
        .unwrap_or(Q::ONE);
    let e_mean = kinds
        .iter()
        .fold(Q::ZERO, |a, (_, s, w, e)| {
            &a + &(&s.checked_div(w).unwrap_or(Q::ZERO) * e)
        })
        .checked_div(&count)
        .unwrap_or(Q::ZERO);
    if e_mean.signum() <= 0 && add.signum() <= 0 {
        return all(Q::ZERO);
    }
    // Each kind's carried spin is l_z = p P_0 (e_z / e_mean)(w_mean / m_z) + add.
    let mut carried: Vec<(Q, Q, Q, u32, Q)> = kinds
        .iter()
        .map(|(z, s, w, e)| {
            let pushed = &(&(p * p0) * &e.checked_div(&e_mean).unwrap_or(Q::ZERO))
                * &w_mean.checked_div(&mass_weight(*z)).unwrap_or(Q::ZERO);
            (&pushed + add, s.clone(), e.clone(), *z, w.clone())
        })
        .collect();
    carried.sort_by(|a, b| a.0.cmp(&b.0));
    let cum: Vec<Q> = carried
        .iter()
        .scan(Q::ZERO, |a, k| {
            *a = &*a + &k.1;
            Some(a.clone())
        })
        .collect();
    let (lo, h) = held_set(&carried, state, p);
    let held = if lo == 0 {
        Q::ZERO
    } else {
        cum[lo - 1].clone()
    };
    let rest_share = &Q::ONE - &held;
    let rest_open = carried[lo..]
        .iter()
        .fold(Q::ZERO, |a, (_, s, e, _, _)| &a + &(s * e))
        .checked_div(&rest_share)
        .unwrap_or(Q::ZERO);
    let rest = carried[lo..]
        .iter()
        .map(|(_, s, _, z, _)| (*z, s.checked_div(&rest_share).unwrap_or(Q::ZERO)))
        .collect();
    let lightest = carried[..lo]
        .iter()
        .min_by(|a, b| a.4.cmp(&b.4))
        .map(|k| k.3);
    let rest_l = carried[lo..]
        .iter()
        .map(|(l, _, _, _, _)| l.clone())
        .collect();
    let reach = carried[..lo]
        .iter()
        .filter_map(|k| k.0.checked_div(&h))
        .max()
        .unwrap_or(Q::ZERO);
    Hold {
        held,
        open: e_mean,
        rest_open,
        rest,
        lightest,
        rest_l,
        reach,
    }
}

/// The held set of `carried` (sorted by carried spin `l`, shares in `.1`):
/// a kind is held where the deficit at its reach `R = l / h` about the held
/// hold still meets the push, `h / (R + h) >= p`. The hold is the largest
/// fixed point `h = state + the shares of every kind so held`, iterated from
/// all held downward; each step can only drop kinds, so it ends within as
/// many steps as kinds. A core holds only by what it has gathered (only a
/// throat has a nonzero `state`). Returns the count held (a prefix of
/// `carried`; 0 where the hold is not positive) and the hold.
fn held_set(carried: &[(Q, Q, Q, u32, Q)], state: &Q, p: &Q) -> (usize, Q) {
    let holds = |l: &Q, h: &Q| {
        let r = l.checked_div(h).unwrap_or(Q::ZERO);
        h.checked_div(&(&r + h)).unwrap_or(Q::ZERO) >= *p
    };
    let total_s = carried.iter().fold(Q::ZERO, |a, k| &a + &k.1);
    let mut h = state + &total_s;
    let mut lo = carried.len();
    for _ in 0..=carried.len() {
        let n_held = carried.iter().take_while(|k| holds(&k.0, &h)).count();
        let h_next = state + &carried[..n_held].iter().fold(Q::ZERO, |a, k| &a + &k.1);
        lo = n_held;
        if h_next == h {
            break;
        }
        h = h_next;
    }
    if h.signum() <= 0 {
        lo = 0;
    }
    (lo, h)
}
