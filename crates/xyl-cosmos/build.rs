//! Build script: hashes the crate's sources with BLAKE3 and exports two
//! digests as compile-time environment variables.
//!
//! - `XYL_SOURCE_DIGEST`: every `.rs` and `.tsv` file under `src/`, `data/`
//!   and `../xyl-core/src/`, so a cached forecast is read only under the
//!   laws that produced it.
//! - `XYL_FOUNDING_DIGEST`: the subset the founding is derived from (origin,
//!   bounce, elements, atoms, compound, fibres, cores, chain, package,
//!   `src/formation/`, `src/circuit/`, `data/` and xyl-core), which keys the founding
//!   cache; a change to a layer above the founding leaves it unchanged.
//!
//! Each digest covers file names and contents in sorted path order and is
//! cut to 16 hex digits. `watch.rs` and every line containing `watch::` are
//! excluded, since the process monitor does not enter any law; comments and
//! blank lines are excluded, so documentation edits leave both digests
//! unchanged.

use std::path::{Path, PathBuf};

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    for e in read.flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, out);
        } else if matches!(p.extension().and_then(|x| x.to_str()), Some("rs" | "tsv")) {
            out.push(p);
        }
    }
}

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let dirs = [
        root.join("src"),
        root.join("data"),
        root.join("../xyl-core/src"),
    ];
    let mut all = Vec::new();
    for d in &dirs {
        println!("cargo:rerun-if-changed={}", d.display());
        files(d, &mut all);
    }
    all.sort();
    // The founding is derived from these files: the origin, the bounce, the
    // elements, atoms, compounds and fibres, the galactic cores and the chain,
    // the package, the formation cascade, the circuit, the data tables and
    // xyl-core. A change to any other file leaves the founding digest unchanged.
    let founding = |name: &str| {
        const FILES: [&str; 9] = [
            "src/origin.rs",
            "src/bounce.rs",
            "src/elements.rs",
            "src/atoms.rs",
            "src/compound.rs",
            "src/fibres.rs",
            "src/cores.rs",
            "src/chain.rs",
            "src/package.rs",
        ];
        FILES.contains(&name)
            || name.starts_with("src/formation/")
            || name.starts_with("src/circuit/")
            || name.starts_with("data/")
            || name.starts_with("../xyl-core/src/")
    };
    let laws = digest(&root, all.iter(), "xylarium source digest v1");
    let found = digest(
        &root,
        all.iter()
            .filter(|p| founding(&p.strip_prefix(&root).unwrap_or(p).to_string_lossy())),
        "xylarium founding digest v1",
    );
    println!("cargo:rustc-env=XYL_SOURCE_DIGEST={laws}");
    println!("cargo:rustc-env=XYL_FOUNDING_DIGEST={found}");
}

/// The digest of `paths` (relative to `root` by name), the watch excluded.
fn digest<'a>(root: &Path, paths: impl Iterator<Item = &'a PathBuf>, key: &str) -> String {
    let mut h = blake3::Hasher::new_derive_key(key);
    for p in paths {
        let name = p
            .strip_prefix(root)
            .unwrap_or(p)
            .to_string_lossy()
            .to_string();
        // The watch (xyl-core watch.rs) monitors the process and enters no
        // law. Its file and every line containing `watch::` are excluded, so a
        // change to watch labels does not invalidate a forecast made under the
        // same laws.
        if name.ends_with("watch.rs") {
            continue;
        }
        let text = std::fs::read_to_string(p).unwrap_or_default();
        // Comments and blank lines are excluded as well, so documentation
        // edits do not invalidate a founding or a forecast; only code
        // reaches the digest.
        let body: Vec<u8> = text
            .lines()
            .filter(|l| !l.contains("watch::"))
            .map(code_of)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
            .into_bytes();
        h.update(&(name.len() as u64).to_le_bytes());
        h.update(name.as_bytes());
        h.update(&(body.len() as u64).to_le_bytes());
        h.update(&body);
    }
    h.finalize().to_hex().as_str()[..16].to_string()
}

/// The code on a source line: the line with any `//` comment removed and
/// surrounding whitespace trimmed. A `//` inside a string literal is kept:
/// the scan tracks double quotes, skipping escaped ones.
fn code_of(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut in_str = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if in_str => i += 1,
            b'"' => in_str = !in_str,
            b'/' if !in_str && bytes.get(i + 1) == Some(&b'/') => return line[..i].trim(),
            _ => {}
        }
        i += 1;
    }
    line.trim()
}
