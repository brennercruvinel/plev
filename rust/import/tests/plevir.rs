//! The plev IR the parser emits (task-71, ADR importers-target-plev-ir),
//! per fixture: the IR and its report as goldens; the IR verified against
//! the registry of the app it is written for, and refused against an empty
//! one naming exactly what the report says the app must bind; every
//! construct of the source in the report, by a census read straight from
//! the parse (independent of the resolver); every IR node with its source.
//!
//! To write the goldens again after an intentional change:
//! UPDATE_GOLDEN=1 cargo test -p import --test ir

// shared with the other test binaries (the html and figma helpers are
// theirs).
#[allow(dead_code)]
mod support;

use import::formats::gpui::parse::{GExpr, GFn};
use import::formats::react::parse::{TsxChild, TsxNode};
use import::tokens::{color_distance, nearest, px_distance, type_distance};
use import::{Category, Need, Tokens, Verdict};
use plevir::{Capabilities, ColorRole, GenerationPolicy, SpacingToken, verify};
use support::checks::*;
use support::*;

/// Every construct of a react source: the component, each element with
/// its attributes and conditional classes, each text and expression child,
/// and each declaration of each rule of the stylesheet.
fn react_census(files: (&str, &str), tsx: &str, sass: &str, vars: &str) -> Vec<Site> {
    fn walk(n: &TsxNode, f: &str, out: &mut Vec<Site>) {
        out.push(site(f, n.line, format!("<{}", n.tag)));
        for a in &n.attrs {
            out.push(site(f, a.line, a.name.clone()));
        }
        for (class, _, line) in &n.cond_classes {
            out.push(site(f, *line, format!("conditional class {class}")));
        }
        for child in &n.children {
            match child {
                TsxChild::Node(m) => walk(m, f, out),
                TsxChild::Text(t, line) => out.push(site(f, *line, format!("\"{t}\""))),
                TsxChild::Expr(e, line) | TsxChild::Other(e, line) => {
                    out.push(site(f, *line, format!("{{{e}}}")))
                }
            }
        }
    }
    let comp = import::formats::react::parse::parse_tsx(tsx).expect("tsx");
    let mut out = vec![site(files.0, comp.line, format!("component {}", comp.name))];
    walk(&comp.root, files.0, &mut out);
    for rule in import::formats::react::sass::parse_sass(sass, vars) {
        for d in &rule.decls {
            out.push(site(files.1, d.line, format!("{}: {}", d.prop, d.value)));
        }
    }
    out
}

/// Every construct of a gpui widget: its `impl RenderOnce`, the base of
/// `render`, and every method call reachable from it (through closures,
/// match arms, call arguments and the `Self::f(..)` helpers it calls).
fn gpui_census(file: &str, src: &str) -> Vec<Site> {
    fn walk<'a>(
        e: &'a GExpr,
        fns: &'a [GFn],
        file: &str,
        seen: &mut Vec<&'a str>,
        out: &mut Vec<Site>,
    ) {
        match e {
            GExpr::Chain { base, calls, .. } => {
                let helper = base
                    .strip_prefix("Self::")
                    .and_then(|b| b.split('(').next());
                if let Some(f) = helper.and_then(|n| fns.iter().find(|f| f.name == n))
                    && !seen.contains(&f.name.as_str())
                {
                    seen.push(f.name.as_str());
                    if let Some(tail) = &f.tail {
                        walk(tail, fns, file, seen, out);
                    }
                }
                for call in calls {
                    out.push(site(file, call.line, format!(".{}(", call.method)));
                    for arg in &call.args {
                        walk(arg, fns, file, seen, out);
                    }
                }
            }
            GExpr::Match { arms, .. } => {
                for arm in arms {
                    walk(&arm.body, fns, file, seen, out);
                }
            }
            GExpr::Closure { body, .. } => walk(body, fns, file, seen, out),
            GExpr::Src { .. } => {}
        }
    }
    let g = import::formats::gpui::parse::parse_gpui(src).expect("gpui");
    let mut out = vec![site(file, g.widget_line, "impl RenderOnce")];
    let render = g.fns.iter().find(|f| f.name == "render").expect("render");
    let tail = render.tail.as_ref().expect("render ends in a chain");
    if let GExpr::Chain {
        base, base_line, ..
    } = tail
    {
        out.push(site(file, *base_line, base.clone()));
    }
    walk(tail, &g.fns, file, &mut Vec::new(), &mut out);
    out
}

