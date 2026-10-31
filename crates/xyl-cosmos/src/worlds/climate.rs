//! The climate solve: absorbed flux, greenhouse, vapour and weathering
//! balanced on a temperature ladder.

use super::*;

/// A pole's year-round sunlight over the world's mean: 4 sin(obliquity) / pi,
/// with the sine taken from the angle (not sqrt(1 - cos^2)).
pub fn polar_share(cos_obliquity: Dy) -> Dy {
    let sin = cos_obliquity.clamp(-Dy::ONE, Dy::ONE).acos().sin().abs();
    (Dy::int(4) * sin / Dy::PI).min(Dy::ONE)
}

/// What a world's climate is solved from.
pub(super) struct Climate {
    pub(super) to_bar: Dy,
    pub(super) flux: Dy,
    pub(super) hpm: Dy,
    pub(super) giant: bool,
    pub(super) polar: Dy,
}

impl Climate {
    /// The air and seas a world would have at surface temperature `t`, with
    /// `open` the open-water share of its seas' surface (1 unless ice and
    /// water coexist at a melting point). Weathering runs on open water, so
    /// the CO2 left in the air rises as ice spreads (balance / open).
    fn state_open(&self, gas: &Mix, t: Dy, open: Dy, ctx: &Ctx) -> (Mix, Mix) {
        let (mut air, mut seas) = (Mix::default(), Mix::default());
        for (s, f) in gas.iter() {
            if s == Species::CO {
                continue;
            }
            let pp = f * self.to_bar;
            let vap = pp.min(ctx.saturation(s, t));
            if vap.signum() > 0 {
                air.set(s, vap);
            }
            if (pp - vap).signum() > 0 {
                seas.set(s, pp - vap);
            }
        }
        let p = air.total();
        let wet = seas.has(Species::H2O)
            && ctx.phase(Species::H2O, t, p.max(Dy::sci(1, -9))) == Phase::Liquid
            && !self.giant;
        if let (true, Some(co2)) = (wet, air.get(Species::CO2)) {
            // the carbonate-silicate cycle: weathering turns CO2 to carbonate
            // until it balances what the interior outgasses, faster the warmer
            let tt = ctx.diagram(Species::H2O).tt;
            let balance = constants().weather_scale
                * self.hpm
                * self.hpm
                * (-(Dy::int(2) * (t - tt) / (constants().weather_e_k * ctx.sc.chem))).exp();
            air.set(
                Species::CO2,
                co2.min((balance / open.max(Dy::sci(1, -9))).max(Dy::sci(1, -6))),
            );
            seas.remove(Species::CO2);
        }
        (air, seas)
    }

    /// A world's poles are colder by as much as its sky fails to carry heat to
    /// them; a gas whose frost point the pole is below freezes there.
    fn cold_trap(&self, air: &mut Mix, seas: &mut Mix, t: Dy, ctx: &Ctx) {
        let carry = air.total() / constants().transport_bar;
        let t_pole = t * ((self.polar + carry) / (Dy::ONE + carry));
        for s in [Species::CO2, Species::N2, Species::CH4, Species::NH3] {
            if let Some(a) = air.get(s) {
                let cap = ctx.saturation(s, t_pole);
                if a > cap {
                    seas.set(s, seas.amount(s) + a - cap);
                    air.set(s, cap);
                }
            }
        }
    }

    /// The temperature the atmosphere makes at an assumed `t`, and its state.
    fn model(&self, gas: &Mix, t: Dy, ctx: &Ctx) -> (Dy, Mix, Mix, Dy) {
        self.model_mixed(gas, t, None, ctx)
    }

    /// The same at a melting point held by ice and water together: `mixed`
    /// is (the ice's share of the seas' surface, the albedo fully iced).
    fn model_mixed(
        &self,
        gas: &Mix,
        t: Dy,
        mixed: Option<(Dy, Dy)>,
        ctx: &Ctx,
    ) -> (Dy, Mix, Mix, Dy) {
        let open = mixed.map_or(Dy::ONE, |(f, _)| Dy::ONE - f);
        let (mut air, mut seas) = self.state_open(gas, t, open, ctx);
        self.cold_trap(&mut air, &mut seas, t, ctx);
        let icy: Dy = seas
            .iter()
            .filter(|(s, _)| t < ctx.diagram(*s).tt)
            .map(|(_, v)| v)
            .sum();
        let albedo = if icy.signum() > 0 {
            Dy::ratio(3, 10) + Dy::ratio(2, 5) * (icy / (icy + Dy::ONE)).clamp(Dy::ZERO, Dy::ONE)
        } else {
            Dy::ratio(3, 10)
        };
        let albedo = mixed.map_or(albedo, |(f, a_iced)| {
            albedo + f * (a_iced - albedo).max(Dy::ZERO)
        });
        let gh: Dy = air.iter().map(|(s, v)| s.hold() * v).sum();
        // equilibrium temperature linear in absorbed flux; optical depth tau
        // from the absorbing gases, the atomic scale and the column's share of
        // a bar; held share h = tau / (1 + tau) (bounded by 1); the surface is
        // warmer by 1 + 3/4 h
        let t_eq = constants().t_unit * self.flux * (Dy::ONE - albedo);
        let air_total = air.total().max(Dy::sci(1, -9));
        let tau =
            constants().greenhouse_tau * gh * ctx.sc.atom * (air_total / (air_total + Dy::ONE));
        let held = tau / (Dy::ONE + tau);
        (t_eq * (Dy::ONE + Dy::ratio(3, 4) * held), air, seas, albedo)
    }

