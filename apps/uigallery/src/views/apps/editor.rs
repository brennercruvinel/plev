//! Editor section: the multi-cursor text editor (`comps::editor`) filling
//! the content rect, the same widget the `comps --example editor` window
//! runs. Pointer events arrive through the section contract; the keyboard
//! goes straight to `EditorView::handle_key` (it wants the winit key and
//! modifiers) via `ShowcaseView::handle_editor_key`, and the input method
//! to `EditorView::handle_ime` via `ShowcaseView::handle_ime`.

use engine::widgets::editor::{EditorTheme, EditorView, MouseEvent};
use engine::widgets::prelude::{EventResult, Rect, Wake, WidgetEvent};
use engine::drawing::compositor::Compositor;
use engine::compose::layout::ComputedBounds;
use engine::styling::theme::Theme;
use editor::Document;
use winit::event::Ime;
use winit::keyboard::{Key, ModifiersState};

use crate::views::ImeEvent;

pub struct EditorPage {
    editor: EditorView,
    theme: EditorTheme,
    /// Left button held inside the editor: moves become drags.
    mouse_down: bool,
    /// Modifier state mirrored from the platform shell so alt+click adds
    /// a cursor and shift+click extends the selection.
    modifiers: ModifiersState,
}

impl EditorPage {
    pub fn new(theme: &Theme) -> Self {
        Self {
            editor: EditorView::new(Document::load(&demo_text())),
            theme: EditorTheme::from_theme(theme),
            mouse_down: false,
            modifiers: ModifiersState::empty(),
        }
    }

    pub fn set_theme(&mut self, theme: &Theme) {
        self.theme = EditorTheme::from_theme(theme);
    }

    pub fn set_modifiers(&mut self, modifiers: ModifiersState) {
        self.modifiers = modifiers;
    }

    /// The editor fills the viewport and scrolls its own lines.
    pub fn content_height(&self, content: Rect) -> f32 {
        content.h
    }

    fn bounds(content: Rect) -> ComputedBounds {
        ComputedBounds {
            x: content.x,
            y: content.y,
            width: content.w,
            height: content.h,
        }
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        self.editor.set_bounds(Self::bounds(content));
        let changed = match *event {
            WidgetEvent::MouseDown { x, y } if content.contains(x, y) => {
                self.mouse_down = true;
                self.editor.handle_mouse(MouseEvent::Down {
                    x,
                    y,
                    alt: self.modifiers.alt_key(),
                    shift: self.modifiers.shift_key(),
                })
            }
            WidgetEvent::MouseMove { x, y } if self.mouse_down => {
                self.editor.handle_mouse(MouseEvent::Drag { x, y })
            }
            WidgetEvent::MouseUp { .. } if self.mouse_down => {
                self.mouse_down = false;
                self.editor.handle_mouse(MouseEvent::Up)
            }
            WidgetEvent::Scroll { x, y, delta } if content.contains(x, y) => {
                self.editor.handle_mouse(MouseEvent::Wheel { dy: delta });
                // Consumed either way: the page must not scroll under it.
                return EventResult::changed();
            }
            _ => false,
        };
        if changed {
            EventResult::changed()
        } else {
            EventResult::IGNORED
        }
    }

    pub fn handle_key(&mut self, key: &Key, modifiers: ModifiersState) -> bool {
        self.editor.handle_key(key, modifiers)
    }

    /// A step of the input method, back in winit's form for the editor.
    pub fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool {
        let ime = match ime {
            ImeEvent::Preedit(text, cursor) => Ime::Preedit(text.to_owned(), cursor),
            ImeEvent::Commit(text) => Ime::Commit(text.to_owned()),
            ImeEvent::Disabled => Ime::Disabled,
        };
        self.editor.handle_ime(&ime)
    }

    /// The editor's caret (the preedit's while composing), where the
    /// input method's candidate window goes.
    pub fn ime_area(&self) -> Rect {
        let r = self.editor.ime_cursor_rect();
        Rect::new(r.x, r.y, r.width, r.height)
    }

    /// Caret blink: a frame on a toggle, the next toggle as the deadline.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.editor.wake(dt)
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect) {
        self.editor.render(c, Self::bounds(content), &self.theme);
    }
}

/// The same sample text the standalone editor example opens.
fn demo_text() -> String {
    let mut s = String::new();
    s.push_str("// plev editor\n");
    s.push_str("//\n");
    s.push_str("// Try it out:\n");
    s.push_str("//   - click / drag to select, double-click a word, triple-click a line\n");
    s.push_str("//   - alt+click to add cursors, then type\n");
    s.push_str("//   - cmd-c / cmd-x / cmd-v with multiple cursors\n");
    s.push_str("//   - cmd-z / cmd-shift-z, cmd-a, home/end, pageup/pagedown\n");
    s.push('\n');
    for i in 0..38 {
        s.push_str(&format!("/// Block {i}: five lines of sample content.\n"));
        s.push_str(&format!("fn sample_{i}(input: &str) -> usize {{\n"));
        s.push_str(&format!("    let value = input.len() + {i};\n"));
        s.push_str("    value * 2\n");
        s.push_str("}\n");
    }
    s
}
