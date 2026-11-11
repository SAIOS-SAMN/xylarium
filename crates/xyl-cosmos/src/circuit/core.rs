// The substrate's sections at time t (rings, as `Dy`): the core's clock, the
// firmament's reverberation and outflow, the ocean's shares and waveform,
// and each galaxy's intake, composition and push.
//
// Inputs: a founded `Substrate` (found.rs), the anchor's bell (anchor.rs),
// `bounce::horizon_reach`, `compound::open` / `compound::hold` for a kind's
// states, `formation::emission`, `galaxy::first_star_mass` (the star mass
// on the chain), and `pole_flux` (poles.rs) with `electron_phase`
// (weights.rs). Reaches are cluster units,
// amounts in the held count, times in rings (one ring one bell), rates per ring.
//
// - Deficit: `xyl_core::shares::deficit_dy(h, r) = h / (r + h)`, the vacuum's
//   share at reach `r` about a hold `h`; `throat_push` is its value
//   at r = h, 1/2; the jet's dilation D(s) = (1 + s) / (1 - s) at the torsion
//   share s, 3 at s = 1/2 and 4 at s = 3/5.
// - Clock: `bell` is one ring, `echo` two; a kind's `onset_echoes` is
//   ln(hold / open) / ln 3 (None for a sealed kind), `onset` the same in
//   rings.
// - Core: `Reverberation` per firmament kind; `outflow` empties a kind as
//   m0 (1 - exp(-rate (t - onset))); `rate`, `state_out`, `books` (held,
//   left, taken, founded); `inbound`, `inbound_peak`; `spin_orbit`, the
//   core's spin over its bounces (T17); `loop_flux` composes filament,
//   vacuum and electron channels.
// - Ocean: `ocean_shares` and `landing` (pull times `coherence`, over the
//   sum, so each kind's shares sum to 1); the held medium inside a reach,
//   `held_within` as founded (1 plus the beads nearer, core masses) and
//   `medium` at t (less the core's share the jet carried past the reach);
//   `carry`, the share the medium carries, H / (r + H), with the ocean's
//   motion (outward s (+) (-(1 - s)) per echo, 360 H / (r + H) degrees a
//   traversal on the isoclinic planes); `amplitude` (carried at that
//   share), `tide`, `c_ocean`.
// - Galaxies: `galaxies` (beads whose lit holding `lit x state` is at least
//   one star's mass `m_p` and whose state is at most 1), `intake`,
//   `composition` (recipe plus intake), `push` (reach, recession, shift,
//   drift, swing).
//
// Every reading is deterministic and closed-form in t; no state is stepped.
// Terms: bell, echo, coherence, jet, jetstream, landing, inbound echo,
// admission, books, vacuum's share, throat push, waveform, ocean,
// reverberation, held medium, ocean's carry (docs/glossary.md).

use super::*;
use xyl_core::dyadic::Dy;
use xyl_core::shares::{deficit_dy, dilation_dy};
use xyl_core::Q;

/// The push at the throat: the vacuum's share read at the throat's reach,
/// `delta(H, H) = 1/2`. Not a constant: the one point of the deficit law
/// where reach and hold coincide.
pub fn throat_push() -> Dy {
    let h = Dy::from_q(&crate::bounce::horizon_reach(&Q::ONE));
    // Stand-in S114: The convergent face is read, the divergent face not resisting it.
    deficit_dy(h, h)
}

/// The echoes a pole point `t` takes to reach the throat (`t = 1`) under the
/// dilation `rho` per echo: `ln(1 / t) / ln rho`, in closed form
/// ([`throat_crossing`] with no added states). Zero where `t >= 1`; `None`
/// where `t <= 0` (nothing open dilates) or `rho <= 1` (no dilation).
pub fn echoes_to_throat(t: Dy, rho: Dy) -> Option<Dy> {
    if t.signum() <= 0 {
        return None;
    }
    throat_crossing(t, rho, Dy::ZERO)
}

/// The echoes a pole point `t0` takes to reach the throat under the echo map
/// `t -> a t + b`, the mean of a jetstream's loop ([`jet_map`]): iterated,
/// `t_n = t* + (t0 - t*) a^n` with the fixed point `t* = b / (1 - a)`, so the
/// throat `t_n = 1` falls after `n = ln((1 - t*) / (t0 - t*)) / ln a` echoes,
/// in closed form. Zero where `t0 >= 1`; `None` where `a <= 1` (the map does
/// not carry the state outward), where `t0 <= 0` (a holder with nothing open
/// has no pole state, as a sealed kind has none) or `t0 - t*` is not
/// positive.
pub fn throat_crossing(t0: Dy, a: Dy, b: Dy) -> Option<Dy> {
    if t0.signum() <= 0 {
        return None;
    }
    if t0 >= Dy::ONE {
        return Some(Dy::ZERO);
    }
    if a <= Dy::ONE {
        return None;
    }
    let fixed = b.div(&(Dy::ONE - a)).unwrap_or_default();
    let (far, near) = (Dy::ONE - fixed, t0 - fixed);
    if near.signum() <= 0 || far.signum() <= 0 {
        return None;
    }
    Some(
        ((far.div(&near).unwrap_or_default())
            .ln()
            .unwrap_or_else(|| xyl_core::dyadic::Dy::int(-200) * xyl_core::dyadic::ln2())
            .div(
                &a.ln()
                    .unwrap_or_else(|| xyl_core::dyadic::Dy::int(-200) * xyl_core::dyadic::ln2()),
            )
            .unwrap_or_default())
        .max(Dy::ZERO),
    )
}

