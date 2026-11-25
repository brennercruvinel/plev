//! The accordion of the hoff core kit: sections stacked one under the
//! other, each a [`Collapsible`] in the Accordion look (a divider above
//! all but the first, a bare header, the chevron in a box) revealing a
//! text body or the kit's thumbnail grid (two square thumbnails to a row,
//! ringed under the pointer). In [`AccordionMode::Single`] opening a section closes the
//! open one; in [`AccordionMode::Multiple`] each opens on its own, as the
//! kit's sections do.
//!
//! The height is intrinsic: each body is measured wrapped at the width
//! the accordion gets, and each section is as tall as its trigger plus
//! the revealed part of its body.
//!
//! With a [`SemId`] ([`Accordion::sem`]) each trigger is `<id>.item[<i>]`
//! and binds where it draws; [`Accordion::project`] declares the
//! accordion a `Role::Group` with the triggers and, for the open
//! sections, their regions (`<id>.item[<i>].panel`, described by the
//! body, or holding a grid's thumbnails as images,
//! `<id>.item[<i>].panel.tile[<k>]`).

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::aspect_ratio::AspectRatio;
use super::collapsible::{Collapsible, DisclosureLook};
use super::tokens::{DisclosureMotion, GridRecipe};
use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use crate::widgets::icons;
use crate::widgets::recipe::{rounded_rect, rounded_rect_stroke};
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// Whether opening a section closes the others.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AccordionMode {
    /// One section open at most.
    Single,
    /// Each section on its own (the kit).
    #[default]
    Multiple,
}

/// What a section reveals.
#[derive(Clone, Debug)]
enum Body {
    Text(String),
    /// Thumbnails, each a label and the glyph that stands for its image,
    /// at the small or the large image size.
    Grid {
        tiles: Vec<(String, &'static str)>,
        large: bool,
    },
}

/// Sections that open and close, one under the other.
#[derive(Clone, Debug, Default)]
pub struct Accordion {
    mode: AccordionMode,
    items: Vec<Collapsible>,
    bodies: Vec<Body>,
    /// The thumbnail under the pointer: `(section, thumbnail)`.
    hovered: Option<(usize, usize)>,
    motion: DisclosureMotion,
    sem: Option<SemId>,
}

impl Accordion {
    pub fn new(mode: AccordionMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }

    /// A closed section titled `title`, revealing `body`.
    pub fn item(self, title: impl Into<String>, body: impl Into<String>) -> Self {
        self.section(title.into(), Body::Text(body.into()))
    }

