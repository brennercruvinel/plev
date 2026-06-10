use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

use super::config::RenderConfig;
use super::utils::ortho_projection;

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
    // Composite pipeline resources
    pub composite_pipeline: wgpu::RenderPipeline,
    pub composite_bind_group_layout: wgpu::BindGroupLayout,
    pub opacity_bind_group_layout: wgpu::BindGroupLayout,
    pub composite_sampler: wgpu::Sampler,
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
            view_formats: vec![],
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
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("projection_bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("projection_bg"),
            layout: &projection_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: projection_buffer.as_entire_binding(),
            }],
        });

        // Text atlas bind group layout (group 1)
        let text_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("text_atlas_bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // -- Pipelines (extracted for hot-reload support) --

        #[cfg(feature = "hot-reload")]
        let quad_src = crate::hot_reload::shader_source("quad.wgsl");
        #[cfg(not(feature = "hot-reload"))]
        let quad_src: std::borrow::Cow<'_, str> = include_str!("../../shaders/quad.wgsl").into();
        let quad_pipeline = Self::create_quad_pipeline(
            &device,
            &quad_src,
            &projection_bind_group_layout,
            surface_format,
            config.msaa_samples,
        );

        #[cfg(feature = "hot-reload")]
        let sdf_src = crate::hot_reload::shader_source("rect_sdf.wgsl");
        #[cfg(not(feature = "hot-reload"))]
        let sdf_src: std::borrow::Cow<'_, str> = include_str!("../../shaders/rect_sdf.wgsl").into();
        let rect_sdf_pipeline = Self::create_rect_sdf_pipeline(
            &device,
            &sdf_src,
            &projection_bind_group_layout,
            surface_format,
            config.msaa_samples,
        );

        #[cfg(feature = "hot-reload")]
        let shadow_src = crate::hot_reload::shader_source("shadow_analytic.wgsl");
        #[cfg(not(feature = "hot-reload"))]
        let shadow_src: std::borrow::Cow<'_, str> =
            include_str!("../../shaders/shadow_analytic.wgsl").into();
        let shadow_analytic_pipeline = Self::create_shadow_analytic_pipeline(
            &device,
            &shadow_src,
            &projection_bind_group_layout,
            surface_format,
            config.msaa_samples,
        );

        #[cfg(feature = "hot-reload")]
        let text_src = crate::hot_reload::shader_source("text.wgsl");
        #[cfg(not(feature = "hot-reload"))]
        let text_src: std::borrow::Cow<'_, str> = include_str!("../../shaders/text.wgsl").into();
        let text_pipeline = Self::create_text_pipeline(
            &device,
            &text_src,
            &projection_bind_group_layout,
            &text_bind_group_layout,
            surface_format,
            config.msaa_samples,
        );

        // Image atlas bind group layout (group 1 of the image pipeline);
        // same shape as the text atlas layout but RGBA8 instead of R8.
        let image_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("image_atlas_bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        #[cfg(feature = "hot-reload")]
        let image_src = crate::hot_reload::shader_source("image.wgsl");
        #[cfg(not(feature = "hot-reload"))]
        let image_src: std::borrow::Cow<'_, str> = include_str!("../../shaders/image.wgsl").into();
        let image_pipeline = Self::create_image_pipeline(
            &device,
            &image_src,
            &projection_bind_group_layout,
            &image_bind_group_layout,
            surface_format,
            config.msaa_samples,
        );

        // Composite bind group layouts (struct fields, not recreated on reload)
        let composite_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("composite_bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let opacity_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("opacity_bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        #[cfg(feature = "hot-reload")]
        let comp_src = crate::hot_reload::shader_source("composite.wgsl");
        #[cfg(not(feature = "hot-reload"))]
        let comp_src: std::borrow::Cow<'_, str> =
            include_str!("../../shaders/composite.wgsl").into();
        let composite_pipeline = Self::create_composite_pipeline(
            &device,
            &comp_src,
            &composite_bind_group_layout,
            &opacity_bind_group_layout,
            surface_format,
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
            composite_pipeline,
            composite_bind_group_layout,
            opacity_bind_group_layout,
            composite_sampler,
        }
    }
}
