//! Compounds at the atom tier: two-part clusters of atoms, their encoding as
//! kind ids, and every cluster property computed from its parts.
//!
//! Reads `atoms` (`unpaired`, `unpaired_hot`, `atom`, `bond`); heat is in
//! the units `atoms::unpaired_hot` takes. Counts are exact `u32`, shares `Q`.
//!
//! - `closing`: the smallest closing cluster of two atoms with pairable
//!   frames u_A, u_B at a heat is (u_B / g) A . (u_A / g) B,
//!   g = gcd(u_A, u_B), every frame paired; hydrides are u_B = 1, water
//!   O . 4H. Returns (id, counts, grip) or None if either pairs nothing.
//! - Grip is `atoms::bond`'s tau_min, the weaker tightness; `held` gives the
//!   share of a meeting's contact held, min(1, tau_min / heat).
//! - `stands`: a cluster stands while the heat strips neither part's open
//!   frames; an atom always stands.
//! - Properties: `charge` = sum n Z; `bonds` = the lesser side's n u;
//!   `open` = sum n u - 2 bonds; `hold` = 3 charge - open, so
//!   hold + open = 3 charge for every kind.
//!
//! Encoding: an atom's id is its Z; a cluster's id is at or above `CLUSTER`
//! (2^31), the parts' Z (each below 1024) and counts (each below 45) in mixed
//! radix. `cluster` returns None outside those bounds. `key` writes parts by
//! Z descending, each "\[n\]Zddd" (count 1 omitted), joined by U+00B7 (middle
//! dot): water is "Z008", U+00B7, "4Z001".

use xyl_core::Q;

/// The first cluster id; every atom's id is its Z, below it.
pub const CLUSTER: u32 = 1 << 31;
/// Representation bounds on a cluster's parts (an encoding limit, not a law:
/// a kind beyond them forms no cluster). Ids are `u32`, so with `CLUSTER` at
/// 2^31 the mixed radix must fit in 2^31: `Z_LIM^2 N_LIM^2 < 2^31`. Z to
/// 1023 covers the vessel kinds; 45 is the largest count that then fits.
const Z_LIM: u32 = 1024;
const N_LIM: u32 = 45;

/// Whether a kind is a cluster.
pub fn is_cluster(k: u32) -> bool {
    k >= CLUSTER
}

/// A plain atom's Z (None for a cluster or for 0): the kinds that may burn, ionize or
/// be cut by their windings.
pub fn atom_z(k: u32) -> Option<u32> {
    (!is_cluster(k) && k > 0).then_some(k)
}

/// The cluster of `na` of `za` and `nb` of `zb`, ordered by Z descending.
/// `None` beyond the id's encoding: a part's Z at or above `Z_LIM` (1024)
/// or a count at or above `N_LIM` (45); the id is a `u32`, so a wider kind
/// space needs a wider id, not a larger limit here.
pub fn cluster(za: u32, na: u32, zb: u32, nb: u32) -> Option<u32> {
    let ((za, na), (zb, nb)) = if za >= zb {
        ((za, na), (zb, nb))
    } else {
        ((zb, nb), (za, na))
    };
    if za >= Z_LIM || zb >= Z_LIM || zb == 0 || na == 0 || nb == 0 || na >= N_LIM || nb >= N_LIM {
        return None;
    }
    Some(CLUSTER + ((za * Z_LIM + zb) * N_LIM + na) * N_LIM + nb)
}

/// A kind's parts: (Z, count), the heavier first; an atom is the single part
/// (Z, 1), followed by (0, 0).
pub fn parts(k: u32) -> [(u32, u32); 2] {
    if !is_cluster(k) {
        return [(k, 1), (0, 0)];
    }
    let x = k - CLUSTER;
    let (nb, x) = (x % N_LIM, x / N_LIM);
    let (na, x) = (x % N_LIM, x / N_LIM);
    let (zb, za) = (x % Z_LIM, x / Z_LIM);
    [(za, na), (zb, nb)]
}

/// A kind's charge: the protons of all its parts.
pub fn charge(k: u32) -> u32 {
    parts(k).iter().map(|(z, n)| z * n).sum()
}

/// The frames a cluster's parts pair with each other: the lesser side's open
/// frames.
pub fn bonds(k: u32) -> u32 {
    if !is_cluster(k) {
        return 0;
    }
    let [(za, na), (zb, nb)] = parts(k);
    (na * crate::atoms::unpaired(za)).min(nb * crate::atoms::unpaired(zb))
}

