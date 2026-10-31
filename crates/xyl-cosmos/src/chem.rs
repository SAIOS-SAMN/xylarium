//! The home galaxy's chemical evolution: each element's abundance in the
//! disc's gas at any radius and time, relative to the Sun's birth gas.
//!
//! Inputs: `bang::BigBang` (first stars, star efficiency, the founding's
//! light nuclei via `elements::founded`, read as [`Light`]), `galaxy`
//! (`birth_gyr` for each ring's first light, `density` for the disc profile,
//! `core_kpc`), `centre::galaxy_episode_myr` for the core's echo, and the
//! element table's channel shares (`elements::Channel`). Radii are kpc,
//! times Gyr after the Big Bang, surface densities suns per pc^2.
//!
//! The disc is [`RINGS`] rings of 0.5 kpc (a spatial grid). On each ring:
//! - gas falls in as a boost on the ring's first-light timescale, scaled to
//!   the structure's profile; a ring whose gas never lights forms no stars;
//! - stars form at `efficiency x Omega / 2 pi` of the gas, `Omega = v / r`;
//! - `prompt` (0.2) of what forms returns at once and `slow` (0.2) later,
//!   half over 0.3 Gyr and half over 3 Gyr, driven by one iterate of the
//!   star formation without it;
//! - core collapse and spallation make their elements at once; type Ia, AGB
//!   and mergers follow `t^-1` delay laws held as four log-spaced boosts
//!   after their minimum delay (e-folds rounded to 1/1024 Gyr);
//! - the fountain throws `loading` of what forms, gas with its metals, and
//!   lands it by a landing map (Archimedes' hat-box over one epicycle, not a
//!   diffusion); what lands inside the centre goes to the core and returns
//!   as jets from the core's reach over the galaxy's echo; fountain and jets
//!   are fed once onto each ring;
//! - metals leave the gas with the stars that lock them away.
//!
//! Each is a linear relaxation driven by boosts, so every [`Ring`] holds
//! sums of `c t^m e^(-lambda t)` with exact rates
//! (`xyl_core::motion::ExpSum`), built once per ring (in parallel) and read
//! at any time.
//!
//! Outputs: [`Model`] (the rings' closed forms), [`Chemistry`] with
//! [`Chemistry::relative`], [`Chemistry::relatives`], [`Chemistry::metals`],
//! [`Chemistry::lump`] (a birth cloud's merger share, Poisson by inverse
//! CDF) and [`Chemistry::birth_along`] (a star's birth time along its ring's
//! star formation). Constructors return `None` when the element table or the
//! calibration cannot be built.
//!
//! Calibration: yield scales are fitted once per process, on the first
//! history built, to the measured gas at `Rulers::r0` today: core collapse
//! from O, type Ia from Fe, AGB from N; mergers and spallation at scale 1.
//! Every later universe in the process uses the same scales. A galactic
//! core's recipe (`cores`) is applied to these abundances in `systems`.
//!
//! Not modelled: radial gas drift, halo winds and their return, and the
//! Toomre threshold.

use crate::bang::BigBang;
use crate::elements::{decimal, table, Channel, Element};
use std::sync::OnceLock;
use xyl_core::certified::{exp, ln, pi, Encl};
use xyl_core::dyadic::Dy;
use xyl_core::motion::ExpSum;
use xyl_core::Q;

const BITS: u32 = 72;
/// Number of radial rings on the galactic disc, each [`Rulers::dr`] (0.5 kpc)
/// wide, so the disc spans 0 to 15 kpc.
pub const RINGS: usize = 30;

fn d(s: &str) -> Q {
    decimal(s).unwrap_or(Q::ZERO)
}

/// The measured rulers.
pub struct Rulers;
impl Rulers {
    /// Width of one disc ring, 0.5 kpc.
    pub fn dr() -> Q {
        d("0.5")
    }
    /// The disc's total surface density at the reference radius `r0`, in solar
    /// masses per square parsec (M_sun pc^-2): 50. The profile across the
    /// rings is the structure's own crowding (galaxy::density), scaled to it.
    pub fn sigma_r0() -> Q {
        d("50")
    }
    /// What forming stars give back at once, and what the giants give back
    /// later (0.4 in all).
    pub fn prompt() -> Q {
        d("0.2")
    }
    /// The share of formed stellar mass the giants' winds return later (0.2),
    /// half over 0.3 Gyr and half over 3 Gyr.
    pub fn slow() -> Q {
        d("0.2")
    }
    /// The fountain's loading: the gas it throws per mass of stars formed
    /// (declared).
    pub fn loading() -> Q {
        d("1")
    }
    /// The gas's mean particle mass (proton masses): what sets the
    /// fountain's reach against air's, x = 28 / A read as the share
    /// 2x / (1 + x) (no root), at A = 1.3.
    pub fn gas_mu() -> Q {
        d("1.3")
    }
    /// Helium-to-metal enrichment ratio dY/dZ, dimensionless (2.0).
    pub fn dy_dz() -> Q {
        d("2.0")
    }
    /// The Sun's metal mass fraction Z, 0.0142.
    pub fn reference_metallicity() -> Q {
        d("0.0142")
    }
    /// The reference galaxy's age today, 13.8 Gyr after the Big Bang.
    pub fn today_gyr() -> Q {
        d("13.8")
    }
    /// The reference ring (kpc): the galaxy's half-holding reach
    /// (galaxy::r0), read from the founding's structure.
    pub fn r0() -> Q {
        crate::galaxy::r0()
    }
    /// The disc's flat rotation speed, 220 km/s.
    pub fn v_flat() -> Q {
        d("220")
    }
    /// The gas at R0 today, in dex against solar (the Cosmic Abundance Standard).
    pub fn ism_today_dex(sym: &str) -> Q {
        d(match sym {
            "C" => "-0.10",
            "N" => "-0.04",
            "O" => "0.07",
            "Mg" => "-0.04",
            "Si" => "-0.01",
            "Fe" => "0.02",
            _ => "0",
        })
    }
}

