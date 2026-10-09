//! Stage 2 (react): tsx tree + sass rules -> ParserNode tree, params, droplist.
//!
//! Three documented rewrites keep the layout content-driven (the manual's
//! rule) instead of position-based:
//! 1. overlay merge: a first child with `position: absolute; inset: 0` and
//!    no children is the parent's background layer; its visual props move
//!    onto the parent. Same for `:before`/`:after` pseudo layers.
//! 2. spacer rewrite: `margin-bottom: v` becomes a `div().h(v)` sibling
//!    (plev has no per-side margin builder; rhythm is preserved exactly).
//! 3. inheritance push-down: typography declared on a container moves onto
//!    its text runs, one TextStyle per run.
//!
//! Everything else that does not map lands on the droplist with file:line.
//! Every prop carries the declaration it came from and every node the
//! element it is (`ir::Origin`), for the plev IR emitter's report; the code
//! emitter reads neither.

use crate::formats::react::css::{MapOut, is_text_prop, map_decl, noop_is_merge, px};
use crate::formats::react::parse::{TsxChild, TsxComponent, TsxNode};
use crate::formats::react::sass::{Decl, Rule, class_decls};
use crate::tree::{
    Attr, Dropped, Note, NoteKind, Origin, Param, ParserNode, Prop, Resolution, Src, Tag,
    TextValue, snake_case,
};
use std::collections::{HashMap, HashSet};

const OVERLAY_RULE: &str =
    "an inset:0 first child is its parent's background layer (overlay merge)";
const COLLAPSE_RULE: &str = "a div holding only one text run and its text styles is that text run";

pub fn resolve_react(
    comp: &TsxComponent,
    rules: &[Rule],
    tsx_file: &str,
    sass_file: &str,
) -> Resolution {
    let mut cx = Ctx {
        rules,
        tsx_file,
        sass_file,
        mapped: 0,
        dropped: Vec::new(),
        notes: Vec::new(),
        params: Vec::new(),
        applied: HashSet::new(),
        consumed_fragments: Vec::new(),
    };
    let root = match cx.node(&comp.root) {
        Resolved::Node { node, .. } => *node,
        _ => ParserNode::new(Tag::Div, vec![], vec![]),
    };
    cx.sweep_unapplied();
    Resolution {
        fn_name: snake_case(&comp.name),
        name: comp.name.clone(),
        decl: Src::new(
            format!("{tsx_file}:{}", comp.line),
            format!("component {}", comp.name),
        ),
        source_label: format!("{tsx_file} + {sass_file}"),
        root,
        params: cx.params,
        mapped: cx.mapped,
        dropped: cx.dropped,
        notes: cx.notes,
    }
}

enum Resolved {
    Node {
        node: Box<ParserNode>,
        spacer: Option<(f32, Src)>,
    },
    /// `position: absolute; inset: 0` layer: props belong to the parent.
    Overlay(Vec<Prop>, Src),
    Gone,
}

struct Ctx<'a> {
    rules: &'a [Rule],
    tsx_file: &'a str,
    sass_file: &'a str,
    mapped: usize,
    dropped: Vec<Dropped>,
    notes: Vec<Note>,
    params: Vec<Param>,
    applied: HashSet<String>,
    /// `.class` fragments consumed by subtree/conditional drops, each with
    /// the droplist entry that answers for the rules it consumes.
    consumed_fragments: Vec<(String, usize)>,
}

/// `<button class=button>`, or `<input>` without a class.
fn element_label(tsx: &TsxNode) -> String {
    if tsx.classes.is_empty() {
        format!("<{}>", tsx.tag)
    } else {
        format!("<{} class={}>", tsx.tag, tsx.classes.join("."))
    }
}

/// `onClick={onOpen}`, `placeholder="Reply"`, `hidden`.
fn attr_label(name: &str, value: &crate::tree::AttrValue) -> String {
    use crate::tree::AttrValue::*;
    match value {
        Text(t) => format!("{name}=\"{t}\""),
        Ident(e) | Expr(e) => format!("{name}={{{e}}}"),
        Flag => name.to_string(),
    }
}

/// The `file:line` of every construct under `tsx` (its own attributes and
/// conditional classes, then its descendants), for an entry that drops
/// the subtree whole.
fn subtree_sites(tsx: &TsxNode, file: &str, out: &mut Vec<String>) {
    for a in &tsx.attrs {
        out.push(format!("{file}:{}", a.line));
    }
    for (_, _, line) in &tsx.cond_classes {
        out.push(format!("{file}:{line}"));
    }
    for child in &tsx.children {
        match child {
            TsxChild::Node(n) => {
                out.push(format!("{file}:{}", n.line));
                subtree_sites(n, file, out);
            }
            TsxChild::Text(_, line) | TsxChild::Expr(_, line) | TsxChild::Other(_, line) => {
                out.push(format!("{file}:{line}"));
            }
        }
    }
}

