//! burn on the engine's device, without a window: the engine makes its
//! device by the headless path, burn takes it through cubecl's
//! `init_device`, a matmul runs on it and reads back equal to the cpu's,
//! and the same device then draws a frame, byte for byte the frame of a
//! renderer burn never touched. skips without a GPU adapter, like the
//! engine's own GPU tests.

#![cfg(feature = "plev")]

use burn::tensor::{Tensor, TensorData};
use engine::compositor::Compositor;
use engine::gpu::RenderConfig;
use engine::headless::{Frame, HeadlessError, Renderer, Viewport};
use llm::LlmError;
use llm::plev::{self, Shared};

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const CLEAR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// The engine's headless renderer, its device asked for burn.
fn renderer() -> Option<Renderer> {
    let config = RenderConfig {
        compute: Some(plev::features()),
        ..RenderConfig::default()
    };
    match Renderer::new(Viewport::new(64.0, 48.0, 2.0), config) {
        Ok(r) => Some(r),
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter; skipping");
            None
        }
        Err(e) => panic!("headless renderer: {e}"),
    }
}

fn frame(renderer: &mut Renderer) -> Frame {
    let mut c = Compositor::new();
    c.begin_frame();
    c.draw_rect(8.0, 8.0, 24.0, 16.0, RED);
    renderer
        .render(&mut c, CLEAR)
        .expect("the frame reads back")
}

/// `a` (m x k) times `b` (k x n), row major, in f64 on the cpu.
fn cpu_matmul(a: &[f32], b: &[f32], m: usize, k: usize, n: usize) -> Vec<f32> {
    let mut out = vec![0.0; m * n];
    for i in 0..m {
        for j in 0..n {
            let dot: f64 = (0..k)
                .map(|p| f64::from(a[i * k + p]) * f64::from(b[p * n + j]))
                .sum();
            out[i * n + j] = dot as f32;
        }
    }
    out
}

#[test]
fn a_matmul_on_the_engine_s_device_matches_the_cpu_and_the_device_still_draws() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    let device = plev::device(renderer.gpu()).expect("burn takes the engine's device");

    let (m, k, n) = (7, 33, 5);
    let a: Vec<f32> = (0..m * k)
        .map(|i| ((i * 37 % 19) as f32 - 9.0) / 8.0)
        .collect();
    let b: Vec<f32> = (0..k * n)
        .map(|i| ((i * 11 % 23) as f32 - 11.0) / 16.0)
        .collect();
    let ta = Tensor::<Shared, 2>::from_data(TensorData::new(a.clone(), [m, k]), &device);
    let tb = Tensor::<Shared, 2>::from_data(TensorData::new(b.clone(), [k, n]), &device);
    let gpu: Vec<f32> = ta.matmul(tb).into_data().to_vec().expect("f32 back");

    let cpu = cpu_matmul(&a, &b, m, k, n);
    assert_eq!(gpu.len(), cpu.len());
    for (i, (g, c)) in gpu.iter().zip(&cpu).enumerate() {
        assert!(
            (g - c).abs() <= 1e-5 * c.abs().max(1.0),
            "[{i}] gpu {g} cpu {c}"
        );
    }

    // the device burn computed on draws the frame, and it is the frame of a
    // renderer whose device burn never touched
    let shared = frame(&mut renderer);
    let mut alone = Renderer::new(Viewport::new(64.0, 48.0, 2.0), RenderConfig::default())
        .expect("a second renderer on the same machine");
    let plain = frame(&mut alone);
    assert_eq!(shared.pixel(40, 30), Some([255, 0, 0, 255]));
    assert_eq!(shared.pixel(100, 80), Some([0, 0, 0, 255]));
    assert!(
        shared.rgba == plain.rgba,
        "sharing the device moved a pixel"
    );
}

/// A model directory that is not there is a file error on the shared
/// device too, before any weight reaches it.
#[test]
fn a_missing_model_directory_is_a_file_error() {
    let Some(renderer) = renderer() else {
        return;
    };
    let result = plev::load(
        std::path::Path::new("/nonexistent/plev-llm"),
        renderer.gpu(),
    );
    assert!(matches!(result, Err(LlmError::File { .. })));
}

/// The ask covers the kernels cubecl may choose: a device granted by it
/// has every non-experimental feature of its adapter but mappable primary
/// buffers, and the adapter's limits.
#[test]
fn the_shared_device_has_what_cubecl_reads_off_the_adapter() {
    let Some(renderer) = renderer() else {
        return;
    };
    let gpu = renderer.gpu();
    let wanted = gpu.adapter().features()
        - engine::wgpu::Features::all_experimental_mask()
        - engine::wgpu::Features::MAPPABLE_PRIMARY_BUFFERS;
    assert_eq!(gpu.device.features(), wanted);
    assert_eq!(
        gpu.device.limits().max_buffer_size,
        gpu.adapter().limits().max_buffer_size
    );
}
