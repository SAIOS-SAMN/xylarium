//! The pulse network: the holes that relay the core's bounce wavefront, the
//! relay chain the wavefront follows, and what a place hears of it at `t`.
//!
//! Model: the core's bounce emits one wavefront (no oscillator, no
//! harmonics) that runs out to the founding's rim and back once per bell.
//! Its wavelength is light speed x the bell (`wavelength_kpc`, one ring of
//! light, twice the home galaxy's rim), so no
//! separate constant is set. Every hole (black face, throat, white face; see
//! `centre`) passes the wavefront on continuous in phase, without a separate
//! period.
//!
//! Reads `origin::Origin`, `cores::shapes` and `cores::held`, `galaxy::star`,
//! `chain::Place` and `chain::radius`, `galaxy::Star::sequence`,
//! `stars::life` and `stars::remnant_share`, `circuit::bell` and `circuit::anchor` (the bell
//! in core time), `chain::units` (r_g) and `package::Si` (kpc). Times are
//! rings after the origin (one ring one bell), distances kpc, masses suns,
//! all as `Dy`.
//!
//! Outputs:
//! - `nodes`, `nodes_among` -> `Vec<Node>`: every galactic core from its
//!   formation, and each sample star's remnant from the end of its life,
//!   of mass `stands_for x remnant_share` of its sequence; a held body that
//!   does not ignite never ends and is no node. A subset of candidates only omits
//!   nodes.
//! - `relays` -> one `Relay` per node: the nearest live node closer to the
//!   core, the route along that chain, and continuity
//!   `cos(2 pi detour / wavelength)`. `nodes[0]` is the universe core.
//! - `hear` -> `Option<Hearing>` (`None` with no live node): each node weighs
//!   mass / arc (first degree, suns per kpc) at the phase of its path; the
//!   dominance is the loudest node's share of the sum, the coherence
//!   `|sum I_k e^(i psi_k)| / sum I_k`, the phasor sum's length read by
//!   projecting it on its angle so no square root is taken.

use crate::chain::Place;
use crate::galaxy::{star, Star};
use crate::origin::Origin;
use xyl_core::dyadic::Dy;
use xyl_core::rotor::Winding;
use xyl_core::Q;

/// Light speed in kpc per ring: the bell in core time (`Anchor::bell`,
/// units of r_g / c) times r_g in kpc, so no second enters. Zero before a
/// founding.
pub fn c_kpc_per_ring() -> Dy {
    let rg_kpc = crate::chain::units()
        .r_g_m
        .checked_div(&crate::package::Si::kpc())
        .unwrap_or(Q::ZERO);
    Dy::from_q(&(&crate::circuit::anchor().bell * &rg_kpc))
}

/// The kind of node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A galactic core: its galaxy's source bell (`cores`).
    Core(Box<crate::cores::Shape>),
    /// The remnant hole a sample star leaves when its life ends.
    Stellar(Box<Star>),
}

/// A bell of the network.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// The node's kind: a galactic core or a stellar hole.
    pub kind: Kind,
    /// The hole mass the node represents (suns), which sets its intensity.
    pub mass: Dy,
    /// When it formed (rings after the origin).
    pub born: Dy,
}

impl Node {
    /// The node's place at `t` rings on the universe's 3-sphere of radius `r`
    /// (galaxy units): a core at the chain's anchor, a stellar hole where its
    /// progenitor star's manifold puts it.
    pub fn place(&self, t: Dy, r: &Q) -> Place {
        match &self.kind {
            Kind::Core(sh) => sh.place_at(t, r),
            // Stand-in S005: "progenitor star" names the star a remnant collapsed from.
            Kind::Stellar(s) => s.place(t, r),
        }
    }
}

/// The network's bells with their formation times: the cores, and every
/// sample star once its life ends (the first `limit` of the disc and bulge
/// sample if given, and all of the nucleus's).
pub fn nodes(seed: u64, origin: &Origin, limit: Option<u64>) -> Vec<Node> {
    let cap = limit.unwrap_or(u64::MAX);
    let candidates: Vec<u64> = crate::cores::held(origin)
        .into_iter()
        .flat_map(|c| crate::galaxy::indices_of(origin, c))
        .filter(|&k| {
            let local = k & ((1u64 << crate::galaxy::CORE_SHIFT) - 1);
            local >= crate::galaxy::NUCLEUS_BASE || local < cap
        })
        .collect();
    nodes_among(seed, origin, &candidates)
}

