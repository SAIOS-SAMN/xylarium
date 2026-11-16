//! Seats: the roles a player, human or AI, can hold in a universe
//! (docs/civilizations.md). Every holder acts through the same inputs and
//! is bound by the same laws.
//!
//! Roles ([`Role`]), one tiered hierarchy in which each tier deals with the
//! tiers adjacent to it:
//!
//!   Ancient      an ancient people's house (cosmic/): holds the reserve
//!                and is the only seat facing the network.
//!   Progenitor   one per ancient house: builds, keeps and runs passage.
//!   Steward      one per realm (transit/realms.rs): diplomacy, trade and
//!                disputes between the realm's sovereigns, and the handling
//!                of the realm's ley lines.
//!   Sovereign    one people's worlds.
//!
//! Inputs: the peoples of a [`Cosmic`] world and the realm keys of the ley
//! network. Outputs: [`Seats`], an append-only list; [`Seats::extend`] adds
//! seats for peoples woken since, [`Seats::extend_realms`] adds stewards'
//! seats, and [`Seats::take`] assigns a [`Holder`], returning `Err` when the
//! seat does not exist or another holder has it.
//!
//! A seat is addressed by its [`SeatKey`], derived from its subject when the
//! seat is raised (#264): a people's seat from its role and the people's
//! origin (its star, the world it woke on, its dawn), a steward's seat from
//! its realm's key (the realm's least constellation key). A seat's index in
//! the list is only its position: a restore that carries the universe to
//! its count in one leap can raise seats in another order, and a people's
//! name can change, but the key read from the seat's subject does not.
//!
//! An untaken seat is held by `Holder::Universe`,
//! the kernel's default rule for that role. Network minds (`Holder::Mind`)
//! hold ancient seats only; the universe releases any other seat they hold.
//! Holders other than the default are held in the checkpoint.
//!
//! Terms: seat, council, house, progenitor, steward, sovereign
//! (docs/glossary.md).

use crate::cosmic::{Cosmic, People};

/// A seat's address, derived from its subject when the seat is raised and
/// held on the seat (#264): written and read as 64 hex digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SeatKey(pub [u8; 32]);

impl SeatKey {
    /// A people's seat of `role`: the role and the people's origin, its star,
    /// the world it woke on and its dawn, none of which a later move or a
    /// new name changes.
    pub fn of_people(role: Role, p: &People) -> SeatKey {
        let mut h = blake3::Hasher::new_derive_key("xylarium seat v1");
        let (m, e) = p.dawn.to_bits();
        h.update(&[role as u8, 0])
            .update(&p.star.to_le_bytes())
            .update(&(p.world as u64).to_le_bytes())
            .update(&m.to_le_bytes())
            .update(&e.to_le_bytes());
        SeatKey(*h.finalize().as_bytes())
    }

    /// A realm's steward seat: the realm's key, its least constellation key
    /// as the realm first formed.
    pub fn of_realm(realm: u64) -> SeatKey {
        let mut h = blake3::Hasher::new_derive_key("xylarium seat v1");
        h.update(&[Role::Steward as u8, 1])
            .update(&realm.to_le_bytes());
        SeatKey(*h.finalize().as_bytes())
    }
}

impl std::fmt::Display for SeatKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().try_for_each(|b| write!(f, "{b:02x}"))
    }
}

impl std::str::FromStr for SeatKey {
    type Err = &'static str;
    /// Reads 64 hex digits; anything else is refused.
    fn from_str(s: &str) -> Result<SeatKey, &'static str> {
        if s.len() != 64 {
            return Err("a seat key is 64 hex digits");
        }
        let mut out = [0u8; 32];
        for (i, b) in out.iter_mut().enumerate() {
            *b = s
                .get(2 * i..2 * i + 2)
                .and_then(|x| u8::from_str_radix(x, 16).ok())
                .ok_or("a seat key is 64 hex digits")?;
        }
        Ok(SeatKey(out))
    }
}

/// A seat's tier and role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    /// An ancient people's house: holds the reserve and faces the network.
    Ancient,
    /// Assigned by the ancients: builds, keeps and runs passage.
    Progenitor,
    /// A realm's steward: diplomacy, trade and disputes between sovereigns.
    Steward,
    /// One people's worlds.
    Sovereign,
}

impl Role {
    /// The role's display name.
    pub fn name(self) -> &'static str {
        match self {
            Role::Ancient => "Ancient",
            Role::Progenitor => "Progenitor",
            Role::Steward => "Steward",
            Role::Sovereign => "Sovereign",
        }
    }
}

/// What a seat is seated on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum On {
    /// A people, by its index in the cosmic world.
    People(usize),
    /// A realm, by its index (realms emerge from passage networks).
    Realm(usize),
}

