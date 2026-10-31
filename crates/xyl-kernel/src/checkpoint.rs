//! The checkpoint: the path-dependent state the laws do not regenerate,
//! taken at a count of the bell and laid back over a universe regenerated
//! from its seed. Restoring reconstructs that state; it is not a byte-exact
//! replay.
//!
//! Held ([`Checkpoint`]): the count it was taken at; each seat's holder
//! other than the default rule; each people's numbers, housing, knowledge
//! stocks, focus and tuned home radiation ([`Held`]); its eminent figures
//! and folds; the ley lines built and any handling other than tolled; the
//! keepers' commit counts with their receipt heights; and every input
//! applied, at its count. Per-people records are keyed by index and the
//! name [`tag`], and are applied only where both match. Regenerated, not
//! held: the galaxy, systems, forecast, the bells' books and drift, and the
//! keepers' chain entries.
//!
//! Outputs: [`Checkpoint::of`] reads a [`Universe`]; [`Checkpoint::encode`]
//! writes a versioned line-per-item text in which every number is an exact
//! rational (`n` or `n/d`); [`Checkpoint::decode`] reads it back, returning
//! `None` on a wrong version line or an unparsable known line; and
//! [`Checkpoint::restore`] carries the universe to the held count without
//! deciding acts (recorded exoduses and settlements are queued instead),
//! applies the records and queues later inputs at their counts.
//!
//! Terms: checkpoint (docs/glossary.md).

use crate::knowledge::Domain;
use crate::seats::Holder;
use crate::universe::{Input, Universe};
use xyl_core::dyadic::Dy;
use xyl_core::Q;

const FORECAST_FORMAT: &str = "xylarium checkpoint v3";

/// A people's held record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held {
    /// The people's index in the cosmic world when the checkpoint was taken.
    pub index: usize,
    /// The people's name tag from [`tag`]; a record is applied only where it matches.
    pub tag: String,
    /// The people's population (count of bodies).
    pub population: Q,
    /// How many of its bodies the people's world carries (count).
    pub capacity: Q,
    /// Knowledge held in each of the four domains, in [`Domain`] index order; `None` when the
    /// people had no knowledge layer.
    pub stock: Option<[Q; 4]>,
    /// The domain the people's inquiry is turned to; `None` when unturned.
    pub focus: Option<Domain>,
    /// The home radiation the people's repair is tuned to; `None` until it first lived in any.
    pub home: Option<Q>,
}

/// The checkpoint.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Checkpoint {
    /// The bell's count the checkpoint was taken at.
    pub at: Q,
    /// Every seat not held by the universe's own mind: (seat index, holder).
    pub holders: Vec<(usize, Holder)>,
    /// Each people's held record, ordered by index.
    pub peoples: Vec<Held>,
    /// Every input applied, each with the bell's count it applies at.
    pub inputs: Vec<(Q, Input)>,
    /// The counts the keepers' chain was committed at, in order, each with the
    /// final point's receipt height heard then.
    pub commits: Vec<(Q, Option<u64>)>,
    /// The ley lines built, held rather than re-derived: each line's mouths,
    /// builder (index and tag), opening time (Gyr) and cost (J).
    pub lines: Vec<(u64, u64, usize, String, Q, Q, bool)>,
    /// Each people's eminent figures (people index, its tag, the figure):
    /// history the laws do not regenerate.
    pub eminent: Vec<(usize, String, crate::civ::history::Eminent)>,
    /// Every ley line whose steward set a handling other than tolled: its
    /// mouths and the handling.
    pub handling: Vec<(u64, u64, crate::transit::Handling)>,
    /// Each people's folds of space (people index, its tag, energy folded in
    /// J, when its folding was last read in Gyr).
    pub folds: Vec<(usize, String, Q, Q)>,
}

