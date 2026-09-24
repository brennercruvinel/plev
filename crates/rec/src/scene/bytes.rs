//! the scene codec's bytes: little-endian integers, an `f32` as its
//! bits, a string as its byte length (`u32`) then its UTF-8, an `Option`
//! as `0` or `1` then the value. reading never panics: a short or
//! malformed input is a `SceneError`.

use super::SceneError;

/// FNV-1a 64: the hash of a layer's body and the key of an asset.
pub(crate) fn fnv(parts: &[&[u8]]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for part in parts {
        for &b in *part {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

/// bytes being written.
#[derive(Default)]
pub(crate) struct Put(pub(crate) Vec<u8>);

impl Put {
    pub(crate) fn u8(&mut self, v: u8) {
        self.0.push(v);
    }

    pub(crate) fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }

    pub(crate) fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }

    pub(crate) fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }

    pub(crate) fn len(&mut self, n: usize) {
        self.u32(u32::try_from(n).unwrap_or(u32::MAX));
    }

    pub(crate) fn f32(&mut self, v: f32) {
        self.u32(v.to_bits());
    }

    pub(crate) fn f32s(&mut self, vs: &[f32]) {
        for v in vs {
            self.f32(*v);
        }
    }

    pub(crate) fn str(&mut self, s: &str) {
        self.len(s.len());
        self.0.extend_from_slice(s.as_bytes());
    }

    pub(crate) fn opt_u32(&mut self, v: Option<u32>) {
        match v {
            Some(v) => {
                self.u8(1);
                self.u32(v);
            }
            None => self.u8(0),
        }
    }

    pub(crate) fn opt_str(&mut self, v: Option<&str>) {
        match v {
            Some(v) => {
                self.u8(1);
                self.str(v);
            }
            None => self.u8(0),
        }
    }
}

/// bytes being read, strictly.
pub(crate) struct Take<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Take<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Take { bytes, at: 0 }
    }

    pub(crate) fn take(&mut self, n: usize, what: &'static str) -> Result<&'a [u8], SceneError> {
        let end = self
            .at
            .checked_add(n)
            .filter(|end| *end <= self.bytes.len())
            .ok_or(SceneError::Truncated(what))?;
        let out = &self.bytes[self.at..end];
        self.at = end;
        Ok(out)
    }

    fn arr<const N: usize>(&mut self, what: &'static str) -> Result<[u8; N], SceneError> {
        let mut out = [0u8; N];
        out.copy_from_slice(self.take(N, what)?);
        Ok(out)
    }

    pub(crate) fn u8(&mut self, what: &'static str) -> Result<u8, SceneError> {
        Ok(self.arr::<1>(what)?[0])
    }

    pub(crate) fn u16(&mut self, what: &'static str) -> Result<u16, SceneError> {
        Ok(u16::from_le_bytes(self.arr(what)?))
    }

    pub(crate) fn u32(&mut self, what: &'static str) -> Result<u32, SceneError> {
        Ok(u32::from_le_bytes(self.arr(what)?))
    }

    pub(crate) fn u64(&mut self, what: &'static str) -> Result<u64, SceneError> {
        Ok(u64::from_le_bytes(self.arr(what)?))
    }

    pub(crate) fn f32(&mut self, what: &'static str) -> Result<f32, SceneError> {
        Ok(f32::from_bits(self.u32(what)?))
    }

    pub(crate) fn f32s<const N: usize>(
        &mut self,
        what: &'static str,
    ) -> Result<[f32; N], SceneError> {
        let mut out = [0f32; N];
        for v in &mut out {
            *v = self.f32(what)?;
        }
        Ok(out)
    }

    pub(crate) fn bool(&mut self, what: &'static str) -> Result<bool, SceneError> {
        match self.u8(what)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(SceneError::Invalid(what)),
        }
    }

    /// a count of items of at least `each` bytes, checked against what
    /// is left, so a count never allocates past the input.
    pub(crate) fn count(&mut self, each: usize, what: &'static str) -> Result<usize, SceneError> {
        let n = self.u32(what)? as usize;
        if n.saturating_mul(each.max(1)) > self.bytes.len() - self.at {
            return Err(SceneError::Truncated(what));
        }
        Ok(n)
    }

    pub(crate) fn str(&mut self, what: &'static str) -> Result<String, SceneError> {
        let n = self.count(1, what)?;
        let bytes = self.take(n, what)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| SceneError::Invalid(what))
    }

    pub(crate) fn opt_u32(&mut self, what: &'static str) -> Result<Option<u32>, SceneError> {
        Ok(if self.bool(what)? {
            Some(self.u32(what)?)
        } else {
            None
        })
    }

    pub(crate) fn opt_str(&mut self, what: &'static str) -> Result<Option<String>, SceneError> {
        Ok(if self.bool(what)? {
            Some(self.str(what)?)
        } else {
            None
        })
    }

    pub(crate) fn done(&self, what: &'static str) -> Result<(), SceneError> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err(SceneError::Trailing(what))
        }
    }
}
