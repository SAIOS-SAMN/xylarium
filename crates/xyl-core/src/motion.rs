//! Motion: closed-form orbits of exponential modes, evaluated at any time `t`
//! in one step, with no loop over time.
//!
//! Inputs: exact amplitudes, rates and times ([`Q`]) in the caller's units;
//! uses `certified` for `ln 2` and `exp`.
//!
//! Outputs:
//! - [`Boost`], one mode `amp x base^(rate t)` with [`Base`] `E` (e-folds) or
//!   `Two` (halvings, so a half-life `h` is the exact rate `-1/h` with no
//!   `ln 2` in the phase); `grown(t)` encloses `base^(rate t)`.
//! - [`Modes`], a sum of boosts (the orbit of a linear process); `at(t)` gives
//!   a [`Reading`] with `value`, `slope` and `integral` from 0, each one
//!   evaluation whatever `t`.
//! - [`ExpSum`], a sum of terms `coef t^power e^(-rate t)` ([`Term`]):
//!   `relaxed(k)` is the closed-form solution of `y' = -k y + self`,
//!   `y(0) = 0`, so chains of linear relaxations compose exactly; `at` and
//!   `at_with` evaluate it, [`Grown`] sharing each rate's exponential between
//!   orbits read at the same `t`.
//!
//! Invariants: phases (`rate t`) and rates are exact; only values are
//! enclosures (`certified::Encl`). Whole halvings are exact powers of two and
//! only the remainder goes through the series; a rapidity below -4096 is
//! held as the bound `[0, 2^-4096]`.

use crate::certified::{exp_of, ln2, pow2, Encl, FIXED};
use crate::q::dyadic;
use crate::Q;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, ToPrimitive};

/// `2^k`, exact, for a whole `k` of either sign; `None` past `u32` halvings.
fn two_to(k: i64) -> Option<Q> {
    let n = u32::try_from(k.unsigned_abs()).ok()?;
    Some(if k < 0 {
        dyadic(BigInt::one(), u64::from(n))
    } else {
        dyadic(pow2(n), 0)
    })
}

/// What a boost's rapidity counts: e-folds or halvings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Base {
    /// The rapidity counts e-folds: the mode grows as `e^(rate t)`.
    E,
    /// The rapidity counts doublings (halvings when negative): the mode grows
    /// as `2^(rate t)`.
    Two,
}

/// One mode: `amp x base^(rate t)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Boost {
    /// Amplitude: the mode's value at `t = 0`, in the caller's units.
    pub amp: Q,
    /// Rapidity per unit time, in units of `base` (negative for decay).
    pub rate: Q,
    /// What the rapidity counts: e-folds or halvings.
    pub base: Base,
}

impl Boost {
    /// The mode `amp e^(rate t)`, `rate` in e-folds per unit time.
    pub fn e(amp: Q, rate: Q) -> Boost {
        Boost {
            amp,
            rate,
            base: Base::E,
        }
    }

    /// The mode `amp 2^(rate t)`, `rate` in doublings per unit time (a decay
    /// with half-life `h` has `rate = -1/h`).
    pub fn halving(amp: Q, rate: Q) -> Boost {
        Boost {
            amp,
            rate,
            base: Base::Two,
        }
    }

    /// The mode's rate in e-folds per unit time (`rate ln 2` for base two).
    fn efolds(&self) -> Encl {
        match self.base {
            Base::E => Encl::exact(self.rate.clone()),
            // The enclosure of ln 2 to 2^-100, finer than any reading requires.
            Base::Two => Encl::exact(self.rate.clone()).mul(&ln2(100)),
        }
    }

