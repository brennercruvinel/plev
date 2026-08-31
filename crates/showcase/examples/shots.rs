//! Print de cada seção do showcase, uma por uma, num ÚNICO TextSystem —
//! o mesmo estado acumulado de uma sessão real navegando as 14 seções.
//! Critério de qualidade: olhar cada imagem.

use engine::compositor::{Compositor, SceneNode};
use engine::text::{TextSystem, TextVertex};
use showcase::view::{Section, ShowcaseView};
use wgpu::util::DeviceExt;

const W: f32 = 1400.0;
const H: f32 = 900.0;
const SCALE: f32 = 2.0;

fn main() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .try_init();
    let out_dir = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    pollster::block_on(run(&out_dir));
}

async fn run(out_dir: &str) {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .expect("no adapter");
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .expect("no device");

    let atlas_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bgl"),
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
    let proj_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("proj"),
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
    let sx = 2.0 / W;
    let sy = -2.0 / H;
    #[rustfmt::skip]
    let proj: [f32; 16] = [sx, 0.0, 0.0, 0.0, 0.0, sy, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0];
    let proj_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&proj),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let proj_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &proj_bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: proj_buf.as_entire_binding(),
        }],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("../../engine/src/gpu/shaders/text.wgsl").into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&proj_bgl, &atlas_bgl],
        immediate_size: 0,
    });
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
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
                format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });

    // UM TextSystem para a sessão inteira, como no app real.
    let mut text = TextSystem::new(&device, &atlas_bgl);
    let _ = text.set_raster_scale(SCALE);

    let mut view = ShowcaseView::new(W, H);
    for section in Section::ALL {
        view.jump_to_section(section.title());
        let mut c = Compositor::new();
        view.render(&mut c);

        // Uma frame igual à do runner: begin, resolve por camada, finish.
        text.begin_frame();
        let mut buffers: Vec<(Vec<TextVertex>, Vec<u32>)> = Vec::new();
        for layer in c.layers() {
            let nodes: Vec<SceneNode> = layer
                .nodes()
                .iter()
                .filter(|n| matches!(n, SceneNode::Text { .. }))
                .cloned()
                .collect();
            if nodes.is_empty() {
                continue;
            }
            buffers.push(text.resolve_for_layer(&device, &queue, &atlas_bgl, &nodes));
        }
        text.finish_frame();

        // Desenha e lê de volta.
        let (pw, ph) = ((W * SCALE) as u32, (H * SCALE) as u32);
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: pw,
                height: ph,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let tview = target.create_view(&Default::default());
        let gpu_bufs: Vec<_> = buffers
            .iter()
            .filter(|(v, i)| !v.is_empty() && !i.is_empty())
            .map(|(v, i)| {
                (
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(v),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(i),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
                    i.len() as u32,
                )
            })
            .collect();
        let bpr = (pw * 4).div_ceil(256) * 256;
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (bpr * ph) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &tview,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.02,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &proj_bg, &[]);
            pass.set_bind_group(1, &text.atlas_bind_group, &[]);
            for (v, i, n) in &gpu_bufs {
                pass.set_vertex_buffer(0, v.slice(..));
                pass.set_index_buffer(i.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..*n, 0, 0..1);
            }
        }
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bpr),
                    rows_per_image: Some(ph),
                },
            },
            wgpu::Extent3d {
                width: pw,
                height: ph,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(Some(enc.finish()));
        let slice = readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        rx.recv().unwrap().unwrap();
        let data = slice.get_mapped_range();
        let mut rgba = Vec::with_capacity((pw * ph * 4) as usize);
        for row in 0..ph {
            let s = (row * bpr) as usize;
            rgba.extend_from_slice(&data[s..s + (pw * 4) as usize]);
        }
        drop(data);
        readback.unmap();
        let path = format!("{out_dir}/{}.png", section.title());
        engine::text::probe::write_png(&path, pw, ph, &rgba);
        println!("shot {path}");
    }
}
