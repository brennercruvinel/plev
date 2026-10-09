//! The document's frame and its stylesheet, for `resolve_html`: the rules
//! read and swept, the cascade an element receives, the frame around the
//! component (doctype, `<html>`, `<head>`, `<body>`, `<title>`, `<style>`,
//! `<script>`) and the attributes a node carries.

use super::resolve::{Ctx, FRAME, SCRIPT, Styled, label};
use crate::formats::html::parse::{CssDecl, HtmlChild, HtmlNode};
use crate::formats::html::css::{self, Selector};
use crate::tree::{Attr, Dropped, Src};

/// The selectors of a list, split on the commas outside parentheses.
fn split_list(list: &str) -> Vec<&str> {
    let (mut out, mut depth, mut start) = (Vec::new(), 0i32, 0);
    for (i, c) in list.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(list[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(list[start..].trim());
    out
}

impl Ctx<'_> {
    /// The simple selectors of every rule; an at-rule, and every selector
    /// outside the subset, is an entry now (covering the declarations when
    /// no selector of the rule applies them).
    pub(super) fn read_rules(&mut self) {
        for (i, rule) in self.doc.rules.iter().enumerate() {
            let at = self.at(rule.line);
            let decl_sites: Vec<String> = rule.decls.iter().map(|d| self.at(d.line)).collect();
            if let Some(lines) = &rule.at_rule {
                let covers = lines.iter().map(|l| self.at(*l)).collect();
                let why = css::at_rule(&rule.selector);
                self.dropped
                    .push(Dropped::new(rule.selector.clone(), why, at).covering(covers));
                self.applied.insert(i);
                continue;
            }
            let parts = split_list(&rule.selector);
            let simple: Vec<Selector> = parts
                .iter()
                .filter_map(|p| css::selector(p).ok())
                .collect();
            for part in &parts {
                if let Err(why) = css::selector(part) {
                    let covers = if simple.is_empty() {
                        decl_sites.clone()
                    } else {
                        Vec::new()
                    };
                    self.dropped
                        .push(Dropped::new(*part, why, at.clone()).covering(covers));
                }
            }
            if simple.is_empty() {
                self.applied.insert(i);
            }
            self.rules.extend(simple.into_iter().map(|s| (i, s)));
        }
    }

    /// Every rule no element matched: an entry covering its declarations.
    pub(super) fn sweep(&mut self) {
        for (i, rule) in self.doc.rules.iter().enumerate() {
            if self.applied.contains(&i) {
                continue;
            }
            let covers = rule.decls.iter().map(|d| self.at(d.line)).collect();
            self.dropped.push(
                Dropped::new(
                    rule.selector.clone(),
                    "a rule no element of the component matches",
                    self.at(rule.line),
                )
                .covering(covers),
            );
        }
    }

    /// The declarations `el` receives, lowest priority first: `<img>`'s
    /// size attributes, then the rules by specificity (tag, class, id) and
    /// order, then `style=""`, then `!important`.
    pub(super) fn cascade(&mut self, el: &HtmlNode) -> Vec<Styled> {
        let mut out = Vec::new();
        if el.tag == "img" {
            for a in el
                .attrs
                .iter()
                .filter(|a| a.name == "width" || a.name == "height")
            {
                let v = a.value.clone().unwrap_or_default();
                let value = if v.parse::<f32>().is_ok() {
                    format!("{v}px")
                } else {
                    v.clone()
                };
                out.push(Styled {
                    decl: CssDecl {
                        prop: a.name.clone(),
                        value,
                        important: false,
                        line: a.line,
                    },
                    src: Src::new(self.at(a.line), format!("{}=\"{v}\"", a.name)),
                    rank: 0,
                });
            }
        }
        let classes = el.classes();
        let id = el.attr("id").and_then(|a| a.value.clone());
        let mut matched: Vec<(usize, u8)> = Vec::new();
        for (i, sel) in &self.rules {
            let hit = match sel {
                Selector::Tag(t) => *t == el.tag,
                Selector::Class(c) => classes.contains(c),
                Selector::Id(x) => id.as_ref() == Some(x),
            };
            if !hit {
                continue;
            }
            match matched.iter_mut().find(|(j, _)| j == i) {
                Some(m) => m.1 = m.1.max(sel.rank()),
                None => matched.push((*i, sel.rank())),
            }
        }
        for (i, rank) in matched {
            self.applied.insert(i);
            let rule = &self.doc.rules[i];
            for d in &rule.decls {
                let what = format!("{} {{ {}: {} }}", rule.selector, d.prop, d.value);
                out.push(Styled {
                    decl: d.clone(),
                    src: Src::new(self.at(d.line), what),
                    rank,
                });
            }
        }
        if let Some(style) = el.attr("style") {
            for d in &style.decls {
                let what = format!("style=\"{}: {}\"", d.prop, d.value);
                out.push(Styled {
                    decl: d.clone(),
                    src: Src::new(self.at(d.line), what),
                    rank: 4,
                });
            }
        }
        for s in &mut out {
            if s.decl.important {
                s.rank += 10;
            }
        }
        out.sort_by_key(|s| s.rank);
        // the last declaration of a property wins; the others hold as the
        // cascade says, overridden
        let mut winners: Vec<Styled> = Vec::new();
        for s in out {
            if let Some(i) = winners.iter().position(|w| w.decl.prop == s.decl.prop) {
                let lost = winners.remove(i);
                self.note(
                    lost.src,
                    "overridden in the cascade by a later or more specific declaration",
                );
            }
            winners.push(s);
        }
        winners
    }

    /// The `file:line` of every construct under `el`, marking the rules it
    /// matches as answered for.
    pub(super) fn sites(&mut self, el: &HtmlNode, out: &mut Vec<String>) {
        for a in &el.attrs {
            out.push(self.at(a.line));
            out.extend(a.decls.iter().map(|d| self.at(d.line)));
        }
        let styled = self.cascade(el);
        out.extend(styled.into_iter().map(|s| s.src.at));
        if let Some((body, start)) = &el.raw {
            for (i, l) in body.lines().enumerate() {
                if !l.trim().is_empty() {
                    out.push(self.at(start + i));
                }
            }
        }
        for c in &el.children {
            match c {
                HtmlChild::Element(n) => {
                    out.push(self.at(n.line));
                    self.sites(n, out);
                }
                HtmlChild::Text(_, l) => out.push(self.at(*l)),
            }
        }
    }

    /// The document's frame (doctype, `<html>`, `<head>`, `<body>` around
    /// one element) and what it holds; `root` becomes the component.
    pub(super) fn scaffold<'d>(&mut self, n: &'d HtmlNode, root: &mut Option<&'d HtmlNode>) {
        let src = Src::new(self.at(n.line), label(n));
        match n.tag.as_str() {
            "!doctype" | "html" | "head" => {
                self.note(src, FRAME);
                self.frame_parts(n);
                for c in n.elements() {
                    self.scaffold(c, root);
                }
            }
            "body" => {
                let content: Vec<&HtmlNode> = n
                    .elements()
                    .filter(|e| !matches!(e.tag.as_str(), "script" | "style" | "link" | "meta"))
                    .collect();
                let text = n.children.iter().any(|c| matches!(c, HtmlChild::Text(..)));
                if content.len() != 1 || text {
                    *root = Some(n);
                    return;
                }
                self.note(src, FRAME);
                self.frame_parts(n);
                for c in n.elements() {
                    if std::ptr::eq(c, content[0]) {
                        *root = Some(c);
                    } else {
                        self.scaffold(c, root);
                    }
                }
            }
            "title" => {
                let text: Vec<&str> = n
                    .children
                    .iter()
                    .filter_map(|c| match c {
                        HtmlChild::Text(t, _) => Some(t.as_str()),
                        HtmlChild::Element(_) => None,
                    })
                    .collect();
                let text = text.join(" ");
                let decl = Src::new(src.at, format!("<title> \"{text}\""));
                self.title = Some((text, decl));
            }
            "meta" => {
                self.note(src, "document metadata draws nothing");
                self.frame_parts(n);
            }
            "style" => self.note(
                src,
                "the stylesheet: each of its rules is reported on its own",
            ),
            "script" => {
                let mut covers = Vec::new();
                self.sites(n, &mut covers);
                let entry = Dropped::new(src.what, SCRIPT, src.at);
                self.dropped.push(entry.covering(covers));
            }
            "link" => {
                let mut covers = Vec::new();
                self.sites(n, &mut covers);
                let why = "an external stylesheet or resource is not read: only <style> and style=\"\" are";
                self.dropped
                    .push(Dropped::new(src.what, why, src.at).covering(covers));
            }
            _ if root.is_none() => *root = Some(n),
            _ => {
                let mut covers = Vec::new();
                self.sites(n, &mut covers);
                let why = "a second top-level element: the component is the first";
                self.dropped
                    .push(Dropped::new(src.what, why, src.at).covering(covers));
            }
        }
    }

    /// The attributes of a frame element, and the styles its rules give
    /// it: the frame is not the component.
    pub(super) fn frame_parts(&mut self, n: &HtmlNode) {
        for a in &n.attrs {
            let what = match &a.value {
                Some(v) => format!("{}=\"{v}\"", a.name),
                None => a.name.clone(),
            };
            self.note(
                Src::new(self.at(a.line), what),
                "an attribute of the document's frame",
            );
        }
        if matches!(n.tag.as_str(), "html" | "body") {
            for s in self.cascade(n) {
                self.lose(
                    &s.src,
                    "a style of the document's frame: the component is drawn on the theme's background",
                );
            }
        }
    }

    /// The attributes a node carries for the IR emitter (the control
    /// inference reads them); `class`, `id` and `style` are reported here.
    pub(super) fn attrs(&mut self, el: &HtmlNode) -> Vec<Attr> {
        let mut out = Vec::new();
        for a in &el.attrs {
            let what = match &a.value {
                Some(v) => format!("{}=\"{v}\"", a.name),
                None => a.name.clone(),
            };
            let src = Src::new(self.at(a.line), what);
            match a.name.as_str() {
                "class" => self.note(
                    src,
                    "the class selects the element's rules and names its node",
                ),
                "id" => self.note(
                    src,
                    "the id selects the element's rules and names a node with no class",
                ),
                "style" => self.note(src, "inline declarations, each reported on its own"),
                "width" | "height" if el.tag == "img" => {}
                _ => {
                    let (name, value) = css::attr(&a.name, a.value.as_deref());
                    out.push(Attr { name, value, src });
                }
            }
        }
        out
    }
}
