//! Export's three sections (`Export/Images`, `Export/Video`,
//! `Export/Object3D`): their controls and one walk down a section that
//! places them and, given a compositor, draws what is around them, so the
//! layout and the drawing never disagree.

use accord::SemId;

use crate::drawing::compositor::Compositor;
use crate::styling::theme::Theme;
use crate::widgets::buttons::{
    Button, ButtonVariant, IconButton, IconButtonSize, IconButtonVariant,
};
use crate::widgets::form::tokens::SelectLook;
use crate::widgets::form::{Select, Slider};
use crate::widgets::nav::Tabs;
use crate::widgets::widget::Rect;

use super::{
    Controls, Kind, child, glyph, hairline, middle, stroke, text, text_centered, text_end, text_w,
    tokens,
};

/// Which section Export shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Section {
    Images,
    Video,
    Object,
}

pub(super) const SECTIONS: [Section; 3] = [Section::Images, Section::Video, Section::Object];

/// The dimensions an export takes, the kit's.
const DIMENSIONS: &str = "3840px × 2160px";
const ESTIMATE: &str = "Estimation — Export time 30 seconds — Output size 35MB";

fn ghost(options: &[&str], selected: usize, icon: &'static str) -> Select {
    Select::new(options.iter().copied(), selected)
        .look(SelectLook::Ghost)
        .icon(icon)
}

fn tool(icon: &'static str) -> IconButton {
    IconButton::new(icon)
        .variant(IconButtonVariant::Ghost)
        .size(IconButtonSize::Sm)
}

fn tabs(labels: &[&str], active: usize) -> Tabs {
    let mut t = Tabs::new(labels.iter().copied());
    t.active = active;
    t
}

/// `section`'s controls under `<id>.<section>`, the export button last.
pub(super) fn controls(section: Section, id: Option<&SemId>, title: &str) -> Controls {
    let mut c = Controls::default();
    let at = |name: &str| child(id, name);
    match section {
        Section::Images => {
            let base = at("images");
            for row in 0..2 {
                let r = child(base.as_ref(), "size").map(|f| f.key(row.to_string()));
                let size = Select::new(["1x", "2x", "3x"], row)
                    .icon("scan")
                    .small(true);
                c.push(Kind::Select(size, format!("Size {}", row + 1)), r);
                let r = child(base.as_ref(), "color").map(|f| f.key(row.to_string()));
                let color = ghost(&["sRGB", "Adobe Color"], 0, "scan-text").small(true);
                c.push(Kind::Select(color, format!("Color profile {}", row + 1)), r);
                let r = child(base.as_ref(), "format").map(|f| f.key(row.to_string()));
                c.push(
                    Kind::Tabs(tabs(&["PNG", "JPG"], 0), format!("Format {}", row + 1)),
                    r,
                );
                let r = child(base.as_ref(), "remove").map(|f| f.key(row.to_string()));
                c.push(
                    Kind::Icon(tool("minus"), format!("Remove size {}", row + 1)),
                    r,
                );
            }
            c.push(
                Kind::Icon(tool("plus"), "Add a size".into()),
                child(base.as_ref(), "add"),
            );
            let slider = Slider::new(0.0, 100.0, 80.0).step(1.0);
            c.push(
                Kind::Slider(slider, "Compression".into()),
                child(base.as_ref(), "compression"),
            );
        }
        Section::Video => {
            let base = at("video");
            let camera = ghost(&["Camera 1", "Camera 2", "Camera 3"], 0, "scan");
            c.push(
                Kind::Select(camera, "Camera".into()),
                child(base.as_ref(), "camera"),
            );
            c.push(
                Kind::Tabs(tabs(&["MP4", "GIF"], 0), "Format".into()),
                child(base.as_ref(), "format"),
            );
            let rate = ghost(&["60", "30", "24"], 0, "film").label("FPS");
            c.push(
                Kind::Select(rate, "Frame rate".into()),
                child(base.as_ref(), "rate"),
            );
            let res = tabs(&["1080p", "4K"], 1);
            c.push(
                Kind::Tabs(res, "Resolution".into()),
                child(base.as_ref(), "resolution"),
            );
        }
        Section::Object => {
            let base = at("object");
            let format = ghost(&["OBJ", "GLTF", "GLB"], 0, "box");
            c.push(
                Kind::Select(format, "Format".into()),
                child(base.as_ref(), "format"),
            );
            let camera = ghost(&["Camera 1", "Camera 2", "Camera 3"], 0, "scan");
            c.push(
                Kind::Select(camera, "Camera".into()),
                child(base.as_ref(), "camera"),
            );
            c.push(
                Kind::Tabs(tabs(&["Yes", "No"], 0), "Material".into()),
                child(base.as_ref(), "material"),
            );
        }
    }
    let name = match section {
        Section::Images => "images",
        Section::Video => "video",
        Section::Object => "object",
    };
    let export = Button::new(format!("Export {title}")).variant(ButtonVariant::Secondary);
    c.push(
        Kind::Button(export),
        child(child(id, name).as_ref(), "export"),
    );
    c
}

