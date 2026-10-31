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
//! Reads `bang::BigBang`, `cores::shapes` and `cores::held`, `galaxy::star`,
//! `chain::Place` and `chain::radius`, `stars::life` and
//! `stars::remnant_share`, `circuit::bell` and `circuit::anchor` (the bell
//! in core time), `chain::units` (r_g) and `package::Si` (kpc). Times are
//! rings after the Big Bang (one ring one bell), distances kpc, masses suns,
//! all as `Dy`.
//!
//! Outputs:
//! - `nodes`, `nodes_among` -> `Vec<Node>`: every galactic core from its
//!   formation, and each sample star's remnant from the end of its life,
//!   of mass `stands_for x remnant_share`. A subset of candidates only omits
//!   nodes.
//! - `relays` -> one `Relay` per node: the nearest live node closer to the
//!   core, the route along that chain, and continuity
//!   `cos(2 pi detour / wavelength)`. `nodes[0]` is the universe core.
//! - `hear` -> `Option<Hearing>` (`None` with no live node): each node weighs
//!   mass / arc (first degree, suns per kpc) at the phase of its path; the
//!   dominance is the loudest node's share of the sum, the coherence
//!   `|sum I_k e^(i psi_k)| / sum I_k`, the phasor sum's length read by
//!   projecting it on its angle so no square root is taken.

use crate::bang::BigBang;
use crate::chain::Place;
use crate::galaxy::{star, Star};
use xyl_core::dyadic::Dy;
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
    /// When it formed (rings after the Big Bang).
    pub born: Dy,
}

impl Node {
    /// The node's place at `t` rings on the universe's 3-sphere of radius `r`
    /// (galaxy units): a core at the chain's anchor, a stellar hole where its
    /// progenitor star's manifold puts it.
    pub fn place(&self, t: Dy, r: &Q) -> Place {
        match &self.kind {
            Kind::Core(sh) => sh.place_at(t, r),
            Kind::Stellar(s) => s.place(t, r),
        }
    }
}

/// The network's bells with their formation times: the cores, and every
/// sample star once its life ends (the first `limit` of the disc and bulge
/// sample if given, and all of the nucleus's).
pub fn nodes(seed: u64, bang: &BigBang, limit: Option<u64>) -> Vec<Node> {
    let cap = limit.unwrap_or(u64::MAX);
    let candidates: Vec<u64> = crate::cores::held(seed, bang)
        .into_iter()
        .flat_map(|c| crate::galaxy::indices_of(bang, c))
        .filter(|&k| {
            let local = k & ((1u64 << crate::galaxy::CORE_SHIFT) - 1);
            local >= crate::galaxy::NUCLEUS_BASE || local < cap
        })
        .collect();
    nodes_among(seed, bang, &candidates)
}

/// The bells among the sample stars `candidates`: the cores, and each
/// candidate's remnant from the end of its life. Callers may pass a subset
/// of the sample; an incomplete list omits bells but never adds one.
pub fn nodes_among(seed: u64, bang: &BigBang, candidates: &[u64]) -> Vec<Node> {
    // Each galactic core is a node from its formation time.
    let mut out: Vec<Node> = crate::cores::shapes(seed, bang)
        .into_iter()
        .map(|sh| Node {
            mass: Dy::from_q(&sh.hole_suns),
            born: Dy::from_q(&sh.born),
            kind: Kind::Core(Box::new(sh)),
        })
        .collect();
    for &k in candidates {
        let Some(s) = star(seed, bang, k) else {
            continue;
        };
        // A stellar node forms when the star's main-sequence life ends
        // (`stars::life`).
        let life = Dy::from_q(&crate::stars::life(s.reach_rings()));
        let born = Dy::from_q(&s.born) + life;
        // Remnant mass: the stellar mass the sample star stands for times
        // the share of the hold not transduced (`stars::remnant_share`).
        let mass = s.stands_for * Dy::from_q(&crate::stars::remnant_share());
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
    c_kpc_per_ring() * core_period()
}

/// What a place hears of the bells at `t` rings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hearing {
    /// The loudest bell's index in the nodes.
    pub dominant: usize,
    /// The loudest bell's share of the summed intensity, in (0, 1].
    pub dominance: Dy,
    /// Phase coherence: the phasor sum's length over the summed intensity, in [0, 1].
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
    /// Continuity, cos(2 pi detour / wavelength), in [-1, 1]; 1 for a direct route.
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
        let continuity = (Dy::int(2) * Dy::PI * detour / lambda).cos();
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
    let (mut sum, mut re, mut im) = (Dy::ZERO, Dy::ZERO, Dy::ZERO);
    let mut best: Option<(usize, Dy)> = None;
    for (k, node) in nodes.iter().enumerate() {
        if node.born > t {
            continue;
        }
        let d = place.arc_kpc(&node.place(t, &r), &r).max(Dy::sci(1, -6));
        // Weight I = M / d (first degree, in place of M / d^2).
        let i = node.mass / d;
        // Path: the route along the relay chain to this node, plus the arc
        // from it to the place.
        let path = relays.get(k).map(|r| r.route).unwrap_or(Dy::ZERO) + d;
        let (s, c) = (Dy::int(2) * Dy::PI * path / lambda).sin_cos();
        sum += i;
        re += i * c;
        im += i * s;
        if best.is_none_or(|(_, b)| i > b) {
            best = Some((k, i));
        }
    }
    let (k, top) = best?;
    // The phasor sum's length from its phase: re cos(phi) + im sin(phi), with
    // phi the sum's angle (avoids the root of re^2 + im^2).
    let phi = if re.signum() == 0 {
        Dy::PI * Dy::ratio(1, 2) * Dy::int(i64::from(im.signum()))
    } else {
        (im / re).atan() + if re.signum() < 0 { Dy::PI } else { Dy::ZERO }
    };
    let (s, c) = phi.sin_cos();
    let length = (re * c + im * s).abs();
    Some(Hearing {
        dominant: k,
        dominance: top / sum,
        coherence: length / sum,
        intensity: sum,
    })
}
