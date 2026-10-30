//! Seats (docs/civilizations.md): the places a player,
//! human or AI, can take in the universe. All seats are bound by the same
//! laws. One tiered hierarchy for every universe; each tier deals with the
//! tiers adjacent to it:
//!
//!   Ancient      an ancient people near the galaxy's core (cosmic.rs): its
//!                house holds the reserve, is the only seat facing the network,
//!                folds space at the core and designs the inner wormholes. It
//!                acts only through the progenitors it assigns.
//!   Progenitor   assigned by the ancients: builds, keeps and runs passage,
//!                takes its tolls, hears the realms. No traversal without one.
//!   Steward      a realm (the shell a passage network reached): diplomacy,
//!                trade and disputes between its sovereigns; settles their
//!                currencies; reports up.
//!   Sovereign    one people's worlds.
//!
//! A seat acts through the same interfaces as the universe's own minds. An
//! untaken seat is held by the universe's own mind (its built-in rule); a
//! taken seat is held by a player or by a mind joined through the ancients'
//! house.

use crate::cosmic::Cosmic;

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
    /// The universe's own mind: its built-in rule (the role's
    /// own rule), until someone takes the seat.
    Universe,
    /// A player, by name.
    Player(String),
    /// A mind joined through its universe's ancients' house, by its key.
    Mind([u8; 32]),
}

/// One seat: its role, what it is seated on, and who holds it now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seat {
    /// The seat's tier and role.
    pub role: Role,
    /// The people or realm the seat is seated on.
    pub on: On,
    /// Who holds the seat: the universe's own mind until it is taken.
    pub holder: Holder,
}

/// Every seat of the universe.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Seats {
    /// Every seat, append-only; a seat's index is its stable id.
    pub list: Vec<Seat>,
}

impl Seats {
    /// The seats the cosmic world raises: an ancient seat for every ancient
    /// people, a sovereign seat for every other people. Stewards' seats open
    /// as realms emerge.
    pub fn of(cosmic: &Cosmic<'_>) -> Seats {
        Seats::default().extend(cosmic)
    }

    /// The same, adding seats for peoples woken since: one seat per people,
    /// sovereign or, for an ancient house, its council's (where the network's
    /// minds sit; a house holds only its home, settling not being modelled).
    /// New seats are appended, so a seat's index never changes.
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
                    role,
                    on: On::People(i),
                    holder: Holder::Universe,
                });
            }
            // an ancient house's progenitor seat: the builders of passage
            // (transit.rs: they open ley lines to the peoples in tune)
            if p.ancient
                && !out
                    .list
                    .iter()
                    .any(|s| s.on == On::People(i) && s.role == Role::Progenitor)
            {
                out.list.push(Seat {
                    role: Role::Progenitor,
                    on: On::People(i),
                    holder: Holder::Universe,
                });
            }
        }
        out
    }

    /// The seats with a steward seat for every realm of the first `realms`
    /// realm keys (transit.rs `Network::realm_keys`) not yet seated; a
    /// realm's seat is `On::Realm(index)`.
    pub fn extend_realms(&self, realms: usize) -> Seats {
        let mut out = self.clone();
        for r in 0..realms {
            if !out
                .list
                .iter()
                .any(|s| s.on == On::Realm(r) && s.role == Role::Steward)
            {
                out.list.push(Seat {
                    role: Role::Steward,
                    on: On::Realm(r),
                    holder: Holder::Universe,
                });
            }
        }
        out
    }

    /// The seats of an ancient house's council (the network's minds sit here
    /// and nowhere else).
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

    /// Take a seat the universe's own mind holds (or re-take one `holder`
    /// already holds), returning the seat.
    ///
    /// Returns `Err` with a message when `seat` does not exist or another
    /// holder already holds it.
    pub fn take(&mut self, seat: usize, holder: Holder) -> Result<&Seat, String> {
        let s = self
            .list
            .get_mut(seat)
            .ok_or_else(|| format!("no seat {seat}"))?;
        if s.holder != Holder::Universe && s.holder != holder {
            return Err(format!("seat {seat} is held"));
        }
        s.holder = holder;
        Ok(s)
    }
}
