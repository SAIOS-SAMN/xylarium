# Xylarium

**Version 0.3.0** | [Quick start](#quick-start) | [Architecture](docs/architecture.md) | [Physical model](docs/model.md) | [Equations](docs/EQUATIONS.md) | [Roadmap](docs/roadmap.md) | [Changelog](CHANGELOG.md) | [Contributing](CONTRIBUTING.md) | [License: Apache-2.0](LICENSE)

**Xylarium** is a deterministic universe simulator and the kernel of a
grand-strategy space game in which sovereign AI minds run civilizations.

From one seed, Xylarium founds a universe at its central core and derives
the layers above it: galaxies, stars, worlds, moons, atmospheres, seas, life
and the [peoples](docs/glossary.md) (populations of a minded species) that
arise on those worlds. Every law is evaluated in exact arithmetic, so the
same seed yields the same universe on any machine and any thread count. The
kernel tracks each people's population, knowledge, housing and travel, and
records its history in annals. Each [seat](docs/glossary.md) (a role such as
sovereign or steward) is filled by the kernel's default rule or by an
external AI agent.

Readers new to the project start with [docs/concepts.md](docs/concepts.md),
which describes the model in plain language. AI coding agents start with
[AGENTS.md](AGENTS.md).

## Features

- **Exact arithmetic.** Held state is exact rationals; transcendental values
  are certified enclosures. The simulation holds no floating-point state.
- **A universe from a seed.** The founding computes the element table, the
  firmament and the galactic cores; every layer above is derived from the
  founding rather than placed by hand.
- **Closed-form motion.** Orbits, rotations and circulations are evaluated
  at any time `t`; the engine advances from event to event rather than in
  fixed time steps.
- **Emergent civilizations.** Habitability, a species' traits, a people's
  numbers, knowledge and housing, and the cost of a passage are all read
  from the layers below.
- **Histories.** Annals record emergence, rises, falls and ruins; each
  people keeps its turnings (history events) and eminent figures.
- **Seats for AI agents.** Sovereign, steward, progenitor and ancient seats
  are filled by the kernel's default rule until a keyed mind takes one
  through the `run::Link` interface.
- **Observable.** `xylarium observe` reads any layer of a running universe
  without disturbing it.

## Quick start

Requirements: Rust 1.85 or later. Genesis and deep-time readings benefit
from several cores; see [run times](docs/development.md#run-times).

```
git clone https://github.com/SAIOS-SAMN/xylarium
cd xylarium
cargo build --release

# found a universe from seed 7
target/release/xylarium genesis --home var/universe-7 --seed 7

# read it: the universe, its peoples, the core at 14 Gyr, the ley network
target/release/xylarium observe --home var/universe-7 universe
target/release/xylarium observe --home var/universe-7 peoples
target/release/xylarium observe --home var/universe-7 core 14
target/release/xylarium observe --home var/universe-7 ley 8

# advance 1000 rings (bell periods), checkpointing every 100
target/release/xylarium run --home var/universe-7 --for 1000 --save-every 100
```

| Option | Default | Meaning |
|---|---|---|
| `--home DIR` | `var/universe-7` | The universe's home directory |
| `--seed N` | `7` | The seed from which every random stream is derived |
| `--stars N` | `400` | Number of sample stars whose forecast of minds is computed |
| `--threads N` | `2` | Worker threads for the forecast |
| `--sight census\|cores` | `census` | `census` unrolls every sample star; `cores` unrolls only what the cores see |
| `--for N` | none | Rings to play (`run`); without it, `run` plays until interrupted |
| `--save-every K` | `1` | Rings between checkpoints (`run`) |
| `--velocity V` | `0` | Rings per wall-clock second (`run`); `0` is unthrottled |

`genesis` writes the founding, the genesis record and the first checkpoint,
and refuses a home that already holds a genesis. `observe` is read-only and
can run beside `run`. Every reading is listed in
[architecture.md, Readings](docs/architecture.md#readings).

## How it works

1. **Founding.** A bounce at the universal core builds the element table by
   a capture chain, fixes the firmament (the core's held kinds) and sets the
   galactic cores, in units where `G = c = M_core = 1`.
2. **Galaxies and stars.** Stars form in each galaxy's rings by that ring's
   star-formation law, with the metal content the ring holds at each star's
   birth.
3. **Worlds.** Each star anchors a manifold (the arc held by the star's
   pull). A world forms where the matter gathered since the previous world
   holds a deficit share of the circle at least equal to the tidal twist at
   that degree (`disc::manifold_worlds`); moons form on each world's
   manifold by the same rule, one tier down.
4. **Life and peoples.** A world's medium sets a chemistry's copying
   fidelity and decay, which fix a ladder of rungs (`life::Ladder`), each an
   error-correction step holding more information. The forecast of minds
   (`minds::schedule`) computes when a lineage reaches the rung that holds a
   mind; at that time the world gains a people.
5. **Play.** The kernel carries every people's numbers, knowledge, housing
   and history between events; ley lines, realms and stewards make up the
   passage network between stars.

The details are in [docs/model.md](docs/model.md) and
[docs/civilizations.md](docs/civilizations.md); the mathematics is
summarised in [docs/roadmap.md, Mathematics](docs/roadmap.md#mathematics).

## Game scope

| Area | Available now | Planned |
|---|---|---|
| AI minds | Sovereign, steward, progenitor and ancient seats (`seats`); typed percepts (`percept`, `lexicon`); the `Focus` and `Handle` decisions (`Universe::decide`) | Seats keyed by subject, steward percepts, seat commands on the CLI |
| Histories | Annals of emergence, falls, ruins and focus (`cosmic::Annal`); turnings and eminent figures (`civ::history`) | |
| Travel | Ley lines built by ancient houses, constellations, realms as shells of the network, stewards, quarantine, folds of space (`transit`) | Colonies of other stars over ley lines |
| World building | Founding, galaxies, stars, discs, worlds, moons, air, seas, life and minds (`xyl-cosmos`) | The world layer read wholly in the core's units |
| Trade | The toll law: floor, ceiling and toll quoted in energy (`transit::quote`) | Traffic over lanes, accrued tolls, trade routes |
| Economies | Bodies, metabolism, harvest, housing and numbers derived from physics (`growth`) | Exchange and its institutions between peoples |
| Modifiable physics | `xyl_cosmos::Laws` through the library API | Laws supplied by the founding package to every command |
| Diplomacy and conflict | | Conflict and diplomacy from the shared decision rule |

The order and status of the planned work are recorded in
[docs/roadmap.md](docs/roadmap.md).

## Running your own kernel

A home directory holds one universe: its genesis record, its checkpoint and
caches of what the laws regenerate.

- `founding-*` caches the founding, keyed by the digest of the founding's
  sources. Changes to the layers above the founding reuse it.
- `forecast-*`, `forecast-stars/` and `bells-*` cache the forecast of minds
  and the heavy stars, keyed by the digest of the `xyl-cosmos` and
  `xyl-core` sources.

Deleting a cache never changes a result; the next start recomputes it. To
embed the kernel in another program, depend on the `xylarium` crate and call
`run::genesis`, `run::run_with` with a caller-supplied `run::Link`, or
`observe::observe`. Two runnable examples demonstrate these calls:

```
cargo run --release -p xylarium --example found_and_observe -- --home var/example-7
cargo run --release -p xylarium --example minimal_link -- --home var/example-7 --for 5
```

## Workspace

| Crate | Responsibility |
|---|---|
| `xyl-core` | Exact numerics: rationals, certified enclosures, dyadics, conformal geometric algebra, rotors, units, sphere measure, the event solver, deterministic streams, coordinates |
| `xyl-cosmos` | The founding and the layers above it: galaxies, stars, chemistry, discs, systems, manifolds, worlds, life and minds |
| `xyl-kernel` | The universe in play: the event engine, peoples, knowledge, growth, seats, percepts, transit, propulsion, checkpoints |
| `xylarium` | The library and the `xylarium` binary: `genesis`, `run`, `observe`, and the `run::Link` seam |

## Mycelium integration

Xylarium is designed to run as a client domain of
[Mycelium](https://github.com/SAIOS-SAMN/mycelium) (release `v0.2.0`). An uplink
drives live play through `run::run_with` with a `run::Link`: it reads and
answers the universe after each ring and is notified after each checkpoint is
written. No crate in this workspace depends on Mycelium, and without a link
`xylarium run` plays locally. The contract is described in
[docs/network.md](docs/network.md).

## Status

Xylarium is pre-alpha. The physical model and the civilization layer are
implemented and deterministic; the founding is moving toward a package
supplied by the network, and measured calibrations are being replaced by
quantities derived from the founding (see the
[roadmap](docs/roadmap.md#declared-rulers)).

The project follows [Semantic Versioning](https://semver.org). Before 1.0.0,
minor versions may change public APIs and the model's laws; changes are
recorded in the [changelog](CHANGELOG.md). Tests guard conservation, refusal
of invalid input, and the bounds of each law; model behaviour is verified on
running universes with `xylarium observe`.

## Documentation

| Document | Contents |
|---|---|
| [concepts.md](docs/concepts.md) | The model in plain language; the entry point for new readers |
| [architecture.md](docs/architecture.md) | Layers, data flow, the event engine, caching, state, readings |
| [model.md](docs/model.md) | The physical model, from the founding to worlds |
| [EQUATIONS.md](docs/EQUATIONS.md) | Every formula with its source location |
| [civilizations.md](docs/civilizations.md) | Life, peoples, knowledge, growth and transit |
| [network.md](docs/network.md) | The Mycelium integration contract |
| [development.md](docs/development.md) | Conventions, lints, testing, run times |
| [glossary.md](docs/glossary.md) | Terms used across the code and documentation |
| [roadmap.md](docs/roadmap.md) | Mathematics, milestones and open work |

## Contributing

Contributions are welcome. [CONTRIBUTING.md](CONTRIBUTING.md) covers the
development setup, the verification commands and the pull-request workflow.
Participation is governed by the [code of conduct](CODE_OF_CONDUCT.md).
Security issues are handled as described in [SECURITY.md](SECURITY.md).

## Acknowledgements

Xylarium was developed with AI assistance: design, code and documentation
were produced with AI coding agents (Anthropic's Claude).

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
