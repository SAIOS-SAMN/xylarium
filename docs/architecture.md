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
| Numbers | `xyl-core` `q`, `certified`, `dyadic`, `cga`, `rotor`, `units`, `measure`, `event`, `rng`, `coordinate` | exact rationals, certified enclosures, conformal geometric algebra `Cl(n+1,1)`, rational rotors, the tier unit ladder, the certified crossing solver, deterministic streams |
| Founding | `xyl-cosmos` `bang`, `bounce`, `elements`, `atoms`, `compound`, `fibres`, `formation`, `chain`, `cores`, `package`, `circuit::found` | the element table, the founding cascade, the firmament, beads and galaxies |
| Substrate | `xyl-cosmos` `circuit` (`core`, `weights`, `anchor`, ...) | the core's reverberation, outflow, the medium, intake, force and tide as functions of `t`; the anchor (bell, rims, radius) |
| Galaxies | `xyl-cosmos` `galaxy`, `stars`, `chem`, `centre`, `pulse`, `field`, `turning` | the home galaxy's stars on orbits, star lives, chemical evolution, the bells and their pulses |
| Systems and worlds | `xyl-cosmos` `disc`, `systems`, `manifold`, `bodies`, `worlds`, `hydro` | the star's disc and cut, worlds on the star's manifold, moons on each world's manifold, world state by degree of the star's life |
| Life and minds | `xyl-cosmos` `life`, `minds` | the hard-step ladder and the forecast of minds |
| Universe in play | `xyl-kernel` `universe`, `cosmic`, `civ`, `checkpoint` | the event engine, peoples, the civilization carry, annals, held state |
| Civilization laws | `xyl-kernel` `civ::housing`, `civ::history`, `growth`, `knowledge`, `field`, `sync` | housing, numbers, turnings and eminent figures, knowledge stocks, radiation environment, step readings |
| Transit | `xyl-kernel` `propulsion`, `transit` | drives, voyages, constellations, ley lines, realms, quarantine, folds, toll quotes |
| Seats | `xyl-kernel` `seats`, `percept`, `lexicon` | seats for external agents and the records they perceive |
| Time | `xyl-kernel` `chronometry` | the keepers chained from the universal core outward |
| Commands | `xylarium` `run`, `observe` | `genesis`, `run`, `observe`, and the `run::Link` seam |

The uplink to Mycelium is not in this repository. It attaches to live play
through `xylarium::run::Link` (see [network.md](network.md)).

## The founding and the package

`xyl_cosmos::package` holds what a universe is founded from that its laws do
not derive: the universal core's mass (`hole_mass`, the bridge to SI used
for display), the SI constants (`Si`, the seam where measured values enter),
and the founding's digest (`founding_digest`). The founding reads only the
package and the parent's spin drawn from the seed; it never reads the layers
built on it.

Units are `G = c = M_core = 1`. The universal core holds the universal
state, 1. The firmament, the throat's held kinds (the vessel band computed
by `atoms::vessel`), carries that state. Each galactic core's mass is its
bead's state in the universal core's units.

Above the founding, `disc::star_cut` takes a star's cut of its piece's
infall, and `disc::manifold_worlds` places worlds on the star's manifold,
read every degree, with no seed draw or partition. Moons gather on each
world's own manifold (`systems`).

## Data flow

1. `genesis` derives the Big Bang from the seed, founds the substrate, forms
   the galaxies' star samples and runs the forecast of minds: for each
   candidate world, when (if ever) its lineage climbs the hard steps to a
   mind. It writes the genesis record and a first checkpoint at the first
   mind's dawn.
2. On every start the universe is regenerated from the seed. State that the
   laws do not determine is restored from the checkpoint. A home with no
   checkpoint is carried to 14 Gyr.
3. The event engine (`Universe::advance_to`) advances from event to event:
   forecast wakings, queued inputs and ley lines falling due. Between events
   nothing is computed.
4. At the target time the engine takes its readings: the bells' rings
   delivered, every people carried over its own span, the ley network
   carried, seats of fallen peoples released.
