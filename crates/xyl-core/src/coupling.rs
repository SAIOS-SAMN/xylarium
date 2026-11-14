//! The coupling (FORMS-COUPLING v1, LAW-COUPLING v1, Mycelium 78.175),
//! reconstructed from its definitions: four laws shared with netstratum
//! and Mycelium, every reading exact and first-degree, every refusal named.
//!
//! - **F1, F2: the dipole parcel, per plane.** A parcel holds `A` counts
//!   held (closed) and `Q` open, both at least zero and not both zero; a
//!   kind `(Z, U)` holds `A = 3Z - U`, `Q = U`. Its pole is
//!   `sigma = (A - Q) / (A + Q)` in `[-1, 1]` and its strength the open
//!   share `o = Q / (A + Q)`, the fluidity; the held share is the cohesion.
//!   A body founded on `n` turning planes holds `((3Z - U) / n, U / n)` on
//!   each, and a bond closes open counts on one plane only.
//! - **F3: the range's field on one turning plane.**
//!   `F = sum w sigma o n h / sum w`, with the nearness `n = M / (|g| + M)`
//!   and the alignment `h = 1 - Delta / 90` of the round separation
//!   `Delta` in `[0, 180]` degrees on the plane, a named L1 reading. The
//!   pull on a rate `x` (a share of the luminal bound) toward the range's
//!   rate `x_bar` is `y = -sigma F o (x_bar (-) x)`, the rate past it
//!   `x (+) y` by the flow law (`shares::flow`). Two bodies are neighbours
//!   where their enclosures meet, `|R_a - R_b| <= R_a e_a + R_b e_b`,
//!   `e = m / (m + H(<R))`.
//! - **F4: the light ledger.** Light reaching a body is received on its
//!   unlocked face, `1 - delta` on the continuum and `delta` on the
//!   vacuum (`delta = H / (H + R)`, `coordinate::Poles::face`), and the
//!   rest drains; a bell's ledger balances, made = received + drained.

use crate::coordinate::{Key, Plane, Poles};
use crate::Q;
use std::cmp::Ordering;

/// Why a coupling reading is refused, by name and with what was read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CouplingRefusal {
    /// A rate outside `(-1, 1)`: the value.
    NotShare(Q),
    /// A ledger whose made is not received plus drained: the three and the
    /// residue `made - received - drained`.
    Unbalanced {
        /// Light made.
        made: Q,
        /// Light received.
        received: Q,
        /// Light drained.
        drained: Q,
        /// The residue.
        residue: Q,
    },
    /// A holding in a range that is not positive.
    NoHolding(Q),
    /// Counts of which one is below zero, or none held at all.
    NotCounts {
        /// The held counts.
        held: Q,
        /// The open counts.
        open: Q,
    },
    /// A dipole with its pole outside `[-1, 1]` or its open share outside
    /// `[0, 1]`.
    NotDipole {
        /// The pole.
        pole: Q,
        /// The open share.
        open: Q,
    },
    /// A round separation outside `[0, 180]` degrees.
    NotSeparation(Q),
    /// A body with no turning plane.
    NoPlanes,
    /// A plane named twice in one body.
    PlaneRepeated(Key),
    /// A bond on a plane the body does not hold.
    NoPlane(Key),
    /// A nearness outside `[0, 1]`.
    NotNearness(Q),
    /// A moment, hold, reach or light below zero.
    Negative(Q),
    /// A share of no whole: its part and the rest.
    NoWhole {
        /// The part.
        part: Q,
        /// The rest.
        rest: Q,
    },
    /// A key that holds no turning, and its plane (`None` for one axis).
    NotTurningPlane {
        /// The key.
        key: Key,
        /// Its plane.
        plane: Option<Plane>,
    },
    /// A neighbour aligned on another plane than the field's.
    PlanesDiffer {
        /// The field's plane.
        field: Key,
        /// The neighbour's.
        neighbour: Key,
    },
}

/// `part / (part + rest)`; refused `NoWhole` where they sum to zero.
fn share_of(part: &Q, rest: &Q) -> Result<Q, CouplingRefusal> {
    part.checked_div(&(part + rest))
        .ok_or_else(|| CouplingRefusal::NoWhole {
            part: part.clone(),
            rest: rest.clone(),
        })
}

