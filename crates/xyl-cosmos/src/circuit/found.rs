// The founding: `Substrate::found` builds the substrate from the parent's
// spin and the founded elements.
//
// Inputs: the spin, its torsion share (`origin::torsion_share`), the elements
// (`elements::Element`), `formation::kinds` and `formation::founding_shells`
// for the cascade, `atoms::vessel`, and `formation::lighting`. Reaches are
// core rings; amounts are held frames, universe units.
//
// Steps: every kind is assigned its reach, spread r (1 +/- s_final). Held
// kinds inside the vessel band form the firmament (`Kind`), scaled so their
// mass is 1, the universal state; each kind's capture phase is the share
// turned Z times. The other shells link into runs where spreads overlap;
// each run's superposed hold is cut at its local minima into beads (`Bead`),
// and each kind goes whole to the bead whose span contains its reach. The
// rim is the outermost shell's outer edge.
//
// Outputs: `Substrate` with the anchor (anchor.rs); `held_share`, `census`,
// `unit` (sum of states), `holding` (everything founded, by mass) and
// `mean_state`. A founding cached under the same founding digest and share
// is loaded instead (cache.rs); the result is the same founding.
// Terms: firmament, bead, state, rim, census, held share, universal state
// (docs/glossary.md).

use super::*;
use crate::formation;
use xyl_core::Q;

/// A firmament kind: one of the vessel's kinds held by the throat, as
/// founded - its amount (held frames, universe units), its spin about the
/// throat, and its capture phase (turns).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kind {
    /// The kind: an element's Z, or a cluster's kind index (compound.rs).
    pub z: u32,
    /// Amount held by the throat (held frames, universe units).
    pub amount: Q,
    /// Spin about the throat: the kind's reach at the founding, on the 2^-64
    /// grid.
    pub spin: Q,
    /// Capture phase about the throat (turns in [0, 1), on the 60^-6
    /// coordinate grid).
    pub phase: Q,
}

/// A bead of the founding, where the cascade's shells pinch: its reach about
/// the universal throat (core rings), the hold inside its ring, its state (its
/// mass, universe units), its phase about the throat, and its recipe
/// (the kinds carried into it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bead {
    /// The bead's position in the founding's bead order (from 0).
    pub index: u32,
    /// Reach about the universal throat (core rings): the reach of its most
    /// abundant kind.
    pub reach: Q,
    /// Hold inside the bead's ring: 1 plus twice the share of the pushed-out
    /// mass carried by the beads within it.
    pub inside: Q,
    /// State: the bead's mass from its recipe (universe units); the states sum
    /// to the founding unit.
    pub state: Q,
    /// Phase about the throat (turns in [0, 1)), the capture phase of its most
    /// abundant kind.
    pub phase: Q,
    /// The kinds carried into the bead and their amounts (held frames).
    pub recipe: Mix,
    /// The bead's span about the throat (core rings): from its run's inner
    /// edge or the pinch before it to the pinch after it or its run's outer
    /// edge. Read one tier down, the beads are the home galaxy's rings.
    pub span: (Q, Q),
}

/// The substrate as founded: the firmament's kinds, the beads, the outermost
/// shell's reach (the rim the pulse crosses) and the nucleons founded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Substrate {
    /// The vessel's kinds held by the throat, as founded.
    pub firmament: Vec<Kind>,
    /// The beads where the cascade's shells pinch; those holding a star
    /// (`Substrate::galaxies`) are galaxies.
    pub beads: Vec<Bead>,
    /// Reach of the outermost shell (core rings): the rim the pulse crosses.
    pub rim: Q,
    /// Everything founded, by mass (universe units): the firmament plus every
    /// bead's recipe (see [`Substrate::holding`]).
    pub nucleons_founded: Q,
    /// Each firmament kind's reverberation (core.rs) at the echo's mean,
    /// computed once at the founding; its swing at t is read by
    /// [`Substrate::beyond_at`].
    pub reverb: Vec<Reverberation>,
    /// The share the jet carries through the throat each crossing (the
    /// torsion share): every kind's capture phase is this share turned Z times.
    pub share: Q,
    /// The home galaxy's lighting (formation.rs, read on these beads one
    /// tier down): the rings until its gas first lights (None: no piece
    /// does), the lit share of its gas, and each piece of gas by reach
    /// (inner edge, outer edge, galaxy rings) with the rings until it lights.
    pub lighting: formation::Lighting,
}

