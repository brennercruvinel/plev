use winit::event::Ime;
use winit::window::Window;

/// IME (Input Method Editor) state for virtual keyboard input.
#[derive(Clone, Debug, Default)]
pub struct ImeState {
    pub enabled: bool,
    pub preedit_text: String,
    pub preedit_cursor: Option<(usize, usize)>,
    /// Text committed this frame. Cleared each frame by `begin_frame()`.
    pub committed_text: String,
    /// Estimated keyboard height in physical pixels.
    pub keyboard_height: f32,
}

impl ImeState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle a winit IME event. Returns true if consumed.
    pub fn handle_event(&mut self, ime: &Ime, window_height: f32) -> bool {
        match ime {
            Ime::Enabled => {
                self.enabled = true;
                self.keyboard_height = Self::estimate_keyboard_height(window_height);
                log::info!(
                    "IME enabled, estimated keyboard height: {}",
                    self.keyboard_height
                );
                true
            }
            Ime::Preedit(text, cursor) => {
                self.preedit_text = text.clone();
                self.preedit_cursor = *cursor;
                true
            }
            Ime::Commit(text) => {
                self.committed_text.push_str(text);
                self.preedit_text.clear();
                self.preedit_cursor = None;
                true
            }
            Ime::Disabled => {
                self.enabled = false;
                self.preedit_text.clear();
                self.preedit_cursor = None;
                self.keyboard_height = 0.0;
                log::info!("IME disabled");
                true
            }
        }
    }

    /// Clear per-frame transient state.
    pub fn begin_frame(&mut self) {
        self.committed_text.clear();
    }

    /// Request the soft keyboard to appear.
    pub fn request_keyboard(window: &Window, x: f64, y: f64, width: f64, height: f64) {
        window.set_ime_allowed(true);
        window.set_ime_cursor_area(
            winit::dpi::LogicalPosition::new(x, y),
            winit::dpi::LogicalSize::new(width, height),
        );
    }

    /// Dismiss the soft keyboard.
    pub fn dismiss_keyboard(window: &Window) {
        window.set_ime_allowed(false);
    }

    pub fn keyboard_visible(&self) -> bool {
        self.enabled
    }

    fn estimate_keyboard_height(window_height: f32) -> f32 {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            window_height * 0.4
        }
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let _ = window_height;
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state() {
        let ime = ImeState::new();
        assert!(!ime.enabled);
        assert!(ime.preedit_text.is_empty());
        assert!(ime.committed_text.is_empty());
        assert_eq!(ime.keyboard_height, 0.0);
    }

    #[test]
    fn handle_enabled() {
        let mut ime = ImeState::new();
        assert!(ime.handle_event(&Ime::Enabled, 1000.0));
        assert!(ime.enabled);
    }

    #[test]
    fn handle_preedit() {
        let mut ime = ImeState::new();
        ime.handle_event(&Ime::Preedit("abc".to_string(), Some((0, 3))), 1000.0);
        assert_eq!(ime.preedit_text, "abc");
        assert_eq!(ime.preedit_cursor, Some((0, 3)));
    }

    #[test]
    fn handle_commit() {
        let mut ime = ImeState::new();
        ime.handle_event(&Ime::Commit("hello".to_string()), 1000.0);
        assert_eq!(ime.committed_text, "hello");
        assert!(ime.preedit_text.is_empty());
    }

    #[test]
    fn handle_multiple_commits() {
        let mut ime = ImeState::new();
        ime.handle_event(&Ime::Commit("a".to_string()), 1000.0);
        ime.handle_event(&Ime::Commit("b".to_string()), 1000.0);
        assert_eq!(ime.committed_text, "ab");
    }

    #[test]
    fn handle_disabled() {
        let mut ime = ImeState::new();
        ime.handle_event(&Ime::Enabled, 1000.0);
        ime.handle_event(&Ime::Disabled, 1000.0);
        assert!(!ime.enabled);
        assert_eq!(ime.keyboard_height, 0.0);
    }

    #[test]
    fn begin_frame_clears_committed() {
        let mut ime = ImeState::new();
        ime.handle_event(&Ime::Commit("text".to_string()), 1000.0);
        ime.begin_frame();
        assert!(ime.committed_text.is_empty());
    }
}
