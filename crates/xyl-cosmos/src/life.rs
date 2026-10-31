//! Life: how far a world's chemistry can climb toward a mind, written as
//! information copied against its decay, and when each rung is reached.
//!
//! Inputs: `worlds` (a world's `Setting`, `State` and `Medium`s, bond
//! energies, `Ctx` with `laws::Scales`), `galaxy::Star`, `systems::System`,
//! `stars::life` for the star's lifetime, `turning::year_yr` for orbital
//! periods, and SI constants from `package::Si`. Times are Gyr after the Big
//! Bang, all quantities `Dy`. No per-world parameter sets how common or fast
//! life is; it follows from these formulas under each universe's scales.
//!
//! - Fidelity ([`fidelity`], [`field_fidelity`]): a monomer is miscopied
//!   with `eps0 = exp(-d E / kT)`, `d = 1/2` of the solvent's weak bond
//!   (Boltzmann). A plasma copies patterns held in its field, with `E` the
//!   field energy per carrier. `eps0 >= 1/2` copies nothing.
//! - Ladder ([`ladder`]): rung `s` holds `b / eps^(s+1)` bits (Eigen's
//!   ceiling, raised a rung at a time by kinetic proofreading), with `eps`
//!   the bare error plus decay ([`decay_rate`]) over one generation; the
//!   ladder is complete where a rung reaches `I_MIND` ([`i_mind`]), at most
//!   [`MAX_RUNGS`]. Each [`Step`]'s mean wait follows from Haldane's cost of
//!   selection capped at one bit per generation ([`bits_per_generation`]),
//!   the copy rate `kT/h e^-barrier` ([`copy_rate`]), a generation of
//!   `t_cell (1 + log2(body / cell))`, and the population the medium's power
//!   feeds.
//! - Climb ([`climb`]): the expected completion of each rung under impacts
//!   at rate `mu` ([`strikes_per_gyr`]), `(e^(mu m) - 1) / mu`; the first
//!   rung is never undone. A surface is sterilised at `1 / hazard`
//!   ([`sterilise_per_gyr`], scaled by cosmic rays and plane exposure); media
//!   under ice or deep are shielded. A surface chemistry's window closes at
//!   0.9 of the star's lifetime; every window closes when the world merges.
//! - Transfer ([`system_life`]): life reaches a sibling world after
//!   [`transfer_gyr`] (its year over the share `R / (2 pi a)` of its orbit
//!   its body covers), divided by the dormant lineage's survival in transit.
//!   Transfer is modelled within a system only.
//! - Feeding ([`foragers`]): living area, growing season ([`growing_share`])
//!   and energy through one trophic link, over each body's metabolism;
//!   [`viable`] is the 500 of Franklin's 50/500 rule.
//!
//! Every function is deterministic: no random draw enters.
//!
//! Terms: rung (docs/glossary.md).

use crate::galaxy::Star;
use crate::systems::System;
use crate::worlds::{
    self, best, build, media, Ctx, Galaxy, Medium, Region, Setting, Solvent, State, Where,
};
use std::sync::OnceLock;
use xyl_core::dyadic::Dy;

fn n(x: i64) -> Dy {
    Dy::int(x)
}

fn r(a: i64, b: i64) -> Dy {
    Dy::ratio(a, b)
}

fn sci(m: i64, e: i32) -> Dy {
    Dy::sci(m, e)
}

/// Life's constants: physical (measured) and declared.
struct K {
    kb_j: Dy,
    h_j: Dy,
    ev_j: Dy,
    gyr_s: Dy,
    flux_w: Dy,
    bits_per_monomer: Dy,
    discrimination: Dy,
    i_mind: Dy,
    body_per_monomer: Dy,
    m_mind_kg: Dy,
    amu_kg: Dy,
    bits_per_generation: Dy,
    reproductive_excess: Dy,
    sterilise_at_r0: Dy,
}

fn k() -> &'static K {
    static C: OnceLock<K> = OnceLock::new();
    C.get_or_init(|| K {
        kb_j: Dy::from_q(&crate::package::Si::kb_j()),
        h_j: Dy::from_q(&crate::package::Si::h_j()),
        ev_j: Dy::from_q(&crate::package::Si::ev_j()),
        gyr_s: Dy::from_q(&crate::package::Si::gyr_s()),
        flux_w: n(340),
        // an alphabet of four: the fewest letters that pair two ways (logic,
        // not chemistry)
        bits_per_monomer: n(2),
        // a partner is told by one contact: half a molecule's cohesion (declared)
        discrimination: r(1, 2),
        // open rulers, to be derived from each world's own chemistry once
        // planets are stable: the information a mind's blueprint holds, the
        // body built per monomer of genome, and the mass of a mind-bearing body
        i_mind: sci(1, 9),
        body_per_monomer: sci(1, 3),
        m_mind_kg: n(10),
        amu_kg: Dy::from_q(&crate::package::Si::amu_kg()),
        bits_per_generation: Dy::ONE,
        reproductive_excess: r(1, 2),
        // open ruler: the chance a dying star's pulse sterilises an open
        // surface over 10 Gyr where the cosmic-ray crowd is 1
        sterilise_at_r0: r(1, 10),
    })
}

