//! Stage 3, the second emitter: ParserNode tree -> plev IR (`uir::Ir`),
//! next to the code emitter (`emit.rs`), which stays until a decision
//! removes it. It reads the same `Resolution` and writes what the IR can
//! say, never a pixel or a color: every value goes to a token by the
//! measure of `tokens`, and every construct of the source to the report
//! (`report`), in exactly one of four categories, at its `file:line`.
//!
//! The mapping, by what the `ParserNode` holds:
//!
//! - a `div` is a container (a `Group`); `row` and `align_center` its stack;
//!   the spacers of the margin-bottom rewrite its gap (the smallest one's
//!   token); a bar as thin as the separator's line in the divider color is
//!   the `separator` component;
//! - a text run is `text`: its literal is its label, a text parameter is
//!   bound to `<surface>.props:<parameter>`, a state the importer names on
//!   the app's node `<surface>.props`: a binding the app has to make,
//!   listed in the report;
//! - an element slot is left out: the IR holds no element the caller
//!   builds at run time;
//! - ids come from the source's class names (or the kind of node), under
//!   `<surface>.root`, made unique among siblings.
//!
//! The IR is built with the `uir` types; ids and references are written as
//! text and read by the `sem` grammar (`parse_as`), so an id the
//! sanitizer lets through and the grammar refuses is an error, never a
//! panic.

mod props;

use std::collections::BTreeSet;
use std::str::FromStr;

use uir::{Axis, Component, Ir, IrNode, Layout, Surface};

use crate::ParserError;
use crate::ir::{Arg, DropKind, NoteKind, ParserNode, Resolution, Src, Tag, TextValue, snake_case};
use crate::report::{Binding, Ledger, Need, Report, Verdict};
use crate::tokens::{Tokens, color_distance, role_named};
use props::{Decision, flex, num};

/// What an import gives: the IR and the truth about it.
#[derive(Clone, Debug, PartialEq)]
pub struct Imported {
    pub ir: Ir,
    pub report: Report,
}

/// The plev IR of `res`, with its report, measured against `tokens`.
pub fn emit_ir(res: &Resolution, tokens: &Tokens) -> Result<Imported, ParserError> {
    let surface = sanitize(&res.fn_name, "surface");
    let mut cx = Cx {
        tokens,
        surface,
        ledger: Ledger::default(),
        bindings: Vec::new(),
    };
    let root_id = format!("{}.root", cx.surface);
    let root = match cx.node(&res.root, &root_id, Axis::Vertical)? {
        Some(node) => node,
        None => cx.new_node(&root_id)?,
    };
    cx.ledger.add(
        &res.decl,
        None,
        Verdict::Transformed {
            ir: format!("surface `{}`, title \"{}\"", cx.surface, res.name),
            rule: "the component names the surface".into(),
        },
    );
    for d in &res.dropped {
        let verdict = match d.kind {
            DropKind::Lost => Verdict::Impossible {
                reason: d.why.clone(),
            },
            DropKind::Rewritten => Verdict::Transformed {
                ir: "carried by the rewrite".into(),
                rule: d.why.clone(),
            },
        };
        let src = Src::new(d.at.clone(), d.what.clone());
        cx.ledger
            .add_covering(&src, None, verdict, d.covers.clone());
    }
    for n in &res.notes {
        let verdict = match n.kind {
            NoteKind::Holds(why) => Verdict::Supported {
                ir: format!("holds by construction: {why}"),
            },
            NoteKind::Rewrite(rule) => Verdict::Transformed {
                ir: "carried by the rewrite".into(),
                rule: rule.into(),
            },
        };
        cx.ledger.add(&n.src, None, verdict);
    }
    let ir = Ir::new(Surface {
        id: parse_as(&cx.surface)?,
        title: res.name.clone(),
        root,
    });
    let report = Report {
        source: res.source_label.clone(),
        surface: cx.surface.clone(),
        entries: cx.ledger.finish(),
        bindings: cx.bindings,
    };
    Ok(Imported { ir, report })
}

