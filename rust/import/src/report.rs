//! The report of the plev IR emitter: every construct of the source in
//! exactly one of four categories, each with its `file:line`, and the
//! bindings the app has to make. It extends the droplist: every droplist
//! entry is an entry here, at the same `file:line`.
//!
//! The categories:
//!
//! - `supported`: the IR says it as the source does (`ir` says where);
//! - `transformed`: the IR says it by a documented rule, with no visual
//!   difference in the theme the source was mapped to (`ir`, `rule`);
//! - `approximated`: the IR says something close (`ir`, `difference`);
//! - `impossible`: the IR cannot say it and it is left out (`reason`).
//!
//! The JSON form ([`Report::to_json`]) is a contract with whoever reads it.
//! Its names are fixed here and written in key order:
//!
//! ```text
//! { "bindings": [ binding ], "counts": { "approximated": n, "impossible": n,
//!   "supported": n, "transformed": n }, "entries": [ entry ],
//!   "plev_report": 1, "source": "...", "surface": "<surface id>" }
//! entry:   { "at": "file:line", "category": "...", "construct": "...",
//!            "covers": ["file:line"]?, "difference": "..."?, "ir": "..."?,
//!            "node": "<IR node id>"?, "reason": "..."?, "rule": "..."? }
//! binding: { "at", "binds": "state", "kind", "node", "prop", "ref", "why" }
//!        | { "action", "at", "binds": "action", "event", "node",
//!            "param": "<kind>" | null, "target", "why" }
//! ```
//!
//! `covers` lists the other lines an entry answers for (the declarations
//! of a rule dropped whole, the elements of a dropped subtree). A
//! `binding` names a state or an action the IR binds that the source does
//! not declare: the app declares it, or the verifier refuses the IR saying
//! which one is missing. The importer never declares one.

use std::collections::HashMap;

use serde_json::{Map, Value, json};

use crate::ir::Src;

/// The version of the JSON form, the value of `plev_report`.
pub const FORMAT: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Supported,
    Transformed,
    Approximated,
    Impossible,
}

impl Category {
    pub const ALL: [Category; 4] = [
        Category::Supported,
        Category::Transformed,
        Category::Approximated,
        Category::Impossible,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Category::Supported => "supported",
            Category::Transformed => "transformed",
            Category::Approximated => "approximated",
            Category::Impossible => "impossible",
        }
    }
}

/// What became of one construct.
#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    Supported { ir: String },
    Transformed { ir: String, rule: String },
    Approximated { ir: String, difference: String },
    Impossible { reason: String },
}

impl Verdict {
    pub fn category(&self) -> Category {
        match self {
            Verdict::Supported { .. } => Category::Supported,
            Verdict::Transformed { .. } => Category::Transformed,
            Verdict::Approximated { .. } => Category::Approximated,
            Verdict::Impossible { .. } => Category::Impossible,
        }
    }

    /// The same verdict with `other`'s texts joined in, when both are of
    /// one category.
    fn joined(&self, other: &Verdict) -> Verdict {
        let join = |a: &str, b: &str| {
            if a.contains(b) {
                a.to_string()
            } else {
                format!("{a}; {b}")
            }
        };
        match (self, other) {
            (Verdict::Supported { ir: a }, Verdict::Supported { ir: b }) => {
                Verdict::Supported { ir: join(a, b) }
            }
            (Verdict::Transformed { ir: a, rule: r }, Verdict::Transformed { ir: b, rule: s }) => {
                Verdict::Transformed {
                    ir: join(a, b),
                    rule: join(r, s),
                }
            }
            (
                Verdict::Approximated {
                    ir: a,
                    difference: d,
                },
                Verdict::Approximated {
                    ir: b,
                    difference: e,
                },
            ) => Verdict::Approximated {
                ir: join(a, b),
                difference: join(d, e),
            },
            (Verdict::Impossible { reason: a }, Verdict::Impossible { reason: b }) => {
                Verdict::Impossible { reason: join(a, b) }
            }
            _ => self.clone(),
        }
    }
}

/// One construct of the source and what became of it.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// `file:line`.
    pub at: String,
    /// The construct, as the droplist names it.
    pub construct: String,
    /// The IR node it landed on.
    pub node: Option<String>,
    pub verdict: Verdict,
    /// Other `file:line`s this entry answers for.
    pub covers: Vec<String>,
}

impl Entry {
    pub fn category(&self) -> Category {
        self.verdict.category()
    }
}

/// What the app has to declare for the IR to verify.
#[derive(Clone, Debug, PartialEq)]
pub enum Need {
    /// The node shows `prop` from the app's state `reference` (`<id>:<key>`),
    /// a value of `kind`.
    State {
        prop: String,
        reference: String,
        kind: String,
    },
    /// The node's `event` runs the app's action `action` on `target`,
    /// which takes a value of `param` (none for a plain activation).
    Action {
        event: String,
        target: String,
        action: String,
        param: Option<String>,
    },
}

