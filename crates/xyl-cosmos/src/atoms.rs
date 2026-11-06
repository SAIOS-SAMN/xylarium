//! The atom: the electron structure of a neutral atom of Z charges, filled
//! electron by electron on S^11, and the chemistry read from it (bonding
//! capacity, tightness, stripping by received turning, bonds, light holding).
//!
//! Reads only `compound::is_cluster`, `elements::TOP_Z` and `fibres::directions`; all counts are
//! exact integers and tightnesses exact `Q`, in hydrogen units (hydrogen's
//! lone electron = 1). Each atom is filled once and cached per process.
//!
//! Model: a band (n, l) is the family of windings closing in n ticks with
//! shape l (0 <= l < n). Its orientations are the degree-l harmonics on
//! S^11, `fibres::directions(l) = C(l + 11, 11) - C(l + 9, 11)` = 1, 12, 77, 352,
//! ...; each holds one counter-turning pair along its Hopf fibre, so a band
//! holds twice its orientations and shells hold 2, 26, 180, 884. `atom`
//! adds each electron to the tightest open band by the ring-density law:
//! band (n, l) reaches (n + 1)(l + 1), the electrons of every other shell m
//! with m (m + 1) below that reach shield it, and its tightness is
//! (Z - inside) / n; exact ties go to the lower (n, l). Free orientations
//! fill before pairing. Shell crossings (from Z 31) follow from this rule.
//!
//! Outputs: `Atom` (bands, outer shell, `unpaired`, `tightness`
//! (Z - inner electrons) / n_out, `closed`, `sealed`), `unpaired`, `unpaired_hot` (strips
//! every band held no tighter than the isoclinic turning received, capped at 1), `vessel`, `bond`
//! (the two faces whole: each face's pole push on the axis plane composed by
//! the flow law, each face's polarity, and every open winding held against
//! the isoclinic turning) and `hold` (a
//! cluster's light holding from stretches, bends and dipole). `atom` returns
//! None for Z = 0 or a cluster id. Seals fall at Z = 2, 28, 208, 1092;
//! `vessel` returns (lower, upper): the last seal within `TOP_Z`, and the
//! heaviest kind below it whose single-nucleon peel releases,
//! u(z - 1) + 1 - u(z) < 0. Terms: seal, vessel (docs/glossary.md).

use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// One band: its turning n, its pivot's degree l and the electrons it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Band {
    /// Turning count: the whole number of ticks in which the band's orbit
    /// closes (1, 2, ...).
    pub n: u32,
    /// Shape degree of the band's windings (its eccentricity), 0 <= l < n.
    pub l: u32,
    /// Electrons occupying the band, at most [`Band::room`].
    pub electrons: u32,
}

impl Band {
    /// Capacity: one counter-turning pair along each orientation's fibre.
    pub fn room(&self) -> u32 {
        2 * crate::fibres::directions(self.l)
    }

    /// Unpaired electrons: free orientations fill first, then pairs.
    pub fn unpaired(&self) -> u32 {
        let frames = crate::fibres::directions(self.l);
        if self.electrons <= frames {
            self.electrons
        } else {
            self.room() - self.electrons
        }
    }
}

impl Band {
    /// The band's axis polarity (T7): the sum of the axis weights `m0` of its
    /// unpaired electrons. The axis level `m0` holds `C(l - |m0| + 9, 9)`
    /// orientations; a band at most half full fills the Z side (`m0 < 0`)
    /// first, singly, and one more than half full has paired the Z side and
    /// keeps its unpaired electrons on the Q side.
    pub fn axis_polarity(&self) -> i64 {
        let l = i64::from(self.l);
        let frames = crate::fibres::directions(self.l);
        let (mut left, z_first) = if self.electrons <= frames {
            (u64::from(self.electrons), true)
        } else {
            (u64::from(self.room() - self.electrons), false)
        };
        let mut a = 0i64;
        for step in 0..=2 * l {
            let m0 = if z_first { step - l } else { l - step };
            let level = crate::fibres::choose(u64::try_from(l - m0.abs()).unwrap_or(0) + 9, 9);
            let k = level.min(left);
            a += i64::try_from(k).unwrap_or(0) * m0;
            left -= k;
            if left == 0 {
                break;
            }
        }
        a
    }
}

