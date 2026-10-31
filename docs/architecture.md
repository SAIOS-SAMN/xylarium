# Architecture

This document describes how Xylarium is put together: the tiers of the
simulation, how data flows between them, the event engine, determinism and
caching, state and its checkpoints, and the readings `xylarium observe`
provides.

## Layers

The simulation is a stack of tiers. Each tier reads only the tiers beneath
it and is evaluated lazily, when a reading or an event requires it.

| Tier | Modules | Produces |
|---|---|---|
| Numbers | `xyl-core` `q`, `certified`, `dyadic`, `cga`, `rotor`, `units`, `measure`, `event`, `rng`, `coordinate` | Exact rationals, certified enclosures, conformal geometric algebra `Cl(n+1,1)`, rational rotors, the tier unit ladder, the certified crossing solver, deterministic streams |
| Founding | `xyl-cosmos` `origin`, `bounce`, `elements`, `atoms`, `compound`, `fibres`, `formation`, `chain`, `cores`, `package`, `circuit::found` | The element table, the founding cascade, the firmament, beads and galaxies |
| Substrate | `xyl-cosmos` `circuit` (`core`, `weights`, `anchor`, ...) | The core's reverberation, outflow, the medium, intake, force and tide as functions of `t`; the anchor (bell, rims, radius) |
| Galaxies | `xyl-cosmos` `galaxy`, `stars`, `chem`, `centre`, `pulse`, `field`, `turning` | The home galaxy's stars on orbits, star lives, chemical evolution, the bells and their pulses |
| Systems and worlds | `xyl-cosmos` `disc`, `systems`, `manifold`, `bodies`, `worlds`, `hydro` | The star's disc and cut, worlds on the star's manifold, moons on each world's manifold, world state by degree of the star's life |
| Life and minds | `xyl-cosmos` `life`, `minds` | The hard-step ladder and the forecast of minds |
| Universe in play | `xyl-kernel` `universe`, `cosmic`, `civ`, `checkpoint` | The event engine, peoples, the civilization carry, annals, held state |
| Civilization laws | `xyl-kernel` `civ::housing`, `civ::history`, `growth`, `knowledge`, `field`, `sync` | Housing, numbers, turnings and eminent figures, knowledge stocks, radiation environment, step readings |
| Transit | `xyl-kernel` `propulsion`, `transit` | Drives, voyages, constellations, ley lines, realms, quarantine, folds, toll quotes |
| Seats | `xyl-kernel` `seats`, `percept`, `lexicon` | Seats for external agents and the records they perceive |
| Time | `xyl-kernel` `chronometry` | The keepers chained from the universal core outward |
| Commands | `xylarium` `run`, `observe` | `genesis`, `run`, `observe`, and the `run::Link` seam |

The uplink to Mycelium is not in this repository. It attaches to live play
through `xylarium::run::Link` (see [network.md](network.md)).

## The founding and the package

`xyl_cosmos::package` holds what a universe is founded from that its laws do not
derive: the universal core's mass in SI units (`hole_mass`, declared by the
package, the one quantity that converts the core's units to SI for display), the
SI constants (`Si`, where measured values enter, converted once), and the
founding's digest (`founding_digest`). The founding reads only the package and
the parent's spin drawn from the seed; it does not read the layers built on it.

Units are `G = c = M_core = 1`. The universal core's holding, the
universal state, is 1 in these units. The [firmament](glossary.md) (the
[throat](glossary.md)'s held kinds, the vessel band computed by
`atoms::vessel`) is scaled so its mass equals that state. Each galactic
core's mass is its [bead](glossary.md)'s state in the universal core's
units.

Above the founding, `disc::star_hold` computes the share of a lit bead's
infall the star holds, and `disc::manifold_worlds` places worlds on the
star's manifold by reading it degree by degree outward (360 degrees), with
no seed draw or partition. Moons form on each world's manifold by the same
function (`systems`).

## Data flow

1. `genesis` derives the origin from the seed, founds the substrate, forms
   the galaxies' star samples and runs the forecast of minds: for each
   candidate world, the time (if any) at which its lineage reaches the rung
   of its ladder that holds a mind (`minds::schedule`). It writes the
   genesis record and a first checkpoint at the time of the first mind.
