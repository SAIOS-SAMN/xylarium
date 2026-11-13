//! The home galaxy's gas: each bead read as a piece, its cooling by the two
//! channels, its collapse into infalls and the galaxy's lighting (see the
//! parent module's header for the formulas).

use super::{emission, half};
use crate::circuit::Bead;
use xyl_core::Q;

/// A piece's cooling: the bead's kinds at its pinch (element, share of the
/// piece), its holding per ring, its collapse
/// (rings to cool, the infall extent, each infall's holding, how many fit),
/// its tide and its heat at ignition (the cooled residue plus the pulse's heat).
pub(crate) struct Cooling {
    pub(crate) mix: Vec<(u32, Q)>,
    pub(crate) lambda: Q,
    pub(crate) cools_in: Option<i64>,
    pub(crate) infall: Q,
    pub(crate) cloud: Q,
    pub(crate) clouds: i64,
    pub(crate) tide: Q,
    pub(crate) heated: Q,
    /// The throat's push under the cooling's swing.
    pub(crate) push: Q,
}

/// A bead's cooling, read one tier down as a piece of the home galaxy's gas:
/// the bead's state over the founding `unit` is its holding (the galaxy's
/// grains hold one ring together), spread over its span; its kinds stand at
/// its pinch. The piece's agitation is what the push leaves there,
/// (p - delta) / p with the throat's push `p` swung by `swing`, and its
/// emission is the share-weighted mean of the kinds' emissions. `h` is the
/// core's hold, 1 plus the held share; `swing` is 1 + A_in. None for an
/// empty bead or span, or a unit that is not positive.
pub(crate) fn cooling(bead: &Bead, unit: &Q, h: &Q, swing: &Q) -> Option<Cooling> {
    let width = &bead.span.1 - &bead.span.0;
    if width.signum() <= 0 || unit.signum() <= 0 {
        return None;
    }
    let reach = bead.reach.clone();
    let total = bead.recipe.iter().fold(Q::ZERO, |a, (_, x)| &a + x);
    if total.signum() <= 0 {
        return None;
    }
    let mix: Vec<(u32, Q)> = bead
        .recipe
        .iter()
        .map(|(z, a)| (*z, a.checked_div(&total).unwrap_or(Q::ZERO)))
        .collect();
    let holding = bead.state.checked_div(unit).unwrap_or(Q::ZERO);
    let lambda = holding.checked_div(&width).unwrap_or(Q::ZERO);
    // Stand-in S116: The swung push is unbounded and unresisted.
    let push = crate::circuit::throat_push(xyl_core::dyadic::Dy::from_q(swing)).to_q();
    let deficit = xyl_core::shares::deficit(h, &reach).unwrap_or(Q::ZERO);
    // Stand-in S121: The deficit is subtracted from the push, not composed against it.
    let agitation = (&push - &deficit)
        .checked_div(&push)
        .unwrap_or(Q::ZERO)
        .max(Q::ZERO);
    let emission = mix
        .iter()
        .fold(Q::ZERO, |a, (z, s)| &a + &(s * &emission(*z)));
    let mu = &lambda * &emission;
    let ball = dimension_at(&reach);
    let (cools_in, infall, cloud, clouds, cooled, heat) = collapse(
        &lambda, &mu, &agitation, &emission, &reach, h, &width, swing, &ball,
    );
    let heated = &cooled + &heat;
    Some(Cooling {
        mix,
        lambda,
        cools_in,
        infall,
        cloud,
        clouds,
        tide: deficit,
        heated,
        push,
    })
}

/// Whether a cooled piece lights: it fragments into at least one infall and
/// its agitation at ignition is under the throat's push, swung as its
/// cooling was.
pub(crate) fn lights(c: &Cooling) -> bool {
    c.clouds > 0 && c.heated < c.push
}

/// The home galaxy's lighting: the rings until its gas first lights (the
/// first lit piece; None: no piece lights), the lit share of its gas by
/// holding, and each piece of gas by reach (inner edge, outer edge, galaxy
/// rings) with the rings until it lights (None: it never does).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lighting {
    /// Rings until the first piece lights.
    pub cools_in: Option<i64>,
    /// The lit pieces' holding (those that collapse into clouds and light)
    /// over every piece's, in [0, 1].
    pub lit: Q,
    /// Each piece: inner edge, outer edge (galaxy units), rings until it
    /// lights, and its infall's extent (galaxy units; zero where it never
    /// lights).
    pub pieces: Vec<(Q, Q, Option<i64>, Q)>,
}

impl Lighting {
    /// The rings until the gas at reach `r` (galaxy units) lights: the piece
    /// covering r, else the nearest piece's; None where that piece never does.
    pub fn cools_in_at(&self, r: &Q) -> Option<i64> {
        self.piece_at(r).and_then(|(_, p)| p.2)
    }

