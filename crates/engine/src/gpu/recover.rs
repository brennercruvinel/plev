//! Recovery from a lost device, in place.
//!
//! Everything a `GpuContext` owns lives on its device and dies with it:
//! device and queue, the surface's configuration, every pipeline and
//! layout, the projection buffer and its bind group, the image atlas
//! texture, the timestamp query sets. [`GpuContext::recover`] asks the
//! same instance for an adapter and a device again and rebuilds the
//! context through the constructor path every context already takes
//! (`from_device`), so nothing is built a second way. A window's surface
//! survives the device (it belongs to the instance) and is configured
//! onto the new one; a headless target is the caller's to recreate.
//!
//! What the frame owns beside the context dies with the device too: the
//! glyph atlas of the `TextSystem`, the `EffectProcessor`, the
//! `TexturePool`, the compositor's layer textures and vertex buffers.
//! [`recover_frame_resources`] rebuilds the context and then those, in
//! that order, and leaves every layer dirty, so the next frame draws the
//! whole scene again on the new device.
//!
//! What survives, being CPU data: the scene, the render config, the
//! logical projection, the shaping cache, the image store (its handles
//! and its log of every upload, pixels included: the fresh atlas writes
//! the whole log on the next `prepare`), the hot reload watchers.

use std::fmt;

use super::context::GpuContext;
use super::loss::LostDevice;
use crate::compositor::Compositor;
use crate::effects::EffectProcessor;
use crate::gpu::texture_pool::TexturePool;
use crate::text::TextSystem;

/// Why a lost device could not be replaced. The context keeps its dead
/// device and stays lost; a later attempt may succeed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoverError {
    /// No adapter answered: the GPU is gone, or
    /// [`GpuContext::set_recovery_backends`] named none.
    NoAdapter(String),
    /// An adapter answered and refused a device.
    NoDevice(String),
}

impl fmt::Display for RecoverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAdapter(e) => write!(f, "no GPU adapter for the recovery: {e}"),
            Self::NoDevice(e) => write!(f, "the adapter refused a device: {e}"),
        }
    }
}

impl std::error::Error for RecoverError {}

impl GpuContext {
    /// Which backends a recovery asks for an adapter on, in place of the
    /// context's own instance. Headless contexts only: a window's surface
    /// belongs to the instance that made it, so a windowed context keeps
    /// asking that one and ignores this. A test models a machine whose
    /// GPU is gone with `wgpu::Backends::empty()`.
    pub fn set_recovery_backends(&mut self, backends: wgpu::Backends) {
        if self.surface.is_some() {
            log::warn!("gpu: recovery backends ignored, the surface belongs to the instance");
            return;
        }
        self.recovery_backends = Some(backends);
    }

    /// Replace the device with a fresh one from a fresh adapter and
    /// rebuild everything this context owns on it (module docs). What the
    /// frame owns beside the context is [`recover_frame_resources`]'s job.
    /// On a live device it replaces it all the same. `Err` changes
    /// nothing: the context keeps its device and the lost flag.
    pub async fn recover_async(&mut self) -> Result<LostDevice, RecoverError> {
        let lost = self.loss.detail().unwrap_or_else(|| LostDevice {
            reason: wgpu::DeviceLostReason::Unknown,
            message: "recovery asked for on a live device".to_string(),
        });
        let instance = match self.recovery_backends {
            Some(backends) if self.surface.is_none() => Self::instance_on(backends),
            _ => self.instance.clone(),
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: self.surface.as_ref(),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| RecoverError::NoAdapter(e.to_string()))?;
        let (device, queue) = adapter
            .request_device(&Self::device_descriptor(&adapter, &self.config))
            .await
            .map_err(|e| RecoverError::NoDevice(e.to_string()))?;

        // the surface outlives the device: the same size, the format and
        // modes the new adapter offers, configured onto the new device
        let surface_config = match &self.surface {
            Some(surface) => {
                let config = Self::surface_configuration(
                    surface,
                    &adapter,
                    &self.config,
                    self.surface_config.width,
                    self.surface_config.height,
                );
                surface.configure(&device, &config);
                config
            }
            None => self.surface_config.clone(),
        };

        let mut fresh = Self::from_device(
            instance,
            device,
            queue,
            self.surface.take(),
            surface_config,
            self.config.clone(),
        );
        fresh.recovery_backends = self.recovery_backends;
        if let Some((width, height)) = self.logical_size {
            fresh.set_projection(width, height);
        }
        #[cfg(hot_reload)]
        {
            fresh.shader_watcher = self.shader_watcher.take();
            fresh._source_watcher = self._source_watcher.take();
        }

        log::info!(
            "gpu: device lost ({:?}: {:?}); recovered on {}",
            lost.reason,
            lost.message,
            adapter.get_info().name
        );
        *self = fresh;
        Ok(lost)
    }

