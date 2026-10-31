// The substrate's sections at time t (rings, as `Dy`): the core's clock, the
// firmament's reverberation and outflow, the ocean's shares and waveform,
// and each galaxy's intake, composition and push.
//
// Inputs: a founded `Substrate` (found.rs), the anchor's bell (anchor.rs),
// `bounce::horizon_reach`, `compound::open` / `compound::hold` for a kind's
// frames, `formation::emission`, `galaxy::first_star_mass` (the star mass
// on the chain), and `pole_flux` (poles.rs) with `electron_phase`
// (weights.rs). Reaches are core rings,
// amounts held frames, times in rings (one ring one bell), rates per ring.
//
// - Deficit: `vacuum_share(r, h) = h / (r + h)`; `throat_push` is its value
//   at r = h, 1/2; `echo_dilation` (1 + p) / (1 - p) = 3 there.
// - Clock: `bell` is one ring, `echo` two; a kind's `onset_echoes` is
//   ln(hold / open) / ln 3 (None for a sealed kind), `onset` the same in
//   rings.
// - Core: `Reverberation` per firmament kind; `outflow` empties a kind as
//   m0 (1 - exp(-rate (t - onset))); `rate`, `state_out`, `books` (held,
//   left, taken, founded); `inbound`, `inbound_peak`, `least_share`,
//   `margin`; `loop_flux` composes filament, vacuum and electron channels.
// - Ocean: `ocean_shares` and `landing` (pull times `coherence`, over the
//   sum, so each kind's shares sum to 1); `amplitude`, `tide`, `c_ocean`.
// - Galaxies: `galaxies` (beads whose lit holding `lit x state` is at least
//   one star's mass `m_p` and whose state is at most 1), `intake`,
//   `composition` (recipe plus intake), `push` (reach, recession, shift,
//   drift, swing).
//
// Every reading is deterministic and closed-form in t; no state is stepped.
// Terms: bell, echo, coherence, jet, jetstream, landing, inbound echo,
// admission, books, vacuum's share, throat push, waveform, ocean,
// reverberation (docs/glossary.md).

use super::*;
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// The vacuum's share at reach `r` about a hold `h`: `h / (r + h)`, the
/// deficit law. It is the push the pulse delivers there: what the hold
/// cannot keep at that reach the vacuum takes back and swings. Read at the
/// throat itself (r = h) it is one half, the bounce's push.
pub fn vacuum_share(r: Dy, h: Dy) -> Dy {
    let h = h.max(Dy::sci(1, -300));
    h / (r.max(Dy::ZERO) + h)
}

/// The push at the throat: the vacuum's share read at the throat's reach,
/// `delta(H, H) = 1/2`. Not a constant: the one point of the deficit law
/// where reach and hold coincide.
pub fn throat_push() -> Dy {
    let h = Dy::from_q(&crate::bounce::horizon_reach(&Q::ONE));
    vacuum_share(h, h)
}

/// The echo's dilation along the filament: a band swings between
/// r (1 - p) and r (1 + p) each echo, so the pole coordinate of what it
/// carries is dilated by (1 + p) / (1 - p) per echo (poles.rs), three at
/// the throat's push.
pub fn echo_dilation() -> Dy {
    let p = throat_push();
    (Dy::ONE + p) / (Dy::ONE - p)
}

/// The jet's onset for kind `k` (rings): a kind's state sits on the pole axis
/// at t = U / W, its open frames over its closed (poles.rs), and reaches
/// the throat (t = 1) after ln(1 / t) / ln rho echoes of the filament's
/// dilation; a sealed kind (no open frames) never does. The band's `beyond`
/// share sets how much passes once it has.
pub fn onset(k: u32) -> Option<Dy> {
    onset_echoes(k).map(|e| e * echo())
}

/// The jet's onset for kind `k` in echoes of the filament's dilation (see
/// [`onset`]); None for a sealed kind, which never reaches the throat.
pub fn onset_echoes(k: u32) -> Option<Dy> {
    let (open, hold) = (crate::compound::open(k), crate::compound::hold(k));
    if open == 0 || hold == 0 {
        return None;
    }
    let t = Dy::int(i64::from(open)) / Dy::int(i64::from(hold));
    if t >= Dy::ONE {
        return Some(Dy::ZERO);
    }
    Some(((Dy::ONE / t).log() / echo_dilation().log()).max(Dy::ZERO))
}

