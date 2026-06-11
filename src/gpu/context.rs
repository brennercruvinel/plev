use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

use super::config::RenderConfig;
use super::utils::{ortho_projection, texture_sampler_bgl, uniform_bgl};

/// Per-pipeline WGSL source: read from disk when `hot-reload` is enabled,
/// embedded at compile time otherwise.
macro_rules! shader_src {
    ($name:literal) => {{
        #[cfg(feature = "hot-reload")]
        {
            crate::hot_reload::shader_source($name)
        }
        #[cfg(not(feature = "hot-reload"))]
        {
            std::borrow::Cow::<'static, str>::from(include_str!(concat!("../../shaders/", $name)))
        }
    }};
}

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: Option<wgpu::Surface<'static>>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub config: RenderConfig,
    pub projection_buffer: wgpu::Buffer,
    pub projection_bind_group_layout: wgpu::BindGroupLayout,
    pub projection_bind_group: wgpu::BindGroup,
    pub quad_pipeline: wgpu::RenderPipeline,
    pub rect_sdf_pipeline: wgpu::RenderPipeline,
    pub shadow_analytic_pipeline: wgpu::RenderPipeline,
    pub text_pipeline: wgpu::RenderPipeline,
    pub text_bind_group_layout: wgpu::BindGroupLayout,
    pub image_pipeline: wgpu::RenderPipeline,
    pub image_bind_group_layout: wgpu::BindGroupLayout,
    pub image_atlas: super::image::ImageAtlasGpu,
    /// Draws blurred-backdrop quads inside layer passes (group 1 reuses
    /// the composite texture+sampler layout).
    pub backdrop_pipeline: wgpu::RenderPipeline,
    // Composite pipeline resources
    pub composite_pipeline: wgpu::RenderPipeline,
    pub composite_bind_group_layout: wgpu::BindGroupLayout,
    pub opacity_bind_group_layout: wgpu::BindGroupLayout,
    pub composite_sampler: wgpu::Sampler,
    /// Logical size set via [`set_projection`]; `None` while scene
    /// coordinates match the physical surface (the default). Used to scale
    /// `SceneNode::PushClip` rects (logical pixels) into physical scissor
    /// rects, so clipping holds on HiDPI surfaces.
    ///
    /// [`set_projection`]: GpuContext::set_projection
    pub(crate) logical_size: Option<(f32, f32)>,
}

impl GpuContext {
    pub async fn new(window: Arc<Window>) -> Self {
        Self::new_with_config(window, RenderConfig::default()).await
    }

    /// Sync the image atlas texture with images loaded since the last
    /// frame. Call once per frame before encoding render passes.
    pub fn prepare_images(&mut self) {
        self.image_atlas
            .prepare(&self.device, &self.queue, &self.image_bind_group_layout);
    }

