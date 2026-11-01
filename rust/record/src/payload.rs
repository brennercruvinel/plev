//! the asset payloads of a session file (section A). the monster
//! container leaves its text-style kind without a layout, so the session
//! defines its own, here, for what a replay needs to lower a node:
//!
//! asset := kind u8, len u32, payload
//! text  (0) := text_len u32, utf8, font_size f32 bits u32, line_height
//!              u32, has_max_width u8 (then u32), font_weight u16,
//!              letter_spacing u32, has_family u8 (then len u16, utf8)
//! image (1) := width u32, height u32, has_pixels u8 (then width * height
//!              * 4 bytes of RGBA8)
//! path  (2) := piece_count u32, (len u32, piece)*; each piece is a
//!              `motion::encoding::asset_path` payload (twips vertices, one color,
//!              u16 indices), so a path of any size fits
//!
//! a text key is kept bit for bit. a path comes back on the twips grid
//! with the color of its first vertex, as monster keeps paths. an image
//! without pixels comes back with its size only and draws nothing: the
//! engine keeps an image's pixels until they reach the GPU, not after, so
//! a recorder cannot read them back from a handle today.

use std::sync::Arc;

use engine::drawing::compositor::TextNodeKey;
use engine::drawing::path::TessellatedPath;
use motion::LoweredAsset;
use motion::encoding::asset_path::{pack_chunks, unpack};

use crate::error::FileError;
use crate::format::{Cur, len32, put_u16, put_u32};
use crate::visuals::reader::Asset;

const TEXT: u8 = 0;
const IMAGE: u8 = 1;
const PATH: u8 = 2;

/// appends one asset: kind, length, payload.
pub(crate) fn put_asset(buf: &mut Vec<u8>, asset: &Asset) -> Result<(), FileError> {
    let mut payload = Vec::new();
    let kind = match asset {
        Asset::Live(LoweredAsset::TextStyle(key)) => {
            put_text(&mut payload, key)?;
            TEXT
        }
        Asset::Live(LoweredAsset::Image(handle)) => {
            put_image(&mut payload, handle.width, handle.height, None)?;
            IMAGE
        }
        Asset::Image {
            width,
            height,
            pixels,
            ..
        } => {
            put_image(&mut payload, *width, *height, pixels.as_deref())?;
            IMAGE
        }
        Asset::Live(LoweredAsset::Path(path)) => {
            let pieces = pack_chunks(path);
            put_u32(&mut payload, len32(pieces.len(), "path pieces")?);
            for piece in pieces {
                put_u32(&mut payload, len32(piece.len(), "path piece")?);
                payload.extend_from_slice(&piece);
            }
            PATH
        }
    };
    buf.push(kind);
    put_u32(buf, len32(payload.len(), "asset payload")?);
    buf.extend_from_slice(&payload);
    Ok(())
}

fn put_text(buf: &mut Vec<u8>, key: &TextNodeKey) -> Result<(), FileError> {
    put_u32(buf, len32(key.text.len(), "text")?);
    buf.extend_from_slice(key.text.as_bytes());
    put_u32(buf, key.font_size_bits);
    put_u32(buf, key.line_height_bits);
    match key.max_width_bits {
        Some(bits) => {
            buf.push(1);
            put_u32(buf, bits);
        }
        None => buf.push(0),
    }
    put_u16(buf, key.font_weight);
    put_u32(buf, key.letter_spacing_bits);
    match &key.font_family {
        Some(family) => {
            let len = u16::try_from(family.len()).map_err(|_| FileError::TooLarge("family"))?;
            buf.push(1);
            put_u16(buf, len);
            buf.extend_from_slice(family.as_bytes());
        }
        None => buf.push(0),
    }
    Ok(())
}

fn put_image(buf: &mut Vec<u8>, w: u32, h: u32, pixels: Option<&[u8]>) -> Result<(), FileError> {
    put_u32(buf, w);
    put_u32(buf, h);
    match pixels {
        Some(p) => {
            buf.push(1);
            buf.extend_from_slice(p);
        }
        None => buf.push(0),
    }
    Ok(())
}

/// what an image payload holds, before it is loaded into a store.
pub(crate) struct ImageRead {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Option<Arc<[u8]>>,
}

