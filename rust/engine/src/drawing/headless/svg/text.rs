//! A text run as glyph outlines.
//!
//! The glyphs come from `TextSystem::shaped_glyphs`, the shaping the frame
//! drew from; their outlines come from swash through cosmic-text
//! (`SwashCache::get_outline_commands`), the same scaler that rasterized
//! them, at the glyph's own size. Each run is a `<g aria-label="<string>">`
//! with the fill and one `<path>` per glyph, so the text stays readable to
//! a tool that reads the label and looks right to one that draws.

use std::collections::HashMap;
use std::fmt::Write as _;

use cosmic_text::{CacheKey, Command, FontSystem, SwashCache};

use crate::drawing::compositor::TextNodeKey;
use crate::drawing::text::TextSystem;

use super::write::{Svg, escape, fill, num};

/// Outline commands by glyph key, kept for the whole export: a glyph drawn
/// many times is scaled once. Each miss scales through a fresh
/// `SwashCache`, for the reason `text::atlas::rasterize_and_upload` gives:
/// a shared swash `ScaleContext` keeps per-font state across calls, and
/// interleaving sizes of one face hands back a stale outline (the
/// sidebar of the Card page came out in the wrong glyphs before this).
pub(super) struct Outlines {
    cache: HashMap<CacheKey, Option<Box<[Command]>>>,
}

impl Outlines {
    pub(super) fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// The `d` of `key`'s outline placed with its origin at `(ox, oy)`;
    /// `Some("")` for a glyph with no contour (a space), `None` when the
    /// face or the glyph is missing.
    fn path(&mut self, fonts: &mut FontSystem, key: CacheKey, ox: f32, oy: f32) -> Option<String> {
        let commands = self
            .cache
            .entry(key)
            .or_insert_with(|| SwashCache::new().get_outline_commands_uncached(fonts, key))
            .as_deref()?;
        let mut d = String::new();
        // Outlines are y-up around the origin; the scene is y-down.
        let x = |v: f32| num(ox + v);
        let y = |v: f32| num(oy - v);
        for command in commands {
            match command {
                Command::MoveTo(p) => {
                    let _ = write!(d, "M{} {}", x(p.x), y(p.y));
                }
                Command::LineTo(p) => {
                    let _ = write!(d, "L{} {}", x(p.x), y(p.y));
                }
                Command::QuadTo(c, p) => {
                    let _ = write!(d, "Q{} {} {} {}", x(c.x), y(c.y), x(p.x), y(p.y));
                }
                Command::CurveTo(c1, c2, p) => {
                    let _ = write!(
                        d,
                        "C{} {} {} {} {} {}",
                        x(c1.x),
                        y(c1.y),
                        x(c2.x),
                        y(c2.y),
                        x(p.x),
                        y(p.y)
                    );
                }
                Command::Close => d.push('Z'),
            }
        }
        Some(d)
    }
}

/// One text node: the group with the run's string and fill, and a path
/// per glyph that has a contour. A glyph without one in its face is
/// reported and left out.
#[allow(clippy::too_many_arguments)] // the node's fields and the export's state
pub(super) fn run(
    svg: &mut Svg,
    text: &mut TextSystem,
    outlines: &mut Outlines,
    key: &TextNodeKey,
    x: f32,
    y: f32,
    color: [f32; 4],
    at: &str,
) {
    let glyphs = text.shaped_glyphs(key);
    if glyphs.is_empty() {
        return;
    }
    svg.open(&format!(
        "<g aria-label=\"{}\"{}>",
        escape(&key.text),
        fill(color)
    ));
    for glyph in glyphs {
        match outlines.path(&mut text.font_system, glyph.key, x + glyph.x, y + glyph.y) {
            Some(d) if d.is_empty() => {}
            Some(d) => svg.element(&format!("<path d=\"{d}\"/>")),
            None => svg.note(
                at,
                &format!(
                    "glyph {} of {:?} has no outline in its face; left out",
                    glyph.key.glyph_id,
                    key.text.get(..40.min(key.text.len())).unwrap_or(&key.text)
                ),
            ),
        }
    }
    svg.close("</g>");
}
