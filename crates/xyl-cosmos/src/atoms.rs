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
//! cluster's light holding from stretches, bends and dipole), `capacity`
//! (`cap`, the room of every turning reached; `3Q = 3 cap`) and `balance`
//! (`R_Q = 6Z - 3 cap`, `rho_Q = (2Z - cap) / cap`), and `occupancy` /
//! `band_occupancy` (axiom 9: each band's room, open states and pairs, over
//! every band reached; `cap = E + U + 2P`), `Band::plane_levels` and
//! `plane_sides` (each of the six Hopf planes by its charge `m_p` and by side,
//! #201), `plane_weights` (the open and held states' weights in QQ per plane)
//! and `plane_polarities` (`A_p`, plane 0 the axis plane). `atom` returns
//! None for Z = 0 or a cluster id. Seals fall at Z = 2, 28, 208, 1092;
//! `vessel` returns (lower, upper): the last seal within `TOP_Z`, and the
//! heaviest kind below it whose single-nucleon peel releases,
//! u(z - 1) + 1 - u(z) < 0. Terms: seal, vessel (docs/glossary.md).

use xyl_core::hopf::PLANES;
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

/// A share of a kind's capacity on the capacity's account (axiom 9,
/// docs/model.md, The capacity's occupancy): its empty slots, its open states
/// (single orientations) and its pairs (paired orientations), so that
/// `capacity = room + open + 2 pairs`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Occupancy {
    /// Empty slots: the room `E`.
    pub room: u32,
    /// Single orientations: the open states `U`.
    pub open: u32,
    /// Paired orientations `P`; the held states are `2P`.
    pub pairs: u32,
}

impl Occupancy {
    /// The held states, `2P`.
    pub fn held(&self) -> u32 {
        2 * self.pairs
    }

    /// The occupied states, `U + 2P`: the electrons.
    pub fn occupied(&self) -> u32 {
        self.open + self.held()
    }

    /// The capacity, `E + U + 2P`.
    pub fn capacity(&self) -> u32 {
        self.room + self.occupied()
    }

    /// Two occupancies summed.
    pub fn plus(self, o: Occupancy) -> Occupancy {
        Occupancy {
            room: self.room + o.room,
            open: self.open + o.open,
            pairs: self.pairs + o.pairs,
        }
    }
}

impl Band {
    /// Capacity: one counter-turning pair along each orientation's fibre.
    pub fn room(&self) -> u32 {
        2 * crate::fibres::directions(self.l)
    }

    /// The band's occupancy: free orientations fill singly before pairing,
    /// so up to `d(l)` electrons are all open; beyond, `e - d` orientations
    /// are paired and `2d - e` stay single. The empty slots are the room
    /// left.
    pub fn occupancy(&self) -> Occupancy {
        let d = crate::fibres::directions(self.l);
        let e = self.electrons.min(self.room());
        let (open, pairs) = if e <= d { (e, 0) } else { (2 * d - e, e - d) };
        Occupancy {
            room: self.room() - e,
            open,
            pairs,
        }
    }

    /// Unpaired electrons, the band's open states: free orientations fill
    /// first, then pairs ([`Band::occupancy`]).
    pub fn unpaired(&self) -> u32 {
        self.occupancy().open
    }
}

/// One level `m` of a band on one Hopf plane, the plane's charge `m_p`, its
/// electrons held as exact shares (#201): its orientations, its open states
/// and its pairs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaneLevel {
    /// The plane's charge, from `-l` to `l`.
    pub m: i64,
    /// The level's orientations, `C(l - |m| + 9, 9)` on every plane.
    pub orientations: Q,
    /// Its open (single) states.
    pub open: Q,
    /// Its pairs.
    pub pairs: Q,
}

impl PlaneLevel {
    /// The level's occupied states, `open + 2 pairs`.
    pub fn occupied(&self) -> Q {
        &self.open + &(&self.pairs * &Q::int(2))
    }
}