/// The bells among the sample stars `candidates`: the cores, and each
/// candidate's remnant from the end of its life. Callers may pass a subset
/// of the sample; an incomplete list omits bells but never adds one.
pub fn nodes_among(seed: u64, origin: &Origin, candidates: &[u64]) -> Vec<Node> {
    // Each galactic core is a node from its formation time.
    let mut out: Vec<Node> = crate::cores::shapes(origin)
        .into_iter()
        .map(|sh| Node {
            mass: Dy::from_q(&sh.hole_suns),
            born: Dy::from_q(&sh.born),
            kind: Kind::Core(Box::new(sh)),
        })
        .collect();
    for &k in candidates {
        let Some(s) = star(seed, origin, k) else {
            continue;
        };
        // A stellar node forms when the star's main-sequence life ends
        // (`stars::life`); a held body whose sequence has no step never ends
        // and forms none.
        let seq = s.sequence();
        if !seq.ignites() {
            continue;
        }
        let life = Dy::from_q(&crate::stars::life(&seq, s.compactness(), s.reach_units()));
        // Stand-in S001: Every igniting star is made a hole at its first step's end.
        let born = Dy::from_q(&s.born) + life;
        // Remnant mass: the stellar mass the sample star stands for times
        // the share of the hold no step of its sequence transduced
        // (`stars::remnant_share`),
        // read in suns as the cores' masses are (boundary
        // `galaxy::suns_of_core_masses`).
        let stands_for = crate::galaxy::suns_of_core_masses(&s.stands_for.to_q());
        // Stand-in S002: The sample star's whole represented population becomes one hole here.
        let mass = Dy::from_q(&stands_for)
            * Dy::from_q(&crate::stars::remnant(
                &seq,
                s.compactness(),
                s.reach_units(),
            ));
        out.push(Node {
            kind: Kind::Stellar(Box::new(s)),
            mass,
            born,
        });
    }
    out
}

/// The core's period (rings): the bell, the pulse's round trip to the
/// founding's rim and back (`circuit::bell`), one ring.
pub fn core_period() -> Dy {
    crate::circuit::bell()
}

/// The wavefront's wavelength (kpc): light speed x the core's period.
pub fn wavelength_kpc() -> Dy {
    c_kpc_per_ring() * crate::circuit::bell()
}

/// What a place hears of the bells at `t` rings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hearing {
    /// The loudest bell's index in the nodes.
    pub dominant: usize,
    /// The loudest bell's share of the summed intensity, in (0, 1].
    pub dominance: Dy,
    /// Phase coherence with the loudest bell, first-degree (E011): each
    /// bell's share of the intensity times its arrival planes' overlap with
    /// the loudest bell's and `1 - arc / 180` of their phase difference on
    /// the T lane, summed, in [0, 1]. No phasor, cosine or root.
    pub coherence: Dy,
    /// The summed intensity (suns per kpc).
    pub intensity: Dy,
}

/// A node's relay at `t`: the bell it relays from (the live bell nearest to
/// it among those closer to the core; none for the core), the wavefront's
/// route length along the relay chain (kpc), and its continuity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Relay {
    /// Index of the node relayed from; `None` for the core and for nodes not yet born.
    pub from: Option<usize>,
    /// Wavefront route length from the core along the relay chain (kpc).
    pub route: Dy,
    /// Continuity: the faces' push of the detour's phase, `1 - arc / 90`
    /// with `arc` its shorter arc in degrees, in [-1, 1]; 1 for a direct
    /// route.
    pub continuity: Dy,
}

/// The relay network at `t` rings over `nodes`, one [`Relay`] per node in the
/// same order. Nodes not yet born keep the default relay (no source, route 0,
/// continuity 1). A node with no live source closer to the core takes the
/// direct arc from the core as its route.
///
/// `nodes[0]` is taken as the universe core; an empty `nodes` gives an empty
/// network.
pub fn relays(nodes: &[Node], t: Dy) -> Vec<Relay> {
    let r = crate::chain::radius();
    let places: Vec<Place> = nodes.iter().map(|x| x.place(t, &r)).collect();
    let arc = |a: &Place, b: &Place| a.arc_kpc(b, &r);
    let Some(core) = places.first() else {
        return Vec::new();
    };
    let from_core: Vec<Dy> = places.iter().map(|p| arc(p, core)).collect();
    // Nearest the core first, so each relay's source already has its route.
    let mut order: Vec<usize> = (0..nodes.len()).collect();
    order.sort_by(|&a, &b| from_core[a].cmp(&from_core[b]));
    let mut out = vec![
        Relay {
            from: None,
            route: Dy::ZERO,
            continuity: Dy::ONE
        };
        nodes.len()
    ];
    let lambda = wavelength_kpc();
    for &i in &order {
        if i == 0 || nodes[i].born > t {
            continue;
        }
        let from = order
            .iter()
            .copied()
            .filter(|&j| j != i && nodes[j].born <= t && from_core[j] < from_core[i])
            .min_by(|&a, &b| arc(&places[i], &places[a]).cmp(&arc(&places[i], &places[b])));
        let route = match from {
            Some(j) => out[j].route + arc(&places[i], &places[j]),
            None => from_core[i],
        };
        let detour = route - from_core[i];
        // The detour's phase on the T lane, its shorter arc in degrees; the
        // continuity is the faces' push of that arc, held less passed,
        // `1 - arc / 90`, first-degree (E011).
        let phase = Winding::of_turns(&detour.div(&lambda).unwrap_or_default().to_q());
        let continuity = Dy::from_q(
            &(&Q::ONE - &xyl_core::hopf::arc(&phase.degrees).over(xyl_core::nonzero!(90))),
        );
        out[i] = Relay {
            from,
            route,
            continuity,
        };
    }
    out
}

