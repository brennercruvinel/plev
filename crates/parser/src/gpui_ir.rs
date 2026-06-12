//! Working types and small helpers shared by the gpui resolver: the
//! mutable node under construction (ParserNode plus the flags the flank
//! rewrite needs) and accessors over the parsed chain mini-AST.

use crate::gpui::{GCall, GExpr};
use crate::ir::{ParserNode, Prop, Tag, TextValue};

/// Node under construction: a ParserNode plus the absolute/full-width flags
/// consumed by the flank rewrite.
#[derive(Default)]
pub(crate) struct WNode {
    pub props: Vec<Prop>,
    pub children: Vec<WNode>,
    pub text: Option<TextValue>,
    /// Line of the `.absolute()` call, when present.
    pub absolute: Option<usize>,
    /// `.w_full()` on the main axis; becomes `grow(1.0)` in the rewrite.
    pub full_main: bool,
}

pub(crate) fn finish(w: WNode) -> ParserNode {
    let tag = match w.text {
        Some(t) => Tag::Text(t),
        None => Tag::Div,
    };
    ParserNode {
        tag,
        props: w.props,
        children: w.children.into_iter().map(finish).collect(),
    }
}

/// Calls resolvable from a combinator body or match arm.
pub(crate) enum Applied {
    Calls(Vec<GCall>),
    None,
}

pub(crate) fn closure_body(arg: &GExpr) -> Option<&GExpr> {
    match arg {
        GExpr::Closure { body, .. } => Some(body),
        _ => None,
    }
}

/// `px(1.)` -> 1.0.
pub(crate) fn arg_px(arg: &GExpr) -> Option<f32> {
    let GExpr::Src { text, .. } = arg else {
        return None;
    };
    let inner = text.strip_prefix("px(")?.strip_suffix(")")?;
    inner.trim_end_matches('.').parse::<f32>().ok()
}

pub(crate) fn expr_line(e: &GExpr) -> usize {
    match e {
        GExpr::Chain { base_line, .. } => *base_line,
        GExpr::Match { line, .. } | GExpr::Closure { line, .. } | GExpr::Src { line, .. } => *line,
    }
}

impl GCall {
    /// Static name for prop construction (only called for known methods).
    pub(crate) fn method_static(&self) -> &'static str {
        match self.method.as_str() {
            "h" => "h",
            "w" => "w",
            "bg" => "bg",
            "text_color" => "text_color",
            other => panic!("method_static on uncovered method {other}"),
        }
    }
}