    /// The piece covering reach `r` (galaxy units), else the nearest: its
    /// index and its record.
    pub fn piece_at(&self, r: &Q) -> Option<(usize, &(Q, Q, Option<i64>, Q))> {
        let covering = self
            .pieces
            .iter()
            .enumerate()
            .find(|(_, (a, b, _, _))| a <= r && r < b);
        let nearest = || {
            self.pieces.iter().enumerate().min_by(|x, y| {
                let dx = (&half(&(&x.1 .0 + &x.1 .1)) - r).abs();
                let dy = (&half(&(&y.1 .0 + &y.1 .1)) - r).abs();
                dx.cmp(&dy)
            })
        };
        covering.or_else(nearest)
    }
}

/// The home galaxy's lighting from the founding's beads (found.rs), the
/// founding `unit`, the `held` share (the firmament's) and the inbound
/// echo's swing (1 + A_in): the cooling alone, without the star tier's cut,
/// so it is cheap enough for the founding (circuit::anchor, the first stars).
pub fn lighting(beads: &[Bead], unit: &Q, held: &Q, swing: &Q) -> Lighting {
    let h = &Q::ONE + held;
    let mut pieces = Vec::new();
    let (mut lit, mut all) = (Q::ZERO, Q::ZERO);
    let mut first: Option<i64> = None;
    for bead in beads {
        let Some(c) = cooling(bead, unit, &h, swing) else {
            continue;
        };
        let holding = &c.lambda * &(&bead.span.1 - &bead.span.0);
        all = &all + &holding;
        let lit_here = lights(&c);
        if lit_here {
            lit = &lit + &holding;
            first = match (first, c.cools_in) {
                (Some(x), Some(y)) => Some(x.min(y)),
                (x, y) => x.or(y),
            };
        }
        pieces.push((
            bead.span.0.clone(),
            bead.span.1.clone(),
            c.cools_in.filter(|_| lit_here),
            if lit_here { c.infall.clone() } else { Q::ZERO },
        ));
    }
    Lighting {
        cools_in: first,
        lit: lit.checked_div(&all).unwrap_or(Q::ZERO),
        pieces,
    }
}

/// The ball's share of the galaxy's manifold at `reach` (galaxy units): the
/// core's face there, `H / (H + reach)` with the core's hold `H = 1` (one
/// galaxy unit, where the bulge's ball and the disc's circle meet at half).
/// What spreads at a reach spreads on both measures, the bulge's ball (S^2)
/// by this share and the disc's circle (S^1) by the rest: a smooth passage,
/// all ball at the centre and all circle far out, never a cut (#233 item 2).
pub(crate) fn dimension_at(reach: &Q) -> Q {
    Q::ONE.checked_div(&(&Q::ONE + reach)).unwrap_or(Q::ZERO)
}

/// The measure of S^n an extent `w` at reach `r` holds: the measure within
/// the extent's angle (measure.rs: the circle's arc, n = 1, which is the
/// face itself; the sphere's cap, n = 2, which falls as the inverse square
/// far out). The angle is the deficit share `w / (w + r)` of the half turn,
/// in turns; the measure is the whole at the half turn. Cut toward zero to
/// the 2^-64 grid.
fn measure_on(w: &Q, r: &Q, n: u32) -> Q {
    use xyl_core::dyadic::Dy;
    let turns = w.checked_div(&(&Q::int(2) * &(w + r))).unwrap_or(Q::ZERO);
    xyl_core::certified::toward_zero(
        &xyl_core::measure::share_turns(n, Dy::from_q(&turns)).to_q(),
        64,
    )
}

/// The share an extent `w` at reach `r` holds on the manifold whose ball's
/// share is `ball` (`dimension_at`): the ball's cap by `ball` and the
/// circle's arc by `1 - ball`. One law for an infall's gathered share and a
/// system's intercepted share of a pulse.
pub(crate) fn gathered(w: &Q, r: &Q, ball: &Q) -> Q {
    &(ball * &measure_on(w, r, 2)) + &(&(&Q::ONE - ball) * &measure_on(w, r, 1))
}