/// The jetstream's mean echo map for a holder that holds the push `p_c` on
/// the continuum and receives `p_v` from the vacuum: the continuum dilates the
/// pole point by the dilation per echo `rho = (1 + p_c) / (1 - p_c)`
/// (`xyl_core::shares::dilation_dy`, one law for every holder: the throat's
/// `1/2` for the firmament, a star's compactness for its collective), and the vacuum
/// moves `p_v` of the held count to the open ones, `t -> (t + p_v) / (1 -
/// p_v)`; the electrons' phase state has mean zero over an echo. Composed,
/// `t -> a t + b` with `a = rho / (1 - p_v)` and `b = p_v / (1 - p_v)`.
/// `None` for a push outside `[0, 1)`: a push is not negative.
pub fn jet_map(p_c: Dy, p_v: Dy) -> Option<(Dy, Dy)> {
    if p_c.signum() < 0 {
        return None;
    }
    // Stand-in S117: The vacuum adds to the continuum instead of resisting it.
    let rho = dilation_dy(p_c)?;
    if p_v.signum() < 0 || p_v >= Dy::ONE {
        return None;
    }
    let keep = Dy::ONE - p_v;
    Some((
        rho.div(&keep).unwrap_or_default(),
        p_v.div(&keep).unwrap_or_default(),
    ))
}

/// The jet's onset for kind `k` (rings) under the vacuum share `p_v` its band
/// receives, at the throat's torsion share `s`: the kind's state sits on the
/// pole axis at `t = U / W`, its open states over its closed (poles.rs), and
/// its jetstream carries it out through the continuum at the torsion share
/// and back through the vacuum at `p_v` ([`jet_map`]); it reaches the throat after
/// [`throat_crossing`] echoes. A sealed kind (no open states) has no pole
/// state and never does. The band's `beyond` share sets how much passes once
/// it has.
pub fn onset(k: u32, p_v: Dy, s: Dy) -> Option<Dy> {
    onset_echoes(k, p_v, s).map(|e| e * echo_at(&s.to_q()))
}

/// The jet's onset for kind `k` in echoes of its jetstream at vacuum share
/// `p_v` and torsion share `s` (see [`onset`]); None for a sealed kind, which
/// never reaches the throat.
pub fn onset_echoes(k: u32, p_v: Dy, s: Dy) -> Option<Dy> {
    let (open, hold) = (crate::compound::open(k), crate::compound::hold(k));
    if open == 0 || hold == 0 {
        return None;
    }
    let t = (Dy::int(i64::from(open)))
        .div(&(Dy::int(i64::from(hold))))
        .unwrap_or_default();
    let (a, b) = jet_map(s, p_v)?;
    throat_crossing(t, a, b)
}

/// The coherence of a galaxy at phase `psi` with a kind carried at phase
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

/// A throat's echo (rings) at its torsion share `s`: the sheared return of
/// its near face (T4, `k / (1 - theta)` with the advance per traversal the
/// torsion share, axiom 7), `bell / (1 - s)`. At zero spin `s = 1/2` and the
/// echo is two bells; a spinning throat's echo is longer, and its far face
/// returns after `bell / s`, the two never equal (the attractor pair). Zero
/// where `s` is not below one.
pub fn echo_at(s: &Q) -> Dy {
    let open = &Q::ONE - s;
    match open.signum() {
        1 => bell().div(&(Dy::from_q(&open))).unwrap_or_default(),
        _ => Dy::ZERO,
    }
}

/// The universal core's echo (rings) after its open kinds have left: the
/// echo at the torsion share of its spin then (`Anchor::core_spin`, T17).
/// Every tier after the core's bounce reads it; the founding itself reads
/// the founded share's ([`Substrate::echo`]). Two bells before a founding.
pub fn echo() -> Dy {
    echo_at(&crate::bounce::torsion_share(&Q::ONE, &anchor().core_spin))
}

/// A firmament kind on the substrate: its reach about the throat, the hold
/// enclosed inside it, its deficit there, the share of its band carried past
/// the throat over one swing, and its rate of leaving (per ring).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reverberation {
    /// Reach about the throat (cluster units): the kind's spin over its hold.
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

/// The push on a galaxy: its reach as founded and at t (cluster units), its recession
/// (cluster units of reach per ring of time) and its light's shift through the
/// ocean, z = v / c_ocean (positive away: a redshift); its drift, the
/// vessel's net push (reach per ring), and its swing, the reach the core's
/// pulse moves it through each echo (cluster units, either way).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Push {
    /// The galaxy's engine index (see [`Substrate::galaxies`]).
    pub index: u32,
    /// Reach about the throat as founded (cluster units).
    pub reach0: Dy,
    /// Reach about the throat at t (cluster units).
    pub reach: Dy,
    /// Recession rate (cluster units of reach per ring), positive away from the
    /// core.
    pub recession: Dy,
    /// Shift of the galaxy's light through the ocean, z = v / c_ocean
    /// (positive: a redshift).
    pub shift: Dy,
    /// Drift: the vessel's net outflow push (cluster units of reach per ring).
    pub drift: Dy,
    /// Swing: the reach the core's pulse moves the galaxy through each echo
    /// (cluster units, either way).
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
    ((outer - h)
        .div(&(outer - inner).max(Dy::sci(1, -300)))
        .unwrap_or_default())
    .clamp(Dy::ZERO, Dy::ONE)
}

