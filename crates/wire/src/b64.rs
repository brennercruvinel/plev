//! bytes in a JSON line: base64 (RFC 4648, the standard alphabet, with
//! padding) for the scene's binary parts, and a `u64` as 16 hex digits
//! (a JSON number loses one past 2^53), the form a `Revision` has.
//! decoding never panics: anything else is a serde error, which the codec
//! turns into the line's `bad_message`.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serializer};

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// `bytes` as base64. Written a quad at a time into a buffer of its
/// final size: the port encodes a camera's frame on the app's thread, in
/// a debug build too.
pub fn encode(bytes: &[u8]) -> String {
    let mut out = vec![b'='; bytes.len().div_ceil(3) * 4];
    let whole = bytes.chunks_exact(3);
    let rest = whole.remainder();
    for (src, dst) in whole.zip(out.chunks_exact_mut(4)) {
        let n = (u32::from(src[0]) << 16) | (u32::from(src[1]) << 8) | u32::from(src[2]);
        dst[0] = ALPHABET[(n >> 18) as usize & 63];
        dst[1] = ALPHABET[(n >> 12) as usize & 63];
        dst[2] = ALPHABET[(n >> 6) as usize & 63];
        dst[3] = ALPHABET[n as usize & 63];
    }
    if !rest.is_empty() {
        let b1 = rest.get(1).copied().unwrap_or(0);
        let n = (u32::from(rest[0]) << 16) | (u32::from(b1) << 8);
        let at = out.len() - 4;
        out[at] = ALPHABET[(n >> 18) as usize & 63];
        out[at + 1] = ALPHABET[(n >> 12) as usize & 63];
        if rest.len() == 2 {
            out[at + 2] = ALPHABET[(n >> 6) as usize & 63];
        }
    }
    // only ASCII of the alphabet and '=' were written
    String::from_utf8(out).unwrap_or_default()
}

/// The value of each byte of the alphabet; 64 for any other byte.
const VALUES: [u8; 256] = {
    let mut v = [64u8; 256];
    let mut i = 0;
    while i < 64 {
        v[ALPHABET[i] as usize] = i as u8;
        i += 1;
    }
    v
};

/// base64 back to bytes; `None` for anything `encode` would not write.
pub fn decode(text: &str) -> Option<Vec<u8>> {
    let text = text.as_bytes();
    if !text.len().is_multiple_of(4) {
        return None;
    }
    let pad = text.iter().rev().take_while(|c| **c == b'=').count();
    if pad > 2 {
        return None;
    }
    let mut out = vec![0u8; text.len() / 4 * 3 - pad];
    let last = text.len() / 4;
    for (n, (quad, dst)) in text.chunks_exact(4).zip(out.chunks_mut(3)).enumerate() {
        let tail = if n + 1 == last { pad } else { 0 };
        let mut v = 0u32;
        for &c in &quad[..4 - tail] {
            let x = VALUES[c as usize];
            if x == 64 {
                return None;
            }
            v = (v << 6) | u32::from(x);
        }
        v <<= 6 * tail as u32;
        let three = [(v >> 16) as u8, (v >> 8) as u8, v as u8];
        // the bits a padded quad leaves unused must be zero, as `encode`
        // writes them: one text per content
        if three[3 - tail..].iter().any(|b| *b != 0) {
            return None;
        }
        dst.copy_from_slice(&three[..dst.len()]);
    }
    Some(out)
}

/// serde for `Vec<u8>` as base64.
pub(crate) mod bytes {
    use super::*;

    pub(crate) fn serialize<S: Serializer>(v: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&encode(v))
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let text = String::deserialize(d)?;
        decode(&text).ok_or_else(|| D::Error::custom("bytes are not base64"))
    }
}

/// serde for `Option<Vec<u8>>` as base64 (with `skip_serializing_if`).
pub(crate) mod opt_bytes {
    use super::*;

    pub(crate) fn serialize<S: Serializer>(v: &Option<Vec<u8>>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(v) => s.serialize_str(&encode(v)),
            None => s.serialize_none(),
        }
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<Vec<u8>>, D::Error> {
        match Option::<String>::deserialize(d)? {
            Some(text) => decode(&text)
                .map(Some)
                .ok_or_else(|| D::Error::custom("bytes are not base64")),
            None => Ok(None),
        }
    }
}

/// serde for a `u64` as 16 hex digits.
pub(crate) mod hex {
    use super::*;

    pub(crate) fn serialize<S: Serializer>(v: &u64, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{v:016x}"))
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
        let text = String::deserialize(d)?;
        if text.len() != 16 || !text.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(D::Error::custom("a hash is 16 hex digits"));
        }
        u64::from_str_radix(&text, 16).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rfc_vectors() {
        for (plain, coded) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(encode(plain.as_bytes()), coded);
            assert_eq!(decode(coded).as_deref(), Some(plain.as_bytes()));
        }
    }

    #[test]
    fn every_byte_goes_and_comes_back() {
        let all: Vec<u8> = (0..=255u8).collect();
        for len in 0..all.len() {
            assert_eq!(decode(&encode(&all[..len])).as_deref(), Some(&all[..len]));
        }
    }

    #[test]
    fn what_encode_never_writes_does_not_decode() {
        for bad in [
            "Zg",
            "Zg=",
            "Z===",
            "Zh==",
            "Zm9=",
            "Zg==Zg==",
            "Zm9v!",
            "Zm9v Zm9v",
            "====",
            "é",
        ] {
            assert_eq!(decode(bad), None, "{bad:?}");
        }
    }
}