/// `amount` laid into `caps` from both ends as exact shares: `from_z` from
/// the first level upward, `from_q` from the last downward, each level taking
/// at most what the other front has left in it. The amounts never exceed the
/// caps' sum, so the fronts meet without crossing.
fn fronts(caps: &[Q], from_z: &Q, from_q: &Q) -> Vec<Q> {
    let mut out = vec![Q::ZERO; caps.len()];
    let mut lay = |order: &mut dyn Iterator<Item = usize>, amount: &Q| {
        let mut left = amount.clone();
        for i in order {
            if left.signum() <= 0 {
                break;
            }
            let room = &caps[i] - &out[i];
            let k = room.min(left.clone());
            out[i] = &out[i] + &k;
            left = &left - &k;
        }
    };
    lay(&mut (0..caps.len()), from_z);
    lay(&mut (0..caps.len()).rev(), from_q);
    out
}

impl Band {
    /// The band on one Hopf plane, level by level, filled from both sides
    /// (#201): the level `m` (the plane's charge, from `-l` to `l`) holds
    /// `C(l - |m| + 9, 9)` orientations, the same count on every plane, since
    /// a harmonic's multiplicity reads only `sum |m_p|`. The share `from_q`
    /// of the fill enters from the plane's Q side (`m = l`) toward Z and the
    /// rest from its Z side (`m = -l`) toward Q, as exact shares, never
    /// floored: singles first while the band is at most half full, then
    /// pairs, each laid from both ends. `from_q` is the plane's face of the
    /// electron phase's channel (`circuit::fill_shares`); at 0 this is the
    /// Z-side-first fill. `None` for a share outside `[0, 1]`.
    pub fn plane_levels(&self, from_q: &Q) -> Option<Vec<PlaneLevel>> {
        if from_q.signum() < 0 || from_q > &Q::ONE {
            return None;
        }
        let from_z = &Q::ONE - from_q;
        let l = i64::from(self.l);
        let d = crate::fibres::directions(self.l);
        let e = self.electrons.min(self.room());
        let caps: Vec<Q> = (-l..=l)
            .map(|m| {
                let level = crate::fibres::choose(u64::try_from(l - m.abs()).unwrap_or(0) + 9, 9);
                Q::int(i64::try_from(level).unwrap_or(i64::MAX))
            })
            .collect();
        let split = |amount: u32| {
            let a = Q::int(i64::from(amount));
            fronts(&caps, &(&a * &from_z), &(&a * from_q))
        };
        let (open, pairs) = if e <= d {
            (split(e), vec![Q::ZERO; caps.len()])
        } else {
            // Every orientation holds one electron; the rest pair from both
            // sides.
            let pairs = split(e - d);
            let open = caps.iter().zip(&pairs).map(|(c, k)| c - k).collect();
            (open, pairs)
        };
        Some(
            (-l..=l)
                .zip(caps)
                .zip(open.into_iter().zip(pairs))
                .map(|((m, orientations), (open, pairs))| PlaneLevel {
                    m,
                    orientations,
                    open,
                    pairs,
                })
                .collect(),
        )
    }

    /// The band's weights in QQ on one Hopf plane (#158, vent 3; #201): its
    /// open states' weight `sum m U_m` and its held states' `sum 2 m P_m`,
    /// filled with the share `from_q` from the plane's Q side
    /// ([`Band::plane_levels`]). A pair's two electrons share the level `m`
    /// and weigh `2 m` on the plane.
    pub fn plane_weights(&self, from_q: &Q) -> Option<(Q, Q)> {
        Some(
            self.plane_levels(from_q)?
                .iter()
                .fold((Q::ZERO, Q::ZERO), |(open, held), lv| {
                    let m = Q::int(lv.m);
                    (
                        &open + &(&m * &lv.open),
                        &held + &(&(&m * &lv.pairs) * &Q::int(2)),
                    )
                }),
        )
    }

    /// The band's polarity on one Hopf plane (T7): the weight of its open
    /// states there ([`Band::plane_weights`]).
    pub fn plane_polarity(&self, from_q: &Q) -> Option<Q> {
        self.plane_weights(from_q).map(|w| w.0)
    }
}

