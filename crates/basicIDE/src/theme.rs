//! Design token system for basicIDE — HOFF visual language.
//!
//! Tokens transcribed 1:1 from the HOFF social/cards specs
//! (`ref/hoff-research-social/styles/variables.sass` et al.):
//! a monochromatic "dark glass" system where almost every color derives
//! from `$n2 = #f8f8f8` or `$n3 = #282828` with an alpha, plus three
//! chromatic accents (red #BD3027, green #55F08B, orange #FF4D00).
use plev::color::Color;

// -- Base palette ($n1..$n4 from variables.sass) ----------------------------
// Catálogo de tokens: constantes ainda não consumidas pelas views fazem
// parte da paleta e ficam disponíveis para os próximos componentes.

#[allow(dead_code)]
pub const N1: Color = Color::hex(0xFFFFFF);
#[allow(dead_code)]
pub const N2: Color = Color::hex(0xF8F8F8);
#[allow(dead_code)]
pub const N3: Color = Color::hex(0x282828);
#[allow(dead_code)]
pub const N4: Color = Color::hex(0x121212);

const N2_F: f32 = 248.0 / 255.0;
const N3_F: f32 = 40.0 / 255.0;

/// `rgba($n2, alpha)` — the white #f8f8f8 every text/surface alpha derives from.
pub const fn n2(alpha: f32) -> Color {
    Color::rgba(N2_F, N2_F, N2_F, alpha)
}

/// `rgba($n3, alpha)` — the graphite #282828 behind buttons/panels.
pub const fn n3(alpha: f32) -> Color {
    Color::rgba(N3_F, N3_F, N3_F, alpha)
}

/// `rgba(255,255,255, alpha)` — edge-light borders are pure white.
pub const fn white(alpha: f32) -> Color {
    Color::rgba(1.0, 1.0, 1.0, alpha)
}

// Light mode derives from "ink" (inverted $n2).
const INK_F: f32 = 7.0 / 255.0;
const fn ink(alpha: f32) -> Color {
    Color::rgba(INK_F, INK_F, INK_F, alpha)
}

