// Wall-clock profiling per section.

/// Wall-clock timing per section (enabled by XYL_PROFILE=1); a diagnostic
/// only, never recorded on the ledger.
pub(crate) struct Prof {
    pub(crate) on: bool,
    pub(crate) gi: usize,
    pub(crate) t: std::time::Instant,
}

impl Prof {
    pub(crate) fn new(gi: usize) -> Prof {
        Prof {
            on: std::env::var("XYL_PROFILE").is_ok(),
            gi,
            t: std::time::Instant::now(),
        }
    }
    pub(crate) fn lap(&mut self, name: &str) {
        if self.on {
            eprintln!(
                "PROF g{} {}: {} ms",
                self.gi,
                name,
                self.t.elapsed().as_millis()
            );
            self.t = std::time::Instant::now();
        }
    }
}
