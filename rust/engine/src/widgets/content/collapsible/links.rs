//! The list a Dropdown reveals (the kit's HomeSidebar): links one under
//! the other, a rail down their left, a rounded branch from the rail to
//! each link, the current link inked and marked by a chevron. It is the
//! content a [`Collapsible`] draws in its closure, so it is drawn and hit
//! in the rect the collapsible gives it, clipped to what is revealed.
//!
//! Each link is projected as a `Role::Link` `<id>[<i>]` with its `active`
//! state and an `activate` that reads it, as a nav link is; the app runs
//! the action ([`DisclosureList::activate`]).
//!
//! [`Collapsible`]: super::Collapsible

use accord::{Role, SemId, SnapshotBuilder};

use super::super::tokens::DisclosureRail;
use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::text::{OverflowWrap, TextMeasurer, TextStyle};
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::hit::hit_rect;
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring_inside, rounded_rect_stroke};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// Links under a rail.
#[derive(Clone, Debug, Default)]
pub struct DisclosureList {
    links: Vec<String>,
    current: Option<usize>,
    hovered: Option<usize>,
    pressed: Option<usize>,
    focused: Option<usize>,
    disabled: bool,
    /// Each label on one line, cut with an ellipsis (a sidebar's rows).
    truncate: bool,
    /// Each link is pressed and bound in [`hit_rect`] around its drawing
    /// ([`DisclosureList::grow_hit`]).
    grow_hit: bool,
    sem: Option<SemId>,
}

