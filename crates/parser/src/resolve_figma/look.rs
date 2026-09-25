//! How a Figma layer looks, for `resolve_figma`: its fills, strokes,
//! corner radius, clipping, effects, opacity and text style as builder
//! props, or entries on the droplist with the reason; and the padding of
//! an auto layout frame.

use serde_json::Value;

use super::Ctx;
use crate::css_map::color_token;
use crate::figma::FigNode;
use crate::ir::{Arg, Prop, Src};

/// A paint as the builder's color expression, or why it has none.
fn paint_color(p: &Value) -> Result<String, String> {
    let kind = p.get("type").and_then(Value::as_str).unwrap_or("SOLID");
    if p.get("variable").and_then(Value::as_str) == Some("unresolved") {
        return Err("the color is bound to a library's variable the export does not hold: its value in the screen's mode is unknown".into());
    }
    match kind {
        "SOLID" => {
            let c = p.get("color");
            let f = |k: &str| {
                c.and_then(|c| c.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0)
            };
            let opacity = p.get("opacity").and_then(Value::as_f64).unwrap_or(1.0);
            let byte = |v: f64| (v * 255.0).round().clamp(0.0, 255.0) as u8;
            let alpha = ((f("a") * opacity) * 1000.0).round() / 1000.0;
            Ok(color_token((
                byte(f("r")),
                byte(f("g")),
                byte(f("b")),
                alpha as f32,
            )))
        }
        "IMAGE" => Err(
            "an image fill: the IR carries no image data; the layer keeps its place and size"
                .into(),
        ),
        k if k.starts_with("GRADIENT") => {
            Err("a gradient has no IR form: a color role is one color".into())
        }
        k => Err(format!("a {k} paint has no IR form")),
    }
}

/// `#353333`, `#FFFFFF at 0.82`: a paint as a construct names it.
fn paint_label(p: &Value) -> String {
    let kind = p.get("type").and_then(Value::as_str).unwrap_or("SOLID");
    let Some(c) = p.get("color") else {
        return kind.to_string();
    };
    let f = |k: &str| c.get(k).and_then(Value::as_f64).unwrap_or(0.0);
    let byte = |v: f64| (v * 255.0).round().clamp(0.0, 255.0) as u8;
    let mut out = format!(
        "#{:02X}{:02X}{:02X}",
        byte(f("r")),
        byte(f("g")),
        byte(f("b"))
    );
    let a = f("a") * p.get("opacity").and_then(Value::as_f64).unwrap_or(1.0);
    if a < 1.0 {
        out.push_str(&format!(" at {}", (a * 100.0).round() / 100.0));
    }
    out
}