2. On every start the universe is regenerated from the seed. State that the
   laws do not determine is restored from the checkpoint. A home with no
   checkpoint is read at the first mind's dawn, where genesis writes its
   first checkpoint.
3. The event engine (`Universe::advance_to`) advances from event to event:
   forecast wakings (a mind arising on its world), queued inputs and ley
   lines falling due. Between events nothing is computed.
4. At the target time the engine takes its readings: the bells' rings delivered,
   every people carried over the span since its last carry, the ley network
   carried, and the seats of peoples whose population has reached zero released.
5. `observe` evaluates the requested reading at the universe's time and
   prints it. Law readings are taken without carrying the universe; the
   others carry it first (see [Readings](#readings)).

## The event engine

Time is counted in [bells](glossary.md) of the universe's core (one bell
is light's round trip to the home galaxy's rim). Events are kept in a queue
ordered by count. An advance performs the following operations:

- It prepares, in parallel, the systems and worlds of every mind due in the
  span; these are pure functions of the seed.
- It adds each forecast mind's people at the epoch of that mind. A world
  that can no longer feed a viable people at that moment produces no
  people.
- It opens ley lines at their due times and begins each builder's next
  line.
- It applies queued inputs (`universe::Input`: seat taken, inquiry
  focused, ley line handling, exodus, settlement) at their counts.
- At the target time, it performs the readings.

Live play (`run`) advances one [ring](glossary.md) (one full bell) at a
time. A reading (`observe`) carries each people once over its whole span.
Both use the same carry (`civ::Civilizations::carry`). The civilization
layer is the only stepped layer; it is carried over spans no longer than
the turnover of the civilization quantities allows.

## Determinism and caching

All randomness is drawn from `xyl_core::Stream`, derived from the seed and a
path of labels, so every draw is reproducible and independent of evaluation
order. Caches hold only values the laws regenerate; deleting a cache never
changes a result. Results are verified by conservation (each reading's
balances close), not by byte-for-byte replay.

Two build-time digests key the caches (`crates/xyl-cosmos/build.rs`):

| Digest | Covers | Keys |
|---|---|---|
| `XYL_FOUNDING_DIGEST` | The founding's files only: `origin`, `bounce`, `elements`, `atoms`, `compound`, `fibres`, `formation`, `cores`, `chain`, `package`, `circuit/`, `data/`, and `xyl-core` | The founding cache, with the torsion share |
| `XYL_SOURCE_DIGEST` | Every source file of `xyl-cosmos` and `xyl-core`, and `data/` | The forecast and the heavy-star caches |

A change to a layer above the founding (a disc, a world or a life law)
leaves the founding digest unchanged, so a home's founding is reused. The
founding anchors every tier above it through `circuit::anchor` (the bell,
the rims, the universe's radius, the firmament's margin).

## Concurrency

Parallel work uses `rayon` over pure functions: system construction, world
reads and fall solves. Structures holding interior-mutable caches (`Cosmic`)
are not shared across threads; parallel stages receive the plain data they
need (`Galaxy`, `Ctx`, the witnessed systems).

## State management

A universe's state is divided by how it is obtained:

| Kind | Examples | Storage |
|---|---|---|
| Derived | The founding, galaxies, star orbits, systems, world states, the forecast of minds, the bells | Recomputed from the seed on every start; never stored as truth |
| Cached derived | The founding (`founding-*`), the forecast (`forecast-*`, `forecast-stars/`), the heavy stars (`bells-*`) | Written to the home directory as accelerators; deleting them does not change results |
| Path-dependent | Each people's numbers, housing, knowledge and focus, home radiation, eminent figures, ley lines built and their handling, folds, the keepers' commits | Written to `checkpoint` at each save |
| Inputs | Seat takes, focus, handling, moves and settlements | Recorded at their counts in `checkpoint` |

A home directory holds:

| File | Content |
|---|---|
| `genesis` | The genesis record (`xylarium genesis v2`): seed, founding digest, laws digest, the anchor, the package (core mass, spin, share), the census by kind, the galaxies, the systems' limits, the bell |
| `founding-<digest>-<share>` | The founding cache |
| `forecast-*`, `forecast-stars/` | The forecast of minds, whole and star by star |
| `bells-*` | The heavy stars whose bells are weighed |
| `checkpoint` | Held state and inputs |
| `observer.log` | The process monitor's progress log (`xyl_core::watch`) |

Checkpoints are written whole to a temporary file and renamed, so a crash
never leaves a partial checkpoint.

## Memory model

- Values are exact rationals backed by arbitrary-precision integers; small
  values are stored inline and promote to big integers only when needed.
  Dyadic intermediates (`Dy`) have bounded mantissas.
- Systems and worlds are materialised only when observed. World states are
  cached per star, world and degree of the star's life.
- The workspace forbids `unsafe`; all sharing across threads is by immutable
  reference to plain data.

## Readings

`xylarium observe [options] WORD [ARGS]` prints one reading
(`crates/xylarium/src/observe/mod.rs`). A law reading is evaluated from the seed
and the laws alone, without carrying the universe. A carried reading first
builds the universe from its seed and restores its checkpoint (or carries a
fresh genesis at the first mind's dawn when the home has none), as a `run` start does. "The
books" in a reading are its conservation balances. Every reading is read-only
and may run beside `run`. Bracketed arguments are optional, with their defaults.

| Word | Arguments | Kind | Shows |
|---|---|---|---|
| `formation` | | Law | Each galaxy's cut, gas, cooling and stars, counted in rings of the galaxy's anchor echo |
| `seals` | `[N=1600]` | Law | The quantum capacity of kinds Z 1 to N on the bands of each kind's atom, the seams and the kinds that seal |
| `founding-reach` | | Law | Every founded kind's reach about the throat from the founding cascade, and its open share |
| `worlds` | `[N=200]` | Law | The planets of the first N sample stars as settling leaves them: orbits, masses, neighbour ratios, and the stars without worlds by cause |
| `push` | `[N=150]` | Law | Each kind's push with Z in both poles against the built and linear pushes, Z 1 to N |
| `bonding` | `[N=150]` | Law | Each kind's contacts, grip and conduction in a body of like atoms, Z 1 to N |
| `core` | `[T=14]` | Law | The substrate at T Gyr: the core's reverberation kind by kind, its outflow, the ocean's shares, every galaxy's intake, the books, Z Q R |
| `founding` | | Law | The equitable cascade and the founding cascade: stacks, spins, reaches and the settled bell |
| `cores` | | Law | The founding's galaxies: each galactic core's hole, its mass ratio to the universal core, its reach, phase and place |
| `tiers` | `S` | Law | Star S and its worlds, each turn counted in its anchor's rings |
| `universe` | | Carried | The default: the bell's count, the peoples, the seats, the bells, the books, the worlds formed and the cores |
| `peoples` | | Carried | Every people: its role, home, numbers and housing |
| `people` | `N [Y]` | Carried | One people: its body, numbers, power, knowledge, radiation, world and seats; with Y, the civilization layer is carried Y more years in memory first |
| `sync` | `[K=10] [Y]` | Carried | Every living people's step readings as quartiles, and the K most out of step per reading; Y as for `people` |
| `ley` | `[K=10]` | Carried | The ley network: constellations, lines, passages, discord and quarantine, the star map; K bounds each list |
| `star` | `N` | Carried | One reached star: its books, drift and worlds |
| `house` | | Carried | The seats held by minds, and the ancient seats |
| `chemistry` | | Carried | The galaxy's elements across the disc, then and now, and its books |
| `galaxy` | | Carried | The units, the bells, and sample stars on their galactic manifolds |
| `star-types` | `[N] [R]` | Carried | The classes the stars' formulas give over each galaxy's first N sample stars, then the home galaxy's main-sequence stars at R of its rim beside the Earth's star's values for comparison |
| `system` | `S [S...]` | Carried | Star S's planets and moons on their manifolds, their caps and relations; several stars in one reading |
| `world` | `S W` | Carried | World W of star S across its star's life: air, seas, surface, field and what it houses |
| `annals` | `[K=20]` | Carried | The last K annals |
| `events` | `[K=20]` | Carried | The last K events logged, each at its count |
| `clock` | | Carried | The bell, the keepers from the core outward, and the measured pace in rings per network final point (`Chronometry::pace`) |
