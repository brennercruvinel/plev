//! the marks: what a developer asserts about a rerun, in a small text file
//! beside the excerpt it reruns. only what was marked is checked, never the
//! whole state: a mark says which value, at which point, and what it must
//! be.
//!
//! one mark per line; blank lines and lines starting with `#` are left
//! out:
//!
//! ```text
//! # the stale submit and the recovery
//! at 0             changes:staged = {"README.md", "src/login.rs"}
//! after request 7  stale = {"changes:staged"}
//! after request 7  changes:staged = {"src/login.rs", "tests/login_test.rs"}
//! after request 9  outcome.files = ["src/login.rs", "src/session.rs", "tests/login_test.rs"]
//! at +110.700s     commit.form:open = false
//! ```
//!
//! - the anchor. `after request <id>`, or `after request <id> origin <n>`
//!   when the excerpt holds that id in several dispatchers: once that
//!   request is over in the recording. `at <ms>` or `at +<s>s`: the time
//!   from the start of the excerpt, as `plev-inspect` prints it and takes
//!   it (`--at`), once every act recorded up to then was rerun. either way
//!   the value is read right before the next act the recording holds, or
//!   at its end (see `harness`).
//! - what. a state ref of the app's projection, `<id>:<key>`;
//!   `outcome.<key>`, a value the anchored request completed with; or
//!   `stale`, the reads the anchored request's stale refusal names, as a
//!   set. the last two go with `after request`.
//! - the value. `true`, `false`, an integer, a number, a "text" (a json
//!   string), a list `["a", "b"]`, a set `{"a", "b"}`, or `absent` (the
//!   projection holds no such value, the outcome no such key, the request
//!   no stale refusal).
//!
//! the first ` = ` on the line splits what from the value.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use sem::{Millis, RequestId, StateRef, Value};

/// where a mark is read.
#[derive(Clone, Debug, PartialEq)]
pub enum Anchor {
    /// once the request is over in the recording.
    After { id: RequestId, origin: Option<u16> },
    /// once every act recorded up to this many ms from the start of the
    /// excerpt was rerun.
    At(Millis),
}

/// which value a mark reads.
#[derive(Clone, Debug, PartialEq)]
pub enum What {
    /// a value of the app's projection.
    State(StateRef),
    /// a value the anchored request completed with.
    Outcome(String),
    /// the reads the anchored request's stale refusal names.
    Stale,
}

/// one line of the marks file.
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    /// its line in the file, from 1.
    pub line: usize,
    /// the line as written, trimmed.
    pub text: String,
    pub anchor: Anchor,
    pub what: What,
    /// the value it must be; `None` for `absent`.
    pub value: Option<Value>,
}

/// reads a marks file. an error names the line and what is wrong with it.
pub fn parse(text: &str) -> Result<Vec<Mark>, String> {
    let mut out = Vec::new();
    for (at, raw) in text.lines().enumerate() {
        let line = at + 1;
        let body = raw.trim();
        if body.is_empty() || body.starts_with('#') {
            continue;
        }
        out.push(mark(line, body).map_err(|e| format!("line {line}: {e}: {body}"))?);
    }
    Ok(out)
}

fn mark(line: usize, body: &str) -> Result<Mark, String> {
    let (anchor, rest) = anchor(body)?;
    let (what, value) = rest
        .split_once(" = ")
        .ok_or("a mark is `<anchor> <what> = <value>`")?;
    let what = match what.trim() {
        "stale" => What::Stale,
        text => match text.strip_prefix("outcome.") {
            Some(key) if !key.is_empty() => What::Outcome(key.to_owned()),
            Some(_) => return Err("outcome.<key> names a key".to_owned()),
            None => What::State(
                StateRef::parse(text).map_err(|e| format!("{text:?} is not a state ref: {e}"))?,
            ),
        },
    };
    if matches!(anchor, Anchor::At(_)) && !matches!(what, What::State(_)) {
        return Err("outcome and stale go with `after request <id>`".to_owned());
    }
    Ok(Mark {
        line,
        text: body.to_owned(),
        anchor,
        what,
        value: value_of(value.trim())?,
    })
}

