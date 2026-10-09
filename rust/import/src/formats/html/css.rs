//! Stage 2 (html, declaration level, task-71 phase 2): the CSS subset of
//! the HTML importer. One declaration -> builder props, a no-op that holds,
//! a spacer, or a drop with its reason; one selector -> simple (`.a`, `#a`,
//! `tag`) or the reason it is outside the subset; the browser's default
//! styles the subset does not apply; the handlers a control names.
//!
//! Flex, positioning and grid are decided here. What the react subset
//! already maps (sizes, padding, colors, radius, fonts, borders, shadows)
//! goes through `css_map::map_decl` after [`normalize`] rewrites what CSS
//! writes and the corpus sass does not (hex and named colors, named
//! weights), so both sources read one mapper and react's output does not
//! move.

use crate::css_map::{MapOut, map_decl, px};
use crate::ir::{AttrValue, Prop};

/// What one declaration becomes.
#[derive(Debug, Clone, PartialEq)]
pub enum Mapped {
    Props(Vec<Prop>),
    /// Holds in plev as the source says it; nothing to emit.
    Holds(&'static str),
    /// `margin-bottom: v`: the phase 1 spacer rewrite.
    Spacer(f32),
    Drop(String),
}

/// A simple selector of the subset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    Class(String),
    Id(String),
    Tag(String),
}

impl Selector {
    /// Specificity rank: tag, class, id.
    pub fn rank(&self) -> u8 {
        match self {
            Selector::Tag(_) => 1,
            Selector::Class(_) => 2,
            Selector::Id(_) => 3,
        }
    }
}

fn ident(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '-')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// One selector of a list: simple, or why it is outside the subset.
pub fn selector(part: &str) -> Result<Selector, String> {
    let part = part.trim();
    if let Some(c) = part.strip_prefix('.').filter(|c| ident(c)) {
        return Ok(Selector::Class(c.into()));
    }
    if let Some(i) = part.strip_prefix('#').filter(|i| ident(i)) {
        return Ok(Selector::Id(i.into()));
    }
    if ident(part) && part.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Ok(Selector::Tag(part.to_ascii_lowercase()));
    }
    let why = if part.contains(':') {
        "a pseudo-class or pseudo-element selects a state or a generated box; the IR has neither"
    } else if part.contains([' ', '>', '+', '~']) {
        "a combinator (descendant, child, sibling) is outside the subset: only .class, #id and a tag"
    } else if part.contains('*') {
        "the universal selector is outside the subset: only .class, #id and a tag"
    } else if part.contains('[') {
        "an attribute selector is outside the subset: only .class, #id and a tag"
    } else {
        "a compound selector is outside the subset: only .class, #id and a tag"
    };
    Err(why.into())
}

/// Why an at-rule is outside the subset.
pub fn at_rule(prelude: &str) -> &'static str {
    let name = prelude.split_whitespace().next().unwrap_or("");
    match name {
        "@media" => "a media query is a responsive variant: the subset imports one layout",
        "@keyframes" => "an animation has no IR form",
        "@import" => "an imported stylesheet is not read: only <style> and style=\"\" are",
        "@font-face" => "a font face is engine-global: the importer loads none",
        _ => "an at-rule outside the subset",
    }
}

/// `#fff`, `#303030`, `#30303080` -> `rgba(r, g, b, a)`.
fn hex(h: &str) -> Option<String> {
    let digits: Vec<u8> = h
        .chars()
        .map(|c| c.to_digit(16).map(|d| d as u8))
        .collect::<Option<_>>()?;
    let pair = |i: usize| {
        digits
            .get(i)
            .zip(digits.get(i + 1))
            .map(|(a, b)| a * 16 + b)
    };
    let (r, g, b, a) = match digits.len() {
        3 | 4 => {
            let one = |i: usize| digits.get(i).map(|d| d * 17);
            (one(0)?, one(1)?, one(2)?, one(3).unwrap_or(255))
        }
        6 | 8 => (pair(0)?, pair(2)?, pair(4)?, pair(6).unwrap_or(255)),
        _ => return None,
    };
    let alpha = (f32::from(a) / 255.0 * 1000.0).round() / 1000.0;
    Some(format!("rgba({r}, {g}, {b}, {alpha})"))
}

