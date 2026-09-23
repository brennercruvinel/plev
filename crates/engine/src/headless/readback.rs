//! GPU texture to CPU pixels, and pixels to PNG.

use std::path::Path;

use image::ImageEncoder;

use super::HeadlessError;
use crate::gpu::GpuContext;

/// Bytes per texel of every format the readback accepts.
const TEXEL: u32 = 4;

/// A frame read back from the GPU: RGBA8, sRGB-encoded (the values a
/// window shows), row-major, `width * height * 4` bytes, no row padding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Frame {
    /// The RGBA of one physical pixel, `None` outside the frame.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = ((y as usize * self.width as usize) + x as usize) * TEXEL as usize;
        let p = self.rgba.get(i..i + TEXEL as usize)?;
        Some([p[0], p[1], p[2], p[3]])
    }

    /// The frame as PNG bytes (8-bit RGBA, no color profile chunk: the
    /// values are sRGB, which is what an untagged PNG means).
    pub fn encode_png(&self) -> Result<Vec<u8>, HeadlessError> {
        // the encoder asserts the length; a hand-built frame must get an
        // error, not a panic
        let expected = self.width as usize * self.height as usize * TEXEL as usize;
        if self.rgba.len() != expected {
            return Err(HeadlessError::Png(image::ImageError::Parameter(
                image::error::ParameterError::from_kind(
                    image::error::ParameterErrorKind::DimensionMismatch,
                ),
            )));
        }
        let mut out = Vec::new();
        image::codecs::png::PngEncoder::new(&mut out)
            .write_image(
                &self.rgba,
                self.width,
                self.height,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(HeadlessError::Png)?;
        Ok(out)
    }

    pub fn write_png(&self, path: impl AsRef<Path>) -> Result<(), HeadlessError> {
        std::fs::write(path, self.encode_png()?)?;
        Ok(())
    }
}

/// Copy `texture` back to the CPU as RGBA8.
///
/// The texture needs `COPY_SRC` usage and an 8-bit four-channel format
/// (RGBA or BGRA, sRGB or not; BGRA is swizzled so the result is always
/// RGBA). The copy goes through a buffer whose rows are padded to
/// `COPY_BYTES_PER_ROW_ALIGNMENT` (256 bytes); the padding is stripped.
/// Blocks until the GPU has finished every submitted command.
pub fn read_texture(gpu: &GpuContext, texture: &wgpu::Texture) -> Result<Frame, HeadlessError> {
    let format = texture.format();
    let bgra = match format {
        wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => false,
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => true,
        other => return Err(HeadlessError::UnsupportedFormat(other)),
    };
    let (width, height) = (texture.width(), texture.height());
    let row = width * TEXEL;
    let padded_row =
        row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("headless_readback"),
        size: u64::from(padded_row) * u64::from(height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("headless_readback"),
        });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit(std::iter::once(encoder.finish()));

    let slice = buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        // The receiver outlives the poll below; a failed send only means
        // the caller already gave up.
        let _ = tx.send(result);
    });
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|e| HeadlessError::Readback(e.to_string()))?;
    rx.recv()
        .map_err(|e| HeadlessError::Readback(e.to_string()))?
        .map_err(|e| HeadlessError::Readback(e.to_string()))?;

    let mut rgba = Vec::with_capacity(row as usize * height as usize);
    {
        let data = slice.get_mapped_range();
        for line in data.chunks_exact(padded_row as usize) {
            rgba.extend_from_slice(&line[..row as usize]);
        }
    }
    buffer.unmap();

    if bgra {
        for texel in rgba.chunks_exact_mut(TEXEL as usize) {
            texel.swap(0, 2);
        }
    }
    Ok(Frame {
        width,
        height,
        rgba,
    })
}
