//! Formation from the bounce (computed, not yet wired into the game's
//! stars): a galaxy's matter, sorted by the bounce, cooled and collapsed.
//!
//! In the galaxy's own units the core holds one ring as a state, its grains
//! together hold one ring (each nucleus its weight's share), and the core's
//! reach is the ring itself, d = 1 (the bounce on S^1, state spin 1 x 1).
//! Every hold is gravitational, normalised by the manifest's total weight.
//!
//! - The cut: the galaxy's core regenerates the push at full amplitude in its
//!   own units (s = 1, not a budget divided across tiers), p the throat's
//!   push, the vacuum's share at the throat (circuit::throat_push). A nucleus
//!   of weight w is carried to spin l = p (w_mean / w) and stands at
//!   R = l / H; it is held where the deficit there, H / (R + H), reaches p.
//!   Holding heavier nuclei raises H monotonically along the weight order, so
//!   the held set is everything above one weight, found by bisection. Gas
//!   inflow from the ocean is not yet modelled.
//! - The gas: the home galaxy is the cascade one tier down, so its gas is
//!   the beads, the pinches of the founding's runs: each bead's state over
//!   the founding unit is a piece's holding, spread over its span, its kinds
//!   at its pinch with the agitation a = (p - delta) / p the push leaves
//!   there. Outside the core's reach (one galaxy ring) the gas lies in the
//!   galaxy's plane, a circle; within it, the bulge's ball.
//! - Cooling: what a grain cannot hold radiates - an atom by its open
//!   frames, e = unpaired / weight per ring - held as halvings, the agitation
//!   halving every ceil(1 / e) rings.
//! - Collapse (`collapse`): an infall of extent w on a piece of hold
//!   (lambda + mu) W at reach R gathers the piece's measure within its own
//!   angle, m(w) = (lambda + mu) W share_n(theta), theta the deficit angle
//!   w / (w + R) of the half turn in turns (measure.rs: the circle's arc in
//!   the disc, the sphere's cap in the bulge; no exponent declared), and
//!   grips it at its own rim by the deficit law,
//!   m / (w + m). It holds where that grip exceeds its agitation (cooled,
//!   plus the heat the pulse leaves: the ring the bell brings at R,
//!   1 / (pi (R + 1)), swung by the inbound echo and taken by the piece's
//!   grip, over what it sheds per ring by its open frames and the vacuum)
//!   and the shear across it, H w / (R (R + w)). The extent is read on a
//!   ladder of halvings of the piece's width: the piece cools until its
//!   residue is under the best extent's spare, then fragments at the
//!   largest extent that holds into as many infalls as fit. A piece whose
//!   grip never exceeds the shear and the heat never collapses.
//! - Ignition and the star tier's cut: an infall ignites at its throat
//!   condition, its agitation the cooled residue plus the pulse's heat (no
//!   ignition where that reaches the throat's push, 1/2). Its core, pushing by
//!   electromagnetic repulsion on open frames, holds the star and carries the
//!   open-frame matter into the disc (`hold_at_heat`); the disc, heated by
//!   the tide the star leaves unlocked, fragments into worlds (`disc`).

use crate::circuit::Bead;
use crate::disc::{star_disc, World};
use crate::elements::Element;
use xyl_core::Q;

pub(crate) fn half(x: &Q) -> Q {
    x.checked_div(&Q::int(2)).unwrap_or(Q::ZERO)
}

/// The throat's push as an exact rational: the vacuum's share where reach
/// and hold coincide (circuit::throat_push).
pub(crate) fn throat_push() -> Q {
    crate::circuit::throat_push().to_q()
}

/// A nucleus's weight (hold) from the atom tier: 3Z - unpaired; for a
/// cluster, 3 charge - open from its parts (compound.rs).
pub fn weight(z: u32) -> Q {
    Q::int(i64::from(crate::compound::hold(z)))
}

/// A nucleus's mass in nucleons, Z + Q (weights.rs). Gravity and inertia use
/// this, independent of kind; chemistry uses the hold (`weight`, the
/// electrons' frames), which is also the unit amounts are counted in.
pub fn mass_weight(z: u32) -> Q {
    crate::circuit::nucleus_weights(z).mass()
}

/// What a nucleus radiates per ring: its open frames over its weight.
pub fn emission(z: u32) -> Q {
    Q::int(i64::from(crate::compound::open(z)))
        .checked_div(&weight(z))
        .unwrap_or(Q::ZERO)
}

/// One element's share of the galaxy, by count and by holding.
#[derive(Clone, Debug)]
pub struct Kind {
    /// Atomic number of the element.
    pub z: u32,
    /// Hold of one nucleus, 3Z - unpaired (see [`weight`]).
    pub weight: Q,
    /// Share of the galaxy's grains' holding (shares sum to 1).
    pub share: Q,
}