#[test]
fn the_separator_ir_and_its_report() {
    let imported = import_gpui();
    goldens(&imported, "gpui");
    let r = &imported.report;
    // the droplist is extended, not lost: each of its entries, at its line
    let out = import::transpile_gpui(("separator.rs", &gpui_source())).expect("gpui");
    for d in &out.dropped {
        let e = r
            .at(&d.at)
            .find(|e| e.construct == d.what)
            .unwrap_or_else(|| panic!("droplist entry {} missing", d.what));
        match e.category() {
            Category::Impossible => {
                assert!(matches!(&e.verdict, Verdict::Impossible { reason } if *reason == d.why))
            }
            Category::Transformed => assert_eq!(d.kind, import::tree::DropKind::Rewritten, "{d:?}"),
            other => panic!("{}: {other:?}", d.what),
        }
    }
    let at = |line: u32, what: &str| {
        r.at(&format!("separator.rs:{line}"))
            .find(|e| e.construct.contains(what))
            .unwrap_or_else(|| panic!("separator.rs:{line} {what}"))
            .category()
    };
    assert_eq!(at(133, ".flex()"), Category::Supported);
    assert_eq!(at(136, ".justify_center()"), Category::Impossible);
    assert_eq!(at(145, ".px_2()"), Category::Approximated);
    assert_eq!(at(148, ".text_xs()"), Category::Approximated);
    assert_eq!(at(82, ".h(px(1.))"), Category::Transformed);
    assert_eq!(at(151, ".child(label)"), Category::Transformed);
    for c in Category::ALL {
        assert!(r.count(c) > 0, "no {c:?} entry");
    }
}

#[test]
fn nothing_of_the_separator_is_lost_or_invented() {
    let imported = import_gpui();
    let src = gpui_source();
    let census = gpui_census("separator.rs", &src);
    nothing_is_lost(&imported.report, &census, &[("separator.rs", &src)]);
    nothing_is_invented(&imported);
}

#[test]
fn the_separator_binds_its_label_and_nothing_is_declared_for_it() {
    let imported = import_gpui();
    assert_eq!(imported.report.bindings.len(), 1);
    refused_for_its_bindings(&imported);
    let verified = accepted_by(&imported, &separator_app("OR"));
    assert!(
        verified.focus_order().is_empty(),
        "a separator is no control"
    );
}

#[test]
fn the_note_card_ir_and_its_report() {
    let imported = import_react();
    goldens(&imported, "react");
    let (tsx, sass, vars) = react_sources();
    let code = import::transpile_react((TSX, &tsx), (SASS, &sass), &vars).expect("react");
    check_golden("react/expected.rs", &code.code);
    let r = &imported.report;
    let at = |at: &str, what: &str| {
        r.at(at)
            .find(|e| e.construct.contains(what))
            .unwrap_or_else(|| panic!("{at} {what}"))
            .category()
    };
    // the fixed width is the card's token, the off-token text color close
    // to text_mid, the asymmetric padding one token, the preview's height
    // and the slot have no IR form
    assert_eq!(at("NoteCard.module.sass:5", "width"), Category::Transformed);
    assert_eq!(
        at("NoteCard.module.sass:42", "color"),
        Category::Approximated
    );
    assert_eq!(
        at("NoteCard.module.sass:28", "padding"),
        Category::Approximated
    );
    assert_eq!(
        at("NoteCard.module.sass:31", "height"),
        Category::Impossible
    );
    assert_eq!(at("index.tsx:22", "{children}"), Category::Impossible);
    // the controls: a component with a role, and the handler a binding
    assert_eq!(at("index.tsx:29", "<button"), Category::Transformed);
    assert_eq!(at("index.tsx:29", "onClick"), Category::Transformed);
    assert_eq!(at("index.tsx:28", "<input"), Category::Transformed);
    for c in Category::ALL {
        assert!(r.count(c) > 0, "no {c:?} entry");
    }
}