/// The value as `css_map::map_decl` reads it: hex and named colors as
/// `rgba(..)`, `bold` and `normal` weights as numbers.
pub fn normalize(prop: &str, value: &str) -> String {
    if prop == "font-weight" {
        return match value {
            "normal" => "400".into(),
            "bold" => "700".into(),
            v => v.into(),
        };
    }
    value
        .split(' ')
        .map(|word| {
            let (core, comma) = match word.strip_suffix(',') {
                Some(c) => (c, ","),
                None => (word, ""),
            };
            let color = match core {
                "white" => Some("rgba(255, 255, 255, 1)".to_string()),
                "black" => Some("rgba(0, 0, 0, 1)".to_string()),
                "transparent" => Some("rgba(0, 0, 0, 0)".to_string()),
                c => c.strip_prefix('#').and_then(hex),
            };
            format!("{}{comma}", color.unwrap_or_else(|| core.to_string()))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The props the react subset maps; any other declaration is outside the
/// HTML subset unless this module decides it.
const SHARED: &[&str] = &[
    "width",
    "height",
    "padding",
    "margin-bottom",
    "background",
    "background-color",
    "color",
    "backdrop-filter",
    "box-shadow",
    "border-radius",
    "border",
    "overflow",
    "font-size",
    "line-height",
    "font-weight",
    "box-sizing",
    "font-family",
    "cursor",
    "opacity",
    "animation",
    "transition",
    "transform",
    "filter",
];

const NO_POSITION: &str = "the IR has no position: an element follows the flow of its container (the subset is flow only)";

fn drop(why: &str) -> Mapped {
    Mapped::Drop(why.into())
}

/// One declaration of an element, `flex` when the element is a flex
/// container, `font_size` the size a ratio line height multiplies.
pub fn map(prop: &str, value: &str, font_size: Option<f32>, flex: bool) -> Mapped {
    use Mapped::*;
    match prop {
        "gap" | "row-gap" | "column-gap" if !flex => {
            Holds("a gap spaces flex children only; this element is a block")
        }
        "gap" | "row-gap" | "column-gap" => match px(value) {
            Some(v) => Props(vec![Prop::f32("gap", v)]),
            None => drop("a gap that is not one length in pixels"),
        },
        "align-items" if !flex => Holds("align-items acts on flex containers only"),
        "align-items" => {
            let a = match value {
                "flex-start" | "start" | "self-start" => "Start",
                "center" => "Center",
                "flex-end" | "end" | "self-end" => "End",
                "stretch" | "normal" => "Stretch",
                _ => return drop("an alignment other than start, center, end or stretch"),
            };
            Props(vec![Prop::token("align_items", format!("Align::{a}"))])
        }
        "justify-content" if !flex => Holds("justify-content acts on flex containers only"),
        "justify-content" => match value {
            "flex-start" | "start" | "normal" => {
                Holds("start is where a stack puts its children, the plev default")
            }
            "center" => Props(vec![Prop::token("justify", "Justify::Center")]),
            "flex-end" | "end" => Props(vec![Prop::token("justify", "Justify::End")]),
            "space-between" => Props(vec![Prop::token("justify", "Justify::SpaceBetween")]),
            _ => drop("a justification other than start, center, end or space-between"),
        },
        "flex-grow" | "flex" => {
            let first = value.split_whitespace().next().unwrap_or("");
            match (first, first.parse::<f32>()) {
                ("none", _) => Holds("flex: none neither grows nor shrinks, the plev default"),
                ("auto", _) => Props(vec![Prop::f32("grow", 1.0)]),
                (_, Ok(0.0)) => Holds("a weight of 0 does not grow, the plev default"),
                (_, Ok(v)) => Props(vec![Prop::f32("grow", v)]),
                _ => drop("a flex value that is not a weight"),
            }
        }
        "flex-shrink" => match value.parse::<f32>() {
            Ok(v) => Props(vec![Prop::f32("shrink", v)]),
            Err(_) => drop("a shrink that is not a number"),
        },
        "flex-wrap" if value == "nowrap" => Holds("a stack does not wrap, the plev default"),
        "flex-wrap" => drop("the IR's stack does not wrap"),
        "flex-basis" => drop("the IR has no flex basis"),
        "position" if value == "static" => Holds("static is the flow, the plev default"),
        "position" | "top" | "right" | "bottom" | "left" | "inset" | "z-index" => drop(NO_POSITION),
        "float" if value == "none" => Holds("no float is the flow, the plev default"),
        "float" | "clear" => drop("float is outside the subset: an element follows the flow"),
        p if p.starts_with("grid") => drop("grid layout is outside the subset"),
        "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => {
            let side = match prop {
                "padding-top" => "pt",
                "padding-right" => "pr",
                "padding-bottom" => "pb",
                _ => "pl",
            };
            match px(value) {
                Some(0.0) => Holds("zero padding"),
                Some(v) => Props(vec![Prop::f32(side, v)]),
                None => drop("a padding that is not a length in pixels"),
            }
        }
        "margin" => margin(value),
        "margin-top" | "margin-left" | "margin-right" if px(value) == Some(0.0) => {
            Holds("margin: 0 is the plev default")
        }
        "margin-top" | "margin-left" | "margin-right" => {
            drop("a margin on a side other than the bottom has no builder or IR form")
        }
        "list-style" | "list-style-type" if value == "none" => {
            Holds("no list markers, as plev draws none")
        }
        "list-style" | "list-style-type" => drop("a list marker has no IR form"),
        "text-align" if matches!(value, "left" | "start") => {
            Holds("text starts at the left, the plev default")
        }
        "text-align" => drop("the IR has no text alignment"),
        "letter-spacing" => drop("the IR has no letter spacing"),
        p if SHARED.contains(&p) => match map_decl(p, &normalize(p, value), font_size) {
            MapOut::Props(props) => Props(props),
            MapOut::Noop(why) => Holds(why),
            MapOut::SpacerAfter(h) => Spacer(h),
            MapOut::Drop(why) => Drop(why.into()),
        },
        _ => drop("a declaration outside the subset"),
    }
}

/// `margin: 0`, or a margin only below (the spacer rewrite); any other
/// side has no form.
fn margin(value: &str) -> Mapped {
    let vals: Option<Vec<f32>> = value.split_whitespace().map(px).collect();
    let (t, r, b, l) = match vals.as_deref() {
        Some([a]) => (*a, *a, *a, *a),
        Some([v, h]) => (*v, *h, *v, *h),
        Some([t, h, b]) => (*t, *h, *b, *h),
        Some([t, r, b, l]) => (*t, *r, *b, *l),
        _ => return drop("a margin that is not lengths in pixels"),
    };
    if t != 0.0 || r != 0.0 || l != 0.0 {
        drop("a margin on a side other than the bottom has no builder or IR form")
    } else if b == 0.0 {
        Mapped::Holds("margin: 0 is the plev default")
    } else {
        Mapped::Spacer(b)
    }
}

/// The browser's default styles of `tag` the subset does not apply: each
/// with the declarations that replace it and what the browser would do.
pub fn defaults(tag: &str) -> &'static [(&'static str, &'static [&'static str], &'static str)] {
    const MARGIN: &[&str] = &["margin", "margin-top", "margin-bottom"];
    const SIZE: &[&str] = &["font-size"];
    const WEIGHT: &[&str] = &["font-weight"];
    match tag {
        "p" => &[("margin", MARGIN, "1em above and below")],
        "h1" => &[
            ("margin", MARGIN, "0.67em above and below"),
            ("font-size", SIZE, "2em"),
            ("font-weight", WEIGHT, "bold"),
        ],
        "h2" => &[
            ("margin", MARGIN, "0.83em above and below"),
            ("font-size", SIZE, "1.5em"),
            ("font-weight", WEIGHT, "bold"),
        ],
        "h3" => &[
            ("margin", MARGIN, "1em above and below"),
            ("font-size", SIZE, "1.17em"),
            ("font-weight", WEIGHT, "bold"),
        ],
        "ul" => &[
            ("margin", MARGIN, "1em above and below"),
            ("padding", &["padding", "padding-left"], "an indent of 40px"),
            (
                "list-style",
                &["list-style", "list-style-type"],
                "a marker per item",
            ),
        ],
        _ => &[],
    }
}

