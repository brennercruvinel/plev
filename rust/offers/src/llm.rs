//! the proposer that asks a language model, behind a trait: the model
//! reads the prompt (`prompt`) and answers text; the text is decoded and
//! verified; the errors go back to the model with its reply, a bounded
//! number of times; an IR the verifier refuses is never returned.
//!
//! with a surface shown, the same model is a transformer (task-70 phase
//! 2): it reads the prompt of a change (`ops`) and answers
//! `{"ops": [...]}`; the operations are applied in order over the surface,
//! each verified, and the one refused goes back with why, the same
//! bounded number of times.
//!
//! no model lives here. a provider (a local model or an API) implements
//! `Model` in phase 3b; the tests use models that answer from a script.

use plevir::{Ir, IrError, parse, verify};

use crate::adapt::{Adapt, Adaptation, Transformer, apply_all};
use crate::error::ProposeError;
use crate::operations;
use crate::prompt::{prompt, repair};
use crate::{Ask, Proposer};

/// a language model: a prompt in, the model's text out. synchronous, like
/// the rest of a proposal; a provider that takes seconds runs it off the
/// app's thread (the dispatcher already ends a request later,
/// `sem::Execution::Started`).
pub trait Model {
    fn complete(&mut self, prompt: &str) -> Result<String, ModelError>;
}

/// a model's failure to answer: no connection, a timeout, a refusal of
/// the provider. its text goes into `ProposeError::Model`.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct ModelError(pub String);

impl<M: Model + ?Sized> Model for &mut M {
    fn complete(&mut self, prompt: &str) -> Result<String, ModelError> {
        (**self).complete(prompt)
    }
}

/// the proposer that asks `M`.
#[derive(Debug)]
pub struct Llm<M> {
    model: M,
    attempts: usize,
}

impl<M: Model> Llm<M> {
    /// how many times a model is asked for one proposal: the first answer
    /// and two repairs. the verifier returns every error at once, each at
    /// its pointer, so one repair usually suffices; the third bounds the
    /// cost of a model that does not converge (each attempt is a call).
    pub const ATTEMPTS: usize = 3;

    pub fn new(model: M) -> Llm<M> {
        Llm::with_attempts(model, Llm::<M>::ATTEMPTS)
    }

    /// a proposer that asks `model` at most `attempts` times, at least
    /// once.
    pub fn with_attempts(model: M, attempts: usize) -> Llm<M> {
        Llm {
            model,
            attempts: attempts.max(1),
        }
    }

    pub fn attempts(&self) -> usize {
        self.attempts
    }

    pub fn model(&self) -> &M {
        &self.model
    }

    pub fn into_model(self) -> M {
        self.model
    }
}

/// the IR in a reply: the reply itself, or the object inside a fence or
/// around it, from the first `{` to the last `}` (models wrap JSON in
/// prose and fences).
fn read(reply: &str) -> Result<Ir, Vec<IrError>> {
    let text = reply.trim();
    let inner = match (text.find('{'), text.rfind('}')) {
        (Some(open), Some(close)) if open < close => &text[open..=close],
        _ => text,
    };
    parse(inner)
}

impl<M: Model> Proposer for Llm<M> {
    /// asks the model, and while the verifier refuses its answer, asks
    /// again with the answer and every error; `Exhausted` with the last
    /// errors after `attempts` refusals. what it returns has passed the
    /// verifier once; `propose_verified` checks it again, as for any
    /// proposer.
    fn propose(&mut self, ask: &Ask<'_>) -> Result<Ir, ProposeError> {
        let base = prompt(ask);
        let mut asked = base.clone();
        let mut last = Vec::new();
        for attempt in 1..=self.attempts {
            let reply = self
                .model
                .complete(&asked)
                .map_err(|e| ProposeError::Model(e.0))?;
            let checked = read(&reply).and_then(|ir| match verify(&ir, ask.caps, ask.policy) {
                Ok(_) => Ok(ir),
                Err(errors) => Err(errors),
            });
            match checked {
                Ok(ir) => return Ok(ir),
                Err(errors) => {
                    asked = repair(&base, attempt, &reply, &errors);
                    last = errors;
                }
            }
        }
        Err(ProposeError::Exhausted {
            attempts: self.attempts,
            errors: last,
        })
    }
}

impl<M: Model> Transformer for Llm<M> {
    /// asks the model for operations, and while one of them is refused
    /// over the surface (or the answer is no list of operations), asks
    /// again with the answer and why; `Unchanged` with the last reasons
    /// after `attempts` answers. what it returns applied over the surface
    /// once; `transform_verified` applies it again, as for any
    /// transformer.
    fn transform(&mut self, ask: &Adapt<'_>) -> Result<Adaptation, ProposeError> {
        let base = operations::prompt(ask);
        let mut asked = base.clone();
        let mut last = Vec::new();
        for attempt in 1..=self.attempts {
            let reply = self
                .model
                .complete(&asked)
                .map_err(|e| ProposeError::Model(e.0))?;
            let checked =
                operations::read(&reply).map_err(|why| vec![why]).and_then(|list| {
                    match apply_all(&list, ask) {
                        Ok(_) => Ok(list),
                        Err((at, e)) => Err(list
                            .get(at)
                            .map(|op| operations::reasons(at + 1, op, &e))
                            .unwrap_or_default()),
                    }
                });
            match checked {
                Ok(list) => return Ok(Adaptation::Ops(list)),
                Err(why) => {
                    asked = operations::repair(&base, attempt, &reply, &why);
                    last = why;
                }
            }
        }
        Err(operations::unchanged(self.attempts, last))
    }
}