/// A quantity not below zero; refused `Negative`.
fn held(x: &Q) -> Result<&Q, CouplingRefusal> {
    if x.signum() < 0 {
        return Err(CouplingRefusal::Negative(x.clone()));
    }
    Ok(x)
}

/// A rate inside `(-1, 1)`; refused `NotShare`.
fn rate(x: &Q) -> Result<&Q, CouplingRefusal> {
    if x.abs() >= Q::ONE {
        return Err(CouplingRefusal::NotShare(x.clone()));
    }
    Ok(x)
}

/// The turning plane `key` names; refused `NotTurningPlane`.
fn turning(key: &Key) -> Result<(), CouplingRefusal> {
    match key.plane() {
        Some(Plane::Turning) => Ok(()),
        plane => Err(CouplingRefusal::NotTurningPlane { key: *key, plane }),
    }
}

/// The hemisphere's L1 reading of a round separation `Delta` in
/// `[0, 180]` degrees: `1 - Delta / 90`, aligned at 0 (`+1`), uncoupled at
/// 90, opposed at 180 (`-1`). A named reading of a round angle, never
/// itself a degree. Refused `NotSeparation` outside `[0, 180]`.
pub fn hemisphere_l1(separation: &Q) -> Result<Q, CouplingRefusal> {
    if separation.signum() < 0 || separation > &Q::int(180) {
        return Err(CouplingRefusal::NotSeparation(separation.clone()));
    }
    Ok(&Q::ONE - &(separation * &crate::ratio!(1, 90)))
}

/// The flow law's inverse, `a (-) b = a (+) (-b)`: the rate `a` read from
/// `b`. Refused `NotShare` with a rate outside `(-1, 1)`.
pub fn relative(a: &Q, b: &Q) -> Result<Q, CouplingRefusal> {
    let back = -rate(b)?;
    crate::shares::flow(rate(a)?, &back).ok_or_else(|| CouplingRefusal::NotShare(b.clone()))
}

/// A parcel's counts: `held` (`A`, closed) and `open` (`Q`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parcel {
    held: Q,
    open: Q,
}

/// A dipole: its pole in `[-1, 1]` and its open share in `[0, 1]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dipole {
    /// The pole `sigma`, closed against open.
    pub pole: Q,
    /// The open share `o`, the strength.
    pub open: Q,
}

impl Parcel {
    /// The parcel of `held` and `open` counts; refused `NotCounts` where one
    /// is below zero or both are zero.
    pub fn of(held: Q, open: Q) -> Result<Parcel, CouplingRefusal> {
        if held.signum() < 0 || open.signum() < 0 || (held.signum() == 0 && open.signum() == 0) {
            return Err(CouplingRefusal::NotCounts { held, open });
        }
        Ok(Parcel { held, open })
    }

    /// A kind's parcel: `A = 3Z - U` held and `Q = U` open; refused
    /// `NotCounts` where `U > 3Z` or `U < 0`.
    pub fn kind(z: &Q, u: &Q) -> Result<Parcel, CouplingRefusal> {
        Parcel::of(&(&Q::int(3) * z) - u, u.clone())
    }

    /// The held counts `A`.
    pub fn held(&self) -> &Q {
        &self.held
    }

    /// The open counts `Q`.
    pub fn open(&self) -> &Q {
        &self.open
    }

    /// The open share `Q / (A + Q)`, the fluidity.
    pub fn open_share(&self) -> Result<Q, CouplingRefusal> {
        share_of(&self.open, &self.held)
    }

    /// The held share `A / (A + Q)`, the cohesion.
    pub fn held_share(&self) -> Result<Q, CouplingRefusal> {
        share_of(&self.held, &self.open)
    }

    /// The dipole the counts hold: `sigma = 1 - 2 o`, `o` the open share.
    pub fn dipole(&self) -> Result<Dipole, CouplingRefusal> {
        let open = self.open_share()?;
        Ok(Dipole {
            pole: &Q::ONE - &(&Q::int(2) * &open),
            open,
        })
    }

    /// The inversion: the counts exchanged, `o -> 1 - o`, `sigma -> -sigma`.
    pub fn inverted(&self) -> Parcel {
        Parcel {
            held: self.open.clone(),
            open: self.held.clone(),
        }
    }
}

impl Dipole {
    /// A dipole stated by its pole and open share; refused `NotDipole`
    /// outside `[-1, 1]` and `[0, 1]`.
    pub fn stated(pole: Q, open: Q) -> Result<Dipole, CouplingRefusal> {
        if pole.abs() > Q::ONE || open.signum() < 0 || open > Q::ONE {
            return Err(CouplingRefusal::NotDipole { pole, open });
        }
        Ok(Dipole { pole, open })
    }
}

