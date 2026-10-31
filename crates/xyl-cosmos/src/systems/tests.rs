//! Tests of systems: the disc's and each body's mass books.

use super::*;
use crate::galaxy::star;

#[test]
fn disc_and_world_element_masses_sum_exactly() {
    let bang = crate::genesis::bang();
    let chem = crate::genesis::chem();
    let elements = crate::elements::table().unwrap();
    let mut framed = 0;
    'outer: for chunk in [0u64, 4000, 9000] {
        for jj in 0..4u64 {
            for k in 0..40u64 {
                let n = chunk + jj * 40 + k;
                let Some(star) = star(7, bang, n) else {
                    continue;
                };
                // a standing star has a disc frame, and the disc law's
                // books close: regions' holdings plus the belt equal the
                // infall less the star's share
                let frame = crate::galaxy::disc_frame(&star.r, &star.mass)
                    .expect("a standing star's frame");
                let (regions, belt) =
                    crate::disc::manifold_worlds(&frame.star_share, &frame.a_disc, &frame.tide);
                let gathered = regions.iter().fold(Q::ZERO, |a, (_, h)| &a + h);
                assert_eq!(
                    &gathered + &belt,
                    &Q::ONE - &frame.star_share,
                    "the disc's books: regions {gathered} + belt {belt}"
                );
                for w in regions.windows(2) {
                    assert!(w[1].0 > w[0].0, "regions chain outward");
                }
                framed += 1;
                let sys = system(7, bang, &[n], &star, chem, &elements).unwrap();
                for w in &sys.worlds {
                    let sum = w.elements.iter().fold(Q::ZERO, |a, x| &a + x);
                    assert_eq!(sum, w.mass, "a world's elements sum to its mass exactly");
                    let layer_sum = w.layers.iter().fold(Q::ZERO, |a, x| &a + x);
                    assert_eq!(layer_sum, Q::ONE);
                    // the mantle's elements sum to its layer, and no
                    // element's mantle mass exceeds the world's
                    let mantle_mass = &w.mass * &w.layers[1];
                    let mantle_sum = w.mantle.iter().fold(Q::ZERO, |a, x| &a + x);
                    assert_eq!(
                        mantle_sum, mantle_mass,
                        "a mantle's elements sum to its layer exactly"
                    );
                    for (m, e) in w.mantle.iter().zip(&w.elements) {
                        assert!(m <= e, "the mantle holds no more than the world");
                    }
                }
                for b in &sys.belts {
                    let sum = b.elements.iter().fold(Q::ZERO, |a, x| &a + x);
                    assert_eq!(sum, b.mass, "a belt too");
                }
                if framed >= 4 {
                    break 'outer;
                }
            }
        }
    }
    assert!(framed > 0, "some standing stars were framed");
}
