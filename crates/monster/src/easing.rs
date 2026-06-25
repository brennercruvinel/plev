//! easing model: mirrors the `engine::animation::Easing` presets and adds
//! the 1-byte wire representation of kdb/adr/monster-format-v0.md decision 4.
//!
//! wire table: 0x00 linear, 0x01 hold, 0x02 ae-default (lowered as
//! ease-in-out), 0x03..=0x20 named presets, 0xFF custom cubic bezier
//! (control points quantized per `crate::quant`, deduped in the header,
//! referenced by index).
//!
//! divergence from the draft spec, surfaced on purpose: the draft
//! reserves 0x03..0x1f (29 slots) for named presets, but plev ships 30
//! named presets beyond linear/hold/ae-default, so the dense table ends
//! at 0x20. 0x21..=0xFE stay reserved.

use crate::quant;
use engine::animation::Easing as PlevEasing;

/// Wire id of a custom cubic bezier segment.
pub const CUSTOM_BEZIER_BYTE: u8 = 0xFF;

/// Highest assigned preset id; 0x21..=0xFE are reserved.
pub const MAX_PRESET_BYTE: u8 = 0x20;

/// Easing of a segment. Preset variants mirror `engine::animation::Easing`
/// one to one; `CustomBezier` mirrors `plev`'s `CubicBezier` with f32
/// control points in memory (x in [0,1], y in [-0.5,1.5] on the wire).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
    Step,
    Hold,
    CustomBezier {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
}

/// One source of truth for the preset table: variant <-> byte, plus the
/// plev conversions (names match plev variant for variant).
macro_rules! preset_table {
    ($(($variant:ident, $byte:literal)),+ $(,)?) => {
        /// Every preset variant, for exhaustive bijection tests.
        pub const PRESETS: [Easing; 33] = [$(Easing::$variant),+];

        impl Easing {
            /// 1-byte wire id; `CustomBezier` is [`CUSTOM_BEZIER_BYTE`]
            /// (the curve itself lives in the header dedup table).
            pub fn byte(self) -> u8 {
                match self {
                    $(Easing::$variant => $byte,)+
                    Easing::CustomBezier { .. } => CUSTOM_BEZIER_BYTE,
                }
            }

            /// Inverse of [`Easing::byte`] for presets. `None` for
            /// reserved ids and for 0xFF (custom curves resolve through
            /// the dedup table, never through this function).
            pub fn from_preset_byte(byte: u8) -> Option<Self> {
                match byte {
                    $($byte => Some(Easing::$variant),)+
                    _ => None,
                }
            }
        }

        impl From<PlevEasing> for Easing {
            fn from(e: PlevEasing) -> Self {
                match e {
                    $(PlevEasing::$variant => Easing::$variant,)+
                    PlevEasing::CubicBezier(x1, y1, x2, y2) => {
                        Easing::CustomBezier { x1, y1, x2, y2 }
                    }
                }
            }
        }

        impl From<Easing> for PlevEasing {
            fn from(e: Easing) -> Self {
                match e {
                    $(Easing::$variant => PlevEasing::$variant,)+
                    Easing::CustomBezier { x1, y1, x2, y2 } => {
                        PlevEasing::CubicBezier(x1, y1, x2, y2)
                    }
                }
            }
        }
    };
}

preset_table![
    (Linear, 0x00),
    (Hold, 0x01),
    // 0x02 is the spec's "ae-default" id; it lowers to ease-in-out.
    (EaseInOut, 0x02),
    (EaseIn, 0x03),
    (EaseOut, 0x04),
    (EaseInCubic, 0x05),
    (EaseOutCubic, 0x06),
    (EaseInOutCubic, 0x07),
    (EaseInQuart, 0x08),
    (EaseOutQuart, 0x09),
    (EaseInOutQuart, 0x0A),
    (EaseInQuint, 0x0B),
    (EaseOutQuint, 0x0C),
    (EaseInOutQuint, 0x0D),
    (EaseInSine, 0x0E),
    (EaseOutSine, 0x0F),
    (EaseInOutSine, 0x10),
    (EaseInExpo, 0x11),
    (EaseOutExpo, 0x12),
    (EaseInOutExpo, 0x13),
    (EaseInCirc, 0x14),
    (EaseOutCirc, 0x15),
    (EaseInOutCirc, 0x16),
    (EaseInBack, 0x17),
    (EaseOutBack, 0x18),
    (EaseInOutBack, 0x19),
    (EaseInElastic, 0x1A),
    (EaseOutElastic, 0x1B),
    (EaseInOutElastic, 0x1C),
    (EaseInBounce, 0x1D),
    (EaseOutBounce, 0x1E),
    (EaseInOutBounce, 0x1F),
    (Step, 0x20),
];

impl Easing {
    /// Sample the eased progress at `t` in [0,1] through plev's own
    /// `ease()`, so the player and ui animation share one curve library.
    pub fn sample(self, t: f32) -> f32 {
        engine::animation::ease(t, self.into())
    }
}

/// Quantized wire form of a custom curve's control points (one byte
/// each, x in [0,1], y in [-0.5,1.5]; see `crate::quant`).
pub fn quantize_curve(x1: f32, y1: f32, x2: f32, y2: f32) -> [u8; 4] {
    [
        quant::bezier_x_to_u8(x1),
        quant::bezier_y_to_u8(y1),
        quant::bezier_x_to_u8(x2),
        quant::bezier_y_to_u8(y2),
    ]
}

/// Dedup table of quantized custom bezier curves; segments reference an
/// index instead of repeating control points (spec decision 4). In the
/// container this is the header easing table.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EasingTable {
    curves: Vec<[u8; 4]>,
}

impl EasingTable {
    /// Register the curve of `easing` if custom, keeping first-seen
    /// order; presets need no table entry.
    pub fn intern(&mut self, easing: Easing) {
        if let Easing::CustomBezier { x1, y1, x2, y2 } = easing {
            let curve = quantize_curve(x1, y1, x2, y2);
            if !self.curves.contains(&curve) {
                self.curves.push(curve);
            }
        }
    }

    /// Index of an interned curve; `None` if it was never interned.
    pub fn index_of(&self, curve: [u8; 4]) -> Option<u16> {
        self.curves
            .iter()
            .position(|c| *c == curve)
            .map(|i| i as u16)
    }

    pub fn len(&self) -> usize {
        self.curves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.curves.is_empty()
    }

    pub fn curves(&self) -> &[[u8; 4]] {
        &self.curves
    }
}
