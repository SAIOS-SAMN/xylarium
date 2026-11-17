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
//! - Proper rate ([`Proper`], [`Faced::link`], #311): the two aspects of a
//!   body's time composed per link of its anchor chain. About a hold `M` at
//!   reach `r` (one calibre) the pull is the hold's dilation `D_h = M / r`
//!   (its faces' ratio `c / d`) and the body moves at `v = delta = M / (r +
//!   M)` by the turning law, its motion's dilation `D(v) = (r + 2M) / r`.
//!   By the flow law the two compose as a product, on the face moving with
//!   the pull `D_h D(v)` and on the face moving against it `D_h / D(v)`, and
//!   each face's rate is its divergent face `1 / (1 + D)`: with the pull
//!   `r^2 / (r^2 + r M + 2 M^2)`, against it `(r + 2M) / (r + 3M)`, both
//!   first-degree in the faces, never a square or a root of a speed. With
//!   no hold both read 1, and their dilations keep `D_with D_against =
//!   D_h^2`, so they part about the well, never an even split. Nested links
//!   multiply face by face: a galaxy's hub moves about the cores nearer the
//!   epicenter, each shell's matter about the hold inside its reach.
//! - Proper span ([`Faced::over`], [`Chronometry::proper_years`], #311): a
//!   body is on one face at a time. By the sheared return (T4) its anchor
//!   throat's near face returns after `bell / (1 - s)` and its far face after
//!   `bell / s`, `s` the throat's torsion share, so of each turn of its orbit
//!   it dwells the share `(1 / (1 - s)) / (1 / (1 - s) + 1 / s) = s` on the
//!   face moving with the pull and `1 - s` against it: an even split only
//!   where the throat does not spin. Between turn counts `a` and `b` the
//!   turns on the near face are `F(b) - F(a)`, `F(x) = floor(x) s + min(frac
//!   x, s)` ([`near_turns`]), exact, and a span's proper span is the span
//!   times each face's rate by its share of those turns. A people's years
//!   are its system's proper years: its star's link about its galaxy's hold,
//!   nested in its hub's about the cores nearer the epicenter, each link read
//!   on the turns of that link's orbit.
//! - Reading ([`Keeper::read`]) at the epicenter's count N (bells), on each
//!   face: local = rate (N - delay) / ring; slip = (1 - rate)(N - delay) /
//!   ring; coherence = 1 - 2 |slip - round(slip)|; rung = floor(slip). N -
//!   delay is clamped at zero.
//! - Delay (`xyl_core::chrono::arrival`): the front's arrival from the epicenter, the
//!   separation of the two places' time coordinates, `tau = d + H` with the
//!   hold the front crosses (the cores nearer the epicenter; for a shell
//!   also its galaxy's hold inside its reach), over the pulse's wavelength
//!   (E032, #280 stage 3).
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

use xyl_core::shares::FacePairs;
use xyl_core::units::Tier;
use xyl_core::Q;
use xyl_cosmos::origin::Origin;

/// A quantity held on the two motion faces of a body about its anchor: the
/// face moving with the anchor's pull and the face moving against it (the
/// attractor pair, never one value and never their mean).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Faced<T> {
    /// On the face moving with the pull.
    pub with: T,
    /// On the face moving against the pull.
    pub against: T,
}

/// A clock's proper rate against the epicenter, on both motion faces, each
/// in `(0, 1]`.
pub type Proper = Faced<Q>;

impl Faced<Q> {
    /// The epicenter's rate, one on both faces: the root, at rest.
    pub fn root() -> Proper {
        Faced {
            with: Q::ONE,
            against: Q::ONE,
        }
    }

    /// One link of an anchor chain: a body at reach `r` about a hold `m`
    /// (one calibre), its well and its motion `v = m / (r + m)` composed by
    /// the flow law on each face. `None` for no reach or a negative hold.
    pub fn link(m: &Q, r: &Q) -> Option<Proper> {
        if r.signum() <= 0 || m.signum() < 0 {
            return None;
        }
        if m.is_zero() {
            return Some(Faced::root());
        }
        let hold = FacePairs::of(&m.checked_div(r)?)?;
        let motion = FacePairs::dilation(&xyl_core::shares::deficit(m, r)?)?;
        let rate = |d: FacePairs| (&Q::ONE + &d.value()).recip();
        Some(Faced {
            with: rate(hold.compose(&motion))?,
            against: rate(hold.compose(&motion.inverse()))?,
        })
    }