/// The coherence of a galaxy at phase `psi` with a kind thrown at phase
/// `phi` (turns): one less twice the shorter way round between them, a share
/// of the turn in the deficit grammar (1 in phase, 0 opposed).
pub fn coherence(psi: &Q, phi: &Q) -> Dy {
    let d = (psi - phi).frac();
    let d = if d.signum() < 0 { &d + &Q::ONE } else { d };
    let near = d.clone().min(&Q::ONE - &d);
    (Dy::ONE - Dy::int(2) * Dy::from_q(&near)).clamp(Dy::ZERO, Dy::ONE)
}

/// One engine bell (rings): the core's period, light's round trip to the
/// home galaxy's rim and back, 2 rim 60^5 in core time r_g / c (anchor.rs);
/// the pulse crosses the universe's rim on the filament within it. The bell
/// is the engine's unit of time, one ring.
pub fn bell() -> Dy {
    Dy::ONE
}

/// One echo (rings): the paired throat, the near face's ring and the far
/// face's - two full bell rings. Time is counted in full rings: one ring is
/// one bell, the pulse's whole round trip from the throat to the rim and
/// back; a one-way crossing (a delay to a reach) is half a ring of travel,
/// not a separate ring.
pub fn echo() -> Dy {
    bell() * Dy::int(2)
}

/// A firmament kind on the substrate: its reach about the throat, the hold
/// enclosed inside it, its deficit there, the share of its band carried past
/// the throat over one swing, and its rate of leaving (per ring).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reverberation {
    /// Reach about the throat (core rings): the kind's spin over its hold.
    pub reach: Dy,
    /// Hold enclosed inside the reach: 1 plus the firmament mass nearer the
    /// throat over the founding unit (at least 1).
    pub hold: Dy,
    /// Deficit at the reach, hold / (reach + hold), in (0, 1].
    pub deficit: Dy,
    /// Share of the band carried past the throat, averaged over one swing, in
    /// [0, 1].
    pub beyond: Dy,
    /// Rate of leaving (per ring): `beyond` over one engine bell.
    pub rate: Dy,
    /// The jet's onset (rings): when the kind's pole state reaches the throat
    /// along the filament (`onset`); None for a kind that never does.
    pub onset: Option<Dy>,
}

/// The push on a galaxy: its reach as founded and at t (rings), its recession
/// (core rings of reach per ring of time) and its light's shift through the
/// ocean, z = v / c_ocean (positive away: a redshift); its drift, the
/// vessel's net push (reach per ring), and its swing, the reach the core's
/// pulse moves it through each echo (core rings, either way).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Push {
    /// The galaxy's engine index (see [`Substrate::galaxies`]).
    pub index: u32,
    /// Reach about the throat as founded (core rings).
    pub reach0: Dy,
    /// Reach about the throat at t (core rings).
    pub reach: Dy,
    /// Recession rate (core rings of reach per ring), positive away from the
    /// core.
    pub recession: Dy,
    /// Shift of the galaxy's light through the ocean, z = v / c_ocean
    /// (positive: a redshift).
    pub shift: Dy,
    /// Drift: the vessel's net outflow push (core rings of reach per ring).
    pub drift: Dy,
    /// Swing: the reach the core's pulse moves the galaxy through each echo
    /// (core rings, either way).
    pub swing: Dy,
}

/// The share of a band swinging between `r (1 - p)` and `r (1 + p)` that lies
/// past the throat at `h`, averaged over the swing: 0 where the outer edge
/// stays inside, 1 where the inner edge is already past, and between them
/// `(r (1 + p) - h) / (2 p r)`.
fn beyond_share(r: Dy, p: Dy, h: Dy) -> Dy {
    let (inner, outer) = (r * (Dy::ONE - p), r * (Dy::ONE + p));
    if outer <= h {
        return Dy::ZERO;
    }
    if inner >= h {
        return Dy::ONE;
    }
    ((outer - h) / (outer - inner).max(Dy::sci(1, -300))).clamp(Dy::ZERO, Dy::ONE)
}

