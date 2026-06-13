//! monster v0 decoder: parses the container of kdb/adr/monster-format-v0.md back
//! into the timeline IR. strict by design: magic, version, flags,
//! section tiling, per-section sha256 and full payload consumption are
//! all enforced; malformed or truncated input returns a typed
//! [`ReadError`], never a panic. payload parsing lives in
//! `crate::read_sec`.
//!
//! scope notes:
//! - modify ops become tracks; place|replace|remove ops become the
//!   timeline's structural op lists, each in wire order
//! - script (X) sections are checksum-verified, then skipped: playback
//!   never requires scripting (spec decision 8)
//! - values come back on their quantization grid (twips, rgba8, u16
//!   fixed, bezier u8), so decode(encode(t)) == t holds exactly for
//!   grid-aligned timelines; the property tests generate those

use crate::container::{
    Asset, Desc, MAGIC, SEC_DELTA, SEC_DESC, SEC_KEYFRAME, SEC_SCRIPT, VERSION,
};
use crate::ir::{IrError, Timeline};
use crate::read_sec::{Cur, DeltaOut, parse_asset, parse_delta, parse_descs, parse_keyframe};
use sha2::{Digest, Sha256};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ReadError {
    #[error("file truncated reading {what}")]
    Truncated { what: &'static str },
    #[error("bad magic; not an monster file")]
    BadMagic,
    #[error("unsupported version {0}; this decoder reads v0")]
    UnsupportedVersion(u16),
    #[error("unsupported flags {0:#06x}; v0 defines none")]
    UnsupportedFlags(u16),
    #[error("unknown asset kind byte {0}")]
    UnknownAssetKind(u8),
    #[error("unknown section tag {0:#04x}")]
    UnknownSectionTag(u8),
    #[error("section index does not tile the file: {0}")]
    SectionLayout(&'static str),
    #[error("sha256 mismatch on section '{tag}' (index entry {entry})")]
    ChecksumMismatch { tag: char, entry: usize },
    #[error("section '{0}' payload has bytes past its last field")]
    TrailingPayload(char),
    #[error("{what} is not a finite, in-range value")]
    BadValue { what: &'static str },
    #[error("unknown node kind byte {0}")]
    UnknownNodeKind(u8),
    #[error("presence mask {0:#06x} sets bits with no v0 prop")]
    UnknownPresenceBits(u16),
    #[error("unknown delta op code {0}")]
    UnknownOpCode(u8),
    #[error("easing byte {0:#04x} is reserved")]
    UnknownEasing(u8),
    #[error("custom curve index {index} outside the table of {len}")]
    CurveIndexOutOfRange { index: u16, len: usize },
    #[error("delta block before any keyframe")]
    DeltaBeforeKeyframe,
    #[error("description text is not valid utf-8")]
    BadUtf8(#[from] std::str::Utf8Error),
    #[error("descriptions out of order at keyframe {0}; the track is sorted and unique")]
    DescOutOfOrder(u16),
    #[error("description references keyframe {keyframe}, file has {keyframes}")]
    DescOutOfRange { keyframe: u16, keyframes: usize },
    #[error("header desc_offset {header} disagrees with the T section at {actual}")]
    DescOffsetMismatch { header: u32, actual: u32 },
    #[error("decoded timeline is not a valid IR: {0}")]
    Ir(#[from] IrError),
}

/// Everything one monster file carries, mirroring the inputs of
/// [`crate::write::encode`].
#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    pub timeline: Timeline,
    pub assets: Vec<Asset>,
    pub descs: Vec<Desc>,
}

struct SecEntry {
    tag: u8,
    off: u32,
    len: u32,
    sha: [u8; 32],
}

struct Header {
    duration_s: f32,
    fps_hint: u16,
    assets: Vec<Asset>,
    curves: Vec<[u8; 4]>,
    desc_offset: u32,
    index: Vec<SecEntry>,
    head_len: usize,
}

/// Decode a v0 monster file. Inverse of [`crate::write::encode`] up to
/// quantization. The reconstructed timeline passes
/// [`Timeline::validate`] before it is returned.
pub fn decode(bytes: &[u8]) -> Result<Document, ReadError> {
    let hdr = parse_header(bytes)?;
    check_sections(bytes, &hdr)?;
    let mut keyframes = Vec::new();
    let mut deltas = DeltaOut::default();
    let mut descs = Vec::new();
    for entry in &hdr.index {
        // in bounds: check_sections proved the index tiles the file.
        let payload = &bytes[entry.off as usize..entry.off as usize + entry.len as usize];
        match entry.tag {
            SEC_KEYFRAME => keyframes.push(parse_keyframe(payload)?),
            SEC_DELTA => {
                let t = keyframes
                    .last()
                    .map(|kf| kf.t)
                    .ok_or(ReadError::DeltaBeforeKeyframe)?;
                parse_delta(payload, t, &hdr.curves, &mut deltas)?;
            }
            SEC_DESC => descs = parse_descs(payload)?,
            _ => {} // SEC_SCRIPT: sidecar for monster/script players, opaque here
        }
    }
    if let Some(d) = descs
        .iter()
        .find(|d| usize::from(d.keyframe) >= keyframes.len())
    {
        return Err(ReadError::DescOutOfRange {
            keyframe: d.keyframe,
            keyframes: keyframes.len(),
        });
    }
    let timeline = Timeline {
        duration_s: hdr.duration_s,
        fps_hint: hdr.fps_hint,
        keyframes,
        tracks: deltas.tracks,
        places: deltas.places,
        replaces: deltas.replaces,
        removes: deltas.removes,
    };
    timeline.validate()?;
    Ok(Document {
        timeline,
        assets: hdr.assets,
        descs,
    })
}

fn parse_header(bytes: &[u8]) -> Result<Header, ReadError> {
    let mut c = Cur::new(bytes);
    if c.arr::<4>("magic")? != MAGIC {
        return Err(ReadError::BadMagic);
    }
    let version = c.u16("version")?;
    if version != VERSION {
        return Err(ReadError::UnsupportedVersion(version));
    }
    let flags = c.u16("flags")?;
    if flags != 0 {
        return Err(ReadError::UnsupportedFlags(flags));
    }
    let duration_s = c.f32("duration_s")?;
    let fps_hint = c.u16("fps_hint")?;
    let mut assets = Vec::new();
    for _ in 0..c.u16("asset_count")? {
        assets.push(parse_asset(&mut c)?);
    }
    let mut curves = Vec::new();
    for _ in 0..c.u16("easing_count")? {
        curves.push(c.arr("easing curve")?);
    }
    let desc_offset = c.u32("desc_offset")?;
    let mut index = Vec::new();
    for _ in 0..c.u16("sec_count")? {
        let tag = c.u8("section tag")?;
        if ![SEC_KEYFRAME, SEC_DELTA, SEC_SCRIPT, SEC_DESC].contains(&tag) {
            return Err(ReadError::UnknownSectionTag(tag));
        }
        index.push(SecEntry {
            tag,
            off: c.u32("section offset")?,
            len: c.u32("section len")?,
            sha: c.arr("section sha256")?,
        });
    }
    Ok(Header {
        duration_s,
        fps_hint,
        assets,
        curves,
        desc_offset,
        index,
        head_len: c.at(),
    })
}

/// Sections must tile the file contiguously after the header (the
/// encoder's exact layout, so truncated or padded files always fail),
/// every sha256 must hold, at most one T may exist and the header
/// desc_offset must point at it (0 when absent).
fn check_sections(bytes: &[u8], hdr: &Header) -> Result<(), ReadError> {
    let mut expect = hdr.head_len;
    for entry in &hdr.index {
        if entry.off as usize != expect {
            return Err(ReadError::SectionLayout(
                "offsets must follow the header contiguously",
            ));
        }
        expect = expect
            .checked_add(entry.len as usize)
            .ok_or(ReadError::SectionLayout(
                "section length overflows the file",
            ))?;
    }
    if expect != bytes.len() {
        return Err(ReadError::SectionLayout(
            "file length disagrees with the index",
        ));
    }
    let mut t_off = None;
    for (i, entry) in hdr.index.iter().enumerate() {
        let payload = &bytes[entry.off as usize..entry.off as usize + entry.len as usize];
        let fresh: [u8; 32] = Sha256::digest(payload).into();
        if fresh != entry.sha {
            return Err(ReadError::ChecksumMismatch {
                tag: entry.tag as char,
                entry: i,
            });
        }
        if entry.tag == SEC_DESC {
            if t_off.is_some() {
                return Err(ReadError::SectionLayout("more than one description track"));
            }
            t_off = Some(entry.off);
        }
    }
    let actual = t_off.unwrap_or(0);
    if hdr.desc_offset != actual {
        return Err(ReadError::DescOffsetMismatch {
            header: hdr.desc_offset,
            actual,
        });
    }
    Ok(())
}
