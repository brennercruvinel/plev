use super::EffectProcessor;

// ---------------------------------------------------------------------------
// Pipeline creation (extracted for hot-reload support)
// ---------------------------------------------------------------------------

impl EffectProcessor {
    pub(super) fn create_blur_pipeline(
        device: &wgpu::Device,
        shader_source: &str,
        texture_bgl: &wgpu::BindGroupLayout,
        uniform_bgl: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blur_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("blur_pipeline_layout"),
            bind_group_layouts: &[texture_bgl, uniform_bgl],
            immediate_size: 0,
        });
        create_fullscreen_pipeline(
            device,
            "blur_pipeline",
            &layout,
            &shader,
            surface_format,
            None,
        )
    }

    pub(super) fn create_shadow_pipeline(
        device: &wgpu::Device,
        shader_source: &str,
        texture_bgl: &wgpu::BindGroupLayout,
        uniform_bgl: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shadow_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow_pipeline_layout"),
            bind_group_layouts: &[texture_bgl, uniform_bgl],
            immediate_size: 0,
        });
        create_fullscreen_pipeline(
            device,
            "shadow_pipeline",
            &layout,
            &shader,
            surface_format,
            None,
        )
    }

    pub(super) fn create_effect_composite_pipeline(
        device: &wgpu::Device,
        shader_source: &str,
        texture_bgl: &wgpu::BindGroupLayout,
        uniform_bgl: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("composite_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("composite_pipeline_layout"),
            bind_group_layouts: &[texture_bgl, uniform_bgl],
            immediate_size: 0,
        });
        create_fullscreen_pipeline(
            device,
            "composite_pipeline",
            &layout,
            &shader,
            surface_format,
            Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
        )
    }
}

// ---------------------------------------------------------------------------
// Hot reload
// ---------------------------------------------------------------------------

#[cfg(feature = "hot-reload")]
impl EffectProcessor {
    pub fn reload_shader(&mut self, device: &wgpu::Device, filename: &str, source: &str) -> bool {
        let sf = self.surface_format;
        match filename {
            "blur.wgsl" => {
                let guard = device.push_error_scope(wgpu::ErrorFilter::Validation);
                let pipeline = Self::create_blur_pipeline(
                    device,
                    source,
                    &self.effect_texture_bgl,
                    &self.blur_uniform_bgl,
                    sf,
                );
                if let Some(err) = pollster::block_on(guard.pop()) {
                    log::error!("Shader reload failed for blur.wgsl: {}", err);
                    return false;
                }
                self.blur_pipeline = pipeline;
                log::info!("Reloaded blur.wgsl");
                true
            }
            "shadow.wgsl" => {
                let guard = device.push_error_scope(wgpu::ErrorFilter::Validation);
                let pipeline = Self::create_shadow_pipeline(
                    device,
                    source,
                    &self.effect_texture_bgl,
                    &self.shadow_uniform_bgl,
                    sf,
                );
                if let Some(err) = pollster::block_on(guard.pop()) {
                    log::error!("Shader reload failed for shadow.wgsl: {}", err);
                    return false;
                }
                self.shadow_pipeline = pipeline;
                log::info!("Reloaded shadow.wgsl");
                true
            }
            "composite.wgsl" => {
                let guard = device.push_error_scope(wgpu::ErrorFilter::Validation);
                let pipeline = Self::create_effect_composite_pipeline(
                    device,
                    source,
                    &self.effect_texture_bgl,
                    &self.composite_uniform_bgl,
                    sf,
                );
                if let Some(err) = pollster::block_on(guard.pop()) {
                    log::error!(
                        "Shader reload failed for composite.wgsl (EffectProcessor): {}",
                        err
                    );
                    return false;
                }
                self.composite_pipeline = pipeline;
                log::info!("Reloaded composite.wgsl (EffectProcessor)");
                true
            }
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Shared fullscreen pipeline builder
// ---------------------------------------------------------------------------

pub(crate) fn create_fullscreen_pipeline(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}