/// An atom's computed structure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Atom {
    /// Atomic number: the centre's charge count, equal to the neutral atom's
    /// electron count.
    pub z: u32,
    /// Occupied bands, ordered by (n, l).
    pub bands: Vec<Band>,
    /// The outermost occupied turning n.
    pub outer_n: u32,
    /// Electrons in the outermost turning `outer_n`.
    pub outer: u32,
    /// Unpaired electrons over the open bands: bonding capacity.
    pub unpaired: u32,
    /// How tightly the outermost electrons are held: the charge they enclose,
    /// Z minus the electrons of shallower turnings, over `outer_n` (exact).
    pub tightness: Q,
}

impl Atom {
    /// The charge band `b`'s windings enclose, by the fill law's shielding
    /// ([`shielded`]): Z less the electrons of every other shell that lies
    /// inside its reach.
    pub fn winding_hold(&self, b: &Band) -> u32 {
        let others = self.bands.iter().map(|c| (c.n, c.electrons));
        self.z.saturating_sub(shielded(others, b.n, b.l))
    }

    /// How tightly band `b` is held: its enclosed charge over its turning
    /// `n`, in hydrogen units (exact), the fill law's tightness.
    pub fn winding_tightness(&self, b: &Band) -> Q {
        Q::new(i64::from(self.winding_hold(b)), i64::from(b.n)).unwrap_or(Q::ZERO)
    }

    /// Band `b`'s push (T20): the deficit share its enclosed charge `H`
    /// keeps at its reach `r = Z (n + 1)(l + 1) / 2`, `H / (r + H)` (axiom 2).
    /// The fill law counts reach in units of hold (the ring-density law,
    /// `R_n = R_(n-1) + n H`), so the two are read in one unit. Hydrogen's
    /// lone winding pushes one half, the throat's push.
    pub fn winding_push(&self, b: &Band) -> Q {
        let hold = Q::int(i64::from(self.winding_hold(b)));
        let reach = Q::new(
            i64::from(self.z) * i64::from(b.n + 1) * i64::from(b.l + 1),
            2,
        )
        .unwrap_or(Q::ZERO);
        xyl_core::shares::deficit(&hold, &reach).unwrap_or(Q::ZERO)
    }

    /// Every frame paired: the atom bonds nothing.
    pub fn closed(&self) -> bool {
        self.unpaired == 0
    }

    /// Every occupied band and every shape of the outermost shell full: an
    /// inert gas.
    pub fn sealed(&self) -> bool {
        self.bands.iter().all(|b| b.electrons == b.room())
            && (0..self.outer_n).all(|l| {
                self.bands
                    .iter()
                    .any(|b| b.n == self.outer_n && b.l == l && b.electrons == b.room())
            })
    }
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
    // A cluster is not an atom: its properties come from its parts (compound.rs).
    if z == 0 || crate::compound::is_cluster(z) {
        return None;
    }
    // Each Z is filled once and cached: fusion products 2Z may lie past any
    // table, and the fill is expensive to repeat.
    static TABLE: std::sync::OnceLock<std::sync::Mutex<std::collections::BTreeMap<u32, Atom>>> =
        std::sync::OnceLock::new();
    let table = TABLE.get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()));
    let mut t = table.lock().ok()?;
    Some(t.entry(z).or_insert_with(|| fill(z)).clone())
}

/// The turning a body receives from the pulse, one component per orthogonal
/// set of planes (model.md, Chemistry from the elements; T18): the axis
/// plane's dilation, which carries the pole state (the nuclear side), and
/// the turning of the five isoclinic planes, which strips the electrons'
/// windings (the atomic side), the five alike in founded matter. Both are
/// read per echo, the isoclinic part in hydrogen units (the turning that
/// strips hydrogen's lone winding is 1), the axis part as a push.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Received {
    /// The axis plane's push per echo: the star's compactness for a star.
    pub axis: Q,
    /// The isoclinic planes' turning per echo, in hydrogen units.
    pub isoclinic: Q,
}

impl Received {
    /// A body that receives nothing on either set of planes.
    pub fn none() -> Received {
        Received {
            axis: Q::ZERO,
            isoclinic: Q::ZERO,
        }
    }

    /// A body that receives `turning` on the isoclinic planes and nothing on
    /// the axis plane.
    pub fn isoclinic(turning: Q) -> Received {
        Received {
            axis: Q::ZERO,
            isoclinic: turning,
        }
    }
}

