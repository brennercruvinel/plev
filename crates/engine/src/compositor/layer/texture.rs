use super::Layer;
use crate::compositor::ResolveResources;

impl Layer {
    pub(crate) fn ensure_texture(&mut self, res: &ResolveResources<'_>, width: u32, height: u32) {
        let want_msaa = res.msaa_samples > 1;
        if self.tex_width == width
            && self.tex_height == height
            && self.texture.is_some()
            && self.msaa_texture.is_some() == want_msaa
        {
            if let Some(ref buf) = self.opacity_buffer {
                res.queue
                    .write_buffer(buf, 0, bytemuck::bytes_of(&self.opacity));
            }
            return;
        }

        let tex_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // With a single sample the layer pass renders directly to the layer
        // texture -- no MSAA texture, no resolve target.
        let (msaa_texture, msaa_view) = if want_msaa {
            let texture = res.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("layer_msaa_texture"),
                size: tex_size,
                mip_level_count: 1,
                sample_count: res.msaa_samples,
                dimension: wgpu::TextureDimension::D2,
                format: res.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            (Some(texture), Some(view))
        } else {
            (None, None)
        };

        let texture = res.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("layer_texture"),
            size: tex_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: res.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let composite_bind_group = res.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("layer_composite_bg"),
            layout: res.composite_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(res.sampler),
                },
            ],
        });

        let opacity_buffer = res.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("layer_opacity_buf"),
            size: 4,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        res.queue
            .write_buffer(&opacity_buffer, 0, bytemuck::bytes_of(&self.opacity));

        let opacity_bind_group = res.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("layer_opacity_bg"),
            layout: res.opacity_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: opacity_buffer.as_entire_binding(),
            }],
        });

        self.msaa_texture = msaa_texture;
        self.msaa_view = msaa_view;
        self.texture = Some(texture);
        self.texture_view = Some(texture_view);
        self.composite_bind_group = Some(composite_bind_group);
        self.opacity_buffer = Some(opacity_buffer);
        self.opacity_bind_group = Some(opacity_bind_group);
        self.tex_width = width;
        self.tex_height = height;
        self.dirty = true;
    }
}
