//! A galaxy's centre: an Einstein-Cartan black hole and the exact books of
//! the matter that falls to it, passes its throat, and returns as jets.
//!
//! Reads `bounce::Bounce` (rebuilt on every change of mass) and
//! `circuit::echo` for the jet episode (`galaxy_episode`, the echo, two
//! rings). Masses are suns and times rings (one ring one bell), all exact
//! `Q`; exponentials are `xyl_core::certified` enclosures whose lower bound
//! is fixed on a 2^-40 grid.
//!
//! What falls (`Core::capture`) is split:
//!
//! - `horizon_share` (1/10, a ruler) crosses the horizon; `efficiency`
//!   (1/10 of that, a ruler) is radiated as light on the way in, and the
//!   rest enters the throat, growing the hole beyond its formation mass.
//!   Each capture's record is folded in order into a BLAKE3 digest that
//!   seeds the far universe (`FarCore::seed`).
//! - The rest enters the accretion reservoir; `Core::run` returns
//!   `reservoir (1 - exp(-dt / tau))` to the bulge as jets, tau the episode.
//! - `Core::far_run` advances the far core's clock and releases
//!   `held (1 - exp(-dt / tau))` of the throat into the far universe; the
//!   hole's mass falls by that amount, never below its formation mass.
//! - `Core::arrive` books matter sent back from the paired universe into the
//!   reservoir, only when that universe sends it.
//!
//! Invariant (`Core::conserved`, exact at all times): fallen + arrived =
//! radiated + reservoir + returned + in throat + far released, and the
//! amount in the throat equals the hole's mass beyond its formation mass.
//! `Core::new`, `capture` and `far_run` return `None` where `Bounce::of`
//! refuses.

use crate::bounce::Bounce;
use xyl_core::certified::{down, exp, Encl};
use xyl_core::Q;

const BITS: u32 = 64;
const GRID_BITS: u32 = 40;

/// The share of infalling matter that crosses the horizon (the rest is held
/// in the accretion flow outside it).
pub fn horizon_share() -> Q {
    Q::ratio(1, 10)
}
/// The share of crossing matter radiated as light on the way in.
pub fn efficiency() -> Q {
    Q::ratio(1, 10)
}
/// A galaxy's jet episode (rings): the galaxy's echo, the pulse out to its
/// rim and back twice (`circuit::echo`, two rings), so the jet returns on
/// the galaxy's clock and no span is declared. A centre carries the episode
/// it was made with.
pub fn galaxy_episode() -> Q {
    crate::circuit::echo().to_q()
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
    /// The far core's elapsed deep time (rings).
    pub clock: Q,
    /// An ordered digest of every record that crossed; seeds the far universe.
    pub records: [u8; 32],
    /// Number of capture records folded into `records`.
    pub crossings: u64,
}

impl FarCore {
    /// The far universe's seed: the first eight bytes of the digest of what
    /// crossed, little-endian.
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
    /// The jet episode's e-folding time (rings): the galaxy's echo
    /// ([`galaxy_episode`]).
    pub episode: Q,
    /// Mass at formation (suns).
    pub mass0: Q,
    /// Held outside the horizon, to be returned as jets (suns).
    pub reservoir: Q,
    /// Fallen in, cumulative (suns).
    pub fallen: Q,
    /// Radiated as light on the way through the horizon, cumulative (suns).
    pub radiated: Q,
    /// Returned from the reservoir to the bulge as jets, cumulative (suns).
    pub returned: Q,
    /// Arrived from the paired universe through the throat, cumulative (suns).
    pub arrived: Q,
    /// The throat's far face: the matter it holds and what it has released
    /// and sent.
    pub far: FarCore,
}

impl Core {
    /// A centre of `mass` suns and Kerr parameter `spin`, its horizon read in
    /// its galaxy's S^360 enclosure of curvature radius `enclosure`, crossed
    /// at the galaxy's phase `phase` (its bead's capture phase, turns), its
    /// jets returning over `episode` rings (the galaxy's echo,
    /// [`galaxy_episode`]). `None` where `Bounce::of` refuses.
    pub fn new(mass: Q, spin: &Q, enclosure: &Q, phase: &Q, episode: &Q) -> Option<Core> {
        let hole = Bounce::of(&mass, spin, enclosure, phase)?;
        Some(Core {
            hole,
            enclosure: enclosure.clone(),
            episode: episode.clone(),
            mass0: mass,
            reservoir: Q::ZERO,
            fallen: Q::ZERO,
            radiated: Q::ZERO,
            returned: Q::ZERO,
            arrived: Q::ZERO,
            far: FarCore::default(),
        })
    }

