//! Image atlas: decoded images are packed into one RGBA8 atlas texture
//! (etagere shelf allocation, same scheme as the glyph atlas) and drawn by
//! the sprite pipeline (`src/gpu/shaders/image.wgsl`).
//!
//! The store is split in two so loading works without a GPU:
//! - [`ImageStore`] (CPU): decode + dedupe + atlas layout + staged pixel
//!   uploads. Lives behind a global lock so immediate-mode builder code can
//!   resolve bytes to handles anywhere (results are memoized by content
//!   hash, so per-frame calls are cheap).
//! - [`ImageAtlasGpu`] (GPU, owned by `GpuContext`): atlas texture + bind
//!   group; `prepare` writes, each frame, the uploads of the store's log
//!   it has not written yet, and grows the texture when the layout
//!   outgrew it. Grow-only: no eviction, and allocations never move, so
//!   handles stay valid for the app lifetime.
//!
//! The store keeps the pixels of every upload (the same budget the
//! grow-only atlas holds), so an atlas that starts from nothing, a second
//! context's or a lost device's replacement, fills itself from the log.
//!
//! A stream image (task-76 item 10: a camera's frames) is one slot whose
//! pixels change in place: [`ImageStore::stream`] packs it once, out of
//! the dedupe, transparent; [`ImageStore::replace`] rewrites its entry in
//! the log (no new entry, no new slot) and answers the same placement
//! under a new `id`, so the scene node that draws it differs and its
//! layer draws again. Every atlas over the store (a context per window,
//! the headless one) writes a replaced entry once, by the revision it
//! carries against the one the atlas saw; an atlas that starts from
//! nothing writes the latest pixels. A stream holds one frame for the
//! app's lifetime: its slot is never freed, like every other.

use std::sync::{LazyLock, Mutex};

use etagere::{BucketedAtlasAllocator, size2};
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};
use std::hash::{Hash, Hasher};

pub const INITIAL_IMAGE_ATLAS_SIZE: u32 = 1024;
pub const MAX_IMAGE_ATLAS_SIZE: u32 = 8192;

/// A packed image: atlas placement in pixels plus the natural size.
/// Placement is stable (grow-only atlas), so the handle is plain data.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageHandle {
    pub id: u32,
    pub atlas_x: u32,
    pub atlas_y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageError {
    Decode(String),
    TooLarge {
        width: u32,
        height: u32,
        max: u32,
    },
    /// A previous load of the same bytes already failed (not re-decoded).
    PreviouslyFailed,
    /// `replace` was given a handle that is not a stream image's.
    NotAStream {
        id: u32,
    },
    /// `replace` was given pixels, or a handle, of another size than the
    /// stream's slot.
    WrongSize {
        width: u32,
        height: u32,
        bytes: usize,
    },
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::Decode(e) => write!(f, "image decode failed: {e}"),
            ImageError::TooLarge { width, height, max } => {
                write!(f, "image {width}x{height} exceeds max atlas size {max}")
            }
            ImageError::PreviouslyFailed => write!(f, "image previously failed to load"),
            ImageError::NotAStream { id } => {
                write!(f, "image {id} is not a stream image: its pixels are fixed")
            }
            ImageError::WrongSize {
                width,
                height,
                bytes,
            } => write!(
                f,
                "stream image is {width}x{height} ({} bytes), given {bytes}",
                *width as usize * *height as usize * 4
            ),
        }
    }
}

struct PendingUpload {
    atlas_x: u32,
    atlas_y: u32,
    width: u32,
    height: u32,
    /// Tightly packed RGBA8.
    pixels: Vec<u8>,
    /// The store's revision when a stream's pixels were last replaced;
    /// 0 for pixels as they were packed.
    revision: u64,
}

/// How far one atlas texture is in the store's log: the entries it has
/// written, and the store's revision it has seen (the replacements of
/// stream entries it has written again).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct AtlasCursor {
    uploaded: usize,
    seen: u64,
}