/// An attribute as the control inference reads it: `onclick` is
/// `onClick`, `oninput` and `onchange` are `onChange`, and a handler that
/// only calls a named function (`open()`, `reply(this.value)`) names it.
pub fn attr(name: &str, value: Option<&str>) -> (String, AttrValue) {
    let canonical = match name {
        "onclick" => "onClick",
        "oninput" | "onchange" => "onChange",
        other => other,
    };
    let Some(v) = value else {
        return (canonical.into(), AttrValue::Flag);
    };
    if !name.starts_with("on") {
        return (canonical.into(), AttrValue::Text(v.into()));
    }
    let call = v.trim().trim_end_matches(';').trim();
    let named = call.split_once('(').and_then(|(f, args)| {
        let args = args.strip_suffix(')')?.trim();
        let plain = matches!(args, "" | "this.value" | "event");
        (plain && ident(f.trim()) && !f.contains('-')).then(|| f.trim().to_string())
    });
    match named {
        Some(f) => (canonical.into(), AttrValue::Ident(f)),
        None => (canonical.into(), AttrValue::Expr(v.into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Arg;

    #[test]
    fn selectors_of_the_subset_and_why_the_rest_is_not() {
        assert_eq!(
            selector(".note-card"),
            Ok(Selector::Class("note-card".into()))
        );
        assert_eq!(selector("#main"), Ok(Selector::Id("main".into())));
        assert_eq!(selector("h2"), Ok(Selector::Tag("h2".into())));
        for (s, word) in [
            (".a:hover", "pseudo"),
            (".a .b", "combinator"),
            ("*", "universal"),
            ("input[type=text]", "attribute"),
            ("button.primary", "compound"),
        ] {
            assert!(selector(s).is_err_and(|e| e.contains(word)), "{s}");
        }
    }

    #[test]
    fn colors_and_weights_read_as_the_react_mapper_reads_them() {
        assert_eq!(normalize("background", "#2e2e2e"), "rgba(46, 46, 46, 1)");
        assert_eq!(normalize("color", "#fff8"), "rgba(255, 255, 255, 0.533)");
        assert_eq!(
            normalize("border", "1px solid white"),
            "1px solid rgba(255, 255, 255, 1)"
        );
        assert_eq!(normalize("font-weight", "bold"), "700");
        assert!(
            matches!(map("background", "#303030", None, false), Mapped::Props(p) if p[0].name == "bg")
        );
    }

    #[test]
    fn flex_in_a_flex_container_and_nothing_in_a_block() {
        assert_eq!(
            map("gap", "12px", None, true),
            Mapped::Props(vec![Prop::f32("gap", 12.0)])
        );
        assert!(matches!(map("gap", "12px", None, false), Mapped::Holds(_)));
        let Mapped::Props(p) = map("align-items", "center", None, true) else {
            panic!()
        };
        assert_eq!(p[0].args, vec![Arg::Token("Align::Center".into())]);
        assert!(matches!(
            map("grid-template-columns", "1fr 1fr", None, false),
            Mapped::Drop(_)
        ));
        assert!(
            matches!(map("position", "absolute", None, false), Mapped::Drop(w) if w.contains("position"))
        );
        assert_eq!(map("margin", "0 0 8px", None, false), Mapped::Spacer(8.0));
        assert!(matches!(
            map("margin", "8px 0", None, false),
            Mapped::Drop(_)
        ));
    }

    #[test]
    fn a_handler_that_calls_a_named_function_names_it() {
        assert_eq!(
            attr("onclick", Some("openNote()")),
            ("onClick".into(), AttrValue::Ident("openNote".into()))
        );
        assert_eq!(
            attr("oninput", Some("reply(this.value)")),
            ("onChange".into(), AttrValue::Ident("reply".into()))
        );
        assert_eq!(
            attr("onclick", Some("a(); b()")).1,
            AttrValue::Expr("a(); b()".into())
        );
        assert_eq!(
            attr("placeholder", Some("Reply")).1,
            AttrValue::Text("Reply".into())
        );
        assert_eq!(attr("disabled", None).1, AttrValue::Flag);
    }
}