/// A people's name as an 8-hex tag (names may hold spaces).
pub fn tag(name: &str) -> String {
    blake3::hash(name.as_bytes()).as_bytes()[..4]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn hex(k: &[u8; 32]) -> String {
    k.iter().map(|b| format!("{b:02x}")).collect()
}

fn key(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut k = [0u8; 32];
    for (j, b) in k.iter_mut().enumerate() {
        *b = u8::from_str_radix(s.get(2 * j..2 * j + 2)?, 16).ok()?;
    }
    Some(k)
}

/// An exact rational from its text (`n` or `n/d`, each an integer or a
/// decimal).
///
/// Returns `None` when either part does not parse or the denominator is zero.
pub fn q_of(s: &str) -> Option<Q> {
    let dec = |x: &str| xyl_cosmos::elements::decimal(x);
    match s.split_once('/') {
        Some((n, d)) => dec(n)?.checked_div(&dec(d)?),
        None => dec(s),
    }
}

fn domain(s: &str) -> Option<Domain> {
    s.parse::<usize>()
        .ok()
        .and_then(|i| Domain::ALL.get(i).copied())
}

fn domain_text(d: Option<Domain>) -> String {
    d.map(|d| d.index().to_string())
        .unwrap_or_else(|| "-".into())
}

impl Checkpoint {
    /// What the universe holds now.
    pub fn of(u: &Universe<'_>) -> Checkpoint {
        let holders = u
            .seats
            .list
            .iter()
            .enumerate()
            .filter(|(_, s)| s.holder != Holder::Universe)
            .map(|(i, s)| (i, s.holder.clone()))
            .collect();
        let peoples = u
            .cosmic
            .peoples
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let k = u.civ.knowledge.of.get(i);
                Held {
                    index: i,
                    tag: tag(&p.name),
                    population: p.population.to_q(),
                    capacity: p.capacity.to_q(),
                    stock: k.map(|k| k.stock.map(|x| x.to_q())),
                    focus: k.and_then(|k| k.focus),
                    home: u.field.home.get(i).copied().flatten().map(|h| h.to_q()),
                }
            })
            .collect();
        Checkpoint {
            at: u.now.clone(),
            holders,
            peoples,
            inputs: u.inputs.clone(),
            commits: u
                .clocks
                .commits
                .iter()
                .cloned()
                .zip(u.clocks.points.iter().copied())
                .collect(),
            lines: u
                .network
                .lines
                .iter()
                .map(|l| {
                    (
                        l.from,
                        l.to,
                        l.builder,
                        l.tag.clone(),
                        l.built.to_q(),
                        l.cost_j.to_q(),
                        l.progenitor,
                    )
                })
                .collect(),
            handling: u
                .network
                .lines
                .iter()
                .filter(|l| l.handling != crate::transit::Handling::Tolled)
                .map(|l| (l.from, l.to, l.handling))
                .collect(),
            folds: u
                .network
                .folds
                .iter()
                .filter_map(|(&i, &(e, at))| {
                    u.cosmic
                        .peoples
                        .get(i)
                        .map(|p| (i, tag(&p.name), e.to_q(), at.to_q()))
                })
                .collect(),
            eminent: u
                .cosmic
                .peoples
                .iter()
                .enumerate()
                .flat_map(|(i, p)| {
                    u.civ
                        .numbers
                        .get(i)
                        .map(|n| n.eminent.clone())
                        .unwrap_or_default()
                        .into_iter()
                        .map(move |e| (i, tag(&p.name), e))
                })
                .collect(),
        }
    }

    /// As text: the version line, then one line per held item (`at`, `held`,
    /// `people`, `know`, `home`, `commit`, `ley`, `input`); [`Checkpoint::decode`] reads it back.
    pub fn encode(&self) -> String {
        let mut out = format!("{FORECAST_FORMAT}\nat {}\n", self.at);
        for (i, h) in &self.holders {
            match h {
                Holder::Universe => {}
                Holder::Player(n) => out.push_str(&format!("held {i} player {n}\n")),
                Holder::Mind(k) => out.push_str(&format!("held {i} mind {}\n", hex(k))),
            }
        }
        for p in &self.peoples {
            out.push_str(&format!(
                "people {} {} {} {}\n",
                p.index, p.tag, p.population, p.capacity
            ));
            if let Some(s) = &p.stock {
                out.push_str(&format!(
                    "know {} {} {} {} {} {} {}\n",
                    p.index,
                    p.tag,
                    s[0],
                    s[1],
                    s[2],
                    s[3],
                    domain_text(p.focus)
                ));
            }
            if let Some(h) = &p.home {
                out.push_str(&format!("home {} {} {h}\n", p.index, p.tag));
            }
        }
        for (a, b, h) in &self.handling {
            out.push_str(&format!("handle {a} {b} {}\n", handling_text(*h)));
        }
        for (i, t, e, at) in &self.folds {
            out.push_str(&format!("fold {i} {t} {e} {at}\n"));
        }
        for (i, t, e) in &self.eminent {
            out.push_str(&format!(
                "eminent {i} {t} {} {} {} {} {} {} {} {} {}\n",
                e.domain.index(),
                e.lead.to_q(),
                held_text(e.held),
                e.born_yr.to_q(),
                e.dies_yr.to_q(),
                e.memory.to_q(),
                u8::from(e.dead),
                e.name,
                e.title
            ));
        }
        for (at, point) in &self.commits {
            match point {
                Some(h) => out.push_str(&format!("commit {at} {h}\n")),
                None => out.push_str(&format!("commit {at}\n")),
            }
        }
        for (a, b, i, t, at, cost, prog) in &self.lines {
            out.push_str(&format!(
                "ley {a} {b} {i} {t} {at} {cost}{}\n",
                if *prog { " progenitor" } else { "" }
            ));
        }
        for (at, input) in &self.inputs {
            match input {
                Input::Take { seat, mind } => {
                    out.push_str(&format!("input {at} take {seat} {}\n", hex(mind)))
                }
                Input::Move { people, world } => {
                    out.push_str(&format!("input {at} move {people} {world}\n"))
                }
                Input::Handle {
                    seat,
                    a,
                    b,
                    handling,
                } => out.push_str(&format!(
                    "input {at} handle {seat} {a} {b} {}\n",
                    handling_text(*handling)
                )),
                Input::Settle {
                    parent,
                    world,
                    emigrants,
                } => out.push_str(&format!("input {at} settle {parent} {world} {emigrants}\n")),
                Input::Focus { seat, domain } => out.push_str(&format!(
                    "input {at} focus {seat} {}\n",
                    domain_text(*domain)
                )),
            }
        }
        out
    }

    /// From its text, as written by [`Checkpoint::encode`]. Unknown lines are
    /// skipped.
    ///
    /// Returns `None` when the first line is not this checkpoint version, or
    /// when a known line holds a number, key or index that does not parse.
    pub fn decode(text: &str) -> Option<Checkpoint> {
        let mut lines = text.lines();
        if lines.next()? != FORECAST_FORMAT {
            return None;
        }
        let mut c = Checkpoint::default();
        let mut by: std::collections::BTreeMap<usize, Held> = std::collections::BTreeMap::new();
        for l in lines {
            let w: Vec<&str> = l.split_whitespace().collect();
            match w.as_slice() {
                ["at", t] => c.at = q_of(t)?,
                ["held", i, "player", n] => c
                    .holders
                    .push((i.parse().ok()?, Holder::Player((*n).to_string()))),
                ["held", i, "mind", k] => c.holders.push((i.parse().ok()?, Holder::Mind(key(k)?))),
                ["handle", a, b, h] => {
                    c.handling
                        .push((a.parse().ok()?, b.parse().ok()?, handling_of(h)?))
                }
                ["fold", i, t, e, at] => {
                    c.folds
                        .push((i.parse().ok()?, (*t).to_string(), q_of(e)?, q_of(at)?))
                }
                ["eminent", i, t, rest @ ..] if rest.len() >= 8 => {
                    c.eminent
                        .push((i.parse().ok()?, (*t).to_string(), eminent_of(rest)?));
                }
                ["people", i, t, p, cap] => {
                    let i: usize = i.parse().ok()?;
                    let h = by.entry(i).or_insert_with(|| blank(i, t));
                    h.population = q_of(p)?;
                    h.capacity = q_of(cap)?;
                }
                ["know", i, t, a, b, cc, d, f] => {
                    let i: usize = i.parse().ok()?;
                    let h = by.entry(i).or_insert_with(|| blank(i, t));
                    h.stock = Some([q_of(a)?, q_of(b)?, q_of(cc)?, q_of(d)?]);
                    h.focus = domain(f);
                }
                ["home", i, t, x] => {
                    let i: usize = i.parse().ok()?;
                    by.entry(i).or_insert_with(|| blank(i, t)).home = Some(q_of(x)?);
                }
                ["ley", a, b, i, t, at, cost] => {
                    c.lines.push(ley_of([a, b, i, t, at, cost], false)?)
                }
                ["ley", a, b, i, t, at, cost, "progenitor"] => {
                    c.lines.push(ley_of([a, b, i, t, at, cost], true)?)
                }
                ["commit", at] => c.commits.push((q_of(at)?, None)),
                ["commit", at, h] => c.commits.push((q_of(at)?, Some(h.parse().ok()?))),
                ["input", at, "take", seat, k] => c.inputs.push((
                    q_of(at)?,
                    Input::Take {
                        seat: seat.parse().ok()?,
                        mind: key(k)?,
                    },
                )),
                ["input", at, "handle", seat, a, b, h] => c.inputs.push((
                    q_of(at)?,
                    Input::Handle {
                        seat: seat.parse().ok()?,
                        a: a.parse().ok()?,
                        b: b.parse().ok()?,
                        handling: handling_of(h)?,
                    },
                )),
                ["input", at, "move", people, world] => c.inputs.push((
                    q_of(at)?,
                    Input::Move {
                        people: people.parse().ok()?,
                        world: world.parse().ok()?,
                    },
                )),
                ["input", at, "settle", parent, world, n] => c.inputs.push((
                    q_of(at)?,
                    Input::Settle {
                        parent: parent.parse().ok()?,
                        world: world.parse().ok()?,
                        emigrants: q_of(n)?,
                    },
                )),
                ["input", at, "focus", seat, d] => c.inputs.push((
                    q_of(at)?,
                    Input::Focus {
                        seat: seat.parse().ok()?,
                        domain: domain(d),
                    },
                )),
                _ => {}
            }
        }
        c.peoples = by.into_values().collect();
        Some(c)
    }

    /// Lay the checkpoint over the regenerated universe: the universe is
    /// carried to the checkpoint's count by its laws, then each held record is
    /// applied where its people matches; inputs held after the count wait for
    /// their own count. Returns how many peoples' records were applied.
    pub fn restore(&self, u: &mut Universe<'_>) -> usize {
        // exoduses and settlements land at their counts as the universe is
        // carried back; the carry decides none of its own meanwhile
        for (at, input) in &self.inputs {
            if at <= &self.at && matches!(input, Input::Move { .. } | Input::Settle { .. }) {
                u.queue(at.clone(), input.clone());
            }
        }
        u.decide = false;
        u.advance_to(&self.at, false);
        u.decide = true;
        for (i, h) in &self.holders {
            let _ = u.seats.take(*i, h.clone());
        }
        u.civ.knowledge.sync(u.cosmic.peoples.len());
        let mut taken = 0;
        for h in &self.peoples {
            let Some(p) = u.cosmic.peoples.get_mut(h.index) else {
                continue;
            };
            if tag(&p.name) != h.tag {
                continue;
            }
            p.population = Dy::from_q(&h.population);
            p.capacity = Dy::from_q(&h.capacity);
            if let (Some(k), Some(s)) = (u.civ.knowledge.of.get_mut(h.index), &h.stock) {
                k.stock = s.clone().map(|x| Dy::from_q(&x));
                k.focus = h.focus;
            }
            if let Some(home) = &h.home {
                if u.field.home.len() <= h.index {
                    u.field.home.resize(h.index + 1, None);
                }
                u.field.home[h.index] = Some(Dy::from_q(home));
            }
            taken += 1;
        }
        // the stewards' handling of their lines, and each people's folds
        for l in &mut u.network.lines {
            if let Some((_, _, h)) = self
                .handling
                .iter()
                .find(|(a, b, _)| (*a, *b) == (l.from, l.to))
            {
                l.handling = *h;
            }
        }
        for (i, t, e, at) in &self.folds {
            if u.cosmic.peoples.get(*i).is_some_and(|p| tag(&p.name) == *t) {
                u.network.folds.insert(*i, (Dy::from_q(e), Dy::from_q(at)));
            }
        }
        // the eminent figures, where their people matches
        u.civ
            .numbers
            .resize_with(u.cosmic.peoples.len(), Default::default);
        let mut laid: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        for (i, t, e) in &self.eminent {
            let matches = u.cosmic.peoples.get(*i).is_some_and(|p| tag(&p.name) == *t);
            if let (true, Some(n)) = (matches, u.civ.numbers.get_mut(*i)) {
                if laid.insert(*i) {
                    n.eminent.clear();
                }
                n.eminent.push(e.clone());
            }
        }
        // re-commit the keepers' chain at its held counts: the entries come
        // from the laws, the commit moments are held
        for (at, point) in &self.commits {
            u.clocks.commit(at, *point);
        }
        // the ley lines as built; works under way are begun again from them
        if !self.lines.is_empty() {
            u.network.lines = self
                .lines
                .iter()
                .map(|(a, b, i, t, at, cost, prog)| held_ley(*a, *b, *i, t, at, cost, *prog))
                .collect();
            u.network.works.clear();
        }
        u.inputs = self
            .inputs
            .iter()
            .filter(|(at, _)| at <= &self.at)
            .cloned()
            .collect();
        for (at, input) in self.inputs.iter().filter(|(at, _)| at > &self.at) {
            u.queue(at.clone(), input.clone());
        }
        taken
    }
}

