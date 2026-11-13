//! A coordinate's motions (FORMS-COORDINATE v4, 78.173 section 1): the
//! advance, which composes; an event's bell, solved in first degree; a
//! merger with offsets and its inverse, fission; and the re-origin across a
//! throat. Every step is exact.
//!
//! The passage itself (FORMS-PASSAGE v4, LAW-PASSAGE v3, 78.172) is stated
//! in the algebra on a pole plane ([`PolePlane`]): the plane's units `e+`
//! (squaring to +1) and `e-` (to -1) carry a throat state `H n_Z + R n_Q`
//! with the poles `n_Z = e+ + e-`, `n_Q = e+ - e-`, which is
//! `(H + R) e+ + (H - R) e-`, the whole and the span. A passage is the
//! reflection in `e+`, `x -> e+ x e+`: the span reverses, the whole is
//! kept, hold and reach exchange (the face goes to `1 - delta`). It writes
//! `T -> T - 2 (H - R)` with the span before; a riding state passes
//! one-sided, `q -> e+ q`, flipping its parity. Two passes are the
//! identity, since `e+ e+ = 1`. [`Coordinate::pass`] is the same law read
//! on a body's lanes.

use super::body::{Coordinate, Local, Origin};
use super::keys::{Key, Lane, Lanes};
use super::CoordinateRefusal;
use crate::cga::Mv;
use crate::Q;

impl Coordinate {
    /// The body advanced to bell `t` under `rates` (keyed as the body's
    /// lanes, H's in degrees per bell): `x(t) = x(h) + r (t - h)` per
    /// component, at the same origin, `K` re-read per H component. Refused
    /// `Backward` for `t < h` and `ShapesDiffer` for rates keyed otherwise.
    pub fn advance(&self, rates: &Lanes, t: u64) -> Result<Coordinate, CoordinateRefusal> {
        let h = self.local.h;
        if t < h {
            return Err(CoordinateRefusal::Backward { held: h, asked: t });
        }
        self.local.lanes.shape_against(rates)?;
        let span = &super::bell_q(t) - &super::bell_q(h);
        let lanes = self
            .local
            .lanes
            .try_map(|l, k, x| Ok(x + &(rates.component(l, k)? * &span)))?;
        Coordinate::of(self.origin.clone(), t, lanes)
    }

    /// The bell at which component `key` of `lane` reaches `value` under
    /// `rates`: `h + (value - x) / r`, exact, possibly between whole bells.
    /// Refused `NoComponent` where the body or the rates hold no such
    /// component.
    pub fn event_bell(
        &self,
        rates: &Lanes,
        lane: &Lane,
        key: &Key,
        value: &Q,
    ) -> Result<EventBell, CoordinateRefusal> {
        let missing = || CoordinateRefusal::NoComponent {
            lane: lane.clone(),
            key: key.written(),
        };
        let x = self.component(lane, key).ok_or_else(missing)?;
        let r = rates
            .get(lane)
            .and_then(|v| v.get(key))
            .ok_or_else(missing)?;
        if r.signum() == 0 {
            return Ok(EventBell::Unmoving);
        }
        let after = (value - x).checked_div(r).ok_or_else(missing)?;
        let bell = &super::bell_q(self.local.h) + &after;
        Ok(if after.signum() < 0 {
            EventBell::Behind(bell)
        } else {
            EventBell::At(bell)
        })
    }

    /// Two bodies merged with holdings `wa` and `wb`, per component
    /// `x = (wa xa + wb xb) / (wa + wb)` at their origin and bell, `K` re-read,
    /// with each body's offset `x_a - x`, `x_b - x`, whose holding-weighted
    /// sum is zero. Refused `OriginsDiffer`, `BellsDiffer`, `ShapesDiffer`,
    /// and `NoHolding` where `wa + wb = 0`.
    pub fn merge(&self, wa: &Q, other: &Coordinate, wb: &Q) -> Result<Merger, CoordinateRefusal> {
        if self.origin != other.origin {
            return Err(CoordinateRefusal::OriginsDiffer {
                held: self.origin.written(),
                asked: other.origin.written(),
            });
        }
        if self.local.h != other.local.h {
            return Err(CoordinateRefusal::BellsDiffer {
                held: self.local.h,
                asked: other.local.h,
            });
        }
        self.local.lanes.shape_against(&other.local.lanes)?;
        let whole = wa + wb;
        if whole.signum() == 0 {
            return Err(CoordinateRefusal::NoHolding);
        }
        let merged = self.local.lanes.try_map(|l, k, x| {
            let theirs = other.local.lanes.component(l, k)?;
            (&(wa * x) + &(wb * theirs))
                .checked_div(&whole)
                .ok_or(CoordinateRefusal::NoHolding)
        })?;
        let offset_a = self
            .local
            .lanes
            .try_map(|l, k, x| Ok(x - merged.component(l, k)?))?;
        let offset_b = other
            .local
            .lanes
            .try_map(|l, k, x| Ok(x - merged.component(l, k)?))?;
        Ok(Merger {
            body: Coordinate::of(self.origin.clone(), self.local.h, merged)?,
            offsets: (offset_a, offset_b),
        })
    }

