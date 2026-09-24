//! Frames without a window.
//!
//! [`Renderer`] owns a headless [`GpuContext`] (no surface; target format
//! [`HEADLESS_FORMAT`]) plus what a window owns per frame (text system,
//! effect processor, texture pool) and one offscreen target texture.
//! [`Renderer::draw`] runs [`draw_frame`], the function `present_frame`
//! runs against the surface: same compositor resolve, same passes, same
//! pipelines, same sRGB encode on write. Only the target differs.
//!
//! [`Renderer::read`] copies the target back to RGBA8 ([`read_texture`];
//! rows padded to 256 bytes on the GPU side, stripped here) and
//! [`Frame::write_png`] writes it.
//!
//! Native only: the constructor blocks on the adapter and device
//! requests, which a browser cannot do.

mod readback;
pub mod svg;

pub(crate) use readback::write_png;
pub use readback::{Frame, read_texture};
pub use svg::{Approximation, SvgExport};

use std::fmt;

use crate::compositor::Compositor;
use crate::effects::EffectProcessor;
use crate::gpu::texture_pool::TexturePool;
use crate::gpu::{GpuContext, HEADLESS_FORMAT, LostDevice, RenderConfig};
use crate::text::TextSystem;
use crate::window::draw_frame;

/// What a headless frame covers: a logical size (the coordinates a scene is
/// laid out in, CSS pixels) and the scale from logical to physical pixels.
/// A phone is `390 x 844` at `3.0`; a desktop window `1280 x 800` at `2.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
}

impl Viewport {
    pub fn new(width: f32, height: f32, scale: f32) -> Self {
        Self {
            width,
            height,
            scale,
        }
    }

    /// The target texture size in physical pixels: logical size times
    /// scale, rounded, at least one pixel.
    pub fn physical(&self) -> (u32, u32) {
        let side = |logical: f32| ((logical * self.scale).round() as u32).max(1);
        (side(self.width), side(self.height))
    }

    fn check(&self) -> Result<(), HeadlessError> {
        let positive = |v: f32| v.is_finite() && v > 0.0;
        if positive(self.width) && positive(self.height) && positive(self.scale) {
            Ok(())
        } else {
            Err(HeadlessError::InvalidViewport(*self))
        }
    }
}

#[derive(Debug)]
pub enum HeadlessError {
    /// No GPU adapter or device on this machine. Tests skip on it.
    NoAdapter,
    /// Width, height or scale is not a positive finite number.
    InvalidViewport(Viewport),
    /// The physical size exceeds the device's largest 2D texture.
    TooLarge { width: u32, height: u32, max: u32 },
    /// A texture format the readback does not convert to RGBA8.
    UnsupportedFormat(wgpu::TextureFormat),
    /// Mapping or polling the readback buffer failed.
    Readback(String),
    /// The pixels could not be encoded as PNG.
    Png(image::ImageError),
    /// The PNG could not be written.
    Io(std::io::Error),
    /// The GPU device was lost and no adapter answered the recovery yet
    /// (`gpu::recover`): the last frame was skipped and there is nothing
    /// to read. The next `draw` tries again.
    DeviceLost(LostDevice),
}

impl fmt::Display for HeadlessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAdapter => write!(f, "no GPU adapter available"),
            Self::DeviceLost(lost) => write!(
                f,
                "GPU device lost ({:?}: {:?}) and not recovered yet",
                lost.reason, lost.message
            ),
            Self::InvalidViewport(v) => write!(
                f,
                "invalid viewport {}x{} at scale {} (all must be positive)",
                v.width, v.height, v.scale
            ),
            Self::TooLarge { width, height, max } => write!(
                f,
                "{width}x{height} physical pixels exceed the device limit of {max} per side"
            ),
            Self::UnsupportedFormat(format) => {
                write!(f, "cannot read back a {format:?} texture as RGBA8")
            }
            Self::Readback(msg) => write!(f, "readback failed: {msg}"),
            Self::Png(e) => write!(f, "png encode: {e}"),
            Self::Io(e) => write!(f, "write: {e}"),
        }
    }
}

impl std::error::Error for HeadlessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Png(e) => Some(e),
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for HeadlessError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// A window's frame path with an offscreen texture where the surface would
/// be. Keep one alive across frames to model a running app (retained
/// layers, a grown glyph atlas); build a fresh one for an isolated frame.
pub struct Renderer {
    gpu: GpuContext,
    text_system: TextSystem,
    effects: EffectProcessor,
    texture_pool: TexturePool,
    target: wgpu::Texture,
    viewport: Viewport,
}

