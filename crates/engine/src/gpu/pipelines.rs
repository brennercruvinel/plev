use crate::compositor::{BackdropVertex, ImageVertex, QuadVertex, RectSdfVertex, ShadowVertex};
use crate::text::TextVertex;

use super::context::GpuContext;
use super::utils::premultiplied_blend;

// ---------------------------------------------------------------------------
// Pipeline creation (extracted for hot-reload support)
// ---------------------------------------------------------------------------

impl GpuContext {
    pub(super) fn create_quad_pipeline(
        device: &wgpu::Device,
        shader_source: &str,
        projection_bgl: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("quad_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("quad_pipeline_layout"),
            bind_group_layouts: &[projection_bgl],
            immediate_size: 0,
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[QuadVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(premultiplied_blend()),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    }

    pub(super) fn create_rect_sdf_pipeline(
        device: &wgpu::Device,
        shader_source: &str,
        projection_bgl: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rect_sdf_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rect_sdf_pipeline_layout"),
            bind_group_layouts: &[projection_bgl],
            immediate_size: 0,
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rect_sdf_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[RectSdfVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(premultiplied_blend()),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    }

    pub(super) fn create_shadow_analytic_pipeline(
        device: &wgpu::Device,
        shader_source: &str,
        projection_bgl: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shadow_analytic_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow_analytic_pipeline_layout"),
            bind_group_layouts: &[projection_bgl],
            immediate_size: 0,
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow_analytic_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[ShadowVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(premultiplied_blend()),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    }

    pub(super) fn create_text_pipeline(
        device: &wgpu::Device,
        shader_source: &str,
        projection_bgl: &wgpu::BindGroupLayout,
        text_bgl: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text_pipeline_layout"),
            bind_group_layouts: &[projection_bgl, text_bgl],
            immediate_size: 0,
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[TextVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(premultiplied_blend()),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    }

    pub(super) fn create_image_pipeline(
        device: &wgpu::Device,
        shader_source: &str,
        projection_bgl: &wgpu::BindGroupLayout,
        image_bgl: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("image_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("image_pipeline_layout"),
            bind_group_layouts: &[projection_bgl, image_bgl],
            immediate_size: 0,
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("image_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[ImageVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(premultiplied_blend()),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    }

    /// Pipeline for blurred-backdrop quads inside layer passes: samples
    /// the pre-blurred backdrop texture by framebuffer position, masked
    /// by a rounded-rect SDF. Group 1 reuses the composite texture+sampler
    /// bind group layout.
    pub(super) fn create_backdrop_pipeline(
        device: &wgpu::Device,
        shader_source: &str,
        projection_bgl: &wgpu::BindGroupLayout,
        composite_bgl: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
        msaa_samples: u32,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("backdrop_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("backdrop_pipeline_layout"),
            bind_group_layouts: &[projection_bgl, composite_bgl],
            immediate_size: 0,
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("backdrop_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[BackdropVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(premultiplied_blend()),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    }

    // -- Hot reload --

    #[cfg(feature = "hot-reload")]
    pub fn reload_shader(&mut self, filename: &str, source: &str) -> bool {
        let surface_format = self.surface_config.format;
        let msaa_samples = self.config.msaa_samples;
        match filename {
            "quad.wgsl" => {
                let guard = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
                let pipeline = Self::create_quad_pipeline(
                    &self.device,
                    source,
                    &self.projection_bind_group_layout,
                    surface_format,
                    msaa_samples,
                );
                if let Some(err) = pollster::block_on(guard.pop()) {
                    log::error!("Shader reload failed for quad.wgsl: {}", err);
                    return false;
                }
                self.quad_pipeline = pipeline;
                log::info!("Reloaded quad.wgsl");
                true
            }
            "rect_sdf.wgsl" => {
                let guard = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
                let pipeline = Self::create_rect_sdf_pipeline(
                    &self.device,
                    source,
                    &self.projection_bind_group_layout,
                    surface_format,
                    msaa_samples,
                );
                if let Some(err) = pollster::block_on(guard.pop()) {
                    log::error!("Shader reload failed for rect_sdf.wgsl: {}", err);
                    return false;
                }
                self.rect_sdf_pipeline = pipeline;
                log::info!("Reloaded rect_sdf.wgsl");
                true
            }
            "shadow_analytic.wgsl" => {
                let guard = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
                let pipeline = Self::create_shadow_analytic_pipeline(
                    &self.device,
                    source,
                    &self.projection_bind_group_layout,
                    surface_format,
                    msaa_samples,
                );
                if let Some(err) = pollster::block_on(guard.pop()) {
                    log::error!("Shader reload failed for shadow_analytic.wgsl: {}", err);
                    return false;
                }
                self.shadow_analytic_pipeline = pipeline;
                log::info!("Reloaded shadow_analytic.wgsl");
                true
            }
            "text.wgsl" => {
                let guard = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
                let pipeline = Self::create_text_pipeline(
                    &self.device,
                    source,
                    &self.projection_bind_group_layout,
                    &self.text_bind_group_layout,
                    surface_format,
                    msaa_samples,
                );
                if let Some(err) = pollster::block_on(guard.pop()) {
                    log::error!("Shader reload failed for text.wgsl: {}", err);
                    return false;
                }
                self.text_pipeline = pipeline;
                log::info!("Reloaded text.wgsl");
                true
            }
            "image.wgsl" => {
                let guard = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
                let pipeline = Self::create_image_pipeline(
                    &self.device,
                    source,
                    &self.projection_bind_group_layout,
                    &self.image_bind_group_layout,
                    surface_format,
                    msaa_samples,
                );
                if let Some(err) = pollster::block_on(guard.pop()) {
                    log::error!("Shader reload failed for image.wgsl: {}", err);
                    return false;
                }
                self.image_pipeline = pipeline;
                log::info!("Reloaded image.wgsl");
                true
            }
            "backdrop.wgsl" => {
                let guard = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
                let pipeline = Self::create_backdrop_pipeline(
                    &self.device,
                    source,
                    &self.projection_bind_group_layout,
                    &self.composite_bind_group_layout,
                    surface_format,
                    msaa_samples,
                );
                if let Some(err) = pollster::block_on(guard.pop()) {
                    log::error!("Shader reload failed for backdrop.wgsl: {}", err);
                    return false;
                }
                self.backdrop_pipeline = pipeline;
                log::info!("Reloaded backdrop.wgsl");
                true
            }
            "composite.wgsl" => {
                let guard = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
                let pipeline = Self::create_composite_pipeline(
                    &self.device,
                    source,
                    &self.composite_bind_group_layout,
                    &self.opacity_bind_group_layout,
                    surface_format,
                );
                if let Some(err) = pollster::block_on(guard.pop()) {
                    log::error!(
                        "Shader reload failed for composite.wgsl (GpuContext): {}",
                        err
                    );
                    return false;
                }
                self.composite_pipeline = pipeline;
                log::info!("Reloaded composite.wgsl (GpuContext)");
                true
            }
            _ => false,
        }
    }

    pub(super) fn create_composite_pipeline(
        device: &wgpu::Device,
        shader_source: &str,
        composite_bgl: &wgpu::BindGroupLayout,
        opacity_bgl: &wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("composite_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("composite_pipeline_layout"),
            bind_group_layouts: &[composite_bgl, opacity_bgl],
            immediate_size: 0,
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("composite_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(premultiplied_blend()),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    }
}
