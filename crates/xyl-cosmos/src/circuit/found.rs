// The founding: `Substrate::found` builds the substrate from the parent's
// spin, the one torsion share read from it and the elements founded at it.
//
// Inputs: the spin, its torsion share (`bounce::torsion_share`, read once), the elements
// (`elements::Element`), `formation::kinds` and `formation::founding_shells`
// for the release, and `formation::lighting`. Reaches are in founding units
// (the core's hold 1, its horizon 2); amounts are in the held count.
//
// Steps: the core's bounces release its open kinds cohort by cohort
// (`formation::spin_orbit`, T17), each standing at its pinch where the
// reach rate carries it and swinging `r (1 +/- delta)` by the deficit of
// the hold inside it. What the core keeps on both faces (T16) forms the
// firmament (`Kind`), scaled so its mass is 1, the universal state; each
// kind's capture phase is the share turned Z times. The released shells
// link into runs where swings overlap; each run's superposed hold is cut at
// its local minima into beads (`Bead`), and each kind goes whole to the
// bead whose span contains its reach. The rim is the outermost swing's
// outer edge.
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
/// founded - its amount (held count, universe units), its spin about the
/// throat, and its capture phase (turns).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kind {
    /// The kind: an element's Z, or a cluster's kind index (compound.rs).
    pub z: u32,
    /// Amount held by the throat (held count, universe units).
    pub amount: Q,
    /// Spin about the throat: the kind's reach at the founding, exact (read
    /// from its face pairs).
    pub spin: Q,
    /// Capture phase about the throat (turns in [0, 1), exact; its
    /// denominator divides the torsion share's).
    pub phase: Q,
}

/// A bead of the founding, where the cascade's shells pinch: its reach about
/// the universal throat (cluster units), its state (its mass, core masses; the
/// hold inside its ring is [`Substrate::held_within`] at its reach), its
/// phase about the throat, and its recipe (the kinds carried into it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bead {
    /// The bead's position in the founding's bead order (from 0).
    pub index: u32,
    /// Reach about the universal throat (cluster units): the reach of its most
    /// abundant kind.
    pub reach: Q,
    /// State: the bead's mass from its recipe (universe units); the states sum
    /// to the founding unit.
    pub state: Q,
    /// Phase about the throat (turns in [0, 1)), the capture phase of its most
    /// abundant kind.
    pub phase: Q,
    /// The kinds carried into the bead and their amounts (held count).
    pub recipe: Mix,
    /// The bead's span about the throat (cluster units): from its run's inner
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
    /// Reach of the outermost shell (cluster units): the rim the pulse crosses.
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
    /// (inner edge, outer edge, galaxy units) with the rings until it lights.
    pub lighting: formation::Lighting,
    /// The universal table: each atom's neutrons per nucleus at the
    /// conversion's rest point (`formation::founding_shells`), built at the
    /// universal core's bounce.
    pub neutrons: super::weights::Table,
}

/// Marks the founding under way while held, so a reading of the neutrons
/// taken inside it reads the capture state instead of founding again.
struct Founding;

impl Founding {
    fn start() -> Self {
        super::weights::set_founding(true);
        Founding
    }
}

impl Drop for Founding {
    fn drop(&mut self) {
        super::weights::set_founding(false);
    }
}