impl Substrate {
    /// The founding cascade. Every kind is carried by the equitable push to its
    /// reach about the throat, spread R (1 +/- s_final). The vessel's kinds
    /// within the throat (atoms::vessel, the band up to the last seal) are held
    /// as the firmament; held kinds outside the vessel leave with the expelled matter.
    /// The pushed-out kinds form shells; kinds with overlapping spreads link
    /// into runs, and each run's superposed hold beads at its maxima (one
    /// galaxy per bead) and parts at its minima. Each kind's matter goes to the
    /// bead whose span contains its reach.
    pub fn found(spin: &Q, elements: &[crate::elements::Element]) -> (Substrate, Anchor) {
        let mut prof = Prof::new(usize::MAX);
        // The founding depends only on the laws and the torsion share, so the
        // cache key is the founding digest and the share.
        let share = crate::bounce::torsion_share(&Q::ONE, spin);
        super::grain::set_founding_bits(crate::elements::founding_bits(&share));
        let key = format!("{}-{}", &env!("XYL_FOUNDING_DIGEST")[..12], share).replace('/', "-");
        let anchor_of = |s: &Substrate| {
            super::anchor::of(&s.rim, s.lighting.cools_in, spin, &share, &s.margin())
        };
        if let Some(mut s) = super::cache::load(&key) {
            s.reverb = s.reverberation();
            let a = anchor_of(&s);
            prof.lap("founding: from the cache");
            return (s, a);
        }
        let kinds = formation::kinds(elements);
        let mix: Vec<(u32, Q)> = kinds.iter().map(|k| (k.z, k.share.clone())).collect();
        let (s_final, shells) = formation::founding_shells(&mix);
        prof.lap("founding: shells");
        let in_vessel = |z: u32| crate::atoms::vessel().is_some_and(|(lo, hi)| z >= lo && z <= hi);
        let shells: Vec<(u32, Q, Q, bool)> = shells
            .into_iter()
            .map(|(z, a, r, held)| (z, a, r, held && in_vessel(z)))
            .collect();
        let firmament = firmament_of(&shells, &share);
        // The pushed-out shells are sorted by reach and linked into runs where
        // spreads overlap.
        let mut out: Vec<(u32, Q, Q)> = shells
            .iter()
            .filter(|(_, a, _, held)| !*held && a.signum() > 0)
            .map(|(z, a, r, _)| (*z, a.clone(), r.clone()))
            .collect();
        out.sort_by(|a, b| a.2.cmp(&b.2));
        let mut beads: Vec<Vec<usize>> = Vec::new();
        let mut spans: Vec<(Q, Q, Vec<usize>)> = Vec::new();
        for run in runs_of(&out, &s_final) {
            bead_run(&out, run, &s_final, &mut beads, &mut spans);
        }
        prof.lap("founding: runs and beads");
        let found = place_beads(&out, &beads, &spans, &share);
        prof.lap("founding: galaxies");
        // The rim is max r (1 + s_final) over all shells, so every bead's span
        // lies within it.
        let rim = shells
            .iter()
            .map(|(_, _, r, _)| outer_edge(r, &s_final))
            .max()
            .unwrap_or(Q::ONE);
        let mut s = Substrate {
            firmament,
            beads: found,
            rim: rim.clone(),
            nucleons_founded: Q::ZERO,
            reverb: Vec::new(),
            share: share.clone(),
            lighting: formation::Lighting::default(),
        };
        s.reverb = s.reverberation();
        prof.lap("founding: reverberation");
        // The home galaxy's gas is these beads read one tier down
        // (formation.rs); the swing is 1 + the inbound echo's mean over the
        // first echo.
        let swing = &Q::ONE + &s.inbound_mean(echo()).to_q();
        s.lighting = formation::lighting(&s.beads, &s.unit(), &s.held_share(), &swing);
        prof.lap("founding: first collapse");
        s.nucleons_founded = s.holding();
        super::cache::store(&key, &s);
        let a = anchor_of(&s);
        (s, a)
    }

    /// The held share of the founded matter, by mass so the books close by
    /// mass: the firmament's mass over everything founded ([`Self::holding`]).
    /// The one definition; the origin reads it too.
    pub fn held_share(&self) -> Q {
        let firm = self
            .firmament
            .iter()
            .fold(Q::ZERO, |a, k| &a + &mass_of(k.z, &k.amount));
        firm.checked_div(&self.holding()).unwrap_or(Q::ZERO)
    }

    /// The founding's census by kind: every kind's mass (the firmament's and
    /// every bead's recipe, the books [`Self::holding`] sums), heaviest share
    /// first. What the genesis was founded from, kind by kind.
    pub fn census(&self) -> Vec<(u32, Q)> {
        let mut by: std::collections::BTreeMap<u32, Q> = std::collections::BTreeMap::new();
        for k in &self.firmament {
            let m = mass_of(k.z, &k.amount);
            let e = by.entry(k.z).or_insert(Q::ZERO);
            *e = &*e + &m;
        }
        for b in &self.beads {
            for (z, a) in &b.recipe {
                let m = mass_of(*z, a);
                let e = by.entry(*z).or_insert(Q::ZERO);
                *e = &*e + &m;
            }
        }
        let mut out: Vec<(u32, Q)> = by.into_iter().collect();
        out.sort_by(|x, y| y.1.cmp(&x.1));
        out
    }