/// A plane reading per Hopf plane (#201): `f` read at each plane's fill
/// share, a share met on an earlier plane read once.
fn per_plane<T: Clone>(from_q: &[Q; PLANES], f: impl Fn(&Q) -> Option<T>) -> Option<[T; PLANES]> {
    let mut read: Vec<(Q, T)> = Vec::with_capacity(PLANES);
    for s in from_q {
        if !read.iter().any(|(q, _)| q == s) {
            read.push((s.clone(), f(s)?));
        }
    }
    let at = |s: &Q| read.iter().find(|(q, _)| q == s).map(|(_, t)| t.clone());
    let out: Vec<T> = from_q.iter().map(at).collect::<Option<_>>()?;
    out.try_into().ok()
}

/// A kind's weights in QQ on each Hopf plane over every band it reaches
/// (#158, vent 3; #201): per plane `p`, the open states' `sum m U_m`, the
/// plane polarity `A_p`, and the held states' `sum 2 m P_m`, with the share
/// `from_q[p]` filled from the plane's Q side. `None` for z = 0, a cluster or
/// a share outside `[0, 1]`.
pub fn plane_weights(z: u32, from_q: &[Q; PLANES]) -> Option<[(Q, Q); PLANES]> {
    let bands = band_occupancy(z)?;
    per_plane(from_q, |s| {
        bands.iter().try_fold((Q::ZERO, Q::ZERO), |(o, h), (b, _)| {
            let (bo, bh) = b.plane_weights(s)?;
            Some((&o + &bo, &h + &bh))
        })
    })
}

/// A kind's states on each Hopf plane by side, summed over every band it
/// reaches (#158, vent 2; #201), as exact shares `(open, pairs)`: per plane
/// `p`, the Z side (`m < 0`), the neutral level (`m = 0`) and the Q side
/// (`m > 0`), with the share `from_q[p]` filled from the plane's Q side.
/// Every plane counts its levels alike, so the planes part only by their
/// fill shares. `None` for z = 0, a cluster or a share outside `[0, 1]`.
pub fn plane_sides(z: u32, from_q: &[Q; PLANES]) -> Option<[[(Q, Q); 3]; PLANES]> {
    let bands = band_occupancy(z)?;
    let zero = || (Q::ZERO, Q::ZERO);
    per_plane(from_q, |s| {
        bands
            .iter()
            .try_fold([zero(), zero(), zero()], |mut acc, (b, _)| {
                for lv in b.plane_levels(s)? {
                    let side = match lv.m.signum() {
                        -1 => 0,
                        0 => 1,
                        _ => 2,
                    };
                    acc[side] = (&acc[side].0 + &lv.open, &acc[side].1 + &lv.pairs);
                }
                Some(acc)
            })
    })
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
    /// How tightly the outermost electrons are held: the least tightness of
    /// the outermost turning's windings ([`Atom::winding_tightness`]), the
    /// electron the atom holds least (exact).
    pub tightness: Q,
}

impl Atom {
    /// How tightly band `b` is held: the charge its windings enclose over
    /// its turning `n`, in hydrogen units (exact), by the fill's law
    /// (`tightness_at`): the electrons of every other shell standing
    /// inside the winding's reach shield it.
    pub fn winding_tightness(&self, b: &Band) -> Q {
        tightness_at(self.z, b.n, b.l, |m| {
            self.bands
                .iter()
                .filter(|c| c.n == m)
                .map(|c| c.electrons)
                .sum()
        })
    }

