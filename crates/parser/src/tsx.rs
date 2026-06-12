//! Stage 1 (structure): tsx source -> raw JSX tree with class references.
//! Uses tree-sitter-typescript (TSX grammar). Understands the corpus
//! patterns: `styles.x`, `styles["x"]`, `cn(styles.x, { [styles.y]: cond })`,
//! `{identifier}` children and literal text children.

use crate::ParserError;
use tree_sitter::Node;

#[derive(Debug, Clone)]
pub struct TsxNode {
    /// Source element tag (div, button, span) kept for droplist labels.
    pub tag: String,
    pub classes: Vec<String>,
    /// (class, condition expression, line) from `cn` object arguments.
    pub cond_classes: Vec<(String, String, usize)>,
    pub children: Vec<TsxChild>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum TsxChild {
    Node(TsxNode),
    /// Trimmed literal JSX text.
    Text(String, usize),
    /// `{identifier}` expression child.
    Expr(String, usize),
}

#[derive(Debug)]
pub struct TsxComponent {
    pub name: String,
    pub root: TsxNode,
}

pub fn parse_tsx(src: &str) -> Result<TsxComponent, ParserError> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())
        .map_err(|e| ParserError::Parse(format!("tsx grammar: {e}")))?;
    let tree = parser
        .parse(src, None)
        .ok_or_else(|| ParserError::Parse("tree-sitter returned no tsx tree".into()))?;
    if tree.root_node().has_error() {
        return Err(ParserError::Parse("tsx source has syntax errors".into()));
    }
    let name = component_name(tree.root_node(), src)
        .ok_or_else(|| ParserError::Parse("no arrow-function component found".into()))?;
    let jsx = find_kind(
        tree.root_node(),
        &["jsx_element", "jsx_self_closing_element"],
    )
    .ok_or_else(|| ParserError::Parse("no JSX root element found".into()))?;
    Ok(TsxComponent {
        name,
        root: convert(jsx, src),
    })
}

fn text<'a>(node: Node, src: &'a str) -> &'a str {
    node.utf8_text(src.as_bytes()).unwrap_or("")
}

fn component_name(root: Node, src: &str) -> Option<String> {
    let decl = find_pred(root, &|n| {
        n.kind() == "variable_declarator"
            && n.child_by_field_name("value")
                .is_some_and(|v| v.kind() == "arrow_function")
    })?;
    Some(text(decl.child_by_field_name("name")?, src).to_string())
}

fn find_kind<'t>(node: Node<'t>, kinds: &[&str]) -> Option<Node<'t>> {
    find_pred(node, &|n| kinds.contains(&n.kind()))
}

fn find_pred<'t>(node: Node<'t>, pred: &dyn Fn(Node) -> bool) -> Option<Node<'t>> {
    if pred(node) {
        return Some(node);
    }
    let mut cursor = node.walk();
    let children: Vec<Node> = node.named_children(&mut cursor).collect();
    children.into_iter().find_map(|c| find_pred(c, pred))
}

fn convert(node: Node, src: &str) -> TsxNode {
    let opening = node.child_by_field_name("open_tag").unwrap_or(node); // self-closing elements carry attributes directly
    let tag = opening
        .child_by_field_name("name")
        .map(|n| text(n, src).to_string())
        .unwrap_or_default();
    let mut out = TsxNode {
        tag,
        classes: Vec::new(),
        cond_classes: Vec::new(),
        children: Vec::new(),
        line: node.start_position().row + 1,
    };
    let mut cursor = opening.walk();
    for attr in opening.named_children(&mut cursor) {
        if attr.kind() != "jsx_attribute" {
            continue;
        }
        let is_class = attr
            .named_child(0)
            .is_some_and(|n| text(n, src) == "className");
        if !is_class {
            continue;
        }
        if let Some(value) = attr.named_child(1) {
            class_refs(value, src, &mut out);
        }
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        let line = child.start_position().row + 1;
        match child.kind() {
            "jsx_element" | "jsx_self_closing_element" => {
                out.children.push(TsxChild::Node(convert(child, src)));
            }
            "jsx_text" => {
                let t = text(child, src).trim().to_string();
                if !t.is_empty() {
                    out.children.push(TsxChild::Text(t, line));
                }
            }
            "jsx_expression" => {
                if let Some(inner) = child.named_child(0)
                    && inner.kind() == "identifier"
                {
                    out.children
                        .push(TsxChild::Expr(text(inner, src).to_string(), line));
                }
            }
            _ => {}
        }
    }
    out
}

/// Extract class names from a className value expression.
fn class_refs(node: Node, src: &str, out: &mut TsxNode) {
    match node.kind() {
        "jsx_expression" | "parenthesized_expression" => {
            if let Some(inner) = node.named_child(0) {
                class_refs(inner, src, out);
            }
        }
        "member_expression" => {
            if let Some(prop) = node.child_by_field_name("property") {
                out.classes.push(text(prop, src).to_string());
            }
        }
        "subscript_expression" => {
            if let Some(index) = node.child_by_field_name("index")
                && let Some(frag) = find_kind(index, &["string_fragment"])
            {
                out.classes.push(text(frag, src).to_string());
            }
        }
        "call_expression" => {
            if let Some(args) = node.child_by_field_name("arguments") {
                let mut cursor = args.walk();
                for arg in args.named_children(&mut cursor) {
                    class_refs(arg, src, out);
                }
            }
        }
        "object" => {
            let mut cursor = node.walk();
            for pair in node.named_children(&mut cursor) {
                if pair.kind() != "pair" {
                    continue;
                }
                let class = pair
                    .child_by_field_name("key")
                    .and_then(|k| find_kind(k, &["property_identifier"]))
                    .map(|p| text(p, src).to_string());
                let cond = pair
                    .child_by_field_name("value")
                    .map(|v| text(v, src).to_string());
                if let (Some(class), Some(cond)) = (class, cond) {
                    out.cond_classes
                        .push((class, cond, pair.start_position().row + 1));
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_classes_text_and_exprs() {
        let src = r#"
import styles from "./X.module.sass";
const Card = ({ title, children }: { title: string; children: React.ReactNode }) => (
    <div className={styles["root"]}>
        <span className={styles.label}>{title}</span>
        <button className={cn(styles.b, { [styles.on]: active })}>Go{children}</button>
    </div>
);
export default Card;
"#;
        let c = parse_tsx(src).unwrap();
        assert_eq!(c.name, "Card");
        assert_eq!(c.root.classes, vec!["root"]);
        assert_eq!(c.root.children.len(), 2);
        let TsxChild::Node(button) = &c.root.children[1] else {
            panic!("expected node");
        };
        assert_eq!(button.classes, vec!["b"]);
        assert_eq!(button.cond_classes[0].0, "on");
        assert!(matches!(&button.children[0], TsxChild::Text(t, _) if t == "Go"));
        assert!(matches!(&button.children[1], TsxChild::Expr(e, _) if e == "children"));
    }

    #[test]
    fn rejects_broken_source() {
        assert!(parse_tsx("const X = (=> <div<").is_err());
    }
}
