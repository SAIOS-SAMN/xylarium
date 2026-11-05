//! Chronometry: the keepers that count time at the epicenter and at each
//! galaxy's hub and shells, and the hashed chain of their readings.
//!
//! Inputs: the seed and [`Origin`] (galactic core shapes from
//! `xyl_cosmos::cores`), galaxy radii and enclosed masses
//! (`xyl_cosmos::galaxy`), the pulse wavelength (`xyl_cosmos::pulse`) and
//! the white-hole units (`xyl_cosmos::chain::units`, G = c = M_core = 1,
//! lengths and masses in r_g).
//!
//! - Keepers ([`Chronometry::of`]): the epicenter (the universe core, rate
//!   1, no delay), then per galaxy a hub and three [`Shell`]s (deep core at
//!   the nuclear-cluster radius, inner mantle at the bulge, outer rim at the
//!   disc edge). Each counts in units of its ring, the echo period of the
//!   enclosure it keeps time in.
//! - Rate: a layer at radius r with M(<r) inside runs at r / (r + M(<r)) of
//!   the layer above it; nested wells multiply, and a galaxy's well holds
//!   the cores nearer the epicenter than it.
//! - Reading ([`Keeper::read`]) at the epicenter's count N (bells): local =
//!   rate (N - delay) / ring; slip = (1 - rate)(N - delay) / ring;
//!   coherence = 1 - 2 |slip - round(slip)|; rung = floor(slip). N - delay
//!   is clamped at zero.
//! - Chain ([`Chronometry::commit`]): every keeper appends an [`Entry`]
//!   hashed (BLAKE3) over its reading, its residual slip against the keeper
//!   inside it (kept, not averaged), its previous entry and the inner
//!   keeper's head (rim -> mantle -> core -> hub -> epicenter).
//! - Network points ([`Point`]): the epicenter's entry records the whole
//!   last TUniversal final point heard (its origin, its height, its lanes
//!   S G T H as exact shares and K); [`Chronometry::anchor`] is ring 0's
//!   point, the point heard with the origin receipt, so every count of the
//!   bell has a place on TUniversal: the anchor plus its rings.
//! - Dilations ([`Chronometry::dilations`]): the binding to the network is
//!   on the height `h`; for each segment between consecutive commits under
//!   one origin the dilation is the rings over the heights there, read,
//!   never fixed and never computed across a change of origin
//!   (MU-CHRONOMETRY 3.3).
//!
//! Every quantity is an exact `Q`. Keepers and entries are recomputed from
//! the seed; the commit counts, their points and the anchor are held in the
//! checkpoint.
//!
//! Terms: TUniversal (docs/glossary.md).

use xyl_core::units::Tier;
use xyl_core::Q;
use xyl_cosmos::origin::Origin;

/// A TUniversal final point as heard from the network: its origin (the
/// network's genesis and epoch), its height `h`, its four lanes
/// `[S, G, T, H]` as exact shares and `K`. Carried unchanged; the kernel
/// reads only the origin and `h` (for [`Chronometry::dilations`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Point {
    /// The network's genesis.
    pub genesis: [u8; 32],
    /// The network's epoch.
    pub epoch: u32,
    /// The final point's height.
    pub h: u64,
    /// The lanes S, G, T and H, exact.
    pub lanes: [Q; 4],
    /// The point's K.
    pub k: u64,
}

impl Point {
    /// The point's bytes as hashed in the chain: the genesis, the epoch, `h`
    /// and `K` little-endian, then each lane's exact text with a zero after
    /// it.
    pub fn bytes(&self) -> Vec<u8> {
        let mut out = self.genesis.to_vec();
        out.extend_from_slice(&self.epoch.to_le_bytes());
        out.extend_from_slice(&self.h.to_le_bytes());
        out.extend_from_slice(&self.k.to_le_bytes());
        for l in &self.lanes {
            out.extend_from_slice(l.to_string().as_bytes());
            out.push(0);
        }
        out
    }

    /// Whether two points share an origin: the same genesis and epoch.
    pub fn same_origin(&self, other: &Point) -> bool {
        self.genesis == other.genesis && self.epoch == other.epoch
    }
}

/// One segment of the binding between two consecutive commits that heard a
/// point under one origin, with the height advanced: the universe's rings
/// over the network's heights there, read, never fixed (MU-CHRONOMETRY 3.3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    /// The heights at the segment's two ends.
    pub from_h: u64,
    /// The later height.
    pub to_h: u64,
    /// Epicenter count advance (bells) over the segment.
    pub rings: Q,
    /// The dilation `rings / (to_h - from_h)`, exact.
    pub dilation: Q,
}