impl Substrate {
    /// The reverberation of every firmament kind, in founded order, at the
    /// echo's mean (the ring averages to zero over an echo and the vessel's
    /// drift is read at `t` by [`Substrate::beyond_at`]). For each kind:
    /// hold h = 1 + (firmament mass nearer) / (founding unit), reach
    /// r = spin / h, and the push at its reach the vacuum's share there,
    /// p = h / (r + h) (its deficit). The bell's torque swings the band
    /// between r (1 - p) and r (1 + p); s is the band's share past the throat
    /// averaged over the swing, and the kind leaves at s / bell (per ring).
    pub(crate) fn reverberation(&self) -> Vec<Reverberation> {
        let unit = Dy::from_q(&self.unit()).max(Dy::sci(1, -300));
        let mut order: Vec<usize> = (0..self.firmament.len()).collect();
        order.sort_by(|&a, &b| self.firmament[a].spin.cmp(&self.firmament[b].spin));
        let mut out = vec![
            Reverberation {
                reach: Dy::ZERO,
                hold: Dy::ONE,
                deficit: Dy::ONE,
                beyond: Dy::ZERO,
                rate: Dy::ZERO,
                onset: None,
            };
            self.firmament.len()
        ];
        let mut inside = Dy::ZERO;
        let nu = Dy::ONE / bell();
        for i in order {
            let k = &self.firmament[i];
            let h = Dy::ONE + inside / unit;
            let r = Dy::from_q(&k.spin) / h;
            let delta = vacuum_share(r, h);
            let beyond = beyond_share(r, delta, h);
            out[i] = Reverberation {
                reach: r,
                hold: h,
                deficit: delta,
                beyond,
                rate: nu * beyond,
                onset: onset(k.z),
            };
            inside += Dy::from_q(&mass_of(k.z, &k.amount));
        }
        out
    }

    /// The inbound echo at the throat at `t` rings, dimensionless: the ocean's
    /// waveform at reach 0 read per echo, `cos(2 pi t / echo) + echo q(t)`
    /// with q the core's whole-state rate ([`Substrate::rate`]): the core's
    /// ring (plus or minus one) and the vessel's drift. The perturbation every
    /// swing carries.
    pub fn inbound(&self, t: Dy) -> Dy {
        echo() * self.amplitude(Dy::ZERO, t)
    }

    /// The inbound echo's mean over the first `t` rings: the ring's integral
    /// over whole echoes is zero, so the mean is the drift, the state out
    /// by t over t in echoes, plus the ring's integral to t (`rung`).
    pub fn inbound_mean(&self, t: Dy) -> Dy {
        if t.signum() <= 0 {
            return Dy::ZERO;
        }
        echo() * (self.rung(t) + self.state_out(t)) / t
    }

    /// The inbound echo's peak: the ring at its crest (one) plus the vessel's
    /// greatest drift, which stands at one of the kinds' onsets (each term
    /// decays from the kind's onset).
    pub fn inbound_peak(&self) -> Dy {
        let echo = echo();
        let drift = self
            .reverb
            .iter()
            .filter_map(|rv| rv.onset)
            .map(|onset| self.rate(onset + Dy::sci(1, -30)).0)
            .fold(Dy::ZERO, Dy::max);
        Dy::ONE + echo * drift
    }

    /// The push at reach `r` about hold `h` at `t`: the vacuum's share there
    /// swung by the inbound echo, `h / (r + h) (1 + A_in(t))`.
    pub fn push_at(&self, r: Dy, h: Dy, t: Dy) -> Dy {
        vacuum_share(r, h) * (Dy::ONE + self.inbound(t))
    }

    /// Firmament kind `i`'s share past the throat at `t`: its band swung by
    /// the push at its reach at t (see [`Substrate::push_at`]).
    pub fn beyond_at(&self, i: usize, t: Dy) -> Dy {
        let Some(rv) = self.reverb.get(i) else {
            return Dy::ZERO;
        };
        beyond_share(rv.reach, self.push_at(rv.reach, rv.hold, t), rv.hold)
    }

    /// The jetstream of firmament kind `i` over the echo at `t`: the channels
    /// the laws compose on its pole state (`t = open / hold`, poles.rs), each
    /// a dilator on the state it finds, and whether their product returns
    /// the state to the kind's initial state, the 3Z hold conserved. The channels, in
    /// order: the filament's dilation at the throat (`rho`, frames Z to Q),
    /// the vacuum's share at the kind's reach swung by the inbound echo (`p`
    /// of the hold, Z to Q; at a crest past the whole hold the state passes
    /// the far face and no coordinate stands: None), and the electrons'
    /// phase state at the pulse (`R` of the open frames, Q to Z: the held
    /// face early in the echo, the open face late). The residual is the
    /// dilator left over, the torsion the echo leaves on the kind; the loop
    /// closes where it is one. None for a sealed kind, which has no pole
    /// state.
    pub fn loop_flux(&self, i: usize, t: Dy) -> Option<PoleFlux> {
        self.loop_flux_swung(i, t, Dy::ONE + self.inbound(t))
    }

