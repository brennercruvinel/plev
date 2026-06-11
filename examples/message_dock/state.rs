//! Data, animation state, and utilities for the MessageDock example.

use plev::compositor::Compositor;
use plev::input::ViewId;

// ── Color Palette ──────────────────────────────────────────────────────

pub(crate) const BG: [f32; 4] = [0.06, 0.06, 0.10, 1.0];
pub(crate) const DOCK_BG: [f32; 4] = [0.12, 0.12, 0.18, 1.0];
pub(crate) const DOCK_BORDER: [f32; 4] = [0.20, 0.20, 0.28, 0.6];
pub(crate) const TEXT_PRIMARY: [f32; 4] = [0.93, 0.93, 0.96, 1.0];
pub(crate) const TEXT_DIM: [f32; 4] = [0.55, 0.55, 0.65, 1.0];
pub(crate) const TEXT_PLACEHOLDER: [f32; 4] = [0.50, 0.50, 0.60, 0.8];
pub(crate) const ONLINE_GREEN: [f32; 4] = [0.20, 0.85, 0.45, 1.0];
pub(crate) const SEND_BTN: [f32; 4] = [0.30, 0.55, 1.0, 1.0];
pub(crate) const SEND_BTN_HOVER: [f32; 4] = [0.40, 0.65, 1.0, 1.0];
pub(crate) const SENT_FLASH: [f32; 4] = [0.20, 0.85, 0.45, 0.9];

// Character colors
pub(crate) const WIZARD_BG: [f32; 4] = [0.30, 0.78, 0.50, 1.0];
pub(crate) const WIZARD_ACCENT: [f32; 4] = [0.18, 0.55, 0.32, 0.25];
pub(crate) const UNICORN_BG: [f32; 4] = [0.65, 0.40, 0.90, 1.0];
pub(crate) const UNICORN_ACCENT: [f32; 4] = [0.45, 0.22, 0.65, 0.25];
pub(crate) const MONKEY_BG: [f32; 4] = [0.95, 0.82, 0.22, 1.0];
pub(crate) const MONKEY_ACCENT: [f32; 4] = [0.70, 0.60, 0.12, 0.25];
pub(crate) const ROBOT_BG: [f32; 4] = [0.92, 0.38, 0.35, 1.0];
pub(crate) const ROBOT_ACCENT: [f32; 4] = [0.65, 0.20, 0.18, 0.25];
pub(crate) const SPARKLE_BG: [f32; 4] = [0.85, 0.75, 0.20, 0.6];

// ── Data ───────────────────────────────────────────────────────────────

pub(crate) struct DockCharacter {
    pub(crate) initial: &'static str,
    pub(crate) name: &'static str,
    pub(crate) online: bool,
    pub(crate) bg_color: [f32; 4],
    pub(crate) accent_color: [f32; 4],
}

pub(crate) const NUM_CHARS: usize = 5;

pub(crate) const CHARACTERS: &[DockCharacter] = &[
    DockCharacter {
        initial: "*",
        name: "Sparkle",
        online: false,
        bg_color: SPARKLE_BG,
        accent_color: SPARKLE_BG,
    },
    DockCharacter {
        initial: "W",
        name: "Wizard",
        online: true,
        bg_color: WIZARD_BG,
        accent_color: WIZARD_ACCENT,
    },
    DockCharacter {
        initial: "U",
        name: "Unicorn",
        online: true,
        bg_color: UNICORN_BG,
        accent_color: UNICORN_ACCENT,
    },
    DockCharacter {
        initial: "M",
        name: "Monkey",
        online: true,
        bg_color: MONKEY_BG,
        accent_color: MONKEY_ACCENT,
    },
    DockCharacter {
        initial: "R",
        name: "Robot",
        online: false,
        bg_color: ROBOT_BG,
        accent_color: ROBOT_ACCENT,
    },
];

// ── Pixel-snap & Animation Utilities ───────────────────────────────────

