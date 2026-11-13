//! The coordinate's keys and lanes (FORMS-COORDINATE v4, 78.173 section 1):
//! an axis is a capital letter and a positive index (`S1`) or a plane unit
//! (`+k`, `-k`); a key is one axis or a generator, a plane named by two axes
//! in the orientation written; H stands only on turning planes; a lane is a
//! vector of components keyed by their axes, never summed; a lane set holds
//! S, G, T and H and any lane a ruled opening admits.

use super::CoordinateRefusal;
use crate::Q;
use std::cmp::Ordering;

/// An axis: a capital letter and a positive index, or a plane unit, plane
/// `k`'s positive unit `e+k` (squaring to +1) or negative unit `e-k`
/// (squaring to -1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    /// A lettered axis, `S1`, `G1`, `T1`: no square named.
    Letter(char, u32),
    /// A plane unit of plane `k`: positive (`+k`) or negative (`-k`).
    Unit(bool, u32),
}

impl Axis {
    /// A lettered axis; refused `AxisUnwritable` for a letter that is not an
    /// ASCII capital or an index of 0.
    pub fn of(letter: char, index: u32) -> Result<Axis, CoordinateRefusal> {
        if !letter.is_ascii_uppercase() || index == 0 {
            return Err(CoordinateRefusal::AxisUnwritable { letter, index });
        }
        Ok(Axis::Letter(letter, index))
    }

    /// Plane `k`'s unit, positive or negative; refused `AxisUnwritable` for
    /// `k = 0`.
    pub fn unit(positive: bool, k: u32) -> Result<Axis, CoordinateRefusal> {
        if k == 0 {
            let letter = if positive { '+' } else { '-' };
            return Err(CoordinateRefusal::AxisUnwritable { letter, index: k });
        }
        Ok(Axis::Unit(positive, k))
    }

    /// The axis written: `S1`, `+2`, `-3`.
    pub fn written(&self) -> String {
        match self {
            Axis::Letter(l, i) => format!("{l}{i}"),
            Axis::Unit(true, k) => format!("+{k}"),
            Axis::Unit(false, k) => format!("-{k}"),
        }
    }
}

/// What a generator's plane is, read from its two axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plane {
    /// Two units of one square on two planes, `e+k e+l` (the continuum) or
    /// `e-k e-l` (the vacuum): a turning plane.
    Turning,
    /// A plane's own pair, `e+k e-k`: a boost.
    Boost,
    /// A mixed pair, `e+k e-l`: a boost.
    Mixed,
    /// A generator of lettered axes: no square named.
    Unsigned,
}

/// A key: one axis, or a generator, the plane of two axes in the
/// orientation written.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    /// One axis, written `e_S1`.
    Axis(Axis),
    /// A generator, written `e_+2+3`.
    Generator(Axis, Axis),
}

impl Key {
    /// A generator of `a` then `b`; refused `NotPlane` for one axis twice.
    pub fn generator(a: Axis, b: Axis) -> Result<Key, CoordinateRefusal> {
        if a == b {
            return Err(CoordinateRefusal::NotPlane { axis: a.written() });
        }
        Ok(Key::Generator(a, b))
    }

    /// The key written: `e_` and its axes.
    pub fn written(&self) -> String {
        match self {
            Key::Axis(a) => format!("e_{}", a.written()),
            Key::Generator(a, b) => format!("e_{}{}", a.written(), b.written()),
        }
    }

    /// The plane a generator names; `None` for one axis.
    pub fn plane(&self) -> Option<Plane> {
        match self {
            Key::Axis(_) => None,
            Key::Generator(Axis::Unit(sa, ka), Axis::Unit(sb, kb)) => Some(if sa == sb {
                Plane::Turning
            } else if ka == kb {
                Plane::Boost
            } else {
                Plane::Mixed
            }),
            Key::Generator(_, _) => Some(Plane::Unsigned),
        }
    }

    /// The same plane named in the other orientation, for a generator.
    fn reversed(&self) -> Option<Key> {
        match self {
            Key::Generator(a, b) => Some(Key::Generator(*b, *a)),
            Key::Axis(_) => None,
        }
    }
}

/// A lane's vector: exact signed components, each keyed by its axis, held
/// in the keys' written order, never summed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vector {
    components: Vec<(Key, Q)>,
}

