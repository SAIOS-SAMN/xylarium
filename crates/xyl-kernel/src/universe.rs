//! The universe in play: one engine that advances from event to event on
//! the bell's count, holding the cosmic world, the civilizations, the field,
//! the seats, the ley network and the keepers' clocks.
//!
//! - Clock: `now` is an exact rational count of the core's rings, the one
//!   present of the universe; the cosmic world reads it as an argument. Deep
//!   time in Gyr is a reading of the count through `circuit::gyr_of_rings`,
//!   converted to `Dy` at the seam with the older layers, and a Gyr epoch
//!   reads back as a count through `circuit::rings_of_gyr`.
//! - Events: [`Universe::next_event`] is the soonest of the forecast's next
//!   waking, the next pending [`Input`] and the next ley line falling due.
//!   [`Universe::advance_to`] applies each at its count, then takes every
//!   reading at the target: the bells' rings delivered in closed form
//!   (field.rs), each people carried once over the span since it was last
//!   carried (civ/mod.rs), the ley network and quarantine carried, and
//!   fallen peoples' seats released.
//! - Live play (`live`): the civilizations' layer is carried once across
//!   each span between two events, in closed form; a people's settling past
//!   the logistic's inflection is an event (`next_event`), so no span
//!   crosses one (#308).
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
use crate::knowledge::Focus;
use crate::seats::{Holder, On, Seats};
use xyl_core::dyadic::Dy;
use xyl_core::Q;
use xyl_cosmos::circuit::{gyr_of_rings, rings_of_gyr};

/// An input: an action no law determines, applied at its count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    /// A seat taken by a mind (by its key) or a player (by name).
    Take {
        /// The seat's key ([`crate::seats::SeatKey`]).
        seat: crate::seats::SeatKey,
        /// Who takes it.
        holder: Holder,
    },
    /// A ruler directing its people's inquiry: a share of free thought per
    /// domain (all zero lifts the direction).
    Focus {
        /// The ruler's seat's key.
        seat: crate::seats::SeatKey,
        /// Who decided: the seat's holder when it was decided (#265).
        by: Holder,
        /// The direction of inquiry (`knowledge::Focus`).
        focus: Focus,
    },
    /// A people moved whole to another world of its star (an exodus decided
    /// by the carry, recorded so a restart lays it back at its count).
    Move {
        /// The people's index.
        people: usize,
        /// The world the people moved to.
        world: usize,
    },
    /// A builder's seat aims its next ley line (#266).
    Aim {
        /// The seat's key: an ancient or a progenitor seat.
        seat: crate::seats::SeatKey,
        /// Who decided.
        by: Holder,
        /// The star whose constellation is aimed at, or `None` to lift it.
        star: Option<u64>,
    },
    /// An ancient seat sets its house's building share (#266).
    Budget {
        /// The ancient seat's key.
        seat: crate::seats::SeatKey,
        /// Who decided.
        by: Holder,
        /// The share of the spare power, in `[0, 1]`.
        share: Q,
    },
    /// A realm's steward sets a ley line's handling.
    Handle {
        /// The steward's seat's key.
        seat: crate::seats::SeatKey,
        /// Who decided: the seat's holder when it was decided (#265).
        by: Holder,
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Direct the people's inquiry by a share of free thought per domain, or
    /// lift the direction with [`Focus::none`].
    Focus(Focus),
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
    /// Aim the next ley line of an ancient seat's house, or of a progenitor
    /// seat's builders, at the constellation of star `star`, or lift the aim
    /// (`None`): the line goes there when the worth law finds it a candidate
    /// of positive worth (#266).
    Aim {
        /// The star whose constellation the next line is to reach.
        star: Option<u64>,
    },
    /// Set the share of an ancient house's spare power, in `[0, 1]`, put to
    /// building its lines (#266).
    Budget {
        /// The share of the spare power.
        share: Q,
    },
}

