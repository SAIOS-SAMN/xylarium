//! The ladder of rungs to a mind, and the climb up it in closed form.

use super::*;

/// One rung of a ladder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    /// Information this rung holds (bits): b / eps^(s+1), capped at I_MIND.
    pub bits: Dy,
    /// Mean wait to climb this rung from the one below (Gyr).
    pub mean_gyr: Dy,
    /// Per-monomer copying error left after this rung's proofreading, in \[0, 1).
    pub eps: Dy,
    /// Bodies the medium's power feeds at this rung (count, not rounded).
    pub population: Dy,
    /// Length of one generation at this rung (s).
    pub generation_s: Dy,
    /// Mass of the body this rung builds (kg).
    pub body_kg: Dy,
}

/// The rungs a medium's chemistry must climb to a mind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ladder {
    /// Per-monomer miscopy probability of the bare chemistry, eps0, in \[0, 1/2).
    pub eps0: Dy,
    /// The rungs in order of climbing, at most [`MAX_RUNGS`].
    pub steps: Vec<Step>,
    /// Whether the last rung holds a mind's blueprint.
    pub complete: bool,
    /// Monomers a copier adds per second at the medium's temperature and richness.
    pub k_copy: Dy,
    /// Per-monomer breakage of stored information per second (see [`decay_rate`]).
    pub decay: Dy,
    /// Per-monomer breakage per second while dormant, copying stopped:
    /// thermal bond-breaking alone; a plasma's pattern, which needs its live
    /// field, does not keep (1 a second).
    pub dormant: Dy,
}

/// A medium's ladder to a mind, on a world of `area_m2`: each rung's
/// information and mean time (Gyr). `None` where nothing copies faithfully
/// enough to begin.
pub fn ladder(m: &Medium, area_m2: Dy, ctx: &Ctx) -> Option<Ladder> {
    let c = constants();
    let plasma = m.solvent == Solvent::Plasma;
    let eps0 = if plasma {
        field_fidelity(m.field_ev, m.t)
    } else {
        fidelity(m.solvent, m.t, ctx)
    };
    if eps0 >= Dy::ratio(1, 2) {
        // eps0 >= 1/2: a copy carries no information
        return None;
    }
    let kc = copy_rate(m.t, m.richness);
    if kc.signum() <= 0 {
        return None;
    }
    let b = c.bits_per_monomer;
    let strong = worlds::covalent_ev(&ctx.sc) * c.ev_j;
    let power = m.energy.max(Dy::ZERO) * c.flux_w * area_m2;
    // plasma decay: reconnection at k_copy exp(-E_B / kT)
    let r_decay = if plasma {
        kc * (-(m.field_ev.max(Dy::ZERO) / (worlds::kb_ev() * m.t.max(Dy::ONE)))).exp()
    } else {
        decay_rate(m.solvent, m.t, kc, ctx)
    };
    let bits_cell = b / eps0;
    let t_cell = bits_cell / (b * kc);
    // monomers are built from the solvent's own molecules (backbone_amu)
    let bond_kg = backbone_amu(m.solvent) * c.amu_kg * ctx.sc.proton;
    let kg_cell = bits_cell / b * c.body_per_monomer * bond_kg;
    let g = Gen {
        bits_cell,
        t_cell,
        kg_cell,
    };
    let mut lad = Ladder {
        eps0,
        steps: Vec::new(),
        complete: false,
        k_copy: kc,
        decay: r_decay,
        dormant: dormant_rate(m, ctx),
    };
    let mut prev = Dy::ZERO;
    let gen_of = |target: Dy| -> Dy {
        if target <= bits_cell {
            target / (b * kc)
        } else {
            generation_of(target, &g)
        }
    };
    for s in 0..MAX_RUNGS {
        let power_s = i32::try_from(s + 1).unwrap_or(1);
        let mut err = eps0;
        let mut gen_s = Dy::ZERO;
        // eps and the generation time depend on each other through the body
        // built: three fixed-point passes
        for _ in 0..3 {
            let cap = b / err.powi(power_s);
            gen_s = gen_of(cap.min(c.i_mind));
            err = eps0 + r_decay * gen_s;
        }
        if err >= Dy::ratio(1, 2) {
            // eps >= 1/2: no further rung holds information
            return Some(lad);
        }
        let cap = b / err.powi(power_s);
        if cap <= prev {
            return Some(lad);
        }
        let target = cap.min(c.i_mind);
        let monomers = target / b;
        let body = body_kg(target, bits_cell, kg_cell);
        // population: the power reaching the medium over the bond energy of
        // one body built per generation
        let pop = power * gen_s / (body / bond_kg * strong).max(Dy::sci(1, -300));
        let gain = (pop * err.powi(power_s) * monomers).min(Dy::ONE);
        if gain.signum() <= 0 {
            return Some(lad);
        }
        let mean = fill_s(prev, target, &g, b, kc, bits_per_generation(pop, b)) / gain / c.gyr_s;
        lad.steps.push(Step {
            bits: target,
            mean_gyr: mean,
            eps: err.powi(power_s),
            population: pop,
            generation_s: gen_s,
            body_kg: body,
        });
        prev = target;
        if cap >= c.i_mind {
            lad.complete = true;
            return Some(lad);
        }
    }
    Some(lad)
}

