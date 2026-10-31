//! The unit world (a ruler) and the degrees of a star's life.

use super::*;

/// The unit world: the unit mass and radius at the unit star's distance and
/// age, of solar metals, with the reference world's bulk-silicate uranium,
/// thorium and potassium - the reference world (hydro.rs uses its field as
/// the unit of fields).
/// Computed under default laws; it is a ruler, not a calibration target.
pub fn unit() -> &'static State {
    static U: OnceLock<State> = OnceLock::new();
    U.get_or_init(|| {
        // The core's share of rock from solar abundances (Lodders 2003): metal
        // (Fe, Ni, S) against silicate (Mg, Si and three O).
        let core = Q::ratio(63_988, 164_468);
        let mantle = &Q::ONE - &core;
        // Ices arrive by cometary delivery across the snow line at unit
        // crowding and metals.
        let late = constants().delivered;
        let ices: Vec<(&'static str, Q)> = Species::ALL
            .iter()
            .map(|s| (s.name(), (late * s.comet()).to_q()))
            .collect();
        // Layers: the delivered ices over the rock, split into core and
        // mantle.
        let ice_sum = ices.iter().fold(Q::ZERO, |a, (_, v)| &a + v);
        let ice = ice_sum
            .checked_div(&(&Q::ONE + &ice_sum))
            .unwrap_or(Q::ZERO);
        let rock = &Q::ONE - &ice;
        let layers_core = &core * &rock;
        let layers = [layers_core.clone(), &mantle * &rock, ice, Q::ZERO];
        let world = World {
            a: Q::ONE,
            mass: Q::ONE,
            radius: Q::ONE,
            giant: false,
            host: None,
            a_host_radii: None,
            layers,
            elements: Vec::new(),
            mantle: Vec::new(),
            ices,
            e_forced: None,
            swing: Q::ZERO,
            phase: Q::ZERO,
            merged_at: None,
        };
        // Bulk-silicate isotopes per kg of mantle today (U238, U235, Th232,
        // K40), scaled back to the Sun's birth 4.57 Gyr ago below.
        let today = [
            Dy::sci(19856, -12),
            Dy::sci(144, -12),
            Dy::sci(80, -9),
            Dy::sci(2808, -11),
        ];
        // The mantle mass is all but the core (bodies::interior's
        // m_kg - m_core).
        let mantle_kg =
            (Dy::ONE - Dy::from_q(&layers_core)) * Dy::from_q(&crate::package::Si::world_mass_kg());
        let mut sources: [Q; 4] = Default::default();
        for (i, conc) in today.iter().enumerate() {
            let half = bodies::isotopes()[i].2.clone();
            let back = (Dy::sci(457, -2) * xyl_core::dyadic::ln2() / Dy::from_q(&half)).exp();
            sources[i] = (mantle_kg * *conc * back).to_q();
        }
        let set = Setting {
            world: &world,
            star_mass: Dy::ONE,
            lum: Dy::ONE,
            a_au: Dy::ONE,
            z: Dy::ONE,
            r_kpc: Dy::int(8),
            compactness: crate::galaxy::first_star_compactness(),
            sequence: crate::galaxy::first_star_sequence(),
            sources,
            path: None,
        };
        build(&set, Dy::sci(457, -2), &Ctx::new(Scales::default()))
    })
}

/// One degree of a star's life (Gyr): its main-sequence lifetime over 360,
/// never shorter than one core period. A world changes at its star's pace,
/// so it is treated as unchanged within a degree. The lifetime is that of a
/// star of transduction sequence `seq` and compactness `delta` at `r_kpc` from
/// the core (`life::lifetime`).
pub fn star_degree_gyr(seq: &crate::stars::Sequence, delta: Dy, r_kpc: Dy, ctx: &Ctx) -> Dy {
    let ring = crate::circuit::gyr_of_rings_dy(crate::pulse::core_period());
    (crate::life::lifetime(seq, delta, r_kpc, ctx) / Dy::int(360))
        .max(ring)
        .max(Dy::sci(1, -12))
}

/// The degree of its star's life an age (Gyr) lies nearest.
pub fn degree_of(age: Dy, degree_gyr: Dy) -> i64 {
    (age / degree_gyr + Dy::ratio(1, 2)).trunc_i64()
}

/// The age (Gyr) at a degree of its star's life.
pub fn age_of_degree(k: i64, degree_gyr: Dy) -> Dy {
    Dy::int(k) * degree_gyr
}
