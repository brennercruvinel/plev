//! Stage 2 (gpui): builder chains -> ParserNode tree via the gpui->plev method
//! table from the separator study. The transpiled instance is the one the
//! study dissected: `Separator::horizontal().label(..)` (horizontal, solid,
//! label present, theme border color). Match arms and setters of other
//! variants land on the droplist with file:line.
//!
//! One documented rewrite: the gpui separator draws an absolute full-width
//! 1px line behind a centered label chip. plev has no absolute positioning,
//! so the line becomes two `grow(1.0)` segments flanking the chip; same
//! pixels, content-driven flow (the manual's rule).

use crate::gpui::{GCall, GExpr, GFn, GpuiSource};
use crate::gpui_ir::{Applied, WNode, arg_px, closure_body, expr_line, finish};
use crate::ir::{Dropped, Param, Prop, Resolution, TextValue, snake_case};

/// Variant configuration of the transpiled instance.
const TARGET_ARMS: &[(&str, &str)] = &[("axis", "Horizontal"), ("line_style", "Solid")];

pub fn resolve_gpui(src: &GpuiSource, file: &str) -> Result<Resolution, crate::ParserError> {
    let render = src
        .fns
        .iter()
        .find(|f| f.name == "render")
        .ok_or_else(|| crate::ParserError::Parse("no render fn in gpui source".into()))?;
    let Some(GExpr::Chain { calls, .. }) = &render.tail else {
        return Err(crate::ParserError::Parse(
            "render fn does not end in a builder chain".into(),
        ));
    };
    let mut cx = Ctx {
        fns: &src.fns,
        file,
        mapped: 0,
        dropped: Vec::new(),
        params: Vec::new(),
        color_noted: false,
    };
    let mut root = WNode::default();
    cx.apply_calls(&mut root, calls);
    cx.flank_rewrite(&mut root);
    let widget = src.widget.clone().unwrap_or_else(|| "Widget".into());
    Ok(Resolution {
        fn_name: snake_case(&widget),
        source_label: format!("{file} ({widget}::horizontal().label(..))"),
        root: finish(root),
        params: cx.params,
        mapped: cx.mapped,
        dropped: cx.dropped,
    })
}

struct Ctx<'a> {
    fns: &'a [GFn],
    file: &'a str,
    mapped: usize,
    dropped: Vec<Dropped>,
    params: Vec<Param>,
    color_noted: bool,
}

impl<'a> Ctx<'a> {
    fn apply_calls(&mut self, node: &mut WNode, calls: &[GCall]) {
        for call in calls {
            self.apply_call(node, call);
        }
    }