/// A mind's blueprint (bits): I_MIND, for the kernel's readings of a mind
/// (how legible its heredity is: xyl-kernel knowledge.rs).
pub fn i_mind() -> Dy {
    k().i_mind
}

/// Bits a monomer carries (the alphabet's).
pub fn bits_per_monomer() -> Dy {
    k().bits_per_monomer
}

/// A chemistry needing more rungs than this never reaches a mind within any
/// star's life.
pub const MAX_RUNGS: usize = 12;

/// Deep time the forecast looks ahead to (Gyr).
pub fn horizon_gyr() -> Dy {
    n(30)
}

/// The earliest age a world is read at (Gyr after its star's birth): one
/// degree of its own star's life (`worlds::star_degree_gyr`), at the star's
/// distance from the core `r_kpc`.
pub fn first_age(r_kpc: Dy, ctx: &Ctx) -> Dy {
    worlds::star_degree_gyr(r_kpc, ctx)
}

/// The ages a world's media are probed at (Gyr after its star's birth): one
/// degree of its star's life and each doubling after, while its star's birth
/// plus the age lies within the horizon.
pub fn probe_ages(home: &Home, ctx: &Ctx) -> Vec<Dy> {
    let mut out = Vec::new();
    let mut a = first_age(home.r_kpc, ctx);
    while home.born + a < horizon_gyr() && out.len() < 64 {
        out.push(a);
        a = a * n(2);
    }
    out
}

/// The settled age a world's chemistry is read at (Gyr).
pub fn settled_gyr() -> Dy {
    n(3)
}

/// Franklin's 50/500: the number that keeps a people's evolutionary potential.
pub fn viable() -> Dy {
    n(500)
}

/// Strikes a world's biosphere suffers per Gyr from the debris its own
/// system left: the debris' share of the system's solids (belts and comet
/// cloud over everything the disc gathered), swept once an orbit over the
/// share of its orbit the world's own body covers, R / (2 pi a).
pub fn strikes_per_gyr(debris: Dy, body: &Body) -> Dy {
    let year = Dy::from_q(&crate::turning::year_yr(
        &body.star_mass.max(sci(1, -9)).to_q(),
        &body.a_au.max(sci(1, -9)).to_q(),
    ));
    let orbits = sci(1, 9) / year.max(sci(1, -12));
    // the world's radius (world radii) and orbit (AU) in metres
    let reach = body.radius * Dy::from_q(&crate::package::Si::world_radius_m())
        / (n(2) * Dy::PI * body.a_au.max(sci(1, -9)) * Dy::from_q(&crate::package::Si::au()));
    debris.clamp(Dy::ZERO, Dy::ONE) * orbits * reach
}

/// How often a world's life is sterilised (per Gyr): a nearby supernova or
/// burst no repair outlasts, growing with the dying stars around (`crowd`:
/// cosmic rays x how much of its orbit it spends among them); none for a sea
/// under ice or in the deep.
pub fn sterilise_per_gyr(crowd: Dy, dark: bool) -> Dy {
    if dark {
        Dy::ZERO
    } else {
        k().sterilise_at_r0 / n(10) * crowd.max(Dy::ZERO)
    }
}

/// eps0: the chance a monomer is miscopied in this solvent at `t` (1 where
/// nothing binds).
pub fn fidelity(s: Solvent, t: Dy, ctx: &Ctx) -> Dy {
    let weak = worlds::weak_bond_ev(s, ctx);
    if weak.signum() <= 0 {
        return Dy::ONE;
    }
    (-(k().discrimination * weak / (worlds::kb_ev() * t.max(Dy::ONE)))).exp()
}

/// eps0 for a plasma: the chance a pattern held in its field's topology is
/// miscopied, exp(-d E_B / kT), E_B the field's energy per carrier
/// (`Medium::field_ev`) and d the same half-contact discrimination a bonded
/// copy uses. A plasma copies only where its field dominates its heat.
pub fn field_fidelity(field_ev: Dy, t: Dy) -> Dy {
    (-(k().discrimination * field_ev.max(Dy::ZERO) / (worlds::kb_ev() * t.max(Dy::ONE)))).exp()
}

/// Monomers a copier adds per second, scaled by how much of what it builds
/// with is at hand: kT/h e^-barrier by corresponding states.
pub fn copy_rate(t: Dy, richness: Dy) -> Dy {
    k().kb_j * t.max(Dy::ONE) / k().h_j
        * (-worlds::solvent_barrier()).exp()
        * richness.max(Dy::ZERO)
}

/// Per-monomer breakage of what a lineage has stored, per second: back along
/// the path that made it (uphill by what the bond holds beyond the solvent's
/// cohesion), and thermal homolysis, (kT/h) e^(-bond/kT).
pub fn decay_rate(s: Solvent, t: Dy, k_copy: Dy, ctx: &Ctx) -> Dy {
    let t = t.max(Dy::ONE);
    let kt = worlds::kb_ev() * t;
    let strong = worlds::strong_bond_ev(s, &ctx.sc);
    let held = (strong - worlds::weak_bond_ev(s, ctx)).max(Dy::ZERO);
    k_copy * (-(held / kt)).exp() + k().kb_j * t / k().h_j * (-(strong / kt)).exp()
}

