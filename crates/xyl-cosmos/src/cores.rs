//! The cores (docs/model.md): the universe core is the Big Bang's anchor, and
//! its founding produces the galaxies. The home galaxy is index 0, at the
//! universe core; its scale is the calibration until the home galaxy is
//! derived from the core's reach. Every other galaxy is a bead of the
//! founding cascade (circuit/found.rs) that holds a sun: its reach and phase
//! about the universal throat, its state (share of the founding's matter),
//! and its recipe (the kinds carried into it plus its intake from the core by
//! t). Each metal's factor is its share of the composition over its share of
//! the elements' founding, not a drawn preset.

use crate::bang::BigBang;
use crate::elements::Element;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// A galactic core's shape: what the galaxy code needs, without its recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shape {
    /// Engine index of the galaxy: 0 for the home galaxy at the universe core.
    pub index: u32,
    /// The core hole's mass (suns).
    pub hole_suns: Q,
    /// Position relative to the universe core at the founding (cluster tier,
    /// exact), kept for reference; the bearing below is used for placement.
    pub place: [Q; 3],
    /// Bearing from the universe core: reach (cluster tier) and phase about
    /// it (turns).
    pub reach: Q,
    /// Phase about the universe core at the founding (turns).
    pub phase: Q,
    /// When it formed (Gyr after the Big Bang).
    pub born: Q,
    /// The galaxy's plane on S^360: tilt and node as half-angle directions,
    /// set by magnetic repulsion among the galaxies; flat for the home galaxy.
    pub tilt: (Dy, Dy),
    /// Node of the galaxy's plane about the universe's axis, as a half-angle
    /// direction (c, s); (1, 0) for the home galaxy.
    pub node: (Dy, Dy),
    /// Circulation rate about the throat, 4 delta per echo, in turns per Gyr;
    /// zero for the home galaxy at the core.
    pub rate: Dy,
}

impl Shape {
    /// The core's place at `t` Gyr on the universe's sphere of radius `r`: its
    /// plane's frame (node about the axis, then tilt) applied to its phase at
    /// t, at its reach.
    pub fn place_at(&self, t: Dy, r: &Q) -> crate::chain::Place {
        use xyl_core::rotor::{Half, Rotor};
        let phase = (Dy::from_q(&self.phase) + self.rate * t).to_q().frac();
        let (c, s) = xyl_core::certified::cos_sin_turns(&xyl_core::certified::Encl::exact(phase));
        let h = |p: (Dy, Dy)| Half {
            c: p.0.to_q(),
            s: p.1.to_q(),
        };
        let frame = Rotor::about(2, &h(self.node)).then_after(&Rotor::about(0, &h(self.tilt)));
        let dir = frame
            .apply(&[c, s, xyl_core::certified::Encl::exact(Q::ZERO)])
            .map(|x| x.mid());
        crate::chain::Place::core(self.anchor().0, dir, r)
    }

    /// Hole mass over the home galaxy's; every mass in the galaxy scales by it.
    pub fn mass_ratio(&self) -> Q {
        self.hole_suns
            .checked_div(&crate::package::hole_mass())
            .unwrap_or(Q::ONE)
    }

    /// Length scale over the home galaxy's, compressed. Under the turning law a
    /// linearly scaled galaxy would be an exact copy (same clocks, years and
    /// heat), so a heavier galaxy is tighter: R = R_loose / (1 + M / M_a), with
    /// R_loose the mass ratio and M_a a declared ruler (the home galaxy's mass),
    /// normalised so the home's is 1: scale = 2 ratio / (1 + ratio), exact. A
    /// galaxy of twice the mass is 4/3 the size, so it turns faster and its
    /// clocks run slower.
    pub fn scale(&self) -> Dy {
        if self.index == 0 {
            return Dy::ONE;
        }
        let ratio = self.mass_ratio();
        Dy::from_q(
            &(&Q::int(2) * &ratio)
                .checked_div(&(&Q::ONE + &ratio))
                .unwrap_or(Q::ONE),
        )
    }

    /// Bearing in the galaxy tier's unit (exact: 60^2 galaxy units per cluster
    /// unit): reach from the universe core and phase.
    pub fn anchor(&self) -> (Q, Q) {
        let k = crate::manifold::dilation(
            xyl_core::units::Tier::Cluster,
            xyl_core::units::Tier::Galaxy,
        );
        (&self.reach * &k, self.phase.clone())
    }
}

/// The mass ratio for a compressed scale: scale = 2 r / (1 + r), so
/// r = scale / (2 - scale), exact.
pub fn ratio_of_scale(scale: &Q) -> Q {
    scale.checked_div(&(&Q::int(2) - scale)).unwrap_or(Q::ONE)
}

/// A galactic core: its shape and its recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GalacticCore {
    /// The core's shape: bearing, plane, circulation and hole mass.
    pub shape: Shape,
    /// The galaxy's recipe: each element's factor on what its stars make.
    pub weights: Weights,
}

