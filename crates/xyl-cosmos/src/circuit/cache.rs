// The founding cache: the founded `Substrate` written to the home directory
// with `xyl_core::codec`, keyed by the founding digest and the torsion share.
//
// Inputs: the directory named by `cache_in` (first call per process wins;
// without one nothing is cached). Outputs: `cache_in`, and crate-private
// `load` and `store`. A file of another format version, or one that does
// not decode whole, is refused (None) and the founding is recomputed. The
// reverberation is not stored; it is recomputed on load. A
// store writes a temporary file and renames it; failures are ignored,
// since the cache only saves time and deleting it changes no reading.

use super::*;
use std::path::{Path, PathBuf};
use xyl_core::codec::{Dec, Enc};

/// The founding is derived (the laws regenerate it from the bounce's share),
/// so it is kept in the home directory only as an accelerator, keyed by the
/// founding digest and the torsion share: a change to either founds anew,
/// and deleting the file changes nothing (docs/architecture.md, cached
/// derived state). The reverberation is not kept; it is recomputed on
/// load.
static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

const VERSION: u32 = 11;

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
            .q(&b.state)
            .q(&b.phase)
            .q(&b.span.0)
            .q(&b.span.1)
            .len(b.recipe.len());
        for (z, a) in &b.recipe {
            e.u32(*z).q(a);
        }
    }
    put_table(&mut e, &s.neutrons);
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
        let (index, reach, state, phase) = (d.u32()?, d.q()?, d.q()?, d.q()?);
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
            state,
            phase,
            recipe,
            span,
        });
    }
    let neutrons = get_table(&mut d)?;
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
        lighting: crate::formation::Lighting {
            cools_in: (cools_in >= 0).then_some(cools_in),
            lit,
            pieces,
        },
        neutrons,
    })
}

/// The universal table: per kind its rest-point parcels, each an amount and
/// its face pairs (each count written as an integer `Q`).
fn put_table(e: &mut Enc, table: &super::weights::Table) {
    e.len(table.len());
    for (z, parcels) in table {
        e.u32(*z).len(parcels.len());
        for p in parcels {
            e.q(&p.amount).len(p.pairs.pairs().len());
            for (c, d) in p.pairs.pairs() {
                e.q(&Q::from_big(c.clone().into()))
                    .q(&Q::from_big(d.clone().into()));
            }
        }
    }
}

/// [`put_table`] read back; `None` for a count or pair that does not read.
fn get_table(d: &mut Dec) -> Option<super::weights::Table> {
    let mut table = super::weights::Table::new();
    for _ in 0..d.count()? {
        let z = d.u32()?;
        let mut parcels = Vec::new();
        for _ in 0..d.count()? {
            let amount = d.q()?;
            let mut pairs = xyl_core::shares::FacePairs::one();
            for _ in 0..d.count()? {
                let (c, dd) = (d.q()?, d.q()?);
                pairs = pairs.compose(&xyl_core::shares::FacePairs::pair(c.numer(), dd.numer())?);
            }
            parcels.push(xyl_core::shares::Parcel { amount, pairs });
        }
        table.insert(z, parcels);
    }
    Some(table)
}

/// The founding shells as `formation::founding_shells` returns them: each
/// shell and the universal table.
pub(crate) type Shells = (Vec<crate::formation::Shell>, super::weights::Table);

/// The cached founding shells for `key` (`shells-{key}` in the directory),
/// if the directory holds them whole.
pub(crate) fn load_shells(key: &str) -> Option<Shells> {
    let buf = std::fs::read(DIR.get()?.join(format!("shells-{key}"))).ok()?;
    decode_shells(&buf)
}

/// Keeps the founding shells for `key`, written whole and renamed as
/// [`store`] does; failures are silent.
pub(crate) fn store_shells(key: &str, s: &Shells) {
    let Some(dir) = DIR.get() else {
        return;
    };
    let _ = std::fs::create_dir_all(dir);
    let p = dir.join(format!("shells-{key}"));
    let tmp = p.with_extension("tmp");
    let _ = std::fs::write(&tmp, encode_shells(s)).and_then(|_| std::fs::rename(&tmp, &p));
}

/// The founding shells written with the format version.
fn encode_shells(s: &Shells) -> Vec<u8> {
    let mut e = Enc::new();
    e.u32(VERSION).len(s.0.len());
    for k in &s.0 {
        e.u32(k.z)
            .q(&k.amount)
            .q(&k.reach)
            .q(&k.swing)
            .u32(u32::from(k.kept));
    }
    put_table(&mut e, &s.1);
    e.buf
}

/// [`encode_shells`] read back; `None` for another version or a buffer
/// that does not read whole.
fn decode_shells(buf: &[u8]) -> Option<Shells> {
    let mut d = Dec::new(buf);
    if d.u32()? != VERSION {
        return None;
    }
    let mut shells = Vec::new();
    for _ in 0..d.count()? {
        shells.push(crate::formation::Shell {
            z: d.u32()?,
            amount: d.q()?,
            reach: d.q()?,
            swing: d.q()?,
            kept: d.u32()? == 1,
        });
    }
    let table = get_table(&mut d)?;
    d.done().then_some((shells, table))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The shells stage reads back as written, face pairs and all, and a
    /// truncated buffer is refused.
    #[test]
    fn the_shells_stage_reads_back_whole() {
        let mix: Vec<(u32, Q)> = [1u32, 2, 8, 26]
            .iter()
            .map(|z| (*z, Q::new(1, i64::from(*z)).unwrap()))
            .collect();
        let shells = crate::formation::founding_shells(&mix, &xyl_core::ratio!(1, 2));
        let buf = encode_shells(&shells);
        assert_eq!(decode_shells(&buf), Some(shells));
        assert_eq!(decode_shells(&buf[..buf.len() - 1]), None);
    }
}
