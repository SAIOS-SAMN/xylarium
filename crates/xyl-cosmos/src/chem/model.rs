//! The rings' closed forms: each ring alone (infall, star formation, the
//! giants' slow return, each channel's making), then the fountain's landing
//! map and the core's jets fed once onto every ring.

use super::{
    delay, delay_modes, infall_gyr, km_s_per_kpc_to_per_gyr, ring_mid, ring_of, sigma_total,
    Params, Rulers, BITS, CHANNELS, RINGS,
};
use crate::elements::{decimal, Channel};
use xyl_core::certified::{exp, pi, Encl};
use xyl_core::motion::ExpSum;
use xyl_core::Q;

/// Where the gas thrown by ring `j` lands, with `reach` the carrier's reach
/// relative to air (`2x / (1 + x)` with `x = 28 / A`, the share in place of
/// Graham's root): each ring's share, and the share falling inside the
/// centre (r < 0, to the core), exact; what lands past the disc leaves it.
///
/// A landing map from orbits, not a diffusion. The fountain throws gas at
/// `v = 70 km/s` isotropically; the radial part `v cos(theta)` is uniform on
/// `[-v, v]` (Archimedes' hat-box). The gas follows one epicycle and lands
/// `v cos(theta) / kappa` from where it rose, with `kappa` taken as `V / r`
/// (`V = 220 km/s`), scaled by `reach`. It therefore lands uniformly over
/// `[r - L, r + L]`, `L = (7 r / 22) reach`, with r at least 0.25 kpc; each
/// ring takes its overlap.
fn landing(reach: &Q, j: usize) -> ([Q; RINGS], Q) {
    let dr = Rulers::dr();
    let r = ring_mid(j).max(&dr * &decimal("0.5").unwrap_or(Q::ZERO));
    let l = &(&r * &Q::ratio(7, 22)) * reach;
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
    /// Per channel (indexed as [`CHANNELS`]), what the ring's stars make at
    /// unit yield, as a closed form in Gyr since that channel's first delay.
    pub landed: Vec<ExpSum>,
    /// Per channel (indexed as [`CHANNELS`]), the metals the ring's gas holds
    /// at unit yield (the ring's making plus the fountain and the core's
    /// jets, less what stars lock away), in Gyr since that channel's first
    /// delay.
    pub metal: Vec<ExpSum>,
}

/// A galaxy's chemistry: every ring's closed forms.
pub struct Model {
    /// The history parameters the model was built from.
    pub params: Params,
    /// The disc's rings, innermost first, [`RINGS`] of them.
    pub rings: Vec<Ring>,
}

/// `f` over `0..n` on `XYL_THREADS` threads (default: the available
/// parallelism, at most `n`), results in order. The rings are independent,
/// so each is built whole on one thread.
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

/// The rate of an e-fold time `tau` (1 / tau; 1 where `tau` is zero).
fn rate_of(tau: &Q) -> Q {
    Q::ONE.checked_div(tau).unwrap_or(Q::ONE)
}

/// A channel's making from a star formation `sfr` (unit yield), in time
/// since its first delay: `sfr` itself for a prompt channel, otherwise the
/// sum over its four delay boosts of `w k (sfr relaxed at k)`, `k = 1 / tau`.
fn delayed(c: Channel, sfr: &ExpSum) -> ExpSum {
    let (lo, hi) = delay(c);
    if lo.is_zero() {
        return sfr.clone();
    }
    delay_modes(&lo, &hi)
        .into_iter()
        .fold(ExpSum::default(), |acc, (w, tau)| {
            let rate = rate_of(&tau);
            acc.plus(&sfr.relaxed(&rate).scaled(&Encl::exact(&w * &rate)))
        })
}

/// A ring before the flows between rings: its feed (infall plus the giants'
/// slow return), gas, star-formation coefficient, depletion rate (what forms,
/// less the prompt return, plus the fountain's loading) and per-channel
/// metals.
struct Alone {
    feed: ExpSum,
    gas: ExpSum,
    form: Q,
    deplete: Q,
    metal: Vec<ExpSum>,
}

