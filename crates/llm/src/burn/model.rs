// adapted from burn-lm (https://github.com/tracel-ai/burn-lm), commit a53baab,
// crates/burn-lm-llama/src/nn/{transformer.rs, attention/mha.rs, fftn.rs},
// MIT OR Apache-2.0; see crates/llm/NOTICE. changes: the backend parameter of
// burn 0.21 back on every type, the fields named as in the Hugging Face
// checkpoint (so the safetensors load without remapping), the linear weights in
// the checkpoint's [out, in] layout, the attention in f32 and the output head
// on the last position only (as candle does), the training paths dropped.

use burn::module::Module;
use burn::nn::{
    Embedding, EmbeddingConfig, Linear, LinearConfig, LinearLayout, RmsNorm, RmsNormConfig,
};
use burn::tensor::activation::{silu, softmax};
use burn::tensor::backend::Backend;
use burn::tensor::{Bool, DType, Int, Tensor};

use super::cache::KvCache;
use super::rope::Rope;

/// The shape of a llama checkpoint, read from its `config.json`.
#[derive(Clone, Debug, PartialEq)]
pub struct Spec {
    /// Vocabulary size.
    pub vocab: usize,
    /// Model width.
    pub hidden: usize,
    /// Feed-forward width.
    pub intermediate: usize,
    /// Transformer blocks.
    pub layers: usize,
    /// Query heads.
    pub heads: usize,
    /// Key and value heads (grouped query attention).
    pub kv_heads: usize,
    /// RMSNorm epsilon.
    pub eps: f64,
    /// RoPE base.
    pub rope_theta: f32,
    /// Longest sequence.
    pub context: usize,
}

impl Spec {
    /// Width of one head.
    pub fn head_dim(&self) -> usize {
        self.hidden / self.heads
    }
}

/// A linear layer without bias. With `LinearLayout::Col` the weight keeps
/// the checkpoint's [out, in] and burn transposes the view on load; with
/// `Row` the loader transposes the bytes to burn's [in, out].
fn linear<B: Backend>(
    d_in: usize,
    d_out: usize,
    layout: LinearLayout,
    device: &B::Device,
) -> Linear<B> {
    LinearConfig::new(d_in, d_out)
        .with_bias(false)
        .with_layout(layout)
        .init(device)
}

fn rms<B: Backend>(spec: &Spec, device: &B::Device) -> RmsNorm<B> {
    RmsNormConfig::new(spec.hidden)
        .with_epsilon(spec.eps)
        .init(device)
}

/// Llama decoder-only transformer, the checkpoint's `model` and `lm_head`.
#[derive(Module, Debug)]
pub struct Llama<B: Backend> {
    model: Body<B>,
    lm_head: Linear<B>,
}

#[derive(Module, Debug)]
struct Body<B: Backend> {
    embed_tokens: Embedding<B>,
    layers: Vec<Layer<B>>,
    norm: RmsNorm<B>,
}

#[derive(Module, Debug)]
struct Layer<B: Backend> {
    self_attn: Attention<B>,
    mlp: Mlp<B>,
    input_layernorm: RmsNorm<B>,
    post_attention_layernorm: RmsNorm<B>,
}

#[derive(Module, Debug)]
struct Attention<B: Backend> {
    q_proj: Linear<B>,
    k_proj: Linear<B>,
    v_proj: Linear<B>,
    o_proj: Linear<B>,
    heads: usize,
    kv_heads: usize,
    head_dim: usize,
}

/// The SwiGLU feed-forward: `down(silu(gate(x)) * up(x))`.
#[derive(Module, Debug)]
struct Mlp<B: Backend> {
    gate_proj: Linear<B>,
    up_proj: Linear<B>,
    down_proj: Linear<B>,
}

impl<B: Backend> Llama<B> {
    /// The model with lazy, uninitialized weights, ready to be loaded.
    pub fn init(spec: &Spec, layout: LinearLayout, device: &B::Device) -> Self {
        let layers = (0..spec.layers)
            .map(|_| Layer::init(spec, layout, device))
            .collect();
        Self {
            model: Body {
                embed_tokens: EmbeddingConfig::new(spec.vocab, spec.hidden).init(device),
                layers,
                norm: rms(spec, device),
            },
            lm_head: linear(spec.hidden, spec.vocab, layout, device),
        }
    }