/// A channel's delay window (Gyr): its first delay and its last.
fn delay(c: Channel) -> (Q, Q) {
    match c {
        Channel::CoreCollapse | Channel::Spallation | Channel::BigBang => (Q::ZERO, Q::ZERO),
        Channel::TypeIa => (d("0.04"), d("14.0")),
        Channel::Agb => (d("0.1"), d("10.0")),
        Channel::Merger => (d("0.01"), d("14.0")),
    }
}

/// The evolved channels (the founding's share is uniform).
pub const CHANNELS: [Channel; 5] = [
    Channel::CoreCollapse,
    Channel::TypeIa,
    Channel::Agb,
    Channel::Merger,
    Channel::Spallation,
];

fn channel_index(c: Channel) -> Option<usize> {
    CHANNELS.iter().position(|&x| x == c)
}

/// A delay law t^-1 over [lo, hi], held as four boosts after `lo`: e-folds
/// log-spaced across the window, each a quarter of the mass (a log-even
/// mixture, as t^-1 is), the e-folds rounded to 1/1024 Gyr (declared).
fn delay_modes(lo: &Q, hi: &Q) -> Vec<(Q, Q)> {
    let ratio = Dy::from_q(&hi.checked_div(lo).unwrap_or(Q::ONE));
    (0..4)
        .map(|j| {
            // log-even: each e-fold an even turn of the window's logarithm
            let tau =
                Dy::from_q(lo) * (ratio.max(Dy::sci(1, -30)).log() * Dy::ratio(2 * j + 1, 8)).exp();
            let tau = Q::new((tau * Dy::int(1024)).trunc_i64().max(1), 1024).unwrap_or(Q::ONE);
            (Q::new(1, 4).unwrap_or(Q::ZERO), tau)
        })
        .collect()
}

/// The galaxy's parameters from its Big Bang: the first stars, each ring's
/// infall and the star efficiency.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Params {
    /// When the first stars shone, in Gyr after the Big Bang.
    pub first_stars: Q,
    /// Each ring's first light (Gyr after the Big Bang), by ring index
    /// (`galaxy::birth_gyr`); None where the ring's gas never lights and no
    /// stars form.
    pub infall: Vec<Option<Q>>,
    /// The star efficiency: the lit share of the gas, turned into stars each
    /// orbit (`BigBang::efficiency`).
    pub efficiency: Q,
}

impl Params {
    /// The parameters read off the Big Bang `bang`'s founding.
    pub fn of(bang: &BigBang) -> Params {
        Params {
            first_stars: bang.first_stars_gyr().mid(),
            infall: (0..RINGS)
                .map(|j| crate::galaxy::birth_gyr(&ring_mid(j)))
                .collect(),
            efficiency: bang.efficiency.clone(),
        }
    }
}

/// What a closed form has formed from 0 to `t`: each term `c t^p e^(-k t)`
/// integrated exactly, `c p! / k^(p+1) (1 - e^(-k t) sum_(j<=p) (k t)^j / j!)`,
/// or `c t^(p+1) / (p + 1)` where it does not decay.
fn formed_by(s: &ExpSum, t: Dy) -> Dy {
    s.terms
        .iter()
        .map(|term| {
            let c = Dy::from_q(&term.coef.mid());
            let k = Dy::from_q(&term.rate);
            let p = term.power;
            if k.signum() == 0 {
                let mut tp = t;
                for _ in 0..p {
                    tp = tp * t;
                }
                return c * tp / Dy::int(i64::from(p) + 1);
            }
            // sum_(j<=p) (k t)^j / j!, and p! / k^(p+1)
            let (mut series, mut each, mut fact, mut kp) = (Dy::ONE, Dy::ONE, Dy::ONE, k);
            for j in 1..=p {
                each = each * k * t / Dy::int(i64::from(j));
                series += each;
                fact = fact * Dy::int(i64::from(j));
                kp = kp * k;
            }
            c * fact / kp * (Dy::ONE - (-(k * t)).exp() * series)
        })
        .sum()
}

