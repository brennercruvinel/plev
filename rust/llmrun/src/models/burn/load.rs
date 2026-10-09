//! the checkpoint into the burn model: `config.json` into a [`Spec`], and
//! `model.safetensors` through burn-store, with the rmsnorm weight renamed to
//! burn's `gamma` and every float cast to the backend's float on the way in
//! (burn-store keeps the file's bf16 otherwise).

use std::path::Path;
use std::rc::Rc;

use burn::nn::LinearLayout;
use burn::tensor::DType;
use burn::tensor::backend::Backend;
use burn_store::{
    ChainAdapter, ModuleAdapter, ModuleSnapshot, PyTorchToBurnAdapter, SafetensorsStore,
    TensorSnapshot,
};

use super::model::{Llama, Spec};
use crate::LlmError;

#[derive(serde::Deserialize)]
struct RawConfig {
    model_type: String,
    vocab_size: usize,
    hidden_size: usize,
    intermediate_size: usize,
    num_hidden_layers: usize,
    num_attention_heads: usize,
    num_key_value_heads: Option<usize>,
    rms_norm_eps: f64,
    rope_theta: Option<f32>,
    max_position_embeddings: usize,
    #[serde(default)]
    rope_scaling: Option<serde_json::Value>,
    #[serde(default)]
    tie_word_embeddings: bool,
}

fn config_err(reason: impl Into<String>) -> LlmError {
    LlmError::Config(reason.into())
}

/// Parses a llama `config.json`; refuses what this model code does not run.
pub fn spec(bytes: &[u8]) -> Result<Spec, LlmError> {
    let raw: RawConfig = serde_json::from_slice(bytes).map_err(|e| config_err(e.to_string()))?;
    if raw.model_type != "llama" {
        return Err(config_err(format!(
            "model_type {} is not llama",
            raw.model_type
        )));
    }
    if raw.rope_scaling.as_ref().is_some_and(|v| !v.is_null()) {
        return Err(config_err("rope_scaling is not supported"));
    }
    if raw.tie_word_embeddings {
        return Err(config_err("tied embeddings are not supported"));
    }
    let heads = raw.num_attention_heads;
    let kv_heads = raw.num_key_value_heads.unwrap_or(heads);
    if heads == 0
        || kv_heads == 0
        || !raw.hidden_size.is_multiple_of(heads)
        || !heads.is_multiple_of(kv_heads)
    {
        return Err(config_err(format!(
            "{heads} heads and {kv_heads} kv heads do not divide {}",
            raw.hidden_size
        )));
    }
    if !(raw.hidden_size / heads).is_multiple_of(2) {
        return Err(config_err("the head width must be even for rope"));
    }
    Ok(Spec {
        vocab: raw.vocab_size,
        hidden: raw.hidden_size,
        intermediate: raw.intermediate_size,
        layers: raw.num_hidden_layers,
        heads,
        kv_heads,
        eps: raw.rms_norm_eps,
        rope_theta: raw.rope_theta.unwrap_or(10_000.0),
        context: raw.max_position_embeddings,
    })
}

/// Casts every float tensor to one dtype as it is read.
#[derive(Clone)]
struct CastTo(DType);

impl ModuleAdapter for CastTo {
    fn adapt(&self, snapshot: &TensorSnapshot) -> TensorSnapshot {
        let target = self.0;
        if snapshot.dtype == target || !snapshot.dtype.is_float() {
            return snapshot.clone();
        }
        let data = snapshot.clone_data_fn();
        TensorSnapshot::from_closure(
            Rc::new(move || Ok(data()?.convert_dtype(target))),
            target,
            snapshot.shape.clone(),
            snapshot.path_stack.clone().unwrap_or_default(),
            snapshot.container_stack.clone().unwrap_or_default(),
            snapshot.tensor_id.unwrap_or_default(),
        )
    }

    fn clone_box(&self) -> Box<dyn ModuleAdapter> {
        Box::new(self.clone())
    }
}

/// Loads `model.safetensors` into `model`, casting to `dtype`; with
/// `LinearLayout::Row` the linear weights are transposed on the way in.
/// Every tensor of the model must come from the file.
pub fn weights<B: Backend>(
    model: &mut Llama<B>,
    path: &Path,
    dtype: DType,
    layout: LinearLayout,
) -> Result<(), LlmError> {
    if !path.is_file() {
        return Err(LlmError::file(path, "not a file"));
    }
    let store = SafetensorsStore::from_file(path)
        .with_key_remapping(r"(.*norm)\.weight$", "$1.gamma")
        .allow_partial(false);
    let mut store = match layout {
        LinearLayout::Row => {
            store.with_from_adapter(ChainAdapter::new(PyTorchToBurnAdapter, CastTo(dtype)))
        }
        LinearLayout::Col => store.with_from_adapter(CastTo(dtype)),
    };
    let result = model.load_from(&mut store).map_err(|e| LlmError::Backend {
        backend: "burn",
        reason: e.to_string(),
    })?;
    if !result.missing.is_empty() || !result.errors.is_empty() {
        let missing: Vec<&str> = result.missing.iter().map(|(p, _)| p.as_str()).collect();
        return Err(LlmError::Backend {
            backend: "burn",
            reason: format!(
                "incomplete checkpoint: missing {missing:?}, errors {:?}",
                result.errors
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TINY: &str = r#"{
        "model_type": "llama", "vocab_size": 32000, "hidden_size": 2048,
        "intermediate_size": 5632, "num_hidden_layers": 22,
        "num_attention_heads": 32, "num_key_value_heads": 4,
        "rms_norm_eps": 1e-05, "rope_theta": 10000.0,
        "max_position_embeddings": 2048, "rope_scaling": null,
        "tie_word_embeddings": false, "torch_dtype": "bfloat16"
    }"#;

    #[test]
    fn tinyllama_config_reads() {
        let spec = spec(TINY.as_bytes()).unwrap();
        assert_eq!(spec.head_dim(), 64);
        assert_eq!((spec.heads, spec.kv_heads, spec.layers), (32, 4, 22));
        assert_eq!(spec.context, 2048);
    }

    #[test]
    fn what_the_model_code_cannot_run_is_refused() {
        let scaled = TINY.replace(
            r#""rope_scaling": null"#,
            r#""rope_scaling": {"factor": 8}"#,
        );
        assert!(matches!(spec(scaled.as_bytes()), Err(LlmError::Config(_))));
        let tied = TINY.replace(
            r#""tie_word_embeddings": false"#,
            r#""tie_word_embeddings": true"#,
        );
        assert!(matches!(spec(tied.as_bytes()), Err(LlmError::Config(_))));
        let odd = TINY.replace(r#""num_key_value_heads": 4"#, r#""num_key_value_heads": 5"#);
        assert!(matches!(spec(odd.as_bytes()), Err(LlmError::Config(_))));
        let other = TINY.replace(r#""llama""#, r#""gpt2""#);
        assert!(matches!(spec(other.as_bytes()), Err(LlmError::Config(_))));
        assert!(matches!(spec(b"[]"), Err(LlmError::Config(_))));
    }
}