    /// The two bodies a merged body, its holdings and its offsets give back,
    /// per component, `K` re-read. Refused `NoHolding` where `wa + wb = 0`
    /// and `Unbalanced`, with the sum, where `wa o_a + wb o_b` is not zero.
    pub fn fission(
        &self,
        (wa, wb): (&Q, &Q),
        (oa, ob): (&Lanes, &Lanes),
    ) -> Result<(Coordinate, Coordinate), CoordinateRefusal> {
        if (wa + wb).signum() == 0 {
            return Err(CoordinateRefusal::NoHolding);
        }
        self.local.lanes.shape_against(oa)?;
        self.local.lanes.shape_against(ob)?;
        let sum = self
            .local
            .lanes
            .try_map(|l, k, _| Ok(&(wa * oa.component(l, k)?) + &(wb * ob.component(l, k)?)))?;
        let unbalanced = sum
            .lanes()
            .iter()
            .any(|(_, v)| v.components().iter().any(|(_, x)| x.signum() != 0));
        if unbalanced {
            return Err(CoordinateRefusal::Unbalanced { sum: sum.written() });
        }
        let back = |o: &Lanes| {
            self.local
                .lanes
                .try_map(|l, k, x| Ok(x + o.component(l, k)?))
        };
        Ok((
            Coordinate::of(self.origin.clone(), self.local.h, back(oa)?)?,
            Coordinate::of(self.origin.clone(), self.local.h, back(ob)?)?,
        ))
    }

    /// The body passed through a throat whose state is `poles` into
    /// `into`. Its S on `e_S1` must be the throat plane's span `H - R`.
    /// Into another enclosure S's span reverses (`|S|` kept) and T on `e_T1`
    /// gains `-2 span`, H and `K` untouched; into the origin it stands at the body is
    /// unchanged. Either way the throat reads its hold and reach exchanged,
    /// `H + R` kept. Refused `NoWhole` (naming the span) where `H + R = 0`,
    /// `SpanDiffers` where S is not the span, and `NoComponent` where the
    /// body holds no `e_S1` or `e_T1`.
    pub fn pass(
        &self,
        into: &Origin,
        poles: &Poles,
    ) -> Result<(Coordinate, Poles), CoordinateRefusal> {
        let span = &poles.hold - &poles.reach;
        if (&poles.hold + &poles.reach).signum() == 0 {
            return Err(CoordinateRefusal::NoWhole {
                span: span.to_string(),
            });
        }
        let (s1, t1) = (
            Key::Axis(super::keys::Axis::Letter('S', 1)),
            Key::Axis(super::keys::Axis::Letter('T', 1)),
        );
        let missing = |lane: Lane, key: &Key| CoordinateRefusal::NoComponent {
            lane,
            key: key.written(),
        };
        let s = self
            .component(&Lane::S, &s1)
            .ok_or_else(|| missing(Lane::S, &s1))?;
        if s != &span {
            return Err(CoordinateRefusal::SpanDiffers {
                held: s.to_string(),
                read: span.to_string(),
            });
        }
        self.component(&Lane::T, &t1)
            .ok_or_else(|| missing(Lane::T, &t1))?;
        let exchanged = Poles {
            hold: poles.reach.clone(),
            reach: poles.hold.clone(),
        };
        if into == &self.origin {
            return Ok((self.clone(), exchanged));
        }
        let lanes = self.local.lanes.try_map(|l, k, x| {
            Ok(match (l, k) {
                (Lane::S, k) if *k == s1 => -x,
                (Lane::T, k) if *k == t1 => x - &(&Q::int(2) * &span),
                _ => x.clone(),
            })
        })?;
        Ok((
            Coordinate {
                origin: into.clone(),
                local: Local {
                    h: self.local.h,
                    lanes,
                    k: self.local.k.clone(),
                },
            },
            exchanged,
        ))
    }
}

/// When a component reaches a value: at a bell (possibly between whole
/// bells), never (no rate), or behind, the bell it stood there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventBell {
    /// It reaches the value at this bell.
    At(Q),
    /// The component does not move.
    Unmoving,
    /// It stood at the value at this earlier bell.
    Behind(Q),
}

/// A merger's result: the merged body and each body's offsets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Merger {
    /// The merged body.
    pub body: Coordinate,
    /// The first and second bodies' offsets from it, per component.
    pub offsets: (Lanes, Lanes),
}

/// A throat's state: its hold and its reach (the poles by role).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Poles {
    /// The hold `H`.
    pub hold: Q,
    /// The reach `R`.
    pub reach: Q,
}

impl Poles {
    /// The face `delta = H / (H + R)`; `None` for no whole.
    pub fn face(&self) -> Option<Q> {
        self.hold.checked_div(&(&self.hold + &self.reach))
    }
}

/// A pole plane: the generator masks of its units `e+` and `e-` in
/// `Cl(p, q)` of the signature `NEG` (the generators squaring to -1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolePlane {
    /// The mask of `e+`, the throat's unit.
    pub positive: usize,
    /// The mask of `e-`.
    pub negative: usize,
}

