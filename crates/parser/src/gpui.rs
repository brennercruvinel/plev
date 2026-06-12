//! Stage 1 (gpui): rust source -> the builder-call chains of every fn,
//! as an owned mini-AST. Chains keep method names, argument expressions
//! (closures and matches structurally, everything else as source text) and
//! line numbers, so the resolver can pick variant arms and report drops
//! with file:line.

use crate::ParserError;
use tree_sitter::Node;

#[derive(Debug, Clone)]
pub enum GExpr {
    /// `base.m1(a).m2(b)...` with base as source text (`div()`, `self.base`,
    /// `Self::render_base(axis)`).
    Chain {
        base: String,
        base_line: usize,
        calls: Vec<GCall>,
    },
    Match {
        scrutinee: String,
        arms: Vec<GArm>,
        line: usize,
    },
    Closure {
        body: Box<GExpr>,
        line: usize,
    },
    /// Any other expression, kept verbatim.
    Src {
        text: String,
        line: usize,
    },
}

#[derive(Debug, Clone)]
pub struct GCall {
    pub method: String,
    pub args: Vec<GExpr>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct GArm {
    pub pattern: String,
    pub body: GExpr,
}

#[derive(Debug)]
pub struct GFn {
    pub name: String,
    pub line: usize,
    /// The fn's trailing expression (builder fns end in their chain).
    pub tail: Option<GExpr>,
}

#[derive(Debug)]
pub struct GpuiSource {
    /// Name of the widget struct from `impl RenderOnce for X`.
    pub widget: Option<String>,
    pub fns: Vec<GFn>,
}

pub fn parse_gpui(src: &str) -> Result<GpuiSource, ParserError> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .map_err(|e| ParserError::Parse(format!("rust grammar: {e}")))?;
    let tree = parser
        .parse(src, None)
        .ok_or_else(|| ParserError::Parse("tree-sitter returned no rust tree".into()))?;
    if tree.root_node().has_error() {
        return Err(ParserError::Parse("rust source has syntax errors".into()));
    }
    let mut out = GpuiSource {
        widget: None,
        fns: Vec::new(),
    };
    collect(tree.root_node(), src, &mut out);
    if out.fns.is_empty() {
        return Err(ParserError::Parse(
            "no functions found in gpui source".into(),
        ));
    }
    Ok(out)
}

fn text<'a>(node: Node, src: &'a str) -> &'a str {
    node.utf8_text(src.as_bytes()).unwrap_or("")
}

fn line(node: Node) -> usize {
    node.start_position().row + 1
}

fn collect(node: Node, src: &str, out: &mut GpuiSource) {
    if node.kind() == "impl_item" {
        let is_render_once = node
            .child_by_field_name("trait")
            .is_some_and(|t| text(t, src).contains("RenderOnce"));
        if is_render_once && let Some(ty) = node.child_by_field_name("type") {
            out.widget = Some(text(ty, src).to_string());
        }
    }
    if node.kind() == "function_item" {
        let name = node
            .child_by_field_name("name")
            .map(|n| text(n, src).to_string())
            .unwrap_or_default();
        let tail = node
            .child_by_field_name("body")
            .and_then(|b| tail_expr(b))
            .map(|e| convert(e, src));
        out.fns.push(GFn {
            name,
            line: line(node),
            tail,
        });
    }
    let mut cursor = node.walk();
    let children: Vec<Node> = node.named_children(&mut cursor).collect();
    for child in children {
        collect(child, src, out);
    }
}

/// The trailing expression of a block (the implicit return).
fn tail_expr(block: Node) -> Option<Node> {
    let mut cursor = block.walk();
    let children: Vec<Node> = block.named_children(&mut cursor).collect();
    children.into_iter().rev().find(|n| {
        !matches!(
            n.kind(),
            "let_declaration" | "line_comment" | "block_comment"
        )
    })
}

