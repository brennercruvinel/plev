//! Stage 2 (react): tsx tree + sass rules -> PrsNode tree, params, droplist.
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

use crate::css_map::{MapOut, is_text_prop, map_decl, px};
use crate::ir::{Dropped, Param, Prop, PrsNode, Resolution, Tag, TextValue, snake_case};
use crate::sass::{Decl, Rule, class_decls};
use crate::tsx::{TsxChild, TsxComponent, TsxNode};
use std::collections::HashSet;

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
        params: Vec::new(),
        applied: HashSet::new(),
        consumed_fragments: Vec::new(),
    };
    let root = match cx.node(&comp.root) {
        Resolved::Node { node, .. } => node,
        _ => PrsNode {
            tag: Tag::Div,
            props: vec![],
            children: vec![],
        },
    };
    cx.sweep_unapplied();
    Resolution {
        fn_name: snake_case(&comp.name),
        source_label: format!("{tsx_file} + {sass_file}"),
        root,
        params: cx.params,
        mapped: cx.mapped,
        dropped: cx.dropped,
    }
}

enum Resolved {
    Node {
        node: PrsNode,
        spacer: Option<f32>,
    },
    /// `position: absolute; inset: 0` layer: props belong to the parent.
    Overlay(Vec<Prop>),
    Gone,
}

struct Ctx<'a> {
    rules: &'a [Rule],
    tsx_file: &'a str,
    sass_file: &'a str,
    mapped: usize,
    dropped: Vec<Dropped>,
    params: Vec<Param>,
    applied: HashSet<String>,
    /// `.class` fragments consumed by subtree/conditional drops.
    consumed_fragments: Vec<String>,
}

