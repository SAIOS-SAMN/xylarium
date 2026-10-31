//! A universe's chemistry: the birth gas's light nuclei and every element's
//! abundance at a radius and time, read against the calibration.

use super::{channel_index, delay, ring_of, Calibration, Model, Params, Rulers, BITS};
use crate::bang::BigBang;
use crate::elements::{decimal, Channel};
use std::sync::OnceLock;
use xyl_core::certified::{exp, pi};
use xyl_core::dyadic::Dy;
use xyl_core::motion::ExpSum;
use xyl_core::Q;

/// What a closed form has formed from 0 to `t`: each term `c t^p e^(-k t)`
/// integrated exactly, `c p! / k^(p+1) (1 - e^(-k t) sum_(j<=p) (k t)^j / j!)`,
/// or `c t^(p+1) / (p + 1)` where it does not decay.
fn formed_by(s: &ExpSum, t: Dy) -> Dy {
    s.terms
        .iter()
        .map(|term| {
            let c = Dy::from_q(&term.coef.mid());
            let k = Dy::from_q(&term.rate);
            let p = term.power;
            if k.signum() == 0 {
                let mut tp = t;
                for _ in 0..p {
                    tp = tp * t;
                }
                return c * tp / Dy::int(i64::from(p) + 1);
            }
            // sum_(j<=p) (k t)^j / j!, and p! / k^(p+1)
            let (mut series, mut each, mut fact, mut kp) = (Dy::ONE, Dy::ONE, Dy::ONE, k);
            for j in 1..=p {
                each = each * k * t / Dy::int(i64::from(j));
                series += each;
                fact = fact * Dy::int(i64::from(j));
                kp = kp * k;
            }
            c * fact / kp * (Dy::ONE - (-(k * t)).exp() * series)
        })
        .sum()
}

/// The birth gas's light nuclei, from the founding's own abundances
/// (`elements::founded`): hydrogen's and helium's mass shares from their
/// counts and the table's masses, and the lighter kinds by count against
/// hydrogen as the table gives them (zero where it holds none).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Light {
    /// Hydrogen's mass share of hydrogen and helium, `1 - y`.
    pub x: Q,
    /// Helium's mass share of hydrogen and helium.
    pub y: Q,
    /// Deuterium to hydrogen by count (the table holds no isotope rows, so
    /// zero until it does).
    pub d_h: Q,
    /// Lithium to hydrogen by count, from the founding's Z = 3 column.
    pub li_h: Q,
}

impl Light {
    /// The light nuclei of the gas the founding of `bang` seeds. `None` when
    /// the table cannot be built or hydrogen and helium carry no mass.
    pub fn of(bang: &BigBang) -> Option<Light> {
        let table = crate::elements::founded(&bang.parent.share()).ok()?;
        let by = |sym: &str| table.iter().find(|e| e.sym == sym);
        let (h, he) = (by("H")?, by("He")?);
        let (m_h, m_he) = (&h.abundance * &h.a, &he.abundance * &he.a);
        let y = m_he.checked_div(&(&m_h + &m_he))?;
        let per_h = |sym: &str| {
            by(sym)
                .and_then(|e| e.abundance.checked_div(&h.abundance))
                .unwrap_or(Q::ZERO)
        };
        Some(Light {
            x: &Q::ONE - &y,
            y,
            d_h: per_h("D"),
            li_h: per_h("Li"),
        })
    }
}

/// A universe's chemistry: its galaxy's closed forms, read against the reference
/// calibration.
pub struct Chemistry {
    /// The universe's galaxy model, built from its Big Bang's parameters.
    pub model: Model,
    /// The universe's light nuclei from its founding.
    pub light: Light,
}

