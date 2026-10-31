//! A system's worlds: their settings and solved events.

use super::*;

/// What a galaxy's worlds are read against: its seed, Big Bang, chemistry,
/// element table and laws.
#[derive(Clone, Copy)]
pub struct Galaxy<'c> {
    /// The galaxy's seed, from which every world's draws derive.
    pub seed: u64,
    /// The Big Bang the galaxy grew from.
    pub bang: &'c BigBang,
    /// The galaxy's chemical evolution (metals by place and time).
    pub chem: &'c Chemistry,
    /// The element table.
    pub elements: &'c [Element],
    /// The universe's scales.
    pub sc: Scales,
}

/// The settings of a system's worlds, in its order: each planet's orbit and
/// spin drawn (bodies::orbits, keyed by `path` as the system was), each moon
/// on its giant's chain with the eccentricity its neighbours force, and each
/// world's isotopes from the elements it was built with, kept as its ring's
/// history and the universe's half-lives allow.
pub fn settings<'a>(gal: &Galaxy, path: &[u64], sys: &'a System, star: &Star) -> Vec<Setting<'a>> {
    let (seed, bang, elements, sc) = (gal.seed, gal.bang, gal.elements, &gal.sc);
    let z = gal.chem.metals(&star.r, &star.born).unwrap_or(Q::ZERO);
    let star_kg = &sys.spec.mass * &crate::package::Si::star_mass_kg();
    let planets: Vec<&World> = sys.worlds.iter().filter(|w| w.host.is_none()).collect();
    let mut parts: Vec<xyl_core::rng::Part> = vec!["orbits".into()];
    parts.extend(path.iter().map(|&p| p.into()));
    let mut rng = xyl_core::Stream::derive(seed, &parts);
    let orbits = bodies::orbits(&star_kg, &planets, &mut rng);
    // The ring's star-formation time scale is its first-light time.
    // The galaxy counts in rings; the isotope clocks read Gyr, converted here.
    let first = crate::circuit::gyr_of_rings(&bang.first_stars().mid());
    let tau_f = crate::galaxy::birth(&star.r)
        .map(|b| crate::circuit::gyr_of_rings(&b))
        .unwrap_or_else(|| first.clone());
    let born = crate::circuit::gyr_of_rings(&star.born);
    // Isotope inventories are measured in the reference world's bulk
    // silicate; dividing by the model's solar birth gas makes that gas carry
    // exactly the measured inventory.
    let mut kept = bodies::kept(&tau_f, &first, &born, sc);
    for (k, (_, el, _, _, _)) in kept.iter_mut().zip(bodies::isotopes().iter()) {
        let sun = crate::chem::reference_birth_gas(el).unwrap_or(Q::ONE);
        *k = k.checked_div(&sun).unwrap_or_else(|| k.clone());
    }
    let (star_mass, lum, z, r_kpc) = (
        Dy::from_q(&sys.spec.mass),
        Dy::from_q(&sys.spec.lum),
        Dy::from_q(&z),
        Dy::from_q(&star.r),
    );
    let mut planet = 0usize;
    let mut paths: Vec<Option<Path>> = Vec::with_capacity(sys.worlds.len());
    for w in &sys.worlds {
        let path = match (w.host, &w.a_host_radii) {
            (Some(h), Some(a_r)) => sys.worlds.get(h).map(|g| {
                let e = w.e_forced.as_ref().map(Dy::from_q).unwrap_or(Dy::ZERO);
                Path {
                    host_kg: Dy::from_q(&g.mass) * Dy::from_q(&crate::package::Si::world_mass_kg()),
                    a_m: Dy::from_q(a_r)
                        * Dy::from_q(&g.radius)
                        * Dy::from_q(&crate::package::Si::world_radius_m()),
                    e0: e,
                    e_forced: e,
                    tau_e: Dy::ZERO,
                    cos_obliquity: Dy::ONE,
                    lock_gyr: Dy::ZERO,
                    spin0: Dy::ZERO,
                }
            }),
            _ => {
                let o = orbits.get(planet).map(Path::of);
                planet += 1;
                o
            }
        };
        paths.push(path);
    }
    sys.worlds
        .iter()
        .zip(paths)
        .map(|(w, path)| Setting {
            world: w,
            star_mass,
            lum,
            a_au: Dy::from_q(&w.a),
            z,
            r_kpc,
            sources: bodies::heat_sources(w, elements, &kept),
            path,
        })
        .collect()
}

/// A world's events over its life to `age` (Gyr since its star's birth),
/// each solved rather than stepped: the host locking its spin (exact on the
/// path), and its dynamo stopping where core-mantle heat flow
/// falls below what the core's adiabat conducts (a crossing of q_cmb - q_ad,
/// read from the world as built at each age).
pub fn events(set: &Setting, age: Dy, ctx: &Ctx) -> Vec<(&'static str, Encl)> {
    let mut out = Vec::new();
    if let Some(p) = set.path {
        if p.spin0.signum() > 0 && p.lock_gyr > Dy::ZERO && p.lock_gyr <= age {
            out.push(("its host locks its spin", Encl::exact(p.lock_gyr.to_q())));
        }
    }
    let start = crate::life::first_age(set.r_kpc, ctx);
    if !set.world.giant && age > start {
        let margin = |a: &Q| {
            let st = build(set, Dy::from_q(a), ctx);
            Encl::exact((st.q_cmb - st.q_ad).to_q())
        };
        if let Some(t) = xyl_core::event::crossing(
            &start.to_q(),
            &age.to_q(),
            &Q::new(1, 1000).unwrap_or(Q::ONE),
            margin,
        ) {
            out.push(("its dynamo stops", t));
        }
    }
    out
}
