//! Select: a control (the value and a chevron) that opens a floating
//! options panel on an overlay layer. Keyboard focus
//! ([`Select::set_focused`]) rings the closed control with the accent
//! ring.
//!
//! Every value is the theme's select recipe (`theme.form.select`,
//! `form/tokens/select.rs`): in the core the kit's Select, a flat
//! control and panel with the selected row filled; in every other theme
//! the glass pill and popover it drew before the kit, the selected row
//! marked by a dot. The panel never grows past the recipe's `max_h`.
//! With more options than
//! fit, the rows scroll inside the panel (`select/panel.rs`): the wheel
//! over the panel scrolls it, the arrow keys ([`Select::highlight_next`],
//! [`Select::highlight_prev`]) move the highlight and keep it in view,
//! Enter picks it ([`Select::pick_highlighted`]), and opening the panel
//! scrolls the selected option into view.
//!
//! With a [`SemId`] ([`Select::sem`]) the render binds the closed control
//! and, while open, each option row it draws (the window the scroll
//! shows) to the rects they are hit-tested with, and [`Select::project`]
//! declares the select in a semantic projection; the app still owns the
//! selection.

pub(crate) mod list;
mod panel;

use crate::control::input::scroll::ScrollState;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::color::Color;
use crate::styling::theme::ShadowSpec;
use crate::styling::theme::Theme;
use crate::widgets::form::tokens::SelectLook;
use crate::widgets::icons;
use crate::widgets::recipe::{rounded_rect, rounded_rect_stroke, shadow_drops, shadow_insets};
use accord::{Constraints, NodeBuilder, ParamSpec, Role, SemId, SnapshotBuilder, ValueKind};

use crate::widgets::recipe::focus_ring;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, with_alpha};

/// Dropdown select. The closed control renders in normal flow; while
/// open, render the dropdown via [`render_dropdown`](Select::render_dropdown)
/// onto an overlay layer (it draws below the control bounds).
#[derive(Clone, Debug)]
pub struct Select {
    pub options: Vec<String>,
    pub selected: usize,
    pub disabled: bool,
    open: bool,
    hovered: bool,
    /// The option the pointer is over or the arrow keys moved to.
    highlighted: Option<usize>,
    focused: bool,
    /// How far the option rows are scrolled inside the panel.
    scroll: ScrollState,
    /// The panel opened and has not been laid out since: the selected
    /// option is scrolled into view the first time the geometry is read.
    reveal: bool,
    /// The view moved the highlight ([`Select::set_highlighted`]) and the
    /// panel has not been laid out since: the highlighted option is
    /// scrolled into view the first time the geometry is read.
    reveal_highlight: bool,
    /// Which control it draws, and whether small (`isSmall`).
    look: SelectLook,
    small: bool,
    /// A label before the value (`X Post`), and a Lucide icon before both.
    label: Option<String>,
    icon: Option<&'static str>,
    sem: Option<SemId>,
}

impl Select {
    pub fn new(options: impl IntoIterator<Item = impl Into<String>>, selected: usize) -> Self {
        let options: Vec<String> = options.into_iter().map(Into::into).collect();
        let selected = selected.min(options.len().saturating_sub(1));
        Self {
            options,
            selected,
            disabled: false,
            open: false,
            hovered: false,
            highlighted: None,
            focused: false,
            scroll: ScrollState::new(),
            reveal: false,
            reveal_highlight: false,
            look: SelectLook::Filled,
            small: false,
            label: None,
            icon: None,
            sem: None,
        }
    }

    /// Draw `look`'s control; the inline one is small.
    pub fn look(mut self, look: SelectLook) -> Self {
        self.look = look;
        self
    }

    /// The small control, 12px text in it and in its options.
    pub fn small(mut self, small: bool) -> Self {
        self.small = small;
        self
    }

    /// A label before the value, which then reads as its property.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// A Lucide icon at the start.
    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    fn is_small(&self) -> bool {
        self.small || self.look == SelectLook::Inline
    }

    /// The text of the control and its options in `theme`.
    pub(super) fn text_style(&self, theme: &Theme) -> TextStyle {
        let r = &theme.form.select;
        if self.is_small() {
            r.small_text.clone()
        } else {
            r.text.clone()
        }
    }

    /// The control's height in `theme`, small or not.
    pub fn height_in(&self, theme: &Theme) -> f32 {
        let r = &theme.form.select;
        if self.is_small() {
            r.small_height
        } else {
            r.height
        }
    }

