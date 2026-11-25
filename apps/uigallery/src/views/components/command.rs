//! Command page: a command palette inline, the kit's Search
//! (`components/Search/index.tsx`) over the list its select opens, as the
//! kit's showcase draws a component (groups with a title and a rule,
//! cards behind a chip naming their states). The widget is
//! `engine::widgets::form::command::Command`. The palette in a dialog
//! is `command_dialog::CommandDialog`, the same palette on the modal's
//! sheet: a card holds its Search with the `⌘ K` (a press opens the
//! palette in the card, the card standing for the window), and one shows
//! it open.
//!
//! The page is a form page (`field::FormPage`): its ids under
//! `command_page`, a palette projected as its search box with its list
//! (`….list`) of the commands shown, each with a `select` that runs it.
//! Typing filters, Up and Down highlight, Enter runs, Escape blurs.

#[cfg(test)]
use std::any::Any;

use accord::{SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::feedback::Modal;
use engine::widgets::form::TextField;
use engine::widgets::form::command::{Command, CommandRow};
use engine::widgets::form::command_dialog::CommandDialog;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::field::spec::{CardWidget, Part};
use super::field::{FormPage, Group, card};

fn palette() -> Command {
    Command::new("Type a command or search...")
        .group(
            "File",
            [
                ("New file", Some("file"), Some("⌘ N")),
                ("Open folder", Some("folder-open"), Some("⌘ O")),
                ("Save", Some("save"), Some("⌘ S")),
            ],
        )
        .group(
            "Go to",
            [
                ("Home", Some("house"), None),
                ("Settings", Some("settings"), Some("⌘ ,")),
                ("Terminal", Some("terminal"), Some("⌃ `")),
            ],
        )
}

/// The Command tab.
pub fn page() -> FormPage {
    let groups = vec![
        Group {
            title: "Command",
            columns: 2,
            cards: vec![
                card("Default, Hover, Focus, Typing", palette()),
                card("Filtered", palette().with_query("s")),
            ],
        },
        Group {
            title: "Empty, Disabled",
            columns: 2,
            cards: vec![
                card("No results", palette().with_query("zebra")),
                card("Disabled", palette().disabled(true)),
            ],
        },
        Group {
            title: "Dialog",
            columns: 2,
            cards: vec![
                card("Closed, Press, ⌘ K", CommandDialog::new(palette())),
                card("Open, Escape", CommandDialog::new(palette()).opened()),
            ],
        },
        Group {
            title: "Content",
            columns: 1,
            cards: vec![card(
                "Long label",
                Command::new("Search the workspace...").group(
                    "Recent",
                    [
                        (
                            "Open the quarterly report of the molecular biology laboratory",
                            Some("file"),
                            Some("⌘ 1"),
                        ),
                        ("Settings", Some("settings"), None),
                    ],
                ),
            )],
        },
    ];
    FormPage::new("command_page", groups)
}

impl CardWidget for Command {
    fn with_sem(self: Box<Self>, id: SemId) -> Box<dyn CardWidget> {
        Box::new((*self).sem(id))
    }

    /// As wide as its widest command asks, from the palette's width up to
    /// the card's room.
    fn size(&self, room: f32, theme: &Theme) -> (f32, f32) {
        let floor = theme.form.group.palette_width.min(room);
        let w = self.preferred_width(theme).clamp(floor, room.max(floor));
        (w, self.height_for(w, theme))
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        Command::render(self, c, rect, theme);
    }

    /// Named by its placeholder.
    fn project(&self, b: &mut SnapshotBuilder, parent: &SemId) {
        if let Some(id) = self.sem_id() {
            let name = self.search.input.input.placeholder.clone();
            Command::project(self, b, id, name, Some(parent));
        }
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        Command::handle_event(self, event, rect, theme)
    }

    fn text(&self) -> Option<&TextField> {
        Some(&self.search.input)
    }

    fn text_mut(&mut self) -> Option<&mut TextField> {
        Some(&mut self.search.input)
    }

    fn enter(&mut self) -> bool {
        Command::enter(self)
    }

    fn nav(&mut self, down: bool) -> bool {
        Command::nav(self, down)
    }

    fn part_id(&self, part: Part) -> Option<SemId> {
        match part {
            Part::Box => self.sem_id().cloned(),
            Part::Action => self.search.action_id(),
            Part::Option(i) => self.list_id().map(|l| l.key(i.to_string())),
            _ => None,
        }
    }

    /// The search icon's button, or a command (a reader's, or Tab).
    fn set_part_focus(&mut self, part: Option<Part>) {
        self.search.set_action_focus(part == Some(Part::Action));
        let item = match part {
            Some(Part::Option(i)) => Some(i),
            _ => None,
        };
        self.focus_item(item);
    }

    fn part_focus(&self) -> Option<Part> {
        if self.search.has_action_focus() {
            return Some(Part::Action);
        }
        self.focused_item().map(Part::Option)
    }

    /// The box, its search button, or a command the list shows.
    fn part(&self, id: &SemId) -> Option<Part> {
        if self.sem_id() == Some(id) {
            return Some(Part::Box);
        }
        if self.search.action_id().as_ref() == Some(id) {
            return Some(Part::Action);
        }
        self.rows()
            .into_iter()
            .filter_map(|r| match r {
                CommandRow::Item(i) => Some(Part::Option(i)),
                CommandRow::Heading(_) => None,
            })
            .find(|p| self.part_id(*p).as_ref() == Some(id))
    }

    fn on_box(&self, x: f32, y: f32, rect: Rect, theme: &Theme) -> bool {
        let search = self.layout(rect, theme).search;
        search.contains(x, y) && !self.search.on_action(x, y, search, theme)
    }

    fn ime_area(&self, rect: Rect, theme: &Theme) -> Option<Rect> {
        Some(self.search.ime_area(self.layout(rect, theme).search, theme))
    }

    /// The box's `set_value` filters, as typing does; a command's
    /// `select` runs it, as a press on its row does.
    fn run(&mut self, part: Part, action: &str, param: Option<&Value>) -> bool {
        match (part, action, param) {
            (Part::Box, "set_value", Some(Value::Text(text))) => self.search.input.set_value(text),
            (Part::Action, "activate", None) => true,
            (Part::Option(i), "select", None) => Command::run(self, i),
            _ => false,
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The card stands for the window: closed, the Search; open, the sheet
/// as a modal is in a window that wide. Open, the rest is the palette's.
impl CardWidget for CommandDialog {
    fn with_sem(self: Box<Self>, id: SemId) -> Box<dyn CardWidget> {
        Box::new((*self).sem(id))
    }

    fn size(&self, room: f32, theme: &Theme) -> (f32, f32) {
        if !self.is_open() {
            let w = theme.form.group.width.min(room);
            return (w, self.trigger_height(theme));
        }
        let w = Modal::width(theme, room).min(room);
        let s = self.sheet_rect(theme, Rect::new(0.0, 0.0, w, 0.0));
        (s.w, s.h)
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        if self.is_open() {
            self.render_in(c, theme, rect);
        } else {
            CommandDialog::render(self, c, rect, rect, theme);
        }
    }

    fn project(&self, b: &mut SnapshotBuilder, parent: &SemId) {
        CommandDialog::project(self, b, Some(parent));
    }

    /// A press off the card is off the window the card stands for: not
    /// the dialog's.
    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        if let WidgetEvent::MouseDown { x, y } = *event
            && !rect.contains(x, y)
        {
            return EventResult::IGNORED;
        }
        CommandDialog::handle_event(self, event, rect, rect, theme)
    }

    fn text(&self) -> Option<&TextField> {
        self.is_open().then_some(&self.command.search.input)
    }

    fn text_mut(&mut self) -> Option<&mut TextField> {
        self.is_open().then_some(&mut self.command.search.input)
    }

    fn enter(&mut self) -> bool {
        CommandDialog::enter(self)
    }

    fn escape(&mut self) -> bool {
        CommandDialog::escape(self)
    }

    fn nav(&mut self, down: bool) -> bool {
        self.is_open() && self.command.nav(down)
    }

    /// Closed, the trigger is the action; open, the palette's parts.
    fn part_id(&self, part: Part) -> Option<SemId> {
        match (self.is_open(), part) {
            (false, Part::Action) => self.trigger_id().cloned(),
            (false, _) => None,
            (true, part) => CardWidget::part_id(&self.command, part),
        }
    }

    /// Closed, the trigger's focus is its search button's ring.
    fn set_part_focus(&mut self, part: Option<Part>) {
        if self.is_open() {
            CardWidget::set_part_focus(&mut self.command, part);
        } else {
            self.trigger.set_action_focus(part == Some(Part::Action));
        }
    }

    fn part_focus(&self) -> Option<Part> {
        if !self.is_open() {
            return self.trigger.has_action_focus().then_some(Part::Action);
        }
        CardWidget::part_focus(&self.command)
    }

    fn part(&self, id: &SemId) -> Option<Part> {
        if !self.is_open() {
            return (self.trigger_id() == Some(id)).then_some(Part::Action);
        }
        CardWidget::part(&self.command, id)
    }

    fn on_box(&self, x: f32, y: f32, rect: Rect, theme: &Theme) -> bool {
        self.is_open()
            && CardWidget::on_box(&self.command, x, y, self.palette_rect(theme, rect), theme)
    }

    fn ime_area(&self, rect: Rect, theme: &Theme) -> Option<Rect> {
        let palette = self.palette_rect(theme, rect);
        self.is_open()
            .then(|| CardWidget::ime_area(&self.command, palette, theme))
            .flatten()
    }

    /// The trigger's `activate` opens; open, the palette's own, and a
    /// command run closes.
    fn run(&mut self, part: Part, action: &str, param: Option<&Value>) -> bool {
        if !self.is_open() {
            let open = part == Part::Action && action == "activate" && param.is_none();
            if open {
                self.open();
            }
            return open;
        }
        let ran = CardWidget::run(&mut self.command, part, action, param);
        if ran && matches!(part, Part::Option(_)) {
            self.close();
        }
        ran
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use accord::Role;
    use engine::widgets::prelude::{Rect, WidgetEvent};

    use super::*;
    use crate::views::components::Component;
    use crate::views::text_input::host::TextHost;

    fn content() -> Rect {
        Rect::new(288.0, 80.0, 900.0, 2400.0)
    }

    fn palette_at(p: &FormPage, at: (usize, usize)) -> &Command {
        p.spec(at).and_then(|s| s.get()).expect("a palette")
    }

    /// From 280 to 1600 px every 8, in the core and the HOFF themes:
    /// every palette inside its card and under its chip, its list under
    /// its box.
    #[test]
    fn every_palette_stays_in_its_card_from_280_to_1600() {
        let p = page();
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            for w in (280..=1600).step_by(8).map(|w| w as f32) {
                let content = Rect::new(0.0, 0.0, w, 800.0);
                for (laid, group) in p.layout(content, &theme).iter().zip(&p.groups) {
                    for ((card, rect), c) in laid.cards.iter().zip(&group.cards) {
                        let what = format!("{w}: {} / {}", group.title, c.title);
                        assert!(card.right() <= w + 0.5, "{what}: card");
                        assert!(rect.x >= card.x && rect.right() <= card.right() + 0.5);
                        assert!(rect.bottom() <= card.bottom() + 0.5, "{what}: under");
                        let dialog: Option<&CommandDialog> = c.spec.get();
                        let (cmd, at) = match dialog {
                            Some(d) if !d.is_open() => continue,
                            Some(d) => (&d.command, d.palette_rect(&theme, *rect)),
                            None => (c.spec.get::<Command>().unwrap(), *rect),
                        };
                        let l = cmd.layout(at, &theme);
                        assert!(l.search.x >= rect.x && l.search.right() <= rect.right() + 0.5);
                        assert!(
                            l.list.y >= l.search.bottom() && l.list.bottom() <= rect.bottom() + 0.5,
                            "{what}: list"
                        );
                    }
                }
            }
        }
    }

    /// The disabled palette has no highlighted command (nothing there
    /// runs), and a press on a row runs nothing.
    #[test]
    fn a_disabled_palette_offers_nothing_to_press() {
        let theme = Theme::hoff_core();
        let mut p = page();
        let cmd = palette_at(&p, (1, 1));
        assert!(cmd.highlighted().is_none());
        let rect = p.widget_rect((1, 1), content(), &theme).unwrap();
        let list = cmd.layout(rect, &theme).list;
        let pr = &theme.form.select.panel;
        let (x, y) = (list.x + 30.0, list.y + pr.pad + pr.row_h * 1.5);
        p.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
        assert!(palette_at(&p, (1, 1)).ran().is_none());
    }

    /// A palette is as wide as its widest command asks, from the token's
    /// width up to the card's room: the long label is whole where the card
    /// has room for it.
    #[test]
    fn a_palette_is_as_wide_as_its_widest_command() {
        let theme = Theme::hoff_core();
        let p = page();
        let long = palette_at(&p, (3, 0));
        let wants = long.preferred_width(&theme);
        assert!(
            wants > theme.form.group.palette_width,
            "the long label asks for more"
        );
        let rect = p.widget_rect((3, 0), content(), &theme).unwrap();
        assert!((rect.w - wants).abs() < 0.5, "{} for {wants}", rect.w);
        let short = p.widget_rect((0, 0), content(), &theme).unwrap();
        assert!(short.w >= theme.form.group.palette_width);
    }

    /// The dialog's card: a press on its Search opens the palette there
    /// with the caret in its box; typing filters, Escape closes it.
    #[test]
    fn a_press_on_the_search_opens_the_dialog_and_escape_closes_it() {
        let theme = Theme::hoff_core();
        let mut p = page();
        let at = (2, 0);
        let dialog = |p: &FormPage| -> bool {
            let d: &CommandDialog = p.spec(at).and_then(|s| s.get()).expect("a dialog");
            d.is_open()
        };
        assert!(!dialog(&p));
        let rect = p.widget_rect(at, content(), &theme).unwrap();
        let (x, y) = rect.center();
        p.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
        assert!(dialog(&p), "a press on the Search opens it");
        assert_eq!(p.focused(), Some(at));
        assert!(p.handle_text("s"));
        assert!(p.handle_escape());
        assert!(!dialog(&p), "Escape closes it");
    }

    /// A click focuses the box; typing filters, Down highlights the next
    /// command shown, Enter runs it; a press on a row runs that one.
    #[test]
    fn typing_filters_and_enter_runs_the_highlighted_command() {
        let theme = Theme::hoff_core();
        let mut p = page();
        let rect = p.widget_rect((0, 0), content(), &theme).unwrap();
        let search = palette_at(&p, (0, 0)).layout(rect, &theme).search;
        let (x, y) = search.center();
        p.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
        assert_eq!(p.focused(), Some((0, 0)));
        assert!(p.handle_text("s"));
        let shown = palette_at(&p, (0, 0)).matches();
        assert_eq!(shown, [2, 4], "Save, Settings");
        assert!(p.handle_nav(true));
        assert!(p.handle_enter());
        assert_eq!(
            palette_at(&p, (0, 0)).ran().map(|c| c.label.as_str()),
            Some("Settings")
        );
        let cmd = palette_at(&p, (0, 0));
        let at = cmd
            .rows()
            .iter()
            .position(|r| *r == CommandRow::Item(2))
            .unwrap();
        let pr = &theme.form.select.panel;
        let list = cmd.layout(rect, &theme).list;
        let (rx, ry) = (
            list.x + 30.0,
            list.y + pr.pad + pr.row_h * (at as f32 + 0.5),
        );
        p.handle_event(&WidgetEvent::MouseDown { x: rx, y: ry }, content(), &theme);
        assert_eq!(
            palette_at(&p, (0, 0)).ran().map(|c| c.label.as_str()),
            Some("Save")
        );
        assert!(palette_at(&p, (1, 0)).rows().is_empty(), "no results");
    }

    /// A palette projects its box and its list; a command's `select` runs
    /// it.
    #[test]
    fn the_palette_projects_its_commands_and_a_select_runs_one() {
        let mut p = page();
        let root = p.root().expect("the root parses").clone();
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Command");
        p.project(&mut b, &root);
        let snap = b.finish().expect("the page projects");
        let list = SemId::parse("command_page.group[0].card[1].list").unwrap();
        assert_eq!(snap.get(&list).map(|n| n.role), Some(Role::List));
        let save = list.key("2");
        let node = snap.get(&save).expect("Save matches \"s\"");
        assert_eq!(node.description.as_deref(), Some("⌘ S"));
        assert!(snap.get(&list.key("0")).is_none(), "New file does not");
        let req = accord::Request::observe(
            &snap,
            accord::Actor::Test { name: "t".into() },
            &save,
            "select",
            None,
        )
        .unwrap();
        assert!(p.run(&req).is_ok());
        assert_eq!(
            palette_at(&p, (0, 1)).ran().map(|c| c.label.as_str()),
            Some("Save")
        );
    }
}
