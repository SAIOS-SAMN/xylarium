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
//! the latest eight annals naming the people. Times are deep time in Gyr.
//!
//! Neighbours ([`Heard`]): every other people, at the current separation of
//! the two worlds (`transit::separation_m`, converted to kpc), whose dawn
//! lies at least the light delay in the past, so that no neighbour is known
//! before light could have crossed the separation.
//!
//! Returns `None` for a seat that does not exist, is not seated on a people,
//! or whose people or world cannot be read now. Read-only.

use crate::growth::Power;
use crate::seats::{On, Role};
use crate::universe::Universe;
use xyl_core::dyadic::Dy;
use xyl_cosmos::pulse::{c_kpc_per_ring, hear, Hearing, Node, Relay};
use xyl_cosmos::worlds::Heartbeat;

/// A neighbouring people: its name, the separation in kpc and the light delay
/// across it in years.
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
    /// Deep time at the present, in Gyr.
    pub gyr: Dy,
    /// When the people woke, in Gyr after the Big Bang.
    pub dawn: Dy,
    /// The people's traits, as their human-readable description.
    pub traits: String,
    /// The people's population (count of bodies).
    pub numbers: Dy,
    /// The carrying capacity of the people's world, as a count of bodies.
    pub capacity: Dy,
    /// The people's growth rate, per year.
    pub growth_per_yr: Dy,
    /// The people's power budget in W; `None` when it is not read.
    pub power: Option<Power>,
    /// Surface temperature of the people's world, in K.
    pub surface_k: Dy,
    /// Surface pressure of the people's world, in bar.
    pub pressure_bar: Dy,
    /// The year's mean light reaching the people's world, in flux units.
    pub light: Dy,
    /// Every liquid medium of the people's world.
    pub media: Vec<MediumSeen>,
    /// The world's day, year and tidal lock, which pace percepts; `None` when
    /// they cannot be read.
    pub heartbeat: Option<Heartbeat>,
    /// The bells as heard at the people's star; `None` when the star has no
    /// place or nothing is heard.
    pub hearing: Option<Hearing>,
    /// Every other people whose dawn lies at least the light delay in the
    /// past.
    pub neighbours: Vec<Heard>,
    /// The latest eight annals that name the people, latest first, each as
    /// `epoch kind text`.
    pub annals: Vec<String>,
}

/// Returns what seat `seat` perceives at the universe's present. `nodes` and
/// `relays` are the bells at the present (pulse.rs). Neighbours are read at
/// current positions and kept only when their dawn lies at least the light
/// delay in the past; annals are the latest eight naming the people.
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
    // The pulse counts in rings: the time and light speed convert here (a
    // speed per ring read per Gyr is `rings_of_gyr_dy`, then per year).
    let at = xyl_cosmos::circuit::rings_of_gyr_dy(gyr);
    let hearing = here.and_then(|x| hear(&x, nodes, relays, at));
    let c = xyl_cosmos::circuit::rings_of_gyr_dy(c_kpc_per_ring()) / Dy::sci(1, 9);
    let neighbours = u
        .cosmic
        .peoples
        .iter()
        .enumerate()
        .filter(|(j, _)| *j != i)
        .filter_map(|(_, q)| {
            // Separation at current positions: two peoples in the same system
            // are separated by their orbital positions, not taken as zero.
            let at = |x: &crate::cosmic::People| crate::transit::Place {
                star: x.star,
                world: x.world,
            };
            let kpc =
                crate::transit::separation_m(&u.cosmic, at(p), at(q))? / crate::transit::kpc_m();
            let light_yr = kpc / c;
            // Heard once light could have crossed the separation since the
            // neighbour's dawn; the factor 10^9 converts Gyr to years.
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