impl Substrate {
    /// The founding: the core's bounces release its open kinds cohort by
    /// cohort (`formation::founding_shells`), each to the reach its pinch
    /// carries it to; what the core keeps on both faces is the firmament
    /// (T16, T17). Released kinds with overlapping swings link into runs,
    /// and each run's superposed hold beads at its maxima (one galaxy per
    /// bead) and parts at its minima. Each kind's matter goes to the bead
    /// whose span contains its reach. The torsion share is read here once
    /// from the parent's `spin` (`bounce::torsion_share` at the universal
    /// core's mass, 1) and held as [`Substrate::share`]; the elements are
    /// founded at it (`elements::founded`). `None` where the element table
    /// fails.
    pub fn found(spin: &Q) -> Option<(Substrate, Anchor)> {
        let mut prof = Prof::new(usize::MAX);
        let _founding = Founding::start();
        // The founding depends only on the laws and the torsion share, so the
        // cache key is the founding digest and the share.
        let share = crate::bounce::torsion_share(&Q::ONE, spin);
        super::grain::set_founding_bits(crate::elements::founding_bits(&share));
        let digest = crate::package::founding_digest();
        let key = format!("{}-{}", digest.get(..12).unwrap_or(digest), share).replace('/', "-");
        let anchor_of = |s: &Substrate| {
            let core = s.spin_orbit(spin)?.spin;
            Some(super::anchor::of(&s.rim, s.lighting.cools_in, spin, &core))
        };
        if let Some(mut s) = super::cache::load(&key) {
            super::weights::set_neutrons(s.neutrons.clone());
            s.reverb = s.reverberation();
            let a = anchor_of(&s)?;
            prof.lap("founding: from the cache");
            return Some((s, a));
        }
        // The shells are a separate stage, cached under the shells' digest
        // and the share: a change to a law above them reads them back.
        let shells_key = format!("{}-{}", crate::package::shells_digest(), share).replace('/', "-");
        let (shells, neutrons) = match super::cache::load_shells(&shells_key) {
            Some(cached) => cached,
            None => {
                let elements = crate::elements::founded(&share).ok()?;
                let kinds = formation::kinds(&elements);
                let mix: Vec<(u32, Q)> = kinds.iter().map(|k| (k.z, k.share.clone())).collect();
                let staged = formation::founding_shells(&mix, spin);
                super::cache::store_shells(&shells_key, &staged);
                staged
            }
        };
        prof.lap("founding: shells");
        // Capture, then conversion: the shells are read at the capture mass
        // (Q = 0), and every reading below weighs each nucleus with its
        // neutrons at the rest point.
        super::weights::set_neutrons(neutrons.clone());
        // What the core keeps on both faces of the paired throat (T16, T17)
        // is the firmament; every released kind stands at its pinch.
        let firmament = firmament_of(&shells, &share);
        // The released shells are sorted by reach and linked into runs where
        // their swings overlap.
        let mut out: Vec<&formation::Shell> = shells
            .iter()
            .filter(|k| !k.kept && k.amount.signum() > 0)
            .collect();
        out.sort_by(|a, b| a.reach.cmp(&b.reach));
        let mut beads: Vec<Vec<usize>> = Vec::new();
        let mut spans: Vec<(Q, Q, Vec<usize>)> = Vec::new();
        for run in runs_of(&out) {
            bead_run(&out, run, &mut beads, &mut spans);
        }
        prof.lap("founding: runs and beads");
        let found = place_beads(&out, &beads, &spans, &share);
        prof.lap("founding: galaxies");
        // The rim is the outermost swing's edge over all shells, so every
        // bead's span lies within it.
        let rim = shells.iter().map(outer_edge).max().unwrap_or(Q::ONE);
        let mut s = Substrate {
            firmament,
            beads: found,
            rim: rim.clone(),
            nucleons_founded: Q::ZERO,
            reverb: Vec::new(),
            share: share.clone(),
            lighting: formation::Lighting::default(),
            neutrons,
        };
        s.reverb = s.reverberation();
        prof.lap("founding: reverberation");
        // The home galaxy's gas is these beads read one tier down
        // (formation.rs), heated at the inbound echo's swing.
        s.lighting = formation::lighting(&s.beads, &s.unit(), &s.held_share(), &s.echo_swing());
        prof.lap("founding: first collapse");
        s.nucleons_founded = s.holding();
        super::cache::store(&key, &s);
        let a = anchor_of(&s)?;
        Some((s, a))
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

/// The capture phase of kind `z`: the torsion `share` turned Z times, exact
/// (its denominator divides the share's).
fn captured_phase(share: &Q, z: u32) -> Q {
    (share * &Q::int(i64::from(z))).frac()
}

/// The inner edge of a shell's swing: r (1 - swing).
fn inner_edge(k: &formation::Shell) -> Q {
    &k.reach * &(&Q::ONE - &k.swing)
}

/// The outer edge of a shell's swing: r (1 + swing).
fn outer_edge(k: &formation::Shell) -> Q {
    &k.reach * &(&Q::ONE + &k.swing)
}

/// The firmament: the kinds the core keeps, each at its reach and its
/// capture phase, scaled so their mass is 1.
fn firmament_of(shells: &[formation::Shell], share: &Q) -> Vec<Kind> {
    let mut firmament: Vec<Kind> = shells
        .iter()
        .filter(|k| k.kept && k.amount.signum() > 0)
        .map(|k| Kind {
            z: k.z,
            amount: k.amount.clone(),
            spin: k.reach.clone(),
            phase: captured_phase(share, k.z),
        })
        .collect();
    // In units G = c = M_core = 1 the core's state is 1, so the kept kinds
    // keep their founded proportions and are scaled to total mass 1.
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

/// The runs of the released shells `out` (sorted by reach): maximal
/// groups whose swings overlap, as indices into `out`.
fn runs_of(out: &[&formation::Shell]) -> Vec<Vec<usize>> {
    let mut runs: Vec<Vec<usize>> = Vec::new();
    let mut reach_hi = Q::ZERO;
    for (k, shell) in out.iter().enumerate() {
        if runs.is_empty() || inner_edge(shell) > reach_hi {
            runs.push(vec![k]);
            reach_hi = outer_edge(shell);
        } else {
            if let Some(run) = runs.last_mut() {
                run.push(k);
            }
            reach_hi = reach_hi.clone().max(outer_edge(shell));
        }
    }
    runs
}

/// A shell's swing and density: (inner edge, outer edge, mass / T), T the
/// contrast of the deficits at its two edges, |d_l - d_h| / (d_l + d_h).
fn shell_density(k: &formation::Shell) -> (Q, Q, Q) {
    let (l, h) = (inner_edge(k), outer_edge(k));
    let d = |r: &Q| xyl_core::shares::deficit(&Q::ONE, r).unwrap_or(Q::ZERO);
    let (dl, dh) = (d(&l), d(&h));
    let t = (&dl - &dh)
        .abs()
        .checked_div(&(&dl + &dh))
        .unwrap_or(Q::ONE);
    let density = mass_of(k.z, &k.amount).checked_div(&t).unwrap_or(Q::ZERO);
    (l, h, density)
}

/// Cuts one run into beads at the local minima of its superposed hold and
/// appends them to `beads`, with each bead's span (from the run's inner edge
/// or the previous pinch to the next pinch or the run's outer edge) and its
/// run to `spans`.
fn bead_run(
    out: &[&formation::Shell],
    run: Vec<usize>,
    beads: &mut Vec<Vec<usize>>,
    spans: &mut Vec<(Q, Q, Vec<usize>)>,
) {
    let run_lo = run
        .iter()
        .map(|&k| inner_edge(out[k]))
        .min()
        .unwrap_or(Q::ZERO);
    let run_hi = run
        .iter()
        .map(|&k| outer_edge(out[k]))
        .max()
        .unwrap_or(Q::ZERO);
    let first = beads.len();
    let dens: Vec<(Q, Q, Q)> = run.iter().map(|&k| shell_density(out[k])).collect();
    // The hold at a reach r is what has opened by r less what has closed
    // before it: the densities in order of inner edge and of outer edge,
    // each as a prefix sum, exact.
    let prefix_by = |edge: fn(&(Q, Q, Q)) -> &Q| {
        let mut by: Vec<&(Q, Q, Q)> = dens.iter().collect();
        by.sort_by(|x, y| edge(x).cmp(edge(y)));
        let mut sums = vec![Q::ZERO];
        for d in &by {
            let next = sums.last().map(|p| p + &d.2).unwrap_or_else(|| d.2.clone());
            sums.push(next);
        }
        let edges: Vec<Q> = by.iter().map(|d| edge(d).clone()).collect();
        (edges, sums)
    };
    let (opens, opened) = prefix_by(|d| &d.0);
    let (closes, closed) = prefix_by(|d| &d.1);
    let local: Vec<Q> = run
        .iter()
        .map(|&k| {
            let r = &out[k].reach;
            let a = opens.partition_point(|l| l <= r);
            let b = closes.partition_point(|h| h < r);
            &opened[a] - &closed[b]
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
                .map(|&k| out[k].reach.clone())
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
/// kind's.
fn place_beads(
    out: &[&formation::Shell],
    beads: &[Vec<usize>],
    spans: &[(Q, Q, Vec<usize>)],
    share: &Q,
) -> Vec<Bead> {
    let carried_to = |k: usize, a: &Q, b: &Q, last: bool| -> bool {
        let r = &out[k].reach;
        *r >= *a && (*r < *b || (last && *r <= *b))
    };
    let mut found = Vec::new();
    for (index, bead) in beads.iter().enumerate() {
        let mut recipe: Mix = Mix::new();
        let (a, b, run) = &spans[index];
        let last = spans.get(index + 1).is_none_or(|(_, _, next)| next != run);
        for &k in run {
            if out[k].amount.signum() > 0 && carried_to(k, a, b, last) {
                add(&mut recipe, out[k].z, &out[k].amount);
            }
        }
        let state = weigh(&recipe);
        if state.signum() <= 0 {
            continue;
        }
        let peak = bead
            .iter()
            .copied()
            .max_by(|&x, &y| out[x].amount.cmp(&out[y].amount))
            .unwrap_or(bead[0]);
        let (pz, reach) = (out[peak].z, out[peak].reach.clone());
        found.push(Bead {
            index: u32::try_from(index).unwrap_or(0),
            reach,
            state,
            phase: captured_phase(share, pz),
            recipe,
            span: (a.clone(), b.clone()),
        });
    }
    found
}
