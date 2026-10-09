//! the streaming interface: the prompt goes in as token ids, the new tokens
//! come out one by one through a tokio channel, the caller can cancel, and
//! the end is always announced with a [`Event::Done`] or an [`Event::Failed`].
//!
//! the decode loop runs on its own thread, so a consumer can await the
//! channel inside a tokio runtime ([`Stream::next`]) or block on it from
//! plain code ([`Stream::blocking_next`]).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;

use crate::{Info, LlmError};

/// A causal language model that decodes greedily.
pub trait Model: Send {
    /// Forgets the previous sequence (the kv cache).
    fn reset(&mut self) -> Result<(), LlmError>;
    /// Feeds `ids` at positions `pos..pos + ids.len()` and returns the id
    /// with the highest logit after the last one.
    fn next(&mut self, ids: &[u32], pos: usize) -> Result<u32, LlmError>;
    /// The longest sequence the model accepts, prompt plus new tokens.
    fn context(&self) -> usize;
    /// The library, device and dtype, for reports.
    fn info(&self) -> Info;
}

impl<M: Model + ?Sized> Model for Box<M> {
    fn reset(&mut self) -> Result<(), LlmError> {
        (**self).reset()
    }
    fn next(&mut self, ids: &[u32], pos: usize) -> Result<u32, LlmError> {
        (**self).next(ids, pos)
    }
    fn context(&self) -> usize {
        (**self).context()
    }
    fn info(&self) -> Info {
        (**self).info()
    }
}

/// Turns ids into text as they arrive.
pub trait Detok: Send {
    /// Takes the next id and returns the text it completes; empty while a
    /// character is still incomplete.
    fn push(&mut self, id: u32) -> String;
}

/// One generation.
#[derive(Clone, Debug)]
pub struct Request {
    /// The prompt, already templated and tokenized.
    pub prompt: Vec<u32>,
    /// How many new tokens at most.
    pub max_new: usize,
    /// The ids that end the answer.
    pub eos: Vec<u32>,
}

/// A new token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    /// The token id.
    pub id: u32,
    /// The text this token completes (may be empty mid character).
    pub text: String,
    /// 0 for the first new token.
    pub index: usize,
}

/// Why a generation ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    /// The model produced an end-of-sequence id.
    Eos,
    /// `max_new` tokens were produced.
    Length,
    /// The caller cancelled or dropped the stream.
    Cancelled,
}

/// What the channel carries.
#[derive(Debug)]
pub enum Event {
    /// A new token.
    Token(Token),
    /// The generation ended; nothing comes after this.
    Done(Stop),
    /// The generation failed; nothing comes after this.
    Failed(LlmError),
}

/// A handle that stops a generation; cheap to clone and to send.
#[derive(Clone, Debug, Default)]
pub struct Cancel(Arc<AtomicBool>);

impl Cancel {
    /// Asks the decode loop to stop before the next token.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    /// Whether [`Cancel::cancel`] was called.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

/// The receiving end of one generation.
pub struct Stream {
    rx: mpsc::UnboundedReceiver<Event>,
    cancel: Cancel,
}

impl Stream {
    /// The next event, awaited; `None` after the final event.
    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    /// The next event, blocking the thread; `None` after the final event.
    /// Panics if called inside an async runtime (tokio's rule).
    pub fn blocking_next(&mut self) -> Option<Event> {
        self.rx.blocking_recv()
    }