/// A binding the IR names and the source does not declare.
#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    /// The construct that asks for it.
    pub at: String,
    /// The IR node that binds it.
    pub node: String,
    pub need: Need,
    pub why: String,
}

/// The report of one import.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub source: String,
    /// The IR surface's id.
    pub surface: String,
    /// In source order: file, line, construct.
    pub entries: Vec<Entry>,
    /// In tree order.
    pub bindings: Vec<Binding>,
}

impl Report {
    pub fn count(&self, category: Category) -> usize {
        self.entries
            .iter()
            .filter(|e| e.category() == category)
            .count()
    }

    /// The entries at `at`.
    pub fn at<'a>(&'a self, at: &'a str) -> impl Iterator<Item = &'a Entry> + 'a {
        self.entries.iter().filter(move |e| e.at == at)
    }

    /// The JSON form, the contract documented on this module.
    pub fn to_json(&self) -> Value {
        let counts = Category::ALL
            .iter()
            .map(|c| (c.as_str(), json!(self.count(*c))))
            .collect();
        object(vec![
            (
                "bindings",
                Value::Array(self.bindings.iter().map(binding_json).collect()),
            ),
            ("counts", object(counts)),
            (
                "entries",
                Value::Array(self.entries.iter().map(entry_json).collect()),
            ),
            ("plev_report", json!(FORMAT)),
            ("source", json!(self.source)),
            ("surface", json!(self.surface)),
        ])
    }

    /// The JSON form, pretty, with a final newline.
    pub fn to_json_string(&self) -> String {
        let mut text = serde_json::to_string_pretty(&self.to_json()).unwrap_or_default();
        text.push('\n');
        text
    }
}

/// An object with its keys in order, whatever map serde_json was built
/// with: the key order of the form never depends on a cargo feature.
fn object(mut pairs: Vec<(&str, Value)>) -> Value {
    pairs.sort_by(|a, b| a.0.cmp(b.0));
    let mut map = Map::new();
    for (k, v) in pairs {
        map.insert(k.to_string(), v);
    }
    Value::Object(map)
}

fn entry_json(e: &Entry) -> Value {
    let mut pairs = vec![
        ("at", json!(e.at)),
        ("category", json!(e.category().as_str())),
        ("construct", json!(e.construct)),
    ];
    if !e.covers.is_empty() {
        pairs.push(("covers", json!(e.covers)));
    }
    if let Some(node) = &e.node {
        pairs.push(("node", json!(node)));
    }
    match &e.verdict {
        Verdict::Supported { ir } => pairs.push(("ir", json!(ir))),
        Verdict::Transformed { ir, rule } => {
            pairs.push(("ir", json!(ir)));
            pairs.push(("rule", json!(rule)));
        }
        Verdict::Approximated { ir, difference } => {
            pairs.push(("ir", json!(ir)));
            pairs.push(("difference", json!(difference)));
        }
        Verdict::Impossible { reason } => pairs.push(("reason", json!(reason))),
    }
    object(pairs)
}

fn binding_json(b: &Binding) -> Value {
    let mut pairs = vec![
        ("at", json!(b.at)),
        ("node", json!(b.node)),
        ("why", json!(b.why)),
    ];
    match &b.need {
        Need::State {
            prop,
            reference,
            kind,
        } => {
            pairs.push(("binds", json!("state")));
            pairs.push(("prop", json!(prop)));
            pairs.push(("ref", json!(reference)));
            pairs.push(("kind", json!(kind)));
        }
        Need::Action {
            event,
            target,
            action,
            param,
        } => {
            pairs.push(("binds", json!("action")));
            pairs.push(("event", json!(event)));
            pairs.push(("target", json!(target)));
            pairs.push(("action", json!(action)));
            pairs.push(("param", json!(param)));
        }
    }
    object(pairs)
}

/// `file:line` as the pair it sorts by.
fn position(at: &str) -> (&str, usize) {
    match at.rsplit_once(':') {
        Some((file, line)) => (file, line.parse().unwrap_or(0)),
        None => (at, 0),
    }
}

/// Adds `covers` to the entry's, once each.
fn cover(entry: &mut Entry, covers: Vec<String>) {
    for c in covers {
        if !entry.covers.contains(&c) {
            entry.covers.push(c);
        }
    }
}

/// The entries collected while emitting, one per construct: a construct
/// met twice (a style pushed onto two text runs, a line drawn twice by a
/// rewrite) keeps its strongest verdict, `impossible` over `approximated`
/// over `transformed` over `supported`: it is only as supported as its
/// weakest use.
#[derive(Debug, Default)]
pub(crate) struct Ledger {
    entries: Vec<Entry>,
    index: HashMap<(String, String), usize>,
}

impl Ledger {
    pub(crate) fn add(&mut self, src: &Src, node: Option<&str>, verdict: Verdict) {
        self.add_covering(src, node, verdict, Vec::new());
    }