    /// The jetstream of kind `i` at the pulse's phase of `t` under an
    /// explicit swing `1 + A_in` (one: the echo's mean), see
    /// [`Substrate::loop_flux`].
    pub fn loop_flux_swung(&self, i: usize, t: Dy, swing: Dy) -> Option<PoleFlux> {
        let k = self.firmament.get(i)?;
        let rv = self.reverb.get(i)?;
        let w = Q::int(i64::from(crate::compound::hold(k.z)));
        let u = Q::int(i64::from(crate::compound::open(k.z)));
        if w.signum() <= 0 || u.signum() <= 0 {
            return None;
        }
        // The filament opens frames by its dilation: (u + f) / (w - f) = rho u / w.
        let rho = echo_dilation().to_q();
        let filament = (&(&(&rho - &Q::ONE) * &u) * &w).checked_div(&(&w + &(&rho * &u)))?;
        // Vacuum channel: f_v = p w, p the vacuum's share swung by 1 + A_in.
        let p = (vacuum_share(rv.reach, rv.hold) * swing).to_q();
        if p >= Q::ONE {
            return None;
        }
        let vacuum = &p * &w;
        // Electron channel: R(pulse) u, the open frames' phase state.
        let echo = &super::weights::electron_phase(k.z, &self.pulse(t)) * &u;
        pole_flux(
            &w,
            &u,
            &w,
            &u,
            &[("filament", -filament), ("vacuum", -vacuum), ("echo", echo)],
        )
    }

    /// The firmament's furthest reach (core rings): the outermost held kind.
    pub fn far_reach(&self) -> Q {
        self.firmament
            .iter()
            .map(|k| k.spin.clone())
            .max()
            .unwrap_or(Q::ZERO)
    }

    /// The least torsion share that holds the firmament whole: the furthest
    /// held kind's reach over the rim, swung by the inbound echo at its peak,
    /// `r_far / rim (1 + A_in)`. A parent whose spin gives less founds no
    /// stable firmament; the drawn spin is floored at the spin giving this
    /// share (`cores::substrate_for`).
    pub fn least_share(&self) -> Q {
        let rim = Dy::from_q(&self.rim).max(Dy::sci(1, -300));
        (Dy::from_q(&self.far_reach()) / rim * self.inbound_peak()).to_q()
    }

    /// The firmament's margin: the torsion share less the least admissible
    /// (negative: the firmament is not whole at this spin).
    pub fn margin(&self) -> Q {
        &self.share - &self.least_share()
    }

    /// The outflow from the core by `t` rings after the bounce, by kind (held
    /// frames): each firmament kind has sent out m0 (1 - exp(-rate (t - onset)))
    /// once its state has reached the throat along the filament (its onset).
    pub fn outflow(&self, t: Dy) -> Vec<(u32, Dy)> {
        let t = t.max(Dy::ZERO);
        self.firmament
            .iter()
            .zip(&self.reverb)
            .map(|(k, rv)| {
                // There is no outflow before the onset, nor for a kind with no
                // onset or a zero rate (its band never passes the throat).
                let Some(onset) = rv.onset else {
                    return (k.z, Dy::ZERO);
                };
                if rv.rate.signum() <= 0 || t <= onset {
                    return (k.z, Dy::ZERO);
                }
                let left = Dy::ONE - (-(rv.rate * (t - onset))).exp();
                (k.z, Dy::from_q(&k.amount) * left)
            })
            .collect()
    }

    /// Whether bead `b` holds a star: its lit holding, `lit x state` (core
    /// masses: the bead's state is its core's mass in units of the universal
    /// core's, 1, and `lit` the share of a galaxy's rings that lights, as in
    /// `galaxy::stellar_mass`), is at least the mass of one star on the
    /// chain, `m_p = s c` of the piece that lights first
    /// (`galaxy::first_star_mass`: the share `s` of that piece's infall the
    /// star tier's cut holds, times the infall's holding `c`). Both sides are
    /// read on this substrate, so the founding does not read the cached
    /// anchor. False where no piece lights.
    pub fn holds_a_star(&self, b: &Bead) -> bool {
        crate::galaxy::first_star_mass(self)
            .is_some_and(|star| &self.lighting.lit * &b.state >= star)
    }

