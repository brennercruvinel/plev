//! HOFF text field: a single-line glass field (`glass.field` at rest,
//! `surface_active` hovered, the field-focus border while editing) at
//! the nav radius, one `Md` control tall, body text with the placeholder
//! in the placeholder tone, a caret in the accent and a selection wash.
//! The editing itself lives in [`TextInput`]; this widget adds the look,
//! the pointer contract and the focus ring every form control shares.
//!
//! One `TextStyle` (`theme.typography.body()`) measures the caret, hit
//! tests clicks and draws the text, so the caret always lands on the
//! glyph that was clicked.
//!
//! An input method's composition ([`TextField::compose`]) draws inline, in
//! place of the selection or at the caret, underlined, with the caret at
//! the input method's; it is only drawn: the text, the projection and the
//! value hold what was committed.
//!
//! With a [`SemId`] ([`TextField::sem`]) the render binds the bounds it
//! draws and hit-tests with, and [`TextField::project`] declares the
//! field in a semantic projection; the app still owns the field.

use engine::clipboard::ClipboardProvider;
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{ControlSize, Theme};
use sem::{Constraints, NodeBuilder, ParamSpec, Role, SemId, SnapshotBuilder, ValueKind};

use crate::core::{EventResult, Rect, WidgetEvent, with_alpha};
use crate::form::text_input::TextInput;
use crate::recipe::{focus_ring, rounded_rect, rounded_rect_stroke};
use crate::wake::Wake;

/// Non-character editing keys a field understands. Apps map their key
/// events onto these; Enter and Tab stay with the screen (submit and
/// focus traversal are not the field's business). Left and Right move on
/// screen (in a right-to-left run Left goes forward in the text); the
/// `Select` variants are the same keys with Shift, extending the
/// selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKey {
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    SelectAll,
    SelectLeft,
    SelectRight,
    SelectHome,
    SelectEnd,
}

/// The clipboard keys of a field: the primary modifier (cmd on macOS,
/// ctrl elsewhere) with c, x, v or a. Apps map their chords onto these
/// ([`ClipKey::of`]) and hand the field their clipboard
/// ([`TextField::clip`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipKey {
    Copy,
    Cut,
    Paste,
    SelectAll,
}

impl ClipKey {
    /// The clipboard key a character names with the primary modifier held
    /// (either case); `None` for any other.
    pub fn of(key: &str) -> Option<Self> {
        Some(match key {
            "c" | "C" => Self::Copy,
            "x" | "X" => Self::Cut,
            "v" | "V" => Self::Paste,
            "a" | "A" => Self::SelectAll,
            _ => return None,
        })
    }
}

/// Retained single-line text field over a [`TextInput`].
#[derive(Clone, Debug)]
pub struct TextField {
    pub input: TextInput,
    pub disabled: bool,
    hovered: bool,
    sem: Option<SemId>,
}

impl TextField {
    pub fn new(placeholder: impl AsRef<str>) -> Self {
        Self {
            input: TextInput::new().with_placeholder(placeholder.as_ref()),
            disabled: false,
            hovered: false,
            sem: None,
        }
    }

    /// The logical identity the render binds its bounds to
    /// (`Compositor::bind_to_layer`).
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Declares this field as `id` in a projection: a `Role::TextInput`
    /// named `label`, its `text` state, editable, enabled unless disabled,
    /// and a `set_value` action that takes a text checked against the
    /// node's constraints and reads the `text` it replaces, so a value
    /// decided on a text another actor changed goes stale. It only reads
    /// the widget; the app owns it and runs `set_value`. Pass the id given
    /// to [`TextField::sem`] so the node and the binding name the same
    /// control. Returns the node for the app to add to (a `submit`, a
    /// `max_len`).
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::TextInput, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("text", self.text())
            .constraints(Constraints {
                editable: true,
                ..Constraints::default()
            })
            .enabled(!self.disabled);
        node.action("set_value")
            .param(ParamSpec {
                kind: ValueKind::Text,
                constrained: true,
            })
            .reads(format!("{id}:text"));
        node
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.input.buffer.set_text(text);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn text(&self) -> &str {
        self.input.buffer.text()
    }

    pub fn set_text(&mut self, text: &str) {
        self.input.buffer.set_text(text);
    }

    pub fn is_empty(&self) -> bool {
        self.input.buffer.is_empty()
    }

    pub fn is_focused(&self) -> bool {
        self.input.focused
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub fn focus(&mut self) {
        if !self.disabled {
            self.input.focus();
        }
    }

    pub fn unfocus(&mut self) {
        self.input.unfocus();
    }

    /// Focus on `true`, blur on `false` (the owner's focus chain calls
    /// this; a field never steals focus on its own).
    pub fn set_focused(&mut self, focused: bool) {
        if focused {
            self.focus();
        } else {
            self.unfocus();
        }
    }

    /// The text style the field measures, hit-tests and draws with.
    pub fn text_style(theme: &Theme) -> TextStyle {
        theme.typography.body()
    }

    /// Canonical field height: one `Md` control.
    pub fn height(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Md)
    }

