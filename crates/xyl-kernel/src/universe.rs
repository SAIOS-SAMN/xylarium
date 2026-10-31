//! The universe in play: one engine that advances from event to event on
//! the bell's count, holding the cosmic world, the civilizations, the field,
//! the seats, the ley network and the keepers' clocks.
//!
//! - Clock: `now` is an exact rational count of the core's rings. [`Bell`]
//!   holds the ring's period in Gyr (`xyl_cosmos::pulse::core_period`, one
//!   ring, converted at the boundary by `circuit::gyr_of_rings` and rounded
//!   down onto a 2^-60 grid); deep time in Gyr is count x period,
//!   converted to `Dy` at the seam with the older layers.
//! - Events: [`Universe::next_event`] is the soonest of the forecast's next
//!   waking, the next pending [`Input`] and the next ley line falling due.
//!   [`Universe::advance_to`] applies each at its count, then takes every
//!   reading at the target: the bells' rings delivered in closed form
//!   (field.rs), each people carried once over the span since it was last
//!   carried (civ/mod.rs), the ley network and quarantine carried, and
//!   fallen peoples' seats released.
//! - Live play (`live`): the civilizations' layer is carried across each
//!   span in pieces no longer than a tenth of the shortest turning (1 / rate
//!   or one generation) of any living people.
//! - Inputs: what no law determines (a seat taken, a focus, a steward's
//!   handling, and the exoduses and settlements the carry decides) is
//!   applied by the kernel ([`Universe::apply`]), logged and recorded with
//!   its count. [`Universe::decide`] lands a seat's [`Decision`] now, or
//!   returns `Err` with the reason it is refused.
//! - State: what the laws give is regenerated from the seed; recorded
//!   inputs and path-dependent state are held in the checkpoint
//!   (checkpoint.rs) and laid back over the regenerated universe.
//!
//! Terms: ring (docs/glossary.md).

use crate::civ::Civilizations;
use crate::cosmic::Cosmic;
use crate::field::Field;
use crate::knowledge::Domain;
use crate::seats::{Holder, On, Seats};
use xyl_core::dyadic::Dy;
use xyl_core::Q;

/// The bell: the core's exact period, used to convert counts to deep time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bell {
    /// The period of one ring of the core (Gyr), rounded down once onto a
    /// 2^-60 grid.
    pub period_gyr: Q,
}

impl Bell {
    /// The bell of the core: the core's period in Gyr, rounded down onto a
    /// 2^-60 grid.
    pub fn of() -> Bell {
        let gyr = xyl_cosmos::circuit::gyr_of_rings(&xyl_cosmos::pulse::core_period().to_q());
        Bell {
            period_gyr: xyl_core::certified::down(&gyr, 60),
        }
    }

    /// Deep time (Gyr since the origin) at a count: count x period.
    pub fn gyr(&self, count: &Q) -> Q {
        count * &self.period_gyr
    }

    /// The count at a deep time `gyr`: gyr / period; zero when the period is
    /// zero.
    pub fn count(&self, gyr: &Q) -> Q {
        gyr.checked_div(&self.period_gyr).unwrap_or(Q::ZERO)
    }

    /// The count at a deep time given as the older layers' `Dy` Gyr (the
    /// seam).
    pub fn count_dy(&self, gyr: Dy) -> Q {
        self.count(&gyr.to_q())
    }

    /// Deep time at a count as the older layers' `Dy` Gyr (the seam).
    pub fn gyr_dy(&self, count: &Q) -> Dy {
        Dy::from_q(&self.gyr(count))
    }
}

