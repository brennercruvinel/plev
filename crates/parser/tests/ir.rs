//! The plev IR the parser emits (task-71, ADR importers-target-plev-ir),
//! per fixture: the IR and its report as goldens; the IR verified against
//! the registry of the app it is written for, and refused against an empty
//! one naming exactly what the report says the app must bind; every
//! construct of the source in the report, by a census read straight from
//! the parse (independent of the resolver); every IR node with its source.
//!
//! To write the goldens again after an intentional change:
//! UPDATE_GOLDEN=1 cargo test -p parser --test ir

mod support;

use parser::gpui::{GExpr, GFn};
use parser::tokens::{color_distance, nearest, px_distance, type_distance};
use parser::{Category, Imported, Need, Report, Tokens, Verdict};
use sem::Snapshot;
use support::*;
use uir::{Capabilities, ColorRole, GenerationPolicy, IrNode, Rule, SpacingToken, verify};

/// One construct the source holds: its line and a text its entry names.
#[derive(Debug)]
struct Site {
    at: String,
    needle: String,
}

fn site(file: &str, line: usize, needle: impl Into<String>) -> Site {
    Site {
        at: format!("{file}:{line}"),
        needle: needle.into(),
    }
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
    let g = parser::gpui::parse_gpui(src).expect("gpui");
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

/// Nothing of the source disappears: every census site is named by an
/// entry at its line, or answered for by an entry that covers the line;
/// and every entry points at a line of the source that holds something.
fn nothing_is_lost(report: &Report, census: &[Site], sources: &[(&str, &str)]) {
    assert!(!census.is_empty());
    for s in census {
        let named = report
            .entries
            .iter()
            .any(|e| e.at == s.at && e.construct.contains(&s.needle));
        let covered = report.entries.iter().any(|e| e.covers.contains(&s.at));
        assert!(
            named || covered,
            "{} `{}` is in no entry of the report",
            s.at,
            s.needle
        );
    }
    for e in &report.entries {
        for at in std::iter::once(&e.at).chain(&e.covers) {
            let (file, line) = at.rsplit_once(':').expect("file:line");
            let line: usize = line.parse().expect("a line number");
            let (_, text) = sources
                .iter()
                .find(|(f, _)| *f == file)
                .unwrap_or_else(|| panic!("{at}: not a file of the source"));
            let held = text.lines().nth(line.wrapping_sub(1));
            assert!(
                held.is_some_and(|l| !l.trim().is_empty()),
                "{at}: no construct on that line ({})",
                e.construct
            );
        }
    }
}

fn ir_nodes<'a>(n: &'a IrNode, out: &mut Vec<&'a IrNode>) {
    out.push(n);
    for c in &n.children {
        ir_nodes(c, out);
    }
}

/// Nothing appears in the IR without a source: every node is the node of
/// some entry; and every binding of the IR is a binding of the report,
/// one for one.
fn nothing_is_invented(imported: &Imported) {
    let mut nodes = Vec::new();
    ir_nodes(&imported.ir.surface.root, &mut nodes);
    let report = &imported.report;
    for n in &nodes {
        let id = n.sem.as_str();
        assert!(
            report.entries.iter().any(|e| e.node.as_deref() == Some(id)),
            "{id} has no source construct in the report"
        );
    }
    let mut bound = 0;
    for n in &nodes {
        for (prop, reference) in &n.bind.state {
            bound += 1;
            let listed = report.bindings.iter().any(|b| {
                b.node == n.sem.as_str()
                    && matches!(&b.need, Need::State { prop: p, reference: r, .. }
                        if p == prop && *r == reference.to_string())
            });
            assert!(listed, "{}: state {prop} is bound and not reported", n.sem);
        }
        for (event, cap) in &n.bind.actions {
            bound += 1;
            let listed = report.bindings.iter().any(|b| {
                b.node == n.sem.as_str()
                    && matches!(&b.need, Need::Action { event: e, target, action, .. }
                        if e == event && *target == cap.target.to_string() && *action == cap.action.to_string())
            });
            assert!(
                listed,
                "{}: action {event} is bound and not reported",
                n.sem
            );
        }
    }
    assert_eq!(
        bound,
        report.bindings.len(),
        "a binding reported and not bound"
    );
}