    /// The hole's current mass (suns): its formation mass plus what the
    /// throat holds.
    pub fn mass(&self) -> &Q {
        &self.hole.mass
    }

    /// `suns` fall to the centre, recorded as `record` (e.g. a star's shed
    /// matter, an unbound world). Returns the mass added to the throat; zero
    /// for a non-positive `suns`, `None` where `Bounce::of` refuses the new
    /// mass.
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
        // The hole grows by what enters the throat; its bounce (horizon and
        // white hole) is rebuilt at the new mass.
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

    /// Advances `dt` rings of deep time: the reservoir returns
    /// `reservoir x (1 - exp(-dt / tau))` to the bulge as jets, the lower
    /// bound rounded down to the 2^-40 grid. Returns the amount returned.
    pub fn run(&mut self, dt: &Q) -> Q {
        if self.reservoir.signum() <= 0 || dt.signum() <= 0 {
            return Q::ZERO;
        }
        let x = dt.checked_div(&self.episode).unwrap_or(Q::ZERO);
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

    /// Advances the far core's clock by `dt` rings: its engine releases
    /// `held x (1 - exp(-dt / tau))` of the throat's matter into the far
    /// universe (the lower bound rounded down to the 2^-40 grid), and the
    /// hole's mass falls by that amount, never below its formation mass.
    /// Returns the amount released, or `None` where `Bounce::of` refuses.
    pub fn far_run(&mut self, dt: &Q) -> Option<Q> {
        if dt.signum() <= 0 {
            return Some(Q::ZERO);
        }
        self.far.clock = &self.far.clock + dt;
        if self.far.held.signum() <= 0 {
            return Some(Q::ZERO);
        }
        let x = dt.checked_div(&self.episode).unwrap_or(Q::ZERO);
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

    /// The far core sends `suns` of the far universe's matter back through
    /// the throat: it arrives at this face, outside the horizon, to be returned as
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
    fn books_conserved_and_jets_drain_reservoir() {
        let mut c = Core::new(
            Q::int(4_000_000),
            &Q::ratio(7, 10),
            &Q::int(1_000_000_000),
            &Q::ZERO,
            &Q::int(100),
        )
        .unwrap();
        assert!(c.conserved());
        let crossed = c.capture(&Q::int(20), b"a star's shed envelope").unwrap();
        // What crosses is a share of what fell, and the hole grows by it.
        assert!(crossed.signum() > 0 && crossed < Q::int(20), "{crossed}");
        assert_eq!(c.mass(), &(&Q::int(4_000_000) + &crossed));
        assert!(c.conserved());
        let r0 = c.reservoir.clone();
        let back = c.run(&Q::int(100));
        // One e-fold returns 1 - 1/e of the reservoir, to the grid.
        assert!(
            back > &r0 * &Q::ratio(632, 1000) && back < &r0 * &Q::ratio(633, 1000),
            "{back}"
        );
        c.arrive(&Q::int(3));
        assert!(c.conserved());
        // The far core releases what the throat holds, on the far core's
        // clock: the hole's mass falls back toward its formation mass, never
        // below it.
        let before = c.mass().clone();
        let out = c.far_run(&Q::int(300)).unwrap();
        assert!(out.signum() > 0 && c.mass() < &before && c.mass() >= &c.mass0);
        assert!(c.conserved());
        for _ in 0..40 {
            c.far_run(&Q::int(100)).unwrap();
        }
        assert!(
            c.conserved() && c.far.held < Q::ratio(1, 1000),
            "the throat passes matter through: {}",
            c.far.held
        );
        assert_eq!(c.far.clock, Q::int(4300));
        for _ in 0..20 {
            c.run(&Q::int(100));
        }
        assert!(c.conserved() && c.reservoir < Q::ONE);
        // The horizon grew with the hole: it stands exactly at the reach where
        // the deficit is one third, curved along the enclosure.
        assert_eq!(
            c.hole.at[1],
            crate::bounce::curved_reach(&crate::bounce::horizon_reach(c.mass()), &c.enclosure)
        );
    }

    #[test]
    fn far_seed_depends_on_capture_order() {
        let mut a = Core::new(
            Q::int(10),
            &Q::ratio(1, 2),
            &Q::int(1_000_000),
            &Q::ZERO,
            &Q::int(100),
        )
        .unwrap();
        let mut b = Core::new(
            Q::int(10),
            &Q::ratio(1, 2),
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
