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

mod shape;
mod text;
mod write;

#[cfg(test)]
mod tests;

use std::fmt;
use std::path::Path;

use crate::compositor::{Compositor, Layer, LayerEffect, SceneNode};
use crate::text::TextSystem;

use super::{HeadlessError, Viewport};
use shape::{Bounds, rounded_rect};
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
            SceneNode::GradientRect {
                x,
                y,
                w,
                h,
                color,
                color2,
                angle_deg,
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
                shape::gradient_rect(
                    svg,
                    bounds,
                    *color,
                    *color2,
                    *angle_deg,
                    *border_width,
                    *border_color,
                    &at("gradient rect"),
                );
            }
            SceneNode::Shadow {
                x,
                y,
                w,
                h,
                corner_radius,
                blur_radius,
                offset,
                color,
                inset,
            } => {
                let bounds = Bounds {
                    x: *x,
                    y: *y,
                    w: *w,
                    h: *h,
                    radius: *corner_radius,
                };
                shape::shadow(
                    svg,
                    bounds,
                    *blur_radius,
                    *offset,
                    *color,
                    *inset,
                    viewport,
                    &at("shadow"),
                );
            }
            SceneNode::Path { data } => shape::path(svg, data, &at("path")),
            SceneNode::Image {
                x,
                y,
                w,
                h,
                image,
                corner_radius,
            } => {
                if outside(viewport, *x, *y, *w, *h) {
                    continue;
                }
                // The store keeps no pixels once they are on the GPU, so
                // there is nothing to embed; the place is marked.
                svg.element(&format!(
                    "<!-- image {}x{} at {} {} ({}x{}, radius {}): not embedded -->",
                    image.width,
                    image.height,
                    num(*x),
                    num(*y),
                    num(*w),
                    num(*h),
                    num(*corner_radius)
                ));
                svg.note(
                    &at("image"),
                    &format!(
                        "image {}x{} left out: the image store keeps no pixels after the \
                         upload, so there is nothing to embed",
                        image.width, image.height
                    ),
                );
            }
            SceneNode::BackdropBlur {
                x, y, w, h, sigma, ..
            } => {
                if outside(viewport, *x, *y, *w, *h) {
                    continue;
                }
                svg.element(&format!(
                    "<!-- backdrop blur at {} {} ({}x{}, sigma {}): left out -->",
                    num(*x),
                    num(*y),
                    num(*w),
                    num(*h),
                    num(*sigma)
                ));
                svg.note(
                    &at("backdrop blur"),
                    &format!(
                        "backdrop blur (sigma {}) left out: what lies below stays sharp \
                         under the glass",
                        num(*sigma)
                    ),
                );
            }
        }
    }
    // A push without its pop ends with the layer, as the scissor does.
    while clips.pop() {
        svg.close("</g>");
    }
    svg.close("</g>");
}
