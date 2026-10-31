//! The Xylarium command library: [`run::genesis`], [`run::run`] and
//! [`observe::observe`], shared by the `xylarium` binary and by any program
//! that embeds the kernel, plus [`options`], which parses the command-line
//! words into [`run::Options`] and the words left over.
//!
//! An uplink runs live play with [`run::run_with`] and a [`run::Link`]
//! beside it. Every command returns `Ok(true)` on success and `Err` with a
//! message otherwise.

pub mod observe;
pub mod run;

struct Args<'a> {
    rest: &'a [String],
}

impl Args<'_> {
    fn get(&self, name: &str) -> Option<&str> {
        self.rest
            .windows(2)
            .find(|w| w[0] == name)
            .map(|w| w[1].as_str())
    }
    fn u64(&self, name: &str, default: u64) -> Result<u64, String> {
        self.get(name).map_or(Ok(default), |v| {
            v.parse()
                .map_err(|_| format!("{name}: '{v}' is not a whole number"))
        })
    }
}

/// The options from the words after the command (`--flag value` pairs), and
/// the words left over (what to observe). Every word starting `--` takes the
/// next word as its value; an option not given takes its default (see
/// [`run::Options`]).
///
/// Returns `Err` with a message when a numeric option's value is not a whole
/// number.
///
/// ```
/// let words: Vec<String> = ["--seed", "9", "--home", "x", "universe"].iter().map(|w| w.to_string()).collect();
/// let (o, what) = xylarium::options(&words)?;
/// assert_eq!(o.seed, 9);
/// assert_eq!(o.home, std::path::PathBuf::from("x"));
/// assert_eq!(what, vec!["universe".to_string()]);
///
/// let bad: Vec<String> = ["--seed", "nine"].iter().map(|w| w.to_string()).collect();
/// assert!(xylarium::options(&bad).is_err());
/// # Ok::<(), String>(())
/// ```
pub fn options(rest: &[String]) -> Result<(run::Options, Vec<String>), String> {
    let a = Args { rest };
    let mut what: Vec<String> = Vec::new();
    let mut words = rest.iter();
    while let Some(w) = words.next() {
        if w.starts_with("--") {
            words.next();
        } else {
            what.push(w.clone());
        }
    }
    let o = run::Options {
        seed: a.u64("--seed", 7)?,
        stars: a.u64("--stars", 400)?,
        threads: usize::try_from(a.u64("--threads", 2)?).unwrap_or(2),
        home: std::path::PathBuf::from(a.get("--home").unwrap_or("var/universe-7")),
        save_every: a.u64("--save-every", 1)?,
        until: a
            .get("--for")
            .map(|v| {
                v.parse::<u64>()
                    .map_err(|_| format!("--for: '{v}' is not a whole number"))
            })
            .transpose()?,
        velocity: a.u64("--velocity", 0)?,
        census: a.get("--sight") != Some("cores"),
    };
    Ok((o, what))
}
