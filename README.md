# xylarium

xylarium is a deterministic universe-simulation kernel for a grand-strategy
space game in which sovereign AI minds operate civilizations.

One seed and a founding package found a universe from its core. Galaxies,
stars, worlds and life follow from laws evaluated in exact arithmetic.
Peoples wake where life climbs to minds. A carry advances their knowledge,
numbers and histories. Annals record their turnings, eminent figures, rises
and falls. Seats for sovereign, steward, progenitor and ancient are held by
the universe's own rule or taken by keyed minds through `run::Link`. Ley
lines, realms and stewards form the network peoples travel. Mu, the
mycelium network's unit, enters a universe only through an uplink, which is
not part of this repository.

This project was developed with substantial AI assistance: design, code and
documentation were produced with AI coding agents (Anthropic's Claude).

## Grand-strategy scope

| Element | Built | Roadmap |
|---|---|---|
| AI minds operate civilizations | seats for sovereign, steward, progenitor and ancient (`seats`), held by the universe's own rule until a keyed mind takes one through `run::Link` (`universe::Input::Take`); typed percepts for seats on a people (`percept`, `lexicon`); the `Focus` and `Handle` decisions (`Universe::decide`) | seats keyed by subject; a holder check in `decide`; steward percepts; seat commands on the CLI; players |
| Histories | annals of emergence, falls, ruins and focus (`cosmic::Annal`); turnings and eminent figures (`civ::history`) | |
| Realms, ley lines, stewards, quarantine | ley lines built by ancient houses, constellations, realms as shells of the network, steward handling, quarantine by share, folds of space (`transit`) | colonies of other stars over ley lines |
| World building | founding, galaxies, stars, discs, worlds on manifolds, moons, air, seas, life and minds (`xyl-cosmos`) | a world layer read wholly in the core's units |
| Space trade | the toll law: floor, ceiling and toll quoted in energy (`transit::quote`, `transit::tithe`) | traffic carried over lanes and tolls accrued; exchange and trade routes |
| Growth economies | bodies, metabolism, harvest, housing and numbers derived from physics (`growth`, `civ::housing`) | exchange and its institutions between peoples |
| Mu | | mu enters a universe only through an uplink, which is not part of this repository |
| Modifiable physics | `xyl_cosmos::Laws` through the library API (`Universe::create`) | laws supplied by the package to `genesis`, `run` and `observe` (today `Laws::default()`) |
| Diplomacy and conflict | | conflict and diplomacy from the shared decision rule |

Items and their order are tracked in [docs/roadmap.md](docs/roadmap.md).

## Design principles

- Exact arithmetic. Held state is exact rationals (`Q`) or certified interval
  enclosures (`Encl`). Floating point is denied by lint. Transcendentals are
  evaluated on demand to certified bounds.
- Closed forms over stepping. Motion is expressed as orbits, rotations and
  circulations evaluated at time `t`. The engine advances from event to
  event, not frame to frame.
- Curved geometry. Positions are bearings on spheres and manifolds; distances
  are arcs. There is no grid.
- Derived quantities. Habitability, a species' traits, a people's housing and
  the cost of a passage are derived from lower layers. Constants that are not
  yet derived are declared rulers, listed in the roadmap.
- Lazy evaluation. Systems, worlds and peoples are materialised only when an
  event or a reading needs them.
- Units from the core. `G = c = M_core = 1`: the universal core holds the
  universal state, 1. The firmament (the founding's held kinds) carries that
  state, and galactic cores are measured against the universal core.
- The physics is the game's own fiction, built from a consistent internal
  rule set. It borrows terminology from physics but makes no claim about the
  natural world.

## Requirements

- Rust 1.82 or later. All dependencies come from crates.io.
- Several cores for genesis and deep-time readings; see
  [docs/development.md](docs/development.md#run-times).

## Build

```
cargo build --release
```

## Quick start

```
# found a universe from seed 7 in a new home directory
target/release/xylarium genesis --home var/universe-7 --seed 7

# read it: the universe, its peoples, the substrate at 14 Gyr, the ley network
target/release/xylarium observe --home var/universe-7 universe
target/release/xylarium observe --home var/universe-7 peoples
target/release/xylarium observe --home var/universe-7 core 14
target/release/xylarium observe --home var/universe-7 ley 8

# play it live for 1000 rings of the core's bell, checkpointing every 100
target/release/xylarium run --home var/universe-7 --for 1000 --save-every 100
```

| Option | Default | Meaning |
|---|---|---|
| `--home DIR` | `var/universe-7` | the universe's home directory |
| `--seed N` | `7` | the seed every stream derives from |
| `--stars N` | `400` | sample stars whose forecast of minds is worked out |
| `--threads N` | `2` | worker threads for the forecast |
| `--sight census\|cores` | `census` | `census` unrolls every sample star; `cores` only what the cores see |
| `--for N` | none | rings to play (`run`); without it, play until interrupted |
| `--save-every K` | `1` | rings between checkpoints (`run`) |
| `--velocity V` | `0` | rings per wall-clock second (`run`); 0 is unthrottled |

`genesis` writes the founding, the genesis record and the first checkpoint,
and refuses a home that already holds a genesis. `observe` is read-only and
may run beside `run`. Every reading word, its arguments and what it shows
are listed in [docs/architecture.md, Readings](docs/architecture.md#readings).

### Running your own kernel

A home directory is one universe. It holds the genesis record, the
checkpoint, and caches of what the laws regenerate:

- `founding-*`: the founding, keyed by the founding's source digest and the
  torsion share. Changes to layers above the founding (discs, worlds, life,
  the kernel) reuse it, so a rebuilt binary starts from the cached founding.
- `forecast-*`, `forecast-stars/`, `bells-*`: the forecast of minds and the
  heavy stars, keyed by the digest of all `xyl-cosmos` and `xyl-core`
  sources.

Deleting a cache changes no result; the next start recomputes it. A fresh
seed needs a fresh home. To embed the kernel in another program, depend on
the `xylarium` crate and call `run::genesis`, `run::run_with` with your own
`run::Link`, or `observe::observe`. Extending a tier is described in
[docs/development.md](docs/development.md).

## Workspace

| Crate | Responsibility |
|---|---|
| `xyl-core` | exact rationals `Q`, certified enclosures, dyadics, conformal geometric algebra `Cl(n+1,1)`, rotors, units and tiers, measure, the event solver, deterministic streams, coordinates |
| `xyl-cosmos` | the founding (bang, bounce, elements, atoms, compounds, fibres, formation, cores, circuit, chain, package) and the layers above it (galaxy, stars, chemistry, discs, systems, manifolds, bodies, worlds, life, minds, pulse, field) |
| `xyl-kernel` | the universe in play: the event engine, peoples, the civilization carry, knowledge, growth, seats, percepts, transit, propulsion, chronometry, checkpoints |
| `xylarium` | the library and binary: `genesis`, `run`, `observe`; `run::Link` is the seam an uplink attaches to live play |

See [docs/architecture.md](docs/architecture.md) for layers, data flow, the
event engine, caching and state.

## Relationship to mycelium

[mycelium](https://github.com/SAIOS-SAMN/mycelium) (tag `v0.2.0`) is a
separate public project. xylarium plugs into it as an SDK client domain
through the `xylarium::run::Link` seam: an uplink runs live play with
`run::run_with` and a `Link` beside it, reads and answers the universe after
each ring, records what it ships at each checkpoint, and reports the last
final clock height it heard. No crate in this workspace depends on mycelium
today, and the uplink is not part of this repository. Without a link,
`xylarium run` plays locally. The intended contract is summarised in
[docs/network.md](docs/network.md).

## Design direction

The universal core's mass (`package::hole_mass`) is the one bridge to SI and
is used for display. Calibrations carried over from measured values are
being replaced by quantities derived from the founding: the SI seam
`package::Si`, the planet layer's units, the chemistry rulers, the measured
element rows Z 1-99 in `data/elements.tsv` (Z 100-150 are predicted rows,
and the table continues past 150 by the periodic law), the volatile species
in `worlds`, and `Laws` expressed as ratios to measured constants. The
order of that work is in [docs/roadmap.md](docs/roadmap.md);
`scripts/calibration-anchors` counts the remaining anchors per crate.

## Status

- Stability: pre-alpha. The physical model and the civilization layer are
  implemented and deterministic.
- Versioning: [Semantic Versioning](https://semver.org). Until 1.0.0, minor
  versions may change public APIs and the model's laws. Changes are recorded
  in [CHANGELOG.md](CHANGELOG.md).
- Tests guard conservation (books that close), refusal of invalid input, and
  the bounds and monotonicity of laws. Model behaviour is verified on running
  universes with `xylarium observe`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the development setup, the
verification commands and the pull-request workflow.

## Documentation

- [docs/architecture.md](docs/architecture.md): layers, data flow, the event engine, state
- [docs/model.md](docs/model.md): the physical model, from the founding to worlds
- [docs/civilizations.md](docs/civilizations.md): life, peoples, knowledge, transit
- [docs/network.md](docs/network.md): the intended network contract
- [docs/glossary.md](docs/glossary.md): terms
- [docs/development.md](docs/development.md): conventions, lints, testing
- [docs/roadmap.md](docs/roadmap.md): open work

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
