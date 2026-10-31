//! The element table, Z 1 to `TOP_Z` (1700), and the founding's nucleus
//! counts built from it by the capture chain.
//!
//! Reads `data/elements.tsv` (Z 1-150, embedded as `TABLE`), `atoms` (each
//! row's computed `Atom`, the vessel's upper edge) and
//! `xyl_core::{certified, measure}`. Values are exact `Q`: temperatures in
//! K, density in g/cm^3, measured abundance in atoms per 10^6 Si.
//!
//! - `table`: Z 1-99 are measured; Z 100-150 (marked predicted in the file)
//!   and Z 151-1700 (systematic names generated in code, homologue U) take
//!   melting and boiling points, condensation temperature and class from
//!   their homologue, density scaled by mass. An element with no mass number
//!   takes the nearest integer A on the valley of beta stability
//!   (`valley_a`, Z = A / (1.98 + 0.334 e(A / 100)), certified bounds). The
//!   island kinds (Z 114, 118, 120, 122, 126) take the r-process fall from
//!   platinum to uranium, `ab_U 10^(-d (Z - 92))`,
//!   `d = log10(ab_Pt / ab_U) / 14`, enclosed and fixed on a 2^-48 grid.
//!   Built once per process; `Err` names the first row that fails to parse.
//! - `founded(a)`: the table with each abundance replaced by a whole nucleus
//!   count, `count_Z = floor(count_(Z-1) a h_Z)`, h_Z = (3Z - unpaired) / 3Z,
//!   from 2^`founding_bits(a)` hydrogen: the fewest bits that found one whole
//!   nucleus of the vessel's sealing kind. Depends on `a` alone.
//! - `decimal`: a decimal literal as an exact rational, None if malformed.
//!
//! The atom (bands, bonding, tightness) is computed for every row; the
//! tightness ordering stands in for electronegativity. Melt, boil, rho, tc,
//! abundances and channels, atomic weight and Goldschmidt class remain
//! catalogue values not yet derived. Terms: capture chain (docs/glossary.md).

use xyl_core::certified::{down, ln, Encl};
use xyl_core::dyadic::Dy;
use xyl_core::Q;

const BITS: u32 = 64;
const GRID_BITS: u32 = 48;

/// The table as shipped.
pub const TABLE: &str = include_str!("../data/elements.tsv");

/// Goldschmidt's geochemical classes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// Lithophile: bonds with oxygen and stays in the rocky crust and mantle.
    Litho,
    /// Siderophile: alloys with iron and sinks to the metal core.
    Sidero,
    /// Chalcophile: bonds with sulfur.
    Chalco,
    /// Atmophile: volatile, held in the atmosphere and hydrosphere.
    Atmo,
}

/// Where an element's atoms were made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Channel {
    /// Made in the Big Bang.
    BigBang,
    /// Made by cosmic-ray spallation of heavier nuclei.
    Spallation,
    /// Made in core-collapse supernovae.
    CoreCollapse,
    /// Made in thermonuclear (type Ia) supernovae.
    TypeIa,
    /// Made in asymptotic-giant-branch stars (the slow neutron-capture process).
    Agb,
    /// Made in neutron-star mergers (the rapid neutron-capture process).
    Merger,
}

/// One element of the table: its identity, catalogue properties, computed atom
/// and nucleosynthetic origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    /// Atomic number, 1 to [`TOP_Z`].
    pub z: u32,
    /// Chemical symbol (a systematic symbol for predicted rows).
    pub sym: String,
    /// Element name (a systematic name for predicted rows).
    pub name: String,
    /// Atomic weight, or the mass number of its longest-lived or island nuclide.
    pub a: Q,
    /// Melting point in K, if known or predicted.
    pub melt: Option<Q>,
    /// Boiling point in K, if known or predicted.
    pub boil: Option<Q>,
    /// Condensed density, g/cm^3.
    pub rho: Option<Q>,
    /// Solar-system abundance, atoms per 10^6 Si (zero: none survives).
    pub abundance: Q,
    /// 50% condensation temperature, K.
    pub tc: Option<Q>,
    /// Goldschmidt's class: measured; a provisional catalogue value until derived
    /// from the bond law.
    pub class: Option<Class>,
    /// The atom, by the turning law (atoms.rs): its bands, what it can bond
    /// and how tightly it is held (the ordering electronegativity measures).
    pub atom: crate::atoms::Atom,
    /// Nucleosynthetic channels and their shares (summing to 1), or none.
    pub origin: Vec<(Channel, Q)>,
    /// Whether the row is predicted by the periodic law rather than measured.
    pub predicted: bool,
    /// In-game trade name, if the element is traded.
    pub alias: Option<String>,
}