    /// The galaxies on the substrate, in engine order: the home galaxy (index
    /// 0, about the core) and every bead that holds a star
    /// ([`Self::holds_a_star`]) and whose core is no heavier than the
    /// universal core, `M >= m` with `M = 1` and `m` the bead's state (core
    /// masses): a heavier core would hold the universal core rather than
    /// circle it (indices 1..).
    pub fn galaxies(&self) -> Vec<(u32, Option<&Bead>)> {
        let mut out = vec![(0u32, None)];
        let held = |b: &&Bead| b.state <= Q::ONE && self.holds_a_star(b);
        for b in self.beads.iter().filter(held) {
            out.push((u32::try_from(out.len()).unwrap_or(u32::MAX), Some(b)));
        }
        out
    }

    /// The ocean's shares, by engine index: (share, delay in rings). A galaxy's
    /// pull is its deficit seen from the core, hold / (reach + hold), with hold
    /// its state over the mean (the home galaxy: reach 0, pull 1); the share is
    /// its pull over the sum. The delay is half a bell scaled by reach / rim.
    pub fn ocean_shares(&self) -> Vec<(u32, Dy, Dy)> {
        let mean = Dy::from_q(&self.mean_state());
        let rim = Dy::from_q(&self.rim).max(Dy::sci(1, -30));
        let pulls: Vec<(u32, Dy, Dy)> = self
            .galaxies()
            .into_iter()
            .map(|(i, b)| match b {
                None => (i, Dy::ONE, Dy::ZERO),
                Some(b) => {
                    let (r, h) = (
                        Dy::from_q(&b.reach),
                        Dy::from_q(&b.state) / mean.max(Dy::sci(1, -300)),
                    );
                    (i, h / (r + h), bell() / Dy::int(2) * r / rim)
                }
            })
            .collect();
        let sum = pulls
            .iter()
            .fold(Dy::ZERO, |a, x| a + x.1)
            .max(Dy::sci(1, -300));
        pulls
            .into_iter()
            .map(|(i, d, delay)| (i, d / sum, delay))
            .collect()
    }

    /// The phase a kind is thrown at (turns): its capture phase about the
    /// throat, the bounce's share turned Z times (found.rs).
    pub fn kind_phase(&self, k: u32) -> Q {
        turn(&(&self.share * &Q::int(i64::from(k))))
    }

    /// The galaxies' pulls read once: by engine index, each galaxy's pull
    /// (its deficit seen from the core), its phase about the throat and its
    /// delay (rings). The home galaxy pulls 1 at phase zero with no delay.
    fn pulls(&self) -> Vec<(u32, Dy, Q, Dy)> {
        let mean = Dy::from_q(&self.mean_state()).max(Dy::sci(1, -300));
        let rim = Dy::from_q(&self.rim).max(Dy::sci(1, -30));
        self.galaxies()
            .into_iter()
            .map(|(i, b)| match b {
                None => (i, Dy::ONE, Q::ZERO, Dy::ZERO),
                Some(b) => {
                    let (r, h) = (Dy::from_q(&b.reach), Dy::from_q(&b.state) / mean);
                    (
                        i,
                        h / (r + h),
                        b.phase.clone(),
                        bell() / Dy::int(2) * r / rim,
                    )
                }
            })
            .collect()
    }

    /// The landing of kind `k` over the galaxies' `pulls`: each galaxy's
    /// share of what the jet throws of it, its pull times its coherence with
    /// the kind's phase, over the sum (the books close kind by kind); where
    /// nothing is in phase, the pull alone shares it. By engine index.
    fn landing_over(&self, pulls: &[(u32, Dy, Q, Dy)], k: u32) -> Vec<(u32, Dy)> {
        let phi = self.kind_phase(k);
        let tuned: Vec<(u32, Dy)> = pulls
            .iter()
            .map(|(i, d, psi, _)| (*i, *d * coherence(psi, &phi)))
            .collect();
        let sum = tuned.iter().fold(Dy::ZERO, |a, x| a + x.1);
        if sum.signum() > 0 {
            return tuned.into_iter().map(|(i, d)| (i, d / sum)).collect();
        }
        let plain = pulls
            .iter()
            .fold(Dy::ZERO, |a, x| a + x.1)
            .max(Dy::sci(1, -300));
        pulls.iter().map(|(i, d, _, _)| (*i, *d / plain)).collect()
    }