/// CPU side of the image atlas. See module docs.
pub struct ImageStore {
    allocator: BucketedAtlasAllocator,
    atlas_size: u32,
    by_hash: FxHashMap<u64, ImageHandle>,
    failed: FxHashSet<u64>,
    next_id: u32,
    /// Every upload, in order, pixels included; append-only, save that a
    /// stream's entry is rewritten in place. An atlas texture remembers
    /// how far into it it has written and which revision it has seen.
    uploads: Vec<PendingUpload>,
    /// The stream slots, by placement: their entry in `uploads`.
    streams: FxHashMap<(u32, u32), usize>,
    /// Replacements so far; the revision of the latest.
    revision: u64,
}

impl Default for ImageStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageStore {
    pub fn new() -> Self {
        Self {
            allocator: BucketedAtlasAllocator::new(size2(
                INITIAL_IMAGE_ATLAS_SIZE as i32,
                INITIAL_IMAGE_ATLAS_SIZE as i32,
            )),
            atlas_size: INITIAL_IMAGE_ATLAS_SIZE,
            by_hash: FxHashMap::default(),
            failed: FxHashSet::default(),
            next_id: 0,
            uploads: Vec::new(),
            streams: FxHashMap::default(),
            revision: 0,
        }
    }

    /// Decode PNG/JPEG bytes and pack them into the atlas. Memoized by
    /// content hash: loading the same bytes again returns the same handle
    /// without decoding.
    pub fn load_bytes(&mut self, bytes: &[u8]) -> Result<ImageHandle, ImageError> {
        let hash = hash_bytes(&[bytes]);
        if let Some(handle) = self.by_hash.get(&hash) {
            return Ok(*handle);
        }
        if self.failed.contains(&hash) {
            return Err(ImageError::PreviouslyFailed);
        }

        let decoded = match ::image::load_from_memory(bytes) {
            Ok(img) => img.into_rgba8(),
            Err(e) => {
                log::warn!("image decode failed: {e}");
                self.failed.insert(hash);
                return Err(ImageError::Decode(e.to_string()));
            }
        };
        let (width, height) = decoded.dimensions();
        self.insert_pixels(hash, width, height, decoded.into_raw())
            .inspect_err(|e| {
                log::warn!("image atlas insert failed: {e}");
                self.failed.insert(hash);
            })
    }

    /// Pack raw RGBA8 pixels (tightly packed, `width * height * 4` bytes).
    /// Memoized by pixel content like `load_bytes`.
    pub fn load_rgba(
        &mut self,
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    ) -> Result<ImageHandle, ImageError> {
        assert_eq!(
            pixels.len() as u32,
            width * height * 4,
            "load_rgba: pixel buffer does not match dimensions"
        );
        let hash = hash_bytes(&[&width.to_le_bytes(), &height.to_le_bytes(), &pixels]);
        if let Some(handle) = self.by_hash.get(&hash) {
            return Ok(*handle);
        }
        self.insert_pixels(hash, width, height, pixels)
    }

    /// A stream image: a `width x height` slot packed once, out of the
    /// dedupe, transparent until its first [`replace`](Self::replace).
    pub fn stream(&mut self, width: u32, height: u32) -> Result<ImageHandle, ImageError> {
        let pixels = vec![0u8; width as usize * height as usize * 4];
        let handle = self.pack(width, height, pixels)?;
        self.streams
            .insert((handle.atlas_x, handle.atlas_y), self.uploads.len() - 1);
        Ok(handle)
    }

    /// New pixels for the stream image `handle` (tightly packed RGBA8, the
    /// slot's size): its entry in the log is rewritten in place, and every
    /// atlas writes it once on its next `prepare`. Answers the same
    /// placement under a new `id`, so a scene that draws the new handle
    /// differs from the one that drew the old. Refuses a handle that is
    /// not a stream's, and pixels or a handle of another size.
    pub fn replace(
        &mut self,
        handle: &ImageHandle,
        pixels: &[u8],
    ) -> Result<ImageHandle, ImageError> {
        let Some(&at) = self.streams.get(&(handle.atlas_x, handle.atlas_y)) else {
            return Err(ImageError::NotAStream { id: handle.id });
        };
        let revision = self.revision + 1;
        let entry = &mut self.uploads[at];
        if (handle.width, handle.height) != (entry.width, entry.height)
            || pixels.len() != entry.pixels.len()
        {
            return Err(ImageError::WrongSize {
                width: entry.width,
                height: entry.height,
                bytes: pixels.len(),
            });
        }
        entry.pixels.copy_from_slice(pixels);
        entry.revision = revision;
        self.revision = revision;
        let replaced = ImageHandle {
            id: self.next_id,
            ..*handle
        };
        self.next_id += 1;
        Ok(replaced)
    }

