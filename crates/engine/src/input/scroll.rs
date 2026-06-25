/// Scroll state for a scrollable region.
///
/// Tracks vertical scroll offset and content/viewport dimensions.
/// Does not render anything — callers use `offset()` to shift their content.
///
/// # Example
/// ```
/// use engine::input::scroll::ScrollState;
/// let mut scroll = ScrollState::new();
/// scroll.set_viewport(400.0);
/// scroll.set_content(1200.0);
/// scroll.scroll_by(30.0);  // scroll down 30px
/// let offset = scroll.offset(); // use to shift scene nodes
/// assert_eq!(offset, 30.0);
/// ```
#[derive(Clone, Debug, Default)]
pub struct ScrollState {
    /// Current scroll offset (>= 0, increases as user scrolls down).
    offset: f32,
    /// Height of the visible viewport in pixels.
    viewport_height: f32,
    /// Total height of the scrollable content in pixels.
    content_height: f32,
}

impl ScrollState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_viewport(&mut self, h: f32) {
        self.viewport_height = h.max(0.0);
        self.clamp();
    }

    pub fn set_content(&mut self, h: f32) {
        self.content_height = h.max(0.0);
        self.clamp();
    }

    /// Scroll by `delta` pixels. Positive = scroll down (content moves up).
    pub fn scroll_by(&mut self, delta: f32) {
        self.offset += delta;
        self.clamp();
    }

    /// Scroll to absolute `offset`.
    pub fn scroll_to(&mut self, offset: f32) {
        self.offset = offset;
        self.clamp();
    }

    /// Current offset — subtract from child node Y positions to clip content.
    pub fn offset(&self) -> f32 {
        self.offset
    }

    /// Whether scrolling is needed (content taller than viewport).
    pub fn is_scrollable(&self) -> bool {
        self.content_height > self.viewport_height
    }

    /// Scrollbar thumb height ratio (0.0–1.0). Zero when not scrollable.
    pub fn thumb_ratio(&self) -> f32 {
        if self.content_height <= 0.0 {
            return 1.0;
        }
        (self.viewport_height / self.content_height).clamp(0.0, 1.0)
    }

    /// Scrollbar thumb position ratio (0.0–1.0).
    pub fn thumb_position(&self) -> f32 {
        let max = self.max_offset();
        if max <= 0.0 {
            return 0.0;
        }
        self.offset / max
    }

    pub(crate) fn max_offset(&self) -> f32 {
        (self.content_height - self.viewport_height).max(0.0)
    }

    fn clamp(&mut self) {
        self.offset = self.offset.clamp(0.0, self.max_offset());
    }
}

// ---------------------------------------------------------------------------
// SpringScroll -- scroll with physics-based deceleration via MotionPhysics
// ---------------------------------------------------------------------------

use crate::animation::Spring;
use crate::theme::MotionPhysics;

pub struct SpringScroll {
    state: ScrollState,
    spring: Spring<f32>,
}

impl SpringScroll {
    pub fn new(motion: &MotionPhysics) -> Self {
        let spring = Spring::new(0.0_f32).with_motion(motion);
        Self {
            state: ScrollState::new(),
            spring,
        }
    }

    pub fn set_viewport(&mut self, h: f32) {
        self.state.set_viewport(h);
    }

    pub fn set_content(&mut self, h: f32) {
        self.state.set_content(h);
    }

    /// Fling scroll: sets spring target to current + delta, clamped.
    pub fn fling(&mut self, delta: f32) {
        let target = (self.state.offset() + delta).clamp(0.0, self.state.max_offset());
        self.spring.set_target(target);
    }

    /// Tick the spring and update scroll offset.
    pub fn tick(&mut self, dt: f32) {
        self.spring.tick(dt);
        let value = self.spring.get().clamp(0.0, self.state.max_offset());
        self.state.offset = value;
    }

    pub fn offset(&self) -> f32 {
        self.state.offset()
    }

    pub fn is_animating(&self) -> bool {
        self.spring.is_animating()
    }

    pub fn is_scrollable(&self) -> bool {
        self.state.is_scrollable()
    }

    pub fn thumb_ratio(&self) -> f32 {
        self.state.thumb_ratio()
    }