/// A piece of the gas, a bead read one tier down, and what it forms.
#[derive(Clone, Debug)]
pub struct Piece {
    /// Inner edge of the piece (galaxy rings): its bead's span.
    pub from: Q,
    /// Outer edge of the piece (galaxy rings).
    pub to: Q,
    /// The pinch the piece's kinds stand at (galaxy rings): its bead's reach.
    pub reach: Q,
    /// Holding per light-ring of reach.
    pub lambda: Q,
    /// Holding-weighted mean agitation of the shells covering the piece.
    pub agitation: Q,
    /// Holding-weighted mean emission of the shells covering the piece (per ring).
    pub emission: Q,
    /// Rings until it cools to collapse (None: it never does).
    pub cools_in: Option<i64>,
    /// Infall width (the smallest holding width, a cloud), each cloud's
    /// holding, and how many fit.
    pub infall: Q,
    /// Holding of each cloud: `lambda` times the infall width.
    pub cloud: Q,
    /// Number of clouds that fit in the piece (at least 1 once it collapses; 0 when it never does).
    pub clouds: i64,
    /// Agitation at collapse (cooled by both channels).
    pub cooled: Q,
    /// The galaxy's tide at the piece, delta_gal = H / (R + H): the vacuum's
    /// share there, the push the pulse delivers.
    pub tide: Q,
    /// The infall's agitation at ignition: cooled residue plus the pulse's
    /// heat, the ring the bell brings at the reach taken by the infall's
    /// grip over what it sheds per ring, swung by the inbound echo.
    pub heated: Q,
    /// The star tier's cut: the share of each infall its core holds (the
    /// star), and the open share of the infall and of its disc.
    pub star_share: Q,
    /// Open share of the infall's matter.
    pub infall_open: Q,
    /// Open share of the matter released to the disc.
    pub disc_open: Q,
    /// The disc's worlds, each with its planetary core's cut, and the belts'
    /// share.
    pub worlds: Vec<World>,
    /// Share of the disc left as belts rather than worlds.
    pub belts: Q,
    /// The stars the clouds make: each one's holding and the count (all
    /// clouds together).
    pub star: Q,
    /// Number of stars: the cloud count when a cloud's star holds anything, else 0.
    pub stars: Q,
}

/// A galaxy's formation.
#[derive(Clone, Debug)]
pub struct Formation {
    /// The push share at the cut, the throat's.
    pub push: Q,
    /// Held share of the founded matter: the firmament's over the whole.
    pub held: Q,
    /// The pieces of gas, the beads one tier down, in bead order.
    pub pieces: Vec<Piece>,
}

/// The kinds of matter in a galaxy's manifest, by count from the element
/// table, weighted by the atom tier, heaviest first.
pub fn kinds(elements: &[Element]) -> Vec<Kind> {
    let mut ks: Vec<(u32, Q, Q)> = elements
        .iter()
        .filter(|e| e.abundance.signum() > 0)
        .map(|e| (e.z, weight(e.z), e.abundance.clone()))
        .collect();
    let total = ks.iter().fold(Q::ZERO, |a, (_, w, n)| &a + &(w * n));
    ks.sort_by(|a, b| b.1.cmp(&a.1));
    ks.into_iter()
        .map(|(z, w, n)| Kind {
            z,
            share: (&w * &n).checked_div(&total).unwrap_or(Q::ZERO),
            weight: w,
        })
        .collect()
}

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
    // outside the core's reach (one galaxy ring) the gas lies in the
    // galaxy's plane, a circle; within it, the bulge's ball
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

/// The galaxy's formation from the founding's beads: each piece's cooling
/// and, where it lights, the star tier's cut on its infalls and their
/// discs. `unit`, `held` and `swing` as for [`lighting`].
pub fn of(beads: &[Bead], unit: &Q, held: &Q, swing: &Q) -> Formation {
    let h = &Q::ONE + held;
    let p_core = throat_push();
    let mut pieces = Vec::new();
    for bead in beads {
        let (from, to) = (bead.span.0.clone(), bead.span.1.clone());
        let Some(c) = cooling(bead, unit, &h, swing) else {
            continue;
        };
        let lit = lights(&c);
        let Cooling {
            mix,
            lambda,
            agitation,
            emission,
            cools_in,
            infall,
            cloud,
            clouds,
            cooled,
            tide,
            heated,
        } = c;
        // the core regenerates its push at full amplitude in its own units;
        // the tide enters only as heat
        let cut = if lit {
            Some(hold_at_heat(&mix, &p_core, &Q::ZERO, &Q::ZERO, &Q::ONE))
        } else {
            None
        };
        let (star_share, infall_open, disc_open) = cut
            .as_ref()
            .map(|c| (c.held.clone(), c.open.clone(), c.rest_open.clone()))
            .unwrap_or((Q::ZERO, Q::ZERO, Q::ZERO));
        // the disc, heated by the tide the star leaves unlocked, with each
        // world cut by its planetary core
        let (worlds, belts) = match &cut {
            Some(c) if star_share < Q::ONE => star_disc(&star_share, &tide, &c.rest),
            _ => (Vec::new(), Q::ZERO),
        };
        let star = &cloud * &star_share;
        let stars = if star.signum() > 0 {
            Q::int(clouds)
        } else {
            Q::ZERO
        };
        pieces.push(Piece {
            from,
            to,
            reach: bead.reach.clone(),
            lambda,
            agitation,
            emission,
            cools_in,
            infall,
            cloud,
            clouds,
            cooled,
            tide,
            heated,
            star_share,
            infall_open,
            disc_open,
            worlds,
            belts,
            star,
            stars,
        });
    }
    Formation {
        push: p_core,
        held: held.clone(),
        pieces,
    }
}

