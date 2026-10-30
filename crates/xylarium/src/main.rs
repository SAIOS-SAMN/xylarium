//! `xylarium` - found, run and observe a universe:
//!
//!   xylarium genesis --home DIR [--seed 7] [--stars 400]
//!                    [--threads 2] [--sight cores]
//!   xylarium run     [--home DIR] [--for N] [--save-every K] [--velocity V]
//!                    \[same options\]
//!   xylarium observe [--home DIR] \[same options\] WHAT   (read-only)
//!
//! The home defaults to var/universe-7. `--sight` defaults to `census`
//! (every sample star unrolled); `--sight cores` unrolls only what the cores
//! see. The readings `observe` takes are listed in observe.rs.

use std::process::ExitCode;
use xylarium::{observe, run};

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    let cmd = argv.get(1).cloned().unwrap_or_default();
    let result: Result<bool, String> = (|| {
        let (o, what) = xylarium::options(argv.get(2..).unwrap_or(&[]))?;
        match cmd.as_str() {
            "genesis" => run::genesis(o),
            "run" => run::run(o),
            "observe" => observe::observe(o, &what),
            _ => {
                Err("usage: xylarium genesis|run|observe [options] (see the source header)".into())
            }
        }
    })();
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("xylarium: {e}");
            ExitCode::from(2)
        }
    }
}