/// An input: an action no law determines, applied at its count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    /// A seat taken by a mind, identified by the mind's key.
    Take {
        /// Index of the seat in [`Seats::list`].
        seat: usize,
        /// The taking mind's 32-byte key.
        mind: [u8; 32],
    },
    /// A ruler directing its people's inquiry to a domain (`None` lifts the
    /// direction).
    Focus {
        /// Index of the ruler's seat in [`Seats::list`].
        seat: usize,
        /// The domain the inquiry is directed to; `None` lets it follow the
        /// people's needs.
        domain: Option<Domain>,
    },
    /// A people moved whole to another world of its star (an exodus decided
    /// by the carry, recorded so a restart lays it back at its count).
    Move {
        /// The people's index.
        people: usize,
        /// The world the people moved to.
        world: usize,
    },
    /// A realm's steward sets a ley line's handling.
    Handle {
        /// Index of the steward's seat in [`Seats::list`].
        seat: usize,
        /// One mouth's star.
        a: u64,
        /// The other mouth's star.
        b: u64,
        /// The handling set.
        handling: crate::transit::Handling,
    },
    /// A people founds a daughter people on another world of its star
    /// (decided by the carry, recorded so a restart lays it back at its
    /// count).
    Settle {
        /// The parent's index.
        parent: usize,
        /// The settled world.
        world: usize,
        /// Bodies that left with the settlers.
        emigrants: Q,
    },
}

/// A decision a seat's holder may make: a people's ruler directs its
/// inquiry; a realm's steward sets a ley line's handling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Direct the people's inquiry to a domain, or lift the direction with
    /// `None`.
    Focus(Option<Domain>),
    /// Set the handling of the ley line between stars `a` and `b` (either
    /// way) in the steward's realm.
    Handle {
        /// One mouth's star.
        a: u64,
        /// The other mouth's star.
        b: u64,
        /// The handling to set.
        handling: crate::transit::Handling,
    },
}

/// A log entry: an event and its exact count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Logged {
    /// The bell's count (rings) at which the event happened.
    pub at: Q,
    /// Category tag, such as `"INPUT"` or an annal's or ley network's kind.
    pub kind: &'static str,
    /// Human-readable description.
    pub text: String,
}

/// The universe in play: the cosmic world, the civilizations, the field, the
/// seats, the clocks, the ley network and the event log.
pub struct Universe<'g> {
    /// The galaxy, its systems, peoples, annals and forecast.
    pub cosmic: Cosmic<'g>,
    /// Every people's numbers, knowledge, reports and gauges.
    pub civ: Civilizations,
    /// The bells' pulses and each reached star's books.
    pub field: Field,
    /// The seats minds may take, and their holders.
    pub seats: Seats,
    /// The core's period, converting counts to deep time.
    pub bell: Bell,
    /// Now, as the bell's count.
    pub now: Q,
    /// Inputs waiting for their count, in order.
    pub pending: Vec<(Q, Input)>,
    /// Every input applied, at its count (held in the checkpoint).
    pub inputs: Vec<(Q, Input)>,
    /// The event log, in order.
    pub log: Vec<Logged>,
    /// The keepers, epicenter to rims, and their chain.
    pub clocks: crate::chronometry::Chronometry,
    /// The ley lines built by ancient peoples and the quarantine.
    pub network: crate::transit::Network,
    /// Whether the carry decides exoduses and settlements; false while a
    /// checkpoint is laid back, when the recorded inputs carry them.
    pub decide: bool,
    /// The number of cosmic annals already copied into the log.
    annals_seen: usize,
    /// Deep time the bells' rings have been delivered to (Gyr).
    rung_to: Dy,
}

