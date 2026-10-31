//! A minimal `run::Link`: the seam an uplink uses to drive live play.
//!
//! It prints the universe's ring, its time in Gyr and the number of peoples
//! after every ring, and the home after every checkpoint. An uplink to the
//! Mycelium network implements the same three methods.
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
            "ring {} at {} Gyr: {} peoples, {} bells",
            u.ring(),
            u.gyr(),
            u.cosmic.peoples.len(),
            bells.len()
        );
    }

    fn saved(&mut self, home: &Path, u: &Universe<'_>) -> Result<(), String> {
        println!(
            "checkpoint written to {} at ring {}",
            home.display(),
            u.ring()
        );
        Ok(())
    }

    // a local link hears no network clock
    fn height(&self) -> Option<u64> {
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
