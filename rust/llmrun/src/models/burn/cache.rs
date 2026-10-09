// adapted from burn-lm (https://github.com/tracel-ai/burn-lm), commit a53baab,
// crates/burn-lm-llama/src/nn/attention/{kv_cache.rs, cache.rs},
// MIT OR Apache-2.0; see rust/llmrun/NOTICE. changes: the backend parameter of
// burn 0.21 back, key and value in one type, and no shift or shrink strategy:
// the stream refuses a request longer than the context before the first step,
// so the buffer never overflows.

use burn::tensor::Tensor;
use burn::tensor::backend::Backend;

/// The keys and values of one transformer block, in a buffer preallocated
/// for the whole context: [batch, kv heads, context, head dim].
#[derive(Clone, Debug)]
pub struct KvCache<B: Backend> {
    key: Tensor<B, 4>,
    value: Tensor<B, 4>,
    len: usize,
}

impl<B: Backend> KvCache<B> {
    /// An empty cache.
    pub fn new(shape: [usize; 4], device: &B::Device) -> Self {
        Self {
            key: Tensor::empty(shape, device),
            value: Tensor::empty(shape, device),
            len: 0,
        }
    }

    /// Forgets the sequence; the buffer stays.
    pub fn reset(&mut self) {
        self.len = 0;
    }

    /// How many positions are cached.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Writes the new keys and values ([batch, kv heads, seq, head dim]) after
    /// the cached ones and returns everything cached so far.
    pub fn append(
        &mut self,
        key: Tensor<B, 4>,
        value: Tensor<B, 4>,
    ) -> (Tensor<B, 4>, Tensor<B, 4>) {
        let [batch, heads, seq, dim] = key.dims();
        let end = self.len + seq;
        let at = [0..batch, 0..heads, self.len..end, 0..dim];
        self.key.inplace(|c| c.slice_assign(at.clone(), key));
        self.value.inplace(|c| c.slice_assign(at, value));
        self.len = end;
        let all = [0..batch, 0..heads, 0..end, 0..dim];
        (
            self.key.clone().slice(all.clone()),
            self.value.clone().slice(all),
        )
    }
}