/// A decimal literal (`2.43e10`, `0.95`, `-1.5e-3`) as an exact rational.
pub fn decimal(s: &str) -> Option<Q> {
    let s = s.trim();
    let (mant, exp) = match s.split_once(['e', 'E']) {
        Some((m, e)) => (m, e.parse::<i32>().ok()?),
        None => (s, 0),
    };
    let neg = mant.starts_with('-');
    let mant = mant.trim_start_matches(['-', '+']);
    let (int, frac) = mant.split_once('.').unwrap_or((mant, ""));
    let digits = format!("{int}{frac}");
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: num_bigint::BigInt = digits.parse().ok()?;
    let n = if neg { -n } else { n };
    let scale = exp - i32::try_from(frac.len()).ok()?;
    let ten = num_bigint::BigInt::from(10);
    let r = if scale >= 0 {
        num_rational::BigRational::from_integer(n * ten.pow(scale.unsigned_abs()))
    } else {
        num_rational::BigRational::new(n, ten.pow(scale.unsigned_abs()))
    };
    Some(Q::from_big(r))
}

fn opt(s: &str) -> Option<Q> {
    if s == "-" {
        None
    } else {
        decimal(s)
    }
}

fn class(s: &str) -> Option<Class> {
    match s {
        "litho" => Some(Class::Litho),
        "sidero" => Some(Class::Sidero),
        "chalco" => Some(Class::Chalco),
        "atmo" => Some(Class::Atmo),
        _ => None,
    }
}

fn channel(s: &str) -> Option<Channel> {
    match s {
        "bb" => Some(Channel::BigBang),
        "spall" => Some(Channel::Spallation),
        "ccsn" => Some(Channel::CoreCollapse),
        "ia" => Some(Channel::TypeIa),
        "agb" => Some(Channel::Agb),
        "merger" => Some(Channel::Merger),
        _ => None,
    }
}

/// One row as written (before the periodic law fills the predicted ones).
struct Row {
    e: Element,
    island: bool,
    homologue: Option<String>,
}

/// The table's highest Z: past the fourth seal of S^11 (Z 1092, the vessel's
/// upper edge) with the open fifth turning above it.
pub const TOP_Z: u32 = 1700;

/// The closed share of a capture that makes kind `z`: (3Z - unpaired) / 3Z.
fn closed_share(z: u32) -> Q {
    let unpaired = crate::atoms::atom(z).map(|x| x.unpaired).unwrap_or(0);
    let full = Q::int(3 * i64::from(z));
    Q::int(3 * i64::from(z) - i64::from(unpaired))
        .checked_div(&full)
        .unwrap_or(Q::ZERO)
}

/// The capture chain from `hydrogen` whole nuclei: count_Z = floor(count_(Z-1)
/// a h_Z) for Z = 2..=`top`, each count whole. Returns the count of `top`.
fn chain_to(hydrogen: &Q, a: &Q, top: u32) -> Q {
    let whole = |x: &Q| x - &x.frac();
    let mut count = hydrogen.clone();
    for z in 2..=top {
        count = whole(&(&(&count * a) * &closed_share(z)));
        if count.signum() <= 0 {
            break;
        }
    }
    count
}

/// The founding's nucleus count, in bits of hydrogen: the fewest bits that
/// carry one whole nucleus of the vessel's sealing kind (atoms::vessel's
/// upper edge) through the capture chain at the bounce's share `a`, so the
/// firmament is founded whole. Derived from the chain itself: the unfloored
/// chain is doubled until the sealing kind's count reaches one, then the
/// floored chain is checked and the bits raised until it holds. No package
/// input and no environment: the same bounce founds the same count.
pub fn founding_bits(a: &Q) -> u32 {
    static BITS: std::sync::OnceLock<std::sync::Mutex<std::collections::BTreeMap<String, u32>>> =
        std::sync::OnceLock::new();
    let table = BITS.get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()));
    let key = format!("{a}");
    if let Ok(t) = table.lock() {
        if let Some(&b) = t.get(&key) {
            return b;
        }
    }
    let bits = {
        let a = a.clone();
        let top = crate::atoms::vessel().map(|(_, hi)| hi).unwrap_or(1);
        // The unfloored chain from one hydrogen is the product of a and the
        // closed share over every kind up to the vessel's top; it is doubled
        // until it reaches one.
        let mut count = Q::ONE;
        for z in 2..=top {
            count = &(&count * &a) * &closed_share(z);
        }
        let mut bits = 0u32;
        while count < Q::ONE && bits < 1 << 14 {
            count = &count * &Q::int(2);
            bits += 1;
        }
        // Each floor along the chain loses less than one nucleus, so the
        // floored chain can fall short; the hydrogen count is doubled until
        // the sealing kind's floored count reaches one.
        let mut hydrogen = (0..bits).fold(Q::ONE, |acc, _| &acc * &Q::int(2));
        while chain_to(&hydrogen, &a, top) < Q::ONE && bits < 1 << 14 {
            hydrogen = &hydrogen * &Q::int(2);
            bits += 1;
        }
        bits
    };
    if let Ok(mut t) = table.lock() {
        t.insert(key, bits);
    }
    bits
}

