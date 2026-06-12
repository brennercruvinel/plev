//! Stage 2 (react, declaration level): one resolved css declaration -> plev
//! builder props, a recognized no-op, or a drop reason. Colors that hit the
//! HOFF palette become theme tokens (kdb/how-to/code-against-the-plev-engine
//! .md: tokens over literals); everything else stays an exact literal.

use crate::ir::{Arg, Prop};

/// Outcome of mapping a single declaration.
pub enum MapOut {
    Props(Vec<Prop>),
    /// Semantics already hold in plev by construction; nothing to emit.
    Noop(&'static str),
    /// `margin-bottom: v` -> insert `div().h(v)` spacer after the node
    /// (plev has no per-side margin builder; flow rhythm is preserved).
    SpacerAfter(f32),
    Drop(&'static str),
}

pub fn map_decl(prop: &str, value: &str, font_size: Option<f32>) -> MapOut {
    use MapOut::*;
    match prop {
        "width" => match px(value) {
            Some(v) => Props(vec![Prop::f32("w", v)]),
            None => Drop("non-pixel width (percent widths come from the parent in plev)"),
        },
        "height" => match px(value) {
            Some(v) => Props(vec![Prop::f32("h", v)]),
            None => Drop("non-pixel height"),
        },
        "padding" => map_padding(value),
        "margin" if px(value) == Some(0.0) => Noop("margin: 0 is the plev default"),
        "margin-bottom" => match px(value) {
            Some(v) => SpacerAfter(v),
            None => Drop("non-pixel margin-bottom"),
        },
        "background" | "background-color" => {
            if value == "none" {
                Noop("no background is the plev default")
            } else if let Some(c) = parse_rgba(value) {
                Props(vec![Prop::token("bg", color_token(c))])
            } else {
                Drop("unsupported background (gradient/image)")
            }
        }
        "color" => match parse_rgba(value) {
            Some(c) => Props(vec![Prop::token("text_color", color_token(c))]),
            None => Drop("unparsed color value"),
        },
        "backdrop-filter" => match blur_px(value) {
            Some(50.0) => Props(vec![Prop::token(
                "backdrop_blur",
                "theme.effects.blur_sigma",
            )]),
            Some(v) => Props(vec![Prop::f32("backdrop_blur", v)]),
            None => Drop("backdrop-filter other than blur()"),
        },
        "box-shadow" => map_box_shadow(value),
        "border-radius" => match px(value) {
            Some(32.0) => Props(vec![Prop::token("rounded", "theme.radius.xl")]),
            Some(v) => Props(vec![Prop::f32("rounded", v)]),
            None => Drop("non-pixel border-radius (50% circles unsupported)"),
        },
        "border" => map_border(value),
        "overflow" if value == "hidden" => Props(vec![Prop::flag("clip_children")]),
        "font-size" => match px(value) {
            Some(v) => Props(vec![Prop::f32("font_size", v)]),
            None => Drop("non-pixel font-size"),
        },
        "line-height" => {
            if value.ends_with("px") {
                match px(value) {
                    Some(v) => Props(vec![Prop::f32("line_height", v)]),
                    None => Drop("unparsed pixel line-height"),
                }
            } else if let (Ok(ratio), Some(fs)) = (value.parse::<f32>(), font_size) {
                Props(vec![Prop::f32("line_height", round1(fs * ratio))])
            } else {
                Drop("line-height ratio without a font-size in scope")
            }
        }
        "font-weight" => match value.parse::<i64>() {
            Ok(w) => Props(vec![Prop::new("font_weight", vec![Arg::Int(w)])]),
            Err(_) => Drop("non-numeric font-weight"),
        },
        "box-sizing" => Noop("plev layout is always border-box"),
        "content" => Noop("pseudo-element marker; merged into the host"),
        "pointer-events" => Noop("merged decorative layers expose no hit target"),
        "inset" if px(value) == Some(0.0) => {
            Noop("inset: 0 overlay covers the host box exactly (merge rewrite)")
        }
        "position" if value == "relative" => {
            Drop("no positioning contexts; plev paints in tree order")
        }
        "position" => Drop("position: absolute outside the overlay-merge pattern"),
        "z-index" => Drop("no z-index; plev paints in tree order"),
        "font-family" => Drop("font family is engine-global (embedded faces only)"),
        "cursor" => Drop("no per-element cursor control in the builder"),
        "mask-image" => Drop("gradient masks unsupported; layer drawn at full strength"),
        "opacity" => Drop("pseudo-layer opacity cannot dim only the merged props"),
        "animation" | "transition" | "transform" | "filter" => {
            Drop("animation/transform/filter have no builder equivalent")
        }
        _ => Drop("declaration not covered by the poc mapper"),
    }
}

/// Properties that live on a plev `text()` run rather than on its box.
pub fn is_text_prop(name: &str) -> bool {
    matches!(
        name,
        "font_size" | "line_height" | "font_weight" | "text_color" | "letter_spacing"
    )
}

fn map_padding(value: &str) -> MapOut {
    let vals: Vec<f32> = value.split_whitespace().filter_map(px).collect();
    let (t, r, b, l) = match vals.as_slice() {
        [a] => (*a, *a, *a, *a),
        [v, h] => (*v, *h, *v, *h),
        [t, h, b] => (*t, *h, *b, *h),
        [t, r, b, l] => (*t, *r, *b, *l),
        _ => return MapOut::Drop("unparsed padding shorthand"),
    };
    let mut props = Vec::new();
    if t == b && l == r && t == l {
        if t != 0.0 {
            props.push(Prop::f32("p", t));
        }
    } else {
        if t == b {
            if t != 0.0 {
                props.push(Prop::f32("py", t));
            }
        } else {
            if t != 0.0 {
                props.push(Prop::f32("pt", t));
            }
            if b != 0.0 {
                props.push(Prop::f32("pb", b));
            }
        }
        if l == r {
            if l != 0.0 {
                props.push(Prop::f32("px", l));
            }
        } else {
            if l != 0.0 {
                props.push(Prop::f32("pl", l));
            }
            if r != 0.0 {
                props.push(Prop::f32("pr", r));
            }
        }
    }
    // Order pt/py before px before pb for deterministic output.
    props.sort_by_key(|p| match p.name {
        "p" => 0,
        "pt" | "py" => 1,
        "px" => 2,
        _ => 3,
    });
    if props.is_empty() {
        MapOut::Noop("zero padding")
    } else {
        MapOut::Props(props)
    }
}

/// `[inset] x y blur [spread] rgba(...)` -> shadow_inset / shadow_drop.
fn map_box_shadow(value: &str) -> MapOut {
    if value.matches("rgba").count() > 1 {
        return MapOut::Drop("multi-shadow stacks unsupported in one builder prop");
    }
    let inset = value.trim_start().starts_with("inset");
    let color = match parse_rgba(value) {
        Some(c) => c,
        None => return MapOut::Drop("box-shadow without rgba color"),
    };
    let head = &value[..value.find("rgb").unwrap_or(value.len())];
    let nums: Vec<f32> = head.split_whitespace().filter_map(px).collect();
    let (x, y, blur, spread) = match nums.as_slice() {
        [x, y, blur] => (*x, *y, *blur, 0.0),
        [x, y, blur, spread] => (*x, *y, *blur, *spread),
        _ => return MapOut::Drop("unparsed box-shadow lengths"),
    };
    if spread != 0.0 {
        return MapOut::Drop("box-shadow spread has no builder equivalent");
    }
    if inset {
        MapOut::Props(vec![Prop::new(
            "shadow_inset",
            vec![
                Arg::F32(blur),
                Arg::Pair([x, y]),
                Arg::Token(color_token(color)),
            ],
        )])
    } else if x == 0.0 {
        MapOut::Props(vec![Prop::new(
            "shadow_drop",
            vec![Arg::F32(blur), Arg::F32(y), Arg::Token(color_token(color))],
        )])
    } else {
        MapOut::Drop("drop shadows support vertical offset only")
    }
}

fn map_border(value: &str) -> MapOut {
    if value.trim() == "0" {
        return MapOut::Noop("border: 0 is the plev default");
    }
    let width = value.split_whitespace().find_map(px);
    let color = parse_rgba(value);
    match (width, color) {
        (Some(w), Some(c)) => MapOut::Props(vec![
            Prop::f32("border", w),
            Prop::token("border_color", color_token(c)),
        ]),
        _ => MapOut::Drop("unparsed border shorthand"),
    }
}

/// `"368px"` / `"0"` / `"1.5px"` -> f32 logical pixels.
pub fn px(s: &str) -> Option<f32> {
    let s = s.trim().trim_end_matches("px");
    if s.contains('%') || s.contains('(') {
        return None;
    }
    s.parse::<f32>().ok()
}

fn blur_px(value: &str) -> Option<f32> {
    let inner = value.strip_prefix("blur(")?.strip_suffix(")")?;
    px(inner)
}

pub fn parse_rgba(value: &str) -> Option<(u8, u8, u8, f32)> {
    let start = value.find("rgba(").or_else(|| value.find("rgb("))?;
    let inner = &value[start..];
    let inner = &inner[inner.find('(')? + 1..inner.find(')')?];
    let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
    let r = parts.first()?.parse::<u8>().ok()?;
    let g = parts.get(1)?.parse::<u8>().ok()?;
    let b = parts.get(2)?.parse::<u8>().ok()?;
    let a = match parts.get(3) {
        Some(a) => a.parse::<f32>().ok()?,
        None => 1.0,
    };
    Some((r, g, b, a))
}

/// HOFF palette token table (src/theme/hoff.rs). Exact color matches become
/// semantic tokens; the n2/n3 base whites and graphites fall back to the
/// palette constructors; anything else is an exact literal.
pub fn color_token((r, g, b, a): (u8, u8, u8, f32)) -> String {
    let close = |x: f32, y: f32| (x - y).abs() < 0.005;
    match (r, g, b) {
        (248, 248, 248) if close(a, 0.95) => "theme.colors.text".into(),
        (248, 248, 248) if close(a, 0.70) => "theme.colors.text_mid".into(),
        (248, 248, 248) if close(a, 0.50) => "theme.colors.text_dim".into(),
        (248, 248, 248) if close(a, 0.06) => "theme.glass.inset_highlight".into(),
        (255, 255, 255) if close(a, 0.05) => "theme.glass.edge_soft".into(),
        (255, 255, 255) if close(a, 0.10) => "theme.glass.edge".into(),
        // Pre-composed card shell tone; see the hoff.rs compositing note.
        (40, 40, 40) if close(a, 0.80) => "plev::theme::hoff::CARD_OVERLAY".into(),
        (40, 40, 40) if close(a, 0.70) => "theme.glass.button".into(),
        (248, 248, 248) => format!("plev::theme::hoff::n2({})", fmt_f32(a)),
        (40, 40, 40) => format!("plev::theme::hoff::n3({})", fmt_f32(a)),
        _ => format!(
            "plev::color::Color::rgba({}, {}, {}, {})",
            fmt_f32(r as f32 / 255.0),
            fmt_f32(g as f32 / 255.0),
            fmt_f32(b as f32 / 255.0),
            fmt_f32(a)
        ),
    }
}

pub fn fmt_f32(v: f32) -> String {
    if v.fract() == 0.0 {
        format!("{v:.1}")
    } else {
        format!("{v}")
    }
}

fn round1(v: f32) -> f32 {
    (v * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_and_tokens() {
        let MapOut::Props(p) = map_decl(
            "box-shadow",
            "inset 2px 4px 16px rgba(248, 248, 248, 0.06)",
            None,
        ) else {
            panic!("expected props");
        };
        assert_eq!(p[0].name, "shadow_inset");
        assert_eq!(
            p[0].args,
            vec![
                Arg::F32(16.0),
                Arg::Pair([2.0, 4.0]),
                Arg::Token("theme.glass.inset_highlight".into())
            ]
        );
    }

    #[test]
    fn padding_shorthand_and_ratio_line_height() {
        let MapOut::Props(p) = map_decl("padding", "16px 16px 32px", None) else {
            panic!()
        };
        let names: Vec<&str> = p.iter().map(|p| p.name).collect();
        assert_eq!(names, vec!["pt", "px", "pb"]);
        let MapOut::Props(lh) = map_decl("line-height", "1.2", Some(20.0)) else {
            panic!()
        };
        assert_eq!(lh[0].args, vec![Arg::F32(24.0)]);
    }

    #[test]
    fn unknown_decl_is_dropped_not_silent() {
        assert!(matches!(
            map_decl("-webkit-line-clamp", "1", None),
            MapOut::Drop(_)
        ));
    }
}