/// km/s per kpc read in 1/Gyr: 1000 m/s per km over the kiloparsec, times
/// the seconds in a Gyr (package::Si).
fn km_s_per_kpc_to_per_gyr() -> Q {
    use crate::package::Si;
    (&Q::int(1000) * &Si::gyr_s())
        .checked_div(&Si::kpc())
        .unwrap_or(Q::ZERO)
}

fn ring_mid(j: usize) -> Q {
    &Q::int(i64::try_from(2 * j + 1).unwrap_or(0)) * &d("0.25")
}

/// Ring `j`'s star-formation timescale (Gyr): its first light, where its
/// gas lights at all.
fn infall_gyr(p: &Params, j: usize) -> Option<Q> {
    p.infall
        .get(j)
        .cloned()
        .flatten()
        .map(|t| t.max(d("0.001")))
}

fn sigma_total(r: &Q) -> Encl {
    // the disc's profile is the galaxy's own (galaxy::density), scaled to
    // the reference ring's surface density
    crate::galaxy::density(r).mul(&Encl::exact(Rulers::sigma_r0()))
}

/// Where what ring `j` throws lands, `reach` the carrier's reach against
/// air's (`2x / (1 + x)` with `x = 28 / A`, the share in place of Graham's
/// root): each ring's share, and the share falling inside the centre (to the
/// core), exact; what lands past the disc leaves it.
///
/// A landing map from orbits, not a diffusion. The fountain throws gas at
/// `70 km/s` isotropically; the radial part `v cos(theta)` is uniform on
/// `[-v, v]` (Archimedes' hat-box). It rides one epicycle and lands
/// `v cos(theta) / kappa` from where it rose, `kappa = sqrt2 V / r`, scaled by
/// weight (`reach`). It therefore lands uniformly over `[r - L, r + L]`,
/// `L = (7 r / 22) reach`; each ring takes its overlap.
fn landing(reach: &Q, j: usize) -> ([Q; RINGS], Q) {
    let dr = Rulers::dr();
    let r = ring_mid(j).max(&dr * &d("0.5"));
    let l = &(&r * &Q::new(7, 22).unwrap_or(Q::ZERO)) * reach;
    let (from, to) = (&r - &l, &r + &l);
    let width = &to - &from;
    let share = |lo: Q, hi: Q| -> Q {
        let (lo, hi) = (lo.max(from.clone()), hi.min(to.clone()));
        if hi > lo {
            (&hi - &lo).checked_div(&width).unwrap_or(Q::ZERO)
        } else {
            Q::ZERO
        }
    };
    let rings = std::array::from_fn(|k| {
        let lo = &Q::int(i64::try_from(k).unwrap_or(0)) * &dr;
        share(lo.clone(), &lo + &dr)
    });
    (rings, share(from.clone(), Q::ZERO))
}

/// One ring's closed forms: its gas (surface density) and star formation in
/// time since the first stars, and for each channel what its stars make and
/// the metals it holds per unit yield, in time since that channel's first
/// delay.
pub struct Ring {
    /// Gas surface density, in solar masses per pc^2, as a closed form in Gyr
    /// since the first stars.
    pub gas: ExpSum,
    /// Star-formation rate, in solar masses per pc^2 per Gyr, as a closed
    /// form in Gyr since the first stars.
    pub sfr: ExpSum,
    /// Per channel (indexed as [`CHANNELS`]), what the ring's own stars make
    /// at unit yield, as a closed form in Gyr since that channel's first delay.
    pub landed: Vec<ExpSum>,
    /// Per channel (indexed as [`CHANNELS`]), the metals the ring's gas holds
    /// at unit yield (own making plus the fountain and the core's jets, less
    /// what stars lock away), in Gyr since that channel's first delay.
    pub metal: Vec<ExpSum>,
}

/// A galaxy's chemistry: every ring's closed forms.
pub struct Model {
    /// The history parameters the model was built from.
    pub params: Params,
    /// The disc's rings, innermost first, [`RINGS`] of them.
    pub rings: Vec<Ring>,
}

/// `f` over `0..n` on `XYL_THREADS` threads (default: all cores), results in
/// order. The rings are independent, so each is built whole.
fn par_map<T: Send>(n: usize, f: impl Fn(usize) -> T + Sync) -> Vec<T> {
    let cores = std::thread::available_parallelism()
        .map(|c| c.get())
        .unwrap_or(1);
    let threads = std::env::var("XYL_THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(cores)
        .clamp(1, n.max(1));
    let next = std::sync::atomic::AtomicUsize::new(0);
    let slots: Vec<std::sync::Mutex<Option<T>>> =
        (0..n).map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if k >= n {
                    break;
                }
                let built = f(k);
                if let Ok(mut slot) = slots[k].lock() {
                    *slot = Some(built);
                }
            });
        }
    });
    slots
        .into_iter()
        .filter_map(|s| s.into_inner().ok().flatten())
        .collect()
}

