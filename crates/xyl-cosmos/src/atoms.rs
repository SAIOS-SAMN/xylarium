//! The atom: the electron structure of a neutral atom of Z charges, filled
//! electron by electron on S^11, and the chemistry read from it (bonding
//! capacity, tightness, ionization by heat, bonds, light holding).
//!
//! Reads only `compound::is_cluster` and `elements::TOP_Z`; all counts are
//! exact integers and tightnesses exact `Q`, in hydrogen units (hydrogen's
//! lone electron = 1). Each atom is filled once and cached per process.
//!
//! Model: a band (n, l) is the family of windings closing in n ticks with
//! shape l (0 <= l < n). Its orientations are the degree-l harmonics on
//! S^11, `orientations(l) = C(l + 11, 11) - C(l + 9, 11)` = 1, 12, 77, 352,
//! ...; each holds one counter-turning pair along its Hopf fibre, so a band
//! holds twice its orientations and shells hold 2, 26, 180, 884. `atom`
//! adds each electron to the tightest open band by the ring-density law:
//! band (n, l) reaches (n + 1)(l + 1), the electrons of every other shell m
//! with m (m + 1) below that reach shield it, and its tightness is
//! (Z - inside) / n; exact ties go to the lower (n, l). Free orientations
//! fill before pairing. Shell crossings (from Z 31) follow from this rule.
//!
//! Outputs: `Atom` (bands, outer shell, `unpaired`, `tightness`
//! n_out / P_out, `closed`, `sealed`), `unpaired`, `unpaired_hot` (strips
//! every band held no tighter than the heat, capped at 1), `vessel`, `bond`
//! (shared share tau_min / tau_max, grip tau_min), `closes` and `hold` (a
//! cluster's light holding from stretches, bends and dipole). `atom` returns
//! None for Z = 0 or a cluster id. Seals fall at Z = 2, 28, 208, 1092;
//! `vessel` returns (lower, upper): the last seal within `TOP_Z`, and the
//! heaviest kind below it whose single-nucleon peel releases,
//! u(z - 1) + 1 - u(z) < 0. Terms: seal, vessel (docs/glossary.md).

use xyl_core::Q;

/// One band: its turning n, its pivot's degree l and the electrons it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Band {
    /// Turning count: the whole number of ticks in which the band's orbit closes (1, 2, ...).
    pub n: u32,
    /// Shape degree of the band's windings (its eccentricity), 0 <= l < n.
    pub l: u32,
    /// Electrons occupying the band, at most [`Band::room`].
    pub electrons: u32,
}

impl Band {
    /// Capacity: one counter-turning pair along each orientation's fibre.
    pub fn room(&self) -> u32 {
        2 * orientations(self.l)
    }

    /// Unpaired electrons: free orientations fill first, then pairs.
    pub fn unpaired(&self) -> u32 {
        let frames = orientations(self.l);
        if self.electrons <= frames {
            self.electrons
        } else {
            self.room() - self.electrons
        }
    }
}

/// An atom's computed structure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Atom {
    /// Atomic number: the centre's charge count, equal to the neutral atom's electron count.
    pub z: u32,
    /// Occupied bands, ordered by (n, l).
    pub bands: Vec<Band>,
    /// The outermost turning, and the electrons sharing it.
    pub outer_n: u32,
    /// Electrons in the outermost turning `outer_n`.
    pub outer: u32,
    /// Unpaired electrons over the open bands: bonding capacity.
    pub unpaired: u32,
    /// How tightly the outermost electrons are held: n_out / P_out (exact).
    pub tightness: Q,
}

impl Atom {
    /// Every frame paired: the atom bonds nothing.
    pub fn closed(&self) -> bool {
        self.unpaired == 0
    }

    /// Every occupied band and every shape of the outermost shell full: an
    /// inert gas.
    pub fn sealed(&self) -> bool {
        // every occupied winding full, and every shape of its outermost shell full
        self.bands.iter().all(|b| b.electrons == b.room())
            && (0..self.outer_n).all(|l| {
                self.bands
                    .iter()
                    .any(|b| b.n == self.outer_n && b.l == l && b.electrons == b.room())
            })
    }
}

/// The independent turnings of shape `l` on S^11: the degree-l harmonics in
/// twelve dimensions, C(l + 11, 11) - C(l + 9, 11) (exact, topological).
pub fn orientations(l: u32) -> u32 {
    let c = |n: u64, k: u64| -> u64 {
        if k > n {
            return 0;
        }
        (1..=k).fold(1u64, |acc, i| acc * (n - k + i) / i)
    };
    let l = u64::from(l);
    let all = c(l + 11, 11);
    let lower = if l >= 2 { c(l + 9, 11) } else { 0 };
    u32::try_from(all - lower).unwrap_or(u32::MAX)
}

