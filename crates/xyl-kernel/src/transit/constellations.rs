//! Binding and constellations: the pair binding of two stars against the
//! galaxy's shear ([`binding_at`]), the stars in play ([`in_play`]), the
//! pairs that can bind ([`binding_pairs`]) and the constellations they form
//! ([`constellations`]).
//!
//! Boundary: the cosmos reads orbits in galaxy rings, masses in core masses
//! and rates in radians per ring; this module still counts spans in kpc and
//! masses in suns, converted at [`radius_kpc`], [`rate_at_kpc`] and
//! [`pair_turning`] (`galaxy::kpc_of_rings_dy`, `galaxy::rings_of_kpc_dy`,
//! `galaxy::core_masses_of_suns`) until the kernel is converted.

use std::collections::BTreeMap;
use xyl_core::dyadic::Dy;
use xyl_core::units::Tier;
use xyl_cosmos::galaxy::{kpc_of_rings_dy, rings_of_kpc_dy, Star};

/// Returns a star's orbit radius in kpc (boundary).
fn radius_kpc(s: &Star) -> Dy {
    kpc_of_rings_dy(s.orbit.radius)
}

/// Returns the galaxy's turning (radians per ring) in the star's galaxy at
/// `at_kpc` kpc from its core (boundary over `Star::rate_at_dy`).
fn rate_at_kpc(s: &Star, at_kpc: Dy) -> Dy {
    s.rate_at_dy(rings_of_kpc_dy(at_kpc))
}

/// Returns the turning (radians per ring) that a body of `suns` gives a
/// neighbour `span_kpc` away: the master equation's pair term, the turning
/// law (`turning::turning_dy`) evaluated at the span (at least 1e-9 kpc),
/// the mass and span read in core masses and galaxy rings at the boundary.
fn pair_turning(suns: Dy, span_kpc: Dy) -> Dy {
    let m = Dy::from_q(&xyl_cosmos::galaxy::core_masses_of_suns(&suns.to_q()));
    let span = rings_of_kpc_dy(span_kpc.max(Dy::sci(1, -9)));
    xyl_cosmos::turning::turning_dy(m, span, Tier::Galaxy)
}

/// Returns the binding share of stars `a` and `b` (masses `ma`, `mb` in suns)
/// at separation `span_kpc`: `turn / (turn + shear)`, where `turn` is the
/// pair turning of the combined mass at the span and `shear` is the
/// difference between the inner star's turning `rate0` at its orbit radius
/// and the galaxy's turning one span further out. Both turnings come from the
/// same law, so the shear is a difference of that law. Returns zero for stars
/// of different cores or when both terms vanish.
pub fn binding_at(a: &Star, ma: Dy, b: &Star, mb: Dy, rate0: Dy, span_kpc: Dy) -> Dy {
    if a.core != b.core {
        return Dy::ZERO;
    }
    let turn = pair_turning(ma + mb, span_kpc);
    let inner = if a.orbit.radius <= b.orbit.radius {
        a
    } else {
        b
    };
    let shear = (rate0 - rate_at_kpc(inner, radius_kpc(inner) + span_kpc.max(Dy::ZERO))).abs();
    let both = turn + shear;
    if both.signum() <= 0 {
        Dy::ZERO
    } else {
        turn / both
    }
}

