//! Stage 1 (html, task-71 phase 2): an HTML document -> its element tree
//! (tree-sitter-html) and the CSS rules of its `<style>` elements and of
//! each `style=""` (tree-sitter-css), every construct with its line. It
//! reads structure only: which elements, attributes, texts, selectors and
//! declarations the source holds. What they mean is `resolve_html`'s.

use crate::ParserError;
use tree_sitter::Node;

/// An element (or the document's doctype, as `!doctype`).
#[derive(Debug, Clone, Default)]
pub struct HtmlNode {
    /// Lowercase tag name.
    pub tag: String,
    pub attrs: Vec<HtmlAttr>,
    pub children: Vec<HtmlChild>,
    pub line: usize,
    /// The body of a `<script>` or `<style>`, with the line it starts on.
    pub raw: Option<(String, usize)>,
}

impl HtmlNode {
    pub fn attr(&self, name: &str) -> Option<&HtmlAttr> {
        self.attrs.iter().find(|a| a.name == name)
    }

    /// The classes of `class=""`, in order.
    pub fn classes(&self) -> Vec<String> {
        self.attr("class")
            .and_then(|a| a.value.as_deref())
            .map(|v| v.split_whitespace().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// The element children, in order.
    pub fn elements(&self) -> impl Iterator<Item = &HtmlNode> {
        self.children.iter().filter_map(|c| match c {
            HtmlChild::Element(n) => Some(n),
            HtmlChild::Text(..) => None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HtmlAttr {
    /// Lowercase attribute name.
    pub name: String,
    /// The value, entities decoded; `None` for a bare attribute.
    pub value: Option<String>,
    pub line: usize,
    /// For `style=""`: the declarations it holds.
    pub decls: Vec<CssDecl>,
}

#[derive(Debug, Clone)]
pub enum HtmlChild {
    Element(HtmlNode),
    /// A run of text, whitespace collapsed, entities decoded, never empty.
    Text(String, usize),
}

/// One declaration: `prop: value`, `!important` stripped into its flag.
#[derive(Debug, Clone, PartialEq)]
pub struct CssDecl {
    /// Lowercase property name.
    pub prop: String,
    pub value: String,
    pub important: bool,
    pub line: usize,
}

/// A rule of a stylesheet: a selector list and its declarations, or an
/// at-rule (`@media`, `@keyframes`) with every line it holds.
#[derive(Debug, Clone)]
pub struct CssRule {
    /// The selector list as written, or the at-rule's prelude
    /// (`@media (max-width: 767px)`).
    pub selector: String,
    pub decls: Vec<CssDecl>,
    pub line: usize,
    /// An at-rule: the lines of everything inside it.
    pub at_rule: Option<Vec<usize>>,
}

/// A parsed document.
#[derive(Debug, Clone)]
pub struct HtmlDoc {
    /// The top-level nodes (doctype, `<html>`, or a fragment's elements).
    pub nodes: Vec<HtmlNode>,
    /// The rules of every `<style>`, in source order.
    pub rules: Vec<CssRule>,
}

pub fn parse_html(src: &str) -> Result<HtmlDoc, ParserError> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_html::LANGUAGE.into())
        .map_err(|e| ParserError::Parse(format!("html grammar: {e}")))?;
    let tree = parser
        .parse(src, None)
        .ok_or_else(|| ParserError::Parse("tree-sitter returned no html tree".into()))?;
    if tree.root_node().has_error() {
        return Err(ParserError::Parse("html source has syntax errors".into()));
    }
    let mut css = Css::new()?;
    let mut doc = HtmlDoc {
        nodes: Vec::new(),
        rules: Vec::new(),
    };
    let root = tree.root_node();
    let mut cursor = root.walk();
    let mut texts = Vec::new();
    for child in root.named_children(&mut cursor) {
        if let Some(HtmlChild::Element(n)) = convert(child, src, &mut css, &mut doc, &mut texts)? {
            doc.nodes.push(n);
        }
    }
    Ok(doc)
}

fn text<'a>(node: Node, src: &'a str) -> &'a str {
    node.utf8_text(src.as_bytes()).unwrap_or("")
}

fn line(node: Node) -> usize {
    node.start_position().row + 1
}

/// `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&#39;`, `&nbsp;` and numeric
/// references; any other `&` stays as written.
pub fn decode_entities(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let Some(end) = tail.find(';').filter(|e| *e <= 10) else {
            out.push('&');
            rest = &tail[1..];
            continue;
        };
        let name = &tail[1..end];
        let ch = match name {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some('\u{a0}'),
            _ => name
                .strip_prefix("#x")
                .or_else(|| name.strip_prefix("#X"))
                .and_then(|h| u32::from_str_radix(h, 16).ok())
                .or_else(|| name.strip_prefix('#').and_then(|d| d.parse().ok()))
                .and_then(char::from_u32),
        };
        match ch {
            Some(c) => {
                out.push(c);
                rest = &tail[end + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Collapses whitespace runs to one space and trims.
fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Pending text pieces (texts and entities): byte range and line.
type Texts = Vec<(usize, usize, usize)>;

/// The pending pieces as one text run: the source from the first piece to
/// the last, so the whitespace between them stays.
fn flush(texts: &mut Texts, src: &str, out: &mut Vec<HtmlChild>) {
    let (Some(&(start, _, line)), Some(&(_, end, _))) = (texts.first(), texts.last()) else {
        return;
    };
    texts.clear();
    let t = collapse(&decode_entities(src.get(start..end).unwrap_or("")));
    if !t.is_empty() {
        out.push(HtmlChild::Text(t, line));
    }
}

fn convert(
    node: Node,
    src: &str,
    css: &mut Css,
    doc: &mut HtmlDoc,
    texts: &mut Texts,
) -> Result<Option<HtmlChild>, ParserError> {
    match node.kind() {
        "doctype" => Ok(Some(HtmlChild::Element(HtmlNode {
            tag: "!doctype".into(),
            line: line(node),
            ..HtmlNode::default()
        }))),
        "element" | "script_element" | "style_element" => element(node, src, css, doc).map(Some),
        "text" | "entity" => {
            texts.push((node.start_byte(), node.end_byte(), line(node)));
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn element(
    node: Node,
    src: &str,
    css: &mut Css,
    doc: &mut HtmlDoc,
) -> Result<HtmlChild, ParserError> {
    let mut out = HtmlNode {
        line: line(node),
        ..HtmlNode::default()
    };
    let mut texts: Texts = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "start_tag" | "self_closing_tag" => tag(child, src, css, &mut out)?,
            "end_tag" | "comment" => {}
            "raw_text" => out.raw = Some((text(child, src).to_string(), line(child))),
            _ => {
                if let Some(c) = convert(child, src, css, doc, &mut texts)? {
                    flush(&mut texts, src, &mut out.children);
                    out.children.push(c);
                }
            }
        }
    }
    flush(&mut texts, src, &mut out.children);
    if out.tag == "style"
        && let Some((body, start)) = &out.raw
    {
        let rules = css.rules(body, start - 1)?;
        doc.rules.extend(rules);
    }
    Ok(HtmlChild::Element(out))
}

fn tag(node: Node, src: &str, css: &mut Css, out: &mut HtmlNode) -> Result<(), ParserError> {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "tag_name" => out.tag = text(child, src).to_ascii_lowercase(),
            "attribute" => out.attrs.push(attribute(child, src, css)?),
            _ => {}
        }
    }
    Ok(())
}

fn attribute(node: Node, src: &str, css: &mut Css) -> Result<HtmlAttr, ParserError> {
    let mut name = String::new();
    let mut value = None;
    let mut value_row = line(node) - 1;
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "attribute_name" => name = text(child, src).to_ascii_lowercase(),
            "attribute_value" => {
                value_row = child.start_position().row;
                value = Some(text(child, src).to_string());
            }
            "quoted_attribute_value" => {
                let mut inner = child.walk();
                let v = child
                    .named_children(&mut inner)
                    .find(|c| c.kind() == "attribute_value");
                value_row = v.map_or(child.start_position().row, |v| v.start_position().row);
                value = Some(v.map(|v| text(v, src)).unwrap_or("").to_string());
            }
            _ => {}
        }
    }
    let decls = match (&name[..], &value) {
        ("style", Some(v)) => css.inline(v, value_row)?,
        _ => Vec::new(),
    };
    Ok(HtmlAttr {
        name,
        value: value.map(|v| decode_entities(&v)),
        line: line(node),
        decls,
    })
}

/// The CSS reader: tree-sitter-css over a `<style>` body or a `style=""`.
struct Css {
    parser: tree_sitter::Parser,
}

impl Css {
    fn new() -> Result<Self, ParserError> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_css::LANGUAGE.into())
            .map_err(|e| ParserError::Parse(format!("css grammar: {e}")))?;
        Ok(Self { parser })
    }

    fn tree(&mut self, src: &str) -> Result<tree_sitter::Tree, ParserError> {
        let tree = self
            .parser
            .parse(src, None)
            .ok_or_else(|| ParserError::Parse("tree-sitter returned no css tree".into()))?;
        if tree.root_node().has_error() {
            return Err(ParserError::Parse("css source has syntax errors".into()));
        }
        Ok(tree)
    }

    /// The rules of a stylesheet whose first line is line `offset + 1` of
    /// the document.
    fn rules(&mut self, body: &str, offset: usize) -> Result<Vec<CssRule>, ParserError> {
        let tree = self.tree(body)?;
        let root = tree.root_node();
        let mut out = Vec::new();
        let mut cursor = root.walk();
        for node in root.named_children(&mut cursor) {
            let at = line(node) + offset;
            match node.kind() {
                "rule_set" => {
                    let selector = node
                        .child_by_field_name("selectors")
                        .or_else(|| node.named_child(0))
                        .map(|s| collapse(text(s, body)))
                        .unwrap_or_default();
                    out.push(CssRule {
                        selector,
                        decls: decls(node, body, offset),
                        line: at,
                        at_rule: None,
                    });
                }
                "comment" | "js_comment" => {}
                _ => {
                    let whole = text(node, body);
                    let prelude = whole.split('{').next().unwrap_or(whole);
                    let mut lines = Vec::new();
                    inner_lines(node, offset, &mut lines);
                    lines.retain(|l| *l != at);
                    lines.dedup();
                    out.push(CssRule {
                        selector: collapse(prelude.trim_end_matches(';')),
                        decls: decls(node, body, offset),
                        line: at,
                        at_rule: Some(lines),
                    });
                }
            }
        }
        Ok(out)
    }

    /// The declarations of `style=""` whose value starts on row `row`.
    fn inline(&mut self, value: &str, row: usize) -> Result<Vec<CssDecl>, ParserError> {
        let wrapped = format!("x{{{value}}}");
        let tree = self.tree(&wrapped)?;
        Ok(decls(tree.root_node(), &wrapped, row))
    }
}

/// Every declaration under `node`, in order.
fn decls(node: Node, src: &str, offset: usize) -> Vec<CssDecl> {
    let mut out = Vec::new();
    let mut stack = vec![node];
    let mut found = Vec::new();
    while let Some(n) = stack.pop() {
        if n.kind() == "declaration" {
            found.push(n);
            continue;
        }
        let mut cursor = n.walk();
        let kids: Vec<Node> = n.named_children(&mut cursor).collect();
        stack.extend(kids.into_iter().rev());
    }
    for d in found {
        let whole = text(d, src).trim().trim_end_matches(';');
        let Some((prop, value)) = whole.split_once(':') else {
            continue;
        };
        let mut value = collapse(value);
        let important = value.ends_with("!important");
        if important {
            value = value.trim_end_matches("!important").trim().to_string();
        }
        out.push(CssDecl {
            prop: prop.trim().to_ascii_lowercase(),
            value,
            important,
            line: line(d) + offset,
        });
    }
    out
}

/// The line of every named node under `node`.
fn inner_lines(node: Node, offset: usize, out: &mut Vec<usize>) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        let l = line(child) + offset;
        if !out.contains(&l) {
            out.push(l);
        }
        inner_lines(child, offset, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "<!doctype html>\n<html>\n<head>\n<style>\n.a { color: #fff; }\n@media (max-width: 700px) {\n  .a { width: 100%; }\n}\n</style>\n</head>\n<body>\n<div class=\"a b\" style=\"padding: 8px; gap: 4px !important\">Hi &amp;\n  there</div>\n<input placeholder=Reply disabled>\n</body>\n</html>\n";

    #[test]
    fn elements_attributes_texts_and_rules_with_their_lines() {
        let doc = parse_html(DOC).expect("parses");
        assert_eq!(doc.nodes[0].tag, "!doctype");
        let html = &doc.nodes[1];
        let body = html.elements().find(|e| e.tag == "body").expect("body");
        let div = body.elements().next().expect("div");
        assert_eq!(
            (div.line, div.classes()),
            (12, vec!["a".into(), "b".into()])
        );
        let style = div.attr("style").expect("style");
        assert_eq!(style.decls.len(), 2);
        assert_eq!(style.decls[1].prop, "gap");
        assert!(style.decls[1].important);
        assert!(matches!(&div.children[0], HtmlChild::Text(t, 12) if t == "Hi & there"));
        let input = body.elements().nth(1).expect("input");
        assert_eq!(
            input.attr("placeholder").and_then(|a| a.value.clone()),
            Some("Reply".into())
        );
        assert_eq!(
            input.attr("disabled").map(|a| a.value.is_none()),
            Some(true)
        );
        assert_eq!(doc.rules.len(), 2);
        assert_eq!(
            (doc.rules[0].selector.as_str(), doc.rules[0].line),
            (".a", 5)
        );
        assert_eq!(doc.rules[0].decls[0].line, 5);
        let media = &doc.rules[1];
        assert!(media.selector.starts_with("@media"));
        assert_eq!(media.decls[0].line, 7);
        assert!(media.at_rule.as_ref().is_some_and(|l| l.contains(&7)));
    }

    #[test]
    fn entities_decode_and_unknown_ones_stay() {
        assert_eq!(
            decode_entities("a &lt;b&gt; &#65;&#x42; &bogus; &"),
            "a <b> AB &bogus; &"
        );
    }

    #[test]
    fn a_broken_stylesheet_is_an_error() {
        assert!(parse_html("<style>.a { color: }}}</style>").is_err());
    }
}
