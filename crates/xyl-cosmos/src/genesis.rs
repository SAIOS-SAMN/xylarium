//! The one genesis the tests stand on. A universe is founded one way: its
//! Big Bang (`BigBang::of`), the founding kept in a home's founding cache
//! (`circuit::cache_in`), as `xylarium genesis` founds a game's home. The
//! tests found seed 7 once, into one home in the workspace's target
//! directory, and every test reads that genesis; no test founds its own.
//! A second test binary, or a later run with the same laws, loads the
//! founding from the cache instead of founding it again.

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
        // the build's target directory (CARGO_TARGET_DIR when set), so the
        // genesis is kept beside the build and found again by a later run
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
