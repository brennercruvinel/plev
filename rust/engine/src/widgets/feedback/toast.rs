//! The toast of the hoff core kit: the alert's notification row (its
//! intent's mark in the lead disc, the message beside it, wrapped) on the
//! kit's dropdown surface (paper, a hairline, a deep drop: `theme.message`),
//! stacked bottom-right and sprung in with the intent's motion.
//! `size.toast_w` wide on a desktop; on a compact viewport it spans the
//! width minus the gutter, so a phone gets a full-width banner instead of a
//! clipped card. The toast is the passing one; the alert stays in the page
//! (`feedback::alert`).
//!
//! With a [`SemId`] ([`ToastManager::sem`]) the render binds each toast
//! on screen as `<id>[<toast id>]` to the rect it draws and hit-tests it
//! with, and [`ToastManager::project`] declares the toasts in a semantic
//! projection.
//!
//! A toast at rest is a deadline, not motion: [`ToastManager::wake`] asks
//! for frames while one springs in or out and, between, for the instant
//! the next one dismisses itself.

use std::time::Duration;

use crate::compose::animation::Spring;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::{Breakpoint, Intent, Theme};
use crate::widgets::icons;
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::styling::color::Color;
use crate::widgets::recipe::{rounded_rect, shadow_drops};
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, contrast_text};

/// The per-theme geometry the rects, the hit test and the render share:
/// the alert's row in the toast's width.
struct Metrics {
    width: f32,
    pad: f32,
    pad_end: f32,
    lead: f32,
    gap: f32,
    icon: f32,
    stack_gap: f32,
    margin: f32,
}

impl Metrics {
    fn of(theme: &Theme, vw: f32) -> Self {
        let bp = theme.layout.breakpoint(vw);
        let margin = theme.layout.gutter(bp).min(theme.spacing.md);
        let width = match bp {
            Breakpoint::Compact => (vw - margin * 2.0).max(0.0),
            _ => theme.size.toast_w.min((vw - margin * 2.0).max(0.0)),
        };
        let a = &theme.message.alert;
        Self {
            width,
            pad: a.pad,
            pad_end: a.pad_end,
            lead: a.lead,
            gap: a.gap,
            // the badge's share of its disc, as the alert's lone mark
            icon: a.lead * a.badge_icon / a.badge,
            stack_gap: a.action_gap,
            margin,
        }
    }

    fn text_w(&self) -> f32 {
        (self.width - self.pad - self.lead - self.gap - self.pad_end).max(0.0)
    }
}

/// One queued notification.
#[derive(Clone, Debug)]
pub struct Toast {
    /// Given by the manager at `push`, unique while it lives: the toast's
    /// identity, whatever its place in the stack.
    id: u64,
    pub message: String,
    pub intent: Intent,
    /// Seconds shown so far (only advances while visible).
    age: f32,
    /// Entry/exit progress spring (0 hidden -> 1 shown).
    anim: Spring<f32>,
    closing: bool,
}