/// A body's counts per turning plane, in the order founded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parcels {
    planes: Vec<(Key, Parcel)>,
}

impl Parcels {
    /// The kind `(Z, U)` founded on `planes` by the even split, each plane
    /// holding `((3Z - U) / n, U / n)`. Refused `NoPlanes` with none,
    /// `NotTurningPlane`, `PlaneRepeated`, and as [`Parcel::kind`] is.
    pub fn founded(z: &Q, u: &Q, planes: &[Key]) -> Result<Parcels, CouplingRefusal> {
        let whole = Parcel::kind(z, u)?;
        let n = Q::int(i64::try_from(planes.len()).map_err(|_| CouplingRefusal::NoPlanes)?);
        let each = |x: &Q| x.checked_div(&n).ok_or(CouplingRefusal::NoPlanes);
        let mut held: Vec<(Key, Parcel)> = Vec::with_capacity(planes.len());
        for key in planes {
            turning(key)?;
            if held.iter().any(|(k, _)| k == key) {
                return Err(CouplingRefusal::PlaneRepeated(*key));
            }
            held.push((
                *key,
                Parcel {
                    held: each(&whole.held)?,
                    open: each(&whole.open)?,
                },
            ));
        }
        if held.is_empty() {
            return Err(CouplingRefusal::NoPlanes);
        }
        Ok(Parcels { planes: held })
    }

    /// The planes and their counts, in the order founded.
    pub fn planes(&self) -> &[(Key, Parcel)] {
        &self.planes
    }

    /// A bond on one plane: `closed` open counts close there,
    /// `Q -> Q - closed`, `A -> A + closed` (a negative `closed` peels).
    /// Refused `NoPlane` and, below zero, `NotCounts`.
    pub fn bond(&self, plane: &Key, closed: &Q) -> Result<Parcels, CouplingRefusal> {
        let mut planes = self.planes.clone();
        let (_, at) = planes
            .iter_mut()
            .find(|(k, _)| k == plane)
            .ok_or(CouplingRefusal::NoPlane(*plane))?;
        *at = Parcel::of(&at.held + closed, &at.open - closed)?;
        Ok(Parcels { planes })
    }
}

/// How a neighbour stands on the field's turning plane: its round
/// separation in degrees and the alignment `h = 1 - Delta / 90`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alignment {
    /// The turning plane.
    pub plane: Key,
    /// The round separation, degrees.
    pub separation: Q,
    /// The alignment, in `[-1, 1]`.
    pub along: Q,
}

impl Alignment {
    /// The alignment at `separation` degrees on `plane`; refused
    /// `NotTurningPlane` and `NotSeparation`.
    pub fn on(plane: Key, separation: Q) -> Result<Alignment, CouplingRefusal> {
        turning(&plane)?;
        let along = hemisphere_l1(&separation)?;
        Ok(Alignment {
            plane,
            separation,
            along,
        })
    }
}

/// One body of a range: its holding, dipole, nearness and alignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Neighbour {
    /// The holding `w`, positive.
    pub holding: Q,
    /// Its dipole.
    pub dipole: Dipole,
    /// Its nearness, in `[0, 1]`.
    pub nearness: Q,
    /// Its alignment.
    pub alignment: Alignment,
}

impl Neighbour {
    /// The neighbour; refused `NoHolding` for a holding not positive and
    /// `NotNearness` outside `[0, 1]`.
    pub fn of(
        holding: Q,
        dipole: Dipole,
        nearness: Q,
        alignment: Alignment,
    ) -> Result<Neighbour, CouplingRefusal> {
        if holding.signum() <= 0 {
            return Err(CouplingRefusal::NoHolding(holding));
        }
        if nearness.signum() < 0 || nearness > Q::ONE {
            return Err(CouplingRefusal::NotNearness(nearness));
        }
        Ok(Neighbour {
            holding,
            dipole,
            nearness,
            alignment,
        })
    }
}

/// The ring law's nearness of a moment `M` at the gap `g`,
/// `M / (|g| + M)`; refused `Negative` and `NoWhole`.
pub fn nearness(moment: &Q, gap: &Q) -> Result<Q, CouplingRefusal> {
    share_of(held(moment)?, &gap.abs())
}