/// Who holds a seat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Holder {
    /// The kernel's default rule for the seat's role, which holds the seat
    /// until a player or a mind takes it.
    Universe,
    /// A player, by name.
    Player(String),
    /// A network mind, by its key, seated on an ancient house's council.
    Mind([u8; 32]),
}

impl std::fmt::Display for Holder {
    /// `universe`, `player:<name>` or `mind:<64 hex digits>`, one token.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Holder::Universe => write!(f, "universe"),
            Holder::Player(n) => write!(f, "player:{n}"),
            Holder::Mind(k) => write!(f, "mind:{}", SeatKey(*k)),
        }
    }
}

impl std::str::FromStr for Holder {
    type Err = &'static str;
    /// Reads what [`Holder`]'s `Display` writes; anything else is refused.
    fn from_str(s: &str) -> Result<Holder, &'static str> {
        match s.split_once(':') {
            None if s == "universe" => Ok(Holder::Universe),
            Some(("player", n)) if !n.is_empty() => Ok(Holder::Player(n.to_string())),
            Some(("mind", k)) => Ok(Holder::Mind(k.parse::<SeatKey>()?.0)),
            _ => Err("a holder is universe, player:<name> or mind:<key>"),
        }
    }
}

/// One seat: its key, its role, what it is seated on, and who holds it now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seat {
    /// The seat's address, derived from its subject when it was raised.
    pub key: SeatKey,
    /// The seat's tier and role.
    pub role: Role,
    /// The people or realm the seat is seated on.
    pub on: On,
    /// Who holds the seat: `Holder::Universe` until the seat is taken.
    pub holder: Holder,
}

/// Every seat of the universe.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Seats {
    /// Every seat, append-only; each addressed by its key.
    pub list: Vec<Seat>,
}

impl Seats {
    /// Returns the seats of the cosmic world's peoples: for every ancient
    /// people an ancient seat and a progenitor seat, for every other people a
    /// sovereign seat. Steward seats are added by [`Seats::extend_realms`] as
    /// realms emerge.
    pub fn of(cosmic: &Cosmic<'_>) -> Seats {
        Seats::default().extend(cosmic)
    }

    /// Returns a copy with seats added for peoples woken since: a sovereign
    /// seat per people or, for an ancient people, an ancient (council) seat,
    /// where network minds sit, and a progenitor seat. An ancient house holds
    /// only its home world; settlement by houses is not modelled. New seats
    /// are appended, so a seat's index never changes.
    pub fn extend(&self, cosmic: &Cosmic<'_>) -> Seats {
        let mut out = self.clone();
        for (i, p) in cosmic.peoples.iter().enumerate() {
            let role = if p.ancient {
                Role::Ancient
            } else {
                Role::Sovereign
            };
            if !out
                .list
                .iter()
                .any(|s| s.on == On::People(i) && s.role == role)
            {
                out.list.push(Seat {
                    key: SeatKey::of_people(role, p),
                    role,
                    on: On::People(i),
                    holder: Holder::Universe,
                });
            }
            // An ancient people also has a progenitor seat; progenitors build
            // passage to the peoples in tune (transit/building.rs).
            if p.ancient
                && !out
                    .list
                    .iter()
                    .any(|s| s.on == On::People(i) && s.role == Role::Progenitor)
            {
                out.list.push(Seat {
                    key: SeatKey::of_people(Role::Progenitor, p),
                    role: Role::Progenitor,
                    on: On::People(i),
                    holder: Holder::Universe,
                });
            }
        }
        out
    }

    /// Returns a copy with a steward seat added for every realm of
    /// `realm_keys` (transit/ `Network::realm_keys`, in the order first seen)
    /// not yet seated; a realm's seat is `On::Realm(index)` into that list,
    /// keyed by the realm's key.
    pub fn extend_realms(&self, realm_keys: &[u64]) -> Seats {
        let mut out = self.clone();
        for (r, &realm) in realm_keys.iter().enumerate() {
            let key = SeatKey::of_realm(realm);
            if !out.list.iter().any(|s| s.key == key) {
                out.list.push(Seat {
                    key,
                    role: Role::Steward,
                    on: On::Realm(r),
                    holder: Holder::Universe,
                });
            }
        }
        out
    }

    /// Every ancient seat, with its index: the council seats, the only seats
    /// network minds hold.
    pub fn council(&self) -> impl Iterator<Item = (usize, &Seat)> {
        self.list
            .iter()
            .enumerate()
            .filter(|(_, s)| s.role == Role::Ancient)
    }

    /// Every steward seat, with its index.
    pub fn stewards(&self) -> impl Iterator<Item = (usize, &Seat)> {
        self.list
            .iter()
            .enumerate()
            .filter(|(_, s)| s.role == Role::Steward)
    }