/// Stripping by received turning (T20): a winding is held while the push
/// the isoclinic planes receive, `q` (a share of a turn per echo), is at
/// most the winding's own push `delta_w` ([`Atom::winding_push`]); the two
/// oppose and compose by the flow law, so past it the winding departs
/// ([`departure`]). Returns the unpaired count of the windings still held:
/// with no turning received every atom is whole, and a push of one or more,
/// outside the flow law's domain, strips every winding.
pub fn unpaired_hot(z: u32, received: &Received) -> u32 {
    let q = &received.isoclinic;
    if q.signum() <= 0 {
        return unpaired(z);
    }
    let Some(a) = atom(z) else { return 0 };
    a.bands
        .iter()
        .filter(|b| departure(&a.winding_push(b), q).is_none())
        .map(Band::unpaired)
        .sum()
}

/// The echoes after which a winding of push `delta_w` departs under a
/// received push `q` (T20): none while `q <= delta_w`, the winding held;
/// else `ln 2 / ln(D(q) / D(delta_w))` (T15's form at the atomic tier), and
/// zero, at once, for `q` of one or more. Dyadic logarithms.
pub fn departure(delta_w: &Q, q: &Q) -> Option<Dy> {
    if q <= delta_w {
        return None;
    }
    let (Some(dq), Some(dw)) = (
        xyl_core::shares::dilation(q),
        xyl_core::shares::dilation(delta_w),
    ) else {
        return Some(Dy::ZERO);
    };
    let ratio = (Dy::from_q(&dq) / Dy::from_q(&dw)).ln()?;
    Some(Dy::int(2).ln()? / ratio)
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

/// A kind's axis polarity `A` (T7): the sum of its bands' axis polarities,
/// Z side negative, Q side positive. `None` for a cluster id or Z = 0.
pub fn axis_polarity(z: u32) -> Option<i64> {
    atom(z).map(|a| a.bands.iter().map(Band::axis_polarity).sum())
}

/// The fill law's shielding of the winding `(n, l)`: the electrons of every
/// other shell `m` whose `m (m + 1)` lies below the winding's reach
/// `(n + 1)(l + 1)` (the ring-density law, `Z / 2` cancelled), given the
/// occupied `(shell, electrons)` pairs.
fn shielded(occupied: impl Iterator<Item = (u32, u32)>, n: u32, l: u32) -> u32 {
    let reach = u64::from(n + 1) * u64::from(l + 1);
    occupied
        .filter(|&(m, _)| m != n && u64::from(m) * u64::from(m + 1) < reach)
        .map(|(_, e)| e)
        .sum()
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
                if have >= 2 * crate::fibres::directions(l) {
                    continue;
                }
                let inside = shielded((1..=nmax + 1).map(|m| (m, shell_count(&occ, m))), n, l);
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

/// A bond between two atoms: a paired throat at the atomic tier, its two
/// atoms its faces (model.md, Chemistry from the elements). It keeps both
/// faces whole and is read by planes, never by one tightness:
/// - on the axis plane, each face's pole push `(U - W) / (U + W)` from its
///   frames (axiom 9), composed across the pair by the flow law, and each
///   face's axis polarity, kept apart;
/// - on the isoclinic planes, every open winding of each face with its
///   tightness, each held against the received turning;
/// - between planes, nothing in founded matter, whose five isoclinic planes
///   turn together (T18).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bond {
    /// The two atoms, the bond's faces.
    pub faces: [Atom; 2],
}

/// One open winding of a face: its band and how much of it a received
/// turning leaves held.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindingHold {
    /// The band's turning count.
    pub n: u32,
    /// The band's shape degree.
    pub l: u32,
    /// Its unpaired electrons.
    pub unpaired: u32,
    /// Its push, the deficit share its enclosed charge keeps at its reach
    /// ([`Atom::winding_push`]).
    pub push: Q,
    /// The echoes after which the received turning strips it ([`departure`]);
    /// `None` while it is held.
    pub departs: Option<Dy>,
}

impl Bond {
    /// Each face's pole push on the axis plane, `(U - W) / (U + W)`, with `U`
    /// its open frames and `W = 3Z - U` its held frames.
    pub fn pole_pushes(&self) -> [Q; 2] {
        self.faces.clone().map(|a| {
            let u = Q::int(i64::from(a.unpaired));
            let w = &Q::int(3 * i64::from(a.z)) - &u;
            (&u - &w).checked_div(&(&u + &w)).unwrap_or(Q::ZERO)
        })
    }

    /// The bond's separation on the axis plane: the first face's pole push
    /// against the second's, composed by the flow law, `p_a (+) (-p_b)`.
    /// `None` where a push lies on the boundary of the flow law's domain.
    pub fn axis_separation(&self) -> Option<Q> {
        let [a, b] = self.pole_pushes();
        xyl_core::shares::flow(&a, &-b)
    }