/// One crossing: the throat state past the swap, its span and the flux on
/// `T`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Crossing<const NEG: usize> {
    /// The state past the swap.
    pub state: Mv<NEG>,
    /// Its span `H - R`, reversed by the swap.
    pub span: Q,
    /// The flux, `T - 2 (H - R)` with the span before.
    pub flux: Q,
}

/// The one generator a unit is: one term, of one generator, at coefficient
/// one. Refused `NotUnit` with the terms it holds.
fn generator_of<const NEG: usize>(unit: &Mv<NEG>) -> Result<usize, CoordinateRefusal> {
    let terms: Vec<(usize, &Q)> = unit.terms().collect();
    match terms.as_slice() {
        [(m, c)] if m.count_ones() == 1 && **c == Q::ONE => Ok(*m),
        _ => Err(CoordinateRefusal::NotUnit {
            terms: terms.iter().map(|(m, c)| (*m, c.to_string())).collect(),
        }),
    }
}

impl PolePlane {
    /// The plane of the units `positive` (`e+`) and `negative` (`e-`).
    /// Refused `NotUnit` for a unit that is not one generator at
    /// coefficient one, and `WrongSign`, naming the generator, for `e+`
    /// squaring to -1 or `e-` to +1.
    pub fn of<const NEG: usize>(
        positive: &Mv<NEG>,
        negative: &Mv<NEG>,
    ) -> Result<PolePlane, CoordinateRefusal> {
        let (plus, minus) = (generator_of(positive)?, generator_of(negative)?);
        if plus & NEG != 0 {
            return Err(CoordinateRefusal::WrongSign { generator: plus });
        }
        if minus & NEG != minus {
            return Err(CoordinateRefusal::WrongSign { generator: minus });
        }
        Ok(PolePlane {
            positive: plus,
            negative: minus,
        })
    }

    /// The throat state `(H + R) e+ + (H - R) e-` of `poles`.
    pub fn state<const NEG: usize>(&self, poles: &Poles) -> Mv<NEG> {
        Mv::blade(self.positive, &poles.hold + &poles.reach)
            .add(&Mv::blade(self.negative, &poles.hold - &poles.reach))
    }

    /// The throat state `x` read: hold and reach from the whole on `e+` and
    /// the span on `e-`. Refused `NotOnPolePlane` with the blades `x` holds
    /// off the plane (none for the zero).
    pub fn read<const NEG: usize>(&self, x: &Mv<NEG>) -> Result<Poles, CoordinateRefusal> {
        let blades: Vec<usize> = x
            .terms()
            .map(|(m, _)| m)
            .filter(|m| *m != self.positive && *m != self.negative)
            .collect();
        if !blades.is_empty() || x.is_zero() {
            return Err(CoordinateRefusal::NotOnPolePlane { blades });
        }
        let (whole, span) = (x.coeff(self.positive), x.coeff(self.negative));
        let half = |v: Q| &v * &crate::ratio!(1, 2);
        Ok(Poles {
            hold: half(&whole + &span),
            reach: half(&whole - &span),
        })
    }

    /// The pole swap `x -> e+ x e+`: the span reversed, the whole kept.
    /// Refused as [`PolePlane::read`] refuses `x`.
    pub fn swap<const NEG: usize>(&self, x: &Mv<NEG>) -> Result<Mv<NEG>, CoordinateRefusal> {
        self.read(x)?;
        let e = Mv::<NEG>::blade(self.positive, Q::ONE);
        Ok(e.geo(x).geo(&e))
    }

    /// One crossing of the throat state `x` with `flux` on `T`: the swap,
    /// its span, and `T - 2 (H - R)` with the span before. Refused as
    /// [`PolePlane::read`] refuses `x`.
    pub fn cross<const NEG: usize>(
        &self,
        x: &Mv<NEG>,
        flux: &Q,
    ) -> Result<Crossing<NEG>, CoordinateRefusal> {
        let before = self.read(x)?;
        let span_before = &before.hold - &before.reach;
        let state = self.swap(x)?;
        let after = self.read(&state)?;
        Ok(Crossing {
            span: &after.hold - &after.reach,
            flux: &(flux - &span_before) - &span_before,
            state,
        })
    }

    /// The riding state's pass, `q -> e+ q`: one-sided, its parity flipped.
    /// Refused `MixedParity`, with the degrees held, for a state of both
    /// parities.
    pub fn ride<const NEG: usize>(&self, q: &Mv<NEG>) -> Result<Mv<NEG>, CoordinateRefusal> {
        let mut degrees: Vec<u32> = q.terms().map(|(m, _)| m.count_ones()).collect();
        degrees.sort_unstable();
        degrees.dedup();
        let (even, odd) = (
            degrees.iter().any(|g| g % 2 == 0),
            degrees.iter().any(|g| g % 2 == 1),
        );
        if even && odd {
            return Err(CoordinateRefusal::MixedParity { degrees });
        }
        Ok(Mv::<NEG>::blade(self.positive, Q::ONE).geo(q))
    }
}