/// One stack of the founding cascade.
#[derive(Clone, Debug)]
pub struct Stack {
    /// The stack's kinds, heaviest first.
    pub kinds: Vec<u32>,
    /// Amount (share of the founding) and mean hold (weight).
    pub amount: Q,
    /// Mean hold (weight) of the stack's kinds.
    pub w_mean: Q,
    /// The bounce that released it (its push share) and its spin at release.
    pub bounce: Q,
    /// Spin at release, before its passages.
    pub released: Q,
    /// Spin once the cascade settles (after every passage through it).
    pub settled: Q,
    /// The beat set by its release: the round trip to it, the nearest stack.
    pub beat: Q,
}

/// The founding cascade (diagnostic): the stack built by the capture chain,
/// emptied from the top by bursts.
/// - Stacks are the beads of the weight profile (each kind's amount x hold,
///   heaviest first): a bead at every local maximum, parted at the minima.
/// - Bounce k releases stack k at its push b_k, carried to
///   l = b_k w_mean / w_k (the whole stack's mean over its own), reach R = l
///   (the throat's state 1).
/// - Once released, it passes out through every earlier stack still inside
///   its reach, innermost first. Each passage is a meeting: the contrast T
///   of their deficits (delta = 1 / (R + 1)), with no grip applied, moves
///   turning outward as a conserved spin exchange (the outer gains
///   T m_inner / M of their spin contrast, the inner gives the mirror; never
///   more than it carries).
/// - What stack k loses in its passages it hands back at the throat:
///   b_(k+1) = p + that loss, so the next bounce is stronger.
/// - The beat: each release's pulse echoes off the nearest stack, a round
///   trip 2 R_k; with none left the throat echoes at 2 (its own reach, 1).
///
/// Returns the stacks and the settled beat.
pub fn cascade(mix: &[(u32, Q)]) -> (Vec<Stack>, Q) {
    let p = half(&Q::ONE);
    // inertia by mass
    let mut kinds: Vec<(u32, Q, Q)> = mix
        .iter()
        .map(|(z, a)| (*z, a.clone(), mass_weight(*z)))
        .filter(|(_, a, w)| a.signum() > 0 && w.signum() > 0)
        .collect();
    kinds.sort_by(|a, b| b.2.cmp(&a.2));
    if kinds.is_empty() {
        return (Vec::new(), Q::int(2));
    }
    let amount = kinds.iter().fold(Q::ZERO, |acc, (_, a, _)| &acc + a);
    let total_w = kinds.iter().fold(Q::ZERO, |acc, (_, a, w)| &acc + &(a * w));
    let w_all = total_w.checked_div(&amount).unwrap_or(Q::ONE);
    // the beads of the weight profile
    let prof: Vec<Q> = kinds.iter().map(|(_, a, w)| a * w).collect();
    let mut cuts: Vec<usize> = Vec::new();
    for k in 1..prof.len().saturating_sub(1) {
        if prof[k] < prof[k - 1] && prof[k] <= prof[k + 1] {
            cuts.push(k);
        }
    }
    let mut segs: Vec<(usize, usize)> = Vec::new();
    let mut start = 0;
    for &k in &cuts {
        segs.push((start, k));
        start = k;
    }
    segs.push((start, kinds.len()));
    let down = |x: &Q| xyl_core::certified::down(x, 64);
    let mut stacks: Vec<Stack> = Vec::new();
    let mut b = p.clone();
    for (a0, a1) in segs {
        let part = &kinds[a0..a1];
        let m = part.iter().fold(Q::ZERO, |acc, (_, a, _)| &acc + a);
        let mw = part.iter().fold(Q::ZERO, |acc, (_, a, w)| &acc + &(a * w));
        let w_mean = mw.checked_div(&m).unwrap_or(Q::ONE);
        let l0 = down(&(&b * &w_all).checked_div(&w_mean).unwrap_or(Q::ZERO));
        let mut l = l0.clone();
        let mut lost = Q::ZERO;
        // its passage out through every earlier stack it overtakes, innermost first
        let mut order: Vec<usize> = (0..stacks.len()).collect();
        order.sort_by(|&x, &y| stacks[x].settled.cmp(&stacks[y].settled));
        for j in order {
            if stacks[j].settled >= l {
                continue;
            }
            let (r_in, r_out) = (stacks[j].settled.clone(), l.clone());
            let (d_in, d_out) = (
                Q::ONE.checked_div(&(&r_in + &Q::ONE)).unwrap_or(Q::ZERO),
                Q::ONE.checked_div(&(&r_out + &Q::ONE)).unwrap_or(Q::ZERO),
            );
            let t = (&d_in - &d_out)
                .abs()
                .checked_div(&(&d_in + &d_out))
                .unwrap_or(Q::ZERO);
            let both = &m + &stacks[j].amount;
            let spread = &l - &stacks[j].settled;
            // the faster (this stack) gives; the slower it overtakes gains
            let give = down(
                &(&(&t * &stacks[j].amount) * &spread)
                    .checked_div(&both)
                    .unwrap_or(Q::ZERO),
            )
            .min(l.clone());
            let gain = down(&(&(&t * &m) * &spread).checked_div(&both).unwrap_or(Q::ZERO));
            l = &l - &give;
            lost = &lost + &give;
            stacks[j].settled = &stacks[j].settled + &gain;
        }
        stacks.push(Stack {
            kinds: part.iter().map(|(z, _, _)| *z).collect(),
            amount: m.clone(),
            w_mean,
            bounce: b.clone(),
            released: l0.clone(),
            settled: l.clone(),
            beat: &Q::int(2) * &l0,
        });
        // the next bounce, stronger by what this stack handed back at the throat
        b = &p + &lost;
    }
    (stacks, Q::int(2))
}