#[test]
fn nothing_of_the_note_card_is_lost_or_invented() {
    let imported = import_react();
    let (tsx, sass, vars) = react_sources();
    let census = react_census((TSX, SASS), &tsx, &sass, &vars);
    nothing_is_lost(&imported.report, &census, &[(TSX, &tsx), (SASS, &sass)]);
    nothing_is_invented(&imported);
}

#[test]
fn the_note_cards_controls_need_the_app_and_invent_nothing() {
    let imported = import_react();
    let needs: Vec<String> = imported
        .report
        .bindings
        .iter()
        .map(|b| match &b.need {
            Need::State { reference, .. } => format!("state {reference}"),
            Need::Action {
                action,
                target,
                param,
                ..
            } => {
                format!("action {action} on {target} ({param:?})")
            }
        })
        .collect();
    assert_eq!(
        needs,
        [
            "state note_card.props:title",
            "state note_card.props:content",
            "action reply on note_card.props (Some(\"text\"))",
            "action open on note_card.props (None)",
        ]
    );
    refused_for_its_bindings(&imported);
    let verified = accepted_by(&imported, &note_card_app("A title", "Some content"));
    let focus: Vec<&str> = verified
        .focus_order()
        .iter()
        .map(|id| id.as_str())
        .collect();
    assert_eq!(
        focus,
        [
            "note_card.root.inner.details.reply",
            "note_card.root.inner.details.button"
        ]
    );
    // an app that declares all but one: refused for that one only
    let mut b = accord::SnapshotBuilder::new();
    let mut props = b.node("note_card.props", accord::Role::Group, "note card");
    props.state("title", "t").state("content", "c");
    props.action("reply").param(accord::ParamSpec {
        kind: accord::ValueKind::Text,
        constrained: false,
    });
    let without_open = b.finish().expect("projects");
    let errors = verify(
        &imported.ir,
        &Capabilities::from_snapshot(&without_open),
        &GenerationPolicy::default(),
    )
    .expect_err("open is not declared");
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].reason,
        "the app declares no action `open` on `note_card.props`"
    );
}

/// A `<button>` that holds only its label collapses into that text run
/// for the code emitter; for the IR it is still a button, named by the
/// text, its action named by the handler or, with none, by the label.
#[test]
fn a_button_that_holds_only_its_label_is_still_a_button() {
    let tsx = r#"
const Bar = ({ go }: Props) => (
    <div>
        <button onClick={go}>Go</button>
        <button>Stop now</button>
    </div>
);
"#;
    let imported =
        import::import_react(("bar.tsx", tsx), ("bar.sass", ""), "", &hoff()).expect("imports");
    let kids = &imported.ir.surface.root.children;
    let shown: Vec<(Option<plevir::Component>, Option<&str>)> = kids
        .iter()
        .map(|k| (k.component, k.label.as_deref()))
        .collect();
    assert_eq!(
        shown,
        [
            (Some(plevir::Component::Button), Some("Go")),
            (Some(plevir::Component::Button), Some("Stop now")),
        ]
    );
    let actions: Vec<&str> = imported
        .report
        .bindings
        .iter()
        .filter_map(|b| match &b.need {
            Need::Action { action, .. } => Some(action.as_str()),
            Need::State { .. } => None,
        })
        .collect();
    assert_eq!(actions, ["go", "stop_now"]);
    refused_for_its_bindings(&imported);
    nothing_is_invented(&imported);
}

#[test]
fn an_import_is_deterministic() {
    assert_eq!(import_react(), import_react());
    assert_eq!(import_gpui(), import_gpui());
}

