//! Deterministic random streams.
//!
//! Inputs: the universe's seed (`u64`) and a path of [`Part`] labels
//! (integers, text or bytes: e.g. `"season"`, a subsystem's name and the
//! cycle, or a mind's id and its turn); no other module.
//!
//! Outputs: [`Stream`], a xoshiro256** generator whose 256-bit state is the
//! BLAKE3 derive-key hash of the seed and the path (each part tagged by kind
//! and length-prefixed); `next_u64`, `below(n)` (unbiased, Lemire's method)
//! and `range(lo, hi)` (inclusive).
//!
//! Invariants: the same seed and path give the same draws on every machine.
//! Streams share no state, so two subsystems, two minds or two threads draw
//! independently, and a draw added to one stream does not shift another; this
//! keeps a universe reproducible when work is spread across threads. `below(0)`
//! and `range` with `lo > hi` return `None`.

/// A deterministic xoshiro256** generator whose state is derived from a seed
/// and a path of labels; the same seed and path always give the same draws.
///
/// ```
/// use xyl_core::Stream;
/// use xyl_core::rng::Part;
///
/// let path = [Part::from("season"), Part::from(3u64)];
/// let mut a = Stream::derive(9, &path);
/// let mut b = Stream::derive(9, &path);
/// let draws_a: Vec<u64> = (0..4).map(|_| a.next_u64()).collect();
/// let draws_b: Vec<u64> = (0..4).map(|_| b.next_u64()).collect();
/// assert_eq!(draws_a, draws_b);
///
/// // A different path names an independent stream.
/// let mut c = Stream::derive(9, &[Part::from("season"), Part::from(4u64)]);
/// assert_ne!(c.next_u64(), draws_a[0]);
///
/// assert_eq!(a.below(0), None);
/// assert!(matches!(a.below(6), Some(v) if v < 6));
/// ```
#[derive(Clone, Debug)]
pub struct Stream {
    s: [u64; 4],
}

/// One part of a stream's path.
pub enum Part<'a> {
    /// An integer label (a cycle, an id, an index), hashed as 8 little-endian bytes.
    U(u64),
    /// A text label, hashed as its UTF-8 bytes.
    S(&'a str),
    /// A raw byte label, hashed as given.
    B(&'a [u8]),
}

impl From<u64> for Part<'_> {
    fn from(v: u64) -> Self {
        Part::U(v)
    }
}
impl From<u32> for Part<'_> {
    fn from(v: u32) -> Self {
        Part::U(u64::from(v))
    }
}
impl<'a> From<&'a str> for Part<'a> {
    fn from(v: &'a str) -> Self {
        Part::S(v)
    }
}

impl Stream {
    /// The stream named by `seed` and `path`: the parts are hashed with
    /// BLAKE3 (each tagged by kind and length-prefixed) into the 256-bit
    /// generator state.
    pub fn derive(seed: u64, path: &[Part<'_>]) -> Stream {
        let mut h = blake3::Hasher::new_derive_key("xylarium 2026-10-07 stream v1");
        h.update(&seed.to_le_bytes());
        for p in path {
            let (tag, bytes): (u8, Vec<u8>) = match p {
                Part::U(v) => (0, v.to_le_bytes().to_vec()),
                Part::S(s) => (1, s.as_bytes().to_vec()),
                Part::B(b) => (2, b.to_vec()),
            };
            h.update(&[tag]);
            h.update(&(bytes.len() as u64).to_le_bytes());
            h.update(&bytes);
        }
        let out = h.finalize();
        let b = out.as_bytes();
        let mut s = [0u64; 4];
        for (i, w) in s.iter_mut().enumerate() {
            let mut a = [0u8; 8];
            a.copy_from_slice(&b[i * 8..i * 8 + 8]);
            *w = u64::from_le_bytes(a);
        }
        if s == [0; 4] {
            s[0] = 1; // xoshiro's one forbidden state
        }
        Stream { s }
    }

    /// The next uniform 64-bit draw, advancing the state.
    pub fn next_u64(&mut self) -> u64 {
        let r = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        r
    }

    /// Uniform in `0..n` without bias (Lemire's method); `None` when `n == 0`.
    pub fn below(&mut self, n: u64) -> Option<u64> {
        if n == 0 {
            return None;
        }
        let threshold = n.wrapping_neg() % n;
        loop {
            let m = u128::from(self.next_u64()) * u128::from(n);
            #[expect(
                clippy::cast_possible_truncation,
                reason = "the low 64 bits of a 128-bit product, by construction"
            )]
            let low = m as u64;
            if low >= threshold {
                return Some((m >> 64) as u64);
            }
        }
    }

    /// Uniform in `lo..=hi`; `None` when `lo > hi`.
    pub fn range(&mut self, lo: i64, hi: i64) -> Option<i64> {
        if lo > hi {
            return None;
        }
        let span = (i128::from(hi) - i128::from(lo) + 1) as u128;
        let off = if span > u128::from(u64::MAX) {
            self.next_u64()
        } else {
            self.below(u64::try_from(span).ok()?)?
        };
        i64::try_from(i128::from(lo) + i128::from(off)).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_path_same_stream_and_paths_are_independent() {
        let mut a = Stream::derive(7, &["season".into(), "demography".into(), 25u64.into()]);
        let mut b = Stream::derive(7, &["season".into(), "demography".into(), 25u64.into()]);
        let mut c = Stream::derive(7, &["season".into(), "fertility".into(), 25u64.into()]);
        let xa: Vec<u64> = (0..8).map(|_| a.next_u64()).collect();
        let xb: Vec<u64> = (0..8).map(|_| b.next_u64()).collect();
        let xc: Vec<u64> = (0..8).map(|_| c.next_u64()).collect();
        assert_eq!(xa, xb);
        assert_ne!(xa, xc);
    }

    #[test]
    fn path_parts_do_not_alias() {
        // "ab"+"c" must not equal "a"+"bc": parts are length-framed.
        let mut a = Stream::derive(1, &["ab".into(), "c".into()]);
        let mut b = Stream::derive(1, &["a".into(), "bc".into()]);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn below_and_range_stay_in_bounds() {
        let mut s = Stream::derive(3, &[]);
        for _ in 0..10_000 {
            assert!(s.below(7).is_some_and(|x| x < 7));
            assert!(s.range(-3, 3).is_some_and(|x| (-3..=3).contains(&x)));
        }
        assert_eq!(s.below(0), None);
        assert_eq!(s.range(2, 1), None);
    }
}
