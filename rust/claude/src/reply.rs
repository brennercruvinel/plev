//! what the child answered: the reply of `claude -p --output-format json`,
//! or why there is none, in a short text a page can show.

use std::process::ExitStatus;

use offers::ModelError;
use serde_json::Value;

/// the longest a piece of the child's output is quoted in an error, in
/// characters.
const QUOTE_CHARS: usize = 240;

/// what one call of the child came to, read from its output.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Answer {
    /// the reply, or why there is none.
    pub reply: Result<String, ModelError>,
    /// what the call cost, as the child counts it (`total_cost_usd`).
    pub cost_usd: Option<f64>,
    /// how many turns the child took (`num_turns`): one without tools.
    pub turns: Option<u64>,
}

/// `text` on one line, at most `QUOTE_CHARS` characters.
pub(crate) fn quote(text: &str) -> String {
    let line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out: String = line.chars().take(QUOTE_CHARS).collect();
    if line.chars().count() > QUOTE_CHARS {
        out.push_str("...");
    }
    out
}

/// the text of an error the child reported in its result: its `errors`,
/// else its `result`, else nothing.
fn detail(result: &Value) -> String {
    let errors: Vec<&str> = result
        .get("errors")
        .and_then(Value::as_array)
        .map(|all| all.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    if !errors.is_empty() {
        return quote(&errors.join("; "));
    }
    result
        .get("result")
        .and_then(Value::as_str)
        .map(quote)
        .unwrap_or_default()
}

/// reads what `program` wrote: the json result object on `stdout` (an
/// answer when `is_error` is false and the child exited with success),
/// else an error that says what the child said, on stdout or stderr.
pub(crate) fn read(program: &str, status: ExitStatus, stdout: &str, stderr: &str) -> Answer {
    let result = serde_json::from_str::<Value>(stdout.trim())
        .ok()
        .filter(|v| v.get("type").and_then(Value::as_str) == Some("result"));
    let Some(result) = result else {
        let said = if stderr.trim().is_empty() {
            stdout
        } else {
            stderr
        };
        let why = if status.success() {
            format!("{program} answered no json result: {}", quote(said))
        } else {
            format!("{program} failed ({status}): {}", quote(said))
        };
        return Answer {
            reply: Err(ModelError(why)),
            cost_usd: None,
            turns: None,
        };
    };
    let cost_usd = result.get("total_cost_usd").and_then(Value::as_f64);
    let turns = result.get("num_turns").and_then(Value::as_u64);
    let failed = result.get("is_error").and_then(Value::as_bool) != Some(false);
    let text = result.get("result").and_then(Value::as_str);
    let reply = match text {
        Some(text) if !failed && status.success() => Ok(text.to_owned()),
        _ => {
            let kind = result
                .get("subtype")
                .and_then(Value::as_str)
                .unwrap_or("error");
            Err(ModelError(format!(
                "{program} failed ({status}, {kind}): {}",
                detail(&result)
            )))
        }
    };
    Answer {
        reply,
        cost_usd,
        turns,
    }
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use std::os::unix::process::ExitStatusExt;

    use serde_json::json;

    use super::*;

    fn exited(code: i32) -> ExitStatus {
        ExitStatus::from_raw(code << 8)
    }

    #[test]
    fn a_result_is_the_reply_with_its_cost_and_turns() {
        let out = json!({
            "type": "result", "subtype": "success", "is_error": false,
            "num_turns": 1, "total_cost_usd": 0.054, "result": "{\"plev_ir\": 1}"
        });
        let a = read("claude", exited(0), &out.to_string(), "");
        assert_eq!(a.reply, Ok("{\"plev_ir\": 1}".to_owned()));
        assert_eq!(a.cost_usd, Some(0.054));
        assert_eq!(a.turns, Some(1));
    }

    /// the budget the probe exceeded: exit 1, nothing on stderr, and the
    /// reason in the result on stdout.
    #[test]
    fn an_error_result_is_the_error_with_its_kind_and_reason() {
        let out = json!({
            "type": "result", "subtype": "error_max_budget_usd", "is_error": true,
            "num_turns": 1, "total_cost_usd": 0.00133,
            "errors": ["Reached maximum budget ($0.0001)"]
        });
        let a = read("claude", exited(1), &out.to_string(), "");
        let e = a.reply.unwrap_err().0;
        assert!(
            e.starts_with("claude failed (exit status: 1, error_max_budget_usd)"),
            "{e}"
        );
        assert!(e.ends_with("Reached maximum budget ($0.0001)"), "{e}");
        assert_eq!(a.cost_usd, Some(0.00133));
    }

    /// a result that says it succeeded from a child that exited with a
    /// failure is not trusted, and a result without its text is no reply.
    #[test]
    fn a_result_is_a_reply_only_when_everything_says_so() {
        let ok = json!({"type": "result", "is_error": false, "result": "{}"}).to_string();
        assert!(read("claude", exited(2), &ok, "").reply.is_err());
        let empty = json!({"type": "result", "is_error": false}).to_string();
        assert!(read("claude", exited(0), &empty, "").reply.is_err());
        let unsure = json!({"type": "result", "result": "{}"}).to_string();
        assert!(read("claude", exited(0), &unsure, "").reply.is_err());
    }

    #[test]
    fn output_that_is_not_a_result_is_quoted_short() {
        let a = read(
            "claude",
            exited(1),
            "",
            "Not logged in\nPlease run /login\n",
        );
        assert_eq!(
            a.reply.unwrap_err().0,
            "claude failed (exit status: 1): Not logged in Please run /login"
        );
        let long = "x".repeat(1000);
        let e = read("claude", exited(0), &long, "").reply.unwrap_err().0;
        assert!(e.starts_with("claude answered no json result: xxx"), "{e}");
        assert_eq!(
            e.chars().count(),
            "claude answered no json result: ".len() + 243
        );
        assert_eq!(a.cost_usd, None);
    }
}
