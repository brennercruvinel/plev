#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use winit::window::Window;

use super::context::GpuContext;
use super::utils::ortho_projection;

// ---------------------------------------------------------------------------
// Surface management: resize, drop, recreate
// ---------------------------------------------------------------------------

impl GpuContext {
    pub fn resize(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        self.surface_config.width = width;
        self.surface_config.height = height;
        if let Some(ref surface) = self.surface {
            surface.configure(&self.device, &self.surface_config);
        }

        // Back to physical coordinates until the app re-applies its logical
        // projection (apps call `set_projection` right after `resize`).
        self.logical_size = None;
        let projection_data = ortho_projection(width as f32, height as f32);
        self.queue.write_buffer(
            &self.projection_buffer,
            0,
            bytemuck::cast_slice(&projection_data),
        );
    }

    /// Drop the surface (Android suspend).
    pub fn drop_surface(&mut self) {
        self.surface = None;
        log::info!("Surface dropped");
    }

    /// Recreate the surface from a new window (Android resume).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn recreate_surface(&mut self, window: Arc<Window>) {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let width = size.width.max(1);
        let height = size.height.max(1);
        self.surface_config.width = width;
        self.surface_config.height = height;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("Failed to find adapter on surface recreate");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);
        self.surface_config.format = format;
        // Mirror GpuContext::new_with_config: render into an sRGB view even
        // when the surface format itself can't be sRGB.
        let render_format = format.add_srgb_suffix();
        self.surface_config.view_formats = if render_format != format {
            vec![render_format]
        } else {
            vec![]
        };
        self.surface_config.alpha_mode = caps.alpha_modes[0];

        surface.configure(&self.device, &self.surface_config);
        self.surface = Some(surface);

        let projection_data = ortho_projection(width as f32, height as f32);
        self.queue.write_buffer(
            &self.projection_buffer,
            0,
            bytemuck::cast_slice(&projection_data),
        );

        log::info!("Surface recreated: {}x{}", width, height);
    }

    /// Override the projection matrix with custom logical dimensions.
    /// Use when scene coordinates differ from physical surface size (e.g. HiDPI).
    pub fn set_projection(&mut self, logical_width: f32, logical_height: f32) {
        self.logical_size = Some((logical_width.max(1.0), logical_height.max(1.0)));
        let projection_data = ortho_projection(logical_width, logical_height);
        self.queue.write_buffer(
            &self.projection_buffer,
            0,
            bytemuck::cast_slice(&projection_data),
        );
    }

    /// Scale from scene (logical) coordinates to physical surface pixels —
    /// `(1, 1)` unless a logical projection is active. Scissor rects derived
    /// from `SceneNode::PushClip` must be multiplied by this.
    pub fn clip_scale(&self) -> (f32, f32) {
        match self.logical_size {
            Some((lw, lh)) => (
                self.surface_config.width as f32 / lw,
                self.surface_config.height as f32 / lh,
            ),
            None => (1.0, 1.0),
        }
    }

    /// The format render passes target. This is the sRGB *view* format when
    /// the surface itself is non-sRGB (WebGPU canvas), so pipelines, layer
    /// textures and the surface view all encode linear→sRGB on write.
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_config
            .view_formats
            .first()
            .copied()
            .unwrap_or(self.surface_config.format)
    }

    /// Create the view render passes must target. Plain
    /// `texture.create_view(&Default::default())` inherits the texture's own
    /// (possibly non-sRGB) format and silently skips gamma encoding — always
    /// go through here for surface render targets.
    pub fn surface_render_view(&self, output: &wgpu::SurfaceTexture) -> wgpu::TextureView {
        output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.surface_format()),
            ..Default::default()
        })
    }
}
