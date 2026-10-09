//! The HTML/CSS subset importer (task-71 phase 2): the note card written
//! as a page (`fixtures/html`), its IR, report and builder code as
//! goldens; every construct of the document in the report, by a census
//! read straight from the parse; the IR refused with nothing declared and
//! verified against the same app the react note card is written for.
//!
//! To write the goldens again after an intentional change:
//! UPDATE_GOLDEN=1 cargo test -p import --test html

// shared with the other test binaries: this one reads the html fixture.
#[allow(dead_code)]
mod support;

use import::Category;
use import::formats::html::parse::{HtmlChild, HtmlNode};
use support::checks::*;
use support::*;

const FILE: &str = HTML;

fn source() -> String {
    fixture("html/note-card.html")
}

/// Every construct of the document: each element with its attributes (and
/// the declarations of its `style=""`), each text, each declaration of
/// each rule, each at-rule, and the lines of each script.
fn census(src: &str) -> Vec<Site> {
    fn walk(n: &HtmlNode, out: &mut Vec<Site>) {
        out.push(site(FILE, n.line, format!("<{}", n.tag)));
        for a in &n.attrs {
            out.push(site(FILE, a.line, a.name.clone()));
            for d in &a.decls {
                out.push(site(FILE, d.line, d.prop.clone()));
            }
        }
        if n.tag == "script"
            && let Some((body, start)) = &n.raw
        {
            for (i, l) in body.lines().enumerate() {
                if !l.trim().is_empty() {
                    out.push(site(FILE, start + i, "script"));
                }
            }
        }
        for c in &n.children {
            match c {
                HtmlChild::Element(m) => walk(m, out),
                HtmlChild::Text(t, line) => out.push(site(FILE, *line, format!("\"{t}\""))),
            }
        }
    }
    let doc = import::formats::html::parse::parse_html(src).expect("html");
    let mut out = Vec::new();
    for n in &doc.nodes {
        walk(n, &mut out);
    }
    for rule in &doc.rules {
        if rule.at_rule.is_some() {
            out.push(site(FILE, rule.line, rule.selector.clone()));
        }
        for d in &rule.decls {
            out.push(site(FILE, d.line, format!("{}: {}", d.prop, d.value)));
        }
    }
    out
}

#[test]
fn the_html_note_card_ir_its_report_and_its_code() {
    let imported = import_html();
    goldens(&imported, "html");
    let code = import::transpile_html((FILE, &source())).expect("html");
    check_golden("html/expected.rs", &code.code);
    let r = &imported.report;
    let at = |line: u32, what: &str| {
        r.at(&format!("{FILE}:{line}"))
            .find(|e| e.construct.contains(what))
            .unwrap_or_else(|| panic!("{FILE}:{line} {what}"))
            .category()
    };
    // flex is the stack; values go to tokens by the phase 1 measure
    assert_eq!(at(15, "display"), Category::Supported);
    assert_eq!(at(17, "gap"), Category::Approximated);
    assert_eq!(at(44, "gap"), Category::Transformed);
    assert_eq!(at(18, "width"), Category::Transformed);
    assert_eq!(at(20, "background"), Category::Transformed);
    assert_eq!(at(27, "color"), Category::Approximated);
    // what the subset does not read, each at its line
    assert_eq!(at(22, "box-shadow"), Category::Impossible);
    assert_eq!(at(61, ".open:hover"), Category::Impossible);
    assert_eq!(at(65, "position"), Category::Impossible);
    assert_eq!(at(71, "display"), Category::Impossible);
    assert_eq!(at(74, ".note-card p + p"), Category::Impossible);
    assert_eq!(at(77, "@media"), Category::Impossible);
    assert_eq!(at(94, "<ul> default padding"), Category::Impossible);
    assert_eq!(at(96, "<script"), Category::Impossible);
    // the controls: a component with a role, the handler a binding
    assert_eq!(at(90, "<input"), Category::Transformed);
    assert_eq!(at(90, "oninput"), Category::Transformed);
    assert_eq!(at(91, "<button"), Category::Transformed);
    assert_eq!(at(91, "onclick"), Category::Transformed);
    for c in Category::ALL {
        assert!(r.count(c) > 0, "no {c:?} entry");
    }
}

#[test]
fn nothing_of_the_html_note_card_is_lost_or_invented() {
    let imported = import_html();
    let src = source();
    nothing_is_lost(&imported.report, &census(&src), &[(FILE, &src)]);
    nothing_is_invented(&imported);
}

#[test]
fn the_html_note_card_needs_the_same_app_as_the_react_one() {
    let imported = import_html();
    assert_eq!(imported.ir.surface.id.as_str(), "note_card");
    let actions: Vec<String> = imported
        .report
        .bindings
        .iter()
        .map(|b| match &b.need {
            import::Need::Action { action, .. } => action.clone(),
            import::Need::State { reference, .. } => reference.clone(),
        })
        .collect();
    assert_eq!(actions, ["reply", "open"]);
    refused_for_its_bindings(&imported);
    let verified = accepted_by(&imported, &note_card_app("unused", "unused"));
    assert_eq!(verified.focus_order().len(), 2, "the field and the button");
}

#[test]
fn an_html_import_is_deterministic_and_a_broken_one_an_error() {
    assert_eq!(import_html(), import_html());
    let broken = import::import_html(("x.html", "<style>.a { color: }}}</style>"), &hoff());
    assert!(matches!(broken, Err(import::ParserError::Parse(_))));
}
