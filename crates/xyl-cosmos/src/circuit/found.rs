// The founding: the cascade, the vessel, the beads, the galaxies' recipes

use super::*;
use crate::formation;
use xyl_core::dyadic::Dy;
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
    /// Spin about the throat: the kind's reach at the founding, on the 2^-64 grid.
    pub spin: Q,
    /// Capture phase about the throat (turns in [0, 1), on the 60^-6 coordinate grid).
    pub phase: Q,
}

/// A bead of the founding, where the cascade's shells pinch: its reach about
/// the universal throat (core rings), the hold inside its ring, its state (its
/// share of the founding's mass), its phase about the throat, and its recipe
/// (the kinds carried into it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bead {
    /// The bead's position in the founding's bead order (from 0).
    pub index: u32,
    /// Reach about the universal throat (core rings): the reach of its most
    /// abundant kind.
    pub reach: Q,
    /// Hold inside the bead's ring: 1 plus twice the share of the pushed-out mass
    /// carried by the beads within it.
    pub inside: Q,
    /// State: the bead's mass from its recipe (universe units); the states sum to
    /// the founding unit.
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
    /// The beads where the cascade's shells pinch; those holding a sun are galaxies.
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
    /// The engine bell (Gyr): light's round trip to the home galaxy's rim and
    /// back, 2 rim_galaxy / c (anchor.rs); every clock on the substrate reads
    /// from it, so the substrate needs no anchor to be read.
    pub bell: Dy,
    /// The home galaxy's lighting (formation.rs, read on these beads one
    /// tier down): the rings until its gas first lights (None: no piece
    /// does), the lit share of its gas, and each piece of gas by reach
    /// (inner edge, outer edge, galaxy rings) with the rings until it lights.
    pub lighting: formation::Lighting,
}