impl Model {
    /// Builds every ring's closed forms for the history `p`: infall, star
    /// formation, the giants' slow return, each channel's making, the
    /// fountain's landing map and the core's jets (each fed once).
    pub fn of(p: &Params) -> Model {
        let today = Rulers::today_gyr();
        let span = &today - &p.first_stars;
        let e = |q: Q| Encl::exact(q);
        let rate_of = |tau: &Q| Q::ONE.checked_div(tau).unwrap_or(Q::ONE);
        // a channel's making from a star formation (unit yield), in time since
        // its first delay: at once, or through its four delay boosts
        let delayed = |c: Channel, sfr: &ExpSum| -> ExpSum {
            let (lo, hi) = delay(c);
            if lo.is_zero() {
                return sfr.clone();
            }
            delay_modes(&lo, &hi)
                .into_iter()
                .fold(ExpSum::default(), |acc, (w, tau)| {
                    let rate = rate_of(&tau);
                    acc.plus(&sfr.relaxed(&rate).scaled(&e(&w * &rate)))
                })
        };
        // Each ring alone: its infall, its giants' slow return (one iterate),
        // and the fountain leaving it (`loading` of what forms)
        struct Alone {
            feed: ExpSum,
            gas: ExpSum,
            form: Q,
            deplete: Q,
            metal: Vec<ExpSum>,
        }
        let alone: Vec<Alone> = par_map(RINGS, |j| {
            let r = ring_mid(j);
            // a ring whose gas never lights forms no stars
            let (tau, lights) = match infall_gyr(p, j) {
                Some(t) => (t, true),
                None => (Q::ONE, false),
            };
            let lam = rate_of(&tau);
            let norm = e(Q::ONE).sub(&exp(&-(span.checked_div(&tau).unwrap_or(Q::ZERO)), BITS));
            let a = sigma_total(&r)
                .mul(&e(lam.clone()))
                .mul(&norm.recip().unwrap_or_else(|| e(Q::ONE)));
            // stars form at the lit share of the gas each orbit: the
            // efficiency times Omega / 2 pi (Omega in 1/Gyr: km/s per kpc
            // times 1000 x gyr_s / kpc)
            let omega = (&Rulers::v_flat() * &km_s_per_kpc_to_per_gyr())
                .checked_div(&r.clone().max(d("0.25")))
                .unwrap_or(Q::ZERO);
            let form = if lights {
                (&p.efficiency * &omega)
                    .checked_div(&(&Q::int(2) * &pi(80).mid()))
                    .unwrap_or(Q::ZERO)
            } else {
                Q::ZERO
            };
            let deplete = &(&(&Q::ONE - &Rulers::prompt()) + &Rulers::loading()) * &form;
            let infall = ExpSum::decay(a.outward(120), lam);
            let first = infall.relaxed(&deplete).scaled(&e(form.clone()));
            let slow = [(d("0.3"), d("0.5")), (d("3"), d("0.5"))].into_iter().fold(
                ExpSum::default(),
                |acc, (tau, share)| {
                    let rate = rate_of(&tau);
                    acc.plus(
                        &first
                            .relaxed(&rate)
                            .scaled(&e(&(&Rulers::slow() * &share) * &rate)),
                    )
                },
            );
            let feed = infall.plus(&slow);
            let gas = feed.relaxed(&deplete);
            let sfr = gas.scaled(&e(form.clone()));
            let metal = CHANNELS
                .iter()
                .map(|&c| delayed(c, &sfr).relaxed(&deplete))
                .collect();
            Alone {
                feed,
                gas,
                form,
                deplete,
                metal,
            }
        });
        // The fountain: each ring throws `loading` of what it forms, gas with
        // its own metals, landing by the orbital map at the gas's reach.
        // The reach against air's (x = 28 / molecular weight) as a share,
        // 2x / (1 + x), not a square root
        let reach = {
            let x = Q::int(28).checked_div(&Rulers::gas_mu()).unwrap_or(Q::ONE);
            (&Q::int(2) * &x)
                .checked_div(&(&Q::ONE + &x))
                .unwrap_or(Q::ONE)
        };
        let maps: Vec<([Q; RINGS], Q)> = (0..RINGS).map(|j| landing(&reach, j)).collect();
        let thrown_gas: Vec<ExpSum> = alone
            .iter()
            .map(|x| x.gas.scaled(&e(&Rulers::loading() * &x.form)))
            .collect();
        let thrown_metal: Vec<Vec<ExpSum>> = alone
            .iter()
            .map(|x| {
                x.metal
                    .iter()
                    .map(|m| m.scaled(&e(&Rulers::loading() * &x.form)))
                    .collect()
            })
            .collect();
        // ring j's mass weight (its area, pi (2j+1)/4 kpc^2, up to pi/4)
        let area = |j: usize| Q::int(i64::try_from(2 * j + 1).unwrap_or(1));
        // The core recycles (Einstein-Cartan bounce): what the fountain drops
        // inside the centre falls to the core, which returns it as jets
        // landing from the core's reach over the galaxy's own echo
        // (centre::galaxy_episode_myr); the jets' own share falling back in
        // stays held
        let edge = ring_of(&crate::galaxy::core_kpc());
        let back_in = maps[edge].1.clone();
        let echo_gyr = crate::centre::galaxy_episode_myr()
            .checked_div(&Q::int(1000))
            .unwrap_or(Q::ZERO);
        let release = &rate_of(&echo_gyr) * &(&Q::ONE - &back_in);
        let into_core = |flows: &dyn Fn(usize) -> ExpSum| -> ExpSum {
            (0..RINGS).fold(ExpSum::default(), |acc, j| {
                let inner = &maps[j].1;
                if inner.is_zero() {
                    return acc;
                }
                acc.plus(&flows(j).scaled(&e(inner * &area(j))))
            })
        };
        let core_gas = into_core(&|j| thrown_gas[j].clone()).relaxed(&release);
        let core_metal: Vec<ExpSum> = par_map(CHANNELS.len(), |ci| {
            into_core(&|j| thrown_metal[j][ci].clone()).relaxed(&release)
        });
        let jets = rate_of(&echo_gyr);
        // what lands on ring k: every ring's fountain, and the core's jets
        let lands = |k: usize, thrown: &dyn Fn(usize) -> ExpSum, core: &ExpSum| -> ExpSum {
            let ak = area(k);
            let from_rings = (0..RINGS).fold(ExpSum::default(), |acc, j| {
                let share = &maps[j].0[k];
                if share.is_zero() {
                    return acc;
                }
                acc.plus(
                    &thrown(j).scaled(&e((share * &area(j)).checked_div(&ak).unwrap_or(Q::ZERO))),
                )
            });
            let jet_share = &maps[edge].0[k];
            if jet_share.is_zero() {
                return from_rings;
            }
            from_rings
                .plus(&core.scaled(&e((&jets * jet_share).checked_div(&ak).unwrap_or(Q::ZERO))))
        };
        // Each ring with the flows: its own feed, the fountain and the jets
        // landing on it (fed once), its star formation and its metals
        let ring = |k: usize| -> Ring {
            let x = &alone[k];
            let gas = x
                .feed
                .plus(&lands(k, &|j| thrown_gas[j].clone(), &core_gas))
                .relaxed(&x.deplete);
            let sfr = gas.scaled(&e(x.form.clone()));
            let made: Vec<ExpSum> = CHANNELS.iter().map(|&c| delayed(c, &sfr)).collect();
            let metal = (0..CHANNELS.len())
                .map(|ci| {
                    made[ci]
                        .plus(&lands(k, &|j| thrown_metal[j][ci].clone(), &core_metal[ci]))
                        .relaxed(&x.deplete)
                })
                .collect();
            Ring {
                gas,
                sfr,
                landed: made,
                metal,
            }
        };
        // the rings are independent once the flows are known
        let rings = par_map(RINGS, ring);
        Model {
            params: p.clone(),
            rings,
        }
    }