/// The equitable cascade (diagnostic): bursts from the top and expansion
/// from the base combined; K is the epicentre's spin, k the orbital speed,
/// and each share is the unlocked face R / (R + H) at its own event. Holds,
/// per form, the spin K, the throat's final share, the stacks (kinds, push,
/// reaches) and the kinds the throat holds.
#[derive(Clone, Debug)]
pub struct EquitableReading {
    /// Label of the form of the cascade read.
    pub form: &'static str,
    /// Epicentre spin K: the sum of every kind's capture share.
    pub k_spin: Q,
    /// The throat's final hold, H = 1 + the held share.
    pub h_final: Q,
    /// The throat's final share, 1/2 R / (R + H) at the edge; each founding shell
    /// spreads R (1 +/- s_final).
    pub s_final: Q,
    /// Per stack: kinds (heaviest first), push K + s_i, reaches (lightest
    /// kind's, heaviest kind's), and the orbital speed at the heaviest.
    pub stacks: Vec<(Vec<u32>, Q, Q, Q, Q)>,
    /// The kinds the throat holds: those whose reach times (1 + s_final) is at most 1.
    pub held: Vec<u32>,
}

/// Runs the equitable cascade (diagnostic) on `mix`, a list of (kind, amount)
/// pairs, and returns one [`EquitableReading`] per form. Kinds with zero amount
/// or zero mass are ignored; an empty result means no kind remained.
pub fn equitable_cascade(mix: &[(u32, Q)]) -> Vec<EquitableReading> {
    let down = |x: &Q| xyl_core::certified::down(x, 64);
    // inertia by mass
    let mut kinds: Vec<(u32, Q, Q)> = mix
        .iter()
        .map(|(z, a)| (*z, a.clone(), mass_weight(*z)))
        .filter(|(_, a, w)| a.signum() > 0 && w.signum() > 0)
        .collect();
    if kinds.is_empty() {
        return Vec::new();
    }
    let amount = kinds.iter().fold(Q::ZERO, |acc, (_, a, _)| &acc + a);
    let total_w = kinds.iter().fold(Q::ZERO, |acc, (_, a, w)| &acc + &(a * w));
    let w_all = total_w.checked_div(&amount).unwrap_or(Q::ONE);
    // the chain builds light to heavy (Z ascending), one share per capture
    kinds.sort_by_key(|k| k.0);
    let mut out = Vec::new();
    // one form: each capture's share is the unlocked face, the half of the
    // sphere the throat does not hold, 1/2 (1 - delta) = 1/2 R / (R + H), with
    // R = held (the stack's depth) and H = 1 + held; the 1/2 is a hemisphere
    for form in ["the unlocked face, R = held"] {
        let mut held = Q::ZERO;
        let mut shares: std::collections::BTreeMap<u32, Q> = std::collections::BTreeMap::new();
        for (z, a, w) in &kinds {
            let frac = a.checked_div(&amount).unwrap_or(Q::ZERO);
            let h = &Q::ONE + &held;
            let r = held.clone();
            let _ = w;
            let s = down(&half(&r.checked_div(&(&r + &h)).unwrap_or(Q::ZERO)));
            shares.insert(*z, s.clone());
            held = down(&(&held + &frac));
        }
        let k_spin = shares.values().fold(Q::ZERO, |acc, s| &acc + s);
        let h_final = &Q::ONE + &held;
        let r_edge = held.clone();
        let s_final = down(&half(
            &r_edge.checked_div(&(&r_edge + &h_final)).unwrap_or(Q::ZERO),
        ));
        // the stacks: beads of the weight profile, heaviest first
        let mut heavy = kinds.clone();
        heavy.sort_by(|a, b| b.2.cmp(&a.2));
        let prof: Vec<Q> = heavy.iter().map(|(_, a, w)| a * w).collect();
        let mut cuts = Vec::new();
        for k in 1..prof.len().saturating_sub(1) {
            if prof[k] < prof[k - 1] && prof[k] <= prof[k + 1] {
                cuts.push(k);
            }
        }
        let mut segs = Vec::new();
        let mut start = 0;
        for &k in &cuts {
            segs.push((start, k));
            start = k;
        }
        segs.push((start, heavy.len()));
        let mut stacks = Vec::new();
        let mut held_z = Vec::new();
        for (a0, a1) in segs {
            let part = &heavy[a0..a1];
            let s_i = part.iter().fold(Q::ZERO, |acc, (z, _, _)| {
                &acc + shares.get(z).unwrap_or(&Q::ZERO)
            });
            let push = &k_spin + &s_i;
            // the carriage: the push depends on the element only through the
            // bounce force and its inertia, (K + s_i) / w / H, with no w_bar
            // factor (as in founding_shells)
            let _ = &w_all;
            let reach = |w: &Q| {
                down(
                    &push
                        .checked_div(w)
                        .unwrap_or(Q::ZERO)
                        .checked_div(&h_final)
                        .unwrap_or(Q::ZERO),
                )
            };
            let (w_light, w_heavy) = (
                part.iter().map(|k| k.2.clone()).min().unwrap_or(Q::ONE),
                part.iter().map(|k| k.2.clone()).max().unwrap_or(Q::ONE),
            );
            let (r_far, r_deep) = (reach(&w_light), reach(&w_heavy));
            let k_orbit = down(
                &h_final
                    .checked_div(&(&r_deep + &h_final))
                    .unwrap_or(Q::ZERO),
            );
            for (z, _, w) in part {
                if &reach(w) * &(&Q::ONE + &s_final) <= Q::ONE {
                    held_z.push(*z);
                }
            }
            stacks.push((
                part.iter().map(|k| k.0).collect(),
                push,
                r_far,
                r_deep,
                k_orbit,
            ));
        }
        out.push(EquitableReading {
            form,
            k_spin,
            h_final,
            s_final,
            stacks,
            held: held_z,
        });
    }
    out
}

