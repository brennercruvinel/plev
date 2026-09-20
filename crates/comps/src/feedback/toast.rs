//! HOFF notify toast: a surface-active glass slab at the item radius,
//! intent icon on the left, base-2r message, stacked bottom-right and
//! sprung in with the intent's motion. `size.toast_w` wide on a desktop;
//! on a compact viewport it spans the width minus the gutter, so a
//! phone gets a full-width banner instead of a clipped card.

use crate::icons;
use engine::animation::Spring;
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{Breakpoint, IconSize, Intent, Theme};

use crate::core::{EventResult, Rect, WidgetEvent, intent_fill, with_alpha};
use crate::recipe::{rounded_rect, rounded_rect_stroke};

/// The per-theme geometry the rects, the hit test and the render share.
struct Metrics {
    width: f32,
    pad_l: f32,
    pad_r: f32,
    pad_y: f32,
    radius: f32,
    gap: f32,
    margin: f32,
    icon: f32,
    icon_gap: f32,
}

impl Metrics {
    fn of(theme: &Theme, vw: f32) -> Self {
        let bp = theme.layout.breakpoint(vw);
        let margin = theme.layout.gutter(bp).min(theme.spacing.md);
        let width = match bp {
            Breakpoint::Compact => (vw - margin * 2.0).max(0.0),
            _ => theme.size.toast_w.min((vw - margin * 2.0).max(0.0)),
        };
        Self {
            width,
            pad_l: theme.spacing.sm,
            pad_r: theme.spacing.lg,
            pad_y: theme.spacing.sm + theme.spacing.xs / 2.0,
            radius: theme.shape.item,
            gap: theme.spacing.sm + theme.spacing.xs / 2.0,
            margin,
            icon: theme.control.icon(IconSize::Sm),
            icon_gap: theme.control.inline_gap,
        }
    }

    fn text_w(&self) -> f32 {
        (self.width - self.pad_l - self.pad_r - self.icon - self.icon_gap).max(0.0)
    }
}

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
        // Drop a toast once its close animation has finished.
        self.toasts.retain(|t| !t.closing || t.anim.is_animating());
        self.is_animating()
    }

    pub fn is_animating(&self) -> bool {
        self.toasts
            .iter()
            .take(self.max_visible)
            .any(|t| t.anim.is_animating())
    }

    /// Message style: base-2r, the same for measuring and rendering
    /// (a mismatch here makes multiline toasts overflow their padding).
    fn text_style(theme: &Theme) -> TextStyle {
        theme.typography.base_2r()
    }

    fn toast_height(message: &str, theme: &Theme, m: &Metrics) -> f32 {
        let style = Self::text_style(theme);
        let (_, th) = TextMeasurer::measure_styled(message, &style, Some(m.text_w()));
        th.max(style.line_height) + m.pad_y * 2.0
    }

    /// On-screen rects for visible toasts (bottom-right, stacking upward),
    /// with entry/exit progress applied as a slide.
    pub fn visible_rects(&self, theme: &Theme, vw: f32, vh: f32) -> Vec<Rect> {
        let m = Metrics::of(theme, vw);
        let x = vw - m.width - m.margin;
        let mut y = vh - m.margin;
        let mut rects = Vec::with_capacity(self.visible_count());
        for t in self.visible() {
            let h = Self::toast_height(&t.message, theme, &m);
            // Slide up while appearing; the gap collapses as it leaves.
            let progress = t.progress();
            y -= (h + m.gap) * progress;
            rects.push(Rect::new(x, y + m.gap * (1.0 - progress), m.width, h));
        }
        rects
    }

    /// Click-to-dismiss. Coordinates in the same space as `render`.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) -> EventResult {
        let WidgetEvent::MouseDown { x, y } = *event else {
            return EventResult::IGNORED;
        };
        let rects = self.visible_rects(theme, vw, vh);
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
        let m = Metrics::of(theme, vw);
        let rects = self.visible_rects(theme, vw, vh);
        let glass = &theme.glass;
        let style = Self::text_style(theme);
        for (t, rect) in self.visible().zip(rects) {
            let alpha = t.progress();
            if alpha <= 0.01 {
                continue;
            }
            let accent = intent_fill(theme, t.intent);

            // Notify surface: surface-active glass; the intent icon pushed
            // later stays on top (push order is preserved across types).
            compositor.push_to_layer(
                layer,
                rounded_rect(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    m.radius,
                    with_alpha(glass.surface_active, glass.surface_active.0[3] * alpha),
                ),
            );
            compositor.push_to_layer(
                layer,
                rounded_rect_stroke(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    m.radius,
                    with_alpha(glass.edge_soft, glass.edge_soft.0[3] * alpha),
                    theme.control.edge_width,
                ),
            );
            if let Some(node) = icons::icon_at(
                t.icon_name(),
                m.icon,
                [accent[0], accent[1], accent[2], accent[3] * alpha],
                rect.x + m.pad_l,
                rect.y + m.pad_y,
            ) {
                compositor.push_to_layer(layer, node);
            }
            let text = glass.text_active;
            compositor.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(&t.message, &style, Some(m.text_w())),
                    x: rect.x + m.pad_l + m.icon + m.icon_gap,
                    y: rect.y + m.pad_y,
                    color: with_alpha(text, text.0[3] * alpha),
                },
            );
        }
    }
}
