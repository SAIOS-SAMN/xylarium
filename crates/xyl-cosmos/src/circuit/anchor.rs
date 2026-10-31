// The anchor: the values the founding fixes for every tier above it, read
// in the cluster tier (60^7 r_g of the core).
//
// Inputs: the founding's rim (cluster units), the rings until the home
// galaxy's gas first lights (formation.rs `cools_in`), the parent's spin,
// its torsion share and the firmament's margin; `chain::units` for r_g in
// metres, `xyl_core::units::Tier` and `package::Si` (kpc, c; the Gyr only
// in the boundary conversions).
//
// Outputs: `Anchor` (rim, cluster and galaxy-tier units in kpc, the bell
// = 2 rim 60^5 in core time r_g / c with the rim read in galaxy-tier units,
// home rim, curvature radius rim / pi in kpc
// and in r_g, first stars in rings, at least one ring), `anchor`, `set`,
// `register_spin`, and the boundary conversions `rings_of_gyr`,
// `gyr_of_rings` (Q) and `rings_of_gyr_dy`, `gyr_of_rings_dy` (Dy).
//
// Time: the engine's unit of time is the ring, one bell. Every time on the
// substrate is a count of rings and every rate is per ring; a time is a
// position on the temporal coordinate (`xyl_core::coordinate`, the T lane),
// whole rings plus the phase within the ring, and its seconds are seconds
// of arc of that phase. SI seconds enter only at the SI seam,
// `package::Si::seconds_per_ring`. The Gyr conversions here are temporary
// boundary and display conversions for layers not yet counted in rings.
//
// One universe per process: the first `set` wins. `anchor` founds the
// substrate at the registered spin when no anchor is set; where no spin is
// registered or the founding fails, every field is zero.
// Terms: anchor (docs/glossary.md).

use xyl_core::dyadic::Dy;
use xyl_core::units::Tier;
use xyl_core::Q;

/// What the founding fixes for the universe, read in the cluster tier
/// (60^7 r_g of the core, the tier of galactic cores about their universe
/// core): the substrate's reaches are in that unit, one ring one cluster unit.
///
/// - The rim: the outermost founding shell's reach.
/// - The bell: the core's pulse period, set where light does the work: the
///   round trip to the home galaxy's rim and back at light speed,
///   2 rim_galaxy / c. The pulse itself rides the filament (the pole axis)
///   across the whole universe within the same bell, so the ocean's speed is
///   rim_universe / (bell / 2) = 3600 c, light's times the ladder step between
///   the galaxy and cluster tiers; a quantum state along the filament moves
///   by dilation per echo, not by arc, and has no light-time floor.
/// - The home galaxy's rim: the home galaxy is the core's own matter, the
///   same cascade read one tier down, so its rim is the rim in the galaxy
///   tier's unit (60^5 r_g), and its rings are the beads read there.
/// - The universe's radius: each enclosed sphere's rim is half a great
///   circle, so the radius is rim / pi.
/// - The first stars: the home galaxy's gas collapses where its first piece
///   cools, in rings of the bell (formation.rs `cools_in`), at least one ring.
/// - The ring: one bell is the engine's unit of time. In core time (G = c =
///   M_core = 1, time unit r_g / c) the bell is exact, 2 rim 60^5. SI
///   seconds enter only at the SI seam (`package::Si::seconds_per_ring`).
///
/// One universe per process: the first founding sets the anchor and every
/// later reading in the process is of that universe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchor {
    /// The rim (cluster units): the outermost shell's reach.
    pub rim: Q,
    /// One cluster unit in kiloparsecs (exact from the core's r_g).
    pub cluster_kpc: Q,
    /// One galaxy-tier ring in kpc (60^5 r_g): the unit every reach inside
    /// the home galaxy is read in, the cascade read one tier down, so the
    /// deficit's hold of 1 means the same thing at a star as at its piece.
    pub galaxy_kpc: Q,
    /// One bell in core time (units of r_g / c, exact): 2 rim 60^5, light's
    /// round trip to the home galaxy's rim, the rim read in galaxy-tier units.
    /// One bell is one ring, the engine's unit of time.
    pub bell: Q,
    /// The home galaxy's rim (kpc).
    pub home_rim_kpc: Q,
    /// The universe's curvature radius (kpc): rim / pi.
    pub radius_kpc: Q,
    /// The universe's curvature radius in r_g of the core: the S^360
    /// enclosure every horizon is read in.
    pub radius_rg: Q,
    /// When the first stars shone (rings after the Big Bang): the rings until
    /// the home galaxy's gas first collapses, at least one.
    pub first_stars: Q,
    /// The parent's Kerr spin as admitted (drawn from the seed, floored by
    /// cores::substrate_for).
    pub spin: Q,
    /// The share the jet carries through the throat at each crossing, from
    /// the spin by torsion (bang::torsion_share).
    pub share: Q,
    /// The firmament's margin: the torsion share less the least share that
    /// holds the firmament whole, `r_far / rim (1 + A_in)` at the inbound
    /// echo's peak (`Substrate::least_share`). Negative where no admissible
    /// spin reaches it: the firmament is not whole.
    pub margin: Q,
}

static ANCHOR: std::sync::OnceLock<Anchor> = std::sync::OnceLock::new();
static SPIN: std::sync::OnceLock<Q> = std::sync::OnceLock::new();

/// Records the parent's drawn spin of the universe this process founds, so a
/// reading taken before the founding can found it. The first Big Bang made
/// in the process is the process's universe; the anchor holds the admitted
/// spin (cores::substrate_for), the drawn one floored at the least that
/// holds the firmament whole.
pub fn register_spin(spin: &Q) {
    let _ = SPIN.set(spin.clone());
}