/// A galaxy's recipe: each element's factor on what its stars make, from its
/// bead's founding mix; the home galaxy is the calibration (1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Weights {
    /// Each metal's factor by symbol; 0 for an element absent from the bead.
    factors: Option<BTreeMap<String, Q>>,
}

impl Weights {
    /// The calibration recipe: every element at its founding abundance.
    pub fn home() -> Weights {
        Weights { factors: None }
    }

    /// A bead's recipe: each metal's share of the bead's metals over its share
    /// of the elements' founding metals (by mass), exact. Hydrogen and helium
    /// are fixed at 1.
    pub fn of_mix(mix: &[(u32, Q)], elements: &[Element]) -> Weights {
        let metals: Vec<&Element> = elements.iter().filter(|e| e.z >= 3).collect();
        let solar = metals
            .iter()
            .fold(Q::ZERO, |a, e| &a + &(&e.a * &e.abundance));
        let bead = mix
            .iter()
            .filter(|(z, _)| *z >= 3)
            .fold(Q::ZERO, |a, (_, m)| &a + m);
        let mut factors = BTreeMap::new();
        for e in metals {
            let mine = mix
                .iter()
                .find(|(z, _)| *z == e.z)
                .map(|(_, m)| m.clone())
                .unwrap_or(Q::ZERO);
            let own = mine.checked_div(&bead).unwrap_or(Q::ZERO);
            let theirs = (&e.a * &e.abundance).checked_div(&solar).unwrap_or(Q::ZERO);
            factors.insert(e.sym.clone(), own.checked_div(&theirs).unwrap_or(Q::ZERO));
        }
        Weights {
            factors: Some(factors),
        }
    }

    /// The factor on element `sym`.
    pub fn factor(&self, sym: &str) -> Q {
        if sym == "H" || sym == "He" {
            return Q::ONE;
        }
        match &self.factors {
            None => Q::ONE,
            Some(f) => f.get(sym).cloned().unwrap_or(Q::ZERO),
        }
    }

    /// A description of the recipe's source, for diagnostics.
    pub fn label(&self) -> &'static str {
        if self.factors.is_none() {
            "home (the calibration)"
        } else {
            "its bead's founding mix"
        }
    }
}

/// The founding, computed once per universe and cached: the substrate and
/// the galaxies' shapes on it.
struct Founding {
    substrate: Arc<crate::circuit::Substrate>,
    shapes: Vec<Shape>,
}

fn cache() -> &'static Mutex<BTreeMap<(u64, String), Arc<Founding>>> {
    static C: OnceLock<Mutex<BTreeMap<(u64, String), Arc<Founding>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// The substrate founded at the parent's `spin`, once per spin per process:
/// the founding depends on the laws and the spin alone (the torsion share
/// sets the chain and the cascade), not on a seed's other draws. The first
/// founding sets the anchor (circuit::anchor). None where the element table
/// fails.
pub fn substrate_for(spin: &Q) -> Option<Arc<crate::circuit::Substrate>> {
    static S: OnceLock<Mutex<BTreeMap<String, Arc<crate::circuit::Substrate>>>> = OnceLock::new();
    let table = S.get_or_init(|| Mutex::new(BTreeMap::new()));
    let drawn = format!("{spin}");
    if let Some(s) = table.lock().ok()?.get(&drawn) {
        return Some(s.clone());
    }
    // the admission: the founding at the drawn spin, refounded at the least
    // admissible spin where its firmament is not whole (bang::spin_of_share
    // of Substrate::least_share), until the margin closes or the spin is
    // extremal; the anchor is the admitted founding's
    let mut spin = spin.clone();
    let mut found = None;
    for _ in 0..3 {
        let elements = crate::elements::founded(&crate::bang::torsion_share(&spin)).ok()?;
        let (s, anchor) = crate::circuit::Substrate::found(&spin, &elements);
        let least = crate::bang::spin_of_share(&s.least_share());
        let whole = s.margin().signum() >= 0 || spin >= least;
        found = Some((s, anchor, spin.clone()));
        if whole {
            break;
        }
        spin = least;
    }
    let (s, anchor, admitted) = found?;
    crate::circuit::set_anchor(anchor);
    let s = Arc::new(s);
    let mut t = table.lock().ok()?;
    t.insert(drawn, s.clone());
    t.insert(format!("{admitted}"), s.clone());
    Some(s)
}

