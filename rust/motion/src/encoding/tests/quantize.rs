//! quantization round-trip tests: exact identity from the quantized
//! side, bounded error from the float side, clamping at the edges.

use crate::quant::*;

#[test]
fn twips_roundtrip_exact_from_quantized_side() {
    for q in -4000i32..=4000 {
        assert_eq!(px_to_twips(twips_to_px(q)), q);
    }
    assert_eq!(px_to_twips(1.0), 20);
    assert_eq!(twips_to_px(20), 1.0);
    assert_eq!(px_to_twips(-0.05), -1);
}

#[test]
fn twips_float_error_is_at_most_half_a_step() {
    for i in 0..=1000 {
        let px = -25.0 + i as f32 * 0.05003; // off-grid values
        let rt = twips_to_px(px_to_twips(px));
        assert!((rt - px).abs() <= 0.025 + 1e-5, "px={px} rt={rt}");
    }
}

#[test]
fn rgba8_roundtrip_exact_and_clamped() {
    for b in 0u8..=u8::MAX {
        assert_eq!(channel_to_u8(u8_to_channel(b)), b);
    }
    assert_eq!(channel_to_u8(-0.5), 0);
    assert_eq!(channel_to_u8(2.0), 255);
    let c = [0u8, 64, 128, 255];
    assert_eq!(rgba_to_bytes(bytes_to_rgba(c)), c);
}

#[test]
fn ratio_u16_roundtrip_exact_and_clamped() {
    for q in [0u16, 1, 2, 327, 12345, 32768, 65534, 65535] {
        assert_eq!(ratio_to_u16(u16_to_ratio(q)), q);
    }
    assert_eq!(ratio_to_u16(-1.0), 0);
    assert_eq!(ratio_to_u16(1.5), u16::MAX);
}

#[test]
fn ratio_u16_roundtrip_exact_exhaustive() {
    for q in 0u16..=u16::MAX {
        assert_eq!(ratio_to_u16(u16_to_ratio(q)), q);
    }
}

#[test]
fn angle_u16_roundtrip_exact_exhaustive() {
    for q in 0u16..=u16::MAX {
        assert_eq!(angle_deg_to_u16(u16_to_angle_deg(q)), q);
    }
}

#[test]
fn angle_wraps_one_turn() {
    assert_eq!(angle_deg_to_u16(0.0), 0);
    assert_eq!(angle_deg_to_u16(360.0), 0);
    assert_eq!(angle_deg_to_u16(720.0), 0);
    assert_eq!(angle_deg_to_u16(180.0), 32768);
    assert_eq!(angle_deg_to_u16(-90.0), angle_deg_to_u16(270.0));
    // near-360 rounds onto 0, never overflows to 65536
    assert_eq!(angle_deg_to_u16(359.9999), 0);
    let err = 360.0 / 65536.0;
    let rt = u16_to_angle_deg(angle_deg_to_u16(123.456));
    assert!((rt - 123.456).abs() <= err, "rt={rt}");
}

#[test]
fn bezier_bytes_roundtrip_exact_and_cover_overshoot() {
    for b in 0u8..=u8::MAX {
        assert_eq!(bezier_x_to_u8(u8_to_bezier_x(b)), b);
        assert_eq!(bezier_y_to_u8(u8_to_bezier_y(b)), b);
    }
    // x clamps to [0,1], y covers [-0.5,1.5] and clamps beyond
    assert_eq!(bezier_x_to_u8(-1.0), 0);
    assert_eq!(bezier_x_to_u8(2.0), 255);
    assert_eq!(bezier_y_to_u8(-0.5), 0);
    assert_eq!(bezier_y_to_u8(1.5), 255);
    assert_eq!(bezier_y_to_u8(-3.0), 0);
    assert_eq!(u8_to_bezier_y(0), -0.5);
    assert_eq!(u8_to_bezier_y(255), 1.5);
}