/// the anchor at the start of `body`, and what follows it.
fn anchor(body: &str) -> Result<(Anchor, &str), String> {
    let mut words = body.split_whitespace();
    let first = words.next().unwrap_or_default();
    let (anchor, taken) = match first {
        "after" => {
            if words.next() != Some("request") {
                return Err("`after` goes with `request <id>`".to_owned());
            }
            let id = words.next().ok_or("`after request` needs an id")?;
            let id = RequestId::parse(id).map_err(|e| e.to_string())?;
            if words.next() == Some("origin") {
                let n = words.next().ok_or("`origin` needs a number")?;
                let origin = n
                    .parse()
                    .map_err(|_| format!("an origin is a number, got {n:?}"))?;
                (
                    Anchor::After {
                        id,
                        origin: Some(origin),
                    },
                    5,
                )
            } else {
                (Anchor::After { id, origin: None }, 3)
            }
        }
        "at" => {
            let at = words.next().ok_or("`at` needs a time")?;
            (Anchor::At(millis(at)?), 2)
        }
        _ => return Err("a mark starts with `after request <id>` or `at <ms>`".to_owned()),
    };
    Ok((anchor, rest_after(body, taken)))
}

/// `body` after its first `n` words.
fn rest_after(body: &str, n: usize) -> &str {
    let mut rest = body;
    for _ in 0..n {
        rest = rest.trim_start();
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        rest = &rest[end..];
    }
    rest.trim_start()
}

/// `1500` (ms) or `+1.500s` (seconds, at most three decimals).
fn millis(text: &str) -> Result<Millis, String> {
    let bad = || format!("a time is <ms> or +<s>s, got {text:?}");
    let Some(secs) = text.strip_prefix('+') else {
        return text.parse().map_err(|_| bad());
    };
    let secs = secs.strip_suffix('s').ok_or_else(bad)?;
    let (whole, part) = secs.split_once('.').unwrap_or((secs, ""));
    if part.len() > 3 || !part.bytes().all(|b| b.is_ascii_digit()) {
        return Err(bad());
    }
    let whole: Millis = whole.parse().map_err(|_| bad())?;
    let part: Millis = format!("{part:0<3}").parse().map_err(|_| bad())?;
    whole
        .checked_mul(1_000)
        .and_then(|ms| ms.checked_add(part))
        .ok_or_else(bad)
}

/// a value in the marks' syntax; `None` for `absent`.
fn value_of(text: &str) -> Result<Option<Value>, String> {
    if text == "absent" {
        return Ok(None);
    }
    if let Some(inner) = text.strip_prefix('{').and_then(|t| t.strip_suffix('}')) {
        let items: Vec<String> = serde_json::from_str(&format!("[{inner}]"))
            .map_err(|_| format!("a set is {{\"a\", \"b\"}}, got {text}"))?;
        return Ok(Some(Value::Set(items.into_iter().collect::<BTreeSet<_>>())));
    }
    let json: serde_json::Value =
        serde_json::from_str(text).map_err(|_| format!("{text} is not a value"))?;
    let value = match json {
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => match (n.as_i64(), n.as_f64()) {
            (Some(i), _) => Value::Int(i),
            (None, Some(f)) => Value::Num(f),
            (None, None) => return Err(format!("{text} does not fit a number")),
        },
        serde_json::Value::String(s) => Value::Text(s),
        serde_json::Value::Array(items) => Value::List(
            items
                .into_iter()
                .map(|item| match item {
                    serde_json::Value::String(s) => Ok(s),
                    other => Err(format!("a list holds texts, got {other}")),
                })
                .collect::<Result<_, _>>()?,
        ),
        serde_json::Value::Null | serde_json::Value::Object(_) => {
            return Err(format!("{text} is not a value"));
        }
    };
    Ok(Some(value))
}

