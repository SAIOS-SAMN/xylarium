# Contributing

Thank you for your interest in xylarium. This document describes how to set
up a development environment, the checks a change must pass, and how changes
are proposed.

## Development environment

- Rust 1.85 or later (`rustup toolchain install stable`), with the `rustfmt`
  and `clippy` components.
- Linux or macOS. The build uses no system libraries beyond the Rust toolchain.

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

## Verification

Every change must pass all of the following:

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --profile fast
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

The same checks, and a coverage report from `cargo llvm-cov`, run in CI
(`.github/workflows/ci.yml`) on every push and pull request.

While iterating, run the tests of the modules you change
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
in [docs/development.md](docs/development.md), with the steps for extending
a tier ([Extending a tier](docs/development.md#extending-a-tier)).

## Commit messages

- A summary line of at most 72 characters in the imperative mood
  ("Add the exodus to the carry", not "Added ...").
- A blank line, then a body explaining what changed and why, wrapped at 72
  columns. Reference readings or issues where relevant.
- One logical change per commit.

## Issues and pull requests

Bug reports and feature requests use the issue templates in
`.github/ISSUE_TEMPLATE/`, and pull requests use
`.github/pull_request_template.md`. Report security vulnerabilities
privately as described in [SECURITY.md](SECURITY.md), not in a public
issue. Everyone taking part follows the
[Code of Conduct](CODE_OF_CONDUCT.md).

1. Open an issue first for any change to the model's laws or to a public
   API, describing the motivation and the expected effect on readings.
2. Create a branch from `main`, make the change with tests and documentation
   in the same pull request, and run the verification commands above.
3. Update [CHANGELOG.md](CHANGELOG.md) under "Unreleased", and
   [docs/roadmap.md](docs/roadmap.md) when an item opens or closes.
4. Open the pull request with a summary, the verification results and, for
   model changes, the before-and-after readings.

## Licence

By contributing you agree that your contributions are licensed under the
Apache License, Version 2.0, as described in [LICENSE](LICENSE).
