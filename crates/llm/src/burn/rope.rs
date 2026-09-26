use burn::tensor::backend::Backend;
use burn::tensor::{Tensor, TensorData};

/// Rotary position embedding in the Hugging Face layout (`rotate_half`): the
/// two halves of each head form the pairs, as the checkpoint was trained and
/// as candle applies it, so `q_proj` and `k_proj` load unpermuted. burn's own
/// `RotaryEncoding` pairs neighbours and builds its angles in the backend
/// float, which in f16 loses the position; these tables are built on the cpu
/// in f32 and only then stored in the backend float.
#[derive(Clone, Debug)]
pub struct Rope<B: Backend> {
    cos: Tensor<B, 2>,
    sin: Tensor<B, 2>,
}

/// The angle tables, [context * head_dim] each, row per position.
fn tables(head_dim: usize, context: usize, theta: f32) -> (Vec<f32>, Vec<f32>) {
    let half = head_dim / 2;
    let inv: Vec<f32> = (0..half)
        .map(|k| 1.0 / theta.powf((2 * k) as f32 / head_dim as f32))
        .collect();
    let mut cos = Vec::with_capacity(context * head_dim);
    let mut sin = Vec::with_capacity(context * head_dim);
    for pos in 0..context {
        for i in 0..head_dim {
            let angle = pos as f32 * inv[i % half];
            cos.push(angle.cos());
            sin.push(angle.sin());
        }
    }
    (cos, sin)
}

impl<B: Backend> Rope<B> {
    /// Tables for positions `0..context`.
    pub fn new(head_dim: usize, context: usize, theta: f32, device: &B::Device) -> Self {
        let (cos, sin) = tables(head_dim, context, theta);
        let shape = [context, head_dim];
        Self {
            cos: Tensor::from_data(TensorData::new(cos, shape), device),
            sin: Tensor::from_data(TensorData::new(sin, shape), device),
        }
    }

    /// Rotates `x` ([batch, heads, seq, head dim]) for positions
    /// `start..start + seq`.
    pub fn apply(&self, x: Tensor<B, 4>, start: usize) -> Tensor<B, 4> {
        let [_, _, seq, dim] = x.dims();
        let half = dim / 2;
        let cos = self.cos.clone().narrow(0, start, seq).unsqueeze::<4>();
        let sin = self.sin.clone().narrow(0, start, seq).unsqueeze::<4>();
        let first = x.clone().narrow(3, 0, half);
        let second = x.clone().narrow(3, half, half);
        let rotated = Tensor::cat(vec![second.neg(), first], 3);
        x * cos + rotated * sin
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::NdArray;

    type B = NdArray<f32>;

    #[test]
    fn position_zero_is_the_identity() {
        let (cos, sin) = tables(4, 3, 10_000.0);
        assert_eq!(&cos[..4], &[1.0; 4]);
        assert_eq!(&sin[..4], &[0.0; 4]);
    }

    #[test]
    fn the_halves_share_a_frequency() {
        let (cos, _) = tables(8, 5, 10_000.0);
        let row = &cos[4 * 8..5 * 8];
        assert_eq!(row[..4], row[4..]);
        assert!(
            (row[0] - 4f32.cos()).abs() < 1e-6,
            "dim 0 turns once per position"
        );
    }

    #[test]
    fn a_rotation_keeps_the_norm_and_matches_rotate_half() {
        let device = Default::default();
        let rope = Rope::<B>::new(4, 8, 10_000.0, &device);
        let x = Tensor::<B, 4>::from_data(
            TensorData::new(vec![1.0f32, 2.0, 3.0, 4.0], [1, 1, 1, 4]),
            &device,
        );
        let y: Vec<f32> = rope.apply(x, 3).into_data().to_vec().unwrap();
        // pair (x0, x2) turns by 3 rad, pair (x1, x3) by 3 * 1e-2 rad.
        let (a, b) = (3.0f32, 3.0f32 * 0.01);
        let expected = [
            a.cos() - 3.0 * a.sin(),
            2.0 * b.cos() - 4.0 * b.sin(),
            3.0 * a.cos() + a.sin(),
            4.0 * b.cos() + 2.0 * b.sin(),
        ];
        for (got, want) in y.iter().zip(expected) {
            assert!((got - want).abs() < 1e-5, "{y:?} vs {expected:?}");
        }
        let norm: f32 = y.iter().map(|v| v * v).sum();
        assert!((norm - 30.0).abs() < 1e-4);
    }
}