    /// Every channel's metals per gas (unit yield) on ring `k` at `t` Gyr,
    /// each read once; None before the first stars or where there is no gas.
    fn concs(&self, k: usize, t: &Q) -> Option<Vec<Q>> {
        let dt = t - &self.params.first_stars;
        if dt.signum() <= 0 {
            return None;
        }
        let ring = self.rings.get(k)?;
        let gas = ring.gas.at(&dt).mid();
        if gas.signum() <= 0 {
            return None;
        }
        // channels sharing a delay share their times, and so every rate's
        // exponential, read once per time
        let mut by_time: Vec<(Q, xyl_core::motion::Grown)> = Vec::new();
        Some(
            CHANNELS
                .iter()
                .enumerate()
                .map(|(ci, &c)| {
                    let s = &dt - &delay(c).0;
                    if s.signum() <= 0 {
                        return Q::ZERO;
                    }
                    let slot = match by_time.iter().position(|(t, _)| t == &s) {
                        Some(i) => i,
                        None => {
                            by_time.push((s.clone(), xyl_core::motion::Grown::default()));
                            by_time.len() - 1
                        }
                    };
                    ring.metal[ci]
                        .at_with(&s, &mut by_time[slot].1)
                        .mid()
                        .checked_div(&gas)
                        .unwrap_or(Q::ZERO)
                })
                .collect(),
        )
    }
}

/// ln 10, as a rational inside its enclosure.
pub fn ln10() -> Q {
    ln(&Q::int(10), BITS).map(|e| e.mid()).unwrap_or(Q::ONE)
}

/// Which ring holds radius `r`.
pub fn ring_of(r: &Q) -> usize {
    let j = (r * &Q::int(2)).floor_i128().unwrap_or(0).max(0);
    usize::try_from(j).unwrap_or(0).min(RINGS - 1)
}

