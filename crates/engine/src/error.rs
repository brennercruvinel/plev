use std::fmt;

/// Unified error type for the plev engine.
#[derive(Debug)]
pub enum PlevError {
    /// Window creation or platform error.
    Window(winit::error::OsError),

    /// GPU initialization or resource error.
    Gpu(String),

    /// WASM-specific: web-sys API unavailable or returned an unexpected value.
    #[cfg(target_arch = "wasm32")]
    Wasm(&'static str),

    /// File watcher error (hot-reload only).
    #[cfg(feature = "hot-reload")]
    Watcher(notify::Error),
}

impl fmt::Display for PlevError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Window(e) => write!(f, "window: {e}"),
            Self::Gpu(msg) => write!(f, "gpu: {msg}"),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(msg) => write!(f, "wasm: {msg}"),
            #[cfg(feature = "hot-reload")]
            Self::Watcher(e) => write!(f, "file watcher: {e}"),
        }
    }
}

impl std::error::Error for PlevError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Window(e) => Some(e),
            #[cfg(feature = "hot-reload")]
            Self::Watcher(e) => Some(e),
            _ => None,
        }
    }
}

impl From<winit::error::OsError> for PlevError {
    fn from(e: winit::error::OsError) -> Self {
        Self::Window(e)
    }
}

#[cfg(feature = "hot-reload")]
impl From<notify::Error> for PlevError {
    fn from(e: notify::Error) -> Self {
        Self::Watcher(e)
    }
}

/// Convenience alias for `Result<T, PlevError>`.
pub type PlevResult<T> = Result<T, PlevError>;