    /// Horizontal inset from the field edge to the text origin.
    fn pad_x(theme: &Theme) -> f32 {
        theme.spacing.md
    }

    /// Type `s` (characters or a pasted string). `false` when unfocused.
    pub fn insert(&mut self, s: &str) -> bool {
        if !self.input.focused || self.disabled {
            return false;
        }
        for c in s.chars() {
            self.input.handle_char(c);
        }
        true
    }

    /// Route a non-character editing key. `false` when unfocused.
    pub fn edit(&mut self, key: EditKey) -> bool {
        if !self.input.focused || self.disabled {
            return false;
        }
        match key {
            EditKey::Backspace => self.input.handle_backspace(),
            EditKey::Delete => self.input.handle_delete(),
            EditKey::Left => self.input.handle_left(),
            EditKey::Right => self.input.handle_right(),
            EditKey::Home => self.input.handle_home(),
            EditKey::End => self.input.handle_end(),
            EditKey::SelectAll => self.input.handle_select_all(),
            EditKey::SelectLeft => self.input.handle_select_left(),
            EditKey::SelectRight => self.input.handle_select_right(),
            EditKey::SelectHome => self.input.handle_select_home(),
            EditKey::SelectEnd => self.input.handle_select_end(),
        }
        true
    }

    /// Copy the selection to `clipboard` (the person's keys or menu; a
    /// field gives an agent no way to it). `false` with nothing selected,
    /// unfocused or disabled.
    pub fn copy(&self, clipboard: &mut dyn ClipboardProvider) -> bool {
        !self.disabled && self.input.handle_copy(clipboard)
    }

    /// Copy the selection, then delete it. `false` with nothing selected,
    /// unfocused or disabled.
    pub fn cut(&mut self, clipboard: &mut dyn ClipboardProvider) -> bool {
        !self.disabled && self.input.handle_cut(clipboard)
    }

    /// Type what `clipboard` holds over the selection, on one line: its
    /// line breaks and tabs become spaces
    /// ([`single_line`](crate::form::text_input::single_line)). `false`
    /// with an empty clipboard, unfocused or disabled.
    pub fn paste(&mut self, clipboard: &mut dyn ClipboardProvider) -> bool {
        !self.disabled && self.input.handle_paste(clipboard)
    }

    /// Select the whole text. `false` when unfocused or disabled.
    pub fn select_all(&mut self) -> bool {
        self.edit(EditKey::SelectAll)
    }

    /// A clipboard key: [`copy`](Self::copy), [`cut`](Self::cut),
    /// [`paste`](Self::paste) or [`select_all`](Self::select_all). `true`
    /// when it did something.
    pub fn clip(&mut self, key: ClipKey, clipboard: &mut dyn ClipboardProvider) -> bool {
        match key {
            ClipKey::Copy => self.copy(clipboard),
            ClipKey::Cut => self.cut(clipboard),
            ClipKey::Paste => self.paste(clipboard),
            ClipKey::SelectAll => self.select_all(),
        }
    }

    /// Some text is selected: what copy and cut take.
    pub fn has_selection(&self) -> bool {
        self.input
            .buffer
            .selected_text()
            .is_some_and(|t| !t.is_empty())
    }

    /// IME commit / preedit, forwarded to the core.
    pub fn ime(&mut self, committed: &str, preedit: &str) {
        self.input.handle_ime(committed, preedit);
    }

    /// An input method's step, with its caret: `committed` is typed, the
    /// same way as [`insert`](Self::insert) (over the selection), and
    /// `preedit` draws inline until the next step (see
    /// [`TextInput::handle_ime_at`]). `false` when unfocused or disabled.
    pub fn compose(
        &mut self,
        committed: &str,
        preedit: &str,
        cursor: Option<(usize, usize)>,
    ) -> bool {
        if !self.input.focused || self.disabled {
            return false;
        }
        self.insert(committed);
        self.input.handle_ime_at("", preedit, cursor);
        true
    }

    /// The input method went away: the composition is dropped, the text
    /// stays. `true` when there was one.
    pub fn end_compose(&mut self) -> bool {
        self.input.clear_preedit()
    }

    /// Where an input method puts its candidate window: the caret, the
    /// preedit's while composing, as a rect in the coordinates of `bounds`,
    /// one caret wide and one line tall.
    pub fn ime_area(&self, bounds: Rect, theme: &Theme) -> Rect {
        let style = Self::text_style(theme);
        let (x, y, w) = Self::text_box(bounds, &style, theme);
        Rect::new(
            x + self.input.caret_x(&style, Some(w)),
            y,
            theme.control.focus_ring_width,
            style.line_height,
        )
    }

