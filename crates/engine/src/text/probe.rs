//! Headless glyph rendering: what the rasterizer actually puts on screen.
//!
//! The project's conventions require pixels for a visual claim, and the
//! window path cannot supply them in CI (or wherever the OS denies screen
//! capture). This renders text through the real [`TextSystem`] — same atlas,
//! same shader, same blend state — into an offscreen texture and returns
//! RGBA8.
//!
//! Multi-frame rendering (`render_frames`) exists because atlas-lifecycle
//! behavior only shows under accumulated state: grown atlas, evictions,
//! layers retained across frames.

use crate::compositor::{SceneNode, TextNodeKey};
use crate::text::{TextStyle, TextSystem, TextVertex};
use wgpu::util::DeviceExt;

/// Above the cleared background (linear 0.15 -> ~110 in sRGB8) and below
/// antialiased glyph edges, so it counts coverage rather than canvas.
const INK_THRESHOLD: u8 = 170;

/// One line of the scene: what to draw, in which style, and where.
#[derive(Clone)]
pub struct Specimen {
    pub text: String,
    pub style: TextStyle,
    pub x: f32,
    pub y: f32,
}

impl Specimen {
    pub fn new(text: impl Into<String>, style: TextStyle, x: f32, y: f32) -> Self {
        Self {
            text: text.into(),
            style,
            x,
            y,
        }
    }
}

/// An offscreen render: RGBA8, row-major, `width * height * 4` bytes.
pub struct Rendered {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Rendered {
    /// Rows `y..y + h`, for comparing one band between renders whose other
    /// content deliberately differs.
    pub fn band(&self, y: u32, h: u32) -> &[u8] {
        let start = (y * self.width * 4) as usize;
        let end = (((y + h) * self.width * 4) as usize).min(self.rgba.len());
        &self.rgba[start..end]
    }

    /// Count of pixels carrying glyph coverage.
    ///
    /// The threshold sits well above the cleared background: the clear color
    /// is linear 0.15, which an sRGB target encodes to about 110, so a naive
    /// `> 60` counts the whole canvas and any comparison built on it is
    /// vacuous.
    pub fn ink(&self) -> usize {
        self.rgba
            .chunks_exact(4)
            .filter(|p| p[0] > INK_THRESHOLD)
            .count()
    }

    /// Horizontal extent of the ink in rows `y..y + h`, as
    /// `(first_x, last_x)` in physical pixels, or `None` if the band is
    /// blank.
    pub fn ink_extent(&self, y: u32, h: u32) -> Option<(u32, u32)> {
        let (mut first, mut last) = (u32::MAX, 0u32);
        for row in y..(y + h).min(self.height) {
            for col in 0..self.width {
                let i = ((row * self.width + col) * 4) as usize;
                if self.rgba[i] > INK_THRESHOLD {
                    first = first.min(col);
                    last = last.max(col);
                }
            }
        }
        (first != u32::MAX).then_some((first, last))
    }

    pub fn write_png(&self, path: &str) {
        write_png(path, self.width, self.height, &self.rgba);
    }
}

/// Distinct glyph/size/weight combinations, enough of them that the glyph
/// atlas must grow partway through the frame.
///
/// The initial atlas holds a few hundred glyphs; a real screen of widgets
/// exceeds that routinely, so pixel tests that want realistic atlas
/// pressure start from these.
pub fn atlas_filling_specimens() -> Vec<(String, TextStyle)> {
    const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut out = Vec::new();
    for (i, weight) in [400u16, 500, 600, 700].iter().enumerate() {
        for size in [11.0f32, 13.0, 15.0, 17.0, 19.0, 23.0, 29.0, 31.0] {
            out.push((
                format!("{i}{ALPHABET}"),
                TextStyle::new(size)
                    .with_line_height(size * 1.4)
                    .with_weight(*weight),
            ));
        }
    }
    out
}

/// A compositor layer: its text, and whether the app re-resolves it every
/// frame.
///
/// `static_layer` models a sidebar or header whose scene never changes: it
/// is skipped by `resolve_layer_text` and keeps the vertex buffer it built
/// on frame one, still pointing into the shared glyph atlas.
pub struct Layer {
    pub specimens: Vec<Specimen>,
    pub redraws_every_frame: bool,
}

impl Layer {
    /// Re-resolved every frame (animated content, hover states).
    pub fn dynamic(specimens: Vec<Specimen>) -> Self {
        Self {
            specimens,
            redraws_every_frame: true,
        }
    }

