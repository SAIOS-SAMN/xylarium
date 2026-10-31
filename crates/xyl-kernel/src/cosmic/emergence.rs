//! Emergence: forecast minds arising as peoples on their worlds
//! ([`Cosmic::wake`]), the traits a world selects ([`select`]), peoples'
//! names, and daughter peoples ([`Cosmic::settle`]).

use super::{Annal, Cosmic, People, Shape, Strategy, Traits, SYLLABLES};
use xyl_core::dyadic::Dy;
use xyl_core::Stream;
use xyl_cosmos::life::{foragers, viable};
use xyl_cosmos::minds::{ancient_window, in_core, Mind};
use xyl_cosmos::worlds::{best, body_temperature, Ctx, Medium, Region, Solvent, State, Where};

impl Cosmic<'_> {
    /// Advances deep time to `to` and wakes the minds the forecast schedules by
    /// then. Peoples' numbers are left to the caller (universe.rs, via
    /// civ). Returns the annal lines of what woke.
    pub fn wake(&mut self, to: Dy) -> Vec<String> {
        if to > self.now {
            self.now = to;
        }
        self.prepare(self.now);
        let mut lines = Vec::new();
        while let Some(m) = self.forecast.minds.get(self.next).cloned() {
            if m.epoch > self.now {
                break;
            }
            self.next += 1;
            if let Some(line) = self.arise(&m) {
                lines.push(line);
            }
        }
        lines
    }

    fn name(&mut self, m: &Mind) -> String {
        self.name_for(m.star, m.w)
    }

    /// A new people's name, drawn from the seed and its star and world, and
    /// unique among the universe's peoples.
    fn name_for(&mut self, star: u64, w: usize) -> String {
        let mut rng = Stream::derive(
            self.gal.seed,
            &["name".into(), star.into(), (w as u64).into()],
        );
        loop {
            let count = 2 + rng.below(2).unwrap_or(0);
            let mut s = String::new();
            for _ in 0..count {
                let i =
                    usize::try_from(rng.below(SYLLABLES.len() as u64).unwrap_or(0)).unwrap_or(0);
                s.push_str(SYLLABLES[i]);
            }
            let mut c = s.chars();
            let name = c
                .next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default();
            if !self.names.contains(&name) {
                self.names.push(name.clone());
                return name;
            }
        }
    }

    /// Founds a daughter people of `parent` on world `w` of its star at `at`
    /// Gyr, with `emigrants` bodies taken from the parent: the same traits,
    /// lineage and ancient standing, its region read from the world's medium
    /// for those traits, its name drawn for its world. Returns its index, or
    /// `None` when the parent does not exist, the world cannot be read, or
    /// the emigrants are not positive or exceed the parent's numbers.
    pub fn settle(&mut self, parent: usize, w: usize, emigrants: Dy, at: Dy) -> Option<usize> {
        let p = self.peoples.get(parent)?.clone();
        if emigrants.signum() <= 0 || emigrants > p.population {
            return None;
        }
        let (_, ms, _) = self.world_at(p.star, w, at)?;
        let region = p.traits.medium(&ms).map(|m| m.region).unwrap_or(p.region);
        let name = self.name_for(p.star, w);
        if let Some(q) = self.peoples.get_mut(parent) {
            q.population -= emigrants;
        }
        self.peoples.push(People {
            name,
            star: p.star,
            world: w,
            region,
            traits: p.traits.clone(),
            dawn: at,
            population: emigrants,
            capacity: emigrants,
            rate: p.rate,
            updated: at,
            ancient: p.ancient,
            steps: p.steps.clone(),
            setbacks: p.setbacks,
            fell: None,
        });
        Some(self.peoples.len() - 1)
    }

    /// Wakes mind `m` on the world its lineage climbed on, witnessing only
    /// that star's system. The mind is counted silent where the world is
    /// already held or feeds fewer foragers than a viable people.
    fn arise(&mut self, m: &Mind) -> Option<String> {
        self.witness(m.star)?;
        if self
            .peoples
            .iter()
            .any(|p| p.star == m.star && p.world == m.w)
        {
            self.silent += 1;
            return None;
        }
        let (state, ms, body) = self.world_at(m.star, m.w, m.epoch)?;
        let fed = foragers(&ms, m.region, &body, &self.ctx);
        if fed < viable() {
            // Fewer foragers than a viable people: the world cannot feed one now.
            self.silent += 1;
            return None;
        }
        let strategy = if body.tilt > 23 || body.star_mass < Dy::ratio(1, 2) {
            Strategy::R
        } else {
            Strategy::K
        };
        let mut rng = Stream::derive(
            self.gal.seed,
            &["body".into(), m.star.into(), (m.w as u64).into()],
        );
        let traits = select(&state, &ms, m.region, strategy, &mut rng, &self.ctx)?;
        // Numbers start at the world's capacity for bodies of these traits
        // (growth.rs), with no knowledge yet: foraging alone
        // (civ/housing.rs `houses` with zero biology and energy mastery).
        let rulers = crate::growth::Rulers::default();
        let capacity = crate::civ::housing::houses(
            &traits,
            &state,
            &ms,
            &body,
            &self.ctx,
            &rulers,
            Dy::ZERO,
            Dy::ZERO,
        );
        let rate = crate::growth::growth_per_yr(&traits, &rulers);
        let name = self.name(m);
        // Ancient is measured against the first core-born mind of the
        // mind's galactic core (T1), not of the universe core.
        let ancient = in_core(m)
            && xyl_cosmos::minds::first_core_dawn(
                &self.forecast,
                xyl_cosmos::galaxy::core_of(m.star),
            )
            .is_some_and(|d| m.epoch - d <= ancient_window(&self.gal));
        if let Some(text) = deep_history_text(m, &name) {
            self.annals.push(Annal {
                epoch: m.epoch,
                kind: "DEEP_HISTORY",
                text,
            });
        }
        let line = emergence_text(m, &name, ancient, &traits);
        self.annals.push(Annal {
            epoch: m.epoch,
            kind: "EMERGENCE",
            text: line.clone(),
        });
        if capacity.signum() <= 0 {
            self.annals.push(Annal {
                epoch: m.epoch,
                kind: "FALL",
                text: format!("{name}'s world cannot feed it; it falls at its dawn"),
            });
        }
        self.peoples.push(People {
            name,
            star: m.star,
            world: m.w,
            region: m.region,
            traits,
            dawn: m.epoch,
            population: capacity,
            capacity,
            rate,
            updated: m.epoch,
            ancient,
            steps: m.steps.clone(),
            setbacks: m.setbacks,
            fell: (capacity.signum() <= 0).then_some(m.epoch),
        });
        Some(line)
    }
}