/// The body a lineage holding `bits` builds: a cell at the first rung, a
/// mind-bearing body at I_MIND, linear in bits between.
pub fn body_kg(bits: Dy, bits_cell: Dy, kg_cell: Dy) -> Dy {
    let c = k();
    if bits <= bits_cell || c.i_mind <= bits_cell {
        return kg_cell;
    }
    let share = ((bits - bits_cell) / (c.i_mind - bits_cell)).min(Dy::ONE);
    kg_cell + (c.m_mind_kg - kg_cell).max(Dy::ZERO) * share
}

/// The unit body's metabolism (W): a mind-bearing body of m_mind_kg, held as
/// a ruler (3.4 W kg^-3/4 x 10 kg^3/4 = 19.1 W). A body's own is this times
/// 2s / (1 + s), s its mass over the unit body's, at its chemistry's pace
/// (worlds::body_pace).
pub fn unit_metabolism_w() -> Dy {
    r(191, 10)
}

/// What a generation is made of, read once per ladder.
struct Gen {
    bits_cell: Dy,
    t_cell: Dy,
    kg_cell: Dy,
}

/// A body's generation (s): the body is built by its cells dividing at their
/// chemistry's pace, each doubling copying the genome by parallel forks of one
/// copier's span, so a generation is t_cell (1 + log2(body / cell)). Food
/// limits how many bodies are raised (the ladder's population), not how fast
/// one is built.
fn generation_of(bits: Dy, g: &Gen) -> Dy {
    let body = body_kg(bits, g.bits_cell, g.kg_cell);
    let cells = (body / g.kg_cell.max(Dy::sci(1, -300))).max(Dy::ONE);
    g.t_cell * (Dy::ONE + cells.log() / Dy::int(2).log())
}

/// Bits selection writes per generation in a population of `pop` (Haldane;
/// capped at MacKay's one bit).
pub fn bits_per_generation(pop: Dy, b: Dy) -> Dy {
    let e = Dy::ONE.exp();
    k().bits_per_generation
        .min(b * k().reproductive_excess / pop.max(e).log())
}

/// Seconds to write from `lo` to `hi` bits at `rate` bits per generation. A
/// genome within one copier's span is copied serially (span written times its
/// mean); beyond it a generation is a body's (`generation_of` at the span's
/// midpoint).
fn fill_s(lo: Dy, hi: Dy, g: &Gen, b: Dy, kc: Dy, rate: Dy) -> Dy {
    let bits_cell = g.bits_cell;
    let c = k();
    let mut out = Dy::ZERO;
    let mut lo = lo;
    if lo < bits_cell {
        let top = hi.min(bits_cell);
        out += (top - lo) * ((top + lo) / n(2)) / (b * kc);
        lo = top;
    }
    if hi > lo {
        let _ = c;
        let mid = (lo + hi) / n(2);
        out += (hi - lo) * generation_of(mid, g);
    }
    out / rate
}

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
    let c = k();
    let plasma = m.solvent == Solvent::Plasma;
    let eps0 = if plasma {
        field_fidelity(m.field_ev, m.t)
    } else {
        fidelity(m.solvent, m.t, ctx)
    };
    if eps0 >= r(1, 2) {
        // a copy no better than a coin: no information is kept
        return None;
    }
    let kc = copy_rate(m.t, m.richness);
    if kc.signum() <= 0 {
        return None;
    }
    let b = c.bits_per_monomer;
    let strong = worlds::covalent_ev(&ctx.sc) * c.ev_j;
    let power = m.energy.max(Dy::ZERO) * c.flux_w * area_m2;
    // a plasma's pattern is lost by reconnection, against its field's hold
    let r_decay = if plasma {
        kc * (-(m.field_ev.max(Dy::ZERO) / (worlds::kb_ev() * m.t.max(Dy::ONE)))).exp()
    } else {
        decay_rate(m.solvent, m.t, kc, ctx)
    };
    let bits_cell = b / eps0;
    let t_cell = bits_cell / (b * kc);
    // its monomers are built from its own solvent's molecules
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
        dormant: if plasma {
            Dy::ONE
        } else {
            let t = m.t.max(Dy::ONE);
            c.kb_j * t / c.h_j
                * (-(worlds::strong_bond_ev(m.solvent, &ctx.sc) / (worlds::kb_ev() * t))).exp()
        },
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
        // the decay a generation suffers depends on the body it builds
        for _ in 0..3 {
            let cap = b / err.powi(power_s);
            gen_s = gen_of(cap.min(c.i_mind));
            err = eps0 + r_decay * gen_s;
        }
        if err >= r(1, 2) {
            // it forgets faster than it can check: no further rung
            return Some(lad);
        }
        let cap = b / err.powi(power_s);
        if cap <= prev {
            return Some(lad);
        }
        let target = cap.min(c.i_mind);
        let monomers = target / b;
        let body = body_kg(target, bits_cell, kg_cell);
        // how many it can feed: the power reaching it over the energy to
        // build one body in a generation
        let pop = power * gen_s / (body / bond_kg * strong).max(sci(1, -300));
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