    /// Resolved once and then skipped, like a static panel.
    pub fn static_layer(specimens: Vec<Specimen>) -> Self {
        Self {
            specimens,
            redraws_every_frame: false,
        }
    }
}

/// Render `frames` frames of `layers`, then draw every layer's *current*
/// vertex buffer — static layers included, still holding the vertices they
/// built on frame one.
///
/// This is the shape of the real render loop: retained buffers and the
/// shared atlas interact across frames, not within one.
pub fn render_frames(
    layers: &[Layer],
    frames: usize,
    logical_w: u32,
    logical_h: u32,
    raster_scale: f32,
) -> Option<Rendered> {
    pollster::block_on(render_frames_async(
        layers,
        frames,
        logical_w,
        logical_h,
        raster_scale,
    ))
}

/// Render `specimens` into a `logical_w x logical_h` scene at `raster_scale`.
///
/// `None` when no GPU adapter is available, so callers can skip instead of
/// failing on machines without one.
pub fn render(
    specimens: &[Specimen],
    logical_w: u32,
    logical_h: u32,
    raster_scale: f32,
) -> Option<Rendered> {
    pollster::block_on(render_async(specimens, logical_w, logical_h, raster_scale))
}

async fn render_frames_async(
    layers: &[Layer],
    frames: usize,
    logical_w: u32,
    logical_h: u32,
    scale: f32,
) -> Option<Rendered> {
    render_impl(RenderJob {
        layers,
        frames,
        logical_w,
        logical_h,
        scale,
    })
    .await
}

struct RenderJob<'a> {
    layers: &'a [Layer],
    frames: usize,
    logical_w: u32,
    logical_h: u32,
    scale: f32,
}

async fn render_async(
    specimens: &[Specimen],
    logical_w: u32,
    logical_h: u32,
    scale: f32,
) -> Option<Rendered> {
    let layers = [Layer::dynamic(
        specimens
            .iter()
            .map(|s| Specimen::new(s.text.clone(), s.style.clone(), s.x, s.y))
            .collect(),
    )];
    render_impl(RenderJob {
        layers: &layers,
        frames: 1,
        logical_w,
        logical_h,
        scale,
    })
    .await
}