/// The range's field on one turning plane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Field {
    /// No body in the range.
    Alone,
    /// The range's whole holding and its field `F`, in `[-1, 1]`.
    Held {
        /// `sum w`.
        holding: Q,
        /// `F`.
        field: Q,
    },
}

impl Field {
    /// The field's value: zero alone.
    pub fn value(&self) -> Q {
        match self {
            Field::Alone => Q::ZERO,
            Field::Held { field, .. } => field.clone(),
        }
    }
}

/// The field of `range` on `plane`, `F = sum w sigma o n h / sum w`.
/// Refused `NotTurningPlane` and `PlanesDiffer`.
pub fn field(plane: &Key, range: &[Neighbour]) -> Result<Field, CouplingRefusal> {
    turning(plane)?;
    if let Some(j) = range.iter().find(|j| j.alignment.plane != *plane) {
        return Err(CouplingRefusal::PlanesDiffer {
            field: *plane,
            neighbour: j.alignment.plane,
        });
    }
    if range.is_empty() {
        return Ok(Field::Alone);
    }
    let holding = range.iter().fold(Q::ZERO, |s, j| &s + &j.holding);
    let pulled = range.iter().fold(Q::ZERO, |s, j| {
        let term = &(&(&(&j.holding * &j.dipole.pole) * &j.dipole.open) * &j.nearness)
            * &j.alignment.along;
        &s + &term
    });
    let field = pulled
        .checked_div(&holding)
        .ok_or_else(|| CouplingRefusal::NoHolding(holding.clone()))?;
    Ok(Field::Held { holding, field })
}

/// The field's pull on a rate: the gap `x_bar (-) x`, the push and the rate
/// past it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pull {
    /// The gap to the range's rate.
    pub gap: Q,
    /// The push `y = -sigma F o gap`.
    pub push: Q,
    /// The rate past it, `x (+) y`.
    pub rate: Q,
}

/// The pull on a body of dipole `body` at rate `x`, toward the range's rate
/// `x_bar`: opposed dipoles lock, aligned ones split, alone the rate is
/// kept. Refused `NotShare` with a rate outside `(-1, 1)`.
pub fn pull(x: &Q, x_bar: &Q, body: &Dipole, field: &Field) -> Result<Pull, CouplingRefusal> {
    let gap = relative(x_bar, x)?;
    let push = -(&(&(&body.pole * &field.value()) * &body.open) * &gap);
    let past = crate::shares::flow(rate(x)?, rate(&push)?)
        .ok_or_else(|| CouplingRefusal::NotShare(push.clone()))?;
    Ok(Pull {
        gap,
        push,
        rate: past,
    })
}

/// A body's own enclosure: its reach `R`, moment `m` and the hold within
/// its reach `H(<R)`; it encloses `R m / (m + H(<R))` about it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Enclosure {
    /// The reach.
    pub reach: Q,
    /// The moment.
    pub moment: Q,
    /// The hold within the reach.
    pub hold: Q,
}

impl Enclosure {
    /// The reach enclosed, `R e`; refused `Negative` and `NoWhole`.
    pub fn enclosed(&self) -> Result<Q, CouplingRefusal> {
        Ok(held(&self.reach)? * &share_of(held(&self.moment)?, held(&self.hold)?)?)
    }
}

/// Where two enclosures stand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Meets {
    /// Overlapping: neighbours.
    Within,
    /// Touching at the bound: neighbours.
    Bound,
    /// Apart: not neighbours.
    Apart,
}

/// The neighbour reading: the separation `|R_a - R_b|`, the reach held
/// together `R_a e_a + R_b e_b`, and where they stand.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Meeting {
    /// `|R_a - R_b|`.
    pub separation: Q,
    /// `R_a e_a + R_b e_b`.
    pub reach: Q,
    /// Where they stand.
    pub meets: Meets,
}

/// Whether two bodies' enclosures meet; refused as
/// [`Enclosure::enclosed`] is.
pub fn meets(a: &Enclosure, b: &Enclosure) -> Result<Meeting, CouplingRefusal> {
    let reach = &a.enclosed()? + &b.enclosed()?;
    let separation = (&a.reach - &b.reach).abs();
    let meets = match separation.cmp(&reach) {
        Ordering::Less => Meets::Within,
        Ordering::Equal => Meets::Bound,
        Ordering::Greater => Meets::Apart,
    };
    Ok(Meeting {
        separation,
        reach,
        meets,
    })
}

