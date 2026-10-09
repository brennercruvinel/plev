//! the burn implementation. burn 0.21 has no llama of its own; the model code
//! is adapted from burn-lm's `burn-lm-llama` (see crates/llm/NOTICE) and runs
//! on ndarray (cpu, f32) or on wgpu with the msl compiler (metal, f16).

mod cache;
mod load;
mod model;
mod rotary;

use std::path::Path;

use burn::tensor::backend::Backend;
use burn::tensor::{Element, ElementConversion, Int, Tensor, TensorData};

use crate::{Device, Info, LlmError, Model};
use cache::KvCache;
use model::Llama;
use rotary::Rope;

pub use burn::nn::LinearLayout;
pub use model::Spec;

/// burn on the cpu.
pub type Cpu = burn::backend::NdArray<f32>;

/// burn on the apple gpu: wgpu with the msl compiler, fusion and autotune.
#[cfg(feature = "metal")]
pub type Gpu = burn::backend::wgpu::Metal<burn::tensor::f16>;

/// TinyLlama on burn, with its kv cache and rope tables.
pub struct BurnModel<B: Backend> {
    llama: Llama<B>,
    cache: Vec<KvCache<B>>,
    rope: Rope<B>,
    device: B::Device,
    context: usize,
    info: Info,
}

fn err(e: impl std::fmt::Display) -> LlmError {
    LlmError::Backend {
        backend: "burn",
        reason: e.to_string(),
    }
}

/// The default linear layout per device, as measured on TinyLlama (the
/// task-78 file): on the cpu the transposed view of `Col` makes ndarray's
/// matmul six times slower than weights transposed once at load (0.3
/// against 1.8 tok/s); on metal `Col` decodes faster (26.3 against 24.4
/// tok/s) and loads without copying the weights.
pub fn layout_for(device: Device) -> LinearLayout {
    match device {
        Device::Cpu => LinearLayout::Row,
        Device::Metal => LinearLayout::Col,
    }
}

/// Loads the model on the device the caller asks for, in the layout that
/// device runs fastest.
pub fn load(dir: &Path, device: Device) -> Result<Box<dyn Model>, LlmError> {
    load_with(dir, device, layout_for(device))
}

/// Loads the model with an explicit linear layout: `Col` keeps the
/// checkpoint's [out, in] and transposes the view, `Row` transposes the bytes
/// at load into burn's [in, out].
pub fn load_with(
    dir: &Path,
    device: Device,
    layout: LinearLayout,
) -> Result<Box<dyn Model>, LlmError> {
    match device {
        Device::Cpu => Ok(Box::new(BurnModel::<Cpu>::load_on(
            dir,
            Default::default(),
            layout,
            "cpu",
        )?)),
        #[cfg(feature = "metal")]
        Device::Metal => Ok(Box::new(BurnModel::<Gpu>::load_on(
            dir,
            Default::default(),
            layout,
            "metal",
        )?)),
        #[cfg(not(feature = "metal"))]
        Device::Metal => Err(err("built without the metal feature")),
    }
}

impl<B: Backend> BurnModel<B> {
    /// Reads `config.json` and `model.safetensors` from `dir` onto `device`,
    /// in the backend's float and the given linear layout. Returns when the
    /// weights are on the device.
    pub fn load_on(
        dir: &Path,
        device: B::Device,
        layout: LinearLayout,
        name: &str,
    ) -> Result<Self, LlmError> {
        let path = dir.join("config.json");
        let bytes = std::fs::read(&path).map_err(|e| LlmError::file(&path, e))?;
        let spec = load::spec(&bytes)?;
        let dtype = <B::FloatElem as Element>::dtype();
        let mut llama = Llama::init(&spec, layout, &device);
        load::weights(&mut llama, &dir.join("model.safetensors"), dtype, layout)?;
        let shape = [1, spec.kv_heads, spec.context, spec.head_dim()];
        let cache = (0..spec.layers)
            .map(|_| KvCache::new(shape, &device))
            .collect();
        let rope = Rope::new(spec.head_dim(), spec.context, spec.rope_theta, &device);
        B::sync(&device).map_err(|e| err(format!("{e:?}")))?;
        let info = Info {
            backend: "burn",
            device: name.to_string(),
            dtype: if dtype.size() == 2 { "f16" } else { "f32" },
        };
        Ok(Self {
            llama,
            cache,
            rope,
            device,
            context: spec.context,
            info,
        })
    }
}

impl<B: Backend> Model for BurnModel<B> {
    fn reset(&mut self) -> Result<(), LlmError> {
        self.cache.iter_mut().for_each(KvCache::reset);
        Ok(())
    }

    fn next(&mut self, ids: &[u32], pos: usize) -> Result<u32, LlmError> {
        let len = ids.len();
        let data = TensorData::new(ids.iter().map(|&i| i as i64).collect(), [1, len]);
        let input = Tensor::<B, 2, Int>::from_data(data, &self.device);
        let logits = self.llama.forward(input, &mut self.cache, &self.rope, pos);
        let id: i64 = logits.argmax(1).into_scalar().elem();
        u32::try_from(id).map_err(|_| err(format!("argmax returned {id}")))
    }

    fn context(&self) -> usize {
        self.context
    }

    fn info(&self) -> Info {
        self.info.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_directory_is_a_file_error() {
        let result = load(Path::new("/nonexistent/plev-llm"), Device::Cpu);
        assert!(matches!(result, Err(LlmError::File { .. })));
    }

    /// A two-layer model with random weights: the forward runs, the cache
    /// grows by the fed positions, and prefill then one step agrees with
    /// feeding the whole sequence at once.
    #[test]
    fn prefill_then_step_matches_one_pass() {
        let device = Default::default();
        let spec = Spec {
            vocab: 16,
            hidden: 8,
            intermediate: 12,
            layers: 2,
            heads: 4,
            kv_heads: 2,
            eps: 1e-5,
            rope_theta: 10_000.0,
            context: 8,
        };
        let llama = Llama::<Cpu>::init(&spec, LinearLayout::Row, &device);
        let rope = Rope::new(spec.head_dim(), spec.context, spec.rope_theta, &device);
        let fresh = || -> Vec<KvCache<Cpu>> {
            (0..spec.layers)
                .map(|_| KvCache::new([1, 2, 8, 2], &device))
                .collect()
        };
        let ids = |v: &[i64]| {
            Tensor::<Cpu, 2, Int>::from_data(TensorData::new(v.to_vec(), [1, v.len()]), &device)
        };

        let mut cache = fresh();
        let whole = llama.forward(ids(&[3, 1, 4, 1]), &mut cache, &rope, 0);
        assert_eq!(cache[0].len(), 4);

        let mut cache = fresh();
        llama.forward(ids(&[3, 1, 4]), &mut cache, &rope, 0);
        let step = llama.forward(ids(&[1]), &mut cache, &rope, 3);
        assert_eq!(cache[1].len(), 4);

        let a: Vec<f32> = whole.into_data().to_vec().unwrap();
        let b: Vec<f32> = step.into_data().to_vec().unwrap();
        assert_eq!(a.len(), spec.vocab);
        for (x, y) in a.iter().zip(&b) {
            assert!((x - y).abs() < 1e-4, "{a:?} vs {b:?}");
        }
    }
}