    /// The landing of kind `k` (see `landing_over`), with each galaxy's delay.
    pub fn landing(&self, k: u32) -> Vec<(u32, Dy, Dy)> {
        let pulls = self.pulls();
        self.landing_over(&pulls, k)
            .into_iter()
            .zip(pulls.iter())
            .map(|((i, share), p)| (i, share, p.3))
            .collect()
    }

    /// What galaxy `index` has taken from the core by `t` (rings), by kind (held
    /// frames): its landing share of each kind the jet threw one delay
    /// earlier.
    pub fn intake(&self, index: u32, t: Dy) -> Vec<(u32, Dy)> {
        self.intakes(t)
            .into_iter()
            .find(|x| x.0 == index)
            .map(|x| x.1)
            .unwrap_or_default()
    }

    /// What every galaxy has taken by `t` (by engine index).
    pub fn intakes(&self, t: Dy) -> Vec<(u32, Vec<(u32, Dy)>)> {
        let pulls = self.pulls();
        let mut took: Vec<(u32, Vec<(u32, Dy)>)> =
            pulls.iter().map(|p| (p.0, Vec::new())).collect();
        // The outflow is evaluated once per distinct delay and reused.
        let mut at_delay: std::collections::BTreeMap<Dy, Vec<(u32, Dy)>> =
            std::collections::BTreeMap::new();
        for (slot, p) in took.iter_mut().zip(pulls.iter()) {
            if t <= p.3 {
                continue;
            }
            let out = at_delay
                .entry(p.3)
                .or_insert_with(|| self.outflow(t - p.3))
                .clone();
            for (z, a) in out {
                if a.signum() <= 0 {
                    continue;
                }
                let share = self
                    .landing_over(&pulls, z)
                    .into_iter()
                    .find(|x| x.0 == p.0)
                    .map(|x| x.1)
                    .unwrap_or(Dy::ZERO);
                let taken = a * share;
                if taken.signum() > 0 {
                    slot.1.push((z, taken));
                }
            }
        }
        took
    }

    /// A galaxy's composition at `t`: its founding recipe plus its intake by t.
    /// A bead's recipe is its kinds; the home galaxy's is the mean state times
    /// each metal's share of the founded elements' mass (the calibration).
    /// None for an index that is not a galaxy.
    pub fn composition(
        &self,
        index: u32,
        t: Dy,
        elements: &[crate::elements::Element],
    ) -> Option<Mix> {
        self.composition_with(index, &self.intake(index, t), elements)
    }

    /// As `composition`, with the intake already computed.
    pub(crate) fn composition_with(
        &self,
        index: u32,
        took: &[(u32, Dy)],
        elements: &[crate::elements::Element],
    ) -> Option<Mix> {
        let (_, bead) = self.galaxies().into_iter().find(|x| x.0 == index)?;
        let mut mix: Mix = match bead {
            Some(b) => b.recipe.clone(),
            None => {
                let all = elements
                    .iter()
                    .fold(Q::ZERO, |a, e| &a + &(&e.a * &e.abundance));
                let mean = self.mean_state();
                let mut m = Mix::new();
                for e in elements.iter().filter(|e| e.z >= 3) {
                    add(
                        &mut m,
                        e.z,
                        &(&mean * &(&e.a * &e.abundance))
                            .checked_div(&all)
                            .unwrap_or(Q::ZERO),
                    );
                }
                m
            }
        };
        for (z, a) in took {
            add(&mut mix, *z, &a.to_q());
        }
        Some(mix)
    }

    /// The books at `t`, by mass: (still held, left, taken by galaxies,
    /// founded). What has left but not been taken is in flight.
    pub fn books(&self, t: Dy) -> (Dy, Dy, Dy, Dy) {
        let founded = self
            .firmament
            .iter()
            .fold(Dy::ZERO, |a, k| a + Dy::from_q(&mass_of(k.z, &k.amount)));
        let left = self.outflow(t).iter().fold(Dy::ZERO, |a, (z, x)| {
            a + Dy::from_q(&mass_of(*z, &x.to_q()))
        });
        let taken = self.intakes(t).iter().fold(Dy::ZERO, |a, (_, took)| {
            a + took.iter().fold(Dy::ZERO, |b, (z, x)| {
                b + Dy::from_q(&mass_of(*z, &x.to_q()))
            })
        });
        (founded - left, left, taken, founded)
    }

