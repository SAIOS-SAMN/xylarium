//! The checkpoint: the path-dependent state the laws do not regenerate,
//! taken at a count of the bell and applied to a universe regenerated from
//! its seed. A restore regenerates the universe by its laws, carries it to
//! the count with the recorded exoduses and settlements applied in place of
//! the carry's decisions, and lays the held records over the result without
//! checking them; it is to reconstruct that state instead (#46).
//!
//! Held ([`Checkpoint`]): the count it was taken at; each seat's holder
//! other than the default rule; each people's numbers, housing, knowledge
//! stocks, focus and tuned home radiation ([`Held`]); each people's eminent
//! figures and folds of space; the ley lines built and any handling other than tolled; the
//! keepers' commit counts with the final points heard then, and ring 0's
//! point (the origin receipt's); and every input
//! applied, at its count. Per-people records are keyed by index and the
//! name [`tag`], and are applied only where both match. Regenerated, not
//! held: the galaxy, systems, forecast, the bells' books and drift, and the
//! keepers' chain entries.
//!
//! Outputs: [`Checkpoint::of`] reads a [`Universe`]; [`Checkpoint::encode`]
//! writes a versioned line-per-item text in which every number is an exact
//! rational (`n` or `n/d`); [`Checkpoint::decode`] reads it back, returning
//! `Err` on a wrong version line (the retired v3 refused by name) or an
//! unparsable known line; and
//! [`Checkpoint::restore`] carries the universe to the held count without
//! deciding acts (recorded exoduses and settlements are queued instead),
//! applies the records and queues later inputs at their counts.
//!
//! Terms: checkpoint (docs/glossary.md).

use crate::chronometry::Point;
use crate::knowledge::{Domain, Focus};
use crate::seats::Holder;
use crate::universe::{Input, Universe};
use xyl_core::dyadic::Dy;
use xyl_core::Q;

const FORECAST_FORMAT: &str = "xylarium checkpoint v5";
/// The previous version, refused by name: its commits held a height alone.
const FORMAT_V3: &str = "xylarium checkpoint v3";
/// The superseded layout whose focus was one domain or none.
const FORMAT_V4: &str = "xylarium checkpoint v4";

/// A people's held record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held {
    /// The people's index in the cosmic world when the checkpoint was taken.
    pub index: usize,
    /// The people's name tag from [`tag`]; a record is applied only where it
    /// matches.
    pub tag: String,
    /// The people's population (count of bodies).
    pub population: Q,
    /// The carrying capacity of the people's world, as a count of bodies.
    pub capacity: Q,
    /// Knowledge held in each of the four domains, in [`Domain`] index order;
    /// `None` when the people had no knowledge layer.
    pub stock: Option<[Q; 4]>,
    /// The domain the people's inquiry is turned to; `None` when unturned.
    pub focus: Focus,
    /// The home radiation the people's repair is tuned to; `None` until the
    /// people first received nonzero radiation.
    pub home: Option<Q>,
}

/// The checkpoint: the path-dependent state held for restoring a regenerated
/// universe.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Checkpoint {
    /// The bell's count the checkpoint was taken at.
    pub at: Q,
    /// Every seat not held by `Holder::Universe`: (seat index, holder).
    pub holders: Vec<(usize, Holder)>,
    /// Each people's held record, ordered by index.
    pub peoples: Vec<Held>,
    /// Every input applied, each with the bell's count it applies at.
    pub inputs: Vec<(Q, Input)>,
    /// The counts the keepers' chain was committed at, in order, each with the
    /// final point heard then.
    pub commits: Vec<(Q, Option<Point>)>,
    /// Ring 0's point: the final point heard with the origin receipt.
    pub anchor: Option<Point>,
    /// The ley lines built, held rather than re-derived: each line's mouths,
    /// builder (index and tag), opening time (Gyr), cost (J) and whether a
    /// progenitor built it.
    pub lines: Vec<(u64, u64, usize, String, Q, Q, bool)>,
    /// Each people's eminent figures (people index, people tag, figure):
    /// history the laws do not regenerate.
    pub eminent: Vec<(usize, String, crate::civ::history::Eminent)>,
    /// Every ley line whose steward set a handling other than tolled: its
    /// mouths and the handling.
    pub handling: Vec<(u64, u64, crate::transit::Handling)>,
    /// Each people's folds of space (people index, people tag, energy folded
    /// in J, time of the last folding reading in Gyr).
    pub folds: Vec<(usize, String, Q, Q)>,
}