/// The mass of a monomer's building block in a solvent (amu): its own
/// molecule (a silicate unit, Si and two O, for a melt); a plasma copies
/// nothing.
fn backbone_amu(s: Solvent) -> Dy {
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
            a(14) + n(2) * a(8)
        }
        // its carriers: protons
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
    let tiny = sci(1, -12);
    let mut t = t0;
    let mut times = Vec::new();
    let mut setbacks = Dy::ZERO;
    for (s, &mean) in means.iter().enumerate() {
        let mean = mean.max(tiny);
        if s == 0 {
            let mut need = mean;
            if let Some(a) = arrives {
                // or carried here from a neighbour
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
            // the expected completion under memoryless restarts, capped so an
            // impossible rung reads as past the horizon rather than overflowing
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
        &target.star_mass.max(sci(1, -9)).to_q(),
        &target.a_au.max(sci(1, -9)).to_q(),
    ));
    let reach = target.radius * Dy::from_q(&crate::package::Si::world_radius_m())
        / (n(2) * Dy::PI * target.a_au.max(sci(1, -9)) * Dy::from_q(&crate::package::Si::au()));
    year / reach.max(sci(1, -30)) / sci(1, 9)
}

// ---------------------------------------------------------------------- a world's life

/// The share of a world's year and surface warm enough to grow, by its tilt
/// (whole degrees): daily-mean sunlight at each latitude through the year,
/// counted where it reaches half that latitude's own yearly mean, weighted by
/// each band's area; to a thousandth.
pub fn growing_share(tilt_deg: i64) -> Dy {
    static T: OnceLock<Vec<Dy>> = OnceLock::new();
    let table = T.get_or_init(|| (0..=90).map(growing).collect());
    table[usize::try_from(tilt_deg.unsigned_abs().min(90)).unwrap_or(0)]
}

fn growing(deg: i64) -> Dy {
    let eps = n(deg) * Dy::PI / n(180);
    let days = 48i64;
    let (mut total, mut weight) = (Dy::ZERO, Dy::ZERO);
    for i in 0..9i64 {
        let phi = n(5 + 10 * i) * Dy::PI / n(180);
        let (sp, cp) = phi.sin_cos();
        let mut qs = Vec::with_capacity(48);
        for d in 0..days {
            let dec = eps * (n(2) * Dy::PI * (n(d) + r(1, 2)) / n(days)).sin();
            let (sd, cd) = dec.sin_cos();
            // -tan(phi) tan(dec)
            let x = -(sp / cp) * (sd / cd);
            let h0 = if x >= Dy::ONE {
                Dy::ZERO
            } else if x <= -Dy::ONE {
                Dy::PI
            } else {
                x.acos()
            };
            qs.push((h0 * sp * sd + cp * cd * h0.sin()).max(Dy::ZERO));
        }
        let mean = qs.iter().copied().sum::<Dy>() / n(days);
        let grow = if mean.signum() > 0 {
            n(i64::try_from(qs.iter().filter(|&&q| q >= mean * r(1, 2)).count()).unwrap_or(0))
                / n(days)
        } else {
            Dy::ZERO
        };
        total += grow * cp;
        weight += cp;
    }
    // to a thousandth, rounded
    n((total / weight * n(1000) + r(1, 2)).trunc_i64()) / n(1000)
}

/// What a world's body is, for its life and its foragers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Body {
    /// Radius in world radii.
    pub radius: Dy,
    /// Whether it is a giant (life confined to a habitable band, a twentieth of the
    /// surface).
    pub giant: bool,
    /// Whether it is tidally locked to its star at the settled age.
    pub locked: bool,
    /// Its axis's tilt from upright (whole degrees, 0 to 90).
    pub tilt: i64,
    /// Its orbit, its reach a (AU), and its star's light and mass (as the
    /// system's spec holds them).
    pub a_au: Dy,
    /// Its star's luminosity, as the system's spec holds it (suns).
    pub star_lum: Dy,
    /// Its star's mass, as the system's spec holds it (suns).
    pub star_mass: Dy,
    /// The age (Gyr) at which the world merges into its neighbour and ends;
    /// None for a world that stands.
    pub merged_at: Option<Dy>,
}

impl Body {
    /// A world's body at the settled age.
    pub fn of(set: &Setting) -> Body {
        let age = settled_gyr();
        let (locked, cos) = match set.path {
            Some(p) => (age >= p.lock_gyr, p.cos_obliquity_at(age)),
            None => (false, Dy::ONE),
        };
        let obl = cos.acos();
        let tilt = obl.min(Dy::PI - obl) * n(180) / Dy::PI;
        Body {
            radius: Dy::from_q(&set.world.radius),
            giant: set.world.giant,
            locked,
            tilt: (tilt + r(1, 2)).trunc_i64(),
            a_au: Dy::from_q(&set.world.a),
            star_lum: set.lum,
            star_mass: set.star_mass,
            merged_at: set.world.merged_at.as_ref().map(Dy::from_q),
        }
    }

    /// Its whole area (m^2), linear in radius against the unit world; a medium
    /// lives across its band of it (`Medium::band`).
    pub fn area_m2(&self) -> Dy {
        let world_radius_m = Dy::from_q(&crate::package::Si::world_radius_m());
        n(4) * Dy::PI * self.radius * world_radius_m * world_radius_m
    }
}

