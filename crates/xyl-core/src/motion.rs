//! Motion: closed-form orbits of exponential modes, evaluated at any time `t`
//! in one step, with no loop over time.
//!
//! Inputs: exact amplitudes, rates and times ([`Q`]) in the caller's units;
//! uses `certified` for `ln 2` and the fixed-point `exp`.
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

use crate::certified::{exp_fixed, exp_of, fixed_bounds, ln2, Encl, FIXED};
use crate::Q;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive};
use std::sync::OnceLock;

/// The enclosure of ln 2 to 2^-100, computed once; finer than any reading
/// requires.
fn ln2_held() -> &'static Encl {
    static LN2: OnceLock<Encl> = OnceLock::new();
    LN2.get_or_init(|| ln2(100).outward(100))
}

/// The lower and upper bounds of ln 2 in units of 2^-60.
fn ln2_fixed() -> (i128, i128) {
    static L: OnceLock<(i128, i128)> = OnceLock::new();
    *L.get_or_init(|| {
        (
            fixed_bounds(&ln2_held().lo).0,
            fixed_bounds(&ln2_held().hi).1,
        )
    })
}

/// 2^k, exact.
fn pow2(k: &BigInt) -> Q {
    let n = k.abs().to_usize().unwrap_or(0);
    let p = BigInt::one() << n;
    let r = if k.is_negative() {
        BigRational::new(BigInt::one(), p)
    } else {
        BigRational::from_integer(p)
    };
    Q::from_big(r)
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
            Base::Two => Encl::exact(self.rate.clone()).mul(ln2_held()),
        }
    }

    /// `base^(rate t)`, enclosed: the whole halvings of the rapidity as an
    /// exact power of two, and only the remainder (under two halvings)
    /// through the series, in fixed-point integers bounded outward
    /// (`exp_fixed`). A rapidity above 4096 goes through `exp_of` directly.
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
                hi: pow2(&BigInt::from(-4096)),
            };
        }
        if x > far {
            return exp_of(&Encl::exact(x).mul(&self.efolds()), FIXED);
        }
        let (l_lo, l_hi) = ln2_fixed();
        let (k, a_lo, a_hi) = match self.base {
            Base::Two => {
                // `x = k + f` with f in [0, 1); f ln 2 is bounded by the products
                // of the fixed-point bounds of f and of ln 2.
                let k = x.floor();
                let f = &x - &Q::from_reduced(BigRational::from_integer(k.clone()));
                let (f_lo, f_hi) = fixed_bounds(&f);
                (k, (f_lo * l_lo) >> FIXED, ((f_hi * l_hi) >> FIXED) + 1)
            }
            Base::E => {
                // `x = k ln 2 + r` with r in [0, 2 ln 2).
                let (x_lo, x_hi) = fixed_bounds(&x);
                let mut k = x_lo.div_euclid(l_hi);
                let r = |k: i128| {
                    (
                        x_lo - (k * l_lo).max(k * l_hi),
                        x_hi - (k * l_lo).min(k * l_hi),
                    )
                };
                let (mut r_lo, mut r_hi) = r(k);
                if r_lo < 0 {
                    k -= 1;
                    (r_lo, r_hi) = r(k);
                }
                (BigInt::from(k), r_lo, r_hi)
            }
        };
        Encl::exact(pow2(&k)).mul(&exp_fixed(a_lo, a_hi))
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
            if a.is_zero() {
                out.push(Term {
                    coef: t
                        .coef
                        .mul(&Encl::exact(
                            Q::ONE
                                .checked_div(&Q::int(i64::from(m) + 1))
                                .unwrap_or(Q::ONE),
                        ))
                        .outward(KEEP),
                    power: m + 1,
                    rate: k.clone(),
                });
                continue;
            }
            let inv = Q::ONE.checked_div(&a).unwrap_or(Q::ZERO);
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
    at: std::collections::BTreeMap<Q, Encl>,
}

impl Grown {
    fn of(&mut self, rate: &Q, t: &Q) -> Encl {
        self.at
            .entry(rate.clone())
            .or_insert_with(|| Boost::e(Q::ONE, -rate.clone()).grown(t))
            .clone()
    }
}