/// Snap to pixel grid -- eliminates sub-pixel aliasing shimmer.
pub(crate) fn px(v: f32) -> f32 {
    v.round()
}

pub(crate) fn lerp_color(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

/// Smooth exponential interpolation for pixel values.
/// Snaps to target when within 0.5px to avoid endless crawl.
pub(crate) fn smooth(current: f32, target: f32, speed: f32) -> f32 {
    let diff = target - current;
    if diff.abs() < 0.5 {
        target
    } else {
        current + diff * speed
    }
}

/// Smooth for normalized values (opacity, scale: 0..1 range).
/// Snaps at 0.005 threshold.
pub(crate) fn smooth_n(current: f32, target: f32, speed: f32) -> f32 {
    let diff = target - current;
    if diff.abs() < 0.005 {
        target
    } else {
        current + diff * speed
    }
}

pub(crate) fn smooth_color(current: [f32; 4], target: [f32; 4], speed: f32) -> [f32; 4] {
    [
        smooth_n(current[0], target[0], speed),
        smooth_n(current[1], target[1], speed),
        smooth_n(current[2], target[2], speed),
        smooth_n(current[3], target[3], speed),
    ]
}

/// Push a pixel-snapped rect.
pub(crate) fn draw_rect(comp: &mut Compositor, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
    comp.push(plev::compositor::SceneNode::Rect {
        x: px(x),
        y: px(y),
        w: px(w).max(0.0),
        h: px(h).max(0.0),
        color,
    });
}

// ── Dock Component ─────────────────────────────────────────────────────

pub(crate) struct AnimatedDock {
    // Interaction state
    pub(crate) selected: Option<usize>,
    pub(crate) hovered_char: Option<usize>,
    pub(crate) send_hovered: bool,
    pub(crate) sent_flash_timer: f32,

    // View IDs (recreated each frame)
    pub(crate) char_view_ids: [Option<ViewId>; NUM_CHARS],
    pub(crate) send_btn_id: Option<ViewId>,

    // Animated properties
    pub(crate) dock_width: f32,
    pub(crate) dock_bg_color: [f32; 4],
    pub(crate) char_x: [f32; NUM_CHARS],
    pub(crate) char_y_offsets: [f32; NUM_CHARS],
    pub(crate) char_opacities: [f32; NUM_CHARS],
    pub(crate) input_opacity: f32,
    pub(crate) send_btn_opacity: f32,
    pub(crate) separator_opacity: f32,

    // Layout
    pub(crate) collapsed_width: f32,
    pub(crate) expanded_width: f32,
    pub(crate) dock_height: f32,
    pub(crate) avatar_size: f32,
    pub(crate) avatar_gap: f32,
}

impl AnimatedDock {
    pub(crate) fn new() -> Self {
        let asz = 40.0_f32;
        let gap = 10.0_f32;
        let pad = 10.0_f32;

        let sparkle_x = pad;
        let chars_start = pad + asz + 16.0;
        let collapsed_w = chars_start + 4.0 * (asz + gap) - gap + 16.0 + asz + pad;

        let mut char_x = [0.0; NUM_CHARS];
        char_x[0] = sparkle_x;
        for i in 1..NUM_CHARS {
            char_x[i] = chars_start + (i - 1) as f32 * (asz + gap);
        }

        Self {
            selected: None,
            hovered_char: None,
            send_hovered: false,
            sent_flash_timer: 0.0,
            char_view_ids: [None; NUM_CHARS],
            send_btn_id: None,

            dock_width: collapsed_w,
            dock_bg_color: DOCK_BG,
            char_x,
            char_y_offsets: [0.0; NUM_CHARS],
            char_opacities: [1.0; NUM_CHARS],
            input_opacity: 0.0,
            send_btn_opacity: 0.0,
            separator_opacity: 1.0,

            collapsed_width: collapsed_w,
            expanded_width: 480.0,
            dock_height: 56.0,
            avatar_size: asz,
            avatar_gap: gap,
        }
    }
}