// Catálogo de tokens: campos ainda não consumidos pelas views fazem parte
// da paleta e ficam disponíveis para os próximos componentes.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    // -- Backgrounds --
    /// Global frame behind the app (`body` = #444444 in common.sass).
    pub bg_body: Color,
    /// Center container surface — `$bg-surface` = rgba(40,40,40,.7).
    pub bg_panel: Color,
    /// Sidebar / right-sidebar surface — rgba(40,40,40,.8).
    pub bg_sidebar: Color,
    /// Solid dropdown body (Actions) — #3b3b3b.
    pub bg_popover: Color,
    /// Tabs container — rgba($n3,.6).
    pub bg_tabs: Color,
    /// Tooltip — #262626.
    pub bg_tooltip: Color,
    /// Modal overlay — rgba(35,34,34,.9).
    pub bg_overlay: Color,

    // -- Surfaces / cards (always rgba($n2, alpha)) --
    /// Card at rest — rgba($n2,.02).
    pub surface: Color,
    /// Card hover — rgba($n2,.05).
    pub surface_hover: Color,
    /// Card active/selected — rgba($n2,.10).
    pub surface_active: Color,
    /// Chips/tags and circular menu icons — rgba($n2,.05).
    pub chip: Color,

    // -- Buttons / fields --
    /// Default glass button — rgba(40,40,40,.70).
    pub button_bg: Color,
    /// Button hover — rgba($n2,.10).
    pub button_hover_bg: Color,
    /// Field bg — rgba($n2,.05).
    pub field_bg: Color,
    /// Field focus border — rgba($n2,.25).
    pub field_focus_border: Color,

    // -- Text scale (alphas over $n2) --
    /// .95 — highlighted names / card titles.
    pub text_primary: Color,
    /// .70 — button labels, support text.
    pub text_secondary: Color,
    /// .50 — bio / tertiary.
    pub text_tertiary: Color,
    /// .76 — active/hover text & icons.
    pub text_active: Color,
    /// .56 — default menu text, icons, inactive titles.
    pub text_default: Color,
    /// .40 — inactive text, default icon fill, usernames.
    pub text_muted: Color,
    /// .25 — placeholders, list dates, accordion heads.
    pub text_placeholder: Color,
    /// .22 — post/topic timestamps.
    pub text_timestamp: Color,

    // -- Accents (the only chromatics) --
    /// #BD3027 — notification badge, unfollow, destructive.
    pub accent_red: Color,
    /// #55F08B — online/new dot, topic status.
    pub accent_green: Color,
    /// rgba(124,255,176,.7) — repost active.
    pub accent_green_soft: Color,
    /// rgba(255,77,0,.9) — reaction heart.
    pub accent_orange: Color,

    // -- Edge-light borders (rgba(255,255,255,...) + top-lit mask) --
    /// 1–1.5px rgba(255,255,255,.05) — card/modal rim.
    pub edge: Color,
    /// rgba(255,255,255,.10) — buttons, active nav item rim.
    pub edge_strong: Color,
    /// rgba($n2,.40) — "active" element border (cards spec).
    pub border_active: Color,

    // -- Motion (seconds) --
    /// .2s — global default for every hover/active transition.
    pub motion: f32,
    /// .3s — tabs slide, sidebar collapse, modal fade.
    pub motion_slow: f32,

    // -- Border radii (px, from the spec radii table) --
    /// 32 — pills/buttons, Modal, message composer.
    pub radius_pill: f32,
    /// 24 — Actions dropdown, post images.
    pub radius_dropdown: f32,
    /// 22 — Tabs container, Search, Select.
    pub radius_tabs: f32,
    /// 20 — all list cards.
    pub radius_card: f32,
    /// 18 — active tab block, counter pill.
    pub radius_block: f32,
    /// 16 — notify toast, Actions item, reaction chip.
    pub radius_item: f32,
    /// 12 — NavLink, Field, Switch.
    pub radius_nav: f32,
    /// 10 — message action cluster.
    pub radius_cluster: f32,
    /// 8 — tooltip, tags, topic thumb.
    pub radius_tooltip: f32,
    /// 6 — 24px micro-action.
    pub radius_micro: f32,
}

pub const DARK: Theme = Theme {
    bg_body: Color::hex(0x444444),
    bg_panel: n3(0.7),
    bg_sidebar: n3(0.8),
    bg_popover: Color::hex(0x3B3B3B),
    bg_tabs: n3(0.6),
    bg_tooltip: Color::hex(0x262626),
    bg_overlay: Color::rgba(35.0 / 255.0, 34.0 / 255.0, 34.0 / 255.0, 0.9),

    surface: n2(0.02),
    surface_hover: n2(0.05),
    surface_active: n2(0.10),
    chip: n2(0.05),

    button_bg: n3(0.70),
    button_hover_bg: n2(0.10),
    field_bg: n2(0.05),
    field_focus_border: n2(0.25),

    text_primary: n2(0.95),
    text_secondary: n2(0.70),
    text_tertiary: n2(0.50),
    text_active: n2(0.76),
    text_default: n2(0.56),
    text_muted: n2(0.40),
    text_placeholder: n2(0.25),
    text_timestamp: n2(0.22),

    accent_red: Color::hex(0xBD3027),
    accent_green: Color::hex(0x55F08B),
    accent_green_soft: Color::rgba(124.0 / 255.0, 1.0, 176.0 / 255.0, 0.7),
    accent_orange: Color::rgba(1.0, 77.0 / 255.0, 0.0, 0.9),

    edge: white(0.05),
    edge_strong: white(0.10),
    border_active: n2(0.40),

    motion: 0.2,
    motion_slow: 0.3,

    radius_pill: 32.0,
    radius_dropdown: 24.0,
    radius_tabs: 22.0,
    radius_card: 20.0,
    radius_block: 18.0,
    radius_item: 16.0,
    radius_nav: 12.0,
    radius_cluster: 10.0,
    radius_tooltip: 8.0,
    radius_micro: 6.0,
};