/// one asset read back; an image comes out unloaded.
pub(crate) enum Read {
    Ready(LoweredAsset),
    Image(ImageRead),
}

/// reads one asset: kind, length, a payload that must end where its
/// length says.
pub(crate) fn read_asset(c: &mut Cur<'_>, asset: usize) -> Result<Read, FileError> {
    let bad = |reason| FileError::Asset { asset, reason };
    let kind = c.u8("asset kind")?;
    let len = usize::try_from(c.u32("asset length")?).map_err(|_| bad("length"))?;
    let mut p = Cur::new(c.take(len, "asset payload")?);
    let out = match kind {
        TEXT => Read::Ready(LoweredAsset::TextStyle(read_text(&mut p)?)),
        IMAGE => {
            let width = p.u32("image width")?;
            let height = p.u32("image height")?;
            let pixels = match p.u8("image pixel flag")? {
                0 => None,
                1 => {
                    let n = u64::from(width) * u64::from(height) * 4;
                    let n = usize::try_from(n).map_err(|_| bad("image too large"))?;
                    Some(Arc::from(p.take(n, "image pixels")?))
                }
                _ => return Err(bad("unknown image pixel flag")),
            };
            Read::Image(ImageRead {
                width,
                height,
                pixels,
            })
        }
        PATH => Read::Ready(LoweredAsset::Path(read_path(&mut p, asset)?)),
        _ => return Err(bad("unknown asset kind")),
    };
    p.done("asset payload")?;
    Ok(out)
}

fn read_text(p: &mut Cur<'_>) -> Result<TextNodeKey, FileError> {
    let len = usize::try_from(p.u32("text length")?).map_err(|_| FileError::Truncated("text"))?;
    let text = p.text(len, "text")?;
    let font_size_bits = p.u32("font size")?;
    let line_height_bits = p.u32("line height")?;
    let max_width_bits = match p.u8("max width flag")? {
        0 => None,
        _ => Some(p.u32("max width")?),
    };
    let font_weight = p.u16("font weight")?;
    let letter_spacing_bits = p.u32("letter spacing")?;
    let font_family = match p.u8("family flag")? {
        0 => None,
        _ => {
            let len = usize::from(p.u16("family length")?);
            Some(p.text(len, "family")?)
        }
    };
    // a literal, not `TextNodeKey::new`: the key comes back bit for bit,
    // never through the hot reload lookup of the text.
    Ok(TextNodeKey {
        text,
        font_size_bits,
        line_height_bits,
        max_width_bits,
        font_weight,
        letter_spacing_bits,
        font_family,
        // the session format (versions 0 and 1) keeps no OpenType features:
        // a replayed numeric run shapes its figures proportional
        features: engine::drawing::text::TextFeatures::NONE,
    })
}

/// the FNV-1a offset basis, where a path's hash starts.
const FNV_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

fn read_path(p: &mut Cur<'_>, asset: usize) -> Result<TessellatedPath, FileError> {
    let bad = |reason| FileError::Asset { asset, reason };
    let pieces = p.count(4, "path pieces")?;
    let mut path = TessellatedPath {
        vertices: Vec::new(),
        indices: Vec::new(),
        hash: FNV_BASIS,
    };
    let mut hashes = Vec::new();
    for _ in 0..pieces {
        let len = usize::try_from(p.u32("path piece length")?)
            .map_err(|_| FileError::Truncated("path piece"))?;
        let piece =
            unpack(p.take(len, "path piece")?).ok_or(bad("a path piece does not unpack"))?;
        let base = u32::try_from(path.vertices.len()).map_err(|_| bad("too many vertices"))?;
        for i in piece.indices {
            path.indices
                .push(i.checked_add(base).ok_or(bad("too many vertices"))?);
        }
        path.vertices.extend(piece.vertices);
        hashes.push(piece.hash);
    }
    // one piece keeps the hash monster gives its payload; several fold
    // theirs together.
    path.hash = match hashes.as_slice() {
        [one] => *one,
        many => many.iter().fold(FNV_BASIS, |h, piece| mix(h, *piece)),
    };
    Ok(path)
}

/// folds a piece's content hash into the path's (FNV-1a over its bytes).
fn mix(h: u64, piece: u64) -> u64 {
    piece.to_le_bytes().iter().fold(h, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