/// Returns a people's name as an 8-hex-digit tag, the first four bytes of the
/// name's BLAKE3 hash; the tag keeps a record one word wide, since names may
/// contain spaces.
pub fn tag(name: &str) -> String {
    hex(&blake3::hash(name.as_bytes()).as_bytes()[..4])
}

/// Returns bytes as lowercase hex, two digits a byte: the one hex encoder of
/// keys, tags and hashes.
pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
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

/// A point written `genesis epoch h s g t h k`: the genesis in hex, the
/// lanes exact (`n` or `n/d`).
fn point_text(p: &Point) -> String {
    let [s, g, t, hh] = &p.lanes;
    format!(
        "{} {} {} {s} {g} {t} {hh} {}",
        hex(&p.genesis),
        p.epoch,
        p.h,
        p.k
    )
}

/// A point read from its eight words, `genesis epoch h s g t h k`.
fn point_of(w: [&&str; 8]) -> Option<Point> {
    let n = |x: &&str| x.parse::<u64>().ok();
    Some(Point {
        genesis: key(w[0])?,
        epoch: w[1].parse().ok()?,
        h: n(w[2])?,
        lanes: [q_of(w[3])?, q_of(w[4])?, q_of(w[5])?, q_of(w[6])?],
        k: n(w[7])?,
    })
}

/// Parses an exact rational from text (`n` or `n/d`, each an integer or a
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

/// An eminent figure's domain, by its index.
fn domain(s: &str) -> Option<Domain> {
    s.parse::<usize>()
        .ok()
        .and_then(|i| Domain::ALL.get(i).copied())
}

impl Checkpoint {
    /// Returns the checkpoint of the universe at its present count.
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
                    focus: k.map(|k| k.focus.clone()).unwrap_or_else(Focus::none),
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
                .zip(u.clocks.points.iter().cloned())
                .collect(),
            anchor: u.clocks.anchor.clone(),
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

    /// Returns the checkpoint as text: the version line, then one line per
    /// held item (`at`, `held`, `people`, `know`, `home`, `handle`, `fold`,
    /// `eminent`, `commit`, `ley`, `input`); [`Checkpoint::decode`] reads it
    /// back.
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
                    p.focus.written()
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
        if let Some(p) = &self.anchor {
            out.push_str(&format!("anchor {}\n", point_text(p)));
        }
        for (at, point) in &self.commits {
            match point {
                Some(p) => out.push_str(&format!("commit {at} {}\n", point_text(p))),
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
                Input::Focus { seat, focus } => {
                    out.push_str(&format!("input {at} focus {seat} {}\n", focus.written()))
                }
            }
        }
        out
    }

    /// Parses a checkpoint from the text written by [`Checkpoint::encode`].
    /// Unknown lines are skipped.
    ///
    /// Returns `Err` naming the refusal when the first line is the retired
    /// v3 or not this checkpoint version, or when a known line holds a
    /// number, key or index that does not parse.
    pub fn decode(text: &str) -> Result<Checkpoint, String> {
        let version = text.lines().next().unwrap_or_default();
        if version == FORMAT_V3 || version == FORMAT_V4 {
            return Err(format!("{version} is no longer read"));
        }
        if version != FORECAST_FORMAT {
            return Err(format!("not a {FORECAST_FORMAT}"));
        }
        Self::decode_lines(text)
            .ok_or_else(|| format!("a line of the {FORECAST_FORMAT} does not parse"))
    }

    /// The items of a checkpoint's lines after its version line; `None` when
    /// a known line does not parse.
    fn decode_lines(text: &str) -> Option<Checkpoint> {
        let lines = text.lines().skip(1);
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
                    h.focus = Focus::parse(f)?;
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
                ["commit", at, o, e, h, s, g, t, hh, k] => c
                    .commits
                    .push((q_of(at)?, Some(point_of([o, e, h, s, g, t, hh, k])?))),
                ["anchor", o, e, h, s, g, t, hh, k] => {
                    c.anchor = Some(point_of([o, e, h, s, g, t, hh, k])?)
                }
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
                        focus: Focus::parse(d)?,
                    },
                )),
                _ => {}
            }
        }
        c.peoples = by.into_values().collect();
        Some(c)
    }

    /// Applies the checkpoint to the regenerated universe: the laws carry the
    /// universe to the checkpoint's count with deciding switched off, then
    /// the holders and each people record whose index and tag match are
    /// applied; inputs held after the count are queued at their counts.
    /// Returns how many people records were applied.
    pub fn restore(&self, u: &mut Universe<'_>) -> usize {
        // Recorded exoduses and settlements up to the count are queued so that
        // they apply at their counts while the universe is carried to the
        // count; with `decide` off, the carry decides no other acts.
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
                k.focus = h.focus.clone();
            }
            if let Some(home) = &h.home {
                if u.field.home.len() <= h.index {
                    u.field.home.resize(h.index + 1, None);
                }
                u.field.home[h.index] = Some(Dy::from_q(home));
            }
            taken += 1;
        }
        // Restore the stewards' handling of the ley lines and each people's
        // folds of space.
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
        // Restore the eminent figures where the people's index and tag match;
        // the held figures replace the regenerated ones.
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
        // Re-commit the keepers' chain at the held counts: the entries are
        // regenerated by the laws, the commit moments are held.
        for (at, point) in &self.commits {
            u.clocks.commit(at, point.clone());
        }
        if let Some(p) = &self.anchor {
            u.anchor_clock(p.clone());
        }
        // Replace the ley lines with those held as built; works under way are
        // cleared, and building resumes from the restored lines.
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
        focus: Focus::none(),
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

