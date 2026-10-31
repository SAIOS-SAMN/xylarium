//! Test-only: the one genesis every test reads (seed 7), founded once per
//! process with `BigBang::of` and `Chemistry::of`.
//!
//! The founding is cached with `circuit::cache_in` in `target/genesis`
//! (under `CARGO_TARGET_DIR` when set), so another test binary, or a later
//! run under the same founding digest, loads it instead of founding again.
//! Outputs: `SEED`, `universe`, `bang` and `chem`. No test founds its own
//! universe.

use crate::chem::Chemistry;
use crate::BigBang;
use std::sync::OnceLock;

/// The seed the test genesis is founded from.
pub const SEED: u64 = 7;

/// The test genesis: seed 7's Big Bang and its chemistry, founded once per
/// process, its founding cached in `target/genesis`.
pub fn universe() -> &'static (BigBang, Chemistry) {
    static G: OnceLock<(BigBang, Chemistry)> = OnceLock::new();
    G.get_or_init(|| {
        // kept in the build's target directory (CARGO_TARGET_DIR when set)
        // so a later run finds the cached genesis
        let target = std::env::var_os("CARGO_TARGET_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target")
            });
        let home = target.join("genesis");
        std::fs::create_dir_all(&home).unwrap();
        crate::circuit::cache_in(&home);
        let bang = BigBang::of(SEED).unwrap();
        let chem = Chemistry::of(&bang).unwrap();
        (bang, chem)
    })
}

/// The test genesis's Big Bang.
pub fn bang() -> &'static BigBang {
    &universe().0
}

/// The test genesis's chemistry.
pub fn chem() -> &'static Chemistry {
    &universe().1
}