impl<'g> Universe<'g> {
    /// The universe at the cosmic world's present (the first mind's dawn
    /// after genesis), with its field, seats and keepers; `heavy` is passed to
    /// `Field::of`. The bells' rings are delivered when the universe is
    /// carried (`advance_to`).
    pub fn begin(cosmic: Cosmic<'g>, heavy: Option<&[u64]>) -> Universe<'g> {
        let bell = Bell::of();
        let now = bell.count_dy(cosmic.now);
        let mut u = Universe {
            field: Field::of(&cosmic, heavy),
            seats: Seats::of(&cosmic),
            cosmic,
            civ: Civilizations::default(),
            bell,
            now,
            pending: Vec::new(),
            inputs: Vec::new(),
            log: Vec::new(),
            clocks: crate::chronometry::Chronometry::default(),
            network: crate::transit::Network::default(),
            decide: true,
            annals_seen: 0,
            rung_to: Dy::ZERO,
        };
        u.clocks = crate::chronometry::Chronometry::of(u.cosmic.gal.seed, u.cosmic.gal.origin);
        u.civ.knowledge.sync(u.cosmic.peoples.len());
        u.log_annals();
        u
    }

    /// Commits the present to the keepers' chain, with the last final point's
    /// receipt height received (None when unlinked).
    pub fn commit_clocks(&mut self, point: Option<u64>) {
        let now = self.now.clone();
        self.clocks.commit(&now, point);
    }

    /// Deep time now (Gyr), as the older layers' `Dy`.
    pub fn gyr(&self) -> Dy {
        self.bell.gyr_dy(&self.now)
    }

    /// Whole rings the bell has rung, floor(now) clamped at zero: the time
    /// coordinate that percepts and ledger entries cite. The phase within the
    /// ring stays in `now`.
    pub fn ring(&self) -> u64 {
        u64::try_from(self.now.floor_i128().unwrap_or(0).max(0)).unwrap_or(u64::MAX)
    }

    /// Applies a seat's decision now as an input, applied by the kernel (not
    /// by the mind) and held in the checkpoint like every input. Returns the
    /// annal written, or `Err` with the refusal reason: no such seat, a
    /// handling decision from a seat that is not a steward's, or a focus
    /// decision from a seat on no people. Repeating the current focus changes
    /// nothing and records no input.
    pub fn decide(
        &mut self,
        seat: usize,
        d: Decision,
    ) -> Result<crate::cosmic::Annal, &'static str> {
        let on = self.seats.list.get(seat).ok_or("no such seat")?.on;
        if let Decision::Handle { a, b, handling } = d {
            let On::Realm(_) = on else {
                return Err("the seat is no steward's");
            };
            let input = Input::Handle {
                seat,
                a,
                b,
                handling,
            };
            let text = self.apply(&input);
            self.log.push(Logged {
                at: self.now.clone(),
                kind: "INPUT",
                text: text.clone(),
            });
            self.inputs.push((self.now.clone(), input));
            return Ok(crate::cosmic::Annal {
                epoch: self.gyr(),
                kind: "HANDLE",
                text,
            });
        }
        let On::People(p) = on else {
            return Err("the seat is on no people");
        };
        let Decision::Focus(domain) = d else {
            return Err("no such decision");
        };
        self.civ.knowledge.sync(self.cosmic.peoples.len());
        let same = self
            .civ
            .knowledge
            .of
            .get(p)
            .is_some_and(|s| s.focus == domain);
        let input = Input::Focus { seat, domain };
        let text = if same {
            let name = self
                .cosmic
                .peoples
                .get(p)
                .map(|x| x.name.clone())
                .unwrap_or_default();
            match domain {
                Some(x) => format!("{name} holds its inquiry on {}", x.name()),
                None => format!("{name} lets its inquiry follow its needs"),
            }
        } else {
            let text = self.apply(&input);
            self.log.push(Logged {
                at: self.now.clone(),
                kind: "INPUT",
                text: text.clone(),
            });
            self.inputs.push((self.now.clone(), input));
            text
        };
        Ok(crate::cosmic::Annal {
            epoch: self.gyr(),
            kind: "FOCUS",
            text,
        })
    }

    /// The count of the soonest event among the forecast's next waking, the
    /// first pending input and the next ley line falling due; `None` when
    /// there is none. A pending input may lie at or before now.
    pub fn next_event(&self) -> Option<Q> {
        let wake = self.cosmic.next_wake().map(|g| self.bell.count_dy(g));
        let input = self.pending.first().map(|(at, _)| at.clone());
        let ley = self.network.next_due().map(|g| self.bell.count_dy(g));
        [wake, input, ley].into_iter().flatten().min()
    }

    /// Each people's standing on the network, None for a fallen people:
    /// whether its constellation holds a ley mouth, and whether it is held
    /// closed by half or more.
    fn standing(&self) -> Vec<Option<(bool, bool)>> {
        self.cosmic
            .peoples
            .iter()
            .map(|p| {
                (p.population.signum() > 0).then(|| {
                    (
                        self.network.holds_mouth(p.star),
                        self.network.shut_of(p.star) >= Dy::ratio(1, 2),
                    )
                })
            })
            .collect()
    }

