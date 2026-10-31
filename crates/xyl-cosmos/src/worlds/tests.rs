//! Tests of the world state and media.

use super::*;

fn dy(s: &str) -> Dy {
    Dy::from_q(&crate::elements::decimal(s).unwrap())
}

#[test]
fn phases_follow_their_diagrams() {
    let ctx = Ctx::new(Scales::default());
    for s in [Species::H2O, Species::CH4] {
        let d = *ctx.diagram(s);
        // At the reference pressure the species is solid below its triple
        // point, liquid between the triple point and the boiling point, gas
        // above the boiling point, and supercritical past the critical point.
        let p = d.pref;
        assert_eq!(ctx.phase(s, d.tt / Dy::int(2), p), Phase::Solid);
        assert_eq!(ctx.phase(s, (d.tt + d.tb) / Dy::int(2), p), Phase::Liquid);
        assert_eq!(ctx.phase(s, (d.tb + d.tc) / Dy::int(2), p), Phase::Gas);
        assert_eq!(
            ctx.phase(s, d.tc * Dy::int(2), d.pc * Dy::int(2)),
            Phase::Supercritical
        );
        // The boiling point at the reference pressure is the diagram's, and
        // it falls under lower pressure.
        let b = ctx.boiling(s, p);
        assert!(
            (b - d.tb).abs() < Dy::sci(1, -9),
            "boils at {b} vs {}",
            d.tb
        );
        assert!(ctx.boiling(s, p / Dy::int(10)) < b);
        // The saturation pressure at the triple point is the triple pressure.
        assert!((ctx.saturation(s, d.tt) - d.pt).abs() < Dy::sci(1, -12));
    }
}

#[test]
fn light_share_above_threshold_falls_with_it() {
    // The share (1 + x) e^(-x) is 1 at x = 0, decreasing in x, and 0 above
    // the cutoff.
    assert_eq!(planck_above(Dy::ZERO), Dy::ONE);
    assert!(planck_above(Dy::ONE) > planck_above(Dy::int(10)));
    assert_eq!(planck_above(Dy::int(61)), Dy::ZERO);
    let sc = Scales::default();
    let (cool, hot) = (band(Dy::int(3000), &sc), band(Dy::int(6000), &sc));
    assert!(
        hot.brk > Dy::ZERO && hot.brk < Dy::ONE,
        "breaks {}",
        hot.brk
    );
    assert!(
        hot.drive > hot.brk,
        "more light drives chemistry than breaks it"
    );
    // A cooler star has a smaller bond-breaking share.
    assert!(cool.brk < hot.brk);
}

#[test]
fn unit_world_state_is_physical() {
    let u = unit();
    let ctx = Ctx::new(Scales::default());
    // Bounds only, not target values: core radius inside the world, field
    // non-negative, positive surface temperature, flux L / a = 1 at the unit
    // distance, and every medium at a positive temperature.
    assert!(
        u.r_core > Dy::ZERO
            && u.r_core < u.radius * Dy::from_q(&crate::package::Si::world_radius_m())
    );
    assert!(u.field >= Dy::ZERO, "field {}", u.field);
    assert!(u.t_surf > Dy::ZERO, "t_surf {}", u.t_surf);
    assert_eq!(u.flux, Dy::ONE, "one sun's light at one unit of reach");
    let ms = media(u, Dy::ONE, &ctx);
    assert!(ms.iter().all(|m| m.t > Dy::ZERO));
}

#[test]
fn surface_temperature_falls_with_distance() {
    let ctx = Ctx::new(Scales::default());
    let u = unit();
    let world = World {
        a: Q::ONE,
        mass: Q::ONE,
        radius: Q::ONE,
        giant: false,
        host: None,
        a_host_radii: None,
        layers: [
            Q::new(2, 5).unwrap(),
            Q::new(3, 5).unwrap(),
            Q::ZERO,
            Q::ZERO,
        ],
        elements: Vec::new(),
        mantle: Vec::new(),
        ices: vec![
            ("H2O", dy("0.002").to_q()),
            ("CO2", dy("0.0002").to_q()),
            ("N2", dy("0.00002").to_q()),
        ],
        e_forced: None,
        swing: Q::ZERO,
        phase: Q::ZERO,
        merged_at: None,
    };
    let sources = [
        dy("1e17").to_q(),
        dy("1e15").to_q(),
        dy("3e17").to_q(),
        dy("5e16").to_q(),
    ];
    let at = |a: Dy| {
        let set = Setting {
            world: &world,
            star_mass: Dy::ONE,
            lum: Dy::ONE,
            a_au: a,
            z: Dy::ONE,
            r_kpc: Dy::int(8),
            compactness: Dy::sci(1, -6),
            sources: sources.clone(),
            path: None,
        };
        build(&set, Dy::int(3), &ctx)
    };
    let (close, far) = (at(Dy::ratio(2, 5)), at(Dy::int(5)));
    assert!(
        close.t_surf > far.t_surf,
        "{} vs {}",
        close.t_surf,
        far.t_surf
    );
    assert!(close.t_surf > u.t_surf && u.t_surf > far.t_surf);
    // A giant's state has no surface air or seas; its media include a deep
    // plasma.
    let giant = World {
        giant: true,
        mass: Q::int(318),
        radius: Q::int(11),
        layers: [Q::ZERO, Q::ZERO, Q::ZERO, Q::ONE],
        ..world.clone()
    };
    let set = Setting {
        world: &giant,
        star_mass: Dy::ONE,
        lum: Dy::ONE,
        a_au: Dy::int(5),
        z: Dy::ONE,
        r_kpc: Dy::int(8),
        compactness: Dy::sci(1, -6),
        sources,
        path: None,
    };
    let g = build(&set, Dy::int(3), &ctx);
    let ms = media(&g, Dy::ONE, &ctx);
    assert!(ms.iter().any(|m| m.region == Region::Plasmic));
}
