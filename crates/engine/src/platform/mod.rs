pub mod ime;
pub mod lifecycle;
use winit::window::Window;

/// Safe area insets in physical pixels.
///
/// On mobile, these represent areas obscured by system UI (notch, status bar,
/// home indicator, navigation bar). On desktop, all values are zero.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SafeAreaInsets {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl SafeAreaInsets {
    pub fn is_zero(&self) -> bool {
        self.top == 0.0 && self.bottom == 0.0 && self.left == 0.0 && self.right == 0.0
    }

    /// Compute safe area insets from the window.
    ///
    /// - **Android**: Uses `content_rect()` from `WindowExtAndroid` to derive insets
    ///   from the system's content area vs. full surface.
    /// - **iOS/Desktop/WASM**: Returns zeros. iOS safe areas require native UIKit
    ///   code (winit's `inner_position()` returns `Err` on iOS).
    pub fn from_window(window: &Window) -> Self {
        #[cfg(target_os = "android")]
        {
            Self::from_window_android(window)
        }

        #[cfg(not(target_os = "android"))]
        {
            let _ = window;
            Self::default()
        }
    }

    #[cfg(target_os = "android")]
    fn from_window_android(window: &Window) -> Self {
        use winit::platform::android::WindowExtAndroid;

        let rect = window.content_rect();
        let size = window.inner_size();

        Self {
            top: rect.top as f32,
            left: rect.left as f32,
            bottom: (size.height as i32 - rect.bottom).max(0) as f32,
            right: (size.width as i32 - rect.right).max(0) as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_insets_are_zero() {
        let insets = SafeAreaInsets::default();
        assert!(insets.is_zero());
    }

    #[test]
    fn non_zero_insets() {
        let insets = SafeAreaInsets {
            top: 44.0,
            bottom: 34.0,
            left: 0.0,
            right: 0.0,
        };
        assert!(!insets.is_zero());
    }
}
