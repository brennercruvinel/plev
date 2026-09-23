//! A frame as SVG, with every approximation declared.
//!
//! [`export`] walks the compositor's scene the way the frame draws it: the
//! visible layers in composite order (z-order, stable), each layer's nodes
//! in push order, and writes one SVG element per node. What has an exact
//! equivalent is written as it is: a rect; a rounded rect with its border
//! as an inset stroke; a tessellated path as the outline of its triangles;
//! a text run as the outlines of the glyphs the frame shaped
//! (`TextSystem::shaped_glyphs`, the one shaping path), inside a
//! `<g aria-label>` holding the string. What has no exact equivalent is
//! approximated or left out, and every such case enters the report
//! ([`Approximation`]), one entry per node, the way the svg importer
//! reports what it cannot carry in the other direction (`crates/svg`).
//!
//! Coordinates are logical pixels: the `viewBox` is the viewport's logical
//! size and `width` and `height` its physical size, so the file renders at
//! the frame's pixel size. Colors are the scene's sRGB values, written as
//! `#rrggbb` plus `fill-opacity`; the SVG is sRGB, nothing is linearized.

mod text;
mod write;

#[cfg(test)]
mod tests;

use std::fmt;
use std::path::Path;

use crate::compositor::{Compositor, Layer, LayerEffect, SceneNode};
use crate::text::TextSystem;

use super::{HeadlessError, Viewport};
use text::Outlines;
use write::{Svg, fill, num};

/// One thing the SVG does not carry exactly: which node, and what the SVG
/// holds in its place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Approximation {
    /// The node, as `layer <id>, node <index> (<kind>)`.
    pub node: String,
    pub what: String,
}

impl fmt::Display for Approximation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.node, self.what)
    }
}

/// The SVG text and the report of what in it is not the frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SvgExport {
    pub svg: String,
    pub report: Vec<Approximation>,
}

impl SvgExport {
    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), HeadlessError> {
        std::fs::write(path, &self.svg)?;
        Ok(())
    }

    /// The report, one line per item, each line ended; empty when the SVG
    /// is exact.
    pub fn report_text(&self) -> String {
        self.report.iter().map(|a| format!("{a}\n")).collect()
    }
}

/// The scene in `compositor` as SVG. Glyph outlines come from `text`, the
/// text system that drew (or will draw) the frame. `background` is the
/// page color in sRGB (`Color::to_array`), the clear the frame starts
/// from; `None` leaves the page transparent.
pub fn export(
    compositor: &Compositor,
    text: &mut TextSystem,
    viewport: Viewport,
    background: Option<[f32; 4]>,
) -> Result<SvgExport, HeadlessError> {
    viewport.check()?;
    let mut svg = Svg::new();
    if let Some(color) = background {
        svg.element(&format!(
            "<rect width=\"{}\" height=\"{}\"{}/>",
            num(viewport.width),
            num(viewport.height),
            fill(color)
        ));
    }
    // The composite order: `Compositor::resolve_scene` sorts by z-order
    // with a stable sort, so a sorted copy of the visible layers is the
    // order the frame composited them in.
    let mut layers: Vec<&Layer> = compositor.layers().iter().filter(|l| l.visible).collect();
    layers.sort_by_key(|l| l.z_order);
    let mut outlines = Outlines::new();
    for layer in layers {
        write_layer(&mut svg, layer, text, &mut outlines, viewport);
    }
    let (svg, report) = svg.finish(viewport);
    Ok(SvgExport { svg, report })
}

/// Whether an axis-aligned box lies entirely outside the viewport, the
/// cull `Layer::build_geometry` applies: what the frame never drew, the
/// SVG does not carry.
fn outside(viewport: Viewport, x: f32, y: f32, w: f32, h: f32) -> bool {
    x + w <= 0.0 || y + h <= 0.0 || x >= viewport.width || y >= viewport.height
}

/// The running intersection of the pushed clips, as `ClipStack` keeps it
/// for the frame, plus how many `<g clip-path>` are open for them.
#[derive(Default)]
struct Clips {
    stack: Vec<[f32; 4]>,
}

impl Clips {
    fn push(&mut self, rect: [f32; 4]) -> [f32; 4] {
        let combined = match self.stack.last() {
            Some(cur) => crate::compositor::intersect_rects(*cur, rect),
            None => rect,
        };
        self.stack.push(combined);
        combined
    }

    fn pop(&mut self) -> bool {
        self.stack.pop().is_some()
    }

    fn is_empty_clip(&self) -> bool {
        matches!(self.stack.last(), Some([_, _, w, h]) if *w <= 0.0 || *h <= 0.0)
    }
}

