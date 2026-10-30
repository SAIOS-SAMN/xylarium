//! An exact, compact binary codec for snapshots.
//!
//! Fixed-width little-endian integers, length-prefixed strings and
//! rationals in [`Q::put_bytes`]' canonical form. A decoder that runs out
//! of bytes, meets an unknown tag or has bytes left over returns `None` —
//! never a default — so a damaged snapshot is refused, not half-read.

use crate::q::Q;
use num_bigint::BigInt;
use num_rational::BigRational;

/// A snapshot encoder: appends fields to a byte buffer; every writer
/// returns `&mut Self` so writes chain.
#[derive(Default)]
pub struct Enc {
    /// The bytes written so far.
    pub buf: Vec<u8>,
}

impl Enc {
    /// An encoder with an empty buffer.
    pub fn new() -> Self {
        Self::default()
    }
    /// Appends one byte.
    pub fn u8(&mut self, v: u8) -> &mut Self {
        self.buf.push(v);
        self
    }
    /// Appends a flag as one byte, `0` or `1`.
    pub fn bool(&mut self, v: bool) -> &mut Self {
        self.u8(u8::from(v))
    }
    /// Appends 4 little-endian bytes.
    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.buf.extend_from_slice(&v.to_le_bytes());
        self
    }
    /// Appends 8 little-endian bytes.
    pub fn u64(&mut self, v: u64) -> &mut Self {
        self.buf.extend_from_slice(&v.to_le_bytes());
        self
    }
    /// Appends 8 little-endian two's-complement bytes.
    pub fn i64(&mut self, v: i64) -> &mut Self {
        self.buf.extend_from_slice(&v.to_le_bytes());
        self
    }
    /// Appends 16 little-endian two's-complement bytes.
    pub fn i128(&mut self, v: i128) -> &mut Self {
        self.buf.extend_from_slice(&v.to_le_bytes());
        self
    }
    /// Appends a length or count as a `u64`.
    pub fn len(&mut self, n: usize) -> &mut Self {
        self.u64(n as u64)
    }
    /// Appends a string: its byte length as a `u64`, then its UTF-8 bytes.
    pub fn str(&mut self, s: &str) -> &mut Self {
        self.len(s.len());
        self.buf.extend_from_slice(s.as_bytes());
        self
    }
    /// Appends a rational in its canonical form ([`Q::put_bytes`]).
    pub fn q(&mut self, v: &Q) -> &mut Self {
        v.put_bytes(&mut self.buf);
        self
    }
    /// Appends each value with [`Enc::i64`], with no length prefix.
    pub fn i64s(&mut self, v: &[i64]) -> &mut Self {
        for x in v {
            self.i64(*x);
        }
        self
    }
}

/// A snapshot decoder: reads the fields [`Enc`] writes, in order, from a
/// borrowed byte slice. Every reader returns `None` when the bytes run out or
/// do not form a valid value.
pub struct Dec<'a> {
    buf: &'a [u8],
    at: usize,
}