impl<'a> Ctx<'a> {
    fn tsx_at(&self, line: usize) -> String {
        format!("{}:{line}", self.tsx_file)
    }

    fn sass_at(&self, line: usize) -> String {
        format!("{}:{line}", self.sass_file)
    }

    fn origin(&self, tsx: &TsxNode) -> Origin {
        Origin {
            src: Some(Src::new(self.tsx_at(tsx.line), element_label(tsx))),
            tag: tsx.tag.clone(),
            name: tsx.classes.first().cloned().unwrap_or_default(),
            attrs: tsx
                .attrs
                .iter()
                .map(|a| Attr {
                    name: a.name.clone(),
                    value: a.value.clone(),
                    src: Src::new(self.tsx_at(a.line), attr_label(&a.name, &a.value)),
                })
                .collect(),
            folded: Vec::new(),
            spacer: false,
        }
    }

    fn node(&mut self, tsx: &TsxNode) -> Resolved {
        for (class, cond, line) in &tsx.cond_classes {
            self.dropped.push(Dropped::new(
                format!("conditional class {class} ({cond})"),
                "prop-driven variant not transpiled in the poc; base instance emitted",
                self.tsx_at(*line),
            ));
            self.consumed_fragments
                .push((format!(".{class}"), self.dropped.len() - 1));
        }

        let own: Vec<Decl> = tsx
            .classes
            .iter()
            .filter_map(|c| {
                class_decls(self.rules, c).map(|r| {
                    self.applied.insert(r.selector.clone());
                    r.decls.clone()
                })
            })
            .flatten()
            .collect();
        let absolute = own
            .iter()
            .any(|d| d.prop == "position" && d.value == "absolute");
        let inset0 = own
            .iter()
            .any(|d| d.prop == "inset" && px(&d.value) == Some(0.0));

        if absolute && !inset0 {
            // Decorative positioned subtree (e.g. the buttonCircle hover
            // animation): nothing of it survives flow layout; drop whole.
            let mut covers: Vec<String> = own.iter().map(|d| self.sass_at(d.line)).collect();
            subtree_sites(tsx, self.tsx_file, &mut covers);
            self.dropped.push(
                Dropped::new(
                    format!("<{} class={}> subtree", tsx.tag, tsx.classes.join(".")),
                    "absolutely positioned decorative subtree; no flow equivalent in plev",
                    self.tsx_at(tsx.line),
                )
                .covering(covers),
            );
            let at = self.dropped.len() - 1;
            for class in &tsx.classes {
                self.consumed_fragments.push((format!(".{class}"), at));
            }
            return Resolved::Gone;
        }

        let mut props: Vec<Prop> = Vec::new();
        let label = tsx
            .classes
            .first()
            .map(|c| format!(".{c}"))
            .unwrap_or_else(|| format!("<{}>", tsx.tag));
        let merge = absolute && inset0;
        let spacer = self.apply_decls(&mut props, &label, &own, merge);
        for class in &tsx.classes {
            self.merge_pseudo_layers(&mut props, class);
        }
        let mut origin = self.origin(tsx);
        if merge && tsx.children.is_empty() {
            let src = origin.src.unwrap_or_default();
            return Resolved::Overlay(props, src);
        }

        let (mut node_props, text_props): (Vec<Prop>, Vec<Prop>) =
            props.into_iter().partition(|p| !is_text_prop(p.name));
        let mut children: Vec<ParserNode> = Vec::new();
        let mut first = true;
        for child in &tsx.children {
            match child {
                TsxChild::Node(n) => match self.node(n) {
                    Resolved::Node { node, spacer } => {
                        children.push(*node);
                        if let Some((h, src)) = spacer {
                            children.push(spacer_div(h, src));
                        }
                    }
                    Resolved::Overlay(overlay_props, overlay) => {
                        if first {
                            for p in overlay_props {
                                match node_props.iter_mut().find(|q| **q == p) {
                                    Some(q) => q.src.extend(p.src),
                                    None => node_props.push(p),
                                }
                            }
                            origin.folded.push((overlay, OVERLAY_RULE));
                        } else {
                            let covers = overlay_props
                                .iter()
                                .flat_map(|p| p.src.iter().map(|s| s.at.clone()))
                                .collect();
                            self.dropped.push(
                                Dropped::new(
                                    format!("<{} class={}> overlay", n.tag, n.classes.join(".")),
                                    "inset:0 overlay only merges as first child",
                                    self.tsx_at(n.line),
                                )
                                .covering(covers),
                            );
                        }
                    }
                    Resolved::Gone => {}
                },
                TsxChild::Text(t, line) => {
                    let mut text = ParserNode::new(
                        Tag::Text(TextValue::Literal(t.clone())),
                        vec![],
                        vec![],
                    );
                    text.origin.src = Some(Src::new(self.tsx_at(*line), format!("\"{t}\"")));
                    text.origin.tag = "text".into();
                    children.push(text);
                }
                TsxChild::Expr(name, line) => {
                    let tag = if text_props.is_empty() {
                        self.params.push(Param::Slot(name.clone()));
                        Tag::Slot(name.clone())
                    } else {
                        self.params.push(Param::Text(name.clone()));
                        Tag::Text(TextValue::Param(name.clone()))
                    };
                    let mut expr = ParserNode::new(tag, vec![], vec![]);
                    expr.origin.src = Some(Src::new(self.tsx_at(*line), format!("{{{name}}}")));
                    expr.origin.tag = "expression".into();
                    expr.origin.name = name.clone();
                    children.push(expr);
                }
                TsxChild::Other(expr, line) => self.dropped.push(Dropped::new(
                    format!("{{{expr}}}"),
                    "only {identifier} children are transpiled; an expression (a call, a condition, a list) is not",
                    self.tsx_at(*line),
                )),
            }
            first = false;
        }

        // Inheritance push-down: container typography lands on its text runs.
        let mut landed = vec![false; text_props.len()];
        if !text_props.is_empty() {
            for child in &mut children {
                if let Tag::Text(_) = child.tag {
                    for (i, p) in text_props.iter().enumerate() {
                        if !child.props.iter().any(|q| q.name == p.name) {
                            child.props.push(p.clone());
                            landed[i] = true;
                        }
                    }
                }
            }
        }
        self.drop_unlanded(&text_props, &landed, &children);
        // A styleless div around a single text run IS that text run.
        if node_props.is_empty()
            && children.len() == 1
            && let Tag::Text(_) = children[0].tag
        {
            let mut text = children.remove(0);
            if let Some(div) = origin.src.take() {
                text.origin.folded.push((div, COLLAPSE_RULE));
            }
            text.origin.folded.append(&mut origin.folded);
            origin.attrs.append(&mut text.origin.attrs);
            text.origin.attrs = origin.attrs;
            // the run is the element now: a <button> around its label
            // stays a button for the IR emitter
            text.origin.tag = origin.tag;
            if text.origin.name.is_empty() {
                text.origin.name = origin.name;
            }
            return Resolved::Node {
                node: Box::new(text),
                spacer,
            };
        }
        Resolved::Node {
            node: Box::new(ParserNode {
                tag: Tag::Div,
                props: node_props,
                children,
                origin,
            }),
            spacer,
        }
    }