impl Chemistry {
    /// The process's chemistry: the first founded universe's (as the anchor
    /// is, the first founding in the process wins), built once and kept;
    /// before the universe is founded it is built but not kept.
    pub fn cached(bang: &BigBang) -> Option<&'static Chemistry> {
        static C: OnceLock<Chemistry> = OnceLock::new();
        if let Some(c) = C.get() {
            return Some(c);
        }
        let c = Chemistry::of(bang)?;
        if crate::circuit::anchor().rim.signum() > 0 {
            let _ = C.set(c);
        }
        C.get()
    }

    /// A star's birth (Gyr after the Big Bang) on its ring's own star
    /// formation: the time by which the share `u` of the ring's stars formed
    /// from `from` (its first light) to the horizon have formed, read from the
    /// ring's closed form (`Ring::sfr`, integrated term by term) and found by
    /// halving. A ring with no star formation after `from` gives `from`.
    pub fn birth_along(&self, r: &Q, from: &Q, u: Dy) -> Q {
        let Some(ring) = self.model.rings.get(ring_of(r)) else {
            return from.clone();
        };
        let first = &self.model.params.first_stars;
        let t0 = Dy::from_q(&(from - first)).max(Dy::ZERO);
        let top = crate::life::horizon_gyr() - Dy::from_q(first);
        if top <= t0 {
            return from.clone();
        }
        let (c0, c1) = (formed_by(&ring.sfr, t0), formed_by(&ring.sfr, top));
        if c1 <= c0 {
            return from.clone();
        }
        let target = c0 + u.max(Dy::ZERO).min(Dy::ONE) * (c1 - c0);
        let (mut lo, mut hi) = (t0, top);
        for _ in 0..48 {
            let mid = (lo + hi) / Dy::int(2);
            if formed_by(&ring.sfr, mid) < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        first + &hi.to_q()
    }

    /// The chemistry of the universe whose Big Bang is `bang`. Returns `None`
    /// when the calibration cannot be built or the founding's light nuclei
    /// cannot be read (see [`Light::of`]).
    pub fn of(bang: &BigBang) -> Option<Chemistry> {
        let p = Params::of(bang);
        // the first history built in the process fixes the calibration
        Calibration::of(&p)?;
        Some(Chemistry {
            model: Model::of(&p),
            light: Light::of(bang)?,
        })
    }

    /// An element's abundance in the gas at radius `r`, epoch `t`, relative to
    /// the Sun's birth gas (1 there): what a star and its worlds born there and
    /// then are made of. `lump` scales the mergers' share (a star's own cloud).
    pub fn relative(&self, sym: &str, r: &Q, t: &Q, lump: &Q) -> Option<Q> {
        let cal = Calibration::held()?;
        let ei = cal.elements.iter().position(|e| e.sym == sym)?;
        if sym == "H" || sym == "He" {
            return self.light(sym, &self.metals(r, t)?);
        }
        let (smooth, merger) = cal.parts(self.model.concs(ring_of(r), t).as_deref(), ei)?;
        Some(&smooth + &(&merger * lump))
    }

    /// Every element's abundance at `r`, `t` (as `relative`), the channels
    /// read once for all of them: what a star's birth gas is made of.
    pub fn relatives(&self, r: &Q, t: &Q, lump: &Q) -> Option<Vec<Q>> {
        let cal = Calibration::held()?;
        let concs = self.model.concs(ring_of(r), t);
        let z = Self::metals_of(cal, concs.as_deref())?;
        Some(
            cal.elements
                .iter()
                .enumerate()
                .map(|(ei, e)| {
                    if e.sym == "H" || e.sym == "He" {
                        return self.light(&e.sym, &z).unwrap_or(Q::ZERO);
                    }
                    cal.parts(concs.as_deref(), ei)
                        .map(|(sm, mg)| &sm + &(&mg * lump))
                        .unwrap_or(Q::ZERO)
                })
                .collect(),
        )
    }

    /// Hydrogen's or helium's abundance relative to the Sun's birth gas, from
    /// the metals `z` (relative): what burning made of the Big Bang's.
    fn light(&self, sym: &str, z: &Q) -> Option<Q> {
        let z = z * &Rulers::reference_metallicity();
        let xp = &self.light.x;
        let burn = &Q::ONE + &Rulers::dy_dz();
        if sym == "H" {
            (xp - &(&burn * &z)).checked_div(&(xp - &(&burn * &Rulers::reference_metallicity())))
        } else {
            let y0 = &self.light.y;
            (y0 + &(&Rulers::dy_dz() * &z))
                .checked_div(&(y0 + &(&Rulers::dy_dz() * &Rulers::reference_metallicity())))
        }
    }

    /// The share of the mergers' smooth expectation a star's birth cloud holds
    /// (enrichment.rprocess_lump), for its own draw `u` in [0, 1): the long-mixed
    /// part, plus the recent part scaled by how many mergers actually struck its
    /// patch (about 300 pc over 100 Myr), Poisson about the expected number.
    pub fn lump(&self, r: &Q, t: &Q, u: &Q) -> Option<Q> {
        let ci = channel_index(Channel::Merger)?;
        let k = ring_of(r);
        let ring = self.model.rings.get(k)?;
        let dt = t - &self.model.params.first_stars;
        let s = &dt - &delay(Channel::Merger).0;
        if s.signum() <= 0 {
            return Some(Q::ZERO);
        }
        let held = ring.metal[ci].at(&s).mid();
        let out = &ring.landed[ci].at(&s).mid() * &decimal("0.1").unwrap_or(Q::ZERO);
        if held.signum() <= 0 {
            return Some(if out.signum() <= 0 { Q::ZERO } else { Q::ONE });
        }
        let recent = out.checked_div(&held)?.min(Q::ONE);
        // stars formed in its patch: sfr (suns/pc^2/Gyr) x pi (300 pc)^2 x 0.1 Gyr
        let sfr = ring.sfr.at(&dt).mid();
        let formed = &(&sfr * &pi(BITS).mid()) * &decimal("9000").unwrap_or(Q::ZERO);
        let expect = &decimal("2.0e-5").unwrap_or(Q::ZERO) * &formed;
        if expect.signum() <= 0 {
            return Some(&Q::ONE - &recent);
        }
        // the Poisson count at u, by its inverse CDF
        let p0 = exp(&-expect.clone(), BITS).mid();
        let (mut kk, mut p, mut cum) = (0i64, p0.clone(), p0);
        while u > &cum && kk < 1000 {
            kk += 1;
            p = (&p * &expect).checked_div(&Q::int(kk))?;
            cum = &cum + &p;
        }
        Some(&(&Q::ONE - &recent) + &(&recent * &Q::int(kk)).checked_div(&expect)?)
    }

    /// Metallicity relative to the Sun's birth gas: the mass-weighted relative
    /// abundance of what metals are mostly made of (O, C, Fe, Si, Mg, N).
    pub fn metals(&self, r: &Q, t: &Q) -> Option<Q> {
        let cal = Calibration::held()?;
        Self::metals_of(cal, self.model.concs(ring_of(r), t).as_deref())
    }

    /// The metallicity from the channels' concentrations, read once.
    fn metals_of(cal: &Calibration, concs: Option<&[Q]>) -> Option<Q> {
        let (mut w, mut tot) = (Q::ZERO, Q::ZERO);
        for s in ["O", "C", "Fe", "Si", "Mg", "N"] {
            let ei = cal.elements.iter().position(|e| e.sym == s)?;
            let e = &cal.elements[ei];
            let wt = &e.a * &e.abundance;
            let (sm, mg) = cal.parts(concs, ei)?;
            tot = &tot + &(&wt * &(&sm + &mg));
            w = &w + &wt;
        }
        tot.checked_div(&w)
    }
}
