//! the session container: a `.monster` file whose magic is `MONS`
//! (an animation keeps `MON0`; tools dispatch on the magic, see `probe`).
//! little-endian, byte-aligned, a sha256 per section, the discipline of
//! the monster container (docs/ADR/session-recording-replay-not-reexecution.md).
//!
//! file := header section_payload*
//! header :=
//!   magic        4   "MONS"
//!   version      u16 0
//!   flags        u16 0
//!   start_ms     u64 the first time the file holds, on the recorder's clock
//!   crate        u8 len, utf8   the rec version that wrote it
//!   git          u8 len, utf8   the commit it was built from, or "unknown"
//!   sec_count    u16
//!   sec_entry*   49  tag u8 ('S'|'I'|'C'|'A'), offset u64, len u64,
//!                    sha256 of the payload (32 bytes)
//!
//! the payloads follow the index in index order and tile the rest of the
//! file; each tag appears exactly once.
//!
//! S := one `rec::Event` per line, the serde json form, in time order
//! I := chunk_count u32, chunk*; chunk := start u64, frame_count u32,
//!      time u32* (ms from start, strictly increasing from 0),
//!      ref_count u32, asset u32* (the chunk's asset id n is asset ref n),
//!      offset u64, len u64 (inside C; the chunks tile C in index order)
//! C := the chunks, each the exact bytes `monster::encode` writes
//! A := asset_count u32, (kind u8, len u32, payload)*; see `payload`

use sha2::{Digest, Sha256};

use crate::error::FileError;

pub const MAGIC: [u8; 4] = *b"MONS";
/// the magic of a monster animation, which a session reader refuses.
pub const ANIMATION_MAGIC: [u8; 4] = *b"MON0";
pub const VERSION: u16 = 0;

pub(crate) const SEC_EVENTS: u8 = b'S';
pub(crate) const SEC_INDEX: u8 = b'I';
pub(crate) const SEC_CHUNKS: u8 = b'C';
pub(crate) const SEC_ASSETS: u8 = b'A';
/// the sections of a session, in the order the writer lays them out.
pub(crate) const SECTIONS: [u8; 4] = [SEC_EVENTS, SEC_INDEX, SEC_CHUNKS, SEC_ASSETS];
const SEC_ENTRY_LEN: usize = 1 + 8 + 8 + 32;

/// what a `.monster` file is, by its magic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    /// `MON0`: an animation, for `monster::decode` and its player.
    Animation,
    /// `MONS`: a recorded session, for `Session::from_bytes`.
    Session,
    Unknown,
}

/// what a file is, from its first four bytes.
pub fn probe(bytes: &[u8]) -> FileKind {
    match bytes.get(..4) {
        Some(m) if m == MAGIC => FileKind::Session,
        Some(m) if m == ANIMATION_MAGIC => FileKind::Animation,
        _ => FileKind::Unknown,
    }
}

/// the build that wrote a session: the crate version and the git commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Build {
    pub version: String,
    pub git: String,
}

impl Build {
    /// this build of the recorder.
    pub fn current() -> Build {
        Build {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            git: env!("REC_GIT_SHA").to_owned(),
        }
    }
}

/// a header read back: when the file starts, who wrote it, and each
/// section's payload.
pub(crate) struct Parsed<'a> {
    pub(crate) start: u64,
    pub(crate) build: Build,
    pub(crate) sections: [&'a [u8]; 4],
}

pub(crate) fn put_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_le_bytes());
}

pub(crate) fn put_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

