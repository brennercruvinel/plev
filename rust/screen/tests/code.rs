//! A `code` materialized (task-97): the comps `CodeBlock` over the app's
//! lines, each row with the kind the app's `kinds` names, drawn by the
//! theme's tokens; as tall as its rows, or bounded by `size.max` and then
//! scrolled inside by the wheel, the offset never projected; projected as
//! a region named by its label, with the lines and kinds it shows.

// shared with the other surface tests: this binary uses the app and ids.
#[allow(dead_code)]
mod support;

use accord::{Role, Snapshot, SnapshotBuilder, Value};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode};
use engine::styling::theme::Theme;
use engine::widgets::content::{CodeBlock, LineKind as Row};
use engine::widgets::{Rect, WidgetEvent};
use plevir::{Capabilities, GenerationPolicy, LineKind, verify_json};
use screen::Surface;
use support::{App, id, page};

/// A diff of `n` lines: a header, then removed, added and context lines
/// in turn, and a name no kind has at the end.
fn diff(n: usize) -> Snapshot {
    let mut lines = Vec::new();
    let mut kinds = Vec::new();
    for i in 0..n {
        let (line, kind) = match i {
            0 => ("src/lib.rs: @@ -1,9 +1,9 @@".to_owned(), LineKind::Header),
            _ if i % 3 == 1 => (format!("-old {i}"), LineKind::Removed),
            _ if i % 3 == 2 => (format!("+new {i}"), LineKind::Added),
            _ => (format!(" same {i}"), LineKind::Context),
        };
        lines.push(line);
        kinds.push(kind.as_str().to_owned());
    }
    if let Some(last) = kinds.last_mut() {
        "sideways".clone_into(last);
    }
    let mut b = SnapshotBuilder::new();
    b.node("diff", Role::Region, "diff")
        .state("lines", lines)
        .state("kinds", kinds)
        .state("text", "fn main() {\n    run();\n}\n");
    b.finish().expect("the diff projects")
}

/// A surface with one code over `diff:lines` and `diff:kinds`, bounded
/// by the one height token when `bounded`.
fn ir(bounded: bool, lines: &str) -> String {
    let size = if bounded {
        r#""size": {"max": "dropdown_max_h"},"#
    } else {
        ""
    };
    format!(
        r#"{{"plev_ir": 1, "surface": {{"id": "s", "title": "the diff", "root": {{
            "sem": "s.root",
            "children": [{{
                "sem": "s.code", "component": "code", "label": "diff", {size}
                "bind": {{"state": {{"lines": "{lines}", "kinds": "diff:kinds"}}}}
            }}]
        }}}}}}"#
    )
}

fn surface(app: &Snapshot, bounded: bool, lines: &str) -> Surface {
    let caps = Capabilities::from_snapshot(app);
    let ir = verify_json(&ir(bounded, lines), &caps, &GenerationPolicy::default())
        .unwrap_or_else(|e| panic!("{e:#?}"));
    Surface::materialize(ir, app)
}

const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 480.0,
    h: 2000.0,
};

fn drawn(surface: &mut Surface, theme: &Theme) -> (Compositor, [f32; 4]) {
    let mut c = Compositor::new();
    surface.render(&mut c, AREA, theme);
    let rect = c.binding(&id("s.code")).expect("the code is bound").rect;
    (c, rect)
}

fn nodes(c: &Compositor) -> Vec<SceneNode> {
    c.layer(LayerId::DEFAULT)
        .map(|l| l.nodes().to_vec())
        .unwrap_or_default()
}

/// The rows drawn: each text with its ink, in order (the caption first).
fn texts(c: &Compositor) -> Vec<(String, [f32; 4])> {
    nodes(c)
        .into_iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, color, .. } => Some((key.text, color)),
            _ => None,
        })
        .collect()
}

fn washes(c: &Compositor) -> Vec<[f32; 4]> {
    nodes(c)
        .into_iter()
        .filter_map(|n| match n {
            SceneNode::Rect { color, .. } => Some(color),
            _ => None,
        })
        .collect()
}

