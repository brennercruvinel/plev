//! Stage 2 (html, task-71 phase 2): an HTML document -> ParserNode tree,
//! droplist and notes, the same `Resolution` the react and gpui resolvers
//! give, so the code emitter and the plev IR emitter read it unchanged.
//!
//! The component is the body's one element (or the body, when it holds
//! more). The subset: block and text elements (`div`, `section`,
//! `header`, `p`, `span`, `h1` to `h3`, `label`, `ul`, `li`), `img`, the
//! controls (`button`, `input`, `textarea`), and the CSS of `<style>` and
//! `style=""` through `.class`, `#id` and tag selectors, cascaded by
//! specificity then order, as a browser does. Text styles inherit down to
//! every text run below, as CSS inherits them. Two phase 1 rewrites hold
//! here too: `margin-bottom` is a spacer after the element, and an element
//! holding only one text run is that run.
//!
//! Everything else lands on the droplist with its line, never silently:
//! elements outside the subset, `<script>`, at-rules, selectors that are
//! not simple, declarations outside the subset, the browser's default
//! styles the subset does not apply, and text styles no run takes.


use std::collections::HashSet;

use crate::ParserError;
use crate::formats::react::css::{is_text_prop, px};
use crate::emit::plevir::sanitize;
use crate::formats::html::parse::{CssDecl, HtmlChild, HtmlDoc, HtmlNode};
use crate::formats::html::css::{self, Mapped, Selector};
use crate::tree::{
    Dropped, Note, NoteKind, Origin, ParserNode, Prop, Resolution, Src, Tag, TextValue,
};

const COLLAPSE_RULE: &str =
    "an element holding only one text run and its text styles is that text run";
pub(super) const FRAME: &str = "the document's frame: the component is the body's element";
pub(super) const SCRIPT: &str =
    "JavaScript is not read: what a control does is the app's (binding required on the control)";

/// The elements of the subset.
fn in_subset(tag: &str) -> bool {
    matches!(
        tag,
        "div"
            | "section"
            | "header"
            | "p"
            | "span"
            | "h1"
            | "h2"
            | "h3"
            | "label"
            | "ul"
            | "li"
            | "img"
            | "button"
            | "input"
            | "textarea"
    )
}

/// `<section class=note-card>`, `<div id=x>`, `<p>`.
pub(super) fn label(el: &HtmlNode) -> String {
    let classes = el.classes();
    match (
        classes.is_empty(),
        el.attr("id").and_then(|a| a.value.as_deref()),
    ) {
        (false, _) => format!("<{} class={}>", el.tag, classes.join(".")),
        (true, Some(id)) => format!("<{} id={id}>", el.tag),
        (true, None) => format!("<{}>", el.tag),
    }
}

/// A declaration an element receives, with the construct it is.
#[derive(Clone)]
pub(super) struct Styled {
    pub(super) decl: CssDecl,
    pub(super) src: Src,
    pub(super) rank: u8,
}

enum Out {
    Node(Box<ParserNode>, Option<(f32, Src)>),
    Gone,
}

pub(super) struct Ctx<'a> {
    pub(super) doc: &'a HtmlDoc,
    file: &'a str,
    /// Each simple selector with the index of its rule.
    pub(super) rules: Vec<(usize, Selector)>,
    /// The rules some element matched, or that an entry already answers.
    pub(super) applied: HashSet<usize>,
    pub(super) mapped: usize,
    pub(super) dropped: Vec<Dropped>,
    notes: Vec<Note>,
    pub(super) title: Option<(String, Src)>,
    /// The constructs behind every text style, to find the ones no run takes.
    text_srcs: Vec<Src>,
}

pub fn resolve_html(doc: &HtmlDoc, file: &str) -> Result<Resolution, ParserError> {
    let mut cx = Ctx {
        doc,
        file,
        rules: Vec::new(),
        applied: HashSet::new(),
        mapped: 0,
        dropped: Vec::new(),
        notes: Vec::new(),
        title: None,
        text_srcs: Vec::new(),
    };
    cx.read_rules();
    let mut root = None;
    for n in &doc.nodes {
        cx.scaffold(n, &mut root);
    }
    let root = root.ok_or_else(|| {
        ParserError::Parse("no element to import: the body (or the fragment) holds none".into())
    })?;
    let root_at = cx.at(root.line);
    let name = root
        .classes()
        .first()
        .cloned()
        .or_else(|| root.attr("id").and_then(|a| a.value.clone()))
        .unwrap_or_else(|| root.tag.clone());
    let node = match cx.element(root, &[]) {
        Out::Node(node, spacer) => {
            if let Some((_, src)) = spacer {
                cx.dropped.push(Dropped::new(
                    src.what,
                    "a margin below the component has nothing after it to space",
                    src.at,
                ));
            }
            *node
        }
        Out::Gone => ParserNode::new(Tag::Div, vec![], vec![]),
    };
    cx.unlanded(&node);
    cx.sweep();
    let (title, decl) = match cx.title.take() {
        Some((t, src)) => (t, src),
        None => (name.clone(), Src::new(root_at, format!("component {name}"))),
    };
    Ok(Resolution {
        fn_name: sanitize(&name, "component"),
        name: title,
        decl,
        source_label: file.to_string(),
        root: node,
        params: Vec::new(),
        mapped: cx.mapped,
        dropped: cx.dropped,
        notes: cx.notes,
    })
}