    /// The seat addressed by `key`, with its position in the list.
    pub fn get(&self, key: &SeatKey) -> Option<(usize, &Seat)> {
        self.list.iter().enumerate().find(|(_, s)| s.key == *key)
    }

    /// The seat addressed by `key` if `by` holds it (#265): a decision is the
    /// seat holder's alone, and the built-in rule's only for a seat it holds.
    /// `Err` names the refusal: no such seat, or a caller who does not hold
    /// it.
    pub fn held_by(&self, key: &SeatKey, by: &Holder) -> Result<&Seat, &'static str> {
        let (_, s) = self.get(key).ok_or("no such seat")?;
        if s.holder != *by {
            return Err("the caller does not hold the seat");
        }
        Ok(s)
    }

    /// Assigns `holder` to the seat addressed by `seat`, held by
    /// `Holder::Universe` (or already by `holder`), and returns the seat.
    ///
    /// Returns `Err` with a message when no seat has that key or another
    /// holder already holds it.
    pub fn take(&mut self, seat: &SeatKey, holder: Holder) -> Result<&Seat, String> {
        let s = self
            .list
            .iter_mut()
            .find(|s| s.key == *seat)
            .ok_or_else(|| format!("no seat {seat}"))?;
        if s.holder != Holder::Universe && s.holder != holder {
            return Err(format!("seat {seat} is held"));
        }
        s.holder = holder;
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A seat's key is read from its subject, so seats raised in another
    /// order are addressed by the same keys and a holder taken by key lands
    /// on the same subject (#264); a key is written and read back exactly,
    /// and anything else is refused.
    #[test]
    fn seats_are_addressed_by_their_subject_not_their_position() {
        let one = Seats::default().extend_realms(&[11, 7, 23]);
        let other = Seats::default().extend_realms(&[23, 11, 7]);
        let keys = |s: &Seats| {
            let mut k: Vec<SeatKey> = s.list.iter().map(|x| x.key).collect();
            k.sort();
            k
        };
        assert_eq!(keys(&one), keys(&other));
        assert_ne!(one.list[0].key, other.list[0].key, "positions differ");
        let realm = SeatKey::of_realm(7);
        let (mut a, mut b) = (one.clone(), other.clone());
        a.take(&realm, Holder::Player("p".into())).unwrap();
        b.take(&realm, Holder::Player("p".into())).unwrap();
        let held = |s: &Seats| {
            let (i, seat) = s.get(&realm).unwrap();
            (seat.holder.clone(), i)
        };
        assert_eq!(held(&a).0, held(&b).0);
        assert_ne!(held(&a).1, held(&b).1, "the same seat at another position");
        // Raising again adds nothing; another holder is refused; an unknown
        // key names no seat.
        assert_eq!(a.extend_realms(&[7, 11, 23]).list.len(), 3);
        assert!(a.take(&realm, Holder::Player("q".into())).is_err());
        assert!(a.take(&SeatKey([0; 32]), Holder::Universe).is_err());
        let text = realm.to_string();
        assert_eq!(text.parse::<SeatKey>(), Ok(realm));
        assert!("12ab".parse::<SeatKey>().is_err());
        assert!(text.replace('0', "g").parse::<SeatKey>().is_err() || !text.contains('0'));
    }

    /// A decision is the seat holder's alone (#265): the holder is admitted,
    /// any other caller and the built-in rule of a taken seat are refused,
    /// and an unknown seat is refused by name; a holder is written as one
    /// token and read back exactly.
    #[test]
    fn only_the_holder_decides_for_a_seat() {
        let mut seats = Seats::default().extend_realms(&[5]);
        let key = SeatKey::of_realm(5);
        assert!(seats.held_by(&key, &Holder::Universe).is_ok());
        let mind = Holder::Mind([4; 32]);
        seats.take(&key, mind.clone()).unwrap();
        assert!(seats.held_by(&key, &mind).is_ok());
        for other in [
            Holder::Universe,
            Holder::Mind([5; 32]),
            Holder::Player("p".into()),
        ] {
            assert_eq!(
                seats.held_by(&key, &other).err(),
                Some("the caller does not hold the seat")
            );
        }
        assert_eq!(
            seats.held_by(&SeatKey([1; 32]), &mind).err(),
            Some("no such seat")
        );
        for h in [Holder::Universe, Holder::Player("ada".into()), mind] {
            assert_eq!(h.to_string().parse::<Holder>(), Ok(h.clone()));
        }
        assert!("player:".parse::<Holder>().is_err());
        assert!("someone".parse::<Holder>().is_err());
    }
}
