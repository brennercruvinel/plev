use std::path::{Path, PathBuf};

/// Everything that can go wrong between the model files and the last token.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    /// A model file is missing or unreadable.
    #[error("{path}: {reason}")]
    File {
        /// The file.
        path: PathBuf,
        /// What the system said.
        reason: String,
    },
    /// The model config does not describe a model this crate runs.
    #[error("model config: {0}")]
    Config(String),
    /// The tokenizer failed to load, encode or decode.
    #[error("tokenizer: {0}")]
    Tokenizer(String),
    /// The inference library failed.
    #[error("{backend}: {reason}")]
    Backend {
        /// `candle` or `burn`.
        backend: &'static str,
        /// What the library said.
        reason: String,
    },
    /// The prompt is empty.
    #[error("the prompt has no tokens")]
    EmptyPrompt,
    /// The prompt plus the new tokens do not fit the model context.
    #[error("{len} tokens do not fit the context of {max}")]
    Context {
        /// Prompt plus new tokens.
        len: usize,
        /// The model context.
        max: usize,
    },
    /// An earlier generation panicked while it held the model.
    #[error("the model was poisoned by a panic in an earlier generation")]
    Poisoned,
}

impl LlmError {
    /// A file error from an io error.
    pub fn file(path: &Path, err: impl std::fmt::Display) -> Self {
        Self::File {
            path: path.to_path_buf(),
            reason: err.to_string(),
        }
    }
}