/// How many foragers a world feeds once a people has evolved there: living
/// area x seasons x energy reaching its chemistry, through one link of a food
/// chain (Lindeman's tenth), divided among bodies by what each burns (at its
/// chemistry's own temperature). A variable world (tilted axis, small flaring
/// star) breeds small r-strategists, a steady one large K-strategists; body
/// mass scales as 1/g. 50 000 for a unit-world area of ideal land.
pub fn foragers(ms: &[Medium], region: Region, body: &Body, ctx: &Ctx) -> Dy {
    let Some(med) = best(ms, Some(region)) else {
        return Dy::ZERO;
    };
    let area = body.radius * med.band;
    let season = if body.locked {
        r(1, 2)
    } else {
        growing_share(body.tilt)
    };
    let gravity = body.radius * if body.giant { r(1, 4) } else { Dy::ONE };
    let r_strategist = body.tilt > 23 || body.star_mass < r(1, 2);
    // a body's mass over a K-strategist's on a unit world: the fertility
    // ceilings' ratio (r 2.5, K 1.2) over the world's gravity; what it burns
    // is its mass's share 2m / (1 + m)
    let ceiling = if r_strategist { r(5, 2) } else { r(6, 5) };
    let mut mass = r(6, 5) / ceiling;
    if region != Region::Plasmic {
        mass = mass / gravity.max(sci(1, -3));
    }
    let factor = (n(2) * mass / (Dy::ONE + mass)) * worlds::body_temperature(med.solvent, ctx)
        / worlds::t_unit();
    n(50_000) * area * season * med.energy * r(1, 10) / factor.max(sci(1, -300))
}

/// A star's main-sequence life (Gyr) at `r_kpc` from the core: its hold over
/// the pulse's burn at its reach (stars::life), under the universe's nuclear
/// scale.
pub fn lifetime(r_kpc: Dy, ctx: &Ctx) -> Dy {
    Dy::from_q(&crate::stars::life(crate::stars::reach_rings(r_kpc))) * ctx.sc.nuclear
}

/// A chemistry a world's physics allows that can copy itself at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chemistry {
    /// The region of the world this chemistry lives in.
    pub region: Region,
    /// The chemistry's ladder to a mind, from its region's best medium.
    pub ladder: Ladder,
    /// The chance it is never sterilised before its window closes.
    pub live: Dy,
    /// The sterilising hazard (per Gyr): the expected time to a sterilising
    /// pulse is its reciprocal.
    pub hazard: Dy,
    /// When its window closes (Gyr after the Big Bang): its star's leaving the
    /// main sequence; never, for a sea in the dark.
    pub end: Option<Dy>,
    /// Strikes per Gyr from its system's debris (see [`strikes_per_gyr`]).
    pub mu: Dy,
    /// When its medium first exists on its world (Gyr after the Big Bang):
    /// its clock's start.
    pub start: Dy,
}

/// Where a star's worlds live: its birth, place and how much of its orbit it
/// spends in the crowded plane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Home {
    /// The star's birth (Gyr after the Big Bang).
    pub born: Dy,
    /// The star's distance from its galaxy's core (kpc).
    pub r_kpc: Dy,
    /// Share of its orbit spent inside the crowded plane layer at the settled age,
    /// in \[0, 1\].
    pub exposure: Dy,
    /// The star's mass (suns).
    pub star_mass: Dy,
    /// The star's luminosity (suns).
    pub lum: Dy,
    /// The share of its system's solids left as debris: belts and comet
    /// cloud over everything the disc gathered, in \[0, 1\].
    pub debris: Dy,
}

impl Home {
    /// The home of `star`'s worlds in `gal`, reading its birth, place and exposure
    /// (at the settled age) and its system's stellar mass and luminosity.
    pub fn of(_gal: &Galaxy, star: &Star, sys: &System) -> Home {
        Home {
            born: Dy::from_q(&star.born),
            r_kpc: Dy::from_q(&star.r),
            exposure: star.exposure(settled_gyr()),
            star_mass: Dy::from_q(&sys.spec.mass),
            lum: Dy::from_q(&sys.spec.lum),
            debris: debris_share(sys),
        }
    }
}

/// The share of a system's solids left as debris: its belts and comet cloud
/// over those plus its worlds.
pub fn debris_share(sys: &System) -> Dy {
    let left = sys
        .belts
        .iter()
        .fold(xyl_core::Q::ZERO, |a, b| &a + &b.mass)
        + sys
            .comets
            .as_ref()
            .map(|(m, _)| m.clone())
            .unwrap_or(xyl_core::Q::ZERO);
    let gathered = sys
        .worlds
        .iter()
        .fold(xyl_core::Q::ZERO, |a, w| &a + &w.mass);
    let all = &left + &gathered;
    if all.signum() > 0 {
        Dy::from_q(&left.checked_div(&all).unwrap_or(xyl_core::Q::ZERO))
    } else {
        Dy::ZERO
    }
}

