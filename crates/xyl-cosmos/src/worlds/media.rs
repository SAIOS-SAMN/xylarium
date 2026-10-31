//! Chemistry from the world: solvents, their regions and the media they
//! allow.

use super::*;

/// What a chemistry runs in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Solvent {
    /// A liquid or supercritical volatile.
    Volatile(Species),
    /// Silicate melt (surface above ~1400 K, scaled by the chemistry scale).
    Melt,
    /// A giant's deep plasma.
    Plasma,
}

impl Solvent {
    /// The solvent's name: the volatile's formula, `"melt"` or `"plasma"`.
    pub fn name(self) -> &'static str {
        match self {
            Solvent::Volatile(s) => s.name(),
            Solvent::Melt => "melt",
            Solvent::Plasma => "plasma",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Where on a world a medium sits.
pub enum Where {
    /// At the surface: an open sea, or a melt.
    Surface,
    /// In a liquid ocean beneath an ice shell.
    UnderIce,
    /// In a supercritical fluid of the air.
    Supercritical,
    /// In a giant's deep interior.
    Deep,
}

/// The engine's five regions of chemistry space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Region {
    /// A lit liquid at moderate temperature (230 to 450 K under the reference scales).
    Biotic,
    /// Cold chemistry: below 230 K under the reference scales, or in methane, nitrogen or
    /// ammonia.
    Cryogenic,
    /// Hot or rock chemistry: a melt, a supercritical fluid, or above 450 K.
    Mineral,
    /// A giant's plasma.
    Plasmic,
    /// A dark medium no starlight reaches, such as water under ice.
    Umbral,
}

impl Region {
    /// The region's lower-case name.
    pub fn name(self) -> &'static str {
        match self {
            Region::Biotic => "biotic",
            Region::Cryogenic => "cryogenic",
            Region::Mineral => "mineral",
            Region::Plasmic => "plasmic",
            Region::Umbral => "umbral",
        }
    }
}

/// A medium a chemistry could run in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Medium {
    /// What it runs in.
    pub solvent: Solvent,
    /// Where on the world it sits.
    pub at: Where,
    /// Its temperature (K).
    pub t: Dy,
    /// The free energy reaching it (flux units): drive-band light, the work the
    /// world's heat can do, and radiolysis, as each applies.
    pub energy: Dy,
    /// How much of the solvent there is: a sea's open part or a supercritical
    /// gas as partial pressure (bar); 1 for a melt or plasma.
    pub amount: Dy,
    /// Whether no starlight reaches it.
    pub dark: bool,
    /// Chemical energy radiolysis supplies it (flux units).
    pub radiolysis: Dy,
    /// Its region of chemistry space.
    pub region: Region,
    /// How much chemistry it allows, 0 to 1 (see [`richness`]).
    pub richness: Dy,
    /// How fast its chemistry runs relative to the unit temperature (see [`pace`]).
    pub pace: Dy,
    /// The share of its world's area this medium lives across, in \[0, 1\]:
    /// 1 on a rocky world; on a giant, the share of its depth over which the
    /// temperature, rising linearly from its surface to its interior, lies in
    /// the solvent's own range (`living_band`).
    pub band: Dy,
    /// The energy its field holds per carrier (eV): what tells a right
    /// pattern from a wrong one in a plasma, which has no bonds; zero for a
    /// bonded solvent.
    pub field_ev: Dy,
}

/// The world's own heat flowing out through each unit of its surface (flux units).
pub fn geothermal(w: &State) -> Dy {
    // heat over radius (not radius squared)
    constants().geo_flux * w.heat / w.radius.max(Dy::sci(1, -6))
}

/// The work the world's heat can do for a chemistry at `t_medium` (flux
/// units): its flow x Carnot's share between its interior and the medium.
pub fn free_heat(w: &State, t_medium: Dy) -> Dy {
    let t_hot = if w.t_int.signum() > 0 {
        t_medium.max(w.t_int)
    } else {
        t_medium
    };
    geothermal(w) * (Dy::ONE - t_medium / t_hot.max(Dy::ONE))
}

