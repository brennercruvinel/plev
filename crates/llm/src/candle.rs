//! the candle implementation: `candle-transformers`' llama, which runs
//! TinyLlama from the Hugging Face safetensors as they are.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use candle_core::{D, DType, Device as CDevice, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::llama::{Cache, Config, Llama, LlamaConfig};

use crate::{Device, Info, LlmError, Model};

/// TinyLlama on candle, with its kv cache.
pub struct CandleModel {
    llama: Llama,
    fresh: Cache,
    cache: Cache,
    device: CDevice,
    context: usize,
    info: Info,
}

fn err(e: impl std::fmt::Display) -> LlmError {
    LlmError::Backend {
        backend: "candle",
        reason: e.to_string(),
    }
}

impl CandleModel {
    /// Loads `config.json` and `model.safetensors` from `dir`: f16 on the
    /// gpu, f32 on the cpu. Returns when the weights are on the device.
    pub fn load(dir: &Path, device: Device) -> Result<Self, LlmError> {
        let (device, dtype) = match device {
            Device::Cpu => (CDevice::Cpu, DType::F32),
            Device::Metal => (CDevice::new_metal(0).map_err(err)?, DType::F16),
        };
        let cfg = config(dir)?;
        let weights = dir.join("model.safetensors");
        if !weights.is_file() {
            return Err(LlmError::file(&weights, "not a file"));
        }
        // SAFETY: the file is mapped read-only and nothing in this process
        // writes to it; the model directory is not modified while a model is
        // loaded (it lives outside the repository, written once).
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[&weights], dtype, &device) }
            .map_err(err)?;
        // candle unwraps each block's load (llama.rs:524), so a file with a
        // missing tensor panics instead of erroring.
        let llama = catch_unwind(AssertUnwindSafe(|| Llama::load(vb, &cfg)))
            .map_err(|_| err("a tensor is missing or malformed in model.safetensors"))?
            .map_err(err)?;
        let fresh = Cache::new(true, dtype, &cfg, &device).map_err(err)?;
        device.synchronize().map_err(err)?;
        let info = Info {
            backend: "candle",
            device: format!("{:?}", device.location()).to_lowercase(),
            dtype: if dtype == DType::F16 { "f16" } else { "f32" },
        };
        Ok(Self {
            llama,
            cache: fresh.clone(),
            fresh,
            device,
            context: cfg.max_position_embeddings,
            info,
        })
    }
}

fn config(dir: &Path) -> Result<Config, LlmError> {
    let path = dir.join("config.json");
    let bytes = std::fs::read(&path).map_err(|e| LlmError::file(&path, e))?;
    let cfg: LlamaConfig =
        serde_json::from_slice(&bytes).map_err(|e| LlmError::Config(e.to_string()))?;
    Ok(cfg.into_config(false))
}

impl Model for CandleModel {
    fn reset(&mut self) -> Result<(), LlmError> {
        self.cache = self.fresh.clone();
        Ok(())
    }

    fn next(&mut self, ids: &[u32], pos: usize) -> Result<u32, LlmError> {
        let input = Tensor::new(ids, &self.device)
            .and_then(|t| t.unsqueeze(0))
            .map_err(err)?;
        let logits = self
            .llama
            .forward(&input, pos, &mut self.cache)
            .map_err(err)?;
        logits
            .squeeze(0)
            .and_then(|l| l.argmax(D::Minus1))
            .and_then(|id| id.to_scalar::<u32>())
            .map_err(err)
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
        let result = CandleModel::load(Path::new("/nonexistent/plev-llm"), Device::Cpu);
        assert!(matches!(result, Err(LlmError::File { .. })));
    }

    #[test]
    fn a_config_that_is_not_llama_is_a_config_error() {
        let dir = std::env::temp_dir().join(format!("plev-llm-candle-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.json"), br#"{"hidden_size": "wide"}"#).unwrap();
        let result = CandleModel::load(&dir, Device::Cpu);
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(matches!(result, Err(LlmError::Config(_))));
    }
}
