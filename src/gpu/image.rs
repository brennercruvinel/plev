//! Image atlas: decoded images are packed into one RGBA8 atlas texture
//! (etagere shelf allocation, same scheme as the glyph atlas) and drawn by
//! the sprite pipeline (`shaders/image.wgsl`).
//!
//! The store is split in two so loading works without a GPU:
//! - [`ImageStore`] (CPU): decode + dedupe + atlas layout + staged pixel
//!   uploads. Lives behind a global lock so immediate-mode builder code can
//!   resolve bytes to handles anywhere (results are memoized by content
//!   hash, so per-frame calls are cheap).
//! - [`ImageAtlasGpu`] (GPU, owned by `GpuContext`): atlas texture + bind
//!   group; `prepare` drains staged uploads each frame and grows the
//!   texture when the layout outgrew it. Grow-only: no eviction, and
//!   allocations never move, so handles stay valid for the app lifetime.

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
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::Decode(e) => write!(f, "image decode failed: {e}"),
            ImageError::TooLarge { width, height, max } => {
                write!(f, "image {width}x{height} exceeds max atlas size {max}")
            }
            ImageError::PreviouslyFailed => write!(f, "image previously failed to load"),
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
}

/// CPU side of the image atlas. See module docs.
pub struct ImageStore {
    allocator: BucketedAtlasAllocator,
    atlas_size: u32,
    by_hash: FxHashMap<u64, ImageHandle>,
    failed: FxHashSet<u64>,
    next_id: u32,
    pending: Vec<PendingUpload>,
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
            pending: Vec::new(),
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

    fn insert_pixels(
        &mut self,
        hash: u64,
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
        self.by_hash.insert(hash, handle);
        self.pending.push(PendingUpload {
            atlas_x,
            atlas_y,
            width,
            height,
            pixels,
        });
        Ok(handle)
    }

    /// Current logical atlas size (the GPU texture is resized to match).
    pub fn atlas_size(&self) -> u32 {
        self.atlas_size
    }

    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn image_count(&self) -> usize {
        self.by_hash.len()
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
        }
    }

    /// `None` until the first image upload; render loops skip the image
    /// pass in that case.
    pub fn bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.bind_group.as_ref()
    }

    /// Sync the GPU texture with the global store: grow the texture if the
    /// layout outgrew it (copying existing contents -- allocations never
    /// move) and write staged pixels. Cheap no-op when nothing changed.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) {
        let mut store = IMAGE_STORE.lock().unwrap();
        if store.by_hash.is_empty() {
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
        for upload in store.pending.drain(..) {
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
        assert!(store.has_pending());
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
}