    /// Stops the generation before its next token.
    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    /// A handle that can cancel from elsewhere.
    pub fn canceller(&self) -> Cancel {
        self.cancel.clone()
    }
}

/// Starts a generation on its own thread and returns its stream. The model
/// is locked for the whole generation; a second stream on the same model
/// waits for the first to end.
pub fn spawn<M>(model: Arc<Mutex<M>>, req: Request, detok: Box<dyn Detok>) -> Stream
where
    M: Model + 'static,
{
    let (tx, rx) = mpsc::unbounded_channel();
    let cancel = Cancel::default();
    let flag = cancel.clone();
    std::thread::spawn(move || {
        let last = match decode(&model, req, detok, &flag, &tx) {
            Ok(stop) => Event::Done(stop),
            Err(err) => Event::Failed(err),
        };
        // the receiver may be gone already; then nobody is waiting for the end.
        let _ = tx.send(last);
    });
    Stream { rx, cancel }
}

fn decode<M: Model>(
    model: &Mutex<M>,
    req: Request,
    mut detok: Box<dyn Detok>,
    cancel: &Cancel,
    tx: &mpsc::UnboundedSender<Event>,
) -> Result<Stop, LlmError> {
    let mut model = model.lock().map_err(|_| LlmError::Poisoned)?;
    if req.prompt.is_empty() {
        return Err(LlmError::EmptyPrompt);
    }
    let len = req.prompt.len() + req.max_new;
    if len > model.context() {
        return Err(LlmError::Context {
            len,
            max: model.context(),
        });
    }
    model.reset()?;
    let mut input = req.prompt;
    let mut pos = 0;
    for index in 0..req.max_new {
        if cancel.is_cancelled() || tx.is_closed() {
            return Ok(Stop::Cancelled);
        }
        let id = model.next(&input, pos)?;
        if cancel.is_cancelled() {
            return Ok(Stop::Cancelled);
        }
        if req.eos.contains(&id) {
            return Ok(Stop::Eos);
        }
        pos += input.len();
        let text = detok.push(id);
        if tx.send(Event::Token(Token { id, text, index })).is_err() {
            return Ok(Stop::Cancelled);
        }
        input.clear();
        input.push(id);
    }
    Ok(Stop::Length)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc as std_mpsc;

    /// A fake model: answers `first + pos`, so the ids say where they came
    /// from; optionally waits for a go per step, and can fail on a step.
    struct Fake {
        first: u32,
        resets: usize,
        calls: Vec<(Vec<u32>, usize)>,
        gate: Option<std_mpsc::Receiver<()>>,
        fail_at: Option<usize>,
        context: usize,
    }

    impl Fake {
        fn new(first: u32) -> Self {
            Self {
                first,
                resets: 0,
                calls: Vec::new(),
                gate: None,
                fail_at: None,
                context: 64,
            }
        }
    }

    impl Model for Fake {
        fn reset(&mut self) -> Result<(), LlmError> {
            self.resets += 1;
            Ok(())
        }
        fn next(&mut self, ids: &[u32], pos: usize) -> Result<u32, LlmError> {
            if let Some(gate) = &self.gate {
                gate.recv().map_err(|e| LlmError::Backend {
                    backend: "fake",
                    reason: e.to_string(),
                })?;
            }
            if self.fail_at == Some(self.calls.len()) {
                return Err(LlmError::Backend {
                    backend: "fake",
                    reason: "step failed".into(),
                });
            }
            self.calls.push((ids.to_vec(), pos));
            Ok(self.first + self.calls.len() as u32 - 1)
        }
        fn context(&self) -> usize {
            self.context
        }
        fn info(&self) -> Info {
            Info {
                backend: "fake",
                device: "none".into(),
                dtype: "none",
            }
        }
    }

    struct Words;

    impl Detok for Words {
        fn push(&mut self, id: u32) -> String {
            format!("t{id} ")
        }
    }

    fn req(prompt: &[u32], max_new: usize, eos: &[u32]) -> Request {
        Request {
            prompt: prompt.to_vec(),
            max_new,
            eos: eos.to_vec(),
        }
    }

    fn drain(stream: &mut Stream) -> (Vec<Token>, Option<Event>) {
        let mut tokens = Vec::new();
        while let Some(event) = stream.blocking_next() {
            match event {
                Event::Token(t) => tokens.push(t),
                last => return (tokens, Some(last)),
            }
        }
        (tokens, None)
    }

    #[test]
    fn tokens_arrive_in_order_and_the_end_is_announced() {
        let model = Arc::new(Mutex::new(Fake::new(100)));
        let mut stream = spawn(model.clone(), req(&[1, 2, 3], 4, &[]), Box::new(Words));
        let (tokens, last) = drain(&mut stream);
        let ids: Vec<u32> = tokens.iter().map(|t| t.id).collect();
        assert_eq!(ids, [100, 101, 102, 103]);
        let index: Vec<usize> = tokens.iter().map(|t| t.index).collect();
        assert_eq!(index, [0, 1, 2, 3]);
        assert_eq!(tokens[2].text, "t102 ");
        assert!(matches!(last, Some(Event::Done(Stop::Length))));
        assert!(stream.blocking_next().is_none(), "nothing after the end");
        // the prompt goes in once, then one token at a time at the right position.
        let calls = model.lock().unwrap().calls.clone();
        assert_eq!(calls[0], (vec![1, 2, 3], 0));
        assert_eq!(calls[1], (vec![100], 3));
        assert_eq!(calls[3], (vec![102], 5));
    }

    #[test]
    fn eos_ends_the_stream_without_a_token() {
        let model = Arc::new(Mutex::new(Fake::new(7)));
        let mut stream = spawn(model, req(&[1], 10, &[9]), Box::new(Words));
        let (tokens, last) = drain(&mut stream);
        let ids: Vec<u32> = tokens.iter().map(|t| t.id).collect();
        assert_eq!(ids, [7, 8]);
        assert!(matches!(last, Some(Event::Done(Stop::Eos))));
    }

    #[test]
    fn cancel_stops_before_the_next_token() {
        let (go, gate) = std_mpsc::channel();
        let mut fake = Fake::new(50);
        fake.gate = Some(gate);
        let model = Arc::new(Mutex::new(fake));
        let mut stream = spawn(model.clone(), req(&[1], 50, &[]), Box::new(Words));
        for expected in [50, 51] {
            go.send(()).unwrap();
            match stream.blocking_next() {
                Some(Event::Token(t)) => assert_eq!(t.id, expected),
                other => panic!("expected a token, got {other:?}"),
            }
        }
        stream.cancel();
        // releases the step the loop may already be waiting in; the loop may
        // also have stopped before it, so the send can fail.
        let _ = go.send(());
        let (tokens, last) = drain(&mut stream);
        assert!(tokens.is_empty(), "no token after cancel: {tokens:?}");
        assert!(matches!(last, Some(Event::Done(Stop::Cancelled))));
        assert!(model.lock().unwrap().calls.len() <= 3);
    }

    #[test]
    fn dropping_the_stream_stops_the_loop() {
        let (go, gate) = std_mpsc::channel();
        let mut fake = Fake::new(0);
        fake.gate = Some(gate);
        let model = Arc::new(Mutex::new(fake));
        let stream = spawn(model.clone(), req(&[1], 50, &[]), Box::new(Words));
        drop(stream);
        // at most one step runs; its token finds the channel closed.
        let _ = go.send(());
        drop(go);
        // the loop releases the model when it ends; the lock waits for it.
        let calls = model.lock().unwrap().calls.len();
        assert!(calls <= 1, "{calls} steps after the drop");
    }

    #[test]
    fn a_failing_step_is_reported_as_the_last_event() {
        let mut fake = Fake::new(10);
        fake.fail_at = Some(2);
        let model = Arc::new(Mutex::new(fake));
        let mut stream = spawn(model, req(&[1], 10, &[]), Box::new(Words));
        let (tokens, last) = drain(&mut stream);
        assert_eq!(tokens.len(), 2);
        assert!(matches!(
            last,
            Some(Event::Failed(LlmError::Backend { .. }))
        ));
    }

    #[test]
    fn a_prompt_past_the_context_fails_before_any_step() {
        let model = Arc::new(Mutex::new(Fake::new(0)));
        let mut stream = spawn(model.clone(), req(&[1; 60], 8, &[]), Box::new(Words));
        let (tokens, last) = drain(&mut stream);
        assert!(tokens.is_empty());
        assert!(matches!(
            last,
            Some(Event::Failed(LlmError::Context { len: 68, max: 64 }))
        ));
        let mut empty = spawn(model.clone(), req(&[], 8, &[]), Box::new(Words));
        assert!(matches!(
            drain(&mut empty).1,
            Some(Event::Failed(LlmError::EmptyPrompt))
        ));
        assert!(model.lock().unwrap().calls.is_empty());
    }

    #[test]
    fn the_model_is_reset_for_each_generation() {
        let model = Arc::new(Mutex::new(Fake::new(0)));
        for _ in 0..2 {
            let mut stream = spawn(model.clone(), req(&[1], 2, &[]), Box::new(Words));
            drain(&mut stream);
        }
        assert_eq!(model.lock().unwrap().resets, 2);
    }

    #[tokio::test]
    async fn the_stream_can_be_awaited() {
        let model = Arc::new(Mutex::new(Fake::new(30)));
        let mut stream = spawn(model, req(&[1, 2], 3, &[]), Box::new(Words));
        let mut ids = Vec::new();
        let mut done = None;
        while let Some(event) = stream.next().await {
            match event {
                Event::Token(t) => ids.push(t.id),
                Event::Done(stop) => done = Some(stop),
                Event::Failed(err) => panic!("{err}"),
            }
        }
        assert_eq!(ids, [30, 31, 32]);
        assert_eq!(done, Some(Stop::Length));
    }
}