impl Vector {
    /// The vector of `components`, held in the keys' written order. Refused
    /// `KeyRepeated` for a key held twice or a plane named in both
    /// orientations.
    pub fn of(mut components: Vec<(Key, Q)>) -> Result<Vector, CoordinateRefusal> {
        #[expect(
            clippy::disallowed_methods,
            reason = "rule 4: the keys' written order is the canonical key, not a choice"
        )]
        components.sort_by_key(|(k, _)| k.written());
        for (i, (k, _)) in components.iter().enumerate() {
            let later = &components[i + 1..];
            let twice = later.iter().any(|(o, _)| o == k);
            let turned = k
                .reversed()
                .is_some_and(|r| later.iter().any(|(o, _)| *o == r));
            if twice || turned {
                return Err(CoordinateRefusal::KeyRepeated { key: k.written() });
            }
        }
        Ok(Vector { components })
    }

    /// The vector of one component.
    pub fn one(key: Key, value: Q) -> Vector {
        Vector {
            components: vec![(key, value)],
        }
    }

    /// The components in the keys' written order.
    pub fn components(&self) -> &[(Key, Q)] {
        &self.components
    }

    /// The keys written, in order.
    pub fn keys(&self) -> Vec<String> {
        self.components.iter().map(|(k, _)| k.written()).collect()
    }

    /// The component keyed `key`.
    pub fn get(&self, key: &Key) -> Option<&Q> {
        self.components
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// The vector written: one component as its value, several joined in
    /// parentheses (`(1/3,7/4)`), each value in lowest terms in hexadecimal.
    pub fn written(&self) -> String {
        let parts: Vec<String> = self.components.iter().map(|(_, v)| hex_q(v)).collect();
        if parts.len() == 1 {
            parts.concat()
        } else {
            format!("({})", parts.join(","))
        }
    }
}

impl Ord for Vector {
    /// Component by component: each key written, then its exact value.
    fn cmp(&self, other: &Vector) -> Ordering {
        for ((ka, va), (kb, vb)) in self.components.iter().zip(&other.components) {
            let by = ka.written().cmp(&kb.written()).then_with(|| va.cmp(vb));
            if by != Ordering::Equal {
                return by;
            }
        }
        self.components.len().cmp(&other.components.len())
    }
}

impl PartialOrd for Vector {
    fn partial_cmp(&self, other: &Vector) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// An exact rational written in lowest terms, numerator and denominator in
/// lowercase hexadecimal, a negative value signed with `-` (`-1e/1`).
pub(super) fn hex_q(v: &Q) -> String {
    let (n, d) = (v.numer(), v.denom());
    let sign = if n.sign() == num_bigint::Sign::Minus {
        "-"
    } else {
        ""
    };
    format!("{sign}{:x}/{:x}", n.magnitude(), d.magnitude())
}

/// A lane: S, G, T and H, the base lanes, or a lane a ruled opening admits,
/// named with its law version.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lane {
    /// Spatial: boost and dilation, the reach.
    S,
    /// Gauge: change of kind.
    G,
    /// Temporal: null flux, the beat and the passages.
    T,
    /// Holonomic: turnings between planes.
    H,
    /// A lane opened by a ruled act: its name and law version.
    Opened(String, u32),
}

/// The ruled openings: the lanes beyond S, G, T and H a ruled act admits.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Openings {
    opened: Vec<(String, u32)>,
}

impl Openings {
    /// The openings ruled: none yet (78.173 section 7).
    pub fn ruled() -> Openings {
        Openings::default()
    }

    /// The openings a claim states, each a lane's name and law version.
    pub fn stated(opened: Vec<(String, u32)>) -> Openings {
        Openings { opened }
    }

    /// Whether `name` at `version` is opened.
    fn admits(&self, name: &str, version: u32) -> bool {
        self.opened.iter().any(|(n, v)| n == name && *v == version)
    }
}

/// A lane set: S, G, T and H always, then any opened lane, in that order.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lanes {
    lanes: Vec<(Lane, Vector)>,
}