    /// Every state paired: the atom bonds nothing.
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
// Stand-in S131: The isoclinic turning is one number for five planes.
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

/// Stripping by received turning: the isoclinic planes' turning strips every
/// winding held no more tightly than it. A winding's tightness, in hydrogen
/// units (hydrogen's lone electron = 1), is the fill's law
/// (`tightness_at`): (Z - the electrons of the shells inside its reach) / n;
/// the isoclinic turning, in the same unit, is what the core
/// receives this echo (capped at 1). Returns the unpaired count of the
/// remaining windings: stripped hydrogen has none, while heavy kinds keep
/// their deep windings. With no turning received every atom is whole.
pub fn unpaired_hot(z: u32, received: &Received) -> u32 {
    let heat = &received.isoclinic;
    if heat.signum() <= 0 {
        return unpaired(z);
    }
    // The isoclinic turning per echo, capped at one (hydrogen's lone
    // electron); windings with tightness at most this are stripped.
    let heat = heat.clone().min(Q::ONE);
    let heat = &heat;
    let Some(a) = atom(z) else { return 0 };
    a.bands
        .iter()
        .filter(|b| a.winding_tightness(b) > *heat)
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

/// A kind's capacity `cap`: the room of every shape of every turning its atom
/// reaches, `sum_(n <= n_out) sum_(l < n) 2 d(l)` (`Band::room`), so that
/// `3Q = 3 cap` stands beside the span's `3Z` (docs/model.md, The dipole).
/// Exact; `None` for z = 0 or a cluster.
pub fn capacity(z: u32) -> Option<u32> {
    let a = atom(z)?;
    (1..=a.outer_n).try_fold(0u32, |acc, n| {
        (0..n).try_fold(acc, |acc, l| {
            acc.checked_add(crate::fibres::directions(l).checked_mul(2)?)
        })
    })
}

/// A kind's occupancy band by band over every band its atom reaches, every
/// `(n, l)` with `n <= n_out` in order, an empty band holding all its room:
/// the capacity's account (axiom 9) before any law sums it. `None` for z = 0
/// or a cluster.
pub fn band_occupancy(z: u32) -> Option<Vec<(Band, Occupancy)>> {
    let a = atom(z)?;
    Some(
        (1..=a.outer_n)
            .flat_map(|n| (0..n).map(move |l| (n, l)))
            .map(|(n, l)| {
                let b = a
                    .bands
                    .iter()
                    .find(|b| b.n == n && b.l == l)
                    .copied()
                    .unwrap_or(Band { n, l, electrons: 0 });
                (b, b.occupancy())
            })
            .collect(),
    )
}

/// A kind's occupancy summed over its bands: the room `E = cap - Z`, the
/// open states `U` and the pairs `P`, `cap = E + U + 2P` (axiom 9). `None`
/// for z = 0 or a cluster.
pub fn occupancy(z: u32) -> Option<Occupancy> {
    Some(
        band_occupancy(z)?
            .iter()
            .fold(Occupancy::default(), |acc, (_, o)| acc.plus(*o)),
    )
}

/// A kind's balance of span against capacity: `R_Q = 6Z - 3 cap` and its
/// share `rho_Q = (2Z - cap) / cap`, which is `+1` exactly where `Z = cap`,
/// the dipole's equator `3Z = 3Q`. Exact; `None` for z = 0 or a cluster.
pub fn balance(z: u32) -> Option<(i64, Q)> {
    let (z, c) = (i64::from(z), i64::from(capacity(z)?));
    let excess = 2 * z - c;
    Some((3 * excess, Q::new(excess, c.max(1)).unwrap_or(Q::ZERO)))
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

/// A kind's plane polarities `A_p` (T7; #201), one per Hopf plane: the sum
/// of its bands' polarities on plane `p`, Z side negative, Q side positive,
/// exact, with the share `from_q[p]` filled from the plane's Q side. Plane 0
/// is the axis plane, where the Z-Q stretch is read. `None` for a cluster
/// id, Z = 0 or a share outside `[0, 1]`.
pub fn plane_polarities(z: u32, from_q: &[Q; PLANES]) -> Option<[Q; PLANES]> {
    let bands = atom(z)?.bands;
    per_plane(from_q, |s| {
        bands
            .iter()
            .try_fold(Q::ZERO, |a, b| Some(&a + &b.plane_polarity(s)?))
    })
}

/// The tightness of a winding of shell `n` and shape `l` in an atom of
/// charge `z`, one law for the fill and every reading after it: the
/// winding comes closest at `(l + 1) / n` of its shell's reach, so (Z / 2
/// cancelled) its reach is `(n + 1)(l + 1)`; the electrons of every other
/// shell `m` with `m (m + 1)` below that reach stand inside it and shield
/// it (`shell(m)` counts them), and its tightness is `(Z - inside) / n`.
/// Same-shell electrons never shield; a low-`l` winding penetrates the
/// shells it reaches past.
fn tightness_at(z: u32, n: u32, l: u32, shell: impl Fn(u32) -> u32) -> Q {
    let reach = u64::from(n + 1) * u64::from(l + 1);
    let inside: u32 = (1..)
        .take_while(|&m: &u32| u64::from(m) * u64::from(m + 1) < reach)
        .filter(|&m| m != n)
        .map(&shell)
        .sum();
    Q::new(i64::from(z) - i64::from(inside), i64::from(n)).unwrap_or(Q::ZERO)
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
                let t = tightness_at(z, n, l, |m| shell_count(&occ, m));
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
    let unpaired = bands.iter().map(Band::unpaired).sum();
    let shell = |m: u32| -> u32 { bands.iter().filter(|b| b.n == m).map(|b| b.electrons).sum() };
    let tightness = bands
        .iter()
        .filter(|b| b.n == outer_n)
        .map(|b| tightness_at(z, b.n, b.l, shell))
        .min()
        .unwrap_or(Q::ZERO);
    Atom {
        z,
        bands,
        outer_n,
        outer,
        unpaired,
        tightness,
    }
}

/// A bond between two atoms: a paired throat at the atomic tier, its two
/// atoms its faces (model.md, Chemistry from the elements). It keeps both
/// faces whole and is read by planes, never by one tightness:
/// - on the axis plane, each face's pole push `(U - W) / (U + W)` from its
///   states (axiom 9), composed across the pair by the flow law, and each
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
    /// The share of it held: 1 within its tightness, else its tightness over
    /// the isoclinic turning (the rule `compound::stands` and
    /// `atoms::unpaired_hot` cut at; the graded stripping law is still to be
    /// derived, #107).
    pub held: Q,
}

impl Bond {
    /// Each face's pole push on the axis plane, `(U - W) / (U + W)`, with `U`
    /// its open states and `W = 3Z - U` its held count.
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

    /// Each face's plane polarities (T7), kept apart, each filled with its
    /// shares `from_q` from the planes' Q sides (`circuit::fill_shares`).
    pub fn polarities(&self, from_q: [&[Q; PLANES]; 2]) -> [Option<[Q; PLANES]>; 2] {
        let [a, b] = &self.faces;
        [
            plane_polarities(a.z, from_q[0]),
            plane_polarities(b.z, from_q[1]),
        ]
    }

    /// Each face's open windings and the share of each the isoclinic
    /// turning `received` leaves held.
    pub fn held(&self, received: &Received) -> [Vec<WindingHold>; 2] {
        let turning = received.isoclinic.clone().min(Q::ONE);
        self.faces.clone().map(|a| {
            a.bands
                .iter()
                .filter(|b| b.unpaired() > 0)
                .map(|b| {
                    let tight = a.winding_tightness(b);
                    let held = if turning <= tight {
                        Q::ONE
                    } else {
                        tight.checked_div(&turning).unwrap_or(Q::ONE)
                    };
                    WindingHold {
                        n: b.n,
                        l: b.l,
                        unpaired: b.unpaired(),
                        held,
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

    /// One tightness law for the fill and every reading: a winding is
    /// shielded only by the shells inside its reach, so it is never looser
    /// than under every shallower shell, the two agree until a winding
    /// penetrates a shell (shell 3 opens at Z 29), and the atom's tightness
    /// is its loosest outer winding.
    #[test]
    fn a_winding_is_shielded_only_inside_its_reach() {
        for z in 1..=1092u32 {
            let a = atom(z).unwrap();
            for b in &a.bands {
                let shallower: u32 = a
                    .bands
                    .iter()
                    .filter(|c| c.n < b.n)
                    .map(|c| c.electrons)
                    .sum();
                let bound = Q::new(i64::from(z) - i64::from(shallower), i64::from(b.n)).unwrap();
                let t = a.winding_tightness(b);
                assert!(t >= bound, "Z {z} ({}, {})", b.n, b.l);
                if z < 29 {
                    assert_eq!(t, bound, "Z {z}");
                }
            }
            let outer = a.bands.iter().filter(|b| b.n == a.outer_n);
            assert_eq!(
                Some(a.tightness.clone()),
                outer.map(|b| a.winding_tightness(b)).min()
            );
        }
        let cu = atom(29).unwrap();
        let s3 = cu.bands.iter().find(|b| b.n == 3 && b.l == 0).unwrap();
        assert_eq!(cu.winding_tightness(s3), Q::int(9), "(29 - 2) / 3");
    }

    /// The axis polarity counted by singles alone: at most half full, the
    /// singles from the Z side; beyond, the `room - e` singles from the Q side.
    fn polarity_by_singles(b: &Band) -> i64 {
        let l = i64::from(b.l);
        let d = crate::fibres::directions(b.l);
        let (mut left, z_first) = if b.electrons <= d {
            (u64::from(b.electrons), true)
        } else {
            (u64::from(b.room() - b.electrons), false)
        };
        let mut a = 0i64;
        for step in 0..=2 * l {
            let m0 = if z_first { step - l } else { l - step };
            let k = choose(u64::try_from(l - m0.abs()).unwrap() + 9, 9).min(left);
            a += i64::try_from(k).unwrap() * m0;
            left -= k;
            if left == 0 {
                break;
            }
        }
        a
    }

    #[test]
    fn each_plane_partitions_each_band() {
        let shares = [
            Q::ZERO,
            xyl_core::ratio!(1, 3),
            xyl_core::ratio!(1, 2),
            xyl_core::ratio!(2, 3),
        ];
        for z in 1..=crate::elements::TOP_Z {
            let (Some(bands), Some(o)) = (band_occupancy(z), occupancy(z)) else {
                continue;
            };
            for from_q in &shares {
                let mirror = &Q::ONE - from_q;
                for (b, ob) in &bands {
                    let levels = b.plane_levels(from_q).unwrap();
                    // The levels hold the band's open states and pairs as
                    // exact shares, each level within its orientations.
                    let open = levels.iter().fold(Q::ZERO, |a, lv| &a + &lv.open);
                    let pairs = levels.iter().fold(Q::ZERO, |a, lv| &a + &lv.pairs);
                    assert_eq!(
                        open,
                        Q::int(i64::from(ob.open)),
                        "Z {z} band {}{}",
                        b.n,
                        b.l
                    );
                    assert_eq!(
                        pairs,
                        Q::int(i64::from(ob.pairs)),
                        "Z {z} band {}{}",
                        b.n,
                        b.l
                    );
                    for lv in &levels {
                        let orientations = choose(u64::from(b.l) - lv.m.unsigned_abs() + 9, 9);
                        assert_eq!(
                            lv.orientations,
                            Q::int(i64::try_from(orientations).unwrap())
                        );
                        assert!(&lv.open + &lv.pairs <= lv.orientations, "Z {z}");
                    }
                    // The fill mirrored across the equator negates the
                    // polarity; an even fill has none.
                    let a = b.plane_polarity(from_q).unwrap();
                    assert_eq!(b.plane_polarity(&mirror).unwrap(), -a.clone(), "Z {z}");
                    if *from_q == xyl_core::ratio!(1, 2) {
                        assert_eq!(a, Q::ZERO, "Z {z}");
                    }
                }
                // Each plane at the share it is given: the open weight is the plane's
                // polarity, a seal is balanced on every plane at every share,
                // and each plane's sides hold the kind's open states and
                // pairs. Planes at equal shares read alike.
                let per: [Q; PLANES] = std::array::from_fn(|p| shares[p % shares.len()].clone());
                let weights = plane_weights(z, &per).unwrap();
                let polarities = plane_polarities(z, &per).unwrap();
                let sides = plane_sides(z, &per).unwrap();
                for p in 0..PLANES {
                    let (open_w, held_w) = &weights[p];
                    assert_eq!(open_w, &polarities[p], "Z {z} plane {p}");
                    if atom(z).is_some_and(|a| a.sealed()) {
                        assert_eq!((open_w, held_w), (&Q::ZERO, &Q::ZERO), "Z {z}");
                    }
                    let open = sides[p].iter().fold(Q::ZERO, |a, s| &a + &s.0);
                    let pairs = sides[p].iter().fold(Q::ZERO, |a, s| &a + &s.1);
                    assert_eq!(
                        (open, pairs),
                        (Q::int(i64::from(o.open)), Q::int(i64::from(o.pairs)))
                    );
                    if per[p] == *from_q {
                        assert_eq!(
                            plane_polarities(z, &std::array::from_fn(|_| from_q.clone())).unwrap()
                                [p],
                            polarities[p],
                            "Z {z} plane {p}"
                        );
                    }
                }
            }
            // With no share from the Q side the fill is the Z-side-first
            // fill, whose polarity the singles alone give.
            for (b, _) in &bands {
                assert_eq!(
                    b.plane_polarity(&Q::ZERO),
                    Some(Q::int(polarity_by_singles(b))),
                    "Z {z}"
                );
            }
        }
        assert!(bands_refuse_a_share_outside_the_unit());
    }

    fn bands_refuse_a_share_outside_the_unit() -> bool {
        band_occupancy(26).is_some_and(|bs| {
            bs.iter().all(|(b, _)| {
                b.plane_levels(&Q::int(2)).is_none() && b.plane_levels(&Q::int(-1)).is_none()
            })
        })
    }

    #[test]
    fn the_occupancy_partitions_the_capacity() {
        for z in 1..=crate::elements::TOP_Z {
            let (Some(a), Some(o), Some(cap)) = (atom(z), occupancy(z), capacity(z)) else {
                continue;
            };
            // The capacity is E + U + 2P, the electrons U + 2P, the open states
            // the atom's unpaired count and the room cap - Z.
            assert_eq!(o.capacity(), cap, "Z {z}");
            assert_eq!(o.occupied(), z, "Z {z}");
            assert_eq!(o.open, a.unpaired, "Z {z}");
            assert_eq!(o.open, unpaired(z), "Z {z}");
            assert_eq!(o.room, cap - z, "Z {z}");
            // Each band partitions the band's room; a seal has no room and no
            // open state.
            for (b, ob) in band_occupancy(z).unwrap() {
                assert_eq!(ob.capacity(), b.room(), "Z {z} band {}{}", b.n, b.l);
            }
            if a.sealed() {
                assert_eq!((o.room, o.open), (0, 0), "Z {z}");
            }
        }
    }

    #[test]
    fn the_seals_are_where_the_span_meets_the_capacity() {
        for z in 1..=crate::elements::TOP_Z {
            let (Some(a), Some(cap)) = (atom(z), capacity(z)) else {
                continue;
            };
            let (r, rho) = balance(z).unwrap();
            // The balance is three times the span's excess over the capacity.
            assert_eq!(r, 3 * (2 * i64::from(z) - i64::from(cap)));
            // The balance share is +1 exactly where Z = cap, the dipole's
            // equator 3Z = 3Q.
            assert_eq!(rho == Q::ONE, z == cap, "Z {z}");
            // A seal fills its capacity and holds no open state: its pole
            // point is the Z pole, t = u / w = 0, not the throat.
            assert_eq!(a.sealed(), z == cap, "Z {z}");
            if a.sealed() {
                assert!(a.closed(), "Z {z}");
            }
        }
    }

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

    #[test]
    fn every_plane_counts_its_levels_alike() {
        // The multiplicity C(q + 4, 4) reads only sum |m_p|, so the count of
        // harmonics with charge m on plane p is the same for every p: the
        // marginal on each plane is M_l(m) = C(l - |m| + 9, 9). Enumerated
        // over the weights for l <= 5, plane by plane.
        for l in 0..=5u32 {
            let li = i64::from(l);
            let mut by_plane = vec![std::collections::BTreeMap::<i64, u64>::new(); PLANES];
            let mut stack = vec![(Vec::<i64>::new(), 0u32)];
            while let Some((m, used)) = stack.pop() {
                if m.len() == PLANES {
                    if (l - used) % 2 == 0 {
                        let mult = choose(u64::from((l - used) / 2) + 4, 4);
                        for (p, mp) in m.iter().enumerate() {
                            *by_plane[p].entry(*mp).or_default() += mult;
                        }
                    }
                    continue;
                }
                for a in -(li - i64::from(used))..=(li - i64::from(used)) {
                    let mut next = m.clone();
                    next.push(a);
                    stack.push((next, used + u32::try_from(a.unsigned_abs()).unwrap()));
                }
            }
            for (p, counts) in by_plane.iter().enumerate() {
                for m in -li..=li {
                    let want = choose(u64::try_from(li - m.abs()).unwrap() + 9, 9);
                    assert_eq!(
                        counts.get(&m).copied().unwrap_or(0),
                        want,
                        "l {l} plane {p}"
                    );
                }
            }
        }
    }
}