/// Per-monomer breakage per second while dormant: thermal homolysis
/// (kT/h) e^(-bond/kT) alone; 1 for a plasma, whose pattern needs its live
/// field.
fn dormant_rate(m: &Medium, ctx: &Ctx) -> Dy {
    if m.solvent == Solvent::Plasma {
        return Dy::ONE;
    }
    let c = constants();
    let t = m.t.max(Dy::ONE);
    c.kb_j * t / c.h_j
        * (-(worlds::strong_bond_ev(m.solvent, &ctx.sc) / (worlds::kb_ev() * t))).exp()
}

/// The mass of a monomer's building block in a solvent (amu): its own
/// molecule (a silicate unit, Si and two O, for a melt); a plasma copies
/// nothing.
pub(super) fn backbone_amu(s: Solvent) -> Dy {
    match s {
        Solvent::Volatile(sp) => sp.mu(),
        Solvent::Melt => {
            let table = crate::elements::table().unwrap_or_default();
            let a = |z: u32| {
                table
                    .iter()
                    .find(|e| e.z == z)
                    .map(|e| Dy::from_q(&e.a))
                    .unwrap_or(Dy::ZERO)
            };
            a(14) + Dy::int(2) * a(8)
        }
        // carriers: protons
        Solvent::Plasma => Dy::ONE,
    }
}

/// A lineage's climb: when each rung was reached, and the impacts that cost
/// it progress.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Climb {
    /// When each rung was reached (Gyr after the Big Bang); empty where life
    /// never began.
    pub times: Vec<Dy>,
    /// Number of impacts that cost the lineage progress (the expectation,
    /// whole).
    pub setbacks: u32,
}

/// A world's climb up its chemistry's rungs, in closed form: each rung
/// takes its mean wait; impacts at rate `mu` erase the rung's progress, so
/// a rung of mean `m` under them completes in the expected
/// `(e^(mu m) - 1) / mu` with `e^(mu m) - 1` setbacks (the memoryless
/// restart's expectation), the mean itself where there are no impacts.
/// Life itself (step 0) is never undone and may arrive from a neighbour
/// first (`arrives`). `steril`: when the surface is sterilised, if ever. No
/// draw enters: the climb is the chemistry's own expectation.
pub fn climb(
    means: &[Dy],
    t0: Dy,
    horizon: Dy,
    steril: Option<Dy>,
    mu: Dy,
    arrives: Option<Dy>,
) -> Climb {
    let past = |t: Dy| t > horizon || steril.is_some_and(|s| t > s);
    let tiny = Dy::sci(1, -12);
    let mut t = t0;
    let mut times = Vec::new();
    let mut setbacks = Dy::ZERO;
    for (s, &mean) in means.iter().enumerate() {
        let mean = mean.max(tiny);
        if s == 0 {
            let mut need = mean;
            if let Some(a) = arrives {
                // life may arrive earlier by transfer from a neighbour
                need = need.min((a - t0).max(Dy::ZERO));
            }
            t += need;
            if past(t) {
                return Climb {
                    times: Vec::new(),
                    setbacks: 0,
                };
            }
            times.push(t);
            continue;
        }
        if mu.signum() <= 0 {
            t += mean;
        } else {
            // expected completion under memoryless restarts; mu m is capped at
            // 60 so an unreachable rung lands past the horizon without overflow
            let x = (mu * mean).min(Dy::int(60));
            let grown = x.exp() - Dy::ONE;
            t += grown / mu;
            setbacks += grown;
        }
        if past(t) {
            break;
        }
        times.push(t);
    }
    Climb {
        times,
        setbacks: u32::try_from(setbacks.min(Dy::sci(4, 9)).trunc_i64().max(0)).unwrap_or(u32::MAX),
    }
}

/// How long rock thrown off one world takes to reach `target` (Gyr): the
/// target sweeps the ejecta once an orbit over the share of its orbit its
/// body covers, R / (2 pi a), so the wait is its year over that share.
pub fn transfer_gyr(target: &Body) -> Dy {
    let year = Dy::from_q(&crate::turning::year_yr(
        &target.star_mass.max(Dy::sci(1, -9)).to_q(),
        &target.a_au.max(Dy::sci(1, -9)).to_q(),
    ));
    let reach = target.radius * Dy::from_q(&crate::package::Si::world_radius_m())
        / (Dy::int(2)
            * Dy::PI
            * target.a_au.max(Dy::sci(1, -9))
            * Dy::from_q(&crate::package::Si::au()));
    year / reach.max(Dy::sci(1, -30)) / Dy::sci(1, 9)
}