impl DisclosureList {
    pub fn new<S: Into<String>>(links: impl IntoIterator<Item = S>) -> Self {
        Self {
            links: links.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }

    /// Link `i` is the current one.
    pub fn current(mut self, i: usize) -> Self {
        if i < self.links.len() {
            self.current = Some(i);
        }
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self.hovered = None;
        self.pressed = None;
        self
    }

    /// Each label on one line, cut with an ellipsis where it does not
    /// fit (CSS `truncate`), rather than wrapped; the projected name is
    /// the whole label.
    pub fn truncate(mut self, on: bool) -> Self {
        self.truncate = on;
        self
    }

    /// Each link's press area, and its binding, grow to the minimum
    /// around its drawing ([`hit_rect`]: 24 px, 44 with a finger), which
    /// does not change; a press two grown links share is the nearer
    /// link's. Off, the press area is the drawing.
    pub fn grow_hit(mut self, on: bool) -> Self {
        self.grow_hit = on;
        self
    }

    /// Where a press on link `at` lands, and what it binds.
    fn press_rect(&self, at: Rect, theme: &Theme) -> Rect {
        if self.grow_hit {
            hit_rect(at, theme)
        } else {
            at
        }
    }

    /// The link a press at `(x, y)` lands on: the one drawn under it, or
    /// else the nearest whose grown area holds it.
    fn link_at(&self, rects: &[Rect], x: f32, y: f32, theme: &Theme) -> Option<usize> {
        rects.iter().position(|r| r.contains(x, y)).or_else(|| {
            let off = |r: &Rect| (y - r.center().1).abs();
            (0..rects.len())
                .filter(|&i| self.press_rect(rects[i], theme).contains(x, y))
                .min_by(|&a, &b| off(&rects[a]).total_cmp(&off(&rects[b])))
        })
    }

    /// The stem of the links' ids: link `i` is `<id>[<i>]`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn len(&self) -> usize {
        self.links.len()
    }

    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }

    pub fn current_link(&self) -> Option<usize> {
        self.current
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// `<id>[<i>]`, the id of link `i`.
    pub fn link_id(&self, i: usize) -> Option<SemId> {
        (i < self.links.len())
            .then(|| self.sem.as_ref().map(|id| id.key(i.to_string())))
            .flatten()
    }

    /// The link whose id is `id`.
    pub fn position(&self, id: &SemId) -> Option<usize> {
        (0..self.links.len()).find(|i| self.link_id(*i).as_ref() == Some(id))
    }

    /// Makes link `i` the current one; `false` when disabled or not there.
    pub fn activate(&mut self, i: usize) -> bool {
        if self.disabled || i >= self.links.len() {
            return false;
        }
        self.current = Some(i);
        true
    }

    /// Link `i` is the current one, or none is (the current page is in
    /// another list): what the app says, not a press.
    pub fn set_current(&mut self, i: Option<usize>) {
        self.current = i.filter(|i| *i < self.links.len());
    }

    /// The keyboard focus on link `i`, or on none.
    pub fn set_focused(&mut self, i: Option<usize>) {
        self.focused = i.filter(|i| *i < self.links.len());
    }

    fn rail(theme: &Theme) -> Option<&DisclosureRail> {
        theme.disclosure.collapsible.rail.as_ref()
    }

    /// The rows' run; a word wider than its row (a larger text, a longer
    /// language) breaks between its letters where the row wraps.
    fn style(theme: &Theme) -> TextStyle {
        let r = &theme.disclosure.collapsible;
        theme
            .typography
            .run(r.body_type)
            .with_overflow_wrap(OverflowWrap::Anywhere)
    }

    /// The width a label wraps at, in a list `w` wide.
    fn label_w(rail: &DisclosureRail, w: f32) -> f32 {
        let [left, right] = rail.link_pad;
        (w - left - right - rail.link_chevron).max(0.0)
    }

    /// Each link's rect in `bounds`: at least the kit's height, taller
    /// when its label wraps (never, truncated).
    pub fn link_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        let Some(rail) = Self::rail(theme) else {
            return Vec::new();
        };
        let style = Self::style(theme);
        let label_w = Self::label_w(rail, bounds.w);
        let mut y = bounds.y;
        self.links
            .iter()
            .map(|label| {
                let h = rail.link_h.max(self.label_h(label, &style, label_w));
                let r = Rect::new(bounds.x, y, bounds.w, h);
                y += h;
                r
            })
            .collect()
    }

    /// A label's height: one line, truncated; as it wraps otherwise.
    fn label_h(&self, label: &str, style: &TextStyle, label_w: f32) -> f32 {
        if self.truncate {
            style.line_height
        } else {
            TextMeasurer::measure_styled(label, style, Some(label_w)).1
        }
    }

    /// The list's height at width `w`.
    pub fn height(&self, w: f32, theme: &Theme) -> f32 {
        self.link_rects(Rect::new(0.0, 0.0, w, 0.0), theme)
            .last()
            .map_or(0.0, |r| r.bottom())
    }

    /// Hover and press on the links; a release on the link pressed makes
    /// it current (`clicked`). A disabled list reacts to nothing.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        if self.disabled {
            return EventResult::IGNORED;
        }
        let rects = self.link_rects(bounds, theme);
        let (x, y) = event.pos();
        let at = self.link_at(&rects, x, y, theme);
        match *event {
            WidgetEvent::MouseMove { .. } if at != self.hovered => {
                self.hovered = at;
                EventResult::changed()
            }
            WidgetEvent::MouseDown { .. } if at.is_some() => {
                self.pressed = at;
                EventResult::changed()
            }
            WidgetEvent::MouseUp { .. } if self.pressed.is_some() => {
                let pressed = self.pressed.take();
                if at.is_some() && at == pressed {
                    self.current = at;
                    EventResult::clicked()
                } else {
                    EventResult::changed()
                }
            }
            _ => EventResult::IGNORED,
        }
    }

    /// The rail, the branches, the labels and the current link's chevron
    /// in `bounds`; each link binds where it draws.
    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let Some(rail) = Self::rail(theme) else {
            return;
        };
        let alpha = if self.disabled {
            theme.glass.disabled_alpha
        } else {
            1.0
        };
        let rects = self.link_rects(bounds, theme);
        let total = rects.last().map_or(0.0, |r| r.bottom() - bounds.y);
        let rail_x = bounds.x + rail.x;
        let rail_h = (total - rail.end).max(0.0);
        if rail_h > 0.0 {
            c.push(SceneNode::Rect {
                x: rail_x,
                y: bounds.y,
                w: rail.width,
                h: rail_h,
                color: faded(rail.color, alpha),
            });
        }
        let style = Self::style(theme);
        let label_w = Self::label_w(rail, bounds.w);
        let [pad_left, pad_right] = rail.link_pad;
        for (i, (label, at)) in self.links.iter().zip(&rects).enumerate() {
            if let Some(id) = self.link_id(i) {
                let p = self.press_rect(*at, theme);
                c.bind(&id, [p.x, p.y, p.w, p.h]);
            }
            // The branch: a rounded box whose top and right edges fall
            // outside the clip, leaving its left edge, its bottom edge and
            // the bottom-left curve.
            let r = rail.branch_radius;
            c.push_clip(rail_x, at.y, rail.branch_w, rail.branch_h);
            c.push(rounded_rect_stroke(
                rail_x,
                at.y - r * 2.0,
                rail.branch_w + r * 2.0,
                rail.branch_h + r * 2.0,
                r,
                faded(rail.color, alpha),
                rail.width,
            ));
            c.pop_clip();

            let lit = self.current == Some(i) || self.hovered == Some(i);
            let color = if lit {
                rail.link_active
            } else {
                rail.link_rest
            };
            let text_h = self.label_h(label, &style, label_w);
            let key = if self.truncate {
                let shown = TextMeasurer::truncate_to_width(label, &style, label_w);
                TextNodeKey::from_style(&shown, &style, None)
            } else {
                TextNodeKey::from_style(label, &style, Some(label_w))
            };
            c.push(SceneNode::Text {
                key,
                x: at.x + pad_left,
                y: at.y + (at.h - text_h) / 2.0,
                color: faded(color, alpha),
            });
            if self.current == Some(i)
                && let Some(node) = icons::icon_at(
                    "chevron-right",
                    rail.link_chevron,
                    faded(theme.disclosure.collapsible.chevron_color, alpha),
                    at.right() - pad_right - rail.link_chevron,
                    at.y + (at.h - rail.link_chevron) / 2.0,
                )
            {
                c.push(node);
            }
            if self.focused == Some(i) && !self.disabled {
                c.push(focus_ring_inside(*at, theme.shape.nav, theme));
            }
        }
    }

    /// Declares each link, `<id>[<i>]`, a `Role::Link` named by its label
    /// under `parent`, enabled unless the list is disabled, with `active`
    /// and an `activate` that reads it.
    #[track_caller]
    pub fn project(&self, b: &mut SnapshotBuilder, parent: Option<&SemId>) {
        for (i, label) in self.links.iter().enumerate() {
            let Some(id) = self.link_id(i) else {
                continue;
            };
            let mut node = b.node(&id, Role::Link, label.as_str());
            if let Some(parent) = parent {
                node.parent(parent);
            }
            node.enabled(!self.disabled)
                .state("active", self.current == Some(i));
            node.action("activate").reads(format!("{id}:active"));
        }
    }
}

fn faded(c: Color, alpha: f32) -> [f32; 4] {
    [c.0[0], c.0[1], c.0[2], c.0[3] * alpha]
}
