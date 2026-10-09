//! The person's keys: text into the intent line or a field of a generated
//! surface while one has the keyboard, text into the commit form,
//! everything else through the keymap (Zed model: keystroke -> matcher ->
//! name -> registry), and the actions that act on a capability through
//! the workbench.

use engine::widgets::form::{ClipKey, EditKey};
use engine::control::keymap::{Action, ContextStack, KeyContext, MatchResult, keystroke_from_key_event};
use engine::runtime::platform::clipboard::default_clipboard;
use record::InputKind;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, ModifiersState, NamedKey};

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
                // A paste is typing: recorded as text, never the text.
                let kind = if action.is::<actions::Paste>() {
                    InputKind::Text
                } else {
                    InputKind::Key
                };
                self.record_input(kind, None);
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
        if !self.bench.has_keyboard() {
            return None;
        }
        // cmd (ctrl elsewhere) with c, x, v or a: the field's clipboard,
        // the person's only; recorded as text, never the text.
        if let Key::Character(text) = &key_event.logical_key
            && primary(self.modifiers)
            && let Some(key) = ClipKey::of(text)
        {
            self.record_input(InputKind::Text, None);
            return Some(self.bench.clip(key, default_clipboard().as_mut()));
        }
        let chord =
            self.modifiers.control_key() || self.modifiers.super_key() || self.modifiers.alt_key();
        if chord {
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
            Key::Named(named) => match edit_key(*named, self.modifiers.shift_key()) {
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
        } else if action.is::<idestudio::interfaces::window::record::Save>() {
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
        } else if let Some(key) = clip_key(action) {
            // Both cmd and ctrl are bound; the platform's primary acts.
            primary(self.modifiers)
                && bench
                    .workspace_mut()
                    .commit_form
                    .clip(key, default_clipboard().as_mut())
        } else {
            log::warn!("no handler for action {}", action.name());
            false
        }
    }
}

/// The platform's primary modifier is held: cmd on macOS, ctrl elsewhere.
fn primary(modifiers: ModifiersState) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.super_key()
    } else {
        modifiers.control_key()
    }
}

/// The clipboard key an `edit::` action names.
fn clip_key(action: &dyn Action) -> Option<ClipKey> {
    if action.is::<actions::Copy>() {
        Some(ClipKey::Copy)
    } else if action.is::<actions::Cut>() {
        Some(ClipKey::Cut)
    } else if action.is::<actions::Paste>() {
        Some(ClipKey::Paste)
    } else if action.is::<actions::SelectAll>() {
        Some(ClipKey::SelectAll)
    } else {
        None
    }
}

/// The editing key a named key is for a field; with `shift` the arrows,
/// Home and End extend the selection.
fn edit_key(named: NamedKey, shift: bool) -> Option<EditKey> {
    Some(match (named, shift) {
        (NamedKey::Backspace, _) => EditKey::Backspace,
        (NamedKey::Delete, _) => EditKey::Delete,
        (NamedKey::ArrowLeft, false) => EditKey::Left,
        (NamedKey::ArrowLeft, true) => EditKey::SelectLeft,
        (NamedKey::ArrowRight, false) => EditKey::Right,
        (NamedKey::ArrowRight, true) => EditKey::SelectRight,
        (NamedKey::Home, false) => EditKey::Home,
        (NamedKey::Home, true) => EditKey::SelectHome,
        (NamedKey::End, false) => EditKey::End,
        (NamedKey::End, true) => EditKey::SelectEnd,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shift turns the moves into selection; the deletions stay deletions.
    #[test]
    fn shift_selects_with_the_moves_only() {
        assert_eq!(
            edit_key(NamedKey::ArrowLeft, true),
            Some(EditKey::SelectLeft)
        );
        assert_eq!(
            edit_key(NamedKey::ArrowRight, true),
            Some(EditKey::SelectRight)
        );
        assert_eq!(edit_key(NamedKey::Home, true), Some(EditKey::SelectHome));
        assert_eq!(edit_key(NamedKey::End, true), Some(EditKey::SelectEnd));
        assert_eq!(edit_key(NamedKey::ArrowLeft, false), Some(EditKey::Left));
        assert_eq!(
            edit_key(NamedKey::Backspace, true),
            Some(EditKey::Backspace)
        );
        assert_eq!(edit_key(NamedKey::Enter, true), None);
    }

    /// Each `edit::` action is its clipboard key; any other action none.
    #[test]
    fn the_edit_actions_are_the_clipboard_keys() {
        assert_eq!(clip_key(&actions::Copy), Some(ClipKey::Copy));
        assert_eq!(clip_key(&actions::Cut), Some(ClipKey::Cut));
        assert_eq!(clip_key(&actions::Paste), Some(ClipKey::Paste));
        assert_eq!(clip_key(&actions::SelectAll), Some(ClipKey::SelectAll));
        assert_eq!(clip_key(&actions::Submit), None);
    }
}
