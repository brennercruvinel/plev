//! Stage 1 (figma, task-71 phase 2): a Figma export (the REST API's JSON
//! shape, as `script/figma-fixture` writes it from a .fig) -> its node
//! tree, every node and every key with its line.
//!
//! `serde_json` keeps no position, so the lines come from the text: a node
//! is the line of its `"id"` (ids are unique in an export), and each of its
//! keys the first line after it that starts with that key. That holds for
//! the writer's shape (one key per line, `id` first, `children` last), which
//! is what the importer reads; any other shape still parses, with the keys
//! it cannot place at the node's line.

use serde_json::{Map, Value};

use crate::ParserError;

/// One node of the export.
#[derive(Debug, Clone)]
pub struct FigNode {
    pub id: String,
    pub name: String,
    /// The node's `type` (`FRAME`, `TEXT`, `INSTANCE`).
    pub kind: String,
    /// The line of its `"id"`.
    pub line: usize,
    /// Every key but `id`, `name`, `type` and `children`, with its value
    /// and its line, in the export's order.
    pub keys: Vec<(String, Value, usize)>,
    /// The line of `"name"`.
    pub name_line: usize,
    pub children: Vec<FigNode>,
}

impl FigNode {
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.keys
            .iter()
            .find(|(k, _, _)| k == key)
            .map(|(_, v, _)| v)
    }

    pub fn line_of(&self, key: &str) -> usize {
        self.keys
            .iter()
            .find(|(k, _, _)| k == key)
            .map_or(self.line, |(_, _, l)| *l)
    }

    pub fn num(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(Value::as_f64).map(|v| v as f32)
    }

    pub fn str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_str)
    }

    pub fn visible(&self) -> bool {
        self.get("visible").and_then(Value::as_bool) != Some(false)
    }

    /// `absoluteBoundingBox` as x, y, width, height.
    pub fn bounds(&self) -> Option<[f32; 4]> {
        let b = self.get("absoluteBoundingBox")?;
        let f = |k: &str| b.get(k).and_then(Value::as_f64).map(|v| v as f32);
        Some([f("x")?, f("y")?, f("width")?, f("height")?])
    }
}

/// A parsed export: the file's name and its document.
#[derive(Debug, Clone)]
pub struct FigDoc {
    pub name: String,
    pub document: FigNode,
}

pub fn parse_figma(src: &str) -> Result<FigDoc, ParserError> {
    let root: Value =
        serde_json::from_str(src).map_err(|e| ParserError::Parse(format!("figma json: {e}")))?;
    let name = root
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let document = root
        .get("document")
        .and_then(Value::as_object)
        .ok_or_else(|| ParserError::Parse("figma json: no `document`".into()))?;
    let lines: Vec<&str> = src.lines().collect();
    Ok(FigDoc {
        name,
        document: node(document, &lines)?,
    })
}

/// The first line at or after `from` whose text starts with `"key":`.
fn find(lines: &[&str], from: usize, key: &str) -> Option<usize> {
    let quoted = format!("{}:", Value::String(key.into()));
    lines
        .iter()
        .enumerate()
        .skip(from.saturating_sub(1))
        .find(|(_, l)| l.trim_start().starts_with(&quoted))
        .map(|(i, _)| i + 1)
}

fn node(obj: &Map<String, Value>, lines: &[&str]) -> Result<FigNode, ParserError> {
    let text = |k: &str| obj.get(k).and_then(Value::as_str).unwrap_or_default();
    let id = text("id").to_string();
    if id.is_empty() {
        return Err(ParserError::Parse("figma json: a node with no `id`".into()));
    }
    let needle = format!("\"id\": {}", Value::String(id.clone()));
    let line = lines
        .iter()
        .position(|l| l.contains(&needle))
        .map_or(1, |i| i + 1);
    let mut keys = Vec::new();
    for (k, v) in obj {
        if matches!(k.as_str(), "id" | "name" | "type" | "children") {
            continue;
        }
        let at = find(lines, line, k).unwrap_or(line);
        keys.push((k.clone(), v.clone(), at));
    }
    keys.sort_by_key(|(_, _, l)| *l);
    let mut children = Vec::new();
    for c in obj
        .get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let c = c.as_object().ok_or_else(|| {
            ParserError::Parse(format!("figma json: a child of {id} is no object"))
        })?;
        children.push(node(c, lines)?);
    }
    Ok(FigNode {
        name: text("name").to_string(),
        kind: text("type").to_string(),
        name_line: find(lines, line, "name").unwrap_or(line),
        id,
        line,
        keys,
        children,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = r#"{
  "name": "demo",
  "document": {
    "id": "0:0",
    "name": "Document",
    "type": "DOCUMENT",
    "children": [
      {
        "id": "1:2",
        "name": "card",
        "type": "FRAME",
        "absoluteBoundingBox": {"x": 10, "y": 20, "width": 368, "height": 100},
        "layoutMode": "VERTICAL",
        "children": [
          {
            "id": "1:3",
            "name": "title",
            "type": "TEXT",
            "visible": false,
            "characters": "Hi"
          }
        ]
      }
    ]
  }
}"#;

    #[test]
    fn nodes_and_keys_with_their_lines() {
        let doc = parse_figma(SRC).expect("parses");
        assert_eq!(doc.name, "demo");
        let card = &doc.document.children[0];
        assert_eq!(
            (card.kind.as_str(), card.line, card.name_line),
            ("FRAME", 9, 10)
        );
        assert_eq!(card.line_of("layoutMode"), 13);
        assert_eq!(card.bounds(), Some([10.0, 20.0, 368.0, 100.0]));
        let title = &card.children[0];
        assert_eq!((title.line, title.line_of("characters")), (16, 20));
        assert!(!title.visible());
        assert_eq!(title.str("characters"), Some("Hi"));
    }

    #[test]
    fn not_an_export_is_an_error() {
        assert!(parse_figma("{}").is_err());
        assert!(parse_figma("{\"document\": {\"type\": \"DOCUMENT\"}}").is_err());
        assert!(parse_figma("[").is_err());
    }
}
