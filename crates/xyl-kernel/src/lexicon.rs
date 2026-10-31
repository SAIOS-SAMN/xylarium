//! The lexicon of the seat surface: the typed records a
//! [`Percept`] is sent as, and the ids and keys that stand in for names on
//! the wire.
//!
//! Input: a [`Percept`] (percept.rs) and the universe's 32-byte domain key.
//!
//! Outputs: [`records`] returns one [`Record`] per item, its kind one of the
//! constants [`PEOPLE`] ..= [`NEIGHBOUR`] and its [`Value`]s in the order
//! documented on that constant; [`solvents`] is the id table for solvent
//! names (`xyl_cosmos::worlds` species order, then melt and plasma); and
//! [`people_key`] hashes a people's name under the domain (BLAKE3 derived
//! key), so neighbours appear only as keys and no people's name is sent.
//!
//! Invariants: every quantity is an exact rational (`Value::Q`, converted
//! from `Dy` without loss); units are fixed per field: deep time in Gyr,
//! day and year in seconds, temperatures in K, pressure in bar, power in W,
//! distance in kpc, light delay in years. A solvent missing from the table
//! is sent with id `u64::MAX`.

use crate::percept::Percept;
use xyl_core::dyadic::Dy;
use xyl_core::Q;
use xyl_cosmos::worlds::{Solvent, Species};

/// A typed exact value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    /// An exact rational quantity, in the units the record kind states.
    Q(Q),
    /// A non-negative count or id (a solvent id, a bell index).
    Count(u64),
    /// A yes/no field (ancient, tidally locked).
    Flag(bool),
    /// A 32-byte key, such as a people key from [`people_key`].
    Key([u8; 32]),
}

/// One record: its kind and its values, in the order the kind states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    /// Record kind id: one of the kind constants in this module ([`PEOPLE`] ..= [`NEIGHBOUR`]).
    pub kind: u16,
    /// The kind's values, in the order documented on its kind constant.
    pub values: Vec<Value>,
}

/// [ancient flag, dawn Gyr, now Gyr]
pub const PEOPLE: u16 = 1;
/// [numbers, capacity, growth per year]
pub const NUMBERS: u16 = 2;
/// [bodies W, starlight W, heat W]
pub const POWER: u16 = 3;
/// [surface K, pressure bar, light flux units]
pub const WORLD: u16 = 4;
/// [solvent id, temperature K, free energy flux units]
pub const MEDIUM: u16 = 5;
/// [day s, year s, locked flag]
pub const HEARTBEAT: u16 = 6;
/// [dominant bell index, dominance, coherence]
pub const HEARING: u16 = 7;
/// [people key, distance kpc, light delay yr]
pub const NEIGHBOUR: u16 = 8;

/// The solvents by id (worlds.rs `Solvent` names): every volatile species in
/// its table order (`Species::ALL`), then the melt and the plasma.
pub fn solvents() -> Vec<&'static str> {
    Species::ALL
        .iter()
        .map(|s| s.name())
        .chain([Solvent::Melt.name(), Solvent::Plasma.name()])
        .collect()
}

fn value_q(x: Dy) -> Value {
    Value::Q(x.to_q())
}

/// A people's key on the seat surface: its name under its universe's domain,
/// hashed (the house knows the mapping; a mind sees only keys and records).
pub fn people_key(domain: &[u8; 32], name: &str) -> [u8; 32] {
    let mut h = blake3::Hasher::new_derive_key("xylarium seat people v1");
    h.update(domain);
    h.update(name.as_bytes());
    *h.finalize().as_bytes()
}