    /// The founding unit: the sum of the beads' states, fixed at the bounce.
    pub fn unit(&self) -> Q {
        self.beads.iter().fold(Q::ZERO, |a, b| &a + &b.state)
    }

    /// Everything founded, by mass (the founded side of the nucleon books): the
    /// firmament plus every bead's recipe.
    pub fn holding(&self) -> Q {
        let firm = self
            .firmament
            .iter()
            .fold(Q::ZERO, |a, k| &a + &mass_of(k.z, &k.amount));
        self.beads.iter().fold(firm, |a, b| &a + &weigh(&b.recipe))
    }

    /// The mean bead state: the founding unit over the bead count (at least
    /// one bead). Lumpiness (origin.rs), the ocean's pulls and the home galaxy's
    /// calibration recipe are read against it.
    pub fn mean_state(&self) -> Q {
        self.unit()
            .checked_div(&Q::int(i64::try_from(self.beads.len().max(1)).unwrap_or(1)))
            .unwrap_or(Q::ONE)
    }
}

/// The capture phase of kind `z`: the torsion `share` turned Z times, on
/// the coordinate grid.
fn captured_phase(share: &Q, z: u32) -> Q {
    turn(&(share * &Q::int(i64::from(z))))
}

/// The inner edge of a shell at reach `r`, spread by the final share:
/// r (1 - s_final).
fn inner_edge(r: &Q, s_final: &Q) -> Q {
    r * &(&Q::ONE - s_final)
}

/// The outer edge of a shell at reach `r`: r (1 + s_final).
fn outer_edge(r: &Q, s_final: &Q) -> Q {
    r * &(&Q::ONE + s_final)
}

/// The firmament: the held shells (inside the vessel), each at its reach on
/// the 2^-64 grid and its capture phase, scaled so their mass is 1.
fn firmament_of(shells: &[(u32, Q, Q, bool)], share: &Q) -> Vec<Kind> {
    let mut firmament: Vec<Kind> = shells
        .iter()
        .filter(|(_, a, _, held)| *held && a.signum() > 0)
        .map(|(z, a, r, _)| Kind {
            z: *z,
            amount: a.clone(),
            spin: grid(r),
            phase: captured_phase(share, *z),
        })
        .collect();
    // In units G = c = M_core = 1 the core's state is 1, so the held kinds
    // keep their founded proportions and are scaled to total mass 1. The
    // capture chain alone would leave the held band a vanishing share.
    let held_mass = firmament
        .iter()
        .fold(Q::ZERO, |a, k| &a + &mass_of(k.z, &k.amount));
    if let Some(scale) = Q::ONE.checked_div(&held_mass) {
        for k in &mut firmament {
            k.amount = &k.amount * &scale;
        }
    }
    firmament
}

/// The runs of the pushed-out shells `out` (sorted by reach): maximal
/// groups whose spreads overlap, as indices into `out`.
fn runs_of(out: &[(u32, Q, Q)], s_final: &Q) -> Vec<Vec<usize>> {
    let mut runs: Vec<Vec<usize>> = Vec::new();
    let mut reach_hi = Q::ZERO;
    for (k, (_, _, r)) in out.iter().enumerate() {
        if runs.is_empty() || inner_edge(r, s_final) > reach_hi {
            runs.push(vec![k]);
            reach_hi = outer_edge(r, s_final);
        } else {
            if let Some(run) = runs.last_mut() {
                run.push(k);
            }
            reach_hi = reach_hi.clone().max(outer_edge(r, s_final));
        }
    }
    runs
}

/// A shell's spread and density: (inner edge, outer edge, mass / T), T the
/// contrast of the deficits at its two edges, |d_l - d_h| / (d_l + d_h).
fn shell_density(z: u32, a: &Q, r: &Q, s_final: &Q) -> (Q, Q, Q) {
    let (l, h) = (inner_edge(r, s_final), outer_edge(r, s_final));
    let d = |r: &Q| xyl_core::shares::deficit(&Q::ONE, r).unwrap_or(Q::ZERO);
    let (dl, dh) = (d(&l), d(&h));
    let t = (&dl - &dh)
        .abs()
        .checked_div(&(&dl + &dh))
        .unwrap_or(Q::ONE);
    let density = mass_of(z, a).checked_div(&t).unwrap_or(Q::ZERO);
    (l, h, density)
}

