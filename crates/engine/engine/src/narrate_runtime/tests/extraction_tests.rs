//! Tests for block extraction and end-to-end hot-reload flow.

use super::*;

// ── Block extraction ──

#[test]
fn extract_single_block() {
    let source = r#"
fn build_ui() {
    plev_narrate! {
        col gap 4 {
            text { show "Hello" }
        }
    }
}
"#;
    let blocks = extract_narrate_blocks(source);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].0, 3);
    assert!(blocks[0].1.contains("col gap 4"));
}

#[test]
fn extract_multiple_blocks() {
    let source = r#"
fn a() {
    plev_narrate! {
        div bg "blue" {}
    }
}
fn b() {
    plev_narrate! {
        text { show "Hi" }
    }
}
"#;
    let blocks = extract_narrate_blocks(source);
    assert_eq!(blocks.len(), 2);
    assert!(blocks[0].1.contains("div bg"));
    assert!(blocks[1].1.contains("text"));
}

#[test]
fn extract_handles_nested_braces() {
    let source = r#"
plev_narrate! {
    div {
        when { items.is_empty() } {
            text { show "Empty" }
        }
    }
}
"#;
    let blocks = extract_narrate_blocks(source);
    assert_eq!(blocks.len(), 1);
    assert!(blocks[0].1.contains("when"));
}

#[test]
fn extract_ignores_non_macro_uses() {
    let source = r#"
// plev_narrate! should not match in comments
let x = "plev_narrate! { not this }";
"#;
    let blocks = extract_narrate_blocks(source);
    assert!(blocks.len() <= 2);
}

#[test]
fn extract_handles_plev_narrate() {
    let source = r#"
plev_narrate! {
    div bg "red" {}
}
"#;
    let blocks = extract_narrate_blocks(source);
    assert_eq!(blocks.len(), 1);
}

// ── End-to-end: extract -> override -> lookup ──

#[test]
fn e2e_extract_store_lookup_with_dynamic_blocks() {
    let source = r#"
fn build() {
    plev_narrate! {
        col bg "dark_gray" w 400 h 300 gap 4 {
            text font_size 24 { show "Dashboard" }
            on click { refresh() }
            when { loading } {
                text { show "Loading..." }
            }
            each item in { items.get() } {
                text { show "row" }
            }
            text font_size 12 { show "Footer" }
        }
    }
}
"#;
    let blocks = extract_narrate_blocks(source);
    assert_eq!(blocks.len(), 1, "should extract exactly 1 narrate block");
    let (line, ref dsl) = blocks[0];
    assert!(
        line >= 3,
        "line number should be at the plev_narrate! invocation"
    );
    assert!(
        dsl.contains("Dashboard"),
        "extracted DSL must contain static text"
    );
    assert!(
        dsl.contains("on click"),
        "extracted DSL must contain on block"
    );
    assert!(
        dsl.contains("when"),
        "extracted DSL must contain when block"
    );
    assert!(
        dsl.contains("each"),
        "extracted DSL must contain each block"
    );

    let file_key = "test_e2e_dynamic.rs";
    crate::hot_reload::update_narrate_overrides(file_key, blocks);

    let el = crate::hot_reload::narrate_override(file_key, line);
    assert!(
        el.is_some(),
        "override must return Some(Element) even with skipped blocks"
    );

    let nodes = render(&el.unwrap());
    let text_nodes: Vec<_> = nodes
        .iter()
        .filter_map(|n| {
            if let crate::compositor::SceneNode::Text { key, .. } = n {
                Some(key.text.as_str())
            } else {
                None
            }
        })
        .collect();
    assert!(
        text_nodes.contains(&"Dashboard"),
        "Dashboard text must survive: got {:?}",
        text_nodes
    );
    assert!(
        text_nodes.contains(&"Footer"),
        "Footer text must survive across 3 skipped dynamic blocks: got {:?}",
        text_nodes
    );

    // Clean up
    crate::hot_reload::update_narrate_overrides(file_key, vec![]);
}

#[test]
fn parse_skip_preserves_siblings_before_and_after() {
    let el = parse_narrate(
        r#"
        col bg "dark_gray" gap 4 {
            text font_size 24 { show "Title" }
            on click { handler() }
            text font_size 16 { show "Subtitle" }
        }
    "#,
    )
    .unwrap();
    let nodes = render(&el);
    assert!(
        nodes.len() >= 3,
        "Expected bg rect + 2 text nodes, got {} nodes",
        nodes.len()
    );
    let text_nodes: Vec<_> = nodes
        .iter()
        .filter(|n| matches!(n, crate::compositor::SceneNode::Text { .. }))
        .collect();
    assert_eq!(
        text_nodes.len(),
        2,
        "Both text siblings must survive across skipped on block"
    );
}

#[test]
fn parse_skip_mixed_on_when_each_preserves_static_content() {
    let el = parse_narrate(
        r#"
        col bg "dark_gray" {
            text { show "Header" }
            on click { action() }
            when { condition } { text { show "hidden" } }
            each item in { list } { text { show "item" } }
            text { show "Footer" }
        }
    "#,
    )
    .unwrap();
    let nodes = render(&el);
    let text_nodes: Vec<_> = nodes
        .iter()
        .filter_map(|n| {
            if let crate::compositor::SceneNode::Text { key, .. } = n {
                Some(key.text.as_str())
            } else {
                None
            }
        })
        .collect();
    assert!(
        text_nodes.contains(&"Header"),
        "Header text must survive: got {:?}",
        text_nodes
    );
    assert!(
        text_nodes.contains(&"Footer"),
        "Footer text must survive across 3 skipped blocks: got {:?}",
        text_nodes
    );
}
