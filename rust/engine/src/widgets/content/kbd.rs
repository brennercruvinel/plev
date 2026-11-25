//! The key cap of the hoff core kit: a shortcut in a raised pill, as its
//! menus (`⇧⌘E` beside "Export...") and its search field (`⌘ K`) draw
//! it. Two sizes, the menu's and the search's; the pill, its four-layer
//! shadow and its text are `theme.disclosure.kbd`'s.
//!
//! A cap is read, never pressed. [`Kbd::project`] declares it a
//! `Role::Label` named by its keys, described as a shortcut, for a page
//! that shows caps on their own; a control that carries a shortcut (a nav
//! link's hint) says it in its own description instead.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::tokens::CapRecipe;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::recipe::{rounded_rect, shadow_drops, shadow_insets};
use crate::widgets::widget::Rect;

/// The kit's two caps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KbdSize {
    /// A menu item's shortcut (11 px).
    #[default]
    Sm,
    /// The search field's (12 px).
    Md,
}

/// A shortcut in a cap.
#[derive(Clone, Debug)]
pub struct Kbd {
    keys: String,
    size: KbdSize,
    disabled: bool,
}

impl Kbd {
    pub fn new(keys: impl Into<String>) -> Self {
        Self {
            keys: keys.into(),
            size: KbdSize::default(),
            disabled: false,
        }
    }

    pub fn size(mut self, size: KbdSize) -> Self {
        self.size = size;
        self
    }

    /// Faded with its control (a disabled menu item's shortcut).
    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn keys(&self) -> &str {
        &self.keys
    }

    fn recipe<'t>(&self, theme: &'t Theme) -> &'t CapRecipe {
        let [sm, md] = &theme.disclosure.kbd;
        match self.size {
            KbdSize::Sm => sm,
            KbdSize::Md => md,
        }
    }

    fn style(r: &CapRecipe, theme: &Theme) -> TextStyle {
        theme.typography.run(r.text_type)
    }

    /// The cap's size: its keys measured, padded.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let r = self.recipe(theme);
        let style = Self::style(r, theme);
        let (w, _) = TextMeasurer::measure_styled(&self.keys, &style, None);
        let [pad_x, pad_y] = r.pad;
        (w + pad_x * 2.0, style.line_height + pad_y * 2.0)
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    /// The drop shadows, the pill, the inset shadows and the keys,
    /// centered in `bounds`.
    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let r = self.recipe(theme);
        let alpha = if self.disabled {
            theme.glass.disabled_alpha
        } else {
            1.0
        };
        let style = Self::style(r, theme);
        let radius = r.radius.min(bounds.h / 2.0);
        let shadow: Vec<_> = r
            .shadow
            .iter()
            .map(|s| {
                let mut s = *s;
                s.color = Color(faded(s.color, alpha));
                s
            })
            .collect();
        shadow_drops(c, layer, bounds, radius, &shadow);
        c.push_to_layer(
            layer,
            rounded_rect(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                radius,
                faded(r.fill, alpha),
            ),
        );
        shadow_insets(c, layer, bounds, radius, &shadow);
        let (w, _) = TextMeasurer::measure_styled(&self.keys, &style, None);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.keys, &style, None),
                x: bounds.x + (bounds.w - w) / 2.0,
                y: bounds.y + (bounds.h - style.line_height) / 2.0,
                color: faded(r.text, alpha),
            },
        );
    }

    /// Declares the cap as `id`: a `Role::Label` named by its keys,
    /// described as a shortcut.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Label, self.keys.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.description("keyboard shortcut");
        node
    }
}

fn faded(c: Color, alpha: f32) -> [f32; 4] {
    [c.0[0], c.0[1], c.0[2], c.0[3] * alpha]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cap_is_as_tall_as_the_kit_draws_it_and_as_wide_as_its_keys() {
        let theme = Theme::hoff_core();
        // py-0.5 around a 16 px line.
        let (w, h) = Kbd::new("⌘N").preferred_size(&theme);
        assert_eq!(h, 20.0);
        let (wide, _) = Kbd::new("⇧⌘E").preferred_size(&theme);
        assert!(wide > w);
        let (md, _) = Kbd::new("⌘N").size(KbdSize::Md).preferred_size(&theme);
        assert!(md > w, "the search's cap is the bigger one");
    }

    #[test]
    fn a_cap_draws_its_shadows_its_pill_and_its_keys() {
        let theme = Theme::hoff_core();
        let mut c = Compositor::new();
        c.begin_frame();
        Kbd::new("⌘ K").render(&mut c, Rect::new(0.0, 0.0, 40.0, 20.0), &theme);
        let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
        let shadows = nodes
            .iter()
            .filter(|n| matches!(n, SceneNode::Shadow { .. }))
            .count();
        assert_eq!(shadows, theme.disclosure.kbd[0].shadow.len());
        assert!(nodes.iter().any(|n| matches!(n, SceneNode::Text { .. })));
    }

    /// A nav link's shortcut is a cap, its hint stays text (task-104d:
    /// the gallery's sidebar adopts the cap; the inspector's statuses do
    /// not).
    #[test]
    fn a_nav_link_draws_its_shortcut_in_a_cap_and_its_hint_as_text() {
        use crate::widgets::nav::NavLink;
        let theme = Theme::hoff_core();
        let bounds = Rect::new(0.0, 0.0, 240.0, NavLink::height(&theme));
        let shadows = |link: NavLink| {
            let mut c = Compositor::new();
            c.begin_frame();
            link.render(&mut c, bounds, &theme);
            c.layer(LayerId::DEFAULT)
                .unwrap()
                .nodes()
                .iter()
                .filter(|n| matches!(n, SceneNode::Shadow { .. }))
                .count()
        };
        let cap = theme.disclosure.kbd[0].shadow.len();
        assert_eq!(shadows(NavLink::new("Accordion").shortcut("5")), cap);
        assert_eq!(shadows(NavLink::new("Accordion").hint("done")), 0);
        let wide = NavLink::new("Accordion")
            .shortcut("⌘5")
            .preferred_width(&theme);
        assert!(wide > NavLink::new("Accordion").preferred_width(&theme));
    }
}