/// The thickness (km) of an ice shell whose base sits at its melting point,
/// conducting `q` W/m^2 to a surface at `t_surf` (conductivity 567 / T);
/// `None` with no heat flow (no shell is thin enough).
pub fn shell_km(q: Dy, t_surf: Dy, t_melt: Dy) -> Option<Dy> {
    if q.signum() <= 0 {
        return None;
    }
    Some(
        constants().k_ice * (t_melt.max(Dy::ONE) / t_surf.min(t_melt).max(Dy::ONE)).log()
            / q
            / Dy::int(1000),
    )
}

/// Every medium on the world a chemistry could run in, with its temperature
/// and the free energy reaching it.
pub(super) fn solvents(w: &State, ctx: &Ctx) -> Vec<(Solvent, Where, Dy, Dy, Dy, bool, Dy)> {
    let c = constants();
    let chem = ctx.sc.chem;
    let mut out = Vec::new();
    let (t, p) = (w.t_surf, w.p_bar.max(Dy::sci(1, -9)));
    let light = w.light * band(w.star_t, &ctx.sc).drive;
    // radiolysis: chemical energy from particles reaching the surface (hydro.rs)
    let rad = hydro::radiolysis(w);
    for (s, amount) in w.seas.iter() {
        let ph = ctx.phase(s, t, p);
        if ph == Phase::Liquid || ph == Phase::Supercritical {
            // a sea partly frozen at its melting point counts only its open water
            let open = Dy::ONE - w.frozen.amount(s);
            out.push((
                Solvent::Volatile(s),
                Where::Surface,
                t,
                (light + free_heat(w, t) + rad) * open,
                amount * open,
                false,
                rad,
            ));
        } else if ph == Phase::Solid && matches!(s, Species::H2O | Species::NH3) && !w.giant {
            // a subsurface ocean exists where the conducted shell thickness at
            // the world's heat flow (tides included) is below the ice layer's
            let t_under = ctx.diagram(s).tt
                + if s == Species::H2O {
                    Dy::int(5) * chem
                } else {
                    Dy::ZERO
                };
            let layer_km = amount * c.bar
                / (c.unit_g * w.g.max(Dy::sci(1, -6)) * Dy::int(1000))
                / Dy::int(1000);
            let q = if w.q.signum() > 0 {
                w.q
            } else {
                geothermal(w) * Dy::int(340)
            };
            if shell_km(q, t, t_under).is_some_and(|d| d < layer_km) {
                out.push((
                    Solvent::Volatile(s),
                    Where::UnderIce,
                    t_under,
                    free_heat(w, t_under) + rad,
                    amount,
                    true,
                    rad,
                ));
            }
        }
    }
    for (s, pp) in w.air.iter() {
        if ctx.phase(s, t, p) == Phase::Supercritical {
            out.push((
                Solvent::Volatile(s),
                Where::Supercritical,
                t,
                light + free_heat(w, t),
                pp,
                false,
                Dy::ZERO,
            ));
        }
    }
    if !w.giant && t >= c.melt_k * chem {
        out.push((
            Solvent::Melt,
            Where::Surface,
            t,
            light + geothermal(w),
            Dy::ONE,
            false,
            Dy::ZERO,
        ));
    }
    if w.giant && w.mass >= c.giant_plasma_mass {
        let energy = c.giant_flux * (w.mass / Dy::int(318)) * (Dy::int(11) / w.radius.max(Dy::ONE));
        out.push((
            Solvent::Plasma,
            Where::Deep,
            Dy::sci(1, 4) * chem,
            energy,
            Dy::ONE,
            true,
            Dy::ZERO,
        ));
    }
    out
}

