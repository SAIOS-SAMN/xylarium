// Z, Q and R: a kind's protons (Z), neutrons (Q) and the electrons' phase
// state (R) at the bounce's pulse p (turns).
//
// Inputs: the universal table (`formation::founding_shells`, set by the
// founding), `compound::parts` and
// `compound::charge`, `fibres::grow` (electron coherence kappa on S^11) and
// `nuclei_of` (grain.rs); a `Substrate`'s outflow and intakes.
//
// Outputs: `TwoWeights` (protons, neutrons, `mass`, `neutron_share`);
// `rest_neutrons` (an atom's Q from the universal table, rational);
// `mass_number` (read back to whole nucleons, A = floor(Z + Q + 1/2));
// `kind_capacity` and `mix_capacity` (the dipole's capacity, `3Q = 3 cap`);
// `neutrons_of` (`rest_neutrons`, summed over a cluster's parts); `nucleus_weights`; `mix_weights`
// (whole nuclei only); `electron_phase`, R = kappa (1 - 2 frac(p)) in
// [-kappa, kappa], zero for a kind with no charge; `fill_shares`, the fill's
// Q-side share on each of the six Hopf planes at its plane's capture phase
// `psi_p = (w_p s Z) mod 1` (#201); and `Substrate::two_weights`,
// the universe's nuclides at t (firmament held, galaxies, matter in
// flight). All counts are exact rationals.

use super::*;
use xyl_core::hopf::PLANES;
use xyl_core::Q;

/// Z, Q and R: Z the protons, Q the neutrons, R the electrons' phase state at
/// the bounce's pulse p (turns).
/// - Z is the charge; Q is the conversion's rest point from the universal
///   table, an exact rational (a cluster sums its parts). A kind is a nuclide (Z, Q) with weight
///   Z + Q nucleons.
/// - R is the electrons' coherence kappa (the share fibred on S^11, fibres.rs)
///   times the hemisphere response 1 - 2 frac(p), so R = kappa (1 - 2 frac(p))
///   in [-kappa, kappa]: held face early in the echo, zero at its turn, open
///   face late. R is not stored; `electron_phase` computes it.
#[derive(Clone, Debug, Default)]
pub struct TwoWeights {
    /// Z: proton count (summed over whole nuclei for a mix).
    pub protons: Q,
    /// Q: neutron count at the rest point (summed over whole nuclei for a
    /// mix).
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

/// The universal table: each atom's neutrons per nucleus at the
/// conversion's rest point, built at the universal core's bounce
/// (`formation::founding_shells`) and set once per process by the founding.
static NEUTRONS: std::sync::OnceLock<std::collections::BTreeMap<u32, Q>> =
    std::sync::OnceLock::new();

/// Whether a founding is under way in this process: a reading taken then
/// does not found again, and reads the capture state.
static FOUNDING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Sets the universal table at the founding; the first founding in the
/// process wins.
pub(crate) fn set_neutrons(table: std::collections::BTreeMap<u32, Q>) {
    let _ = NEUTRONS.set(table);
}

/// Marks a founding under way (`true`) or done (`false`).
pub(crate) fn set_founding(on: bool) {
    FOUNDING.store(on, std::sync::atomic::Ordering::SeqCst);
}

/// The process's universal table. Where no founding has happened yet, the
/// substrate is founded first at the registered spin (as `anchor`); `None`
/// where no spin is registered, during the founding, or where it fails.
fn neutron_table() -> Option<&'static std::collections::BTreeMap<u32, Q>> {
    if let Some(t) = NEUTRONS.get() {
        return Some(t);
    }
    if !FOUNDING.load(std::sync::atomic::Ordering::SeqCst) {
        let _ = super::anchor();
    }
    NEUTRONS.get()
}

/// An atom's neutrons per nucleus `Q_z` from the universal table: the
/// conversion's rest point at its parcels' reaches, an exact rational
/// (E014, #201). `None` for a cluster, a kind with no parcel or before the
/// founding: such a kind is in its capture state, `Q = 0`.
pub fn rest_neutrons(z: u32) -> Option<Q> {
    // Stand-in S138: Every galaxy reads the universal table, not its core's own.
    neutron_table()?.get(&z).cloned()
}

/// An atom's mass number read back to whole nucleons, `A_z = floor(Z + Q_z
/// + 1/2)` ([`rest_neutrons`], #149). `None` where `Q_z` is not read.
pub fn mass_number(z: u32) -> Option<u32> {
    let a =
        (&(&Q::int(i64::from(z)) + &rest_neutrons(z)?) + &xyl_core::ratio!(1, 2)).floor_i128()?;
    u32::try_from(a).ok().filter(|&a| a > 0)
}

/// A kind's neutrons: each atom's [`rest_neutrons`] (zero in the capture
/// state); a cluster sums its parts.
pub fn neutrons_of(k: u32) -> Q {
    crate::compound::parts(k)
        .iter()
        .filter(|(z, n)| *z > 0 && *n > 0)
        .fold(Q::ZERO, |acc, (z, n)| {
            &acc + &(&rest_neutrons(*z).unwrap_or(Q::ZERO) * &Q::int(i64::from(*n)))
        })
}