#[test]
fn every_line_shows_with_the_kind_the_app_names() {
    let theme = Theme::hoff();
    let app = diff(5);
    let mut s = surface(&app, false, "diff:lines");
    let (c, rect) = drawn(&mut s, &theme);
    let shown = texts(&c);
    let rows: Vec<&str> = shown.iter().skip(1).map(|(t, _)| t.as_str()).collect();
    assert_eq!(shown[0].0, "diff", "the label is the caption");
    assert_eq!(
        rows,
        [
            "src/lib.rs: @@ -1,9 +1,9 @@",
            "-old 1",
            "+new 2",
            " same 3",
            "-old 4"
        ]
    );
    assert!(
        shown
            .iter()
            .skip(1)
            .all(|(_, ink)| *ink == theme.colors.text.0)
    );
    let wash = |row: Row| row.wash(&theme).expect("a wash");
    assert_eq!(
        washes(&c),
        [wash(Row::Header), wash(Row::Removed), wash(Row::Added)],
        "header, removed, added; none behind context, nor behind a name no kind has"
    );
    let full = CodeBlock::lines(vec![Default::default(); 5])
        .caption("diff")
        .height_for(0.0, &theme);
    assert!(
        (rect[3] - full).abs() < 0.5,
        "as tall as its rows: {rect:?}"
    );
}

#[test]
fn a_code_shows_one_text_a_row_per_line() {
    let theme = Theme::hoff();
    let app = diff(2);
    let mut s = surface(&app, false, "diff:text");
    let (c, _) = drawn(&mut s, &theme);
    let rows: Vec<String> = texts(&c).into_iter().skip(1).map(|(t, _)| t).collect();
    assert_eq!(rows, ["fn main() {", "    run();", "}"]);
}

#[test]
fn a_bounded_code_scrolls_its_rows_with_the_wheel_and_projects_no_offset() {
    let theme = Theme::hoff();
    let app = diff(40);
    let mut s = surface(&app, true, "diff:lines");
    let mut dispatch = App::over(&app);
    let (c, rect) = drawn(&mut s, &theme);
    assert!(
        (rect[3] - theme.size.dropdown_max_h).abs() < 0.5,
        "bounded by the token: {rect:?}"
    );
    let first = texts(&c)[1].0.clone();
    assert_eq!(first, "src/lib.rs: @@ -1,9 +1,9 @@");
    let shown = texts(&c).len() - 1;
    assert!(shown < 40, "only the rows that fit are drawn: {shown}");
    let before = page(&app, &s);
    let row = CodeBlock::row_h(&theme);
    let (x, y) = (rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0);
    let wheel = WidgetEvent::Scroll {
        x,
        y,
        delta: row * 3.0,
    };
    let moved = s.handle_event(&wheel, &theme, &mut dispatch);
    assert!(moved.changed, "the wheel moves the rows");
    assert!(
        dispatch.executed.is_empty(),
        "scrolling asks nothing of the app"
    );
    let (c, _) = drawn(&mut s, &theme);
    assert_eq!(texts(&c)[1].0, " same 3", "the fourth row at the top");
    assert_eq!(page(&app, &s), before, "the offset is never projected");
    // far down it stops at the last row; outside the block the wheel is not its
    let far = WidgetEvent::Scroll {
        x,
        y,
        delta: 100_000.0,
    };
    s.handle_event(&far, &theme, &mut dispatch);
    let (c, _) = drawn(&mut s, &theme);
    assert_eq!(texts(&c).last().map(|t| t.0.as_str()), Some(" same 39"));
    let outside = WidgetEvent::Scroll {
        x: rect[0] + rect[2] / 2.0,
        y: rect[1] + rect[3] + 40.0,
        delta: -row,
    };
    assert!(!s.handle_event(&outside, &theme, &mut dispatch).changed);
    // other lines start at the top again
    let fresh = diff(30);
    s.sync(&fresh);
    let (c, _) = drawn(&mut s, &theme);
    assert_eq!(texts(&c)[1].0, "src/lib.rs: @@ -1,9 +1,9 @@");
}

#[test]
fn a_code_projects_as_a_region_named_by_its_label_with_what_it_shows() {
    let app = diff(3);
    let s = surface(&app, true, "diff:lines");
    let page = page(&app, &s);
    let node = page.get(&id("s.code")).expect("projected");
    assert_eq!(node.role, Role::Region);
    assert_eq!(node.label, "diff");
    let lines = app.value(&"diff:lines".parse().unwrap()).cloned();
    let kinds = app.value(&"diff:kinds".parse().unwrap()).cloned();
    assert_eq!(page.value(&"s.code:lines".parse().unwrap()).cloned(), lines);
    assert_eq!(page.value(&"s.code:kinds".parse().unwrap()).cloned(), kinds);
    assert!(node.actions.is_empty(), "read, not operated");
    assert!(matches!(lines, Some(Value::List(l)) if l.len() == 3));
}
