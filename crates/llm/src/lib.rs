//! local llm inference in process.
//!
//! one streaming interface ([`stream`]): a prompt goes in, tokens come out one
//! by one through a tokio channel, the caller can cancel, and the end is
//! announced. two implementations of the [`Model`] behind it, each behind its
//! own feature: `candle` and `burn`, on the cpu or, with `metal`, on the apple
//! gpu. both decode greedily from the same token ids, so their outputs are
//! comparable token by token.
//!
//! the model files live outside the repository; the directory comes in by
//! argument or through `PLEV_LLM_MODEL`, never fixed in code.

mod error;
pub mod stream;

#[cfg(feature = "text")]
pub mod chat;
#[cfg(feature = "text")]
pub mod text;

#[cfg(feature = "candle")]
pub mod candle;

pub use error::LlmError;
pub use stream::{Cancel, Event, Model, Request, Stop, Stream, Token};

/// The environment variable that names the model directory.
pub const MODEL_ENV: &str = "PLEV_LLM_MODEL";

/// Where the weights live and run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    /// The cpu, in f32.
    Cpu,
    /// The apple gpu, in f16.
    Metal,
}

impl Device {
    /// Parses `cpu` or `metal`.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "cpu" => Some(Self::Cpu),
            "metal" => Some(Self::Metal),
            _ => None,
        }
    }
}

/// What a loaded model reports about itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Info {
    /// The library: `candle` or `burn`.
    pub backend: &'static str,
    /// The device as the library names it.
    pub device: String,
    /// The float type of the weights and activations.
    pub dtype: &'static str,
}
