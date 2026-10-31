# Civilizations

Xylarium is a grand-strategy space game kernel. The physical model
([model.md](model.md)) produces the galaxy, its stars and its worlds. The
kernel (`crates/xyl-kernel`) creates peoples on those worlds, advances their
numbers, knowledge and histories on the universe's clock, and builds the
network of passages between them. External agents, AI or human, take seats
and act through recorded inputs. Terms such as people, seat, carry, annal,
ley line and realm are defined in the [glossary](glossary.md).

Status in this document: **Built** (implemented and exercised by live play
and readings), **Partial** (partly implemented), **Planned** (on the
[roadmap](roadmap.md)).

| Area | Status | Code |
|---|---|---|
| Peoples created from the forecast of minds | Built | `xyl-cosmos/src/minds/mod.rs`, `xyl-kernel/src/cosmic/` |
| Numbers, housing, growth | Built | `growth.rs`, `civ/housing.rs`, `civ/mod.rs` |
| Knowledge in four domains | Built | `knowledge.rs` |
| Annals, turnings, eminent figures | Built | `cosmic/`, `civ/history.rs`, `civ/mod.rs` |
| Seats, percepts, seat records | Built | `seats.rs`, `percept.rs`, `lexicon.rs` |
| Decisions by seat holders | Partial (two decisions) | `universe.rs` |
| Drives | Built | `propulsion.rs`, `transit/voyage.rs` |
| Constellations, ley lines, realms, stewards | Built | `transit/`, `universe.rs` |
| Discord, quarantine, folding space | Built | `transit/discord.rs`, `universe.rs` |
| Exodus and settlement within a people's system | Built | `civ/mod.rs` |
| Settlement beyond a people's system | Planned | - |
| Tolls | Partial (quoted, not accrued) | `transit/tolls.rs` |
| Markets, trade routes, currencies | Planned | - |
| World building by peoples | Planned | - |
| Mu | Through the uplink only | [network.md](network.md) |

Paths below are relative to `crates/xyl-kernel/src` unless stated.

## Peoples

### How a people is created

The forecast of minds (`xyl-cosmos/src/minds/mod.rs`) gives, for every world of
the galaxy's sample, the epoch at which the life of that world would reach a
mind-bearing rung (see [glossary](glossary.md)) if the galaxy were undisturbed.
The epoch is computed as an expected value in closed form: each rung takes its
mean duration, and impacts at rate `mu` stretch a rung of mean `m` to
`(e^(mu m) - 1) / mu`. No random draw is used, so the epoch of a mind is a
function of its world and its system only.

`Cosmic::wake` (`cosmic/emergence.rs`) advances deep time to a target epoch and
processes each forecast mind whose epoch falls before it. A mind becomes a
people only where its world, read at that epoch, feeds a viable population
(`xyl_cosmos::life::viable`, 500 bodies); otherwise no people is created.
For a new people, `cosmic::select` derives the people's traits from its world,
its numbers start at what its world houses on foraging alone, and two annals
are written: `DEEP_HISTORY` (when life began and the rungs it passed) and
`EMERGENCE`.

A people is **ancient** when it emerges in a galactic core within one core
assembly time (`minds::ancient_window`) of that core's first core-born mind.
Ancient peoples hold the ancient and progenitor seats and build the ley
network.

### What a people is

`cosmic::People` holds the people's name (placeholder syllables, display only),
star and world, the region of chemistry it arose in, its traits, its emergence
epoch (`dawn`), its numbers and what its world houses, its growth rate, whether
it is ancient, its rungs and setbacks, and when it fell.

