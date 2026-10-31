# Development

This guide is for contributors working on the code. It describes the standards
the lints enforce, the modelling conventions every law follows, how to extend a
tier, how to build and test efficiently, and how to keep the documentation
consistent with the code. The pull-request workflow is in
[CONTRIBUTING.md](../CONTRIBUTING.md).

## Code standards

The workspace lints (`Cargo.toml`, `clippy.toml`) enforce the following
rules:

- `unsafe_code` is denied.
- Floating point is denied (`float_arithmetic`, `cast_precision_loss`,
  `cast_possible_truncation`). Use `Q` for held values, `Encl` for certified
  transcendentals and `Dy` for bounded intermediate readings.
- `unwrap`, `expect`, `panic`, `todo` and `unimplemented` are denied outside
  tests. The kernel refuses invalid input (returns `None` or `Err`) rather
  than aborting.
- `missing_docs` warns, so every public item is documented, with units for
  physical quantities and the conditions under which it returns `None` or
  `Err`.

## Code style

The following conventions apply in addition to `rustfmt` (run
`cargo fmt --all`):

- **Lines.** A line has at most 100 columns, string literals included.
  Long `format!` strings are split across `concat!` pieces, or output rows
  are built with small helpers. `scripts/check-style` checks this.
- **Modules.** A file over about 800 lines is split into a directory module
  (`worlds/mod.rs` with `worlds/climate.rs`, ...) along its sections; the
  public paths stay the same through `pub use`.
- **Functions.** A function over about 100 lines is split into named steps.
  A closure longer than a few lines becomes a named function.
- **Names.** Items are named for what they compute (`stellar_mass`,
  `manifold_worlds`), not by single letters. Numbers are built with the
  shared constructors - `Q::int`, `Q::ratio`, `Q::new` (checked),
  `Dy::int`, `Dy::ratio`, `Dy::sci`, `elements::decimal` - never with
  private one-letter wrappers. Single-letter bindings are for the symbols of
  a formula written in the comment above them (`m`, `r`, `delta`).
- **Comments.** A `//` comment says why the code does what it does, or
  which formula a block evaluates (with its symbols), in academic technical
  prose: complete sentences that start with a capital letter and end with a
  full stop, third person, facts only, ASCII only. Formulas sit inside the
  sentence ("The amplitude is `A = vz / nu`."). A comment does not narrate
  what the next line obviously does, does not use figurative language or
  personification, and names what is meant rather than writing "its own".
  Model vocabulary is used as defined in [glossary.md](glossary.md).
  `scripts/check-style` checks capitalisation, sentence endings and "its
  own".
- **Doc comments.** Every public item has a `///` comment that states what it
  returns, its units, and when it returns `None` or `Err`. Entry points carry an
  `# Examples` section.
- **Tests.** Test names state the property checked
  (`outflow_conserves_founded_mass`), in snake case, at most about 60
  characters.

## Modelling conventions

- Express change as closed forms of `t` (orbits, rotations, circulation, the
  logistic) or as events found by the certified crossing solver
  (`xyl_core::event`). Do not add fixed-timestep loops.
- Places are bearings on spheres; distances are arcs (`chain::Place::arc`).
  Do not introduce grids or Euclidean distances.
- Work in the core's units, `G = c = M_core = 1`, and in the unit of each
  tier on the sexagesimal ladder (`xyl_core::units`).
- Prefer deficit shares `M / (R + M)` and linear laws in shares and angles.
- Derive quantities from lower tiers. A constant that cannot yet be derived
  is a declared ruler: name it in code and list it in
  [roadmap.md](roadmap.md).
- One law per quantity. Replace a model in place and delete the old one; do
  not add parallel versions.
- Draw randomness only through `xyl_core::Stream`, derived from the seed
  and a label path.

## Extending a tier