/// Light mode — the dark spec inverted: ink (#070707) alphas over pale glass.
pub const LIGHT: Theme = Theme {
    bg_body: Color::hex(0xBBBBBB),
    bg_panel: Color::rgba(215.0 / 255.0, 215.0 / 255.0, 215.0 / 255.0, 0.7),
    bg_sidebar: Color::rgba(215.0 / 255.0, 215.0 / 255.0, 215.0 / 255.0, 0.8),
    bg_popover: Color::hex(0xC4C4C4),
    bg_tabs: Color::rgba(215.0 / 255.0, 215.0 / 255.0, 215.0 / 255.0, 0.6),
    bg_tooltip: Color::hex(0xD9D9D9),
    bg_overlay: Color::rgba(220.0 / 255.0, 221.0 / 255.0, 221.0 / 255.0, 0.9),

    surface: ink(0.02),
    surface_hover: ink(0.05),
    surface_active: ink(0.10),
    chip: ink(0.05),

    button_bg: Color::rgba(215.0 / 255.0, 215.0 / 255.0, 215.0 / 255.0, 0.70),
    button_hover_bg: ink(0.10),
    field_bg: ink(0.05),
    field_focus_border: ink(0.25),

    text_primary: ink(0.95),
    text_secondary: ink(0.70),
    text_tertiary: ink(0.50),
    text_active: ink(0.76),
    text_default: ink(0.56),
    text_muted: ink(0.40),
    text_placeholder: ink(0.25),
    text_timestamp: ink(0.22),

    accent_red: Color::hex(0xBD3027),
    accent_green: Color::hex(0x2BB163),
    accent_green_soft: Color::rgba(43.0 / 255.0, 177.0 / 255.0, 99.0 / 255.0, 0.7),
    accent_orange: Color::rgba(1.0, 77.0 / 255.0, 0.0, 0.9),

    edge: Color::rgba(0.0, 0.0, 0.0, 0.05),
    edge_strong: Color::rgba(0.0, 0.0, 0.0, 0.10),
    border_active: ink(0.40),

    motion: 0.2,
    motion_slow: 0.3,

    radius_pill: 32.0,
    radius_dropdown: 24.0,
    radius_tabs: 22.0,
    radius_card: 20.0,
    radius_block: 18.0,
    radius_item: 16.0,
    radius_nav: 12.0,
    radius_cluster: 10.0,
    radius_tooltip: 8.0,
    radius_micro: 6.0,
};

// -- Analytic drop-shadow specs (box-shadow values from the spec) -----------

/// One CSS box-shadow layer, rendered via `SceneNode::Shadow`
/// (`spread` shrinks/expands the casting rect; plev has no spread param).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowSpec {
    pub offset: [f32; 2],
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

const fn shade5(alpha: f32) -> Color {
    Color::rgba(5.0 / 255.0, 5.0 / 255.0, 5.0 / 255.0, alpha)
}

/// Floating menus: `0 24px 32px -12px rgba(18,18,18,.10)` (Actions dropdown).
pub const SHADOW_MENU: ShadowSpec = ShadowSpec {
    offset: [0.0, 24.0],
    blur: 32.0,
    spread: -12.0,
    color: Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.10),
};

/// Deep stack (modal/nav/options), from Modal.module.sass:
/// `0 24px 24px -16px rgba(5,5,5,.09), 0 6px 13px rgba(5,5,5,.10),
///  0 6px 4px -4px rgba(5,5,5,.10), 0 5px 2px -4px rgba(5,5,5,.25)`.
pub const SHADOW_MODAL: [ShadowSpec; 4] = [
    ShadowSpec {
        offset: [0.0, 24.0],
        blur: 24.0,
        spread: -16.0,
        color: shade5(0.09),
    },
    ShadowSpec {
        offset: [0.0, 6.0],
        blur: 13.0,
        spread: 0.0,
        color: shade5(0.10),
    },
    ShadowSpec {
        offset: [0.0, 6.0],
        blur: 4.0,
        spread: -4.0,
        color: shade5(0.10),
    },
    ShadowSpec {
        offset: [0.0, 5.0],
        blur: 2.0,
        spread: -4.0,
        color: shade5(0.25),
    },
];