    /// `base^(rate t)`, enclosed: the rapidity `x = rate t` is split into
    /// whole halvings `k` and a remainder `r` in `[0, 2 ln 2)` (base two:
    /// `x = k + f`, `r = f ln 2`; base e: `x = k ln 2 + r`), `2^k` is exact
    /// and `e^r` is `certified::exp_of` of the remainder's enclosure, so the
    /// value keeps its relative precision at any magnitude. A rapidity above
    /// 4096 goes through `exp_of` directly.
    pub fn grown(&self, t: &Q) -> Encl {
        let x = &self.rate * t;
        if x.is_zero() {
            return Encl::exact(Q::ONE);
        }
        // A decay past 4096 halvings (or e-folds) is below 2^-4096 and is
        // returned as the enclosure [0, 2^-4096].
        let far = Q::int(4096);
        if x < -far.clone() {
            return Encl {
                lo: Q::ZERO,
                hi: dyadic(BigInt::one(), 4096),
            };
        }
        if x > far {
            return exp_of(&Encl::exact(x).mul(&self.efolds()), FIXED);
        }
        let l2 = ln2(100);
        let whole = |k: &BigInt| Encl::exact(Q::from_reduced(BigRational::from_integer(k.clone())));
        let (k, r) = match self.base {
            Base::Two => {
                let k = x.floor();
                let f = Encl::exact(x.clone()).sub(&whole(&k));
                (k, f.mul(&l2))
            }
            Base::E => {
                // The first `k` whose remainder is certainly non-negative.
                // The bound on ln 2 is positive, so the quotient reads; were it
                // not to, the rapidity goes through `exp_of` whole, as past
                // 4096.
                let Some(q) = x.checked_div(&l2.hi) else {
                    return exp_of(&Encl::exact(x).mul(&self.efolds()), FIXED);
                };
                let mut k = q.floor();
                let mut r = Encl::exact(x.clone()).sub(&whole(&k).mul(&l2));
                if r.lo.signum() < 0 {
                    k -= 1;
                    r = Encl::exact(x.clone()).sub(&whole(&k).mul(&l2));
                }
                (k, r)
            }
        };
        let Some(scale) = k.to_i64().and_then(two_to) else {
            return exp_of(&Encl::exact(x).mul(&self.efolds()), FIXED);
        };
        Encl::exact(scale).mul(&exp_of(&r, FIXED))
    }
}

/// A sum of modes: the orbit of a linear process.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Modes {
    /// The modes summed; an empty list is the zero orbit.
    pub terms: Vec<Boost>,
}

impl Modes {
    /// The orbit summing `terms`.
    pub fn of(terms: Vec<Boost>) -> Modes {
        Modes { terms }
    }

    /// The orbit read at `t`: each mode's `base^(rate t)` is evaluated once,
    /// and the value, slope and integral are computed from it (enclosed to
    /// about 2^-60 relative).
    pub fn at(&self, t: &Q) -> Reading<'_> {
        Reading {
            modes: self,
            t: t.clone(),
            grown: self.terms.iter().map(|b| b.grown(t)).collect(),
        }
    }

    /// The orbit's value at `t`, enclosed (shorthand for `at(t).value()`).
    pub fn value(&self, t: &Q) -> Encl {
        self.at(t).value()
    }

    /// The same orbit with every amplitude scaled by `s`.
    pub fn scaled(&self, s: &Q) -> Modes {
        Modes {
            terms: self
                .terms
                .iter()
                .map(|b| Boost {
                    amp: &b.amp * s,
                    ..b.clone()
                })
                .collect(),
        }
    }

    /// Two orbits added.
    pub fn plus(&self, other: &Modes) -> Modes {
        Modes {
            terms: self.terms.iter().chain(&other.terms).cloned().collect(),
        }
    }
}

/// An orbit read at one time.
pub struct Reading<'m> {
    modes: &'m Modes,
    t: Q,
    grown: Vec<Encl>,
}

impl Reading<'_> {
    fn sum(&self, f: impl Fn(&Boost, &Encl) -> Encl) -> Encl {
        self.modes
            .terms
            .iter()
            .zip(&self.grown)
            .fold(Encl::exact(Q::ZERO), |acc, (b, g)| acc.add(&f(b, g)))
    }

    /// The orbit's value at the reading's time.
    pub fn value(&self) -> Encl {
        self.sum(|b, g| Encl::exact(b.amp.clone()).mul(g))
    }

    /// The orbit's rate of change: each mode's value times its e-fold rate.
    pub fn slope(&self) -> Encl {
        self.sum(|b, g| Encl::exact(b.amp.clone()).mul(g).mul(&b.efolds()))
    }

    /// The orbit's integral from 0: `amp (grown - 1) / k` for each mode, with
    /// `k` its rate in e-folds, and `amp t` for a constant mode.
    pub fn integral(&self) -> Encl {
        self.sum(|b, g| {
            if b.rate.is_zero() {
                return Encl::exact(&b.amp * &self.t);
            }
            // 1/k is rounded outward to a dyadic grid: it is still an
            // enclosure, and every later product stays on the gcd-free dyadic
            // path of `Q`.
            match b.efolds().recip().map(|inv| inv.outward(FIXED + 32)) {
                Some(inv) => Encl::exact(b.amp.clone())
                    .mul(&g.sub(&Encl::exact(Q::ONE)))
                    .mul(&inv),
                None => Encl::exact(&b.amp * &self.t),
            }
        })
    }
}