/// Each chemistry a world's settled media allow: per region, its best medium
/// by the same rule that later feeds a people (`worlds::best`: richness x
/// energy capped at a flux unit), with its ladder, its chance never to be
/// sterilised, when its window ends and its impact rate.
pub fn chemistries(ms: &[Medium], body: &Body, home: &Home, ctx: &Ctx) -> Vec<Chemistry> {
    let mut regions: Vec<Region> = Vec::new();
    for m in ms {
        if !regions.contains(&m.region) {
            regions.push(m.region);
        }
    }
    regions.sort_by_key(|rg| rg.name());
    let by: Vec<(Region, &Medium)> = regions
        .into_iter()
        .filter_map(|rg| best(ms, Some(rg)).map(|m| (rg, m)))
        .collect();
    // a lit world's window closes as its star climbs the giant branch (its
    // last tenth): the swelling light boils its seas
    let star_end = home.born + lifetime(home.r_kpc, ctx) * r(9, 10);
    let crowd = worlds::cosmic_rays(home.r_kpc) * home.exposure;
    let area = body.area_m2();
    let mut out = Vec::new();
    for (region, m) in by {
        let Some(lad) = ladder(m, area * m.band, ctx) else {
            continue;
        };
        if lad.steps.is_empty() {
            continue;
        }
        let dark = matches!(m.at, Where::UnderIce | Where::Deep);
        // the window closes with the star's swelling (a lit world) or with
        // the world's own merging into its neighbour (any world)
        let end = match (dark, body.merged_at) {
            (true, None) => None,
            (true, Some(merged)) => Some(merged),
            (false, None) => Some(star_end),
            (false, Some(merged)) => Some(star_end.min(merged)),
        };
        let window = end
            .map(|e| e.min(horizon_gyr()))
            .unwrap_or_else(horizon_gyr)
            - home.born;
        // strikes come from its own system's debris; a sterilising pulse
        // from the dying stars around it
        let rate = sterilise_per_gyr(crowd, dark);
        let hazard = rate * window;
        out.push(Chemistry {
            region,
            ladder: lad,
            live: (-hazard).exp(),
            hazard: rate,
            end,
            mu: strikes_per_gyr(home.debris, body),
            start: home.born,
        });
    }
    out
}

/// A world's life: the furthest of its chemistries' lineages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lineage {
    /// The region of the chemistry this lineage belongs to.
    pub region: Region,
    /// When each rung was reached (Gyr after the Big Bang).
    pub steps: Vec<Dy>,
    /// Rungs its ladder has, and whether the last holds a mind.
    pub n: usize,
    /// Whether its ladder's last rung holds a mind's blueprint.
    pub complete: bool,
    /// Number of impacts that cost it progress.
    pub setbacks: u32,
    /// Whether its surface was sterilised before its window closed.
    pub sterilised: bool,
    /// Impacts beyond adaptation per Gyr it climbed under.
    pub mu: Dy,
    /// Chance it is never sterilised before its window closes, in \[0, 1\].
    pub live: Dy,
    /// When its window closes (Gyr after the Big Bang); `None` for a sea in the
    /// dark.
    pub end: Option<Dy>,
    /// Per-monomer miscopy probability of its bare chemistry, eps0.
    pub eps0: Dy,
    /// Mean wait of each rung of its ladder (Gyr).
    pub means: Vec<Dy>,
    /// Whether its first rung came on a neighbour's ejecta.
    pub seeded: bool,
}

impl Lineage {
    /// Whether it raised a mind (every rung of a complete ladder climbed).
    pub fn mind(&self) -> bool {
        self.complete && self.steps.len() == self.n
    }
}

/// A world's life as its chemistry gives it, in closed form: for each
/// chemistry, the rungs it climbs and when, and whether it was sterilised;
/// the furthest lineage (soonest, at a tie) is the world's. `seeded`: per
/// region, when life of that chemistry could arrive from a neighbour.
pub fn lineage(key: &[u64], chems: &[Chemistry], seeded: &[(Region, Dy)]) -> Option<Lineage> {
    let _ = key;
    let mut best: Option<((bool, Dy, Dy), Lineage)> = None;
    for ch in chems {
        let t0 = ch.start;
        if t0 >= horizon_gyr() {
            continue;
        }
        let horizon = ch
            .end
            .map(|e| e.min(horizon_gyr()))
            .unwrap_or_else(horizon_gyr);
        // the sterilising pulse at its expected time, the hazard's reciprocal
        let steril = if ch.hazard.signum() > 0 && horizon > t0 {
            let at = t0 + Dy::ONE / ch.hazard;
            (at < horizon).then_some(at)
        } else {
            None
        };
        let means: Vec<Dy> = ch.ladder.steps.iter().map(|s| s.mean_gyr).collect();
        let arrives = seeded
            .iter()
            .find(|(rg, _)| *rg == ch.region)
            .map(|(_, t)| *t);
        let got = climb(&means, t0, horizon, steril, ch.mu, arrives);
        let rec = Lineage {
            region: ch.region,
            n: means.len(),
            complete: ch.ladder.complete,
            setbacks: got.setbacks,
            sterilised: steril.is_some_and(|s| s < horizon),
            mu: ch.mu,
            live: ch.live,
            end: ch.end,
            eps0: ch.ladder.eps0,
            means,
            seeded: arrives.is_some_and(|a| got.times.first() == Some(&a.max(t0))),
            steps: got.times,
        };
        let share = n(i64::try_from(rec.steps.len()).unwrap_or(0))
            / n(i64::try_from(rec.n.max(1)).unwrap_or(1));
        let k = (
            rec.mind(),
            share,
            -rec.steps.last().copied().unwrap_or(Dy::ZERO),
        );
        if best.as_ref().is_none_or(|(bk, _)| k > *bk) {
            best = Some((k, rec));
        }
    }
    best.map(|(_, l)| l)
}

