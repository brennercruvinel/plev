//! The prompt input's pointer: what is open takes the press first, a
//! press off it closes it; then every control takes its own.

use super::premade::Hit;
use super::{FILES, PromptInput, PromptLayout};
use crate::styling::theme::Theme;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

impl PromptInput {
    /// A pointer event against the box at `frame`.
    pub fn handle_event(&mut self, event: &WidgetEvent, frame: Rect, theme: &Theme) -> EventResult {
        let l = self.layout(frame, theme);
        let mut result = EventResult::IGNORED;
        if let WidgetEvent::MouseMove { x, y } = *event {
            let inside = frame.contains(x, y);
            if inside != self.hovered {
                self.hovered = inside;
                result = EventResult::changed();
            }
        }
        // an open select takes every event (a press off it closes it)
        if self.setting.is_open() {
            return result.merge(self.setting.handle_event(event, l.setting, theme));
        }
        let (r, stop) = self.open_model(event, &l, theme);
        result = result.merge(r);
        if stop {
            return result;
        }
        if let Some(r) = self.open_files(event, &l) {
            return result.merge(r);
        }
        if let (Some(text), false) = (l.text, self.voice) {
            let look = self.line_look(theme);
            result = result.merge(self.line.handle_event(event, text, &look, theme));
        }
        let r = self.add.handle_event(event, l.add);
        if r.clicked {
            self.toggle_files();
        }
        result = result.merge(r);
        result = result.merge(self.setting.handle_event(event, l.setting, theme));
        // the model's trigger presses in the least target around it
        let model = crate::widgets::hit::hit_rect(l.model, theme);
        let r = self.model_press.handle(event, model, self.voice);
        if r.clicked {
            self.set_models_open(!self.model_open);
        }
        result = result.merge(r);
        let r = self.voice_button.handle_event(event, l.voice);
        if r.clicked {
            self.toggle_voice();
        }
        result = result.merge(r);
        // the view controller's arrows press, as the kit's
        if let (Some(v), Some(at)) = (&mut self.view, l.view) {
            result = result.merge(v.handle_event(event, at, theme));
        }
        let r = self.submit.handle_event(event, l.submit);
        if r.clicked {
            self.submit();
        }
        result = result.merge(r);
        if let (Some(p), Some(pl)) = (&mut self.premade, &l.premade) {
            let (r, hit) = p.handle_event(event, pl);
            result = result.merge(r);
            match hit {
                Some(Hit::Prompt(i)) => {
                    self.use_prompt(i);
                }
                Some(Hit::Surprise) => {
                    self.surprise();
                }
                // a category presses, as in the kit
                Some(Hit::Category(_)) | None => {}
            }
        }
        result
    }

    /// The open model listbox: its rows take the pointer, a choice closes
    /// it, a press off it and off its trigger closes it and goes on. The
    /// flag is `true` when the event stops here (it was on the listbox).
    fn open_model(
        &mut self,
        event: &WidgetEvent,
        l: &PromptLayout,
        theme: &Theme,
    ) -> (EventResult, bool) {
        let Some(panel) = l.models else {
            return (EventResult::IGNORED, false);
        };
        let r = self.model.handle_event(event, panel, theme);
        if r.clicked {
            self.set_models_open(false);
            return (EventResult::changed(), true);
        }
        let on_panel = |x: f32, y: f32| panel.contains(x, y);
        match *event {
            WidgetEvent::MouseDown { x, y } if !on_panel(x, y) && !l.model.contains(x, y) => {
                self.set_models_open(false);
                (EventResult::changed(), false)
            }
            WidgetEvent::MouseDown { x, y }
            | WidgetEvent::MouseUp { x, y }
            | WidgetEvent::MouseMove { x, y } => (r, on_panel(x, y)),
            WidgetEvent::Scroll { .. } => (r, false),
        }
    }

    /// The open add-files menu: a row picked adds a file and closes it, a
    /// press off it and off its button closes it and goes on.
    fn open_files(&mut self, event: &WidgetEvent, l: &PromptLayout) -> Option<EventResult> {
        let (panel, rows) = l.files.as_ref()?;
        let at = |x: f32, y: f32| rows.iter().position(|r| r.contains(x, y));
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let row = at(x, y);
                if row != self.files_hover {
                    self.files_hover = row;
                    return Some(EventResult::changed());
                }
                None
            }
            WidgetEvent::MouseDown { x, y } if panel.contains(x, y) => {
                self.files_pressed = at(x, y);
                Some(EventResult::changed())
            }
            WidgetEvent::MouseDown { x, y } if !l.add.contains(x, y) => {
                self.files_open = false;
                self.files_hover = None;
                None
            }
            WidgetEvent::MouseUp { x, y } => {
                let down = self.files_pressed.take()?;
                if at(x, y) == Some(down) && down < FILES.len() {
                    self.add_file(down);
                    return Some(EventResult::clicked());
                }
                Some(EventResult::changed())
            }
            _ => None,
        }
    }
}