/// What `place` hears of the live bells at `t` rings; `None` when no node is
/// live. Arcs are at least 1e-6 kpc.
pub fn hear(place: &Place, nodes: &[Node], relays: &[Relay], t: Dy) -> Option<Hearing> {
    let r = crate::chain::radius();
    let lambda = wavelength_kpc();
    // Each live bell: its intensity, its arrival on the T lane (whole
    // wavelengths and phase) and the planes it arrives on.
    let mut heard: Vec<(usize, Dy, Winding, Option<[Q; 3]>)> = Vec::new();
    for (k, node) in nodes.iter().enumerate() {
        if node.born > t {
            continue;
        }
        let there = node.place(t, &r);
        let d = place.arc_kpc(&there, &r).max(Dy::sci(1, -6));
        // Weight I = M / d (first degree, in place of M / d^2).
        // Stand-in S025: Intensity falls as M / d, the hold missing from its denominator.
        let i = node.mass.div(&d).unwrap_or_default();
        // Path: the route along the relay chain to this node, plus the arc
        // from it to the place.
        let path = relays.get(k).map(|r| r.route).unwrap_or(Dy::ZERO) + d;
        let arrival = Winding::of_turns(&path.div(&lambda).unwrap_or_default().to_q());
        heard.push((k, i, arrival, arrival_planes(place, &there)));
    }
    let sum = heard.iter().fold(Dy::ZERO, |s, h| s + h.1);
    let (k, top, dom_phase, dom_planes) = heard
        .iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
        .map(|h| (h.0, h.1, h.2.clone(), h.3.clone()))?;
    let coherence = heard.iter().fold(Dy::ZERO, |c, (_, i, phase, planes)| {
        let share = i.div(&sum).unwrap_or_default();
        let along = match (planes, &dom_planes) {
            (Some(p), Some(q)) => p
                .iter()
                .zip(q)
                .fold(Q::ZERO, |s, (a, b)| &s + &a.clone().min(b.clone())),
            (None, None) => Q::ONE,
            _ => Q::ZERO,
        };
        let apart = xyl_core::hopf::arc(&phase.less(&dom_phase).degrees);
        let held = &along * &(&Q::ONE - &apart.over(xyl_core::nonzero!(180)));
        c + share * Dy::from_q(&held)
    });
    Some(Hearing {
        dominant: k,
        dominance: top.div(&sum).unwrap_or_default(),
        coherence,
        intensity: sum,
    })
}

/// The planes a bell arrives on at a place: the pivot's plane between their
/// directions on one sphere (between their cores' across galaxies), as
/// shares `|b_ij| / sum |b|` of planes (1, 2), (1, 3) and (2, 3). `None`
/// where no one plane carries one direction to the other (the same
/// direction, or antipodal ones).
fn arrival_planes(place: &Place, there: &Place) -> Option<[Q; 3]> {
    let core_of = |p: &Place| p.core.as_deref().cloned().unwrap_or_else(|| p.clone());
    let (a, b) = if place.galaxy == there.galaxy {
        (place.clone(), there.clone())
    } else {
        (core_of(place), core_of(there))
    };
    let pivot = xyl_core::rotor::pivot(&a.dir, &b.dir, 64)?;
    let total = pivot.planes.iter().fold(Q::ZERO, |s, p| &s + &p.1.abs());
    let share = |plane: (usize, usize)| {
        pivot
            .planes
            .iter()
            .filter(|p| p.0 == plane)
            .fold(Q::ZERO, |s, p| &s + &p.1.abs())
            .checked_div(&total)
    };
    Some([share((1, 2))?, share((1, 3))?, share((2, 3))?])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The hearing and the relays read first-degree (E011): coherence lies
    /// in [0, 1] and one bell is wholly coherent with itself; continuity lies
    /// in [-1, 1] and is 1 on the core's own direct route.
    #[test]
    fn a_place_hears_the_bells_first_degree() {
        let origin = crate::genesis::origin();
        let all = nodes(7, origin, Some(8));
        // Read once every bell is live.
        let t = all.iter().map(|n| n.born).max().unwrap_or(Dy::ZERO) + Dy::ONE;
        let relays = relays(&all, t);
        assert!(relays
            .iter()
            .all(|r| r.continuity >= -Dy::ONE && r.continuity <= Dy::ONE));
        assert_eq!(relays.first().map(|r| r.continuity), Some(Dy::ONE));
        let r = crate::chain::radius();
        let Some(core) = all.first() else {
            return;
        };
        let here = core.place(t, &r);
        let heard = hear(&here, &all, &relays, t).unwrap();
        assert!(
            heard.coherence >= Dy::ZERO && heard.coherence <= Dy::ONE,
            "{heard:?}"
        );
        assert!(heard.dominance > Dy::ZERO && heard.dominance <= Dy::ONE);
        let alone = hear(&here, &all[..1], &relays[..1], t).unwrap();
        assert_eq!(alone.coherence, Dy::ONE);
    }
}
