//! The gallery's command palette (task-104 front H, #163): ⌘K (Ctrl+K
//! off macOS) on any page, or the Search under the sidebar's logo, opens
//! front A's [`CommandDialog`] over the whole window, on the view's one
//! overlay stack (`ActiveOverlay::Palette`): the page and the sidebar
//! beneath are inert while it holds the pointer. It lists every tab,
//! grouped as the sidebar groups them, the themes and the sidebar's pin.
//!
//! Typing filters. Enter, a press on a command, a reader's or an agent's
//! `select` run it, and running one is asked of the shell by the same
//! actor, as if it had asked there: a tab is `shell.page select`, a theme
//! `shell.theme select`, the pin `shell.sidebar.pin toggle`. Escape, a
//! press off the sheet, ⌘K again and a `close` on the dialog close it.
//!
//! Ids: the dialog `palette`, its box `palette.palette` (`set_value`
//! filters), its commands `palette.palette.list[<i>]` (`select`). The
//! Search that opens it is the sidebar's `shell.sidebar.search`.

use accord::{Actor, Request, SemId, SnapshotBuilder, Value};
use engine::styling::theme::{Intent, Theme};
use engine::widgets::form::command::Command;
use engine::widgets::form::command_dialog::CommandDialog;
use engine::widgets::overlay::{OverlayId, OverlayKind};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::dispatcher::ShellTarget;
use super::pages::{Group, PAGES};
use super::{ActiveOverlay, ShowcaseView};

/// The dialog's id.
pub(super) const PALETTE: &str = "palette";

/// The Search's and the dialog's name to a reader.
pub(super) const NAME: &str = "Command palette";

/// What a command does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Cmd {
    /// The page at this index of the table.
    Page(usize),
    /// The theme of this name.
    Theme(&'static str),
    /// The sidebar's pin flips.
    Pin,
}

/// The themes as the palette names them, in the Theme page's order
/// (`foundations::theme::THEMES`, a test holds them equal).
pub(super) const THEMES: [(&str, &str); 14] = [
    ("core", "Core theme"),
    ("hoff", "Hoff theme"),
    ("dark", "Dark theme"),
    ("light", "Light theme"),
    ("catppuccin", "Catppuccin theme"),
    ("dracula", "Dracula theme"),
    ("tokyo-night", "Tokyo Night theme"),
    ("rose-pine", "Rose Pine theme"),
    ("nord", "Nord theme"),
    ("gruvbox", "Gruvbox theme"),
    ("github-dark", "GitHub Dark theme"),
    ("one-dark", "One Dark theme"),
    ("kanagawa", "Kanagawa theme"),
    ("moonlight", "Moonlight theme"),
];

/// The palette's commands, in the order the list holds them, and what
/// each does: the tabs by group, the themes, the pin (`pinned` names it
/// "Unpin").
pub(super) fn commands(pinned: bool) -> (Command, Vec<Cmd>) {
    let mut command = Command::new("Search tabs and actions...");
    let mut cmds = Vec::new();
    for group in Group::ALL {
        let pages: Vec<usize> = (0..PAGES.len())
            .filter(|&i| PAGES[i].group == group)
            .collect();
        let items = pages
            .iter()
            .map(|&i| (PAGES[i].title, Some(PAGES[i].icon), None));
        command = command.group(group.label(), items);
        cmds.extend(pages.into_iter().map(Cmd::Page));
    }
    let themes = THEMES.iter().map(|(_, label)| (*label, Some("sun"), None));
    command = command.group("Themes", themes);
    cmds.extend(THEMES.iter().map(|(name, _)| Cmd::Theme(name)));
    let pin = if pinned {
        ("Unpin the sidebar", Some("panel-left-close"), None)
    } else {
        ("Pin the sidebar", Some("panel-left-open"), None)
    };
    command = command.group("Sidebar", [pin]);
    cmds.push(Cmd::Pin);
    (command, cmds)
}

