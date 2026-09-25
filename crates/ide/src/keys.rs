//! The person's keys: text into the intent line or a field of a generated
//! surface while one has the keyboard, text into the commit form,
//! everything else through the keymap (Zed model: keystroke -> matcher ->
//! name -> registry), and the actions that act on a capability through
//! the workbench.

use comps::form::EditKey;
use engine::actions::{Action, ContextStack, KeyContext, MatchResult, keystroke_from_key_event};
use rec::InputKind;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};

use crate::{App, actions};

impl App {
    /// Keyboard handling: text input for the commit form, everything else
    /// through the keymap. Returns true if state changed.
    pub(crate) fn handle_key_event(
        &mut self,
        key_event: &KeyEvent,
        event_loop: &ActiveEventLoop,
    ) -> bool {
        if let Some(changed) = self.column_key(key_event) {
            return changed;
        }
        // The visible commit form captures plain characters as text before
        // the keymap sees them (named keys, Enter/Escape/Backspace, still
        // dispatch through the CommitForm context bindings). Typing is
        // editing, not a capability: it stays with the field.
        let form = &mut self.bench.workspace_mut().commit_form;
        if form.visible
            && !self.modifiers.control_key()
            && !self.modifiers.super_key()
            && !self.modifiers.alt_key()
            && let Key::Character(text) = &key_event.logical_key
        {
            for ch in text.chars() {
                form.append_char(ch);
            }
            // typed into the field: recorded as text, never the text.
            self.record_input(InputKind::Text, None);
            return true;
        }

        let Some(keystroke) = keystroke_from_key_event(key_event, self.modifiers) else {
            return false;
        };
        let stack = self.context_stack();
        match self.matcher.match_keystroke(&keystroke, &stack) {
            MatchResult::Complete(name) => {
                let Some(action) = self.registry.build(&name) else {
                    log::warn!("keymap resolved unknown action `{name}`");
                    return false;
                };
                self.record_input(InputKind::Key, None);
                self.dispatch(action.as_ref(), event_loop)
            }
            MatchResult::Pending | MatchResult::None => false,
        }
    }

    /// While the intent line or a field of a generated surface has the
    /// keyboard (task-96), what is typed and its editing keys are theirs:
    /// characters into the field, Enter proposes (or submits the
    /// surface's field), Escape lets the keyboard go, the rest of the
    /// named keys stop there. A chord with a modifier still reaches the
    /// keymap (the session's save). `None` when the column does not have
    /// the keyboard.
    fn column_key(&mut self, key_event: &KeyEvent) -> Option<bool> {
        let chord =
            self.modifiers.control_key() || self.modifiers.super_key() || self.modifiers.alt_key();
        if !self.bench.has_keyboard() || chord {
            return None;
        }
        let bench = &mut self.bench;
        let changed = match &key_event.logical_key {
            Key::Character(text) => {
                let typed = bench.type_text(text);
                // typed into the field: recorded as text, never the text.
                self.record_input(InputKind::Text, None);
                return Some(typed);
            }
            Key::Named(NamedKey::Enter) => bench.enter(),
            Key::Named(NamedKey::Escape) => bench.escape(),
            Key::Named(NamedKey::Space) => bench.type_text(" "),
            Key::Named(named) => match edit_key(*named) {
                Some(key) => bench.edit_key(key),
                None => false,
            },
            _ => false,
        };
        self.record_input(InputKind::Key, None);
        Some(changed)
    }

    /// Key contexts from outermost to innermost; deeper entries win binding
    /// conflicts (e.g. Escape closes an overlay before quitting the app).
    fn context_stack(&self) -> ContextStack {
        let ws = self.bench.workspace();
        let mut stack = ContextStack::new();
        stack.push(KeyContext::new("Workspace"));
        if ws.commit_form.visible {
            stack.push(KeyContext::new("CommitForm"));
        }
        if !ws.overlay_mgr.is_empty() {
            stack.push(KeyContext::new("Overlay"));
        }
        stack
    }

    /// Executes a resolved action. Returns true if state changed. The
    /// keymap names input actions; the ones that act on a capability
    /// (`commit::Submit`, `file::Stage`, ...) go through the workbench,
    /// which asks the dispatcher as the person.
    fn dispatch(&mut self, action: &dyn Action, event_loop: &ActiveEventLoop) -> bool {
        let bench = &mut self.bench;
        if action.is::<actions::Quit>() {
            event_loop.exit();
            false
        } else if action.is::<ide::record::Save>() {
            self.save_session("key");
            false
        } else if action.is::<actions::Toggle>() {
            bench.workspace_mut().toggle_theme();
            true
        } else if action.is::<actions::Up>() {
            bench.nav_up()
        } else if action.is::<actions::Down>() {
            bench.nav_down()
        } else if action.is::<actions::ShowDiff>() {
            bench.show_selected_diff()
        } else if action.is::<actions::Stage>() {
            bench.stage_selected()
        } else if action.is::<actions::Discard>() {
            bench.discard_selected()
        } else if action.is::<actions::OpenForm>() {
            bench.toggle_commit_form()
        } else if action.is::<actions::Submit>() {
            bench.submit_commit()
        } else if action.is::<actions::Cancel>() {
            bench.cancel_commit_form()
        } else if action.is::<actions::Backspace>() {
            bench.workspace_mut().commit_form.backspace();
            true
        } else if action.is::<actions::Close>() {
            bench.close_top_overlay()
        } else {
            log::warn!("no handler for action {}", action.name());
            false
        }
    }
}

/// The editing key a named key is for a field.
fn edit_key(named: NamedKey) -> Option<EditKey> {
    Some(match named {
        NamedKey::Backspace => EditKey::Backspace,
        NamedKey::Delete => EditKey::Delete,
        NamedKey::ArrowLeft => EditKey::Left,
        NamedKey::ArrowRight => EditKey::Right,
        NamedKey::Home => EditKey::Home,
        NamedKey::End => EditKey::End,
        _ => return None,
    })
}