/// The founding shells (the equitable cascade): each kind's reach about the
/// Wellspring, R_z = (K + s_i) / w_z / H, after the filter below, and whether
/// the Wellspring holds it (R_z (1 + s_final) <= 1). Returns the final share
/// s_final (each shell spreads R (1 +/- s_final)) and, per kind,
/// (z, amount, reach, held).
pub fn founding_shells(mix: &[(u32, Q)]) -> (Q, Vec<(u32, Q, Q, bool)>) {
    let down = |x: &Q| xyl_core::certified::down(x, 64);
    // the founding by mass: each kind's amount converted to mass (held frames
    // x mass / hold); capture shares, the profile's beads and the carriage's
    // inertia all use mass; amounts are returned in held frames
    let to_mass = |z: u32| mass_weight(z).checked_div(&weight(z)).unwrap_or(Q::ONE);
    let mut kinds: Vec<(u32, Q, Q)> = mix
        .iter()
        .map(|(z, a)| (*z, a * &to_mass(*z), mass_weight(*z)))
        .filter(|(_, a, w)| a.signum() > 0 && w.signum() > 0)
        .collect();
    if kinds.is_empty() {
        return (Q::ZERO, Vec::new());
    }
    let amount = kinds.iter().fold(Q::ZERO, |acc, (_, a, _)| &acc + a);
    kinds.sort_by_key(|k| k.0);
    let mut held = Q::ZERO;
    let mut shares: std::collections::BTreeMap<u32, Q> = std::collections::BTreeMap::new();
    for (z, a, _) in &kinds {
        let h = &Q::ONE + &held;
        let s = down(&half(&held.checked_div(&(&held + &h)).unwrap_or(Q::ZERO)));
        shares.insert(*z, s);
        held = down(&(&held + &a.checked_div(&amount).unwrap_or(Q::ZERO)));
    }
    let k_spin = shares.values().fold(Q::ZERO, |acc, s| &acc + s);
    let h_final = &Q::ONE + &held;
    let s_final = down(&half(
        &held.checked_div(&(&held + &h_final)).unwrap_or(Q::ZERO),
    ));
    // the stacks (beads of the mass profile, heaviest first) give each kind its s_i
    let mut heavy = kinds.clone();
    heavy.sort_by(|a, b| b.2.cmp(&a.2));
    // the mass profile (each kind's mass, its amount already weighed)
    let prof: Vec<Q> = heavy.iter().map(|(_, a, _)| a.clone()).collect();
    let mut cuts = Vec::new();
    for k in 1..prof.len().saturating_sub(1) {
        if prof[k] < prof[k - 1] && prof[k] <= prof[k + 1] {
            cuts.push(k);
        }
    }
    let mut segs = Vec::new();
    let mut start = 0;
    for &k in &cuts {
        segs.push((start, k));
        start = k;
    }
    segs.push((start, heavy.len()));
    // each kind's shell: (z, amount, reach), released heaviest first
    let mut shells: Vec<(u32, Q, Q)> = Vec::new();
    for (a0, a1) in segs {
        let part = &heavy[a0..a1];
        let s_i = part.iter().fold(Q::ZERO, |acc, (z, _, _)| {
            &acc + shares.get(z).unwrap_or(&Q::ZERO)
        });
        let push = &k_spin + &s_i;
        for (z, a, w) in part {
            // the force comes from the cascade alone; each kind's carriage is
            // its own inertia (not relative to the others)
            let r = down(
                &push
                    .checked_div(w)
                    .unwrap_or(Q::ZERO)
                    .checked_div(&h_final)
                    .unwrap_or(Q::ZERO),
            );
            shells.push((*z, a.clone(), r));
        }
    }
    // the filter: each lighter shell, released later and pushed harder,
    // passes out through every heavier shell whose spread lies between the
    // throat and its reach (a meeting). Contact, for isotropic shells: a whole
    // passage through the other's spread is 1/2, a partial one (ending inside
    // it) 1/2 its share of that spread. The grip split: s the two kinds' grip
    // (tightness ratio), D = M / (M + 1 - contact), T the contrast of their
    // deficits at their reaches; the gripped fraction g = min(s D, T) / T is
    // caught: the heavier gives contact g m_i / (m_i + m_j) of its matter to
    // the lighter, carried to the lighter's reach, and the lighter pays the
    // drag (sum m l conserved, l = R H).
    // All passages are evaluated at once against the release state, so the
    // result does not depend on order: where lighter shells ask more of a
    // kind than a shell holds, every take is scaled by holding over asked, so
    // none gives more than it has; each shell's reach is then one
    // holding-weighted mean of its parts' release reaches (sum m l unchanged).
    // Caught matter is not caught again in the same founding.
    let tight = |z: u32| crate::compound::tightness(z);
    let nucleus = |x: &Q| xyl_core::certified::down(x, 232);
    let released = shells;
    // each released shell read once: its share, grip, deficit and spread
    let t_rel: Vec<Q> = released.iter().map(|(z, _, _)| tight(*z)).collect();
    let d_rel: Vec<Q> = released
        .iter()
        .map(|(_, _, r)| h_final.checked_div(&(r + &h_final)).unwrap_or(Q::ZERO))
        .collect();
    let band: Vec<(Q, Q)> = released
        .iter()
        .map(|(_, _, r)| (r * &(&Q::ONE - &s_final), r * &(&Q::ONE + &s_final)))
        .collect();
    // (i catching, j giving, its take on the nucleus grain): every passage is
    // evaluated against the release state alone, so each lighter shell's
    // meetings are computed independently and in parallel
    let trace: Option<u32> = std::env::var("XYL_FILTER_TRACE")
        .ok()
        .and_then(|v| v.parse::<u32>().ok());
    let meetings = |i: usize, zi: &u32, mi: &Q, ri: &Q| -> Vec<(usize, usize, Q)> {
        let mut mine = Vec::new();
        for j in 0..i {
            let (mj, rj) = (&released[j].1, &released[j].2);
            let (lo_j, hi_j) = (&band[j].0, &band[j].1);
            if mj.signum() <= 0 || mi.signum() <= 0 || rj >= ri || ri <= lo_j {
                continue;
            }
            let near = if ri >= hi_j {
                half(&Q::ONE)
            } else {
                half(
                    &(ri - lo_j)
                        .checked_div(&(hi_j - lo_j))
                        .unwrap_or(Q::ZERO)
                        .min(Q::ONE),
                )
            };
            let (ti, tj) = (&t_rel[i], &t_rel[j]);
            let grip_s = ti
                .clone()
                .min(tj.clone())
                .checked_div(&ti.clone().max(tj.clone()))
                .unwrap_or(Q::ZERO);
            let both = mi + mj;
            let d = both
                .checked_div(&(&both + &(&Q::ONE - &near)))
                .unwrap_or(Q::ZERO);
            let (di, dj) = (&d_rel[i], &d_rel[j]);
            let t = (dj - di).abs().checked_div(&(dj + di)).unwrap_or(Q::ZERO);
            if t.signum() <= 0 {
                continue;
            }
            let g = (&grip_s * &d)
                .min(t.clone())
                .checked_div(&t)
                .unwrap_or(Q::ONE);
            let frac = (&(&near * &g) * mi)
                .checked_div(&both)
                .unwrap_or(Q::ZERO)
                .min(Q::ONE);
            // diagnostic trace (XYL_FILTER_TRACE=z_i): prints every term of one
            // lighter kind's meetings; no effect on results
            if trace == Some(*zi) {
                eprintln!("FILTER z_i {} meets Z{}: m_i {} m_j {} r_i {} r_j {} near {} s {} D {} T {} g {} frac {}", zi, released[j].0, mi, mj, ri, rj, near, grip_s, d, t, g, frac);
            }
            let take = nucleus(&(mj * &frac)).min(mj.clone());
            if take.signum() > 0 {
                mine.push((i, j, take));
            }
        }
        mine
    };
    use rayon::prelude::*;
    let takes: Vec<(usize, usize, Q)> = released
        .par_iter()
        .enumerate()
        .flat_map_iter(|(i, (zi, mi, ri))| meetings(i, zi, mi, ri))
        .collect();
    let mut asked: Vec<Q> = vec![Q::ZERO; released.len()];
    for (_, j, take) in &takes {
        asked[*j] = &asked[*j] + take;
    }
    // each shell's parts: its own kept, and every take at its giver's release reach
    let mut given: Vec<Q> = vec![Q::ZERO; released.len()];
    let mut parts: Vec<Vec<(u32, Q, Q)>> = vec![Vec::new(); released.len()];
    for (i, j, take) in takes {
        let (has, all) = (&released[j].1, &asked[j]);
        let take = if all > has {
            nucleus(&(&take * has).checked_div(all).unwrap_or(Q::ZERO))
        } else {
            take
        };
        if take.signum() <= 0 {
            continue;
        }
        given[j] = &given[j] + &take;
        parts[i].push((released[j].0, take, released[j].2.clone()));
    }
    let placed: Vec<(std::collections::BTreeMap<u32, Q>, Q)> = released
        .iter()
        .enumerate()
        .map(|(i, (z, m, r))| {
            let own = m - &given[i];
            let mut mix: std::collections::BTreeMap<u32, Q> = std::collections::BTreeMap::new();
            mix.insert(*z, own.clone());
            let (mut mass, mut ml) = (own.clone(), &own * r);
            for (zj, a, rj) in &parts[i] {
                let e = mix.entry(*zj).or_insert(Q::ZERO);
                *e = &*e + a;
                mass = &mass + a;
                ml = &ml + &(a * rj);
            }
            (
                mix,
                down(&ml.checked_div(&mass).unwrap_or_else(|| r.clone())),
            )
        })
        .collect();
    let out = placed
        .into_iter()
        .flat_map(|(mix, r)| {
            let is_held = &r * &(&Q::ONE + &s_final) <= Q::ONE;
            mix.into_iter()
                .filter(|(_, a)| a.signum() > 0)
                .map(move |(z, a)| {
                    (
                        z,
                        a.checked_div(&mass_weight(z).checked_div(&weight(z)).unwrap_or(Q::ONE))
                            .unwrap_or(Q::ZERO),
                        r.clone(),
                        is_held,
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    (s_final, out)
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
    // the piece's whole hold, and an infall's grip at its own rim
    let shell = &(lambda + mu) * width;
    let grip = |w: &Q| {
        let m = &shell * &gathered(w, reach, n);
        m.checked_div(&(w + &m)).unwrap_or(Q::ZERO)
    };
    // the two cooling channels per ring: open frames radiate e of the
    // agitation, and the vacuum takes back a halving every k_v rings
    let vacuum = vacuum_halving(reach, h);
    let ln2 = xyl_core::certified::ln2(64).mid();
    let shed = e + &vacuum
        .map(|k| ln2.checked_div(&Q::int(k)).unwrap_or(Q::ZERO))
        .unwrap_or(Q::ZERO);
    // the pulse's heat: the ring the bell brings at the reach, swung by the
    // inbound echo and taken by the whole piece's grip, over what the piece
    // sheds per ring; a piece that sheds nothing holds the whole ring
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
    // what an infall of extent w can hold against, beyond its agitation
    let spare = |w: &Q| {
        let shear = (h * w)
            .checked_div(&(reach * &(reach + w)))
            .unwrap_or(Q::ZERO);
        &(&grip(w) - &shear) - &heat
    };
    // the extents read: the width and its halvings
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
    // whole halvings until the cooled residue no longer exceeds the best
    // extent's spare
    let mut halvings = 0i64;
    let mut a = a0.clone();
    while a > critical {
        a = half(&a);
        halvings += 1;
        if halvings > 4096 {
            return none;
        }
    }
    // by ring n the piece has halved floor(n / P_e) + floor(n / k_v) times
    // (P_e = ceil(1 / e) rings per radiative halving); it collapses at the
    // first ring reaching the halvings it needs
    let emission = Q::ONE
        .checked_div(e)
        .map(|x| -((-x).floor_i128().unwrap_or(-1)))
        .filter(|&p| p >= 1);
    let cools_in = match (emission, vacuum) {
        _ if halvings == 0 => 0,
        (None, None) => return none,
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
    };
    // the infall's extent: the largest on the ladder that holds against the
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

/// What a core (a star's or a planetary core) holds at its heat: the share
/// of its matter held, and the released remainder.
///
/// The star tier's cut: the core has no bounce, only torsion; its push is
/// the electromagnetic repulsion on open frames. Each kind of the infall
/// (`mix`: element and share of the infall's holding) is carried to spin
/// l_z = p (e_z / e_mean)(w_mean / w_z) + `add` - p the core's push (1/2),
/// e_z its open share, means by count, `add` a turning the matter brings (a
/// world's heat) - and stands at R_z = l_z / H about the core's hold. It is
/// held (the star) where H / (R_z + H) >= p, else carried into the disc.
/// Noble kinds (e = 0) carry no spin and fall in; an infall with no open
/// frames is all star.
#[derive(Clone, Debug)]
pub struct Hold {
    /// The share held.
    pub held: Q,
    /// The matter's mean open share, and the released part's.
    pub open: Q,
    /// Mean open share of the released part.
    pub rest_open: Q,
    /// The released part's mix (element, share of it).
    pub rest: Vec<(u32, Q)>,
    /// The lightest kind held.
    pub lightest: Option<u32>,
    /// Each released kind's carried spin (aligned with `rest`).
    pub rest_l: Vec<Q>,
    /// The held core's extent: the reach of the loosest kind still held,
    /// R = l / H* on the held hold, in the core's own units. A core holding
    /// everything is large; one that has released its light kinds is compact.
    pub reach: Q,
}

/// What a core holds at its heat. Each kind's open share is taken after the
/// heat strips its loosely held windings (atoms::unpaired_hot; a cold core
/// passes heat 0), so the push grips only what stays open. The core's light
/// adds over-pressure `p0` = P_0 = 1 + PHI / H (1 for a dark core), carrying
/// every kind out by P_0 times its push while the held threshold stays p: a
/// dark core holds its whole infall and a burning one releases a tube, the
/// lightest first.
pub fn hold_at_heat(mix: &[(u32, Q)], p: &Q, add: &Q, heat: &Q, p0: &Q) -> Hold {
    hold_about(mix, p, add, &Q::ZERO, heat, p0)
}

fn hold_about(mix: &[(u32, Q)], p: &Q, add: &Q, state: &Q, heat: &Q, p0: &Q) -> Hold {
    let open_of = |z: u32| {
        Q::int(i64::from(crate::compound::open_hot(z, heat)))
            .checked_div(&weight(z))
            .unwrap_or(Q::ZERO)
    };
    let kinds: Vec<(u32, Q, Q, Q)> = mix
        .iter()
        .map(|(z, share)| (*z, share.clone(), weight(*z), open_of(*z)))
        .filter(|(_, _, w, _)| w.signum() > 0)
        .collect();
    let all = |open: Q| Hold {
        held: Q::ONE,
        open,
        rest_open: Q::ZERO,
        rest: Vec::new(),
        lightest: kinds.iter().min_by(|a, b| a.2.cmp(&b.2)).map(|k| k.0),
        rest_l: Vec::new(),
        reach: Q::ZERO,
    };
    let count = kinds.iter().fold(Q::ZERO, |a, (_, s, w, _)| {
        &a + &s.checked_div(w).unwrap_or(Q::ZERO)
    });
    if count.signum() <= 0 {
        return all(Q::ZERO);
    }
    // the mean mass per nucleus (the carriage's inertia): nuclei counted on
    // their holds, each weighed by mass
    let w_mean = kinds
        .iter()
        .fold(Q::ZERO, |a, (z, sh, w, _)| {
            &a + &(&sh.checked_div(w).unwrap_or(Q::ZERO) * &mass_weight(*z))
        })
        .checked_div(&count)
        .unwrap_or(Q::ONE);
    let e_mean = kinds
        .iter()
        .fold(Q::ZERO, |a, (_, s, w, e)| {
            &a + &(&s.checked_div(w).unwrap_or(Q::ZERO) * e)
        })
        .checked_div(&count)
        .unwrap_or(Q::ZERO);
    if e_mean.signum() <= 0 && add.signum() <= 0 {
        return all(Q::ZERO);
    }
    // each kind's carried spin: the push on its open frames, plus the turning it brings
    let mut carried: Vec<(Q, Q, Q, u32, Q)> = kinds
        .iter()
        .map(|(z, s, w, e)| {
            let pushed = &(&(p * p0) * &e.checked_div(&e_mean).unwrap_or(Q::ZERO))
                * &w_mean.checked_div(&mass_weight(*z)).unwrap_or(Q::ZERO);
            (&pushed + add, s.clone(), e.clone(), *z, w.clone())
        })
        .collect();
    carried.sort_by(|a, b| a.0.cmp(&b.0));
    let cum: Vec<Q> = carried
        .iter()
        .scan(Q::ZERO, |a, k| {
            *a = &*a + &k.1;
            Some(a.clone())
        })
        .collect();
    // a core holds only by what it has gathered (only a throat has a state of
    // its own; callers pass `state`). The held set is order-independent: a kind
    // is held where its reach about the whole held hold lies within it,
    // h / (R + h) >= p, R = l / h. The hold is the largest fixed point
    // h* = state + the shares of every kind so held, found from "all held"
    // downward (each step can only drop kinds, so it ends within as many
    // steps as kinds)
    let _ = &cum;
    // held where the deficit at its reach R = l / h, on the held hold, still
    // meets the push: h / (R + h) >= p, compared directly (no square of the
    // hold)
    let holds = |l: &Q, h: &Q| {
        let r = l.checked_div(h).unwrap_or(Q::ZERO);
        h.checked_div(&(&r + h)).unwrap_or(Q::ZERO) >= *p
    };
    let total_s = carried.iter().fold(Q::ZERO, |a, k| &a + &k.1);
    let mut h = state + &total_s;
    let mut lo = carried.len();
    for _ in 0..=carried.len() {
        let n_held = carried.iter().take_while(|k| holds(&k.0, &h)).count();
        let h_next = state + &carried[..n_held].iter().fold(Q::ZERO, |a, k| &a + &k.1);
        lo = n_held;
        if h_next == h {
            break;
        }
        h = h_next;
    }
    if h.signum() <= 0 {
        lo = 0;
    }
    let held = if lo == 0 {
        Q::ZERO
    } else {
        cum[lo - 1].clone()
    };
    let rest_share = &Q::ONE - &held;
    let rest_open = carried[lo..]
        .iter()
        .fold(Q::ZERO, |a, (_, s, e, _, _)| &a + &(s * e))
        .checked_div(&rest_share)
        .unwrap_or(Q::ZERO);
    let rest = carried[lo..]
        .iter()
        .map(|(_, s, _, z, _)| (*z, s.checked_div(&rest_share).unwrap_or(Q::ZERO)))
        .collect();
    let lightest = carried[..lo]
        .iter()
        .min_by(|a, b| a.4.cmp(&b.4))
        .map(|k| k.3);
    let rest_l = carried[lo..]
        .iter()
        .map(|(l, _, _, _, _)| l.clone())
        .collect();
    let reach = carried[..lo]
        .iter()
        .filter_map(|k| k.0.checked_div(&h))
        .max()
        .unwrap_or(Q::ZERO);
    Hold {
        held,
        open: e_mean,
        rest_open,
        rest,
        lightest,
        rest_l,
        reach,
    }
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
mod tests {
    use super::*;

    #[test]
    fn the_pieces_hold_the_galaxy_gas_whole() {
        let bang = crate::genesis::bang();
        let s = crate::cores::substrate(7, bang).unwrap();
        let f = of(&s.beads, &s.unit(), &s.held_share(), &Q::ONE);
        // the pieces' holdings are the beads' states over the unit: the
        // galaxy's gas whole; the held share and the push are shares
        let gas = f
            .pieces
            .iter()
            .fold(Q::ZERO, |a, p| &a + &(&p.lambda * &(&p.to - &p.from)));
        assert!(
            (&gas - &Q::ONE).abs() < Q::new(1, 1_000_000).unwrap(),
            "the gas {gas}"
        );
        assert!(f.held >= Q::ZERO && f.held <= Q::ONE && f.push < Q::ONE);
        let l = &s.lighting;
        assert!(l.lit >= Q::ZERO && l.lit <= Q::ONE, "lit {}", l.lit);
        assert_eq!(l.pieces.len(), f.pieces.len());
        // a piece that never lights has no first light; one that does, a
        // whole count of rings
        for (p, piece) in l.pieces.iter().zip(&f.pieces) {
            assert_eq!(p.2.is_some(), piece.clouds > 0);
        }
    }
}