    fn insert_pixels(
        &mut self,
        hash: u64,
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    ) -> Result<ImageHandle, ImageError> {
        let handle = self.pack(width, height, pixels)?;
        self.by_hash.insert(hash, handle);
        Ok(handle)
    }

    /// A slot for `width x height` pixels, the atlas grown when the
    /// layout needs it, and its entry in the log.
    fn pack(
        &mut self,
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    ) -> Result<ImageHandle, ImageError> {
        // 1px padding so clamped linear sampling never bleeds a neighbor.
        let padded_w = width as i32 + 2;
        let padded_h = height as i32 + 2;
        if width.max(height) + 2 > MAX_IMAGE_ATLAS_SIZE {
            return Err(ImageError::TooLarge {
                width,
                height,
                max: MAX_IMAGE_ATLAS_SIZE,
            });
        }

        let alloc = loop {
            if let Some(alloc) = self.allocator.allocate(size2(padded_w, padded_h)) {
                break alloc;
            }
            let new_size = (self.atlas_size * 2).min(MAX_IMAGE_ATLAS_SIZE);
            if new_size == self.atlas_size {
                return Err(ImageError::TooLarge {
                    width,
                    height,
                    max: MAX_IMAGE_ATLAS_SIZE,
                });
            }
            self.allocator.grow(size2(new_size as i32, new_size as i32));
            self.atlas_size = new_size;
        };

        let atlas_x = alloc.rectangle.min.x as u32 + 1;
        let atlas_y = alloc.rectangle.min.y as u32 + 1;
        let handle = ImageHandle {
            id: self.next_id,
            atlas_x,
            atlas_y,
            width,
            height,
        };
        self.next_id += 1;
        self.uploads.push(PendingUpload {
            atlas_x,
            atlas_y,
            width,
            height,
            pixels,
            revision: 0,
        });
        Ok(handle)
    }

    /// The entries an atlas at `cursor` has not written as they are now,
    /// each once: the stream entries replaced since it last looked, then
    /// the entries new to it. The cursor moves past them. An atlas that
    /// starts from nothing gets every entry with its latest pixels.
    fn unwritten(&self, cursor: &mut AtlasCursor) -> Vec<usize> {
        let written = cursor.uploaded.min(self.uploads.len());
        let mut out = Vec::new();
        if cursor.seen != self.revision {
            out.extend(
                self.streams
                    .values()
                    .copied()
                    .filter(|&i| i < written && self.uploads[i].revision > cursor.seen),
            );
            out.sort_unstable();
        }
        out.extend(written..self.uploads.len());
        cursor.uploaded = self.uploads.len();
        cursor.seen = self.revision;
        out
    }

    /// Current logical atlas size (the GPU texture is resized to match).
    pub fn atlas_size(&self) -> u32 {
        self.atlas_size
    }

    /// Uploads in the log, every image ever packed (one per slot: a
    /// stream's replacements rewrite its entry).
    pub fn upload_count(&self) -> usize {
        self.uploads.len()
    }

    /// Images in the atlas: the distinct loads and the stream slots.
    pub fn image_count(&self) -> usize {
        self.by_hash.len() + self.streams.len()
    }
}

fn hash_bytes(parts: &[&[u8]]) -> u64 {
    let mut hasher = FxHasher::default();
    for part in parts {
        part.hash(&mut hasher);
    }
    hasher.finish()
}

static IMAGE_STORE: LazyLock<Mutex<ImageStore>> = LazyLock::new(|| Mutex::new(ImageStore::new()));

/// Load PNG/JPEG bytes into the global image store (memoized by content).
pub fn load_image_bytes(bytes: &[u8]) -> Result<ImageHandle, ImageError> {
    IMAGE_STORE.lock().unwrap().load_bytes(bytes)
}

/// Load raw RGBA8 pixels into the global image store (memoized by content).
pub fn load_image_rgba(
    width: u32,
    height: u32,
    pixels: Vec<u8>,
) -> Result<ImageHandle, ImageError> {
    IMAGE_STORE.lock().unwrap().load_rgba(width, height, pixels)
}