/// Where a keeper stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shell {
    /// The universe core, counting in the universe's echo at rate 1 with no
    /// delay.
    Epicenter,
    /// A galactic core as its galaxy's hub: the galaxy's link to the
    /// epicenter, counting in the galaxy's echo.
    Hub,
    /// A galaxy's deep core, at its nuclear-cluster radius.
    Core,
    /// A galaxy's inner mantle, at its bulge radius.
    Mantle,
    /// A galaxy's outer rim, at its disc edge.
    Rim,
}

impl Shell {
    /// Human-readable name of the shell, as used in reports ("epicenter",
    /// "galaxy hub", "deep core", "inner mantle", "outer rim").
    pub fn name(self) -> &'static str {
        match self {
            Shell::Epicenter => "epicenter",
            Shell::Hub => "galaxy hub",
            Shell::Core => "deep core",
            Shell::Mantle => "inner mantle",
            Shell::Rim => "outer rim",
        }
    }
}

/// A keeper: its galaxy (None at the epicenter), shell, distance from the
/// epicenter (kpc), rate and echo delay (bells), all exact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keeper {
    /// Index of the keeper's galaxy core; `None` for the epicenter.
    pub galaxy: Option<u32>,
    /// Which shell of its galaxy (or the epicenter) the keeper stands at.
    pub shell: Shell,
    /// Distance from the epicenter in kpc: the galaxy core's reach along the
    /// core's bearing plus the shell's radius.
    pub from_epicenter_kpc: Q,
    /// Clock rate relative to the epicenter, in (0, 1]: the product of the
    /// nested wells r / (r + M(<r)).
    pub rate: Q,
    /// Travel time of the pulse from the epicenter to the keeper, in bells:
    /// the distance over the pulse wavelength.
    pub delay: Q,
    /// The keeper's ring, in the engine's bells: the echo period of the
    /// enclosure it keeps time in (the universe's sphere for the epicenter,
    /// the galaxy's sphere for a hub and its shells).
    pub ring: Q,
}

/// What a keeper reads at the epicenter's count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    /// Local count in the keeper's rings: rate x (N - delay) / ring.
    pub local: Q,
    /// Turns fallen behind the epicenter: (1 - rate) x (N - delay) / ring.
    pub slip: Q,
    /// Phase agreement with the epicenter in \[0, 1\]:
    /// 1 - 2 |slip - round(slip)| (1 in phase, 0 half a turn out).
    pub coherence: Q,
    /// Whole turns of slip (the floor of `slip`); 0 for a keeper at rate 1.
    pub rung: i128,
}

impl Keeper {
    /// The keeper's count, slip, coherence and rung, in the keeper's rings,
    /// at the engine's count `n` (bells). A zero ring gives a zero count.
    pub fn read(&self, n: &Q) -> Reading {
        let since = (n - &self.delay).max(Q::ZERO);
        let own = since.checked_div(&self.ring).unwrap_or(Q::ZERO);
        let local = &self.rate * &own;
        let slip = &(&Q::ONE - &self.rate) * &own;
        let rung = slip.floor_i128().unwrap_or(0);
        let frac = &slip - &Q::from_i128(rung, 1).unwrap_or(Q::ZERO);
        let off = if frac > Q::new(1, 2).unwrap_or(Q::ZERO) {
            &Q::ONE - &frac
        } else {
            frac
        };
        Reading {
            local,
            slip,
            coherence: &Q::ONE - &(&Q::int(2) * &off),
            rung,
        }
    }
}

/// One entry of the hashed chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Index of the keeper in [`Chronometry::keepers`] that wrote the entry.
    pub keeper: usize,
    /// Epicenter count N (bells) the entry was committed at.
    pub at: Q,
    /// The keeper's reading at that count.
    pub reading: Reading,
    /// The keeper's slip minus the slip of the keeper inside it (turns), kept
    /// as computed and not averaged; zero for the epicenter.
    pub residual: Q,
    /// Hash of the same keeper's previous entry (all zeros for its first).
    pub parent: [u8; 32],
    /// Head hash of the keeper inside it at commit time (all zeros for the
    /// epicenter).
    pub inner: [u8; 32],
    /// BLAKE3 hash of this entry over the keeper index, the count, the local
    /// count and slip, the residual, the parent and inner hashes and, for the
    /// epicenter, the final point heard.
    pub hash: [u8; 32],
}

