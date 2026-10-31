//! The home galaxy's gas: each bead read as a piece, its cooling by the two
//! channels, its collapse into infalls and the galaxy's lighting (see the
//! parent module's header for the formulas).

use super::{emission, half, throat_push};
use crate::circuit::Bead;
use xyl_core::Q;

/// A piece's cooling: the bead's kinds at its pinch (element, share of the
/// piece), its holding per ring, its agitation and emission, its collapse
/// (rings to cool, the infall extent, each infall's holding, how many fit,
/// the cooled residue), its tide and its heat at ignition.
pub(crate) struct Cooling {
    pub(crate) mix: Vec<(u32, Q)>,
    pub(crate) lambda: Q,
    pub(crate) agitation: Q,
    pub(crate) emission: Q,
    pub(crate) cools_in: Option<i64>,
    pub(crate) infall: Q,
    pub(crate) cloud: Q,
    pub(crate) clouds: i64,
    pub(crate) cooled: Q,
    pub(crate) tide: Q,
    pub(crate) heated: Q,
}

/// A bead's cooling, read one tier down as a piece of the home galaxy's gas:
/// the bead's state over the founding `unit` is its holding (the galaxy's
/// grains hold one ring together), spread over its span; its kinds stand at
/// its pinch, each with the agitation the push leaves there and its own
/// emission. `h` is the core's hold, 1 plus the held share; `swing` is
/// 1 + A_in. None for an empty bead or span.
pub(crate) fn cooling(bead: &Bead, unit: &Q, h: &Q, swing: &Q) -> Option<Cooling> {
    let grid = |x: &Q| xyl_core::certified::down(x, 64);
    let width = &bead.span.1 - &bead.span.0;
    if width.signum() <= 0 || unit.signum() <= 0 {
        return None;
    }
    let reach = grid(&bead.reach);
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
    let lambda = grid(&holding.checked_div(&width).unwrap_or(Q::ZERO));
    let push = throat_push();
    let deficit = h.checked_div(&(&reach + h)).unwrap_or(Q::ZERO);
    let agitation = grid(
        &(&push - &deficit)
            .checked_div(&push)
            .unwrap_or(Q::ZERO)
            .max(Q::ZERO),
    );
    let emission = grid(
        &mix.iter()
            .fold(Q::ZERO, |a, (z, s)| &a + &(s * &emission(*z))),
    );
    let mu = grid(&(&lambda * &emission));
    // the measure's dimension: n = 1 (a circle in the galaxy's plane) outside
    // the core's reach of one galaxy ring, n = 2 (the bulge's ball) within it
    let n = if reach > Q::ONE { 1 } else { 2 };
    let (cools_in, infall, cloud, clouds, cooled, heat) = collapse(
        &lambda, &mu, &agitation, &emission, &reach, h, &width, swing, n,
    );
    let heated = &cooled + &heat;
    Some(Cooling {
        mix,
        lambda,
        agitation,
        emission,
        cools_in,
        infall,
        cloud,
        clouds,
        cooled,
        tide: deficit,
        heated,
    })
}

/// Whether a cooled piece lights: it fragments into at least one infall and
/// its agitation at ignition is under the throat's push.
pub(crate) fn lights(c: &Cooling) -> bool {
    c.clouds > 0 && c.heated < throat_push()
}

/// The home galaxy's lighting: the rings until its gas first lights (the
/// first lit piece; None: no piece lights), the lit share of its gas by
/// holding, and each piece of gas by reach (inner edge, outer edge, galaxy
/// rings) with the rings until it lights (None: it never does).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lighting {
    /// Rings until the first piece lights.
    pub cools_in: Option<i64>,
    /// The lit pieces' holding (those that collapse into clouds) over every
    /// piece's, in [0, 1].
    pub lit: Q,
    /// Each piece: inner edge, outer edge (galaxy rings), rings until it
    /// lights, and its infall's extent (galaxy rings; zero where it never
    /// lights).
    pub pieces: Vec<(Q, Q, Option<i64>, Q)>,
}

impl Lighting {
    /// The rings until the gas at reach `r` (galaxy rings) lights: the piece
    /// covering r, else the nearest piece's; None where that piece never does.
    pub fn cools_in_at(&self, r: &Q) -> Option<i64> {
        self.piece_at(r).and_then(|(_, p)| p.2)
    }