    /// Carries the ley network to now: folds of space are read, the lines
    /// that have fallen due open at their moments, the quarantine is read from
    /// the peoples' discord, realm seats are raised, and the peoples' network
    /// turnings are recorded.
    fn carry_network(&mut self) {
        let at = self.gyr();
        let before = self.standing();
        // Folding space: each living non-ancient people with a cruise speed
        // above zero has its folds since the last read evaluated
        // (transit/discord.rs `Network::fold`).
        let rulers = crate::growth::Rulers::default();
        for (i, parts) in &self.discord_parts() {
            let Some(p) = self.cosmic.peoples.get(*i) else {
                continue;
            };
            if parts.cruise.signum() <= 0 {
                continue;
            }
            let (body, gen) = (
                crate::growth::body_kg(&p.traits, &rulers),
                crate::growth::generation_yr(&p.traits, &rulers),
            );
            let name = p.name.clone();
            let folded = self
                .network
                .fold(&self.cosmic, *i, parts.v_e, body, gen, at);
            if folded.signum() > 0 && self.network.folds.get(i).is_none_or(|(e, _)| *e <= folded) {
                self.cosmic.annals.push(crate::cosmic::Annal {
                    epoch: at,
                    kind: "FOLD",
                    text: format!(
                        concat!(
                            "The {} fold space without a lawful passage ({} J): ",
                            "they ring out of tune"
                        ),
                        name, folded
                    ),
                });
                self.civ
                    .turning(*i, "fold", "the first fold of space".into(), at);
            }
        }
        let discord = self.discord();
        for (epoch, kind, text) in self.network.step(&self.cosmic, &discord, at) {
            self.log.push(Logged {
                at: self.bell.count_dy(epoch),
                kind,
                text,
            });
        }
        // Each realm formed by the lines raises a steward's seat.
        for key in self.network.note_realms() {
            self.log.push(Logged {
                at: self.now.clone(),
                kind: "REALM",
                text: format!(
                    "A realm forms about constellation {key}; its steward's seat is raised"
                ),
            });
        }
        self.seats = self.seats.extend_realms(self.network.realm_keys.len());
        // The network's turnings in each people's history: joining the
        // network ("network") and its constellation closing or reopening
        // ("quarantine").
        for (i, now) in self.standing().into_iter().enumerate() {
            let (Some(Some(was)), Some(now)) = (before.get(i).cloned(), now) else {
                continue;
            };
            if !was.0 && now.0 {
                self.civ
                    .turning(i, "network", "joining the ley network".into(), at);
            }
            if was.1 != now.1 {
                let title = if now.1 {
                    "its constellation's closing"
                } else {
                    "its constellation's reopening"
                };
                self.civ.turning(i, "quarantine", title.into(), at);
            }
        }
    }

    /// Queues an input at its count, after any input already queued at the
    /// same count; an input at or before now is applied at the next advance.
    pub fn queue(&mut self, at: Q, input: Input) {
        let i = self
            .pending
            .iter()
            .position(|(t, _)| t > &at)
            .unwrap_or(self.pending.len());
        self.pending.insert(i, (at, input));
    }