#[test]
fn a_broken_source_is_an_error_and_an_empty_table_loses_values_loudly() {
    assert!(matches!(
        import::import_gpui(("x.rs", "fn broken( {{{"), &hoff()),
        Err(import::ParserError::Parse(_))
    ));
    assert!(
        import::import_react(
            ("x.tsx", "const X = (=> <div<"),
            ("x.sass", ""),
            "",
            &hoff()
        )
        .is_err()
    );
    // with no values to measure against, every measured construct is
    // impossible, said so, and the IR still verifies
    let empty = Tokens::default();
    assert!(!empty.missing().is_empty());
    let imported = import::import_gpui(("separator.rs", &gpui_source()), &empty).expect("imports");
    let padding = imported
        .report
        .at("separator.rs:145")
        .next()
        .expect("the padding");
    assert_eq!(padding.category(), Category::Impossible);
    accepted_by(&imported, &separator_app("OR"));
}

/// The tolerances the ADR states, measured on `Theme::hoff()`: half the
/// distance from each token to its nearest neighbour of another value.
#[test]
fn the_tolerances_measured_on_hoff() {
    let t = hoff();
    assert!(t.missing().is_empty(), "{:?}", t.missing());
    let color = |role: ColorRole| {
        let value = t.color(role).expect("a color");
        nearest(&t.colors, &value, color_distance)
            .expect("near")
            .tolerance
    };
    let close = |a: f32, b: f32| (a - b).abs() < 0.01;
    assert!(close(color(ColorRole::Bg), 1.0));
    assert!(close(color(ColorRole::Surface), 1.0));
    assert!(close(color(ColorRole::BgHover), 3.5));
    assert!(close(color(ColorRole::Divider), 3.5));
    assert!(close(color(ColorRole::TextMid), 25.5));
    assert!(close(color(ColorRole::Text), 31.875));
    let spacing = |s: SpacingToken| {
        let v = t.spacing_of(s).expect("a spacing");
        nearest(&t.spacing, &v, px_distance)
            .expect("near")
            .tolerance
    };
    for (s, tol) in [
        (SpacingToken::Xs, 2.0),
        (SpacingToken::Sm, 2.0),
        (SpacingToken::Md, 2.0),
        (SpacingToken::Lg, 2.0),
        (SpacingToken::Xl, 4.0),
        (SpacingToken::Xxl, 4.0),
    ] {
        assert!(close(spacing(s), tol), "{s:?}");
    }
    let card = nearest(&t.sizes_of(false), &368.0, px_distance).expect("card_w");
    assert!(close(card.tolerance, 16.0));
    let pill = nearest(&t.shapes, &32.0, px_distance).expect("pill");
    assert!(close(pill.tolerance, 4.0));
    let caption = t
        .type_of(plevir::TypographyRole::CaptionR)
        .expect("caption_r");
    let near = nearest(&t.typography, &caption, type_distance).expect("near");
    assert!(close(near.tolerance, 1.0));
}

/// The card of the external corpus, when this checkout has it: the same
/// checks, no golden (the corpus is not in the repo).
#[test]
fn the_corpus_card_when_present() {
    let Some((tsx, sass, vars)) = corpus() else {
        return;
    };
    let imported = import::import_react(
        ("index.tsx", &tsx),
        ("HoffResearchCard.module.sass", &sass),
        &vars,
        &hoff(),
    )
    .expect("the corpus card imports");
    let files = ("index.tsx", "HoffResearchCard.module.sass");
    let census = react_census(files, &tsx, &sass, &vars);
    nothing_is_lost(
        &imported.report,
        &census,
        &[("index.tsx", &tsx), ("HoffResearchCard.module.sass", &sass)],
    );
    nothing_is_invented(&imported);
    refused_for_its_bindings(&imported);
    accepted_by(&imported, &corpus_app("Title", "Content"));
    eprintln!(
        "corpus: {} entries, {:?}",
        imported.report.entries.len(),
        Category::ALL.map(|c| imported.report.count(c))
    );
}
