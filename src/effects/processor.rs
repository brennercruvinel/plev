use wgpu::util::DeviceExt;

use super::types::*;

// ---------------------------------------------------------------------------
// EffectProcessor -- owns pipelines and processes effects
// ---------------------------------------------------------------------------

pub struct EffectProcessor {
    pub blur_pipeline: wgpu::RenderPipeline,
    pub shadow_pipeline: wgpu::RenderPipeline,
    pub composite_pipeline: wgpu::RenderPipeline,
    pub effect_texture_bgl: wgpu::BindGroupLayout,
    pub blur_uniform_bgl: wgpu::BindGroupLayout,
    pub shadow_uniform_bgl: wgpu::BindGroupLayout,
    pub composite_uniform_bgl: wgpu::BindGroupLayout,
    pub linear_sampler: wgpu::Sampler,
    pub(super) blur_uniform_buffer: wgpu::Buffer,
    pub(super) shadow_uniform_buffer: wgpu::Buffer,
    pub(super) composite_uniform_buffer: wgpu::Buffer,
    pub(super) surface_format: wgpu::TextureFormat,
}

impl EffectProcessor {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("effect_linear_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Texture + sampler bind group layout (group 0, shared by all effect shaders)
        let effect_texture_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("effect_texture_bgl"),
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

        // Uniform bind group layout (group 1) -- same shape for all effect uniforms
        let uniform_bgl_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let blur_uniform_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("blur_uniform_bgl"),
            entries: &[uniform_bgl_entry],
        });

        let shadow_uniform_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shadow_uniform_bgl"),
                entries: &[uniform_bgl_entry],
            });

        let composite_uniform_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("composite_uniform_bgl"),
                entries: &[uniform_bgl_entry],
            });

        // -- Effect pipelines (extracted for hot-reload support) --

        #[cfg(feature = "hot-reload")]
        let blur_src = crate::hot_reload::shader_source("blur.wgsl");
        #[cfg(not(feature = "hot-reload"))]
        let blur_src: std::borrow::Cow<'_, str> = include_str!("../../shaders/blur.wgsl").into();
        let blur_pipeline = Self::create_blur_pipeline(
            device,
            &blur_src,
            &effect_texture_bgl,
            &blur_uniform_bgl,
            surface_format,
        );

        #[cfg(feature = "hot-reload")]
        let shadow_src = crate::hot_reload::shader_source("shadow.wgsl");
        #[cfg(not(feature = "hot-reload"))]
        let shadow_src: std::borrow::Cow<'_, str> =
            include_str!("../../shaders/shadow.wgsl").into();
        let shadow_pipeline = Self::create_shadow_pipeline(
            device,
            &shadow_src,
            &effect_texture_bgl,
            &shadow_uniform_bgl,
            surface_format,
        );

        #[cfg(feature = "hot-reload")]
        let comp_src = crate::hot_reload::shader_source("composite.wgsl");
        #[cfg(not(feature = "hot-reload"))]
        let comp_src: std::borrow::Cow<'_, str> =
            include_str!("../../shaders/composite.wgsl").into();
        let composite_pipeline = Self::create_effect_composite_pipeline(
            device,
            &comp_src,
            &effect_texture_bgl,
            &composite_uniform_bgl,
            surface_format,
        );

        // Uniform buffers (reused across frames)
        let blur_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("blur_uniforms"),
            contents: bytemuck::bytes_of(&BlurUniforms {
                direction: [1.0, 0.0],
                texel_size: [0.0, 0.0],
                weights: [0.0; 16],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let shadow_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shadow_uniforms"),
            contents: bytemuck::bytes_of(&ShadowUniforms {
                color: [0.0, 0.0, 0.0, 0.5],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let composite_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("composite_uniforms"),
                contents: bytemuck::bytes_of(&CompositeUniforms {
                    alpha: 1.0,
                    _padding: [0.0; 3],
                }),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        Self {
            blur_pipeline,
            shadow_pipeline,
            composite_pipeline,
            effect_texture_bgl,
            blur_uniform_bgl,
            shadow_uniform_bgl,
            composite_uniform_bgl,
            linear_sampler,
            blur_uniform_buffer,
            shadow_uniform_buffer,
            composite_uniform_buffer,
            surface_format,
        }
    }
}
