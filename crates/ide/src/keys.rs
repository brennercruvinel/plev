//! The person's keys: text into the commit form, everything else through
//! the keymap (Zed model: keystroke -> matcher -> name -> registry), and
//! the actions that act on a capability through the workbench.

use engine::actions::{Action, ContextStack, KeyContext, MatchResult, keystroke_from_key_event};
use rec::InputKind;
use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key;

use crate::{App, actions};

impl App {
    /// Keyboard handling: text input for the commit form, everything else
    /// through the keymap. Returns true if state changed.
    pub(crate) fn handle_key_event(
        &mut self,
        key_event: &KeyEvent,
        event_loop: &ActiveEventLoop,
    ) -> bool {
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
