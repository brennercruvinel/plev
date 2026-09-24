//! CPU-side WGSL validation: parse + validate every engine shader with naga
//! so a broken shader fails `cargo test` instead of panicking at runtime.

use wgpu::naga;

fn validate_wgsl(name: &str, source: &str) {
    let module = naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|e| panic!("{name}: WGSL parse error: {e}"));
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .unwrap_or_else(|e| panic!("{name}: WGSL validation error: {e:?}"));
}

#[test]
fn all_engine_shaders_are_valid_wgsl() {
    let shaders = [
        ("quad.wgsl", include_str!("shaders/quad.wgsl")),
        ("rect_sdf.wgsl", include_str!("shaders/rect_sdf.wgsl")),
        (
            "shadow_analytic.wgsl",
            include_str!("shaders/shadow_analytic.wgsl"),
        ),
        ("image.wgsl", include_str!("shaders/image.wgsl")),
        ("backdrop.wgsl", include_str!("shaders/backdrop.wgsl")),
        ("text.wgsl", include_str!("shaders/text.wgsl")),
        ("composite.wgsl", include_str!("shaders/composite.wgsl")),
        ("blur.wgsl", include_str!("shaders/blur.wgsl")),
        ("shadow.wgsl", include_str!("shaders/shadow.wgsl")),
    ];
    for (name, source) in shaders {
        validate_wgsl(name, source);
    }
}

/// The lost-device flag (`gpu::loss`). Needs a device; skips without an
/// adapter like the other GPU tests.
#[cfg(not(target_arch = "wasm32"))]
mod loss {
    use crate::gpu::{GpuContext, RenderConfig};

    fn headless() -> Option<GpuContext> {
        let gpu = pollster::block_on(GpuContext::new_headless(8, 8, RenderConfig::default()));
        if gpu.is_none() {
            eprintln!("no GPU adapter; skipping");
        }
        gpu
    }

    #[test]
    fn a_live_device_is_not_lost() {
        let Some(gpu) = headless() else {
            return;
        };
        assert!(!gpu.device_lost());
        assert!(!gpu.poll_device_loss());
        assert_eq!(gpu.lost_device(), None);
    }

    /// `destroy` alone tells nobody (the probe): the next poll delivers
    /// the callback, and the flag carries wgpu's reason.
    #[test]
    fn a_destroyed_device_is_lost_once_polled_with_its_reason() {
        let Some(gpu) = headless() else {
            return;
        };
        gpu.device.destroy();
        assert!(gpu.poll_device_loss());
        assert!(gpu.device_lost());
        let lost = gpu.lost_device().expect("the reason is kept");
        assert_eq!(lost.reason, wgpu::DeviceLostReason::Destroyed);
        assert_eq!(lost.message, "");
    }

    #[test]
    fn a_destroy_the_engine_asked_for_is_not_a_loss() {
        let Some(gpu) = headless() else {
            return;
        };
        gpu.loss.expect_destroy();
        gpu.device.destroy();
        assert!(!gpu.poll_device_loss());
        assert_eq!(gpu.lost_device(), None);
    }
}