/// Ring `j` alone over the history `p`, `span` Gyr from the first stars to
/// today.
fn ring_alone(p: &Params, span: &Q, j: usize) -> Alone {
    let r = ring_mid(j);
    // A ring whose gas never lights forms no stars.
    let (tau, lights) = match infall_gyr(p, j) {
        Some(t) => (t, true),
        None => (Q::ONE, false),
    };
    let lam = rate_of(&tau);
    // Infall a e^(-lam t), normalised so that its integral to today is the
    // ring's surface density: a = sigma lam / (1 - e^(-span / tau)).
    let norm = Encl::exact(Q::ONE).sub(&exp(&-(span.checked_div(&tau).unwrap_or(Q::ZERO)), BITS));
    let a = sigma_total(&r)
        .mul(&Encl::exact(lam.clone()))
        .mul(&norm.recip().unwrap_or_else(|| Encl::exact(Q::ONE)));
    // Star formation at efficiency x Omega / (2 pi) of the gas, Omega = v / r
    // converted from km/s per kpc to 1/Gyr, with r at least 0.25 kpc.
    let omega = (&Rulers::v_flat() * &km_s_per_kpc_to_per_gyr())
        .checked_div(&r.clone().max(decimal("0.25").unwrap_or(Q::ZERO)))
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
    // The giants' slow return, driven by one iterate of the star formation
    // computed without the return: half over 0.3 Gyr and half over 3 Gyr.
    let first = infall.relaxed(&deplete).scaled(&Encl::exact(form.clone()));
    let slow = [("0.3", "0.5"), ("3", "0.5")]
        .map(|(tau, share)| {
            (
                decimal(tau).unwrap_or(Q::ZERO),
                decimal(share).unwrap_or(Q::ZERO),
            )
        })
        .into_iter()
        .fold(ExpSum::default(), |acc, (tau, share)| {
            let rate = rate_of(&tau);
            acc.plus(
                &first
                    .relaxed(&rate)
                    .scaled(&Encl::exact(&(&Rulers::slow() * &share) * &rate)),
            )
        });
    let feed = infall.plus(&slow);
    let gas = feed.relaxed(&deplete);
    let sfr = gas.scaled(&Encl::exact(form.clone()));
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
}

/// Ring `j`'s mass weight: its area, pi (2j + 1) / 4 kpc^2, in units of pi / 4.
fn ring_area(j: usize) -> Q {
    Q::int(i64::try_from(2 * j + 1).unwrap_or(1))
}

/// The flows between rings: the fountain's landing maps, what each ring
/// throws (gas and per-channel metals), and what the core holds and returns
/// as jets from the ring at its reach.
struct Flows {
    maps: Vec<([Q; RINGS], Q)>,
    thrown_gas: Vec<ExpSum>,
    thrown_metal: Vec<Vec<ExpSum>>,
    core_gas: ExpSum,
    core_metal: Vec<ExpSum>,
    /// The ring at the core's reach, where the jets are launched from.
    edge: usize,
    /// The jets' rate, one over the galaxy's echo (1/Gyr).
    jets: Q,
}