/// A predicted row for `z` with its systematic name (digits nil un bi tri
/// quad pent hex sept oct enn), in the file's layout.
fn systematic_row(z: u32) -> String {
    const NAMES: [&str; 10] = [
        "nil", "un", "bi", "tri", "quad", "pent", "hex", "sept", "oct", "enn",
    ];
    const SYMS: [char; 10] = ['n', 'u', 'b', 't', 'q', 'p', 'h', 's', 'o', 'e'];
    let digits: Vec<usize> = z
        .to_string()
        .chars()
        .filter_map(|c| c.to_digit(10))
        .map(|d| d as usize)
        .collect();
    let mut name: String = digits.iter().map(|&d| NAMES[d]).collect();
    name = name.replace("ii", "i").replace("nnn", "nn");
    name.push_str("ium");
    let mut sym: String = digits.iter().map(|&d| SYMS[d]).collect();
    if let Some(first) = sym.get(0..1) {
        sym = format!("{}{}", first.to_uppercase(), &sym[1..]);
    }
    format!("{z}\t{sym}\t{name}\t-\t-\t-\t-\t0\t-\t-\t-\tpredicted\tU\t-")
}

fn row(line: &str) -> Result<Row, String> {
    let f: Vec<&str> = line.split('\t').collect();
    if f.len() != 14 {
        return Err(format!("{} fields, 14 wanted: {line}", f.len()));
    }
    let bad = |what: &str| format!("{what} in: {line}");
    let z: u32 = f[0].parse().map_err(|_| bad("z"))?;
    let origin = if f[10] == "-" {
        Vec::new()
    } else {
        f[10]
            .split(',')
            .map(|p| {
                let (c, s) = p.split_once(':').ok_or_else(|| bad("origin"))?;
                Ok((
                    channel(c).ok_or_else(|| bad("channel"))?,
                    decimal(s).ok_or_else(|| bad("share"))?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?
    };
    let island = f[7] == "island";
    let abundance = if island {
        Q::ZERO
    } else {
        decimal(f[7]).ok_or_else(|| bad("abundance"))?
    };
    let predicted = match f[11] {
        "measured" => false,
        "predicted" => true,
        _ => return Err(bad("status")),
    };
    Ok(Row {
        e: Element {
            z,
            sym: f[1].to_string(),
            name: f[2].to_string(),
            a: opt(f[3]).unwrap_or(Q::ZERO),
            melt: opt(f[4]),
            boil: opt(f[5]),
            rho: opt(f[6]),
            abundance,
            tc: opt(f[8]),
            class: class(f[9]),
            atom: crate::atoms::atom(z).ok_or_else(|| bad("z"))?,
            origin,
            predicted,
            alias: (f[13] != "-").then(|| f[13].to_string()),
        },
        island,
        homologue: (f[12] != "-").then(|| f[12].to_string()),
    })
}

/// The integer A on the valley of beta stability nearest `z`, by certified
/// bounds: `Z = A/(1.98 + 0.334 e(A/100))`, where the surface term e is the
/// extent of A against a hundred nucleons on S^2 (`extent`) in place of
/// (A/100)^(2/3); the two agree at A = 100. None if an enclosure fails.
pub fn valley_a(z: u32) -> Option<u32> {
    let target = Q::int(i64::from(z));
    let miss = |a: u32| -> Option<Q> {
        let aq = Q::int(i64::from(a));
        let t = crate::elements::extent(&aq.checked_div(&Q::int(100))?);
        let den = Encl::exact(Q::new(198, 100)?)
            .add(&Encl::exact(Q::new(334, 1000)?).mul(&Encl::exact(t)));
        let zv = Encl::exact(aq).mul(&den.recip()?);
        Some((&zv.mid() - &target).abs())
    };
    // Z(A) rises with A, so the miss has a single minimum: start from the
    // inverted formula (surface term at A ~ 5Z/2), clamped to [2Z, 3Z], and
    // step to the neighbour with the smaller miss, preferring the smaller A on
    // a tie.
    let (lo, hi) = (2 * z, 3 * z);
    let guess = Dy::int(i64::from(z))
        * (Dy::ratio(198, 100)
            + Dy::ratio(334, 1000)
                * Dy::from_q(&crate::elements::extent(
                    &Q::new(5 * i64::from(z), 200).unwrap_or(Q::ONE),
                )));
    let mut a = u32::try_from(guess.trunc_i64().max(0)).ok()?.clamp(lo, hi);
    let mut here = miss(a)?;
    loop {
        if a > lo {
            let m = miss(a - 1)?;
            if m <= here {
                (a, here) = (a - 1, m);
                continue;
            }
        }
        if a < hi {
            let m = miss(a + 1)?;
            if m < here {
                (a, here) = (a + 1, m);
                continue;
            }
        }
        return Some(a);
    }
}

/// The founding's matter: the elements this universe's bounce made, with
/// each element's nucleus count in place of the measured abundance. Nuclei
/// are built one capture at a time from hydrogen, so
/// count_Z = count_(Z-1) s_Z with s_Z = a h_Z: each capture is one pass of the
/// outbound jet through the throat, a is the share the jet carries (the
/// parent's torsion share, from 1/2 to 3/5) and h_Z the new nucleus's closed share,
/// (3Z - unpaired) / 3Z. Counts are whole nuclei out of 2^`founding_bits()`
/// hydrogen, the fewest that found the vessel's sealing kind whole, so a kind
/// with fewer than one nucleus has none and every mixture stays in whole
/// numbers. Only count ratios are used. Other columns (masses, phases,
/// classes) are as in the table, measured or predicted. `Err` as for the
/// table.
pub fn founded(a: &Q) -> Result<Vec<Element>, String> {
    let mut out = table()?;
    out.sort_by_key(|e| e.z);
    let whole = |x: &Q| x - &x.frac();
    let hydrogen = (0..founding_bits(a)).fold(Q::ONE, |acc, _| &acc * &Q::int(2));
    let mut count = hydrogen;
    for e in out.iter_mut() {
        if e.z > 1 {
            count = whole(&(&(&count * a) * &closed_share(e.z)));
        }
        e.abundance = count.clone();
    }
    Ok(out)
}

/// The whole table, Z = 1..=TOP_Z, predicted rows filled by the periodic law.
/// `Err` names the first row that fails to parse. Built once per process
/// (the island masses use slow certified logarithms) and cloned after.
pub fn table() -> Result<Vec<Element>, String> {
    static TABLE_BUILT: std::sync::OnceLock<Result<Vec<Element>, String>> =
        std::sync::OnceLock::new();
    TABLE_BUILT.get_or_init(build_table).clone()
}

fn build_table() -> Result<Vec<Element>, String> {
    let mut rows: Vec<Row> = TABLE
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(row)
        .collect::<Result<_, _>>()?;
    // Rows past the file's last Z up to TOP_Z follow the rule of the file's
    // predicted rows: systematic name, valley A, uranium as the homologue for
    // the material columns. Their counts come from the founding chain and
    // stellar transduction (chem), as for every kind.
    let top = rows.iter().map(|r| r.e.z).max().unwrap_or(0);
    for z in (top + 1)..=TOP_Z {
        rows.push(row(&systematic_row(z))?);
    }
    let measured: Vec<Element> = rows.iter().map(|r| r.e.clone()).collect();
    let by_sym = |s: &str| measured.iter().find(|e| e.sym == s).cloned();
    let (u, pt) = (by_sym("U").ok_or("no U")?, by_sym("Pt").ok_or("no Pt")?);
    // The r-process fall per unit Z is d = log10(Pt / U) / 14, evaluated with
    // enclosed logarithms.
    let ratio = pt
        .abundance
        .checked_div(&u.abundance)
        .ok_or("U abundance zero")?;
    let ln10 = ln(&Q::int(10), BITS).ok_or("ln 10")?;
    let d = ln(&ratio, BITS)
        .ok_or("ln ratio")?
        .mul(&ln10.recip().ok_or("1/ln 10")?)
        .mul(&Encl::exact(Q::new(1, 14).ok_or("1/14")?));
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let mut e = r.e;
        if e.a.is_zero() {
            e.a = Q::int(i64::from(
                valley_a(e.z).ok_or_else(|| format!("no valley A for Z = {}", e.z))?,
            ));
        }
        if let Some(h) = r.homologue.as_deref() {
            let h = by_sym(h).ok_or_else(|| format!("{}: no homologue {h}", e.sym))?;
            e.class = e.class.or(h.class);
            e.melt = e.melt.or(h.melt.clone());
            let own_boil = e.boil.is_some();
            e.boil = e.boil.or(h.boil.clone());
            // Without a density in the row, rho = rho_h A / A_h from the
            // homologue.
            if e.rho.is_none() {
                e.rho = match (&h.rho, h.a.is_zero()) {
                    (Some(hr), false) => Some((hr * &e.a).checked_div(&h.a).ok_or("homologue A")?),
                    _ => None,
                };
            }
            // T_c = T_c,h boil / boil_h when the row gives a boiling point,
            // else the homologue's T_c.
            if e.tc.is_none() {
                e.tc = match (&h.tc, &h.boil, &e.boil, own_boil) {
                    (Some(htc), Some(hb), Some(b), true) => {
                        Some((htc * b).checked_div(hb).ok_or("homologue boil")?)
                    }
                    (Some(htc), _, _, _) => Some(htc.clone()),
                    _ => None,
                };
            }
        }
        if r.island {
            // The abundance is ab_U 10^(-d (Z - 92)) = ab_U exp(-d ln10 (Z - 92)).
            let x = d
                .mul(&ln10)
                .mul(&Encl::exact(Q::int(i64::from(e.z) - 92)))
                .neg();
            let f = xyl_core::certified::exp_of(&x, BITS);
            e.abundance = down(&Encl::exact(u.abundance.clone()).mul(&f).mid(), GRID_BITS);
        }
        out.push(e);
    }
    Ok(out)
}

/// A body's radius from its mass (world units): the share s = m / (m + 1) is
/// held by the angle theta on S^2 (measure.rs), and R = 4 theta (turns).
/// The unit world gives 1; a mass of 0.012 gives 0.14, of 0.107 gives 0.40,
/// of 10 gives 1.61. A mass that is not positive gives 0.
pub(crate) fn extent(m: &Q) -> Q {
    use xyl_core::dyadic::Dy;
    if m.signum() <= 0 {
        return Q::ZERO;
    }
    let share = Dy::from_q(&m.checked_div(&(m + &Q::ONE)).unwrap_or(Q::ZERO));
    (Dy::int(4) * xyl_core::measure::angle_holding(2, share)).to_q()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_whole_and_consistent() {
        let t = table().unwrap();
        assert_eq!(t.len(), TOP_Z as usize);
        for (i, e) in t.iter().enumerate() {
            assert_eq!(e.z as usize, i + 1, "Z runs 1..=TOP_Z in order");
            assert!(e.a > Q::ZERO, "{} has a mass", e.sym);
            if !e.origin.is_empty() {
                let s = e.origin.iter().fold(Q::ZERO, |a, (_, f)| &a + f);
                assert_eq!(s, Q::ONE, "{}'s origins sum to one", e.sym);
            }
            if e.abundance > Q::ZERO {
                assert!(
                    !e.origin.is_empty(),
                    "{} survives, so something made it",
                    e.sym
                );
            }
            assert!(e.sym.is_ascii() && e.name.is_ascii());
        }
    }

    #[test]
    fn valley_of_stability_places_unmeasured_rows() {
        // On the valley every nucleus holds more nucleons than protons,
        // heavier kinds hold more neutrons per proton, and the table's
        // predicted rows lie on it beyond the measured ones.
        let (fe, u) = (valley_a(26).unwrap(), valley_a(92).unwrap());
        assert!(fe > 26 && u > 92, "{fe} {u}");
        assert!(
            (u - 92) * 26 > (fe - 26) * 92,
            "heavier kinds hold more neutrons per proton: {fe} {u}"
        );
        let t = table().unwrap();
        let uue = t.iter().find(|e| e.z == 119).unwrap();
        assert!(uue.a > Q::int(u as i64), "Uue at A = {}", uue.a);
    }

    #[test]
    fn decimals_are_exact() {
        assert_eq!(decimal("2.43e10").unwrap(), Q::int(24_300_000_000));
        assert_eq!(decimal("0.009").unwrap(), Q::new(9, 1000).unwrap());
        assert_eq!(decimal("-1.5e-3").unwrap(), Q::new(-3, 2000).unwrap());
        assert!(decimal("1.2.3").is_none() && decimal("x").is_none());
    }
}