    pub fn thumb_position(&self) -> f32 {
        self.state.thumb_position()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_offset_is_zero() {
        let s = ScrollState::new();
        assert_eq!(s.offset(), 0.0);
    }

    #[test]
    fn scroll_by_positive_increases_offset() {
        let mut s = ScrollState::new();
        s.set_viewport(100.0);
        s.set_content(500.0);
        s.scroll_by(50.0);
        assert_eq!(s.offset(), 50.0);
    }

    #[test]
    fn clamps_at_zero() {
        let mut s = ScrollState::new();
        s.set_viewport(100.0);
        s.set_content(500.0);
        s.scroll_by(-100.0);
        assert_eq!(s.offset(), 0.0);
    }

    #[test]
    fn clamps_at_max() {
        let mut s = ScrollState::new();
        s.set_viewport(100.0);
        s.set_content(500.0);
        s.scroll_by(10000.0);
        assert_eq!(s.offset(), 400.0); // 500 - 100
    }

    #[test]
    fn not_scrollable_when_content_fits() {
        let mut s = ScrollState::new();
        s.set_viewport(500.0);
        s.set_content(300.0);
        assert!(!s.is_scrollable());
    }

    #[test]
    fn thumb_ratio_full_when_not_scrollable() {
        let mut s = ScrollState::new();
        s.set_viewport(500.0);
        s.set_content(300.0);
        assert_eq!(s.thumb_ratio(), 1.0);
    }

    #[test]
    fn thumb_ratio_proportional() {
        let mut s = ScrollState::new();
        s.set_viewport(200.0);
        s.set_content(400.0);
        assert!((s.thumb_ratio() - 0.5).abs() < 1e-5);
    }

    // -- SpringScroll tests --

    fn test_motion() -> MotionPhysics {
        MotionPhysics {
            mass: 1.0,
            stiffness: 170.0,
            damping: 26.0,
        }
    }

    #[test]
    fn spring_scroll_default_zero() {
        let ss = SpringScroll::new(&test_motion());
        assert_eq!(ss.offset(), 0.0);
        assert!(!ss.is_animating());
    }

    #[test]
    fn spring_scroll_fling_converges() {
        let mut ss = SpringScroll::new(&test_motion());
        ss.set_viewport(100.0);
        ss.set_content(500.0);
        ss.fling(200.0);
        assert!(ss.is_animating());
        for _ in 0..300 {
            ss.tick(1.0 / 60.0);
        }
        assert!(!ss.is_animating());
        assert!((ss.offset() - 200.0).abs() < 1.0, "offset={}", ss.offset());
    }

    #[test]
    fn spring_scroll_clamps_at_max() {
        let mut ss = SpringScroll::new(&test_motion());
        ss.set_viewport(100.0);
        ss.set_content(500.0);
        ss.fling(9999.0);
        for _ in 0..300 {
            ss.tick(1.0 / 60.0);
        }
        assert!(
            ss.offset() <= 400.0,
            "offset={} should be <= 400",
            ss.offset()
        );
    }

    #[test]
    fn spring_scroll_destructive_motion_faster() {
        use crate::theme::{Intent, Theme};
        let theme = Theme::dark();
        let neutral_motion = &theme.motion;
        let destructive_motion = theme.intent_motion(Intent::Destructive);

        let mut ss_n = SpringScroll::new(neutral_motion);
        ss_n.set_viewport(100.0);
        ss_n.set_content(500.0);
        ss_n.fling(200.0);

        let mut ss_d = SpringScroll::new(&destructive_motion);
        ss_d.set_viewport(100.0);
        ss_d.set_content(500.0);
        ss_d.fling(200.0);

        // Tick both for 1 second
        for _ in 0..60 {
            ss_n.tick(1.0 / 60.0);
            ss_d.tick(1.0 / 60.0);
        }
        // Destructive (snappier) should be closer to target after same time
        let n_dist = (ss_n.offset() - 200.0).abs();
        let d_dist = (ss_d.offset() - 200.0).abs();
        assert!(
            d_dist <= n_dist + 1.0,
            "destructive dist={d_dist} should be <= neutral dist={n_dist}"
        );
    }

    #[test]
    fn spring_scroll_linear_still_works() {
        let mut s = ScrollState::new();
        s.set_viewport(100.0);
        s.set_content(500.0);
        s.scroll_by(50.0);
        assert_eq!(s.offset(), 50.0);
    }
}