    /// Advances the universe to `to`, event by event: each waking, input and
    /// ley line falling due at its count, then every reading at `to` (the
    /// bells' rings since last delivered, in closed form; peoples' numbers by
    /// their logistic from the last carry, a fall solved as a crossing inside
    /// the span; the ley network; seat releases). With `live`, the
    /// civilizations' layer is carried across each span between events
    /// instead. Returns the entries logged.
    pub fn advance_to(&mut self, to: &Q, live: bool) -> Vec<Logged> {
        let from_log = self.log.len();
        // Systems and worlds for every waking in the span are computed first,
        // in parallel.
        let clock = std::time::Instant::now();
        xyl_core::watch::stage("carry: prepare");
        self.cosmic.prepare(self.bell.gyr_dy(to));
        let prepared = clock.elapsed();
        xyl_core::watch::note(&format!(
            "prepare {prepared:.1?}: {} systems witnessed",
            self.cosmic.systems.len()
        ));
        xyl_core::watch::stage("carry: wakings");
        let (mut events, mut seats) = (0u64, std::time::Duration::ZERO);
        loop {
            let next = self.next_event().filter(|t| t <= to && t > &self.now);
            let target = next.clone().unwrap_or_else(|| to.clone());
            if target > self.now {
                let at = self.bell.gyr_dy(&target);
                if live {
                    let before = self.gyr();
                    self.civilizations(before, at);
                } else {
                    // A waking: the mind arises at the waking's moment;
                    // peoples' numbers and knowledge are carried at the
                    // reading at `to`.
                    self.cosmic.wake(at);
                }
                self.now = target.clone();
                if self
                    .network
                    .next_due()
                    .is_some_and(|d| self.bell.count_dy(d) <= self.now)
                {
                    self.carry_network();
                }
                let t = std::time::Instant::now();
                self.seats = self.seats.extend(&self.cosmic);
                self.civ.knowledge.sync(self.cosmic.peoples.len());
                self.log_annals();
                seats += t.elapsed();
                events += 1;
                if events % 100 == 0 {
                    xyl_core::watch::note(&format!(
                        "carry: {events} events, {} peoples, {:.1?} in",
                        self.cosmic.peoples.len(),
                        clock.elapsed()
                    ));
                }
            }
            self.land_inputs();
            if next.is_none() || &self.now >= to {
                break;
            }
        }
        let woke = clock.elapsed();
        xyl_core::watch::stage("carry: readings");
        self.read_at_now(live);
        // Timing diagnostics: preparation, events (including seat and annal
        // bookkeeping) and readings.
        if events > 0 {
            xyl_core::watch::note(&format!(
                concat!(
                    "carried {} events - prepare {:.1?}, wakings {:.1?} ",
                    "(seats and annals {:.1?}), readings {:.1?}"
                ),
                events,
                prepared,
                woke - prepared,
                seats,
                clock.elapsed() - woke
            ));
        }
        self.log[from_log..].to_vec()
    }

    /// Takes every reading at now: the bells' rings delivered, the peoples
    /// carried (unless `live`, where the carry ran per span), the ley network
    /// carried and the seats of fallen peoples released.
    fn read_at_now(&mut self, live: bool) {
        let at = self.gyr();
        let t = std::time::Instant::now();
        // The bells' rings are delivered first because a people's environment
        // reads them.
        if at > self.rung_to {
            self.field.step(&self.cosmic, self.rung_to, at);
            self.rung_to = at;
        }
        xyl_core::watch::note(&format!(
            "reading: bells' rings delivered in {:.1?}",
            t.elapsed()
        ));
        let t = std::time::Instant::now();
        if !live {
            // Every mind due by now wakes; each people is then carried once
            // over the span since it was last carried (civ/mod.rs `carry`),
            // the same carry the live path uses.
            self.cosmic.wake(at);
            let environment = self.environment(at, None);
            let acts = self
                .civ
                .carry(&mut self.cosmic, at, &environment, self.decide);
            self.record(acts);
        }
        xyl_core::watch::note(&format!("reading: peoples carried in {:.1?}", t.elapsed()));
        let t = std::time::Instant::now();
        self.carry_network();
        xyl_core::watch::note(&format!(
            "reading: ley network ({} constellations, {} lines) in {:.1?}",
            self.network.constellations.len(),
            self.network.lines.len(),
            t.elapsed()
        ));
        // A fallen people's seat returns to the universe, and so does every
        // non-ancient seat a mind holds: network minds hold ancient seats only.
        let fallen: Vec<usize> = self
            .cosmic
            .peoples
            .iter()
            .enumerate()
            .filter(|(_, p)| p.population.signum() <= 0)
            .map(|(i, _)| i)
            .collect();
        for s in &mut self.seats.list {
            let off_council =
                s.role != crate::seats::Role::Ancient && matches!(s.holder, Holder::Mind(_));
            if matches!(s.on, On::People(p) if fallen.contains(&p)) || off_council {
                s.holder = Holder::Universe;
            }
        }
        self.log_annals();
    }

    /// Carries the civilizations' layer (civ/mod.rs) across the span from
    /// `from` to `to` (Gyr) in live play, in pieces no longer than a tenth of
    /// the shortest turning, min(1 / rate, generation), of any living people
    /// and no shorter than 1e-12 Gyr.
    fn civilizations(&mut self, from: Dy, to: Dy) {
        let rulers = crate::growth::Rulers::default();
        let mut at = from;
        while at < to {
            let turning = self
                .cosmic
                .peoples
                .iter()
                .filter(|p| p.population.signum() > 0)
                .map(|p| {
                    (Dy::ONE / p.rate.max(Dy::sci(1, -12)))
                        .min(crate::growth::generation_yr(&p.traits, &rulers))
                })
                .min();
            let piece = turning
                .map(|yr| yr / Dy::int(10) / Dy::sci(1, 9))
                .unwrap_or(to - at)
                .max(Dy::sci(1, -12))
                .min(to - at);
            let next = at + piece;
            if next > self.rung_to {
                self.field.step(&self.cosmic, self.rung_to, next);
                self.rung_to = next;
            }
            self.cosmic.wake(next);
            let environment = self.environment(next, Some(piece * Dy::sci(1, 9)));
            let acts = self
                .civ
                .carry(&mut self.cosmic, next, &environment, self.decide);
            self.record(acts);
            at = next;
        }
    }