/// Returns a star's binding reach (kpc): the span at which the star binds an
/// identical neighbour with share one half, found by 40 bisections over
/// [0, r] with r the orbit radius (at least 1e-3 kpc). Returns r when the
/// share at r is still at least one half.
pub fn binding_reach(s: &Star, m: Dy) -> Dy {
    let half = Dy::ratio(1, 2);
    let rate0 = s.rate_at_dy(s.orbit.radius);
    let (mut lo, mut hi) = (Dy::ZERO, radius_kpc(s).max(Dy::sci(1, -3)));
    if binding_at(s, m, s, m, rate0, hi) >= half {
        return hi;
    }
    for _ in 0..40 {
        let mid = (lo + hi) / Dy::int(2);
        if binding_at(s, m, s, m, rate0, mid) >= half {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// A constellation: a set of stars joined by pairs whose mutual turning is
/// at least the galaxy's shear at their place (binding share >= 1/2). It is
/// the unit of passage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constellation {
    /// The stars by sample number, in ascending order; the least is the key.
    pub stars: Vec<u64>,
    /// The galactic core index of the stars (pairs bind only within one
    /// core).
    pub core: u32,
    /// The reach (kpc): the largest binding reach of the stars, and the width
    /// of a ley line's mouth in this constellation.
    pub reach_kpc: Dy,
}

impl Constellation {
    /// Returns the key: the least sample number of the stars (0 for an empty
    /// constellation, which the network never builds).
    pub fn key(&self) -> u64 {
        self.stars.first().copied().unwrap_or(0)
    }
}

fn find(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

/// A star in play: the star as the origin made it and the suns it stands
/// for at the sample's resolution (the star's `stands_for` times the sample
/// size over the number of stars in play in its galaxy).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InPlay {
    /// The star as the origin made it.
    pub star: Star,
    /// How many suns it carries at the sample's resolution (count of stars).
    pub suns: Dy,
    /// The binding reach (kpc), read once because it does not change with
    /// time.
    pub reach: Dy,
    /// The star's turning about the galaxy at its orbit radius, in radians per
    /// ring, read once.
    pub rate0: Dy,
}

impl InPlay {
    pub(super) fn of(star: Star, suns: Dy) -> InPlay {
        let reach = binding_reach(&star, suns);
        let rate0 = star.rate_at_dy(star.orbit.radius);
        InPlay {
            star,
            suns,
            reach,
            rate0,
        }
    }

    /// Returns an upper bound (kpc per Gyr) on the star's motion through the
    /// galaxy at time `t`, and so on how fast any arc to it can change: the
    /// circulation (orbital rate in radians per Gyr times orbit radius) plus
    /// the vertical swing through the plane at its largest (the vertical
    /// speed read 10 Gyr later, where the share saturates, times the scale).
    pub fn speed_bound(&self, t: Dy) -> Dy {
        // The cosmos counts in rings and galaxy rings; the rate, the speed
        // and the age convert at this boundary (a rate per ring read per Gyr
        // is `rings_of_gyr_dy`, a reach in kpc is `kpc_of_rings_dy`).
        let rate = xyl_cosmos::circuit::rings_of_gyr_dy(self.star.orbit.rate.abs());
        let circ = rate * radius_kpc(&self.star);
        let born = xyl_cosmos::circuit::gyr_of_rings_dy(Dy::from_q(&self.star.born));
        let age = (t - born).max(Dy::ZERO) + Dy::int(10);
        let swing = self
            .star
            .vertical_at(xyl_cosmos::circuit::rings_of_gyr_dy(age))
            .map(|v| {
                let per_ring = kpc_of_rings_dy(v.vz.abs() * self.star.scale);
                xyl_cosmos::circuit::rings_of_gyr_dy(per_ring)
            })
            .unwrap_or(Dy::ZERO);
        circ + swing
    }
}

/// Returns the stars in play: of each galaxy's first `limit` sample stars,
/// those the sight opens (as `minds::schedule` reads them), regenerated from
/// the seed (orbit and place only, no system). Each carries
/// m = stands_for x sample / n, read in suns (stands_for is in core masses,
/// converted at the boundary `galaxy::suns_of_core_masses`), where n is the
/// number of stars in play in that galaxy.
pub fn in_play(seed: u64, origin: &xyl_cosmos::origin::Origin, limit: u64) -> Vec<InPlay> {
    use xyl_cosmos::minds::Sight;
    // The stars are seated with the cores' sight only; each witnessed system
    // is added later by `Network::admit`, so the whole sample is never placed
    // at once.
    let sight = Sight::Cores;
    let sample = xyl_cosmos::galaxy::sample(origin);
    let total = limit.min(sample);
    let mut out = Vec::new();
    for c in xyl_cosmos::cores::held(seed, origin) {
        let stars: Vec<Star> = (0..total)
            .filter_map(|l| {
                xyl_cosmos::galaxy::star(seed, origin, xyl_cosmos::galaxy::in_galaxy(c, l))
            })
            .filter(|st| sight.opens(st))
            .collect();
        let w = Dy::int(i64::try_from(sample).unwrap_or(i64::MAX))
            / Dy::int(i64::try_from(stars.len().max(1)).unwrap_or(1));
        out.extend(stars.into_iter().map(|st| {
            // The stellar mass stood for is in core masses; this layer reads
            // it in suns (boundary `galaxy::suns_of_core_masses`).
            let stands_for = xyl_cosmos::galaxy::suns_of_core_masses(&st.stands_for.to_q());
            let suns = Dy::from_q(&stands_for) * w;
            InPlay::of(st, suns)
        }));
    }
    out
}

/// Returns the pairs that can bind, read once. The stars are sorted by
/// (core, orbit radius, sample number), and each is paired outward while the
/// binding at the difference of the orbit radii (a lower bound on the arc,
/// fixed for circling stars) is still at least one half. Each pair's span
/// s_ij, where the binding falls to one half, is bracketed by doubling (up to
/// 1000 kpc) and refined by 24 bisections. Returns (i, j, s_ij) with indices
/// into `stars`.
pub fn binding_pairs(stars: &[InPlay]) -> Vec<(usize, usize, Dy)> {
    let half = Dy::ratio(1, 2);
    let mut order: Vec<usize> = (0..stars.len()).collect();
    order.sort_by_key(|&a| {
        (
            stars[a].star.core,
            stars[a].star.orbit.radius,
            stars[a].star.n,
        )
    });
    let mut out = Vec::new();
    for (oi, &i) in order.iter().enumerate() {
        for &j in &order[oi + 1..] {
            let (a, b) = (&stars[i], &stars[j]);
            let dr = radius_kpc(&b.star) - radius_kpc(&a.star);
            let rate0 = if a.star.orbit.radius <= b.star.orbit.radius {
                a.rate0
            } else {
                b.rate0
            };
            let bind = |s: Dy| binding_at(&a.star, a.suns, &b.star, b.suns, rate0, s);
            if a.star.core != b.star.core || bind(dr) < half {
                break;
            }
            let mut lo = dr.max(Dy::ZERO);
            let mut hi = (lo * Dy::int(2)).max(Dy::sci(1, -6));
            while bind(hi) >= half && hi < Dy::sci(1, 3) {
                lo = hi;
                hi = hi * Dy::int(2);
            }
            for _ in 0..24 {
                let mid = (lo + hi) / Dy::int(2);
                if bind(mid) >= half {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            out.push((i, j, lo));
        }
    }
    out
}

/// Returns the constellations formed by joining every pair marked in
/// `bound`, sorted by key; an unbound star forms a constellation alone.
pub fn constellations(
    stars: &[InPlay],
    pairs: &[(usize, usize, Dy)],
    bound: &[bool],
) -> Vec<Constellation> {
    let mut parent: Vec<usize> = (0..stars.len()).collect();
    for (&(i, j, _), _) in pairs.iter().zip(bound).filter(|(_, b)| **b) {
        let (x, y) = (find(&mut parent, i), find(&mut parent, j));
        parent[x.max(y)] = x.min(y);
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..stars.len() {
        let r = find(&mut parent, i);
        groups.entry(r).or_default().push(i);
    }
    let mut out: Vec<Constellation> = groups
        .into_values()
        .map(|g| {
            let mut ks: Vec<u64> = g.iter().map(|&i| stars[i].star.n).collect();
            ks.sort_unstable();
            let reach = g.iter().map(|&i| stars[i].reach).max().unwrap_or(Dy::ZERO);
            Constellation {
                stars: ks,
                core: stars[g[0]].star.core,
                reach_kpc: reach,
            }
        })
        .collect();
    out.sort_by_key(|c| c.key());
    out
}
