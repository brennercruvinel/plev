//! the real model, when `PLEV_LLM_MODEL` names its directory: each
//! implementation generates 16 tokens of a fixed prompt. ignored by default
//! (the weights live outside the repository); run with
//! `cargo test --release --features <impl> -- --ignored`.

#![cfg(all(feature = "text", any(feature = "candle", feature = "burn")))]

use std::path::PathBuf;

use llm::text::{Engine, Text};
use llm::{Device, Event, Model, Stop};

const PROMPT: &str = "What is the capital of France?";
const NEW: usize = 16;

fn model_dir() -> Option<PathBuf> {
    let dir = PathBuf::from(std::env::var_os(llm::MODEL_ENV)?);
    dir.is_dir().then_some(dir)
}

fn device() -> Device {
    if cfg!(feature = "metal") {
        Device::Metal
    } else {
        Device::Cpu
    }
}

/// Runs the fixed prompt and checks the stream's shape: 16 tokens in
/// order (or fewer and an eos), then the end.
fn sixteen_tokens<M: Model + 'static>(model: M, text: Text) -> String {
    let engine = Engine::new(model, text);
    let mut stream = engine.ask(PROMPT, NEW).expect("prompt");
    let mut answer = String::new();
    let mut count = 0;
    let stop = loop {
        match stream
            .blocking_next()
            .expect("the stream ends with an event")
        {
            Event::Token(t) => {
                assert_eq!(t.index, count);
                count += 1;
                answer.push_str(&t.text);
            }
            Event::Done(stop) => break stop,
            Event::Failed(err) => panic!("{err}"),
        }
    };
    assert!(count > 0, "no token");
    assert!(stop == Stop::Length && count == NEW || stop == Stop::Eos);
    assert!(!answer.trim().is_empty());
    eprintln!("{}: {answer:?}", engine.info().expect("info").backend);
    answer
}

#[cfg(feature = "candle")]
#[test]
#[ignore = "needs the model in PLEV_LLM_MODEL"]
fn candle_generates_sixteen_tokens() {
    let Some(dir) = model_dir() else {
        eprintln!("PLEV_LLM_MODEL not set; skipped");
        return;
    };
    let model = llm::candle::CandleModel::load(&dir, device()).expect("load");
    let answer = sixteen_tokens(model, Text::from_dir(&dir).expect("tokenizer"));
    assert!(answer.contains("Paris"), "{answer:?}");
}

#[cfg(feature = "burn")]
#[test]
#[ignore = "needs the model in PLEV_LLM_MODEL"]
fn burn_generates_sixteen_tokens() {
    let Some(dir) = model_dir() else {
        eprintln!("PLEV_LLM_MODEL not set; skipped");
        return;
    };
    let model = llm::burn::load(&dir, device()).expect("load");
    let answer = sixteen_tokens(model, Text::from_dir(&dir).expect("tokenizer"));
    assert!(answer.contains("Paris"), "{answer:?}");
}