impl Substrate {
    /// The founding cascade. Every kind is thrown by the equitable push to its
    /// own reach about the Wellspring, spread R (1 +/- s_final). The vessel's
    /// kinds within the throat (atoms::vessel, the band up to the last seal) are held as the
    /// firmament; held kinds outside the vessel leave with the expelled matter.
    /// The pushed-out kinds form shells; kinds with overlapping spreads link
    /// into runs, and each run's superposed hold beads at its maxima (one
    /// galaxy per bead) and parts at its minima. Each kind's matter goes to the
    /// bead whose span contains its reach.
    pub fn found(spin: &Q, elements: &[crate::elements::Element]) -> (Substrate, Anchor) {
        let mut prof = Prof::new(usize::MAX);
        // the share the jet carries, from the parent's spin by torsion; the
        // founding is derived from the laws and that share alone, so a
        // cached founding under the same laws is the same founding
        let share = crate::bang::torsion_share(spin);
        super::grain::set_founding_bits(crate::elements::founding_bits(&share));
        let key = format!("{}-{}", &env!("XYL_FOUNDING_DIGEST")[..12], share).replace('/', "-");
        let anchor_of = |s: &Substrate| {
            super::anchor::of(&s.rim, s.lighting.cools_in, spin, &share, &s.margin())
        };
        if let Some(mut s) = super::cache::load(&key) {
            s.bell = super::anchor::of(&s.rim, None, spin, &share, &Q::ZERO).bell_gyr;
            s.reverb = s.reverberation();
            let a = anchor_of(&s);
            prof.lap("founding: from the cache");
            return (s, a);
        }
        let kinds = formation::kinds(elements);
        let mix: Vec<(u32, Q)> = kinds.iter().map(|k| (k.z, k.share.clone())).collect();
        let (s_final, shells) = formation::founding_shells(&mix);
        prof.lap("founding: shells");
        let captured = |z: u32| &share * &Q::int(i64::from(z));
        let in_vessel = |z: u32| crate::atoms::vessel().is_some_and(|(lo, hi)| z >= lo && z <= hi);
        let shells: Vec<(u32, Q, Q, bool)> = shells
            .into_iter()
            .map(|(z, a, r, held)| (z, a, r, held && in_vessel(z)))
            .collect();
        // the firmament: the vessel's held kinds, each at its reach and its own
        // capture phase (the bounce's crossing turned Z times by the capture
        // share)
        let mut firmament: Vec<Kind> = shells
            .iter()
            .filter(|(_, a, _, held)| *held && a.signum() > 0)
            .map(|(z, a, r, _)| Kind {
                z: *z,
                amount: a.clone(),
                spin: grid(r),
                phase: turn(&captured(*z)),
            })
            .collect();
        // the firmament carries the universal state: the core holds 1 in its
        // own units (G = c = M_core = 1), so the throat's held kinds are the
        // allocation's heavy set, each in its founded proportion, scaled so
        // their mass is that 1. The capture chain alone leaves the held band
        // a vanishing share; what holds the throat open is allocated to it
        let held_mass = firmament
            .iter()
            .fold(Q::ZERO, |a, k| &a + &mass_of(k.z, &k.amount));
        if let Some(scale) = Q::ONE.checked_div(&held_mass) {
            for k in &mut firmament {
                k.amount = &k.amount * &scale;
            }
        }
        // the pushed-out shells, by reach; runs where spreads overlap
        let mut out: Vec<(u32, Q, Q)> = shells
            .iter()
            .filter(|(_, a, _, held)| !*held && a.signum() > 0)
            .map(|(z, a, r, _)| (*z, a.clone(), r.clone()))
            .collect();
        out.sort_by(|a, b| a.2.cmp(&b.2));
        let lo = |r: &Q| r * &(&Q::ONE - &s_final);
        let hi = |r: &Q| r * &(&Q::ONE + &s_final);
        let mut runs: Vec<Vec<usize>> = Vec::new();
        let mut reach_hi = Q::ZERO;
        for (k, (_, _, r)) in out.iter().enumerate() {
            if runs.is_empty() || lo(r) > reach_hi {
                runs.push(vec![k]);
                reach_hi = hi(r);
            } else {
                if let Some(run) = runs.last_mut() {
                    run.push(k);
                }
                reach_hi = reach_hi.clone().max(hi(r));
            }
        }
        // each run's superposed hold beads at its maxima; each bead's span in
        // reach runs from its run's inner edge or the pinch before it to the
        // pinch after it or its run's outer edge
        let mut beads: Vec<Vec<usize>> = Vec::new();
        let mut spans: Vec<(Q, Q, Vec<usize>)> = Vec::new();
        for run in runs {
            let run_lo = run.iter().map(|&k| lo(&out[k].2)).min().unwrap_or(Q::ZERO);
            let run_hi = run.iter().map(|&k| hi(&out[k].2)).max().unwrap_or(Q::ZERO);
            let first = beads.len();
            // each kind's density over its spread, a property of the kind alone
            let dens: Vec<(Q, Q, Q)> = run
                .iter()
                .map(|&k| {
                    let r = &out[k].2;
                    let (l, h) = (lo(r), hi(r));
                    let (dl, dh) = (deficit(&l, &Q::ONE), deficit(&h, &Q::ONE));
                    let t = (&dl - &dh)
                        .abs()
                        .checked_div(&(&dl + &dh))
                        .unwrap_or(Q::ONE);
                    (
                        l,
                        h,
                        mass_of(out[k].0, &out[k].1)
                            .checked_div(&t)
                            .unwrap_or(Q::ZERO),
                    )
                })
                .collect();
            // the shells covering a reach are one window of the run: its hold
            // there a difference of two prefix sums, exact
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
        prof.lap("founding: runs and beads");
        // each kind's matter goes to the bead whose span contains its reach
        // (no share formula spreads it)
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
            // the bead's state, by mass
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
            // inside its ring: the universal state plus twice each inner galaxy's share
            let inside = &Q::ONE + &(&Q::int(2) * &inner);
            inner = &inner + &state.checked_div(&pushed_out).unwrap_or(Q::ZERO);
            found.push(Bead {
                index: u32::try_from(index).unwrap_or(0),
                reach,
                inside,
                state,
                phase: turn(&captured(pz)),
                recipe,
                span: (a.clone(), b.clone()),
            });
        }
        prof.lap("founding: galaxies");
        // the rim: the outermost shell's outer edge, its reach spread by the
        // final share, so every bead's span lies within it (out to the
        // outermost shell and back is the bell)
        let rim = shells
            .iter()
            .map(|(_, _, r, _)| hi(r))
            .max()
            .unwrap_or(Q::ONE);
        // the bell from the rim alone (the home galaxy, the core's own
        // matter, is the same cascade read one tier down, so its rim is the
        // rim in the galaxy tier's unit); the reverberation reads it
        let bell = super::anchor::of(&rim, None, spin, &share, &Q::ZERO).bell_gyr;
        let mut s = Substrate {
            firmament,
            beads: found,
            rim: rim.clone(),
            nucleons_founded: Q::ZERO,
            reverb: Vec::new(),
            share: share.clone(),
            bell,
            lighting: formation::Lighting::default(),
        };
        s.reverb = s.reverberation();
        prof.lap("founding: reverberation");
        // the first stars: the home galaxy's gas is these beads one tier
        // down (formation.rs), heated by the pulse at each pinch swung by
        // the inbound echo's mean over the first echo
        let swing = &Q::ONE + &s.inbound_mean(s.echo()).to_q();
        s.lighting = formation::lighting(&s.beads, &s.unit(), &s.held_share(), &swing);
        prof.lap("founding: first collapse");
        s.nucleons_founded = s.holding();
        super::cache::store(&key, &s);
        let a = anchor_of(&s);
        (s, a)
    }

    /// The held share of the founded matter, by mass so the books close by
    /// mass: the firmament's mass over everything founded ([`Self::holding`]).
    /// The one definition; the Big Bang reads it too.
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

    /// The mean bead state: the home galaxy's equivalent. Its hole is the
    /// calibration; every bead's hole scales by its state over this.
    pub fn mean_state(&self) -> Q {
        self.unit()
            .checked_div(&Q::int(i64::try_from(self.beads.len().max(1)).unwrap_or(1)))
            .unwrap_or(Q::ONE)
    }
}