/// a value written back in the marks' syntax.
pub fn render(value: Option<&Value>) -> String {
    let quoted = |s: &str| serde_json::to_string(s).unwrap_or_else(|_| format!("{s:?}"));
    let items = |it: &mut dyn Iterator<Item = &String>| {
        let mut out = String::new();
        for (n, item) in it.enumerate() {
            if n > 0 {
                out.push_str(", ");
            }
            out.push_str(&quoted(item));
        }
        out
    };
    let mut out = String::new();
    let _ = match value {
        None => write!(out, "absent"),
        Some(Value::Bool(b)) => write!(out, "{b}"),
        Some(Value::Int(i)) => write!(out, "{i}"),
        Some(Value::Num(f)) => write!(out, "{f:?}"),
        Some(Value::Text(t)) => write!(out, "{}", quoted(t)),
        Some(Value::List(l)) => write!(out, "[{}]", items(&mut l.iter())),
        Some(Value::Set(s)) => write!(out, "{{{}}}", items(&mut s.iter())),
    };
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(line: &str) -> Mark {
        match parse(line) {
            Ok(mut marks) if marks.len() == 1 => marks.remove(0),
            other => panic!("{line}: {other:?}"),
        }
    }

    fn state(text: &str) -> What {
        What::State(StateRef::parse(text).unwrap_or_else(|e| panic!("{text}: {e}")))
    }

    fn rid(n: &str) -> RequestId {
        RequestId::parse(n).unwrap_or_else(|e| panic!("{n}: {e}"))
    }

    #[test]
    fn every_form_reads_and_writes_back() {
        let set = one(r#"after request 7  changes:staged = {"a.rs", "b.rs"}"#);
        assert_eq!(
            set.anchor,
            Anchor::After {
                id: rid("7"),
                origin: None
            }
        );
        assert_eq!(set.what, state("changes:staged"));
        assert_eq!(render(set.value.as_ref()), r#"{"a.rs", "b.rs"}"#);

        let origin = one(r#"after request 2 origin 1  commit.message:text = "a = b""#);
        assert_eq!(
            origin.anchor,
            Anchor::After {
                id: rid("2"),
                origin: Some(1)
            }
        );
        assert_eq!(origin.value, Some(Value::Text("a = b".into())));

        let list = one(r#"after request 9 outcome.files = ["b", "a"]"#);
        assert_eq!(list.what, What::Outcome("files".into()));
        assert_eq!(render(list.value.as_ref()), r#"["b", "a"]"#);

        assert_eq!(one("at 1500 x:open = true").anchor, Anchor::At(1_500));
        assert_eq!(one("at +1.5s x:open = false").anchor, Anchor::At(1_500));
        assert_eq!(one("at +70.679s x:n = 3").value, Some(Value::Int(3)));
        assert_eq!(one("at 0 x:n = 0.5").value, Some(Value::Num(0.5)));
        assert_eq!(
            one("at 0 x:set = {}").value,
            Some(Value::Set(BTreeSet::new()))
        );
        assert_eq!(one("at 0 x:gone = absent").value, None);
        assert_eq!(
            one(r#"after request 3 stale = {"changes:staged"}"#).what,
            What::Stale
        );

        let file = "# a comment\n\n  at 0 x:n = 1\n# another\nafter request 1 x:n = 2\n";
        let marks = parse(file).unwrap_or_default();
        assert_eq!(marks.iter().map(|m| m.line).collect::<Vec<_>>(), vec![3, 5]);
    }

    #[test]
    fn a_bad_line_says_which_and_why() {
        for (text, needle) in [
            ("before request 3 x:n = 1", "starts with"),
            ("after 3 x:n = 1", "`after` goes with"),
            ("after request x:n = 1", "decimal"),
            ("after request 07 x:n = 1", "decimal"),
            ("after request 3 origin x:n = 1", "origin is a number"),
            ("at soon x:n = 1", "a time is"),
            ("at +1.2345s x:n = 1", "a time is"),
            ("at 5 x:n 1", "is `<anchor> <what> = <value>`"),
            ("at 5 Bad:n = 1", "not a state ref"),
            ("at 5 outcome.files = []", "go with `after request"),
            ("at 5 stale = {}", "go with `after request"),
            ("after request 1 outcome. = 1", "names a key"),
            ("at 5 x:n = maybe", "not a value"),
            ("at 5 x:n = null", "not a value"),
            ("at 5 x:n = {\"a\": 1}", "a set is"),
            ("at 5 x:n = [1, 2]", "a list holds texts"),
        ] {
            let file = format!("# first\n{text}\n");
            match parse(&file) {
                Err(e) => {
                    assert!(e.starts_with("line 2: "), "{text}: {e}");
                    assert!(e.contains(needle), "{text}: {e:?} lacks {needle:?}");
                }
                Ok(marks) => panic!("{text}: {marks:?}"),
            }
        }
    }
}