/// The face a body stands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    /// The continuum: unlocked `1 - delta`.
    Continuum,
    /// The vacuum, the faces exchanged: unlocked `delta`.
    Vacuum,
}

/// The unlocked face of a body of hold `H` at reach `R` on `face`, from the
/// throat's face `delta = H / (H + R)` (`coordinate::Poles::face`).
/// Refused `Negative` and, for no whole, `NoWhole`.
pub fn unlocked(face: Face, hold: &Q, reach: &Q) -> Result<Q, CouplingRefusal> {
    let poles = Poles {
        hold: held(hold)?.clone(),
        reach: held(reach)?.clone(),
    };
    let delta = poles.face().ok_or_else(|| CouplingRefusal::NoWhole {
        part: hold.clone(),
        rest: reach.clone(),
    })?;
    Ok(match face {
        Face::Continuum => &Q::ONE - &delta,
        Face::Vacuum => delta,
    })
}

/// Light taken by one body: received and drained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Taken {
    /// Received on the unlocked face.
    pub received: Q,
    /// The rest, drained.
    pub drained: Q,
}

/// The light `light` reaching a body: `light u` received, the rest
/// drained. Refused `Negative` and as [`unlocked`] is.
pub fn take(light: &Q, face: Face, hold: &Q, reach: &Q) -> Result<Taken, CouplingRefusal> {
    let received = held(light)? * &unlocked(face, hold, reach)?;
    Ok(Taken {
        drained: light - &received,
        received,
    })
}

/// One bell's light ledger, balanced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ledger {
    /// Light made.
    pub made: Q,
    /// Light received.
    pub received: Q,
    /// Light drained.
    pub drained: Q,
}