/// With nothing declared, the verifier refuses the IR once per binding the
/// report lists, naming it; nothing else is refused.
fn refused_for_its_bindings(imported: &Imported) {
    let errors = verify(
        &imported.ir,
        &Capabilities::default(),
        &GenerationPolicy::default(),
    )
    .expect_err("an IR that binds the app's capabilities needs them declared");
    let bindings = &imported.report.bindings;
    assert_eq!(errors.len(), bindings.len(), "{errors:#?}");
    for b in bindings {
        let (rule, reason, tail) = match &b.need {
            Need::State {
                prop, reference, ..
            } => (
                Rule::State,
                format!("the app declares no state `{reference}`"),
                format!("/bind/state/{prop}"),
            ),
            Need::Action {
                event,
                target,
                action,
                ..
            } => (
                Rule::Action,
                format!("the app declares no action `{action}` on `{target}`"),
                format!("/bind/actions/{event}"),
            ),
        };
        assert!(
            errors
                .iter()
                .any(|e| e.rule == rule && e.reason == reason && e.pointer.ends_with(&tail)),
            "no refusal for {b:?} in {errors:#?}"
        );
    }
}

fn accepted_by(imported: &Imported, app: &Snapshot) -> uir::VerifiedIr {
    verify(
        &imported.ir,
        &Capabilities::from_snapshot(app),
        &GenerationPolicy::default(),
    )
    .unwrap_or_else(|errors| panic!("{errors:#?}"))
}

fn goldens(imported: &Imported, dir: &str) {
    let ir = serde_json::to_string_pretty(&imported.ir).expect("the IR writes") + "\n";
    check_golden(&format!("{dir}/expected.ir.json"), &ir);
    check_golden(
        &format!("{dir}/expected.report.json"),
        &imported.report.to_json_string(),
    );
    // the golden reads back as the same IR, through uir's one reader
    let back = uir::parse(&ir).unwrap_or_else(|e| panic!("{e:#?}"));
    assert_eq!(back, imported.ir);
}

#[test]
fn the_separator_ir_and_its_report() {
    let imported = import_gpui();
    goldens(&imported, "gpui");
    let r = &imported.report;
    // the droplist is extended, not lost: each of its entries, at its line
    let out = parser::transpile_gpui(("separator.rs", &gpui_source())).expect("gpui");
    for d in &out.dropped {
        let e = r
            .at(&d.at)
            .find(|e| e.construct == d.what)
            .unwrap_or_else(|| panic!("droplist entry {} missing", d.what));
        match e.category() {
            Category::Impossible => {
                assert!(matches!(&e.verdict, Verdict::Impossible { reason } if *reason == d.why))
            }
            Category::Transformed => assert_eq!(d.kind, parser::ir::DropKind::Rewritten, "{d:?}"),
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
fn an_import_is_deterministic() {
    assert_eq!(import_gpui(), import_gpui());
}

#[test]
fn a_broken_source_is_an_error_and_an_empty_table_loses_values_loudly() {
    assert!(matches!(
        parser::import_gpui(("x.rs", "fn broken( {{{"), &hoff()),
        Err(parser::ParserError::Parse(_))
    ));
    assert!(
        parser::import_react(
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
    let imported = parser::import_gpui(("separator.rs", &gpui_source()), &empty).expect("imports");
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
    let caption = t.type_of(uir::TypographyRole::CaptionR).expect("caption_r");
    let near = nearest(&t.typography, &caption, type_distance).expect("near");
    assert!(close(near.tolerance, 1.0));
}
