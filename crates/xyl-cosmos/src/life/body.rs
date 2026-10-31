//! The world's body as life reads it: growing seasons, surface area and the
//! number of foragers the world feeds.

use super::*;

/// The share of a world's year and surface warm enough to grow, by the axial
/// tilt (whole degrees, read as its absolute value capped at 90). Daily-mean
/// sunlight at each latitude through the year is counted where it reaches
/// half the latitude's yearly mean, weighted by each band's area and rounded
/// to a thousandth. The 91 values are computed once.
pub fn growing_share(tilt_deg: i64) -> Dy {
    static T: OnceLock<Vec<Dy>> = OnceLock::new();
    let table = T.get_or_init(|| (0..=90).map(growing).collect());
    table[usize::try_from(tilt_deg.unsigned_abs().min(90)).unwrap_or(0)]
}

/// The growing share at tilt `deg` (see [`growing_share`]), over 9 latitude
/// bands of 10 degrees weighted by `cos(phi)` and 48 days a year. The daily
/// insolation is `h0 sin(phi) sin(dec) + cos(phi) cos(dec) sin(h0)`, with
/// `h0` the sunset hour angle and `dec = tilt x sin(2 pi (d + 1/2) / 48)`.
pub(super) fn growing(deg: i64) -> Dy {
    let eps = Dy::int(deg) * Dy::PI / Dy::int(180);
    let days = 48i64;
    let (mut total, mut weight) = (Dy::ZERO, Dy::ZERO);
    for i in 0..9i64 {
        let phi = Dy::int(5 + 10 * i) * Dy::PI / Dy::int(180);
        let (sp, cp) = phi.sin_cos();
        let mut qs = Vec::with_capacity(48);
        for d in 0..days {
            let dec =
                eps * (Dy::int(2) * Dy::PI * (Dy::int(d) + Dy::ratio(1, 2)) / Dy::int(days)).sin();
            let (sd, cd) = dec.sin_cos();
            // The sunset hour angle obeys cos(h0) = -tan(phi) tan(dec); polar
            // day and polar night clamp h0 to pi and 0.
            let x = -(sp / cp) * (sd / cd);
            let h0 = if x >= Dy::ONE {
                Dy::ZERO
            } else if x <= -Dy::ONE {
                Dy::PI
            } else {
                x.acos()
            };
            qs.push((h0 * sp * sd + cp * cd * h0.sin()).max(Dy::ZERO));
        }
        let mean = qs.iter().copied().sum::<Dy>() / Dy::int(days);
        let grow = if mean.signum() > 0 {
            Dy::int(
                i64::try_from(qs.iter().filter(|&&q| q >= mean * Dy::ratio(1, 2)).count())
                    .unwrap_or(0),
            ) / Dy::int(days)
        } else {
            Dy::ZERO
        };
        total += grow * cp;
        weight += cp;
    }
    // The share is rounded to a thousandth.
    Dy::int((total / weight * Dy::int(1000) + Dy::ratio(1, 2)).trunc_i64()) / Dy::int(1000)
}

/// The properties of a world's body that its life and foragers depend on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Body {
    /// Radius in world radii.
    pub radius: Dy,
    /// Whether the world is a giant (life confined to a habitable band, a
    /// twentieth of the surface).
    pub giant: bool,
    /// Whether the world is tidally locked to its star at the settled age.
    pub locked: bool,
    /// The axis's tilt from upright (whole degrees, 0 to 90).
    pub tilt: i64,
    /// The orbit's reach `a` (AU).
    pub a_au: Dy,
    /// The star's luminosity, as the system's spec holds it (suns).
    pub star_lum: Dy,
    /// The star's mass, as the system's spec holds it (suns).
    pub star_mass: Dy,
    /// The age (Gyr) at which the world merges into its neighbour and ends;
    /// `None` for a world that does not merge.
    pub merged_at: Option<Dy>,
}

impl Body {
    /// A world's body at the settled age: tilt and tidal lock are read from
    /// the spin path at that age (upright and unlocked without a path).
    pub fn of(set: &Setting) -> Body {
        let age = settled_gyr();
        let (locked, cos) = match set.path {
            Some(p) => (age >= p.lock_gyr, p.cos_obliquity_at(age)),
            None => (false, Dy::ONE),
        };
        let obl = cos.acos();
        let tilt = obl.min(Dy::PI - obl) * Dy::int(180) / Dy::PI;
        Body {
            radius: Dy::from_q(&set.world.radius),
            giant: set.world.giant,
            locked,
            tilt: (tilt + Dy::ratio(1, 2)).trunc_i64(),
            a_au: Dy::from_q(&set.world.a),
            star_lum: set.lum,
            star_mass: set.star_mass,
            merged_at: set.world.merged_at.as_ref().map(Dy::from_q),
        }
    }

    /// The world's area (m^2), `4 pi R R_w^2` with `R` in world radii: linear
    /// in the radius ratio to the unit world. A medium occupies the share
    /// `Medium::band` of it.
    pub fn area_m2(&self) -> Dy {
        let world_radius_m = Dy::from_q(&crate::package::Si::world_radius_m());
        Dy::int(4) * Dy::PI * self.radius * world_radius_m * world_radius_m
    }
}

/// The number of foragers a world feeds once a people has evolved there:
/// living area x growing season x energy reaching the best medium's
/// chemistry, through one link of a food chain (Lindeman's tenth), divided
/// among bodies by what each burns at the chemistry's body temperature. A
/// locked world grows for half the year. A variable world (tilt above 23
/// degrees, or a star below half a solar mass) supports small r-strategists,
/// a steady one large K-strategists; outside plasma, body mass scales as
/// 1/g. The count is 50 000 for a unit-world area of ideal land. It is zero
/// where no medium exists in `region`.
pub fn foragers(ms: &[Medium], region: Region, body: &Body, ctx: &Ctx) -> Dy {
    let Some(med) = best(ms, Some(region)) else {
        return Dy::ZERO;
    };
    let area = body.radius * med.band;
    let season = if body.locked {
        Dy::ratio(1, 2)
    } else {
        growing_share(body.tilt)
    };
    let gravity = body.radius * if body.giant { Dy::ratio(1, 4) } else { Dy::ONE };
    let r_strategist = body.tilt > 23 || body.star_mass < Dy::ratio(1, 2);
    // Body mass relative to a K-strategist's on a unit world is the ratio of
    // the fertility ceilings (r 2.5, K 1.2) over the world's gravity (a
    // giant's surface gravity is taken as a quarter of its radius);
    // metabolism scales as 2m / (1 + m).
    let ceiling = if r_strategist {
        Dy::ratio(5, 2)
    } else {
        Dy::ratio(6, 5)
    };
    let mut mass = Dy::ratio(6, 5) / ceiling;
    if region != Region::Plasmic {
        mass = mass / gravity.max(Dy::sci(1, -3));
    }
    let factor = (Dy::int(2) * mass / (Dy::ONE + mass))
        * worlds::body_temperature(med.solvent, ctx)
        / worlds::t_unit();
    Dy::int(50_000) * area * season * med.energy * Dy::ratio(1, 10) / factor.max(Dy::sci(1, -300))
}

/// A star's main-sequence lifetime (Gyr) at `r_kpc` from the core:
/// `stars::life` at the star's reach in rings, times the universe's nuclear
/// scale.
pub fn lifetime(r_kpc: Dy, ctx: &Ctx) -> Dy {
    Dy::from_q(&crate::stars::life(crate::stars::reach_rings(r_kpc))) * ctx.sc.nuclear
}