impl Lanes {
    /// The lane set of `lanes` under `openings`. Refused by name: a base
    /// lane missing (`LaneMissing`), a lane held twice (`LaneRepeated`), a
    /// lane no ruled opening admits (`LaneNotOpened`), and on H a key that
    /// is one axis (`NotGenerator`) or a plane that holds no turning
    /// (`NotTurningPlane`, naming the plane).
    pub fn of(
        mut lanes: Vec<(Lane, Vector)>,
        openings: &Openings,
    ) -> Result<Lanes, CoordinateRefusal> {
        for (i, (l, _)) in lanes.iter().enumerate() {
            if lanes[i + 1..].iter().any(|(o, _)| o == l) {
                return Err(CoordinateRefusal::LaneRepeated { lane: l.clone() });
            }
            if let Lane::Opened(name, version) = l {
                if !openings.admits(name, *version) {
                    return Err(CoordinateRefusal::LaneNotOpened {
                        name: name.clone(),
                        version: *version,
                    });
                }
            }
        }
        for base in [Lane::S, Lane::G, Lane::T, Lane::H] {
            if !lanes.iter().any(|(l, _)| *l == base) {
                return Err(CoordinateRefusal::LaneMissing { lane: base });
            }
        }
        for (l, v) in &lanes {
            if *l != Lane::H {
                continue;
            }
            for (k, _) in v.components() {
                match k.plane() {
                    None => {
                        return Err(CoordinateRefusal::NotGenerator { key: k.written() });
                    }
                    Some(Plane::Turning) => {}
                    Some(plane) => {
                        return Err(CoordinateRefusal::NotTurningPlane {
                            key: k.written(),
                            plane,
                        });
                    }
                }
            }
        }
        #[expect(
            clippy::disallowed_methods,
            reason = "rule 4: the lanes' order S, G, T, H, opened is the coordinate's canonical key"
        )]
        lanes.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(Lanes { lanes })
    }

    /// The four base lanes with no opening.
    pub fn base(s: Vector, g: Vector, t: Vector, h: Vector) -> Result<Lanes, CoordinateRefusal> {
        Lanes::of(
            vec![(Lane::S, s), (Lane::G, g), (Lane::T, t), (Lane::H, h)],
            &Openings::ruled(),
        )
    }

    /// The lanes in order.
    pub fn lanes(&self) -> &[(Lane, Vector)] {
        &self.lanes
    }

    /// The lane `lane`.
    pub fn get(&self, lane: &Lane) -> Option<&Vector> {
        self.lanes.iter().find(|(l, _)| l == lane).map(|(_, v)| v)
    }

    /// The lane set with each component mapped, keys kept; the first
    /// refusal of `f` refuses the whole.
    pub(super) fn try_map(
        &self,
        f: impl Fn(&Lane, &Key, &Q) -> Result<Q, CoordinateRefusal>,
    ) -> Result<Lanes, CoordinateRefusal> {
        let mut lanes = Vec::with_capacity(self.lanes.len());
        for (l, v) in &self.lanes {
            let mut components = Vec::with_capacity(v.components().len());
            for (k, x) in v.components() {
                components.push((*k, f(l, k, x)?));
            }
            lanes.push((l.clone(), Vector { components }));
        }
        Ok(Lanes { lanes })
    }

    /// The component `key` of `lane`; refused `NoComponent`.
    pub fn component(&self, lane: &Lane, key: &Key) -> Result<&Q, CoordinateRefusal> {
        self.get(lane)
            .and_then(|v| v.get(key))
            .ok_or_else(|| CoordinateRefusal::NoComponent {
                lane: lane.clone(),
                key: key.written(),
            })
    }

    /// Whether `other` holds the same lanes keyed alike; where not, the
    /// first lane that differs, with its keys held and asked.
    pub(super) fn shape_against(&self, other: &Lanes) -> Result<(), CoordinateRefusal> {
        for (l, v) in &self.lanes {
            let asked = match other.get(l) {
                Some(o) => o.keys(),
                None => Vec::new(),
            };
            if v.keys() != asked {
                return Err(CoordinateRefusal::ShapesDiffer {
                    lane: l.clone(),
                    held: v.keys(),
                    asked,
                });
            }
        }
        if other.lanes.len() != self.lanes.len() {
            return Err(CoordinateRefusal::ShapesDiffer {
                lane: Lane::H,
                held: Vec::new(),
                asked: Vec::new(),
            });
        }
        Ok(())
    }

    /// The H lane's whole turns per component, `floor(|theta| / 360)`.
    /// Refused `TurnsUnbounded` past a machine word.
    pub(super) fn windings(&self) -> Result<Vec<u64>, CoordinateRefusal> {
        let Some(h) = self.get(&Lane::H) else {
            return Ok(Vec::new());
        };
        h.components()
            .iter()
            .map(|(k, theta)| {
                let turns = theta.abs().over(super::DEGREES_NONZERO);
                u64::try_from(turns.floor())
                    .map_err(|_| CoordinateRefusal::TurnsUnbounded { key: k.written() })
            })
            .collect()
    }

    /// The lanes written, `:`-joined in order.
    pub fn written(&self) -> String {
        let parts: Vec<String> = self.lanes.iter().map(|(_, v)| v.written()).collect();
        parts.join(":")
    }
}