/// A held people record with nothing yet read but its index and tag.
fn blank(i: usize, t: &str) -> Held {
    Held {
        index: i,
        tag: t.to_string(),
        population: Q::ZERO,
        capacity: Q::ZERO,
        stock: None,
        focus: None,
        home: None,
    }
}

/// An eminent figure from the words after an `eminent` line's index and
/// tag: domain, lead, held, born, dies, memory, dead, name and title words.
fn eminent_of(w: &[&str]) -> Option<crate::civ::history::Eminent> {
    let [d, lead, held, born, dies, memory, dead, name, title @ ..] = w else {
        return None;
    };
    Some(crate::civ::history::Eminent {
        name: (*name).to_string(),
        domain: domain(d)?,
        lead: Dy::from_q(&q_of(lead)?),
        held: held_of(held)?,
        title: title.join(" "),
        born_yr: Dy::from_q(&q_of(born)?),
        dies_yr: Dy::from_q(&q_of(dies)?),
        memory: Dy::from_q(&q_of(memory)?),
        dead: *dead == "1",
    })
}

/// A held ley line from a `ley` line's words: mouths, builder, tag, opening
/// time and cost.
fn ley_of(w: [&&str; 6], progenitor: bool) -> Option<(u64, u64, usize, String, Q, Q, bool)> {
    let [a, b, i, t, at, cost] = w;
    Some((
        a.parse().ok()?,
        b.parse().ok()?,
        i.parse().ok()?,
        (*t).to_string(),
        q_of(at)?,
        q_of(cost)?,
        progenitor,
    ))
}