/// One bell's ledger: admitted where made = received + drained, refused
/// `Unbalanced` with the residue otherwise.
pub fn ledger(made: &Q, received: &Q, drained: &Q) -> Result<Ledger, CouplingRefusal> {
    let residue = &(made - received) - drained;
    if residue.signum() != 0 {
        return Err(CouplingRefusal::Unbalanced {
            made: made.clone(),
            received: received.clone(),
            drained: drained.clone(),
            residue,
        });
    }
    Ok(Ledger {
        made: made.clone(),
        received: received.clone(),
        drained: drained.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Axis;

    fn rational(n: i64, d: i64) -> Q {
        Q::new(n, d).unwrap()
    }

    fn tau(k: u32, l: u32) -> Key {
        Key::generator(Axis::unit(true, k).unwrap(), Axis::unit(true, l).unwrap()).unwrap()
    }

    /// F1, F2: the pole and the open share are ratios of the counts: the
    /// closed pole +1, the throat 0, the open pole -1; the inversion
    /// exchanges the shares and negates the pole; a body founded on its
    /// planes splits evenly and a bond moves one plane only, its whole kept,
    /// and a bond then its peel is the identity.
    #[test]
    fn a_dipole_is_read_on_its_counts() {
        let d = |a: i64, o: i64| Parcel::of(Q::int(a), Q::int(o)).unwrap().dipole().unwrap();
        assert_eq!(d(1, 0).pole, Q::ONE);
        assert_eq!(d(1, 1).pole, Q::ZERO);
        assert_eq!(d(0, 1).pole, -Q::ONE);
        let p = Parcel::of(Q::int(2), Q::int(4)).unwrap();
        let (a, b) = (p.dipole().unwrap(), p.inverted().dipole().unwrap());
        assert_eq!(b.pole, -a.pole.clone());
        assert_eq!(&a.open + &b.open, Q::ONE);
        assert_eq!(&p.open_share().unwrap() + &p.held_share().unwrap(), Q::ONE);
        assert!(matches!(
            Parcel::of(Q::ZERO, Q::ZERO),
            Err(CouplingRefusal::NotCounts { .. })
        ));
        let planes = [tau(2, 3), tau(3, 1), tau(1, 2)];
        let body = Parcels::founded(&Q::ONE, &Q::ONE, &planes).unwrap();
        assert!(body
            .planes()
            .iter()
            .all(|(_, p)| p.held() == &rational(2, 3) && p.open() == &rational(1, 3)));
        let bonded = body.bond(&tau(1, 2), &rational(1, 3)).unwrap();
        assert_eq!(&bonded.planes()[0], &body.planes()[0]);
        let (_, on) = &bonded.planes()[2];
        assert_eq!(on.held() + on.open(), Q::ONE);
        assert_eq!(bonded.bond(&tau(1, 2), &rational(-1, 3)).unwrap(), body);
        assert_eq!(
            Parcels::founded(&Q::ONE, &Q::ONE, &[tau(1, 2), tau(1, 2)]),
            Err(CouplingRefusal::PlaneRepeated(tau(1, 2)))
        );
        assert_eq!(
            Parcels::founded(&Q::ONE, &Q::ONE, &[]),
            Err(CouplingRefusal::NoPlanes)
        );
    }

    /// F3: the field is bounded by one and the rate stays inside the
    /// luminal bound; opposed dipoles draw the rate toward the range's,
    /// aligned ones push it away; a body alone, or the pure closed pole,
    /// keeps its rate; the alignment is uncoupled at 90 degrees.
    #[test]
    fn the_field_pulls_within_the_bound() {
        let t = tau(1, 2);
        let n = |w: i64, (pole, open): (Q, Q), along: i64| {
            Neighbour::of(
                Q::int(w),
                Dipole::stated(pole, open).unwrap(),
                rational(1, 2),
                Alignment::on(t, Q::int(along)).unwrap(),
            )
            .unwrap()
        };
        let h = (rational(1, 3), rational(1, 3));
        let f = field(&t, &[n(1, h.clone(), 0), n(2, h.clone(), 45)]).unwrap();
        assert!(f.value().abs() <= Q::ONE);
        let (x, x_bar) = (rational(1, 2), rational(1, 4));
        let drawn = pull(
            &x,
            &x_bar,
            &Dipole::stated(-rational(1, 3), rational(2, 3)).unwrap(),
            &f,
        )
        .unwrap();
        assert!(drawn.rate < x && drawn.rate.abs() < Q::ONE);
        let split = pull(
            &x,
            &x_bar,
            &Dipole::stated(h.0.clone(), h.1.clone()).unwrap(),
            &f,
        )
        .unwrap();
        assert!(split.rate > x);
        assert_eq!(
            pull(
                &x,
                &x_bar,
                &Dipole::stated(h.0, h.1).unwrap(),
                &Field::Alone
            )
            .unwrap()
            .rate,
            x
        );
        let closed = Dipole::stated(Q::ONE, Q::ZERO).unwrap();
        assert_eq!(pull(&x, &x_bar, &closed, &f).unwrap().rate, x);
        assert_eq!(hemisphere_l1(&Q::int(90)), Ok(Q::ZERO));
        assert_eq!(
            hemisphere_l1(&Q::int(181)),
            Err(CouplingRefusal::NotSeparation(Q::int(181)))
        );
        assert!(matches!(
            pull(&Q::ONE, &x_bar, &closed, &f),
            Err(CouplingRefusal::NotShare(_))
        ));
        let e = |r: i64, m: i64, h: i64| Enclosure {
            reach: Q::int(r),
            moment: Q::int(m),
            hold: Q::int(h),
        };
        // R e = 1 and 3 against a separation of 4: at the bound; with the
        // hold three times the moment, 1/2 and 3/2: apart; nearer, within.
        assert_eq!(meets(&e(2, 1, 1), &e(6, 1, 1)).unwrap().meets, Meets::Bound);
        assert_eq!(meets(&e(2, 1, 3), &e(6, 1, 3)).unwrap().meets, Meets::Apart);
        assert_eq!(
            meets(&e(10, 1, 1), &e(12, 1, 1)).unwrap().meets,
            Meets::Within
        );
    }

    /// F4: light taken balances its ledger on either face, the faces
    /// exchanging what is received; a ledger off by a residue is refused
    /// with it.
    #[test]
    fn light_is_conserved() {
        let light = rational(7, 2);
        let (h, r) = (Q::int(3), Q::ONE);
        let c = take(&light, Face::Continuum, &h, &r).unwrap();
        let v = take(&light, Face::Vacuum, &h, &r).unwrap();
        assert_eq!(c.received, v.drained);
        assert!(ledger(&light, &c.received, &c.drained).is_ok());
        assert_eq!(
            ledger(&light, &c.received, &Q::ZERO),
            Err(CouplingRefusal::Unbalanced {
                made: light.clone(),
                received: c.received.clone(),
                drained: Q::ZERO,
                residue: c.drained.clone(),
            })
        );
    }
}