    /// The control's width that shows its widest option whole: the
    /// padding, the icon and the label when it has them, the option and
    /// the chevron, with their gaps.
    pub fn width_in(&self, theme: &Theme) -> f32 {
        let r = &theme.form.select;
        let style = self.text_style(theme);
        let measure = |t: &str| TextMeasurer::measure_styled(t, &style, None).0.ceil();
        let widest = self.options.iter().map(|o| measure(o)).fold(0.0, f32::max);
        let icon = self.icon.map_or(0.0, |_| r.icon + r.gap);
        let label = self.label.as_deref().map_or(0.0, |l| measure(l) + r.gap);
        r.pad_start + icon + label + widest + r.gap + r.chevron + r.pad_end
    }

    /// Between the control and its open panel.
    pub(super) fn panel_gap(&self, theme: &Theme) -> f32 {
        let p = &theme.form.select.panel;
        if self.look == SelectLook::Inline {
            p.inline_gap
        } else {
            p.gap
        }
    }

    /// The logical identity the render binds the control to
    /// (`Compositor::bind`); the open options bind as `<id>[<index>]`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Declares this select as `id` in a projection: a `Role::Select`
    /// named `label`, its `selected` option label (empty when it has no
    /// option), whether its panel is `expanded`, editable with the option
    /// labels as the accepted choices, enabled unless disabled, and the
    /// actions `select` (a text, one of the options), `open` and `close`.
    /// While open, every option is a `Role::Option` child `<id>[<index>]`
    /// with a `select` of its own, the rows scrolled out of the panel
    /// included; the render binds only the rows it draws. Each action
    /// reads the state it changes (`selected` or `expanded`). The
    /// highlighted option and the scroll are never projected. It only
    /// reads the widget; the app owns it and runs the actions
    /// ([`Select::select_label`], [`Select::select_index`],
    /// [`Select::open`], [`Select::close`]). Pass the id given to
    /// [`Select::sem`] so the nodes and the bindings name the same
    /// controls. Returns the select's node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let selected = format!("{id}:selected");
        let expanded = format!("{id}:expanded");
        if self.open {
            for (i, option) in self.options.iter().enumerate() {
                let mut row = b.node(id.key(i.to_string()), Role::Option, option.as_str());
                row.parent(id).enabled(!self.disabled);
                row.action("select").reads(selected.as_str());
            }
        }
        let mut choices: Vec<String> = Vec::with_capacity(self.options.len());
        for option in &self.options {
            if !choices.contains(option) {
                choices.push(option.clone());
            }
        }
        let mut node = b.node(id, Role::Select, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("selected", self.selected_label().unwrap_or_default())
            .state("expanded", self.open)
            .constraints(Constraints {
                editable: true,
                options: Some(choices),
                ..Constraints::default()
            })
            .enabled(!self.disabled);
        node.action("select")
            .param(ParamSpec {
                kind: ValueKind::Text,
                constrained: true,
            })
            .reads(selected.as_str());
        node.action("open").reads(expanded.as_str());
        node.action("close").reads(expanded.as_str());
        node
    }

    /// Opens the options panel, like a click on the control, scrolled so
    /// the selected option shows. A disabled select stays closed.
    pub fn open(&mut self) {
        if !self.disabled {
            self.open = true;
            self.reveal = true;
        }
    }

    /// Selects the option at `index` and closes the panel, like a click on
    /// that option. `false` when there is no such option.
    pub fn select_index(&mut self, index: usize) -> bool {
        if index >= self.options.len() {
            return false;
        }
        self.selected = index;
        self.close();
        true
    }

