//! cmd+click to the code, in a desktop debug build (where the engine has
//! `hot_reload`; the parent gates this module on the same terms). A left
//! click with cmd held (ctrl off macOS) never reaches the page, press or
//! release: it asks `engine::hot_reload::view_source::origin_at` what made
//! the pixel under the cursor in the frame on screen, and opens the one
//! site it names in the editor (`PLEV_EDITOR`, else zed, else `code -g`).
//! Anything else is logged at warn, the level the shell shows: no control
//! and no text there, a control with no source, a text several literals
//! hold (each listed), a formatted text. It draws nothing and asks for no
//! frame, so cmd alone or cmd+click changes nothing on screen.
//!
//! With shift too (task-65 phase c), a text exactly one literal holds is
//! edited in place: a `comps::form::TextField` over the text, on a layer
//! of its own above the frame, owns the keyboard. Enter writes what was
//! typed over the literal (`literals::write`, as the person: the edit is
//! no capability, so nothing of it reaches the dispatcher or the agent
//! port), and the hot reload shows it; Escape, or a click outside the
//! field, drops it. A text that is not one literal, and a write the
//! engine refuses (the file's code changed since this build, the literal
//! moved), are told in a toast and logged.

use engine::widgets::form::{EditKey, TextField};
use engine::widgets::prelude::{Rect, WidgetEvent};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode};
use engine::runtime::hot_reload::Site;
use engine::runtime::hot_reload::literals::{self, Change, Refused};
use engine::runtime::hot_reload::view_source::{self, Origin, origin_at, text_run_at};
use engine::styling::theme::{Intent, Theme};
use accord::Actor;
use winit::event::ElementState;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::WindowId;

use super::super::App;

/// The shell's view source: the button a click took, the literal edited
/// in place, the layer its field is drawn on.
#[derive(Default)]
pub(super) struct ViewSource {
    /// A cmd+click, or a press on the field, takes the button until it is
    /// released.
    click: bool,
    edit: Option<Edit>,
    /// Made once in the window of the first edit, above every layer then.
    layer: Option<(WindowId, LayerId)>,
}

/// A literal edited in place.
struct Edit {
    window: WindowId,
    layer: LayerId,
    site: Site,
    /// The text as the literal holds it, and as the frame drew it.
    was: String,
    /// Where the frame drew it: the field covers it.
    run: [f32; 4],
    bounds: Rect,
    field: TextField,
}

impl ViewSource {
    /// The field over the frame of window `id`, when the edit is there:
    /// the page's ground where the text was drawn, then the field.
    pub(super) fn draw(&self, id: WindowId, c: &mut Compositor, theme: &Theme) {
        let Some(edit) = self.edit.as_ref().filter(|e| e.window == id) else {
            return;
        };
        let [x, y, w, h] = edit.run;
        let color = theme.colors.bg.0;
        c.push_to_layer(edit.layer, SceneNode::Rect { x, y, w, h, color });
        edit.field
            .render_to_layer(c, edit.layer, edit.bounds, theme);
    }
}

impl App {
    /// `true` when this left click in window `id` is a view-source click,
    /// press and its release both: the caller hands it to nobody else.
    pub(super) fn view_source(&mut self, id: WindowId, state: ElementState) -> bool {
        self.view_source_with(id, state, view_source::open)
    }

    /// [`view_source`](Self::view_source) with the opener given: the
    /// window opens the editor; a test keeps the site. The pixel is read
    /// in the frame of the window the click landed in. With shift, the
    /// text there is edited in place instead; a plain press while editing
    /// places the caret in the field, or drops the edit outside it.
    fn view_source_with(
        &mut self,
        id: WindowId,
        state: ElementState,
        open: impl FnOnce(&Site) -> std::io::Result<String>,
    ) -> bool {
        match state {
            ElementState::Pressed if primary(self.modifiers) => {
                self.source.click = true;
                let (x, y) = self.cursor_in(id);
                if self.modifiers.shift_key() {
                    self.begin_edit(id, x, y);
                    return true;
                }
                let Some(host) = self.windows.get(id) else {
                    return true;
                };
                let snapshot = self.view.semantics();
                let origin = origin_at(host.compositor(), snapshot.as_ref(), x, y);
                match origin.site().map(|site| (site, open(site))) {
                    Some((_, Ok(ran))) => log::info!("view source: {origin}; ran {ran}"),
                    Some((_, Err(e))) => log::warn!("view source: {origin}; {e}"),
                    None => log::warn!("view source: {origin}"),
                }
                true
            }
            ElementState::Pressed => self.press_while_editing(id),
            ElementState::Released => std::mem::take(&mut self.source.click),
        }
    }