impl Flows {
    /// The flows of the rings `alone`.
    fn of(alone: &[Alone]) -> Flows {
        // The fountain's reach relative to air as a share, 2x / (1 + x) with
        // x = 28 / A, in place of a square root.
        let reach = {
            let x = Q::int(28).checked_div(&Rulers::gas_mu()).unwrap_or(Q::ONE);
            (&Q::int(2) * &x)
                .checked_div(&(&Q::ONE + &x))
                .unwrap_or(Q::ONE)
        };
        let maps: Vec<([Q; RINGS], Q)> = (0..RINGS).map(|j| landing(&reach, j)).collect();
        let thrown_gas: Vec<ExpSum> = alone
            .iter()
            .map(|x| x.gas.scaled(&Encl::exact(&Rulers::loading() * &x.form)))
            .collect();
        let thrown_metal: Vec<Vec<ExpSum>> = alone
            .iter()
            .map(|x| {
                x.metal
                    .iter()
                    .map(|m| m.scaled(&Encl::exact(&Rulers::loading() * &x.form)))
                    .collect()
            })
            .collect();
        // Einstein-Cartan recycling: what the fountain drops inside the
        // centre falls to the core, which returns it as jets launched from
        // the ring at the core's reach with e-fold time the galaxy's echo
        // (`centre::galaxy_episode_myr`); the share of the jets that falls
        // back inside stays in the core.
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
                acc.plus(&flows(j).scaled(&Encl::exact(inner * &ring_area(j))))
            })
        };
        let core_gas = into_core(&|j| thrown_gas[j].clone()).relaxed(&release);
        let core_metal: Vec<ExpSum> = par_map(CHANNELS.len(), |ci| {
            into_core(&|j| thrown_metal[j][ci].clone()).relaxed(&release)
        });
        Flows {
            maps,
            thrown_gas,
            thrown_metal,
            core_gas,
            core_metal,
            edge,
            jets: rate_of(&echo_gyr),
        }
    }

    /// What lands on ring `k` per unit area: each ring's `thrown` times its
    /// landing share on `k` and its area, plus the core's `core` times the
    /// jet rate and the launching ring's landing share on `k`, all over the
    /// area of `k`.
    fn lands(&self, k: usize, thrown: &dyn Fn(usize) -> ExpSum, core: &ExpSum) -> ExpSum {
        let ak = ring_area(k);
        let from_rings = (0..RINGS).fold(ExpSum::default(), |acc, j| {
            let share = &self.maps[j].0[k];
            if share.is_zero() {
                return acc;
            }
            let weight = (share * &ring_area(j)).checked_div(&ak).unwrap_or(Q::ZERO);
            acc.plus(&thrown(j).scaled(&Encl::exact(weight)))
        });
        let jet_share = &self.maps[self.edge].0[k];
        if jet_share.is_zero() {
            return from_rings;
        }
        let weight = (&self.jets * jet_share).checked_div(&ak).unwrap_or(Q::ZERO);
        from_rings.plus(&core.scaled(&Encl::exact(weight)))
    }

    /// Ring `k` with the flows: the ring's feed plus the fountain and the
    /// jets landing on it (fed once), its star formation and its metals.
    fn ring(&self, x: &Alone, k: usize) -> Ring {
        let gas = x
            .feed
            .plus(&self.lands(k, &|j| self.thrown_gas[j].clone(), &self.core_gas))
            .relaxed(&x.deplete);
        let sfr = gas.scaled(&Encl::exact(x.form.clone()));
        let made: Vec<ExpSum> = CHANNELS.iter().map(|&c| delayed(c, &sfr)).collect();
        let metal = (0..CHANNELS.len())
            .map(|ci| {
                let landed = self.lands(
                    k,
                    &|j| self.thrown_metal[j][ci].clone(),
                    &self.core_metal[ci],
                );
                made[ci].plus(&landed).relaxed(&x.deplete)
            })
            .collect();
        Ring {
            gas,
            sfr,
            landed: made,
            metal,
        }
    }
}

impl Model {
    /// Builds every ring's closed forms for the history `p`: infall, star
    /// formation, the giants' slow return, each channel's making, the
    /// fountain's landing map and the core's jets (each fed once).
    pub fn of(p: &Params) -> Model {
        let span = &Rulers::today_gyr() - &p.first_stars;
        let alone: Vec<Alone> = par_map(RINGS, |j| ring_alone(p, &span, j));
        let flows = Flows::of(&alone);
        // The rings are independent once the flows are known.
        let rings = par_map(RINGS, |k| flows.ring(&alone[k], k));
        Model {
            params: p.clone(),
            rings,
        }
    }

    /// Every channel's metals per gas (unit yield) on ring `k` at `t` Gyr,
    /// each read once; `None` before the first stars, for a ring index out of
    /// range or where there is no gas.
    pub(super) fn concs(&self, k: usize, t: &Q) -> Option<Vec<Q>> {
        let dt = t - &self.params.first_stars;
        if dt.signum() <= 0 {
            return None;
        }
        let ring = self.rings.get(k)?;
        let gas = ring.gas.at(&dt).mid();
        if gas.signum() <= 0 {
            return None;
        }
        // Channels with the same first delay share the elapsed time, so each
        // rate's exponential is evaluated once per distinct time.
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