/// Tooltip: `0 1.5px 2px rgba(24,24,24,.15)`.
pub const SHADOW_TOOLTIP: ShadowSpec = ShadowSpec {
    offset: [0.0, 1.5],
    blur: 2.0,
    spread: 0.0,
    color: Color::rgba(24.0 / 255.0, 24.0 / 255.0, 24.0 / 255.0, 0.15),
};

/// Active tabs block: `0 8px 16px -4px rgba(18,18,18,.20)`.
#[allow(dead_code)]
pub const SHADOW_TABS_BLOCK: ShadowSpec = ShadowSpec {
    offset: [0.0, 8.0],
    blur: 16.0,
    spread: -4.0,
    color: Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.20),
};

/// File status colors — harmonized with the HOFF accent set (the spec is
/// monochromatic; the only chromatics are red/green/orange, so file states
/// map onto those instead of a foreign yellow/blue/purple palette).
#[allow(dead_code)]
pub struct StatusColors {
    pub modified: Color,
    pub added: Color,
    pub deleted: Color,
    pub renamed: Color,
    pub untracked: Color,
}

#[allow(dead_code)]
impl StatusColors {
    pub fn of(theme: &Theme) -> Self {
        Self {
            modified: theme.accent_orange,
            added: theme.accent_green,
            deleted: theme.accent_red,
            renamed: theme.accent_green_soft,
            untracked: theme.text_muted,
        }
    }

    pub fn dark() -> Self {
        Self::of(&DARK)
    }

