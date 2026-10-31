//! The package: the inputs a universe is founded from that its laws do not
//! derive.
//!
//! Outputs: [`hole_mass`], the universal core's mass in suns (4.3e6, a
//! declared value), the one bridge to SI: its gravitational radius is the
//! unit of length (`xyl_core::units`); [`Si`], the SI constants and rulers
//! (unit world, sun, AU, G, c, kpc, Gyr, Boltzmann, Planck, eV, amu, gas and
//! Faraday constants) as exact rationals, the seam where measured laws enter
//! and are converted once; and [`founding_digest`], the build-time digest of
//! the files the founding is derived from (build.rs), recorded in a genesis.
//!
//! A genesis is determined by this package and its seed. The founding reads
//! only this module and the founding files listed in build.rs, so a change to a world, a
//! disc or a life law keeps the genesis and its cache.
//! Terms: package, founding digest (docs/glossary.md).

use crate::elements::decimal;
use xyl_core::Q;

/// The universal core's mass (suns): the package's one bridge to SI, the
/// mass whose gravitational radius is the unit of length (xyl-core
/// units.rs). The home galaxy surrounds the core, so its hole is the core.
/// Declared by the package until the founding manifest supplies it; its spin
/// is the parent's spin, drawn from the universe's seed (bang.rs).
pub fn hole_mass() -> Q {
    Q::int(4_300_000)
}

/// The SI constants and rulers the package declares, as exact rationals; the
/// unit world (5.972e24 kg, 6.371e6 m) is the planet layer's unit.
pub struct Si;
impl Si {
    /// The unit world mass, 5.972e24 kg.
    pub fn world_mass_kg() -> Q {
        decimal("5.972e24").unwrap_or(Q::ZERO)
    }
    /// The unit world radius, 6.371e6 m.
    pub fn world_radius_m() -> Q {
        decimal("6.371e6").unwrap_or(Q::ZERO)
    }
    /// The solar mass, 1.989e30 kg.
    pub fn star_mass_kg() -> Q {
        decimal("1.989e30").unwrap_or(Q::ZERO)
    }
    /// The solar radius, 6.957e8 m: the one-sun star's rim, where its deficit
    /// share is read (stars::delta_star).
    pub fn star_radius_m() -> Q {
        decimal("6.957e8").unwrap_or(Q::ZERO)
    }
    /// The astronomical unit, 1.496e11 m.
    pub fn au() -> Q {
        decimal("1.496e11").unwrap_or(Q::ZERO)
    }
    /// The gravitational constant, 6.674e-11 m^3/(kg s^2).
    pub fn g() -> Q {
        decimal("6.674e-11").unwrap_or(Q::ZERO)
    }
    /// Seconds in one Gyr: a thousand million years (`yr_s`).
    pub fn gyr_s() -> Q {
        &Si::yr_s() * &Q::int(1_000_000_000)
    }
    /// The kiloparsec, 3.0857e19 m.
    pub fn kpc() -> Q {
        decimal("3.0857e19").unwrap_or(Q::ZERO)
    }
    /// The speed of light, 299 792 458 m/s.
    pub fn c() -> Q {
        Q::int(299_792_458)
    }
    /// Seconds in one day, 86 400.
    pub fn day_s() -> Q {
        Q::int(86_400)
    }
    /// Seconds in one year, 3.15576e7 (the Julian year).
    pub fn yr_s() -> Q {
        decimal("3.15576e7").unwrap_or(Q::ZERO)
    }
    /// The Boltzmann constant, 1.380649e-23 J/K.
    pub fn kb_j() -> Q {
        decimal("1.380649e-23").unwrap_or(Q::ZERO)
    }
    /// The Planck constant, 6.62607015e-34 J s.
    pub fn h_j() -> Q {
        decimal("6.62607015e-34").unwrap_or(Q::ZERO)
    }
    /// The electronvolt, 1.602176634e-19 J.
    pub fn ev_j() -> Q {
        decimal("1.602176634e-19").unwrap_or(Q::ZERO)
    }
    /// The atomic mass unit, 1.66053907e-27 kg.
    pub fn amu_kg() -> Q {
        decimal("1.66053907e-27").unwrap_or(Q::ZERO)
    }
    /// The hydrogen atom's mass, 1.6726e-27 kg.
    pub fn m_h_kg() -> Q {
        decimal("1.6726e-27").unwrap_or(Q::ZERO)
    }
    /// The gas constant, 8.314 J/(mol K).
    pub fn r_gas() -> Q {
        decimal("8.314").unwrap_or(Q::ZERO)
    }
    /// The Faraday constant, 96 485 C/mol.
    pub fn faraday() -> Q {
        Q::int(96_485)
    }
    /// The Boltzmann constant in electronvolts per kelvin, 8.617333e-5.
    pub fn kb_ev() -> Q {
        decimal("8.617333e-5").unwrap_or(Q::ZERO)
    }
}

/// The founding's digest: the files the founding is derived from (the Big
/// Bang, the bounce, the elements, atoms, compounds and fibres, the galactic
/// cores and the chain, this package, the formation cascade, the circuit, the
/// data tables and xyl-core), hashed at build time. A
/// genesis records it; a founding cached under it is the same founding, and
/// a change to any layer built on the founding leaves it unchanged.
pub fn founding_digest() -> &'static str {
    env!("XYL_FOUNDING_DIGEST")
}
