//! The SVG text under construction: elements in draw order, definitions
//! (clip paths, filters, gradients) gathered into one `<defs>`, and the
//! report of approximations, plus the attribute formatting the elements
//! share (numbers, colors, escaped text).

use std::fmt::Write as _;

use super::super::Viewport;
use super::Approximation;

pub(super) struct Svg {
    body: String,
    defs: String,
    ids: u32,
    depth: usize,
    report: Vec<Approximation>,
}

impl Svg {
    pub(super) fn new() -> Self {
        Self {
            body: String::new(),
            defs: String::new(),
            ids: 0,
            depth: 1,
            report: Vec::new(),
        }
    }

    fn indent(&mut self) {
        for _ in 0..self.depth {
            self.body.push_str("  ");
        }
    }

    /// A self-closing element (or any one-line content) at the current depth.
    pub(super) fn element(&mut self, line: &str) {
        self.indent();
        self.body.push_str(line);
        self.body.push('\n');
    }

    /// An opening tag; what follows is nested until [`close`](Self::close).
    pub(super) fn open(&mut self, tag: &str) {
        self.element(tag);
        self.depth += 1;
    }

    pub(super) fn close(&mut self, tag: &str) {
        self.depth = self.depth.saturating_sub(1).max(1);
        self.element(tag);
    }

    /// A fresh id for a definition.
    fn id(&mut self, prefix: &str) -> String {
        self.ids += 1;
        format!("{prefix}{}", self.ids)
    }

    /// A rectangular `<clipPath>` in `<defs>`; its id, for `clip-path`.
    pub(super) fn clip_path(&mut self, [x, y, w, h]: [f32; 4]) -> String {
        let id = self.id("clip");
        let _ = writeln!(
            self.defs,
            "    <clipPath id=\"{id}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/></clipPath>",
            num(x),
            num(y),
            num(w.max(0.0)),
            num(h.max(0.0))
        );
        id
    }

    /// Something the SVG does not hold exactly.
    pub(super) fn note(&mut self, node: &str, what: &str) {
        self.report.push(Approximation {
            node: node.to_owned(),
            what: what.to_owned(),
        });
    }

    /// The document: header, `<defs>` when there are any, the body.
    pub(super) fn finish(self, viewport: Viewport) -> (String, Vec<Approximation>) {
        let (pw, ph) = viewport.physical();
        let mut out = String::with_capacity(self.body.len() + self.defs.len() + 256);
        out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        let _ = writeln!(
            out,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{pw}\" height=\"{ph}\" \
             viewBox=\"0 0 {} {}\">",
            num(viewport.width),
            num(viewport.height)
        );
        if !self.defs.is_empty() {
            out.push_str("  <defs>\n");
            out.push_str(&self.defs);
            out.push_str("  </defs>\n");
        }
        out.push_str(&self.body);
        out.push_str("</svg>\n");
        (out, self.report)
    }
}

/// A coordinate or length: two decimals, trailing zeros dropped, never a
/// negative zero.
pub(super) fn num(v: f32) -> String {
    let v = if v.is_finite() { v } else { 0.0 };
    let mut s = format!("{v:.2}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" { "0".to_owned() } else { s }
}

/// An sRGB color as `#rrggbb` (the alpha travels separately).
pub(super) fn hex(c: [f32; 4]) -> String {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", byte(c[0]), byte(c[1]), byte(c[2]))
}

/// ` name="a"` when the alpha is under one, nothing when it is opaque.
pub(super) fn opacity(name: &str, a: f32) -> String {
    if a < 1.0 {
        format!(" {name}=\"{}\"", num(a.max(0.0)))
    } else {
        String::new()
    }
}

/// The fill attributes of a solid sRGB color.
pub(super) fn fill(c: [f32; 4]) -> String {
    format!(" fill=\"{}\"{}", hex(c), opacity("fill-opacity", c[3]))
}

/// Text safe inside an attribute value.
pub(super) fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            '\t' => out.push_str("&#9;"),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out
}
