//! A minimal `run::Link`: the seam an uplink uses to drive live play.
//!
//! After every ring it prints the time on the T lane (rings and the ring's
//! phase in degrees) and the numbers of peoples and bells; after every
//! checkpoint it prints the home and the time. An uplink to the Mycelium
//! network implements the same methods and also `origin`.
//!
//! ```text
//! cargo run --release -p xylarium --example minimal_link -- --home var/example-7 --for 5
//! ```
//!
//! The home must already hold a genesis (see the `found_and_observe` example).

use std::path::Path;
use xyl_kernel::universe::Universe;
use xylarium::{options, run};

struct Printer;

impl run::Link for Printer {
    fn ring(
        &mut self,
        u: &mut Universe<'_>,
        bells: &[xyl_cosmos::pulse::Node],
        _relays: &[xyl_cosmos::pulse::Relay],
    ) {
        println!(
            "T {}: {} peoples, {} bells",
            xyl_core::coordinate::beat(&u.t_coordinate().value, 5).unwrap_or_default(),
            u.cosmic.peoples.len(),
            bells.len()
        );
    }

    fn saved(&mut self, home: &Path, u: &Universe<'_>) -> Result<(), String> {
        println!(
            "checkpoint written to {} at T {}",
            home.display(),
            xyl_core::coordinate::beat(&u.t_coordinate().value, 5).unwrap_or_default()
        );
        Ok(())
    }

    // A local link has heard no TUniversal point.
    fn point(&self) -> Option<xyl_kernel::chronometry::Point> {
        None
    }
}

fn main() -> Result<(), String> {
    let words: Vec<String> = std::env::args().skip(1).collect();
    let (o, _) = options(&words)?;
    run::run_with(o, |_, _stop| {
        Ok(Some(Box::new(Printer) as Box<dyn run::Link>))
    })?;
    Ok(())
}