impl<'a> Ctx<'a> {
    pub(super) fn at(&self, line: usize) -> String {
        format!("{}:{line}", self.file)
    }

    pub(super) fn note(&mut self, src: Src, why: &'static str) {
        self.notes.push(Note {
            src,
            kind: NoteKind::Holds(why),
        });
    }

    pub(super) fn lose(&mut self, src: &Src, why: impl Into<String>) {
        self.dropped
            .push(Dropped::new(src.what.clone(), why, src.at.clone()));
    }

    fn element(&mut self, el: &HtmlNode, inherited: &[Prop]) -> Out {
        let src = Src::new(self.at(el.line), label(el));
        if matches!(
            el.tag.as_str(),
            "script" | "style" | "link" | "meta" | "title"
        ) {
            let mut none = None;
            self.scaffold(el, &mut none);
            return Out::Gone;
        }
        if !in_subset(&el.tag) {
            let mut covers = Vec::new();
            self.sites(el, &mut covers);
            let why = format!("the element <{}> is outside the subset", el.tag);
            self.dropped
                .push(Dropped::new(src.what, why, src.at).covering(covers));
            return Out::Gone;
        }
        let styled = self.cascade(el);
        let display = styled
            .iter()
            .find(|s| s.decl.prop == "display")
            .map(|s| s.decl.value.clone());
        if display.as_deref() == Some("none") {
            let mut covers = Vec::new();
            self.sites(el, &mut covers);
            let why =
                "display: none draws nothing: the element is left out, as the source shows it";
            self.dropped.push(
                Dropped::new(src.what, why, src.at)
                    .rewritten()
                    .covering(covers),
            );
            return Out::Gone;
        }
        let flex = matches!(display.as_deref(), Some("flex" | "inline-flex"));
        let font_size = styled
            .iter()
            .find(|s| s.decl.prop == "font-size")
            .and_then(|s| px(&s.decl.value))
            .or_else(|| font_size_of(inherited));
        let mut props: Vec<Prop> = Vec::new();
        let mut spacer = None;
        for s in &styled {
            match s.decl.prop.as_str() {
                "display" => self.display(s, &styled, &mut props),
                "flex-direction" => {}
                prop => match css::map(prop, &s.decl.value, font_size, flex) {
                    Mapped::Props(new) => {
                        self.mapped += 1;
                        for p in new {
                            if let Some(i) = props.iter().position(|q| q.name == p.name) {
                                let old = props.remove(i);
                                for o in old.src {
                                    self.note(o, "overridden in the cascade by a later declaration of the same effect");
                                }
                            }
                            props.push(p.from(s.src.clone()));
                        }
                    }
                    Mapped::Holds(why) => {
                        self.mapped += 1;
                        self.note(s.src.clone(), why);
                    }
                    Mapped::Spacer(h) => {
                        self.mapped += 1;
                        spacer = Some((h, s.src.clone()));
                    }
                    Mapped::Drop(why) => self.lose(&s.src, why),
                },
            }
        }
        self.defaults(el, &styled);
        let (mut node_props, own_text): (Vec<Prop>, Vec<Prop>) =
            props.into_iter().partition(|p| !is_text_prop(p.name));
        self.text_srcs
            .extend(own_text.iter().flat_map(|p| p.src.iter().cloned()));
        let mut merged: Vec<Prop> = inherited
            .iter()
            .filter(|p| !own_text.iter().any(|q| q.name == p.name))
            .cloned()
            .collect();
        merged.extend(own_text.iter().cloned());
        // a field or an image has no text run: its own text style stays on
        // it, for the IR emitter to report against the control
        let leaf = matches!(el.tag.as_str(), "input" | "textarea" | "img");
        if leaf {
            node_props.extend(own_text);
        }
        let mut children = self.children(el, &merged, &src);
        let attrs = self.attrs(el);
        let name = el
            .classes()
            .first()
            .cloned()
            .or_else(|| el.attr("id").and_then(|a| a.value.clone()))
            .unwrap_or_default();
        let mut origin = Origin {
            src: Some(src),
            tag: el.tag.clone(),
            name,
            attrs,
            folded: Vec::new(),
            spacer: false,
        };
        if !leaf
            && node_props.is_empty()
            && children.len() == 1
            && matches!(children[0].tag, Tag::Text(_))
        {
            let mut run = children.remove(0);
            if let Some(div) = origin.src.take() {
                run.origin.folded.push((div, COLLAPSE_RULE));
            }
            run.origin.attrs = origin.attrs;
            run.origin.tag = origin.tag;
            run.origin.name = origin.name;
            return Out::Node(Box::new(run), spacer);
        }
        let node = ParserNode {
            tag: Tag::Div,
            props: node_props,
            children,
            origin,
        };
        Out::Node(Box::new(node), spacer)
    }