impl<'a> Dec<'a> {
    /// A decoder positioned at the start of `buf`.
    pub fn new(buf: &'a [u8]) -> Self {
        Dec { buf, at: 0 }
    }
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.at.checked_add(n)?;
        let s = self.buf.get(self.at..end)?;
        self.at = end;
        Some(s)
    }
    fn arr<const N: usize>(&mut self) -> Option<[u8; N]> {
        self.take(N)?.try_into().ok()
    }
    /// Reads one byte. Returns `None` at the end of the buffer.
    pub fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|b| b[0])
    }
    /// Reads a flag. Returns `None` at the end of the buffer or for a byte
    /// other than `0` or `1`.
    pub fn bool(&mut self) -> Option<bool> {
        match self.u8()? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
    /// Reads 4 little-endian bytes. Returns `None` when fewer remain.
    pub fn u32(&mut self) -> Option<u32> {
        self.arr().map(u32::from_le_bytes)
    }
    /// Reads 8 little-endian bytes. Returns `None` when fewer remain.
    pub fn u64(&mut self) -> Option<u64> {
        self.arr().map(u64::from_le_bytes)
    }
    /// Reads 8 little-endian two's-complement bytes. Returns `None` when
    /// fewer remain.
    pub fn i64(&mut self) -> Option<i64> {
        self.arr().map(i64::from_le_bytes)
    }
    /// Reads 16 little-endian two's-complement bytes. Returns `None` when
    /// fewer remain.
    pub fn i128(&mut self) -> Option<i128> {
        self.arr().map(i128::from_le_bytes)
    }
    /// A length, bounded by the bytes that remain (a hostile length cannot
    /// make the decoder allocate more than the snapshot holds).
    pub fn count(&mut self) -> Option<usize> {
        let n = usize::try_from(self.u64()?).ok()?;
        (n <= self.buf.len().saturating_sub(self.at)).then_some(n)
    }
    /// Reads a length-prefixed string. Returns `None` when the length
    /// exceeds the remaining bytes or the bytes are not valid UTF-8.
    pub fn str(&mut self) -> Option<String> {
        let n = self.count()?;
        String::from_utf8(self.take(n)?.to_vec()).ok()
    }
    /// Reads a rational written by [`Enc::q`]. Returns `None` when the bytes
    /// run out, the tag is unknown, a small form is not in lowest terms with
    /// a positive denominator, or a big denominator is not positive.
    pub fn q(&mut self) -> Option<Q> {
        match self.u8()? {
            0 => {
                let n = self.i64()?;
                let d = self.i64()?;
                // canonical form is checked, not trusted
                let q = Q::new(n, d)?;
                (q == (Q::Small { n, d })).then_some(q)
            }
            1 => {
                let mut part = || -> Option<BigInt> {
                    let n = self.count()?;
                    Some(BigInt::from_signed_bytes_le(self.take(n)?))
                };
                let (n, d) = (part()?, part()?);
                if d.sign() != num_bigint::Sign::Plus {
                    return None;
                }
                Some(Q::from_big(BigRational::new(n, d)))
            }
            _ => None,
        }
    }
    /// Reads `N` consecutive `i64` values. Returns `None` when fewer than
    /// `8 * N` bytes remain.
    pub fn i64s<const N: usize>(&mut self) -> Option<[i64; N]> {
        let mut out = [0i64; N];
        for x in &mut out {
            *x = self.i64()?;
        }
        Some(out)
    }
    /// The bytes not yet read.
    pub fn rest(&self) -> &'a [u8] {
        self.buf.get(self.at..).unwrap_or(&[])
    }

    /// Whether every byte has been read.
    pub fn done(&self) -> bool {
        self.at == self.buf.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_refuses_damage() {
        let big = &Q::int(i64::MAX) * &Q::int(i64::MAX);
        let mut e = Enc::new();
        e.u8(7)
            .bool(true)
            .u64(1 << 40)
            .i64(-5)
            .i128(-1)
            .str("Terra")
            .q(&Q::new(-3, 7).unwrap())
            .q(&big);
        let mut d = Dec::new(&e.buf);
        assert_eq!(d.u8(), Some(7));
        assert_eq!(d.bool(), Some(true));
        assert_eq!(d.u64(), Some(1 << 40));
        assert_eq!(d.i64(), Some(-5));
        assert_eq!(d.i128(), Some(-1));
        assert_eq!(d.str().as_deref(), Some("Terra"));
        assert_eq!(d.q(), Q::new(-3, 7));
        assert_eq!(d.q(), Some(big));
        assert!(d.done());
        let mut short = Dec::new(&e.buf[..e.buf.len() - 1]);
        let _ = (
            short.u8(),
            short.bool(),
            short.u64(),
            short.i64(),
            short.i128(),
            short.str(),
            short.q(),
        );
        assert_eq!(short.q(), None);
    }
}
