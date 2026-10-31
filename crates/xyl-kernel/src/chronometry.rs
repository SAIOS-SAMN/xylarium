//! Chronometry: the keepers that count time at the epicenter and at each
//! galaxy's hub and shells, and the hashed chain of their readings.
//!
//! Inputs: the seed and [`BigBang`] (galactic core shapes from
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
//! - Pace ([`Chronometry::pace`]): the epicenter's entry records the last
//!   TUniversal final point's receipt height, so rings per final-point
//!   advance is measured from the commit pairs, not declared.
//!
//! Every quantity is an exact `Q`. Keepers and entries are recomputed from
//! the seed; only the commit counts and receipt heights are held in the
//! checkpoint.
//!
//! Terms: TUniversal (docs/glossary.md).

use xyl_core::units::Tier;
use xyl_core::Q;
use xyl_cosmos::bang::BigBang;

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
    /// epicenter, the receipt height.
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
    /// For each commit, the last final point's receipt height received (None
    /// when unlinked): the pairs `pace` is measured from.
    pub points: Vec<Option<u64>>,
}

/// The pace measured from the commit/receipt pairs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pace {
    /// Rings advanced per final-point advance, between the first and last
    /// commits that recorded a receipt height.
    Measured {
        /// Epicenter count advance (bells) per receipt-height advance.
        rings_per_point: Q,
        /// Epicenter count advance (bells) between those two commits.
        rings: Q,
        /// Receipt-height advance between those two commits.
        points: u64,
    },
    /// No pace is measurable, because no two commits recorded receipt
    /// heights that advanced: only the span of the commits is known.
    Unlinked {
        /// Epicenter count advance (bells) between the first and last
        /// commits; 0 with none.
        rings: Q,
        /// Number of commits made.
        commits: usize,
    },
}

/// The rate of a well of mass `inside_rg` at radius `r_rg` (both in r_g):
/// r / (r + M), exact; 1 when r + M is zero.
fn well(r_rg: &Q, inside_rg: &Q) -> Q {
    r_rg.checked_div(&(r_rg + inside_rg)).unwrap_or(Q::ONE)
}

impl Chronometry {
    /// The keepers of a founding: the epicenter, then for each galactic core
    /// its hub, deep core, inner mantle and outer rim. The chain starts empty.
    pub fn of(seed: u64, bang: &BigBang) -> Chronometry {
        let units = xyl_cosmos::chain::units();
        let kpc_m = xyl_cosmos::package::Si::kpc();
        let kpc_rg = kpc_m.checked_div(&units.r_g_m).unwrap_or(Q::ONE);
        let cluster_kpc = (&Tier::Cluster.in_rg() * &units.r_g_m)
            .checked_div(&kpc_m)
            .unwrap_or(Q::ONE);
        let suns_rg = |suns: &Q| {
            suns.checked_div(&xyl_cosmos::package::hole_mass())
                .unwrap_or(Q::ZERO)
        };
        let lambda = xyl_cosmos::pulse::wavelength_kpc().to_q();
        let bells = |kpc: &Q| kpc.checked_div(&lambda).unwrap_or(Q::ZERO);
        let shapes = xyl_cosmos::cores::shapes(seed, bang);
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
            let galaxy_rate = well(&(&d * &kpc_rg), &suns_rg(&nearer));
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
            for (shell, r_home) in [
                (Shell::Core, xyl_cosmos::galaxy::nucleus_kpc()),
                (Shell::Mantle, xyl_cosmos::galaxy::core_kpc()),
                (Shell::Rim, xyl_cosmos::galaxy::rim_kpc()),
            ] {
                let r = &r_home * &scale;
                let inside = &xyl_cosmos::galaxy::inside_mass(&r_home) * &s.mass_ratio();
                let rate = &galaxy_rate * &well(&(&r * &kpc_rg), &suns_rg(&inside));
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
    /// inside it. `point` is the receipt height of the last final point
    /// received (None when unlinked), recorded in the epicenter's entry.
    pub fn commit(&mut self, n: &Q, point: Option<u64>) {
        let mut slips: Vec<Q> = Vec::with_capacity(self.keepers.len());
        for (k, keeper) in self.keepers.iter().enumerate() {
            let reading = keeper.read(n);
            let inner = self.inside[k];
            let residual = inner.map(|i| &reading.slip - &slips[i]).unwrap_or(Q::ZERO);
            let parent = self.heads[k];
            let inner_hash = inner.map(|i| self.heads[i]).unwrap_or([0; 32]);
            let mut h = blake3::Hasher::new_derive_key("xylarium chronometry entry v1");
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
                h.update(&point.map(|p| p.to_le_bytes()).unwrap_or([0xff; 8]));
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

    /// Pace: rings per final-point advance, measured between the first and
    /// last commits that recorded a receipt height. Returns
    /// `Pace::Unlinked` when the height did not advance between them.
    pub fn pace(&self) -> Pace {
        let paired: Vec<(&Q, u64)> = self
            .commits
            .iter()
            .zip(&self.points)
            .filter_map(|(n, p)| p.map(|h| (n, h)))
            .collect();
        let rings = match (self.commits.first(), self.commits.last()) {
            (Some(a), Some(b)) => b - a,
            _ => Q::ZERO,
        };
        match (paired.first(), paired.last()) {
            (Some(&(n0, h0)), Some(&(n1, h1))) if h1 > h0 => {
                let points = h1 - h0;
                let span = n1 - n0;
                Pace::Measured {
                    rings_per_point: span
                        .checked_div(&Q::int(i64::try_from(points).unwrap_or(i64::MAX)))
                        .unwrap_or(Q::ZERO),
                    rings: span,
                    points,
                }
            }
            _ => Pace::Unlinked {
                rings,
                commits: self.commits.len(),
            },
        }
    }
}
