//! The birth gas a system forms from, and the apportioning of a body's
//! matter among the elements.

use super::*;

/// An ice: its species, condensation temperature (K), and (element, atoms).
pub(super) type Ice = (&'static str, i64, &'static [(&'static str, i64)]);

/// The share of each siderophile and chalcophile element left in the
/// silicate mantle by core formation: mass in the reference world's bulk
/// silicate over mass in the whole body (McDonough 2003, Treatise on
/// Geochemistry 2.15: BSE ppm x 0.675 / bulk ppm). Declared ruler, measured
/// on the reference world and applied to all worlds; values as transcribed,
/// to be checked against the source table. An element not listed keeps none
/// in its mantle.
pub(super) const MANTLE_KEPT: [(&str, &str); 30] = [
    ("Fe", "0.1325"),
    ("Ni", "0.0726"),
    ("Co", "0.0805"),
    ("S", "0.0266"),
    ("Cu", "0.3375"),
    ("Zn", "0.928"),
    ("Ga", "0.9"),
    ("Ge", "0.106"),
    ("As", "0.0199"),
    ("Se", "0.0188"),
    ("Mo", "0.0199"),
    ("Ru", "0.0026"),
    ("Rh", "0.0025"),
    ("Pd", "0.0026"),
    ("Ag", "0.108"),
    ("Cd", "0.3375"),
    ("In", "1"),
    ("Sn", "0.351"),
    ("Sb", "0.0743"),
    ("Te", "0.027"),
    ("W", "0.115"),
    ("Re", "0.0025"),
    ("Os", "0.00255"),
    ("Ir", "0.00257"),
    ("Pt", "0.00252"),
    ("Au", "0.0042"),
    ("Hg", "0.3375"),
    ("Tl", "0.197"),
    ("Pb", "0.44"),
    ("Bi", "0.169"),
];

/// The share of a siderophile or chalcophile element its mantle keeps.
pub fn mantle_kept(sym: &str) -> Q {
    MANTLE_KEPT
        .iter()
        .find(|(s, _)| *s == sym)
        .map(|(_, v)| decimal(v).unwrap_or(Q::ZERO))
        .unwrap_or(Q::ZERO)
}

/// The ices a cooling disc condenses.
pub(super) const ICES: [Ice; 6] = [
    ("H2O", 182, &[("H", 2), ("O", 1)]),
    ("NH3", 131, &[("N", 1), ("H", 3)]),
    ("CO2", 72, &[("C", 1), ("O", 2)]),
    ("CH4", 41, &[("C", 1), ("H", 4)]),
    ("N2", 22, &[("N", 2)]),
    ("CO", 25, &[("C", 1), ("O", 1)]),
];

/// The share of a cooling nebula's carbon held as species `sp`.
pub(super) fn carbon_as(sp: &str) -> Q {
    match sp {
        "CO" => Q::ratio(1, 2),
        "CO2" => Q::ratio(1, 5),
        "CH4" => Q::ratio(3, 10),
        _ => Q::ZERO,
    }
}

/// The share of a cooling nebula's nitrogen held as species `sp`.
pub(super) fn nitrogen_as(sp: &str) -> Q {
    match sp {
        "NH3" | "N2" => Q::ratio(1, 2),
        _ => Q::ZERO,
    }
}

/// The local gas: each element's atoms per 10^6 Si, here and then.
pub struct Gas<'a> {
    /// The element table the counts are indexed by.
    pub elements: &'a [Element],
    /// Each element's atoms per 10^6 Si atoms, on a 2^-32 grid, indexed as
    /// `elements`.
    pub atoms: Vec<Q>,
    /// Primordial hydrogen mass fraction, a share in \[0, 1\].
    pub x: Q,
    /// Primordial helium-4 mass fraction, a share in \[0, 1\].
    pub y: Q,
}

impl<'a> Gas<'a> {
    /// The gas a star at `r`, born at `born`, formed from (its merger lump `lump`).
    ///
    /// `weights` is its galactic core's recipe (cores.rs): each metal's share
    /// tilted by its dial, the metals' mass kept.
    pub fn at(
        chem: &Chemistry,
        elements: &'a [Element],
        r: &Q,
        born: &Q,
        lump: &Q,
        weights: &crate::cores::Weights,
    ) -> Gas<'a> {
        // the composition is fixed on a grid (2^-32 of an atom per 10^6 Si) so
        // every world's shares divide by small numbers
        let grid = Q::int(1 << 32);
        // counts as atoms per 10^6 Si, whatever unit the founding uses (only
        // ratios matter), floored exactly
        let si = elements
            .iter()
            .find(|e| e.sym == "Si")
            .map(|e| e.abundance.clone())
            .filter(|x| x.signum() > 0);
        let per_si = si
            .and_then(|x| Q::int(1_000_000).checked_div(&x))
            .unwrap_or(Q::ONE);
        let rel = chem.relatives(r, born, lump).unwrap_or_default();
        let atoms = elements
            .iter()
            .enumerate()
            .map(|(i, e)| {
                if e.abundance.is_zero() {
                    return Q::ZERO;
                }
                let v = &(&(&(&e.abundance * &per_si) * rel.get(i).unwrap_or(&Q::ZERO))
                    * &weights.factor(&e.sym))
                    * &grid;
                (&v - &v.frac()).checked_div(&grid).unwrap_or(Q::ZERO)
            })
            .collect();
        Gas {
            elements,
            atoms,
            x: chem.light.x.clone(),
            y: chem.light.y.clone(),
        }
    }

    fn idx(&self, sym: &str) -> Option<usize> {
        self.elements.iter().position(|e| e.sym == sym)
    }

    fn atoms_of(&self, sym: &str) -> Q {
        self.idx(sym)
            .and_then(|i| self.atoms.get(i).cloned())
            .unwrap_or(Q::ZERO)
    }

    /// Per unit of rock: the core's share, and each ice (by mass) the disc holds
    /// below `t_form`, from this gas's own abundances (worlds._solids).
    pub fn solids(&self, t_form: &Q) -> (Q, Vec<(&'static str, Q)>) {
        // the metals: what core formation sank, and what the mantle kept (iron
        // as FeO, carrying its oxygen)
        let metal = |sym: &str, a: i64| &Q::int(a) * &self.atoms_of(sym);
        let (fe, ni, s) = (metal("Fe", 56), metal("Ni", 59), metal("S", 32));
        let (kfe, kni, ks) = (mantle_kept("Fe"), mantle_kept("Ni"), mantle_kept("S"));
        let sunk = |m: &Q, k: &Q| m * &(&Q::ONE - k);
        let core = &(&sunk(&fe, &kfe) + &sunk(&ni, &kni)) + &sunk(&s, &ks);
        let kept = &(&(&fe * &kfe) + &(&ni * &kni)) + &(&s * &ks);
        let feo_oxygen = &(&Q::int(16) * &self.atoms_of("Fe")) * &kfe;
        let mantle = &(&(&(&Q::int(24) * &self.atoms_of("Mg"))
            + &(&Q::int(76) * &self.atoms_of("Si")))
            + &kept)
            + &feo_oxygen;
        let rock = &core + &mantle;
        if rock.is_zero() {
            return (Q::ZERO, Vec::new());
        }
        let c = self.atoms_of("C");
        let o_left = (&(&self.atoms_of("O") - &(&Q::int(3) * &self.atoms_of("Si")))
            - &(&c * &(&carbon_as("CO") + &(&Q::int(2) * &carbon_as("CO2")))))
            .max(Q::ZERO);
        let mass_of = |sp: &str| -> Q {
            match sp {
                "H2O" => &Q::int(18) * &o_left,
                "CO2" => &(&Q::int(44) * &c) * &carbon_as("CO2"),
                "CH4" => &(&Q::int(16) * &c) * &carbon_as("CH4"),
                "CO" => &(&Q::int(28) * &c) * &carbon_as("CO"),
                "NH3" => &(&Q::int(17) * &self.atoms_of("N")) * &nitrogen_as("NH3"),
                "N2" => (&(&Q::int(28) * &self.atoms_of("N")) * &nitrogen_as("N2"))
                    .checked_div(&Q::int(2))
                    .unwrap_or(Q::ZERO),
                _ => Q::ZERO,
            }
        };
        let ices = ICES
            .iter()
            .filter(|(_, cond, _)| t_form < &Q::int(*cond))
            .map(|(sp, _, _)| (*sp, mass_of(sp).checked_div(&rock).unwrap_or(Q::ZERO)))
            .collect();
        (core.checked_div(&rock).unwrap_or(Q::ZERO), ices)
    }

    /// A body's element masses (world masses), exact, from its layers:
    /// `core`, `mantle` and `envelope` masses, and each ice's mass, condensed
    /// where the disc was at `t_form`.
    pub fn apportion(
        &self,
        t_form: &Q,
        core: &Q,
        mantle: &Q,
        ices: &[(&str, Q)],
        envelope: &Q,
    ) -> Vec<Q> {
        self.apportion_layers(t_form, core, mantle, ices, envelope)
            .0
    }

    /// As `apportion`, with the mantle's own share of each element beside the
    /// whole (what the crust is made from).
    pub fn apportion_layers(
        &self,
        t_form: &Q,
        core: &Q,
        mantle: &Q,
        ices: &[(&str, Q)],
        envelope: &Q,
    ) -> (Vec<Q>, Vec<Q>) {
        let n = self.elements.len();
        let mut out = vec![Q::ZERO; n];
        let mut in_mantle = vec![Q::ZERO; n];
        let condensed = |e: &Element| e.tc.as_ref().is_some_and(|tc| tc > t_form);
        // a layer's mass shared among its elements by weight, exactly
        let shares = |mass: &Q, weights: Vec<(usize, Q)>| -> Vec<(usize, Q)> {
            let total = weights.iter().fold(Q::ZERO, |a, (_, w)| &a + w);
            if total.is_zero() || mass.is_zero() {
                return Vec::new();
            }
            weights
                .into_iter()
                .map(|(i, w)| (i, (mass * &w).checked_div(&total).unwrap_or(Q::ZERO)))
                .collect()
        };
        let w = |e: &Element, i: usize| &e.a * &self.atoms[i];
        let metal =
            |e: &Element| matches!(e.class, Some(Class::Sidero | Class::Chalco)) && condensed(e);
        // the metal core: what core formation sank of the siderophile and
        // chalcophile elements that condensed
        for (i, m) in shares(
            core,
            self.elements
                .iter()
                .enumerate()
                .filter(|(_, e)| metal(e))
                .map(|(i, e)| (i, &w(e, i) * &(&Q::ONE - &mantle_kept(&e.sym))))
                .collect(),
        ) {
            out[i] = &out[i] + &m;
        }
        // the silicate mantle: lithophiles that condensed, the metals it kept,
        // oxygen as three per silicon and one per iron it kept (FeO)
        let oi = self.idx("O");
        let o_weight = |e: &Element| {
            &(&(&e.a * &Q::int(3)) * &self.atoms_of("Si"))
                + &(&(&e.a * &self.atoms_of("Fe")) * &mantle_kept("Fe"))
        };
        for (i, m) in shares(
            mantle,
            self.elements
                .iter()
                .enumerate()
                .filter_map(|(i, e)| {
                    if Some(i) == oi {
                        Some((i, o_weight(e)))
                    } else if e.class == Some(Class::Litho) && condensed(e) {
                        Some((i, w(e, i)))
                    } else if metal(e) {
                        Some((i, &w(e, i) * &mantle_kept(&e.sym)))
                    } else {
                        None
                    }
                })
                .filter(|(_, wt)| !wt.is_zero())
                .collect(),
        ) {
            out[i] = &out[i] + &m;
            in_mantle[i] = &in_mantle[i] + &m;
        }
        // the ices, by their own formulas
        for (sp, mass) in ices {
            let Some((_, _, parts)) = ICES.iter().find(|(s, _, _)| s == sp) else {
                continue;
            };
            for (i, m) in shares(
                mass,
                parts
                    .iter()
                    .filter_map(|(sym, k)| {
                        self.idx(sym)
                            .map(|i| (i, &self.elements[i].a * &Q::int(*k)))
                    })
                    .collect(),
            ) {
                out[i] = &out[i] + &m;
            }
        }
        // an envelope: the Big Bang's hydrogen and helium
        if let (Some(h), Some(he)) = (self.idx("H"), self.idx("He")) {
            out[h] = &out[h] + &(envelope * &self.x);
            out[he] = &out[he] + &(envelope * &self.y);
        }
        (out, in_mantle)
    }
}
