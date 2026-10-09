//! burn on the engine's device (feature `plev`): the renderer and the model
//! share one wgpu instance, adapter, device and queue. the engine hands its
//! handles out (`GpuContext::instance`, `adapter`, `device`, `queue`) and
//! cubecl takes them through `init_device`, so a tensor lives in the same
//! device memory a frame is drawn from, and both submit to the same queue.
//!
//! the device has to be asked for with [`features`] in the engine's
//! `RenderConfig::compute`: cubecl chooses its kernels by the adapter's
//! features (f16, i64, subgroups) and sizes its buffers by the device's
//! limits, so a device with the renderer's defaults would refuse them.
//!
//! burn runs its wgsl path here. with the `metal` feature on, cubecl compiles
//! to msl for a metal adapter through passthrough shaders, an experimental
//! wgpu feature the engine's device never asks for; [`device`] refuses that
//! case instead of failing at the first kernel.

use std::path::Path;

use burn::backend::wgpu::{RuntimeOptions, WgpuDevice, WgpuSetup, init_device};
use burn::nn::LinearLayout;
use engine::gpu::GpuContext;
use engine::wgpu;

use crate::burn::BurnModel;
use crate::{LlmError, Model};

/// burn on a wgpu device through cubecl's wgsl compiler.
pub type Shared<F = f32> = burn::backend::Wgpu<F>;

/// What the engine's `RenderConfig::compute` asks of the device for burn:
/// every feature the adapter has (the engine never grants an experimental
/// one) but mappable primary buffers, as cubecl asks of a device of its own.
pub fn features() -> wgpu::Features {
    wgpu::Features::all() - wgpu::Features::MAPPABLE_PRIMARY_BUFFERS
}

/// The engine context's device as a burn device. Each call registers a new
/// device with cubecl (its note on `init_device`): call it once per context,
/// and again after a recovery, which replaces the context's device.
pub fn device(gpu: &GpuContext) -> Result<WgpuDevice, LlmError> {
    let backend = gpu.adapter().get_info().backend;
    if cfg!(feature = "metal") && backend == wgpu::Backend::Metal {
        return Err(LlmError::Backend {
            backend: "burn",
            reason: "the metal feature compiles burn to msl through passthrough shaders, \
                     which the engine's device does not enable"
                .to_string(),
        });
    }
    let setup = WgpuSetup {
        instance: gpu.instance().clone(),
        adapter: gpu.adapter().clone(),
        device: gpu.device.clone(),
        queue: gpu.queue.clone(),
        backend,
    };
    Ok(init_device(setup, RuntimeOptions::default()))
}

/// Loads the model from `dir` on the engine context's device: f16 when the
/// device has `SHADER_F16`, f32 otherwise, the weights kept [out, in] with
/// the transposed view (the layout burn decodes fastest on the gpu).
pub fn load(dir: &Path, gpu: &GpuContext) -> Result<Box<dyn Model>, LlmError> {
    let device = device(gpu)?;
    if gpu.device.features().contains(wgpu::Features::SHADER_F16) {
        let model = BurnModel::<Shared<burn::tensor::f16>>::load_on(
            dir,
            device,
            LinearLayout::Col,
            "plev",
        )?;
        Ok(Box::new(model))
    } else {
        let model = BurnModel::<Shared>::load_on(dir, device, LinearLayout::Col, "plev")?;
        Ok(Box::new(model))
    }
}
