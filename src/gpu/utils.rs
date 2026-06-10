/// Premultiplied alpha blend state: src already has color * alpha,
/// so src_factor is One (not SrcAlpha).
pub(crate) fn premultiplied_blend() -> wgpu::BlendState {
    wgpu::BlendState {
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
    }
}

/// Maps [0,w] -> [-1,1] X, [0,h] -> [1,-1] Y (Y-down), Z: [0,1]
pub(crate) fn ortho_projection(width: f32, height: f32) -> [f32; 16] {
    let sx = 2.0 / width;
    let sy = -2.0 / height;
    [
        sx, 0.0, 0.0, 0.0, //
        0.0, sy, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        -1.0, 1.0, 0.0, 1.0, //
    ]
}
