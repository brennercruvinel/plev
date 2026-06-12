//! Path asset payload codec: the bytes behind `AssetKind::Path`. The
//! container treats asset payloads as opaque (decision 6); this module
//! defines the v0 layout for tessellated geometry, so any importer can
//! pack a `TessellatedPath` and any player can unpack it without
//! knowing where it came from.
//!
//! payload := color rgba8 (uniform vertex color: fill and stroke
//!   tessellation assign one color to every vertex), vertex_count u16,
//!   index_count u32, (x i32 twips, y i32 twips)*, index u16*
//!
//! positions are quantized to twips (decision 5: integer determinism),
//! which also makes dedup-by-payload collapse sub-twip float jitter:
//! two frames of a static shape pack to the same bytes, the same
//! asset id, and therefore zero delta bytes (the swf display-list
//! lesson at shape granularity). a path too large for the u16 asset
//! limit splits at triangle boundaries into several payloads; the
//! split is deterministic, so dedup still holds across frames.

use crate::quant::{bytes_to_rgba, px_to_twips, rgba_to_bytes, twips_to_px};
use plev::compositor::QuadVertex;
use plev::path::TessellatedPath;

/// Fixed payload head: color rgba8 + vertex_count u16 + index_count u32.
const HEAD: usize = 4 + 2 + 4;
/// Bytes per packed vertex (x i32, y i32 twips).
const VERT: usize = 8;
/// Bytes per packed index (u16).
const IDX: usize = 2;
/// Hard payload cap: the container writes asset data_len as u16.
const MAX_PAYLOAD: usize = u16::MAX as usize;

/// Pack one tessellated path into one or more payloads, splitting at
/// triangle boundaries when the u16 asset limit demands it. An empty
/// path (nothing to draw) packs to no payloads.
pub fn pack_chunks(path: &TessellatedPath) -> Vec<Vec<u8>> {
    let tris = path.indices.chunks_exact(3);
    if path.vertices.is_empty() || tris.len() == 0 {
        return Vec::new();
    }
    let color = rgba_to_bytes(path.vertices[0].color);
    let mut chunks = Vec::new();
    // first-seen vertex remap per chunk; positional, so the split is
    // deterministic and identical geometry packs to identical bytes.
    let mut remap: Vec<Option<u16>> = vec![None; path.vertices.len()];
    let mut verts: Vec<u16> = Vec::new();
    let mut idxs: Vec<u16> = Vec::new();
    for tri in tris {
        let fresh = tri
            .iter()
            .filter(|&&i| remap.get(i as usize).copied().flatten().is_none())
            .count();
        let grown = HEAD + (verts.len() + fresh) * VERT + (idxs.len() + 3) * IDX;
        if grown > MAX_PAYLOAD && !idxs.is_empty() {
            chunks.push(emit(color, &verts, &idxs, &path.vertices));
            for v in &verts {
                remap[usize::from(*v)] = None;
            }
            verts.clear();
            idxs.clear();
        }
        for &i in tri {
            let slot = match remap[i as usize] {
                Some(s) => s,
                None => {
                    let s = verts.len() as u16;
                    remap[i as usize] = Some(s);
                    verts.push(i as u16);
                    s
                }
            };
            idxs.push(slot);
        }
    }
    if !idxs.is_empty() {
        chunks.push(emit(color, &verts, &idxs, &path.vertices));
    }
    chunks
}

fn emit(color: [u8; 4], verts: &[u16], idxs: &[u16], source: &[QuadVertex]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(HEAD + verts.len() * VERT + idxs.len() * IDX);
    buf.extend_from_slice(&color);
    buf.extend_from_slice(&(verts.len() as u16).to_le_bytes());
    buf.extend_from_slice(&(idxs.len() as u32).to_le_bytes());
    for &v in verts {
        let p = source[usize::from(v)].position;
        buf.extend_from_slice(&px_to_twips(p[0]).to_le_bytes());
        buf.extend_from_slice(&px_to_twips(p[1]).to_le_bytes());
    }
    for &i in idxs {
        buf.extend_from_slice(&i.to_le_bytes());
    }
    buf
}

/// Unpack one payload back into a drawable path. `None` on malformed
/// bytes (wrong length, index out of range); playback never panics on
/// a corrupt asset. The hash is derived from the payload bytes, so
/// equal payloads lower to equal compositor hashes.
pub fn unpack(data: &[u8]) -> Option<TessellatedPath> {
    if data.len() < HEAD {
        return None;
    }
    let color = bytes_to_rgba([data[0], data[1], data[2], data[3]]);
    let vcount = usize::from(u16::from_le_bytes([data[4], data[5]]));
    let icount = u32::from_le_bytes([data[6], data[7], data[8], data[9]]) as usize;
    if data.len() != HEAD + vcount * VERT + icount * IDX || !icount.is_multiple_of(3) {
        return None;
    }
    let mut vertices = Vec::with_capacity(vcount);
    let mut at = HEAD;
    for _ in 0..vcount {
        let x = i32::from_le_bytes(data[at..at + 4].try_into().ok()?);
        let y = i32::from_le_bytes(data[at + 4..at + 8].try_into().ok()?);
        at += VERT;
        vertices.push(QuadVertex {
            position: [twips_to_px(x), twips_to_px(y)],
            color,
        });
    }
    let mut indices = Vec::with_capacity(icount);
    for _ in 0..icount {
        let i = u16::from_le_bytes(data[at..at + 2].try_into().ok()?);
        at += IDX;
        if usize::from(i) >= vcount {
            return None;
        }
        indices.push(u32::from(i));
    }
    Some(TessellatedPath {
        vertices,
        indices,
        hash: fnv1a(data),
    })
}

/// FNV-1a over the payload bytes: a stable content hash for the
/// compositor's dirty tracking, independent of any std hasher.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}