/// Sets the anchor at the founding; the first founding in the process wins.
pub fn set(anchor: Anchor) {
    let _ = ANCHOR.set(anchor);
}

/// The anchor of the process's universe. Where no founding has happened yet
/// the substrate is founded first (once per process, cores.rs): the founding
/// depends on the parent's spin alone, so it needs no Big Bang. Where no spin
/// is registered or the founding fails, every value is zero: a reading then
/// shows zeros, not a hidden default.
pub fn anchor() -> Anchor {
    if let Some(a) = ANCHOR.get() {
        return a.clone();
    }
    // Founding the substrate at the registered spin calls `set`
    // (cores::substrate_for), so the anchor is available below.
    if let Some(spin) = SPIN.get() {
        let _ = crate::cores::substrate_for(spin);
    }
    if let Some(a) = ANCHOR.get() {
        return a.clone();
    }
    Anchor {
        rim: Q::ZERO,
        cluster_kpc: Q::ZERO,
        galaxy_kpc: Q::ZERO,
        bell: Q::ZERO,
        home_rim_kpc: Q::ZERO,
        radius_kpc: Q::ZERO,
        radius_rg: Q::ZERO,
        first_stars: Q::ZERO,
        spin: Q::ZERO,
        share: Q::ZERO,
        margin: Q::ZERO,
    }
}

/// The anchor from the founding's rim (cluster units), the rings until the
/// home galaxy's gas first collapses, the parent's spin with its torsion
/// share, and the firmament's margin.
pub(crate) fn of(rim: &Q, cools_in: Option<i64>, spin: &Q, share: &Q, margin: &Q) -> Anchor {
    let units = crate::chain::units();
    let kpc_m = crate::package::Si::kpc();
    let cluster_m = &Tier::Cluster.in_rg() * &units.r_g_m;
    let cluster_kpc = cluster_m.checked_div(&kpc_m).unwrap_or(Q::ZERO);
    let galaxy_kpc = (&Tier::Galaxy.in_rg() * &units.r_g_m)
        .checked_div(&kpc_m)
        .unwrap_or(Q::ZERO);
    // The bell is 2 rim 60^5 in core time (r_g / c): light's round trip to
    // the home rim, the rim read in galaxy-tier units (60^5 r_g).
    let bell = &(&Q::int(2) * rim) * &Tier::Galaxy.in_rg();
    let pi = xyl_core::certified::pi(80).mid();
    let radius_kpc = (rim * &cluster_kpc).checked_div(&pi).unwrap_or(Q::ZERO);
    let radius_rg = (rim * &Tier::Cluster.in_rg())
        .checked_div(&pi)
        .unwrap_or(Q::ZERO);
    // The first stars shine after the rings the gas takes to cool, with at
    // least one ring.
    let rings = Q::int(cools_in.unwrap_or(1).max(1));
    Anchor {
        rim: rim.clone(),
        cluster_kpc: cluster_kpc.clone(),
        galaxy_kpc: galaxy_kpc.clone(),
        bell,
        home_rim_kpc: rim * &galaxy_kpc,
        radius_kpc,
        radius_rg,
        first_stars: rings,
        spin: spin.clone(),
        share: share.clone(),
        margin: margin.clone(),
    }
}

/// Gyr per ring (exact): SI seconds per ring (`package::Si::seconds_per_ring`)
/// over the Gyr's (`package::Si::gyr_s`), read once per process after the founding. Zero
/// until a universe is founded in the process.
fn gyr_per_ring() -> Q {
    static G: std::sync::OnceLock<Q> = std::sync::OnceLock::new();
    if let Some(g) = G.get() {
        return g.clone();
    }
    let g = crate::package::Si::seconds_per_ring()
        .checked_div(&crate::package::Si::gyr_s())
        .unwrap_or(Q::ZERO);
    if g.signum() > 0 {
        let _ = G.set(g.clone());
    }
    g
}

/// Gyr per ring as a `Dy` (see [`gyr_per_ring`]).
fn gyr_per_ring_dy() -> Dy {
    static G: std::sync::OnceLock<Dy> = std::sync::OnceLock::new();
    if let Some(g) = G.get() {
        return *g;
    }
    let g = Dy::from_q(&gyr_per_ring());
    if g.signum() > 0 {
        let _ = G.set(g);
    }
    g
}

/// A time in Gyr as a count of rings (exact). A temporary boundary and
/// display conversion: the one place, with [`gyr_of_rings`], where Gyr and
/// rings meet, used by layers not yet counted in rings. Zero until a universe
/// is founded in the process.
pub fn rings_of_gyr(gyr: &Q) -> Q {
    gyr.checked_div(&gyr_per_ring()).unwrap_or(Q::ZERO)
}

/// A count of rings as a time in Gyr (exact). A temporary boundary and
/// display conversion (see [`rings_of_gyr`]).
pub fn gyr_of_rings(rings: &Q) -> Q {
    rings * &gyr_per_ring()
}

/// A time in Gyr as a count of rings, as a `Dy` at a seam. A temporary
/// boundary and display conversion (see [`rings_of_gyr`]).
pub fn rings_of_gyr_dy(gyr: Dy) -> Dy {
    let g = gyr_per_ring_dy();
    if g.signum() <= 0 {
        return Dy::ZERO;
    }
    gyr / g
}

/// A count of rings as a time in Gyr, as a `Dy` at a seam. A temporary
/// boundary and display conversion (see [`rings_of_gyr`]).
pub fn gyr_of_rings_dy(rings: Dy) -> Dy {
    rings * gyr_per_ring_dy()
}
