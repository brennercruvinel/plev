//! SVG path data (`d`) parser into a `PathBuilder`, with the elliptical
//! arc to cubic bezier conversion of SVG 1.1 appendix B.2.4.

use engine::path::PathBuilder;

/// Parse an SVG path `d` string into a [`PathBuilder`], scaling every
/// coordinate by `scale`. Supports the full command set Lucide uses
/// (M/L/H/V/C/S/Q/T/A/Z, absolute and relative, implicit repetition).
pub(super) fn parse_svg_path(d: &str, scale: f32) -> Option<PathBuilder> {
    let mut p = Tokens::new(d);
    let mut pb = PathBuilder::new();

    let mut cmd = b' ';
    // Current point, subpath start, and previous control point (for S/T
    // reflection), all in unscaled (viewBox) coordinates.
    let (mut cx, mut cy) = (0.0_f32, 0.0_f32);
    let (mut sx, mut sy) = (0.0_f32, 0.0_f32);
    let mut prev_cubic_ctrl: Option<(f32, f32)> = None;
    let mut prev_quad_ctrl: Option<(f32, f32)> = None;
    let mut open = false;

    macro_rules! ensure_open {
        () => {
            if !open {
                pb = pb.move_to(cx * scale, cy * scale);
                (sx, sy) = (cx, cy);
                open = true;
            }
        };
    }

    while let Some(next) = p.next_command_or_number() {
        match next {
            Token::Command(c) => {
                cmd = c;
                if cmd == b'Z' || cmd == b'z' {
                    if open {
                        pb = pb.close();
                        open = false;
                    }
                    (cx, cy) = (sx, sy);
                    prev_cubic_ctrl = None;
                    prev_quad_ctrl = None;
                    continue;
                }
            }
            Token::NumberStart => {
                // Implicit repetition: M becomes L (m becomes l) after the
                // first coordinate pair; everything else repeats as-is.
                cmd = match cmd {
                    b'M' => b'L',
                    b'm' => b'l',
                    c => c,
                }
            }
        }

        let rel = cmd.is_ascii_lowercase();
        match cmd.to_ascii_uppercase() {
            b'M' => {
                let (x, y) = (p.number()?, p.number()?);
                if open {
                    pb = pb.end_open();
                }
                (cx, cy) = if rel { (cx + x, cy + y) } else { (x, y) };
                pb = pb.move_to(cx * scale, cy * scale);
                (sx, sy) = (cx, cy);
                open = true;
                prev_cubic_ctrl = None;
                prev_quad_ctrl = None;
            }
            b'L' => {
                let (x, y) = (p.number()?, p.number()?);
                ensure_open!();
                (cx, cy) = if rel { (cx + x, cy + y) } else { (x, y) };
                pb = pb.line_to(cx * scale, cy * scale);
                prev_cubic_ctrl = None;
                prev_quad_ctrl = None;
            }
            b'H' => {
                let x = p.number()?;
                ensure_open!();
                cx = if rel { cx + x } else { x };
                pb = pb.line_to(cx * scale, cy * scale);
                prev_cubic_ctrl = None;
                prev_quad_ctrl = None;
            }
            b'V' => {
                let y = p.number()?;
                ensure_open!();
                cy = if rel { cy + y } else { y };
                pb = pb.line_to(cx * scale, cy * scale);
                prev_cubic_ctrl = None;
                prev_quad_ctrl = None;
            }
            b'C' | b'S' => {
                let (c1x, c1y) = if cmd.eq_ignore_ascii_case(&b'C') {
                    let (a, b) = (p.number()?, p.number()?);
                    if rel { (cx + a, cy + b) } else { (a, b) }
                } else {
                    // S: first control point reflects the previous one.
                    match prev_cubic_ctrl {
                        Some((px, py)) => (2.0 * cx - px, 2.0 * cy - py),
                        None => (cx, cy),
                    }
                };
                let (a, b, ex, ey) = (p.number()?, p.number()?, p.number()?, p.number()?);
                let (c2x, c2y) = if rel { (cx + a, cy + b) } else { (a, b) };
                let (nx, ny) = if rel { (cx + ex, cy + ey) } else { (ex, ey) };
                ensure_open!();
                pb = pb.cubic_bezier_to(
                    [c1x * scale, c1y * scale],
                    [c2x * scale, c2y * scale],
                    [nx * scale, ny * scale],
                );
                prev_cubic_ctrl = Some((c2x, c2y));
                prev_quad_ctrl = None;
                (cx, cy) = (nx, ny);
            }
            b'Q' | b'T' => {
                let (qx, qy) = if cmd.eq_ignore_ascii_case(&b'Q') {
                    let (a, b) = (p.number()?, p.number()?);
                    if rel { (cx + a, cy + b) } else { (a, b) }
                } else {
                    match prev_quad_ctrl {
                        Some((px, py)) => (2.0 * cx - px, 2.0 * cy - py),
                        None => (cx, cy),
                    }
                };
                let (ex, ey) = (p.number()?, p.number()?);
                let (nx, ny) = if rel { (cx + ex, cy + ey) } else { (ex, ey) };
                ensure_open!();
                pb = pb.quadratic_bezier_to([qx * scale, qy * scale], [nx * scale, ny * scale]);
                prev_quad_ctrl = Some((qx, qy));
                prev_cubic_ctrl = None;
                (cx, cy) = (nx, ny);
            }
            b'A' => {
                let (rx, ry) = (p.number()?, p.number()?);
                let rot = p.number()?;
                let large_arc = p.flag()?;
                let sweep = p.flag()?;
                let (ex, ey) = (p.number()?, p.number()?);
                let (nx, ny) = if rel { (cx + ex, cy + ey) } else { (ex, ey) };
                ensure_open!();
                pb = arc_to_cubics(pb, cx, cy, rx, ry, rot, large_arc, sweep, nx, ny, scale);
                prev_cubic_ctrl = None;
                prev_quad_ctrl = None;
                (cx, cy) = (nx, ny);
            }
            _ => return None, // unknown command — refuse the whole path
        }
    }

    if open {
        pb = pb.end_open();
    }
    Some(pb)
}

