//! Minimal perf HUD: an overlay the compositor draws itself, on its own
//! high-z layer, anchored to the top-right corner.
//!
//! Pure compositor calls, so it is testable headlessly. Text uses one
//! [`TextStyle`] (JetBrains Mono) shared by `TextMeasurer::measure_styled`
//! and `TextNodeKey::from_style`, per the one-style-measure-and-draw ADR.
//! The layer is created lazily on the first draw and removed by
//! [`PerfHud::clear`], so a disabled HUD costs no layer texture.

use crate::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::text::{TextMeasurer, TextStyle};

use super::{PerfSnapshot, mb};

const FONT_SIZE: f32 = 11.0;
const PAD: f32 = 8.0;
const MARGIN: f32 = 12.0;
const TEXT: [f32; 4] = [0.92, 0.93, 0.95, 1.0];
const PANEL: [f32; 4] = [0.05, 0.06, 0.08, 0.85];
const BORDER: [f32; 4] = [1.0, 1.0, 1.0, 0.08];

pub struct PerfHud {
    layer: Option<LayerId>,
}

impl Default for PerfHud {
    fn default() -> Self {
        Self::new()
    }
}

impl PerfHud {
    /// Above any app layer; apps stay well below this.
    pub const Z_ORDER: i32 = 1_000_000;

    pub fn new() -> Self {
        Self { layer: None }
    }

    /// The overlay layer, once created by a draw.
    pub fn layer(&self) -> Option<LayerId> {
        self.layer
    }

    /// Push this frame's overlay. Call after the app scene is built (the
    /// compositor clears all layers in `begin_frame`) and before
    /// `resolve`. `viewport_w` is the scene-coordinate width used to
    /// anchor the panel to the right edge.
    pub fn draw(&mut self, c: &mut Compositor, snapshot: &PerfSnapshot, viewport_w: f32) {
        let layer = match self.layer {
            Some(id) => id,
            None => {
                let id = c.create_layer(Self::Z_ORDER);
                self.layer = Some(id);
                id
            }
        };

        let style = TextStyle::new(FONT_SIZE).with_family("JetBrains Mono");
        let lines = hud_lines(snapshot);
        let text_w = lines
            .iter()
            .map(|l| TextMeasurer::measure_styled(l, &style, None).0)
            .fold(0.0f32, f32::max);
        let panel_w = text_w + PAD * 2.0;
        let panel_h = lines.len() as f32 * style.line_height + PAD * 2.0;
        let x = (viewport_w - panel_w - MARGIN).max(0.0);
        let y = MARGIN;

        c.push_to_layer(
            layer,
            SceneNode::RoundedRect {
                x,
                y,
                w: panel_w,
                h: panel_h,
                color: PANEL,
                corner_radius: 6.0,
                border_width: 1.0,
                border_color: BORDER,
            },
        );
        for (i, line) in lines.iter().enumerate() {
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(line, &style, None),
                    x: x + PAD,
                    y: y + PAD + i as f32 * style.line_height,
                    color: TEXT,
                },
            );
        }
    }

    /// Remove the overlay layer (frees its full-window texture). No-op
    /// when the HUD never drew.
    pub fn clear(&mut self, c: &mut Compositor) {
        if let Some(id) = self.layer.take() {
            c.remove_layer(id);
        }
    }
}

fn hud_lines(s: &PerfSnapshot) -> Vec<String> {
    let mut lines = vec![
        format!("fps {:>6.1}  p99 {:>6.2} ms", s.fps, s.dt_p99_ms),
        format!(
            "enc {:>6} us res {:>5} us",
            s.encode_avg_micros, s.resolve_avg_micros
        ),
        format!("draw {:>5}  glyphs {:>6}", s.draw_calls, s.glyphs),
        match s.memory.process_rss_bytes {
            Some(rss) => format!(
                "gpu {:>5.1} MB  rss {:>4.0} MB",
                mb(s.memory.gpu_total_bytes()),
                mb(rss)
            ),
            None => format!("gpu {:>5.1} MB", mb(s.memory.gpu_total_bytes())),
        },
    ];
    if let Some(gpu) = s.gpu_micros {
        lines.push(format!("gpu time {gpu:>8} us"));
    }
    lines
}