/// The palette into `b`: the dialog, which Escape closes (`close`), its
/// box and its commands; the focus on the command `focus` names, or on
/// the box.
pub(super) fn project(widget: &CommandDialog, focus: Option<&SemId>, b: &mut SnapshotBuilder) {
    let Some(id) = widget.sem_id() else {
        return;
    };
    widget.project(b, None);
    if let Some(mut node) = b.reopen(id) {
        node.action("close");
    }
    let list = widget.command.list_id();
    let on_item = focus.filter(|f| {
        list.as_ref()
            .is_some_and(|l| f.parent_path().as_ref() == Some(l))
    });
    if let Some(at) = on_item.or(widget.command.sem_id()) {
        b.focus(at);
    }
}

/// What a request asks of the palette.
pub(super) enum Ask {
    Close,
    /// The box's text, set: the list filters.
    Filter(String),
    /// Command `i` of the list, run.
    Run(usize),
}

/// What `req` asks of the palette `widget`, if anything.
pub(super) fn asked(widget: &CommandDialog, req: &Request) -> Option<Ask> {
    let dialog = widget.sem_id()?;
    let action = req.action.as_str();
    if req.target == *dialog {
        return (action == "close").then_some(Ask::Close);
    }
    if widget.command.sem_id() == Some(&req.target) && action == "set_value" {
        let text = match &req.param {
            Some(Value::Text(t)) => t.clone(),
            _ => String::new(),
        };
        return Some(Ask::Filter(text));
    }
    let list = widget.command.list_id()?;
    let i = req
        .target
        .as_str()
        .strip_prefix(list.as_str())?
        .strip_prefix('[')?
        .strip_suffix(']')?
        .parse::<usize>()
        .ok()?;
    (action == "select" && widget.command.matches().contains(&i)).then_some(Ask::Run(i))
}