/// One term of an orbit of a linear process: `coef t^power e^(-rate t)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Term {
    /// The coefficient, an enclosure of a real.
    pub coef: Encl,
    /// The power of `t` multiplying the exponential.
    pub power: u32,
    /// The decay rate in e-folds per unit time (exact).
    pub rate: Q,
}

/// A sum of decaying modes with polynomial factors: the closed form of any
/// chain of linear relaxations driven by exponential modes (for example a
/// gas reservoir, the stars formed from it and the elements they produce).
/// Rates are exact; only coefficients carry enclosures.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExpSum {
    /// The terms summed; an empty list is the zero function.
    pub terms: Vec<Term>,
}

/// Coefficients are rounded outward to the grid of `2^-KEEP`: an absolute
/// precision far finer than any reading requires.
const KEEP: u32 = 120;

impl ExpSum {
    /// `coef e^(-rate t)`.
    pub fn decay(coef: Encl, rate: Q) -> ExpSum {
        ExpSum {
            terms: vec![Term {
                coef,
                power: 0,
                rate,
            }],
        }
    }

    /// The same orbit times `k`.
    pub fn scaled(&self, k: &Encl) -> ExpSum {
        ExpSum {
            terms: self
                .terms
                .iter()
                .map(|t| Term {
                    coef: t.coef.mul(k).outward(KEEP),
                    ..t.clone()
                })
                .collect(),
        }
    }

    /// Two orbits added; terms with equal power and rate are merged.
    pub fn plus(&self, other: &ExpSum) -> ExpSum {
        let mut out = self.clone();
        for t in &other.terms {
            out.push(t.clone());
        }
        out
    }

    fn push(&mut self, t: Term) {
        match self
            .terms
            .iter_mut()
            .find(|x| x.power == t.power && x.rate == t.rate)
        {
            Some(x) => x.coef = x.coef.add(&t.coef).outward(KEEP),
            None => self.terms.push(t),
        }
    }

    /// The process this orbit drives through a relaxation at rate `k`:
    /// `y' = -k y + self`, `y(0) = 0`, in closed form. For a term
    /// `c t^m e^(-l t)` with `a = k - l`: if `a = 0`, the result is
    /// `c t^(m+1) / (m+1) e^(-k t)`; otherwise it is
    /// `c [sum over i of (-1)^i m!/(m-i)! t^(m-i) / a^(i+1)] e^(-l t)` minus
    /// `c (-1)^m m! / a^(m+1) e^(-k t)`.
    pub fn relaxed(&self, k: &Q) -> ExpSum {
        let mut out = ExpSum::default();
        for t in &self.terms {
            let a = k - &t.rate;
            let m = t.power;
            let Some(inv) = a.recip() else {
                // The resonant term, `a = 0`. The power is unsigned, so the
                // divisor `m + 1` is at least one
                // (`tests::a_resonant_relaxation_integrates_the_power`).
                let next = std::num::NonZeroI64::new(i64::from(m) + 1);
                out.push(Term {
                    coef: t
                        .coef
                        .mul(&Encl::exact(next.map_or(Q::ONE, |d| Q::ONE.over(d))))
                        .outward(KEEP),
                    power: m + 1,
                    rate: k.clone(),
                });
                continue;
            };
            // The falling factorial m!/(m-i)! and (1/a)^(i+1), both exact.
            let (mut fall, mut pow) = (Q::ONE, inv.clone());
            for i in 0..=m {
                let sign = if i % 2 == 0 { Q::ONE } else { -Q::ONE };
                out.push(Term {
                    coef: t
                        .coef
                        .mul(&Encl::exact(&(&sign * &fall) * &pow))
                        .outward(KEEP),
                    power: m - i,
                    rate: t.rate.clone(),
                });
                fall = &fall * &Q::int(i64::from(m - i));
                pow = &pow * &inv;
            }
            // The homogeneous part, chosen so that y(0) = 0, has the
            // coefficient -c (-1)^m m! / a^(m+1).
            let m_fact = (1..=m).fold(Q::ONE, |acc, j| &acc * &Q::int(i64::from(j)));
            let sign = if m % 2 == 0 { -Q::ONE } else { Q::ONE };
            let last = (0..=m).fold(Q::ONE, |acc, _| &acc * &inv);
            out.push(Term {
                coef: t
                    .coef
                    .mul(&Encl::exact(&(&sign * &m_fact) * &last))
                    .outward(KEEP),
                power: 0,
                rate: k.clone(),
            });
        }
        out
    }