    /// Selects the first option labeled `label` and closes the panel.
    /// `false` when no option has that label.
    pub fn select_label(&mut self, label: &str) -> bool {
        match self.options.iter().position(|o| o == label) {
            Some(index) => self.select_index(index),
            None => false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Keyboard focus, driven by the owning view (plev has no global focus
    /// chain). Disabled selects refuse focus.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused && !self.disabled;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn close(&mut self) {
        self.open = false;
        self.highlighted = None;
    }

    pub fn selected_label(&self) -> Option<&str> {
        self.options.get(self.selected).map(String::as_str)
    }

    /// The option the pointer is over or the arrow keys moved to, while
    /// the panel is open.
    pub fn highlighted(&self) -> Option<usize> {
        self.highlighted
    }

    /// Highlights the option at `index`, or none, as the arrow keys do: the
    /// owning view calls it when a person's keys or a screen reader move
    /// the focus to an open option, and the panel scrolls the least that
    /// shows it the next time it is laid out. A closed panel or an index
    /// past the options highlights nothing.
    pub fn set_highlighted(&mut self, index: Option<usize>) {
        self.highlighted = index.filter(|i| self.open && *i < self.options.len());
        self.reveal_highlight = self.highlighted.is_some();
    }

    /// The options panel is laid out from the theme, so the event path
    /// takes the same `theme` the render path draws with. The wheel over
    /// the open panel scrolls its rows and is consumed there, even when
    /// they are already at the end.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        if self.disabled {
            if self.hovered || self.open {
                self.hovered = false;
                self.close();
                return EventResult::changed();
            }
            return EventResult::IGNORED;
        }
        if self.open {
            self.settle(theme);
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let mut result = EventResult::IGNORED;
                let inside = bounds.contains(x, y);
                if inside != self.hovered {
                    self.hovered = inside;
                    result = EventResult::changed();
                }
                if self.open {
                    let hit = self.option_at(x, y, bounds, theme);
                    if hit != self.highlighted {
                        self.highlighted = hit;
                        result = result.merge(EventResult::changed());
                    }
                }
                result
            }
            WidgetEvent::MouseDown { x, y } => {
                if self.open {
                    if let Some(i) = self.option_at(x, y, bounds, theme) {
                        let changed = i != self.selected;
                        self.selected = i;
                        self.close();
                        return if changed {
                            EventResult::clicked()
                        } else {
                            EventResult::changed()
                        };
                    }
                    // Click anywhere else closes the dropdown. Re-clicking
                    // the control itself just toggles closed.
                    self.close();
                    return EventResult::changed();
                }
                if bounds.contains(x, y) {
                    self.open();
                    self.settle(theme);
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::Scroll { x, y, delta } if self.open => {
                self.scroll_panel(x, y, delta, bounds, theme)
            }
            _ => EventResult::IGNORED,
        }
    }

    /// The default control's height in `theme`: the recipe's.
    pub fn height(theme: &Theme) -> f32 {
        theme.form.select.height
    }

    /// Render the closed control: an icon, a label and the value, cut with
    /// an ellipsis before the chevron.
    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        if let Some(id) = &self.sem {
            compositor.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let r = &theme.form.select;
        let alpha = if self.disabled { r.disabled_alpha } else { 1.0 };
        let fade = |color: Color| with_alpha(color, color.0[3] * alpha);
        let radius = if self.look == SelectLook::Inline {
            r.inline_radius
        } else {
            r.radius
        }
        .min(bounds.h / 2.0);
        if self.focused {
            compositor.push(focus_ring(bounds, radius, theme));
        }
        let looks = r.surfaces(self.look);
        let surface = if self.open {
            &looks.open
        } else if self.hovered {
            &looks.hover
        } else if self.focused {
            &looks.focus
        } else {
            &looks.rest
        };
        let shadows: Vec<_> = surface
            .shadow
            .iter()
            .map(|s| ShadowSpec {
                color: Color(fade(s.color)),
                ..*s
            })
            .collect();
        let layer = LayerId::DEFAULT;
        shadow_drops(compositor, layer, bounds, radius, &shadows);
        let Rect { x, y, w, h } = bounds;
        compositor.push(rounded_rect(x, y, w, h, radius, fade(surface.fill)));
        compositor.push(rounded_rect_stroke(
            x,
            y,
            w,
            h,
            radius,
            fade(surface.border),
            r.border_width,
        ));
        shadow_insets(compositor, layer, bounds, radius, &shadows);

        let style = self.text_style(theme);
        let text_y = y + TextMeasurer::vertical_center(&style, h);
        let end = x + w - r.pad_end - r.chevron - r.gap;
        let mut at = x + r.pad_start;
        if let Some(name) = self.icon
            && let Some(node) =
                icons::icon_at(name, r.icon, fade(r.icon_color), at, y + (h - r.icon) / 2.0)
        {
            compositor.push(node);
            at += r.icon + r.gap;
        }
        if let Some(label) = &self.label {
            let shown = TextMeasurer::truncate_to_width(label, &style, (end - at).max(0.0));
            let (lw, _) = TextMeasurer::measure_styled(&shown, &style, None);
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(&shown, &style, None),
                x: at,
                y: text_y,
                color: fade(r.label_color),
            });
            at += lw.ceil() + r.gap;
        }
        if let Some(value) = self.selected_label() {
            let shown = TextMeasurer::truncate_to_width(value, &style, (end - at).max(0.0));
            let color = if self.label.is_some() {
                r.labelled_color
            } else {
                r.value_color
            };
            if !shown.is_empty() {
                compositor.push(SceneNode::Text {
                    key: TextNodeKey::from_style(&shown, &style, None),
                    x: at,
                    y: text_y,
                    color: fade(color),
                });
            }
        }

        let chevron = if self.open {
            "chevron-up"
        } else {
            "chevron-down"
        };
        if let Some(node) = icons::icon_at(
            chevron,
            r.chevron,
            fade(r.chevron_color),
            x + w - r.pad_end - r.chevron,
            y + (h - r.chevron) / 2.0,
        ) {
            compositor.push(node);
        }
    }
}