pub(crate) fn put_u64(buf: &mut Vec<u8>, v: u64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

/// a length that must fit a u32 field.
pub(crate) fn len32(n: usize, what: &'static str) -> Result<u32, FileError> {
    u32::try_from(n).map_err(|_| FileError::TooLarge(what))
}

fn put_short_text(buf: &mut Vec<u8>, text: &str) {
    // a version and a short sha always fit; a longer text is cut at a
    // char boundary rather than failing the write.
    let mut end = text.len().min(usize::from(u8::MAX));
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    buf.push(end as u8);
    buf.extend_from_slice(&text.as_bytes()[..end]);
}

/// the whole file: header, index with sha256 per section, payloads.
/// `sections` are the payloads of `SECTIONS`, in that order.
pub(crate) fn assemble(start: u64, build: &Build, sections: [&[u8]; 4]) -> Vec<u8> {
    let mut head = Vec::new();
    head.extend_from_slice(&MAGIC);
    put_u16(&mut head, VERSION);
    put_u16(&mut head, 0);
    put_u64(&mut head, start);
    put_short_text(&mut head, &build.version);
    put_short_text(&mut head, &build.git);
    put_u16(&mut head, SECTIONS.len() as u16);
    let body: usize = sections.iter().map(|s| s.len()).sum();
    let header_len = head.len() + SEC_ENTRY_LEN * SECTIONS.len();
    let mut out = Vec::with_capacity(header_len + body);
    out.extend_from_slice(&head);
    let mut offset = header_len as u64;
    for (tag, payload) in SECTIONS.iter().zip(sections) {
        out.push(*tag);
        put_u64(&mut out, offset);
        put_u64(&mut out, payload.len() as u64);
        let digest: [u8; 32] = Sha256::digest(payload).into();
        out.extend_from_slice(&digest);
        offset += payload.len() as u64;
    }
    for payload in sections {
        out.extend_from_slice(payload);
    }
    out
}

/// a bounds-checked reader over untrusted bytes: every read names what it
/// reads, and runs out as `Truncated`, never as a panic.
pub(crate) struct Cur<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cur<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Cur { bytes, at: 0 }
    }

    pub(crate) fn at(&self) -> usize {
        self.at
    }

    pub(crate) fn rest(&self) -> usize {
        self.bytes.len() - self.at
    }

    pub(crate) fn take(&mut self, n: usize, what: &'static str) -> Result<&'a [u8], FileError> {
        let end = self
            .at
            .checked_add(n)
            .filter(|&end| end <= self.bytes.len())
            .ok_or(FileError::Truncated(what))?;
        let out = &self.bytes[self.at..end];
        self.at = end;
        Ok(out)
    }

    fn arr<const N: usize>(&mut self, what: &'static str) -> Result<[u8; N], FileError> {
        let mut out = [0u8; N];
        out.copy_from_slice(self.take(N, what)?);
        Ok(out)
    }

    pub(crate) fn u8(&mut self, what: &'static str) -> Result<u8, FileError> {
        Ok(self.arr::<1>(what)?[0])
    }

    pub(crate) fn u16(&mut self, what: &'static str) -> Result<u16, FileError> {
        Ok(u16::from_le_bytes(self.arr(what)?))
    }

    pub(crate) fn u32(&mut self, what: &'static str) -> Result<u32, FileError> {
        Ok(u32::from_le_bytes(self.arr(what)?))
    }

    pub(crate) fn u64(&mut self, what: &'static str) -> Result<u64, FileError> {
        Ok(u64::from_le_bytes(self.arr(what)?))
    }

    /// a count of items of at least `each` bytes, refused when the bytes
    /// left cannot hold them (so no allocation trusts a count).
    pub(crate) fn count(&mut self, each: usize, what: &'static str) -> Result<usize, FileError> {
        let n = usize::try_from(self.u32(what)?).map_err(|_| FileError::Truncated(what))?;
        match n.checked_mul(each) {
            Some(need) if need <= self.rest() => Ok(n),
            _ => Err(FileError::Truncated(what)),
        }
    }

    pub(crate) fn text(&mut self, len: usize, what: &'static str) -> Result<String, FileError> {
        let raw = self.take(len, what)?;
        std::str::from_utf8(raw)
            .map(str::to_owned)
            .map_err(|_| FileError::BadUtf8(what))
    }

    fn short_text(&mut self, what: &'static str) -> Result<String, FileError> {
        let len = usize::from(self.u8(what)?);
        self.text(len, what)
    }

    /// the payload must end here.
    pub(crate) fn done(&self, what: &'static str) -> Result<(), FileError> {
        if self.rest() == 0 {
            Ok(())
        } else {
            Err(FileError::Trailing(what))
        }
    }
}

/// reads and checks the header and the section index: the magic, the
/// version, the flags, each tag once, the payloads tiling the file, every
/// sha256.
pub(crate) fn parse(bytes: &[u8]) -> Result<Parsed<'_>, FileError> {
    let mut c = Cur::new(bytes);
    let magic: [u8; 4] = c.arr("magic")?;
    if magic == ANIMATION_MAGIC {
        return Err(FileError::Animation);
    }
    if magic != MAGIC {
        return Err(FileError::BadMagic);
    }
    let version = c.u16("version")?;
    if version != VERSION {
        return Err(FileError::UnsupportedVersion(version));
    }
    let flags = c.u16("flags")?;
    if flags != 0 {
        return Err(FileError::UnsupportedFlags(flags));
    }
    let start = c.u64("start")?;
    let build = Build {
        version: c.short_text("crate version")?,
        git: c.short_text("git sha")?,
    };
    let count = usize::from(c.u16("section count")?);
    let mut entries = Vec::new();
    for _ in 0..count {
        let tag = c.u8("section tag")?;
        let offset = c.u64("section offset")?;
        let len = c.u64("section length")?;
        let digest: [u8; 32] = c.arr("section sha256")?;
        entries.push((tag, offset, len, digest));
    }
    let mut sections: [Option<&[u8]>; 4] = [None; 4];
    let mut next = c.at() as u64;
    for (tag, offset, len, digest) in entries {
        let Some(slot) = SECTIONS.iter().position(|t| *t == tag) else {
            return Err(FileError::UnknownSection(tag));
        };
        if sections[slot].is_some() {
            return Err(FileError::DuplicateSection(char::from(tag)));
        }
        if offset != next {
            return Err(FileError::SectionLayout(
                "a section does not start where the one before ends",
            ));
        }
        let end = offset
            .checked_add(len)
            .filter(|&end| end <= bytes.len() as u64)
            .ok_or(FileError::Truncated("section payload"))?;
        let payload = &bytes[offset as usize..end as usize];
        let actual: [u8; 32] = Sha256::digest(payload).into();
        if actual != digest {
            return Err(FileError::Checksum(char::from(tag)));
        }
        sections[slot] = Some(payload);
        next = end;
    }
    if next != bytes.len() as u64 {
        return Err(FileError::SectionLayout("bytes past the last section"));
    }
    let mut out: [&[u8]; 4] = [&[]; 4];
    for (slot, tag) in SECTIONS.iter().enumerate() {
        out[slot] = sections[slot].ok_or(FileError::MissingSection(char::from(*tag)))?;
    }
    Ok(Parsed {
        start,
        build,
        sections: out,
    })
}