/// The universe's keepers and their chain.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Chronometry {
    /// All keepers: the epicenter first, then for each galaxy its hub, deep
    /// core, inner mantle and outer rim.
    pub keepers: Vec<Keeper>,
    /// For each keeper, the keeper inside it (the epicenter has none).
    pub inside: Vec<Option<usize>>,
    /// Every entry committed, in commit order, epicenter first within each
    /// commit.
    pub entries: Vec<Entry>,
    /// Each keeper's head: its last entry's hash.
    pub heads: Vec<[u8; 32]>,
    /// The counts committed at, in order; they depend on the run's history and
    /// are held in the checkpoint.
    pub commits: Vec<Q>,
    /// For each commit, the last final point heard (None when unlinked): the
    /// pairs the dilations are read from.
    pub points: Vec<Option<Point>>,
    /// Ring 0's point: the final point heard with the universe's origin
    /// receipt (None until the universe is originated on the network).
    pub anchor: Option<Point>,
}

/// The rate of a well of mass `inside_rg` at radius `r_rg` (both in r_g):
/// r / (r + M), the complement of the deficit share M / (r + M)
/// (`shares::deficit`), exact; 1 where the deficit is refused (r + M zero or
/// a negative part).
fn well(r_rg: &Q, inside_rg: &Q) -> Q {
    &Q::ONE - &xyl_core::shares::deficit(inside_rg, r_rg).unwrap_or(Q::ZERO)
}

impl Chronometry {
    /// The keepers of a founding: the epicenter, then for each galactic core
    /// its hub, deep core, inner mantle and outer rim. The chain starts empty.
    pub fn of(origin: &Origin) -> Chronometry {
        // One kpc in r_g: the galaxy tier's kpc (`galaxy::rings_of_kpc`) times the
        // tier's unit in r_g; one cluster unit in kpc.
        let kpc_rg = &xyl_cosmos::galaxy::rings_of_kpc(&Q::ONE) * &Tier::Galaxy.in_rg();
        let cluster_kpc = crate::transit::tier_m(Tier::Cluster)
            .checked_div(&xyl_cosmos::package::Si::kpc())
            .unwrap_or(Q::ONE);
        let lambda = xyl_cosmos::pulse::wavelength_kpc().to_q();
        let bells = |kpc: &Q| kpc.checked_div(&lambda).unwrap_or(Q::ZERO);
        let shapes = xyl_cosmos::cores::shapes(origin);
        let mass =
            |s: &xyl_cosmos::cores::Shape| &xyl_cosmos::turning::home_mass() * &s.mass_ratio();
        // A core's distance from the universe core is its reach along the
        // core's bearing, not a Euclidean norm.
        let dist = |s: &xyl_cosmos::cores::Shape| &s.reach * &cluster_kpc;
        // Rings: the universe's echo around its great circle is 2 pi R_u / c;
        // a galaxy's echo out to its rim (where the pulse refocuses) and back
        // is 2 rim / c. Each is expressed in the engine's bells, the
        // founding's 2 rim / c (circuit::anchor), so the epicenter's ring is
        // pi R_u / rim and a galaxy's ring is its scale.
        let r_u = xyl_cosmos::galaxy::radius_kpc();
        let ring_u = (&xyl_core::certified::pi(64).mid() * &r_u)
            .checked_div(&xyl_cosmos::galaxy::rim_kpc())
            .unwrap_or(Q::ONE);
        let mut c = Chronometry::default();
        c.keepers.push(Keeper {
            galaxy: None,
            shell: Shell::Epicenter,
            from_epicenter_kpc: Q::ZERO,
            rate: Q::ONE,
            delay: Q::ZERO,
            ring: ring_u,
        });
        c.inside.push(None);
        for s in &shapes {
            let d = dist(s);
            // The galaxy's well holds the masses of the cores nearer the
            // epicenter than this one.
            let nearer = shapes
                .iter()
                .filter(|o| o.index != s.index && dist(o) < d)
                .fold(Q::ZERO, |a, o| &a + &mass(o));
            // Masses are core masses (r_g of the core, G = c = 1).
            let galaxy_rate = well(&(&d * &kpc_rg), &nearer);
            let scale = s.scale().to_q();
            // The hub is the galactic core, counting in the galaxy's echo.
            c.keepers.push(Keeper {
                galaxy: Some(s.index),
                shell: Shell::Hub,
                from_epicenter_kpc: d.clone(),
                rate: galaxy_rate.clone(),
                delay: bells(&d),
                ring: scale.clone(),
            });
            c.inside.push(Some(0));
            let mut inner = c.keepers.len() - 1;
            // The shells' reaches are galaxy rings (60^5 r_g); the
            // distances from the epicenter stay in kpc, converted at the
            // boundary `galaxy::kpc_of_rings`.
            let ring_rg = Tier::Galaxy.in_rg();
            for (shell, r_home) in [
                (Shell::Core, xyl_cosmos::galaxy::nucleus_reach()),
                (Shell::Mantle, xyl_cosmos::galaxy::core_reach()),
                (Shell::Rim, xyl_cosmos::circuit::anchor().rim),
            ] {
                let r_rings = &r_home * &scale;
                let r = xyl_cosmos::galaxy::kpc_of_rings(&r_rings);
                let inside = &xyl_cosmos::galaxy::inside_mass(&r_home) * &s.mass_ratio();
                let rate = &galaxy_rate * &well(&(&r_rings * &ring_rg), &inside);
                c.keepers.push(Keeper {
                    galaxy: Some(s.index),
                    shell,
                    from_epicenter_kpc: &d + &r,
                    rate,
                    delay: bells(&(&d + &r)),
                    ring: scale.clone(),
                });
                c.inside.push(Some(inner));
                inner = c.keepers.len() - 1;
            }
        }
        c.heads = vec![[0; 32]; c.keepers.len()];
        c
    }