/// The middle of a solvent's own liquid range (K).
pub fn mid(s: Solvent, ctx: &Ctx) -> Dy {
    let chem = ctx.sc.chem;
    match s {
        Solvent::Melt => Dy::int(2000) * chem,
        Solvent::Plasma => Dy::sci(1, 4) * chem,
        Solvent::Volatile(v) => {
            let d = ctx.diagram(v);
            // a solvent that sublimes at a bar is liquid only under pressure
            let top = if d.tb > d.tt { d.tb } else { d.tc };
            (d.tt + top.min(d.tc)) * Dy::ratio(1, 2)
        }
    }
}

/// How fast a chemistry runs, relative to the unit temperature (Eyring, the
/// activation energy scaled to its solvent's own range).
pub fn pace(s: Solvent, t: Dy, ctx: &Ctx) -> Dy {
    let m = mid(s, ctx);
    let mut t = t;
    if let Solvent::Volatile(v) = s {
        let d = ctx.diagram(v);
        if t > d.tc {
            // a supercritical fluid runs at its own temperature
            return t / constants().t_unit;
        }
        t = t.max(d.tt);
    }
    (t / constants().t_unit) * (constants().solvent_barrier * (Dy::ONE - m / t.max(Dy::ONE))).exp()
}

/// A minded body keeps itself where its chemistry works best: the middle of
/// its solvent's range; its pace there is that temperature over the unit's.
pub fn body_temperature(s: Solvent, ctx: &Ctx) -> Dy {
    mid(s, ctx)
}

/// The pace of a minded body's chemistry: its body temperature (see
/// [`body_temperature`]) over the unit temperature.
pub fn body_pace(s: Solvent, ctx: &Ctx) -> Dy {
    mid(s, ctx) / constants().t_unit
}

/// The region of chemistry space a medium falls in: plasma is plasmic; a
/// melt, a supercritical fluid or a medium above 450 K (scaled) is mineral;
/// dark water under ice is umbral; below 230 K (scaled) or in methane, nitrogen
/// or ammonia is cryogenic; otherwise umbral when dark and biotic when lit.
pub fn region(s: Solvent, at: Where, t: Dy, dark: bool, ctx: &Ctx) -> Region {
    let chem = ctx.sc.chem;
    if s == Solvent::Plasma {
        return Region::Plasmic;
    }
    if s == Solvent::Melt || at == Where::Supercritical || t > Dy::int(450) * chem {
        return Region::Mineral;
    }
    if dark && at == Where::UnderIce && s == Solvent::Volatile(Species::H2O) {
        return Region::Umbral;
    }
    if t < Dy::int(230) * chem
        || matches!(
            s,
            Solvent::Volatile(Species::CH4 | Species::N2 | Species::NH3)
        )
    {
        return Region::Cryogenic;
    }
    if dark {
        Region::Umbral
    } else {
        Region::Biotic
    }
}

/// The solvent's cohesion per molecule (eV).
pub fn weak_bond_ev(s: Solvent, ctx: &Ctx) -> Dy {
    match s {
        Solvent::Melt => constants().melt_network_ev * ctx.sc.chem,
        Solvent::Plasma => Dy::ZERO,
        Solvent::Volatile(v) => ctx.diagram(v).l_kj * Dy::int(1000) / constants().faraday,
    }
}

