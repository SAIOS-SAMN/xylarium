// The founding's cache: the substrate as founded, kept in the home directory

use super::*;
use std::path::{Path, PathBuf};
use xyl_core::codec::{Dec, Enc};

/// The founding is derived (the laws regenerate it from the bounce's share),
/// so it is kept in the home directory only as an accelerator, keyed by the
/// laws' source digest and the bounce's share: a change to either founds
/// anew, and deleting the file changes nothing (docs/architecture.md, cached
/// derived state). The reverberation is not kept; it is recomputed from the
/// anchor on load.
static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

const VERSION: u32 = 8;

/// Names the directory the founding is cached in (a universe's home). The
/// first call in a process wins; without one, nothing is cached.
pub fn cache_in(dir: &Path) {
    let _ = DIR.set(dir.to_path_buf());
}

/// The cache file for `key`, where a directory is named.
fn path(key: &str) -> Option<PathBuf> {
    DIR.get().map(|d| d.join(format!("founding-{key}")))
}

fn encode(s: &Substrate) -> Vec<u8> {
    let mut e = Enc::new();
    e.u32(VERSION)
        .q(&s.rim)
        .q(&s.nucleons_founded)
        .q(&s.share)
        .i64(s.lighting.cools_in.unwrap_or(-1))
        .q(&s.lighting.lit)
        .len(s.lighting.pieces.len());
    for (from, to, cools_in, infall) in &s.lighting.pieces {
        e.q(from).q(to).i64(cools_in.unwrap_or(-1)).q(infall);
    }
    e.len(s.firmament.len());
    for k in &s.firmament {
        e.u32(k.z).q(&k.amount).q(&k.spin).q(&k.phase);
    }
    e.len(s.beads.len());
    for b in &s.beads {
        e.u32(b.index)
            .q(&b.reach)
            .q(&b.inside)
            .q(&b.state)
            .q(&b.phase)
            .q(&b.span.0)
            .q(&b.span.1)
            .len(b.recipe.len());
        for (z, a) in &b.recipe {
            e.u32(*z).q(a);
        }
    }
    e.buf
}

fn decode(buf: &[u8]) -> Option<Substrate> {
    let mut d = Dec::new(buf);
    if d.u32()? != VERSION {
        return None;
    }
    let rim = d.q()?;
    let nucleons_founded = d.q()?;
    let share = d.q()?;
    let cools_in = d.i64()?;
    let lit = d.q()?;
    let n = d.count()?;
    let mut pieces = Vec::with_capacity(n);
    for _ in 0..n {
        let (from, to, c, infall) = (d.q()?, d.q()?, d.i64()?, d.q()?);
        pieces.push((from, to, (c >= 0).then_some(c), infall));
    }
    let n = d.count()?;
    let mut firmament = Vec::with_capacity(n);
    for _ in 0..n {
        firmament.push(Kind {
            z: d.u32()?,
            amount: d.q()?,
            spin: d.q()?,
            phase: d.q()?,
        });
    }
    let n = d.count()?;
    let mut beads = Vec::with_capacity(n);
    for _ in 0..n {
        let (index, reach, inside, state, phase) = (d.u32()?, d.q()?, d.q()?, d.q()?, d.q()?);
        let span = (d.q()?, d.q()?);
        let m = d.count()?;
        let mut recipe = Mix::new();
        for _ in 0..m {
            let z = d.u32()?;
            recipe.insert(z, d.q()?);
        }
        beads.push(Bead {
            index,
            reach,
            inside,
            state,
            phase,
            recipe,
            span,
        });
    }
    if !d.done() {
        return None;
    }
    Some(Substrate {
        firmament,
        beads,
        rim,
        nucleons_founded,
        reverb: Vec::new(),
        share,
        bell: xyl_core::dyadic::Dy::ZERO,
        lighting: crate::formation::Lighting {
            cools_in: (cools_in >= 0).then_some(cools_in),
            lit,
            pieces,
        },
    })
}

/// The cached founding for `key`, if the directory holds one that reads whole.
pub(crate) fn load(key: &str) -> Option<Substrate> {
    let p = path(key)?;
    let buf = std::fs::read(p).ok()?;
    decode(&buf)
}

/// Keeps the founding for `key`: written whole to a temporary file and
/// renamed, so a crash never leaves a partial cache. Failures are silent;
/// the founding is derived and the cache only an accelerator.
pub(crate) fn store(key: &str, c: &Substrate) {
    let Some(p) = path(key) else {
        return;
    };
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let tmp = p.with_extension("tmp");
    let _ = std::fs::write(&tmp, encode(c)).and_then(|_| std::fs::rename(&tmp, &p));
}