    /// Each face's axis polarity (T7), kept apart.
    pub fn polarities(&self) -> [Option<i64>; 2] {
        self.faces.clone().map(|a| axis_polarity(a.z))
    }

    /// Each face's open windings and the share of each the isoclinic
    /// turning `received` leaves held.
    pub fn held(&self, received: &Received) -> [Vec<WindingHold>; 2] {
        self.faces.clone().map(|a| {
            a.bands
                .iter()
                .filter(|b| b.unpaired() > 0)
                .map(|b| {
                    let push = a.winding_push(b);
                    WindingHold {
                        n: b.n,
                        l: b.l,
                        unpaired: b.unpaired(),
                        departs: departure(&push, &received.isoclinic),
                        push,
                    }
                })
                .collect()
        })
    }
}

/// The bond between atoms `a` and `b` (see [`Bond`]).
pub fn bond(a: &Atom, b: &Atom) -> Bond {
    Bond {
        faces: [a.clone(), b.clone()],
    }
}

/// How a cluster holds outgoing light: every motion that swings charge
/// holds light, counted per motion of the cluster.
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
    // The shape (straight or bent) and the dipole the cluster's symmetry
    // leaves uncancelled.
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
    // Twists are bonds between two inner atoms. A cluster is a star about the
    // first atom with every other atom an arm, so no bond joins two inner
    // atoms; twists apply only to chains, which are not yet modelled.
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
        // The orientations are the degree-l harmonics in twelve dimensions.
        assert_eq!(
            (
                crate::fibres::directions(0),
                crate::fibres::directions(1),
                crate::fibres::directions(2),
                crate::fibres::directions(3)
            ),
            (1, 12, 77, 352)
        );
        // The sealed kinds fill whole shells (rooms 2, 26, 180).
        let sealed: Vec<u32> = (1..=300)
            .filter(|&z| atom(z).is_some_and(|a| a.sealed()))
            .collect();
        assert_eq!(sealed, vec![2, 28, 208]);
        // The ring-density fill's crossings begin at Z 31: a deeper shell's
        // round winding penetrates and opens ahead of the shell beneath it.
        let d = |z| atom(z).unwrap().outer_n;
        assert_eq!((d(28), d(31), d(33)), (2, 4, 5));
        // Open shells hold many unpaired electrons: twelve at shell 2's half,
        // seventy-seven at most.
        assert_eq!(atom(16).unwrap().unpaired, 12);
        assert_eq!((1..=300).map(|z| atom(z).unwrap().unpaired).max(), Some(77));
    }

    #[test]
    fn clusters_hold_light_by_their_asymmetry() {
        // On S^11 oxygen has four unpaired electrons (its closing with four
        // hydrogens is `compound::closing`).
        assert_eq!(atom(8).unwrap().unpaired, 4);
        // A symmetric pair swings no charge and holds no light; a mixed cluster
        // holds some.
        assert_eq!(hold(&[7, 7]), Q::ZERO);
        assert!(hold(&[8, 1, 1, 1, 1]) > Q::ZERO);
    }

    use crate::fibres::choose;

    #[test]
    fn a_bands_harmonics_split_by_weight_on_the_six_planes() {
        // R^12 = C^6: the axis plane and the five isoclinic planes. A
        // degree-l harmonic of weight m = (m0, ..., m5) occurs C(q + 4, 4)
        // times, q = (l - sum |mi|) / 2; summed over every weight it is the
        // band's orientations. Enumerated over the weights for l <= 6.
        for l in 0..=6u32 {
            let mut total = 0u64;
            let mut stack = vec![(0usize, 0u32)];
            while let Some((plane, used)) = stack.pop() {
                if plane == 6 {
                    if (l - used) % 2 == 0 {
                        total += choose(u64::from((l - used) / 2) + 4, 4);
                    }
                    continue;
                }
                for a in 0..=(l - used) {
                    // |mi| = a: one weight for a = 0, two (+a, -a) otherwise.
                    for _ in 0..(if a == 0 { 1 } else { 2 }) {
                        stack.push((plane + 1, used + a));
                    }
                }
            }
            assert_eq!(total, u64::from(crate::fibres::directions(l)), "l = {l}");
        }
        // By axis weight alone, M_l(m0) = C(l - |m0| + 9, 9), summing to the
        // orientations for every band of the table.
        for l in 0..=18u32 {
            let by_axis: u64 = (0..=l)
                .map(|a| choose(u64::from(l - a) + 9, 9) * if a == 0 { 1 } else { 2 })
                .sum();
            assert_eq!(by_axis, u64::from(crate::fibres::directions(l)), "l = {l}");
        }
    }
}
