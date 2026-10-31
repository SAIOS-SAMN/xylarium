// Z, Q and R: a kind's protons (Z), neutrons (Q) and the electrons' phase
// state (R) at the bounce's pulse p (turns).
//
// Inputs: `elements::table` (mass numbers), `compound::parts` and
// `compound::charge`, `fibres::grow` (electron coherence kappa on S^11) and
// `nuclei_of` (grain.rs); a `Substrate`'s outflow and intakes.
//
// Outputs: `TwoWeights` (protons, neutrons, `mass`, `neutron_share`);
// `neutrons_of` (mass number rounded to the nearest nucleon, less the
// charge, summed over a cluster's parts); `nucleus_weights`; `mix_weights`
// (whole nuclei only); `electron_phase`, R = kappa (1 - 2 frac(p)) in
// [-kappa, kappa], zero for a neutral kind; and `Substrate::two_weights`,
// the universe's nuclides at t (firmament held, galaxies, matter in
// flight). All counts are exact rationals.

use super::*;
use xyl_core::Q;

/// Z, Q and R: Z the protons, Q the neutrons, R the electrons' phase state at
/// the bounce's pulse p (turns).
/// - Z is the charge; Q is the mass number less the charge (from the element
///   table; a cluster sums its parts). A kind is a nuclide (Z, Q) with weight
///   Z + Q nucleons.
/// - R is the electrons' coherence kappa (the share fibred on S^11, fibres.rs)
///   times the hemisphere response 1 - 2 frac(p), so R = kappa (1 - 2 frac(p))
///   in [-kappa, kappa]: held face early in the echo, zero at its turn, open
///   face late.
#[derive(Clone, Debug, Default)]
pub struct TwoWeights {
    /// Z: proton count (summed over whole nuclei for a mix).
    pub protons: Q,
    /// Q: neutron count, mass number less charge (summed over whole nuclei for a mix).
    pub neutrons: Q,
}

impl TwoWeights {
    /// The weight in nucleons, Z + Q.
    pub fn mass(&self) -> Q {
        &self.protons + &self.neutrons
    }
    /// The neutron share of the nucleons, Q / (Z + Q).
    pub fn neutron_share(&self) -> Option<Q> {
        self.neutrons.checked_div(&self.mass())
    }
}

/// A kind's neutrons: its mass number (the table's mass rounded to the nearest
/// whole nucleon) less its charge; a cluster sums its parts.
pub fn neutrons_of(k: u32) -> Q {
    let table = crate::elements::table().ok();
    let of_atom = |z: u32| -> Q {
        let Some(e) = table
            .as_ref()
            .and_then(|t| t.get(z as usize - 1))
            .filter(|e| e.z == z)
        else {
            return Q::ZERO;
        };
        let a = &e.a + &Q::new(1, 2).unwrap_or(Q::ZERO);
        let whole = &a - &a.frac();
        (&whole - &Q::int(i64::from(z))).max(Q::ZERO)
    };
    crate::compound::parts(k)
        .iter()
        .filter(|(z, n)| *z > 0 && *n > 0)
        .fold(Q::ZERO, |acc, (z, n)| {
            &acc + &(&of_atom(*z) * &Q::int(i64::from(*n)))
        })
}

/// The protons and neutrons of one nucleus of kind k.
pub fn nucleus_weights(k: u32) -> TwoWeights {
    TwoWeights {
        protons: Q::int(i64::from(crate::compound::charge(k))),
        neutrons: neutrons_of(k),
    }
}

/// A mix's nuclides summed: each kind's whole nuclei (amount in held frames
/// over one nucleus's) times its protons and neutrons.
pub fn mix_weights(m: &Mix) -> TwoWeights {
    let (mut p, mut n) = (Q::ZERO, Q::ZERO);
    for (k, a) in m {
        let count = nuclei_of(a, *k).1;
        let w = nucleus_weights(*k);
        p = &p + &(&count * &w.protons);
        n = &n + &(&count * &w.neutrons);
    }
    TwoWeights {
        protons: p,
        neutrons: n,
    }
}

/// R, a kind's electron phase state at the pulse p (turns): the electrons'
/// coherence on S^11 (fibres.rs, 2 G / ends for the charge's electrons) times
/// the hemisphere response 1 - 2 frac(p).
pub fn electron_phase(k: u32, p: &Q) -> Q {
    let charge = u64::from(crate::compound::charge(k));
    if charge == 0 {
        return Q::ZERO;
    }
    let c = crate::fibres::grow(charge);
    let (num, den) = c.kappa();
    let kappa = Q::new(
        i64::try_from(num).unwrap_or(0),
        i64::try_from(den).unwrap_or(1),
    )
    .unwrap_or(Q::ZERO);
    &kappa * &(&Q::ONE - &(&Q::int(2) * &p.frac()))
}

impl Substrate {
    /// The universe's nuclides at `t` Gyr: the firmament still held, every
    /// galaxy's composition (recipe plus intake), and matter in flight.
    pub fn two_weights(
        &self,
        t: xyl_core::dyadic::Dy,
        elements: &[crate::elements::Element],
    ) -> TwoWeights {
        let mut all = Mix::new();
        // the firmament still held, plus what has left it and is in flight
        // until a galaxy takes it (galaxy compositions carry their intake)
        let out = self.outflow(t);
        let mut taken: std::collections::BTreeMap<u32, Q> = std::collections::BTreeMap::new();
        for (i, took) in self.intakes(t) {
            for (z, a) in &took {
                let e = taken.entry(*z).or_insert(Q::ZERO);
                *e = &*e + &a.to_q();
            }
            if let Some(m) = self.composition_with(i, &took, elements) {
                for (z, a) in m {
                    add(&mut all, z, &a);
                }
            }
        }
        for (k, (_, left)) in self.firmament.iter().zip(&out) {
            let left = left.to_q();
            add(&mut all, k.z, &(&k.amount - &left).max(Q::ZERO));
            add(
                &mut all,
                k.z,
                &(&left - taken.get(&k.z).unwrap_or(&Q::ZERO)).max(Q::ZERO),
            );
        }
        mix_weights(&all)
    }
}