A tier is a layer of the simulation that reads only the tiers beneath it
(the table in [architecture.md, Layers](architecture.md#layers)). Adding or
changing a law means touching one tier, its tests, one observer reading and
its documentation, in a single change.

### 1. Locate the inputs

Identify the tier in the architecture table and read the modules beneath
it. A new law takes its inputs from quantities those modules already
produce, in their units; search for an existing function before writing a
new one (`grep -rn "pub fn" crates/xyl-cosmos/src`). Where a model is
replaced, change it in place and delete the old code path: there is one
implementation per quantity.

### 2. Write the law

- Hold values as `Q`. Use `Encl` (`xyl_core::certified`) where a decision
  or a held value depends on a transcendental, and `Dy`
  (`xyl_core::dyadic`) for heavy intermediate readings, gridded back to
  `Q` before they are stored (`xyl_core::certified::down`).
- Express time dependence as a closed form of `t` or as a crossing found by
  `xyl_core::event`. No loop advances a state by a time step.
- Read quantities in the unit of the tier (`xyl_core::units::Tier`) and
  convert between tiers with `Tier::in_rg`.
- Return `Option` or `Result` for inputs outside the law's domain; the
  kernel refuses rather than panics (`unwrap` and `expect` are denied
  outside tests).
- A constant that is not yet derived from a lower tier is a declared ruler: give
  it a named constant or a field of the module's `Rulers`, document its source,
  and add it to [roadmap.md, Declared rulers](roadmap.md#declared-rulers).
- Draw randomness only from `xyl_core::Stream::derive(seed, &[parts])`, with a
  label path unique to the call site.

### 3. Keep the founding cached

`crates/xyl-cosmos/build.rs` hashes two digests at build time:

| Digest | Covers | Keys |
|---|---|---|
| `XYL_FOUNDING_DIGEST` | `bang`, `bounce`, `elements`, `atoms`, `compound`, `fibres`, `formation`, `cores`, `chain`, `package`, `circuit/`, `data/` and all of `xyl-core/src` | The founding cache (`founding-*` in a home, `target/genesis` for tests) |
| `XYL_SOURCE_DIGEST` | Every `.rs` and `.tsv` file of `xyl-cosmos` and `xyl-core` | The forecast caches (`forecast-*`, `forecast-stars/`, `bells-*`) |

Both digests hash file contents, comments included. An edit to any
founding file invalidates every cached founding, and the next genesis or
test run founds the universe again, which takes minutes. Keep changes above
the founding out of those files so homes and the test genesis are reused.

### 4. Test the law's invariants

Tests are placed in a `#[cfg(test)] mod tests` at the end of the module. Tests
that need a founded universe read the shared test genesis
(`crates/xyl-cosmos/src/genesis.rs`, seed 7) and never found a universe:

```rust
#[test]
fn a_galaxy_ring_conserves_its_metals() {
    let bang = crate::genesis::bang();
    let chem = crate::genesis::chem();
    let made = chem.metals(&Rulers::r0(), &Rulers::today_gyr()).unwrap();
    assert!(made.signum() >= 0, "metals {made}");
}
```

Assert properties of the law rather than numbers it produces:
conservation (inputs equal outputs plus residue, exactly), identities,
bounds, monotonicity in an input, and refusal of out-of-domain input. A
test does not pin a measured value or a previous run's output. Run the
tests of the module changed, not the workspace (see
[Building and testing](#building-and-testing)), and run them at least once
in the `dev` profile: `fast` inherits `release`, which disables integer
overflow checks.

### 5. Read the change on a running universe

Every tier has a reading in `crates/xylarium/src/observe/mod.rs`, dispatched by
word in `observe::observe` and listed in the module header, the
unknown-word message and [architecture.md, Readings](architecture.md#readings).
Extend the tier's reading, or add a word, so the new quantity is printed.
Take the reading before and after the change on the same home and compare:

```
target/release/xylarium observe --home var/universe-7 worlds 200
```

A reading is read-only and can run beside `xylarium run`.

### 6. Document it in the same change

- [EQUATIONS.md](EQUATIONS.md) records the formula, its code location and
  its status (built, partial or placeholder).
- [model.md](model.md) or [civilizations.md](civilizations.md) states how
  the law follows from the tier below.
- In [roadmap.md](roadmap.md), the item moves from Next to Landed, and
  declared rulers are added or removed.
- [CHANGELOG.md](../CHANGELOG.md) receives an entry under Unreleased.

### Checklist

- [ ] Inputs come from existing lower-tier functions, and no model is
      duplicated.
- [ ] Held values are `Q`, no floating point is used, and out-of-domain
      input is refused.
- [ ] Founding files are unchanged unless the change is to the founding.
- [ ] Invariant tests are added, and the module tests pass in `dev` and
      `fast`.
- [ ] `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`
      and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
      report no problems.
- [ ] The observer reading is compared before and after the change.
- [ ] EQUATIONS, model or civilizations, roadmap and changelog are
      updated.

## Building and testing

```
cargo build --release
cargo build --profile fast     # optimised, incremental, for iteration
```

Run targeted tests rather than the whole workspace while iterating:

```
cargo test --profile fast -p xyl-cosmos systems::
cargo test --profile fast -p xyl-kernel transit::
```

Tests in `xyl-cosmos` that need a founded universe share one test genesis,
`genesis::universe` (seed 7, `crates/xyl-cosmos/src/genesis.rs`). Its
founding is cached in `target/genesis`, so it is founded once and every
later test run under the same founding digest reads it from the cache. Do
not found a universe inside a test.

### Run times

`xylarium genesis` founds a universe, which takes minutes, and then runs the
forecast of minds over the sample stars. Later starts of the same home
reuse the cached founding and forecast while their digests are unchanged
(see [architecture.md](architecture.md#determinism-and-caching)). The
first test run founds the shared test genesis into `target/genesis`; later
test runs under the same founding digest reuse it.

### Helper scripts

On a shared machine, two tracked helper scripts are available. Both use
`flock` (util-linux), so they run on Linux; they are optional, and every
command works without them.

| Script | Use |
|---|---|
| `scripts/heavy CMD...` | Runs a build or test suite under a machine-wide lock at low priority |
| `scripts/long CMD...` | Runs a long job (genesis, a deep-time reading) under a separate lock, with at most 3 worker threads |

```
scripts/heavy cargo test --profile fast -p xyl-kernel
scripts/long target/release/xylarium observe --home var/universe-7 universe
```

## Verification

Tests guard invariants: conservation (balances that close), refusal of
invalid input, monotonicity and bounds of laws. Model behaviour is verified on a
running universe with `xylarium observe`, comparing readings before and
after a change. Byte-for-byte replay is not a verification criterion.

## Documentation

Update [model.md](model.md) or [civilizations.md](civilizations.md) in the
same change as a law, and [roadmap.md](roadmap.md) when an item opens or
closes.
