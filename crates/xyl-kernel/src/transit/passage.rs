//! Passage times between places within a system, within a constellation, or
//! across constellations over the ley lines ([`Network::passage_years`]).

use super::{crossing_years, cruise_speed, kpc_m, separation_m, voyage, window_years, year_s};
use super::{Network, Place};
use crate::cosmic::Cosmic;
use crate::propulsion::Spec;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};
use xyl_core::dyadic::Dy;
use xyl_cosmos::chain::units;
use xyl_cosmos::manifold::manifolds;

impl Network {
    /// Returns the passage time between places `a` and `b` (years). Within a
    /// system it is the drive's leg across the gap between the orbits plus
    /// the window wait `w`, weighted by `w v_orb / (v_orb + cruise)`. Within a
    /// constellation it is the drive's leg along the arc between the stars.
    /// Between constellations the passage runs only over the ley lines (each
    /// crossing takes the light time across the reach of the constellation
    /// at the line's second mouth, divided by the line's open share), with
    /// drive legs to and from the mouths. A line
    /// carries the passer `who` (a people's index) as its handling allows: a
    /// closed line carries no one, a guarded line only its builder and the
    /// peoples of its realm. Returns None when the cruise speed is not
    /// positive or no passage joins the two places.
    pub fn passage_years(
        &self,
        cosmic: &Cosmic,
        spec: &Spec,
        ratio: Dy,
        escape: Dy,
        a: Place,
        b: Place,
        who: Option<usize>,
    ) -> Option<Dy> {
        let cruise = cruise_speed(spec, ratio, escape);
        if cruise.signum() <= 0 {
            return None;
        }
        if a.star == b.star {
            let years = voyage(spec, ratio, escape, separation_m(cosmic, a, b)?)?.years;
            let wit = cosmic.systems.get(&a.star)?;
            let all = manifolds(cosmic.gal.seed, &[a.star], &wit.system, &wit.star, &units());
            let age = (cosmic.now
                - xyl_cosmos::circuit::gyr_of_rings_dy(Dy::from_q(&wit.star.born)))
            .max(Dy::ZERO)
            .to_q();
            let wait = window_years(&all, a.world, b.world, &age, years)
                .map(|(w, v)| w * v / (v + cruise))
                .unwrap_or(Dy::ZERO);
            return Some(years + wait);
        }
        let (ca, cb) = (*self.of.get(&a.star)?, *self.of.get(&b.star)?);
        let star_leg = |x: u64, y: u64| {
            self.arc(cosmic, x, y)
                .map(|arc| arc * kpc_m() / cruise / year_s())
        };
        if ca == cb {
            return star_leg(a.star, b.star);
        }
        // The graph's nodes are every line mouth plus the two ends a and b. Its
        // edges are each line that carries the passer, weighted by the
        // crossing time divided by the open share, and a drive leg between
        // every two nodes of one constellation.
        let mut nodes: Vec<u64> = vec![a.star, b.star];
        for l in &self.lines {
            nodes.push(l.from);
            nodes.push(l.to);
        }
        nodes.sort_unstable();
        nodes.dedup();
        let mut edges: Vec<(u64, u64, Dy)> = Vec::new();
        for l in &self.lines {
            let open = self.open_share(l);
            if open.signum() <= 0 || !self.carries(cosmic, l, who) {
                continue;
            }
            edges.push((
                l.from,
                l.to,
                crossing_years(self.reach_of(l.to).unwrap_or(Dy::ZERO)) / open,
            ));
        }
        let mut by_c: BTreeMap<usize, Vec<u64>> = BTreeMap::new();
        for &x in &nodes {
            if let Some(&c) = self.of.get(&x) {
                by_c.entry(c).or_default().push(x);
            }
        }
        for group in by_c.values() {
            for (i, &x) in group.iter().enumerate() {
                for &y in &group[i + 1..] {
                    if let Some(t) = star_leg(x, y) {
                        edges.push((x, y, t));
                    }
                }
            }
        }
        shortest(&edges, a.star, b.star)
    }
}

/// Returns the least total weight from `from` to `to` over the undirected
/// weighted `edges` (Dijkstra's search), or None when they are not joined.
fn shortest(edges: &[(u64, u64, Dy)], from: u64, to: u64) -> Option<Dy> {
    let mut adj: BTreeMap<u64, Vec<(u64, Dy)>> = BTreeMap::new();
    for &(x, y, c) in edges {
        adj.entry(x).or_default().push((y, c));
        adj.entry(y).or_default().push((x, c));
    }
    let mut dist: BTreeMap<u64, Dy> = BTreeMap::new();
    dist.insert(from, Dy::ZERO);
    let mut heap = BinaryHeap::new();
    heap.push(Reverse((Dy::ZERO, from)));
    while let Some(Reverse((d, s))) = heap.pop() {
        if s == to {
            return Some(d);
        }
        if dist.get(&s).is_some_and(|&best| d > best) {
            continue;
        }
        for &(w, c) in adj.get(&s).map(Vec::as_slice).unwrap_or(&[]) {
            let nd = d + c;
            if dist.get(&w).is_none_or(|&best| nd < best) {
                dist.insert(w, nd);
                heap.push(Reverse((nd, w)));
            }
        }
    }
    None
}