async fn render_impl(job: RenderJob<'_>) -> Option<Rendered> {
    let RenderJob {
        layers,
        frames,
        logical_w,
        logical_h,
        scale,
    } = job;
    let w = logical_w;
    let h = logical_h;
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .ok()?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("text_probe"),
            ..Default::default()
        })
        .await
        .ok()?;

    // --- pipeline: the engine's own text shader ---------------------------
    let atlas_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
    let proj_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("proj_bgl"),
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

    // Physical target; the projection is in logical units so the scene is
    // laid out exactly as the window path lays it out.
    let (pw, ph) = ((w as f32 * scale) as u32, (h as f32 * scale) as u32);
    let sx = 2.0 / w as f32;
    let sy = -2.0 / h as f32;
    #[rustfmt::skip]
    let proj: [f32; 16] = [
        sx, 0.0, 0.0, 0.0,
        0.0, sy, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        -1.0, 1.0, 0.0, 1.0,
    ];
    let proj_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("proj"),
        contents: bytemuck::cast_slice(&proj),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let proj_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("proj_bg"),
        layout: &proj_bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: proj_buf.as_entire_binding(),
        }],
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("text"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../../src/gpu/shaders/text.wgsl").into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("text_pl"),
        bind_group_layouts: &[&proj_bgl, &atlas_bgl],
        immediate_size: 0,
    });
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                format,
                blend: Some(wgpu::BlendState {
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

    // --- the scene --------------------------------------------------------
    let mut text = TextSystem::new(&device, &atlas_bgl);
    // Fresh TextSystem per render, so there is nothing stale to invalidate.
    let _ = text.set_raster_scale(scale);

    // Each layer keeps the vertices from the last frame that resolved it —
    // a static layer keeps frame one's forever, exactly like a clean layer
    // in `resolve_layer_text`.
    let mut buffers: Vec<(Vec<TextVertex>, Vec<u32>)> =
        layers.iter().map(|_| (Vec::new(), Vec::new())).collect();

    for frame in 0..frames.max(1) {
        text.begin_frame();
        // Mirror `resolve_layer_text`: a disturbance recorded during the
        // previous frame (eviction, scale change) forces every layer this
        // frame, static ones included.
        let disturbed = text.take_atlas_disturbed();
        for (i, layer) in layers.iter().enumerate() {
            if frame > 0 && !layer.redraws_every_frame && !disturbed {
                continue;
            }
            let nodes: Vec<_> = layer
                .specimens
                .iter()
                .map(|s| SceneNode::Text {
                    key: TextNodeKey::from_style(&s.text, &s.style, None),
                    x: s.x,
                    y: s.y,
                    color: [1.0, 1.0, 1.0, 1.0],
                })
                .collect();
            buffers[i] = text.resolve_for_layer(&device, &queue, &atlas_bgl, &nodes);
        }
        text.finish_frame();
    }

    let gpu_buffers: Vec<_> = buffers
        .iter()
        .filter(|(v, i)| !v.is_empty() && !i.is_empty())
        .map(|(v, i)| {
            (
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("v"),
                    contents: bytemuck::cast_slice(v),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("i"),
                    contents: bytemuck::cast_slice(i),
                    usage: wgpu::BufferUsages::INDEX,
                }),
                i.len() as u32,
            )
        })
        .collect();

    // --- render + readback ------------------------------------------------
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("target"),
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
    let view = target.create_view(&Default::default());

    let bpr = (pw * 4).div_ceil(256) * 256;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: (bpr * ph) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut enc = device.create_command_encoder(&Default::default());
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("text_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.15,
                        g: 0.15,
                        b: 0.15,
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
        for (vbuf, ibuf, count) in &gpu_buffers {
            pass.set_vertex_buffer(0, vbuf.slice(..));
            pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..*count, 0, 0..1);
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
        let start = (row * bpr) as usize;
        rgba.extend_from_slice(&data[start..start + (pw * 4) as usize]);
    }
    drop(data);
    readback.unmap();

    Some(Rendered {
        width: pw,
        height: ph,
        rgba,
    })
}

/// Minimal PNG writer (stored/uncompressed deflate) — avoids pulling an
/// image encoder into the engine's dev-dependencies for a debug tool.
pub fn write_png(path: &str, w: u32, h: u32, rgba: &[u8]) {
    fn crc32(data: &[u8]) -> u32 {
        let mut table = [0u32; 256];
        for (i, e) in table.iter_mut().enumerate() {
            let mut c = i as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    0xEDB8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
            *e = c;
        }
        let mut c = 0xFFFF_FFFFu32;
        for &b in data {
            c = table[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
        }
        c ^ 0xFFFF_FFFF
    }
    fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], body: &[u8]) {
        out.extend_from_slice(&(body.len() as u32).to_be_bytes());
        let mut full = kind.to_vec();
        full.extend_from_slice(body);
        out.extend_from_slice(&full);
        out.extend_from_slice(&crc32(&full).to_be_bytes());
    }

    // Raw scanlines with filter byte 0.
    let mut raw = Vec::with_capacity(((w * 4 + 1) * h) as usize);
    for y in 0..h {
        raw.push(0u8);
        let s = (y * w * 4) as usize;
        raw.extend_from_slice(&rgba[s..s + (w * 4) as usize]);
    }
    // zlib stream, stored deflate blocks.
    let mut z = vec![0x78, 0x01];
    for (i, block) in raw.chunks(65535).enumerate() {
        let last = if (i + 1) * 65535 >= raw.len() { 1 } else { 0 };
        z.push(last);
        z.extend_from_slice(&(block.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(block.len() as u16)).to_le_bytes());
        z.extend_from_slice(block);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in &raw {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &z);
    chunk(&mut png, b"IEND", &[]);
    std::fs::write(path, png).unwrap();
}