    /// Each people's environment at `at` (field.rs): the radiation at the
    /// people's place against the radiation of its home, with repair following
    /// over `span` years or, when `span` is `None`, over the years since the
    /// people was last carried (or since its dawn).
    fn environment(&mut self, at: Dy, span: Option<Dy>) -> Vec<Dy> {
        let rulers = crate::growth::Rulers::default();
        (0..self.cosmic.peoples.len())
            .map(|i| {
                let p = &self.cosmic.peoples[i];
                let years = span.unwrap_or_else(|| {
                    (at - self
                        .civ
                        .carried_to
                        .get(i)
                        .copied()
                        .unwrap_or(Dy::ZERO)
                        .max(p.dawn))
                    .max(Dy::ZERO)
                        * Dy::sci(1, 9)
                });
                let gen = crate::growth::generation_yr(&p.traits, &rulers);
                let radiation = self.field.radiation(&self.cosmic, i, at);
                self.field.environment(i, radiation, years, gen)
            })
            .collect()
    }

    /// Each living non-ancient people's discord and its parts, for peoples
    /// whose star lies in a constellation: drive by energy mastery,
    /// v_e = v_chem + mu (v_best - v_chem); reach per generation,
    /// rho = cruise x generation; share off its constellation,
    /// o = rho / (rho + reach_C); forgetting share f = 1 - z; whether its
    /// constellation holds a ley mouth, J; d = o f (1 - J); breach b from
    /// folded space; discord = 1 - (1 - d)(1 - b).
    pub fn discord_parts(&self) -> std::collections::BTreeMap<usize, crate::transit::Discord> {
        use crate::transit::{
            best_spec, blended, cruise_speed, home_escape, kpc_m, year_s, Discord,
        };
        let rulers = crate::growth::Rulers::default();
        let net = &self.network;
        let mut out = std::collections::BTreeMap::new();
        for (i, step) in crate::sync::living(self) {
            let p = &self.cosmic.peoples[i];
            if p.ancient {
                continue;
            }
            let Some(mouth) = net.reach_of(p.star) else {
                continue;
            };
            let mastery = self
                .civ
                .knowledge
                .of
                .get(i)
                .map(|k| k.mastery(crate::knowledge::Domain::Energy))
                .unwrap_or(Dy::ZERO);
            let first = crate::propulsion::Spec::of(
                crate::propulsion::Drive::Chemical,
                &p.traits,
                &self.cosmic.gal.sc,
                self.cosmic.gal.elements,
            );
            let spec = blended(&first, &best_spec(&self.cosmic, p), mastery);
            let escape = home_escape(&self.cosmic, p);
            let cruise = cruise_speed(&spec, Dy::int(crate::propulsion::MASS_RATIO), escape);
            let reach =
                cruise * crate::growth::generation_yr(&p.traits, &rulers) * year_s() / kpc_m();
            let both = reach + mouth;
            let off = if both.signum() > 0 {
                reach / both
            } else {
                Dy::ZERO
            };
            let forget = Dy::ONE - step.z.unwrap_or(Dy::ONE);
            let joined = net.holds_mouth(p.star);
            let d = if joined { Dy::ZERO } else { off * forget };
            // A people that folds space breaches the network: the breach
            // weighs its folded energy against its constellation's lawful
            // lines and the energy of one passage of the people's drive.
            let body = crate::growth::body_kg(&p.traits, &rulers);
            let breach = net.breach(&self.cosmic, i, crate::transit::passage_j(body, spec.v_e));
            out.insert(
                i,
                Discord {
                    reach_kpc: reach,
                    mouth_kpc: mouth,
                    forget,
                    joined,
                    v_e: spec.v_e,
                    cruise,
                    escape,
                    breach,
                    discord: Dy::ONE - (Dy::ONE - d) * (Dy::ONE - breach),
                },
            );
        }
        out
    }