    /// The field over the text at `(x, y)` of window `id`, when exactly one
    /// literal holds it; else the toast says why not.
    fn begin_edit(&mut self, id: WindowId, x: f32, y: f32) {
        self.source.edit = None;
        let snapshot = self.view.semantics();
        let Some(host) = self.windows.get_mut(id) else {
            return;
        };
        let origin = origin_at(host.compositor(), snapshot.as_ref(), x, y);
        let (Origin::Literal { text, sites }, Some((_, run))) =
            (&origin, text_run_at(host.compositor(), x, y))
        else {
            return self.tell(
                format!("view source: {}", not_one_literal(&origin)),
                Intent::Informational,
            );
        };
        let [site] = sites.as_slice() else {
            return self.tell(
                format!("view source: {}", not_one_literal(&origin)),
                Intent::Informational,
            );
        };
        let c = host.compositor_mut();
        let layer = match self.source.layer {
            Some((window, layer)) if window == id => layer,
            _ => {
                let top = c.layers().iter().map(|l| l.z_order).max().unwrap_or(0);
                let layer = c.create_layer(top + 1);
                self.source.layer = Some((id, layer));
                layer
            }
        };
        let width = host.logical_size().0;
        let mut field = TextField::new("").with_text(text);
        field.focus();
        log::info!("view source: editing {text:?}, the literal at {site}");
        self.source.edit = Some(Edit {
            window: id,
            layer,
            site: site.clone(),
            was: text.clone(),
            run,
            bounds: field_bounds(run, width, &self.view.theme),
            field,
        });
        self.invalidate_in(id);
    }

    /// A press with no cmd in window `id`: on the field it places the
    /// caret and is taken; elsewhere the edit is dropped and the page has
    /// the press. `false` with no edit.
    fn press_while_editing(&mut self, id: WindowId) -> bool {
        let (x, y) = self.cursor_in(id);
        let Some(edit) = self.source.edit.as_mut() else {
            return false;
        };
        if edit.window == id && edit.bounds.contains(x, y) {
            let down = WidgetEvent::MouseDown { x, y };
            edit.field
                .handle_event(&down, edit.bounds, &self.view.theme);
            self.source.click = true;
            self.invalidate_in(id);
            return true;
        }
        self.source.edit = None;
        log::info!("view source: edit dropped, nothing written");
        self.invalidate();
        false
    }

    /// A key while a literal is edited in place: the field's, every one
    /// (`false` with no edit, the key is the page's). Enter writes it as
    /// the person, Escape drops it.
    pub(super) fn edit_key(&mut self, key: &Key) -> bool {
        self.edit_key_with(key, |site, from, to| {
            literals::write(site, from, to, &Actor::Human)
        })
    }

    /// [`edit_key`](Self::edit_key) with the writer given: the window
    /// writes the file; a test keeps what it was asked.
    fn edit_key_with(
        &mut self,
        key: &Key,
        write: impl FnOnce(&Site, &str, &str) -> Result<Change, Refused>,
    ) -> bool {
        let command = primary(self.modifiers);
        let shift = self.modifiers.shift_key();
        let Some(edit) = self.source.edit.as_mut() else {
            return false;
        };
        match key {
            Key::Named(NamedKey::Enter) => self.commit(write),
            Key::Named(NamedKey::Escape) => {
                self.source.edit = None;
                log::info!("view source: edit dropped, nothing written");
            }
            Key::Named(NamedKey::Space) => {
                edit.field.insert(" ");
            }
            Key::Named(named) => {
                if let Some(k) = editing(named, shift) {
                    edit.field.edit(k);
                }
            }
            Key::Character(c) if command => {
                if c.eq_ignore_ascii_case("a") {
                    edit.field.edit(EditKey::SelectAll);
                }
            }
            Key::Character(c) => {
                edit.field.insert(c);
            }
            _ => {}
        }
        self.invalidate();
        true
    }

