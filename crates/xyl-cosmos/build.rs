//! Source digest: every source file the forecast's laws are written in
//! (xyl-cosmos's and xyl-core's code and the elements' table) is hashed at
//! build time, so a cached forecast is never read under laws other than the
//! ones that produced it. The watch (the program's observer) is excluded: it
//! never enters a law.

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
    // every law, and the founding's own: the files the founding (the
    // genesis) is derived from - the Big Bang, the bounce, the elements and
    // atoms, the cascade and lighting, the circuit, the package and the
    // mathematics - so a change to any layer built on the founding keeps it
    let founding = |name: &str| {
        const FILES: [&str; 10] = [
            "src/bang.rs",
            "src/bounce.rs",
            "src/elements.rs",
            "src/atoms.rs",
            "src/compound.rs",
            "src/fibres.rs",
            "src/formation.rs",
            "src/cores.rs",
            "src/chain.rs",
            "src/package.rs",
        ];
        FILES.contains(&name)
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
        // the watch observes the program, not the laws (xyl-core watch.rs):
        // its file and every line naming it are excluded, so changing watch
        // labels never invalidates a forecast produced by the same laws
        if name.ends_with("watch.rs") {
            continue;
        }
        let text = std::fs::read_to_string(p).unwrap_or_default();
        let body: Vec<u8> = text
            .lines()
            .filter(|l| !l.contains("watch::"))
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
