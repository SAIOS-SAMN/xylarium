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
//!   energy's share of light's, v_e = e / c, capped at c (no square root of
//!   2e). Chemical bonds give centimetres per second, nuclear fuels under a
//!   percent of light, annihilation light itself.
//! - Voyage: [`delta_v`] = c tanh((v_e / c) ln R), the relativistic rocket
//!   (Tsiolkovsky's v_e ln R while slow, below c always);
//!   [`propellant_per_kg`] = R - 1. transit.rs spends the speed change.
//!
//! Outputs are bundled as a [`Spec`] (`Spec::of`). civ/mod.rs and transit.rs
//! build each people's chemical drive and blend it toward
//! `transit::best_spec` by energy mastery. Declared rulers:
//! [`FISSION_SHARE`], [`CHEM_EXHAUST_AMU`], [`MASS_RATIO`]. Fuel choice,
//! bunkers, ports, refits and their prices are not modelled here.

use crate::cosmic::Traits;
use xyl_core::dyadic::Dy;
use xyl_cosmos::laws::Scales;
use xyl_cosmos::worlds::{covalent_ev, Solvent};

fn n(x: i64) -> Dy {
    Dy::int(x)
}

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

/// The share of its rest mass a fissioning heavy nucleus releases (measured:
/// about 200 MeV per 235 u). Declared ruler: the binding curve is not yet
/// derived in xyl-cosmos; scaled by the universe's nuclear scale.
pub const FISSION_SHARE: (i64, i32) = (914, -6);

/// The exhaust molecule's mass (amu) of a chemistry with no volatile of its
/// own (a melt's or a plasma's). Declared ruler: a light oxide's.
pub const CHEM_EXHAUST_AMU: i64 = 18;

/// A single stage's mass ratio, fuelled over dry. Declared ruler: a stage
/// whose tanks are nine tenths of its mass.
pub const MASS_RATIO: i64 = 10;

/// What a fuel releases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Drive {
    /// Its own chemistry's bonds.
    Chemical,
    /// Heavy nuclei split (uranium, thorium).
    Fission,
    /// Hydrogen fused to helium.
    Fusion,
    /// Matter annihilated (it must be made: nature holds none).
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

/// The share of a nucleus's rest mass hydrogen fusing to helium releases:
/// four hydrogen atoms' mass less helium's, over four hydrogens' (from the
/// element table's measured masses).
pub fn fusion_share(elements: &[xyl_cosmos::elements::Element]) -> Dy {
    let a = |s: &str| {
        elements
            .iter()
            .find(|e| e.sym == s)
            .map(|e| Dy::from_q(&e.a))
    };
    match (a("H"), a("He")) {
        (Some(h), Some(he)) if h.signum() > 0 => ((n(4) * h - he) / (n(4) * h)).max(Dy::ZERO),
        _ => Dy::ZERO,
    }
}

/// The energy a drive's fuel releases per kg (J/kg), under a universe's laws,
/// for a people of this chemistry.
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
                _ => n(CHEM_EXHAUST_AMU),
            };
            covalent_ev(sc) * ev_j() / (amu * amu_kg())
        }
        Drive::Fission => Dy::sci(FISSION_SHARE.0, FISSION_SHARE.1) * sc.nuclear * c2,
        Drive::Fusion => fusion_share(elements) * sc.nuclear * c2,
        Drive::Annihilation => c2,
    }
}

/// Exhaust speed (m/s): the turning law's speed for the energy's share of
/// light's, e / c, never past light.
pub fn exhaust_speed(e_per_kg: Dy) -> Dy {
    (e_per_kg.max(Dy::ZERO) / c_m_s()).min(c_m_s())
}

/// tanh x, exactly from exp.
fn tanh(x: Dy) -> Dy {
    let e = (n(-2) * x.max(Dy::ZERO)).exp();
    (Dy::ONE - e) / (Dy::ONE + e)
}

/// The speed change a mass ratio buys (m/s): the relativistic rocket,
/// c tanh((v_e / c) ln R) - Tsiolkovsky's v_e ln R while it is slow.
pub fn delta_v(v_e: Dy, ratio: Dy) -> Dy {
    let c = c_m_s();
    c * tanh(v_e / c * ratio.max(Dy::ONE).log())
}

/// The fuel a payload burns over a voyage at this mass ratio (kg per kg):
/// R - 1 (mass moved over exhaust speed).
pub fn propellant_per_kg(ratio: Dy) -> Dy {
    (ratio - Dy::ONE).max(Dy::ZERO)
}

/// What a drive does: its fuel's energy per kg and its exhaust speed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    /// The drive this spec describes.
    pub drive: Drive,
    /// Energy its fuel releases, in J/kg.
    pub energy_per_kg: Dy,
    /// Exhaust speed, in m/s, never past light.
    pub v_e: Dy,
}

impl Spec {
    /// The spec of `drive` for a people of `traits` (its chemistry sets the
    /// chemical fuel) under the universe's scales `sc` and its `elements`.
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
    fn exhaust_follows_the_energy_its_fuel_releases_and_light_caps_it() {
        let sc = Scales::default();
        let el = xyl_cosmos::elements::table().unwrap();
        let water = Solvent::Volatile(Species::H2O);
        let chem = exhaust_speed(energy_per_kg(Drive::Chemical, water, &sc, &el));
        let fis = exhaust_speed(energy_per_kg(Drive::Fission, water, &sc, &el));
        let fus = exhaust_speed(energy_per_kg(Drive::Fusion, water, &sc, &el));
        let anni = exhaust_speed(energy_per_kg(Drive::Annihilation, water, &sc, &el));
        // each fuel's exhaust is the energy's share of light's times c, in
        // the order of what the fuels release, annihilation at light itself
        let c = c_m_s();
        assert!(chem > Dy::ZERO && chem < fis, "chemical {chem}");
        assert!(fis < fus && fus < c, "fission {fis} fusion {fus}");
        assert_eq!(anni, c);
        // a lighter exhaust molecule runs faster on the same bond
        let methane = exhaust_speed(energy_per_kg(
            Drive::Chemical,
            Solvent::Volatile(Species::CH4),
            &sc,
            &el,
        ));
        assert!(methane > chem);
        // Tsiolkovsky while slow, never past light when fast
        let dv = delta_v(chem, n(MASS_RATIO));
        let tsiolkovsky = chem * n(MASS_RATIO).log();
        assert!(
            (dv - tsiolkovsky).abs() < tsiolkovsky / n(1_000_000),
            "{dv} vs {tsiolkovsky}"
        );
        assert!(delta_v(anni, n(MASS_RATIO)) < c);
        assert!(delta_v(anni, n(MASS_RATIO)) > fus);
    }
}
