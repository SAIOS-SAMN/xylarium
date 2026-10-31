//! The rates a ladder is built from: impacts, sterilisation, copying
//! fidelity, copy and decay rates, bodies and generations.

use super::*;

/// Strikes a world's biosphere suffers per Gyr from the debris its own
/// system left: the debris' share of the system's solids (belts and comet
/// cloud over everything the disc gathered), swept once an orbit over the
/// share of its orbit the world's own body covers, R / (2 pi a).
pub fn strikes_per_gyr(debris: Dy, body: &Body) -> Dy {
    let year = Dy::from_q(&crate::turning::year_yr(
        &body.star_mass.max(Dy::sci(1, -9)).to_q(),
        &body.a_au.max(Dy::sci(1, -9)).to_q(),
    ));
    let orbits = Dy::sci(1, 9) / year.max(Dy::sci(1, -12));
    // R / (2 pi a), radius (world radii) and orbit (AU) both in metres
    let reach = body.radius * Dy::from_q(&crate::package::Si::world_radius_m())
        / (Dy::int(2)
            * Dy::PI
            * body.a_au.max(Dy::sci(1, -9))
            * Dy::from_q(&crate::package::Si::au()));
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
        constants().sterilise_at_r0 / Dy::int(10) * crowd.max(Dy::ZERO)
    }
}

/// eps0: the chance a monomer is miscopied in this solvent at `t` (1 where
/// nothing binds).
pub fn fidelity(s: Solvent, t: Dy, ctx: &Ctx) -> Dy {
    let weak = worlds::weak_bond_ev(s, ctx);
    if weak.signum() <= 0 {
        return Dy::ONE;
    }
    (-(constants().discrimination * weak / (worlds::kb_ev() * t.max(Dy::ONE)))).exp()
}

/// eps0 for a plasma: the chance a pattern held in its field's topology is
/// miscopied, exp(-d E_B / kT), E_B the field's energy per carrier
/// (`Medium::field_ev`) and d the same half-contact discrimination a bonded
/// copy uses. A plasma copies only where its field dominates its heat.
pub fn field_fidelity(field_ev: Dy, t: Dy) -> Dy {
    (-(constants().discrimination * field_ev.max(Dy::ZERO) / (worlds::kb_ev() * t.max(Dy::ONE))))
        .exp()
}

/// Monomers a copier adds per second, scaled by how much of what it builds
/// with is at hand: kT/h e^-barrier by corresponding states.
pub fn copy_rate(t: Dy, richness: Dy) -> Dy {
    constants().kb_j * t.max(Dy::ONE) / constants().h_j
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
    k_copy * (-(held / kt)).exp() + constants().kb_j * t / constants().h_j * (-(strong / kt)).exp()
}

/// The body a lineage holding `bits` builds: a cell at the first rung, a
/// mind-bearing body at I_MIND, linear in bits between.
pub fn body_kg(bits: Dy, bits_cell: Dy, kg_cell: Dy) -> Dy {
    let c = constants();
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
    Dy::ratio(191, 10)
}

/// What a generation is made of, read once per ladder.
pub(super) struct Gen {
    pub(super) bits_cell: Dy,
    pub(super) t_cell: Dy,
    pub(super) kg_cell: Dy,
}

/// A body's generation (s): the body is built by its cells dividing at their
/// chemistry's pace, each doubling copying the genome by parallel forks of one
/// copier's span, so a generation is t_cell (1 + log2(body / cell)). Food
/// limits how many bodies are raised (the ladder's population), not how fast
/// one is built.
pub(super) fn generation_of(bits: Dy, g: &Gen) -> Dy {
    let body = body_kg(bits, g.bits_cell, g.kg_cell);
    let cells = (body / g.kg_cell.max(Dy::sci(1, -300))).max(Dy::ONE);
    g.t_cell * (Dy::ONE + cells.log() / Dy::int(2).log())
}

/// Bits selection writes per generation in a population of `pop` (Haldane;
/// capped at MacKay's one bit).
pub fn bits_per_generation(pop: Dy, b: Dy) -> Dy {
    let e = Dy::ONE.exp();
    constants()
        .bits_per_generation
        .min(b * constants().reproductive_excess / pop.max(e).log())
}

/// Seconds to write from `lo` to `hi` bits at `rate` bits per generation. A
/// genome within one copier's span is copied serially (span written times its
/// mean); beyond it a generation is a body's (`generation_of` at the span's
/// midpoint).
pub(super) fn fill_s(lo: Dy, hi: Dy, g: &Gen, b: Dy, kc: Dy, rate: Dy) -> Dy {
    let bits_cell = g.bits_cell;
    let mut out = Dy::ZERO;
    let mut lo = lo;
    if lo < bits_cell {
        let top = hi.min(bits_cell);
        out += (top - lo) * ((top + lo) / Dy::int(2)) / (b * kc);
        lo = top;
    }
    if hi > lo {
        let mid = (lo + hi) / Dy::int(2);
        out += (hi - lo) * generation_of(mid, g);
    }
    out / rate
}
