//! The kit's SidebarPage (`templates/SidebarPage`): its four sidebars.
//! [`HomeSidebar`] and [`GuidelineSidebar`] are the app's rails, a rim at
//! their right; the canvas sidebars (`SceneSidebar`, the scene and its
//! assets, and `DesignSidebar`, design and animation) are rounded
//! panels with a rim all round.
//!
//! What they share is here: the shell, the head with the logo, a group's
//! caption, a folder's row. Every value is `theme.navigation.sidebars`
//! ([`SidebarsRecipe`](super::tokens::SidebarsRecipe)), the links the
//! kit's `navigation.link`, the folder rows the folder picker's
//! (`theme.selection.folders`, front E), the dropdowns the collapsible's
//! (`theme.disclosure`, front D).
//!
//! The logo is a stand-in (a mark and the wordmark) until the Logo of
//! front F is in the engine. TODO(task-104f): draw `Logo` here.

mod guideline;
mod home;

pub use guideline::{GuideGroup, GuidelineSidebar};
pub use home::{HomeLayout, HomeRow, HomeSidebar};

use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::text::TextStyle;
use crate::styling::theme::Theme;
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring, rounded_rect, rounded_rect_stroke};
use crate::widgets::widget::Rect;

/// The rail's surface and its right rim (`border-r`), or a canvas
/// panel's rounded surface and rim all round.
pub(crate) fn shell(c: &mut Compositor, r: Rect, canvas: bool, theme: &Theme) {
    let t = &theme.navigation.sidebars;
    if canvas {
        c.push(rounded_rect(r.x, r.y, r.w, r.h, t.canvas_radius, t.fill.0));
        c.push(rounded_rect_stroke(
            r.x,
            r.y,
            r.w,
            r.h,
            t.canvas_radius,
            t.edge.0,
            t.edge_width,
        ));
    } else {
        c.push(rounded_rect(r.x, r.y, r.w, r.h, 0.0, t.fill.0));
        let w = t.edge_width;
        c.push(rounded_rect(r.right() - w, r.y, w, r.h, 0.0, t.edge.0));
    }
}

/// A rule across `r` at `y`, the rim's tone (`border-b border-[#ECECEC]`).
pub(crate) fn rule(c: &mut Compositor, x: f32, y: f32, w: f32, theme: &Theme) {
    let t = &theme.navigation.sidebars;
    c.push(rounded_rect(x, y, w, t.edge_width, 0.0, t.edge.0));
}

/// The head's height: the logo in its padding.
pub(crate) fn head_h(theme: &Theme) -> f32 {
    let t = &theme.navigation.sidebars;
    t.head_pad * 2.0 + t.logo_h
}

/// The logo at the head's padding: a stand-in mark and wordmark.
pub(crate) fn logo(c: &mut Compositor, frame: Rect, theme: &Theme) {
    let t = &theme.navigation.sidebars;
    let (x, y) = (frame.x + t.head_pad, frame.y + t.head_pad);
    let mark = t.logo_h;
    c.push(rounded_rect(x, y, mark, mark, mark / 4.0, t.logo_mark.0));
    let style = t.logo_text.style(&theme.typography);
    let gap = (t.logo_w - mark).max(0.0).min(mark) / 2.0;
    c.push(text(
        "hoff",
        &style,
        x + mark + gap,
        y + (mark - style.line_height) / 2.0,
        t.logo_mark.0,
        None,
    ));
}

/// A caption's height: its run in its padding, and the gap under it.
pub(crate) fn caption_h(theme: &Theme) -> f32 {
    let t = &theme.navigation.sidebars;
    t.caption_pad * 2.0 + t.caption.line_height + t.caption_gap
}

pub(crate) fn caption(c: &mut Compositor, label: &str, x: f32, y: f32, w: f32, theme: &Theme) {
    let t = &theme.navigation.sidebars;
    let style = t.caption.style(&theme.typography);
    let at = x + t.caption_pad;
    c.push(text(
        label,
        &style,
        at,
        y + t.caption_pad,
        t.caption_color.0,
        Some((w - t.caption_pad * 2.0).max(0.0)),
    ));
}

pub(crate) fn text(
    label: &str,
    style: &TextStyle,
    x: f32,
    y: f32,
    color: [f32; 4],
    max_w: Option<f32>,
) -> SceneNode {
    SceneNode::Text {
        key: TextNodeKey::from_style(label, style, max_w),
        x,
        y,
        color,
    }
}

/// A folder's row as the folder picker draws it: its hover, its ring
/// while focused, the icon in its slot and the label.
pub(crate) fn folder_row(
    c: &mut Compositor,
    r: Rect,
    label: &str,
    icon: (&str, [f32; 4]),
    hovered: bool,
    focused: bool,
    theme: &Theme,
) {
    let t = &theme.selection.folders;
    if hovered {
        c.push(rounded_rect(
            r.x,
            r.y,
            r.w,
            r.h,
            t.row_radius,
            t.row_hover.0,
        ));
    }
    if focused {
        c.push(focus_ring(r, t.row_radius, theme));
    }
    let sx = r.x + (r.h - t.slot) / 2.0;
    let ix = sx + (t.slot - t.icon) / 2.0;
    if let Some(n) = icons::icon_at(icon.0, t.icon, icon.1, ix, r.y + (r.h - t.icon) / 2.0) {
        c.push(n);
    }
    let style = theme
        .typography
        .exact(t.font_size, t.line_height, t.font_weight);
    let x = sx + t.slot + t.gap;
    c.push(text(
        label,
        &style,
        x,
        r.y + (r.h - style.line_height) / 2.0,
        t.label.0,
        Some((r.right() - x).max(0.0)),
    ));
}

#[cfg(test)]
mod tests;
