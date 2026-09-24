//! the assets a frame references, written and read: an image with its
//! pixels, a path with its geometry, each under its key. the layout is
//! in the module doc of `scene`.

use std::sync::Arc;

use engine::compositor::QuadVertex;
use engine::gpu::ImageHandle;
use engine::path::TessellatedPath;

use super::SceneError;
use super::bytes::{Put, Take, fnv};

/// the key an image travels under: its handle, which the store gives one
/// content (a load) or one frame of a stream (a `replace`).
pub(crate) fn image_key(h: &ImageHandle) -> u64 {
    let mut b = Put::default();
    for v in [h.id, h.atlas_x, h.atlas_y, h.width, h.height] {
        b.u32(v);
    }
    fnv(&[b"image", &b.0])
}

/// the key a path travels under: the engine's content hash (the one its
/// layer hash takes) and its sizes.
pub(crate) fn path_key(p: &TessellatedPath) -> u64 {
    let mut b = Put::default();
    b.u64(p.hash);
    b.len(p.vertices.len());
    b.len(p.indices.len());
    fnv(&[b"path", &b.0])
}

const IMAGE: u8 = 1;
const PATH: u8 = 2;

/// an image asset: where the sender's store holds it and its pixels
/// (none when the store had none for the handle).
pub(crate) fn image_asset(h: &ImageHandle, pixels: Option<&[u8]>) -> Arc<[u8]> {
    let mut b = Put::default();
    b.u8(IMAGE);
    for v in [h.id, h.atlas_x, h.atlas_y, h.width, h.height] {
        b.u32(v);
    }
    match pixels {
        Some(p) => {
            b.u8(1);
            b.0.extend_from_slice(p);
        }
        None => b.u8(0),
    }
    b.0.into()
}

pub(crate) fn path_asset(p: &TessellatedPath) -> Arc<[u8]> {
    let mut b = Put::default();
    b.u8(PATH);
    b.u64(p.hash);
    b.len(p.vertices.len());
    for v in &p.vertices {
        b.f32s(&v.position);
        b.f32s(&v.color);
    }
    b.len(p.indices.len());
    for i in &p.indices {
        b.u32(*i);
    }
    b.0.into()
}

/// an asset read back.
pub(crate) enum Asset {
    Image {
        handle: ImageHandle,
        pixels: Option<Vec<u8>>,
    },
    Path(TessellatedPath),
}

pub(crate) fn read_asset(bytes: &[u8]) -> Result<Asset, SceneError> {
    let mut t = Take::new(bytes);
    let asset = match t.u8("an asset's kind")? {
        IMAGE => {
            let mut v = [0u32; 5];
            for x in &mut v {
                *x = t.u32("an image")?;
            }
            let [id, atlas_x, atlas_y, width, height] = v;
            let pixels = if t.bool("an image's pixel flag")? {
                let n = (width as usize)
                    .checked_mul(height as usize)
                    .and_then(|n| n.checked_mul(4))
                    .ok_or(SceneError::Invalid("an image's size"))?;
                Some(t.take(n, "an image's pixels")?.to_vec())
            } else {
                None
            };
            Asset::Image {
                handle: ImageHandle {
                    id,
                    atlas_x,
                    atlas_y,
                    width,
                    height,
                },
                pixels,
            }
        }
        PATH => {
            let hash = t.u64("a path")?;
            let n = t.count(24, "a path's vertices")?;
            let mut vertices = Vec::with_capacity(n);
            for _ in 0..n {
                vertices.push(QuadVertex {
                    position: t.f32s("a path's vertex")?,
                    color: t.f32s("a path's vertex")?,
                });
            }
            let n = t.count(4, "a path's indices")?;
            let mut indices = Vec::with_capacity(n);
            for _ in 0..n {
                indices.push(t.u32("a path's index")?);
            }
            Asset::Path(TessellatedPath {
                vertices,
                indices,
                hash,
            })
        }
        kind => return Err(SceneError::Tag(kind)),
    };
    t.done("an asset")?;
    Ok(asset)
}