/// Reads `text` as an id or a reference of the `sem` grammar.
pub(crate) fn parse_as<T: FromStr>(text: &str) -> Result<T, ParserError>
where
    T::Err: std::fmt::Display,
{
    text.parse()
        .map_err(|e| ParserError::Ir(format!("`{text}`: {e}")))
}

/// A name segment of the `sem` grammar (`[a-z0-9_]+`) from any text:
/// `hoff-research-card` -> `hoff_research_card`; `fallback` when nothing is
/// left.
pub(crate) fn sanitize(raw: &str, fallback: &str) -> String {
    let mut out = String::new();
    for c in raw.chars().map(|c| c.to_ascii_lowercase()) {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    let name = out.trim_matches('_');
    if name.is_empty() {
        fallback.to_string()
    } else {
        name.to_string()
    }
}

pub(crate) struct Cx<'a> {
    tokens: &'a Tokens,
    surface: String,
    ledger: Ledger,
    bindings: Vec<Binding>,
}

impl Cx<'_> {
    fn new_node(&self, id: &str) -> Result<IrNode, ParserError> {
        Ok(IrNode::new(parse_as(id)?))
    }

    /// The app's node every binding of the surface names.
    fn props_id(&self) -> String {
        format!("{}.props", self.surface)
    }

    fn decided(&mut self, d: &Decision, id: &str) {
        for s in &d.srcs {
            self.ledger.add(s, Some(id), d.verdict.clone());
        }
    }

    /// The node's own construct with `verdict`, and the ones a rewrite
    /// folded into it, each with its rule.
    fn origin_entries(&mut self, p: &ParserNode, id: &str, verdict: Verdict) {
        if let Some(src) = &p.origin.src {
            self.ledger.add(src, Some(id), verdict);
        }
        for (src, rule) in &p.origin.folded {
            let folded = Verdict::Transformed {
                ir: format!("folded into `{id}`"),
                rule: (*rule).into(),
            };
            self.ledger.add(src, Some(id), folded);
        }
    }

    /// The node's attributes: the ones `used` names with their verdict,
    /// the rest lost.
    fn attrs(&mut self, p: &ParserNode, id: &str, used: &[(&str, Verdict)]) {
        for a in &p.origin.attrs {
            let verdict = match used.iter().find(|(n, _)| *n == a.name) {
                Some((_, v)) => v.clone(),
                None => Verdict::Impossible {
                    reason: format!("the attribute `{}` is not carried by the IR", a.name),
                },
            };
            self.ledger.add(&a.src, Some(id), verdict);
        }
    }

    /// Every construct under `p` is lost for `reason`.
    fn lost_subtree(&mut self, p: &ParserNode, reason: &str) {
        let verdict = Verdict::Impossible {
            reason: reason.into(),
        };
        let mut srcs: Vec<Src> = Vec::new();
        collect(p, &mut srcs);
        for s in &srcs {
            self.ledger.add(s, None, verdict.clone());
        }
    }

    fn node(
        &mut self,
        p: &ParserNode,
        id: &str,
        axis: Axis,
    ) -> Result<Option<IrNode>, ParserError> {
        match &p.tag {
            Tag::Slot(name) => {
                let reason = format!(
                    "`{{{name}}}` is an element the caller builds at run time: the IR holds no slot for one"
                );
                self.lost_subtree(p, &reason);
                Ok(None)
            }
            Tag::Text(value) => self.text_node(p, id, value).map(Some),
            Tag::Div if self.is_separator(p, axis) => self.separator(p, id).map(Some),
            Tag::Div => self.container(p, id).map(Some),
        }
    }

    fn text_node(
        &mut self,
        p: &ParserNode,
        id: &str,
        value: &TextValue,
    ) -> Result<IrNode, ParserError> {
        let mut node = self.new_node(id)?;
        node.component = Some(Component::Text);
        match value {
            TextValue::Literal(s) => {
                node.label = Some(s.clone());
                let verdict = Verdict::Supported {
                    ir: "component text, its label".into(),
                };
                self.origin_entries(p, id, verdict);
            }
            TextValue::Param(name) => {
                let key = sanitize(&snake_case(name), "text");
                let reference = format!("{}:{key}", self.props_id());
                node.bind.state.insert("text".into(), parse_as(&reference)?);
                let verdict = Verdict::Transformed {
                    ir: "component text, bind.state.text".into(),
                    rule: "a text parameter is text the app's state holds: binding required".into(),
                };
                self.origin_entries(p, id, verdict);
                self.bindings.push(Binding {
                    at: p
                        .origin
                        .src
                        .as_ref()
                        .map(|s| s.at.clone())
                        .unwrap_or_default(),
                    node: id.into(),
                    need: Need::State {
                        prop: "text".into(),
                        reference,
                        kind: "text".into(),
                    },
                    why: format!("`{name}` is a parameter of the component"),
                });
            }
        }
        self.attrs(p, id, &[]);
        let (props, decisions) = props::text(&p.props, self.tokens);
        node.props = props;
        for d in &decisions {
            self.decided(d, id);
        }
        Ok(node)
    }

    /// A `div` with no children, one side as thin as the separator's line
    /// (`theme.control.edge_width`), the divider color, and a shape the
    /// separator draws along `axis`: horizontal in a column, or in a row
    /// when it grows; vertical in a row.
    fn is_separator(&self, p: &ParserNode, axis: Axis) -> bool {
        let edge = self.tokens.edge_width;
        if !p.children.is_empty() || edge <= 0.0 {
            return false;
        }
        let (mut thin_h, mut thin_w, mut divider, mut grows) = (false, false, false, false);
        for prop in &p.props {
            let arg = prop.args.first();
            match (prop.name, arg) {
                ("h", Some(Arg::F32(v))) => thin_h = *v == edge,
                ("w", Some(Arg::F32(v))) => thin_w = *v == edge,
                ("grow", Some(Arg::F32(v))) => grows = *v > 0.0,
                ("bg", Some(Arg::Token(e))) => {
                    let want = role_named("divider").and_then(|r| self.tokens.color(r));
                    let got = self.tokens.color_of(e);
                    divider =
                        matches!((want, got), (Some(a), Some(b)) if color_distance(&a, &b) == 0.0);
                }
                _ => return false,
            }
        }
        let across = thin_h && !thin_w && (axis == Axis::Vertical || grows);
        let down = thin_w && !thin_h && axis == Axis::Horizontal;
        divider && (across || down)
    }

    fn separator(&mut self, p: &ParserNode, id: &str) -> Result<IrNode, ParserError> {
        let mut node = self.new_node(id)?;
        node.component = Some(Component::Separator);
        let rule = format!(
            "a bar {} px thin in theme.colors.divider is the separator component, whose line is theme.control.edge_width in that color",
            num(self.tokens.edge_width)
        );
        let verdict = Verdict::Transformed {
            ir: "component separator".into(),
            rule: rule.clone(),
        };
        self.origin_entries(p, id, verdict.clone());
        self.attrs(p, id, &[]);
        for prop in &p.props {
            if prop.name == "grow" {
                let (v, d) = flex(prop);
                node.size.flex = v;
                self.decided(&d, id);
            } else {
                let d = Decision {
                    srcs: prop.src.clone(),
                    verdict: verdict.clone(),
                };
                self.decided(&d, id);
            }
        }
        Ok(node)
    }

    fn container(&mut self, p: &ParserNode, id: &str) -> Result<IrNode, ParserError> {
        let mut node = self.new_node(id)?;
        let verdict = Verdict::Supported {
            ir: "a container (no component: a group)".into(),
        };
        self.origin_entries(p, id, verdict);
        self.attrs(p, id, &[]);
        let c = props::container(&p.props, self.tokens);
        node.props = c.props;
        node.size = c.size;
        for d in &c.decisions {
            self.decided(d, id);
        }
        let axis = if c.row {
            Axis::Horizontal
        } else {
            Axis::Vertical
        };
        let mut taken = BTreeSet::new();
        let mut kids: Vec<IrNode> = Vec::new();
        // the spacing between consecutive children, and the spacers with
        // no child before or after them
        let mut pairs: Vec<(f32, Vec<Src>)> = Vec::new();
        let mut pending: (f32, Vec<Src>) = (0.0, Vec::new());
        let mut stray: Vec<Src> = Vec::new();
        for child in &p.children {
            if child.origin.spacer {
                for prop in &child.props {
                    if let (Some(Arg::F32(h)), "h") = (prop.args.first(), prop.name) {
                        pending.0 += h;
                    }
                    pending.1.extend(prop.src.iter().cloned());
                }
                continue;
            }
            let cid = self.child_id(id, child, axis, &mut taken);
            if let Some(ir) = self.node(child, &cid, axis)? {
                let spacing = std::mem::take(&mut pending);
                if kids.is_empty() {
                    stray.extend(spacing.1);
                } else {
                    pairs.push(spacing);
                }
                kids.push(ir);
            }
        }
        stray.extend(pending.1);
        let mut gap = None;
        if c.row {
            let reason =
                "a margin below a child of a row has no IR form: the row's gap runs across";
            for s in pairs.iter().flat_map(|(_, s)| s) {
                self.ledger.add(
                    s,
                    Some(id),
                    Verdict::Impossible {
                        reason: reason.into(),
                    },
                );
            }
        } else {
            let (g, decisions) = props::gap(&pairs, self.tokens);
            gap = g;
            for d in &decisions {
                self.decided(d, id);
            }
        }
        let reason =
            "a margin with no child after it has no IR form: the IR's gap is between children only";
        for s in &stray {
            self.ledger.add(
                s,
                Some(id),
                Verdict::Impossible {
                    reason: reason.into(),
                },
            );
        }
        if c.row || gap.is_some() || c.align.is_some() {
            node.layout = Some(Layout::Stack {
                axis,
                gap,
                align: c.align,
            });
        }
        node.children = kids;
        Ok(node)
    }

    /// `<parent>.<name>`: the child's class name, or the kind of node it
    /// becomes; `_2`, `_3` when a sibling took it.
    fn child_id(
        &self,
        parent: &str,
        child: &ParserNode,
        axis: Axis,
        taken: &mut BTreeSet<String>,
    ) -> String {
        let hint = match &child.tag {
            Tag::Text(_) => "text",
            Tag::Slot(_) => "slot",
            Tag::Div if self.is_separator(child, axis) => "line",
            _ => "box",
        };
        let base = sanitize(&snake_case(&child.origin.name), hint);
        let mut id = format!("{parent}.{base}");
        let mut n = 2;
        while taken.contains(&id) {
            id = format!("{parent}.{base}_{n}");
            n += 1;
        }
        taken.insert(id.clone());
        id
    }
}

/// Every source construct of a subtree.
fn collect(p: &ParserNode, out: &mut Vec<Src>) {
    out.extend(p.origin.src.iter().cloned());
    out.extend(p.origin.folded.iter().map(|(s, _)| s.clone()));
    out.extend(p.origin.attrs.iter().map(|a| a.src.clone()));
    for prop in &p.props {
        out.extend(prop.src.iter().cloned());
    }
    for c in &p.children {
        collect(c, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_of_the_id_grammar_from_any_text() {
        assert_eq!(sanitize("hoff-research-card", "x"), "hoff_research_card");
        assert_eq!(sanitize("button_title", "x"), "button_title");
        assert_eq!(sanitize("--", "box"), "box");
        assert_eq!(sanitize("Card 2!", "x"), "card_2");
        assert!(parse_as("a.b_2").map(IrNode::new).is_ok());
        let refused = parse_as("A.b").map(IrNode::new);
        assert!(matches!(refused, Err(ParserError::Ir(e)) if e.starts_with("`A.b`")));
    }
}
