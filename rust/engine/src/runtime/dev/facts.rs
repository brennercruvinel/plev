//! What the Inspect view says about the thing chosen, a name and a value
//! a line: a control (the app's semantic snapshot and the compositor's
//! binding) or a scene node (what the frame draws there: its kind, its
//! paint, its type, its place).
//!
//! Nothing here draws. The view lays the lines out; a line can carry a
//! color to show beside its value, and a press of its own (the control a
//! node sits in).

use accord::{SemId, Snapshot};

use super::Act;
use super::paint::time;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::runtime::perf::FrameReport;

/// A line about the thing chosen.
pub(super) struct Fact {
    pub label: &'static str,
    pub value: String,
    /// The value is code: an id, a source position.
    pub code: bool,
    /// A color the value names, shown beside it.
    pub swatch: Option<[f32; 4]>,
    /// What a press on the line does.
    pub act: Option<Act>,
}

fn fact(label: &'static str, value: impl Into<String>) -> Fact {
    Fact {
        label,
        value: value.into(),
        code: false,
        swatch: None,
        act: None,
    }
}

fn code(label: &'static str, value: impl Into<String>) -> Fact {
    Fact {
        code: true,
        ..fact(label, value)
    }
}

/// A color as the page writes it: hex, with its opacity when it has one.
fn color(label: &'static str, rgba: [f32; 4]) -> Fact {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let mut value = format!(
        "#{:02X}{:02X}{:02X}",
        byte(rgba[0]),
        byte(rgba[1]),
        byte(rgba[2])
    );
    if rgba[3] < 0.995 {
        value = format!("{value} at {:.0}%", rgba[3] * 100.0);
    }
    Fact {
        swatch: Some(rgba),
        ..code(label, value)
    }
}

/// A rect's size and place, the two lines every thing has.
fn place(out: &mut Vec<Fact>, [x, y, w, h]: [f32; 4]) {
    out.push(fact("size", format!("{w:.0} × {h:.0}")));
    out.push(fact("position", format!("x {x:.0} · y {y:.0}")));
}

/// What a control is called over its lines: its role and its label.
pub(super) fn control_name(id: &SemId, snapshot: Option<&Snapshot>) -> String {
    match snapshot.and_then(|s| s.get(id)) {
        Some(node) if node.label.is_empty() => format!("{:?}", node.role),
        Some(node) => format!("{:?} · {}", node.role, node.label),
        None => "Control".to_owned(),
    }
}

/// The lines about the control `id`.
pub(super) fn control(
    id: &SemId,
    c: &Compositor,
    snapshot: Option<&Snapshot>,
    shown: Option<&FrameReport>,
    dev: LayerId,
) -> Vec<Fact> {
    let mut out = vec![code("id", id.to_string())];
    match snapshot.and_then(|s| s.get(id)) {
        Some(node) => {
            if let Some(description) = &node.description {
                out.push(fact("description", description.clone()));
            }
            if !node.state.is_empty() {
                let state: Vec<String> = node
                    .state
                    .iter()
                    .map(|(key, value)| format!("{key} = {value:?}"))
                    .collect();
                out.push(fact("state", state.join(" · ")));
            }
            if !node.actions.is_empty() {
                let actions: Vec<String> =
                    node.actions.iter().map(|a| a.name.to_string()).collect();
                out.push(fact("actions", actions.join(" · ")));
            }
            let mut flags = Vec::new();
            if !node.enabled {
                flags.push("disabled");
            }
            if node.inert {
                flags.push("inert");
            }
            if node.focusable {
                flags.push("focusable");
            }
            if !flags.is_empty() {
                out.push(fact("flags", flags.join(" · ")));
            }
            if let Some(source) = &node.source {
                let at = format!("{}:{}:{}", source.file, source.line, source.column);
                out.push(code("source", at));
            }
        }
        None => out.push(fact("meaning", "not in the app's snapshot")),
    }
    match c.binding(id).copied() {
        Some(binding) => {
            place(&mut out, binding.rect);
            out.push(fact("order", format!("{} in its layer", binding.order)));
            out.push(fact("layer", layer_line(binding.layer, shown, dev)));
        }
        None => out.push(fact("position", "not bound in the frame on screen")),
    }
    out
}