    /// A text style no text run took (none to take it, or each sets its
    /// own) is carried by nothing: each of its declarations is dropped,
    /// and no longer counted as mapped.
    fn drop_unlanded(&mut self, text_props: &[Prop], landed: &[bool], children: &[ParserNode]) {
        let has_text = children.iter().any(|c| matches!(c.tag, Tag::Text(_)));
        for (p, _) in text_props.iter().zip(landed).filter(|(_, l)| !**l) {
            let why = if has_text {
                "every text run of the element sets its own; nothing carries it"
            } else {
                "a text style on an element with no text run of its own; nothing carries it"
            };
            for src in &p.src {
                self.mapped = self.mapped.saturating_sub(1);
                self.dropped
                    .push(Dropped::new(src.what.clone(), why, src.at.clone()));
            }
        }
    }

    /// Map decls onto `props`; returns the spacer height (with its
    /// declaration) when a `margin-bottom` was rewritten. `merge` marks
    /// overlay/pseudo context where `position: absolute` is consumed by the
    /// merge itself.
    fn apply_decls(
        &mut self,
        props: &mut Vec<Prop>,
        sel: &str,
        decls: &[Decl],
        merge: bool,
    ) -> Option<(f32, Src)> {
        let font_size = decls
            .iter()
            .find(|d| d.prop == "font-size")
            .and_then(|d| px(&d.value));
        let mut spacer = None;
        for d in decls {
            let src = Src::new(
                self.sass_at(d.line),
                format!("{sel} {{ {}: {} }}", d.prop, d.value),
            );
            if merge && d.prop == "position" && d.value == "absolute" {
                self.mapped += 1; // consumed by the overlay-merge rewrite
                self.note(src, NoteKind::Rewrite(OVERLAY_RULE));
                continue;
            }
            match map_decl(&d.prop, &d.value, font_size) {
                MapOut::Props(new) => {
                    let mut agreed = true;
                    for p in new {
                        let p = p.from(src.clone());
                        match props.iter_mut().find(|q| q.name == p.name) {
                            None => props.push(p),
                            // agreement, e.g. repeated radius
                            Some(q) if *q == p => q.src.push(src.clone()),
                            Some(_) => agreed = false,
                        }
                    }
                    if agreed {
                        self.mapped += 1;
                    } else {
                        self.drop_decl(sel, d, "conflicts with an earlier declaration; first wins");
                    }
                }
                MapOut::Noop(why) => {
                    self.mapped += 1;
                    let kind = if noop_is_merge(&d.prop) {
                        NoteKind::Rewrite(why)
                    } else {
                        NoteKind::Holds(why)
                    };
                    self.note(src, kind);
                }
                MapOut::SpacerAfter(h) => {
                    spacer = Some((h, src));
                    self.mapped += 1;
                }
                MapOut::Drop(why) => self.drop_decl(sel, d, why),
            }
        }
        spacer
    }

