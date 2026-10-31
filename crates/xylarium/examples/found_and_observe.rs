//! Found a universe through the library API and take two readings.
//!
//! ```text
//! cargo run --release -p xylarium --example found_and_observe -- --home var/example-7 --seed 7
//! ```
//!
//! The first run founds the universe into the home (minutes); later runs reuse
//! it and only take the readings.

use xylarium::{observe, options, run};

fn main() -> Result<(), String> {
    let words: Vec<String> = std::env::args().skip(1).collect();
    let (o, _) = options(&words)?;

    // A home is founded once; `genesis` refuses a home that already holds a
    // genesis.
    if !o.home.join("genesis").exists() {
        run::genesis(o.clone())?;
    }

    // The universe summary, then the planets of the first 20 sample stars.
    let readings: [&[&str]; 2] = [&["universe"], &["worlds", "20"]];
    for words in readings {
        let words: Vec<String> = words.iter().map(|w| w.to_string()).collect();
        observe::observe(o.clone(), &words)?;
    }
    Ok(())
}
