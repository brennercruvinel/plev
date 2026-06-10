/// Pool of reusable GPU textures keyed by (width, height, format).
/// Grow-only: never destroys textures. In steady state, zero allocations.
use rustc_hash::FxHashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TextureKey {
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}

struct PoolEntry {
    #[allow(dead_code)] // Kept alive to own the GPU allocation; view references it
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    in_use: bool,
}

/// Handle to a pooled texture. Stores a cloned TextureView to avoid borrow conflicts.
pub struct TextureHandle {
    key: TextureKey,
    index: usize,
    view: wgpu::TextureView,
}

impl TextureHandle {
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

pub struct TexturePool {
    entries: FxHashMap<TextureKey, Vec<PoolEntry>>,
}

impl Default for TexturePool {
    fn default() -> Self {
        Self::new()
    }
}

impl TexturePool {
    pub fn new() -> Self {
        Self {
            entries: FxHashMap::default(),
        }
    }

    pub fn acquire(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> TextureHandle {
        let key = TextureKey {
            width,
            height,
            format,
        };
        let entries = self.entries.entry(key).or_default();

        for (i, entry) in entries.iter_mut().enumerate() {
            if !entry.in_use {
                entry.in_use = true;
                return TextureHandle {
                    key,
                    index: i,
                    view: entry.view.clone(),
                };
            }
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("pool_texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let handle_view = view.clone();
        let index = entries.len();
        entries.push(PoolEntry {
            texture,
            view,
            in_use: true,
        });

        log::debug!(
            "TexturePool: created {}x{} {:?} (total: {})",
            width,
            height,
            format,
            index + 1
        );

        TextureHandle {
            key,
            index,
            view: handle_view,
        }
    }

    pub fn release(&mut self, handle: TextureHandle) {
        if let Some(entries) = self.entries.get_mut(&handle.key)
            && handle.index < entries.len()
        {
            entries[handle.index].in_use = false;
        }
    }

    /// Drop textures that don't match the given surface dimensions.
    pub fn invalidate_size(&mut self, width: u32, height: u32) {
        self.entries.retain(|key, entries| {
            if key.width != width || key.height != height {
                let any_in_use = entries.iter().any(|e| e.in_use);
                if any_in_use {
                    log::warn!(
                        "TexturePool: cannot invalidate {}x{} — texture still in use",
                        key.width,
                        key.height
                    );
                    return true;
                }
                false
            } else {
                true
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_equality() {
        let k1 = TextureKey {
            width: 100,
            height: 200,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
        };
        let k2 = TextureKey {
            width: 100,
            height: 200,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
        };
        let k3 = TextureKey {
            width: 200,
            height: 100,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
        };
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }

    #[test]
    fn pool_starts_empty() {
        let pool = TexturePool::new();
        assert!(pool.entries.is_empty());
    }
}