/// A percept as the seat surface's records: one [`PEOPLE`], [`NUMBERS`] and
/// [`WORLD`] record always, then [`POWER`], [`HEARTBEAT`] and [`HEARING`] when
/// read, one [`MEDIUM`] per solvent seen and one [`NEIGHBOUR`] per people heard.
/// Neighbours appear only as keys hashed under `domain`; a solvent not in
/// [`solvents`] is sent with id `u64::MAX`.
pub fn records(domain: &[u8; 32], p: &Percept) -> Vec<Record> {
    let mut out = vec![
        Record {
            kind: PEOPLE,
            values: vec![Value::Flag(p.ancient), value_q(p.dawn), value_q(p.gyr)],
        },
        Record {
            kind: NUMBERS,
            values: vec![
                value_q(p.numbers),
                value_q(p.capacity),
                value_q(p.growth_per_yr),
            ],
        },
        Record {
            kind: WORLD,
            values: vec![
                value_q(p.surface_k),
                value_q(p.pressure_bar),
                value_q(p.light),
            ],
        },
    ];
    if let Some(pw) = p.power {
        out.push(Record {
            kind: POWER,
            values: vec![
                value_q(pw.bodies_w),
                value_q(pw.starlight_w),
                value_q(pw.heat_w),
            ],
        });
    }
    for m in &p.media {
        let id = solvents()
            .iter()
            .position(|s| *s == m.solvent)
            .map_or(u64::MAX, |i| i as u64);
        out.push(Record {
            kind: MEDIUM,
            values: vec![Value::Count(id), value_q(m.t_k), value_q(m.energy)],
        });
    }
    if let Some(hb) = p.heartbeat {
        out.push(Record {
            kind: HEARTBEAT,
            values: vec![
                value_q(hb.day_s),
                value_q(hb.year_s),
                Value::Flag(hb.locked),
            ],
        });
    }
    if let Some(h) = p.hearing {
        out.push(Record {
            kind: HEARING,
            values: vec![
                Value::Count(h.dominant as u64),
                value_q(h.dominance),
                value_q(h.coherence),
            ],
        });
    }
    for n in &p.neighbours {
        out.push(Record {
            kind: NEIGHBOUR,
            values: vec![
                Value::Key(people_key(domain, &n.people)),
                value_q(n.kpc),
                value_q(n.light_yr),
            ],
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::percept::{Heard, MediumSeen};
    use crate::seats::Role;

    #[test]
    fn percept_records_are_exact_typed_and_self_keyed() {
        let p = Percept {
            seat: 0,
            role: Role::Sovereign,
            people: "Kazi".into(),
            ancient: false,
            gyr: Dy::int(14),
            dawn: Dy::ratio(366, 100),
            traits: String::new(),
            numbers: Dy::sci(39, 11),
            capacity: Dy::sci(39, 11),
            growth_per_yr: Dy::ratio(1, 10),
            power: None,
            surface_k: Dy::int(288),
            pressure_bar: Dy::ONE,
            light: Dy::ONE,
            media: vec![MediumSeen {
                solvent: "H2O",
                t_k: Dy::int(288),
                energy: Dy::ratio(1, 2),
            }],
            heartbeat: None,
            hearing: None,
            neighbours: vec![Heard {
                people: "Thsha".into(),
                kpc: Dy::ratio(1, 4),
                light_yr: Dy::int(815),
            }],
            annals: vec![],
        };
        let d = [7u8; 32];
        let r = records(&d, &p);
        assert_eq!(
            r[0],
            Record {
                kind: PEOPLE,
                values: vec![
                    Value::Flag(false),
                    Value::Q(Dy::ratio(366, 100).to_q()),
                    Value::Q(Q::int(14))
                ]
            }
        );
        let medium = r.iter().find(|x| x.kind == MEDIUM).unwrap();
        assert_eq!(medium.values[0], Value::Count(0), "water is solvent 0");
        let nb = r.iter().find(|x| x.kind == NEIGHBOUR).unwrap();
        assert_eq!(
            nb.values[0],
            Value::Key(people_key(&d, "Thsha")),
            "a neighbour is a key, never a name"
        );
        assert_ne!(
            people_key(&d, "Thsha"),
            people_key(&[8u8; 32], "Thsha"),
            "keys are the domain's own"
        );
    }
}
