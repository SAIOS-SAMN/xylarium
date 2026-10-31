//! Binding and constellations: the pair binding of two stars against the
//! galaxy's shear ([`binding_at`]), the stars in play ([`in_play`]), the
//! pairs that can bind ([`binding_pairs`]) and the constellations they form
//! ([`constellations`]).

use super::kms_to_kpc_gyr;
use std::collections::BTreeMap;
use xyl_core::dyadic::Dy;
use xyl_cosmos::galaxy::Star;

/// The turn a body of `suns` gives a neighbour `span_kpc` away (degrees per
/// Gyr): the master equation's pair term, the turning law at their span.
fn pair_turning(suns: Dy, span_kpc: Dy) -> Dy {
    xyl_cosmos::turning::turning_kpc_dy(suns, span_kpc.max(Dy::sci(1, -9)))
}

/// The same, with the inner star's own turning `rate0` already read (by the
/// same law as the turning across the span, so the shear is the law's own
/// difference).
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
    let shear = (rate0 - inner.rate_at_dy(inner.orbit.radius + span_kpc.max(Dy::ZERO))).abs();
    let both = turn + shear;
    if both.signum() <= 0 {
        Dy::ZERO
    } else {
        turn / both
    }
}

/// A star's binding reach (kpc): the span at which it binds a neighbour like
/// itself by a half, found by bisection over [0, r].
pub fn binding_reach(s: &Star, m: Dy) -> Dy {
    let half = Dy::ratio(1, 2);
    let rate0 = s.rate_at_dy(s.orbit.radius);
    let (mut lo, mut hi) = (Dy::ZERO, s.orbit.radius.max(Dy::sci(1, -3)));
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

/// A constellation: the stars whose mutual turnings bind them tighter than
/// the galaxy's shear at their place; the unit of passage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constellation {
    /// Its stars (sample indices), least first: the least is its key.
    pub stars: Vec<u64>,
    /// Galactic core index of its least star.
    pub core: u32,
    /// Its reach (kpc): the widest binding reach of its stars, and the width of
    /// a ley line's mouth there.
    pub reach_kpc: Dy,
}

impl Constellation {
    /// Its key: its least star's sample number (0 for an empty
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

/// A star in play: the star as the Big Bang made it and the suns it carries
/// at the sample's resolution (its own share times the sample over the stars
/// worked out in its galaxy).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InPlay {
    /// The star as the Big Bang made it.
    pub star: Star,
    /// How many suns it carries at the sample's resolution (count of stars).
    pub suns: Dy,
    /// Its binding reach (kpc) and its own turning (degrees per Gyr), read
    /// once: neither moves with time.
    pub reach: Dy,
    /// Its own turning about its galaxy at its orbit's radius, in degrees per Gyr.
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

    /// The most it moves through its galaxy a Gyr (kpc): its circulation,
    /// its turning times its reach, and its swing through the plane at its
    /// fullest (its spread read 10 Gyr on, the share's saturation); a bound
    /// on how fast any arc to it can change.
    pub fn speed_bound(&self, t: Dy) -> Dy {
        let circ = self.star.orbit.rate.abs() * Dy::PI / Dy::int(180) * self.star.orbit.radius;
        let age = (t - Dy::from_q(&self.star.born)).max(Dy::ZERO) + Dy::int(10);
        let swing = self
            .star
            .vertical_at(age)
            .map(|v| v.vz_kms.abs() * self.star.scale * kms_to_kpc_gyr())
            .unwrap_or(Dy::ZERO);
        circ + swing
    }
}

/// The stars in play: each galaxy's first `limit` sample stars that its sight
/// opens (as `minds::schedule` reads them), regenerated from the seed (orbit
/// and place only, no system), each carrying
/// m = stands_for x sample / worked in its galaxy.
pub fn in_play(seed: u64, bang: &xyl_cosmos::bang::BigBang, limit: u64) -> Vec<InPlay> {
    use xyl_cosmos::minds::Sight;
    // Seated with the cores' sight only; each witnessed system is added later
    // by `Network::admit`, so the whole sample is never placed at once.
    let sight = Sight::Cores;
    let sample = xyl_cosmos::galaxy::sample(bang);
    let total = limit.min(sample);
    let mut out = Vec::new();
    for c in xyl_cosmos::cores::held(seed, bang) {
        let stars: Vec<Star> = (0..total)
            .filter_map(|l| {
                xyl_cosmos::galaxy::star(seed, bang, xyl_cosmos::galaxy::in_galaxy(c, l))
            })
            .filter(|st| sight.opens(st))
            .collect();
        let w = Dy::int(i64::try_from(sample).unwrap_or(i64::MAX))
            / Dy::int(i64::try_from(stars.len().max(1)).unwrap_or(1));
        out.extend(stars.into_iter().map(|st| {
            let suns = st.stands_for * w;
            InPlay::of(st, suns)
        }));
    }
    out
}

/// The pairs that can bind, read once: stars sorted by (core, reach), each
/// paired outward while their binding at the reaches' difference (a lower
/// bound on their arc, fixed for circling stars) still holds a half; each
/// pair's span s_ij where its binding falls to a half, found by bisection.
/// Returns (i, j, s_ij), indices into `stars`.
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
            let dr = b.star.orbit.radius - a.star.orbit.radius;
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

/// The constellations from the pairs bound now: each bound pair joined.
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