/// The neutral atom of `z` charges (`None` for z = 0 or a cluster). The fill
/// adds electrons one by one, each to the tightest open winding. Shells nest
/// by the ring-density law, R_n = R_(n-1) + n H(<r) with H = Z, so
/// R_n = Z n (n + 1) / 2; a winding (n, l) comes closest at (l + 1) / n of its
/// shell's reach, so (Z / 2 cancelled) its reach is (n + 1)(l + 1). The
/// electrons inside it are those of every other shell m with m (m + 1) below
/// that reach; its tightness is (Z - inside) / n. Shells up to one past the
/// deepest occupied are open; an exact tie goes to the lower (n, l). Shell
/// crossings follow from the law rather than a stated order.
pub fn atom(z: u32) -> Option<Atom> {
    // a cluster is not an atom: its properties come from its parts (compound.rs)
    if z == 0 || crate::compound::is_cluster(z) {
        return None;
    }
    // each Z is filled once and cached (fusion products 2Z may exceed any
    // table; refilling on every call is expensive)
    static TABLE: std::sync::OnceLock<std::sync::Mutex<std::collections::BTreeMap<u32, Atom>>> =
        std::sync::OnceLock::new();
    let table = TABLE.get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()));
    let mut t = table.lock().ok()?;
    Some(t.entry(z).or_insert_with(|| fill(z)).clone())
}

/// Ionization by heat: a core strips every winding held no more tightly than
/// the core's turning. A winding's tightness, in hydrogen units (hydrogen's
/// lone electron = 1), is (Z - electrons of every shallower shell) / n; the
/// heat, in the same unit, is the core's turning this echo (capped at 1).
/// Returns the unpaired count of the remaining windings: stripped hydrogen
/// has none, while heavy kinds keep their deep windings. At heat 0 every atom
/// is whole.
pub fn unpaired_hot(z: u32, heat: &Q) -> u32 {
    if heat.signum() <= 0 {
        return unpaired(z);
    }
    // the core's turning in whole turns of its bell, at most one: it strips
    // windings held as loosely as hydrogen's lone electron or looser; deeper
    // windings, held tighter than one turn, remain
    let heat = heat.clone().min(Q::ONE);
    let heat = &heat;
    let Some(a) = atom(z) else { return 0 };
    a.bands
        .iter()
        .filter(|b| {
            let shallower: u32 = a
                .bands
                .iter()
                .filter(|c| c.n < b.n)
                .map(|c| c.electrons)
                .sum();
            let tight =
                Q::new(i64::from(z) - i64::from(shallower), i64::from(b.n)).unwrap_or(Q::ZERO);
            // stripped at or below the heat (heat 1 strips hydrogen's 1)
            tight > *heat
        })
        .map(Band::unpaired)
        .sum()
}

/// The vessel: the band of kinds no passing light shell can grip, derived
/// from the atom alone. Upper edge: the last sealed kind the table holds
/// (every shell full; the fourth seal of S^11, Z 1092). Lower edge: the
/// heaviest seam below it, a kind whose single-nucleon peel releases,
/// u(z - 1) + 1 - u(z) < 0. Between them windings are held tightly; past the
/// sealed shell a new shell opens and kinds grip at about 1. Both edges are
/// computed, not constants. The universal core holds this band as its
/// firmament: the heavier the seal, the deeper the firmament lies and the
/// less of it the bell's swing carries past the throat.
pub fn vessel() -> Option<(u32, u32)> {
    static V: std::sync::OnceLock<Option<(u32, u32)>> = std::sync::OnceLock::new();
    *V.get_or_init(|| {
        let top = crate::elements::TOP_Z;
        let sealed: Vec<u32> = (1..=top)
            .filter(|&z| atom(z).is_some_and(|a| a.sealed()))
            .collect();
        let upper = *sealed.last()?;
        let u = |z: u32| i64::from(unpaired(z));
        let lower = (2..upper).filter(|&z| u(z - 1) + 1 - u(z) < 0).max()?;
        Some((lower, upper))
    })
}

/// A kind's unpaired count (used by its weight and open share), cached
/// separately to avoid cloning the atom.
pub fn unpaired(z: u32) -> u32 {
    static UN: std::sync::OnceLock<std::sync::Mutex<std::collections::BTreeMap<u32, u32>>> =
        std::sync::OnceLock::new();
    let table = UN.get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()));
    if let Ok(t) = table.lock() {
        if let Some(&u) = t.get(&z) {
            return u;
        }
    }
    let u = atom(z).map(|a| a.unpaired).unwrap_or(0);
    if let Ok(mut t) = table.lock() {
        t.insert(z, u);
    }
    u
}