    /// The proper span over `span` (any unit) of a body whose orbit turned
    /// from `a` to `b` turns about an anchor of torsion share `s`: the span
    /// times each face's rate by its share of those turns, the first `s` of
    /// each turn on the face moving with the pull; with no turn made, the
    /// face its phase stands on. `None` for `s` outside `(0, 1)` or turns
    /// that run back.
    pub fn over(&self, s: &Q, (a, b): (&Q, &Q), span: &Q) -> Option<Q> {
        if s.signum() <= 0 || *s >= Q::ONE || b < a {
            return None;
        }
        let turned = b - a;
        if turned.is_zero() {
            let face = if a.frac() < *s {
                &self.with
            } else {
                &self.against
            };
            return Some(span * face);
        }
        let near = near_turns(a, b, s)?;
        let far = &turned - &near;
        let rate = (&(&near * &self.with) + &(&far * &self.against)).checked_div(&turned)?;
        Some(span * &rate)
    }

    /// This rate nested inside `outer`'s: face by face, the product.
    pub fn within(&self, outer: &Proper) -> Proper {
        Faced {
            with: &self.with * &outer.with,
            against: &self.against * &outer.against,
        }
    }
}

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

/// The turns between turn counts `a <= b` that fall on the first `s` of each
/// turn (the near face): `F(b) - F(a)`, `F(x) = floor(x) s + min(frac x,
/// s)`, exact. `None` past `i128` turns.
pub fn near_turns(a: &Q, b: &Q, s: &Q) -> Option<Q> {
    let f = |x: &Q| -> Option<Q> {
        let whole = Q::from_i128(x.floor_i128()?, 1)?;
        Some(&(&whole * s) + &x.frac().min(s.clone()))
    };
    Some(&f(b)? - &f(a)?)
}

/// The torsion share of an anchor throat, the share of each of its bodies'
/// turns spent on the face moving with its pull: the universal core's at
/// its spin then (`Anchor::core_spin`, T17). A galactic core's own spin is
/// unbuilt (T11); its bell spins at the parent's.
pub fn anchor_share() -> Q {
    // Stand-in S021: Every galactic core's bell spins about z at the parent's drawn spin.
    xyl_cosmos::bounce::torsion_share(&Q::ONE, &xyl_cosmos::circuit::anchor().core_spin)
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
    /// Proper rate relative to the epicenter on both motion faces, each in
    /// (0, 1]: every link of its anchor chain, its well and its motion
    /// composed by the flow law, nested links multiplying ([`Faced::link`]).
    pub rate: Proper,
    /// Travel time of the pulse from the epicenter to the keeper, in bells:
    /// the front's arrival, the separation of the two places' time
    /// coordinates, `tau = d + H` with the hold `H` the front crosses
    /// (E032, #280 stage 3), over the pulse wavelength (`xyl_core::chrono::arrival`).
    pub delay: Q,
    /// The keeper's ring, in the engine's bells: the echo period of the
    /// enclosure it keeps time in (the universe's sphere for the epicenter,
    /// the galaxy's sphere for a hub and its shells).
    pub ring: Q,
    /// A hub's galactic core, whose own orbit about the epicenter sets
    /// which face the hub is on; `None` for the epicenter and the shells.
    pub core: Option<xyl_cosmos::cores::Shape>,
}

/// What a keeper reads at the root's winding: its local count, its slip,
/// its coherence and its rung (`xyl_core::chrono::keep`).
pub use xyl_core::chrono::Keeping as Reading;

impl Keeper {
    /// The keeper's count, slip, coherence and rung on each motion face, in
    /// the keeper's rings, at the root's winding `n`
    /// (`xyl_core::chrono::keep`). `None` for a ring that is not positive.
    pub fn read(&self, n: &Q) -> Option<Faced<Reading>> {
        let at = |rate: &Q| xyl_core::chrono::keep(rate, &self.delay, &self.ring, n);
        Some(Faced {
            with: at(&self.rate.with)?,
            against: at(&self.rate.against)?,
        })
    }
}

