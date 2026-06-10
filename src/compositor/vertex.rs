#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl QuadVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectSdfVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub rect_params: [f32; 4],
    pub border_color: [f32; 4],
    /// Second gradient stop. Equal to `color` for solid fills.
    pub color2: [f32; 4],
    /// Linear-gradient brush: (dir_x, dir_y, enabled, unused). The direction
    /// is a unit vector in screen space (y down); enabled <= 0 means solid.
    pub gradient: [f32; 4],
}

impl RectSdfVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RectSdfVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 64,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 80,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowVertex {
    pub position: [f32; 2],
    /// Pixel offset from the shadow rect center. Drop shadows cover the
    /// padded quad around the OFFSET rect; inset shadows cover exactly the
    /// casting rect (the shadow is clipped inside it).
    pub local: [f32; 2],
    pub color: [f32; 4],
    /// half_w, half_h, corner_radius, sigma.
    pub params: [f32; 4],
    /// inset (>0.5 = inset mode), offset_x, offset_y, unused. Drop shadows
    /// bake the offset into the quad position and leave this zeroed; inset
    /// shadows evaluate the blurred mask at `local - offset` in-shader so
    /// the hard clip to the rect stays put.
    pub params2: [f32; 4],
}

impl ShadowVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ShadowVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageVertex {
    pub position: [f32; 2],
    /// Sample position in atlas pixels (normalized in the shader against
    /// textureDimensions, so it survives atlas growth).
    pub atlas_px: [f32; 2],
    /// Pixel offset from the rect center (rounded-corner SDF mask).
    pub local: [f32; 2],
    /// half_w, half_h, corner_radius, unused.
    pub params: [f32; 4],
    /// Sampling clamp rect in atlas pixels: min_x, min_y, max_x, max_y.
    pub uv_bounds: [f32; 4],
}

impl ImageVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ImageVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 40,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Gaussian sigma for a CSS-like blur radius (CSS box-shadow convention).
pub fn shadow_sigma(blur_radius: f32) -> f32 {
    blur_radius / 2.0
}

/// How far the shadow quad extends past the casting rect on each side:
/// 3 sigma covers >99.7% of the Gaussian.
pub fn shadow_padding(blur_radius: f32) -> f32 {
    3.0 * shadow_sigma(blur_radius)
}

/// Direction unit vector for a CSS-style gradient angle in degrees:
/// 0 points up (first stop at the bottom), 90 points right, measured
/// clockwise. Screen space has y down, hence the negated cosine.
pub fn gradient_direction(angle_deg: f32) -> [f32; 2] {
    let rad = angle_deg.to_radians();
    [rad.sin(), -rad.cos()]
}
