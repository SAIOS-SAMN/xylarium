//! The schedule kept: its exact encoding and the digests that key the cache.

use super::*;

/// The digest of the source of the laws under which the forecast was
/// computed: a cache stored under other laws is recomputed, not reused.
pub fn laws_digest() -> String {
    // Every source file of the laws, hashed at build time (build.rs).
    env!("XYL_SOURCE_DIGEST").to_string()
}

pub(super) const REGIONS: [Region; 5] = [
    Region::Biotic,
    Region::Cryogenic,
    Region::Mineral,
    Region::Plasmic,
    Region::Umbral,
];

pub(super) fn region_code(r: Region) -> u8 {
    u8::try_from(REGIONS.iter().position(|x| *x == r).unwrap_or(0)).unwrap_or(0)
}

pub(super) fn put_dy(e: &mut xyl_core::codec::Enc, x: Dy) {
    let (m, ex) = x.to_bits();
    e.i64(m).i64(i64::from(ex));
}

pub(super) fn get_dy(d: &mut xyl_core::codec::Dec) -> Option<Dy> {
    let m = d.i64()?;
    let e = i32::try_from(d.i64()?).ok()?;
    Dy::from_bits(m, e)
}

pub(super) const FORECAST_FORMAT: &str = "xylarium forecast v3";

impl Schedule {
    /// The schedule as exact bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut e = xyl_core::codec::Enc::new();
        e.str(FORECAST_FORMAT).len(self.minds.len());
        for m in &self.minds {
            e.u64(m.star).u64(m.w as u64);
            put_dy(&mut e, m.epoch);
            e.u8(region_code(m.region));
            put_dy(&mut e, m.star_mass);
            put_dy(&mut e, m.r_kpc);
            e.len(m.steps.len());
            for &s in &m.steps {
                put_dy(&mut e, s);
            }
            e.u32(m.setbacks).bool(m.seeded).bool(m.fed_settled);
        }
        e.len(self.curve.len());
        for &c in &self.curve {
            put_dy(&mut e, c);
        }
        let l = &self.limits;
        for x in [
            l.worlds,
            l.no_solvent,
            l.cannot_copy,
            l.star_died,
            l.sterilised,
            l.too_slow,
            l.unfed,
            l.minds,
            l.alive,
            l.merged,
        ] {
            e.u64(x);
        }
        e.len(l.by_region.len());
        for &(r, x) in &l.by_region {
            e.u8(region_code(r)).u64(x);
        }
        e.len(l.rungs.len());
        for &x in &l.rungs {
            e.u64(x);
        }
        e.buf
    }

    /// A schedule from its bytes; `None` for damaged input, a different
    /// format tag or trailing bytes.
    pub fn decode(buf: &[u8]) -> Option<Schedule> {
        let mut d = xyl_core::codec::Dec::new(buf);
        if d.str()? != FORECAST_FORMAT {
            return None;
        }
        let n = d.count()?;
        let mut minds = Vec::with_capacity(n);
        for _ in 0..n {
            let star = d.u64()?;
            let w = usize::try_from(d.u64()?).ok()?;
            let epoch = get_dy(&mut d)?;
            let region = *REGIONS.get(usize::from(d.u8()?))?;
            let star_mass = get_dy(&mut d)?;
            let r_kpc = get_dy(&mut d)?;
            let ns = d.count()?;
            let mut steps = Vec::with_capacity(ns);
            for _ in 0..ns {
                steps.push(get_dy(&mut d)?);
            }
            let (setbacks, seeded, fed_settled) = (d.u32()?, d.bool()?, d.bool()?);
            minds.push(Mind {
                star,
                w,
                epoch,
                region,
                star_mass,
                r_kpc,
                steps,
                setbacks,
                seeded,
                fed_settled,
            });
        }
        let nc = d.count()?;
        let mut curve = Vec::with_capacity(nc);
        for _ in 0..nc {
            curve.push(get_dy(&mut d)?);
        }
        let mut f = [0u64; 10];
        for x in f.iter_mut() {
            *x = d.u64()?;
        }
        let nb = d.count()?;
        let mut by_region = Vec::with_capacity(nb);
        for _ in 0..nb {
            by_region.push((*REGIONS.get(usize::from(d.u8()?))?, d.u64()?));
        }
        let nr = d.count()?;
        let mut rungs = Vec::with_capacity(nr);
        for _ in 0..nr {
            rungs.push(d.u64()?);
        }
        let limits = Limits {
            worlds: f[0],
            no_solvent: f[1],
            cannot_copy: f[2],
            star_died: f[3],
            sterilised: f[4],
            too_slow: f[5],
            unfed: f[6],
            minds: f[7],
            alive: f[8],
            merged: f[9],
            by_region,
            rungs,
        };
        d.done().then_some(Schedule {
            minds,
            curve,
            limits,
        })
    }
}

/// The key under which a universe's schedule is cached: the seed and a hash
/// of the seed and the universe's scales (`laws::Scales`), so a universe
/// founded under other scales has another key.
pub fn key(gal: &Galaxy) -> String {
    let mut h = blake3::Hasher::new_derive_key("xylarium 2026-10-08 schedule key v1");
    h.update(&gal.seed.to_le_bytes());
    let s = &gal.sc;
    for x in [
        s.chem,
        s.pressure,
        s.atom,
        s.proton,
        s.density,
        s.star_mass,
        s.star_radius,
        s.nuclear,
        s.alpha,
    ] {
        let (m, e) = x.to_bits();
        h.update(&m.to_le_bytes());
        h.update(&e.to_le_bytes());
    }
    format!("seed{}-{}", gal.seed, &h.finalize().to_hex().as_str()[..12])
}