    /// The piece covering reach `r` (galaxy rings), else the nearest: its
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

/// The share of a piece an infall of extent `w` at reach `r` gathers: the
/// measure of S^n within the infall's angle (measure.rs: the circle's arc
/// for the disc, n = 1; the sphere's cap for the bulge, n = 2), the angle
/// the deficit share gives in turns, `w / (w + r)` of the half turn, exact;
/// on the 2^-64 grid; the whole at the half turn.
fn gathered(w: &Q, r: &Q, n: u32) -> Q {
    use xyl_core::dyadic::Dy;
    let turns = w.checked_div(&(&Q::int(2) * &(w + r))).unwrap_or(Q::ZERO);
    xyl_core::certified::down(
        &xyl_core::measure::share_turns(n, Dy::from_q(&turns)).to_q(),
        64,
    )
}

/// A piece's collapse. The piece holds (lambda + mu) W at reach R about the
/// galaxy's core (lambda its holding per ring, mu the magnetism of its open
/// frames, W its width); an infall of extent w on it gathers the piece's
/// measure within its own angle, m(w) = (lambda + mu) W share_n(theta), theta
/// the deficit angle w / (w + R) of the half turn in turns
/// (`n` 1 in the disc, 2 in the bulge), and grips it at its own rim by the
/// deficit law, m / (w + m). It holds where that grip exceeds its agitation
/// plus the shear across it, H w / (R (R + w)), and the heat the pulse
/// leaves: the ring the bell brings at R, 1 / (pi (R + 1)), swung by the
/// inbound echo (`swing` = 1 + A_in) and taken by the piece's grip, over
/// what the piece sheds per ring (its open frames' emission e and the
/// vacuum's halving). The extents are read on a ladder of halvings of the
/// width: the piece cools by whole halvings (radiative every ceil(1 / e)
/// rings, plus the vacuum's) until its residue is under the best extent's
/// spare, then fragments at the largest extent that holds into as many
/// infalls as fit. Returns (rings to cool, the extent, each infall's
/// holding, count, cooled residue, the heat); a piece whose grip never
/// exceeds the shear and the heat never collapses (None, no infall).
pub(crate) fn collapse(
    lambda: &Q,
    mu: &Q,
    a0: &Q,
    e: &Q,
    reach: &Q,
    h: &Q,
    width: &Q,
    swing: &Q,
    n: u32,
) -> (Option<i64>, Q, Q, i64, Q, Q) {
    let grid = |x: &Q| xyl_core::certified::down(x, 64);
    // the piece's hold (lambda + mu) W, and an infall's grip at its rim,
    // m / (w + m) with m = (lambda + mu) W share_n(theta)
    let shell = &(lambda + mu) * width;
    let grip = |w: &Q| {
        let m = &shell * &gathered(w, reach, n);
        m.checked_div(&(w + &m)).unwrap_or(Q::ZERO)
    };
    // agitation shed per ring: e by the open frames' emission plus
    // ln 2 / k_v for the vacuum's halving every k_v rings
    let vacuum = vacuum_halving(reach, h);
    let ln2 = xyl_core::certified::ln2(64).mid();
    let shed = e + &vacuum
        .map(|k| ln2.checked_div(&Q::int(k)).unwrap_or(Q::ZERO))
        .unwrap_or(Q::ZERO);
    // the pulse's heat: 1 / (pi (R + 1)) times the swing and the whole
    // piece's grip, over the shed per ring, capped at 1; with no shedding
    // the heat is 1
    let ring = Q::ONE
        .checked_div(&(&xyl_core::certified::pi(64).mid() * &(reach + &Q::ONE)))
        .unwrap_or(Q::ZERO);
    let taken = &(&ring * swing) * &grip(width);
    let heat = grid(&if shed.signum() > 0 {
        taken.checked_div(&shed).unwrap_or(Q::ONE).min(Q::ONE)
    } else {
        Q::ONE
    });
    let none = (None, Q::ZERO, Q::ZERO, 0, Q::ZERO, heat.clone());
    // spare(w) = grip(w) - H w / (R (R + w)) - heat: the agitation an
    // infall of extent w can still hold against
    let spare = |w: &Q| {
        let shear = (h * w)
            .checked_div(&(reach * &(reach + w)))
            .unwrap_or(Q::ZERO);
        &(&grip(w) - &shear) - &heat
    };
    // the extents read: W, W/2, ..., W/2^47 (nonzero ones only)
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
    // the halvings needed for the residue to fall to the best spare; past
    // 4096 the piece is treated as never collapsing
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
    // the infall's extent: the largest on the ladder whose spare covers the
    // cooled residue
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
        &shell * &gathered(&infall, reach, n),
        clouds,
        a,
        heat,
    )
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