/// A stream image in the global image store: one slot whose pixels
/// change in place (`ImageStore::stream`).
pub fn stream_image_rgba(width: u32, height: u32) -> Result<ImageHandle, ImageError> {
    IMAGE_STORE.lock().unwrap().stream(width, height)
}

/// New pixels for a stream image of the global image store
/// (`ImageStore::replace`): the same slot under a new `id`.
pub fn replace_image_rgba(handle: &ImageHandle, pixels: &[u8]) -> Result<ImageHandle, ImageError> {
    IMAGE_STORE.lock().unwrap().replace(handle, pixels)
}

// ---------------------------------------------------------------------------
// GPU side
// ---------------------------------------------------------------------------

/// Atlas texture + bind group, owned by `GpuContext`. `prepare` must run
/// before encoding any pass that samples images.
pub struct ImageAtlasGpu {
    texture: Option<wgpu::Texture>,
    bind_group: Option<wgpu::BindGroup>,
    sampler: Option<wgpu::Sampler>,
    size: u32,
    /// How far into the store's upload log this texture has written, and
    /// the revision of the store it has seen. A new atlas (a new context,
    /// a recovered device) starts at zero and writes the whole log as it
    /// is now; it never drains the store, so two atlases over one store
    /// each get every image and every replacement.
    cursor: AtlasCursor,
}

impl Default for ImageAtlasGpu {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageAtlasGpu {
    pub fn new() -> Self {
        Self {
            texture: None,
            bind_group: None,
            sampler: None,
            size: 0,
            cursor: AtlasCursor::default(),
        }
    }

    /// `None` until the first image upload; render loops skip the image
    /// pass in that case.
    pub fn bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.bind_group.as_ref()
    }

    /// Sync the GPU texture with the global store: grow the texture if the
    /// layout outgrew it (copying existing contents -- allocations never
    /// move) and write the uploads not written yet and the stream entries
    /// replaced since the last sync. Cheap no-op when nothing changed.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) {
        let store = IMAGE_STORE.lock().unwrap();
        if store.uploads.is_empty() {
            return;
        }

        if self.texture.is_none() || self.size != store.atlas_size {
            let new_size = store.atlas_size;
            let new_texture = create_atlas_texture(device, new_size);

            if let Some(ref old) = self.texture {
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("image_atlas_grow_encoder"),
                });
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: old,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &new_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: self.size,
                        height: self.size,
                        depth_or_array_layers: 1,
                    },
                );
                queue.submit(std::iter::once(encoder.finish()));
            }

            let sampler = self.sampler.get_or_insert_with(|| {
                device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("image_atlas_sampler"),
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                })
            });
            let view = new_texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("image_atlas_bg"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            }));
            self.texture = Some(new_texture);
            self.size = new_size;
        }

        let texture = self.texture.as_ref().unwrap();
        for at in store.unwritten(&mut self.cursor) {
            let upload = &store.uploads[at];
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: upload.atlas_x,
                        y: upload.atlas_y,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &upload.pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(upload.width * 4),
                    rows_per_image: Some(upload.height),
                },
                wgpu::Extent3d {
                    width: upload.width,
                    height: upload.height,
                    depth_or_array_layers: 1,
                },
            );
        }
    }
}