    /// What was typed, over the literal: unchanged is nothing to write; a
    /// refusal is told.
    fn commit(&mut self, write: impl FnOnce(&Site, &str, &str) -> Result<Change, Refused>) {
        let Some(edit) = self.source.edit.take() else {
            return;
        };
        let to = edit.field.text();
        if to == edit.was {
            log::info!("view source: {to:?} unchanged, nothing written");
            return;
        }
        match write(&edit.site, &edit.was, to) {
            Ok(change) => log::info!(
                "view source: {to:?} written over {:?} at {}; {change:?}",
                edit.was,
                edit.site
            ),
            Err(why) => self.tell(format!("view source: {why}"), Intent::Destructive),
        }
    }

    /// Tells the person `line`: a toast on screen, and the log. Not the
    /// shell's notice: that one an agent reads.
    fn tell(&mut self, line: String, intent: Intent) {
        log::warn!("{line}");
        let view = &mut self.view;
        view.toasts.push(line, intent, &view.theme);
        self.invalidate();
    }
}

/// Why the text at a point is not edited in place, in one line.
fn not_one_literal(origin: &Origin) -> String {
    match origin {
        Origin::Projected { id, .. } | Origin::Unsourced(id) => {
            format!("{id} is a control; only a text one literal holds is edited here")
        }
        Origin::Literal { text, sites } => {
            format!(
                "{text:?}: {} literals hold it; edit it in the file",
                sites.len()
            )
        }
        other => other.to_string(),
    }
}

/// The field over a text drawn at `run`: from the field's inset before
/// the text to the window's right edge less that inset (never narrower
/// than the text), at least one field tall, centered on the text.
fn field_bounds([x, y, w, h]: [f32; 4], window_width: f32, theme: &Theme) -> Rect {
    let inset = theme.spacing.md;
    let height = TextField::height(theme).max(h);
    let left = x - inset;
    let width = (window_width - inset - left).max(w + 2.0 * inset);
    Rect::new(left, y + (h - height) / 2.0, width, height)
}