/// A piece's collapse. The piece holds (lambda + mu) W at reach R about the
/// galaxy's core (lambda its holding per ring, mu the magnetism of its open
/// states, W its width); an infall of extent w on it gathers the piece's
/// measure within the infall's angle, m(w) = (lambda + mu) W share_n(theta),
/// theta the deficit angle w / (w + R) of the half turn in turns
/// (the ball's share `ball` at the reach, `dimension_at`), and grips it at the infall's rim by
/// the deficit law, m / (w + m). It holds where that grip exceeds its agitation
/// plus the shear across it, H w / (R (R + w)), and the heat the pulse
/// leaves: the ring the bell brings at R, 1 / (pi (R + 1)), swung by the
/// inbound echo (`swing` = 1 + A_in) and taken by the whole piece's grip,
/// over what the piece sheds per ring, capped at 1 (its open states' emission e and the
/// vacuum's halving). The extents are read on a ladder of halvings of the
/// width: the piece cools by whole halvings (radiative every ceil(1 / e)
/// rings, plus the vacuum's) until its residue is under the best extent's
/// spare, then fragments at the largest extent that holds into as many
/// infalls as fit. Returns (rings to cool, the extent, each infall's
/// holding, count, cooled residue, the heat). A piece whose grip never
/// exceeds the shear and the heat, or that needs more than 4096 halvings or
/// has no cooling channel, never collapses (None, no infall).
pub(crate) fn collapse(
    lambda: &Q,
    mu: &Q,
    a0: &Q,
    e: &Q,
    reach: &Q,
    h: &Q,
    width: &Q,
    swing: &Q,
    ball: &Q,
) -> (Option<i64>, Q, Q, i64, Q, Q) {
    // The piece's hold is (lambda + mu) W, and an infall's grip at its rim is
    // m / (w + m) with m = (lambda + mu) W share_n(theta).
    let shell = &(lambda + mu) * width;
    let grip = |w: &Q| {
        let m = &shell * &gathered(w, reach, ball);
        m.checked_div(&(w + &m)).unwrap_or(Q::ZERO)
    };
    // The pulse's heat is 1 / (pi (R + 1)) times the swing and the whole
    // piece's grip, over what the piece sheds per ring (`pulse_heat`).
    let vacuum = vacuum_halving(reach, h);
    let ring = Q::ONE
        .checked_div(&(&xyl_core::certified::pi(64).mid() * &(reach + &Q::ONE)))
        .unwrap_or(Q::ZERO);
    let heat = pulse_heat(&(&(&ring * swing) * &grip(width)), e, vacuum);
    let none = (None, Q::ZERO, Q::ZERO, 0, Q::ZERO, heat.clone());
    // The spare, spare(w) = grip(w) - H w / (R (R + w)) - heat, is the
    // agitation an infall of extent w can still hold against.
    let spare = |w: &Q| {
        let shear = (h * w)
            .checked_div(&(reach * &(reach + w)))
            .unwrap_or(Q::ZERO);
        &(&grip(w) - &shear) - &heat
    };
    // The extents read are W, W/2, ..., W/2^47 (nonzero ones only).
    let ladder: Vec<Q> = (0..48)
        .scan(width.clone(), |w, _| {
            let out = w.clone();
            *w = half(w);
            Some(out)
        })
        .filter(|w| w.signum() > 0)
        .collect();
    let spares: Vec<(Q, Q)> = ladder.iter().map(|w| (w.clone(), spare(w))).collect();
    let Some((best, critical)) = spares.iter().max_by(|x, y| x.1.cmp(&y.1)).cloned() else {
        return none;
    };
    if critical.signum() <= 0 {
        return none;
    }
    // Count the halvings needed for the residue to fall to the best spare;
    // past 4096 the piece is treated as never collapsing.
    let mut halvings = 0i64;
    let mut a = a0.clone();
    while a > critical {
        a = half(&a);
        halvings += 1;
        if halvings > 4096 {
            return none;
        }
    }
    let Some(cools_in) = rings_to_cool(halvings, e, vacuum) else {
        return none;
    };
    // The infall's extent is the largest on the ladder whose spare covers the
    // cooled residue.
    let infall = spares
        .iter()
        .filter(|(_, sp)| *sp >= a)
        .map(|(w, _)| w.clone())
        .max()
        .unwrap_or(best);
    let clouds = i64::try_from(
        width
            .checked_div(&infall)
            .map(|x| x.floor_i128().unwrap_or(0))
            .unwrap_or(0),
    )
    .unwrap_or(0)
    .max(1);
    (
        Some(cools_in),
        infall.clone(),
        &shell * &gathered(&infall, reach, ball),
        clouds,
        a,
        heat,
    )
}

/// The heat the pulse leaves in matter that takes `taken` of it per ring
/// (the ring the bell brings, times the swing and the matter's grip): the
/// taken share over what the matter sheds per ring, capped at 1. The shed is
/// the open states' emission `open` plus `ln 2 / k_v` for the vacuum's
/// halving every `k_v` rings (`vacuum`, from [`vacuum_halving`]). With no
/// shedding the heat is 1. One form for a piece's gas (`collapse`) and a
/// star's disc (`disc::star_hold`).
pub(crate) fn pulse_heat(taken: &Q, open: &Q, vacuum: Option<i64>) -> Q {
    let ln2 = xyl_core::certified::ln2(64).mid();
    let shed = open
        + &vacuum
            .map(|k| ln2.checked_div(&Q::int(k)).unwrap_or(Q::ZERO))
            .unwrap_or(Q::ZERO);
    if shed.signum() > 0 {
        taken.checked_div(&shed).unwrap_or(Q::ONE).min(Q::ONE)
    } else {
        Q::ONE
    }
}

