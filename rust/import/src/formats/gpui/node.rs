//! Working types and small helpers shared by the gpui resolver: the
//! mutable node under construction (ParserNode plus the flags the flank
//! rewrite needs) and accessors over the parsed chain mini-AST.

use crate::gpui::{GCall, GExpr, GFn};
use crate::ir::{Origin, ParserNode, Prop, Src, Tag, TextValue};

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
    /// The `.w_full()` calls behind `full_main`, for the grow prop.
    pub full_src: Vec<Src>,
    pub origin: Origin,
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
        origin: w.origin,
    }
}

/// `.h(px(1.))`, `.child(..)`: a call as a construct of the report, its
/// arguments as written when they are plain source text.
pub(crate) fn call_label(call: &GCall) -> String {
    let args: Vec<String> = call
        .args
        .iter()
        .map(|a| match a {
            GExpr::Src { text, .. } => text.clone(),
            GExpr::Match { scrutinee, .. } => format!("match {scrutinee}"),
            GExpr::Chain { base, .. } => format!("{base}.."),
            GExpr::Closure { .. } => "|..|".into(),
        })
        .collect();
    format!(".{}({})", call.method, args.join(", "))
}

/// The `file:line` of every call under `e`, following closures, match arms,
/// call arguments and the `Self::f(..)` helpers a chain starts from: what
/// an entry that drops `e` whole answers for.
pub(crate) fn call_sites(e: &GExpr, fns: &[GFn], file: &str, out: &mut Vec<String>) {
    let mut seen = Vec::new();
    sites(e, fns, file, &mut seen, out);
}

fn sites<'a>(
    e: &'a GExpr,
    fns: &'a [GFn],
    file: &str,
    seen: &mut Vec<&'a str>,
    out: &mut Vec<String>,
) {
    match e {
        GExpr::Chain { base, calls, .. } => {
            let helper = base
                .strip_prefix("Self::")
                .and_then(|b| b.split('(').next());
            if let Some(f) = helper.and_then(|n| fns.iter().find(|f| f.name == n))
                && !seen.contains(&f.name.as_str())
            {
                seen.push(f.name.as_str());
                if let Some(tail) = &f.tail {
                    sites(tail, fns, file, seen, out);
                }
            }
            for call in calls {
                let site = format!("{file}:{}", call.line);
                if !out.contains(&site) {
                    out.push(site);
                }
                for arg in &call.args {
                    sites(arg, fns, file, seen, out);
                }
            }
        }
        GExpr::Match { arms, .. } => {
            for arm in arms {
                sites(&arm.body, fns, file, seen, out);
            }
        }
        GExpr::Closure { body, .. } => sites(body, fns, file, seen, out),
        GExpr::Src { .. } => {}
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