/// The protons and neutrons of one nucleus of kind k.
pub fn nucleus_weights(k: u32) -> TwoWeights {
    TwoWeights {
        protons: Q::int(i64::from(crate::compound::charge(k))),
        neutrons: neutrons_of(k),
    }
}

/// A mix's nuclides summed: each kind's whole nuclei (amount in held count
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

/// A kind's capacity: an atom's `atoms::capacity`, a cluster's the sum over
/// its parts; zero where none is read. `3Q = 3 cap` is the dipole's Q
/// side beside the span `3Z` (docs/model.md, The dipole).
pub fn kind_capacity(k: u32) -> Q {
    crate::compound::parts(k)
        .iter()
        .filter(|(z, n)| *z > 0 && *n > 0)
        .fold(Q::ZERO, |acc, (z, n)| {
            let cap = crate::atoms::capacity(*z).map_or(0, i64::from);
            &acc + &Q::int(cap * i64::from(*n))
        })
}

/// A mix's capacity summed over each kind's whole nuclei (as
/// [`mix_weights`]): with the protons it gives the mix's `3Z` and `3Q`.
pub fn mix_capacity(m: &Mix) -> Q {
    m.iter().fold(Q::ZERO, |acc, (k, a)| {
        &acc + &(&nuclei_of(a, *k).1 * &kind_capacity(*k))
    })
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

/// The shares of a kind's fill entering from the Q side toward Z on each
/// Hopf plane (#201): per plane `p`, the face of the electron phase's channel,
/// which moves `R_p u` of the kind's states from Q to Z against its electron
/// hold `W_e = 3Z - U` (the jetstream law), `f_p = |R_p| u / (W_e + |R_p|
/// u)`. `R_p` is read at the plane's capture phase about the throat, `psi_p = (w_p s Z) mod
/// 1`, with `s` the torsion share (the crossing phase the founding carries)
/// and `w_p` the plane's winding: the fibre turns plane `p` at `w_p` times
/// the common rate, and its coherence `kappa` is the whole fibre's on every
/// plane. The sign of `R_p` picks the plane's direction: for `R_p > 0` the
/// channel runs Q to Z and `f_p` is that face; for `R_p <= 0` it runs Z to
/// Q and the plane's fill enters from the Z side alone (`f_p = 0`). A kind
/// with no open state has no aperture on any plane. In the common phase
/// (`hopf::COMMON`, the Hopf action) the six shares are equal.
pub fn fill_shares(k: u32, share: &Q, winding: &[i64; PLANES]) -> [Q; PLANES] {
    let u = Q::int(i64::from(crate::compound::open(k)));
    let w_e = Q::int(i64::from(crate::compound::electron_hold(k)));
    let charge = &(share * &Q::int(i64::from(k)));
    winding.map(|wp| {
        let psi = (charge * &Q::int(wp)).frac();
        let r = electron_phase(k, &psi);
        if r.signum() <= 0 || u.signum() <= 0 {
            return Q::ZERO;
        }
        let channel = &r * &u;
        channel.checked_div(&(&w_e + &channel)).unwrap_or(Q::ZERO)
    })
}

/// [`fill_shares`] at the founded universe's torsion share, read from the
/// anchor's admitted spin, in the common phase: the shares of the universal
/// table (#201).
pub fn universal_fill_shares(k: u32) -> [Q; PLANES] {
    let share = crate::bounce::torsion_share(&Q::ONE, &crate::circuit::anchor().spin);
    fill_shares(k, &share, &xyl_core::hopf::COMMON)
}

impl Substrate {
    /// The universe's nuclides at `t` rings: the firmament still held, every
    /// galaxy's composition (recipe plus intake), and matter in flight.
    pub fn two_weights(
        &self,
        t: xyl_core::dyadic::Dy,
        elements: &[crate::elements::Element],
    ) -> TwoWeights {
        let mut all = Mix::new();
        // The galaxies' compositions already include their intake, so the
        // firmament adds its held part (amount - left) and the matter in
        // flight (left - taken).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_plane_fills_at_its_own_capture_phase() {
        // In the common phase the six planes' shares are equal; at any
        // winding each plane's share is the share of a common winding of its
        // own rate, and every share is a face in [0, 1), zero without an
        // aperture.
        let share = xyl_core::ratio!(3, 5);
        let winding = [1, 2, -1, 3, 0, 5];
        for k in 1..=crate::elements::TOP_Z {
            let common = fill_shares(k, &share, &xyl_core::hopf::COMMON);
            assert!(common.iter().all(|f| f == &common[0]), "Z{k}");
            let f = fill_shares(k, &share, &winding);
            for (p, wp) in winding.iter().enumerate() {
                assert_eq!(f[p], fill_shares(k, &share, &[*wp; PLANES])[0], "Z{k}");
                assert!(f[p].signum() >= 0 && f[p] < Q::ONE, "Z{k}");
                if crate::compound::open(k) == 0 {
                    assert_eq!(f[p], Q::ZERO, "Z{k}");
                }
            }
        }
    }
}