fn fill(z: u32) -> Atom {
    use std::collections::BTreeMap;
    let mut occ: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for _ in 0..z {
        let deepest = occ.keys().map(|&(n, _)| n).max().unwrap_or(0);
        let nmax = deepest + 1;
        let shell_count = |occ: &BTreeMap<(u32, u32), u32>, m: u32| -> u32 {
            occ.iter()
                .filter(|((n, _), _)| *n == m)
                .map(|(_, v)| *v)
                .sum()
        };
        let mut best: Option<(Q, u32, u32)> = None;
        for n in 1..=nmax {
            for l in 0..n {
                let have = occ.get(&(n, l)).copied().unwrap_or(0);
                if have >= 2 * orientations(l) {
                    continue;
                }
                let reach = u64::from(n + 1) * u64::from(l + 1);
                let inside: u32 = (1..=nmax + 1)
                    .filter(|&m| m != n && u64::from(m) * u64::from(m + 1) < reach)
                    .map(|m| shell_count(&occ, m))
                    .sum();
                let t = Q::new(i64::from(z) - i64::from(inside), i64::from(n)).unwrap_or(Q::ZERO);
                let better = match &best {
                    None => true,
                    Some((bt, _, _)) => t > *bt,
                };
                if better {
                    best = Some((t, n, l));
                }
            }
        }
        if let Some((_, n, l)) = best {
            *occ.entry((n, l)).or_insert(0) += 1;
        }
    }
    let bands: Vec<Band> = occ
        .iter()
        .filter(|(_, v)| **v > 0)
        .map(|(&(n, l), &v)| Band { n, l, electrons: v })
        .collect();
    let outer_n = bands.iter().map(|b| b.n).max().unwrap_or(1);
    let outer: u32 = bands
        .iter()
        .filter(|b| b.n == outer_n)
        .map(|b| b.electrons)
        .sum();
    let inner: u32 = bands
        .iter()
        .filter(|b| b.n < outer_n)
        .map(|b| b.electrons)
        .sum();
    let unpaired = bands.iter().map(Band::unpaired).sum();
    let enclosed = i64::from(z) - i64::from(inner);
    Atom {
        z,
        bands,
        outer_n,
        outer,
        unpaired,
        tightness: Q::new(enclosed, i64::from(outer_n)).unwrap_or(Q::ZERO),
    }
}

/// A bond between two atoms, from the ratio of their tightnesses: the shared
/// share s = tau_min / tau_max (1 fully shared, near 0 transferred to the
/// tighter atom), and the grip tau_min, the weaker hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bond {
    /// Shared share in (0, 1]: tau_min / tau_max; 1 is fully shared, near 0 is
    /// transferred to the tighter atom.
    pub shared: Q,
    /// Grip: tau_min, the smaller of the two tightnesses (the weaker hold).
    pub grip: Q,
}

/// The bond between atoms `a` and `b`, from their tightnesses (see [`Bond`]).
/// When the larger tightness is zero the shared share is taken as 1.
pub fn bond(a: &Atom, b: &Atom) -> Bond {
    let (lo, hi) = if a.tightness <= b.tightness {
        (&a.tightness, &b.tightness)
    } else {
        (&b.tightness, &a.tightness)
    };
    Bond {
        shared: lo.checked_div(hi).unwrap_or(Q::ONE),
        grip: lo.clone(),
    }
}

/// Whether a cluster (compound) closes: every unpaired electron pairs with
/// one on another atom and the bonds hold it together. Requires no closed
/// atom, an even unpaired total, no atom with more than all the others
/// combined, and at least atoms - 1 bonds. Exact, no search.
pub fn closes(zs: &[u32]) -> bool {
    let Some(counts) = zs
        .iter()
        .map(|&z| atom(z).map(|a| a.unpaired))
        .collect::<Option<Vec<u32>>>()
    else {
        return false;
    };
    if zs.len() < 2 || counts.contains(&0) {
        return false;
    }
    let total: u32 = counts.iter().sum();
    let most = counts.iter().copied().max().unwrap_or(0);
    total % 2 == 0
        && most <= total - most
        && total / 2 + 1 >= u32::try_from(zs.len()).unwrap_or(u32::MAX)
}