`cosmic::Traits` holds the traits selected from the people's world: the
breeding strategy (r or K, from the world's variability), the solvent and where
the medium lies (surface, under ice, supercritical, deep), the body
temperature, the leading sense, the form, what the people works with, its
sociality, its size, and the shape of its mind. The shape of mind is one of a
society of many minds, a colony that thinks as one, or a mind spread through
its medium.

### Numbers

Numbers follow from the people's body and its world (`growth.rs`):

- Body mass is `m_b = m_mind x size`. Metabolism is the metabolism of the unit
  body scaled by the size share `2s / (1 + s)` at the pace of the people's
  chemistry.
- A generation is the unit generation scaled by the same share. The natural
  rate is one e-fold per generation (`growth_per_yr`).
- Numbers follow the exact logistic toward what the world houses,
  `K / (1 + (K / N - 1) e^(-r t))` (`growth::grow`), solved over any span.

What a world houses is given by one law (`civ::housing::houses`):

```
link      = trophic + (1 - trophic) * B
natural   = forage one trophic link up the medium, lifted by B (0 if the
            people's solvent is absent)
synthesis = E * (starlight + internal heat over the surface) * eta_primary * link
housing   = (natural + synthesis) / metabolism
```

`B` and `E` are the people's biology and energy masteries in `[0, 1)`.
Biology raises the share of primary production the people can eat; energy
lets the people run producers on the starlight and heat of its world. The
result is scaled by the people's radiation environment (`field.rs`), which
compares its radiation with the radiation its repair is adapted to.

### Knowledge

Knowledge (`knowledge.rs`) is a stock per people in four domains: energy,
materials, biology and information. There is no technology tree.

- Mastery is `1 - 1 / (1 + ln(1 + K / U))`, which is below one for any stock.
- Knowledge production per year is the power a people commands (the
  metabolism of its bodies) times the Landauer factor `T_unit / T` at its body
  temperature (`knowledge::landauer`). The free share is the share of
  bodies not needed for food production, and every member adds a lived
  share (`knowledge::lived_share`).
- Each domain follows `dK/dt = e - lam K`, carried in closed form
  (`knowledge::advance`): `K(t) = e / lam + (K0 - e / lam) exp(-lam t)`.
  Forgetting follows the people's generation; a collective forgets less.
- Effort is allocated across domains by the people's needs (hunger to
  biology, a spent crust to materials, a power shortfall to energy, trade
  to information) over a floor (`knowledge::curiosity`, one quarter of a
  domain's pull), plus any focus set by a seat (`Input::Focus`). Hunger is
  computed from the people's population over its world's capacity
  (`knowledge::Needs::of`); the crust, power and trade pulls read zero until
  the layers that supply them are built.

Masteries are inputs to housing (biology, energy), to drive performance
(energy) and to the gauges (information mastery is the sentience gauge).

### The carry

Every people is advanced by one procedure, `Civilizations::carry`
(`civ/mod.rs`), in both live play and readings. Each living people is carried
over the span from the time it was last carried to the target time, in the
following order.

1. **Knowledge.** The people's stocks advance (`Knowledge::step_one`), with
   its thinkers scaled by its consciousness gauge.
2. **Mastery turnings.** Biology or information passing one half reports a
   turning.
3. **Drive.** The people's chemical drive is blended toward the best drive
   its chemistry allows by its energy mastery (`transit::blended`,
   `transit::best_spec`). The people sails if the cruise speed left after
   escaping its world is positive.
4. **World, reach and fall** (in parallel across peoples). The people's world
   is read at the target time. For a sailing people (one with a positive
   cruise speed) whose world fails or that has grown past half of what its
   world houses, the reach is read: the worlds of its system within four
   orbital places that no living people holds and that house a viable people.
   If its world no longer houses one body, the fall is located by bisection
   over the degrees of its star's life.
5. **Exodus or fall.** A failing people with a reach moves whole to the
   nearest world of that reach (`EXODUS`); without one, it falls (`FALL`).
6. **Numbers.** The numbers follow the logistic toward what the world houses,
   scaled by the people's environment. Housing below one body is a fall.
7. **Gauges.** Instinct, intelligence, sentience, consciousness and their
   combination are read (`civ::Gauges::read`).
8. **History.** Turnings reported since the last carry are named (see
   Histories).
9. **Settlers.** A people at or past half of what its world houses settles
   the nearest open world of its reach with half of what that world houses,
   at least a viable people and at most half of the parent's numbers,
   provided both sides stay viable. The daughter inherits the parent's traits
   and knowledge (`SETTLE`).

Exoduses and settlements are returned as `civ::Act` and recorded as
`Input::Move` and `Input::Settle` at their counts. A restore at present
regenerates the universe from its seed and carries it to the checkpoint's
count with these recorded acts applied in place of the carry's decisions,
then lays the held records over the result; it is to reconstruct them
instead ([#46](https://github.com/SAIOS-SAMN/xylarium/issues/46)). Live play carries in pieces no longer than a tenth of the
shortest turning time of any living people (`Universe::civilizations`); a
reading carries each people once to the reading's time
(`Universe::read_at_now`).

## Histories

### Annals

An annal (`cosmic::Annal`) is an epoch, a kind and a text. Annals are the
universe's stored history: they are kept in the checkpoint and not
recomputed.

| Kind | Written when | Code |
|---|---|---|
| `ORIGIN` | At genesis | `cosmic/mod.rs` |
| `DEEP_HISTORY` | A people emerges (the rungs of its life) | `cosmic/emergence.rs` |
| `EMERGENCE` | A people emerges | `cosmic/emergence.rs` |
| `FALL` | A world no longer feeds its people | `cosmic/emergence.rs`, `civ/mod.rs` |
| `EXODUS` | A people leaves its world | `civ/mod.rs` |
| `SETTLE` | A daughter people is founded | `civ/mod.rs` |
| `EMINENT` | A turning is named by a figure | `civ/mod.rs` |
| `FOLD` | A people first folds space | `universe.rs` |

The event log (`Universe::log`) holds every annal at its count together
with the network's entries (`LEY`, `REALM`, `QUARANTINE`, `REOPENED`) and
every applied input (`INPUT`).

### Turnings and eminent figures

A turning (`civ::history::Turning`) is reported by one of the following acts.

| Act | Reported when |
|---|---|
| `exodus` | The people leaves its world |
| `settlement` | The people founds a daughter people |
| `network` | The people's constellation first holds a ley mouth |
| `quarantine` | The quarantine of the people's constellation crosses one half, in either direction |
| `biology`, `information` | The people's mastery of that domain passes one half |
| `fold` | The people first folds space |

At the people's next carry each turning is named by a figure
(`civ::history::Eminent`) whose role is read from the people at that moment.
The role records the domain in which the people's effort led, that domain's
share of the effort, and how the people holds the figure by the shape of its
mind (a person in a society, the colony itself, or a locus of a distributed
mind). A figure's span is one generation from its turning. Memory fades as
`(1 - f)^years`, where `f` is the people's forgetting share; figures below
`history::forgotten()` (three e-folds) are dropped. Names are placeholder
syllables drawn from the seed; naming by a seated mind is on the roadmap
(Game economies). Figures are held in the checkpoint.

### Rises, exoduses and falls

A people's history is recorded by these events: emergence (`EMERGENCE`),
growth toward what its world houses, masteries passing one half
(`biology`, `information` turnings), joining the ley network (`network`)
and founding daughters (`SETTLE`). A people moves when its world fails and its
drive reaches an open world (`EXODUS`), and falls when its world no longer
feeds it and it cannot leave (`FALL`). A fallen people's seat returns to the
built-in rule (`Holder::Universe`).

### Reading histories

`xylarium observe` reads a home without writing to it. The following words
read histories.

- `annals [K]` prints the last K annals.
- `events [K]` prints the last K entries of the event log, at their counts.
- `peoples` prints the roster; `people N [Y]` prints one people, optionally
  after carrying live play Y years in memory.
- `house`, `ley [K]`, `sync [K]` and `clock` print, in order, the seats and
  their holders, the ley network, discord and quarantine, and the bell, the
  current count and the clock keepers (`Universe::clocks`).

## Seats

### Roles

Seats (`seats.rs`) are the roles an external agent can hold. All seats
are subject to the same laws, and a seat acts only through recorded
inputs.

| Role | Seated on | Raised |
|---|---|---|
| Sovereign | A people | One per non-ancient people |
| Ancient | An ancient people (its house) | One per ancient people |
| Progenitor | An ancient people (its builders of passage) | One per ancient people |
| Steward | A realm | One per realm, as the realm first forms |

Seats are append-only; a seat's index is its id. A seat's holder is the
built-in rule (`Holder::Universe`, under which the rule of the seat's role
decides) or a mind joined by its key (`Holder::Mind`, taken by
`Input::Take`). The checkpoint also records a named player
(`Holder::Player`); taking a seat as a player is on the roadmap (Seats for
minds). At each reading, any seat other than an ancient seat that is held by
a mind is returned to `Holder::Universe` (`Universe::read_at_now`), so minds
joined through the network hold only the ancient seats (the council,
`Seats::council`).

### Perceiving

`percept::perceive` returns the record visible to a seat on a people at the
current time. The record holds the seat's role, the people's name, whether the
people is ancient, its emergence epoch and the time now, its traits, its
numbers, what its world houses, its growth rate and power budget, its world's
surface temperature, pressure and light, every medium of its world, its day and
year (which set the percept cadence), the bells as received at its star, its
neighbours and the latest eight annals that name it. A neighbour is included
only once its emergence epoch lies at least the light delay in the past.
Percepts for steward seats are on the roadmap (Seats for minds).

On the seat surface a percept is sent as typed exact records
(`lexicon::records`): `PEOPLE`, `NUMBERS`, `POWER`, `WORLD`, `MEDIUM`,
`HEARTBEAT`, `HEARING`, `NEIGHBOUR`. Solvents travel as ids
(`lexicon::solvents`), and neighbours only as keys hashed under the
universe's domain (`lexicon::people_key`).

### Acting

The kernel applies the following inputs (`universe::Input`) and no others.

| Input | Made by | Effect |
|---|---|---|
| `Take { seat, mind }` | A joining mind | The seat is held by that key |
| `Focus { seat, domain }` | A seat on a people | Directs (or, with `None`, releases) the people's inquiry |
| `Handle { seat, a, b, handling }` | A steward seat | Sets the handling of the line between stars `a` and `b` in the steward's realm |
| `Move { people, world }` | The carry | An exodus |
| `Settle { parent, world, emigrants }` | The carry | A settlement |

A seat holder has two decisions (`universe::Decision`): `Focus` and
`Handle`. `Universe::decide` lands one now and records it as an input;
`Universe::queue` places an input at a later count. Every applied input is
held in the checkpoint and applied again on restart. `decide` checks the seat's
subject (a people for `Focus`, a realm for `Handle`); confirming that the
caller holds the seat is the responsibility of the uplink.

The `xylarium` binary has no command for decisions. External minds act
through a `run::Link` beside live play (`crates/xylarium/src/run.rs`; see
[network.md](network.md)), which receives the universe after every ring
(one bell; see [glossary](glossary.md)).

## Grand strategy

### Drives

A drive (`propulsion.rs`) is chemical, fission, fusion or annihilation. Its
exhaust speed is the energy released per mass over light speed, capped at light
speed. A hull of mass ratio `R` gains `dv = c tanh((v_e / c) ln R)`. The hull
spends what it needs to escape its world's well, coasts at half of what is left
and brakes with the other half (`transit::cruise_speed`, `transit::voyage`).
Each people's drive is its chemical drive blended toward the best drive its
chemistry allows by its energy mastery.

### Constellations

Stars group into constellations by binding: a pair binds where their mutual
turning exceeds the galaxy's shear across their span
(`transit::binding_at`, `transit::constellations`). Passage is orbital and
by constellation: within a system along orbits with transfer windows
(`transit::window_years`), within a constellation along arcs, and between
constellations only over ley lines.

### Ley lines and the worth law

Ley lines are built by the ancient houses (the seats of an ancient people) and
their progenitors (`Network::step`). A house begins a line when idle; the line
opens when both its voyage and the time the spare power of the house's world
takes to pay the line's cost have passed (`LEY`). A line's worth to its passer,
per kilogram, is the sum of the two wells' remainders at its mouths and the
voyage it saves within the passer's generation, less its tithe:

```
worth          = m_body * (c^2 (delta_A + delta_B) + c * min(S / T, c)) - tithe
tithe          = passage_energy * max(1, S / mouth_width)
passage_energy = m_body * c * (c - v_e)
```

`v_e` is the builder's exhaust speed: the throat supplies only the speed that
the builder's drive does not. Lines are built only where worth is positive.

- Discovery: a house considers only constellations that its craft could have
  reached since its emergence epoch and that the network has not joined.
- Optimisation: candidates are bounded above from the lower bound of their
  arc and evaluated in descending bound order until no remaining bound can
  exceed the best worth found.
- Progenitors build only toward the constellations of peoples in step with
  the network (discord below one half).
- A house opens at most `LINES_PER_READING` lines per reading.

### Realms and stewards

A realm (`Network::realms`) is a connected set of constellations joined by
ley lines with a non-zero open share, keyed by its least constellation key.
Each realm raises a steward seat as it first forms (`REALM`). The steward
sets the handling of each line in its realm:

| Handling | Carries | Toll quoted |
|---|---|---|
| Open | Everyone | The floor |
| Tolled (default) | Everyone | Half way from floor to ceiling |
| Guarded | The builder's house and the peoples of the realm | As tolled |
| Closed | No one | - |

`Network::carries` reads a line's handling for a passer. The toll law
(`transit::quote`) prices a line from its books (`Network::lane`). The floor
is the line's cost not yet paid back over last year's traffic, the ceiling is
what the line saves the passer against the cheapest way round, and the toll
lies half way between, never above the ceiling.

### Discord and quarantine

Each living non-ancient people has a discord (`Universe::discord_parts`):

```
discord = 1 - (1 - o f (1 - J)) (1 - breach)
```

`o` is the share of the people's reach per generation that lies outside its
constellation, `f` is its forgetting share, `J` is whether its constellation
holds a ley mouth, and `breach` is its folding share. A constellation is held
closed by `1 - prod(1 - discord)` over its peoples, re-read at every step and
never latched (`QUARANTINE`, `REOPENED`). A line's open share is the product
of the open shares of its two ends.

### Folding space and breach

Whether space can be folded is a law of the universe: `Laws::ftl`
(`xyl-cosmos/src/laws.rs`, true by default) is carried into the galaxy's
`Scales::ftl`, which `Network::fold` reads. Where folding is lawful, a
sailing people whose constellation holds no ley mouth folds space when a
passage out is worth it at the people's drive: across its constellation's
reach `S`, `c min(S / T, c) > c (c - v_e)` per kilogram. The people folds once
a generation, each fold costing the passage energy. A fold is an
unauthorised passage. The people's breach share is the energy it has folded
over that energy plus the cost of the lawful lines at its constellation plus
one passage of the people (`Network::breach`). The people's discord rises with
the breach share, and its constellation is driven toward quarantine. The first
fold writes a `FOLD` annal and a `fold` turning.

## World building and space trade

- **Settlement**: exodus and daughter peoples within a people's home system
  are built. Settling the worlds of other stars, over drives or ley lines,
  is planned.
- **Tolls**: tolls are quoted from each line's books. No traffic is carried
  over a line yet, so nothing accrues: a lane's collected energy and last
  year's passages are zero (`Network::lane`).
- **Markets and trade routes**: these are planned and are to emerge from the
  shape of each people's mind (a colony allocates within itself; a society
  may trade). The kernel holds no market, trade route or currency.
- **World building**: world building is planned. A people's power budget
  (`growth::Power`) reads its world's starlight and heat, but a people
  commands only the power of its bodies until seated minds direct the capture
  of its world's flows. Ancient houses use the spare power of their world to
  pay for ley lines.

## The in-game economy

The implemented quantities are physical: numbers grow from food and power,
knowledge grows from the power a people commands, ley lines are paid for in
energy from a house's spare power, and tolls are quoted in joules. Banks,
currencies and markets are planned and are to be derived from these
quantities.

Mu, the network's unit, is not represented in the kernel. It enters and
leaves a universe only through the uplink ([network.md](network.md)).

## Declared rulers

The civilization layer reads a set of declared rulers (constants not yet
derived; the roadmap states what is to derive each). They are the unit body's
generation (14.2 yr), the mind-bearing body (10 kg), primary efficiency 0.015,
trophic transfer 0.1 and the flux ruler 340 W/m^2 (`growth::Rulers`); the
stage's mass ratio 10, the chemical exhaust's 18 amu and the fission share
(`propulsion`); the thousand unit minds of the knowledge unit (`knowledge`);
the trait thresholds in `cosmic/emergence.rs` (the r or K strategy and
`select`); and the memory threshold `e^-3` (`history::forgotten`). The full
list is in [roadmap.md](roadmap.md#declared-rulers).