impl Toast {
    pub fn id(&self) -> u64 {
        self.id
    }

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
    next_id: u64,
    sem: Option<SemId>,
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
            next_id: 0,
            sem: None,
        }
    }

    /// The logical identity the toasts are keys under: the render binds
    /// each toast on screen as `<id>[<toast id>]`
    /// (`Compositor::bind_to_layer`).
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Declares the toasts as `id` in a projection: a `Role::Region` named
    /// `label` with one child `<id>[<toast id>]` per toast on screen that
    /// is not already leaving, a `Role::Alert` for a destructive toast and
    /// a `Role::Status` otherwise, named by its message, with the
    /// `dismiss` a click performs. Queued toasts are not on screen yet and
    /// the entry and exit slides are animation, so neither is projected.
    /// It only reads the widget; the app owns it and runs `dismiss`
    /// ([`ToastManager::dismiss_id`]). Pass the id given to
    /// [`ToastManager::sem`] so the nodes and the bindings name the same
    /// toasts. Returns the region's node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        for t in self.visible().filter(|t| !t.closing) {
            let role = if t.intent == Intent::Destructive {
                Role::Alert
            } else {
                Role::Status
            };
            let mut node = b.node(id.key(t.id.to_string()), role, t.message.as_str());
            node.parent(id);
            node.action("dismiss");
        }
        let mut node = b.node(id, Role::Region, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node
    }

    /// Begin dismissing the toast `id` if it is on screen, like a click on
    /// it. `false` when it is not on screen or already leaving.
    pub fn dismiss_id(&mut self, id: u64) -> bool {
        let Some(index) = self.visible().position(|t| t.id == id && !t.closing) else {
            return false;
        };
        self.dismiss(index);
        true
    }

    /// Enqueue a toast. Spring physics follow the toast's intent.
    pub fn push(&mut self, message: impl Into<String>, intent: Intent, theme: &Theme) {
        let mut anim = Spring::new(0.0_f32).with_motion(&theme.intent_motion(intent));
        anim.set_target(1.0);
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.toasts.push(Toast {
            id,
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

    /// Advance animations and lifetimes by `dt` seconds and say what the
    /// toasts ask of the loop: a frame now and on the next while one
    /// springs in or out; while all rest, the time until the first on
    /// screen dismisses itself; nothing without toasts. Auto-dismiss only
    /// ages *visible* toasts, so queued ones get their full time on
    /// screen. A `dt` that is not time (negative, NaN, infinite) counts as
    /// none. The tick counts all the time it is given: a shell advances
    /// the toasts to the instant of an event before a push or a dismiss,
    /// so the gap before it is not counted as time on screen.
    pub fn wake(&mut self, dt: f32) -> Wake {
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };
        let duration = self.duration;
        for t in self.toasts.iter_mut().take(self.max_visible) {
            t.anim.tick(dt);
            if !t.closing {
                t.age += dt;
                if t.age >= duration {
                    t.closing = true;
                    t.anim.set_target(0.0);
                }
            }
        }
        // Drop a toast once its close animation has finished; the frame
        // that no longer draws it is a visible change.
        let count = self.toasts.len();
        self.toasts.retain(|t| !t.closing || t.anim.is_animating());
        let mut wake = if self.toasts.len() < count {
            Wake::now()
        } else {
            Wake::idle()
        };
        for t in self.visible() {
            if t.anim.is_animating() {
                wake = wake.merge(Wake::continuous());
            } else if !t.closing
                && let Ok(left) = Duration::try_from_secs_f32(duration - t.age)
            {
                // an infinite or NaN duration never dismisses: no deadline
                wake = wake.merge(Wake::after(left));
            }
        }
        wake
    }

    pub fn is_animating(&self) -> bool {
        self.toasts
            .iter()
            .take(self.max_visible)
            .any(|t| t.anim.is_animating())
    }

    /// The message's style: the alert's detail run, the same for
    /// measuring and rendering (a mismatch makes a long toast overflow).
    fn text_style(theme: &Theme) -> TextStyle {
        theme.message.alert.detail.style(&theme.typography)
    }

    /// As tall as the message wraps, never under the lead disc.
    fn toast_height(message: &str, theme: &Theme, m: &Metrics) -> f32 {
        let style = Self::text_style(theme);
        let (_, th) = TextMeasurer::measure_styled(message, &style, Some(m.text_w()));
        th.max(style.line_height).max(m.lead) + m.pad * 2.0
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
            y -= (h + m.stack_gap) * progress;
            rects.push(Rect::new(x, y + m.stack_gap * (1.0 - progress), m.width, h));
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
        let t = &theme.message.toast;
        let a = &theme.message.alert;
        let style = Self::text_style(theme);
        let fade = |c: [f32; 4], alpha: f32| [c[0], c[1], c[2], c[3] * alpha];
        for (toast, rect) in self.visible().zip(rects) {
            let alpha = toast.progress();
            if alpha <= 0.01 {
                continue;
            }
            // A toast binds while it is drawn and a click still dismisses
            // it; a leaving one is already dismissed.
            if let Some(id) = &self.sem
                && !toast.closing
            {
                compositor.bind_to_layer(
                    layer,
                    &id.key(toast.id.to_string()),
                    [rect.x, rect.y, rect.w, rect.h],
                );
            }
            // The surface: the drop, then the paper with its hairline; the
            // row pushed later stays on top (push order is kept).
            let mut drop = t.shadow;
            drop.color = Color(fade(drop.color.0, alpha));
            shadow_drops(
                compositor,
                layer,
                rect,
                t.radius,
                std::slice::from_ref(&drop),
            );
            compositor.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: rect.x,
                    y: rect.y,
                    w: rect.w,
                    h: rect.h,
                    color: fade(t.fill.0, alpha),
                    corner_radius: t.radius,
                    border_width: t.border_width,
                    border_color: fade(t.border.0, alpha),
                },
            );
            // The lead: the intent's mark, as the alert without an avatar.
            let fill = a.intents.of(toast.intent).0;
            let lead = Rect::new(
                rect.x + m.pad,
                rect.y + (rect.h - m.lead) / 2.0,
                m.lead,
                m.lead,
            );
            compositor.push_to_layer(
                layer,
                rounded_rect(
                    lead.x,
                    lead.y,
                    lead.w,
                    lead.h,
                    lead.w / 2.0,
                    fade(fill, alpha),
                ),
            );
            let (cx, cy) = lead.center();
            if let Some(node) = icons::icon_at(
                toast.icon_name(),
                m.icon,
                fade(contrast_text(fill), alpha),
                cx - m.icon / 2.0,
                cy - m.icon / 2.0,
            ) {
                compositor.push_to_layer(layer, node);
            }
            let (_, th) = TextMeasurer::measure_styled(&toast.message, &style, Some(m.text_w()));
            compositor.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(&toast.message, &style, Some(m.text_w())),
                    x: lead.right() + m.gap,
                    y: rect.y + (rect.h - th.max(style.line_height)) / 2.0,
                    color: fade(a.detail.color.0, alpha),
                },
            );
        }
    }
}
