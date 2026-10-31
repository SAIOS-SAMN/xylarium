//! Propulsion: a drive's exhaust speed and speed change from the energy its
//! fuel releases per kilogram.
//!
//! Inputs: a people's `cosmic::Traits` (its solvent sets the chemical
//! fuel's exhaust molecule), the universe's `xyl_cosmos::laws::Scales`
//! (covalent bond energy, nuclear scale), the element table (hydrogen and
//! helium masses for fusion), and light's speed from `package::Si`.
//!
//! - Energy ([`energy_per_kg`], J/kg) per [`Drive`]: chemical, the covalent
//!   bond over the exhaust molecule's mass; fission, [`FISSION_SHARE`] of
//!   rest mass; fusion, [`fusion_share`]; annihilation, c^2.
//! - Exhaust ([`exhaust_speed`], m/s): the turning law's speed for the
//!   energy's share of c^2, v_e = e / c, capped at c (no square root of
//!   2e). Chemical bonds give centimetres per second, nuclear fuels less
//!   than one percent of c, and annihilation c.
//! - Voyage: [`delta_v`] = c tanh((v_e / c) ln R), the relativistic rocket
//!   (Tsiolkovsky's v_e ln R at low speed, always below c);
//!   [`propellant_per_kg`] = R - 1. transit/voyage.rs spends the speed
//!   change.
//!
//! Outputs are bundled as a [`Spec`] (`Spec::of`). civ/mod.rs and
//! transit/voyage.rs build each people's chemical drive and blend it toward
//! `transit::best_spec` by energy mastery. Declared rulers:
//! [`FISSION_SHARE`], [`CHEM_EXHAUST_AMU`], [`MASS_RATIO`]. Fuel choice,
//! bunkers, ports, refits and their prices are not modelled here.

use crate::cosmic::Traits;
use xyl_core::dyadic::Dy;
use xyl_cosmos::laws::Scales;
use xyl_cosmos::worlds::{covalent_ev, Solvent};

/// Light's speed (m/s).
pub fn c_m_s() -> Dy {
    Dy::from_q(&xyl_cosmos::package::Si::c())
}

/// An electronvolt (J), measured.
pub fn ev_j() -> Dy {
    Dy::sci(1_602_177, -25)
}

/// An atomic mass unit (kg), measured.
pub fn amu_kg() -> Dy {
    Dy::sci(166_054, -32)
}

/// The share of rest mass a fissioning heavy nucleus releases, 9.14e-4
/// (measured: about 200 MeV per 235 u), as (mantissa, decimal exponent). It
/// is a declared ruler because the binding curve is not yet derived in
/// xyl-cosmos; `energy_per_kg` scales it by the universe's nuclear scale.
pub const FISSION_SHARE: (i64, i32) = (914, -6);

/// The exhaust molecule's mass (amu) for a chemistry whose solvent is not a
/// volatile (a melt or a plasma). Declared ruler: the mass of a light oxide.
pub const CHEM_EXHAUST_AMU: i64 = 18;

/// A single stage's mass ratio, fuelled mass over dry mass. Declared ruler:
/// a stage whose propellant is nine tenths of its fuelled mass.
pub const MASS_RATIO: i64 = 10;

/// The kind of fuel a drive burns, which sets the energy released per kg.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Drive {
    /// The covalent bonds of the people's chemistry.
    Chemical,
    /// Heavy nuclei split (uranium, thorium).
    Fission,
    /// Hydrogen fused to helium.
    Fusion,
    /// Matter annihilated with antimatter, which has to be manufactured
    /// because no natural reservoir is modelled.
    Annihilation,
}

impl Drive {
    /// Every drive, in declaration order.
    pub const ALL: [Drive; 4] = [
        Drive::Chemical,
        Drive::Fission,
        Drive::Fusion,
        Drive::Annihilation,
    ];

    /// The drive's lower-case display name.
    pub fn name(self) -> &'static str {
        match self {
            Drive::Chemical => "chemical",
            Drive::Fission => "fission",
            Drive::Fusion => "fusion",
            Drive::Annihilation => "annihilation",
        }
    }
}

/// The share of rest mass that hydrogen fusing to helium releases,
/// (4 m_H - m_He) / (4 m_H), from the element table's measured masses.
/// Returns zero when either element is missing or the share is negative.
pub fn fusion_share(elements: &[xyl_cosmos::elements::Element]) -> Dy {
    let a = |s: &str| {
        elements
            .iter()
            .find(|e| e.sym == s)
            .map(|e| Dy::from_q(&e.a))
    };
    match (a("H"), a("He")) {
        (Some(h), Some(he)) if h.signum() > 0 => {
            ((Dy::int(4) * h - he) / (Dy::int(4) * h)).max(Dy::ZERO)
        }
        _ => Dy::ZERO,
    }
}

