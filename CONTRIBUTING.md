# Contributing

This document describes how to set up a development environment, the checks
a change must pass, and how changes are proposed.

## Development environment

- Rust 1.85 or later is required (`rustup toolchain install stable`), with
  the `rustfmt` and `clippy` components.
- The supported platforms are Linux and macOS. The build uses no system
  libraries beyond the Rust toolchain.

From a clone of the repository:

```
git clone https://github.com/SAIOS-SAMN/xylarium
cd xylarium
cargo build --release
```

The `fast` profile builds an optimised binary with incremental compilation and
is the recommended profile for iteration:

```
cargo build --profile fast
```

## Getting started

The following steps lead from a clone to a first change.

1. Read [docs/concepts.md](docs/concepts.md) for the model's layers and
   vocabulary, and keep [docs/glossary.md](docs/glossary.md) open.
2. Build and run the fast tests of the numerics crate (seconds):

   ```
   cargo build --profile fast
   cargo test --profile fast -p xyl-core
   ```

3. Found a universe and take a reading (minutes):

   ```
   target/fast/xylarium genesis --home var/universe-7 --seed 7
   target/fast/xylarium observe --home var/universe-7 worlds 50
   ```

4. Pick a task from the [good first
   issue](https://github.com/SAIOS-SAMN/xylarium/labels/good%20first%20issue)
   label or the roadmap's Next lists, read the module's `//!` header, and
   follow [Extending a tier](docs/development.md#extending-a-tier).

The first `cargo test -p xyl-cosmos` founds the shared test genesis, which
takes about 20 minutes in the `dev` profile; later runs reuse it while the
founding's sources are unchanged.

## Verification

Every change must pass all of the following:

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --profile fast
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
scripts/check-docs
scripts/check-style
```

Run them locally before opening a pull request; the repository runs no
hosted CI.

While iterating, run the tests of the changed modules only
(`cargo test --profile fast -p <crate> <module>::`). Tests that need a
founded universe share one test genesis; see
[docs/development.md](docs/development.md).

A change to a law of the model is also checked on a running universe with
`xylarium observe`: take one reading before and one after the change, and
summarise the comparison in the pull request.

The optional helper scripts `scripts/heavy` and `scripts/long` serialise
builds and long runs on shared machines. They need `flock` (util-linux, so
Linux); every command works without them. See
[docs/development.md](docs/development.md#helper-scripts).

## Code standards

The code standards, the workspace lints and the modelling conventions are
described in [docs/development.md](docs/development.md), together with the
steps in [Extending a tier](docs/development.md#extending-a-tier).

## Commit messages

- The summary line has at most 72 characters and uses the imperative mood
  ("Add the exodus to the carry", not "Added ...").
- A blank line follows, then a body that explains what changed and why,
  wrapped at 72 columns. The body references readings or issues where
  relevant.
- Each commit holds one logical change.

## Development workflow

Every change, by a maintainer or anyone else, reaches `main` through a
pull request. `main` is protected: the maintainer merges
([MAINTAINERS.md](MAINTAINERS.md)).

1. **Pick or open an issue.** The open work is listed in the
   [issues](https://github.com/SAIOS-SAMN/xylarium/issues) and in
   [docs/roadmap.md, Open items](docs/roadmap.md#open-items). Newcomers start
   with [good first
   issue](https://github.com/SAIOS-SAMN/xylarium/labels/good%20first%20issue).
   Comment on the issue to say you are taking it. A change to a law of the model
   or a public API starts as an issue with the proposal template, describing the
   derivation and the expected effect on readings.
2. **Specify before coding.** A law's formula goes into
   [docs/model.md](docs/model.md) and [docs/EQUATIONS.md](docs/EQUATIONS.md)
   first, derived from the founding and the laws already built
   ([Extending a tier](docs/development.md#extending-a-tier)); no Earth,
   Sun or Milky Way value and no fitted constant.
3. **Branch from `main`**, named for the issue (`3-fresh-home-epoch`).
   Make the change with its tests and documentation in the same branch.
4. **Verify.** Run the checks above and the tests of the modules you
   changed. For a model change, take the same `xylarium observe` reading on
   the same seed before and after the change.
5. **Open the pull request** with the template: what and why,
   `Closes #<issue>`, the checks run and, for model changes, the readings
   before and after. Update [CHANGELOG.md](CHANGELOG.md) under "Unreleased"
   and the roadmap when an item opens or closes.
6. **Review and merge.** The maintainer reviews the pull request against
   the rules in [AGENTS.md](AGENTS.md) and
   [docs/development.md](docs/development.md) and merges it. An issue
   labelled `needs-decision` waits for the operator's decision before work
   is merged.
7. **Release.** Releases are tagged from `main` only where the readings
   show the universe working (stars placed, worlds forming); each release's
   notes are its CHANGELOG section. The next release is the
   [v0.4.0 milestone](https://github.com/SAIOS-SAMN/xylarium/milestone/1).

Labels:

| Label | Meaning |
|---|---|
| `defect` | something does not work as specified |
| `proposal` | a proposed change to a law, interface or design |
| `track` | a body of work across several pull requests |
| `law` | changes a law of the model |
| `docs` | documentation |
| `needs-decision` | waits on a decision by the operator |
| `cross-project` | affects mycelium or netstratum as well |
| `good first issue`, `help wanted` | a place to start; help is welcome |
| `area/<crate>` | the crate the work is in |

Report security vulnerabilities privately as described in
[SECURITY.md](SECURITY.md), not in a public issue. Everyone taking part
follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## Licence

By contributing you agree that your contributions are licensed under the
Apache License, Version 2.0, as described in [LICENSE](LICENSE).