/// A world of a system, ready for its life to be read: its setting, its
/// settled state and media, its body and chemistries.
pub struct Living<'a> {
    /// The world's place in its system and around its star.
    pub setting: Setting<'a>,
    /// The world's state at its last probe (see [`probe_ages`]).
    pub state: State,
    /// Every medium its state offers at any probe.
    pub media: Vec<Medium>,
    /// Its body as life reads it.
    pub body: Body,
    /// The chemistries its media allow (see [`chemistries`]).
    pub chems: Vec<Chemistry>,
}

/// Every world of a system's life, with what its neighbours carry to it: rock
/// an impact throws off one world reaches the others in about a million of
/// its orbits - microbes travel, minds do not.
pub fn system_life<'a>(
    gal: &Galaxy,
    star: &Star,
    sys: &'a System,
    ctx: &Ctx,
) -> (Vec<Living<'a>>, Vec<Option<Lineage>>) {
    let home = Home::of(gal, star, sys);
    let path = [star.n];
    let ages = probe_ages(&home, ctx);
    let living: Vec<Living> = worlds::settings(gal, &path, sys, star)
        .into_iter()
        .filter_map(|setting| {
            // each chemistry starts at the first probe its medium exists at,
            // its ladder read from the medium as it first is
            let body = Body::of(&setting);
            let (mut chems, mut all, mut last): (Vec<Chemistry>, Vec<Medium>, Option<State>) =
                (Vec::new(), Vec::new(), None);
            for &age in &ages {
                let state = build(&setting, age, ctx);
                let ms = media(&state, setting.z, ctx);
                for ch in chemistries(&ms, &body, &home, ctx) {
                    if !chems.iter().any(|c| c.region == ch.region) {
                        chems.push(Chemistry {
                            start: home.born + age,
                            ..ch
                        });
                    }
                }
                for m in ms {
                    if !all
                        .iter()
                        .any(|x: &Medium| x.solvent == m.solvent && x.at == m.at)
                    {
                        all.push(m);
                    }
                }
                last = Some(state);
            }
            let state = last?;
            Some(Living {
                setting,
                state,
                media: all,
                body,
                chems,
            })
        })
        .collect();
    let key = |w: usize| [star.n, w as u64];
    let first: Vec<Option<Lineage>> = living
        .iter()
        .enumerate()
        .map(|(w, l)| lineage(&key(w), &l.chems, &[]))
        .collect();
    let mut arrivals: Vec<Vec<(Region, Dy)>> = vec![Vec::new(); living.len()];
    for (w, lin) in first.iter().enumerate() {
        let Some(lin) = lin else { continue };
        let Some(&t1) = lin.steps.first() else {
            continue;
        };
        for (v, arr) in arrivals.iter_mut().enumerate() {
            if v == w {
                continue;
            }
            // dormant in transit: the share whose first rung survives the
            // crossing, exp(-dormant x monomers x transit); the arrival waits
            // on the survivors, transit / survival
            let transit = transfer_gyr(&living[v].body);
            let survival = living[w]
                .chems
                .iter()
                .find(|c| c.region == lin.region)
                .and_then(|c| c.ladder.steps.first().map(|st| (c.ladder.dormant, st.bits)))
                .map(|(dormant, bits)| {
                    let monomers = bits / bits_per_monomer();
                    (-(dormant * monomers * transit * k().gyr_s).min(n(200))).exp()
                })
                .unwrap_or(Dy::ZERO);
            if survival <= Dy::ZERO {
                continue;
            }
            let when = t1 + transit / survival;
            match arr.iter_mut().find(|(rg, _)| *rg == lin.region) {
                Some(slot) => slot.1 = slot.1.min(when),
                None => arr.push((lin.region, when)),
            }
        }
    }
    let lins = first
        .into_iter()
        .enumerate()
        .map(|(w, f)| {
            if arrivals[w].is_empty() {
                f
            } else {
                lineage(&key(w), &living[w].chems, &arrivals[w])
            }
        })
        .collect();
    (living, lins)
}