/// The DEEP_HISTORY annal text for mind `m` of people `name`: when its life
/// began and the rungs it climbed; None for a mind with no rungs.
fn deep_history_text(m: &Mind, name: &str) -> Option<String> {
    let first = *m.steps.first()?;
    let setbacks = if m.setbacks > 0 {
        format!(
            ", thrown back {} times by the impacts of its sky",
            m.setbacks
        )
    } else {
        String::new()
    };
    Some(format!(
        "Life began on world {} of star {} {} Gyr before the {name}; it passed {} hard steps{}",
        m.w,
        m.star,
        m.epoch - first,
        m.steps.len(),
        setbacks
    ))
}

/// The EMERGENCE annal text for mind `m` arising as people `name`.
fn emergence_text(m: &Mind, name: &str, ancient: bool, traits: &Traits) -> String {
    let at = match traits.at {
        Where::Surface => "surface",
        Where::UnderIce => "sea under ice",
        Where::Supercritical => "supercritical fluid",
        Where::Deep => "deep",
    };
    format!(
        concat!(
            "{}The {} look up: on world {} of star {}, {} kpc from the galactic centre, ",
            "in {}{} {}, a people became aware of itself - {}"
        ),
        if ancient { "[ANCIENT] " } else { "" },
        name,
        m.w,
        m.star,
        m.r_kpc,
        if traits.dark { "a dark " } else { "" },
        traits.solvent.name(),
        at,
        traits.describe()
    )
}

/// The traits a world selected in a lineage, read from the world's best
/// medium in `region`, its surface gravity, the `strategy` and draws from
/// `rng`. None when the world has no medium in the region.
pub fn select(
    state: &State,
    ms: &[Medium],
    region: Region,
    strategy: Strategy,
    rng: &mut Stream,
    ctx: &Ctx,
) -> Option<Traits> {
    let med = best(ms, Some(region))?;
    let dark = matches!(med.at, Where::UnderIce | Where::Deep);
    let g = state.g;
    let plasma = med.solvent == Solvent::Plasma;
    let cold_sea = matches!(
        med.solvent,
        Solvent::Volatile(
            xyl_cosmos::worlds::Species::CH4
                | xyl_cosmos::worlds::Species::N2
                | xyl_cosmos::worlds::Species::NH3
        )
    );
    let sense = if plasma {
        "fields"
    } else if dark {
        "heat and pressure"
    } else if cold_sea {
        "chemistry"
    } else if med.t > Dy::int(450) {
        "heat"
    } else {
        "light"
    };
    let form = if plasma {
        "drifting"
    } else if dark || med.at == Where::UnderIce {
        "swimming"
    } else if g > Dy::ratio(3, 2) {
        "low and many-limbed"
    } else if g < Dy::ratio(1, 2) {
        "tall and slender"
    } else {
        "walking"
    };
    let u = Dy::unit(rng.next_u64());
    let social =
        (u + if strategy == Strategy::R {
            Dy::ratio(3, 10)
        } else {
            Dy::ZERO
        } + if med.energy < Dy::ratio(1, 10) {
            Dy::ratio(1, 5)
        } else {
            Dy::ZERO
        } - Dy::ratio(1, 4))
        .clamp(Dy::ZERO, Dy::ONE);
    let shape = if social > Dy::ratio(3, 4) && strategy == Strategy::R {
        Shape::Superorganism
    } else if plasma || (dark && med.pace < Dy::ratio(1, 20)) {
        Shape::Distributed
    } else {
        Shape::Society
    };
    let hands = if plasma {
        "shaped fields"
    } else {
        [
            "grasping limbs",
            "tendrils",
            "mouths and tongues",
            "jointed palps",
        ][usize::try_from(rng.below(4).unwrap_or(0)).unwrap_or(0)]
    };
    // Size relative to a K-strategist on a unit-gravity world:
    // `1.2 / ceiling` with the strategies' size rulers (r 2.5, K 1.2),
    // divided by the world's gravity g (floored at 0.001) except in plasma.
    let ceiling = if strategy == Strategy::R {
        Dy::ratio(5, 2)
    } else {
        Dy::ratio(6, 5)
    };
    let mut size = Dy::ratio(6, 5) / ceiling;
    if !plasma {
        size = size / g.max(Dy::sci(1, -3));
    }
    Some(Traits {
        strategy,
        solvent: med.solvent,
        dark,
        at: med.at,
        body_t: body_temperature(med.solvent, ctx),
        sense,
        form,
        hands,
        social,
        shape,
        size,
    })
}
