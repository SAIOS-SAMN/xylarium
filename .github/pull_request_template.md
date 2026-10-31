## What and why

<!-- Describe what this changes and why. Link the issue, if any. -->

## Readings (model changes)

<!-- The `xylarium observe` reading before and after the change, with the
seed and the command used, and a short comparison. -->

## Tests run

- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `cargo test --workspace --profile fast` passes.
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` passes.
- [ ] `scripts/check-docs` passes.
- [ ] `scripts/check-style` passes.

## Documentation

- [ ] `CHANGELOG.md` is updated under "Unreleased".
- [ ] `docs/model.md` or `docs/civilizations.md` is updated for a law change.
- [ ] `docs/roadmap.md` is updated when an item opens or closes.