/// A world as it is at an epoch (Gyr after the Big Bang): its media read at
/// its star's age then, in the quarter-Gyr steps a world in play is read at -
/// for judging what a mind waking then can live on.
pub fn world_at(set: &Setting, born: Dy, epoch: Dy, ctx: &Ctx) -> Vec<Medium> {
    let quarters = ((epoch - born) * n(4) + r(1, 2)).trunc_i64();
    let age = (n(quarters) / n(4)).max(first_age(set.r_kpc, ctx));
    let state = build(set, age, ctx);
    media(&state, set.z, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::laws::Scales;

    fn medium(s: Solvent, t: Dy, energy: Dy, richness: Dy) -> Medium {
        Medium {
            solvent: s,
            at: Where::Surface,
            t,
            energy,
            amount: Dy::ONE,
            dark: false,
            radiolysis: Dy::ZERO,
            region: Region::Biotic,
            richness,
            pace: Dy::ONE,
            band: Dy::ONE,
            field_ev: Dy::ZERO,
        }
    }

    #[test]
    fn a_plasma_copies_only_where_its_field_dominates_its_heat() {
        let t = sci(1, 7);
        let kt = worlds::kb_ev() * t;
        // a field holding many times kT per carrier copies faithfully
        assert!(field_fidelity(kt * n(20), t) < r(1, 1000));
        // a field far below kT keeps nothing: a coin toss or worse
        assert!(field_fidelity(kt / n(100), t) > r(1, 2));
        // and the fidelity only worsens as the field falls against the heat
        assert!(field_fidelity(kt * n(2), t) < field_fidelity(kt, t));
    }

    #[test]
    fn water_copies_as_a_polymerase_and_plasma_not_at_all() {
        let ctx = Ctx::new(Scales::default());
        let e = fidelity(Solvent::Volatile(worlds::Species::H2O), n(300), &ctx);
        assert!(e > Dy::ZERO && e < Dy::ONE, "eps0 {e}");
        // a warmer sea copies worse than a cooler one
        assert!(fidelity(Solvent::Volatile(worlds::Species::H2O), n(350), &ctx) > e);
        assert_eq!(fidelity(Solvent::Plasma, sci(1, 4), &ctx), Dy::ONE);
        // methane's weak cohesion copies worse than water's: it needs more
        // rungs of proofreading to reach a mind
        let ch4 = fidelity(Solvent::Volatile(worlds::Species::CH4), n(100), &ctx);
        assert!(ch4 > e && ch4 < r(1, 2), "methane {ch4} vs water {e}");
        let world_radius_m = Dy::from_q(&crate::package::Si::world_radius_m());
        let area = n(4) * Dy::PI * world_radius_m * world_radius_m;
        let rungs = |s: worlds::Species, t: Dy| {
            ladder(
                &medium(Solvent::Volatile(s), t, r(1, 2), r(3, 4)),
                area,
                &ctx,
            )
            .map(|l| l.steps.len())
            .unwrap_or(0)
        };
        assert!(rungs(worlds::Species::CH4, n(100)) > rungs(worlds::Species::H2O, n(280)));
    }

    #[test]
    fn a_temperate_sea_climbs_to_a_mind_in_a_few_rungs() {
        let ctx = Ctx::new(Scales::default());
        let m = medium(
            Solvent::Volatile(worlds::Species::H2O),
            n(288),
            r(1, 2),
            r(3, 4),
        );
        let world_radius_m = Dy::from_q(&crate::package::Si::world_radius_m());
        let area = n(4) * Dy::PI * world_radius_m * world_radius_m;
        let lad = ladder(&m, area, &ctx).expect("water copies");
        assert!(
            lad.complete,
            "water reaches a mind's blueprint: {} rungs",
            lad.steps.len()
        );
        assert!(!lad.steps.is_empty());
        let total: Dy = lad.steps.iter().map(|s| s.mean_gyr).sum();
        assert!(total > Dy::ZERO, "mean climb {total} Gyr");
        // rungs hold more as they rise, and bodies grow
        for w in lad.steps.windows(2) {
            assert!(w[1].bits > w[0].bits && w[1].body_kg >= w[0].body_kg);
        }
        // a melt never holds enough
        let melt = medium(Solvent::Melt, n(2000), Dy::ONE, r(1, 2));
        assert!(ladder(&melt, area, &ctx).is_none_or(|l| !l.complete));
    }

    #[test]
    fn a_climb_is_its_expectation_and_impacts_only_delay_it() {
        let means = [r(1, 2), Dy::ONE, Dy::ONE];
        let run = |mu: Dy| climb(&means, Dy::ZERO, n(30), None, mu, None);
        let calm = run(Dy::ZERO);
        assert_eq!(calm, run(Dy::ZERO));
        assert_eq!(calm.times.len(), 3);
        assert_eq!(calm.times[2], r(5, 2));
        // impacts delay every rung after the first and never advance one
        let hit = run(n(2));
        assert!(hit.times.len() <= 3 && hit.times.first() == calm.times.first());
        for (a, b) in hit.times.iter().zip(calm.times.iter()).skip(1) {
            assert!(a > b);
        }
        assert!(hit.setbacks > 0);
        // sterilised before life begins: nothing
        let dead = climb(&means, Dy::ZERO, n(30), Some(sci(1, -9)), Dy::ZERO, None);
        assert!(dead.times.is_empty());
    }

    #[test]
    fn seasons_and_gamma() {
        assert_eq!(growing_share(0), Dy::ONE);
        assert!(
            growing_share(23) < Dy::ONE && growing_share(23) > r(1, 2),
            "23: {}",
            growing_share(23)
        );
        assert!(
            growing_share(90) < growing_share(23),
            "a world on its side has polar nights"
        );
        // rock reaches a small, far world more slowly than a large, near one
        let near = Body {
            radius: n(2),
            giant: false,
            locked: false,
            tilt: 20,
            a_au: Dy::ONE,
            star_lum: Dy::ONE,
            star_mass: Dy::ONE,
            merged_at: None,
        };
        let far = Body {
            radius: Dy::ONE,
            a_au: n(5),
            ..near
        };
        assert!(transfer_gyr(&far) > transfer_gyr(&near));
    }
}