fn write_layer(
    svg: &mut Svg,
    layer: &Layer,
    text: &mut TextSystem,
    outlines: &mut Outlines,
    viewport: Viewport,
) {
    let layer_name = |what: &str| format!("layer {} ({what})", layer.id.0);
    let mut attrs = format!(" id=\"layer-{}\"", layer.id.0);
    if layer.opacity < 1.0 {
        attrs.push_str(&format!(" opacity=\"{}\"", num(layer.opacity)));
    }
    if let Some((x, y, w, h)) = layer.clip_rect {
        // The layer's scissor is in physical pixels.
        let s = viewport.scale;
        let rect = [x as f32 / s, y as f32 / s, w as f32 / s, h as f32 / s];
        let id = svg.clip_path(rect);
        attrs.push_str(&format!(" clip-path=\"url(#{id})\""));
        svg.note(
            &layer_name("clip rect"),
            "layer clip rect as a clipPath: the frame scissors at whole physical pixels",
        );
    }
    for effect in layer.effects() {
        let what = match effect {
            LayerEffect::Blur { sigma } => format!("layer blur (sigma {}) left out", num(*sigma)),
            LayerEffect::Shadow { sigma, .. } => {
                format!("layer shadow (sigma {}) left out", num(*sigma))
            }
        };
        svg.note(
            &layer_name("effect"),
            &format!("{what}: the shared frame path does not apply layer effects either"),
        );
    }
    svg.open(&format!("<g{attrs}>"));

    let mut clips = Clips::default();
    for (index, node) in layer.nodes().iter().enumerate() {
        let at = |kind: &str| format!("layer {}, node {index} ({kind})", layer.id.0);
        match node {
            SceneNode::PushClip { x, y, w, h } => {
                let rect = clips.push([*x, *y, *w, *h]);
                let id = svg.clip_path(rect);
                svg.open(&format!("<g clip-path=\"url(#{id})\">"));
                svg.note(
                    &at("clip"),
                    "clip as a clipPath at its logical bounds: the frame scissors at whole \
                     physical pixels",
                );
            }
            SceneNode::PopClip => {
                if clips.pop() {
                    svg.close("</g>");
                } else {
                    svg.note(
                        &at("pop clip"),
                        "PopClip without a PushClip: ignored, as the frame does",
                    );
                }
            }
            // Under an empty clip the frame draws nothing.
            _ if clips.is_empty_clip() => {}
            SceneNode::Rect { x, y, w, h, color } => {
                if outside(viewport, *x, *y, *w, *h) {
                    continue;
                }
                svg.element(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{}/>",
                    num(*x),
                    num(*y),
                    num(*w),
                    num(*h),
                    fill(*color)
                ));
            }
            SceneNode::RoundedRect {
                x,
                y,
                w,
                h,
                color,
                corner_radius,
                border_width,
                border_color,
            } => {
                if outside(viewport, *x, *y, *w, *h) {
                    continue;
                }
                let bounds = Bounds {
                    x: *x,
                    y: *y,
                    w: *w,
                    h: *h,
                    radius: *corner_radius,
                };
                rounded_rect(svg, bounds, &fill(*color), *border_width, *border_color);
            }
            SceneNode::Text { key, x, y, color } => {
                text::run(svg, text, outlines, key, *x, *y, *color, &at("text"));
            }
            SceneNode::GradientRect { .. } => svg.note(&at("gradient rect"), "not written yet"),
            SceneNode::Path { .. } => svg.note(&at("path"), "not written yet"),
            SceneNode::Image { .. } => svg.note(&at("image"), "not written yet"),
            SceneNode::BackdropBlur { .. } => svg.note(&at("backdrop blur"), "not written yet"),
            SceneNode::Shadow { .. } => svg.note(&at("shadow"), "not written yet"),
        }
    }
    // A push without its pop ends with the layer, as the scissor does.
    while clips.pop() {
        svg.close("</g>");
    }
    svg.close("</g>");
}

/// A rounded rect's box and corner radius.
#[derive(Clone, Copy)]
struct Bounds {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
}

/// A rounded rect as the SDF shader draws it: with a border, the ring of
/// `border_width` inside the outer edge holds the border color and only
/// the inset box holds the fill; without one, the fill reaches the edge.
/// `fill_attrs` is the fill (a color, or a gradient reference).
fn rounded_rect(svg: &mut Svg, b: Bounds, fill_attrs: &str, border_width: f32, border: [f32; 4]) {
    let rect = |x: f32, y: f32, w: f32, h: f32, radius: f32, paint: &str| {
        format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\"{paint}/>",
            num(x),
            num(y),
            num(w),
            num(h),
            num(radius.max(0.0))
        )
    };
    if border_width <= 0.0 {
        svg.element(&rect(b.x, b.y, b.w, b.h, b.radius, fill_attrs));
        return;
    }
    let bw = border_width;
    if b.w > 2.0 * bw && b.h > 2.0 * bw {
        svg.element(&rect(
            b.x + bw,
            b.y + bw,
            b.w - 2.0 * bw,
            b.h - 2.0 * bw,
            b.radius - bw,
            fill_attrs,
        ));
    }
    if border[3] > 0.0 && b.w > bw && b.h > bw {
        // A stroke centered half a border inside the edge covers exactly
        // the ring between the outer edge and the inset box.
        let stroke = format!(
            " fill=\"none\" stroke=\"{}\"{} stroke-width=\"{}\"",
            write::hex(border),
            write::opacity("stroke-opacity", border[3]),
            num(bw)
        );
        svg.element(&rect(
            b.x + bw / 2.0,
            b.y + bw / 2.0,
            b.w - bw,
            b.h - bw,
            b.radius - bw / 2.0,
            &stroke,
        ));
    }
}
