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

/// A library computing on the renderer's device (`RenderConfig::compute`,
/// `GpuContext::instance` and `adapter`). Skips without an adapter.
#[cfg(not(target_arch = "wasm32"))]
mod compute {
    use crate::gpu::{GpuContext, RenderConfig};

    fn headless(compute: Option<wgpu::Features>) -> Option<GpuContext> {
        let config = RenderConfig {
            compute,
            ..RenderConfig::default()
        };
        let gpu = pollster::block_on(GpuContext::new_headless(8, 8, config));
        if gpu.is_none() {
            eprintln!("no GPU adapter; skipping");
        }
        gpu
    }

    /// The limits that decide how big a tensor's buffer can be.
    fn sizes(limits: &wgpu::Limits) -> (u64, u64) {
        (
            limits.max_buffer_size,
            limits.max_storage_buffer_binding_size,
        )
    }

    #[test]
    fn the_renderer_alone_asks_for_the_default_limits_and_no_feature() {
        let Some(gpu) = headless(None) else {
            return;
        };
        assert_eq!(sizes(&gpu.device.limits()), sizes(&wgpu::Limits::default()));
        assert_eq!(gpu.device.features(), wgpu::Features::empty());
        // the handles a compute library takes: the device's own adapter
        // (its features and limits hold the device's), and the instance
        // that answers with that adapter again
        assert!(gpu.adapter().features().contains(gpu.device.features()));
        let again = pollster::block_on(gpu.instance().request_adapter(&Default::default()))
            .expect("the instance still answers");
        assert_eq!(again.get_info().name, gpu.adapter().get_info().name);
    }

    /// Asked for f16 and f64: the device has the adapter's limits and
    /// exactly the asked features the adapter has; one it lacks is left
    /// out, not a failed device.
    #[test]
    fn compute_asks_for_the_adapter_limits_and_the_features_it_has() {
        let asked = wgpu::Features::SHADER_F16 | wgpu::Features::SHADER_F64;
        let Some(gpu) = headless(Some(asked)) else {
            return;
        };
        let adapter = gpu.adapter().features();
        assert_eq!(sizes(&gpu.device.limits()), sizes(&gpu.adapter().limits()));
        assert_eq!(gpu.device.features() & asked, asked & adapter);
        assert!(!gpu.device.features().intersects(asked - adapter));
    }

    /// Everything wgpu names: the device gets what the adapter has but
    /// the experimental features, which would fail the request.
    #[test]
    fn asking_for_every_feature_grants_the_adapter_s_but_the_experimental_ones() {
        let Some(gpu) = headless(Some(wgpu::Features::all())) else {
            return;
        };
        let experimental = wgpu::Features::all_experimental_mask();
        assert_eq!(
            gpu.device.features(),
            gpu.adapter().features() - experimental
        );
    }

    /// A recovery asks the new device for what the lost one had.
    #[test]
    fn a_recovery_keeps_what_compute_asked_for() {
        let Some(mut gpu) = headless(Some(wgpu::Features::SHADER_F16)) else {
            return;
        };
        let before = (sizes(&gpu.device.limits()), gpu.device.features());
        gpu.device.destroy();
        assert!(gpu.poll_device_loss());
        gpu.recover().expect("an adapter is there");
        assert_eq!((sizes(&gpu.device.limits()), gpu.device.features()), before);
        assert_eq!(sizes(&gpu.device.limits()), sizes(&gpu.adapter().limits()));
    }
}