    /// The rate leaving the core at `t` rings, per ring: the derivative of the
    /// outflow, the sum over kinds past their onset of
    /// rate m0 exp(-rate (t - onset)). Returns (whole state, with each kind's
    /// magnetism m (1 + e); mass).
    pub fn rate(&self, t: Dy) -> (Dy, Dy) {
        let t = t.max(Dy::ZERO);
        let (mut state, mut mass) = (Dy::ZERO, Dy::ZERO);
        for (k, rv) in self.firmament.iter().zip(&self.reverb) {
            let Some(onset) = rv.onset else {
                continue;
            };
            if rv.rate.signum() <= 0 || t <= onset {
                continue;
            }
            let now = rv.rate * (-(rv.rate * (t - onset))).exp();
            let m0 = Dy::from_q(&k.amount);
            state += m0 * (Dy::ONE + Dy::from_q(&crate::formation::emission(k.z))) * now;
            mass += Dy::from_q(&mass_of(k.z, &k.amount)) * now;
        }
        (state, mass)
    }

    /// What has left the core by `t`, in the firmament's whole state (the
    /// force's Q).
    pub fn state_out(&self, t: Dy) -> Dy {
        self.outflow(t).iter().fold(Dy::ZERO, |a, (z, x)| {
            a + *x * (Dy::ONE + Dy::from_q(&crate::formation::emission(*z)))
        })
    }

    /// The pulse's speed along the filament through the ocean (core rings of
    /// reach per ring): the universe's rim crossed in half a bell, light's
    /// speed times the ladder step between the galaxy and cluster tiers
    /// (3600 c).
    pub fn c_ocean(&self) -> Dy {
        Dy::from_q(&self.rim) / (bell() / Dy::int(2))
    }

    /// The core's pulse in the ocean, per ring, at `s` rings after reaching a
    /// place: the universal state 1, regenerated each echo and not drained,
    /// cos(2 pi s / echo) / echo. Its integral over an echo is zero (the
    /// echo's torque invariant), so it oscillates without drift.
    fn ringing(&self, s: Dy) -> Dy {
        let echo = echo();
        (Dy::int(2) * Dy::PI * s / echo).cos() / echo
    }

    /// The core's pulse integrated from its arrival to `s` rings after it:
    /// sin(2 pi s / echo) / (2 pi).
    fn rung(&self, s: Dy) -> Dy {
        (Dy::int(2) * Dy::PI * s / echo()).sin() / (Dy::int(2) * Dy::PI)
    }

    /// The ocean's non-tidal waveform at reach `r` (rings) and `t`: the core's
    /// ringing plus the vessel's rate, both taken one pulse delay earlier,
    /// falling as the vacuum's share about the universal state, 1 / (r + 1).
    pub fn amplitude(&self, r: Dy, t: Dy) -> Dy {
        let delay = bell() / Dy::int(2) * r / Dy::from_q(&self.rim).max(Dy::sci(1, -30));
        if t <= delay {
            return Dy::ZERO;
        }
        (self.ringing(t - delay) + self.rate(t - delay).0) * vacuum_share(r, Dy::ONE)
    }

    /// The force on every galaxy at `t`, in closed form: push over inertia on
    /// its spin about the throat. The vessel's outflow gives the drift and the
    /// core's ringing the swing; returns, per galaxy other than the home
    /// galaxy, reach as founded and now, recession, the light's shift, drift
    /// and swing.
    pub fn push(&self, t: Dy) -> Vec<Push> {
        let mean = Dy::from_q(&self.mean_state()).max(Dy::sci(1, -300));
        let shares = self.ocean_shares();
        let c = self.c_ocean().max(Dy::sci(1, -300));
        self.galaxies()
            .into_iter()
            .filter_map(|(i, b)| {
                let b = b?;
                let (_, d, delay) = *shares.iter().find(|x| x.0 == i)?;
                let (r0, inside) = (
                    Dy::from_q(&b.reach),
                    Dy::from_q(&b.inside).max(Dy::sci(1, -300)),
                );
                let hold = Dy::from_q(&b.state) / mean;
                // The gain is delta(r0, 1) d / hold, with delta(r0, 1) = 1 / (r0 + 1)
                // the vacuum's share at the galaxy's reach about the universal state.
                let gain = vacuum_share(r0, Dy::ONE) * d / hold.max(Dy::sci(1, -300));
                let reached = t > delay;
                let sent = if reached { t - delay } else { Dy::ZERO };
                let (ring, rung) = if reached {
                    (self.ringing(sent), self.rung(sent))
                } else {
                    (Dy::ZERO, Dy::ZERO)
                };
                let spin = r0 * inside + gain * (self.state_out(sent) + rung);
                let drift = gain * self.rate(sent).0 / inside;
                let recession = drift + gain * ring / inside;
                let swing = if reached {
                    gain / (Dy::int(2) * Dy::PI) / inside
                } else {
                    Dy::ZERO
                };
                Some(Push {
                    index: i,
                    reach0: r0,
                    reach: spin / inside,
                    recession,
                    shift: recession / c,
                    drift,
                    swing,
                })
            })
            .collect()
    }