    fn note(&mut self, src: Src, kind: NoteKind) {
        self.notes.push(Note { src, kind });
    }

    /// `:before`/`:after` rules of `class` that are inset:0 layers merge
    /// into the host; any other pseudo rule is dropped whole.
    fn merge_pseudo_layers(&mut self, props: &mut Vec<Prop>, class: &str) {
        for pseudo in ["before", "after"] {
            let sel = format!(".{class}:{pseudo}");
            let Some(rule) = self.rules.iter().find(|r| r.selector == sel) else {
                continue;
            };
            self.applied.insert(sel.clone());
            let overlay = rule
                .decls
                .iter()
                .any(|d| d.prop == "inset" && px(&d.value) == Some(0.0));
            if overlay {
                self.apply_decls(props, &sel, &rule.decls.clone(), true);
            } else {
                let covers = rule.decls.iter().map(|d| self.sass_at(d.line)).collect();
                self.dropped.push(
                    Dropped::new(
                        sel.clone(),
                        "pseudo-element outside the inset:0 overlay pattern",
                        self.sass_at(rule.line),
                    )
                    .covering(covers),
                );
            }
        }
    }

    /// Every rule never applied nor consumed becomes a droplist entry, so
    /// nothing in the stylesheet disappears silently. Each entry lists the
    /// declarations it answers for.
    fn sweep_unapplied(&mut self) {
        let mut keyframes_seen: HashMap<String, usize> = HashMap::new();
        for rule in self.rules {
            let mut sel = rule.selector.as_str();
            if self.applied.contains(sel) {
                continue;
            }
            let decl_sites: Vec<String> = rule.decls.iter().map(|d| self.sass_at(d.line)).collect();
            if let Some(&(_, at)) = self
                .consumed_fragments
                .iter()
                .find(|(f, _)| sel.contains(f.as_str()))
            {
                // accounted by a subtree/conditional drop entry
                if let Some(entry) = self.dropped.get_mut(at) {
                    entry.covers.extend(decl_sites);
                }
                continue;
            }
            // One entry per animation, not per keyframe block.
            let keyframes_name;
            if sel.starts_with("@keyframes") {
                keyframes_name = sel.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
                if let Some(&at) = keyframes_seen.get(&keyframes_name) {
                    if let Some(entry) = self.dropped.get_mut(at) {
                        entry.covers.extend(decl_sites);
                    }
                    continue;
                }
                keyframes_seen.insert(keyframes_name.clone(), self.dropped.len());
                sel = &keyframes_name;
            }
            let why = if sel.starts_with("@media") {
                "responsive variant out of poc scope; desktop instance transpiled"
            } else if sel.starts_with("@keyframes") {
                "keyframe animation has no builder equivalent"
            } else if sel.contains(":hover") {
                "interactive state styles out of poc scope"
            } else if sel.contains('*') {
                "universal selector unsupported; box-sizing is border-box by construction"
            } else if sel.contains(' ') {
                "descendant selector unsupported by the poc resolver"
            } else {
                "modifier variant class not present on the transpiled instance"
            };
            self.dropped
                .push(Dropped::new(sel, why, self.sass_at(rule.line)).covering(decl_sites));
        }
    }

    fn drop_decl(&mut self, sel: &str, d: &Decl, why: &str) {
        self.dropped.push(Dropped::new(
            format!("{sel} {{ {}: {} }}", d.prop, d.value),
            why,
            self.sass_at(d.line),
        ));
    }
}

fn spacer_div(h: f32, src: Src) -> ParserNode {
    let mut node = ParserNode::new(Tag::Div, vec![Prop::f32("h", h).from(src)], vec![]);
    node.origin.spacer = true;
    node
}