fn held_ley(
    from: u64,
    to: u64,
    builder: usize,
    tag: &str,
    built: &Q,
    cost: &Q,
    progenitor: bool,
) -> crate::transit::Ley {
    crate::transit::Ley {
        from,
        to,
        builder,
        tag: tag.to_string(),
        progenitor,
        built: Dy::from_q(built),
        cost_j: Dy::from_q(cost),
        handling: crate::transit::Handling::Tolled,
    }
}

/// How a figure is held, as its checkpoint word.
fn held_text(h: crate::civ::history::Held) -> &'static str {
    match h {
        crate::civ::history::Held::Person => "person",
        crate::civ::history::Held::Colony => "colony",
        crate::civ::history::Held::Locus => "locus",
    }
}

/// How a figure is held, from its checkpoint word; `None` for any other word.
fn held_of(w: &str) -> Option<crate::civ::history::Held> {
    Some(match w {
        "person" => crate::civ::history::Held::Person,
        "colony" => crate::civ::history::Held::Colony,
        "locus" => crate::civ::history::Held::Locus,
        _ => return None,
    })
}

/// A handling as its checkpoint word.
fn handling_text(h: crate::transit::Handling) -> &'static str {
    match h {
        crate::transit::Handling::Open => "open",
        crate::transit::Handling::Tolled => "tolled",
        crate::transit::Handling::Guarded => "guarded",
        crate::transit::Handling::Closed => "closed",
    }
}

