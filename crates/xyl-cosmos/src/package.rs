//! The package: what a universe is founded from that its laws do not derive.
//! A genesis is seeded from this and its seed (the genesis record names it
//! with the founding's digest); everything else is derived from them. The
//! core's mass is the one bridge to SI, and the SI constants are the seam
//! where measured laws enter, converted once. The
//! founding reads only this, never the layers built on top of it, so a
//! change to a world, a disc or a life law keeps the genesis.

use crate::elements::decimal;
use xyl_core::Q;

fn d(s: &str) -> Q {
    decimal(s).unwrap_or(Q::ZERO)
}

/// The universal core's mass (suns): the package's one bridge to SI, the
/// mass whose gravitational radius is the unit of length (xyl-core
/// units.rs). The home galaxy surrounds the core, so its hole is the core.
/// Declared by the package until the founding manifest supplies it; its spin is the
/// parent's spin, the universe's own draw (bang.rs).
pub fn hole_mass() -> Q {
    Q::int(4_300_000)
}

/// Units (SI): the unit world's mass and radius (5.972e24 kg, 6.371e6 m), the planet layer's unit.
pub struct Si;
impl Si {
    /// The unit world mass, 5.972e24 kg.
    pub fn world_mass_kg() -> Q {
        d("5.972e24")
    }
    /// The unit world radius, 6.371e6 m.
    pub fn world_radius_m() -> Q {
        d("6.371e6")
    }
    /// The solar mass, 1.989e30 kg.
    pub fn star_mass_kg() -> Q {
        d("1.989e30")
    }
    /// The solar radius, 6.957e8 m: the one-sun star's rim, where its deficit
    /// share is read (stars::delta_star).
    pub fn star_radius_m() -> Q {
        d("6.957e8")
    }
    /// The astronomical unit, 1.496e11 m.
    pub fn au() -> Q {
        d("1.496e11")
    }
    /// The gravitational constant, 6.674e-11 m^3/(kg s^2).
    pub fn g() -> Q {
        d("6.674e-11")
    }
    /// Seconds in one Gyr: a thousand million years (`yr_s`).
    pub fn gyr_s() -> Q {
        &Si::yr_s() * &Q::int(1_000_000_000)
    }
    /// The kiloparsec, 3.0857e19 m.
    pub fn kpc() -> Q {
        d("3.0857e19")
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
        d("3.15576e7")
    }
    /// The Boltzmann constant, 1.380649e-23 J/K.
    pub fn kb_j() -> Q {
        d("1.380649e-23")
    }
    /// The Planck constant, 6.62607015e-34 J s.
    pub fn h_j() -> Q {
        d("6.62607015e-34")
    }
    /// The electronvolt, 1.602176634e-19 J.
    pub fn ev_j() -> Q {
        d("1.602176634e-19")
    }
    /// The atomic mass unit, 1.66053907e-27 kg.
    pub fn amu_kg() -> Q {
        d("1.66053907e-27")
    }
    /// The hydrogen atom's mass, 1.6726e-27 kg.
    pub fn m_h_kg() -> Q {
        d("1.6726e-27")
    }
    /// The gas constant, 8.314 J/(mol K).
    pub fn r_gas() -> Q {
        d("8.314")
    }
    /// The Faraday constant, 96 485 C/mol.
    pub fn faraday() -> Q {
        Q::int(96_485)
    }
    /// The Boltzmann constant in electronvolts per kelvin, 8.617333e-5.
    pub fn kb_ev() -> Q {
        d("8.617333e-5")
    }
}

/// The founding's digest: the files the founding is derived from (the Big
/// Bang, the bounce, the elements and atoms, the cascade and lighting, the
/// circuit, this package and the mathematics), hashed at build time. A
/// genesis records it; a founding cached under it is the same founding, and
/// a change to any layer built on the founding leaves it unchanged.
pub fn founding_digest() -> &'static str {
    env!("XYL_FOUNDING_DIGEST")
}
