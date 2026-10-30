//! A galaxy's centre: an Einstein–Cartan black hole, with every unit of
//! matter in exactly one place.
//!
//! What falls to the centre is split, and nothing is counted twice:
//!
//! - The share that crosses the horizon. A fraction of it (the accretion
//!   efficiency) is radiated as light on the way in, a declared sink of
//!   mass-energy. The rest is in the throat: the hole's mass beyond its
//!   formation mass. A core is a larger wormhole with no open-ended
//!   singularity. At the Cartan density the matter bounces into the far core,
//!   the persistent central engine of the universe on the other side
//!   ([`crate::bounce`]), which has its own clock. The far engine releases
//!   what the throat holds into its universe over its own episodes, and the
//!   hole's mass falls accordingly: matter passes through rather than being
//!   stored. The far core also sends its own universe's matter back through
//!   the same throat.
//! - The rest stays outside, in the accretion reservoir, and returns to the
//!   bulge as jets over an episode's e-folding time (the galaxy's own echo,
//!   [`galaxy_episode_myr`]):
//!   `returned = reservoir·(1 − exp(−Δt/τ))`, a certified enclosure, fixed on
//!   the grid.
//!
//! Matter arriving from a paired universe through a white hole is a separate
//! declared source, booked only when that universe sends it (a handoff on the
//! network). Crossed matter is never also kept on this side, so nothing is
//! counted twice; [`Core::conserved`] holds exactly at all times.

use crate::bounce::Bounce;
use xyl_core::certified::{down, exp, Encl};
use xyl_core::Q;

const BITS: u32 = 64;
const GRID_BITS: u32 = 40;

fn q(n: i64, d: i64) -> Q {
    Q::new(n, d).unwrap_or(Q::ZERO)
}

/// The share of infalling matter that crosses the horizon (the rest is held
/// in the accretion flow outside it).
pub fn horizon_share() -> Q {
    q(1, 10)
}
/// The share of crossing matter radiated as light on the way in.
pub fn efficiency() -> Q {
    q(1, 10)
}
/// A galaxy's jet episode (Myr): its own echo, the pulse out to its rim and
/// back twice (circuit::anchor), so the jet returns on the galaxy's clock
/// and no span is declared. Zero until a universe is founded in the process;
/// a centre carries the episode it was made with.
pub fn galaxy_episode_myr() -> Q {
    (crate::circuit::echo_gyr() * xyl_core::dyadic::Dy::int(1000)).to_q()
}

/// The throat's far face: the persistent central engine of the universe on
/// the other side.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FarCore {
    /// Matter in the throat, not yet released on the far side (suns): the
    /// hole's mass beyond its formation mass.
    pub held: Q,
    /// Released into the far universe by its engine, cumulative (suns).
    pub released: Q,
    /// Sent back through the throat from the far side, cumulative (suns).
    pub sent: Q,
    /// The far core's own elapsed deep time (Myr).
    pub clock_myr: Q,
    /// An ordered digest of every record that crossed; seeds the far universe.
    pub records: [u8; 32],
    /// Number of capture records folded into `records`.
    pub crossings: u64,
}

impl FarCore {
    /// The far universe's seed, from the digest of what crossed.
    pub fn seed(&self) -> u64 {
        let b = self.records;
        u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
    }
}

/// A galaxy's central black hole and the matter around it.
#[derive(Clone, Debug)]
pub struct Core {
    /// The hole: its mass, spin and horizon, and the white hole beyond.
    pub hole: Bounce,
    /// The curvature radius of the galaxy's S^360 enclosure the horizon is
    /// read in (the hole's units).
    pub enclosure: Q,
    /// The jet episode's e-folding time (Myr): the galaxy's own echo
    /// ([`galaxy_episode_myr`]).
    pub episode_myr: Q,
    /// Mass at formation (suns).
    pub mass0: Q,
    /// Held outside the horizon, to be returned as jets (suns).
    pub reservoir: Q,
    /// Every account, cumulative (suns): fallen in, radiated as light,
    /// returned as jets, and arrived from a paired universe.
    pub fallen: Q,
    /// Radiated as light on the way through the horizon, cumulative (suns).
    pub radiated: Q,
    /// Returned from the reservoir to the bulge as jets, cumulative (suns).
    pub returned: Q,
    /// Arrived from the paired universe through the throat, cumulative (suns).
    pub arrived: Q,
    /// The throat's far face: the matter it holds and what it has released and sent.
    pub far: FarCore,
}