impl Substrate {
    /// The universal core's echo (rings) during its founding: the echo at
    /// the founded torsion share ([`echo_at`] of [`Substrate::share`]),
    /// before its open kinds leave and its spin turns up (T17).
    pub fn echo(&self) -> Dy {
        echo_at(&self.share)
    }

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
        let nu = (Dy::ONE).div(&bell()).unwrap_or_default();
        for i in order {
            let k = &self.firmament[i];
            let h = Dy::ONE + inside.div(&unit).unwrap_or_default();
            let r = (Dy::from_q(&k.spin)).div(&h).unwrap_or_default();
            let delta = deficit_dy(h, r);
            let beyond = beyond_share(r, delta, h);
            out[i] = Reverberation {
                reach: r,
                hold: h,
                deficit: delta,
                beyond,
                rate: nu * beyond,
                onset: onset(k.z, delta, Dy::from_q(&self.share)),
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
        self.echo() * self.amplitude(Dy::ZERO, t)
    }

    /// The inbound echo's mean over the first `t` rings: the ring's integral
    /// over whole echoes is zero, so the mean is the drift, the state out
    /// by t over t in echoes, plus the ring's integral to t (`rung`).
    pub fn inbound_mean(&self, t: Dy) -> Dy {
        if t.signum() <= 0 {
            return Dy::ZERO;
        }
        (self.echo() * (self.rung(t) + self.state_out(t)))
            .div(&t)
            .unwrap_or_default()
    }

    /// The inbound echo's swing over the first echo, `1 + A_in` with `A_in`
    /// the inbound echo's mean over the first echo ([`Self::inbound_mean`]):
    /// the swing the founding's gas and every piece's star cut are heated by.
    pub fn echo_swing(&self) -> Q {
        &Q::ONE + &self.inbound_mean(self.echo()).to_q()
    }

    /// The inbound echo's peak: the ring at its crest (one) plus the vessel's
    /// greatest drift, which stands at one of the kinds' onsets (each term
    /// decays from the kind's onset).
    pub fn inbound_peak(&self) -> Dy {
        let echo = self.echo();
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
        // Stand-in S116: The swung push is unbounded and unresisted.
        deficit_dy(h, r) * (Dy::ONE + self.inbound(t))
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
    /// order: the filament's dilation at the throat (`rho`, Z to Q),
    /// the vacuum's share at the kind's reach swung by the inbound echo (`p`
    /// of the hold, Z to Q; at a crest past the whole hold the state passes
    /// the far face and no coordinate stands: None), and the electrons'
    /// phase state at the pulse (`R` of the open states, Q to Z: the held
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
        // The filament opens states by the jet's dilation at the torsion share
        // s, D(s) = (1 + s) / (1 - s): (u + f) / (w - f) = rho u / w.
        let rho = dilation_dy(Dy::from_q(&self.share))?.to_q();
        let filament = (&(&(&rho - &Q::ONE) * &u) * &w).checked_div(&(&w + &(&rho * &u)))?;
        // Vacuum channel: f_v = p w, p the vacuum's share swung by 1 + A_in.
        let p = (deficit_dy(rv.hold, rv.reach) * swing).to_q();
        if p >= Q::ONE {
            return None;
        }
        let vacuum = &p * &w;
        // Electron channel: R(pulse) u, the open states' phase state.
        let echo = &super::weights::electron_phase(k.z, &self.pulse(t)) * &u;
        pole_flux(
            &w,
            &u,
            &w,
            &u,
            &[("filament", -filament), ("vacuum", -vacuum), ("echo", echo)],
        )
    }

    /// The universal core's spin as a sheared orbit over its bounces (T17).
    ///
    /// At the bounce every founded kind is at the core, of hold `M` and
    /// turning `J = a M^2`. A kind's states stand on both faces: its open
    /// share `q = U / (U + W)` on the near face, which advances `s` per
    /// traversal, its held share on the antipode, which advances `1 - s`.
    /// Its separation is their flow-law sum, the rapidity
    /// `rho = q ln D(s) - (1 - q) ln D(1 - s)`; the far face exchanges `q`
    /// and `1 - q`. A cohort of hold `m` takes the turning in proportion to
    /// its separation against the throat's present hold,
    /// `J (m / M) (rho_cohort / rho_hold)`, so releasing everything takes
    /// `J` whole (axiom 11), and the spin after it is `a' = J' / M'^2`.
    ///
    /// A closed kind is held on both faces (T16). An open kind of axis
    /// polarity `A` is dilated per echo by `D(s) D(n)^A` on the near face;
    /// where that is at most one the near face holds it and the far face
    /// releases it at once. Every other open kind climbs to the throat in
    /// closed form, `ln t += N* ln(D(s) D(n)^A)` per bounce interval
    /// `N* = 3 (4 + a) / (2a)`, and leaves in the cohort of the first bounce
    /// after it arrives (at the core the folds' meeting is a bounce through
    /// the far face, never a pinch). The bounces are events found in closed
    /// form; the
    /// interval moves with the spin. Logarithms and the separations are
    /// dyadic readings, the holds exact, and each cohort's share of the
    /// turning is gridded back to `Q`. A spinless core never bounces a cohort out. `None`
    /// where the founded elements cannot be read.
    pub fn spin_orbit(&self, spin: &Q) -> Option<SpinOrbit> {
        // (kind, hold, ln t, axis polarity) of each founded kind, its hold
        // its founded mass `share mass_weight / hold` (the founding's own
        // units, before the firmament is scaled to the universal state); ln t
        // and the polarity are read for open kinds only.
        let elements = crate::elements::founded(&self.share).ok()?;
        let mut kinds: Vec<(u32, Q, Dy, i64)> = crate::formation::kinds(&elements)
            .into_iter()
            .filter_map(|k| {
                let hold = Q::int(i64::from(crate::compound::hold(k.z)));
                let m = (&k.share * &crate::formation::mass_weight(k.z)).checked_div(&hold)?;
                (m.signum() > 0).then_some((k.z, m))
            })
            .filter_map(|(z, m)| {
                let (u, w) = (crate::compound::open(z), crate::compound::hold(z));
                if u == 0 {
                    return Some((z, m, Dy::ZERO, 0));
                }
                let t = (Dy::from_q(&Q::int(i64::from(u))))
                    .div(&(Dy::from_q(&Q::int(i64::from(w)))))
                    .unwrap_or_default();
                Some((z, m, t.ln()?, crate::atoms::axis_polarity(z)?))
            })
            .collect();
        let mut hold = kinds.iter().fold(Q::ZERO, |m, k| &m + &k.1);
        let founded = hold.clone();
        let mut a = spin.clone();
        let mut turning = &(&a * &hold) * &hold;
        let mut cohorts = Vec::new();
        let mut echoes = Dy::ZERO;
        while a.signum() > 0 && a <= Q::ONE {
            let s = crate::bounce::torsion_share(&Q::ONE, &a);
            let (Some(near), Some(anti)) = (
                Dy::from_q(&(&Q::int(3) + &a)).ln(),
                Dy::from_q(&(&Q::int(2) - &s).checked_div(&s).unwrap_or(Q::ZERO)).ln(),
            ) else {
                break;
            };
            let lnn = Dy::from_q(&(&Q::ONE + &(&a * &xyl_core::ratio!(1, 2))))
                .ln()
                .unwrap_or(Dy::ZERO);
            let separation = |z: u32, far: bool| {
                let (u, w) = (crate::compound::open(z), crate::compound::hold(z));
                let q = (Dy::from_q(&Q::int(i64::from(u))))
                    .div(&(Dy::from_q(&Q::int(i64::from(u + w)))))
                    .unwrap_or_default();
                let q = if far { Dy::ONE - q } else { q };
                q * near - (Dy::ONE - q) * anti
            };
            let spread = |members: &[(u32, Q, Dy, i64)], far: bool| {
                members
                    .iter()
                    .fold(Dy::ZERO, |r, k| r + Dy::from_q(&k.1) * separation(k.0, far))
            };
            let open = |k: &(u32, Q, Dy, i64)| crate::compound::open(k.0) > 0;
            // The near face holds by pressure what the far face releases.
            let pressed = |k: &(u32, Q, Dy, i64)| open(k) && near + lnn * Dy::int(k.3) <= Dy::ZERO;
            let (out, far, at) = if cohorts.is_empty() && kinds.iter().any(pressed) {
                let (out, stay): (Vec<_>, Vec<_>) = kinds.drain(..).partition(pressed);
                kinds = stay;
                (out, true, Dy::ZERO)
            } else {
                let interval = (Dy::from_q(&(&Q::int(3) * &(&Q::int(4) + &a))))
                    .div(&(Dy::from_q(&(&Q::int(2) * &a))))
                    .unwrap_or_default();
                // The bounce at which each open kind arrives: the least whole
                // k with k N* ln(D(s) D(n)^A) >= -ln t.
                let arrive = |k: &(u32, Q, Dy, i64)| {
                    let rate = near + lnn * Dy::int(k.3);
                    if !open(k) || rate.signum() <= 0 {
                        return None;
                    }
                    let need = ((-k.2).div(&(interval * rate)).unwrap_or_default()).to_q();
                    let whole = i64::try_from(-(-&need).floor_i128()?).ok()?;
                    Some(whole.max(1))
                };
                let Some(first) = kinds.iter().filter_map(arrive).min() else {
                    break;
                };
                echoes += interval * Dy::int(first);
                let (out, stay): (Vec<_>, Vec<_>) =
                    kinds.drain(..).partition(|k| arrive(k) == Some(first));
                kinds = stay
                    .into_iter()
                    .map(|(z, m, lt, ax)| {
                        let climbed = if m.signum() > 0 && crate::compound::open(z) > 0 {
                            lt + interval * Dy::int(first) * (near + lnn * Dy::int(ax))
                        } else {
                            lt
                        };
                        (z, m, climbed, ax)
                    })
                    .collect();
                (out, false, echoes)
            };
            let all: Vec<_> = out.iter().chain(kinds.iter()).cloned().collect();
            let m = out.iter().fold(Q::ZERO, |s, k| &s + &k.1);
            // The cohort's separation over the hold's, each mass-weighted.
            let per = |sum: Dy, of: &Q| sum.div(&(Dy::from_q(of))).unwrap_or_default();
            let share = (per(spread(&out, far), &m)
                .div(&per(spread(&all, false), &hold))
                .unwrap_or_default())
            .to_q();
            let taken = &(&turning * &m.checked_div(&hold).unwrap_or(Q::ZERO)) * &share;
            turning = &turning - &xyl_core::certified::down(&taken, 64);
            hold = &hold - &m;
            a = turning.checked_div(&(&hold * &hold)).unwrap_or(Q::ZERO);
            cohorts.push(Cohort {
                echoes: at,
                far,
                hold: m,
                share,
                spin: a.clone(),
            });
        }
        // T19: the turning held beyond the founded turning per unit hold.
        let remanence = &turning - &(&(spin * &founded) * &hold);
        Some(SpinOrbit {
            cohorts,
            spin: a,
            founded,
            held: hold,
            turning,
            remanence,
        })
    }

    /// The outflow from the core by `t` rings after the bounce, by kind (held
    /// states): each firmament kind has sent out m0 (1 - exp(-rate (t - onset)))
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
        let pulls: Vec<(u32, Dy, Dy)> = self
            .galaxies()
            .into_iter()
            .map(|(i, b)| match b {
                None => (i, Dy::ONE, Dy::ZERO),
                Some(b) => {
                    let (r, h) = (
                        Dy::from_q(&b.reach),
                        (Dy::from_q(&b.state))
                            .div(&mean.max(Dy::sci(1, -300)))
                            .unwrap_or_default(),
                    );
                    (i, deficit_dy(h, r), self.delay(r))
                }
            })
            .collect();
        let sum = pulls
            .iter()
            .fold(Dy::ZERO, |a, x| a + x.1)
            .max(Dy::sci(1, -300));
        pulls
            .into_iter()
            .map(|(i, d, delay)| (i, d.div(&sum).unwrap_or_default(), delay))
            .collect()
    }