/// The rings a piece takes to halve its agitation `halvings` times: by ring
/// n it has halved floor(n / P_e) + floor(n / k_v) times, P_e = ceil(1 / e)
/// rings per radiative halving and k_v the vacuum's (`vacuum`); the answer is
/// the first ring reaching `halvings`, found by bisection. None where neither
/// channel cools.
fn rings_to_cool(halvings: i64, e: &Q, vacuum: Option<i64>) -> Option<i64> {
    let emission = Q::ONE
        .checked_div(e)
        .map(|x| -((-x).floor_i128().unwrap_or(-1)))
        .filter(|&p| p >= 1);
    Some(match (emission, vacuum) {
        _ if halvings == 0 => 0,
        (None, None) => return None,
        (Some(pe), None) => halvings * i64::try_from(pe).unwrap_or(i64::MAX),
        (None, Some(kv)) => halvings * kv,
        (Some(pe), Some(kv)) => {
            let pe = i64::try_from(pe).unwrap_or(i64::MAX);
            let reached = |n: i64| n / pe + n / kv >= halvings;
            let (mut lo, mut hi) = (0i64, halvings * pe.min(kv));
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                if reached(mid) {
                    hi = mid;
                } else {
                    lo = mid + 1;
                }
            }
            lo
        }
    })
}

/// The rings the vacuum takes to halve an agitation at reach `r` about a
/// hold `h`: the first whole k with (r / (r + h))^k <= 1/2, i.e.
/// k >= ln 2 / ln((r + h) / r), using the certified logarithms' upper bound.
/// None where nothing holds.
pub(crate) fn vacuum_halving(r: &Q, h: &Q) -> Option<i64> {
    if h.signum() <= 0 || r.signum() <= 0 {
        return None;
    }
    let ratio = (r + h).checked_div(r)?;
    let ln2 = xyl_core::certified::ln(&Q::int(2), 64)?;
    let lr = xyl_core::certified::ln(&ratio, 64)?;
    let k = ln2.hi.checked_div(&lr.lo)?;
    let whole = -((-k).floor_i128()?);
    i64::try_from(whole.max(1)).ok()
}

#[cfg(test)]
mod share_tests {
    use super::*;

    /// A share falls as its manifold's own measure: on the disc's circle
    /// (n = 1) it is the face `w / (w + r)` itself and halves as the reach
    /// doubles far out; in the bulge's ball (n = 2) it is the cap, below the
    /// face, and quarters as the reach doubles; both hold the whole at the
    /// centre.
    #[test]
    fn a_share_falls_as_its_manifolds_measure() {
        let w = Q::ONE;
        let (near, far) = (Q::int(1000), Q::int(2000));
        let face = w.checked_div(&(&w + &near)).unwrap();
        let tiny = xyl_core::ratio!(1, 1_000_000_000);
        assert!((&measure_on(&w, &near, 1) - &face).abs() < tiny);
        let ratio = |n| {
            measure_on(&w, &near, n)
                .checked_div(&measure_on(&w, &far, n))
                .unwrap()
        };
        assert!(
            (&ratio(1) - &Q::int(2)).abs() < xyl_core::ratio!(1, 100),
            "{}",
            ratio(1)
        );
        assert!(
            (&ratio(2) - &Q::int(4)).abs() < xyl_core::ratio!(1, 100),
            "{}",
            ratio(2)
        );
        assert!(measure_on(&w, &near, 2) < measure_on(&w, &near, 1));
        assert!((&measure_on(&w, &Q::ZERO, 2) - &Q::ONE).abs() < tiny);
        // The ball's share passes smoothly: whole at the centre, half at the
        // core's reach, falling with the reach and never cut; the gathered
        // share lies between the two measures at every reach (the cap above
        // the arc where the extent rivals the reach, below it far out).
        assert_eq!(dimension_at(&Q::ZERO), Q::ONE);
        assert_eq!(dimension_at(&Q::ONE), xyl_core::ratio!(1, 2));
        assert!(dimension_at(&Q::int(2)) < dimension_at(&xyl_core::ratio!(1, 2)));
        for r in [xyl_core::ratio!(1, 2), Q::ONE, Q::int(3), near.clone()] {
            let (cap, arc) = (measure_on(&w, &r, 2), measure_on(&w, &r, 1));
            let g = gathered(&w, &r, &dimension_at(&r));
            let (lo, hi) = if cap < arc {
                (&cap, &arc)
            } else {
                (&arc, &cap)
            };
            assert!(lo <= &g && &g <= hi, "{r}: {cap} {g} {arc}");
        }
    }
}