    /// Surface temperature, pressure, air, seas and albedo: the warmest stable
    /// state at which the temperature the atmosphere makes is the one it was
    /// assumed at.
    pub(super) fn solve(&self, gas: &Mix, ctx: &Ctx) -> (Dy, Dy, Mix, Mix, Dy, Mix) {
        let runaway = constants().runaway_flux * ctx.sc.chem;
        let base = constants().t_unit * self.flux * Dy::ratio(3, 10);
        let step = Dy::ratio(108, 100);
        let mut grid = Vec::with_capacity(40);
        let mut t = base.max(Dy::int(15));
        for _ in 0..40 {
            grid.push(t);
            t = t * step;
        }
        let vals: Vec<Dy> = grid
            .iter()
            .map(|&t| self.model(gas, t, ctx).0 - t)
            .collect();
        let mut root = None;
        let mut frozen = Mix::default();
        let mut pinned: Option<(Dy, Dy)> = None;
        for i in (1..grid.len()).rev() {
            // the warmest crossing from warming to cooling
            if vals[i - 1].signum() >= 0 && vals[i].signum() < 0 {
                let (_, _, seas_i, alb_i) = self.model(gas, grid[i], ctx);
                if gas.amount(Species::H2O).signum() > 0
                    && !seas_i.has(Species::H2O)
                    && self.flux * (Dy::ONE - alb_i) < runaway
                {
                    // below the runaway limit a steam sky rains out
                    continue;
                }
                let (mut lo, mut hi) = (grid[i - 1], grid[i]);
                for _ in 0..16 {
                    let mid = (lo + hi) * Dy::ratio(1, 2);
                    if (self.model(gas, mid, ctx).0 - mid).signum() >= 0 {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                // take the cooling side: at a melting-point pin the world is liquid
                root = Some(hi);
                // Pinned at a melting point: frozen, the world warms; open, it
                // cools, so ice and water coexist. The ice share f changes the
                // albedo and closes the open water weathering runs on (CO2);
                // solve for the f at which the model temperature equals the
                // melting point (0 cools, 1 warms).
                let (_, _, seas_ice, a_iced) = self.model(gas, lo, ctx);
                if let Some(s) = seas_ice.iter().map(|(s, _)| s).find(|&s| {
                    let tt = ctx.diagram(s).tt;
                    lo <= tt && tt <= hi
                }) {
                    let at = |f: Dy| self.model_mixed(gas, hi, Some((f, a_iced)), ctx).0 - hi;
                    if at(Dy::ZERO).signum() < 0 && at(Dy::ONE).signum() >= 0 {
                        let (mut f_lo, mut f_hi) = (Dy::ZERO, Dy::ONE);
                        for _ in 0..24 {
                            let mid = (f_lo + f_hi) * Dy::ratio(1, 2);
                            if at(mid).signum() < 0 {
                                f_lo = mid;
                            } else {
                                f_hi = mid;
                            }
                        }
                        frozen.set(s, f_hi);
                        pinned = Some((f_hi, a_iced));
                    }
                }
                break;
            }
        }
        // no crossing kept: if cooling everywhere, take the coldest state;
        // otherwise the state nearest balance (a steam atmosphere that rains
        // out leaves the world where it warms least, not at the grid's top)
        let root = root.unwrap_or_else(|| {
            if vals[0].signum() < 0 {
                return grid[0];
            }
            grid.iter()
                .zip(&vals)
                .min_by(|a, b| a.1.abs().cmp(&b.1.abs()))
                .map(|(&t, _)| t)
                .unwrap_or(grid[grid.len() - 1])
        });
        let (_, air, seas, albedo) = self.model_mixed(gas, root, pinned, ctx);
        (root, air.total(), air, seas, albedo, frozen)
    }
}
