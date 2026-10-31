# Civilizations

Xylarium is a grand-strategy space game in which sovereign minds operate
civilizations. The physical model ([model.md](model.md)) produces the
galaxy, its stars and its worlds; the kernel (`crates/xyl-kernel`) wakes
peoples on those worlds, carries their numbers, knowledge and histories on
the universe's clock, and builds the network of passages between them.
External minds, AI or human, take seats and act through recorded inputs.

Status in this document: **Built** (in the code and run by live play and
readings), **Partial** (some of it is in the code), **Planned** (on the
[roadmap](roadmap.md)).

| Area | Status | Code |
|---|---|---|
| Peoples waking from the forecast of minds | Built | `xyl-cosmos/src/minds.rs`, `xyl-kernel/src/cosmic.rs` |
| Numbers, housing, growth | Built | `growth.rs`, `civ/housing.rs`, `civ/mod.rs` |
| Knowledge in four domains | Built | `knowledge.rs` |
| Annals, turnings, eminent figures | Built | `cosmic.rs`, `civ/history.rs`, `civ/mod.rs` |
| Seats, percepts, seat records | Built | `seats.rs`, `percept.rs`, `lexicon.rs` |
| Decisions by seat holders | Partial (two decisions) | `universe.rs` |
| Drives | Built | `propulsion.rs`, `transit.rs` |
| Constellations, ley lines, realms, stewards | Built | `transit.rs`, `universe.rs` |
| Discord, quarantine, folding space | Built | `transit.rs`, `universe.rs` |
| Exodus and settlement within a people's system | Built | `civ/mod.rs` |
| Settlement beyond a people's system | Planned | - |
| Tolls | Partial (quoted, not accrued) | `transit.rs` |
| Markets, trade routes, currencies | Planned | - |
| World building by peoples | Planned | - |
| Mu | Through the uplink only | [network.md](network.md) |

Paths below are relative to `crates/xyl-kernel/src` unless stated.

## Peoples

### How a people wakes

The forecast of minds (`xyl-cosmos/src/minds.rs`) gives, for every world of
the galaxy's sample, the epoch at which its life would climb to a
mind-bearing rung in the undisturbed galaxy. The climb is the chemistry's
own expectation in closed form: each rung takes its mean, and impacts at
rate `mu` stretch a rung of mean `m` to `(e^(mu m) - 1) / mu`. No draw
enters, so a mind's epoch is a function of its world and its system.

`Cosmic::wake` (`cosmic.rs`) advances deep time and wakes each forecast
mind whose epoch has come. A mind becomes a people only where its world, read
at that epoch, feeds a viable people (`xyl_cosmos::life::viable`, 500
bodies); otherwise it stays silent. The world then selects the people's
traits (`cosmic::select`), its numbers start at what its world houses on
foraging alone, and two annals are written: `DEEP_HISTORY` (when life began
and the rungs it passed) and `EMERGENCE`.

A people is **ancient** when it wakes in a galactic core within one core
assembly time (`minds::ancient_window`) of that core's first core-born mind.
Ancient peoples hold the ancient and progenitor seats and build the ley
network.

### What a people is

`cosmic::People` holds its name (placeholder syllables, display only), star
and world, the region of chemistry it arose in, its traits, its dawn, its
numbers and what its world houses, its growth rate, whether it is ancient,
its rungs and setbacks, and when it fell.