    fn apply_call(&mut self, node: &mut WNode, call: &GCall) {
        let at = format!("{}:{}", self.file, call.line);
        match call.method.as_str() {
            "flex" => self.push(node, Prop::flag("row")),
            "flex_shrink_0" => self.push(node, Prop::f32("shrink", 0.0)),
            "items_center" => self.push(node, Prop::flag("align_center")),
            "justify_center" => self.push(node, Prop::token("justify", "Justify::Center")),
            "absolute" => node.absolute = Some(call.line),
            "w_full" => {
                node.full_main = true; // becomes grow(1.0) in the flank rewrite
                self.mapped += 1;
            }
            "h" | "w" => match call.args.first().and_then(arg_px) {
                Some(v) => self.push(node, Prop::f32(call.method_static(), v)),
                None => self.drop_call(call, &at, "non-pixel size argument"),
            },
            "px_2" => self.push(node, Prop::f32("px", 8.0)),
            "py_1" => self.push(node, Prop::f32("py", 4.0)),
            "text_xs" => {
                // tailwind text-xs: 12px font, 16px line height.
                node.props.push(Prop::f32("font_size", 12.0));
                node.props.push(Prop::f32("line_height", 16.0));
                self.mapped += 1;
            }
            "mx_auto" => self.dropped.push(Dropped::new(
                ".mx_auto()",
                "no auto margins; centering comes from the flank rewrite",
                at,
            )),
            "h_full" => self.drop_call(call, &at, "percent height (vertical variant only)"),
            "bg" | "text_color" => match call.args.first().and_then(|a| self.color_expr(a, &at)) {
                Some(token) => self.push(node, Prop::token(call.method_static(), token)),
                None => self.drop_call(call, &at, "unresolved color expression"),
            },
            "refine_style" => self.drop_call(
                call,
                &at,
                "runtime StyleRefinement merge from the user; no static equivalent",
            ),
            "map" | "when_some" => {
                if let Some(body) = call.args.iter().find_map(closure_body) {
                    // when_some(label): the transpiled instance has a label.
                    match self.resolve_expr_onto(body) {
                        Applied::Calls(calls) => {
                            self.mapped += 1;
                            self.apply_calls(node, &calls);
                        }
                        Applied::None => self.drop_call(call, &at, "unresolved combinator body"),
                    }
                } else {
                    self.drop_call(call, &at, "combinator without closure argument");
                }
            }
            "child" => match call.args.first() {
                Some(arg) => self.child_arg(node, arg, &at),
                None => self.drop_call(call, &at, "child() without argument"),
            },
            "into_any_element" => self.mapped += 1, // type erasure, no visual effect
            _ => self.drop_call(call, &at, "method not covered by the poc table"),
        }
    }

    fn push(&mut self, node: &mut WNode, prop: Prop) {
        node.props.push(prop);
        self.mapped += 1;
    }

    fn child_arg(&mut self, node: &mut WNode, arg: &GExpr, at: &str) {
        match arg {
            GExpr::Src { text, .. } if text == "label" => {
                self.mapped += 1;
                if !self.params.contains(&Param::Text("label".into())) {
                    self.params.push(Param::Text("label".into()));
                }
                let mut child = WNode {
                    text: Some(TextValue::Param("label".into())),
                    ..WNode::default()
                };
                // One TextStyle per run: typography stays on the text run.
                let (text_props, box_props): (Vec<Prop>, Vec<Prop>) = node
                    .props
                    .drain(..)
                    .partition(|p| crate::css_map::is_text_prop(p.name));
                node.props = box_props;
                child.props = text_props;
                node.children.push(child);
            }
            GExpr::Chain {
                base,
                base_line,
                calls,
            } => {
                let mut child = WNode::default();
                if base != "div()" {
                    match self.inline_fn(base) {
                        Some(inner) => self.apply_calls(&mut child, &inner),
                        None => {
                            self.dropped.push(Dropped::new(
                                format!(".child({base})"),
                                "child expression base not resolvable in the poc",
                                format!("{}:{}", self.file, base_line),
                            ));
                            return;
                        }
                    }
                }
                self.mapped += 1;
                self.apply_calls(&mut child, calls);
                node.children.push(child);
            }
            GExpr::Match { .. } => match self.resolve_expr_onto(arg) {
                Applied::Calls(calls) => {
                    let mut child = WNode::default();
                    self.mapped += 1;
                    self.apply_calls(&mut child, &calls);
                    node.children.push(child);
                }
                Applied::None => {
                    self.dropped
                        .push(Dropped::new(".child(match ..)", "no arm resolvable", at));
                }
            },
            _ => self.dropped.push(Dropped::new(
                ".child(..)",
                "unsupported child expression",
                at,
            )),
        }
    }