impl Input {
    /// The seat, the caller and the decision of a decided input (`Focus`,
    /// `Handle`, `Aim`, `Budget`), as [`Universe::decide`] takes them; `None`
    /// for `Take` and for the carry's own acts.
    pub fn decision(&self) -> Option<(crate::seats::SeatKey, Holder, Decision)> {
        match self {
            Input::Focus { seat, by, focus } => {
                Some((*seat, by.clone(), Decision::Focus(focus.clone())))
            }
            Input::Handle {
                seat,
                by,
                a,
                b,
                handling,
            } => Some((
                *seat,
                by.clone(),
                Decision::Handle {
                    a: *a,
                    b: *b,
                    handling: *handling,
                },
            )),
            Input::Aim { seat, by, star } => {
                Some((*seat, by.clone(), Decision::Aim { star: *star }))
            }
            Input::Budget { seat, by, share } => Some((
                *seat,
                by.clone(),
                Decision::Budget {
                    share: share.clone(),
                },
            )),
            _ => None,
        }
    }

    /// The builders' input a decision makes (#266), checked against the
    /// deciding seat's role: an aim from an ancient or a progenitor seat, a
    /// building share in `[0, 1]` from an ancient seat; `Ok(None)` for a
    /// decision that is not a builder's, `Err` naming the refusal.
    pub fn of_build(
        seat: crate::seats::SeatKey,
        by: &Holder,
        role: crate::seats::Role,
        d: &Decision,
    ) -> Result<Option<Input>, &'static str> {
        Ok(match d {
            Decision::Aim { star } => {
                if !matches!(
                    role,
                    crate::seats::Role::Ancient | crate::seats::Role::Progenitor
                ) {
                    return Err("only an ancient or a progenitor seat aims a line");
                }
                Some(Input::Aim {
                    seat,
                    by: by.clone(),
                    star: *star,
                })
            }
            Decision::Budget { share } => {
                if role != crate::seats::Role::Ancient {
                    return Err("only an ancient seat sets its house's building share");
                }
                if share.signum() < 0 || *share > Q::ONE {
                    return Err("a building share lies in [0, 1]");
                }
                Some(Input::Budget {
                    seat,
                    by: by.clone(),
                    share: share.clone(),
                })
            }
            _ => None,
        })
    }
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
        let now = rings_of_gyr(&cosmic.dawn().to_q());
        let mut u = Universe {
            field: Field::of(&cosmic, heavy),
            seats: Seats::of(&cosmic),
            cosmic,
            civ: Civilizations::default(),
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
        u.clocks = crate::chronometry::Chronometry::of(u.cosmic.gal.origin);
        u.civ.knowledge.sync(u.cosmic.peoples.len());
        u.log_annals();
        u
    }

    /// Commits the present to the keepers' chain, with the last final point
    /// heard (None when unlinked).
    pub fn commit_clocks(&mut self, point: Option<crate::chronometry::Point>) {
        let now = self.now.clone();
        self.clocks.commit(&now, point);
    }

    /// Anchors ring 0 at `point`, the final point heard with the universe's
    /// origin receipt; the first anchor stands.
    pub fn anchor_clock(&mut self, point: crate::chronometry::Point) {
        self.clocks.anchor_at(point);
    }

    /// The present on the T lane, in rings of the bell; written by
    /// `xyl_core::coordinate::beat` as whole rings and the ring's phase in
    /// degrees.
    pub fn t_coordinate(&self) -> xyl_core::coordinate::Reading {
        xyl_core::coordinate::Reading::of(xyl_core::coordinate::Lane::T, &self.now)
    }

    /// Deep time now (Gyr), as the older layers' `Dy`.
    pub fn gyr(&self) -> Dy {
        Dy::from_q(&gyr_of_rings(&self.now))
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
    /// caller `by` who does not hold the seat (#265), a handling decision
    /// from a seat that is not a steward's, or a focus decision from a seat
    /// on no people. Repeating the current focus changes
    /// nothing and records no input.
    pub fn decide(
        &mut self,
        seat: crate::seats::SeatKey,
        by: &Holder,
        d: Decision,
    ) -> Result<crate::cosmic::Annal, &'static str> {
        // The kernel's own check (#265), before anything is applied.
        let held = self.seats.held_by(&seat, by)?;
        let (on, role) = (held.on, held.role);
        let by = by.clone();
        // The builders' choices (#266): a progenitor seat aims its builders'
        // next line, an ancient seat its house's and sets the house's share of
        // spare power put to building.
        let built = Input::of_build(seat, &by, role, &d)?;
        if let Some(input) = built {
            let text = self.apply(&input);
            self.log.push(Logged {
                at: self.now.clone(),
                kind: "INPUT",
                text: text.clone(),
            });
            self.inputs.push((self.now.clone(), input));
            return Ok(crate::cosmic::Annal {
                epoch: self.gyr(),
                kind: "BUILD",
                text,
            });
        }
        if let Decision::Handle { a, b, handling } = d {
            let On::Realm(_) = on else {
                return Err("the seat is no steward's");
            };
            let input = Input::Handle {
                seat,
                by,
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
        let Decision::Focus(focus) = d else {
            return Err("no such decision");
        };
        self.civ.knowledge.sync(self.cosmic.peoples.len());
        let same = self
            .civ
            .knowledge
            .of
            .get(p)
            .is_some_and(|s| s.focus == focus);
        let input = Input::Focus {
            seat,
            by,
            focus: focus.clone(),
        };
        let text = if same {
            let name = self
                .cosmic
                .peoples
                .get(p)
                .map(|x| x.name.clone())
                .unwrap_or_default();
            match focus.is_none() {
                false => format!("{name} holds its inquiry on {}", focus.described()),
                true => format!("{name} lets its inquiry follow its needs"),
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

    /// A seat taken by `holder` now, applied and recorded as an input
    /// (#267). Returns the annal written, or `Err` where no seat has that
    /// key or another holder has it.
    pub fn take(
        &mut self,
        seat: crate::seats::SeatKey,
        holder: Holder,
    ) -> Result<crate::cosmic::Annal, String> {
        self.seats.take(&seat, holder.clone())?;
        let input = Input::Take { seat, holder };
        let text = format!("seat {seat} taken");
        self.log.push(Logged {
            at: self.now.clone(),
            kind: "INPUT",
            text: text.clone(),
        });
        self.inputs.push((self.now.clone(), input));
        Ok(crate::cosmic::Annal {
            epoch: self.gyr(),
            kind: "TAKE",
            text,
        })
    }

    /// The count of the soonest event among the forecast's next waking, the
    /// first pending input and the next ley line falling due; `None` when
    /// there is none. A pending input may lie at or before now.
    pub fn next_event(&self) -> Option<Q> {
        let wake = self.cosmic.next_wake().map(|g| rings_of_gyr(&g.to_q()));
        let input = self.pending.first().map(|(at, _)| at.clone());
        let ley = self.network.next_due().map(|g| rings_of_gyr(&g.to_q()));
        let settling = self.decide.then(|| self.next_settling()).flatten();
        [wake, input, ley, settling].into_iter().flatten().min()
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
                        self.network.shut_of(p.star) >= xyl_core::dyadic!(1, 2),
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
                at: rings_of_gyr(&epoch.to_q()),
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
        self.seats = self.seats.extend_realms(&self.network.realm_keys);
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
        self.cosmic.prepare(Dy::from_q(&gyr_of_rings(to)));
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
                let at = Dy::from_q(&gyr_of_rings(&target));
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
                    .is_some_and(|d| rings_of_gyr(&d.to_q()) <= self.now)
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
    /// `from` to `to` (Gyr) in live play, once and in closed form: the bells'
    /// rings delivered, the minds due woken, every people's environment read
    /// and every people carried.
    fn civilizations(&mut self, from: Dy, to: Dy) {
        // No time-stepping loop (AGENTS.md): the span is carried once in
        // closed form; what the carry decides inside a span (a settling past
        // the logistic's inflection) is an event in `next_event`
        // (`Universe::next_settling`), so the span never crosses one (#308).
        if to <= from {
            return;
        }
        if to > self.rung_to {
            self.field.step(&self.cosmic, self.rung_to, to);
            self.rung_to = to;
        }
        self.cosmic.wake(to);
        let environment = self.environment(to, Some((to - from) * Dy::sci(1, 9)));
        let acts = self
            .civ
            .carry(&mut self.cosmic, to, &environment, self.decide);
        self.record(acts);
    }

    /// The soonest moment (rings) a living people's numbers reach the
    /// logistic's inflection, half its capacity, where the carry settles a
    /// daughter (civ/mod.rs `expand`): from the deep time its numbers were
    /// last carried to (`People::updated`), `ln(K / N - 1) / r` years on
    /// (`growth::years_to_inflection`).
    /// `None` where no people is growing toward one.
    fn next_settling(&self) -> Option<Q> {
        self.cosmic
            .peoples
            .iter()
            .enumerate()
            .filter(|(_, p)| p.population.signum() > 0 && p.population * Dy::int(2) < p.capacity)
            .filter_map(|(_, p)| {
                let from = p.updated.max(p.dawn);
                let years = crate::growth::years_to_inflection(p.population, p.capacity, p.rate)?;
                Some(from + years * Dy::sci(1, -9))
            })
            .min()
            .map(|g| rings_of_gyr(&g.to_q()))
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
    /// whose star lies in a constellation: drive by energy mastery
    /// (`civ::drive`), v_e = v_chem + mu (v_best - v_chem); reach per
    /// generation (`transit::reach_kpc`), rho = cruise x generation; share
    /// off its constellation, the deficit share o = rho / (rho + reach_C);
    /// forgetting share f = 1 - z; whether its constellation holds a ley
    /// mouth, J; d = o f (1 - J); breach b from folded space;
    /// discord = 1 - (1 - d)(1 - b).
    pub fn discord_parts(&self) -> std::collections::BTreeMap<usize, crate::transit::Discord> {
        use crate::transit::{reach_kpc, Discord};
        let rulers = crate::growth::Rulers::default();
        let net = &self.network;
        let mut out = std::collections::BTreeMap::new();
        for (i, step) in crate::sync::living(self) {
            let p = &self.cosmic.peoples[i];
            // Stand-in S106: Ancient peoples are skipped from discord.
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
            let (spec, escape, cruise) = crate::civ::drive(&self.cosmic, p, mastery);
            let reach = reach_kpc(
                &spec,
                Dy::int(crate::propulsion::MASS_RATIO),
                escape,
                crate::growth::generation_yr(&p.traits, &rulers),
            );
            let off = xyl_core::shares::deficit_dy(reach, mouth);
            // Stand-in S108: An unknown z reads 1, so the discord is 0.
            let forget = Dy::ONE - step.z.unwrap_or(Dy::ONE);
            let joined = net.holds_mouth(p.star);
            // Stand-in S107: Holding a ley mouth switches discord off.
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
            Input::Take { seat, holder } => match self.seats.take(seat, holder.clone()) {
                Ok(_) => format!("seat {seat} taken by {holder}"),
                Err(e) => format!("seat {seat} not taken: {e}"),
            },
            Input::Focus { seat, by, focus } => {
                let s = match self.seats.held_by(seat, by) {
                    Ok(s) => s,
                    Err(why) => return format!("seat {seat}: a focus from {by} refused: {why}"),
                };
                let On::People(p) = s.on else {
                    return format!("seat {seat} is on no people");
                };
                self.civ.knowledge.sync(self.cosmic.peoples.len());
                self.civ.knowledge.focus(p, focus.clone());
                let name = self
                    .cosmic
                    .peoples
                    .get(p)
                    .map(|x| x.name.clone())
                    .unwrap_or_default();
                match focus.is_none() {
                    false => format!("{name} turns its inquiry to {}", focus.described()),
                    true => format!("{name} lets its inquiry follow its needs"),
                }
            }
            Input::Handle {
                seat,
                by,
                a,
                b,
                handling,
            } => {
                let s = match self.seats.held_by(seat, by) {
                    Ok(s) => s,
                    Err(why) => return format!("seat {seat}: a handling from {by} refused: {why}"),
                };
                let On::Realm(r) = s.on else {
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
            Input::Aim { seat, by, star } => {
                let s = match self.seats.held_by(seat, by) {
                    Ok(s) => s,
                    Err(why) => return format!("seat {seat}: an aim from {by} refused: {why}"),
                };
                let On::People(i) = s.on else {
                    return format!("seat {seat} is on no people");
                };
                let key = (i, s.role == crate::seats::Role::Progenitor);
                match star {
                    Some(k) => {
                        self.network.aims.insert(key, *k);
                        format!("seat {seat} aims its next line at the constellation of star {k}")
                    }
                    None => {
                        self.network.aims.remove(&key);
                        format!("seat {seat} lifts its aim")
                    }
                }
            }
            Input::Budget { seat, by, share } => {
                let s = match self.seats.held_by(seat, by) {
                    Ok(s) => s,
                    Err(why) => return format!("seat {seat}: a budget from {by} refused: {why}"),
                };
                let (On::People(i), crate::seats::Role::Ancient) = (s.on, s.role) else {
                    return format!("seat {seat} is no ancient house's");
                };
                self.network.budgets.insert(i, share.clone());
                format!("seat {seat} puts {share} of its house's spare power to building")
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
            self.inputs.push((rings_of_gyr(&at.to_q()), input));
        }
    }

    /// Copies the cosmic annals written since the last call into the log, each
    /// at the count of its epoch.
    fn log_annals(&mut self) {
        for a in &self.cosmic.annals[self.annals_seen..] {
            self.log.push(Logged {
                at: rings_of_gyr(&a.epoch.to_q()),
                kind: a.kind,
                text: a.text.clone(),
            });
        }
        self.annals_seen = self.cosmic.annals.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seats::{Role, SeatKey};

    /// The builders' decisions (#266) are made only by the seats whose
    /// choices they are: an aim by an ancient or a progenitor seat, a
    /// building share in [0, 1] by an ancient seat; every other decision is
    /// not a builder's.
    #[test]
    fn the_builders_decisions_belong_to_the_builders_seats() {
        let (seat, by) = (SeatKey([2; 32]), Holder::Player("p".into()));
        let aim = Decision::Aim { star: Some(7) };
        for role in [Role::Ancient, Role::Progenitor] {
            assert!(matches!(
                Input::of_build(seat, &by, role, &aim),
                Ok(Some(Input::Aim { star: Some(7), .. }))
            ));
        }
        for role in [Role::Sovereign, Role::Steward] {
            assert!(Input::of_build(seat, &by, role, &aim).is_err());
        }
        let share = |n: i64, d: i64| Decision::Budget {
            share: Q::new(n, d).unwrap(),
        };
        assert!(matches!(
            Input::of_build(seat, &by, Role::Ancient, &share(1, 2)),
            Ok(Some(Input::Budget { .. }))
        ));
        assert!(Input::of_build(seat, &by, Role::Progenitor, &share(1, 2)).is_err());
        assert!(Input::of_build(seat, &by, Role::Ancient, &share(3, 2)).is_err());
        assert!(Input::of_build(seat, &by, Role::Ancient, &share(-1, 2)).is_err());
        assert!(Input::of_build(seat, &by, Role::Ancient, &share(0, 1)).is_ok());
        let focus = Decision::Focus(crate::knowledge::Focus::none());
        assert!(matches!(
            Input::of_build(seat, &by, Role::Sovereign, &focus),
            Ok(None)
        ));
    }
}