/// Cuts one run into beads at the local minima of its superposed hold and
/// appends them to `beads`, with each bead's span (from the run's inner edge
/// or the previous pinch to the next pinch or the run's outer edge) and its
/// run to `spans`.
fn bead_run(
    out: &[(u32, Q, Q)],
    run: Vec<usize>,
    s_final: &Q,
    beads: &mut Vec<Vec<usize>>,
    spans: &mut Vec<(Q, Q, Vec<usize>)>,
) {
    let run_lo = run
        .iter()
        .map(|&k| inner_edge(&out[k].2, s_final))
        .min()
        .unwrap_or(Q::ZERO);
    let run_hi = run
        .iter()
        .map(|&k| outer_edge(&out[k].2, s_final))
        .max()
        .unwrap_or(Q::ZERO);
    let first = beads.len();
    let dens: Vec<(Q, Q, Q)> = run
        .iter()
        .map(|&k| shell_density(out[k].0, &out[k].1, &out[k].2, s_final))
        .collect();
    // The shells covering a reach form one contiguous window of the run, so
    // the hold there is a difference of two prefix sums, exact.
    let mut prefix: Vec<Q> = Vec::with_capacity(dens.len() + 1);
    prefix.push(Q::ZERO);
    for (_, _, d) in &dens {
        let next = prefix.last().map(|p| p + d).unwrap_or_else(|| d.clone());
        prefix.push(next);
    }
    let local: Vec<Q> = run
        .iter()
        .map(|&k| {
            let r = &out[k].2;
            let a = dens.partition_point(|(_, h, _)| h < r);
            let b = dens.partition_point(|(l, _, _)| l <= r);
            if b > a {
                &prefix[b] - &prefix[a]
            } else {
                Q::ZERO
            }
        })
        .collect();
    let mut start = 0;
    for m in 1..run.len().saturating_sub(1) {
        if local[m] < local[m - 1] && local[m] <= local[m + 1] {
            beads.push(run[start..m].to_vec());
            start = m;
        }
    }
    beads.push(run[start..].to_vec());
    let mut edge = run_lo;
    for b in first..beads.len() {
        let cut = if b + 1 < beads.len() {
            beads[b + 1]
                .first()
                .map(|&k| out[k].2.clone())
                .unwrap_or(run_hi.clone())
        } else {
            run_hi.clone()
        };
        spans.push((edge.clone(), cut.clone(), run.clone()));
        edge = cut;
    }
}

/// The founded beads: each kind's matter goes whole to the bead whose span
/// contains its reach (the last span of a run includes its outer edge); a
/// bead with no state is dropped. Its reach and phase are its most abundant
/// kind's; `inside` is 1 plus twice the pushed-out share of the beads within.
fn place_beads(
    out: &[(u32, Q, Q)],
    beads: &[Vec<usize>],
    spans: &[(Q, Q, Vec<usize>)],
    share: &Q,
) -> Vec<Bead> {
    let carried_to = |k: usize, a: &Q, b: &Q, last: bool| -> bool {
        let r = &out[k].2;
        *r >= *a && (*r < *b || (last && *r <= *b))
    };
    let pushed_out: Q = beads
        .iter()
        .flat_map(|b| b.iter())
        .fold(Q::ZERO, |a, &k| &a + &mass_of(out[k].0, &out[k].1));
    let mut inner = Q::ZERO;
    let mut found = Vec::new();
    for (index, bead) in beads.iter().enumerate() {
        let mut recipe: Mix = Mix::new();
        let (a, b, run) = &spans[index];
        let last = spans.get(index + 1).is_none_or(|(_, _, next)| next != run);
        for &k in run {
            if out[k].1.signum() > 0 && carried_to(k, a, b, last) {
                add(&mut recipe, out[k].0, &out[k].1);
            }
        }
        let state = weigh(&recipe);
        if state.signum() <= 0 {
            continue;
        }
        let peak = bead
            .iter()
            .copied()
            .max_by(|&x, &y| out[x].1.cmp(&out[y].1))
            .unwrap_or(bead[0]);
        let (pz, reach) = (out[peak].0, out[peak].2.clone());
        // The inside hold is 1 + 2 x (pushed-out share of the beads nearer the
        // throat).
        let inside = &Q::ONE + &(&Q::int(2) * &inner);
        inner = &inner + &state.checked_div(&pushed_out).unwrap_or(Q::ZERO);
        found.push(Bead {
            index: u32::try_from(index).unwrap_or(0),
            reach,
            inside,
            state,
            phase: captured_phase(share, pz),
            recipe,
            span: (a.clone(), b.clone()),
        });
    }
    found
}