/// A handling from its checkpoint word; `None` for any other word.
fn handling_of(w: &str) -> Option<crate::transit::Handling> {
    Some(match w {
        "open" => crate::transit::Handling::Open,
        "tolled" => crate::transit::Handling::Tolled,
        "guarded" => crate::transit::Handling::Guarded,
        "closed" => crate::transit::Handling::Closed,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_text_round_trips_exactly() {
        let c = Checkpoint {
            at: Q::ratio(143_078_729_920, 1_000_000),
            holders: vec![(3, Holder::Mind([7; 32])), (5, Holder::Player("x0".into()))],
            peoples: vec![Held {
                index: 2,
                tag: tag("Zimarol"),
                population: Q::ratio(108_971, 1),
                capacity: Q::ratio(3, 7),
                stock: Some([Q::ratio(1, 3), Q::ZERO, Q::ratio(2, 9), Q::ONE]),
                focus: Some(Domain::Materials),
                home: Some(Q::ratio(5, 11)),
            }],
            inputs: vec![
                (
                    Q::ratio(1, 2),
                    Input::Take {
                        seat: 3,
                        mind: [7; 32],
                    },
                ),
                (
                    Q::ratio(2, 3),
                    Input::Focus {
                        seat: 3,
                        domain: None,
                    },
                ),
                (
                    Q::ratio(3, 4),
                    Input::Move {
                        people: 2,
                        world: 5,
                    },
                ),
                (
                    Q::ratio(7, 9),
                    Input::Handle {
                        seat: 4,
                        a: 9,
                        b: 11,
                        handling: crate::transit::Handling::Closed,
                    },
                ),
                (
                    Q::ratio(4, 5),
                    Input::Settle {
                        parent: 2,
                        world: 6,
                        emigrants: Q::ratio(1001, 2),
                    },
                ),
            ],
            commits: vec![(Q::ratio(1, 2), None), (Q::ratio(2, 3), Some(41))],
            lines: vec![
                (
                    4,
                    9,
                    2,
                    tag("Zimarol"),
                    Q::ratio(51, 4),
                    Q::ratio(7, 1),
                    false,
                ),
                (
                    9,
                    11,
                    2,
                    tag("Zimarol"),
                    Q::ratio(53, 4),
                    Q::ratio(3, 1),
                    true,
                ),
            ],
            handling: vec![(9, 11, crate::transit::Handling::Guarded)],
            folds: vec![(2, tag("Zimarol"), Q::ratio(5_000_000, 1), Q::ratio(13, 1))],
            eminent: vec![(
                2,
                tag("Zimarol"),
                crate::civ::history::Eminent {
                    name: "Kathol".into(),
                    domain: Domain::Energy,
                    lead: Dy::ratio(3, 8),
                    held: crate::civ::history::Held::Person,
                    title: "the settling of world 6".into(),
                    born_yr: Dy::int(1_000),
                    dies_yr: Dy::int(1_070),
                    memory: Dy::ratio(3, 4),
                    dead: true,
                },
            )],
        };
        assert_eq!(Checkpoint::decode(&c.encode()), Some(c));
        assert_eq!(Checkpoint::decode("not a checkpoint"), None);
    }
}