/// How a cluster holds outgoing light: every motion that swings charge
/// holds light, in units of the cluster's own motions.
/// - In plane: stretches, one per bond, each holding the most unequal pair's
///   transfer, 1 - tau_min / tau_max; bends, one per atom beyond the second,
///   each holding the cluster's mixture (twice the share of atoms not of the
///   most common kind, at most 1), since a bend makes a dipole whenever the
///   cluster has more than one kind of atom.
/// - Perpendicular: spin about the cluster's axes (two if straight, three if
///   bent), each holding the permanent dipole left by its symmetry; and
///   twists, torsion about each bond between two inner atoms (none in a star
///   about one centre), each holding the mixture.
///
/// The shape follows from the atoms: the centre (first atom) keeps lone
/// pairs, (outer electrons - electrons bonded by the arms) / 2, which bend
/// it and leave its dipole uncancelled. A centre with no lone pairs and like
/// arms is symmetric (CO2 straight, CH4 a tetrahedron) and has no spin term.
/// A pair of unlike atoms is straight with its transfer as its dipole.
pub fn hold(zs: &[u32]) -> Q {
    let atoms: Vec<Atom> = zs.iter().filter_map(|&z| atom(z)).collect();
    let (Some(lo), Some(hi)) = (
        atoms.iter().map(|a| &a.tightness).min(),
        atoms.iter().map(|a| &a.tightness).max(),
    ) else {
        return Q::ZERO;
    };
    let asymmetry = &Q::ONE - &lo.checked_div(hi).unwrap_or(Q::ONE);
    let k = i64::try_from(atoms.len()).unwrap_or(0);
    let (stretches, bends) = ((k - 1).max(0), (k - 2).max(0));
    let share_not_most = |of: &[Atom]| {
        let most = of
            .iter()
            .map(|a| of.iter().filter(|b| b.z == a.z).count())
            .max()
            .unwrap_or(0);
        let n = i64::try_from(of.len()).unwrap_or(0);
        Q::new(2 * (n - i64::try_from(most).unwrap_or(0)), n.max(1))
            .unwrap_or(Q::ZERO)
            .min(Q::ONE)
    };
    let mixture = share_not_most(&atoms);
    // the shape: straight or bent, and the dipole its symmetry leaves
    let (axes, dipole) = match atoms.as_slice() {
        [] | [_] => (0, Q::ZERO),
        [_, _] => (2, asymmetry.clone()),
        [centre, arms @ ..] => {
            let bonded: u32 = arms.iter().map(|a| a.unpaired).sum();
            let lone = i64::from(centre.outer.saturating_sub(bonded) / 2);
            let n_arms = i64::try_from(arms.len()).unwrap_or(0);
            let bent = Q::new(lone, lone + n_arms).unwrap_or(Q::ZERO);
            let unlike_arms = share_not_most(arms);
            let left = if bent > unlike_arms {
                bent
            } else {
                unlike_arms
            };
            let axes = if lone == 0 && n_arms == 2 { 2 } else { 3 };
            (axes, &asymmetry * &left)
        }
    };
    // twists: bonds between two inner atoms. A cluster is a star about its
    // first atom with every other atom an arm, so no bond joins two inner
    // atoms; twists apply only to chains, which are not yet modelled
    let twists = 0i64;
    let in_plane = &(&asymmetry * &Q::int(stretches)) + &(&mixture * &Q::int(bends));
    let perpendicular = &(&dipole * &Q::int(axes)) + &(&mixture * &Q::int(twists));
    &in_plane + &perpendicular
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_atom_on_s11_closes_where_its_shells_fill() {
        // orientations: the degree-l harmonics in twelve dimensions
        assert_eq!(
            (
                orientations(0),
                orientations(1),
                orientations(2),
                orientations(3)
            ),
            (1, 12, 77, 352)
        );
        // full shells (rooms 2, 26, 180): the sealed kinds
        let sealed: Vec<u32> = (1..=300)
            .filter(|&z| atom(z).is_some_and(|a| a.sealed()))
            .collect();
        assert_eq!(sealed, vec![2, 28, 208]);
        // the ring-density fill's crossings emerge from Z 31: a deeper shell's
        // round winding penetrates and opens ahead of the shell beneath it
        let d = |z| atom(z).unwrap().outer_n;
        assert_eq!((d(28), d(31), d(33)), (2, 4, 5));
        // wide open shells: twelve unpaired at shell 2's half, seventy-seven at most
        assert_eq!(atom(16).unwrap().unpaired, 12);
        assert_eq!((1..=300).map(|z| atom(z).unwrap().unpaired).max(), Some(77));
    }

    #[test]
    fn clusters_close_and_hold_light_by_their_asymmetry() {
        // on S^11 oxygen opens four (never two): it closes with four
        // hydrogens, not two; a closed kind (Z 28) bonds nothing
        assert_eq!(atom(8).unwrap().unpaired, 4);
        assert!(
            closes(&[8, 1, 1, 1, 1]) && closes(&[1, 1]) && !closes(&[8, 1, 1]) && !closes(&[28, 1])
        );
        // a symmetric pair swings no charge and holds no light; a mixed one holds some
        assert_eq!(hold(&[7, 7]), Q::ZERO);
        assert!(hold(&[8, 1, 1, 1, 1]) > Q::ZERO);
    }
}