impl Core {
    /// A centre of `mass` suns and Kerr parameter `spin`, its horizon read in
    /// its galaxy's S^360 enclosure of curvature radius `enclosure`, crossed
    /// at the galaxy's phase `phase` (its bead's capture phase, turns), its
    /// jets returning over `episode_myr` (the galaxy's echo,
    /// [`galaxy_episode_myr`]).
    pub fn new(mass: Q, spin: &Q, enclosure: &Q, phase: &Q, episode_myr: &Q) -> Option<Core> {
        let hole = Bounce::of(&mass, spin, enclosure, phase)?;
        Some(Core {
            hole,
            enclosure: enclosure.clone(),
            episode_myr: episode_myr.clone(),
            mass0: mass,
            reservoir: Q::ZERO,
            fallen: Q::ZERO,
            radiated: Q::ZERO,
            returned: Q::ZERO,
            arrived: Q::ZERO,
            far: FarCore::default(),
        })
    }

    /// The hole's current mass (suns): its formation mass plus what the throat holds.
    pub fn mass(&self) -> &Q {
        &self.hole.mass
    }

    /// `suns` fall to the centre, recorded as `record` (e.g. a star's shed
    /// matter, an unbound world). Returns the mass added to the throat.
    pub fn capture(&mut self, suns: &Q, record: &[u8]) -> Option<Q> {
        if suns.signum() <= 0 {
            return Some(Q::ZERO);
        }
        let crossed = suns * &horizon_share();
        let light = &crossed * &efficiency();
        let grows = &crossed - &light;
        self.fallen = &self.fallen + suns;
        self.radiated = &self.radiated + &light;
        self.reservoir = &self.reservoir + &(suns - &crossed);
        // into the throat: the hole grows; its horizon and white hole are
        // rebuilt, exact
        self.hole = Bounce::of(
            &(self.mass() + &grows),
            &self.hole.spin,
            &self.enclosure,
            &self.hole.phase,
        )?;
        self.far.held = &self.far.held + &grows;
        let mut h = blake3::Hasher::new_derive_key("xylarium 2026-10-07 what crossed");
        h.update(&self.far.records).update(record);
        self.far.records = *h.finalize().as_bytes();
        self.far.crossings += 1;
        Some(grows)
    }

    /// Advances `myr` of deep time: the reservoir returns
    /// `reservoir·(1 − exp(−Δt/τ))` to the bulge as jets (fixed on the grid).
    /// Returns the amount returned.
    pub fn run(&mut self, myr: &Q) -> Q {
        if self.reservoir.signum() <= 0 || myr.signum() <= 0 {
            return Q::ZERO;
        }
        let x = myr.checked_div(&self.episode_myr).unwrap_or(Q::ZERO);
        let kept = exp(&(-x), BITS);
        let share = Encl::exact(Q::ONE).sub(&kept);
        let back = down(
            &Encl::exact(self.reservoir.clone()).mul(&share).lo,
            GRID_BITS,
        )
        .min(self.reservoir.clone());
        self.reservoir = &self.reservoir - &back;
        self.returned = &self.returned + &back;
        back
    }

    /// Advances the far core's clock by `myr`: its engine releases
    /// `held x (1 - exp(-dt/tau))` of the throat's matter into its universe
    /// (fixed on the grid), and the hole's mass falls by that amount, never
    /// below its formation mass. Returns the amount released.
    pub fn far_run(&mut self, myr: &Q) -> Option<Q> {
        if myr.signum() <= 0 {
            return Some(Q::ZERO);
        }
        self.far.clock_myr = &self.far.clock_myr + myr;
        if self.far.held.signum() <= 0 {
            return Some(Q::ZERO);
        }
        let x = myr.checked_div(&self.episode_myr).unwrap_or(Q::ZERO);
        let share = Encl::exact(Q::ONE).sub(&exp(&(-x), BITS));
        let out = down(
            &Encl::exact(self.far.held.clone()).mul(&share).lo,
            GRID_BITS,
        )
        .min(self.far.held.clone());
        self.hole = Bounce::of(
            &(self.mass() - &out),
            &self.hole.spin,
            &self.enclosure,
            &self.hole.phase,
        )?;
        self.far.held = &self.far.held - &out;
        self.far.released = &self.far.released + &out;
        Some(out)
    }

