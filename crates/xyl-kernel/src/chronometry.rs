//! Chronometry inside a universe: time is kept by keepers chained from the
//! epicenter outward, not by one scalar read everywhere.
//!
//! - Keepers: the epicenter (the universe core) rings on the universe's own
//!   echo. Each galactic core has a hub (its uplink; each galaxy is its own
//!   enclosed sphere) ringing on its galaxy's echo, with three shells under
//!   it: deep core (nuclear-cluster scale), inner mantle (bulge scale) and
//!   outer rim (disc edge). Every keeper counts in its own rings, the echo of
//!   the enclosure it keeps time in.
//! - Rate (law of the wells): at radius r with M(<r) inside, in white-hole
//!   units (G = c = M_core = 1, r and M both in r_g), a layer runs at
//!   r / (r + M(<r)) of the layer above it, exactly; nested wells multiply. A
//!   galaxy's well counts the cores nearer the epicenter than it.
//! - Count: at the epicenter's count N (bells), a keeper reads
//!   local = rate (N - delay) / ring, where delay is the pulse's travel time
//!   from the epicenter in bells.
//! - Slip, coherence, rung: a slower keeper falls behind by
//!   slip = (1 - rate)(N - delay) / ring turns; coherence is
//!   1 - 2 |slip - round(slip)| (1 in phase, 0 half a turn out); rungs are
//!   whole slips. A keeper at rate 1 has no rungs.
//! - Chain: at each committed moment every keeper appends a hashed entry
//!   citing its own previous entry (parent) and the entry of the keeper inside
//!   it (rim -> mantle -> core -> hub -> epicenter). Its slip relative to the
//!   inner keeper is kept as the entry's residual, never averaged. A
//!   universe's ledger cites the rims' heads, and through them the chain.
//! - Pace: the universe's physics is independent of the machine computing it;
//!   its pace is not. The epicenter's entry pairs the committed count with the
//!   last TUniversal final point received (receipt height h), so rings per
//!   final-point advance is measured, never declared. A faster machine
//!   computes more rings between two final points, so its universe's time
//!   runs faster relative to the network.

use xyl_core::units::Tier;
use xyl_core::Q;
use xyl_cosmos::bang::BigBang;

/// Where a keeper stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shell {
    /// The universe core, ringing on the universe's own echo at rate 1 with no delay.
    Epicenter,
    /// A galactic core as its galaxy's hub: its uplink to the epicenter,
    /// ringing on its galaxy's own echo.
    Hub,
    /// A galaxy's deep core, at its nuclear-cluster radius.
    Core,
    /// A galaxy's inner mantle, at its bulge radius.
    Mantle,
    /// A galaxy's outer rim, at its disc edge.
    Rim,
}

impl Shell {
    /// Human-readable name of the shell, as used in reports ("epicenter", "galaxy hub", "deep core", "inner mantle", "outer rim").
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
    /// Distance from the epicenter in kpc: the galaxy core's reach along its bearing plus the shell's radius.
    pub from_epicenter_kpc: Q,
    /// Clock rate relative to the epicenter, in (0, 1]: the product of the nested wells r / (r + M(<r)).
    pub rate: Q,
    /// Travel time of the pulse from the epicenter to the keeper, in bells.
    pub delay: Q,
    /// Its own ring, in the engine's bells: the echo of the enclosure it keeps
    /// time in (the universe's sphere for the epicenter, its galaxy's sphere
    /// for a hub and its shells).
    pub ring: Q,
}

/// What a keeper reads at the epicenter's count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    /// Local count in the keeper's own rings: rate x (N - delay) / ring.
    pub local: Q,
    /// Turns fallen behind the epicenter: (1 - rate) x (N - delay) / ring.
    pub slip: Q,
    /// Phase agreement with the epicenter in [0, 1]: 1 - 2 |slip - round(slip)| (1 in phase, 0 half a turn out).
    pub coherence: Q,
    /// Whole turns of slip (the floor of `slip`); 0 for a keeper at rate 1.
    pub rung: i128,
}

