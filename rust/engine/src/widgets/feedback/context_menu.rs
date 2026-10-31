//! The menu of the hoff core kit (`ProfileMenu`, `FileMenu`,
//! `FiltersMenu`, the header's `UserDropdown`): a surface with rows (an
//! optional leading icon or check, a label, an optional shortcut in a key
//! cap at the end, highlighted under the pointer or the keys) and
//! separators. Every value is the recipe of its [`MenuStyle`] in
//! `theme.overlay`: the kit's on core, and every other theme's own menu
//! (the popover glass, the key-light) as it was. Width fits the widest
//! row and never drops under the menu's `min_w`.
//!
//! With a [`SemId`] ([`ContextMenu::sem`]) the render binds the menu and
//! each item to the rects it draws and hit-tests them with, and
//! [`ContextMenu::project`] declares the menu in a semantic projection.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::{Intent, Theme};
use crate::widgets::content::kbd::Kbd;
use crate::widgets::icons;
use crate::widgets::overlay::tokens::{MenuStyle, MenuTokens};
use accord::{NodeBuilder, Policy, Role, SemId, SnapshotBuilder};

use crate::widgets::recipe::{inset_keylight, rounded_rect, rounded_rect_stroke, shadow_node};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, intent_fill, with_alpha};

/// One row of a [`ContextMenu`].
#[derive(Clone, Debug)]
pub enum MenuEntry {
    Item {
        /// Opaque id reported on click.
        id: u64,
        label: String,
        /// Optional leading icon ([`crate::widgets::icons`] name).
        icon: Option<&'static str>,
        disabled: bool,
        /// Colors the label (Destructive = red row, etc.).
        intent: Intent,
        /// The keys that run it, shown in a cap at the row's end.
        shortcut: Option<String>,
        /// A choice the row shows checked or not (a filter); `None`, a
        /// row with no check.
        checked: Option<bool>,
    },
    Separator,
}

/// The check a checkable row draws in its icon slot.
const CHECK: &str = "check";

impl MenuEntry {
    pub fn item(id: u64, label: impl Into<String>) -> Self {
        Self::Item {
            id,
            label: label.into(),
            icon: None,
            disabled: false,
            intent: Intent::Neutral,
            shortcut: None,
            checked: None,
        }
    }

    pub fn icon(mut self, name: &'static str) -> Self {
        if let Self::Item { icon, .. } = &mut self {
            *icon = Some(name);
        }
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        if let Self::Item { disabled, .. } = &mut self {
            *disabled = value;
        }
        self
    }

    pub fn intent(mut self, value: Intent) -> Self {
        if let Self::Item { intent, .. } = &mut self {
            *intent = value;
        }
        self
    }

    pub fn shortcut(mut self, keys: impl Into<String>) -> Self {
        if let Self::Item { shortcut, .. } = &mut self {
            *shortcut = Some(keys.into());
        }
        self
    }

    pub fn checked(mut self, on: bool) -> Self {
        if let Self::Item { checked, .. } = &mut self {
            *checked = Some(on);
        }
        self
    }
}

/// Context menu rendered at an arbitrary screen position (overlay layer).
/// Width fits the widest label; rows hover; disabled rows are inert.
#[derive(Clone, Debug)]
pub struct ContextMenu {
    pub entries: Vec<MenuEntry>,
    pub style: MenuStyle,
    hovered: Option<usize>,
    sem: Option<SemId>,
}

impl ContextMenu {
    pub fn new(entries: Vec<MenuEntry>) -> Self {
        Self {
            entries,
            style: MenuStyle::default(),
            hovered: None,
            sem: None,
        }
    }

    /// Drawn as the kit's `style` menu.
    pub fn style(mut self, style: MenuStyle) -> Self {
        self.style = style;
        self
    }