/// The editing keys of the field, shift extending the selection.
fn editing(named: &NamedKey, shift: bool) -> Option<EditKey> {
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

/// The view-source modifier: cmd on macOS, ctrl elsewhere (the super key
/// belongs to the window manager there).
fn primary(modifiers: ModifiersState) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.super_key()
    } else {
        modifiers.control_key()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::path::Path;
    use std::time::{Duration, Instant};

    use engine::widgets::prelude::WidgetEvent;
    use engine::drawing::compositor::SceneNode;
    use engine::runtime::hot_reload::{literals, source_dirs};
    use accord::{SemId, Value};

    use super::*;
    use crate::views::ShowcaseView;

    const PRIMARY: ModifiersState = if cfg!(target_os = "macos") {
        ModifiersState::SUPER
    } else {
        ModifiersState::CONTROL
    };

    /// The shell on the todo page with one frame drawn, the cursor on the
    /// first item's row (its checkbox side).
    fn todo() -> App {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.jump_to_page("Todo");
        view.resize(1200.0, 800.0, 1.0);
        let mut app = App::with_bare_window(view);
        let App { view, windows, .. } = &mut app;
        let host = windows.get_mut(MAIN).unwrap();
        view.render(host.compositor_mut());
        let item = SemId::parse("todo.item[1]").unwrap();
        let [x, y, _, h] = host.compositor().binding(&item).unwrap().rect;
        app.cursor = (x + 19.0, y + h / 2.0);
        app
    }

    /// The bare main window of the tests.
    const MAIN: WindowId = WindowId::dummy();

    fn compositor(app: &App) -> &engine::drawing::compositor::Compositor {
        app.windows.get(MAIN).unwrap().compositor()
    }

    fn completed(app: &App) -> Value {
        let item = SemId::parse("todo.item[1]").unwrap();
        let snap = app.view.semantics().unwrap();
        snap.get(&item).unwrap().state["completed"].clone()
    }

    #[test]
    fn a_primary_click_opens_the_projection_and_never_reaches_the_page() {
        let mut app = todo();
        let before = completed(&app);
        app.modifiers = PRIMARY;
        let opened = RefCell::new(Vec::new());
        let open = |site: &Site| {
            opened.borrow_mut().push(site.clone());
            Ok(String::new())
        };
        assert!(app.view_source_with(MAIN, ElementState::Pressed, open));
        // cmd let go before the button: the release is still taken
        app.modifiers = ModifiersState::empty();
        assert!(app.view_source_with(MAIN, ElementState::Released, |_| unreachable!()));
        assert_eq!(completed(&app), before, "the item was not toggled");

        let opened = opened.into_inner();
        assert_eq!(opened.len(), 1);
        let site = &opened[0];
        assert!(
            site.file
                .ends_with("crates/showcase/src/view/apps/app/semantics.rs")
        );
        let src = std::fs::read_to_string(&site.file).unwrap();
        let line = src.lines().nth(site.line as usize - 1).unwrap();
        assert!(line.contains("b.node(&id, Role::ListItem"), "{line}");
    }

    #[test]
    fn a_plain_click_is_the_page_s() {
        let mut app = todo();
        let before = completed(&app);
        assert!(!app.view_source_with(MAIN, ElementState::Pressed, |_| unreachable!()));
        assert!(!app.view_source_with(MAIN, ElementState::Released, |_| unreachable!()));
        // what the shell then does with it: the page toggles the item
        let (x, y) = app.cursor;
        app.view.handle_event(&WidgetEvent::MouseDown { x, y });
        app.view.handle_event(&WidgetEvent::MouseUp { x, y });
        assert_ne!(completed(&app), before);
    }

    #[test]
    fn a_primary_click_on_nothing_opens_nothing() {
        let mut app = todo();
        app.modifiers = PRIMARY;
        // the margin between the sidebar and the page title
        app.cursor = (270.0, 20.0);
        assert!(app.view_source_with(MAIN, ElementState::Pressed, |_| unreachable!()));
        assert!(app.view_source_with(MAIN, ElementState::Released, |_| unreachable!()));
    }

    /// The source at `file:line:column`, from the column to the line's end.
    fn source_at(at: &str) -> (String, String) {
        let mut parts = at.rsplitn(3, ':');
        let column: usize = parts.next().unwrap().parse().unwrap();
        let line: usize = parts.next().unwrap().parse().unwrap();
        let file = parts.next().unwrap().to_owned();
        let src = std::fs::read_to_string(&file).unwrap();
        let text = src
            .lines()
            .nth(line - 1)
            .unwrap()
            .chars()
            .skip(column - 1)
            .collect();
        (file, text)
    }

    /// The lines `path` holds once it holds `n`, or what it has after 5 s.
    fn lines(path: &Path, n: usize) -> Vec<String> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let got: Vec<String> = std::fs::read_to_string(path)
                .unwrap_or_default()
                .lines()
                .map(str::to_owned)
                .collect();
            if got.len() >= n || Instant::now() > deadline {
                return got;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// The window's proof, minus the posted click: the function the click
    /// calls, over the todo's frame and the workspace's literals, with the
    /// real opener running a `PLEV_EDITOR` that only records its arguments.
    #[test]
    fn the_click_hands_the_editor_the_line_under_the_cursor() {
        literals::take_baseline(&source_dirs());
        let dir = std::env::temp_dir().join("plev-view-source-shell");
        std::fs::create_dir_all(&dir).unwrap();
        let (editor, got) = (dir.join("rec.sh"), dir.join("args.txt"));
        let _ = std::fs::remove_file(&got);
        let body = format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\n", got.display());
        std::fs::write(&editor, body).unwrap();
        {
            use std::os::unix::fs::PermissionsExt;
            let exe = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(&editor, exe).unwrap();
        }
        let editor = editor.to_str().unwrap();

        let mut app = todo();
        app.modifiers = PRIMARY;
        let input = SemId::parse("todo.input").unwrap();
        let [x, y, w, h] = compositor(&app).binding(&input).unwrap().rect;
        let title = compositor(&app)
            .layers()
            .iter()
            .flat_map(|l| l.nodes())
            .find_map(|n| match n {
                SceneNode::Text { key, x, y, .. } if key.text == "Todo" => Some((x + 3.0, y + 5.0)),
                _ => None,
            })
            .unwrap();
        for (n, at) in [(x + w / 2.0, y + h / 2.0), title].into_iter().enumerate() {
            app.cursor = at;
            let open = |site: &Site| view_source::open_with(site, Some(editor), None);
            assert!(app.view_source_with(MAIN, ElementState::Pressed, open));
            assert!(app.view_source_with(MAIN, ElementState::Released, |_| unreachable!()));
            assert_eq!(
                lines(&got, n + 1).len(),
                n + 1,
                "the editor ran once per click"
            );
        }

        let got = lines(&got, 2);
        println!("PLEV_EDITOR received:\n{}", got.join("\n"));
        let (file, text) = source_at(&got[0]);
        assert!(file.ends_with("crates/showcase/src/view/apps/app/semantics.rs"));
        assert!(
            text.starts_with("project(b, &ids.input, \"new todo\""),
            "{text}"
        );
        let (file, text) = source_at(&got[1]);
        assert!(file.ends_with("crates/showcase/src/view/pages/table.rs"));
        assert!(text.starts_with("\"Todo\","), "{text}");
    }

    // -- phase c: the literal edited in place. The writer is always a
    // test's: nothing here writes a file of the repository (the write
    // itself is proven on a copy, in tests/edit_literal.rs).

    /// The todo page with the workspace's literals read, as the window's
    /// watcher reads them.
    fn todo_with_literals() -> App {
        literals::take_baseline(&source_dirs());
        todo()
    }

    /// A point inside the first text run drawn with `text`.
    fn on_text(app: &App, text: &str) -> (f32, f32) {
        compositor(app)
            .layers()
            .iter()
            .flat_map(|l| l.nodes())
            .find_map(|n| match n {
                SceneNode::Text { key, x, y, .. } if key.text == text => Some((x + 3.0, y + 5.0)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no text {text:?} in the frame"))
    }

    /// cmd+shift+click at `at`, press and release, both taken.
    fn edit_at(app: &mut App, at: (f32, f32)) {
        app.cursor = at;
        app.modifiers = PRIMARY | ModifiersState::SHIFT;
        assert!(app.view_source_with(MAIN, ElementState::Pressed, |_| unreachable!()));
        assert!(app.view_source_with(MAIN, ElementState::Released, |_| unreachable!()));
        app.modifiers = ModifiersState::empty();
    }

    fn key(app: &mut App, key: Key) -> bool {
        app.edit_key_with(&key, |_, _, _| unreachable!("no write"))
    }

    fn typed(app: &mut App, text: &str) {
        for c in text.chars() {
            let k = match c {
                ' ' => Key::Named(NamedKey::Space),
                c => Key::Character(c.to_string().into()),
            };
            assert!(key(app, k));
        }
    }

    fn toasts(app: &App) -> Vec<String> {
        app.view
            .toasts
            .visible()
            .map(|t| t.message.clone())
            .collect()
    }

    #[test]
    fn a_title_edited_in_place_hands_its_literal_and_the_new_text_to_the_writer() {
        let mut app = todo_with_literals();
        assert!(
            !key(&mut app, Key::Character("x".into())),
            "no edit: the page's key"
        );
        let title = on_text(&app, "Todo");
        edit_at(&mut app, title);

        let edit = app.source.edit.as_ref().expect("a field over the title");
        assert_eq!(edit.was, "Todo");
        assert_eq!(edit.field.text(), "Todo");
        let (file, text) = source_at(&edit.site.to_string());
        assert!(file.ends_with("crates/showcase/src/view/pages/table.rs"));
        assert!(text.starts_with("\"Todo\","), "{text}");
        assert!(edit.bounds.contains(title.0, title.1), "over the text");

        // drawn over the frame, above every layer of it
        let App {
            source,
            view,
            windows,
            ..
        } = &mut app;
        let c = windows.get_mut(MAIN).unwrap().compositor_mut();
        view.render(c);
        source.draw(MAIN, c, &view.theme);
        let top = c.layers().iter().max_by_key(|l| l.z_order).unwrap();
        assert_eq!(top.id, source.edit.as_ref().unwrap().layer);
        let drawn: Vec<&str> = top
            .nodes()
            .iter()
            .filter_map(|n| match n {
                SceneNode::Text { key, .. } => Some(key.text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(drawn, ["Todo"]);

        typed(&mut app, " list");
        assert!(key(&mut app, Key::Named(NamedKey::ArrowLeft)));
        assert!(key(&mut app, Key::Named(NamedKey::Backspace)));
        typed(&mut app, "S");
        assert_eq!(app.source.edit.as_ref().unwrap().field.text(), "Todo liSt");

        let asked = RefCell::new(None);
        let write = |site: &Site, from: &str, to: &str| {
            *asked.borrow_mut() = Some((site.clone(), from.to_owned(), to.to_owned()));
            Ok(Change::Literals(1))
        };
        assert!(app.edit_key_with(&Key::Named(NamedKey::Enter), write));
        let (site, from, to) = asked.into_inner().expect("Enter writes");
        assert!(
            site.file
                .ends_with("crates/showcase/src/view/pages/table.rs")
        );
        assert_eq!((from.as_str(), to.as_str()), ("Todo", "Todo liSt"));
        assert!(app.source.edit.is_none(), "the field is gone");
        assert!(
            toasts(&app).is_empty(),
            "a write that went through is not told"
        );
    }

    #[test]
    fn escape_or_a_press_outside_drops_the_edit_and_writes_nothing() {
        let mut app = todo_with_literals();
        let title = on_text(&app, "Todo");
        edit_at(&mut app, title);
        typed(&mut app, "!");
        assert!(key(&mut app, Key::Named(NamedKey::Escape)));
        assert!(app.source.edit.is_none());

        // a press on the field is the field's, the button's release too
        edit_at(&mut app, title);
        app.cursor = title;
        assert!(app.view_source_with(MAIN, ElementState::Pressed, |_| unreachable!()));
        assert!(app.view_source_with(MAIN, ElementState::Released, |_| unreachable!()));
        assert!(app.source.edit.is_some());
        // one outside drops it, and is the page's
        let item = SemId::parse("todo.item[1]").unwrap();
        let [x, y, _, h] = compositor(&app).binding(&item).unwrap().rect;
        app.cursor = (x + 19.0, y + h / 2.0);
        assert!(!app.view_source_with(MAIN, ElementState::Pressed, |_| unreachable!()));
        assert!(!app.view_source_with(MAIN, ElementState::Released, |_| unreachable!()));
        assert!(app.source.edit.is_none());

        // Enter with nothing changed writes nothing
        edit_at(&mut app, title);
        assert!(key(&mut app, Key::Named(NamedKey::Enter)));
        assert!(app.source.edit.is_none());
    }

    #[test]
    fn a_write_the_engine_refuses_is_told_in_the_window() {
        let mut app = todo_with_literals();
        let title = on_text(&app, "Todo");
        edit_at(&mut app, title);
        typed(&mut app, "s");
        let refuse = |site: &Site, _: &str, _: &str| Err(Refused::Code(site.file.clone()));
        assert!(app.edit_key_with(&Key::Named(NamedKey::Enter), refuse));
        assert!(app.source.edit.is_none());
        let told = toasts(&app);
        assert_eq!(told.len(), 1);
        assert!(
            told[0].ends_with(
                "crates/showcase/src/view/pages/table.rs: its code changed since this build; rebuild, then edit"
            ),
            "{told:?}"
        );
    }

    #[test]
    fn a_text_that_is_not_one_literal_is_not_edited_and_says_why() {
        let mut app = todo_with_literals();
        // a control
        let input = SemId::parse("todo.input").unwrap();
        let [x, y, w, h] = compositor(&app).binding(&input).unwrap().rect;
        edit_at(&mut app, (x + w / 2.0, y + h / 2.0));
        assert!(app.source.edit.is_none());
        // a text several literals hold
        let brand = on_text(&app, "plev");
        edit_at(&mut app, brand);
        assert!(app.source.edit.is_none());
        let told = toasts(&app);
        assert_eq!(told.len(), 2, "{told:?}");
        assert!(told[0].contains("todo.input is a control"), "{told:?}");
        assert!(
            told[1].contains("literals hold it; edit it in the file"),
            "{told:?}"
        );
    }
}