/// The energy a drive's fuel releases per kg (J/kg) under the universe's
/// scales `sc`, for a people whose solvent is `solvent`.
pub fn energy_per_kg(
    drive: Drive,
    solvent: Solvent,
    sc: &Scales,
    elements: &[xyl_cosmos::elements::Element],
) -> Dy {
    let c2 = c_m_s() * c_m_s();
    match drive {
        Drive::Chemical => {
            let amu = match solvent {
                Solvent::Volatile(s) => s.mu(),
                _ => Dy::int(CHEM_EXHAUST_AMU),
            };
            covalent_ev(sc) * ev_j() / (amu * amu_kg())
        }
        Drive::Fission => Dy::sci(FISSION_SHARE.0, FISSION_SHARE.1) * sc.nuclear * c2,
        Drive::Fusion => fusion_share(elements) * sc.nuclear * c2,
        Drive::Annihilation => c2,
    }
}

/// Exhaust speed (m/s): the turning law's speed v_e = e / c for the energy
/// `e_per_kg`, capped at c.
pub fn exhaust_speed(e_per_kg: Dy) -> Dy {
    (e_per_kg.max(Dy::ZERO) / c_m_s()).min(c_m_s())
}

/// The hyperbolic tangent, tanh x = (1 - e^(-2x)) / (1 + e^(-2x)), computed
/// from exp; a negative x is clamped to zero.
fn tanh(x: Dy) -> Dy {
    let e = (Dy::int(-2) * x.max(Dy::ZERO)).exp();
    (Dy::ONE - e) / (Dy::ONE + e)
}

/// The speed change a mass ratio gives (m/s): the relativistic rocket,
/// c tanh((v_e / c) ln R), which reduces to Tsiolkovsky's v_e ln R at low
/// speed. A ratio below one is taken as one.
pub fn delta_v(v_e: Dy, ratio: Dy) -> Dy {
    let c = c_m_s();
    c * tanh(v_e / c * ratio.max(Dy::ONE).log())
}

/// The propellant burned per kg of dry mass at this mass ratio (kg per kg):
/// R - 1, never below zero.
pub fn propellant_per_kg(ratio: Dy) -> Dy {
    (ratio - Dy::ONE).max(Dy::ZERO)
}

/// A drive's performance: the fuel's energy per kg and the exhaust speed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    /// The drive this spec describes.
    pub drive: Drive,
    /// The energy the fuel releases, in J/kg.
    pub energy_per_kg: Dy,
    /// Exhaust speed, in m/s, at most c.
    pub v_e: Dy,
}

impl Spec {
    /// The spec of `drive` for a people of `traits` (the people's solvent
    /// sets the chemical fuel) under the universe's scales `sc` and element
    /// table `elements`.
    pub fn of(
        drive: Drive,
        traits: &Traits,
        sc: &Scales,
        elements: &[xyl_cosmos::elements::Element],
    ) -> Spec {
        let e = energy_per_kg(drive, traits.solvent, sc, elements);
        Spec {
            drive,
            energy_per_kg: e,
            v_e: exhaust_speed(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xyl_cosmos::worlds::Species;

    #[test]
    fn exhaust_follows_fuel_energy_and_stays_below_c() {
        let sc = Scales::default();
        let el = xyl_cosmos::elements::table().unwrap();
        let water = Solvent::Volatile(Species::H2O);
        let chem = exhaust_speed(energy_per_kg(Drive::Chemical, water, &sc, &el));
        let fis = exhaust_speed(energy_per_kg(Drive::Fission, water, &sc, &el));
        let fus = exhaust_speed(energy_per_kg(Drive::Fusion, water, &sc, &el));
        let anni = exhaust_speed(energy_per_kg(Drive::Annihilation, water, &sc, &el));
        // Each fuel's exhaust speed is e / c, ordered as the fuels' energies
        // are ordered; annihilation reaches c.
        let c = c_m_s();
        assert!(chem > Dy::ZERO && chem < fis, "chemical {chem}");
        assert!(fis < fus && fus < c, "fission {fis} fusion {fus}");
        assert_eq!(anni, c);
        // A lighter exhaust molecule gives a higher exhaust speed for the
        // same bond energy.
        let methane = exhaust_speed(energy_per_kg(
            Drive::Chemical,
            Solvent::Volatile(Species::CH4),
            &sc,
            &el,
        ));
        assert!(methane > chem);
        // The speed change matches Tsiolkovsky at low speed and stays below
        // c at high speed.
        let dv = delta_v(chem, Dy::int(MASS_RATIO));
        let tsiolkovsky = chem * Dy::int(MASS_RATIO).log();
        assert!(
            (dv - tsiolkovsky).abs() < tsiolkovsky / Dy::int(1_000_000),
            "{dv} vs {tsiolkovsky}"
        );
        assert!(delta_v(anni, Dy::int(MASS_RATIO)) < c);
        assert!(delta_v(anni, Dy::int(MASS_RATIO)) > fus);
    }
}