/// The calibration: each channel's yield scale (from the measured gas
/// rulers) and its unit-yield concentration at R0 today, on the universe's
/// own history (one universe per process: the first history calibrates).
pub struct Calibration {
    elements: Vec<Element>,
    model: Model,
    scale: Vec<Q>,
    reference: Vec<Q>,
    /// Each element's abundance in the Sun's birth gas (R0, 9.2 Gyr, the
    /// reference Big Bang) under this calibration; rulers measured on the
    /// Sun's own matter (an isotope inventory) are read against it.
    sun: Vec<Q>,
}

static CAL: OnceLock<Option<Calibration>> = OnceLock::new();

impl Calibration {
    fn build(p: &Params) -> Option<Calibration> {
        let elements = table().ok()?;
        let model = Model::of(p);
        let k0 = ring_of(&Rulers::r0());
        let today = Rulers::today_gyr();
        let reference = model
            .concs(k0, &today)
            .unwrap_or_else(|| vec![Q::ONE; CHANNELS.len()]);
        let t = |s: &str| -> Option<Q> {
            let x = &Rulers::ism_today_dex(s) * &ln(&Q::int(10), BITS)?.mid();
            Some(exp(&x, BITS).mid())
        };
        let origin = |s: &str, c: Channel| {
            elements
                .iter()
                .find(|e| e.sym == s)
                .and_then(|e| {
                    e.origin
                        .iter()
                        .find(|(x, _)| *x == c)
                        .map(|(_, f)| f.clone())
                })
                .unwrap_or(Q::ZERO)
        };
        let k_cc = t("O")?;
        let k_ia = (&t("Fe")? - &(&origin("Fe", Channel::CoreCollapse) * &k_cc))
            .checked_div(&origin("Fe", Channel::TypeIa))?;
        let k_agb = (&t("N")? - &(&origin("N", Channel::CoreCollapse) * &k_cc))
            .checked_div(&origin("N", Channel::Agb))?;
        let scale = CHANNELS
            .iter()
            .map(|&c| match c {
                Channel::CoreCollapse => k_cc.clone(),
                Channel::TypeIa => k_ia.clone(),
                Channel::Agb => k_agb.clone(),
                _ => Q::ONE,
            })
            .collect();
        let mut cal = Calibration {
            elements,
            model,
            scale,
            reference,
            sun: Vec::new(),
        };
        // the Sun was born 9.2 Gyr after the Big Bang (13.8 - 4.6)
        let at_birth = cal.model.concs(k0, &d("9.2"));
        let sun = (0..cal.elements.len())
            .map(|ei| {
                cal.parts(at_birth.as_deref(), ei)
                    .map(|(sm, mg)| &sm + &mg)
                    .unwrap_or(Q::ZERO)
            })
            .collect();
        cal.sun = sun;
        Some(cal)
    }

    /// The calibration on the history `p`, built once per process and
    /// shared. Returns `None` when the element table fails to load or a
    /// channel's yield scale cannot be solved (a zero origin share for Fe
    /// or N).
    pub fn of(p: &Params) -> Option<&'static Calibration> {
        CAL.get_or_init(|| Calibration::build(p)).as_ref()
    }

    /// The calibration already built in this process, if any.
    fn held() -> Option<&'static Calibration> {
        CAL.get().and_then(|c| c.as_ref())
    }

    /// Element `ei`'s (smooth, merger) relative parts from the channels'
    /// concentrations `concs` (None: nothing made yet there and then): each
    /// channel's share of it, its yield scale, and its concentration against
    /// the reference's.
    fn parts(&self, concs: Option<&[Q]>, ei: usize) -> Option<(Q, Q)> {
        let e = self.elements.get(ei)?;
        let (mut smooth, mut merger) = (Q::ZERO, Q::ZERO);
        for (c, f) in &e.origin {
            if *c == Channel::BigBang {
                smooth = &smooth + f;
                continue;
            }
            let Some(ci) = channel_index(*c) else {
                continue;
            };
            // nothing made yet here and then: none of it
            let Some(concs) = concs else {
                return Some((Q::ZERO, Q::ZERO));
            };
            let v = (&(f * &self.scale[ci]) * &concs[ci])
                .checked_div(&self.reference[ci])
                .unwrap_or(Q::ZERO);
            if *c == Channel::Merger {
                merger = &merger + &v;
            } else {
                smooth = &smooth + &v;
            }
        }
        Some((smooth, merger))
    }
}

/// An element's abundance in the gas the Sun formed from, as the calibration
/// reads it (R0, 9.2 Gyr): 1 for an element the Big Bang alone made. None
/// before a chemistry has been built in the process.
pub fn reference_birth_gas(sym: &str) -> Option<Q> {
    let cal = Calibration::held()?;
    let ei = cal.elements.iter().position(|e| e.sym == sym)?;
    cal.sun.get(ei).cloned()
}

/// The birth gas's light nuclei, from the founding's own abundances
/// (`elements::founded`): hydrogen's and helium's mass shares from their
/// counts and the table's masses, and the lighter kinds by count against
/// hydrogen as the table gives them (zero where it holds none).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Light {
    /// Hydrogen's mass share of hydrogen and helium, `1 - y`.
    pub x: Q,
    /// Helium's mass share of hydrogen and helium.
    pub y: Q,
    /// Deuterium to hydrogen by count (the table holds no isotope rows, so
    /// zero until it does).
    pub d_h: Q,
    /// Lithium to hydrogen by count, from the founding's Z = 3 column.
    pub li_h: Q,
}

