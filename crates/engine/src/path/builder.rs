//! Ergonomic path construction via fluent API.

use lyon_path::math::point;

use super::tessellation;
use super::types::{PathCommand, TessellatedPath};

// ---------------------------------------------------------------------------
// PathBuilder -- ergonomic path construction
// ---------------------------------------------------------------------------

pub struct PathBuilder {
    pub(crate) builder: lyon_path::path::Builder,
    pub(crate) commands: Vec<PathCommand>,
    /// Whether a sub-path is currently open (started via `move_to`, not yet
    /// ended via `close`/`end_open`). Lyon's `build()` panics on an open
    /// sub-path, so tessellation finalizes it first via `finish_open`.
    open: bool,
}

impl Default for PathBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PathBuilder {
    pub fn new() -> Self {
        Self {
            builder: lyon_path::Path::builder(),
            commands: Vec::new(),
            open: false,
        }
    }

    pub fn move_to(mut self, x: f32, y: f32) -> Self {
        self.builder.begin(point(x, y));
        self.open = true;
        self.commands
            .push(PathCommand::MoveTo(x.to_bits(), y.to_bits()));
        self
    }

    pub fn line_to(mut self, x: f32, y: f32) -> Self {
        self.builder.line_to(point(x, y));
        self.commands
            .push(PathCommand::LineTo(x.to_bits(), y.to_bits()));
        self
    }

    pub fn quadratic_bezier_to(mut self, ctrl: [f32; 2], to: [f32; 2]) -> Self {
        self.builder
            .quadratic_bezier_to(point(ctrl[0], ctrl[1]), point(to[0], to[1]));
        self.commands.push(PathCommand::QuadTo(
            ctrl[0].to_bits(),
            ctrl[1].to_bits(),
            to[0].to_bits(),
            to[1].to_bits(),
        ));
        self
    }

    pub fn cubic_bezier_to(mut self, ctrl1: [f32; 2], ctrl2: [f32; 2], to: [f32; 2]) -> Self {
        self.builder.cubic_bezier_to(
            point(ctrl1[0], ctrl1[1]),
            point(ctrl2[0], ctrl2[1]),
            point(to[0], to[1]),
        );
        self.commands.push(PathCommand::CubicTo(
            ctrl1[0].to_bits(),
            ctrl1[1].to_bits(),
            ctrl2[0].to_bits(),
            ctrl2[1].to_bits(),
            to[0].to_bits(),
            to[1].to_bits(),
        ));
        self
    }

    pub fn close(mut self) -> Self {
        self.builder.end(true);
        self.open = false;
        self.commands.push(PathCommand::Close);
        self
    }

    /// End the current sub-path without closing it (for open strokes).
    pub fn end_open(mut self) -> Self {
        self.builder.end(false);
        self.open = false;
        self
    }

    /// Finalize a still-open sub-path so the underlying Lyon builder can be
    /// `build()`-ed safely. Lyon aborts (non-unwinding panic on macOS, inside
    /// the objc2 draw callback) if a sub-path was started with `begin()` but
    /// never ended; a stroke over an open polyline — the common case for line
    /// charts — would otherwise crash the whole frame. Idempotent.
    pub(crate) fn finish_open(&mut self) {
        if self.open {
            self.builder.end(false);
            self.open = false;
        }
    }

    /// Convenience: build a circle path.
    pub fn circle(cx: f32, cy: f32, r: f32) -> Self {
        // Approximate circle with 4 cubic beziers (standard kappa = 0.5522847498)
        let k = r * 0.552_284_8;
        Self::new()
            .move_to(cx + r, cy)
            .cubic_bezier_to([cx + r, cy + k], [cx + k, cy + r], [cx, cy + r])
            .cubic_bezier_to([cx - k, cy + r], [cx - r, cy + k], [cx - r, cy])
            .cubic_bezier_to([cx - r, cy - k], [cx - k, cy - r], [cx, cy - r])
            .cubic_bezier_to([cx + k, cy - r], [cx + r, cy - k], [cx + r, cy])
            .close()
    }

    /// Convenience: build a rounded rectangle path.
    pub fn rounded_rect(x: f32, y: f32, w: f32, h: f32, radius: f32) -> Self {
        let r = radius.min(w / 2.0).min(h / 2.0);
        if r <= 0.0 {
            return Self::new()
                .move_to(x, y)
                .line_to(x + w, y)
                .line_to(x + w, y + h)
                .line_to(x, y + h)
                .close();
        }
        let k = r * 0.552_284_8;
        Self::new()
            .move_to(x + r, y)
            .line_to(x + w - r, y)
            .cubic_bezier_to([x + w - r + k, y], [x + w, y + r - k], [x + w, y + r])
            .line_to(x + w, y + h - r)
            .cubic_bezier_to(
                [x + w, y + h - r + k],
                [x + w - r + k, y + h],
                [x + w - r, y + h],
            )
            .line_to(x + r, y + h)
            .cubic_bezier_to([x + r - k, y + h], [x, y + h - r + k], [x, y + h - r])
            .line_to(x, y + r)
            .cubic_bezier_to([x, y + r - k], [x + r - k, y], [x + r, y])
            .close()
    }

    /// Convenience: build an ellipse path.
    pub fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32) -> Self {
        let kx = rx * 0.552_284_8;
        let ky = ry * 0.552_284_8;
        Self::new()
            .move_to(cx + rx, cy)
            .cubic_bezier_to([cx + rx, cy + ky], [cx + kx, cy + ry], [cx, cy + ry])
            .cubic_bezier_to([cx - kx, cy + ry], [cx - rx, cy + ky], [cx - rx, cy])
            .cubic_bezier_to([cx - rx, cy - ky], [cx - kx, cy - ry], [cx, cy - ry])
            .cubic_bezier_to([cx + kx, cy - ry], [cx + rx, cy - ky], [cx + rx, cy])
            .close()
    }

    /// Fill the path with the given color, using the configured default
    /// tolerance (see [`super::set_default_tolerance`]).
    pub fn fill(self, color: [f32; 4]) -> TessellatedPath {
        self.fill_with_tolerance(color, super::default_tolerance())
    }

    /// Fill the path with custom tolerance (lower = more vertices, higher quality).
    pub fn fill_with_tolerance(self, color: [f32; 4], tolerance: f32) -> TessellatedPath {
        tessellation::fill(self, color, tolerance)
    }

    /// Stroke the path with the given color and line width, using the
    /// configured default tolerance.
    pub fn stroke(self, color: [f32; 4], line_width: f32) -> TessellatedPath {
        self.stroke_with_tolerance(color, line_width, super::default_tolerance())
    }

    /// Stroke the path with custom tolerance.
    pub fn stroke_with_tolerance(
        self,
        color: [f32; 4],
        line_width: f32,
        tolerance: f32,
    ) -> TessellatedPath {
        tessellation::stroke(self, color, line_width, tolerance)
    }

    /// Stroke with round caps and round joins (Lucide icon style), using the
    /// configured default tolerance.
    pub fn stroke_round(self, color: [f32; 4], line_width: f32) -> TessellatedPath {
        self.stroke_round_with_tolerance(color, line_width, super::default_tolerance())
    }

    /// Round-cap/round-join stroke with custom tolerance.
    pub fn stroke_round_with_tolerance(
        self,
        color: [f32; 4],
        line_width: f32,
        tolerance: f32,
    ) -> TessellatedPath {
        tessellation::stroke_round(self, color, line_width, tolerance)
    }
}