5. `observe` evaluates the requested reading at the universe's time and
   prints it. Law readings are taken without carrying the universe; the
   others carry it first (see [Readings](#readings)).

## The event engine

Time is the bell count of the universe's core. Events are kept in a queue
ordered by count. An advance:

- prepares, in parallel, the systems and worlds of every mind due in the
  span (pure functions of the seed);
- wakes each mind at its own epoch; a world that can no longer feed a viable
  people at that moment produces no people;
- opens ley lines at their due times and begins each builder's next line;
- lands queued inputs (`universe::Input`: seat taken, inquiry focused, ley
  line handling, exodus, settlement) at their counts;
- at the target, performs the readings.

Live play (`run`) advances one ring of the bell at a time. A reading
(`observe`) carries each people once over its whole span. Both use the same
carry (`civ::Civilizations::carry`). The civilization layer is the only
stepped layer; it is carried over spans no longer than its own turnover
allows.

## Determinism and caching

All randomness is drawn from `xyl_core::Stream`, derived from the seed and a
path of labels, so every draw is reproducible and independent of evaluation
order. Caches hold only values the laws regenerate; deleting a cache never
changes a result. Verification is by conservation (books that close), not
byte-for-byte replay.

Two build-time digests key the caches (`crates/xyl-cosmos/build.rs`):

| Digest | Covers | Keys |
|---|---|---|
| `XYL_FOUNDING_DIGEST` | the founding's files only: `bang`, `bounce`, `elements`, `atoms`, `compound`, `fibres`, `formation`, `cores`, `chain`, `package`, `circuit/`, `data/`, and `xyl-core` | the founding cache, with the torsion share |
| `XYL_SOURCE_DIGEST` | every source file of `xyl-cosmos` and `xyl-core`, and `data/` | the forecast and the heavy-star caches |

A change to a layer above the founding (a disc, a world or a life law)
leaves the founding digest unchanged, so a home's founding is reused. The
founding anchors every tier above it through `circuit::anchor` (the bell,
the rims, the universe's radius, the firmament's margin).

## Concurrency

Parallel work uses `rayon` over pure functions: system construction, world
reads, fall solves. Structures holding interior-mutable caches (`Cosmic`)
are not shared across threads; parallel stages receive the plain data they
need (`Galaxy`, `Ctx`, the witnessed systems).

## State management

A universe's state is split by how it is obtained:

| Kind | Examples | Storage |
|---|---|---|
| Derived | the founding, galaxies, star orbits, systems, world states, the forecast of minds, the bells | recomputed from the seed on every start; never stored as truth |
| Cached derived | the founding (`founding-*`), the forecast (`forecast-*`, `forecast-stars/`), the heavy stars (`bells-*`) | written to the home directory as accelerators; deleting them changes nothing |
| Path-dependent | each people's numbers, housing, knowledge and focus, home radiation, eminent figures, ley lines built and their handling, folds, the keepers' commits | written to `checkpoint` at each save |
| Inputs | seat takes, focus, handling, moves and settlements | recorded at their counts in `checkpoint` |

A home directory holds:

| File | Content |
|---|---|
| `genesis` | the genesis record (`xylarium genesis v2`): seed, founding digest, laws digest, the anchor, the package (core mass, spin, share), the census by kind, the galaxies, the systems' limits, the bell |
| `founding-<digest>-<share>` | the founding cache |
| `forecast-*`, `forecast-stars/` | the forecast of minds, whole and star by star |
| `bells-*` | the heavy stars whose bells are weighed |
| `checkpoint` | held state and inputs |
| `observer.log` | the process monitor's progress log (`xyl_core::watch`) |

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
(`crates/xylarium/src/observe.rs`). A law reading is evaluated from the
seed and the laws alone, without carrying the universe. A carried reading
first builds the universe from its seed and lays back its checkpoint (or
carries a fresh genesis to 14 Gyr when the home has none), exactly as a
`run` start does. Every reading is read-only and may run beside `run`.
Bracketed arguments are optional, with their defaults.

| Word | Arguments | Kind | Shows |
|---|---|---|---|
| `formation` | | law | each galaxy's cut, gas, cooling and stars, counted in its own echo rings |
| `seals` | `[N=1600]` | law | the quantum capacity of kinds Z 1 to N on their own bands, the seams and the kinds that seal |
| `founding-reach` | | law | every founded kind's reach about the throat from the founding cascade, and its open share |
| `worlds` | `[N=200]` | law | the planets of the first N sample stars as settling leaves them: orbits, masses, neighbour ratios, and the stars without worlds by cause |
| `push` | `[N=150]` | law | each kind's push with Z in both poles against the built and linear pushes, Z 1 to N |
| `bonding` | `[N=150]` | law | each kind's contacts, grip and conduction in a body of like atoms, Z 1 to N |
| `core` | `[T=14]` | law | the substrate at T Gyr: the core's reverberation kind by kind, its outflow, the ocean's shares, every galaxy's intake, the books, Z Q R |
| `founding` | | law | the equitable cascade and the founding cascade: stacks, spins, reaches and the settled bell |
| `cores` | | law | the founding's galaxies: each galactic core's hole, its mass ratio to the universal core, its reach, phase and place |
| `tiers` | `S` | law | star S and its worlds, each turn counted in its anchor's rings |
| `universe` | | carried | the default: the bell's count, the peoples, the seats, the bells, the books, the worlds formed and the cores |
| `peoples` | | carried | every people: its role, home, numbers and housing |
| `people` | `N [Y]` | carried | one people: its body, numbers, power, knowledge, radiation, world and seats; with Y, the civilization layer is carried Y more years in memory first |
| `sync` | `[K=10] [Y]` | carried | every living people's step readings as quartiles, and the K most out of step per reading; Y as for `people` |
| `ley` | `[K=10]` | carried | the ley network: constellations, lines, passages, discord and quarantine, the star map; K bounds each list |
| `star` | `N` | carried | one reached star: its books, drift and worlds |
| `house` | | carried | the seats held by minds, and the ancient seats |
| `chemistry` | | carried | the galaxy's elements across the disc, then and now, and its books |
| `galaxy` | | carried | the units, the bells, and sample stars on their galactic manifolds |
| `system` | `S [S...]` | carried | star S's planets and moons on their manifolds, their caps and relations; several stars in one reading |
| `world` | `S W` | carried | world W of star S across its star's life: air, seas, surface, field and what it houses |
| `annals` | `[K=20]` | carried | the last K annals |
| `events` | `[K=20]` | carried | the last K events logged, each at its count |
| `clock` | | carried | the bell, the keepers from the core outward, and the bridge's pace |