/// A control's own height.
fn height(controls: &Controls, i: usize, theme: &Theme) -> f32 {
    match controls.0.get(i).map(|c| &c.kind) {
        Some(Kind::Select(s, _)) => s.height_in(theme),
        Some(Kind::Tabs(..)) => Tabs::height(theme),
        Some(Kind::Icon(b, _)) => b.preferred_size(theme).1,
        Some(Kind::Button(b)) => b.preferred_size(theme).1,
        _ => 0.0,
    }
}

/// The slider's value, as the box beside it shows it.
fn compression(controls: &Controls) -> f32 {
    controls
        .0
        .iter()
        .find_map(|c| match &c.kind {
            Kind::Slider(s, _) => Some(s.value()),
            _ => None,
        })
        .unwrap_or_default()
}

/// Down `section` in `area`: each control's rect into `out`, what is
/// around them drawn when `c` is given. Returns the section's own height
/// (the export button right after the rest); the button sits at the
/// area's bottom when the area is taller (`mt-auto`).
pub(super) fn walk(
    section: Section,
    controls: &Controls,
    area: Rect,
    theme: &Theme,
    out: &mut Vec<Rect>,
    mut c: Option<&mut Compositor>,
) -> f32 {
    let t = tokens(theme);
    let (e, ink) = (&t.export, &t.ink);
    let scale = &theme.typography;
    let (head_style, note_style) = (scale.run(e.head), scale.run(e.note));
    let (value_style, title_style) = (scale.run(e.value), scale.run(e.line.title));
    let x = area.x + e.pad;
    let w = (area.w - e.pad * 2.0).max(0.0);
    out.clear();
    out.resize(controls.0.len(), Rect::new(0.0, 0.0, 0.0, 0.0));
    let mut y = area.y + e.pad;
    match section {
        Section::Images => {
            let add = 8;
            out[add] = Rect::new(
                x + w - e.tool,
                y + (e.head_h - e.tool) / 2.0,
                e.tool,
                e.tool,
            );
            if let Some(c) = c.as_deref_mut() {
                text(
                    c,
                    "Export images",
                    &head_style,
                    x,
                    middle(&head_style, y, e.head_h),
                    w - e.tool,
                    ink.ink,
                );
            }
            y += e.head_h + e.head_bottom;
            for row in 0..2 {
                let k = row * 4;
                let h = (0..4)
                    .map(|j| height(controls, k + j, theme))
                    .fold(0.0, f32::max);
                // The format tabs keep their labels whole; the color
                // profile's select gives way first.
                let tabs_min = match &controls.0[k + 2].kind {
                    Kind::Tabs(tb, _) => tb.preferred_size(theme).0,
                    _ => 0.0,
                };
                let fixed = e.size_w + e.tool + e.tool_gap + e.row_gap * 3.0;
                let color_w = e.color_w.min((w - fixed - tabs_min).max(0.0));
                let place = |out: &mut Vec<Rect>, j: usize, at: f32, top: f32, cw: f32| {
                    let ch = height(controls, k + j, theme);
                    out[k + j] = Rect::new(at, top + (h - ch) / 2.0, cw, ch);
                };
                if color_w >= e.size_w {
                    // One line: the scale, the profile, the format, the
                    // remove.
                    let tabs_w = (w - fixed - color_w).max(0.0);
                    place(out, 0, x, y, e.size_w);
                    place(out, 1, x + e.size_w + e.row_gap, y, color_w);
                    let tx = x + e.size_w + color_w + e.row_gap * 2.0;
                    place(out, 2, tx, y, tabs_w);
                } else {
                    // Short of room (the profile would be narrower than
                    // the scale): the scale and the profile, then the
                    // format beside the remove.
                    let color_w = (w - e.size_w - e.row_gap).max(0.0);
                    place(out, 0, x, y, e.size_w);
                    place(out, 1, x + e.size_w + e.row_gap, y, color_w);
                    y += h + e.row_gap;
                    let tabs_w = (w - e.tool - e.tool_gap - e.row_gap).max(0.0);
                    place(out, 2, x, y, tabs_w);
                }
                out[k + 3] = Rect::new(x + w - e.tool, y + (h - e.tool) / 2.0, e.tool, e.tool);
                y += h + if row == 0 { e.row_gap } else { 0.0 };
            }
            y += e.pad;
            if let Some(c) = c.as_deref_mut() {
                hairline(c, area.x, y, area.w, ink.rule_w, ink.rule);
            }
            y += ink.rule_w + e.pad;
            let head = head_style.line_height;
            if let Some(c) = c.as_deref_mut() {
                text(c, "Compression", &head_style, x, y, w / 2.0, ink.ink);
                let dims_y = middle(&note_style, y, head);
                text_end(
                    c,
                    DIMENSIONS,
                    &note_style,
                    x + w,
                    dims_y,
                    w / 2.0,
                    ink.muted,
                );
            }
            y += head + e.note_gap;
            let value = Rect::new(x + w - e.value_w, y, e.value_w, e.range_h);
            out[9] = Rect::new(x, y, (value.x - e.row_gap - x).max(0.0), e.range_h);
            if let Some(c) = c.as_deref_mut() {
                stroke(c, value, e.value_radius, ink.edge, ink.rule_w);
                let label = format!("{}%", compression(controls).round());
                let inner = e.value_icon + e.row_gap + text_w(&label, &value_style);
                let ix = value.x + (value.w - inner) / 2.0;
                let icon = Rect::new(ix, value.y, e.value_icon, value.h);
                glyph(c, "chevrons-left-right", e.value_icon, ink.icon, icon);
                let tx = icon.right() + e.row_gap;
                text(
                    c,
                    &label,
                    &value_style,
                    tx,
                    middle(&value_style, value.y, value.h),
                    value.right() - tx,
                    ink.ink,
                );
            }
            y += e.range_h + e.pad;
        }
        Section::Video | Section::Object => {
            let lines: &[(&str, usize)] = if section == Section::Video {
                &[
                    ("Camera", 0),
                    ("Format", 1),
                    ("Frame rate", 2),
                    ("Resolution", 3),
                ]
            } else {
                &[("Format", 0), ("Camera", 1), ("Material", 2)]
            };
            let l = &e.line;
            // The control keeps the kit's width while the longest title
            // still fits beside it; in less room it gives way.
            let longest = lines
                .iter()
                .map(|(t, _)| text_w(t, &title_style))
                .fold(0.0, f32::max);
            let control_w = l.control_w.min((w - l.title_gap - longest).max(0.0));
            for (n, (title, k)) in lines.iter().enumerate() {
                if n > 0 {
                    y += l.gap;
                }
                let ch = height(controls, *k, theme);
                out[*k] = Rect::new(x + w - control_w, y, control_w, ch);
                let mut h = ch.max(l.title_top + title_style.line_height);
                if let Some(c) = c.as_deref_mut() {
                    let room = w - control_w - l.title_gap;
                    text(c, title, &title_style, x, y + l.title_top, room, ink.ink);
                }
                if section == Section::Video && *k == 3 {
                    let ny = y + ch + e.note_top;
                    if let Some(c) = c.as_deref_mut() {
                        text_end(c, DIMENSIONS, &note_style, x + w, ny, control_w, ink.muted);
                    }
                    h = h.max(ch + e.note_top + note_style.line_height);
                }
                y += h;
            }
            y += l.bottom;
        }
    }
    let last = controls.0.len() - 1;
    let bh = height(controls, last, theme);
    let estimate = if section == Section::Video {
        note_style.line_height + e.note_gap
    } else {
        0.0
    };
    let natural = y + estimate - area.y + bh + e.pad;
    let by = (area.bottom() - e.pad - bh).max(y + estimate);
    out[last] = Rect::new(x, by, w, bh);
    if section == Section::Video
        && let Some(c) = c
    {
        let ny = by - e.note_gap - note_style.line_height;
        text_centered(c, ESTIMATE, &note_style, x + w / 2.0, ny, w, ink.muted);
    }
    natural
}
