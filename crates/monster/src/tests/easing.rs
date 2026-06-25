//! easing wire-table tests: preset <-> byte bijection on every preset,
//! spec anchor ids, custom bezier escape, plev mirror round-trip.

use crate::easing::{CUSTOM_BEZIER_BYTE, Easing, MAX_PRESET_BYTE, PRESETS};
use engine::animation::Easing as PlevEasing;

#[test]
fn preset_byte_roundtrip_is_bijective() {
    let mut seen = Vec::new();
    for &e in PRESETS.iter() {
        let b = e.byte();
        assert!(b <= MAX_PRESET_BYTE, "{e:?} got reserved id {b:#04x}");
        assert!(!seen.contains(&b), "duplicate id {b:#04x} for {e:?}");
        seen.push(b);
        assert_eq!(Easing::from_preset_byte(b), Some(e));
    }
    // dense table: every byte up to MAX_PRESET_BYTE is assigned
    assert_eq!(seen.len(), usize::from(MAX_PRESET_BYTE) + 1);
}

#[test]
fn every_assigned_byte_decodes_back_to_its_own_id() {
    for b in 0u8..=u8::MAX {
        if let Some(e) = Easing::from_preset_byte(b) {
            assert_eq!(e.byte(), b);
            assert!(b <= MAX_PRESET_BYTE);
        }
    }
}

#[test]
fn spec_anchor_ids_hold() {
    // kdb/adr/monster-format-v0.md decision 4: 0x00 linear, 0x01 hold,
    // 0x02 ae-default (ease-in-out).
    assert_eq!(Easing::Linear.byte(), 0x00);
    assert_eq!(Easing::Hold.byte(), 0x01);
    assert_eq!(Easing::EaseInOut.byte(), 0x02);
}

#[test]
fn custom_bezier_is_the_escape_byte_not_a_preset() {
    let curve = Easing::CustomBezier {
        x1: 0.25,
        y1: 0.1,
        x2: 0.25,
        y2: 1.0,
    };
    assert_eq!(curve.byte(), CUSTOM_BEZIER_BYTE);
    assert_eq!(Easing::from_preset_byte(CUSTOM_BEZIER_BYTE), None);
}

#[test]
fn reserved_bytes_decode_to_none() {
    for b in (MAX_PRESET_BYTE + 1)..u8::MAX {
        assert_eq!(Easing::from_preset_byte(b), None, "byte {b:#04x}");
    }
}

#[test]
fn plev_mirror_roundtrips_every_preset() {
    for &e in PRESETS.iter() {
        let lowered: PlevEasing = e.into();
        assert_eq!(Easing::from(lowered), e);
    }
    let curve = Easing::CustomBezier {
        x1: 0.42,
        y1: 0.0,
        x2: 0.58,
        y2: 1.0,
    };
    let lowered: PlevEasing = curve.into();
    assert_eq!(lowered, PlevEasing::CubicBezier(0.42, 0.0, 0.58, 1.0));
    assert_eq!(Easing::from(lowered), curve);
}

#[test]
fn sample_goes_through_plev_ease() {
    assert_eq!(Easing::Linear.sample(0.25), 0.25);
    assert_eq!(Easing::Hold.sample(0.99), 0.0);
    assert_eq!(Easing::Hold.sample(1.0), 1.0);
}
