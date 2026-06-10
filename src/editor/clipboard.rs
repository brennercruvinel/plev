//! Clipboard abstraction: the editor talks to a trait so tests (and wasm)
//! never touch the OS clipboard.

/// Minimal text clipboard interface used by the editor.
pub trait ClipboardProvider {
    /// Current clipboard text, if any.
    fn get_text(&mut self) -> Option<String>;
    /// Replace the clipboard contents.
    fn set_text(&mut self, text: &str);
}

/// In-memory clipboard: used in tests and as the wasm fallback.
#[derive(Debug, Default)]
pub struct LocalClipboard {
    content: Option<String>,
}

impl LocalClipboard {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ClipboardProvider for LocalClipboard {
    fn get_text(&mut self) -> Option<String> {
        self.content.clone()
    }

    fn set_text(&mut self, text: &str) {
        self.content = Some(text.to_string());
    }
}

/// OS clipboard via arboard. Construction can fail in headless environments;
/// the inner handle is then `None` and operations become no-ops.
#[cfg(not(target_arch = "wasm32"))]
pub struct SystemClipboard {
    inner: Option<arboard::Clipboard>,
}

#[cfg(not(target_arch = "wasm32"))]
impl SystemClipboard {
    pub fn new() -> Self {
        let inner = arboard::Clipboard::new()
            .map_err(|e| log::warn!("system clipboard unavailable: {e}"))
            .ok();
        Self { inner }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for SystemClipboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ClipboardProvider for SystemClipboard {
    fn get_text(&mut self) -> Option<String> {
        self.inner.as_mut()?.get_text().ok()
    }

    fn set_text(&mut self, text: &str) {
        if let Some(cb) = self.inner.as_mut()
            && let Err(e) = cb.set_text(text.to_string())
        {
            log::warn!("clipboard write failed: {e}");
        }
    }
}

/// The default clipboard for the current platform.
pub(crate) fn default_clipboard() -> Box<dyn ClipboardProvider> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::new(SystemClipboard::new())
    }
    #[cfg(target_arch = "wasm32")]
    {
        Box::new(LocalClipboard::new())
    }
}