`cosmic::Traits` holds what its world selected: its breeding strategy (r or
K, from the world's variability), its solvent and where its medium lies
(surface, under ice, supercritical, deep), its body temperature, leading
sense, form, what it works with, its sociality, its size, and the shape of
its mind: a society of many minds, a colony that thinks as one, or a mind
spread through its medium.

### Numbers

Numbers follow from the people's body and its world (`growth.rs`):

- body mass `m_b = m_mind x size`; metabolism the unit body's scaled by the
  size share `2s / (1 + s)` at its chemistry's pace;
- a generation is the unit generation scaled by the same share; the natural
  rate is one e-fold per generation (`growth_per_yr`);
- numbers follow the exact logistic toward what the world houses,
  `K / (1 + (K / N - 1) e^(-r t))` (`growth::grow`), solved over any span.

What a world houses is one law (`civ::housing::houses`):

```
link      = trophic + (1 - trophic) * B
natural   = forage one trophic link up the medium, lifted by B (0 if the
            people's solvent is absent)
synthesis = E * (starlight + internal heat over the surface) * eta_primary * link
housing   = (natural + synthesis) / metabolism
```

`B` and `E` are the people's biology and energy masteries in `[0, 1)`.
Biology raises the share of primary production it can eat; energy lets it
run its own producers on its world's starlight and heat. The result is
scaled by the people's radiation environment (`field.rs`): its radiation
against what its repair is adapted to.

### Knowledge

Knowledge (`knowledge.rs`) is a stock per people in four domains: energy,
materials, biology and information. There is no technology tree.

- Mastery is `1 - 1 / (1 + ln(1 + K / U))`, below one for any stock.
- A people thinks with the power it commands (its bodies' metabolism), at
  Landauer's cost for its temperature; free thought is the share of bodies
  food no longer needs, and every member adds a lived share.
- Each domain follows `dK/dt = e - lam K`, carried in closed form
  (`knowledge::advance`): `K(t) = e / lam + (K0 - e / lam) exp(-lam t)`.
  Forgetting follows the people's generation; a collective forgets less.
- Thought is directed by the people's needs (hunger to biology, a spent
  crust to materials, a power shortfall to energy, trade to information)
  over a floor of curiosity, plus any focus its ruler sets. The trade pull
  reads zero until trade exists.

Masteries feed housing (biology, energy), drive performance (energy) and
the gauges (information is sentience).

### The carry

Every people is carried by one procedure, `Civilizations::carry`
(`civ/mod.rs`), in live play and in readings alike. For each living people,
over its own span from where it was last carried to the target time:

1. **Knowledge.** Its stocks advance (`Knowledge::step_one`), its thinkers
   scaled by its consciousness gauge.
2. **Mastery turnings.** Biology or information passing one half reports a
   turning.
3. **Drive.** Its chemical drive is blended toward the best drive its
   chemistry allows by its energy mastery (`transit::blended`,
   `transit::best_spec`). It sails if the cruise speed left after escaping
   its world is positive.
4. **World, reach and fall** (in parallel across peoples). Its world is read
   at the target time. For a sailing people whose world fails or that has
   grown past half of what its world houses, the reach is read: the worlds
   of its system within four orbital places that no living people holds and
   that house a viable people. If its world no longer houses one body, the
   fall is located by bisection over the degrees of its star's life.
5. **Exodus or fall.** A failing people with a reach moves whole to the
   nearest world of it (`EXODUS`); without one, it falls (`FALL`).
6. **Numbers.** The logistic toward what its world houses, scaled by its
   environment. Housing below one body is a fall.
7. **Gauges.** Instinct, intelligence, sentience, consciousness and their
   combination are read (`civ::Gauges::read`).
8. **History.** Turnings reported since the last carry are named (see
   Histories).
9. **Settlers.** A people at or past half of what its world houses settles
   the nearest open world of its reach with half of what that world houses,
   at least a viable people and at most half its own numbers, provided both
   sides stay viable. The daughter inherits its parent's traits and
   knowledge (`SETTLE`).

Exoduses and settlements are returned as `civ::Act` and recorded as
`Input::Move` and `Input::Settle` at their counts, so a checkpoint lays them
back; while a checkpoint is laid back the carry decides neither. Live play
carries in pieces no longer than a tenth of the shortest turning time of any
living people (`Universe::civilizations`); a reading carries each people
once to the reading's time (`Universe::read_at_now`).

## Histories

### Annals

An annal (`cosmic::Annal`) is an epoch, a kind and a text. The annals are
the universe's history: what happened is kept, never re-derived.

| Kind | Written when | Code |
|---|---|---|
| `BIG_BANG` | genesis | `cosmic.rs` |
| `DEEP_HISTORY` | a people wakes: its life's climb | `cosmic.rs` |
| `EMERGENCE` | a people wakes | `cosmic.rs` |
| `FALL` | a world no longer feeds its people | `cosmic.rs`, `civ/mod.rs` |
| `EXODUS` | a people leaves its world | `civ/mod.rs` |
| `SETTLE` | a daughter people is founded | `civ/mod.rs` |
| `EMINENT` | a turning is named by a figure | `civ/mod.rs` |
| `FOLD` | a people first folds space | `universe.rs` |

The event log (`Universe::log`) holds every annal at its count together
with the network's entries (`LEY`, `REALM`, `QUARANTINE`, `REOPENED`) and
every applied input (`INPUT`).

### Turnings and eminent figures

A turning (`civ::history::Turning`) is reported by an act:

| Act | Reported when |
|---|---|
| `exodus` | the people leaves its world |
| `settlement` | it founds a daughter people |
| `network` | its constellation first holds a ley mouth |
| `quarantine` | its constellation's quarantine crosses one half, either way |
| `biology`, `information` | its mastery of that domain passes one half |
| `fold` | it first folds space |

At the people's next carry each turning is named by a figure
(`civ::history::Eminent`) whose role is read from the people at that
moment: the domain its effort led in and that domain's share of its effort,
and how the people holds the figure, by the shape of its mind (a person in a
society, the colony itself, or a locus of a distributed mind). A figure's
span is one generation from its turning. Memory fades as `(1 - f)^years`,
`f` the people's forgetting share; figures below `history::forgotten()`
(three e-folds) are dropped. Names are placeholders drawn from the seed
until a mind of the people names its figures. Figures are held in the
checkpoint.

### Rises, exoduses and falls

A people rises by waking (`EMERGENCE`), growing toward what its world
houses, raising its masteries (`biology`, `information` turnings), joining
the ley network (`network`) and founding daughters (`SETTLE`). It moves when
its world fails and its drive reaches an open world (`EXODUS`), and falls
when its world no longer feeds it and it cannot leave (`FALL`). A fallen
people's seat returns to the universe's own mind.

### Reading histories

`xylarium observe` reads a home without writing to it:

- `annals [K]`: the last K annals;
- `events [K]`: the last K entries of the event log, at their counts;
- `peoples`: the roster; `people N [Y]`: one people, optionally after
  carrying live play Y years in memory;
- `house`, `ley [K]`, `sync [K]`, `clock`: the ancient houses, the ley
  network, discord and quarantine, and the keepers of time.

## Seats

### Roles

Seats (`seats.rs`) are the places an external mind takes. Every seat is
bound by the same laws; a seat acts only through recorded inputs.

| Role | Seated on | Raised |
|---|---|---|
| Sovereign | a people | one per non-ancient people |
| Ancient | an ancient people (its house) | one per ancient people |
| Progenitor | an ancient people (its builders of passage) | one per ancient people |
| Steward | a realm | one per realm, as the realm first forms |

Seats are append-only; a seat's index is its id. A seat's holder is the
universe's own mind (`Holder::Universe`: the built-in laws decide) or a
mind joined by its key (`Holder::Mind`, taken by `Input::Take`); the
checkpoint also records a named player (`Holder::Player`), and taking a
seat as a player is on the roadmap (Seats for minds). At
each reading a mind holding any seat other than an ancient seat is returned
to the universe's own mind, so minds joined through the network sit on the
ancient houses' councils.

### Perceiving

`percept::perceive` gives the record a seat on a people perceives at the
present: its role, the people's name, whether it is ancient, its dawn and
the time now, its traits, numbers, what its world houses, its growth rate
and power budget, its world's surface temperature, pressure and light, every
medium of its world, its day and year (which pace percepts), the bells as
heard at its star, its neighbours and the latest eight annals that name it.
A neighbour is heard only once its dawn lies at least the light delay in the
past. Percepts for steward seats are on the roadmap (Seats for minds).

On the seat surface a percept is sent as typed exact records
(`lexicon::records`): `PEOPLE`, `NUMBERS`, `POWER`, `WORLD`, `MEDIUM`,
`HEARTBEAT`, `HEARING`, `NEIGHBOUR`. Solvents travel as ids
(`lexicon::solvents`), and neighbours only as keys hashed under the
universe's domain (`lexicon::people_key`).

### Acting

These are all the inputs the kernel applies (`universe::Input`):

| Input | Made by | Effect |
|---|---|---|
| `Take { seat, mind }` | a joining mind | the seat is held by that key |
| `Focus { seat, domain }` | a seat on a people | directs (or with `None` releases) the people's inquiry |
| `Handle { seat, a, b, handling }` | a steward seat | sets the handling of the line between stars `a` and `b` in its realm |
| `Move { people, world }` | the carry | an exodus |
| `Settle { parent, world, emigrants }` | the carry | a settlement |

A seat holder has two decisions (`universe::Decision`): `Focus` and
`Handle`. `Universe::decide` lands one now and records it as an input;
`Universe::queue` places an input at a later count. Every applied input is
held in the checkpoint and laid back on restart. `decide` checks the seat's
subject (a people for `Focus`, a realm for `Handle`); confirming that the
caller holds the seat is the uplink's part.

The `xylarium` binary has no command for decisions. External minds act
through a `run::Link` beside live play (`crates/xylarium/src/run.rs`; see
[network.md](network.md)), which receives the universe after every ring.

## Grand strategy

### Drives

A drive (`propulsion.rs`) is chemical, fission, fusion or annihilation. Its
exhaust speed is the energy released per mass over light speed, capped at
light. A hull of mass ratio `R` gains `dv = c tanh((v_e / c) ln R)`; it
climbs out of its world's well, coasts at half of what is left and brakes
with the other half (`transit::cruise_speed`, `transit::voyage`). Each
people's drive is its chemical drive blended toward the best its chemistry
allows by its energy mastery.

### Constellations

Stars group into constellations by binding: a pair binds where their mutual
turning exceeds the galaxy's shear across their span
(`transit::binding_at`, `transit::constellations`). Passage is orbital and
by constellation: within a system along orbits with transfer windows
(`transit::window_years`), within a constellation along arcs, and between
constellations only over ley lines.

### Ley lines and the worth law

Ley lines are built by the ancient houses and their progenitors
(`Network::step`). A house begins a line when idle; the line opens when both
its voyage and the time its world's spare power takes to pay the line's
cost have passed (`LEY`). A line's worth to its passer, per kilogram, is
the two wells' remainders at its mouths plus the voyage it saves within the
passer's generation, less its tithe:

```
worth          = m_body * (c^2 (delta_A + delta_B) + c * min(S / T, c)) - tithe
tithe          = passage_energy * max(1, S / mouth_width)
passage_energy = m_body * c * (c - v_e)
```

`v_e` is the builder's exhaust speed: the throat supplies only what the
builder's drive does not. Lines are built only where worth is positive.

- Discovery: a house considers only constellations its craft could have
  reached since its dawn and that the network has not joined.
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
| Open | everyone | its floor |
| Tolled (default) | everyone | half way from floor to ceiling |
| Guarded | its builder's house and the peoples of its realm | as tolled |
| Closed | no one | - |

`Network::carries` reads a line's handling for a passer. The toll law
(`transit::quote`) prices a line from its books (`Network::lane`): the floor
is its cost not yet paid back over last year's traffic, the ceiling what it
saves the passer against the cheapest way round, and the toll half way
between, never above the ceiling.

### Discord and quarantine

Each living non-ancient people has a discord (`Universe::discord_parts`):

```
discord = 1 - (1 - o f (1 - J)) (1 - breach)
```

`o` is the share of its reach per generation that lies outside its
constellation, `f` its forgetting share, `J` whether its constellation holds
a ley mouth, and `breach` its folding share. A constellation is held closed
by `1 - prod(1 - discord)` over its peoples, re-read at every step and never
latched (`QUARANTINE`, `REOPENED`). A line's open share is the product of
its two ends' open shares.

### Folding space and breach

Whether space can be folded is a law of the universe: `Laws::ftl`
(`xyl-cosmos/src/laws.rs`, true by default) is carried into the galaxy's
`Scales::ftl`, which `Network::fold` reads. Where folding is lawful, a
sailing people whose constellation holds no ley mouth folds space when a
passage out is worth it at its own drive: across its constellation's reach
`S`, `c min(S / T, c) > c (c - v_e)` per kilogram. It folds once a
generation, each fold costing the passage energy. A fold is an
unauthorised passage: the people's breach share is the energy it has folded
over that energy plus the cost of the lawful lines at its constellation plus
one passage of its own (`Network::breach`). Its discord rises with it, and
its constellation is driven toward quarantine. The first fold writes a
`FOLD` annal and a `fold` turning.

## World building and space trade

- **Settlement**: exodus and daughter peoples within a people's own system
  are built. Settling the worlds of other stars, over drives or ley lines,
  is planned.
- **Tolls**: quoted from each line's books. No traffic is carried over a
  line yet, so nothing accrues: a lane's collected energy and last year's
  passages are zero (`Network::lane`).
- **Markets and trade routes**: planned, emergent from the shape of each
  people's mind (a colony allocates within itself; a society may trade).
  The kernel holds no market, trade route or currency.
- **World building**: planned. A people's power budget (`growth::Power`)
  reads its world's starlight and heat, but a people commands only its
  bodies' power until seated minds direct the capture of its world's flows.
  Ancient houses use their world's spare power to pay for ley lines.

## The in-game economy

What is built is physical: numbers grow from food and power, knowledge from
the power a people commands, ley lines are paid for in energy from a house's
spare power, and tolls are quoted in joules. Banks, currencies and markets
are planned and will emerge from these quantities.

Mu, the network's unit, is not represented in the kernel. It enters and
leaves a universe only through the uplink ([network.md](network.md)).

## Declared rulers

The civilization layer reads a set of declared rulers that the roadmap
derives from each people's own chemistry, senses and economy: the unit
body's generation (14.2 yr), the mind-bearing body (10 kg), primary
efficiency 0.015, trophic transfer 0.1 and the flux ruler 340 W/m^2
(`growth::Rulers`); the stage's mass ratio 10, the chemical exhaust's 18 amu
and the fission share (`propulsion`); the thousand unit minds of the
knowledge unit (`knowledge`); the trait thresholds in `cosmic.rs` (the r
or K strategy and `select`); and
the memory threshold `e^-3` (`history::forgotten`). The full list is in
[roadmap.md](roadmap.md#declared-rulers).
