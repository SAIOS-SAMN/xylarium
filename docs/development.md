# Development

## Code standards

Enforced by workspace lints (`Cargo.toml`, `clippy.toml`):

- `unsafe_code` is denied.
- Floating point is denied (`float_arithmetic`, `cast_precision_loss`,
  `cast_possible_truncation`). Use `Q` for held values, `Encl` for certified
  transcendentals, `Dy` for bounded intermediate readings.
- `unwrap`, `expect`, `panic`, `todo` and `unimplemented` are denied outside
  tests. The kernel refuses invalid input (returns `None` or `Err`) rather
  than aborting.
- `missing_docs` warns: every public item is documented, with units for
  physical quantities and the conditions under which it returns `None` or
  `Err`.

## Modelling conventions

- Express change as closed forms of `t` (orbits, rotations, circulation, the
  logistic) or as events found by the certified crossing solver
  (`xyl_core::event`). Do not add fixed-timestep loops.
- Places are bearings on spheres; distances are arcs (`chain::Place::arc`).
  Do not introduce grids or Euclidean distances.
- Work in the core's units, `G = c = M_core = 1`, and in each tier's own unit
  on the sexagesimal ladder (`xyl_core::units`).
- Prefer deficit shares `M / (R + M)` and linear laws in shares and angles.
- Derive quantities from lower tiers. A constant that cannot yet be derived
  is a declared ruler: name it in code and list it in
  [roadmap.md](roadmap.md).
- One law per quantity. Replace a model in place and delete the old one; do
  not add parallel versions.
- Randomness only through `xyl_core::Stream`, derived from the seed and a
  label path.

## Extending a tier

1. Find the tier in [architecture.md](architecture.md) and read the modules
   beneath it. Reuse the machinery that exists; do not duplicate it.
2. Derive the new law from the tier below. Its inputs are quantities the
   lower tiers already produce, in the tier's own units.
3. Land one tier per change. Keep the founding's files untouched unless the
   change is to the founding: a change above the founding keeps
   `XYL_FOUNDING_DIGEST`, so homes and tests reuse the cached founding.
4. Add targeted tests for the law's invariants (books that close, exact
   identities, bounds, monotonicity, refusal of invalid input). Tests read
   the shared test genesis; no test asserts a measured number or the
   model's own output.
5. Take one observer reading on a running universe (`xylarium observe`)
   that shows the change, and compare it with the reading before the change.
6. Update [model.md](model.md) or [civilizations.md](civilizations.md) and
   [roadmap.md](roadmap.md) in the same change.

Files that belong to the founding are listed in `crates/xyl-cosmos/build.rs`
(`bang`, `bounce`, `elements`, `atoms`, `compound`, `fibres`, `formation`,
`cores`, `chain`, `package`, `circuit/`, `data/`, and all of `xyl-core`).

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
(see [architecture.md](architecture.md#determinism-and-caching)). The first test run founds the
shared test genesis into `target/genesis`; later test runs under the same
founding digest reuse it.

### Helper scripts

On a shared machine, two tracked helper scripts are available. Both use
`flock` (util-linux), so they run on Linux; they are optional, and every
command works without them.

| Script | Use |
|---|---|
| `scripts/heavy CMD...` | runs a build or test suite under a machine-wide lock at low priority |
| `scripts/long CMD...` | runs a long job (genesis, a deep-time reading) under a separate lock, with at most 3 worker threads |

```
scripts/heavy cargo test --profile fast -p xyl-kernel
scripts/long target/release/xylarium observe --home var/universe-7 universe
```

## Verification

Tests guard invariants: conservation (books that close), refusal of invalid
input, monotonicity and bounds of laws. Model behaviour is verified on a
running universe with `xylarium observe`, comparing readings before and
after a change. Byte-for-byte replay is not a verification criterion.

## Documentation

Update [model.md](model.md) or [civilizations.md](civilizations.md) in the
same change as a law, and [roadmap.md](roadmap.md) when an item opens or
closes.