    /// The orbit's value at `t` (t >= 0), enclosed.
    pub fn at(&self, t: &Q) -> Encl {
        self.at_with(t, &mut Grown::default())
    }

    /// The orbit's value at `t`, with each distinct rate's `e^(-rate t)`
    /// evaluated once in `grown` and shared with every other orbit read at
    /// the same `t`.
    pub fn at_with(&self, t: &Q, grown: &mut Grown) -> Encl {
        let mut sum = Encl::exact(Q::ZERO);
        for term in &self.terms {
            let g = grown.of(&term.rate, t);
            let tp = (0..term.power).fold(Q::ONE, |acc, _| &acc * t);
            sum = sum.add(&term.coef.mul(&g).mul(&Encl::exact(tp)));
        }
        sum
    }
}

/// A cache of `e^(-rate t)` by rate, for one `t`; each rate is evaluated
/// once.
#[derive(Default)]
pub struct Grown {
    at: Vec<(Q, Encl)>,
}

impl Grown {
    fn of(&mut self, rate: &Q, t: &Q) -> Encl {
        if let Some((_, e)) = self.at.iter().find(|(r, _)| r == rate) {
            return e.clone();
        }
        let e = Boost::e(Q::ONE, -rate.clone()).grown(t);
        self.at.push((rate.clone(), e.clone()));
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certified::exp;

    /// A relaxation at a term's own rate integrates its power: the term
    /// `c t^m e^(-k t)` relaxed at `k` is `c t^(m+1) / (m+1) e^(-k t)`, so
    /// `(m + 1)` times the new coefficient is the old one and the divisor
    /// `m + 1` is never zero.
    #[test]
    fn a_resonant_relaxation_integrates_the_power() {
        let k = crate::ratio!(3, 7);
        for m in [0u32, 1, 4, 11] {
            let c = Encl::exact(crate::ratio!(5, 2));
            let s = ExpSum {
                terms: vec![Term {
                    coef: c.clone(),
                    power: m,
                    rate: k.clone(),
                }],
            };
            let r = s.relaxed(&k);
            assert_eq!(r.terms.len(), 1);
            let t = &r.terms[0];
            assert_eq!((t.power, &t.rate), (m + 1, &k));
            let back = t.coef.mul(&Encl::exact(Q::int(i64::from(m) + 1)));
            assert!(back.contains(&c.lo), "m = {m}");
        }
    }

    #[test]
    fn a_mode_grown_and_its_reverse_compose_to_one() {
        for (n, d) in [(1, 3), (-7, 2), (250, 1), (-3000, 7), (2, 1)] {
            let x = Q::new(n, d).unwrap();
            for b in [
                Boost::e(Q::ONE, x.clone()),
                Boost::halving(Q::ONE, x.clone()),
            ] {
                let there = b.grown(&Q::ONE);
                let back = Boost {
                    rate: -x.clone(),
                    ..b.clone()
                }
                .grown(&Q::ONE);
                assert!(there.mul(&back).contains(&Q::ONE));
            }
            // Base e is the certified exponential.
            let e = Boost::e(Q::ONE, x.clone()).grown(&Q::ONE);
            let direct = exp(&x, 80);
            assert!(e.lo <= direct.hi && direct.lo <= e.hi);
        }
        // A rapidity past -4096 is held as the bound [0, 2^-4096].
        let far = Boost::halving(Q::ONE, Q::int(-5000)).grown(&Q::ONE);
        assert_eq!(far.lo, Q::ZERO);
        assert_eq!(far.hi, two_to(-4096).unwrap());
        // Whole halvings are exact powers of two.
        assert_eq!(
            Boost::halving(Q::ONE, Q::int(-5)).grown(&Q::ONE),
            Encl::exact(Q::new(1, 32).unwrap())
        );
    }
}