    /// The phase a kind is carried at (turns): its capture phase about the
    /// throat, the bounce's share turned Z times (found.rs).
    pub fn kind_phase(&self, k: u32) -> Q {
        (&self.share * &Q::int(i64::from(k))).frac()
    }

    /// The galaxies' pulls read once: by engine index, each galaxy's pull
    /// (its deficit seen from the core), its phase about the throat and its
    /// delay (rings). The home galaxy pulls 1 at phase zero with no delay.
    fn pulls(&self) -> Vec<(u32, Dy, Q, Dy)> {
        let mean = Dy::from_q(&self.mean_state()).max(Dy::sci(1, -300));
        self.galaxies()
            .into_iter()
            .map(|(i, b)| match b {
                None => (i, Dy::ONE, Q::ZERO, Dy::ZERO),
                Some(b) => {
                    let (r, h) = (
                        Dy::from_q(&b.reach),
                        (Dy::from_q(&b.state)).div(&mean).unwrap_or_default(),
                    );
                    (i, deficit_dy(h, r), b.phase.clone(), self.delay(r))
                }
            })
            .collect()
    }

    /// The landing of kind `k` over the galaxies' `pulls`: each galaxy's
    /// share of what the jet carries of it, its pull times its coherence with
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
            return tuned
                .into_iter()
                .map(|(i, d)| (i, d.div(&sum).unwrap_or_default()))
                .collect();
        }
        let plain = pulls
            .iter()
            .fold(Dy::ZERO, |a, x| a + x.1)
            .max(Dy::sci(1, -300));
        pulls
            .iter()
            .map(|(i, d, _, _)| (*i, (*d).div(&plain).unwrap_or_default()))
            .collect()
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
    /// states): its landing share of each kind the jet threw one delay
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

    /// The pulse's speed along the filament through the ocean (cluster units of
    /// reach per ring): the universe's rim crossed in half a bell, light's
    /// speed times the ladder step between the galaxy and cluster tiers
    /// (3600 c).
    pub fn c_ocean(&self) -> Dy {
        (Dy::from_q(&self.rim))
            .div(&(bell().div(&(Dy::int(2))).unwrap_or_default()))
            .unwrap_or_default()
    }

    /// The core's pulse in the ocean, per ring, at `s` rings after reaching a
    /// place: the universal state 1, regenerated each echo and not drained,
    /// cos(2 pi s / echo) / echo. Its integral over an echo is zero (the
    /// echo's torque invariant), so it oscillates without drift.
    fn ringing(&self, s: Dy) -> Dy {
        let echo = self.echo();
        ((Dy::int(2) * Dy::pi() * s).div(&echo).unwrap_or_default())
            .cos()
            .div(&echo)
            .unwrap_or_default()
    }

    /// The core's pulse integrated from its arrival to `s` rings after it:
    /// sin(2 pi s / echo) / (2 pi).
    fn rung(&self, s: Dy) -> Dy {
        ((Dy::int(2) * Dy::pi() * s)
            .div(&self.echo())
            .unwrap_or_default())
        .sin()
        .div(&(Dy::int(2) * Dy::pi()))
        .unwrap_or_default()
    }

    /// The ocean's non-tidal waveform at reach `r` (cluster units) and `t`: the core's
    /// ringing plus the vessel's rate, both taken one pulse delay earlier,
    /// carried at the share the held medium inside the reach holds there,
    /// `H(r, t) / (r + H(r, t))` ([`Substrate::carry`]). At the throat
    /// (`r = 0`) the share is 1 whatever the medium.
    pub fn amplitude(&self, r: Dy, t: Dy) -> Dy {
        let delay = self.delay(r);
        if t <= delay {
            return Dy::ZERO;
        }
        let share = if r.signum() > 0 {
            deficit_dy(self.medium(r, t), r)
        } else {
            Dy::ONE
        };
        (self.ringing(t - delay) + self.rate(t - delay).0) * share
    }

    /// The pulse's delay to reach `r` (cluster units), in rings: half a bell scaled by
    /// `r / rim`, the front crossing the rim in half a bell.
    fn delay(&self, r: Dy) -> Dy {
        (bell().div(&(Dy::int(2))).unwrap_or_default() * r.max(Dy::ZERO))
            .div(&(Dy::from_q(&self.rim).max(Dy::sci(1, -30))))
            .unwrap_or_default()
    }

    /// The hold inside reach `r` (cluster units) as founded, in core masses:
    /// the core's own hold, 1, plus the state of every bead nearer the
    /// throat (each bead's mass, in the core's units). The deficit at a reach
    /// is read about the matter enclosed within it (axiom 2), so this is the
    /// hold a bead's ring, a galaxy's circulation and the ocean's carry are
    /// read about. Exact.
    pub fn held_within(&self, r: &Q) -> Q {
        // Stand-in S036: One of two interim models of the hold inside a reach.
        self.beads
            .iter()
            .filter(|b| b.reach < *r)
            .fold(Q::ONE, |a, b| &a + &b.state)
    }

    /// The ocean's held medium inside reach `r` (cluster units) at `t` (rings),
    /// in core masses: the hold as founded ([`Substrate::held_within`]) less
    /// the share of the core's hold the jet has carried past `r` by `t`. What
    /// leaves the core is carried at the ocean's speed, so the matter that
    /// left by `t - delay(r)` has crossed `r` unless its landing is a galaxy
    /// within `r` (`landing`): the share bound beyond is subtracted, the
    /// share bound within stays in the medium, in flight or taken. The books
    /// close: the medium inside the rim at any `t` is the founded hold less
    /// what the outermost galaxies have taken.
    pub fn medium(&self, r: Dy, t: Dy) -> Dy {
        let founded = Dy::from_q(&self.held_within(&r.max(Dy::ZERO).to_q()));
        let delay = self.delay(r);
        if t <= delay {
            return founded;
        }
        let firm = self
            .firmament
            .iter()
            .fold(Dy::ZERO, |a, k| a + Dy::from_q(&mass_of(k.z, &k.amount)));
        if firm.signum() <= 0 {
            return founded;
        }
        let pulls = self.pulls();
        let mut passed = Dy::ZERO;
        for (z, left) in self.outflow(t - delay) {
            if left.signum() <= 0 {
                continue;
            }
            let beyond = self
                .landing_over(&pulls, z)
                .into_iter()
                .zip(pulls.iter())
                .filter(|(_, p)| p.3 > delay)
                .fold(Dy::ZERO, |a, ((_, share), _)| a + share);
            passed += Dy::from_q(&mass_of(z, &left.to_q())) * beyond;
        }
        (founded - passed.div(&firm).unwrap_or_default()).max(Dy::ZERO)
    }

    /// The ocean's carry at reach `r` (cluster units) at `t` (rings): what the
    /// held medium inside `r` holds of the pulse there and where the ocean
    /// moves it. The share is the deficit about the medium,
    /// `delta = H(r, t) / (r + H(r, t))` ([`Substrate::medium`]). The motion
    /// has two parts, from the echo and the torsion. Outward: the echo's near
    /// face carries the open share `s` out and its far face returns the held
    /// share `1 - s`, opposed, so the ocean carries `s (+) (-(1 - s))` out per
    /// echo (the pair share), zero only for a spinless throat. Turning: on the
    /// six planes in the common phase (`bounce::spin_bivector`, T18, #183)
    /// each traversal at
    /// `r` advances `360 delta` degrees, returned after `1 / (1 - delta)`
    /// traversals (T4), read from the same medium.
    pub fn carry(&self, r: Dy, t: Dy) -> Carry {
        let hold = self.medium(r, t);
        let share = if r.signum() > 0 {
            deficit_dy(hold, r)
        } else {
            Dy::ONE
        };
        let open = &Q::ONE - &self.share;
        let outward = xyl_core::shares::flow(&self.share, &-&open).unwrap_or(Q::ZERO);
        let held = Dy::ONE - share;
        Carry {
            hold,
            share,
            outward,
            turning: Dy::int(360) * share,
            returns: if held.signum() > 0 {
                Some((Dy::ONE).div(&held).unwrap_or_default())
            } else {
                None
            },
        }
    }

    /// The force on every galaxy at `t`, in closed form: push over inertia on
    /// its spin about the throat. The vessel's outflow gives the drift and the
    /// core's ringing the swing; returns, per galaxy other than the home
    /// galaxy, reach as founded and now, recession, the light's shift, drift
    /// and swing.
    pub fn push(&self, t: Dy) -> Vec<Push> {
        // Stand-in S118: The recession is pushed with no hold resisting it.
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
                    Dy::from_q(&self.held_within(&b.reach)).max(Dy::sci(1, -300)),
                );
                let hold = (Dy::from_q(&b.state)).div(&mean).unwrap_or_default();
                // The gain is delta(r0, H) d / hold, with delta(r0, H) the share
                // the held medium inside the galaxy's reach carries there, as
                // founded (`held_within`), so the push stays closed in t.
                let gain = (deficit_dy(inside, r0) * d)
                    .div(&hold.max(Dy::sci(1, -300)))
                    .unwrap_or_default();
                let reached = t > delay;
                let sent = if reached { t - delay } else { Dy::ZERO };
                let (ring, rung) = if reached {
                    (self.ringing(sent), self.rung(sent))
                } else {
                    (Dy::ZERO, Dy::ZERO)
                };
                let spin = r0 * inside + gain * (self.state_out(sent) + rung);
                let drift = (gain * self.rate(sent).0).div(&inside).unwrap_or_default();
                let recession = drift + (gain * ring).div(&inside).unwrap_or_default();
                let swing = if reached {
                    gain.div(&(Dy::int(2) * Dy::pi()))
                        .unwrap_or_default()
                        .div(&inside)
                        .unwrap_or_default()
                } else {
                    Dy::ZERO
                };
                Some(Push {
                    index: i,
                    reach0: r0,
                    reach: spin.div(&inside).unwrap_or_default(),
                    recession,
                    shift: recession.div(&c).unwrap_or_default(),
                    drift,
                    swing,
                })
            })
            .collect()
    }

    /// The tidal waveform across a galaxy at `t`: the waveform's difference
    /// across its extent `x` (cluster units) about its reach, A(R - x) - A(R + x) (the
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
        (t.max(Dy::ZERO).div(&self.echo()).unwrap_or_default())
            .to_q()
            .checked_div(&self.t_u())
            .unwrap_or(Q::ZERO)
            .frac()
    }
}