    /// Commits the readings at count `n`: every keeper appends an entry,
    /// epicenter first, each hashing its parent and the head of the keeper
    /// inside it. `point` is the last final point heard (None when
    /// unlinked), recorded whole in the epicenter's entry.
    pub fn commit(&mut self, n: &Q, point: Option<Point>) {
        let mut slips: Vec<Q> = Vec::with_capacity(self.keepers.len());
        for (k, keeper) in self.keepers.iter().enumerate() {
            let reading = keeper.read(n);
            let inner = self.inside[k];
            let residual = inner.map(|i| &reading.slip - &slips[i]).unwrap_or(Q::ZERO);
            let parent = self.heads[k];
            let inner_hash = inner.map(|i| self.heads[i]).unwrap_or([0; 32]);
            let mut h = blake3::Hasher::new_derive_key("xylarium chronometry entry v3");
            h.update(&(k as u64).to_le_bytes());
            for part in [
                n.to_string(),
                reading.local.to_string(),
                reading.slip.to_string(),
                residual.to_string(),
            ] {
                h.update(part.as_bytes());
                h.update(&[0]);
            }
            h.update(&parent);
            h.update(&inner_hash);
            if k == 0 {
                match &point {
                    Some(p) => h.update(&p.bytes()),
                    None => h.update(&[0xff; 8]),
                };
            }
            let hash: [u8; 32] = *h.finalize().as_bytes();
            slips.push(reading.slip.clone());
            self.entries.push(Entry {
                keeper: k,
                at: n.clone(),
                reading,
                residual,
                parent,
                inner: inner_hash,
                hash,
            });
            self.heads[k] = hash;
        }
        self.commits.push(n.clone());
        self.points.push(point);
    }

    /// Sets ring 0's point, once: the final point heard with the universe's
    /// origin receipt. A second anchor is ignored; the first stands.
    pub fn anchor_at(&mut self, p: Point) {
        if self.anchor.is_none() {
            self.anchor = Some(p);
        }
    }

    /// The binding's segments: for each pair of consecutive commits that
    /// heard a point under one origin, with the height advanced, the rings
    /// over the heights there. Never computed across a change of origin, and
    /// never one ratio for the whole chain: the dilation is read per segment.
    pub fn dilations(&self) -> Vec<Segment> {
        let heard: Vec<(&Q, &Point)> = self
            .commits
            .iter()
            .zip(&self.points)
            .filter_map(|(n, p)| p.as_ref().map(|p| (n, p)))
            .collect();
        heard
            .windows(2)
            .filter_map(|w| {
                let ((n0, p0), (n1, p1)) = (w[0], w[1]);
                if !p0.same_origin(p1) || p1.h <= p0.h {
                    return None;
                }
                let rings = n1 - n0;
                let dilation = rings.checked_div(&Q::int(i64::try_from(p1.h - p0.h).ok()?))?;
                Some(Segment {
                    from_h: p0.h,
                    to_h: p1.h,
                    rings,
                    dilation,
                })
            })
            .collect()
    }
}