    /// A closed section titled `title`, revealing the kit's grid of
    /// thumbnails, each `(label, glyph)`, at the large image size or the
    /// small one.
    pub fn grid<S: Into<String>>(
        self,
        title: impl Into<String>,
        tiles: impl IntoIterator<Item = (S, &'static str)>,
        large: bool,
    ) -> Self {
        let tiles = tiles.into_iter().map(|(l, g)| (l.into(), g)).collect();
        self.section(title.into(), Body::Grid { tiles, large })
    }

    fn section(mut self, title: String, body: Body) -> Self {
        let first = self.items.is_empty();
        let mut item = Collapsible::new(title)
            .look(DisclosureLook::Accordion)
            .first(first)
            .with_motion(&self.motion);
        if let Some(id) = self.item_sem(self.items.len()) {
            item = item.sem(id);
        }
        self.items.push(item);
        self.bodies.push(body);
        self
    }

    /// Section `i` open from the start, at rest; in single mode the
    /// others close.
    pub fn opened(mut self, i: usize) -> Self {
        if i >= self.items.len() {
            return self;
        }
        let single = self.mode == AccordionMode::Single;
        self.items = std::mem::take(&mut self.items)
            .into_iter()
            .enumerate()
            .map(|(k, item)| match (k == i, single) {
                (true, _) => item.open(true),
                (false, true) => item.open(false),
                (false, false) => item,
            })
            .collect();
        self
    }

    /// Section `i` disabled: it stays as it is and reacts to nothing.
    pub fn disabled_item(mut self, i: usize) -> Self {
        if let Some(item) = self.items.get_mut(i) {
            *item = std::mem::replace(item, Collapsible::new("")).disabled(true);
        }
        self
    }

    /// The motion of a theme (`theme.disclosure.motion`), for the
    /// sections there are and the ones to come.
    pub fn with_motion(mut self, motion: &DisclosureMotion) -> Self {
        self.motion = *motion;
        self.items = std::mem::take(&mut self.items)
            .into_iter()
            .map(|item| item.with_motion(motion))
            .collect();
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        let ids: Vec<_> = (0..self.items.len()).map(|i| self.item_sem(i)).collect();
        self.items = std::mem::take(&mut self.items)
            .into_iter()
            .zip(ids)
            .map(|(item, id)| match id {
                Some(id) => item.sem(id),
                None => item,
            })
            .collect();
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// `<id>.item[<i>]`.
    fn item_sem(&self, i: usize) -> Option<SemId> {
        let family = self.sem.as_ref()?.child("item").ok()?;
        Some(family.key(i.to_string()))
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> &[Collapsible] {
        &self.items
    }

    /// The section whose trigger is `id`.
    pub fn position(&self, id: &SemId) -> Option<usize> {
        self.items.iter().position(|item| item.sem_id() == Some(id))
    }

    pub fn is_open(&self, i: usize) -> bool {
        self.items.get(i).is_some_and(Collapsible::is_open)
    }

    pub fn is_disabled(&self, i: usize) -> bool {
        self.items.get(i).is_some_and(Collapsible::is_disabled)
    }

    /// Flips section `i`; in single mode, opening it closes the others.
    /// `false` when it is disabled or not there.
    pub fn toggle(&mut self, i: usize) -> bool {
        let Some(item) = self.items.get_mut(i) else {
            return false;
        };
        if !item.toggle() {
            return false;
        }
        self.close_others(i);
        true
    }

    /// In single mode, the sections other than an open `i` close.
    fn close_others(&mut self, i: usize) {
        if self.mode != AccordionMode::Single || !self.is_open(i) {
            return;
        }
        for (k, other) in self.items.iter_mut().enumerate() {
            if k != i {
                other.set_open(false);
            }
        }
    }

    /// The keyboard focus on the trigger of section `i`, or on none.
    pub fn set_focused(&mut self, i: Option<usize>) {
        for (k, item) in self.items.iter_mut().enumerate() {
            item.set_focused(i == Some(k));
        }
    }

    /// Every section's motion, merged.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.items
            .iter_mut()
            .fold(Wake::idle(), |w, item| w.merge(item.wake(dt)))
    }

    fn body_style(theme: &Theme) -> TextStyle {
        let r = &theme.disclosure.accordion;
        theme.typography.run(r.body_type)
    }

    /// The width a body wraps at, in a section `w` wide.
    fn body_width(w: f32, theme: &Theme) -> f32 {
        let [_, right, _, left] = theme.disclosure.accordion.content_pad;
        (w - left - right).max(0.0)
    }

    /// Body `i` measured, wrapped in a section `w` wide.
    fn body_height(&self, i: usize, w: f32, theme: &Theme) -> f32 {
        match self.bodies.get(i) {
            Some(Body::Text(body)) if !body.is_empty() => {
                let style = Self::body_style(theme);
                TextMeasurer::measure_styled(body, &style, Some(Self::body_width(w, theme))).1
            }
            Some(Body::Grid { tiles, .. }) => {
                let at = Rect::new(0.0, 0.0, Self::body_width(w, theme), 0.0);
                Self::tile_rects(tiles.len(), at, theme)
                    .last()
                    .map_or(0.0, Rect::bottom)
            }
            _ => 0.0,
        }
    }

    /// `n` square thumbnails from the top of `at`, the grid's columns
    /// sharing its width.
    fn tile_rects(n: usize, at: Rect, theme: &Theme) -> Vec<Rect> {
        let Some(grid) = theme.disclosure.accordion.grid.as_ref() else {
            return Vec::new();
        };
        let cols = grid.columns.max(1);
        let edge = ((at.w - grid.gap * (cols as f32 - 1.0)) / cols as f32).max(0.0);
        let h = AspectRatio::SQUARE.height(edge);
        (0..n)
            .map(|k| {
                let (col, row) = ((k % cols) as f32, (k / cols) as f32);
                Rect::new(
                    at.x + col * (edge + grid.gap),
                    at.y + row * (h + grid.gap),
                    edge,
                    h,
                )
            })
            .collect()
    }

    /// The thumbnail under `(x, y)`, among the sections that show some.
    fn tile_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<(usize, usize)> {
        let rects = self.item_rects(bounds, theme);
        self.items
            .iter()
            .zip(rects)
            .enumerate()
            .find_map(|(i, (item, rect))| {
                let Some(Body::Grid { tiles, .. }) = self.bodies.get(i) else {
                    return None;
                };
                if !item.is_open() || !item.shown_rect(rect, theme).contains(x, y) {
                    return None;
                }
                let at = item.content_rect(rect, self.body_height(i, rect.w, theme), theme);
                let k = Self::tile_rects(tiles.len(), at, theme)
                    .iter()
                    .position(|t| t.contains(x, y))?;
                Some((i, k))
            })
    }

    /// The height now at width `w`.
    pub fn height(&self, w: f32, theme: &Theme) -> f32 {
        (0..self.items.len())
            .map(|i| self.items[i].height(w, self.body_height(i, w, theme), theme))
            .sum()
    }

    /// Each section's rect in `bounds`, top to bottom.
    pub fn item_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        let mut y = bounds.y;
        (0..self.items.len())
            .map(|i| {
                let h = self.items[i].height(bounds.w, self.body_height(i, bounds.w, theme), theme);
                let r = Rect::new(bounds.x, y, bounds.w, h);
                y += h;
                r
            })
            .collect()
    }

    /// Pointer events to every section; a section a click opened closes
    /// the others in single mode.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let rects = self.item_rects(bounds, theme);
        let mut result = EventResult::IGNORED;
        if let WidgetEvent::MouseMove { x, y } = *event {
            let under = self.tile_at(x, y, bounds, theme);
            if under != self.hovered {
                self.hovered = under;
                result = EventResult::changed();
            }
        }
        let mut flipped = None;
        for (i, (item, rect)) in self.items.iter_mut().zip(rects).enumerate() {
            let r = item.handle_event(event, rect, theme);
            if r.clicked {
                flipped = Some(i);
            }
            result = result.merge(r);
        }
        if let Some(i) = flipped {
            self.close_others(i);
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let style = Self::body_style(theme);
        let color = theme.disclosure.accordion.body;
        for (i, (item, rect)) in self
            .items
            .iter()
            .zip(self.item_rects(bounds, theme))
            .enumerate()
        {
            let alpha = if item.is_disabled() {
                theme.glass.disabled_alpha
            } else {
                1.0
            };
            let body_h = self.body_height(i, bounds.w, theme);
            let hovered = self.hovered.filter(|(s, _)| *s == i).map(|(_, k)| k);
            item.render(c, rect, body_h, theme, |c, at| match &self.bodies[i] {
                Body::Text(body) if !body.is_empty() => c.push(SceneNode::Text {
                    key: TextNodeKey::from_style(body, &style, Some(at.w)),
                    x: at.x,
                    y: at.y,
                    color: [color.0[0], color.0[1], color.0[2], color.0[3] * alpha],
                }),
                Body::Text(_) => {}
                Body::Grid { tiles, large } => {
                    let panel = item.sem_id().and_then(|id| id.child("panel").ok());
                    let family = panel.and_then(|p| p.child("tile").ok());
                    let rects = Self::tile_rects(tiles.len(), at, theme);
                    for (k, ((_, glyph), r)) in tiles.iter().zip(rects).enumerate() {
                        if let Some(family) = &family {
                            c.bind(&family.key(k.to_string()), [r.x, r.y, r.w, r.h]);
                        }
                        if let Some(grid) = theme.disclosure.accordion.grid.as_ref() {
                            paint_tile(c, r, glyph, *large, hovered == Some(k), alpha, grid);
                        }
                    }
                }
            });
        }
    }

    /// Declares the accordion as `id`, a `Role::Group` named `label`, and
    /// in it each section's trigger as [`Collapsible::project`] does and,
    /// for an open one, its region described by its body. It only reads
    /// the widget; the app runs `toggle_expanded` ([`Accordion::toggle`]).
    /// Returns the group's node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        for (item, body) in self.items.iter().zip(&self.bodies) {
            let Some(at) = item.sem_id() else {
                continue;
            };
            item.project(b, at, Some(id));
            let Some(mut panel) = item.project_panel(b, at, Some(id)) else {
                continue;
            };
            match body {
                Body::Text(text) if !text.is_empty() => {
                    panel.description(text.as_str());
                }
                Body::Text(_) => {}
                Body::Grid { tiles, .. } => {
                    let Ok(panel) = at.child("panel") else {
                        continue;
                    };
                    let Ok(family) = panel.child("tile") else {
                        continue;
                    };
                    for (k, (label, _)) in tiles.iter().enumerate() {
                        b.node(family.key(k.to_string()), Role::Image, label.as_str())
                            .parent(&panel);
                    }
                }
            }
        }
        let mut node = b.node(id, Role::Group, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node
    }
}

/// One thumbnail: the wash, the glyph where the image goes and, under the
/// pointer, the two inset rings (the inner first, the outer on top).
fn paint_tile(
    c: &mut Compositor,
    r: Rect,
    glyph: &str,
    large: bool,
    hovered: bool,
    alpha: f32,
    grid: &GridRecipe,
) {
    let fade = |color: crate::styling::color::Color| {
        [color.0[0], color.0[1], color.0[2], color.0[3] * alpha]
    };
    let radius = grid.radius.min(r.w / 2.0);
    c.push(rounded_rect(r.x, r.y, r.w, r.h, radius, fade(grid.fill)));
    let image = if large { grid.image_large } else { grid.image };
    let size = image.min(r.w - grid.pad * 2.0).max(0.0);
    if size > 0.0
        && let Some(node) = icons::icon_at(
            glyph,
            size,
            fade(grid.glyph_color),
            r.x + (r.w - size) / 2.0,
            r.y + (r.h - size) / 2.0,
        )
    {
        c.push(node);
    }
    if hovered {
        for (width, color) in [grid.ring_inner, grid.ring_outer] {
            c.push(rounded_rect_stroke(
                r.x,
                r.y,
                r.w,
                r.h,
                radius,
                fade(color),
                width,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kit() -> Accordion {
        Accordion::new(AccordionMode::Multiple)
            .grid(
                "3D Objects",
                [
                    ("Cube", "square"),
                    ("Sphere", "circle"),
                    ("Stack", "layers"),
                ],
                true,
            )
            .opened(0)
            .sem(SemId::parse("assets").unwrap())
    }

    /// Two square thumbnails to a row: three make two rows, each as tall
    /// as the column is wide.
    #[test]
    fn the_grid_is_as_tall_as_its_square_rows() {
        let theme = Theme::hoff_core();
        let grid = theme.disclosure.accordion.grid.unwrap();
        let w = 240.0;
        let edge = (Accordion::body_width(w, &theme) - grid.gap) / 2.0;
        assert_eq!(kit().body_height(0, w, &theme), edge * 2.0 + grid.gap);
        let narrow = kit().body_height(0, 160.0, &theme);
        assert!(
            narrow < edge * 2.0 + grid.gap,
            "the squares follow the width"
        );
    }

    /// The pointer over a thumbnail rings it; the open section's region
    /// holds the thumbnails as images.
    #[test]
    fn a_thumbnail_rings_under_the_pointer_and_projects_as_an_image() {
        let theme = Theme::hoff_core();
        let mut a = kit();
        let bounds = Rect::new(0.0, 0.0, 240.0, a.height(240.0, &theme));
        let rect = a.item_rects(bounds, &theme)[0];
        let at = a.items[0].content_rect(rect, a.body_height(0, 240.0, &theme), &theme);
        let (x, y) = Accordion::tile_rects(3, at, &theme)[1].center();
        assert!(
            a.handle_event(&WidgetEvent::MouseMove { x, y }, bounds, &theme)
                .changed
        );
        assert_eq!(a.hovered, Some((0, 1)));
        let id = SemId::parse("assets").unwrap();
        let mut b = SnapshotBuilder::new();
        a.project(&mut b, &id, "Assets", None);
        let seen = b.finish().unwrap();
        let tile = SemId::parse("assets.item[0].panel.tile[1]").unwrap();
        let node = seen.get(&tile).expect("a thumbnail");
        assert_eq!((node.role, node.label.as_str()), (Role::Image, "Sphere"));
    }
}
