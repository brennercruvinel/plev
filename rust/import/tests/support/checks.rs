//! The checks every importer's IR passes (task-71), shared by the test
//! binaries of each source: nothing of the source lost, nothing invented,
//! the bindings refused with nothing declared and accepted by the app, the
//! goldens.

use import::{Imported, Need, Report};
use accord::Snapshot;
use plevir::{Capabilities, GenerationPolicy, IrNode, Rule, verify};

use super::check_golden;

/// One construct the source holds: its line and a text its entry names.
#[derive(Debug)]
pub struct Site {
    pub at: String,
    pub needle: String,
}

pub fn site(file: &str, line: usize, needle: impl Into<String>) -> Site {
    Site {
        at: format!("{file}:{line}"),
        needle: needle.into(),
    }
}

/// Nothing of the source disappears: every census site is named by an
/// entry at its line, or answered for by an entry that covers the line;
/// and every entry points at a line of the source that holds something.
pub fn nothing_is_lost(report: &Report, census: &[Site], sources: &[(&str, &str)]) {
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

pub fn ir_nodes<'a>(n: &'a IrNode, out: &mut Vec<&'a IrNode>) {
    out.push(n);
    for c in &n.children {
        ir_nodes(c, out);
    }
}

/// Nothing appears in the IR without a source: every node is the node of
/// some entry; and every binding of the IR is a binding of the report,
/// one for one.
pub fn nothing_is_invented(imported: &Imported) {
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
pub fn refused_for_its_bindings(imported: &Imported) {
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

pub fn accepted_by(imported: &Imported, app: &Snapshot) -> plevir::VerifiedIr {
    verify(
        &imported.ir,
        &Capabilities::from_snapshot(app),
        &GenerationPolicy::default(),
    )
    .unwrap_or_else(|errors| panic!("{errors:#?}"))
}

pub fn goldens(imported: &Imported, dir: &str) {
    let ir = serde_json::to_string_pretty(&imported.ir).expect("the IR writes") + "\n";
    check_golden(&format!("{dir}/expected.ir.json"), &ir);
    check_golden(
        &format!("{dir}/expected.report.json"),
        &imported.report.to_json_string(),
    );
    // the golden reads back as the same IR, through uir's one reader
    let back = plevir::parse(&ir).unwrap_or_else(|e| panic!("{e:#?}"));
    assert_eq!(back, imported.ir);
}