    pub async fn new_with_config(window: Arc<Window>, mut config: RenderConfig) -> Self {
        config.msaa_samples = config.effective_msaa_samples();
        crate::path::set_default_tolerance(config.path_tolerance);

        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find a suitable GPU adapter");

        log::info!("GPU adapter: {}", adapter.get_info().name);
        log::info!("GPU: requesting device...");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("plev_device"),
                required_features: wgpu::Features::empty(),
                #[cfg(not(target_arch = "wasm32"))]
                required_limits: wgpu::Limits::default(),
                #[cfg(target_arch = "wasm32")]
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: Default::default(),
                experimental_features: Default::default(),
            })
            .await
            .expect("Failed to create device");
        log::info!(
            "GPU: device created, configuring surface {}x{}...",
            size.width,
            size.height
        );

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // Render-target format. WebGPU canvases only expose non-sRGB formats
        // (bgra8unorm), which would skip the linear→sRGB encode-on-write that
        // every shader and clear color assumes — colors come out ~2.5× darker
        // (#303030 background measured (8,8,8)). Registering the sRGB variant
        // as a view format and rendering into that view restores the encode.
        // On desktop the surface is already sRGB and this is a no-op.
        let render_format = surface_format.add_srgb_suffix();
        let view_formats = if render_format != surface_format {
            vec![render_format]
        } else {
            vec![]
        };

        let width = size.width.max(1);
        let height = size.height.max(1);

        let present_mode = if matches!(
            config.present_mode,
            wgpu::PresentMode::AutoVsync | wgpu::PresentMode::AutoNoVsync
        ) || surface_caps.present_modes.contains(&config.present_mode)
        {
            config.present_mode
        } else {
            log::warn!(
                "Present mode {:?} unsupported by surface -- falling back to AutoVsync",
                config.present_mode
            );
            wgpu::PresentMode::AutoVsync
        };

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats,
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Projection uniform buffer
        let projection_data = ortho_projection(width as f32, height as f32);
        let projection_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("projection_buffer"),
            contents: bytemuck::cast_slice(&projection_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let projection_bind_group_layout =
            uniform_bgl(&device, "projection_bgl", wgpu::ShaderStages::VERTEX);

        let projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("projection_bg"),
            layout: &projection_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: projection_buffer.as_entire_binding(),
            }],
        });

        // Text atlas bind group layout (group 1)
        let text_bind_group_layout = texture_sampler_bgl(&device, "text_atlas_bgl");

        // -- Pipelines (extracted for hot-reload support) --

        let quad_pipeline = Self::create_quad_pipeline(
            &device,
            &shader_src!("quad.wgsl"),
            &projection_bind_group_layout,
            render_format,
            config.msaa_samples,
        );

        let rect_sdf_pipeline = Self::create_rect_sdf_pipeline(
            &device,
            &shader_src!("rect_sdf.wgsl"),
            &projection_bind_group_layout,
            render_format,
            config.msaa_samples,
        );

        let shadow_analytic_pipeline = Self::create_shadow_analytic_pipeline(
            &device,
            &shader_src!("shadow_analytic.wgsl"),
            &projection_bind_group_layout,
            render_format,
            config.msaa_samples,
        );

        let text_pipeline = Self::create_text_pipeline(
            &device,
            &shader_src!("text.wgsl"),
            &projection_bind_group_layout,
            &text_bind_group_layout,
            render_format,
            config.msaa_samples,
        );

        // Image atlas bind group layout (group 1 of the image pipeline);
        // same shape as the text atlas layout but RGBA8 instead of R8.
        let image_bind_group_layout = texture_sampler_bgl(&device, "image_atlas_bgl");

        let image_pipeline = Self::create_image_pipeline(
            &device,
            &shader_src!("image.wgsl"),
            &projection_bind_group_layout,
            &image_bind_group_layout,
            render_format,
            config.msaa_samples,
        );

        // Composite bind group layouts (struct fields, not recreated on reload)
        let composite_bind_group_layout = texture_sampler_bgl(&device, "composite_bgl");

        let opacity_bind_group_layout =
            uniform_bgl(&device, "opacity_bgl", wgpu::ShaderStages::FRAGMENT);

        let composite_pipeline = Self::create_composite_pipeline(
            &device,
            &shader_src!("composite.wgsl"),
            &composite_bind_group_layout,
            &opacity_bind_group_layout,
            render_format,
        );

        let backdrop_pipeline = Self::create_backdrop_pipeline(
            &device,
            &shader_src!("backdrop.wgsl"),
            &projection_bind_group_layout,
            &composite_bind_group_layout,
            render_format,
            config.msaa_samples,
        );

        let composite_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("composite_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            device,
            queue,
            surface: Some(surface),
            surface_config,
            config,
            projection_buffer,
            projection_bind_group_layout,
            projection_bind_group,
            quad_pipeline,
            rect_sdf_pipeline,
            shadow_analytic_pipeline,
            text_pipeline,
            text_bind_group_layout,
            image_pipeline,
            image_bind_group_layout,
            image_atlas: super::image::ImageAtlasGpu::new(),
            backdrop_pipeline,
            composite_pipeline,
            composite_bind_group_layout,
            opacity_bind_group_layout,
            composite_sampler,
            logical_size: None,
        }
    }
}
