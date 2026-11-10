//! The watch: a process monitor for the running program on a loaded host.
//!
//! Inputs: notes, stage names and work labels from callers; the process's CPU
//! time and resident memory (`/proc/self/stat`, `/proc/self/status`) and the
//! host load (`/proc/loadavg`).
//!
//! Outputs: `note` (a line stamped with seconds since start), `stage` (sets
//! the current stage and notes it), `busy` (registers work in flight; the
//! returned [`Busy`] guard removes it on drop), `heartbeat(every, stop)` (a
//! thread that notes, every `every`, the stage, CPU share of one core since the
//! last beat, resident memory, host load, the count of work in flight and any
//! piece running 30 seconds or more) and `into(home)`, after which notes are
//! also appended to `<home>/observer.log`. Notes always go to stderr.
//!
//! Invariants: it observes the program, not the physics: nothing it notes
//! enters a law or a reading of the universe. It is the one place in
//! `xyl-core` that reads wall time (MU-STANDARDS law 9 binds the laws, and
//! the watch is an instrument; each use carries its `#[expect]`). A value it
//! cannot read (`/proc`, a poisoned lock) is noted as `-`, never as zero;
//! failing to write the log is ignored.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
#[expect(
    clippy::disallowed_types,
    reason = "law 9: the watch is an instrument of the process and reads wall time to report it"
)]
use std::time::{Duration, Instant};

#[expect(
    clippy::disallowed_types,
    reason = "law 9: the watch is an instrument of the process and reads wall time to report it"
)]
struct Watch {
    began: Instant,
    stage: Mutex<String>,
    log: Mutex<Option<std::fs::File>>,
    /// The work in flight, by label, and when each began.
    busy: Mutex<Vec<(u64, String, Instant)>>,
    next: std::sync::atomic::AtomicU64,
}

#[expect(
    clippy::disallowed_types,
    reason = "law 9: the watch is an instrument of the process and reads wall time to report it"
)]
fn watch() -> &'static Watch {
    static W: OnceLock<Watch> = OnceLock::new();
    W.get_or_init(|| Watch {
        began: Instant::now(),
        stage: Mutex::new("starting".into()),
        log: Mutex::new(None),
        busy: Mutex::new(Vec::new()),
        next: std::sync::atomic::AtomicU64::new(0),
    })
}

/// Appends every later note to `<home>/observer.log`, creating `home` if
/// needed.
pub fn into(home: &std::path::Path) {
    let _ = std::fs::create_dir_all(home);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(home.join("observer.log"))
        .ok();
    if let Ok(mut log) = watch().log.lock() {
        *log = file;
    }
}

/// Writes a note to stderr (and to the log, if set), stamped with the seconds
/// since the watch started.
pub fn note(text: &str) {
    let w = watch();
    let line = format!("[{:>8.1}s] {text}", w.began.elapsed().as_secs_f64());
    eprintln!("{line}");
    if let Ok(mut log) = w.log.lock() {
        if let Some(f) = log.as_mut() {
            let _ = writeln!(f, "{line}");
        }
    }
}

/// Enters a stage: the stage is noted and named in every heartbeat until the
/// next stage is entered.
pub fn stage(name: &str) {
    if let Ok(mut s) = watch().stage.lock() {
        *s = name.to_string();
    }
    note(&format!("STAGE {name}"));
}

/// A piece of work in flight, named in every heartbeat once it has run 30
/// seconds; it leaves the list when the guard drops.
pub struct Busy(u64);

/// Registers a piece of work named `label` with the watch and returns the
/// guard that removes it when dropped.
#[expect(
    clippy::disallowed_types,
    reason = "law 9: the watch is an instrument of the process and reads wall time to report it"
)]
pub fn busy(label: String) -> Busy {
    let w = watch();
    let id = w.next.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut b) = w.busy.lock() {
        b.push((id, label, Instant::now()));
    }
    Busy(id)
}

impl Drop for Busy {
    fn drop(&mut self) {
        if let Ok(mut b) = watch().busy.lock() {
            b.retain(|(id, _, _)| *id != self.0);
        }
    }
}

/// The process's CPU time so far (user and system, in clock ticks of
/// 1/100 s).
fn cpu_ticks() -> Option<u64> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    // Fields are counted after the command's closing parenthesis; utime and
    // stime are the 12th and 13th of them (fields 14 and 15 of the line).
    let rest = stat.rsplit_once(')')?.1;
    let f: Vec<&str> = rest.split_whitespace().collect();
    Some(f.get(11)?.parse::<u64>().ok()? + f.get(12)?.parse::<u64>().ok()?)
}

fn rss_mb() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    // The line reads `VmRSS:  <kB> kB` (proc(5)).
    let kb: u64 = status
        .lines()
        .find_map(|l| l.strip_prefix("VmRSS:"))?
        .trim()
        .split_once(' ')?
        .0
        .parse()
        .ok()?;
    Some(kb / 1024)
}

fn load() -> Option<String> {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|l| l.split_whitespace().next().map(String::from))
}

/// A reading as the heartbeat shows it: the value, or `-` where it was not
/// read.
fn shown<T: std::fmt::Display>(x: Option<T>) -> String {
    match x {
        Some(x) => x.to_string(),
        None => "-".to_string(),
    }
}

/// Starts the heartbeat thread: every `every` it notes the stage, the CPU
/// share used since the last beat (100% is one core), resident memory, the
/// host load, the count of work in flight and each piece running 30 seconds
/// or more, until `stop` is set. A value it cannot read is shown as `-`.
#[expect(
    clippy::disallowed_types,
    reason = "law 9: the watch is an instrument of the process and reads wall time to report it"
)]
pub fn heartbeat(every: Duration, stop: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let mut last = (Instant::now(), cpu_ticks());
        while !stop.load(Ordering::Relaxed) {
            std::thread::sleep(every);
            let (now, ticks) = (Instant::now(), cpu_ticks());
            // CPU ticks are hundredths of a second, so the percentage of one
            // core is `ticks x 1000 / wall_ms`, truncated to an integer.
            let wall_ms = u64::try_from(now.duration_since(last.0).as_millis())
                .ok()
                .and_then(std::num::NonZeroU64::new);
            let cores = match (ticks, last.1, wall_ms) {
                (Some(t), Some(l), Some(ms)) => t
                    .checked_sub(l)
                    .and_then(|d| d.checked_mul(1000))
                    .map(|d| d / ms),
                _ => None,
            };
            let stage = watch().stage.lock().ok().map(|s| s.clone());
            let busy = watch().busy.lock().ok().map(|b| {
                let long: Vec<String> = b
                    .iter()
                    .filter(|(_, _, t)| t.elapsed() >= Duration::from_secs(30))
                    .map(|(_, l, t)| format!("{l} {:.0}s", t.elapsed().as_secs_f64()))
                    .collect();
                (b.len(), long)
            });
            let (in_flight, long) = match busy {
                Some((n, long)) => (Some(n), long),
                None => (None, Vec::new()),
            };
            note(&format!(
                concat!(
                    "HEARTBEAT {}: cpu {}% of one core, rss {} MB, host load {}; ",
                    "{} in flight{}"
                ),
                shown(stage),
                shown(cores),
                shown(rss_mb()),
                shown(load()),
                shown(in_flight),
                if long.is_empty() {
                    String::new()
                } else {
                    format!(", long: {}", long.join(", "))
                }
            ));
            last = (now, ticks);
        }
    });
}