/// The ocean's carry at a reach ([`Substrate::carry`]): the held medium,
/// the share it holds of the pulse, and the ocean's motion there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Carry {
    /// The held medium inside the reach (core masses).
    pub hold: Dy,
    /// The share the medium holds there, `hold / (reach + hold)`, in (0, 1].
    pub share: Dy,
    /// The share carried outward per echo, `s (+) (-(1 - s))`: the near
    /// face's open share out composed with the far face's held share back.
    pub outward: Q,
    /// Degrees advanced per traversal on each isoclinic plane, `360 share`.
    pub turning: Dy,
    /// Traversals until the turning returns, `1 / (1 - share)`; None where
    /// the medium holds the whole pulse (the throat).
    pub returns: Option<Dy>,
}

/// One cohort of the core's spin orbit (T17).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cohort {
    /// Echoes after the bounce at which it leaves.
    pub echoes: Dy,
    /// Whether it leaves by the far face.
    pub far: bool,
    /// Its hold, in the founding's mass units.
    pub hold: Q,
    /// Its share of the turning against its share of the hold: its
    /// separation over the throat's, `rho_cohort / rho_hold`.
    pub share: Q,
    /// The core's spin once it has left.
    pub spin: Q,
}

/// The core's spin orbit (T17): its cohorts and its spin once they have
/// left. The pair closes on both faces where the spin is at most one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpinOrbit {
    /// The cohorts, in the order they leave.
    pub cohorts: Vec<Cohort>,
    /// The spin once the last cohort has left.
    pub spin: Q,
    /// The founded hold `M_0`, in the founding's mass units.
    pub founded: Q,
    /// The hold once the last cohort has left.
    pub held: Q,
    /// The turning once the last cohort has left, `a M^2`.
    pub turning: Q,
    /// The remanence (T19): the turning held beyond the founded turning per
    /// unit hold, `J - (a_0 M_0) M`.
    pub remanence: Q,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_held_medium_closes_its_books_and_bounds_its_carry() {
        let origin = crate::genesis::origin();
        let sub = crate::cores::substrate_for(&origin.parent.spin).unwrap();
        // As founded the hold inside the throat is the core's alone, it never
        // falls outward, and past the rim it is the core plus every bead.
        assert_eq!(sub.held_within(&Q::ZERO), Q::ONE);
        let mut reaches: Vec<Q> = sub.beads.iter().map(|b| b.reach.clone()).collect();
        reaches.sort();
        let held: Vec<Q> = reaches.iter().map(|r| sub.held_within(r)).collect();
        assert!(held.windows(2).all(|w| w[0] <= w[1]));
        let past = &sub.rim + &Q::ONE;
        assert_eq!(sub.held_within(&past), &Q::ONE + &sub.unit());
        // At the bounce the medium is the founded hold; later it is never
        // more, never negative, and falls with t. Nothing passes the rim, so
        // the medium there keeps the whole founded hold (the books close).
        let rim = Dy::from_q(&sub.rim);
        let late = [Dy::ZERO, Dy::int(10), Dy::int(1000), Dy::int(100000)];
        for r in [
            rim.div(&(Dy::int(4))).unwrap_or_default(),
            rim.div(&(Dy::int(2))).unwrap_or_default(),
            rim,
        ] {
            let founded = Dy::from_q(&sub.held_within(&r.to_q()));
            assert_eq!(sub.medium(r, Dy::ZERO), founded);
            let m: Vec<Dy> = late.iter().map(|t| sub.medium(r, *t)).collect();
            assert!(m.iter().all(|x| *x <= founded && x.signum() >= 0));
            assert!(m.windows(2).all(|w| w[1] <= w[0]));
        }
        let beyond = Dy::from_q(&past);
        for t in late {
            assert_eq!(sub.medium(beyond, t), Dy::from_q(&sub.held_within(&past)));
        }
        // The carry is the whole pulse at the throat and a share in (0, 1)
        // beyond it, falling outward; its turning is 360 of that share, and
        // the outward carry has the sign of 2s - 1 and stays below light.
        let throat = sub.carry(Dy::ZERO, Dy::ZERO);
        assert_eq!(throat.share, Dy::ONE);
        assert!(throat.returns.is_none());
        let near = sub.carry(rim.div(&(Dy::int(4))).unwrap_or_default(), Dy::ZERO);
        let far = sub.carry(rim, Dy::ZERO);
        assert!(far.share < near.share && far.share.signum() > 0 && near.share < Dy::ONE);
        assert_eq!(far.turning, Dy::int(360) * far.share);
        let tilt = (&Q::int(2) * &sub.share - &Q::ONE).signum();
        assert_eq!(far.outward.signum(), tilt);
        assert!(far.outward.abs() < Q::ONE);
    }

    #[test]
    fn the_jet_lands_by_phase_and_leaves_by_the_filaments_count() {
        // Coherence is 1 in phase, 0 opposed and one half a quarter turn apart,
        // measured the shorter way round in either direction.
        assert_eq!(coherence(&Q::ZERO, &Q::ZERO), Dy::ONE);
        assert_eq!(coherence(&Q::ZERO, &xyl_core::ratio!(1, 2)), Dy::ZERO);
        assert_eq!(
            coherence(&Q::ZERO, &xyl_core::ratio!(1, 4)),
            xyl_core::dyadic!(1, 2)
        );
        assert_eq!(
            coherence(&xyl_core::ratio!(9, 10), &xyl_core::ratio!(1, 10)),
            xyl_core::dyadic!(3, 5)
        );
        assert_eq!(
            coherence(&xyl_core::ratio!(1, 10), &xyl_core::ratio!(9, 10)),
            xyl_core::dyadic!(3, 5)
        );
        // The throat's push is the vacuum's share where reach and hold
        // coincide, one half; the jet's dilation is three for a spinless
        // throat and four at the extremal spin's torsion share, three fifths.
        assert_eq!(throat_push(), xyl_core::dyadic!(1, 2));
        assert_eq!(dilation_dy(xyl_core::dyadic!(1, 2)), Some(Dy::int(3)));
        assert_eq!(
            xyl_core::shares::dilation(&xyl_core::ratio!(3, 5)),
            Some(Q::int(4))
        );
        assert!(
            (dilation_dy(xyl_core::dyadic!(3, 5)).unwrap() - Dy::int(4)).abs() < Dy::sci(1, -15)
        );
        // A band inside the throat stays, one outside leaves, and between them
        // the share is linear.
        assert_eq!(
            beyond_share(xyl_core::dyadic!(1, 4), xyl_core::dyadic!(1, 2), Dy::ONE),
            Dy::ZERO
        );
        assert_eq!(
            beyond_share(Dy::int(4), xyl_core::dyadic!(1, 2), Dy::ONE),
            Dy::ONE
        );
        assert_eq!(
            beyond_share(Dy::ONE, xyl_core::dyadic!(1, 2), Dy::ONE),
            xyl_core::dyadic!(1, 2)
        );
        // A sealed kind (helium) never reaches the throat; hydrogen, one open
        // state over two closed, reaches it after ln 2 / ln D(s) echoes on the
        // continuum alone; the smaller a kind's open share, the later its
        // onset.
        let s = xyl_core::dyadic!(1, 2);
        assert!(onset_echoes(2, Dy::ZERO, s).is_none());
        let h = onset_echoes(1, Dy::ZERO, s).unwrap();
        let expect = (Dy::int(2)
            .ln()
            .unwrap_or_else(|| xyl_core::dyadic::Dy::int(-200) * xyl_core::dyadic::ln2()))
        .div(
            &(Dy::int(3)
                .ln()
                .unwrap_or_else(|| xyl_core::dyadic::Dy::int(-200) * xyl_core::dyadic::ln2())),
        )
        .unwrap_or_default();
        assert!((h - expect).abs() < Dy::sci(1, -9), "{h} vs {expect}");
        // The vacuum leg and a greater torsion share both bring it earlier.
        assert!(onset_echoes(1, xyl_core::dyadic!(1, 10), s).unwrap() < h);
        assert!(onset_echoes(1, Dy::ZERO, xyl_core::dyadic!(3, 5)).unwrap() < h);
        let (li, o) = (
            onset_echoes(3, Dy::ZERO, s).unwrap(),
            onset_echoes(8, Dy::ZERO, s).unwrap(),
        );
        assert!(li >= Dy::ZERO && o >= Dy::ZERO);
        let share = |k: u32| {
            (Dy::int(i64::from(crate::compound::open(k))))
                .div(&(Dy::int(i64::from(crate::compound::hold(k)))))
                .unwrap_or_default()
        };
        assert_eq!(li > o, share(3) < share(8));
    }

    #[test]
    fn a_kind_is_held_exactly_when_it_has_no_open_states() {
        // Every kind of the table, at the extremal and the spinless torsion
        // share, on the continuum alone (no vacuum leg, b = 0): a kind with
        // no open states has no onset; every other kind crosses the throat
        // within ln(W / U) / ln D(s) echoes, the bound of the jet map
        // t -> a t with a = D(s).
        for s in [xyl_core::dyadic!(1, 2), xyl_core::dyadic!(3, 5)] {
            let ln_d = dilation_dy(s)
                .unwrap()
                .ln()
                .unwrap_or_else(|| xyl_core::dyadic::Dy::int(-200) * xyl_core::dyadic::ln2());
            for z in 1..=crate::elements::TOP_Z {
                if crate::compound::is_cluster(z) {
                    continue;
                }
                let (u, w) = (crate::compound::open(z), crate::compound::hold(z));
                let onset = onset_echoes(z, Dy::ZERO, s);
                if u == 0 || w == 0 {
                    assert!(onset.is_none(), "Z{z} has no open states and no onset");
                    continue;
                }
                let n = onset.unwrap_or_else(|| panic!("Z{z} has open states and an onset"));
                let bound = (((Dy::int(i64::from(w)))
                    .div(&(Dy::int(i64::from(u))))
                    .unwrap_or_default())
                .ln()
                .unwrap_or_else(|| xyl_core::dyadic::Dy::int(-200) * xyl_core::dyadic::ln2())
                .max(Dy::ZERO))
                .div(&ln_d)
                .unwrap_or_default();
                assert!(n <= bound + Dy::sci(1, -9), "Z{z}: {n} within {bound}");
            }
        }
    }
}
