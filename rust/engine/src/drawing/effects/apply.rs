use wgpu::util::DeviceExt;

use super::processor::EffectProcessor;
use super::types::*;
use crate::gpu::texture_pool::{TextureHandle, TexturePool};

/// Shared GPU context for effect passes.
pub(crate) struct EffectContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub pool: &'a mut TexturePool,
    pub source_view: &'a wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    /// Physical px per logical px, per axis (`GpuContext::clip_scale`):
    /// a sigma is in logical px, and the blur spreads as far in them at
    /// every scale.
    pub scale: (f32, f32),
}

// ---------------------------------------------------------------------------
// Effect application methods
// ---------------------------------------------------------------------------

impl EffectProcessor {
    /// Create a bind group for a source texture (group 0).
    pub fn create_source_bind_group(
        &self,
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("effect_source_bg"),
            layout: &self.effect_texture_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.linear_sampler),
                },
            ],
        })
    }

    /// One-shot uniform buffer + bind group for a blur pass. Transient on
    /// purpose: `queue.write_buffer` stages ALL writes before the next
    /// submit executes any pass, so reusing one buffer across the H and V
    /// passes (or across several blurs in one frame -- e.g. backdrop
    /// nodes) would make every pass read the LAST write.
    fn create_blur_uniform_bg(&self, device: &wgpu::Device, contents: &[u8]) -> wgpu::BindGroup {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("blur_uniforms"),
            contents,
            usage: wgpu::BufferUsages::UNIFORM,
        });
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blur_uniform_bg"),
            layout: &self.blur_uniform_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }

    /// The pipeline and the uniforms of one blur pass along `direction`,
    /// at `scale` physical px per logical px on that axis. Scale 1 is the
    /// blur as it always was (`fs_main`, 13 taps of
    /// `gaussian_weights`); any other lays that kernel over the physical
    /// texels (`fs_scaled`, `scaled_weights`), so a sigma in logical px
    /// reaches as far at every scale.
    fn blur_pass(
        &self,
        device: &wgpu::Device,
        direction: [f32; 2],
        texel_size: [f32; 2],
        (sigma, scale): (f32, f32),
    ) -> (&wgpu::RenderPipeline, wgpu::BindGroup) {
        if scale == 1.0 {
            let uniforms = BlurUniforms {
                direction,
                texel_size,
                weights: gaussian_weights(sigma),
            };
            let bg = self.create_blur_uniform_bg(device, bytemuck::bytes_of(&uniforms));
            return (&self.blur_pipeline, bg);
        }
        let (taps, weights) = scaled_weights(sigma, scale);
        let uniforms = ScaledBlurUniforms {
            direction,
            texel_size,
            taps: [taps as f32, 0.0, 0.0, 0.0],
            weights,
        };
        let bg = self.create_blur_uniform_bg(device, bytemuck::bytes_of(&uniforms));
        (&self.scaled_blur_pipeline, bg)
    }

    /// Apply a two-pass Gaussian blur of `sigma` logical px. Returns a
    /// TextureHandle with the result.
    pub(crate) fn apply_blur(&self, ctx: &mut EffectContext<'_>, sigma: f32) -> TextureHandle {
        let texel_size = [1.0 / ctx.width as f32, 1.0 / ctx.height as f32];

        let temp_a = ctx
            .pool
            .acquire(ctx.device, ctx.width, ctx.height, self.surface_format);
        let temp_b = ctx
            .pool
            .acquire(ctx.device, ctx.width, ctx.height, self.surface_format);

        // Horizontal pass: source -> temp_a
        {
            let (pipeline, blur_uniform_bg) =
                self.blur_pass(ctx.device, [1.0, 0.0], texel_size, (sigma, ctx.scale.0));
            let source_bg = self.create_source_bind_group(ctx.device, ctx.source_view);

            let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blur_h_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: temp_a.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &source_bg, &[]);
            pass.set_bind_group(1, &blur_uniform_bg, &[]);
            pass.draw(0..3, 0..1);
        }

        // Vertical pass: temp_a -> temp_b
        {
            let (pipeline, blur_uniform_bg) =
                self.blur_pass(ctx.device, [0.0, 1.0], texel_size, (sigma, ctx.scale.1));
            let temp_a_bg = self.create_source_bind_group(ctx.device, temp_a.view());

            let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blur_v_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: temp_b.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &temp_a_bg, &[]);
            pass.set_bind_group(1, &blur_uniform_bg, &[]);
            pass.draw(0..3, 0..1);
        }

        ctx.pool.release(temp_a);
        temp_b
    }

    /// Apply shadow extraction + blur. Returns a TextureHandle with the blurred shadow.
    pub(crate) fn apply_shadow(
        &self,
        ctx: &mut EffectContext<'_>,
        sigma: f32,
        color: [f32; 4],
    ) -> TextureHandle {
        // Step 1: Extract silhouette
        let silhouette = ctx
            .pool
            .acquire(ctx.device, ctx.width, ctx.height, self.surface_format);

        {
            // Transient uniforms for the same staged-write reason as blur.
            let shadow_uniform_buffer =
                ctx.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("shadow_uniforms"),
                        contents: bytemuck::bytes_of(&ShadowUniforms { color }),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });
            let source_bg = self.create_source_bind_group(ctx.device, ctx.source_view);
            let shadow_uniform_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("shadow_uniform_bg"),
                layout: &self.shadow_uniform_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: shadow_uniform_buffer.as_entire_binding(),
                }],
            });

            let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow_extract_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: silhouette.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.shadow_pipeline);
            pass.set_bind_group(0, &source_bg, &[]);
            pass.set_bind_group(1, &shadow_uniform_bg, &[]);
            pass.draw(0..3, 0..1);
        }

        // Step 2: Blur the silhouette
        if sigma > 0.0 {
            let sil_view = silhouette.view().clone();
            let blurred = self.apply_blur(
                &mut EffectContext {
                    device: ctx.device,
                    queue: ctx.queue,
                    encoder: ctx.encoder,
                    pool: ctx.pool,
                    source_view: &sil_view,
                    width: ctx.width,
                    height: ctx.height,
                    scale: ctx.scale,
                },
                sigma,
            );
            ctx.pool.release(silhouette);
            blurred
        } else {
            silhouette
        }
    }

    /// Composite a texture onto the current render pass with the given opacity.
    pub fn composite_pass(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'_>,
        source_view: &wgpu::TextureView,
        alpha: f32,
    ) {
        let uniforms = CompositeUniforms {
            alpha,
            _padding: [0.0; 3],
        };
        queue.write_buffer(
            &self.composite_uniform_buffer,
            0,
            bytemuck::bytes_of(&uniforms),
        );

        let source_bg = self.create_source_bind_group(device, source_view);
        let composite_uniform_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("composite_uniform_bg"),
            layout: &self.composite_uniform_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.composite_uniform_buffer.as_entire_binding(),
            }],
        });

        pass.set_pipeline(&self.composite_pipeline);
        pass.set_bind_group(0, &source_bg, &[]);
        pass.set_bind_group(1, &composite_uniform_bg, &[]);
        pass.draw(0..3, 0..1);
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }
}