/// How much chemistry a medium allows, 0 to 1: the elements that can join it
/// x strong bonds that outlast a structure (bond energy over kT above ~40) x
/// weak bonds that hold yet flex on the solvent's own cohesion (near ~10 kT).
pub fn richness(w: &State, s: Solvent, t: Dy, z: Dy, ctx: &Ctx) -> Dy {
    let c = constants();
    let t = t.max(Dy::ONE);
    let has = |sp: Species| w.ices.amount(sp).signum() > 0;
    let mut elements = Dy::ONE + z.min(Dy::ONE);
    if has(Species::CO2) || has(Species::CH4) {
        elements += Dy::ONE;
    }
    if has(Species::NH3) || has(Species::N2) || w.air.has(Species::N2) {
        elements += Dy::ONE;
    }
    let share = (elements / Dy::int(4)).clamp(Dy::ZERO, Dy::ONE);
    if s == Solvent::Plasma {
        // a plasma has no bonds; its structure is carried by fields
        return share * c.plasma_window;
    }
    let kt = c.kb_ev * t;
    let strong_ev = if s == Solvent::Melt {
        c.silicate_ev
    } else {
        c.covalent_ev
    } * ctx.sc.chem;
    let strong = Dy::ONE / (Dy::ONE + (-((strong_ev / kt - c.strong_kt) / Dy::int(4))).exp());
    let x = ((weak_bond_ev(s, ctx) / kt).log() - c.weak_kt.log()) / Dy::ratio(3, 5);
    let weak = (-x.abs()).exp();
    share * strong * weak
}

/// Every chemistry the world allows, with its region, richness, energy and pace.
pub fn media(w: &State, z: Dy, ctx: &Ctx) -> Vec<Medium> {
    solvents(w, ctx)
        .into_iter()
        .map(
            |(solvent, at, t, energy, amount, dark, radiolysis)| Medium {
                solvent,
                at,
                t,
                energy,
                amount,
                dark,
                radiolysis,
                region: region(solvent, at, t, dark, ctx),
                richness: richness(w, solvent, t, z, ctx),
                pace: pace(solvent, t, ctx),
                band: living_band(w, solvent, ctx),
                field_ev: if solvent == Solvent::Plasma {
                    plasma_field_ev(w)
                } else {
                    Dy::ZERO
                },
            },
        )
        .collect()
}

/// A giant's interior field energy per carrier (eV): the electron's
/// magnetic moment, 5.788e-5 eV per tesla, in its dynamo's field - its
/// strength (`State::field`, 0 to 1) times the dynamo's unit, 1e-4 T (an
/// open ruler).
pub fn plasma_field_ev(w: &State) -> Dy {
    Dy::sci(5788, -8) * w.field.max(Dy::ZERO) * Dy::sci(1, -4)
}

/// The share of a world's area a solvent lives across (`Medium::band`): 1 on
/// a rocky world; on a giant, the depth share x_hi - x_lo over which the
/// temperature T(x) = T_surf + (T_int - T_surf) x lies in the solvent's own
/// range - a volatile's from its triple point to its critical point, a
/// melt's and a plasma's from where they form to the interior.
pub fn living_band(w: &State, s: Solvent, ctx: &Ctx) -> Dy {
    if !w.giant {
        return Dy::ONE;
    }
    let chem = ctx.sc.chem;
    let (lo, hi) = match s {
        Solvent::Volatile(v) => {
            let d = ctx.diagram(v);
            (d.tt, d.tc)
        }
        Solvent::Melt => (constants().melt_k * chem, w.t_int),
        Solvent::Plasma => (mid(Solvent::Plasma, ctx), w.t_int),
    };
    let span = w.t_int - w.t_surf;
    if span.signum() <= 0 {
        return if w.t_surf >= lo && w.t_surf <= hi {
            Dy::ONE
        } else {
            Dy::ZERO
        };
    }
    let at = |t: Dy| ((t - w.t_surf) / span).clamp(Dy::ZERO, Dy::ONE);
    (at(hi) - at(lo)).max(Dy::ZERO)
}

/// The richest medium (of a region, if asked): richness x energy (capped at 1).
pub fn best(ms: &[Medium], region: Option<Region>) -> Option<&Medium> {
    ms.iter()
        .filter(|m| region.is_none_or(|r| m.region == r))
        .max_by(|a, b| {
            let ka = a.richness * a.energy.min(Dy::ONE);
            let kb = b.richness * b.energy.min(Dy::ONE);
            ka.cmp(&kb)
                .then_with(|| a.solvent.name().cmp(b.solvent.name()))
        })
}