    pub fn light() -> Self {
        Self::of(&LIGHT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    fn assert_color(c: Color, r: u8, g: u8, b: u8, a: f32) {
        let [cr, cg, cb, ca] = c.to_array();
        assert!(close(cr, r as f32 / 255.0), "r: {cr} != {r}");
        assert!(close(cg, g as f32 / 255.0), "g: {cg} != {g}");
        assert!(close(cb, b as f32 / 255.0), "b: {cb} != {b}");
        assert!(close(ca, a), "a: {ca} != {a}");
    }

    #[test]
    fn dark_text_scale_matches_hoff_alphas() {
        assert_color(DARK.text_primary, 248, 248, 248, 0.95);
        assert_color(DARK.text_secondary, 248, 248, 248, 0.70);
        assert_color(DARK.text_tertiary, 248, 248, 248, 0.50);
        assert_color(DARK.text_active, 248, 248, 248, 0.76);
        assert_color(DARK.text_default, 248, 248, 248, 0.56);
        assert_color(DARK.text_muted, 248, 248, 248, 0.40);
        assert_color(DARK.text_placeholder, 248, 248, 248, 0.25);
        assert_color(DARK.text_timestamp, 248, 248, 248, 0.22);
    }

    #[test]
    fn dark_surfaces_match_hoff_recipe() {
        assert_color(DARK.bg_body, 0x44, 0x44, 0x44, 1.0);
        assert_color(DARK.bg_panel, 40, 40, 40, 0.7);
        assert_color(DARK.bg_sidebar, 40, 40, 40, 0.8);
        assert_color(DARK.bg_popover, 0x3B, 0x3B, 0x3B, 1.0);
        assert_color(DARK.bg_tabs, 40, 40, 40, 0.6);
        assert_color(DARK.bg_tooltip, 0x26, 0x26, 0x26, 1.0);
        assert_color(DARK.bg_overlay, 35, 34, 34, 0.9);
        assert_color(DARK.surface, 248, 248, 248, 0.02);
        assert_color(DARK.surface_hover, 248, 248, 248, 0.05);
        assert_color(DARK.surface_active, 248, 248, 248, 0.10);
        assert_color(DARK.button_bg, 40, 40, 40, 0.70);
        assert_color(DARK.button_hover_bg, 248, 248, 248, 0.10);
        assert_color(DARK.field_bg, 248, 248, 248, 0.05);
        assert_color(DARK.field_focus_border, 248, 248, 248, 0.25);
        assert_color(DARK.edge, 255, 255, 255, 0.05);
        assert_color(DARK.edge_strong, 255, 255, 255, 0.10);
        assert_color(DARK.border_active, 248, 248, 248, 0.40);
    }

    #[test]
    fn accents_are_the_only_chromatics() {
        assert_color(DARK.accent_red, 0xBD, 0x30, 0x27, 1.0);
        assert_color(DARK.accent_green, 0x55, 0xF0, 0x8B, 1.0);
        assert_color(DARK.accent_green_soft, 124, 255, 176, 0.7);
        assert_color(DARK.accent_orange, 255, 77, 0, 0.9);
    }

    #[test]
    fn radii_match_spec_table() {
        assert_eq!(DARK.radius_pill, 32.0);
        assert_eq!(DARK.radius_dropdown, 24.0);
        assert_eq!(DARK.radius_tabs, 22.0);
        assert_eq!(DARK.radius_card, 20.0);
        assert_eq!(DARK.radius_block, 18.0);
        assert_eq!(DARK.radius_item, 16.0);
        assert_eq!(DARK.radius_nav, 12.0);
        assert_eq!(DARK.radius_cluster, 10.0);
        assert_eq!(DARK.radius_tooltip, 8.0);
        assert_eq!(DARK.radius_micro, 6.0);
    }

    #[test]
    fn motion_durations_match_spec() {
        assert_eq!(DARK.motion, 0.2);
        assert_eq!(DARK.motion_slow, 0.3);
        assert_eq!(LIGHT.motion, 0.2);
        assert_eq!(LIGHT.motion_slow, 0.3);
    }

    #[test]
    fn shadow_specs_match_spec_values() {
        assert_eq!(SHADOW_MENU.offset, [0.0, 24.0]);
        assert_eq!(SHADOW_MENU.blur, 32.0);
        assert_eq!(SHADOW_MENU.spread, -12.0);
        assert_color(SHADOW_MENU.color, 18, 18, 18, 0.10);

        assert_eq!(SHADOW_MODAL.len(), 4);
        assert_eq!(SHADOW_MODAL[0].offset, [0.0, 24.0]);
        assert_eq!(SHADOW_MODAL[0].blur, 24.0);
        assert_eq!(SHADOW_MODAL[0].spread, -16.0);
        assert_color(SHADOW_MODAL[0].color, 5, 5, 5, 0.09);
        assert_color(SHADOW_MODAL[3].color, 5, 5, 5, 0.25);

        assert_eq!(SHADOW_TOOLTIP.offset, [0.0, 1.5]);
        assert_color(SHADOW_TOOLTIP.color, 24, 24, 24, 0.15);
    }

    #[test]
    fn status_colors_reuse_hoff_accents() {
        let s = StatusColors::dark();
        assert_eq!(s.added.to_array(), DARK.accent_green.to_array());
        assert_eq!(s.deleted.to_array(), DARK.accent_red.to_array());
        assert_eq!(s.modified.to_array(), DARK.accent_orange.to_array());
        assert_eq!(s.renamed.to_array(), DARK.accent_green_soft.to_array());
        assert_eq!(s.untracked.to_array(), DARK.text_muted.to_array());
    }

    #[test]
    fn light_theme_keeps_structure_with_inverted_ink() {
        let [r, g, b, a] = LIGHT.text_primary.to_array();
        assert!(r < 0.1 && g < 0.1 && b < 0.1, "light text must be ink");
        assert!(close(a, 0.95));
        assert!(close(LIGHT.surface.to_array()[3], 0.02));
        assert!(close(LIGHT.surface_active.to_array()[3], 0.10));
        assert_eq!(LIGHT.radius_card, 20.0);
    }
}