fn founding_of(seed: u64, bang: &BigBang) -> Option<Arc<Founding>> {
    let _ = seed;
    let key = (0u64, format!("{}", bang.parent.spin));
    if let Some(f) = cache().lock().ok()?.get(&key) {
        return Some(f.clone());
    }
    let substrate = substrate_for(&bang.parent.spin)?;
    let first = bang.first_stars_gyr().mid();
    // the home galaxy at the universe core
    let flat = (Dy::ONE, Dy::ZERO);
    let mut shapes = vec![Shape {
        index: 0,
        hole_suns: crate::package::hole_mass(),
        place: [Q::ZERO, Q::ZERO, Q::ZERO],
        reach: Q::ZERO,
        phase: Q::ZERO,
        born: first.clone(),
        tilt: flat,
        node: flat,
        rate: Dy::ZERO,
    }];
    // a bead's reach about the throat is read in the cluster tier as founded
    // (one core ring is one cluster unit, circuit::anchor)
    for (index, bead) in substrate.galaxies().into_iter().skip(1) {
        let g = bead?;
        let out = g.reach.clone();
        let (cs, sn) = xyl_core::rotor::Half::of_turns(&g.phase, 60).cos_sin();
        // a galactic core's mass in the universal core's units: the core
        // holds the universal state, 1, so a bead's state is its mass over
        // the core's; a galaxy is held where its core is no heavier than the
        // universal core (the deficit law, M >= m)
        let hole_suns = &crate::package::hole_mass() * &g.state;
        // the plane, by magnetic repulsion among the galaxies (the stars' law
        // one tier up): the golden turn sets the node, the plastic share of
        // the sphere's measure sets the tilt
        let pivot = |by: &Q| Dy::from_q(&(&Q::int(i64::from(index)) * by).frac());
        let (tilt, node) = (
            crate::galaxy::tilt_of(pivot(&crate::galaxy::plastic_turns())),
            crate::galaxy::node_of(pivot(&crate::galaxy::golden_turns())),
        );
        // circulation about the throat: 4 delta per echo at its reach
        let delta = Dy::from_q(&g.inside)
            / (Dy::from_q(&g.reach) + Dy::from_q(&g.inside)).max(Dy::sci(1, -300));
        let rate = Dy::int(4) * delta / crate::circuit::echo_gyr();
        // place at the founding, for reference (the bearing at t is place_at)
        let dir0 = {
            use xyl_core::rotor::{Half, Rotor};
            let h = |p: (Dy, Dy)| Half {
                c: p.0.to_q(),
                s: p.1.to_q(),
            };
            let frame = Rotor::about(2, &h(node)).then_after(&Rotor::about(0, &h(tilt)));
            frame
                .apply(&[
                    xyl_core::certified::Encl::exact(cs.clone()),
                    xyl_core::certified::Encl::exact(sn.clone()),
                    xyl_core::certified::Encl::exact(Q::ZERO),
                ])
                .map(|x| x.mid())
        };
        shapes.push(Shape {
            index,
            hole_suns,
            place: [&dir0[0] * &out, &dir0[1] * &out, &dir0[2] * &out],
            reach: out.clone(),
            phase: g.phase.clone(),
            born: first.clone(),
            tilt,
            node,
            rate,
        });
    }
    let f = Arc::new(Founding { substrate, shapes });
    cache().lock().ok()?.insert(key, f.clone());
    Some(f)
}

/// The galaxies' indices, home first.
pub fn held(seed: u64, bang: &BigBang) -> Vec<u32> {
    shapes(seed, bang).into_iter().map(|s| s.index).collect()
}

/// Galactic core `index`'s shape (None: no such galaxy).
pub fn shape(seed: u64, bang: &BigBang, index: u32) -> Option<Shape> {
    founding_of(seed, bang)?
        .shapes
        .iter()
        .find(|s| s.index == index)
        .cloned()
}

/// The founding's galaxies' shapes, home first.
pub fn shapes(seed: u64, bang: &BigBang) -> Vec<Shape> {
    founding_of(seed, bang)
        .map(|f| f.shapes.clone())
        .unwrap_or_default()
}

/// The substrate of a universe's founding, founded once per process and
/// shared (None if the founding fails).
pub fn substrate(seed: u64, bang: &BigBang) -> Option<Arc<crate::circuit::Substrate>> {
    founding_of(seed, bang).map(|f| f.substrate.clone())
}

/// Galactic core `index`'s recipe at `t` Gyr: its composition on the
/// substrate (founding recipe plus intake from the core by t) relative to the
/// elements' founding. The home galaxy with no intake is the calibration.
pub fn weights_at(
    seed: u64,
    bang: &BigBang,
    index: u32,
    t: &Q,
    elements: &[Element],
) -> Option<Weights> {
    let f = founding_of(seed, bang)?;
    let t = Dy::from_q(t);
    if index == 0 && f.substrate.intake(0, t).is_empty() {
        return Some(Weights::home());
    }
    let mix: Vec<(u32, Q)> = f
        .substrate
        .composition(index, t, elements)?
        .into_iter()
        .collect();
    Some(Weights::of_mix(&mix, elements))
}

/// The founding: every galaxy with its recipe as founded (t = 0).
pub fn founding(seed: u64, bang: &BigBang, elements: &[Element]) -> Vec<GalacticCore> {
    let Some(f) = founding_of(seed, bang) else {
        return Vec::new();
    };
    f.shapes
        .iter()
        .filter_map(|shape| {
            Some(GalacticCore {
                weights: weights_at(seed, bang, shape.index, &Q::ZERO, elements)?,
                shape: shape.clone(),
            })
        })
        .collect()
}
