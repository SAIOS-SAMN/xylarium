//! The rates a ladder is built from: impacts, sterilisation, copying
//! fidelity, copy and decay rates, bodies and generations.

use super::*;

/// Impacts on a world's biosphere per Gyr from the debris of the world's
/// system: the debris share of the system's solids (belts and comet cloud
/// over everything the disc gathered), clamped to [0, 1], times the orbits
/// per Gyr, times the share `R / (2 pi a)` of the orbit that the world's
/// body covers.
pub fn strikes_per_gyr(debris: Dy, body: &Body) -> Dy {
    let year = Dy::from_q(&crate::turning::year_yr(
        &body.star_mass.max(Dy::sci(1, -9)).to_q(),
        &body.a_au.max(Dy::sci(1, -9)).to_q(),
    ));
    let orbits = Dy::sci(1, 9) / year.max(Dy::sci(1, -12));
    // The share R / (2 pi a), with the radius (world radii) and the orbit
    // (AU) both converted to metres.
    let reach = body.radius * Dy::from_q(&crate::package::Si::world_radius_m())
        / (Dy::int(2)
            * Dy::PI
            * body.a_au.max(Dy::sci(1, -9))
            * Dy::from_q(&crate::package::Si::au()));
    debris.clamp(Dy::ZERO, Dy::ONE) * orbits * reach
}

/// The rate at which a world's life is sterilised (per Gyr) by a nearby
/// supernova or burst. It is the declared chance per 10 Gyr over 10, times
/// `crowd` (the cosmic-ray level times the share of the star's orbit spent
/// among dying stars); it is zero for a `dark` medium (a sea under ice or in
/// the deep).
pub fn sterilise_per_gyr(crowd: Dy, dark: bool) -> Dy {
    if dark {
        Dy::ZERO
    } else {
        constants().sterilise_at_r0 / Dy::int(10) * crowd.max(Dy::ZERO)
    }
}

/// The chance `eps0` that a monomer is miscopied in solvent `s` at
/// temperature `t` (K): `eps0 = exp(-d E_weak / kT)`, with `E_weak` the
/// solvent's weak bond. It is 1 where the weak bond is not positive.
pub fn fidelity(s: Solvent, t: Dy, ctx: &Ctx) -> Dy {
    let weak = worlds::weak_bond_ev(s, ctx);
    if weak.signum() <= 0 {
        return Dy::ONE;
    }
    (-(constants().discrimination * weak / (worlds::kb_ev() * t.max(Dy::ONE)))).exp()
}

/// The chance `eps0` that a pattern held in a plasma's field topology is
/// miscopied: `eps0 = exp(-d E_B / kT)`, with `E_B` the field energy per
/// carrier (`Medium::field_ev`) and `d` the same discrimination of one half
/// that a bonded copy uses. The chance is small only where `E_B` exceeds
/// `kT`.
pub fn field_fidelity(field_ev: Dy, t: Dy) -> Dy {
    (-(constants().discrimination * field_ev.max(Dy::ZERO) / (worlds::kb_ev() * t.max(Dy::ONE))))
        .exp()
}

/// The monomers a copier adds per second, `(kT / h) e^-barrier` with the
/// solvent barrier in units of `kT` (corresponding states), times the
/// `richness` of building material at hand.
pub fn copy_rate(t: Dy, richness: Dy) -> Dy {
    constants().kb_j * t.max(Dy::ONE) / constants().h_j
        * (-worlds::solvent_barrier()).exp()
        * richness.max(Dy::ZERO)
}

/// The breakage rate per monomer of a lineage's stored sequence (per
/// second), the sum of two terms: the reverse of the copying reaction,
/// `k_copy exp(-(E_strong - E_weak) / kT)`, uphill by what the strong bond
/// holds beyond the solvent's cohesion; and thermal homolysis,
/// `(kT / h) exp(-E_strong / kT)`.
pub fn decay_rate(s: Solvent, t: Dy, k_copy: Dy, ctx: &Ctx) -> Dy {
    let t = t.max(Dy::ONE);
    let kt = worlds::kb_ev() * t;
    let strong = worlds::strong_bond_ev(s, &ctx.sc);
    let held = (strong - worlds::weak_bond_ev(s, ctx)).max(Dy::ZERO);
    k_copy * (-(held / kt)).exp() + constants().kb_j * t / constants().h_j * (-(strong / kt)).exp()
}

/// The body mass (kg) a lineage holding `bits` builds: a cell's mass at or
/// below the cell's bits, the mind-bearing body's mass at `I_MIND`, and
/// linear in bits between.
pub fn body_kg(bits: Dy, bits_cell: Dy, kg_cell: Dy) -> Dy {
    let c = constants();
    if bits <= bits_cell || c.i_mind <= bits_cell {
        return kg_cell;
    }
    let share = ((bits - bits_cell) / (c.i_mind - bits_cell)).min(Dy::ONE);
    kg_cell + (c.m_mind_kg - kg_cell).max(Dy::ZERO) * share
}

/// The unit body's metabolism (W): a mind-bearing body of `m_mind_kg`, held
/// as a ruler (3.4 W kg^-3/4 x 10 kg^3/4 = 19.1 W). The metabolism of a body
/// of mass ratio `s` to the unit body is this value times `2s / (1 + s)`, at
/// the pace of the body's chemistry (`worlds::body_pace`).
pub fn unit_metabolism_w() -> Dy {
    Dy::ratio(191, 10)
}

/// The cell's bits, generation time (s) and mass (kg), read once per ladder.
pub(super) struct Gen {
    pub(super) bits_cell: Dy,
    pub(super) t_cell: Dy,
    pub(super) kg_cell: Dy,
}

/// A body's generation time (s). The body is built by cell divisions at the
/// chemistry's pace, each division copying the genome by parallel forks of
/// one copier's span, so a generation is `t_cell (1 + log2(body / cell))`.
/// Food limits how many bodies are raised (the ladder's population), not how
/// fast one is built.
pub(super) fn generation_of(bits: Dy, g: &Gen) -> Dy {
    let body = body_kg(bits, g.bits_cell, g.kg_cell);
    let cells = (body / g.kg_cell.max(Dy::sci(1, -300))).max(Dy::ONE);
    g.t_cell * (Dy::ONE + cells.log() / Dy::int(2).log())
}

/// The bits selection writes per generation in a population of `pop`:
/// Haldane's cost, `b x excess / ln(pop)` with the declared reproductive
/// excess of 1/2, capped at MacKay's one bit.
pub fn bits_per_generation(pop: Dy, b: Dy) -> Dy {
    let e = Dy::ONE.exp();
    constants()
        .bits_per_generation
        .min(b * constants().reproductive_excess / pop.max(e).log())
}

/// The seconds needed to write from `lo` to `hi` bits at `rate` bits per
/// generation. Below the cell's bits the genome is copied serially, so a
/// generation lasts the genome's mean length over `b x kc` (bits per monomer
/// times the copy rate); above it a generation is a body's (`generation_of`
/// at the interval's midpoint).
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
