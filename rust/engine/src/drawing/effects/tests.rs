use super::*;

#[test]
fn gaussian_weights_sum_to_one() {
    for sigma in [0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0] {
        let w = gaussian_weights(sigma);
        let sum: f32 = w[0] + 2.0 * w[1..7].iter().sum::<f32>();
        assert!(
            (sum - 1.0).abs() < 1e-5,
            "sigma={sigma}: sum={sum} (expected 1.0)"
        );
    }
}

#[test]
fn gaussian_weights_zero_sigma() {
    let w = gaussian_weights(0.0);
    assert_eq!(w[0], 1.0);
    for &tail in &w[1..7] {
        assert_eq!(tail, 0.0);
    }
}

#[test]
fn gaussian_weights_symmetric_decay() {
    let w = gaussian_weights(3.0);
    assert!(w[0] > w[1]);
    for i in 1..6 {
        assert!(
            w[i] >= w[i + 1],
            "weight[{}]={} < weight[{}]={}",
            i,
            w[i],
            i + 1,
            w[i + 1]
        );
    }
}

#[test]
fn gaussian_weights_padding_is_zero() {
    let w = gaussian_weights(3.0);
    for (i, &pad) in w.iter().enumerate().take(16).skip(7) {
        assert_eq!(pad, 0.0, "padding index {i} should be zero");
    }
}

// -- the blur and the scale (task-77 phase 1b) ---------------------------------

#[test]
fn scaled_weights_at_scale_1_are_the_weights_of_scale_1() {
    for sigma in [0.5, 3.0, 4.0, 8.0, 50.0] {
        let (taps, w) = scaled_weights(sigma, 1.0);
        assert_eq!(taps, 6, "sigma {sigma}");
        assert_eq!(w[..7], gaussian_weights(sigma)[..7], "sigma {sigma}");
        assert!(w[7..].iter().all(|v| *v == 0.0), "sigma {sigma}");
    }
}

#[test]
fn scaled_weights_keep_the_cut_in_logical_px_and_sum_to_one() {
    // the cut of 1x: 6 logical px and a half either side of the center
    for (scale, taps) in [(0.5, 3), (1.25, 8), (1.5, 10), (2.0, 13), (3.0, 19)] {
        for sigma in [0.5, 3.0, 8.0, 50.0] {
            let (n, w) = scaled_weights(sigma, scale);
            assert_eq!(n, taps, "{scale}x, sigma {sigma}");
            let n = n as usize;
            let sum = w[0] + 2.0 * w[1..=n].iter().sum::<f32>();
            assert!(
                (sum - 1.0).abs() < 1e-5,
                "{scale}x, sigma {sigma}: the weights sum to {sum}"
            );
            assert!(w[n] > 0.0, "{scale}x, sigma {sigma}: the last tap is empty");
            assert!(w[n + 1..].iter().all(|v| *v == 0.0));
        }
    }
}

#[test]
fn scaled_weights_of_no_blur_are_one_tap() {
    for scale in [1.25, 2.0, 3.0] {
        let (taps, w) = scaled_weights(0.0, scale);
        assert_eq!(taps, 0);
        assert_eq!(w[0], 1.0);
        assert!(w[1..].iter().all(|v| *v == 0.0));
    }
}

#[test]
fn scaled_weights_stop_at_what_the_uniform_holds() {
    let (taps, w) = scaled_weights(8.0, 20.0);
    assert_eq!(taps as usize, SCALED_WEIGHTS - 1);
    assert!(w[SCALED_WEIGHTS - 1] > 0.0);
}

/// `apply_blur` (or `apply_shadow`, black) of sigma 4 over an opaque
/// white edge at x = 48 logical on transparent, in a 100 x 20 logical
/// texture at `scale`, drawn by the effect processor as a window's layer
/// effect is: the alpha of the middle row. `None` without a GPU.
#[cfg(not(target_arch = "wasm32"))]
fn layer_effect(shadow: bool, scale: f32) -> Option<Vec<u8>> {
    use crate::drawing::gpu::texture_pool::TexturePool;
    use crate::drawing::gpu::{GpuContext, HEADLESS_FORMAT, RenderConfig};

    let (w, h) = ((100.0 * scale) as u32, (20.0 * scale) as u32);
    let gpu = pollster::block_on(GpuContext::new_headless(w, h, RenderConfig::default()))?;
    let texture = |usage| {
        gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("layer_effect_test"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: HEADLESS_FORMAT,
            usage,
            view_formats: &[],
        })
    };
    let source = texture(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST);
    let edge = (48.0 * scale) as u32;
    let pixels: Vec<u8> = (0..w * h)
        .flat_map(|i| if i % w < edge { [255; 4] } else { [0; 4] })
        .collect();
    gpu.queue.write_texture(
        source.as_image_copy(),
        &pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(h),
        },
        source.size(),
    );
    let out = texture(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC);
    let (source_view, out_view) = (
        source.create_view(&Default::default()),
        gpu.texture_render_view(&out),
    );

    let effects = EffectProcessor::new(&gpu.device, HEADLESS_FORMAT);
    let mut pool = TexturePool::new();
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    let mut ctx = EffectContext {
        device: &gpu.device,
        queue: &gpu.queue,
        encoder: &mut encoder,
        pool: &mut pool,
        source_view: &source_view,
        width: w,
        height: h,
        scale: (scale, scale),
    };
    let result = if shadow {
        effects.apply_shadow(&mut ctx, 4.0, [0.0, 0.0, 0.0, 1.0])
    } else {
        effects.apply_blur(&mut ctx, 4.0)
    };
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("layer_effect_test"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &out_view,
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
        effects.composite_pass(&gpu.device, &gpu.queue, &mut pass, result.view(), 1.0);
    }
    gpu.queue.submit(std::iter::once(encoder.finish()));
    let frame = match crate::drawing::headless::read_texture(&gpu, &out) {
        Ok(f) => f,
        Err(e) => panic!("read back: {e}"),
    };
    let y = h / 2;
    Some(
        (0..w)
            .filter_map(|x| frame.pixel(x, y))
            .map(|p| p[3])
            .collect(),
    )
}

/// The 10%-90% ramp of the alpha row, in logical px.
#[cfg(not(target_arch = "wasm32"))]
fn alpha_reach(row: &[u8], scale: f32) -> f32 {
    row.iter().filter(|a| (26..=229).contains(*a)).count() as f32 / scale
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn the_layer_blur_and_shadow_reach_as_far_in_logical_px_at_every_scale() {
    for shadow in [false, true] {
        let name = if shadow { "shadow" } else { "blur" };
        let Some(one) = layer_effect(shadow, 1.0) else {
            eprintln!("no GPU adapter; skipping");
            return;
        };
        let one = alpha_reach(&one, 1.0);
        assert!(one > 5.0, "{name} at 1x spreads only {one} px");
        for scale in [1.25, 1.5, 2.0, 3.0] {
            let Some(row) = layer_effect(shadow, scale) else {
                return;
            };
            let reach = alpha_reach(&row, scale);
            assert!(
                (reach - one).abs() <= 1.0,
                "{name} at {scale}x spreads {reach} logical px, {one} at 1x"
            );
        }
    }
}