    /// [`Self::recover_async`] driven to completion. Native only: it
    /// blocks on the adapter and device requests.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn recover(&mut self) -> Result<LostDevice, RecoverError> {
        pollster::block_on(self.recover_async())
    }
}

/// Rebuild, on a fresh device, everything a frame draws with, after the
/// device was lost: the context ([`GpuContext::recover_async`]), then the
/// glyph atlas (empty; the glyphs come back on the next frame through the
/// same raster path), the effect pipelines, the texture pool (empty) and
/// the compositor's layer textures and vertex buffers (dropped, every
/// layer dirty). `Err` leaves everything as it was, the context still
/// lost; a later call may succeed.
pub async fn recover_frame_resources_async(
    compositor: &mut Compositor,
    gpu: &mut GpuContext,
    text_system: &mut TextSystem,
    effects: &mut EffectProcessor,
    texture_pool: &mut TexturePool,
) -> Result<LostDevice, RecoverError> {
    let lost = gpu.recover_async().await?;
    text_system.recreate_atlas(&gpu.device, &gpu.text_bind_group_layout);
    *effects = EffectProcessor::new(&gpu.device, gpu.surface_format());
    texture_pool.clear();
    compositor.drop_gpu_resources();
    Ok(lost)
}

/// What a frame does once it sees the device lost: recover everything in
/// place and go on drawing (`true`), or skip this frame (`false`) with
/// the error logged once per loss, not once per frame; the next requested
/// frame tries again. Native only for the recovery itself: on wasm the
/// device request cannot block inside a frame, so the frame is skipped
/// and the loss reported.
pub(crate) fn recover_or_skip(
    compositor: &mut Compositor,
    gpu: &mut GpuContext,
    text_system: &mut TextSystem,
    effects: &mut EffectProcessor,
    texture_pool: &mut TexturePool,
    label: &str,
) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        match recover_frame_resources(compositor, gpu, text_system, effects, texture_pool) {
            Ok(_) => true,
            Err(e) => {
                if gpu.first_loss_report() {
                    log::error!(
                        "{label}: GPU device lost and not recovered ({e}); frames are \
                         skipped until an adapter answers"
                    );
                }
                false
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (compositor, text_system, effects, texture_pool);
        if gpu.first_loss_report() {
            let lost = gpu.lost_device();
            log::error!(
                "{label}: GPU device lost ({lost:?}); recovery on wasm needs the async \
                 device request and is not wired: frames are skipped"
            );
        }
        false
    }
}

/// [`recover_frame_resources_async`] driven to completion. Native only.
#[cfg(not(target_arch = "wasm32"))]
pub fn recover_frame_resources(
    compositor: &mut Compositor,
    gpu: &mut GpuContext,
    text_system: &mut TextSystem,
    effects: &mut EffectProcessor,
    texture_pool: &mut TexturePool,
) -> Result<LostDevice, RecoverError> {
    pollster::block_on(recover_frame_resources_async(
        compositor,
        gpu,
        text_system,
        effects,
        texture_pool,
    ))
}
