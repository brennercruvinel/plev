//! the assets a frame references, written and read: an image with its
//! pixels, a path with its geometry, each under its key. a stream image
//! (a camera's frames) travels as its size and place only, never its
//! pixels. the layout is in the module doc of `scene`.

use std::sync::Arc;

use engine::drawing::compositor::QuadVertex;
use engine::drawing::gpu::ImageHandle;
use engine::drawing::path::TessellatedPath;

use super::SceneError;
use super::bytes::{Put, Take, fnv};

/// the key an image travels under: a fixed image's handle, one per
/// content; a stream's place and size, the same for all its frames, so a
/// new frame of a camera moves nothing that travels.
pub(crate) fn image_key(h: &ImageHandle, live: bool) -> u64 {
    let mut b = Put::default();
    if live {
        for v in [h.atlas_x, h.atlas_y, h.width, h.height] {
            b.u32(v);
        }
        return fnv(&[b"live", &b.0]);
    }
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

/// what an image asset carries of the pixels.
pub(crate) enum Pixels<'a> {
    /// a fixed image's, as its slot holds them.
    Here(&'a [u8]),
    /// none: the store had none for the handle.
    Missing,
    /// none, ever: a stream image (a camera), whose frames are never
    /// shown to whoever watches. its id is not written either.
    Live,
}

/// an image asset: where the sender's store holds it and what it carries
/// of its pixels (flag 1, then them; 0 none; 2 a stream's, never).
pub(crate) fn image_asset(h: &ImageHandle, pixels: Pixels<'_>) -> Arc<[u8]> {
    let mut b = Put::default();
    b.u8(IMAGE);
    let id = if matches!(pixels, Pixels::Live) {
        0
    } else {
        h.id
    };
    for v in [id, h.atlas_x, h.atlas_y, h.width, h.height] {
        b.u32(v);
    }
    match pixels {
        Pixels::Here(p) => {
            b.u8(1);
            b.0.extend_from_slice(p);
        }
        Pixels::Missing => b.u8(0),
        Pixels::Live => b.u8(2),
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
        /// a stream's: it has no pixels here, ever.
        live: bool,
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
            let flag = t.u8("an image's pixel flag")?;
            let pixels = match flag {
                0 | 2 => None,
                1 => {
                    let n = (width as usize)
                        .checked_mul(height as usize)
                        .and_then(|n| n.checked_mul(4))
                        .ok_or(SceneError::Invalid("an image's size"))?;
                    Some(t.take(n, "an image's pixels")?.to_vec())
                }
                _ => return Err(SceneError::Invalid("an image's pixel flag")),
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
                live: flag == 2,
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
