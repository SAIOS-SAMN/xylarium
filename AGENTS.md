# AGENTS.md

This file orients AI coding agents working in this repository. Human
contributors start with [docs/concepts.md](docs/concepts.md) and
[CONTRIBUTING.md](CONTRIBUTING.md); the rules below apply to them as well.

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
4. Formulas with their code locations are listed in
   [docs/EQUATIONS.md](docs/EQUATIONS.md). Planned work and its order are
   listed in [docs/roadmap.md](docs/roadmap.md).

## Map

A module `foo` is `src/foo.rs` or, when split, the directory `src/foo/`
(entry `mod.rs`).

| Concern | Crate / module | Doc |
|---|---|---|
| Exact numbers: `Q`, `Encl`, `Dy`, CGA, rotors, units, streams | `crates/xyl-core/src` | [roadmap.md#mathematics](docs/roadmap.md#mathematics) |
| Founding: origin, bounce, elements, atoms, firmament, beads, galactic cores | `xyl-cosmos` modules `origin`, `bounce`, `elements`, `atoms`, `compound`, `fibres`, `formation`, `cores`, `chain`, `package`, `circuit` | [model.md](docs/model.md) sections 2-4 |
| Galaxy, stars, chemistry | `xyl-cosmos` modules `galaxy`, `stars`, `chem`, `turning`, `pulse`, `field` | [model.md](docs/model.md) section 5 |
| Discs, worlds, moons, world state | `xyl-cosmos` modules `disc`, `systems`, `manifold`, `bodies`, `worlds`, `hydro` | [model.md](docs/model.md) section 6 |
| Life and the forecast of minds | `xyl-cosmos` modules `life`, `minds` | [model.md](docs/model.md) section 7 |
| Universe in play, peoples, carry, annals | `xyl-kernel` modules `universe`, `cosmic`, `civ` | [civilizations.md](docs/civilizations.md) |
| Knowledge, growth, transit, seats, clocks | `xyl-kernel` modules `knowledge`, `growth`, `transit`, `propulsion`, `seats`, `percept`, `chronometry` | [civilizations.md](docs/civilizations.md), [network.md](docs/network.md) |
| CLI and library: `genesis`, `run`, `observe`, `run::Link` | `xylarium` modules `run`, `observe` | [architecture.md](docs/architecture.md) |

## Rules that must hold

- No floating point. Hold values as `Q`; use `Encl` for transcendentals and
  `Dy` for intermediate readings, gridded back to `Q` before storing.
  `clippy::float_arithmetic` is denied.
- Refuse, do not panic: out-of-domain input returns `None` or `Err`.
  `unwrap`, `expect` and `panic!` are denied outside tests.
- No time-stepping loops. Change is a closed form of `t` or an event found
  by `xyl_core::event`.
- Randomness only from `xyl_core::Stream::derive(seed, &[parts])`.
- Every turn names its axis (the plane, by its generators) and is stated in
  degrees; its physics is torsion, as a degree of modification, structure or
  perturbation; its return is stated with it, read from the gravitational
  density (the deficit at the return's reach), never assumed equal
  ([model.md](docs/model.md#turns-axis-degrees-and-the-return)).
- Every share reads its reach and hold in one calibre: tier-bound,
  conformal (a change of tier dilates both together, never one) and
  share-preserving; every equation and every reading states its calibre,
  and a reading is checked for magnitude in it, not only for having moved
  ([model.md](docs/model.md#calibre-and-dimensional-capacity)).
- Every pair reading (a separation, an arc, a closeness: every `a, b` or
  `x, y`) keeps its dimensional capacity: coordinates of one origin, read
  in one calibre, related by the pivot (the relative versor by
  conjugation), read plane by plane in degrees with each winding and each
  return, never one scalar, never a square or a root
  ([model.md](docs/model.md#pairs-the-pivot-per-plane)).
- One model per quantity: replace in place and delete the old code; never
  add a parallel version (`foo2`, `new_foo`).
- Derive from lower tiers. An underived constant is a declared ruler and is
  listed in [roadmap.md#declared-rulers](docs/roadmap.md#declared-rulers).
- A law that is not built is refused (`None`) or registered as a stand-in
  in [assumptions.md](docs/assumptions.md), with the comment
  `Stand-in S###` at every site that uses it. Never take the nearest
  quantity of another body, tier or event in its place, and never read a
  missing value as a law value. A new consumer of a stand-in cites its ID;
  a refactor keeps the marker; the marker leaves with the PR that builds
  the law. `scripts/check-stand-ins` (run by `scripts/check-docs`) keeps the
  register and the markers in agreement.
- Tests assert invariants (conservation, identities, bounds, monotonicity,
  refusal), never a number the model produced.
- Never found a universe inside a test; read `crate::genesis::origin()` and
  `crate::genesis::chem()` (xyl-cosmos, test only).

## Commands and costs

| Command | Time | Notes |
|---|---|---|
| `cargo build --profile fast` | 1-3 min cold | Optimised and incremental; the profile for iteration |
| `cargo test --profile fast -p xyl-core` | Seconds | |
| `cargo test --profile fast -p xyl-kernel` | ~1-2 min | |
| `cargo test --profile fast -p xyl-cosmos <module>::` | Seconds to minutes | Runs only the module changed |
| `cargo test -p xyl-cosmos` (dev profile) | ~20 min | Founds the test genesis if the founding digest changed |
| `xylarium genesis --home DIR --seed N` | Minutes | Founds a universe; refuses an existing home |
| `xylarium observe --home DIR <word>` | Seconds to minutes | Read-only; the words are listed in [architecture.md#readings](docs/architecture.md#readings) |

- The `fast` profile inherits `release`, which turns integer overflow checks
  off. Run changed modules' tests once in the `dev` profile before
  finishing.
- A code edit to any founding file (`crates/xyl-cosmos/build.rs` lists
  them; all of `xyl-core` is included) changes `XYL_FOUNDING_DIGEST` and
  forces the next genesis or test run to found the universe again; an edit
  outside the shells' files (`XYL_SHELLS_DIGEST`) reads the cached shells
  and re-derives only the stages above them. Comment edits do not: the
  digests hash code only. Test laws above the founding against the cached
  test genesis by running only the changed modules' tests.
- Before finishing, run `cargo fmt --all`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc
  --workspace --no-deps`, `scripts/check-docs` and `scripts/check-style`.

## Documentation duties

A change to a law updates, in the same commit: the module header,
[EQUATIONS.md](docs/EQUATIONS.md), [model.md](docs/model.md) or
[civilizations.md](docs/civilizations.md), [roadmap.md](docs/roadmap.md),
the glossary for any new term, and the Unreleased section of
[CHANGELOG.md](CHANGELOG.md). Write plain technical prose: what is
computed, from what, in which units, where. `scripts/check-docs` verifies
that every link and every backticked code path in the docs resolves.