    fn children(&mut self, el: &HtmlNode, merged: &[Prop], src: &Src) -> Vec<ParserNode> {
        let mut out = Vec::new();
        for child in &el.children {
            match child {
                HtmlChild::Element(n) => match self.element(n, merged) {
                    Out::Node(node, spacer) => {
                        out.push(*node);
                        if let Some((h, s)) = spacer {
                            let mut div =
                                ParserNode::new(Tag::Div, vec![Prop::f32("h", h).from(s)], vec![]);
                            div.origin.spacer = true;
                            out.push(div);
                        }
                    }
                    Out::Gone => {}
                },
                HtmlChild::Text(t, line) => {
                    let tag = Tag::Text(TextValue::Literal(t.clone()));
                    let mut run = ParserNode::new(tag, merged.to_vec(), vec![]);
                    run.origin.src = Some(Src::new(self.at(*line), format!("\"{t}\"")));
                    run.origin.tag = "text".into();
                    out.push(run);
                }
            }
        }
        let texts = el
            .children
            .iter()
            .filter(|c| matches!(c, HtmlChild::Text(..)))
            .count();
        if texts > 0 && el.children.len() > 1 {
            let why = "inline content: its text runs and elements are stacked one below the other, not flowed on a line";
            let entry = Dropped::new(format!("{} inline content", src.what), why, src.at.clone());
            self.dropped.push(entry.approximated());
        }
        out
    }

    /// `display` and the `flex-direction` that goes with it.
    fn display(&mut self, s: &Styled, styled: &[Styled], props: &mut Vec<Prop>) {
        let dir = styled.iter().find(|d| d.decl.prop == "flex-direction");
        let value = s.decl.value.as_str();
        if matches!(value, "flex" | "inline-flex") {
            let (name, reversed) = match dir.map(|d| d.decl.value.as_str()) {
                Some("column") => ("col", false),
                Some("column-reverse") => ("col", true),
                Some("row-reverse") => ("row", true),
                _ => ("row", false),
            };
            let mut p = Prop::flag(name).from(s.src.clone());
            match dir {
                Some(d) if reversed => self.lose(
                    &d.src,
                    "a reversed direction has no IR form: the children stay in source order",
                ),
                Some(d) => p = p.from(d.src.clone()),
                None => {}
            }
            props.push(p);
            self.mapped += 1;
            return;
        }
        match value {
            "block" => {
                self.mapped += 1;
                self.note(
                    s.src.clone(),
                    "a block stacks its children in a column, the plev default",
                );
            }
            "grid" | "inline-grid" => self.lose(&s.src, "grid layout is outside the subset"),
            v => self.lose(
                &s.src,
                format!("display: {v} is outside the subset: the element is laid out as a block"),
            ),
        }
        if let Some(d) = dir {
            self.note(d.src.clone(), "flex-direction acts on flex containers only");
        }
    }

    /// The browser's default styles of `el` the stylesheet does not replace.
    fn defaults(&mut self, el: &HtmlNode, styled: &[Styled]) {
        for (group, props, what) in css::defaults(&el.tag) {
            if styled.iter().any(|s| props.contains(&s.decl.prop.as_str())) {
                continue;
            }
            let why = format!(
                "the browser's default {group} of <{}> ({what}) is not applied: the subset reads only the stylesheet",
                el.tag
            );
            let what = format!("<{}> default {group}", el.tag);
            self.dropped.push(Dropped::new(what, why, self.at(el.line)));
        }
    }

    /// Every text style no text run (and no field) took: carried by
    /// nothing, so dropped and no longer counted as mapped.
    fn unlanded(&mut self, root: &ParserNode) {
        fn walk(n: &ParserNode, out: &mut HashSet<Src>) {
            for p in n.props.iter().filter(|p| is_text_prop(p.name)) {
                out.extend(p.src.iter().cloned());
            }
            for c in &n.children {
                walk(c, out);
            }
        }
        let mut landed = HashSet::new();
        walk(root, &mut landed);
        let mut seen = HashSet::new();
        for src in std::mem::take(&mut self.text_srcs) {
            if landed.contains(&src) || !seen.insert(src.clone()) {
                continue;
            }
            self.mapped = self.mapped.saturating_sub(1);
            self.lose(
                &src,
                "a text style no text run below the element takes; nothing carries it",
            );
        }
    }
}

/// The font size a run below inherits.
fn font_size_of(props: &[Prop]) -> Option<f32> {
    props
        .iter()
        .find(|p| p.name == "font_size")
        .and_then(|p| match p.args.first() {
            Some(crate::tree::Arg::F32(v)) => Some(*v),
            _ => None,
        })
}
