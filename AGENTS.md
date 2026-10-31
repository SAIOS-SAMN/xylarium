# AGENTS.md

Orientation for AI coding agents working in this repository. Human
contributors: start with [docs/concepts.md](docs/concepts.md) and
[CONTRIBUTING.md](CONTRIBUTING.md); everything here also applies to you.

## What this repository is

Xylarium is a deterministic universe simulator in Rust: one seed founds a
universe at a central core, and galaxies, stars, worlds, life and
civilizations are derived from it in exact arithmetic. It is the kernel of a
grand-strategy game in which external AI agents hold seats. The physics is
the project's own model; terms such as bell, throat or firmament are defined
in [docs/glossary.md](docs/glossary.md), not by their everyday meaning.

## How to find things

1. Look the term up in [docs/glossary.md](docs/glossary.md): each entry
   gives a plain meaning, the defining Rust item and the doc section that
   explains it.
2. Open the defining module and read its `//!` header: what it computes,
   its inputs and units, its invariants. Module headers are the source of
   truth; the Markdown docs summarise them.
3. Search by identifier, not by prose: `grep -rn "fn manifold_worlds" crates/`.
4. Formulas with their code locations: [docs/EQUATIONS.md](docs/EQUATIONS.md).
   Planned work and its order: [docs/roadmap.md](docs/roadmap.md).

## Map

| Concern | Crate / module | Doc |
|---|---|---|
| Exact numbers: `Q`, `Encl`, `Dy`, CGA, rotors, units, streams | `crates/xyl-core/src` | [roadmap.md#mathematics](docs/roadmap.md#mathematics) |
| Founding: Big Bang, bounce, elements, atoms, firmament, beads, galactic cores | `crates/xyl-cosmos/src/{bang,bounce,elements,atoms,compound,fibres,formation,cores,chain,package}.rs`, `circuit/` | [model.md](docs/model.md) sections 2-4 |
| Galaxy, stars, chemistry | `galaxy.rs`, `stars.rs`, `chem.rs`, `turning.rs`, `pulse.rs`, `field.rs` | [model.md](docs/model.md) section 5 |
| Discs, worlds, moons, world state | `disc.rs`, `systems.rs`, `manifold.rs`, `bodies.rs`, `worlds.rs`, `hydro.rs` | [model.md](docs/model.md) section 6 |
| Life and the forecast of minds | `life.rs`, `minds.rs` | [model.md](docs/model.md) section 7 |
| Universe in play, peoples, carry, annals | `crates/xyl-kernel/src/{universe,cosmic,civ/}` | [civilizations.md](docs/civilizations.md) |
| Knowledge, growth, transit, seats, clocks | `knowledge.rs`, `growth.rs`, `transit.rs`, `propulsion.rs`, `seats.rs`, `percept.rs`, `chronometry.rs` | [civilizations.md](docs/civilizations.md), [network.md](docs/network.md) |
| CLI and library: `genesis`, `run`, `observe`, `run::Link` | `crates/xylarium/src/{run,observe}.rs` | [architecture.md](docs/architecture.md) |

## Rules that must hold

- No floating point. Hold values as `Q`; use `Encl` for transcendentals and
  `Dy` for intermediate readings, gridded back to `Q` before storing.
  `clippy::float_arithmetic` is denied.
- Refuse, do not panic: out-of-domain input returns `None` or `Err`.
  `unwrap`, `expect` and `panic!` are denied outside tests.
- No time-stepping loops. Change is a closed form of `t` or an event found
  by `xyl_core::event`.
- Randomness only from `xyl_core::Stream::derive(seed, &[parts])`.
- One model per quantity: replace in place and delete the old code; never
  add a parallel version (`foo2`, `new_foo`).
- Derive from lower tiers. An underived constant is a declared ruler and is
  listed in [roadmap.md#declared-rulers](docs/roadmap.md#declared-rulers).
- Tests assert invariants (conservation, identities, bounds, monotonicity,
  refusal), never a number the model produced.
- Never found a universe inside a test; read `crate::genesis::bang()` and
  `crate::genesis::chem()` (xyl-cosmos, test only).

## Commands and costs

| Command | Time | Notes |
|---|---|---|
| `cargo build --profile fast` | 1-3 min cold | optimised, incremental; use for iteration |
| `cargo test --profile fast -p xyl-core` | seconds | |
| `cargo test --profile fast -p xyl-kernel` | ~1-2 min | |
| `cargo test --profile fast -p xyl-cosmos <module>::` | seconds to minutes | run only the module changed |
| `cargo test -p xyl-cosmos` (dev profile) | ~20 min | founds the test genesis if the founding digest changed |
| `xylarium genesis --home DIR --seed N` | minutes | founds a universe; refuses an existing home |
| `xylarium observe --home DIR <word>` | seconds to minutes | read-only; list of words in [architecture.md#readings](docs/architecture.md#readings) |

- The `fast` profile inherits `release`, which turns integer overflow checks
  off. Run changed modules' tests once in the `dev` profile before
  finishing.
- Editing any founding file (`crates/xyl-cosmos/build.rs` lists them; all of
  `xyl-core` is included), even a comment, changes `XYL_FOUNDING_DIGEST` and
  forces the next genesis or test run to found the universe again.
- Before finishing: `cargo fmt --all`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc
  --workspace --no-deps`, and `scripts/check-docs`.

## Documentation duties

A change to a law updates, in the same commit: the module header,
[EQUATIONS.md](docs/EQUATIONS.md), [model.md](docs/model.md) or
[civilizations.md](docs/civilizations.md), [roadmap.md](docs/roadmap.md),
the glossary for any new term, and the Unreleased section of
[CHANGELOG.md](CHANGELOG.md). Write plain technical prose: what is
computed, from what, in which units, where. `scripts/check-docs` verifies
that every link and every backticked code path in the docs resolves.