    /// The far core sends `suns` of its universe's matter back through the
    /// throat: it arrives at this face, outside the horizon, to be returned as
    /// jets (a handoff over the uplink from the joined domain).
    pub fn arrive(&mut self, suns: &Q) {
        if suns.signum() > 0 {
            self.reservoir = &self.reservoir + suns;
            self.arrived = &self.arrived + suns;
            self.far.sent = &self.far.sent + suns;
        }
    }

    /// Conservation across both faces, exactly:
    /// fallen + arrived = radiated + reservoir + returned + in throat +
    /// far released; and the amount in the throat equals the hole's mass
    /// beyond its formation mass.
    pub fn conserved(&self) -> bool {
        let in_throat = self.mass() - &self.mass0;
        &self.fallen + &self.arrived
            == &(&(&(&in_throat + &self.radiated) + &self.reservoir) + &self.returned)
                + &self.far.released
            && in_throat == self.far.held
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_counted_twice_and_the_jets_drain_the_reservoir() {
        let mut c = Core::new(
            Q::int(4_000_000),
            &q(7, 10),
            &Q::int(1_000_000_000),
            &Q::ZERO,
            &Q::int(100),
        )
        .unwrap();
        assert!(c.conserved());
        let crossed = c.capture(&Q::int(20), b"a star's shed envelope").unwrap();
        // what crosses is a share of what fell, and the hole grows by it
        assert!(crossed.signum() > 0 && crossed < Q::int(20), "{crossed}");
        assert_eq!(c.mass(), &(&Q::int(4_000_000) + &crossed));
        assert!(c.conserved());
        let r0 = c.reservoir.clone();
        let back = c.run(&Q::int(100));
        // one e-fold returns 1 - 1/e of the reservoir, to the grid
        assert!(
            back > &r0 * &q(632, 1000) && back < &r0 * &q(633, 1000),
            "{back}"
        );
        c.arrive(&Q::int(3));
        assert!(c.conserved());
        // the far core releases what the throat holds, on its own clock: the
        // hole's mass falls back toward what it formed with, never below
        let before = c.mass().clone();
        let out = c.far_run(&Q::int(300)).unwrap();
        assert!(out.signum() > 0 && c.mass() < &before && c.mass() >= &c.mass0);
        assert!(c.conserved());
        for _ in 0..40 {
            c.far_run(&Q::int(100)).unwrap();
        }
        assert!(
            c.conserved() && c.far.held < q(1, 1000),
            "the throat passes matter through: {}",
            c.far.held
        );
        assert_eq!(c.far.clock_myr, Q::int(4300));
        for _ in 0..20 {
            c.run(&Q::int(100));
        }
        assert!(c.conserved() && c.reservoir < Q::ONE);
        // the horizon grew with the hole, exactly at the one-third reach along
        // its enclosure
        assert_eq!(
            c.hole.at[1],
            crate::bounce::curved_reach(&crate::bounce::horizon_reach(c.mass()), &c.enclosure)
        );
    }

    #[test]
    fn the_far_universe_is_seeded_by_what_crossed_in_order() {
        let mut a = Core::new(
            Q::int(10),
            &q(1, 2),
            &Q::int(1_000_000),
            &Q::ZERO,
            &Q::int(100),
        )
        .unwrap();
        let mut b = Core::new(
            Q::int(10),
            &q(1, 2),
            &Q::int(1_000_000),
            &Q::ZERO,
            &Q::int(100),
        )
        .unwrap();
        a.capture(&Q::int(1), b"one");
        a.capture(&Q::int(1), b"two");
        b.capture(&Q::int(1), b"two");
        b.capture(&Q::int(1), b"one");
        assert_eq!(a.far.held, b.far.held);
        assert_ne!(
            a.far.seed(),
            b.far.seed(),
            "the records are history, not a sum"
        );
    }
}
