//! Percepts: the typed record a seat seated on a people receives of its
//! people and world at the universe's present. The uplink posts it to the
//! seat's holder (encoded by lexicon.rs); the kernel's default rule reads
//! the same record.
//!
//! Inputs: a [`Universe`] (peoples, their worlds via
//! `cosmic::Cosmic::world_at`, power, heartbeat, annals), and the bells'
//! nodes and relays at the present (`xyl_cosmos::pulse`).
//!
//! Output: [`perceive`] returns a [`Percept`]: the seat's role, the people's
//! traits, numbers, capacity, growth rate and power; its world's surface
//! temperature (K), pressure (bar), light and liquid media; its
//! [`Heartbeat`] (day, year, lock), which the caller uses to pace percepts;
//! the bells' [`Hearing`] at its star (`pulse::hear`); its neighbours; and
//! the latest eight annals naming it. Times are deep time in Gyr.
//!
//! Neighbours ([`Heard`]): every other people at its current separation
//! (`transit::separation_m`, in kpc) whose dawn lies at least the light
//! delay in the past; a word does not outrun its fastest physical carrier.
//!
//! Returns `None` for a seat that does not exist, is not seated on a people,
//! or whose people or world cannot be read now. Read-only.

use crate::growth::Power;
use crate::seats::{On, Role};
use crate::universe::Universe;
use xyl_core::dyadic::Dy;
use xyl_cosmos::pulse::{c_kpc_yr, hear, Hearing, Node, Relay};
use xyl_cosmos::worlds::Heartbeat;

/// A neighbour as heard: its name, how far (kpc), and how long its word took
/// to arrive (yr).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heard {
    /// The neighbouring people's name (sent on the wire only as a key).
    pub people: String,
    /// Current separation between the two peoples' worlds, in kpc.
    pub kpc: Dy,
    /// Light delay across that separation, in years.
    pub light_yr: Dy,
}

/// A medium of the seat's world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediumSeen {
    /// Solvent name, one of [`crate::lexicon::solvents`].
    pub solvent: &'static str,
    /// Temperature of the medium, in K.
    pub t_k: Dy,
    /// Free energy reaching the medium, in flux units.
    pub energy: Dy,
}

/// The record a seat perceives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Percept {
    /// Index of the perceiving seat in [`crate::seats::Seats::list`].
    pub seat: usize,
    /// The seat's tier and role.
    pub role: Role,
    /// Name of the people the seat sits on.
    pub people: String,
    /// Whether that people is an ancient people.
    pub ancient: bool,
    /// Deep time now (Gyr) and the people's dawn.
    pub gyr: Dy,
    /// When the people woke, in Gyr after the Big Bang.
    pub dawn: Dy,
    /// The people's traits, as their human-readable description.
    pub traits: String,
    /// The people's population (count of bodies).
    pub numbers: Dy,
    /// How many of its bodies the people's world carries (count).
    pub capacity: Dy,
    /// The people's growth rate, per year.
    pub growth_per_yr: Dy,
    /// The people's power budget in W; `None` when it is not read.
    pub power: Option<Power>,
    /// Its world: surface temperature (K), pressure (bar), light (flux units).
    pub surface_k: Dy,
    /// Surface pressure of the people's world, in bar.
    pub pressure_bar: Dy,
    /// The year's mean light reaching the people's world, in flux units.
    pub light: Dy,
    /// Every liquid medium of the people's world.
    pub media: Vec<MediumSeen>,
    /// The world's day and year (and lock), which pace percepts; `None` when unread.
    pub heartbeat: Option<Heartbeat>,
    /// The bells as heard at the people's star; `None` when the star has no place or nothing is heard.
    pub hearing: Option<Hearing>,
    /// Every other people whose dawn's word has had time to arrive.
    pub neighbours: Vec<Heard>,
    /// The annals that name its people, latest first.
    pub annals: Vec<String>,
}

/// What the seat `seat` perceives now. `nodes` and `relays` are the bells at
/// the universe's present (pulse.rs). Neighbours are read at current
/// positions and kept only when their dawn lies at least the light delay in
/// the past; annals are the latest eight naming the people.
///
/// Returns `None` when `seat` does not exist, is not seated on a people, or
/// the people or its world cannot be read at the present.
pub fn perceive(
    u: &Universe<'_>,
    seat: usize,
    nodes: &[Node],
    relays: &[Relay],
) -> Option<Percept> {
    let s = u.seats.list.get(seat)?;
    let On::People(i) = s.on else { return None };
    let p = u.cosmic.peoples.get(i)?;
    let gyr = u.gyr();
    let (state, media, _) = u.cosmic.world_at(p.star, p.world, gyr)?;
    let power = u.cosmic.power(p);
    let here = u.cosmic.place(p.star);
    let hearing = here.and_then(|x| hear(&x, nodes, relays, gyr));
    let c = c_kpc_yr();
    let neighbours = u
        .cosmic
        .peoples
        .iter()
        .enumerate()
        .filter(|(j, _)| *j != i)
        .filter_map(|(_, q)| {
            // current positions: a neighbour in the same system is separated
            // by where both orbits have carried them, never at zero distance
            let at = |x: &crate::cosmic::People| crate::transit::Place {
                star: x.star,
                world: x.world,
            };
            let kpc =
                crate::transit::separation_m(&u.cosmic, at(p), at(q))? / crate::transit::kpc_m();
            let light_yr = kpc / c;
            // heard once its dawn's word could have arrived
            ((gyr - q.dawn) * Dy::sci(1, 9) >= light_yr).then(|| Heard {
                people: q.name.clone(),
                kpc,
                light_yr,
            })
        })
        .collect();
    Some(Percept {
        seat,
        role: s.role,
        people: p.name.clone(),
        ancient: p.ancient,
        gyr,
        dawn: p.dawn,
        traits: p.traits.describe(),
        numbers: p.population,
        capacity: p.capacity,
        growth_per_yr: p.rate,
        power,
        surface_k: state.t_surf,
        pressure_bar: state.p_bar,
        light: state.flux,
        media: media
            .iter()
            .map(|m| MediumSeen {
                solvent: m.solvent.name(),
                t_k: m.t,
                energy: m.energy,
            })
            .collect(),
        heartbeat: u.cosmic.heartbeat(p.star, p.world, gyr),
        hearing,
        neighbours,
        annals: u
            .cosmic
            .annals
            .iter()
            .rev()
            .filter(|a| a.text.contains(&p.name))
            .take(8)
            .map(|a| format!("{} {} {}", a.epoch, a.kind, a.text))
            .collect(),
    })
}
