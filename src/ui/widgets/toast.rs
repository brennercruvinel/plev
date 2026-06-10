use crate::animation::Spring;
use crate::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::text::{TextMeasurer, TextStyle};
use crate::theme::{Intent, Theme};
use crate::ui::icons;

use super::{EventResult, Rect, WidgetEvent, intent_fill, with_alpha};

const WIDTH: f32 = 320.0;
const PAD: f32 = 12.0;
const GAP: f32 = 8.0;
const FONT: f32 = 13.0;
const MARGIN: f32 = 16.0;
const ICON: f32 = 16.0;

/// One queued notification.
#[derive(Clone, Debug)]
pub struct Toast {
    pub message: String,
    pub intent: Intent,
    /// Seconds shown so far (only advances while visible).
    age: f32,
    /// Entry/exit progress spring (0 hidden -> 1 shown).
    anim: Spring<f32>,
    closing: bool,
}

impl Toast {
    fn icon_name(&self) -> &'static str {
        match self.intent {
            Intent::Neutral | Intent::Informational => "info",
            Intent::Constructive => "check",
            Intent::Destructive => "alert-triangle",
        }
    }

    pub fn progress(&self) -> f32 {
        self.anim.get().clamp(0.0, 1.0)
    }

    pub fn is_closing(&self) -> bool {
        self.closing
    }
}

/// Notification queue with auto-dismiss.
///
/// At most [`max_visible`](ToastManager::max_visible) toasts show at once
/// (bottom-right stack); the rest wait in the queue. Each visible toast
/// dismisses itself after [`duration`](ToastManager::duration) seconds,
/// or immediately when clicked.
#[derive(Clone, Debug)]
pub struct ToastManager {
    toasts: Vec<Toast>,
    pub max_visible: usize,
    /// Auto-dismiss time in seconds.
    pub duration: f32,
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            max_visible: 4,
            duration: 4.0,
        }
    }

    /// Enqueue a toast. Spring physics follow the toast's intent.
    pub fn push(&mut self, message: impl Into<String>, intent: Intent, theme: &Theme) {
        let mut anim = Spring::new(0.0_f32).with_motion(&theme.intent_motion(intent));
        anim.set_target(1.0);
        self.toasts.push(Toast {
            message: message.into(),
            intent,
            age: 0.0,
            anim,
            closing: false,
        });
    }

    /// Total queued (visible + waiting).
    pub fn len(&self) -> usize {
        self.toasts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }

    /// Toasts currently on screen, oldest first.
    pub fn visible(&self) -> impl Iterator<Item = &Toast> {
        self.toasts.iter().take(self.max_visible)
    }

    pub fn visible_count(&self) -> usize {
        self.toasts.len().min(self.max_visible)
    }

    /// Begin dismissing the visible toast at `index` (animated).
    pub fn dismiss(&mut self, index: usize) {
        if let Some(t) = self.toasts.get_mut(index)
            && index < self.max_visible
            && !t.closing
        {
            t.closing = true;
            t.anim.set_target(0.0);
        }
    }

    /// Advance animations and lifetimes. Returns `true` while any toast is
    /// animating (callers keep requesting frames). Auto-dismiss only ages
    /// *visible* toasts, so queued ones get their full time on screen.
    pub fn tick(&mut self, dt: f32) -> bool {
        let max_visible = self.max_visible;
        let duration = self.duration;
        for (i, t) in self.toasts.iter_mut().enumerate() {
            if i >= max_visible {
                break;
            }
            t.anim.tick(dt);
            if !t.closing {
                t.age += dt;
                if t.age >= duration {
                    t.closing = true;
                    t.anim.set_target(0.0);
                }
            }
        }
        self.toasts
            .retain(|t| !(t.closing && !t.anim.is_animating()));
        self.is_animating()
    }

    pub fn is_animating(&self) -> bool {
        self.toasts
            .iter()
            .take(self.max_visible)
            .any(|t| t.anim.is_animating())
    }

    fn toast_height(message: &str) -> f32 {
        let style = TextStyle::new(FONT);
        let text_w = WIDTH - PAD * 2.0 - ICON - GAP;
        let (_, th) = TextMeasurer::measure_styled(message, &style, Some(text_w));
        th.max(FONT * 1.3) + PAD * 2.0
    }

    /// On-screen rects for visible toasts (bottom-right, stacking upward),
    /// with entry/exit progress applied as a slide.
    pub fn visible_rects(&self, vw: f32, vh: f32) -> Vec<Rect> {
        let x = vw - WIDTH - MARGIN;
        let mut y = vh - MARGIN;
        let mut rects = Vec::with_capacity(self.visible_count());
        for t in self.visible() {
            let h = Self::toast_height(&t.message);
            // Slide up while appearing; the gap collapses as it leaves.
            let progress = t.progress();
            y -= (h + GAP) * progress;
            rects.push(Rect::new(x, y + GAP * (1.0 - progress), WIDTH, h));
        }
        rects
    }

    /// Click-to-dismiss. Coordinates in the same space as `render`.
    pub fn handle_event(&mut self, event: &WidgetEvent, vw: f32, vh: f32) -> EventResult {
        let WidgetEvent::MouseDown { x, y } = *event else {
            return EventResult::IGNORED;
        };
        let rects = self.visible_rects(vw, vh);
        for (i, rect) in rects.iter().enumerate() {
            if rect.contains(x, y) {
                self.dismiss(i);
                return EventResult::clicked();
            }
        }
        EventResult::IGNORED
    }

    pub fn render(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) {
        let rects = self.visible_rects(vw, vh);
        for (t, rect) in self.visible().zip(rects) {
            let alpha = t.progress();
            if alpha <= 0.01 {
                continue;
            }
            let accent = intent_fill(theme, t.intent);

            compositor.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: rect.x,
                    y: rect.y,
                    w: rect.w,
                    h: rect.h,
                    color: with_alpha(theme.colors.bg_panel, 0.98 * alpha),
                    corner_radius: theme.radius.lg,
                    border_width: 1.0,
                    border_color: with_alpha(theme.colors.divider, alpha),
                },
            );
            // Intent accent bar on the left edge.
            compositor.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: rect.x,
                    y: rect.y,
                    w: 3.0,
                    h: rect.h,
                    color: [accent[0], accent[1], accent[2], alpha],
                    corner_radius: 1.5,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                },
            );
            if let Some(node) = icons::icon_at(
                t.icon_name(),
                ICON,
                [accent[0], accent[1], accent[2], alpha],
                rect.x + PAD,
                rect.y + PAD - 1.0,
            ) {
                compositor.push_to_layer(layer, node);
            }
            compositor.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::new(
                        &t.message,
                        FONT,
                        FONT * 1.3,
                        Some(rect.w - PAD * 2.0 - ICON - GAP),
                    ),
                    x: rect.x + PAD + ICON + GAP,
                    y: rect.y + PAD,
                    color: with_alpha(theme.colors.text, alpha),
                },
            );
        }
    }
}