    /// The tidal waveform across a galaxy at `t`: the waveform's difference
    /// across its extent `x` (rings) about its reach, A(R - x) - A(R + x) (the
    /// bodies' stretch law, one tier up).
    pub fn tide(&self, reach: Dy, x: Dy, t: Dy) -> Dy {
        self.amplitude((reach - x).max(Dy::ZERO), t) - self.amplitude(reach + x, t)
    }

    /// The universe's bell in echoes, floor(2 (rim + 2)) + 1: out to the
    /// outermost shell and back, including the throat's hold.
    pub fn t_u(&self) -> Q {
        let bubble = &Q::int(2) * &(&self.rim + &Q::int(2));
        Q::int(i64::try_from(bubble.floor_i128().unwrap_or(59) + 1).unwrap_or(60))
    }

    /// The bounce's pulse at `t` rings (turns): the echoes elapsed by t over the
    /// universe's bell; the fractional part is the atoms' phase.
    pub fn pulse(&self, t: Dy) -> Q {
        (t.max(Dy::ZERO) / echo())
            .to_q()
            .checked_div(&self.t_u())
            .unwrap_or(Q::ZERO)
            .frac()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_jet_lands_by_phase_and_leaves_by_the_filaments_count() {
        // Coherence is 1 in phase, 0 opposed and one half a quarter turn apart,
        // measured the shorter way round in either direction.
        assert_eq!(coherence(&Q::ZERO, &Q::ZERO), Dy::ONE);
        assert_eq!(coherence(&Q::ZERO, &Q::ratio(1, 2)), Dy::ZERO);
        assert_eq!(coherence(&Q::ZERO, &Q::ratio(1, 4)), Dy::ratio(1, 2));
        assert_eq!(
            coherence(&Q::ratio(9, 10), &Q::ratio(1, 10)),
            Dy::ratio(3, 5)
        );
        assert_eq!(
            coherence(&Q::ratio(1, 10), &Q::ratio(9, 10)),
            Dy::ratio(3, 5)
        );
        // The throat's push is the vacuum's share where reach and hold
        // coincide, one half; the dilation per echo there is three.
        assert_eq!(throat_push(), Dy::ratio(1, 2));
        assert_eq!(echo_dilation(), Dy::int(3));
        // A band inside the throat stays, one outside leaves, and between them
        // the share is linear.
        assert_eq!(
            beyond_share(Dy::ratio(1, 4), Dy::ratio(1, 2), Dy::ONE),
            Dy::ZERO
        );
        assert_eq!(beyond_share(Dy::int(4), Dy::ratio(1, 2), Dy::ONE), Dy::ONE);
        assert_eq!(
            beyond_share(Dy::ONE, Dy::ratio(1, 2), Dy::ONE),
            Dy::ratio(1, 2)
        );
        // A sealed kind (helium) never reaches the throat; hydrogen, one open
        // frame over two closed, reaches it after ln 2 / ln 3 echoes; the
        // smaller a kind's open share, the later its onset.
        assert!(onset_echoes(2).is_none());
        let h = onset_echoes(1).unwrap();
        let expect = Dy::int(2).log() / Dy::int(3).log();
        assert!((h - expect).abs() < Dy::sci(1, -9), "{h} vs {expect}");
        let (li, o) = (onset_echoes(3).unwrap(), onset_echoes(8).unwrap());
        assert!(li >= Dy::ZERO && o >= Dy::ZERO);
        let share = |k: u32| {
            Dy::int(i64::from(crate::compound::open(k)))
                / Dy::int(i64::from(crate::compound::hold(k)))
        };
        assert_eq!(li > o, share(3) < share(8));
    }
}