    /// The recipe it draws with.
    pub fn tokens<'t>(&self, theme: &'t Theme) -> &'t MenuTokens {
        theme.overlay.menu_for(self.style)
    }

    /// The logical identity the render binds the menu to
    /// (`Compositor::bind_to_layer`); each item binds as `<id>[<item id>]`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Declares this menu as `id` in a projection: a `Role::Menu` named
    /// `label` with a `close` action, and one `Role::MenuItem` child
    /// `<id>[<item id>]` per item (separators are not items), named by its
    /// label, enabled unless disabled, described by its shortcut, with a
    /// `checked` state when it has a check, and an `activate` whose policy
    /// is destructive for a destructive item. The hovered row is never
    /// projected. It only reads the widget; the app owns it and runs the
    /// actions (what an item does, and closing the menu). Pass the id
    /// given to [`ContextMenu::sem`] so the nodes and the bindings name the
    /// same controls. Returns the menu's node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        for entry in &self.entries {
            if let MenuEntry::Item {
                id: item,
                label,
                disabled,
                intent,
                shortcut,
                checked,
                ..
            } = entry
            {
                let mut node = b.node(id.key(item.to_string()), Role::MenuItem, label.as_str());
                node.parent(id).enabled(!disabled);
                if let Some(keys) = shortcut {
                    node.description(keys.as_str());
                }
                if let Some(on) = checked {
                    node.state("checked", *on);
                }
                node.action("activate").policy(Policy {
                    destructive: *intent == Intent::Destructive,
                    ..Policy::default()
                });
            }
        }
        let mut node = b.node(id, Role::Menu, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.action("close");
        node
    }

    pub fn hovered(&self) -> Option<usize> {
        self.hovered
    }

    /// The item `id` highlighted as the pointer highlights it, or none:
    /// what the owning view calls when the arrow keys or a screen reader
    /// move the focus to an item, so the one highlight follows both. A
    /// separator or an unknown id highlights nothing.
    pub fn set_highlighted(&mut self, id: Option<u64>) {
        self.hovered = id.and_then(|want| {
            self.entries
                .iter()
                .position(|e| matches!(e, MenuEntry::Item { id, .. } if *id == want))
        });
    }

    /// A row's shortcut as a key cap, faded with a disabled row.
    fn cap(shortcut: &Option<String>, disabled: bool) -> Option<Kbd> {
        shortcut
            .as_ref()
            .map(|k| Kbd::new(k.as_str()).disabled(disabled))
    }

    /// Menu size from real text measurement: the widest row (icon or
    /// check slot, label, shortcut, item padding on both sides, menu
    /// padding on both sides), never narrower than the menu width token.
    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let m = self.tokens(theme);
        let mut w: f32 = m.min_w;
        let mut h = m.pad * 2.0;
        for entry in &self.entries {
            match entry {
                MenuEntry::Item {
                    label,
                    icon,
                    shortcut,
                    checked,
                    ..
                } => {
                    let (tw, _) = TextMeasurer::measure_styled(label, &m.label, None);
                    let lead = icon.is_some() || checked.is_some();
                    let icon_w = if lead { m.icon + m.gap } else { 0.0 };
                    let cap_w = Self::cap(shortcut, false)
                        .map_or(0.0, |k| k.preferred_size(theme).0 + m.gap);
                    w = w.max(tw + icon_w + cap_w + (m.pad + m.item_pad_x) * 2.0 + m.gap);
                    h += m.item_h;
                }
                MenuEntry::Separator => h += m.sep_h,
            }
        }
        (w.ceil(), h)
    }

    /// Row rects (separators included, in entry order) for a menu whose
    /// top-left is at (x, y).
    pub fn entry_rects(&self, x: f32, y: f32, theme: &Theme) -> Vec<Rect> {
        let m = self.tokens(theme);
        let (w, _) = self.size(theme);
        let mut rects = Vec::with_capacity(self.entries.len());
        let mut cy = y + m.pad;
        for entry in &self.entries {
            let h = match entry {
                MenuEntry::Item { .. } => m.item_h,
                MenuEntry::Separator => m.sep_h,
            };
            rects.push(Rect::new(x, cy, w, h));
            cy += h;
        }
        rects
    }

    fn item_at(&self, px: f32, py: f32, x: f32, y: f32, theme: &Theme) -> Option<usize> {
        self.entry_rects(x, y, theme)
            .iter()
            .enumerate()
            .find(|(i, r)| matches!(self.entries[*i], MenuEntry::Item { .. }) && r.contains(px, py))
            .map(|(i, _)| i)
    }

    /// Returns the clicked item id, if any. `(x, y)` is the menu origin.
    pub fn clicked_id(&self, index: usize) -> Option<u64> {
        match self.entries.get(index) {
            Some(MenuEntry::Item { id, .. }) => Some(*id),
            _ => None,
        }
    }

    /// Handle an event for a menu anchored at `(x, y)`. On item
    /// activation, `EventResult::clicked` is set and the item id is
    /// returned. Rows are laid out from the theme, so the event path
    /// takes the same `theme` the render path draws with.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        x: f32,
        y: f32,
        theme: &Theme,
    ) -> (EventResult, Option<u64>) {
        match *event {
            WidgetEvent::MouseMove { x: px, y: py } => {
                let hit = self.item_at(px, py, x, y, theme).filter(|&i| {
                    !matches!(self.entries[i], MenuEntry::Item { disabled: true, .. })
                });
                if hit != self.hovered {
                    self.hovered = hit;
                    (EventResult::changed(), None)
                } else {
                    (EventResult::IGNORED, None)
                }
            }
            WidgetEvent::MouseDown { x: px, y: py } => {
                if let Some(i) = self.item_at(px, py, x, y, theme)
                    && let MenuEntry::Item { id, disabled, .. } = &self.entries[i]
                {
                    if *disabled {
                        // Swallow the click but do nothing.
                        return (
                            EventResult {
                                handled: true,
                                ..EventResult::IGNORED
                            },
                            None,
                        );
                    }
                    return (EventResult::clicked(), Some(*id));
                }
                (EventResult::IGNORED, None)
            }
            _ => (EventResult::IGNORED, None),
        }
    }

    pub fn render(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        x: f32,
        y: f32,
    ) {
        let m = self.tokens(theme);
        let (w, h) = self.size(theme);
        let body = Rect::new(x, y, w, h);
        let rows = self.entry_rects(x, y, theme);
        if let Some(id) = &self.sem {
            compositor.bind_to_layer(layer, id, [x, y, w, h]);
            for (entry, r) in self.entries.iter().zip(&rows) {
                if let MenuEntry::Item { id: item, .. } = entry {
                    compositor.bind_to_layer(
                        layer,
                        &id.key(item.to_string()),
                        [r.x, r.y, r.w, r.h],
                    );
                }
            }
        }

        // The shadows, then the surface and its border; row icons pushed
        // later stack on top (push order is preserved across types).
        for spec in &m.shadow {
            if let Some(shadow) = shadow_node(body, m.radius, spec) {
                compositor.push_to_layer(layer, shadow);
            }
        }
        compositor.push_to_layer(layer, rounded_rect(x, y, w, h, m.radius, m.fill.0));
        if m.border.0[3] > 0.0 && m.border_w > 0.0 {
            compositor.push_to_layer(
                layer,
                rounded_rect_stroke(x, y, w, h, m.radius, m.border.0, m.border_w),
            );
        }
        if m.keylight {
            inset_keylight(compositor, layer, body, m.radius, theme);
        }

        let style = &m.label;
        for (i, (entry, rect)) in self.entries.iter().zip(rows).enumerate() {
            match entry {
                MenuEntry::Separator => {
                    let inset = if m.sep_full { 0.0 } else { m.pad };
                    let line = m.border_w.max(1.0);
                    compositor.push_to_layer(
                        layer,
                        SceneNode::Rect {
                            x: rect.x + inset,
                            y: rect.y + (rect.h - line) / 2.0,
                            w: rect.w - inset * 2.0,
                            h: line,
                            color: m.sep_color.0,
                        },
                    );
                }
                MenuEntry::Item {
                    label,
                    icon,
                    disabled,
                    intent,
                    shortcut,
                    checked,
                    ..
                } => {
                    let alpha = if *disabled { m.disabled_alpha } else { 1.0 };
                    let hovered = self.hovered == Some(i);
                    if hovered {
                        compositor.push_to_layer(
                            layer,
                            rounded_rect(
                                rect.x + m.pad,
                                rect.y,
                                rect.w - m.pad * 2.0,
                                rect.h,
                                m.item_radius,
                                m.hover_fill.0,
                            ),
                        );
                    }
                    // The label and the icon, each at rest or highlighted; an
                    // intent colors both.
                    let (fg, ink) = match intent {
                        Intent::Neutral => {
                            let (t, i) = if hovered {
                                (m.text_hover, m.icon_hover)
                            } else {
                                (m.text, m.icon_color)
                            };
                            (with_alpha(t, t.0[3] * alpha), with_alpha(i, i.0[3] * alpha))
                        }
                        other => {
                            let c = intent_fill(theme, *other);
                            let c = [c[0], c[1], c[2], c[3] * alpha];
                            (c, c)
                        }
                    };
                    let mut tx = rect.x + m.pad + m.item_pad_x;
                    // The icon, or the check (its slot kept unchecked).
                    let lead = match (icon, checked) {
                        (Some(name), _) => Some(Some(*name)),
                        (None, Some(on)) => Some(on.then_some(CHECK)),
                        (None, None) => None,
                    };
                    if let Some(name) = lead {
                        let iy = rect.y + (rect.h - m.icon) / 2.0;
                        if let Some(node) =
                            name.and_then(|n| icons::icon_at(n, m.icon, ink, tx, iy))
                        {
                            compositor.push_to_layer(layer, node);
                        }
                        tx += m.icon + m.gap;
                    }
                    if let Some(cap) = Self::cap(shortcut, *disabled) {
                        let (cw, ch) = cap.preferred_size(theme);
                        let cx = rect.right() - m.pad - m.item_pad_x - cw;
                        let cy = rect.y + (rect.h - ch) / 2.0;
                        cap.render_to_layer(compositor, layer, Rect::new(cx, cy, cw, ch), theme);
                    }
                    compositor.push_to_layer(
                        layer,
                        SceneNode::Text {
                            key: TextNodeKey::from_style(label, style, None),
                            x: tx,
                            y: rect.y + TextMeasurer::vertical_center(style, rect.h),
                            color: fg,
                        },
                    );
                }
            }
        }
    }
}
