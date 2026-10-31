//! The calibration: each channel's yield scale fitted once per process to
//! the measured gas at `Rulers::r0` today, and the Sun's birth gas under it.

use super::{channel_index, ring_of, Model, Params, Rulers, BITS, CHANNELS};
use crate::elements::{decimal, table, Channel, Element};
use std::sync::OnceLock;
use xyl_core::certified::{exp, ln};
use xyl_core::Q;

/// The calibration: each channel's yield scale (from the measured gas
/// rulers) and its unit-yield concentration at R0 today, on the history of
/// the first universe built in the process (one universe per process).
pub struct Calibration {
    pub(super) elements: Vec<Element>,
    model: Model,
    scale: Vec<Q>,
    reference: Vec<Q>,
    /// Each element's abundance in the Sun's birth gas (R0, 9.2 Gyr in rings, the
    /// reference origin) under this calibration; rulers measured on the
    /// Sun's matter (an isotope inventory) are read against it.
    sun: Vec<Q>,
}

static CAL: OnceLock<Option<Calibration>> = OnceLock::new();

impl Calibration {
    fn build(p: &Params) -> Option<Calibration> {
        let elements = table().ok()?;
        let model = Model::of(p);
        let k0 = ring_of(&Rulers::r0());
        let today = Rulers::today();
        let reference = model
            .concs(k0, &today)
            .unwrap_or_else(|| vec![Q::ONE; CHANNELS.len()]);
        let t = |s: &str| -> Option<Q> {
            let x = &Rulers::ism_today_dex(s) * &ln(&Q::int(10), BITS)?.mid();
            Some(exp(&x, BITS).mid())
        };
        let origin = |s: &str, c: Channel| {
            elements
                .iter()
                .find(|e| e.sym == s)
                .and_then(|e| {
                    e.origin
                        .iter()
                        .find(|(x, _)| *x == c)
                        .map(|(_, f)| f.clone())
                })
                .unwrap_or(Q::ZERO)
        };
        let k_cc = t("O")?;
        let k_ia = (&t("Fe")? - &(&origin("Fe", Channel::CoreCollapse) * &k_cc))
            .checked_div(&origin("Fe", Channel::TypeIa))?;
        let k_agb = (&t("N")? - &(&origin("N", Channel::CoreCollapse) * &k_cc))
            .checked_div(&origin("N", Channel::Agb))?;
        let scale = CHANNELS
            .iter()
            .map(|&c| match c {
                Channel::CoreCollapse => k_cc.clone(),
                Channel::TypeIa => k_ia.clone(),
                Channel::Agb => k_agb.clone(),
                _ => Q::ONE,
            })
            .collect();
        let mut cal = Calibration {
            elements,
            model,
            scale,
            reference,
            sun: Vec::new(),
        };
        // The Sun was born 9.2 Gyr after the origin (13.8 - 4.6), read in
        // rings: a calibration epoch, removed with `Rulers::today` in phase 5.
        let born = crate::turning::span_in_rings(&decimal("9.2").unwrap_or(Q::ZERO));
        let at_birth = cal.model.concs(k0, &born);
        let sun = (0..cal.elements.len())
            .map(|ei| {
                cal.parts(at_birth.as_deref(), ei)
                    .map(|(sm, mg)| &sm + &mg)
                    .unwrap_or(Q::ZERO)
            })
            .collect();
        cal.sun = sun;
        Some(cal)
    }

    /// The calibration on the history `p`, built once per process and
    /// shared; a failed build is also kept. Returns `None` when the element
    /// table fails to load, ln 10 cannot be enclosed or a
    /// channel's yield scale cannot be solved (a zero origin share for Fe
    /// or N).
    pub fn of(p: &Params) -> Option<&'static Calibration> {
        CAL.get_or_init(|| Calibration::build(p)).as_ref()
    }

    /// The calibration already built in this process, if any.
    pub(super) fn held() -> Option<&'static Calibration> {
        CAL.get().and_then(|c| c.as_ref())
    }

    /// Element `ei`'s (smooth, merger) relative parts from the channels'
    /// concentrations `concs` (`None`: nothing made yet at that place and
    /// time). Each channel contributes its origin share of the element times
    /// its yield scale times its concentration over the reference
    /// concentration; the origin share enters the smooth part unchanged.
    /// `None` for an index out of range.
    pub(super) fn parts(&self, concs: Option<&[Q]>, ei: usize) -> Option<(Q, Q)> {
        let e = self.elements.get(ei)?;
        let (mut smooth, mut merger) = (Q::ZERO, Q::ZERO);
        for (c, f) in &e.origin {
            if *c == Channel::Origin {
                smooth = &smooth + f;
                continue;
            }
            let Some(ci) = channel_index(*c) else {
                continue;
            };
            // No concentrations (before the first stars, or no gas): an
            // element any stellar channel makes is absent.
            let Some(concs) = concs else {
                return Some((Q::ZERO, Q::ZERO));
            };
            let v = (&(f * &self.scale[ci]) * &concs[ci])
                .checked_div(&self.reference[ci])
                .unwrap_or(Q::ZERO);
            if *c == Channel::Merger {
                merger = &merger + &v;
            } else {
                smooth = &smooth + &v;
            }
        }
        Some((smooth, merger))
    }
}

/// An element's abundance in the gas the Sun formed from, as the calibration
/// reads it (R0, 9.2 Gyr in rings): 1 for an element the origin alone made. `None`
/// before a chemistry has been built in the process or for an unknown
/// symbol.
pub fn reference_birth_gas(sym: &str) -> Option<Q> {
    let cal = Calibration::held()?;
    let ei = cal.elements.iter().position(|e| e.sym == sym)?;
    cal.sun.get(ei).cloned()
}
