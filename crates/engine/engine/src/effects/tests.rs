use super::*;

#[test]
fn gaussian_weights_sum_to_one() {
    for sigma in [0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0] {
        let w = gaussian_weights(sigma);
        let sum: f32 = w[0] + 2.0 * w[1..7].iter().sum::<f32>();
        assert!(
            (sum - 1.0).abs() < 1e-5,
            "sigma={sigma}: sum={sum} (expected 1.0)"
        );
    }
}

#[test]
fn gaussian_weights_zero_sigma() {
    let w = gaussian_weights(0.0);
    assert_eq!(w[0], 1.0);
    for &tail in &w[1..7] {
        assert_eq!(tail, 0.0);
    }
}

#[test]
fn gaussian_weights_symmetric_decay() {
    let w = gaussian_weights(3.0);
    assert!(w[0] > w[1]);
    for i in 1..6 {
        assert!(
            w[i] >= w[i + 1],
            "weight[{}]={} < weight[{}]={}",
            i,
            w[i],
            i + 1,
            w[i + 1]
        );
    }
}

#[test]
fn gaussian_weights_padding_is_zero() {
    let w = gaussian_weights(3.0);
    for (i, &pad) in w.iter().enumerate().take(16).skip(7) {
        assert_eq!(pad, 0.0, "padding index {i} should be zero");
    }
}