fn convert(node: Node, src: &str) -> GExpr {
    match node.kind() {
        "call_expression" | "field_expression" => convert_chain(node, src),
        "match_expression" => {
            let scrutinee = node
                .child_by_field_name("value")
                .map(|v| text(v, src).to_string())
                .unwrap_or_default();
            let mut arms = Vec::new();
            if let Some(body) = node.child_by_field_name("body") {
                let mut cursor = body.walk();
                for arm in body.named_children(&mut cursor) {
                    if arm.kind() != "match_arm" {
                        continue;
                    }
                    let pattern = arm
                        .child_by_field_name("pattern")
                        .map(|p| text(p, src).to_string())
                        .unwrap_or_default();
                    let value = arm
                        .child_by_field_name("value")
                        .map(|v| convert(v, src))
                        .unwrap_or(GExpr::Src {
                            text: String::new(),
                            line: line(arm),
                        });
                    arms.push(GArm {
                        pattern,
                        body: value,
                    });
                }
            }
            GExpr::Match {
                scrutinee,
                arms,
                line: line(node),
            }
        }
        "closure_expression" => {
            let body = node
                .child_by_field_name("body")
                .map(|b| convert(b, src))
                .unwrap_or(GExpr::Src {
                    text: String::new(),
                    line: line(node),
                });
            GExpr::Closure {
                body: Box::new(body),
                line: line(node),
            }
        }
        "parenthesized_expression" | "block" => match tail_of(node) {
            Some(inner) => convert(inner, src),
            None => GExpr::Src {
                text: text(node, src).to_string(),
                line: line(node),
            },
        },
        _ => GExpr::Src {
            text: text(node, src).to_string(),
            line: line(node),
        },
    }
}

fn tail_of(node: Node) -> Option<Node> {
    let mut cursor = node.walk();
    let children: Vec<Node> = node.named_children(&mut cursor).collect();
    children.into_iter().next_back()
}

/// Flatten the `recv.method(args)` spine into base + ordered calls.
fn convert_chain(node: Node, src: &str) -> GExpr {
    let mut calls_rev: Vec<GCall> = Vec::new();
    let mut cur = node;
    loop {
        if cur.kind() == "call_expression" {
            let func = match cur.child_by_field_name("function") {
                Some(f) => f,
                None => break,
            };
            if func.kind() == "field_expression" {
                let field = func.child_by_field_name("field");
                let method = field.map(|f| text(f, src).to_string()).unwrap_or_default();
                // The line of the method name itself, not of the receiver
                // (chains span many lines; droplist locations must be exact).
                let method_line = field.map(line).unwrap_or_else(|| line(func));
                let mut args = Vec::new();
                if let Some(arglist) = cur.child_by_field_name("arguments") {
                    let mut cursor = arglist.walk();
                    for arg in arglist.named_children(&mut cursor) {
                        args.push(convert(arg, src));
                    }
                }
                calls_rev.push(GCall {
                    method,
                    args,
                    line: method_line,
                });
                cur = match func.child_by_field_name("value") {
                    Some(v) => v,
                    None => break,
                };
                continue;
            }
        }
        break;
    }
    if calls_rev.is_empty() {
        return GExpr::Src {
            text: text(node, src).to_string(),
            line: line(node),
        };
    }
    calls_rev.reverse();
    GExpr::Chain {
        base: text(cur, src).to_string(),
        base_line: line(cur),
        calls: calls_rev,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chain_match_and_closure() {
        let src = r#"
impl RenderOnce for Widget {
    fn render(self) -> Div {
        self.base
            .flex()
            .map(|this| match axis {
                Axis::Vertical => this.w(px(1.)),
                Axis::Horizontal => this.h(px(1.)),
            })
            .child(div().px_2().child(label))
    }
}
"#;
        let parsed = parse_gpui(src).unwrap();
        assert_eq!(parsed.widget.as_deref(), Some("Widget"));
        let render = parsed.fns.iter().find(|f| f.name == "render").unwrap();
        let Some(GExpr::Chain { base, calls, .. }) = &render.tail else {
            panic!("expected chain tail");
        };
        assert_eq!(base, "self.base");
        let names: Vec<&str> = calls.iter().map(|c| c.method.as_str()).collect();
        assert_eq!(names, vec!["flex", "map", "child"]);
        let GExpr::Closure { body, .. } = &calls[1].args[0] else {
            panic!("expected closure arg");
        };
        let GExpr::Match { arms, .. } = body.as_ref() else {
            panic!("expected match body");
        };
        assert_eq!(arms.len(), 2);
    }

    #[test]
    fn rejects_broken_rust() {
        assert!(parse_gpui("fn broken( {{{").is_err());
    }
}
