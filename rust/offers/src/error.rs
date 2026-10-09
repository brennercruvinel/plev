//! why a proposal did not come: the proposer did not understand, the app
//! has nothing of the kind, a review cannot be made, the model failed or
//! kept proposing what the verifier refuses, or the verifier refused it;
//! and why a change of the
//! surface shown did not come: the intent asks for none, nothing on the
//! surface takes it, or an operation was refused.

use plevir::{IrError, TransformError};

use crate::words::Part;

/// why there is no proposal. the text says what a person or an agent can
/// do about it; a page shows it as it is.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ProposeError {
    /// no word of the intent is one the proposer reads.
    #[error(
        "the rules do not understand \"{intent}\": they read {grammar}, and words the app uses"
    )]
    NotUnderstood { intent: String, grammar: String },
    /// words of the intent that are neither the proposer's nor the app's.
    #[error("{} not a word of the app; its words are {}", quoted(words), app.join(", "))]
    NoMatch {
        /// the words, as the intent wrote them.
        words: Vec<String>,
        /// every word the app uses for what a surface can show, folded.
        app: Vec<String>,
    },
    /// the intent asks for a part the app has nothing for.
    #[error("{part} asked for, and the app declares {missing}")]
    Nothing { part: Part, missing: String },
    /// the app declares nothing the rules can show: no text input, no
    /// choice, no number, no list.
    #[error(
        "the app declares nothing these rules can show: no text input, no choice, no number, \
         no list"
    )]
    Empty,
    /// the model did not answer.
    #[error("the model failed: {0}")]
    Model(String),
    /// every attempt of the model was refused; the errors of the last one.
    #[error(
        "{attempts} proposals and none verified; the last one had {} errors, the first: {}",
        errors.len(),
        first(errors)
    )]
    Exhausted {
        attempts: usize,
        errors: Vec<IrError>,
    },
    /// the verifier refused the proposal: every error at once.
    #[error("the verifier refused the proposal ({} errors), the first: {}", errors_len(.0), first(.0))]
    Refused(Vec<IrError>),
    /// the intent asks for no change of the surface shown: it asks for a
    /// surface (a proposer answers it).
    #[error(
        "\"{intent}\" asks for no change of the surface shown: the rules change it for {changes}"
    )]
    NotAChange { intent: String, changes: String },
    /// a change asked for that nothing on the surface takes, and why.
    #[error("{0}")]
    NoChange(String),
    /// an operation of a change was refused (the first one; `at` counts
    /// from 1): the surface stays as it is.
    #[error("operation {at}, {op}, was refused: {error}")]
    Unapplied {
        at: usize,
        op: String,
        error: TransformError,
    },
    /// a review the rules cannot make (task-96), and why: nothing to
    /// review, no word that says which items, fewer items than asked for.
    #[error("{0}")]
    Review(String),
    /// every answer of the model had an operation refused, or was not
    /// operations; the reasons of the last one.
    #[error(
        "{attempts} answers and none changed the surface; the last one: {}",
        errors.first().map_or("none", String::as_str)
    )]
    Unchanged {
        attempts: usize,
        errors: Vec<String>,
    },
}

fn quoted(words: &[String]) -> String {
    let list: Vec<String> = words.iter().map(|w| format!("\"{w}\"")).collect();
    let verb = if list.len() == 1 { "is" } else { "are" };
    format!("{} {verb}", list.join(", "))
}

fn first(errors: &[IrError]) -> String {
    errors
        .first()
        .map_or_else(|| "none".to_owned(), ToString::to_string)
}

fn errors_len(errors: &[IrError]) -> usize {
    errors.len()
}
