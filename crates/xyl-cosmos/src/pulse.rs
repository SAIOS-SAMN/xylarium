//! The pulse field: time as the singularities' bounce ringing out and echoing
//! back, and the network of bells that carries it.
//!
//! Every singularity is a bell, and every bell is a triple manifold: its
//! black-hole face, its throat, and its white-hole face (centre.rs: a paired
//! throat). The core's bounce emits a single wavefront (not an oscillator or
//! harmonics) as spheres from its epicentre at the speed of gravity, to the
//! rim and back; its wavelength is light speed x the core's round trip, so no
//! constant is set for it.
//!
//! The singularities act as shock absorbers and relays: a wavefront reaching
//! a bell passes through its three faces and continues, continuous in phase;
//! a relay has no period of its own. Its far face continues in another
//! universe, so a universe's bells join external networks through their
//! throats. A place hears each live bell by its strength (mass) over its
//! distance, at the phase of the wavefront's path (core, then that bell, then
//! the place):
//!
//!   Dominance    the strongest bell's share of the total: 1 is one clear
//!                bell (near the core), small is many faint ones (the rim)
//!   Coherence    |sum_k I_k e^(i psi_k)| / sum_k I_k, psi_k the phase of bell
//!                k's path: whether the wavefronts arrive in step
//!   Continuity   for each relay, cos(2 pi x route detour / wavelength): 1
//!                where the wavefront passes through unbroken, falling as the
//!                route (core to relay along the relay chain) departs from the
//!                direct ray
//!
//! The nodes: the core's hole, and every sample star once its life ends
//! (stars::life, the pulse's burn of its hold), leaving the hold the pulse did
//! not transduce (stars::remnant_share), standing for the remnants of the
//! stars its sample represents.

use crate::bang::BigBang;
use crate::chain::Place;
use crate::galaxy::{star, Star};
use xyl_core::dyadic::Dy;
use xyl_core::Q;

fn n(x: i64) -> Dy {
    Dy::int(x)
}

/// Light speed in kpc per year, from the one SI seam (`package::Si`).
pub fn c_kpc_yr() -> Dy {
    use crate::package::Si;
    Dy::from_q(&Si::c()) * Dy::from_q(&Si::yr_s()) / Dy::from_q(&Si::kpc())
}

/// The kind of node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A galactic core: its galaxy's source bell (cores.rs).
    Core(Box<crate::cores::Shape>),
    /// The hole a heavy sample star left.
    Stellar(Box<Star>),
}

/// A bell of the network.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// What rings: a galactic core or a stellar hole.
    pub kind: Kind,
    /// Ringing strength: the hole mass it represents (suns).
    pub mass: Dy,
    /// When it formed (Gyr after the Big Bang).
    pub born: Dy,
}

impl Node {
    /// The node's place at `t` Gyr on the universe's 3-sphere of radius `r`
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
    // every galactic core rings from when it formed
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
        // lifetime: its hold over the pulse's burn at its reach
        // (stars::life), then the remnant
        let life = Dy::from_q(&crate::stars::life(s.reach_rings()));
        let born = Dy::from_q(&s.born) + life;
        // the remnants of the stars this sample represents: stands_for suns
        // of similar stars, each keeping the hold the pulse did not transduce
        let mass = s.stands_for * Dy::from_q(&crate::stars::remnant_share());
        out.push(Node {
            kind: Kind::Stellar(Box::new(s)),
            mass,
            born,
        });
    }
    out
}

/// The core's period (yr): the bell, the pulse's round trip to the
/// founding's rim and back (circuit::anchor).
pub fn core_period_yr() -> Dy {
    crate::circuit::bell_gyr() * Dy::sci(1, 9)
}

/// The wavefront's wavelength (kpc): light speed x the core's period.
pub fn wavelength_kpc() -> Dy {
    c_kpc_yr() * core_period_yr()
}

/// What a place hears of the bells at `t` Gyr.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hearing {
    /// The loudest bell (its index in the nodes) and that bell's share.
    pub dominant: usize,
    /// The loudest bell's share of the summed intensity, in (0, 1].
    pub dominance: Dy,
    /// Phase coherence: the phasor sum's length over the summed intensity, in [0, 1].
    pub coherence: Dy,
    /// The summed intensity (suns per kpc^2).
    pub intensity: Dy,
}

/// The relay network at `t`: for each live node, the bell it relays from (the
/// nearest live bell closer to the core; none for the core), the wavefront's
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

/// The relay network at `t` Gyr over `nodes`, one [`Relay`] per node in the
/// same order. Nodes not yet born keep the default relay (no source, route 0).
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
    // nearest the core first, so each relay's source has its route already
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
        let continuity = (n(2) * Dy::PI * detour / lambda).cos();
        out[i] = Relay {
            from,
            route,
            continuity,
        };
    }
    out
}

/// What `place` hears of the live bells at `t` Gyr.
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
        // a bell's weight here: its mass over the arc (first degree, not d^2)
        let i = node.mass / d;
        // the wavefront's path: along the relay chain to this bell, then here
        let path = relays.get(k).map(|r| r.route).unwrap_or(Dy::ZERO) + d;
        let (s, c) = (n(2) * Dy::PI * path / lambda).sin_cos();
        sum += i;
        re += i * c;
        im += i * s;
        if best.is_none_or(|(_, b)| i > b) {
            best = Some((k, i));
        }
    }
    let (k, top) = best?;
    // the phasor sum's length via its own phase: re cos(phi) + im sin(phi),
    // phi its angle (avoids the root of re^2 + im^2)
    let phi = if re.signum() == 0 {
        Dy::PI * Dy::ratio(1, 2) * n(i64::from(im.signum()))
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
