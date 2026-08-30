//! OPEN screen: path field + Open button + recents + drag-and-drop hint
//! on desktop; a file-picker button on the web (browsers get no path
//! input and no canvas drag-and-drop). The screen owns only widget state;
//! recents, status and the embedder probe live in `NestuiView` and flow
//! in as render context.

use engine::compositor::Compositor;
use engine::text::TextMeasurer;
use engine::theme::Theme;
use engine::ui::widgets::{
    Button, EventResult, Rect, Spinner, SpinnerSize, WidgetEvent, rounded_rect_stroke,
};

use super::field::FIELD_H;
#[cfg(not(target_arch = "wasm32"))]
use super::field::Field;
use super::{Action, group_label, panel, text};

const GAP: f32 = 12.0;
const ROW_H: f32 = 40.0;

/// Everything the Open screen needs from the central view state.
pub struct OpenContext<'a> {
    pub recents: &'a [String],
    /// Last open attempt's error (empty string = none).
    pub error: &'a str,
    /// Embedder probe result, once the worker answered.
    pub embedder: Option<&'a Result<String, String>>,
    pub opening: bool,
    /// A file is hovering over the window (drag-and-drop feedback).
    pub file_hover: bool,
}

pub struct OpenScreen {
    /// Desktop path entry (rendered native-only; browsers can't take
    /// filesystem paths).
    #[cfg(not(target_arch = "wasm32"))]
    path: Field,
    open_button: Button,
    recents_hover: Option<usize>,
    spinner: Spinner,
    opening: bool,
}

impl OpenScreen {
    pub fn new(theme: &Theme) -> Self {
        // `theme` only feeds the native path Field; on wasm there is no
        // filesystem path entry.
        #[cfg(target_arch = "wasm32")]
        let _ = theme;
        #[cfg(not(target_arch = "wasm32"))]
        let label = "Open";
        #[cfg(target_arch = "wasm32")]
        let label = "Choose a .nest file";
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            path: Field::new("/path/to/corpus.nest", theme),
            open_button: Button::new(label).icon("folder-open"),
            recents_hover: None,
            spinner: Spinner::new().size(SpinnerSize::Sm),
            opening: false,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn field_rect(&self, content: Rect) -> Rect {
        let (bw, bh) = self.open_button.preferred_size();
        Rect::new(
            content.x,
            content.y,
            (content.w - bw - GAP).max(120.0),
            FIELD_H.max(bh),
        )
    }

    fn button_rect(&self, content: Rect) -> Rect {
        let (bw, bh) = self.open_button.preferred_size();
        #[cfg(not(target_arch = "wasm32"))]
        return Rect::new(content.x + content.w - bw, content.y, bw, bh);
        // Web: the picker button is the primary control; left-align it.
        #[cfg(target_arch = "wasm32")]
        Rect::new(content.x, content.y, bw, bh)
    }

    fn recent_rects(&self, content: Rect, count: usize) -> (f32, Vec<Rect>) {
        let y = content.y + FIELD_H + GAP * 3.0 + 40.0 + 28.0;
        let rects = (0..count)
            .map(|i| Rect::new(content.x, y + i as f32 * ROW_H, content.w, ROW_H))
            .collect();
        (y, rects)
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        recents: &[String],
        opening: bool,
    ) -> (EventResult, Action) {
        self.opening = opening;
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.open_button.disabled = opening || self.path.is_empty();
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.open_button.disabled = opening;
        }

        // The Open button.
        let r = self
            .open_button
            .handle_event(event, self.button_rect(content));
        if r.clicked {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.path.unfocus();
                return (r, Action::OpenPath(self.path.text().trim().to_string()));
            }
            #[cfg(target_arch = "wasm32")]
            return (r, Action::PickFile);
        }