impl Light {
    /// The light nuclei of the gas the founding of `bang` seeds. `None` when
    /// the table cannot be built or hydrogen and helium carry no mass.
    pub fn of(bang: &BigBang) -> Option<Light> {
        let table = crate::elements::founded(&bang.parent.share()).ok()?;
        let by = |sym: &str| table.iter().find(|e| e.sym == sym);
        let (h, he) = (by("H")?, by("He")?);
        let (m_h, m_he) = (&h.abundance * &h.a, &he.abundance * &he.a);
        let y = m_he.checked_div(&(&m_h + &m_he))?;
        let per_h = |sym: &str| {
            by(sym)
                .and_then(|e| e.abundance.checked_div(&h.abundance))
                .unwrap_or(Q::ZERO)
        };
        Some(Light {
            x: &Q::ONE - &y,
            y,
            d_h: per_h("D"),
            li_h: per_h("Li"),
        })
    }
}

/// A universe's chemistry: its galaxy's closed forms, read against the reference
/// calibration.
pub struct Chemistry {
    /// The universe's galaxy model, built from its Big Bang's parameters.
    pub model: Model,
    /// The universe's light nuclei from its founding.
    pub light: Light,
}

impl Chemistry {
    /// The process's chemistry: the first founded universe's (as the anchor
    /// is, the first founding in the process wins), built once and kept;
    /// before the universe is founded it is built but not kept.
    pub fn cached(bang: &BigBang) -> Option<&'static Chemistry> {
        static C: OnceLock<Chemistry> = OnceLock::new();
        if let Some(c) = C.get() {
            return Some(c);
        }
        let c = Chemistry::of(bang)?;
        if crate::circuit::anchor().rim.signum() > 0 {
            let _ = C.set(c);
        }
        C.get()
    }

    /// A star's birth (Gyr after the Big Bang) on its ring's own star
    /// formation: the time by which the share `u` of the ring's stars formed
    /// from `from` (its first light) to the horizon have formed, read from the
    /// ring's closed form (`Ring::sfr`, integrated term by term) and found by
    /// halving. A ring with no star formation after `from` gives `from`.
    pub fn birth_along(&self, r: &Q, from: &Q, u: Dy) -> Q {
        let Some(ring) = self.model.rings.get(ring_of(r)) else {
            return from.clone();
        };
        let first = &self.model.params.first_stars;
        let t0 = Dy::from_q(&(from - first)).max(Dy::ZERO);
        let top = crate::life::horizon_gyr() - Dy::from_q(first);
        if top <= t0 {
            return from.clone();
        }
        let (c0, c1) = (formed_by(&ring.sfr, t0), formed_by(&ring.sfr, top));
        if c1 <= c0 {
            return from.clone();
        }
        let target = c0 + u.max(Dy::ZERO).min(Dy::ONE) * (c1 - c0);
        let (mut lo, mut hi) = (t0, top);
        for _ in 0..48 {
            let mid = (lo + hi) / Dy::int(2);
            if formed_by(&ring.sfr, mid) < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        first + &hi.to_q()
    }

    /// The chemistry of the universe whose Big Bang is `bang`. Returns `None`
    /// when the calibration cannot be built or the founding's light nuclei
    /// cannot be read (see [`Light::of`]).
    pub fn of(bang: &BigBang) -> Option<Chemistry> {
        let p = Params::of(bang);
        // the calibration on the universe's own history
        Calibration::of(&p)?;
        Some(Chemistry {
            model: Model::of(&p),
            light: Light::of(bang)?,
        })
    }

    /// An element's abundance in the gas at radius `r`, epoch `t`, relative to
    /// the Sun's birth gas (1 there): what a star and its worlds born there and
    /// then are made of. `lump` scales the mergers' share (a star's own cloud).
    pub fn relative(&self, sym: &str, r: &Q, t: &Q, lump: &Q) -> Option<Q> {
        let cal = Calibration::held()?;
        let ei = cal.elements.iter().position(|e| e.sym == sym)?;
        if sym == "H" || sym == "He" {
            return self.light(sym, &self.metals(r, t)?);
        }
        let (smooth, merger) = cal.parts(self.model.concs(ring_of(r), t).as_deref(), ei)?;
        Some(&smooth + &(&merger * lump))
    }

    /// Every element's abundance at `r`, `t` (as `relative`), the channels
    /// read once for all of them: what a star's birth gas is made of.
    pub fn relatives(&self, r: &Q, t: &Q, lump: &Q) -> Option<Vec<Q>> {
        let cal = Calibration::held()?;
        let concs = self.model.concs(ring_of(r), t);
        let z = Self::metals_of(cal, concs.as_deref())?;
        Some(
            cal.elements
                .iter()
                .enumerate()
                .map(|(ei, e)| {
                    if e.sym == "H" || e.sym == "He" {
                        return self.light(&e.sym, &z).unwrap_or(Q::ZERO);
                    }
                    cal.parts(concs.as_deref(), ei)
                        .map(|(sm, mg)| &sm + &(&mg * lump))
                        .unwrap_or(Q::ZERO)
                })
                .collect(),
        )
    }

    /// Hydrogen's or helium's abundance relative to the Sun's birth gas, from
    /// the metals `z` (relative): what burning made of the Big Bang's.
    fn light(&self, sym: &str, z: &Q) -> Option<Q> {
        let z = z * &Rulers::reference_metallicity();
        let xp = &self.light.x;
        let burn = &Q::ONE + &Rulers::dy_dz();
        if sym == "H" {
            (xp - &(&burn * &z)).checked_div(&(xp - &(&burn * &Rulers::reference_metallicity())))
        } else {
            let y0 = &self.light.y;
            (y0 + &(&Rulers::dy_dz() * &z))
                .checked_div(&(y0 + &(&Rulers::dy_dz() * &Rulers::reference_metallicity())))
        }
    }

    /// The share of the mergers' smooth expectation a star's birth cloud holds
    /// (enrichment.rprocess_lump), for its own draw `u` in [0, 1): the long-mixed
    /// part, plus the recent part scaled by how many mergers actually struck its
    /// patch (about 300 pc over 100 Myr), Poisson about the expected number.
    pub fn lump(&self, r: &Q, t: &Q, u: &Q) -> Option<Q> {
        let ci = channel_index(Channel::Merger)?;
        let k = ring_of(r);
        let ring = self.model.rings.get(k)?;
        let dt = t - &self.model.params.first_stars;
        let s = &dt - &delay(Channel::Merger).0;
        if s.signum() <= 0 {
            return Some(Q::ZERO);
        }
        let held = ring.metal[ci].at(&s).mid();
        let out = &ring.landed[ci].at(&s).mid() * &d("0.1");
        if held.signum() <= 0 {
            return Some(if out.signum() <= 0 { Q::ZERO } else { Q::ONE });
        }
        let recent = out.checked_div(&held)?.min(Q::ONE);
        // stars formed in its patch: sfr (suns/pc^2/Gyr) x pi (300 pc)^2 x 0.1 Gyr
        let sfr = ring.sfr.at(&dt).mid();
        let formed = &(&sfr * &pi(BITS).mid()) * &d("9000");
        let expect = &d("2.0e-5") * &formed;
        if expect.signum() <= 0 {
            return Some(&Q::ONE - &recent);
        }
        // the Poisson count at u, by its inverse CDF
        let p0 = exp(&-expect.clone(), BITS).mid();
        let (mut kk, mut p, mut cum) = (0i64, p0.clone(), p0);
        while u > &cum && kk < 1000 {
            kk += 1;
            p = (&p * &expect).checked_div(&Q::int(kk))?;
            cum = &cum + &p;
        }
        Some(&(&Q::ONE - &recent) + &(&recent * &Q::int(kk)).checked_div(&expect)?)
    }

    /// Metallicity relative to the Sun's birth gas: the mass-weighted relative
    /// abundance of what metals are mostly made of (O, C, Fe, Si, Mg, N).
    pub fn metals(&self, r: &Q, t: &Q) -> Option<Q> {
        let cal = Calibration::held()?;
        Self::metals_of(cal, self.model.concs(ring_of(r), t).as_deref())
    }

    /// The metallicity from the channels' concentrations, read once.
    fn metals_of(cal: &Calibration, concs: Option<&[Q]>) -> Option<Q> {
        let (mut w, mut tot) = (Q::ZERO, Q::ZERO);
        for s in ["O", "C", "Fe", "Si", "Mg", "N"] {
            let ei = cal.elements.iter().position(|e| e.sym == s)?;
            let e = &cal.elements[ei];
            let wt = &e.a * &e.abundance;
            let (sm, mg) = cal.parts(concs, ei)?;
            tot = &tot + &(&wt * &(&sm + &mg));
            w = &w + &wt;
        }
        tot.checked_div(&w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_books_close_and_metals_follow_the_first_stars() {
        let bang = crate::genesis::bang();
        let chem = crate::genesis::chem();
        let today = Rulers::today_gyr();
        // the books close: everything made is somewhere
        let o = chem.relative("O", &Rulers::r0(), &today, &Q::ONE).unwrap();
        assert!(o.signum() >= 0, "O at R0 today: {o}");
        // the galaxy has made metals by today where its gas lit
        let made = chem.metals(&Rulers::r0(), &today).unwrap();
        assert!(made.signum() >= 0, "metals {made}");
        // before the first stars (half their epoch, the founding's own),
        // nothing but the Big Bang's
        let before = bang.first_stars_gyr().lo.checked_div(&Q::int(2)).unwrap();
        assert_eq!(
            chem.relative("U", &Rulers::r0(), &before, &Q::ONE).unwrap(),
            Q::ZERO
        );
    }
}