impl Keeper {
    /// Count, slip, coherence and rungs in its own rings at the engine's
    /// count `n` (bells).
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

/// One link of the chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Index of the keeper in [`Chronometry::keepers`] that wrote the entry.
    pub keeper: usize,
    /// Epicenter count N (bells) the entry was committed at.
    pub at: Q,
    /// The keeper's reading at that count.
    pub reading: Reading,
    /// Its slip against the keeper inside it (turns): kept, never averaged.
    pub residual: Q,
    /// Hash of the same keeper's previous entry (all zeros for its first).
    pub parent: [u8; 32],
    /// Head hash of the keeper inside it at commit time (all zeros for the epicenter).
    pub inner: [u8; 32],
    /// BLAKE3 hash of this entry over the keeper index, count, reading, residual, parent and inner hashes (and, for the epicenter, the receipt height).
    pub hash: [u8; 32],
}

/// The universe's keepers and their chain.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Chronometry {
    /// All keepers: the epicenter first, then for each galaxy its hub, deep core, inner mantle and outer rim.
    pub keepers: Vec<Keeper>,
    /// For each keeper, the keeper inside it (the epicenter has none).
    pub inside: Vec<Option<usize>>,
    /// Every entry committed, in commit order, epicenter first within each commit.
    pub entries: Vec<Entry>,
    /// Each keeper's head: its last entry's hash.
    pub heads: Vec<[u8; 32]>,
    /// The counts committed at, in order (path-dependent: held).
    pub commits: Vec<Q>,
    /// For each commit, the last final point's receipt height received (None
    /// when unlinked): the pairs `pace` is measured from.
    pub points: Vec<Option<u64>>,
}

/// The pace measured from the commit/receipt pairs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pace {
    /// Rings advanced per final-point advance, between the first and last
    /// commits that heard a point.
    /// Measured pace between the first and last commits that heard a point.
    Measured {
        /// Epicenter count advance (bells) per receipt-height advance.
        rings_per_point: Q,
        /// Epicenter count advance (bells) between those two commits.
        rings: Q,
        /// Receipt-height advance between those two commits.
        points: u64,
    },
    /// Fewer than two commits received a point (unlinked): only the rings
    /// committed are known.
    /// No pace measurable: only the commits' own span is known.
    Unlinked {
        /// Epicenter count advance (bells) between the first and last commits; 0 with none.
        rings: Q,
        /// Number of commits made.
        commits: usize,
    },
}

/// The rate of a well of `inside_rg` at `r_rg` (both lengths of r_g):
/// r / (r + M), exact.
fn well(r_rg: &Q, inside_rg: &Q) -> Q {
    r_rg.checked_div(&(r_rg + inside_rg)).unwrap_or(Q::ONE)
}

impl Chronometry {
    /// The keepers of a founding: the epicenter, then each galactic core's
    /// deep core, inner mantle and outer rim.
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
        // a core's distance from the universe core: its reach along its own
        // bearing (not a Euclidean norm)
        let dist = |s: &xyl_cosmos::cores::Shape| &s.reach * &cluster_kpc;
        // rings: the universe's echo around its great circle, 2 pi R_u / c;
        // a galaxy's echo out to its rim (where the pulse refocuses) and back,
        // 2 rim / c; each in the engine's bells (the founding's 2 rim / c,
        // circuit::anchor)
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
            // the galaxy's well: the cores nearer the epicenter than it
            let nearer = shapes
                .iter()
                .filter(|o| o.index != s.index && dist(o) < d)
                .fold(Q::ZERO, |a, o| &a + &mass(o));
            let galaxy_rate = well(&(&d * &kpc_rg), &suns_rg(&nearer));
            let scale = s.scale().to_q();
            // hub: the galactic core, ringing on its galaxy's own echo
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

    /// Commits a moment: every keeper appends its entry, epicenter first, each
    /// citing its parent and the keeper inside it. `point` is the last final
    /// point's receipt height received (None when unlinked), recorded in the
    /// epicenter's entry.
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

    /// Pace: rings per final-point advance, measured from the pairs.
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