impl Renderer {
    /// Create the context and the target for `viewport`. Blocks until the
    /// adapter and device are ready.
    pub fn new(viewport: Viewport, config: RenderConfig) -> Result<Self, HeadlessError> {
        viewport.check()?;
        let (width, height) = viewport.physical();
        let mut gpu = pollster::block_on(GpuContext::new_headless(width, height, config))
            .ok_or(HeadlessError::NoAdapter)?;
        fits(&gpu, width, height)?;
        gpu.set_projection(viewport.width, viewport.height);
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let effects = EffectProcessor::new(&gpu.device, gpu.surface_format());
        let target = create_target(&gpu);
        Ok(Self {
            gpu,
            text_system,
            effects,
            texture_pool: TexturePool::new(),
            target,
            viewport,
        })
    }

    /// Change the size or scale, as a window resize does: the context is
    /// resized, the logical projection reapplied, the target recreated.
    pub fn resize(&mut self, viewport: Viewport) -> Result<(), HeadlessError> {
        viewport.check()?;
        let (width, height) = viewport.physical();
        fits(&self.gpu, width, height)?;
        self.gpu.resize(width, height);
        self.gpu.set_projection(viewport.width, viewport.height);
        self.target = create_target(&self.gpu);
        self.viewport = viewport;
        Ok(())
    }

    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    /// The context the frames are drawn with. A test reaches the device
    /// through it, to force a loss with `Device::destroy`.
    pub fn gpu(&self) -> &GpuContext {
        &self.gpu
    }

    /// The context, mutably: to point a recovery elsewhere
    /// (`GpuContext::set_recovery_backends`).
    pub fn gpu_mut(&mut self) -> &mut GpuContext {
        &mut self.gpu
    }

    /// Draw the scene already pushed into `compositor` (after its
    /// `begin_frame`) into the target. `clear_linear` is the background
    /// in linear space, as for `present_frame`. Returns the draw calls,
    /// or 0 when the frame was skipped because the device was lost and no
    /// adapter answers yet (logged once; `read` says so too).
    ///
    /// A lost device is recovered here, before drawing, as `present_frame`
    /// does for a window; the target is recreated with it.
    pub fn draw(&mut self, compositor: &mut Compositor, clear_linear: [f32; 4]) -> u32 {
        if self.gpu.poll_device_loss() {
            if !crate::gpu::recover_or_skip(
                compositor,
                &mut self.gpu,
                &mut self.text_system,
                &mut self.effects,
                &mut self.texture_pool,
                "headless_frame",
            ) {
                return 0;
            }
            self.target = create_target(&self.gpu);
        }
        let view = self.gpu.texture_render_view(&self.target);
        draw_frame(
            compositor,
            &mut self.gpu,
            &mut self.text_system,
            &mut self.effects,
            &mut self.texture_pool,
            &view,
            clear_linear,
            "headless_frame",
        )
    }

    /// The target's pixels as they are now (the last drawn frame).
    /// `DeviceLost` while the device is gone and not recovered: the
    /// target died with it, and mapping a dead buffer is a fatal wgpu
    /// error, not a readable frame.
    pub fn read(&self) -> Result<Frame, HeadlessError> {
        if self.gpu.poll_device_loss() {
            let lost = self.gpu.lost_device().unwrap_or(LostDevice {
                reason: wgpu::DeviceLostReason::Unknown,
                message: String::new(),
            });
            return Err(HeadlessError::DeviceLost(lost));
        }
        read_texture(&self.gpu, &self.target)
    }

    /// The scene in `compositor` as SVG, every approximation reported
    /// ([`svg::export`]). The glyph outlines come from this renderer's
    /// text system, the shaping the last [`draw`](Self::draw) drew from.
    /// `background` is the page color in sRGB (`Color::to_array`), the
    /// clear the frame started from; `None` for a transparent page.
    pub fn svg(
        &mut self,
        compositor: &Compositor,
        background: Option<[f32; 4]>,
    ) -> Result<SvgExport, HeadlessError> {
        svg::export(compositor, &mut self.text_system, self.viewport, background)
    }

    /// [`draw`](Self::draw) then [`read`](Self::read).
    pub fn render(
        &mut self,
        compositor: &mut Compositor,
        clear_linear: [f32; 4],
    ) -> Result<Frame, HeadlessError> {
        self.draw(compositor, clear_linear);
        self.read()
    }
}

fn fits(gpu: &GpuContext, width: u32, height: u32) -> Result<(), HeadlessError> {
    let max = gpu.device.limits().max_texture_dimension_2d;
    if width > max || height > max {
        return Err(HeadlessError::TooLarge { width, height, max });
    }
    Ok(())
}

/// The offscreen stand-in for the surface texture: the context's size and
/// format, drawn into and copied out of.
fn create_target(gpu: &GpuContext) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("headless_target"),
        size: wgpu::Extent3d {
            width: gpu.surface_config.width,
            height: gpu.surface_config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: HEADLESS_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

#[cfg(test)]
mod tests;