/// The lines about the scene node `node`, the `index`th of `layer`.
/// `owner` is the control it sits in, when one is bound there.
pub(super) fn node(
    node: &SceneNode,
    layer: LayerId,
    index: usize,
    c: &Compositor,
    owner: Option<&SemId>,
    shown: Option<&FrameReport>,
    dev: LayerId,
) -> Vec<Fact> {
    let mut out = Vec::new();
    match node {
        SceneNode::Text {
            key, color: ink, ..
        } => {
            out.push(fact("text", format!("\"{}\"", key.text)));
            let (size, line) = (
                f32::from_bits(key.font_size_bits),
                f32::from_bits(key.line_height_bits),
            );
            let family = key.font_family.as_deref().unwrap_or("the theme's face");
            out.push(fact(
                "type",
                format!(
                    "{size:.0} px on {line:.0} · weight {} · {family}",
                    key.font_weight
                ),
            ));
            let tracking = f32::from_bits(key.letter_spacing_bits);
            if tracking != 0.0 {
                out.push(fact("tracking", format!("{tracking:.2} px")));
            }
            if let Some(width) = key.max_width_bits.map(f32::from_bits) {
                out.push(fact("wraps at", format!("{width:.0} px")));
            }
            out.push(color("color", *ink));
        }
        SceneNode::Rect { color: fill, .. } => out.push(color("fill", *fill)),
        SceneNode::RoundedRect {
            color: fill,
            corner_radius,
            border_width,
            border_color,
            ..
        } => {
            out.push(color("fill", *fill));
            out.push(fact("radius", format!("{corner_radius:.0} px")));
            if *border_width > 0.0 {
                out.push(color("border", *border_color));
                out.push(fact("border width", format!("{border_width:.1} px")));
            }
        }
        SceneNode::GradientRect {
            color: from,
            color2: to,
            angle_deg,
            corner_radius,
            ..
        } => {
            out.push(color("from", *from));
            out.push(color("to", *to));
            out.push(fact("angle", format!("{angle_deg:.0} deg")));
            out.push(fact("radius", format!("{corner_radius:.0} px")));
        }
        SceneNode::Path { data } => {
            out.push(fact(
                "mesh",
                format!(
                    "{} vertices · {} triangles",
                    data.vertices.len(),
                    data.indices.len() / 3
                ),
            ));
            if let Some(first) = data.vertices.first() {
                out.push(color("color", first.color));
            }
        }
        SceneNode::Image {
            image,
            corner_radius,
            ..
        } => {
            out.push(fact(
                "image",
                format!(
                    "#{} · {} × {} in the atlas",
                    image.id, image.width, image.height
                ),
            ));
            out.push(fact("radius", format!("{corner_radius:.0} px")));
        }
        SceneNode::BackdropBlur {
            corner_radius,
            sigma,
            ..
        } => {
            out.push(fact("blur", format!("sigma {sigma:.1} px")));
            out.push(fact("radius", format!("{corner_radius:.0} px")));
        }
        SceneNode::Shadow { .. } | SceneNode::PushClip { .. } | SceneNode::PopClip => {}
    }
    if let Some(bounds) = node.bounds() {
        place(&mut out, bounds);
        // what its clip leaves of it, when that is not all of it
        match c.node_rect(layer, index) {
            Some(shows) if shows != bounds => out.push(fact(
                "shows",
                format!("{:.0} × {:.0} inside its clip", shows[2], shows[3]),
            )),
            Some(_) => {}
            None => out.push(fact("shows", "none of it: outside its clip")),
        }
    }
    let of = c.layer(layer).map_or(0, |l| l.nodes().len());
    out.push(fact("node", format!("{} of {of} in its layer", index + 1)));
    out.push(fact("layer", layer_line(layer, shown, dev)));
    match owner {
        Some(id) => out.push(Fact {
            act: Some(Act::Control(id.clone())),
            ..code("control", id.to_string())
        }),
        None => out.push(fact("control", "none bound here: the page gives it no id")),
    }
    out
}

/// What a layer did in the frame shown.
fn layer_line(id: LayerId, shown: Option<&FrameReport>, dev: LayerId) -> String {
    let name = if id == dev {
        format!("{} (dev mode)", id.0)
    } else {
        id.0.to_string()
    };
    let Some(frame) = shown else {
        return name;
    };
    let Some(layer) = frame.layers.iter().find(|l| l.id == id) else {
        return format!("{name} · not in frame {}", frame.index);
    };
    let state = if layer.redrawn { "redrawn" } else { "reused" };
    let gpu: u64 = frame
        .gpu
        .iter()
        .flat_map(|g| g.passes.iter())
        .filter(|pass| pass.layer == id)
        .filter_map(|pass| pass.nanos)
        .sum();
    let mut line = format!(
        "{name} · z {} · {state} in frame {} · {} nodes · {} draws",
        layer.z_order, frame.index, layer.nodes, layer.draws
    );
    if gpu > 0 {
        line = format!("{line} · gpu {}", time(gpu));
    }
    line
}