/// One entry of the hashed chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Index of the keeper in [`Chronometry::keepers`] that wrote the entry.
    pub keeper: usize,
    /// Epicenter count N (bells) the entry was committed at.
    pub at: Q,
    /// The keeper's reading at that count, on each motion face.
    pub reading: Faced<Reading>,
    /// The keeper's slip minus the slip of the keeper inside it (turns), on
    /// each face, kept as computed and not averaged; zero for the epicenter.
    pub residual: Faced<Q>,
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

impl Chronometry {
    /// The keepers of a founding: the epicenter, then for each galactic core
    /// its hub, deep core, inner mantle and outer rim. The chain starts empty.
    pub fn of(origin: &Origin) -> Chronometry {
        // One kpc in r_g: the galaxy tier's kpc (`galaxy::units_of_kpc`) times the
        // tier's unit in r_g; one cluster unit in kpc.
        let kpc_rg = &xyl_cosmos::galaxy::units_of_kpc(&Q::ONE) * &Tier::Galaxy.in_rg();
        let cluster_kpc = crate::transit::tier_m(Tier::Cluster)
            .checked_div(&xyl_cosmos::package::Si::kpc())
            .unwrap_or(Q::ONE);
        let lambda = xyl_cosmos::pulse::wavelength_kpc().to_q();
        let front = |kpc: &Q, held: &Q| {
            xyl_core::chrono::arrival(kpc, held, &kpc_rg, &lambda).unwrap_or(Q::ZERO)
        };
        let shapes = xyl_cosmos::cores::shapes(origin);
        let mass =
            |s: &xyl_cosmos::cores::Shape| &xyl_cosmos::turning::home_mass() * &s.mass_ratio();
        // A core's distance from the universe core is its reach along the
        // core's bearing, not a Euclidean norm.
        let dist = |s: &xyl_cosmos::cores::Shape| &s.reach * &cluster_kpc;
        // Rings: the engine's count is the root orbit's winding (no
        // background clock, docs/chronometry.md), so the epicenter keeps time
        // in root windings, a ring of one; a galaxy's echo out to its rim and
        // back is its scale in those windings. Every ring is rational: the
        // universe's great-circle period, which carries pi, is a display
        // reading, never a stored ring.
        let ring_u = Q::ONE;
        let mut c = Chronometry::default();
        c.keepers.push(Keeper {
            galaxy: None,
            shell: Shell::Epicenter,
            from_epicenter_kpc: Q::ZERO,
            rate: Proper::root(),
            delay: Q::ZERO,
            ring: ring_u,
            core: None,
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
            // Masses are core masses (r_g of the core, G = c = 1). The hub
            // moves about the cores nearer the epicenter, one link.
            let galaxy_rate = Proper::link(&nearer, &(&d * &kpc_rg)).unwrap_or_else(Proper::root);
            let scale = s.scale().to_q();
            // The hub is the galactic core, counting in the galaxy's echo.
            c.keepers.push(Keeper {
                galaxy: Some(s.index),
                shell: Shell::Hub,
                from_epicenter_kpc: d.clone(),
                rate: galaxy_rate.clone(),
                // The front to the hub crosses the cores nearer the
                // epicenter, the hold its well reads.
                delay: front(&d, &nearer),
                ring: scale.clone(),
                core: Some(s.clone()),
            });
            c.inside.push(Some(0));
            let mut inner = c.keepers.len() - 1;
            // The shells' reaches are galaxy units (60^5 r_g); the
            // distances from the epicenter stay in kpc, converted at the
            // boundary `galaxy::kpc_of_units`.
            let ring_rg = Tier::Galaxy.in_rg();
            for (shell, r_home) in [
                (Shell::Core, xyl_cosmos::galaxy::nucleus_reach()),
                (Shell::Mantle, xyl_cosmos::galaxy::core_reach()),
                (Shell::Rim, xyl_cosmos::circuit::anchor().rim),
            ] {
                let r_rings = &r_home * &scale;
                let r = xyl_cosmos::galaxy::kpc_of_units(&r_rings);
                let inside = &xyl_cosmos::galaxy::inside_mass(&r_home) * &s.mass_ratio();
                // The shell's matter moves about the hold inside its reach,
                // a link nested inside the hub's.
                let rate = Proper::link(&inside, &(&r_rings * &ring_rg))
                    .unwrap_or_else(Proper::root)
                    .within(&galaxy_rate);
                c.keepers.push(Keeper {
                    galaxy: Some(s.index),
                    shell,
                    from_epicenter_kpc: &d + &r,
                    rate,
                    // The front to a shell at `r` also crosses its galaxy's
                    // hold inside `r`, all within `d + r` of the epicenter.
                    delay: front(&(&d + &r), &(&nearer + &inside)),
                    ring: scale.clone(),
                    core: None,
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
        // Each keeper's slip, `None` for one with no reading (a ring that is
        // not positive, never the case for a founded keeper): no entry is
        // written for it, and a keeper outside it reads its residual against
        // the next keeper in that has one.
        let mut slips: Vec<Option<Faced<Q>>> = Vec::with_capacity(self.keepers.len());
        for (k, keeper) in self.keepers.iter().enumerate() {
            let Some(reading) = keeper.read(n) else {
                slips.push(None);
                continue;
            };
            let inner = self.inside[k];
            let mut inner_slip = None;
            let mut at = inner;
            while let Some(i) = at {
                if let Some(Some(si)) = slips.get(i) {
                    inner_slip = Some(si.clone());
                    break;
                }
                at = self.inside.get(i).copied().flatten();
            }
            let residual = match inner_slip {
                Some(si) => Faced {
                    with: &reading.with.slip - &si.with,
                    against: &reading.against.slip - &si.against,
                },
                None => Faced {
                    with: Q::ZERO,
                    against: Q::ZERO,
                },
            };
            let parent = self.heads[k];
            let inner_hash = inner.map(|i| self.heads[i]).unwrap_or([0; 32]);
            let mut h = blake3::Hasher::new_derive_key("xylarium chronometry entry v4");
            h.update(&(k as u64).to_le_bytes());
            for part in [
                n.to_string(),
                reading.with.local.to_string(),
                reading.with.slip.to_string(),
                residual.with.to_string(),
                reading.against.local.to_string(),
                reading.against.slip.to_string(),
                residual.against.to_string(),
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
            slips.push(Some(Faced {
                with: reading.with.slip.clone(),
                against: reading.against.slip.clone(),
            }));
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

    /// A system's proper years over `years` from `from` to `to` (Gyr): its
    /// star's link about its galaxy's hold nested in its hub's link, each
    /// read on the turns of that link's orbit over the span ([`Faced::over`]), the
    /// links' products. Exact within one face of each link; where both
    /// links change face inside one span, each link's share is read on that link's turns over
    /// the span. `None` for a star not founded, a galaxy with no hub, or a
    /// link refused.
    pub fn proper_years(
        &self,
        (seed, origin): (u64, &Origin),
        star: u64,
        (from, to): (xyl_core::dyadic::Dy, xyl_core::dyadic::Dy),
        years: &Q,
    ) -> Option<Q> {
        let st = xyl_cosmos::galaxy::star(seed, origin, star)?;
        let hub = self
            .keepers
            .iter()
            .find(|k| k.galaxy == Some(st.core) && k.shell == Shell::Hub)?;
        let core = hub.core.as_ref()?;
        let (t0, t1) = (
            xyl_cosmos::circuit::rings_of_gyr_dy(from),
            xyl_cosmos::circuit::rings_of_gyr_dy(to),
        );
        let s = anchor_share();
        let hub_turns = |t| core.place_at(t, &Q::ONE).winding.turns();
        let hub_years = hub.rate.over(&s, (&hub_turns(t0), &hub_turns(t1)), years)?;
        let scale = st.scale.to_q();
        let inside =
            &xyl_cosmos::galaxy::inside_mass(&st.r) * &xyl_cosmos::cores::ratio_of_scale(&scale);
        let reach = &(&st.r * &scale) * &Tier::Galaxy.in_rg();
        let link = Proper::link(&inside, &reach)?;
        let (a, b) = (st.orbit.turns_at(t0).to_q(), st.orbit.turns_at(t1).to_q());
        link.over(&s, (&a, &b), &hub_years)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The dilation a face's rate reads: `1 / rate - 1`.
    fn dilation(rate: &Q) -> Q {
        &rate.recip().unwrap() - &Q::ONE
    }

    /// A link's two faces: refused without a reach or with a negative hold;
    /// the root with no hold; each in (0, 1) with a hold, the face moving
    /// with the pull the slower; parting about the well, `D_with D_against =
    /// D_h^2` with `D_h = M / r`; both slower deeper in the well; unchanged
    /// by a change of tier that dilates reach and hold together; and nested
    /// links multiplying face by face, the root their identity.
    #[test]
    fn a_bodys_proper_rate_composes_its_well_and_its_motion() {
        assert_eq!(Proper::link(&Q::ONE, &Q::ZERO), None);
        assert_eq!(Proper::link(&xyl_core::ratio!(-1, 2), &Q::ONE), None);
        assert_eq!(Proper::link(&Q::ZERO, &Q::int(7)), Some(Proper::root()));
        let mut last: Option<Proper> = None;
        for (m, r) in [(1, 1000), (1, 30), (1, 3), (1, 1), (3, 1)] {
            let (m, r) = (Q::int(m), Q::int(r));
            let f = Proper::link(&m, &r).unwrap();
            for rate in [&f.with, &f.against] {
                assert!(rate.signum() > 0 && *rate < Q::ONE);
            }
            assert!(f.with < f.against, "the face with the pull is the slower");
            let hold = m.checked_div(&r).unwrap();
            assert_eq!(&dilation(&f.with) * &dilation(&f.against), &hold * &hold);
            let well = xyl_core::chrono::well(&r, &m).unwrap();
            assert!(
                f.with < well && well < f.against,
                "the faces part about the well"
            );
            if let Some(l) = &last {
                assert!(f.with < l.with && f.against < l.against, "deeper is slower");
            }
            let tier = xyl_core::ratio!(60, 7);
            assert_eq!(Proper::link(&(&m * &tier), &(&r * &tier)), Some(f.clone()));
            last = Some(f);
        }
        let (a, b) = (
            Proper::link(&Q::ONE, &Q::int(4)).unwrap(),
            Proper::link(&Q::int(2), &Q::int(9)).unwrap(),
        );
        let nested = a.within(&b);
        assert_eq!(nested.with, &a.with * &b.with);
        assert_eq!(nested.against, &a.against * &b.against);
        assert_eq!(a.within(&Proper::root()), a);
    }

    /// A body dwells the torsion share `s` of each turn on the face moving
    /// with the pull: the near turns add over consecutive spans and are
    /// exactly `s` per whole turn; a span with no turn reads the face its
    /// phase stands on; over whole turns the proper span weighs each face by
    /// its share, an even split only at `s = 1/2`, and a spinning anchor
    /// (`s > 1/2`) holds its bodies longer on the slower face; refused for a
    /// share outside (0, 1) or turns that run back; the anchor's share lies
    /// in [1/2, 3/5].
    #[test]
    fn a_body_dwells_the_torsion_share_of_each_turn_on_the_near_face() {
        let s = xyl_core::ratio!(4, 7);
        let (a, b, c) = (xyl_core::ratio!(3, 10), xyl_core::ratio!(27, 8), Q::int(9));
        let near = |x: &Q, y: &Q| near_turns(x, y, &s).unwrap();
        assert_eq!(&near(&a, &b) + &near(&b, &c), near(&a, &c));
        assert_eq!(near(&a, &(&a + &Q::int(5))), &s * &Q::int(5));
        let f = Proper::link(&Q::ONE, &Q::int(3)).unwrap();
        let span = Q::int(100);
        assert_eq!(
            f.over(&s, (&Q::ZERO, &Q::ZERO), &span),
            Some(&span * &f.with)
        );
        let late = xyl_core::ratio!(5, 7);
        assert_eq!(f.over(&s, (&late, &late), &span), Some(&span * &f.against));
        let whole = |share: &Q| f.over(share, (&a, &(&a + &Q::int(4))), &span).unwrap();
        let weighed = &(share_rate(&f, &s)) * &span;
        assert_eq!(whole(&s), weighed);
        let half = xyl_core::ratio!(1, 2);
        assert_eq!(whole(&half), &(&(&f.with + &f.against) * &half) * &span);
        assert!(
            whole(&s) < whole(&half),
            "spin holds the body on the slower face longer"
        );
        assert_eq!(f.over(&Q::ONE, (&a, &b), &span), None);
        assert_eq!(f.over(&Q::ZERO, (&a, &b), &span), None);
        assert_eq!(f.over(&s, (&b, &a), &span), None);
        let share = anchor_share();
        assert!(share >= half && share <= xyl_core::ratio!(3, 5));
    }

    fn share_rate(f: &Proper, s: &Q) -> Q {
        &(s * &f.with) + &(&(&Q::ONE - s) * &f.against)
    }
}