    /// Resolve a combinator body or match into calls applicable to `this`.
    /// Match arms not taken by the target configuration are dropped.
    fn resolve_expr_onto(&mut self, expr: &GExpr) -> Applied {
        match expr {
            GExpr::Chain { base, calls, .. } => {
                if base == "this" {
                    Applied::Calls(calls.clone())
                } else if let Some(mut inner) = self.inline_fn(base) {
                    inner.extend(calls.iter().cloned());
                    Applied::Calls(inner)
                } else {
                    Applied::None
                }
            }
            GExpr::Match {
                scrutinee, arms, ..
            } => {
                let want = TARGET_ARMS
                    .iter()
                    .find(|(s, _)| scrutinee.contains(s))
                    .map(|(_, v)| *v);
                let Some(want) = want else {
                    return Applied::None;
                };
                let mut taken = Applied::None;
                for arm in arms {
                    if arm.pattern.contains(want) {
                        taken = self.resolve_expr_onto(&arm.body);
                    } else {
                        self.dropped.push(Dropped::new(
                            format!("match arm {}", arm.pattern),
                            format!(
                                "variant not in the transpiled instance ({})",
                                "horizontal, solid, with label"
                            ),
                            format!("{}:{}", self.file, expr_line(&arm.body)),
                        ));
                    }
                }
                taken
            }
            GExpr::Closure { body, .. } => self.resolve_expr_onto(body),
            GExpr::Src { .. } => Applied::None,
        }
    }

    /// `Self::render_solid(axis, color)` -> calls of that fn's tail chain
    /// (prefixed by its own base resolution, recursively).
    fn inline_fn(&mut self, base: &str) -> Option<Vec<GCall>> {
        let name = base.strip_prefix("Self::")?.split('(').next()?;
        let f = self.fns.iter().find(|f| f.name == name)?;
        let Some(GExpr::Chain { base, calls, .. }) = &f.tail else {
            return None;
        };
        let mut out = if base == "div()" {
            Vec::new()
        } else {
            self.inline_fn(&base.clone())?
        };
        out.extend(calls.iter().cloned());
        Some(out)
    }

    /// gpui color expressions -> hoff theme tokens (study table).
    fn color_expr(&mut self, arg: &GExpr, at: &str) -> Option<String> {
        let GExpr::Src { text, .. } = arg else {
            return None;
        };
        match text.as_str() {
            "cx.theme().border" => Some("theme.colors.divider".into()),
            "cx.theme().background" => Some("theme.colors.bg".into()),
            "cx.theme().muted_foreground" => Some("theme.colors.text_mid".into()),
            "color" => {
                // let color = self.color.unwrap_or(cx.theme().border);
                if !self.color_noted {
                    self.color_noted = true;
                    self.dropped.push(Dropped::new(
                        "self.color override (color() setter)",
                        "not exercised by the transpiled instance; theme border token used",
                        at,
                    ));
                }
                Some("theme.colors.divider".into())
            }
            _ => None,
        }
    }

    /// The absolute 1px full-width line behind the chip becomes two grow
    /// segments flanking it.
    fn flank_rewrite(&mut self, root: &mut WNode) {
        let line_idx = root
            .children
            .iter()
            .position(|c| c.absolute.is_some() && c.full_main);
        let Some(idx) = line_idx else { return };
        if root.children.len() < 2 {
            return;
        }
        let mut line = root.children.remove(idx);
        self.dropped.push(Dropped::new(
            ".absolute() line overlay",
            "no absolute positioning; rewritten as two grow(1.0) segments flanking the label",
            format!("{}:{}", self.file, line.absolute.unwrap_or(0)),
        ));
        line.absolute = None;
        line.full_main = false;
        line.props.push(Prop::f32("grow", 1.0));
        // h before grow before bg for deterministic output.
        line.props.sort_by_key(|p| match p.name {
            "h" | "w" => 0,
            "grow" => 1,
            _ => 2,
        });
        let second = WNode {
            props: line.props.clone(),
            ..WNode::default()
        };
        root.children.insert(0, line);
        root.children.push(second);
    }

    fn drop_call(&mut self, call: &GCall, at: &str, why: &str) {
        self.dropped
            .push(Dropped::new(format!(".{}(..)", call.method), why, at));
    }
}
