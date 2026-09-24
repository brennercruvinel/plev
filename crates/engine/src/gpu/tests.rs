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

    /// `recover` replaces the device in place: the flag is down, the
    /// reason is returned, the logical projection survives, and the new
    /// device does work (a buffer maps).
    #[test]
    fn a_lost_device_is_replaced_in_place() {
        let Some(mut gpu) = headless() else {
            return;
        };
        gpu.set_projection(4.0, 4.0);
        gpu.device.destroy();
        assert!(gpu.poll_device_loss());

        let lost = gpu.recover().expect("an adapter is there");
        assert_eq!(lost.reason, wgpu::DeviceLostReason::Destroyed);
        assert!(!gpu.device_lost());
        assert_eq!(gpu.lost_device(), None);
        assert_eq!(gpu.clip_scale(), (2.0, 2.0));

        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("alive"),
            size: 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&buffer, 0, &[1, 2, 3, 4]);
        // a staged write reaches the buffer with the next submit
        gpu.queue.submit(std::iter::empty());
        let (tx, rx) = std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        gpu.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("the new device polls");
        rx.recv().expect("mapped").expect("mapped ok");
        assert_eq!(&buffer.slice(..).get_mapped_range()[..], &[1, 2, 3, 4]);
    }

    /// No adapter answers: the error names it, the context keeps its dead
    /// device and the flag, and a later attempt with an adapter succeeds.
    #[test]
    fn a_recovery_without_an_adapter_is_an_error_and_the_context_stays_lost() {
        use crate::gpu::RecoverError;
        let Some(mut gpu) = headless() else {
            return;
        };
        gpu.set_recovery_backends(wgpu::Backends::empty());
        gpu.device.destroy();
        assert!(gpu.poll_device_loss());

        assert!(matches!(gpu.recover(), Err(RecoverError::NoAdapter(_))));
        assert!(gpu.device_lost());
        assert!(gpu.lost_device().is_some());

        gpu.set_recovery_backends(wgpu::Backends::PRIMARY);
        assert!(gpu.recover().is_ok());
        assert!(!gpu.device_lost());
    }
}
