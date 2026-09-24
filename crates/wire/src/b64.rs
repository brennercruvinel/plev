//! bytes in a JSON line: base64 (RFC 4648, the standard alphabet, with
//! padding) for the scene's binary parts, and a `u64` as 16 hex digits
//! (a JSON number loses one past 2^53), the form a `Revision` has.
//! decoding never panics: anything else is a serde error, which the codec
//! turns into the line's `bad_message`.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serializer};

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// `bytes` as base64.
pub fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn value(c: u8) -> Option<u32> {
    Some(u32::from(match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => return None,
    }))
}

/// base64 back to bytes; `None` for anything `encode` would not write.
pub fn decode(text: &str) -> Option<Vec<u8>> {
    let text = text.as_bytes();
    if text.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let quads = text.chunks(4);
    let last = text.len() / 4;
    for (n, quad) in quads.enumerate() {
        let pad = quad.iter().rev().take_while(|c| **c == b'=').count();
        if pad > 2 || (pad > 0 && n + 1 != last) {
            return None;
        }
        let mut v = 0u32;
        for &c in &quad[..4 - pad] {
            v = (v << 6) | value(c)?;
        }
        v <<= 6 * pad as u32;
        let bytes = [(v >> 16) as u8, (v >> 8) as u8, v as u8];
        // the bits a padded quad leaves unused must be zero, as `encode`
        // writes them: one text per content
        if pad > 0 && bytes[3 - pad..].iter().any(|b| *b != 0) {
            return None;
        }
        out.extend_from_slice(&bytes[..3 - pad]);
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