    /// Runs `ids` ([batch, seq]) at positions `pos..pos + seq` and returns
    /// the logits of the last position, [batch, vocab].
    pub fn forward(
        &self,
        ids: Tensor<B, 2, Int>,
        cache: &mut [KvCache<B>],
        rope: &Rope<B>,
        pos: usize,
    ) -> Tensor<B, 2> {
        let [batch, seq] = ids.dims();
        let device = ids.device();
        let mask = (seq > 1).then(|| {
            Tensor::<B, 2, Bool>::tril_mask([seq, pos + seq], pos as i64, &device).unsqueeze::<4>()
        });
        let mut h = self.model.embed_tokens.forward(ids);
        for (layer, kv) in self.model.layers.iter().zip(cache.iter_mut()) {
            h = layer.forward(h, kv, rope, pos, mask.clone());
        }
        let h = self.model.norm.forward(h);
        let [_, _, hidden] = h.dims();
        let last = h.slice([0..batch, seq - 1..seq]).reshape([batch, hidden]);
        self.lm_head.forward(last)
    }
}

impl<B: Backend> Layer<B> {
    fn init(spec: &Spec, layout: LinearLayout, device: &B::Device) -> Self {
        let head_dim = spec.head_dim();
        let kv = spec.kv_heads * head_dim;
        Self {
            self_attn: Attention {
                q_proj: linear(spec.hidden, spec.heads * head_dim, layout, device),
                k_proj: linear(spec.hidden, kv, layout, device),
                v_proj: linear(spec.hidden, kv, layout, device),
                o_proj: linear(spec.heads * head_dim, spec.hidden, layout, device),
                heads: spec.heads,
                kv_heads: spec.kv_heads,
                head_dim,
            },
            mlp: Mlp {
                gate_proj: linear(spec.hidden, spec.intermediate, layout, device),
                up_proj: linear(spec.hidden, spec.intermediate, layout, device),
                down_proj: linear(spec.intermediate, spec.hidden, layout, device),
            },
            input_layernorm: rms(spec, device),
            post_attention_layernorm: rms(spec, device),
        }
    }

    fn forward(
        &self,
        x: Tensor<B, 3>,
        kv: &mut KvCache<B>,
        rope: &Rope<B>,
        pos: usize,
        mask: Option<Tensor<B, 4, Bool>>,
    ) -> Tensor<B, 3> {
        let h = x.clone()
            + self
                .self_attn
                .forward(self.input_layernorm.forward(x), kv, rope, pos, mask);
        h.clone() + self.mlp.forward(self.post_attention_layernorm.forward(h))
    }
}

impl<B: Backend> Attention<B> {
    /// Self-attention with the kv cache; `x` is [batch, seq, hidden].
    fn forward(
        &self,
        x: Tensor<B, 3>,
        kv: &mut KvCache<B>,
        rope: &Rope<B>,
        pos: usize,
        mask: Option<Tensor<B, 4, Bool>>,
    ) -> Tensor<B, 3> {
        let [batch, seq, hidden] = x.dims();
        let heads =
            |t: Tensor<B, 3>, n: usize| t.reshape([batch, seq, n, self.head_dim]).swap_dims(1, 2);
        let q = heads(self.q_proj.forward(x.clone()), self.heads);
        let k = heads(self.k_proj.forward(x.clone()), self.kv_heads);
        let v = heads(self.v_proj.forward(x), self.kv_heads);

        let q = rope.apply(q, pos);
        let k = rope.apply(k, pos);
        let (k, v) = kv.append(k, v);
        let k = self.repeat_kv(k);
        let v = self.repeat_kv(v);

        let dtype = q.dtype();
        let q = q.cast(DType::F32);
        let k = k.cast(DType::F32);
        let v = v.cast(DType::F32);
        let mut scores = q
            .matmul(k.swap_dims(2, 3))
            .div_scalar((self.head_dim as f32).sqrt());
        if let Some(mask) = mask {
            scores = scores.mask_fill(mask, f32::NEG_INFINITY);
        }
        let out = softmax(scores, 3).matmul(v).cast(dtype);
        let out = out.swap_dims(1, 2).reshape([batch, seq, hidden]);
        self.o_proj.forward(out)
    }

    /// Repeats each key and value head for the query heads that share it.
    fn repeat_kv(&self, x: Tensor<B, 4>) -> Tensor<B, 4> {
        let rep = self.heads / self.kv_heads;
        if rep == 1 {
            return x;
        }
        let [batch, kv_heads, seq, head_dim] = x.dims();
        x.unsqueeze_dim::<5>(2)
            .expand([batch, kv_heads, rep, seq, head_dim])
            .reshape([batch, kv_heads * rep, seq, head_dim])
    }
}

impl<B: Backend> Mlp<B> {
    fn forward(&self, x: Tensor<B, 3>) -> Tensor<B, 3> {
        let gate = silu(self.gate_proj.forward(x.clone()));
        self.down_proj.forward(gate * self.up_proj.forward(x))
    }
}