/// A kind's open frames: its parts' open frames less two per bond.
pub fn open(k: u32) -> u32 {
    if !is_cluster(k) {
        return crate::atoms::unpaired(k);
    }
    let all: u32 = parts(k)
        .iter()
        .map(|(z, n)| n * crate::atoms::unpaired(*z))
        .sum();
    all - 2 * bonds(k)
}

/// Open frames at a place's heat: an atom's after ionization; a standing
/// cluster keeps the open frames it has at rest, `open` (a cluster that would
/// not stand has already parted).
pub fn open_hot(k: u32, heat: &Q) -> u32 {
    if is_cluster(k) {
        open(k)
    } else {
        crate::atoms::unpaired_hot(k, heat)
    }
}

/// A kind's hold, 3 charge - open: pairing moves frames from the Q pole to
/// the Z pole with the span fixed.
pub fn hold(k: u32) -> u32 {
    3 * charge(k) - open(k)
}

/// How tightly a kind holds its outermost frame: an atom's tightness; for a
/// cluster, the weaker of its parts' (atoms::bond).
pub fn tightness(k: u32) -> Q {
    parts(k)
        .iter()
        .filter(|(z, n)| *z > 0 && *n > 0)
        .filter_map(|(z, _)| crate::atoms::atom(*z))
        .map(|a| a.tightness)
        .min()
        .unwrap_or(Q::ONE)
}

/// Whether a cluster stands at a place's heat: the heat strips neither part's
/// open winding (an atom always stands).
pub fn stands(k: u32, heat: &Q) -> bool {
    !is_cluster(k)
        || parts(k)
            .iter()
            .filter(|(_, n)| *n > 0)
            .all(|(z, _)| crate::atoms::unpaired_hot(*z, heat) == crate::atoms::unpaired(*z))
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// What two atoms meeting at a place's heat close into: (cluster id, count of
/// `za`, count of `zb`, bond grip) for the smallest closing cluster of their
/// pairable frames. None if either has nothing to pair.
pub fn closing(za: u32, zb: u32, heat: &Q) -> Option<(u32, u32, u32, Q)> {
    let (ua, ub) = (
        crate::atoms::unpaired_hot(atom_z(za)?, heat),
        crate::atoms::unpaired_hot(atom_z(zb)?, heat),
    );
    if ua == 0 || ub == 0 {
        return None;
    }
    let g = gcd(ua, ub);
    let (na, nb) = (ub / g, ua / g);
    let id = cluster(za, na, zb, nb)?;
    let (a, b) = (crate::atoms::atom(za)?, crate::atoms::atom(zb)?);
    Some((id, na, nb, crate::atoms::bond(&a, &b).grip))
}

/// The share of a meeting's contact a bond holds at a place's heat: 1 within
/// its grip, else grip / heat (the remainder parts).
pub fn held(grip: &Q, heat: &Q) -> Q {
    if heat <= grip {
        Q::ONE
    } else {
        grip.checked_div(heat).unwrap_or(Q::ZERO)
    }
}

/// The kind's string key (see the module docs).
pub fn key(k: u32) -> String {
    let [(za, na), (zb, nb)] = parts(k);
    let ps = if za == zb {
        vec![(za, na + nb)]
    } else {
        vec![(za, na), (zb, nb)]
    };
    ps.iter()
        .filter(|(_, n)| *n > 0)
        .map(|(z, n)| {
            if *n == 1 {
                format!("Z{z:03}")
            } else {
                format!("{n}Z{z:03}")
            }
        })
        .collect::<Vec<_>>()
        .join("\u{b7}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_closes_from_its_parts_and_parts_by_heat() {
        // Oxygen opens four frames on S^11 and hydrogen one, so the smallest
        // closing cluster is O . 4H.
        let (w, no, nh, grip) = closing(8, 1, &Q::ZERO).unwrap();
        assert_eq!((no, nh, key(w)), (1, 4, "Z008\u{b7}4Z001".to_string()));
        assert_eq!((charge(w), open(w), bonds(w)), (12, 0, 4));
        // The identity hold + open = 3 charge holds for the cluster as for its
        // parts.
        assert_eq!(hold(w) + open(w), 3 * 12);
        assert_eq!(grip, Q::ONE);
        // The cluster stands at heat 0; at heat 1 hydrogen's winding is
        // stripped and the cluster parts.
        assert!(stands(w, &Q::ZERO) && !stands(w, &Q::ONE));
        // Sealed kinds pair nothing; any other pair closes by the gcd.
        assert!(closing(28, 1, &Q::ZERO).is_none());
        let (c, ..) = closing(8, 8, &Q::ZERO).unwrap();
        assert_eq!((key(c), open(c)), ("2Z008".to_string(), 0));
        assert!(atom_z(w).is_none() && atom_z(8) == Some(8));
    }
}
