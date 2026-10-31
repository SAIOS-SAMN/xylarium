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
//! enters a law or a reading of the universe. Failures to read `/proc` or to
//! write the log are ignored.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

struct Watch {
    began: Instant,
    stage: Mutex<String>,
    log: Mutex<Option<std::fs::File>>,
    /// The work in flight, by label, and when each began.
    busy: Mutex<Vec<(u64, String, Instant)>>,
    next: std::sync::atomic::AtomicU64,
}

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
    let kb: u64 = status
        .lines()
        .find(|l| l.starts_with("VmRSS:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()?;
    Some(kb / 1024)
}

fn load() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|l| l.split_whitespace().next().map(String::from))
        .unwrap_or_else(|| "-".into())
}

/// Starts the heartbeat thread: every `every` it notes the stage, the CPU
/// share used since the last beat (100% is one core), resident memory, the
/// host load, the count of work in flight and each piece running 30 seconds
/// or more, until `stop` is set.
pub fn heartbeat(every: Duration, stop: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let mut last = (Instant::now(), cpu_ticks().unwrap_or(0));
        while !stop.load(Ordering::Relaxed) {
            std::thread::sleep(every);
            let (now, ticks) = (Instant::now(), cpu_ticks().unwrap_or(0));
            // CPU ticks are hundredths of a second, so the percentage of one
            // core is `ticks x 1000 / wall_ms`, truncated to an integer.
            let wall_ms = u64::try_from(now.duration_since(last.0).as_millis())
                .unwrap_or(u64::MAX)
                .max(1);
            let cores = ticks.saturating_sub(last.1).saturating_mul(1000) / wall_ms;
            let stage = watch().stage.lock().map(|s| s.clone()).unwrap_or_default();
            let long: Vec<String> = watch()
                .busy
                .lock()
                .map(|b| {
                    b.iter()
                        .filter(|(_, _, t)| t.elapsed() >= Duration::from_secs(30))
                        .map(|(_, l, t)| format!("{l} {:.0}s", t.elapsed().as_secs_f64()))
                        .collect()
                })
                .unwrap_or_default();
            let in_flight = watch().busy.lock().map(|b| b.len()).unwrap_or(0);
            note(&format!(
                concat!(
                    "HEARTBEAT {}: cpu {}% of one core, rss {} MB, host load {}; ",
                    "{} in flight{}"
                ),
                stage,
                cores,
                rss_mb().unwrap_or(0),
                load(),
                in_flight,
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