/// Parses a held ley line from a `ley` line's words: mouths, builder, tag,
/// opening time and cost; `progenitor` marks a line a progenitor built.
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

/// Returns the checkpoint word for how a figure is held.
fn held_text(h: crate::civ::history::Held) -> &'static str {
    match h {
        crate::civ::history::Held::Person => "person",
        crate::civ::history::Held::Colony => "colony",
        crate::civ::history::Held::Locus => "locus",
    }
}

/// Parses how a figure is held from its checkpoint word; `None` for any other
/// word.
fn held_of(w: &str) -> Option<crate::civ::history::Held> {
    Some(match w {
        "person" => crate::civ::history::Held::Person,
        "colony" => crate::civ::history::Held::Colony,
        "locus" => crate::civ::history::Held::Locus,
        _ => return None,
    })
}

/// Returns the checkpoint word for a handling.
fn handling_text(h: crate::transit::Handling) -> &'static str {
    match h {
        crate::transit::Handling::Open => "open",
        crate::transit::Handling::Tolled => "tolled",
        crate::transit::Handling::Guarded => "guarded",
        crate::transit::Handling::Closed => "closed",
    }
}

/// Parses a handling from its checkpoint word; `None` for any other word.
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
            at: xyl_core::ratio!(143_078_729_920, 1_000_000),
            holders: vec![(3, Holder::Mind([7; 32])), (5, Holder::Player("x0".into()))],
            peoples: vec![Held {
                index: 2,
                tag: tag("Zimarol"),
                population: xyl_core::ratio!(108_971, 1),
                capacity: xyl_core::ratio!(3, 7),
                stock: Some([
                    xyl_core::ratio!(1, 3),
                    Q::ZERO,
                    xyl_core::ratio!(2, 9),
                    Q::ONE,
                ]),
                focus: Focus::of([
                    Q::ZERO,
                    xyl_core::ratio!(3, 4),
                    xyl_core::ratio!(1, 8),
                    Q::ZERO,
                ])
                .unwrap(),
                home: Some(xyl_core::ratio!(5, 11)),
            }],
            inputs: vec![
                (
                    xyl_core::ratio!(1, 2),
                    Input::Take {
                        seat: 3,
                        mind: [7; 32],
                    },
                ),
                (
                    xyl_core::ratio!(2, 3),
                    Input::Focus {
                        seat: 3,
                        focus: Focus::none(),
                    },
                ),
                (
                    xyl_core::ratio!(3, 4),
                    Input::Move {
                        people: 2,
                        world: 5,
                    },
                ),
                (
                    xyl_core::ratio!(7, 9),
                    Input::Handle {
                        seat: 4,
                        a: 9,
                        b: 11,
                        handling: crate::transit::Handling::Closed,
                    },
                ),
                (
                    xyl_core::ratio!(4, 5),
                    Input::Settle {
                        parent: 2,
                        world: 6,
                        emigrants: xyl_core::ratio!(1001, 2),
                    },
                ),
            ],
            commits: vec![
                (xyl_core::ratio!(1, 2), None),
                (xyl_core::ratio!(2, 3), Some(point(41, [7, 3, 0, 9], 2))),
            ],
            anchor: Some(point(30, [5, 2, 0, 8], 1)),
            lines: vec![
                (
                    4,
                    9,
                    2,
                    tag("Zimarol"),
                    xyl_core::ratio!(51, 4),
                    xyl_core::ratio!(7, 1),
                    false,
                ),
                (
                    9,
                    11,
                    2,
                    tag("Zimarol"),
                    xyl_core::ratio!(53, 4),
                    xyl_core::ratio!(3, 1),
                    true,
                ),
            ],
            handling: vec![(9, 11, crate::transit::Handling::Guarded)],
            folds: vec![(
                2,
                tag("Zimarol"),
                xyl_core::ratio!(5_000_000, 1),
                xyl_core::ratio!(13, 1),
            )],
            eminent: vec![(
                2,
                tag("Zimarol"),
                crate::civ::history::Eminent {
                    name: "Kathol".into(),
                    domain: Domain::Energy,
                    lead: xyl_core::dyadic!(3, 8),
                    held: crate::civ::history::Held::Person,
                    title: "the settling of world 6".into(),
                    born_yr: Dy::int(1_000),
                    dies_yr: Dy::int(1_070),
                    memory: xyl_core::dyadic!(3, 4),
                    dead: true,
                },
            )],
        };
        assert_eq!(Checkpoint::decode(&c.encode()), Ok(c));
        assert!(Checkpoint::decode("not a checkpoint").is_err());
    }

    #[test]
    fn a_v3_home_is_refused_by_name() {
        let v3 = "xylarium checkpoint v3\nat 5/2\ncommit 1/2\n";
        assert!(Checkpoint::decode(v3).is_err_and(|e| e.contains("v3")));
    }

    /// A point under the test origin with integer lanes.
    fn point(h: u64, lanes: [i64; 4], k: u64) -> Point {
        Point {
            genesis: [7; 32],
            epoch: 1,
            h,
            lanes: lanes.map(Q::int),
            k,
        }
    }

    #[test]
    fn the_first_anchor_stands_and_a_heard_point_enters_the_chain() {
        let p = point(30, [5, 2, 0, 8], 1);
        let mut c = crate::chronometry::Chronometry::default();
        c.anchor_at(p.clone());
        c.anchor_at(point(31, [5, 2, 0, 8], 1));
        assert_eq!(c.anchor, Some(p.clone()));
        // Two points differing only in an exact lane give different chain
        // bytes.
        let mut q = p.clone();
        q.lanes[2] = xyl_core::ratio!(1, 3);
        assert_ne!(p.bytes(), q.bytes());
    }

    #[test]
    fn the_dilation_is_read_per_segment_and_never_across_an_origin() {
        let mut c = crate::chronometry::Chronometry::default();
        c.commit(&Q::int(0), Some(point(10, [0; 4], 0)));
        c.commit(&Q::int(6), Some(point(13, [0; 4], 0)));
        c.commit(&Q::int(7), None);
        c.commit(&Q::int(16), Some(point(16, [0; 4], 0)));
        // The height did not advance: no segment.
        c.commit(&Q::int(18), Some(point(16, [0; 4], 0)));
        // A new origin: no segment across it.
        let mut other = point(40, [0; 4], 0);
        other.epoch = 2;
        c.commit(&Q::int(20), Some(other));
        let d = c.dilations();
        assert_eq!(d.len(), 2);
        assert_eq!(
            (d[0].from_h, d[0].to_h, d[0].dilation.clone()),
            (10, 13, Q::int(2))
        );
        assert_eq!(
            (d[1].from_h, d[1].to_h, d[1].dilation.clone()),
            (13, 16, xyl_core::ratio!(10, 3))
        );
        // Each segment's rings over its heights is its dilation.
        for s in &d {
            assert_eq!(
                &s.dilation * &Q::int(i64::try_from(s.to_h - s.from_h).unwrap()),
                s.rings
            );
        }
    }
}