impl<'a> Ctx<'a> {
    fn node(&mut self, tsx: &TsxNode) -> Resolved {
        for (class, cond, line) in &tsx.cond_classes {
            self.dropped.push(Dropped::new(
                format!("conditional class {class} ({cond})"),
                "prop-driven variant not transpiled in the poc; base instance emitted",
                format!("{}:{line}", self.tsx_file),
            ));
            self.consumed_fragments.push(format!(".{class}"));
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
            for class in &tsx.classes {
                self.consumed_fragments.push(format!(".{class}"));
            }
            self.dropped.push(Dropped::new(
                format!("<{} class={}> subtree", tsx.tag, tsx.classes.join(".")),
                "absolutely positioned decorative subtree; no flow equivalent in plev",
                format!("{}:{}", self.tsx_file, tsx.line),
            ));
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
        if merge && tsx.children.is_empty() {
            return Resolved::Overlay(props);
        }

        let (mut node_props, text_props): (Vec<Prop>, Vec<Prop>) =
            props.into_iter().partition(|p| !is_text_prop(p.name));
        let mut children: Vec<PrsNode> = Vec::new();
        let mut first = true;
        for child in &tsx.children {
            match child {
                TsxChild::Node(n) => match self.node(n) {
                    Resolved::Node { node, spacer } => {
                        children.push(node);
                        if let Some(h) = spacer {
                            children.push(spacer_div(h));
                        }
                    }
                    Resolved::Overlay(overlay_props) => {
                        if first {
                            for p in overlay_props {
                                if !node_props.contains(&p) {
                                    node_props.push(p);
                                }
                            }
                        } else {
                            self.dropped.push(Dropped::new(
                                format!("<{} class={}> overlay", n.tag, n.classes.join(".")),
                                "inset:0 overlay only merges as first child",
                                format!("{}:{}", self.tsx_file, n.line),
                            ));
                        }
                    }
                    Resolved::Gone => {}
                },
                TsxChild::Text(t, _) => children.push(PrsNode {
                    tag: Tag::Text(TextValue::Literal(t.clone())),
                    props: vec![],
                    children: vec![],
                }),
                TsxChild::Expr(name, _) => {
                    if text_props.is_empty() {
                        self.params.push(Param::Slot(name.clone()));
                        children.push(PrsNode {
                            tag: Tag::Slot(name.clone()),
                            props: vec![],
                            children: vec![],
                        });
                    } else {
                        self.params.push(Param::Text(name.clone()));
                        children.push(PrsNode {
                            tag: Tag::Text(TextValue::Param(name.clone())),
                            props: vec![],
                            children: vec![],
                        });
                    }
                }
            }
            first = false;
        }

        // Inheritance push-down: container typography lands on its text runs.
        if !text_props.is_empty() {
            for child in &mut children {
                if let Tag::Text(_) = child.tag {
                    for p in &text_props {
                        if !child.props.iter().any(|q| q.name == p.name) {
                            child.props.push(p.clone());
                        }
                    }
                }
            }
        }
        // A styleless div around a single text run IS that text run.
        if node_props.is_empty()
            && children.len() == 1
            && let Tag::Text(_) = children[0].tag
        {
            return Resolved::Node {
                node: children.remove(0),
                spacer,
            };
        }
        Resolved::Node {
            node: PrsNode {
                tag: Tag::Div,
                props: node_props,
                children,
            },
            spacer,
        }
    }

    /// Map decls onto `props`; returns the spacer height when a
    /// `margin-bottom` was rewritten. `merge` marks overlay/pseudo context
    /// where `position: absolute` is consumed by the merge itself.
    fn apply_decls(
        &mut self,
        props: &mut Vec<Prop>,
        sel: &str,
        decls: &[Decl],
        merge: bool,
    ) -> Option<f32> {
        let font_size = decls
            .iter()
            .find(|d| d.prop == "font-size")
            .and_then(|d| px(&d.value));
        let mut spacer = None;
        for d in decls {
            if merge && d.prop == "position" && d.value == "absolute" {
                self.mapped += 1; // consumed by the overlay-merge rewrite
                continue;
            }
            match map_decl(&d.prop, &d.value, font_size) {
                MapOut::Props(new) => {
                    let mut agreed = true;
                    for p in new {
                        match props.iter().find(|q| q.name == p.name) {
                            None => props.push(p),
                            Some(q) if *q == p => {} // agreement, e.g. repeated radius
                            Some(_) => agreed = false,
                        }
                    }
                    if agreed {
                        self.mapped += 1;
                    } else {
                        self.drop_decl(sel, d, "conflicts with an earlier declaration; first wins");
                    }
                }
                MapOut::Noop(_) => self.mapped += 1,
                MapOut::SpacerAfter(h) => {
                    spacer = Some(h);
                    self.mapped += 1;
                }
                MapOut::Drop(why) => self.drop_decl(sel, d, why),
            }
        }
        spacer
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
                self.dropped.push(Dropped::new(
                    sel.clone(),
                    "pseudo-element outside the inset:0 overlay pattern",
                    format!("{}:{}", self.sass_file, rule.line),
                ));
            }
        }
    }

    /// Every rule never applied nor consumed becomes a droplist entry, so
    /// nothing in the stylesheet disappears silently.
    fn sweep_unapplied(&mut self) {
        let mut keyframes_seen: HashSet<String> = HashSet::new();
        for rule in self.rules {
            let mut sel = rule.selector.as_str();
            if self.applied.contains(sel) {
                continue;
            }
            if self
                .consumed_fragments
                .iter()
                .any(|f| sel.contains(f.as_str()))
            {
                continue; // accounted by a subtree/conditional drop entry
            }
            // One entry per animation, not per keyframe block.
            let keyframes_name;
            if sel.starts_with("@keyframes") {
                keyframes_name = sel.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
                if !keyframes_seen.insert(keyframes_name.clone()) {
                    continue;
                }
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
            self.dropped.push(Dropped::new(
                sel,
                why,
                format!("{}:{}", self.sass_file, rule.line),
            ));
        }
    }

    fn drop_decl(&mut self, sel: &str, d: &Decl, why: &str) {
        self.dropped.push(Dropped::new(
            format!("{sel} {{ {}: {} }}", d.prop, d.value),
            why,
            format!("{}:{}", self.sass_file, d.line),
        ));
    }
}

fn spacer_div(h: f32) -> PrsNode {
    PrsNode {
        tag: Tag::Div,
        props: vec![Prop::f32("h", h)],
        children: vec![],
    }
}