    pub(crate) fn add_covering(
        &mut self,
        src: &Src,
        node: Option<&str>,
        verdict: Verdict,
        covers: Vec<String>,
    ) {
        let key = (src.at.clone(), src.what.clone());
        let Some(&at) = self.index.get(&key) else {
            self.index.insert(key, self.entries.len());
            self.entries.push(Entry {
                at: src.at.clone(),
                construct: src.what.clone(),
                node: node.map(str::to_string),
                verdict,
                covers: Vec::new(),
            });
            if let Some(entry) = self.entries.last_mut() {
                cover(entry, covers);
            }
            return;
        };
        let Some(entry) = self.entries.get_mut(at) else {
            return;
        };
        match verdict.category().cmp(&entry.category()) {
            std::cmp::Ordering::Greater => entry.verdict = verdict,
            std::cmp::Ordering::Equal => entry.verdict = entry.verdict.joined(&verdict),
            std::cmp::Ordering::Less => {}
        }
        if entry.node.is_none() {
            entry.node = node.map(str::to_string);
        }
        cover(entry, covers);
    }

    /// The entries in source order.
    pub(crate) fn finish(mut self) -> Vec<Entry> {
        self.entries.sort_by(|a, b| {
            position(&a.at)
                .cmp(&position(&b.at))
                .then_with(|| a.construct.cmp(&b.construct))
        });
        self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(at: &str, what: &str) -> Src {
        Src::new(at, what)
    }

    #[test]
    fn a_construct_met_twice_keeps_its_weakest_use() {
        let mut ledger = Ledger::default();
        let size = src("a.sass:9", ".a { font-size: 12px }");
        let ir = |t: &str| t.to_string();
        ledger.add(
            &size,
            Some("s.root.a"),
            Verdict::Transformed {
                ir: ir("typography caption_r"),
                rule: ir("12 px is caption_r"),
            },
        );
        ledger.add(
            &size,
            Some("s.root.b"),
            Verdict::Approximated {
                ir: ir("typography caption_r"),
                difference: ir("line height 16 is 15.96"),
            },
        );
        ledger.add(&size, None, Verdict::Supported { ir: ir("anything") });
        ledger.add(
            &src("a.sass:10", "x"),
            None,
            Verdict::Impossible { reason: ir("r") },
        );
        ledger.add(
            &src("a.sass:10", "x"),
            None,
            Verdict::Impossible { reason: ir("q") },
        );
        let entries = ledger.finish();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].category(), Category::Approximated);
        assert_eq!(entries[0].node.as_deref(), Some("s.root.a"));
        assert_eq!(
            entries[1].verdict,
            Verdict::Impossible { reason: ir("r; q") }
        );
    }

    #[test]
    fn entries_sort_by_file_then_line_as_a_number() {
        let mut ledger = Ledger::default();
        for at in ["b.rs:3", "a.rs:10", "a.rs:9", "a.rs:10"] {
            ledger.add(
                &src(at, at),
                None,
                Verdict::Impossible { reason: "r".into() },
            );
        }
        let at: Vec<String> = ledger.finish().into_iter().map(|e| e.at).collect();
        assert_eq!(at, ["a.rs:9", "a.rs:10", "b.rs:3"]);
    }

    #[test]
    fn the_json_form_keeps_its_names_and_its_key_order() {
        let report = Report {
            source: "x.tsx".into(),
            surface: "x".into(),
            entries: vec![Entry {
                at: "x.tsx:3".into(),
                construct: "<div>".into(),
                node: Some("x.root".into()),
                verdict: Verdict::Approximated {
                    ir: "padding lg".into(),
                    difference: "bottom 32 is 16".into(),
                },
                covers: vec!["x.sass:4".into()],
            }],
            bindings: vec![Binding {
                at: "x.tsx:5".into(),
                node: "x.root.button".into(),
                need: Need::Action {
                    event: "activate".into(),
                    target: "x.props".into(),
                    action: "open".into(),
                    param: None,
                },
                why: "no handler".into(),
            }],
        };
        let text = report.to_json_string();
        let keys: Vec<&str> = [
            "\"bindings\"",
            "\"counts\"",
            "\"entries\"",
            "\"plev_report\"",
            "\"source\"",
            "\"surface\"",
        ]
        .to_vec();
        let mut last = 0;
        for key in keys {
            let at = text.find(key).unwrap_or_else(|| panic!("{key} missing"));
            assert!(at >= last, "{key} out of order");
            last = at;
        }
        let json = report.to_json();
        assert_eq!(json["counts"]["approximated"], 1);
        assert_eq!(json["counts"]["impossible"], 0);
        assert_eq!(json["entries"][0]["difference"], "bottom 32 is 16");
        assert_eq!(json["entries"][0]["covers"][0], "x.sass:4");
        assert!(json["entries"][0].get("reason").is_none());
        assert_eq!(json["bindings"][0]["binds"], "action");
        assert_eq!(json["bindings"][0]["param"], Value::Null);
        assert!(text.ends_with("}\n"));
    }
}