fn create_atlas_texture(device: &wgpu::Device, size: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image_atlas"),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_rgba(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
        rgba.repeat((width * height) as usize)
    }

    #[test]
    fn load_rgba_allocates_and_stages_upload() {
        let mut store = ImageStore::new();
        let handle = store
            .load_rgba(4, 2, solid_rgba(4, 2, [255, 0, 0, 255]))
            .unwrap();
        assert_eq!(handle.width, 4);
        assert_eq!(handle.height, 2);
        // 1px padding keeps content off the atlas border
        assert!(handle.atlas_x >= 1 && handle.atlas_y >= 1);
        assert_eq!(store.upload_count(), 1);
        assert_eq!(store.image_count(), 1);
    }

    #[test]
    fn same_pixels_reuse_the_same_handle() {
        let mut store = ImageStore::new();
        let a = store
            .load_rgba(4, 4, solid_rgba(4, 4, [0, 255, 0, 255]))
            .unwrap();
        let b = store
            .load_rgba(4, 4, solid_rgba(4, 4, [0, 255, 0, 255]))
            .unwrap();
        assert_eq!(a, b);
        assert_eq!(store.image_count(), 1);

        let c = store
            .load_rgba(4, 4, solid_rgba(4, 4, [0, 0, 255, 255]))
            .unwrap();
        assert_ne!(a.id, c.id);
        assert_eq!(store.image_count(), 2);
    }

    #[test]
    fn distinct_images_do_not_overlap_in_the_atlas() {
        let mut store = ImageStore::new();
        let a = store
            .load_rgba(64, 64, solid_rgba(64, 64, [1, 2, 3, 255]))
            .unwrap();
        let b = store
            .load_rgba(64, 64, solid_rgba(64, 64, [4, 5, 6, 255]))
            .unwrap();
        let disjoint_x = a.atlas_x + a.width <= b.atlas_x || b.atlas_x + b.width <= a.atlas_x;
        let disjoint_y = a.atlas_y + a.height <= b.atlas_y || b.atlas_y + b.height <= a.atlas_y;
        assert!(
            disjoint_x || disjoint_y,
            "allocations overlap: {a:?} vs {b:?}"
        );
    }

    #[test]
    fn atlas_grows_for_large_images() {
        let mut store = ImageStore::new();
        assert_eq!(store.atlas_size(), INITIAL_IMAGE_ATLAS_SIZE);
        let w = INITIAL_IMAGE_ATLAS_SIZE + 100;
        let handle = store
            .load_rgba(w, 32, solid_rgba(w, 32, [9, 9, 9, 255]))
            .unwrap();
        assert!(store.atlas_size() > INITIAL_IMAGE_ATLAS_SIZE);
        assert_eq!(handle.width, w);
    }

    #[test]
    fn oversized_image_is_rejected() {
        let mut store = ImageStore::new();
        // Don't actually allocate gigabytes: validate the early size check.
        let err = store
            .insert_pixels(42, MAX_IMAGE_ATLAS_SIZE, 8, Vec::new())
            .unwrap_err();
        assert!(matches!(err, ImageError::TooLarge { .. }));
    }

    #[test]
    fn load_bytes_decodes_png_and_dedupes() {
        // Encode a tiny PNG in memory (png feature also enables encoding).
        let img = ::image::RgbaImage::from_pixel(3, 2, ::image::Rgba([10, 20, 30, 255]));
        let mut png = std::io::Cursor::new(Vec::new());
        img.write_to(&mut png, ::image::ImageFormat::Png).unwrap();
        let bytes = png.into_inner();

        let mut store = ImageStore::new();
        let a = store.load_bytes(&bytes).unwrap();
        assert_eq!((a.width, a.height), (3, 2));
        let b = store.load_bytes(&bytes).unwrap();
        assert_eq!(a, b);
        assert_eq!(store.image_count(), 1);
    }

    /// The upload log is append-only and keeps the pixels: an atlas that
    /// starts from nothing (a second context, a lost device's replacement)
    /// finds every image in it, at the place its handle names, and a
    /// repeated load adds nothing.
    #[test]
    fn the_upload_log_keeps_every_image_for_a_new_atlas() {
        let mut store = ImageStore::new();
        let pixels = solid_rgba(2, 2, [1, 2, 3, 255]);
        let a = store.load_rgba(2, 2, pixels.clone()).unwrap();
        let b = store
            .load_rgba(3, 1, solid_rgba(3, 1, [9, 8, 7, 255]))
            .unwrap();
        assert_eq!(store.load_rgba(2, 2, pixels.clone()).unwrap(), a);
        assert_eq!(store.upload_count(), 2);

        // what a first atlas writes, and what a second one writes: all of it
        for (up, handle) in store.uploads.iter().zip([a, b]) {
            assert_eq!(
                (up.atlas_x, up.atlas_y, up.width, up.height),
                (handle.atlas_x, handle.atlas_y, handle.width, handle.height)
            );
        }
        assert_eq!(store.uploads[0].pixels, pixels);
        // a cursor past the end (a store reset under an old atlas) is empty
        assert!(store.uploads[store.upload_count().min(5)..].is_empty());
    }

    #[test]
    fn invalid_bytes_fail_once_and_are_remembered() {
        let mut store = ImageStore::new();
        let garbage = b"definitely not an image";
        assert!(matches!(
            store.load_bytes(garbage),
            Err(ImageError::Decode(_))
        ));
        assert!(matches!(
            store.load_bytes(garbage),
            Err(ImageError::PreviouslyFailed)
        ));
    }

    /// The entry of the stream at `handle`, as an atlas would write it.
    fn entry<'a>(store: &'a ImageStore, handle: &ImageHandle) -> &'a PendingUpload {
        let at = store.streams[&(handle.atlas_x, handle.atlas_y)];
        &store.uploads[at]
    }

    /// A stream is one slot: packed transparent, and every replacement
    /// rewrites its entry in place under a new id at the same placement.
    /// Fifty frames later the log, the images and the atlas are as they
    /// were after the first, and the entry holds one frame, the last.
    #[test]
    fn a_stream_is_one_slot_whose_pixels_change_in_place() {
        let mut store = ImageStore::new();
        let fixed = store
            .load_rgba(2, 2, solid_rgba(2, 2, [9, 9, 9, 255]))
            .unwrap();
        let first = store.stream(4, 2).unwrap();
        assert_eq!((first.width, first.height), (4, 2));
        assert_eq!(entry(&store, &first).pixels, vec![0u8; 32], "transparent");
        let (uploads, images, atlas) = (
            store.upload_count(),
            store.image_count(),
            store.atlas_size(),
        );
        assert_eq!((uploads, images), (2, 2));

        let mut handle = first;
        for frame in 0..50u8 {
            let next = store
                .replace(&handle, &solid_rgba(4, 2, [frame, 0, 0, 255]))
                .unwrap();
            assert_ne!(next.id, handle.id, "a new id each time");
            assert_eq!(
                (next.atlas_x, next.atlas_y, next.width, next.height),
                (first.atlas_x, first.atlas_y, 4, 2),
                "the same place"
            );
            handle = next;
        }
        assert_eq!(store.upload_count(), uploads);
        assert_eq!(store.image_count(), images);
        assert_eq!(store.atlas_size(), atlas);
        assert_eq!(
            entry(&store, &handle).pixels,
            solid_rgba(4, 2, [49, 0, 0, 255])
        );
        assert_eq!(
            entry(&store, &handle).pixels.capacity(),
            32,
            "one frame held"
        );
        // an old handle of the stream still names it
        assert!(
            store
                .replace(&first, &solid_rgba(4, 2, [1, 1, 1, 255]))
                .is_ok()
        );
        assert_eq!(store.uploads[0].pixels, solid_rgba(2, 2, [9, 9, 9, 255]));
        assert_ne!(fixed.id, handle.id);
    }

    #[test]
    fn replace_refuses_a_fixed_image_and_another_size() {
        let mut store = ImageStore::new();
        let fixed = store
            .load_rgba(4, 2, solid_rgba(4, 2, [1, 2, 3, 255]))
            .unwrap();
        let err = store
            .replace(&fixed, &solid_rgba(4, 2, [0, 0, 0, 255]))
            .unwrap_err();
        assert_eq!(err, ImageError::NotAStream { id: fixed.id });
        assert_eq!(
            err.to_string(),
            format!(
                "image {} is not a stream image: its pixels are fixed",
                fixed.id
            )
        );
        assert_eq!(store.uploads[0].pixels, solid_rgba(4, 2, [1, 2, 3, 255]));

        let stream = store.stream(4, 2).unwrap();
        let short = store.replace(&stream, &[0u8; 8]).unwrap_err();
        assert_eq!(
            short,
            ImageError::WrongSize {
                width: 4,
                height: 2,
                bytes: 8
            }
        );
        assert_eq!(short.to_string(), "stream image is 4x2 (32 bytes), given 8");
        let wide = ImageHandle {
            width: 8,
            height: 1,
            ..stream
        };
        assert!(matches!(
            store.replace(&wide, &[0u8; 32]),
            Err(ImageError::WrongSize {
                width: 4,
                height: 2,
                ..
            })
        ));
        assert_eq!(store.revision, 0, "a refusal replaces nothing");
        assert_eq!(entry(&store, &stream).pixels, vec![0u8; 32]);
    }

    /// Two atlases over one store (a context per window, the headless
    /// one): each writes a replaced entry once, whenever it looks, and a
    /// look with nothing new writes nothing.
    #[test]
    fn two_atlases_write_a_replacement_once_each() {
        let mut store = ImageStore::new();
        store
            .load_rgba(2, 2, solid_rgba(2, 2, [5, 5, 5, 255]))
            .unwrap();
        let stream = store.stream(4, 2).unwrap();
        let (mut a, mut b) = (AtlasCursor::default(), AtlasCursor::default());
        assert_eq!(store.unwritten(&mut a), [0, 1]);
        assert_eq!(store.unwritten(&mut a), [] as [usize; 0], "nothing new");

        let stream = store
            .replace(&stream, &solid_rgba(4, 2, [7, 0, 0, 255]))
            .unwrap();
        assert_eq!(store.unwritten(&mut a), [1], "a writes the replacement");
        assert_eq!(store.unwritten(&mut a), [] as [usize; 0], "once");
        assert_eq!(
            store.unwritten(&mut b),
            [0, 1],
            "b, new, writes all as it is"
        );
        assert_eq!(store.unwritten(&mut b), [] as [usize; 0]);

        // two replacements between looks are one write; a new image beside
        // a replacement is written after it
        let stream = store
            .replace(&stream, &solid_rgba(4, 2, [8, 0, 0, 255]))
            .unwrap();
        store
            .replace(&stream, &solid_rgba(4, 2, [9, 0, 0, 255]))
            .unwrap();
        store
            .load_rgba(3, 1, solid_rgba(3, 1, [4, 4, 4, 255]))
            .unwrap();
        assert_eq!(store.unwritten(&mut a), [1, 2]);
        assert_eq!(store.unwritten(&mut b), [1, 2]);
        assert_eq!(store.unwritten(&mut a), [] as [usize; 0]);
        assert_eq!(store.unwritten(&mut b), [] as [usize; 0]);
    }

    /// An atlas that starts from nothing (a new context, a recovered
    /// device) writes the stream's latest pixels, not its first.
    #[test]
    fn a_new_atlas_gets_the_latest_frame() {
        let mut store = ImageStore::new();
        let mut stream = store.stream(2, 1).unwrap();
        let mut old = AtlasCursor::default();
        assert_eq!(store.unwritten(&mut old), [0]);
        for v in [10u8, 20, 30] {
            stream = store
                .replace(&stream, &[v, v, v, 255, v, v, v, 255])
                .unwrap();
        }
        let mut fresh = AtlasCursor::default();
        let written: Vec<&[u8]> = store
            .unwritten(&mut fresh)
            .into_iter()
            .map(|i| store.uploads[i].pixels.as_slice())
            .collect();
        assert_eq!(written, [[30, 30, 30, 255, 30, 30, 30, 255]]);
        assert_eq!(store.unwritten(&mut old), [0], "and the old one catches up");
    }

    /// The dedupe holds for every other image: the same pixels loaded
    /// twice are one handle, and a stream is never the answer to a load,
    /// not even of the pixels it holds.
    #[test]
    fn the_dedupe_holds_beside_streams() {
        let mut store = ImageStore::new();
        let stream = store.stream(2, 2).unwrap();
        let clear = store.load_rgba(2, 2, vec![0u8; 16]).unwrap();
        assert_ne!(
            (clear.atlas_x, clear.atlas_y),
            (stream.atlas_x, stream.atlas_y)
        );
        assert_eq!(store.load_rgba(2, 2, vec![0u8; 16]).unwrap(), clear);
        let stream = store
            .replace(&stream, &solid_rgba(2, 2, [3, 3, 3, 255]))
            .unwrap();
        let same = store
            .load_rgba(2, 2, solid_rgba(2, 2, [3, 3, 3, 255]))
            .unwrap();
        assert_ne!(
            (same.atlas_x, same.atlas_y),
            (stream.atlas_x, stream.atlas_y)
        );
        assert_eq!(store.image_count(), 3);
        assert_eq!(store.upload_count(), 3);
    }
}