    /// Each people's discord alone: the share by which its constellation is
    /// held closed.
    pub fn discord(&self) -> std::collections::BTreeMap<usize, Dy> {
        self.discord_parts()
            .into_iter()
            .map(|(i, d)| (i, d.discord))
            .collect()
    }

    /// Applies every pending input whose count is at or before now, logs it
    /// and records it.
    fn land_inputs(&mut self) {
        while self.pending.first().is_some_and(|(at, _)| at <= &self.now) {
            let (at, input) = self.pending.remove(0);
            let text = self.apply(&input);
            self.log.push(Logged {
                at: at.clone(),
                kind: "INPUT",
                text,
            });
            self.inputs.push((at, input));
        }
    }

    /// Applies an input in the kernel (not in the mind) and returns a
    /// description of what it did or why it did nothing.
    pub fn apply(&mut self, input: &Input) -> String {
        match input {
            Input::Take { seat, mind } => match self.seats.take(*seat, Holder::Mind(*mind)) {
                Ok(_) => format!("seat {seat} taken by a mind"),
                Err(e) => format!("seat {seat} not taken: {e}"),
            },
            Input::Focus { seat, domain } => {
                let Some(On::People(p)) = self.seats.list.get(*seat).map(|s| s.on) else {
                    return format!("seat {seat} is on no people");
                };
                self.civ.knowledge.sync(self.cosmic.peoples.len());
                self.civ.knowledge.focus(p, *domain);
                let name = self
                    .cosmic
                    .peoples
                    .get(p)
                    .map(|x| x.name.clone())
                    .unwrap_or_default();
                match domain {
                    Some(d) => format!("{name} turns its inquiry to {}", d.name()),
                    None => format!("{name} lets its inquiry follow its needs"),
                }
            }
            Input::Handle {
                seat,
                a,
                b,
                handling,
            } => {
                let Some(On::Realm(r)) = self.seats.list.get(*seat).map(|s| s.on) else {
                    return format!("seat {seat} is no steward's");
                };
                let Some(&key) = self.network.realm_keys.get(r) else {
                    return format!("realm {r} has not formed");
                };
                let n = self.network.handle(key, *a, *b, *handling);
                format!(
                    "the steward of realm {r} sets the line {a} - {b} to {:?} ({n} changed)",
                    handling
                )
            }
            Input::Move { people, world } => {
                let at = self.gyr();
                self.civ.move_people(&mut self.cosmic, *people, *world, at);
                format!("people {people} moved to world {world}")
            }
            Input::Settle {
                parent,
                world,
                emigrants,
            } => {
                let at = self.gyr();
                match self
                    .civ
                    .settle(&mut self.cosmic, *parent, *world, Dy::from_q(emigrants), at)
                {
                    Some(d) => format!("people {parent} settled world {world} as people {d}"),
                    None => format!("people {parent} could not settle world {world}"),
                }
            }
        }
    }

    /// Records what the carry decided as inputs at their counts (already
    /// applied), so a checkpoint holds them.
    fn record(&mut self, acts: Vec<crate::civ::Act>) {
        for act in acts {
            let (at, input) = match act {
                crate::civ::Act::Move { people, world, at } => (at, Input::Move { people, world }),
                crate::civ::Act::Settle {
                    parent,
                    world,
                    emigrants,
                    at,
                    ..
                } => (
                    at,
                    Input::Settle {
                        parent,
                        world,
                        emigrants: emigrants.to_q(),
                    },
                ),
            };
            self.inputs.push((self.bell.count_dy(at), input));
        }
    }

    /// Copies the cosmic annals written since the last call into the log, each
    /// at the count of its epoch.
    fn log_annals(&mut self) {
        for a in &self.cosmic.annals[self.annals_seen..] {
            self.log.push(Logged {
                at: self.bell.count_dy(a.epoch),
                kind: a.kind,
                text: a.text.clone(),
            });
        }
        self.annals_seen = self.cosmic.annals.len();
    }
}