        // Recent rows.
        let mut result = r;
        let (_, rects) = self.recent_rects(content, recents.len());
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = rects.iter().position(|r| r.contains(x, y));
                if hit != self.recents_hover {
                    self.recents_hover = hit;
                    result = result.merge(EventResult::changed());
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if let Some(i) = rects.iter().position(|r| r.contains(x, y)) {
                    return (EventResult::clicked(), Action::OpenPath(recents[i].clone()));
                }
            }
            _ => {}
        }

        // The path field: click focuses, characters flow via
        // `handle_key`/`handle_paste` on the view.
        #[cfg(not(target_arch = "wasm32"))]
        if let WidgetEvent::MouseDown { x, y } = *event {
            let field = self.field_rect(content);
            if field.contains(x, y) {
                self.path.click(x - field.x);
                return (EventResult::changed(), Action::None);
            }
            if self.path.input.focused {
                self.path.unfocus();
                return (EventResult::changed(), Action::None);
            }
        }
        (result, Action::None)
    }

    /// Type characters into the path field (desktop; the web screen has
    /// no text input).
    pub fn handle_text(&mut self, s: &str) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        return self.path.insert(s);
        #[cfg(target_arch = "wasm32")]
        {
            let _ = s;
            false
        }
    }

    pub fn handle_edit_key(&mut self, key: super::EditKey) -> (bool, Action) {
        #[cfg(not(target_arch = "wasm32"))]
        if key == super::EditKey::Enter && self.path.input.focused && !self.path.is_empty() {
            return (true, Action::OpenPath(self.path.text().trim().to_string()));
        }
        #[cfg(not(target_arch = "wasm32"))]
        let out = (self.path.edit(key), Action::None);
        #[cfg(target_arch = "wasm32")]
        let out = {
            let _ = key;
            (false, Action::None)
        };
        out
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        // The spinner animates only while an open is in flight.
        let spinning = self.opening && self.spinner.tick(dt);
        #[cfg(not(target_arch = "wasm32"))]
        return self.path.tick(dt) | spinning;
        #[cfg(target_arch = "wasm32")]
        {
            let _ = dt;
            spinning
        }
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme, ctx: &OpenContext) {
        let style_14 = engine::text::TextStyle::new(14.0).with_weight(400);
        let button = self.button_rect(content);
        #[cfg(not(target_arch = "wasm32"))]
        {
            let field = self.field_rect(content);
            self.open_button.disabled = ctx.opening || self.path.is_empty();
            self.open_button.label = if ctx.opening { "Opening…" } else { "Open" }.to_string();
            self.path.render(c, field, theme);
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.open_button.disabled = ctx.opening;
            self.open_button.label = if ctx.opening {
                "Opening…"
            } else {
                "Choose a .nest file"
            }
            .to_string();
        }
        self.open_button.render(c, button, theme);
        if ctx.opening {
            self.spinner.render(
                c,
                Rect::new(
                    button.x + button.w + GAP,
                    button.y + (button.h - 16.0) / 2.0,
                    16.0,
                    16.0,
                ),
                theme,
            );
        }

        // Status line: the last open error (destructive) or the hint.
        let status_y = content.y + button.h.max(FIELD_H) + GAP;
        if !ctx.error.is_empty() {
            text(
                c,
                ctx.error,
                13.0,
                500,
                content.x,
                status_y,
                theme.colors.danger.0,
            );
        } else {
            #[cfg(not(target_arch = "wasm32"))]
            let hint = "drop a .nest file anywhere to open it";
            #[cfg(target_arch = "wasm32")]
            let hint = "pick a .nest file (drag-and-drop is not available on the web canvas)";
            text(
                c,
                hint,
                13.0,
                400,
                content.x,
                status_y,
                theme.colors.text_dim.0,
            );
        }

        // Embedder doctor line.
        let embedder_y = status_y + 24.0;
        match ctx.embedder {
            Some(Ok(status)) => {
                text(
                    c,
                    "text search:",
                    13.0,
                    600,
                    content.x,
                    embedder_y,
                    theme.colors.text_mid.0,
                );
                text(
                    c,
                    status,
                    13.0,
                    400,
                    content.x + 92.0,
                    embedder_y,
                    theme.colors.success.0,
                );
            }
            Some(Err(reason)) => {
                text(
                    c,
                    "text search:",
                    13.0,
                    600,
                    content.x,
                    embedder_y,
                    theme.colors.text_mid.0,
                );
                let msg = TextMeasurer::truncate_to_width(reason, &style_14, content.w - 92.0);
                text(
                    c,
                    &msg,
                    13.0,
                    400,
                    content.x + 92.0,
                    embedder_y,
                    theme.colors.danger.0,
                );
            }
            None => {
                text(
                    c,
                    "text search: (probe runs after the first open)",
                    13.0,
                    400,
                    content.x,
                    embedder_y,
                    theme.colors.text_dim.0,
                );
            }
        }

        // Recents.
        if !ctx.recents.is_empty() {
            let (y, rects) = self.recent_rects(content, ctx.recents.len());
            group_label(c, "RECENT FILES", content.x, y - 24.0, theme);
            for (i, (path, rect)) in ctx.recents.iter().zip(&rects).enumerate() {
                if self.recents_hover == Some(i) {
                    c.push(engine::ui::widgets::rounded_rect(
                        rect.x,
                        rect.y,
                        rect.w,
                        rect.h,
                        theme.radius.md,
                        theme.glass.surface_hover.0,
                    ));
                }
                if let Some(node) = engine::ui::icons::icon_at(
                    "file",
                    16.0,
                    theme.glass.text_faint.0,
                    rect.x + 8.0,
                    rect.y + (rect.h - 16.0) / 2.0,
                ) {
                    c.push(node);
                }
                let label = TextMeasurer::truncate_to_width(path, &style_14, rect.w - 44.0);
                text(
                    c,
                    &label,
                    14.0,
                    400,
                    rect.x + 32.0,
                    rect.y + 12.0,
                    theme.colors.text_mid.0,
                );
            }
        }

        // Drag-and-drop hover: an accent stroke around the whole content.
        if ctx.file_hover {
            c.push(rounded_rect_stroke(
                content.x - 8.0,
                content.y - 8.0,
                content.w + 16.0,
                content.h + 16.0,
                theme.radius.lg,
                theme.colors.accent.0,
                2.0,
            ));
            let hint = "release to open";
            let (tw, _) = engine::text::TextMeasurer::measure_styled(hint, &style_14, None);
            panel(
                c,
                Rect::new(
                    content.x + (content.w - tw - 32.0) / 2.0,
                    content.y + content.h / 2.0 - 22.0,
                    tw + 32.0,
                    44.0,
                ),
                theme,
            );
            text(
                c,
                hint,
                14.0,
                600,
                content.x + (content.w - tw) / 2.0,
                content.y + content.h / 2.0 - 7.0,
                theme.colors.text.0,
            );
        }
    }
}
