//! Breadcrumb: the path to where the user is. Each crumb is a small
//! target in its own box, the last one where the user is (not a target),
//! chevrons between them; every crumb but the last is a target (click
//! fires on release inside, like a button). When the trail is wider than
//! its bounds the leading crumbs collapse into an ellipsis crumb, so the
//! current place always shows. Every visual value is the theme's
//! `navigation.breadcrumb` recipe.
//!
//! With a [`SemId`] ([`Breadcrumb::sem`]) the render binds the trail and
//! each crumb it draws to the rects it draws and hit-tests them with, and
//! [`Breadcrumb::project`] declares the trail in a semantic projection.
//! Keyboard focus on a crumb ([`Breadcrumb::set_focused`], driven by the
//! owning view) rings it where it is drawn.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring, rounded_rect};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

const ELLIPSIS: &str = "\u{2026}";

/// How far a crumb may run past its room before it is cut: the sums of
/// measured widths drift by a fraction of a pixel.
const SLACK: f32 = 0.5;

#[derive(Clone, Debug)]
pub struct Breadcrumb {
    pub crumbs: Vec<String>,
    hovered: Option<usize>,
    pressed: Option<usize>,
    focused: Option<usize>,
    sem: Option<SemId>,
}

impl Breadcrumb {
    pub fn new(crumbs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            crumbs: crumbs.into_iter().map(Into::into).collect(),
            hovered: None,
            pressed: None,
            focused: None,
            sem: None,
        }
    }

    /// The logical identity the render binds the trail to
    /// (`Compositor::bind_to_layer`); each drawn crumb binds as
    /// `<id>[<index>]`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Declares this trail as `id` in a projection: a `Role::Group` named
    /// `label` with one child `<id>[<index>]` per crumb, collapsed ones
    /// included (they are still places to go, only not drawn). Every crumb
    /// but the last is a `Role::Link` with `activate`; the last is where
    /// the user is, not a target, so it is a `Role::Label`. It only reads
    /// the widget; the app owns it and runs `activate` (navigates). Pass
    /// the id given to [`Breadcrumb::sem`] so the nodes and the bindings
    /// name the same controls. Returns the group's node for the app to add
    /// to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let last = self.crumbs.len().saturating_sub(1);
        for (i, crumb) in self.crumbs.iter().enumerate() {
            let role = if i == last { Role::Label } else { Role::Link };
            let mut node = b.node(id.key(i.to_string()), role, crumb.as_str());
            node.parent(id);
            if i != last {
                node.action("activate");
            }
        }
        let mut node = b.node(id, Role::Group, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node
    }

    pub fn hovered(&self) -> Option<usize> {
        self.hovered
    }

    /// Keyboard focus on the crumb at `index`, or on none, driven by the
    /// owning view (plev has no global focus chain): the accent ring
    /// around it while it is drawn (a collapsed crumb shows none).
    pub fn set_focused(&mut self, index: Option<usize>) {
        self.focused = index.filter(|i| *i < self.crumbs.len());
    }

    pub fn focused(&self) -> Option<usize> {
        self.focused
    }

    pub fn pressed(&self) -> Option<usize> {
        self.pressed
    }

    pub fn text_style(theme: &Theme) -> TextStyle {
        theme.navigation.breadcrumb.text.style(&theme.typography)
    }

    /// A crumb's height, its box included: what the trail needs.
    pub fn height(theme: &Theme) -> f32 {
        let recipe = &theme.navigation.breadcrumb;
        Self::text_style(theme).line_height + recipe.pad_y * 2.0
    }

    /// The width a chevron takes between two crumbs.
    fn sep(theme: &Theme) -> f32 {
        let recipe = &theme.navigation.breadcrumb;
        recipe.gap * 2.0 + recipe.chevron
    }

    /// A crumb's width: its text and its box.
    fn crumb_w(text: &str, style: &TextStyle, theme: &Theme) -> f32 {
        TextMeasurer::measure_styled(text, style, None).0 + theme.navigation.breadcrumb.pad_x * 2.0
    }

    /// The trail's whole width, nothing folded: what it asks of a row.
    pub fn preferred_width(&self, theme: &Theme) -> f32 {
        let style = Self::text_style(theme);
        let crumbs: f32 = self
            .crumbs
            .iter()
            .map(|c| Self::crumb_w(c, &style, theme))
            .sum();
        crumbs + Self::sep(theme) * self.crumbs.len().saturating_sub(1) as f32
    }

    /// Visible crumbs for `width`: indices into `crumbs`, with `None`
    /// standing for the ellipsis. Leading crumbs collapse first; the
    /// first and the last stay as long as they fit. A crumb with the
    /// keyboard focus is never folded away: the focus goes where it can
    /// be seen.
    pub fn visible(&self, width: f32, theme: &Theme) -> Vec<Option<usize>> {
        let n = self.crumbs.len();
        if n == 0 {
            return Vec::new();
        }
        let style = Self::text_style(theme);
        let sep = Self::sep(theme);
        let w = |s: &str| Self::crumb_w(s, &style, theme);
        let total = |items: &[Option<usize>]| -> f32 {
            items
                .iter()
                .map(|i| match i {
                    Some(i) => w(&self.crumbs[*i]),
                    None => w(ELLIPSIS),
                })
                .sum::<f32>()
                + sep * (items.len().saturating_sub(1)) as f32
        };
        let all: Vec<Option<usize>> = (0..n).map(Some).collect();
        if n == 1 || total(&all) <= width {
            return all;
        }
        let shows = |items: &[Option<usize>]| match self.focused {
            Some(f) => items.contains(&Some(f)),
            None => true,
        };
        // Collapse from the second crumb onward, keeping the first.
        for keep_tail in (1..n - 1).rev() {
            let mut items = vec![Some(0), None];
            items.extend((n - keep_tail..n).map(Some));
            if shows(&items) && total(&items) <= width {
                return items;
            }
        }
        // Nothing but the last fits (it truncates if even it does not),
        // and the focused crumb before it, with what it folds marked.
        match self.focused.filter(|f| f + 1 < n) {
            Some(0) => vec![Some(0), None, Some(n - 1)],
            Some(f) if f + 2 == n => vec![None, Some(f), Some(n - 1)],
            Some(f) => vec![None, Some(f), None, Some(n - 1)],
            None => vec![None, Some(n - 1)],
        }
    }

    /// Crumb rects for the visible trail (parallel to `visible`): each
    /// crumb's box, centered on the bounds' height, the last one cut to
    /// what is left of the bounds (its text truncates).
    pub fn crumb_rects(&self, bounds: Rect, theme: &Theme) -> Vec<(Option<usize>, Rect)> {
        let style = Self::text_style(theme);
        let sep = Self::sep(theme);
        let h = Self::height(theme);
        let y = bounds.y + (bounds.h - h) / 2.0;
        let mut x = bounds.x;
        self.visible(bounds.w, theme)
            .into_iter()
            .map(|i| {
                let text = match i {
                    Some(i) => self.crumbs[i].as_str(),
                    None => ELLIPSIS,
                };
                let x0 = x.min(bounds.right());
                let (full, room) = (Self::crumb_w(text, &style, theme), bounds.right() - x0);
                let w = if full > room + SLACK { room } else { full };
                let r = Rect::new(x0, y, w, h);
                x += w + sep;
                (i, r)
            })
            .collect()
    }

    fn crumb_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<usize> {
        let last = self.crumbs.len().saturating_sub(1);
        self.crumb_rects(bounds, theme)
            .into_iter()
            .find_map(|(i, r)| match i {
                Some(i) if i != last && r.contains(x, y) => Some(i),
                _ => None,
            })
    }

    /// Returns the clicked crumb index on activation.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> (EventResult, Option<usize>) {
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = self.crumb_at(x, y, bounds, theme);
                if hit != self.hovered {
                    self.hovered = hit;
                    (EventResult::changed(), None)
                } else {
                    (EventResult::IGNORED, None)
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                self.pressed = self.crumb_at(x, y, bounds, theme);
                if self.pressed.is_some() {
                    (EventResult::changed(), None)
                } else {
                    (EventResult::IGNORED, None)
                }
            }
            WidgetEvent::MouseUp { x, y } => {
                let Some(pressed) = self.pressed.take() else {
                    return (EventResult::IGNORED, None);
                };
                if self.crumb_at(x, y, bounds, theme) == Some(pressed) {
                    (EventResult::clicked(), Some(pressed))
                } else {
                    (EventResult::changed(), None)
                }
            }
            WidgetEvent::Scroll { .. } => (EventResult::IGNORED, None),
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let style = Self::text_style(theme);
        let recipe = &theme.navigation.breadcrumb;
        let last = self.crumbs.len().saturating_sub(1);
        let rects = self.crumb_rects(bounds, theme);
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [bounds.x, bounds.y, bounds.w, bounds.h]);
            // The ellipsis stands for no crumb and binds nothing.
            for (i, r) in &rects {
                if let Some(i) = i {
                    c.bind_to_layer(layer, &id.key(i.to_string()), [r.x, r.y, r.w, r.h]);
                }
            }
        }
        let radius = recipe.radius.min(Self::height(theme) / 2.0);
        for (k, (i, r)) in rects.iter().enumerate() {
            if i.is_some() && *i == self.focused {
                c.push_to_layer(layer, focus_ring(*r, radius, theme));
            }
            let (text, color, fill) = match i {
                Some(i) if *i == last => (self.crumbs[*i].as_str(), recipe.current, None),
                Some(i) if self.pressed == Some(*i) => (
                    self.crumbs[*i].as_str(),
                    recipe.pressed,
                    Some(recipe.pressed_fill),
                ),
                Some(i) if self.hovered == Some(*i) => (
                    self.crumbs[*i].as_str(),
                    recipe.hover,
                    Some(recipe.hover_fill),
                ),
                Some(i) => (self.crumbs[*i].as_str(), recipe.rest, None),
                None => (ELLIPSIS, recipe.faint, None),
            };
            // Only a crumb cut to its room truncates (the measure of a
            // whole one, padding taken off again, may round under it).
            let text = if r.w + SLACK < Self::crumb_w(text, &style, theme) {
                TextMeasurer::truncate_to_width(text, &style, r.w - recipe.pad_x * 2.0)
            } else {
                text.to_string()
            };
            if let Some(fill) = fill.filter(|f| f.0[3] > 0.0) {
                c.push_to_layer(layer, rounded_rect(r.x, r.y, r.w, r.h, radius, fill.0));
            }
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(&text, &style, None),
                    x: r.x + recipe.pad_x,
                    y: r.y + TextMeasurer::vertical_center(&style, r.h),
                    color: color.0,
                },
            );
            if k + 1 < rects.len()
                && let Some(node) = icons::icon_at(
                    "chevron-right",
                    recipe.chevron,
                    recipe.faint.0,
                    r.x + r.w + recipe.gap,
                    r.y + (r.h - recipe.chevron) / 2.0,
                )
            {
                c.push_to_layer(layer, node);
            }
        }
    }
}