    /// The text's box inside `bounds`: its origin, `x` and `y`, and its
    /// width.
    fn text_box(bounds: Rect, style: &TextStyle, theme: &Theme) -> (f32, f32, f32) {
        let pad_x = Self::pad_x(theme);
        (
            bounds.x + pad_x,
            bounds.y + TextMeasurer::vertical_center(style, bounds.h),
            (bounds.w - pad_x * 2.0).max(0.0),
        )
    }

    /// Advance the caret blink by `dt` seconds. Focused, the ask is the
    /// caret's: a frame when it flipped (`now`) and the time to the next
    /// flip (`next`); blurred, nothing. The owner merges it with its
    /// other motion and the shell sleeps until the earliest deadline.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.input.wake(dt)
    }

    /// Hover tracks the bounds; a press inside focuses and places the
    /// caret on the clicked glyph. A press outside blurs, so a screen
    /// with several fields needs no extra bookkeeping.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        if self.disabled {
            if self.hovered || self.input.focused {
                self.hovered = false;
                self.input.unfocus();
                return EventResult::changed();
            }
            return EventResult::IGNORED;
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let inside = bounds.contains(x, y);
                if inside != self.hovered {
                    self.hovered = inside;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if bounds.contains(x, y) {
                    let style = Self::text_style(theme);
                    let pad_x = Self::pad_x(theme);
                    let text_w = (bounds.w - pad_x * 2.0).max(0.0);
                    self.input
                        .handle_click_in(x - bounds.x - pad_x, &style, Some(text_w));
                    EventResult::clicked()
                } else if self.input.focused {
                    self.input.unfocus();
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            _ => EventResult::IGNORED,
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let glass = &theme.glass;
        let alpha = if self.disabled {
            glass.disabled_alpha
        } else {
            1.0
        };
        let focused = self.input.focused;
        let style = Self::text_style(theme);
        let radius = theme.shape.nav.min(bounds.h / 2.0);

        if focused {
            c.push_to_layer(layer, focus_ring(bounds, radius, theme));
        }

        let bg = if focused || self.hovered {
            glass.surface_active
        } else {
            glass.field
        };
        c.push_to_layer(
            layer,
            rounded_rect(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                radius,
                with_alpha(bg, bg.0[3] * alpha),
            ),
        );
        let edge = if focused {
            glass.field_focus_border
        } else {
            glass.edge_soft
        };
        c.push_to_layer(
            layer,
            rounded_rect_stroke(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                radius,
                with_alpha(edge, edge.0[3] * alpha),
                theme.control.edge_width,
            ),
        );

        let (text_x, text_y, text_w) = Self::text_box(bounds, &style, theme);
        // While composing, the text as drawn has the preedit in it.
        let composed = self.input.composed();
        let text = composed
            .as_ref()
            .map_or(self.input.buffer.text(), |(t, _)| t.as_str());

        // Selection wash under the text: the accent at the wash alpha, one
        // box per stretch the logical range covers on screen.
        if focused {
            for (sel_x, sel_w) in self.input.selection_boxes(&style, Some(text_w)) {
                c.push_to_layer(
                    layer,
                    SceneNode::Rect {
                        x: text_x + sel_x,
                        y: text_y,
                        w: sel_w,
                        h: style.line_height,
                        color: with_alpha(theme.colors.accent, glass.wash_alpha),
                    },
                );
            }
        }

        // Text, or the placeholder while empty.
        let (shown, color) = if text.is_empty() {
            (
                self.input.placeholder.as_str(),
                with_alpha(glass.text_placeholder, glass.text_placeholder.0[3] * alpha),
            )
        } else {
            (
                text,
                with_alpha(theme.colors.text, theme.colors.text.0[3] * alpha),
            )
        };
        if !shown.is_empty() {
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(shown, &style, Some(text_w)),
                    x: text_x,
                    y: text_y,
                    color,
                },
            );
        }

        // The preedit's underline: one edge stroke at the foot of the line
        // box, under each stretch the preedit covers on screen.
        let underline = theme.control.edge_width;
        for (x, w) in self.input.preedit_boxes(&style, Some(text_w)) {
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: text_x + x,
                    y: text_y + style.line_height - underline,
                    w,
                    h: underline,
                    color: theme.glass.text_active.0,
                },
            );
        }

        // Caret: the accent, one focus-ring stroke wide, the line box tall.
        if self.input.cursor_visible() {
            let caret_x = self.input.caret_x(&style, Some(text_w));
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: text_x + caret_x,
                    y: text_y,
                    w: theme.control.focus_ring_width,
                    h: style.line_height,
                    color: theme.colors.accent.0,
                },
            );
        }
    }
}