impl Ctx<'_> {
    /// `fills`: the top paint is the layer's color (`bg`, or `text_color`
    /// for a text); a paint under it is covered, and is lost.
    pub(super) fn fills(&mut self, n: &FigNode, prop: &'static str, props: &mut Vec<Prop>) {
        let Some(list) = n.get("fills").and_then(Value::as_array) else {
            return;
        };
        let line = n.line_of("fills");
        let top = list.len().saturating_sub(1);
        for (i, p) in list.iter().enumerate() {
            let src = self.src(line, format!("fills[{i}] {}", paint_label(p)));
            if i < top {
                let why = "a paint under the top one: the IR has one color per layer";
                self.lose(&src, why);
                continue;
            }
            match paint_color(p) {
                Ok(expr) => {
                    self.mapped += 1;
                    props.push(Prop::token(prop, expr).from(src));
                }
                Err(why) => self.lose(&src, why),
            }
        }
    }

    /// `strokes` and `strokeWeight`: the builder's border; the IR has none.
    pub(super) fn strokes(&mut self, n: &FigNode, props: &mut Vec<Prop>) {
        let Some(list) = n.get("strokes").and_then(Value::as_array) else {
            return;
        };
        let line = n.line_of("strokes");
        let weight = n.num("strokeWeight").unwrap_or(1.0);
        let wsrc = self.src(n.line_of("strokeWeight"), format!("strokeWeight {weight}"));
        let top = list.len().saturating_sub(1);
        for (i, p) in list.iter().enumerate() {
            let src = self.src(line, format!("strokes[{i}] {}", paint_label(p)));
            match (i == top, paint_color(p)) {
                (true, Ok(expr)) => {
                    self.mapped += 1;
                    props.push(Prop::f32("border", weight).from(wsrc.clone()));
                    props.push(Prop::token("border_color", expr).from(src));
                }
                (true, Err(why)) => {
                    self.lose(&src, why);
                    self.lose(&wsrc, "the width of a stroke that has no color");
                }
                (false, _) => self.lose(&src, "a stroke under the top one: a border has one color"),
            }
        }
    }

    /// `cornerRadius`, `rectangleCornerRadii`, `clipsContent`, `opacity`,
    /// `rotation` and `effects`.
    pub(super) fn shape(&mut self, n: &FigNode, props: &mut Vec<Prop>) {
        if let Some(r) = n.num("cornerRadius") {
            // Figma draws no radius past half the shorter side: a 191 on a
            // 46 px circle is 23
            let [_, _, w, h] = n.bounds().unwrap_or_default();
            let half = w.min(h) / 2.0;
            let drawn = if half > 0.0 { r.min(half) } else { r };
            let what = if drawn < r {
                format!("cornerRadius {r} (drawn {drawn})")
            } else {
                format!("cornerRadius {r}")
            };
            let src = self.src(n.line_of("cornerRadius"), what);
            self.mapped += 1;
            props.push(Prop::f32("rounded", drawn).from(src));
        }
        if let Some(v) = n.get("rectangleCornerRadii") {
            let src = self.src(
                n.line_of("rectangleCornerRadii"),
                format!("rectangleCornerRadii {v}"),
            );
            self.lose(&src, "a radius per corner: the IR has one shape per node");
        }
        if n.get("clipsContent").and_then(Value::as_bool) == Some(true) {
            let src = self.src(n.line_of("clipsContent"), "clipsContent true");
            self.mapped += 1;
            props.push(Prop::flag("clip_children").from(src));
        }
        if let Some(o) = n.num("opacity") {
            let src = self.src(n.line_of("opacity"), format!("opacity {o}"));
            self.lose(&src, "the IR has no layer opacity");
        }
        if let Some(r) = n.num("rotation") {
            let src = self.src(n.line_of("rotation"), format!("rotation {r}"));
            self.lose(&src, "the IR has no rotation: a node is laid out upright");
        }
        let effects = n.get("effects").and_then(Value::as_array);
        for (i, e) in effects.into_iter().flatten().enumerate() {
            let kind = e.get("type").and_then(Value::as_str).unwrap_or("EFFECT");
            let src = self.src(n.line_of("effects"), format!("effects[{i}] {kind}"));
            let why = match kind {
                "DROP_SHADOW" => {
                    "the IR's shadows are the theme's elevation stacks, not a single shadow"
                }
                "INNER_SHADOW" => "the IR has no inner shadow",
                "BACKGROUND_BLUR" => "the IR has no backdrop blur",
                _ => "the IR has no blur",
            };
            self.lose(&src, why);
        }
    }

    /// The padding of an auto layout frame, as the builder's shortest calls.
    pub(super) fn padding(&mut self, n: &FigNode, props: &mut Vec<Prop>) {
        let side = |k: &'static str| (n.num(k).unwrap_or(0.0), n.get(k).map(|_| k));
        let [t, r, b, l] = [
            side("paddingTop"),
            side("paddingRight"),
            side("paddingBottom"),
            side("paddingLeft"),
        ];
        let srcs = |cx: &Self, keys: &[Option<&'static str>]| -> Vec<Src> {
            keys.iter()
                .flatten()
                .map(|k| cx.src(n.line_of(k), format!("{k} {}", n.num(k).unwrap_or(0.0))))
                .collect()
        };
        let mut push =
            |cx: &mut Self, name: &'static str, v: f32, keys: &[Option<&'static str>]| {
                let found = srcs(cx, keys);
                if v != 0.0 && !found.is_empty() {
                    cx.mapped += found.len();
                    let mut p = Prop::f32(name, v);
                    p.src = found;
                    props.push(p);
                }
            };
        if t.0 == b.0 && l.0 == r.0 && t.0 == l.0 {
            push(self, "p", t.0, &[t.1, r.1, b.1, l.1]);
            return;
        }
        if t.0 == b.0 {
            push(self, "py", t.0, &[t.1, b.1]);
        } else {
            push(self, "pt", t.0, &[t.1]);
            push(self, "pb", b.0, &[b.1]);
        }
        if l.0 == r.0 {
            push(self, "px", l.0, &[l.1, r.1]);
        } else {
            push(self, "pl", l.0, &[l.1]);
            push(self, "pr", r.0, &[r.1]);
        }
    }

    /// A text's `style`: size, line height and weight as the run's style;
    /// family, letter spacing and alignment lost; `textAutoResize` is
    /// returned, with its construct, for the sizing.
    pub(super) fn style(&mut self, n: &FigNode, props: &mut Vec<Prop>) -> Option<(String, Src)> {
        let style = n.get("style")?.as_object()?;
        let line = n.line_of("style");
        let mut auto = None;
        for (k, v) in style {
            let src = self.src(line, format!("style.{k} {v}"));
            let num = v.as_f64().map(|x| x as f32);
            match (k.as_str(), num) {
                ("fontSize", Some(x)) => props.push(Prop::f32("font_size", x).from(src)),
                ("lineHeightPx", Some(x)) => props.push(Prop::f32("line_height", x).from(src)),
                ("fontWeight", Some(x)) => {
                    props.push(Prop::new("font_weight", vec![Arg::Int(x as i64)]).from(src))
                }
                ("textAutoResize", _) => {
                    auto = v.as_str().map(|a| (a.to_string(), src));
                    continue;
                }
                ("fontFamily", _) => {
                    self.lose(&src, "font family is engine-global (embedded faces only)");
                    continue;
                }
                ("letterSpacing", _) => {
                    self.lose(&src, "the IR has no letter spacing");
                    continue;
                }
                ("textAlignHorizontal", _) => {
                    self.lose(&src, "the IR has no text alignment");
                    continue;
                }
                _ => {
                    self.lose(&src, "a text style field the importer does not read");
                    continue;
                }
            }
            self.mapped += 1;
        }
        auto
    }
}
