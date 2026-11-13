//! A body's coordinate (FORMS-COORDINATE v4, 78.173 section 1):
//! `origin / h : S:G:T:H : K`, its writing, its canonical order, `within`
//! on the caller's intervals, and the bodies a kernel holds, with a range on
//! one component found by two bisections and a filter within it.

use super::keys::{hex_q, Key, Lane, Lanes};
use super::{CoordinateRefusal, Side};
use crate::Q;
use std::cmp::Ordering;

/// A body's place within its enclosure: its bell `h`, its lanes and `K`,
/// the whole turns of each H component. An origin's enclosures are written
/// as the places their branches were cut at.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Local {
    /// The bell.
    pub h: u64,
    /// The lanes.
    pub lanes: Lanes,
    /// The whole turns per H component.
    pub k: Vec<u64>,
}

impl Local {
    /// Written: `h` in sixteen hexadecimal digits, the lanes, and `K`, one
    /// count as sixteen digits, several in parentheses.
    pub fn written(&self) -> String {
        let ks: Vec<String> = self.k.iter().map(|k| format!("{k:016x}")).collect();
        let k = if ks.len() == 1 {
            ks.concat()
        } else {
            format!("({})", ks.join(","))
        };
        format!("{:016x}:{}:{}", self.h, self.lanes.written(), k)
    }
}

/// A body's origin: the universe's root (its genesis hash and the epoch
/// TUniversal carries) and the enclosures the body stands in, outermost
/// first, each written out as the place its branch was cut at.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Origin {
    /// The genesis hash.
    pub genesis: [u8; 32],
    /// The epoch.
    pub epoch: u32,
    /// The enclosures, outermost first.
    pub enclosures: Vec<Local>,
}

impl Origin {
    /// The universe's root.
    pub fn root(genesis: [u8; 32], epoch: u32) -> Origin {
        Origin {
            genesis,
            epoch,
            enclosures: Vec::new(),
        }
    }

    /// This origin enclosed once more, in the branch cut at `at`.
    pub fn within(&self, at: Local) -> Origin {
        let mut enclosures = self.enclosures.clone();
        enclosures.push(at);
        Origin {
            enclosures,
            ..self.clone()
        }
    }

    /// Written: the genesis hash in hexadecimal, `.`, the epoch in eight
    /// digits, then `/` and each enclosure's place.
    pub fn written(&self) -> String {
        let hash: String = self.genesis.iter().map(|b| format!("{b:02x}")).collect();
        let mut out = format!("{hash}.{:08x}", self.epoch);
        for e in &self.enclosures {
            out.push('/');
            out.push_str(&e.written());
        }
        out
    }
}

/// A body's coordinate: its origin and its place there.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Coordinate {
    /// The origin.
    pub origin: Origin,
    /// The place within it.
    pub local: Local,
}

impl Coordinate {
    /// A coordinate at `origin`, bell `h`, with `lanes`, `K` read from H.
    /// Refused `TurnsUnbounded` past a machine word.
    pub fn of(origin: Origin, h: u64, lanes: Lanes) -> Result<Coordinate, CoordinateRefusal> {
        let k = lanes.windings()?;
        Ok(Coordinate {
            origin,
            local: Local { h, lanes, k },
        })
    }

    /// A coordinate with `K` stated, not read.
    pub fn with_k(origin: Origin, h: u64, lanes: Lanes, k: Vec<u64>) -> Coordinate {
        Coordinate {
            origin,
            local: Local { h, lanes, k },
        }
    }

    /// Written: the origin, `/`, and the place.
    pub fn written(&self) -> String {
        format!("{}/{}", self.origin.written(), self.local.written())
    }

    /// Where the body stands against `bounds`: a side per component of the
    /// bounds' lane. Refused `OriginsDiffer` across origins and
    /// `ShapesDiffer` where the bounds are keyed otherwise than the lane.
    pub fn within(&self, bounds: &Bounds) -> Result<Sides, CoordinateRefusal> {
        if self.origin != bounds.origin {
            return Err(CoordinateRefusal::OriginsDiffer {
                held: self.origin.written(),
                asked: bounds.origin.written(),
            });
        }
        let held = match self.local.lanes.get(&bounds.lane) {
            Some(v) => v.keys(),
            None => Vec::new(),
        };
        let asked: Vec<String> = bounds.intervals.iter().map(|(k, _)| k.written()).collect();
        if held != asked {
            return Err(CoordinateRefusal::ShapesDiffer {
                lane: bounds.lane.clone(),
                held,
                asked,
            });
        }
        let mut sides = Vec::with_capacity(bounds.intervals.len());
        for (k, i) in &bounds.intervals {
            let v = self.local.lanes.component(&bounds.lane, k)?;
            sides.push((k.written(), i.side(v)));
        }
        Ok(Sides { sides })
    }

    /// A lane read for display at `depth`: each component's sector written
    /// (`Sector`), joined by ` x ` (`0;20 x 1;45`); an H component reads
    /// directly as degrees;minutes,seconds. Display only. Refused where the
    /// lane is not held or a sector is unbounded.
    pub fn reading(&self, lane: &Lane, depth: u32) -> Result<String, CoordinateRefusal> {
        let v = self
            .local
            .lanes
            .get(lane)
            .ok_or(CoordinateRefusal::LaneMissing { lane: lane.clone() })?;
        let mut parts = Vec::with_capacity(v.components().len());
        for (_, x) in v.components() {
            let sector = super::Sector::of(x, depth)?;
            parts.push(
                sector
                    .written()
                    .ok_or(CoordinateRefusal::Unbounded { depth })?,
            );
        }
        Ok(parts.join(" \u{d7} "))
    }