enum Token {
    Command(u8),
    /// Next token is a number — caller re-reads it via `number()`.
    NumberStart,
}

struct Tokens<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Tokens<'a> {
    fn new(d: &'a str) -> Self {
        Self {
            bytes: d.as_bytes(),
            pos: 0,
        }
    }

    fn skip_separators(&mut self) {
        while self.pos < self.bytes.len()
            && (self.bytes[self.pos].is_ascii_whitespace() || self.bytes[self.pos] == b',')
        {
            self.pos += 1;
        }
    }

    fn next_command_or_number(&mut self) -> Option<Token> {
        self.skip_separators();
        let b = *self.bytes.get(self.pos)?;
        if b.is_ascii_alphabetic() && b != b'e' && b != b'E' {
            self.pos += 1;
            Some(Token::Command(b))
        } else {
            Some(Token::NumberStart)
        }
    }

    /// Parse `[+-]?digits[.digits][(e|E)[+-]digits]`.
    fn number(&mut self) -> Option<f32> {
        self.skip_separators();
        let start = self.pos;
        if matches!(self.bytes.get(self.pos), Some(b'+') | Some(b'-')) {
            self.pos += 1;
        }
        let mut seen_digit = false;
        while self.bytes.get(self.pos).is_some_and(u8::is_ascii_digit) {
            self.pos += 1;
            seen_digit = true;
        }
        if self.bytes.get(self.pos) == Some(&b'.') {
            self.pos += 1;
            while self.bytes.get(self.pos).is_some_and(u8::is_ascii_digit) {
                self.pos += 1;
                seen_digit = true;
            }
        }
        if !seen_digit {
            self.pos = start;
            return None;
        }
        if matches!(self.bytes.get(self.pos), Some(b'e') | Some(b'E')) {
            let exp_start = self.pos;
            self.pos += 1;
            if matches!(self.bytes.get(self.pos), Some(b'+') | Some(b'-')) {
                self.pos += 1;
            }
            if self.bytes.get(self.pos).is_some_and(u8::is_ascii_digit) {
                while self.bytes.get(self.pos).is_some_and(u8::is_ascii_digit) {
                    self.pos += 1;
                }
            } else {
                self.pos = exp_start;
            }
        }
        std::str::from_utf8(&self.bytes[start..self.pos])
            .ok()?
            .parse()
            .ok()
    }

    /// Arc flags are single `0`/`1` chars that may not be separated from the
    /// following number (`"a1 1 0 01-.7"`), so they cannot use `number()`.
    fn flag(&mut self) -> Option<bool> {
        self.skip_separators();
        match self.bytes.get(self.pos)? {
            b'0' => {
                self.pos += 1;
                Some(false)
            }
            b'1' => {
                self.pos += 1;
                Some(true)
            }
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Elliptical arc -> cubic beziers (SVG 1.1 spec, appendix B.2.4)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn arc_to_cubics(
    mut pb: PathBuilder,
    x1: f32,
    y1: f32,
    rx: f32,
    ry: f32,
    rotation_deg: f32,
    large_arc: bool,
    sweep: bool,
    x2: f32,
    y2: f32,
    scale: f32,
) -> PathBuilder {
    if rx == 0.0 || ry == 0.0 || (x1 == x2 && y1 == y2) {
        return pb.line_to(x2 * scale, y2 * scale);
    }
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    let plev = rotation_deg.to_radians();
    let (sin_plev, cos_plev) = plev.sin_cos();

    // Step 1: midpoint coordinates.
    let dx2 = (x1 - x2) / 2.0;
    let dy2 = (y1 - y2) / 2.0;
    let x1p = cos_plev * dx2 + sin_plev * dy2;
    let y1p = -sin_plev * dx2 + cos_plev * dy2;

    // Correct out-of-range radii.
    let lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    if lambda > 1.0 {
        let l = lambda.sqrt();
        rx *= l;
        ry *= l;
    }

    // Step 2: center in primed coordinates.
    let rx_sq = rx * rx;
    let ry_sq = ry * ry;
    let num = rx_sq * ry_sq - rx_sq * y1p * y1p - ry_sq * x1p * x1p;
    let den = rx_sq * y1p * y1p + ry_sq * x1p * x1p;
    let mut coef = if den == 0.0 {
        0.0
    } else {
        (num / den).max(0.0).sqrt()
    };
    if large_arc == sweep {
        coef = -coef;
    }
    let cxp = coef * rx * y1p / ry;
    let cyp = -coef * ry * x1p / rx;

    // Step 3: center in original coordinates.
    let cx = cos_plev * cxp - sin_plev * cyp + (x1 + x2) / 2.0;
    let cy = sin_plev * cxp + cos_plev * cyp + (y1 + y2) / 2.0;

    // Step 4: start angle and sweep extent.
    let angle = |ux: f32, uy: f32, vx: f32, vy: f32| -> f32 {
        let dot = ux * vx + uy * vy;
        let len = (ux * ux + uy * uy).sqrt() * (vx * vx + vy * vy).sqrt();
        let mut a = (dot / len).clamp(-1.0, 1.0).acos();
        if ux * vy - uy * vx < 0.0 {
            a = -a;
        }
        a
    };
    let theta1 = angle(1.0, 0.0, (x1p - cxp) / rx, (y1p - cyp) / ry);
    let mut dtheta = angle(
        (x1p - cxp) / rx,
        (y1p - cyp) / ry,
        (-x1p - cxp) / rx,
        (-y1p - cyp) / ry,
    );
    if !sweep && dtheta > 0.0 {
        dtheta -= 2.0 * std::f32::consts::PI;
    } else if sweep && dtheta < 0.0 {
        dtheta += 2.0 * std::f32::consts::PI;
    }

    // Split into <= 90 degree segments, each approximated by one cubic.
    let segments = (dtheta.abs() / std::f32::consts::FRAC_PI_2).ceil().max(1.0) as usize;
    let delta = dtheta / segments as f32;
    let alpha = 4.0 / 3.0 * (delta / 4.0).tan();

    let point = |t: f32| -> (f32, f32) {
        let (sin_t, cos_t) = t.sin_cos();
        (
            cx + rx * cos_t * cos_plev - ry * sin_t * sin_plev,
            cy + rx * cos_t * sin_plev + ry * sin_t * cos_plev,
        )
    };
    let derivative = |t: f32| -> (f32, f32) {
        let (sin_t, cos_t) = t.sin_cos();
        (
            -rx * sin_t * cos_plev - ry * cos_t * sin_plev,
            -rx * sin_t * sin_plev + ry * cos_t * cos_plev,
        )
    };

    let mut t = theta1;
    for _ in 0..segments {
        let t_next = t + delta;
        let (px, py) = point(t);
        let (nx, ny) = point(t_next);
        let (dx1, dy1) = derivative(t);
        let (dx2_, dy2_) = derivative(t_next);
        pb = pb.cubic_bezier_to(
            [(px + alpha * dx1) * scale, (py + alpha * dy1) * scale],
            [(nx - alpha * dx2_) * scale, (ny - alpha * dy2_) * scale],
            [nx * scale, ny * scale],
        );
        t = t_next;
    }
    pb
}