/// A pointer event on the open palette in `window`: what it asks (a
/// command pressed, a press off the sheet), through the dispatcher, and
/// whether it changed what shows. The box and the list's hover and wheel
/// are the widget's own.
pub(super) fn pointer(
    widget: &mut CommandDialog,
    event: &WidgetEvent,
    theme: &Theme,
    window: Rect,
) -> (Option<(SemId, &'static str)>, bool) {
    let sheet = widget.sheet_rect(theme, window);
    let palette = widget.palette_rect(theme, sheet);
    if let WidgetEvent::MouseDown { x, y } = *event {
        if !sheet.contains(x, y) {
            return (widget.sem_id().map(|id| (id.clone(), "close")), true);
        }
        let item = widget.command.item_at(x, y, palette, theme);
        if let (Some(i), Some(list)) = (item, widget.command.list_id()) {
            return (Some((list.key(i.to_string()), "select")), true);
        }
    }
    let r: EventResult = widget.command.handle_event(event, palette, theme);
    (None, r.changed)
}

impl ShowcaseView {
    /// The palette, open and holding the pointer.
    pub(super) fn palette_mut(&mut self) -> Option<&mut CommandDialog> {
        if !self.palette_open() {
            return None;
        }
        match &mut self.active_overlay {
            Some(ActiveOverlay::Palette { widget, .. }) => Some(widget),
            _ => None,
        }
    }

    pub fn palette_open(&self) -> bool {
        matches!(self.active_overlay, Some(ActiveOverlay::Palette { .. }))
            && self.overlay_holds_pointer()
    }

    /// The palette opens over the window, its caret in the box, with
    /// nothing typed; `opener`, the part of the sidebar the focus was on,
    /// has it back when it closes. `false` when it is open already.
    pub(super) fn open_palette(&mut self, opener: Option<SemId>) -> bool {
        if self.palette_open() {
            return false;
        }
        let pinned = self.shell.dock.as_ref().is_some_and(|d| d.is_pinned());
        let (command, commands) = commands(pinned);
        let mut widget = CommandDialog::new(command);
        if let Some(id) = super::dispatcher::overlay::id(PALETTE) {
            widget = widget.sem(id);
        }
        widget.open();
        self.overlay_focus = None;
        let window = Rect::new(0.0, 0.0, self.width, self.height);
        let s = widget.sheet_rect(&self.theme, window);
        let motion = self.theme.intent_motion(Intent::Neutral);
        let id: OverlayId =
            self.overlay_mgr
                .push_animated(OverlayKind::Anchored, s.x, s.y, s.w, s.h, &motion);
        self.active_overlay = Some(ActiveOverlay::Palette {
            id,
            widget: Box::new(widget),
            commands,
            opener,
        });
        true
    }

    /// After a request or a key that may have closed the palette, open
    /// before (`was_open`): the part of the sidebar that opened it has
    /// the focus back, once.
    pub(super) fn palette_closed(&mut self, was_open: bool) {
        if !was_open || self.palette_open() {
            return;
        }
        let opener = match &mut self.active_overlay {
            Some(ActiveOverlay::Palette { opener, .. }) => opener.take(),
            _ => None,
        };
        if let Some(id) = opener {
            self.focus_sidebar(&id);
        }
    }

    /// ⌘K: the person asks the palette open (the Search's `activate`)
    /// or, open, closed (the dialog's `close`), through the dispatcher.
    pub fn toggle_palette(&mut self) -> bool {
        if self.palette_open() {
            let id = super::dispatcher::overlay::id(PALETTE);
            return id.is_some_and(|id| self.ask_overlay(&id, "close"));
        }
        if self.overlay_holds_pointer() {
            // another overlay holds the window: ⌘K waits for it
            return false;
        }
        match self.nav.search.sem_id().cloned() {
            Some(id) => self.ask_sidebar(&id, "activate"),
            None => self.open_palette(None),
        }
    }

    /// A character typed while the palette is open: its box's.
    pub(super) fn palette_type(&mut self, key: &str) -> bool {
        match self.palette_mut() {
            Some(p) => {
                p.command.search.input.insert(key);
                true
            }
            None => false,
        }
    }

    /// An editing key while the palette is open: its box's (Tab stays in
    /// the palette).
    pub(super) fn palette_edit(&mut self, key: super::EditKey) -> bool {
        match self.palette_mut() {
            Some(p) => {
                if key != super::EditKey::Tab {
                    p.command.search.input.edit(key.into());
                }
                true
            }
            None => false,
        }
    }

    /// Up and Down while the palette is open: the highlight moves.
    pub(super) fn palette_nav(&mut self, down: bool) -> bool {
        self.palette_mut().is_some_and(|p| p.command.nav(down))
    }

    /// Enter while the palette is open: the person asks the highlighted
    /// command run.
    pub(super) fn palette_enter(&mut self) -> bool {
        let Some(p) = self.palette_mut() else {
            return false;
        };
        let target = p
            .command
            .highlighted()
            .zip(p.command.list_id())
            .map(|(i, list)| list.key(i.to_string()));
        match target {
            Some(id) => self.ask_overlay(&id, "select"),
            // nothing matches: Enter does nothing, the palette stays
            None => true,
        }
    }

    /// A command ran (`dispatcher::overlay`, the palette closing): what
    /// it does, asked of the shell by `actor`, who asked it of the
    /// palette.
    pub(super) fn run_palette(&mut self, cmd: Cmd, actor: Actor) -> bool {
        match cmd {
            Cmd::Page(i) => {
                let Some(id) = self.panes.shell.id(ShellTarget::Page) else {
                    return false;
                };
                let title = Value::Text(PAGES[i].title.to_owned());
                self.shell_as(actor, &id, "select", Some(title))
            }
            Cmd::Theme(name) => {
                let Some(id) = self.panes.shell.id(ShellTarget::Theme) else {
                    return false;
                };
                self.shell_as(actor, &id, "select", Some(Value::Text(name.into())))
            }
            Cmd::Pin => {
                let Some(id) = self.shell.dock.as_ref().and_then(|d| d.pin_id()) else {
                    return false;
                };
                self.shell_as(actor, &id, "toggle", None)
            }
        }
    }

    /// `action` on the shell's `target`, observed and asked by `actor`.
    fn shell_as(
        &mut self,
        actor: Actor,
        target: &SemId,
        action: &str,
        param: Option<Value>,
    ) -> bool {
        let seen = match self.shell_semantics() {
            Ok(seen) => seen,
            Err(e) => {
                log::warn!("palette: the shell's projection does not validate: {e}");
                return false;
            }
        };
        match Request::observe(&seen, actor, target, action, param) {
            Ok(req) => self.act_shell(req).is_some(),
            Err(e) => {
                log::debug!("palette: {target} {action}: {e}");
                false
            }
        }
    }
}

#[cfg(test)]
mod tests;