    /// The component `key` of `lane`.
    pub fn component(&self, lane: &Lane, key: &Key) -> Option<&Q> {
        self.local.lanes.get(lane)?.get(key)
    }
}

/// An exact interval `[lo, hi)` the caller states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interval {
    /// The lower bound, held.
    pub lo: Q,
    /// The upper bound, not held.
    pub hi: Q,
}

impl Interval {
    /// Where `v` lies against the interval.
    pub fn side(&self, v: &Q) -> Side {
        if v < &self.lo {
            Side::Below
        } else if v < &self.hi {
            Side::Within
        } else {
            Side::Above
        }
    }
}

/// Bounds stated at an origin: an interval per component of one lane, keyed
/// in the lane's order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bounds {
    /// The origin they are stated at.
    pub origin: Origin,
    /// The lane they bound.
    pub lane: Lane,
    /// An interval per key, in the keys' written order.
    pub intervals: Vec<(Key, Interval)>,
}

/// A side per component; within as a whole only where every component is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sides {
    /// Each component's key written and its side.
    pub sides: Vec<(String, Side)>,
}

impl Sides {
    /// Within as a whole, or the first component that is not, by name.
    pub fn whole(&self) -> Result<(), (String, Side)> {
        match self.sides.iter().find(|(_, s)| *s != Side::Within) {
            Some((k, s)) => Err((k.clone(), *s)),
            None => Ok(()),
        }
    }
}

/// The bodies a kernel holds, in the canonical order: the index it
/// addresses them by. An address routes and never identifies.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bodies {
    held: Vec<Coordinate>,
}

/// A range's result: the bodies in it, in order, and the probes each of its
/// two bisections took.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Range<'a> {
    /// The bodies, in the canonical order.
    pub bodies: &'a [Coordinate],
    /// The probes of the two bisections.
    pub probes: (u32, u32),
}

impl Bodies {
    /// No bodies.
    pub fn new() -> Bodies {
        Bodies::default()
    }

    /// The bodies, in order.
    pub fn held(&self) -> &[Coordinate] {
        &self.held
    }

    /// Holds `body` at its place in the order; returns its index.
    pub fn hold(&mut self, body: Coordinate) -> usize {
        let at = self.held.partition_point(|b| b < &body);
        self.held.insert(at, body);
        at
    }

    /// The bodies sharing `at`'s origin, bell, the lanes before `lane` and
    /// the components of `lane` keyed before `key`, whose component `key`
    /// lies in `interval`: one contiguous run of the order, found by two
    /// bisections. Refused `NoComponent` where `at` holds no such component.
    pub fn range<'a>(
        &'a self,
        at: &Coordinate,
        lane: &Lane,
        key: &Key,
        interval: &Interval,
    ) -> Result<Range<'a>, CoordinateRefusal> {
        at.component(lane, key)
            .ok_or_else(|| CoordinateRefusal::NoComponent {
                lane: lane.clone(),
                key: key.written(),
            })?;
        let prefix = |b: &Coordinate| -> Ordering {
            b.origin
                .cmp(&at.origin)
                .then(b.local.h.cmp(&at.local.h))
                .then_with(|| {
                    for (l, v) in b.local.lanes.lanes() {
                        let other = at.local.lanes.get(l);
                        if l == lane {
                            for (k, x) in v.components() {
                                if k == key {
                                    return Ordering::Equal;
                                }
                                let theirs = other.and_then(|o| o.get(k));
                                let by = theirs.map_or(Ordering::Greater, |t| x.cmp(t));
                                if by != Ordering::Equal {
                                    return by;
                                }
                            }
                            return Ordering::Equal;
                        }
                        let by = other.map_or(Ordering::Greater, |o| v.cmp(o));
                        if by != Ordering::Equal {
                            return by;
                        }
                    }
                    Ordering::Equal
                })
        };
        let value = |b: &Coordinate| b.component(lane, key).cloned();
        let below = |b: &Coordinate, bound: &Q| match prefix(b) {
            Ordering::Less => true,
            Ordering::Greater => false,
            Ordering::Equal => value(b).is_none_or(|v| &v < bound),
        };
        let (lo, p1) = bisect(&self.held, |b| below(b, &interval.lo));
        let (hi, p2) = bisect(&self.held, |b| below(b, &interval.hi));
        let bodies =
            self.held
                .get(lo..hi.max(lo))
                .ok_or_else(|| CoordinateRefusal::NoComponent {
                    lane: lane.clone(),
                    key: key.written(),
                })?;
        Ok(Range {
            bodies,
            probes: (p1, p2),
        })
    }
}

impl Range<'_> {
    /// The bodies of the range whose component `key` of `lane` lies in
    /// `interval`: a filter within the range.
    pub fn filter(&self, lane: &Lane, key: &Key, interval: &Interval) -> Vec<&Coordinate> {
        self.bodies
            .iter()
            .filter(|b| {
                b.component(lane, key)
                    .is_some_and(|v| interval.side(v) == Side::Within)
            })
            .collect()
    }
}

/// The first index where `below` turns false, and the probes taken, by
/// bisection over an order in which `below` holds on a prefix.
fn bisect(held: &[Coordinate], below: impl Fn(&Coordinate) -> bool) -> (usize, u32) {
    let (mut lo, mut hi, mut probes) = (0usize, held.len(), 0u32);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        probes += 1;
        if held.get(mid).is_some_and(&below) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    (lo, probes)
}

/// The writing of a value in a lane: kept for the TUniversal point's case.
pub fn written_value(v: &Q) -> String {
    hex_q(v)
}
