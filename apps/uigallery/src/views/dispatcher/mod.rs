//! Dispatch to the active pane: its natural height, its share of an
//! event, its visible-only tick and its drawing. One match per verb over
//! `pages::Pane`, so the rest of the view never names a pane's field.

use accord::{Millis, Receipt, ReceiptSink, Request, SemId, Snapshot, SnapshotBuilder, Value};
use engine::control::input::FileDrop;
use engine::drawing::compositor::Compositor;
use engine::runtime::platform::capture::Capture;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{EmptyState, EventResult, Rect, Wake, WidgetEvent};

use super::{Layers, Pane, ShowcaseView, panel};
use component::After;

impl ShowcaseView {
    /// Natural (unclipped) height of the active page's content.
    pub(super) fn page_content_height(&self, content: Rect) -> f32 {
        self.pane_content_height(content) + self.note_h()
    }

    /// The active pane's natural height in `content`.
    fn pane_content_height(&self, content: Rect) -> f32 {
        let t = &self.theme;
        match self.pane() {
            None => t.size.card_min_w / 2.0,
            Some(pane) => match pane {
                Pane::Theme => self.panes.themes.content_height(content, t),
                Pane::Typography => self.panes.typography.content_height(content),
                Pane::Icons => self.panes.icons.content_height(content),
                Pane::Effects => self.panes.effects.content_height(content, t),
                Pane::Avatar => self.panes.avatar.content_height(content, t),
                Pane::Badge => self.panes.badge.content_height(content, t),
                Pane::Breadcrumb => self.panes.breadcrumb.content_height(content, t),
                Pane::Button => self.panes.button.content_height(content, t),
                Pane::Card => self.panes.card.content_height(content, t),
                Pane::Chart => self.panes.chart.content_height(content),
                Pane::Checkbox => self.panes.checkbox.content_height(content, t),
                Pane::Chip => self.panes.chip.content_height(content, t),
                Pane::CodeBlock => self.panes.code_block.content_height(content, t),
                Pane::ContextMenu => self.panes.context_menu.content_height(content, t),
                Pane::Dialog => self.panes.dialog.content_height(content, t),
                Pane::Empty => self.panes.empty.content_height(content, t),
                Pane::Graph => self.panes.graph.content_height(content, t),
                Pane::IconButton => self.panes.icon_button.content_height(content, t),
                Pane::Input => self.panes.input.content_height(content, t),
                Pane::NavigationMenu => self.panes.navigation_menu.content_height(content, t),
                Pane::Panel => self.panes.panel.content_height(content, t),
                Pane::PanelHeader => self.panes.panel_header.content_height(content, t),
                Pane::Progress => self.panes.progress.content_height(content, t),
                Pane::Resizable => self.panes.resizable.content_height(content, t),
                Pane::ScrollArea => self.panes.scroll_area.content_height(content, t),
                Pane::Select => self.panes.select.content_height(content, t),
                Pane::Separator => self.panes.separator.content_height(content, t),
                Pane::Sidebar => self.panes.sidebar.content_height(content, t),
                Pane::Skeleton => self.panes.skeleton.content_height(content, t),
                Pane::Slider => self.panes.slider.content_height(content, t),
                Pane::Spinner => self.panes.spinner.content_height(content, t),
                Pane::Stat => self.panes.stat.content_height(content, t),
                Pane::Switch => self.panes.switch.content_height(content, t),
                Pane::Table => self.panes.table.content_height(content, t),
                Pane::Tabs => self.panes.tabs.content_height(content, t),
                Pane::Toast => self.panes.toast.content_height(content, t),
                Pane::Tooltip => self.panes.tooltip.content_height(content, t),
                Pane::Tree => self.panes.tree.content_height(content, t),
                // task-104 front A, fields (#154)
                Pane::Field => self.panes.field.content_height(content, t),
                // task-104 front B, overlays (#155)
                Pane::Popover => self.panes.popover.content_height(content, t),
                // task-104 front C, conversation and feedback (#156)
                Pane::Message => self.panes.message.content_height(content, t),
                // task-104 front D, disclosure and display (#157)
                Pane::Accordion => self.panes.accordion.content_height(content, t),
                Pane::AspectRatio => self.panes.aspect_ratio.content_height(content, t),
                Pane::Collapsible => self.panes.collapsible.content_height(content, t),
                Pane::Item => self.panes.item.content_height(content, t),
                Pane::Kbd => self.panes.kbd.content_height(content, t),
                // task-104 front E, selection and dates (#158)
                Pane::Toggle => self.panes.toggle.content_height(content, t),
                Pane::Todo => self.panes.todo.content_height(content),
                Pane::Surface => self
                    .panes
                    .surface
                    .content_height(content, t, &self.panes.todo),
                Pane::Silences => self.panes.silences.content_height(content, t),
                Pane::Files => self.panes.files.content_height(content, t),
                Pane::Capture => self.panes.capture.content_height(content, t),
                Pane::Dock => self.panes.dock.content_height(content),
                Pane::Editor => self.panes.editor.content_height(content),
                Pane::Git => self.panes.git.content_height(content),
                Pane::Builder => self.panes.builder.content_height(content),
            },
        }
    }

    /// The active pane's share of a pointer event in the main window,
    /// with the view-level effects a page asks for (a dialog, a toast, a
    /// theme). The events in the page's own window come through
    /// `detached_event`, laid out for that window.
    pub(super) fn page_event(&mut self, event: &WidgetEvent) -> EventResult {
        let content = self.page_rect();
        self.pane_event(event, content)
    }

    /// The active pane's share of a pointer event, laid out in `content`
    /// (the main window's page rect, or the detached window's).
    fn pane_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        let t = &self.theme;
        match self.pane() {
            None => EventResult::IGNORED,
            Some(pane) => match pane {
                // A card picked is the shell's `shell.theme`, the same
                // capability as the T key and the menu, through the
                // dispatcher.
                Pane::Theme => {
                    let (r, picked) = self.panes.themes.handle_event(event, content);
                    if let Some(name) = picked {
                        let name = Value::Text(name.to_owned());
                        self.ask_shell(ShellTarget::Theme, "select", Some(name));
                    }
                    r
                }
                Pane::Typography => self.panes.typography.handle_event(event, content),
                Pane::Icons => self.panes.icons.handle_event(event, content),
                Pane::Effects => self.panes.effects.handle_event(event, content),
                Pane::Avatar => self.panes.avatar.handle_event(event, content, t),
                Pane::Badge => self.panes.badge.handle_event(event, content, t),
                Pane::Breadcrumb => self.panes.breadcrumb.handle_event(event, content, t),
                Pane::Button => self.panes.button.handle_event(event, content, t),
                Pane::Card => self.panes.card.handle_event(event, content, t),
                Pane::Chart => self.panes.chart.handle_event(event, content),
                Pane::Checkbox => self.panes.checkbox.handle_event(event, content, t),
                Pane::Chip => self.panes.chip.handle_event(event, content, t),
                Pane::CodeBlock => self.panes.code_block.handle_event(event, content, t),
                Pane::ContextMenu => self.panes.context_menu.handle_event(event, content, t),
                Pane::Dialog => {
                    let (r, open) = self.panes.dialog.handle_event(event, content, t);
                    if let Some(destructive) = open {
                        self.after_component(After::Modal { destructive });
                    }
                    r
                }
                Pane::Empty => self.panes.empty.handle_event(event, content, t),
                Pane::Graph => self.panes.graph.handle_event(event, content, t),
                Pane::IconButton => self.panes.icon_button.handle_event(event, content, t),
                Pane::Input => self.panes.input.handle_event(event, content, t),
                Pane::NavigationMenu => self.panes.navigation_menu.handle_event(event, content, t),
                Pane::Panel => self.panes.panel.handle_event(event, content, t),
                Pane::PanelHeader => self.panes.panel_header.handle_event(event, content, t),
                Pane::Progress => self.panes.progress.handle_event(event, content, t),
                Pane::Resizable => self.panes.resizable.handle_event(event, content, t),
                Pane::ScrollArea => self.panes.scroll_area.handle_event(event, content, t),
                Pane::Select => self.panes.select.handle_event(event, content, t),
                Pane::Separator => self.panes.separator.handle_event(event, content, t),
                Pane::Sidebar => self.panes.sidebar.handle_event(event, content, t),
                Pane::Skeleton => self.panes.skeleton.handle_event(event, content, t),
                Pane::Slider => self.panes.slider.handle_event(event, content, t),
                Pane::Spinner => self.panes.spinner.handle_event(event, content, t),
                Pane::Stat => self.panes.stat.handle_event(event, content, t),
                Pane::Switch => self.panes.switch.handle_event(event, content, t),
                Pane::Table => self.panes.table.handle_event(event, content, t),
                Pane::Tabs => self.panes.tabs.handle_event(event, content, t),
                Pane::Toast => {
                    let (r, queued) = self.panes.toast.handle_event(event, content, t);
                    if let Some(intent) = queued {
                        self.after_component(After::Toast(intent));
                    }
                    r
                }
                Pane::Tooltip => self.panes.tooltip.handle_event(event, content, t),
                Pane::Tree => self.panes.tree.handle_event(event, content, t),
                // task-104 front A, fields (#154)
                Pane::Field => self.panes.field.handle_event(event, content, t),
                // task-104 front B, overlays (#155)
                Pane::Popover => {
                    let bounds = self.popover_bounds(content);
                    self.panes.popover.handle_event(event, content, bounds, t)
                }
                // task-104 front C, conversation and feedback (#156)
                Pane::Message => self.panes.message.handle_event(event, content, t),
                // task-104 front D, disclosure and display (#157)
                Pane::Accordion => self.panes.accordion.handle_event(event, content, t),
                Pane::AspectRatio => self.panes.aspect_ratio.handle_event(event, content, t),
                Pane::Collapsible => self.panes.collapsible.handle_event(event, content, t),
                Pane::Item => self.panes.item.handle_event(event, content, t),
                Pane::Kbd => self.panes.kbd.handle_event(event, content, t),
                // task-104 front E, selection and dates (#158)
                Pane::Toggle => self.panes.toggle.handle_event(event, content, t),
                Pane::Todo => {
                    let inert = self.overlay_holds_pointer();
                    self.panes.todo.set_inert(inert);
                    self.panes.todo.handle_event(event, content, t)
                }
                Pane::Surface => {
                    let inert = self.overlay_holds_pointer();
                    let todo = &mut self.panes.todo;
                    self.panes
                        .surface
                        .handle_event(event, content, t, todo, inert)
                }
                Pane::Silences => {
                    let inert = self.overlay_holds_pointer();
                    let todo = &mut self.panes.todo;
                    self.panes
                        .silences
                        .handle_event(event, content, t, todo, inert)
                }
                Pane::Files => {
                    let inert = self.overlay_holds_pointer();
                    let todo = &mut self.panes.todo;
                    self.panes
                        .files
                        .handle_event(event, content, t, todo, inert)
                }
                Pane::Capture => {
                    let inert = self.overlay_holds_pointer();
                    let todo = &mut self.panes.todo;
                    self.panes
                        .capture
                        .handle_event(event, content, t, todo, inert)
                }
                Pane::Dock => self.panes.dock.handle_event(event, content),
                Pane::Editor => self.panes.editor.handle_event(event, content),
                Pane::Git => self.panes.git.handle_event(event, content),
                Pane::Builder => self.panes.builder.handle_event(event, content),
            },
        }
    }

    /// The active page's share of a file drag over the window (the
    /// engine's event, forwarded whole by the shell): the Files page
    /// receives it; the other pages take no files.
    pub(super) fn page_file_drop(&mut self, drop: &FileDrop) -> EventResult {
        let content = self.page_rect();
        let t = &self.theme;
        match self.pane() {
            Some(Pane::Files) => {
                let inert = self.overlay_holds_pointer();
                let todo = &mut self.panes.todo;
                self.panes
                    .files
                    .handle_file_drop(drop, content, t, todo, inert)
            }
            _ => EventResult::IGNORED,
        }
    }

    /// Motion that would run forever ticks only while visible: the
    /// dock's caret, the chart reveal, spinners, breathing skeletons,
    /// the todo caret, the editor. Continuous motion asks for every
    /// frame; a caret asks for its next flip. While the page shows in a
    /// window of its own too, its ask goes to that window as well
    /// (`detached_wake`): both windows draw its motion, each its own frame.
    ///
    /// The capture page is told on every wake whether it is the active
    /// one, shown or not: a device it runs stops as the page is left.
    pub(super) fn page_wake(&mut self, dt: f32) -> Wake {
        let capture = self
            .panes
            .capture
            .wake(dt, self.pane() == Some(Pane::Capture));
        let wake = match self.pane() {
            Some(Pane::Dock) => Wake::moving(self.panes.dock.tick(dt)),
            Some(Pane::Chart) => Wake::moving(self.panes.chart.tick(dt)),
            Some(Pane::Spinner) => Wake::moving(self.panes.spinner.tick(dt)),
            Some(Pane::Skeleton) => Wake::moving(self.panes.skeleton.tick(dt)),
            Some(Pane::Todo) => self.panes.todo.wake(dt),
            Some(Pane::Surface) => self.panes.surface.wake(dt, &mut self.panes.todo),
            Some(Pane::Silences) => self.panes.silences.wake(dt, &mut self.panes.todo),
            Some(Pane::Capture) => capture,
            Some(Pane::Editor) => self.panes.editor.wake(dt),
            // task-104 front A, fields (#154)
            Some(Pane::Field) => self.panes.field.wake(dt),
            // task-104 front B, overlays (#155)
            Some(Pane::Popover) => {
                let (content, viewport) = (self.page_rect(), self.content_rect());
                let popover = &mut self.panes.popover;
                Wake::moving(popover.sync(&mut self.overlay_mgr, content, viewport, &self.theme))
            }
            // task-104 front C, conversation and feedback (#156)
            // task-104 front D, disclosure and display (#157)
            Some(Pane::Accordion) => self.panes.accordion.wake(dt),
            Some(Pane::Collapsible) => self.panes.collapsible.wake(dt),
            // task-104 front E, selection and dates (#158)
            _ => Wake::idle(),
        };
        if let Some(detached) = self.panes.detached.as_mut() {
            detached.wake = detached.wake.merge(wake);
        }
        wake
    }

    /// Draw the active page into `content` of the main window (a
    /// placeholder names its slot).
    pub(super) fn page_render(
        &mut self,
        c: &mut Compositor,
        layers: Layers,
        content: Rect,
        theme: &Theme,
    ) {
        self.pane_render(c, layers, content, theme);
    }

    /// Draw the active pane into `content`, whichever window's.
    fn pane_render(&mut self, c: &mut Compositor, layers: Layers, content: Rect, theme: &Theme) {
        match self.pane() {
            None => self.render_placeholder(c, content, theme),
            Some(pane) => match pane {
                Pane::Theme => self
                    .panes
                    .themes
                    .render(c, content, theme, &self.theme_name),
                Pane::Typography => self.panes.typography.render(c, content, theme),
                Pane::Icons => self.panes.icons.render(c, content, theme),
                Pane::Effects => self.panes.effects.render(c, content, theme),
                Pane::Avatar => self.panes.avatar.render(c, content, theme),
                Pane::Badge => self.panes.badge.render(c, content, theme),
                Pane::Breadcrumb => self.panes.breadcrumb.render(c, content, theme),
                Pane::Button => self.panes.button.render(c, content, theme),
                Pane::Card => self.panes.card.render(c, content, theme),
                Pane::Chart => self.panes.chart.render(c, content, theme),
                Pane::Checkbox => self.panes.checkbox.render(c, content, theme),
                Pane::Chip => self.panes.chip.render(c, content, theme),
                Pane::CodeBlock => self.panes.code_block.render(c, content, theme),
                Pane::ContextMenu => self.panes.context_menu.render(c, content, theme),
                Pane::Dialog => self.panes.dialog.render(c, content, theme),
                Pane::Empty => self.panes.empty.render(c, content, theme),
                Pane::Graph => self.panes.graph.render(c, content, theme),
                Pane::IconButton => self.panes.icon_button.render(c, content, theme),
                Pane::Input => self.panes.input.render(c, content, theme),
                Pane::NavigationMenu => self.panes.navigation_menu.render(c, content, theme),
                Pane::Panel => self.panes.panel.render(c, content, theme),
                Pane::PanelHeader => self.panes.panel_header.render(c, content, theme),
                Pane::Progress => self.panes.progress.render(c, content, theme),
                Pane::Resizable => self.panes.resizable.render(c, content, theme),
                Pane::ScrollArea => self
                    .panes
                    .scroll_area
                    .render(c, layers.list, content, theme),
                Pane::Select => self.panes.select.render(c, layers.overlay, content, theme),
                Pane::Separator => self.panes.separator.render(c, content, theme),
                Pane::Sidebar => self.panes.sidebar.render(c, content, theme),
                Pane::Skeleton => self.panes.skeleton.render(c, content, theme),
                Pane::Slider => self.panes.slider.render(c, content, theme),
                Pane::Spinner => self.panes.spinner.render(c, content, theme),
                Pane::Stat => self.panes.stat.render(c, content, theme),
                Pane::Switch => self.panes.switch.render(c, content, theme),
                Pane::Table => self.panes.table.render(c, content, theme),
                Pane::Tabs => self.panes.tabs.render(c, content, theme),
                Pane::Toast => self.panes.toast.render(c, content, theme),
                Pane::Tooltip => {
                    self.panes.tooltip.render(c, content, theme);
                    self.panes.tooltip.render_tooltip(
                        c,
                        layers.toast,
                        theme,
                        self.width,
                        self.height,
                    );
                }
                Pane::Tree => self.panes.tree.render(c, content, theme),
                // task-104 front A, fields (#154)
                Pane::Field => self.panes.field.render(c, content, theme),
                // task-104 front B, overlays (#155)
                Pane::Popover => {
                    let bounds = self.popover_bounds(content);
                    let popover = &mut self.panes.popover;
                    popover.render(c, layers.overlay, content, bounds, theme);
                }
                // task-104 front C, conversation and feedback (#156)
                Pane::Message => self.panes.message.render(c, content, theme),
                // task-104 front D, disclosure and display (#157)
                Pane::Accordion => self.panes.accordion.render(c, content, theme),
                Pane::AspectRatio => self.panes.aspect_ratio.render(c, content, theme),
                Pane::Collapsible => self.panes.collapsible.render(c, content, theme),
                Pane::Item => self.panes.item.render(c, content, theme),
                Pane::Kbd => self.panes.kbd.render(c, content, theme),
                // task-104 front E, selection and dates (#158)
                Pane::Toggle => self.panes.toggle.render(c, content, theme),
                Pane::Todo => {
                    let inert = self.overlay_holds_pointer();
                    self.panes.todo.set_inert(inert);
                    self.panes.todo.render(c, content, theme);
                }
                Pane::Surface => {
                    let inert = self.overlay_holds_pointer();
                    let todo = &mut self.panes.todo;
                    self.panes.surface.render(c, content, theme, todo, inert);
                }
                Pane::Silences => {
                    let inert = self.overlay_holds_pointer();
                    self.panes.silences.set_inert(inert);
                    self.panes.silences.render(c, content, theme);
                }
                Pane::Files => {
                    let inert = self.overlay_holds_pointer();
                    self.panes.files.set_inert(inert);
                    self.panes.files.render(c, content, theme);
                }
                Pane::Capture => {
                    let inert = self.overlay_holds_pointer();
                    self.panes.capture.set_inert(inert);
                    self.panes.capture.render(c, content, theme);
                }
                Pane::Dock => self.panes.dock.render(c, content, theme),
                Pane::Editor => self.panes.editor.render(c, content),
                Pane::Git => self.panes.git.render(c, content, theme),
                Pane::Builder => self.panes.builder.render(c, content, theme),
            },
        }
    }

    /// What the active page means, when it projects it (the todo page,
    /// the surface over it, the silences, the files, the capture, a
    /// component page whose widgets project in comps), and
    /// what the view shows over it while it does (`overlay`: the toasts on
    /// screen, the modal or the context menu that holds the pointer): the
    /// snapshot assistive technology is shown (`reader_semantics` builds
    /// on it) and the port serves with the shell. A page without a
    /// projection has no tree but its overlays. While the page shows in a
    /// window of its own too, the projection carries that window
    /// (`with_window`); with one window and nothing over it, it is the
    /// page's alone, byte for byte.
    pub fn semantics(&self) -> Option<Snapshot> {
        let page = self.page_semantics();
        if !self.overlays_shown() {
            return page;
        }
        let mut b = SnapshotBuilder::new();
        if let Some(page) = &page {
            detached::extend(&mut b, page);
        }
        self.project_overlays(&mut b);
        b.finish()
            .inspect_err(|e| log::warn!("the overlays' projection does not validate: {e}"))
            .ok()
            .or(page)
    }

    /// The active page's projection alone (see `semantics`).
    fn page_semantics(&self) -> Option<Snapshot> {
        let snapshot = match self.pane() {
            Some(Pane::Todo) => self.panes.todo.snapshot(),
            Some(Pane::Surface) => self.panes.surface.snapshot(&self.panes.todo),
            Some(Pane::Silences) => self.panes.silences.projection(),
            Some(Pane::Files) => self.panes.files.snapshot(),
            Some(Pane::Capture) => self.panes.capture.snapshot(),
            Some(pane) => self.component_semantics(pane)?,
            None => return None,
        };
        let snapshot = snapshot
            .inspect_err(|e| log::warn!("the projection does not validate: {e}"))
            .ok()?;
        if self.panes.detached.is_none() {
            return Some(snapshot);
        }
        detached::with_window(&snapshot, self.current().title)
            .inspect_err(|e| log::warn!("the window projection does not validate: {e}"))
            .ok()
    }

    /// A request from an actor outside the page's own input (assistive
    /// technology, an agent through the port, the menu), through the
    /// gallery's one dispatcher: aimed at the shell it runs over the
    /// shell (`shell.rs`); aimed at the active page it runs over the page
    /// (an open overlay leaves the page inert, as for a click). The
    /// receipt as it stands after the dispatch; `None` when the page
    /// takes none.
    pub fn act(&mut self, req: Request) -> Option<Receipt> {
        if self.is_shell(&req.target) {
            return self.act_shell(req);
        }
        if self.is_overlay(&req.target) {
            return self.act_overlay(req);
        }
        match self.pane() {
            Some(Pane::Todo) => {
                let inert = self.overlay_holds_pointer();
                self.panes.todo.set_inert(inert);
                let id = self.panes.todo.dispatch(req);
                self.panes.todo.receipt(id).cloned()
            }
            Some(Pane::Surface) => {
                let inert = self.overlay_holds_pointer();
                let todo = &mut self.panes.todo;
                self.panes.surface.act(req, todo, inert)
            }
            Some(Pane::Silences) => {
                let inert = self.overlay_holds_pointer();
                let todo = &mut self.panes.todo;
                self.panes.silences.act(req, todo, inert)
            }
            Some(Pane::Files) => {
                let inert = self.overlay_holds_pointer();
                let todo = &mut self.panes.todo;
                self.panes.files.act(req, todo, inert)
            }
            Some(Pane::Capture) => {
                let inert = self.overlay_holds_pointer();
                let todo = &mut self.panes.todo;
                self.panes.capture.act(req, todo, inert)
            }
            _ => self.act_component(req),
        }
    }

    /// Adds a sink to the dispatcher `act` goes through, so every stage of
    /// every receipt reaches it, later ones included (the todo page's is
    /// the only dispatcher of the gallery today; the surface and the
    /// silences pages ask through it too).
    pub fn add_sink(&mut self, sink: Box<dyn ReceiptSink>) {
        self.panes.todo.add_sink(sink);
    }

    /// Milliseconds on the clock the gallery's receipts are stamped with
    /// (the one dispatcher's): the clock anything recorded beside them (a
    /// `record::Recorder`) must use, or its events and the receipts would
    /// not line up. Read through the dispatcher's lending, hence `&mut`.
    pub fn now(&mut self) -> Millis {
        self.panes.todo.with_dispatcher(|_, _, now| now)
    }

    /// Keyboard focus from assistive technology to a node of the active
    /// page, or of the overlay that holds the pointer; `false` when
    /// neither takes it.
    pub fn assist_focus(&mut self, id: &SemId) -> bool {
        if self.is_overlay(id) {
            return self.focus_overlay(id);
        }
        match self.pane() {
            Some(Pane::Todo) => {
                let inert = self.overlay_holds_pointer();
                self.panes.todo.set_inert(inert);
                self.panes.todo.focus(id)
            }
            Some(Pane::Surface) => {
                let inert = self.overlay_holds_pointer();
                let todo = &mut self.panes.todo;
                self.panes.surface.focus(id, todo, inert)
            }
            Some(Pane::Silences) => {
                let inert = self.overlay_holds_pointer();
                self.panes.silences.set_inert(inert);
                self.panes.silences.focus(id)
            }
            Some(Pane::Files) => {
                let inert = self.overlay_holds_pointer();
                self.panes.files.set_inert(inert);
                self.panes.files.focus(id)
            }
            Some(Pane::Capture) => {
                let inert = self.overlay_holds_pointer();
                self.panes.capture.set_inert(inert);
                self.panes.capture.focus(id)
            }
            _ => self.focus_component(id),
        }
    }

    /// The platform's camera and microphone for the Capture page (the
    /// shell's, once its window is up; a fake in a test). Until then the
    /// page has none, so a gallery built headless never reaches a device.
    pub fn set_capture(&mut self, capture: Box<dyn Capture>) {
        self.panes.capture.set_capture(capture);
    }

    /// The shell's word on whether a window that shows the page is awake
    /// (not covered, the app not asleep): the Capture page stops its
    /// devices the moment it is not, and starts what the person had
    /// turned on as it is again. `true` when that moved what the page
    /// shows.
    pub fn set_capture_shown(&mut self, shown: bool) -> bool {
        self.panes.capture.set_shown(shown) && self.pane() == Some(Pane::Capture)
    }

    /// A placeholder page: an empty state on a panel, naming the slot.
    fn render_placeholder(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rect = Rect::new(
            content.x,
            content.y,
            content.w.min(theme.size.readable_max_w),
            theme.size.card_min_w / 2.0,
        );
        panel(c, rect, theme);
        EmptyState::new(
            self.current().title,
            "Not in comps yet. The slot is here so the tree reads complete.",
        )
        .render(c, rect, theme);
    }
}

/// The gallery through the agent port (task-61): the same hooks a screen
/// reader goes through, desktop only.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod agent;

/// The component pages whose widgets project in comps, projected and run
/// through the gallery's dispatcher (task-75 phase 2b).
pub(in crate::views) mod component;

/// The active page in a window of its own as well (task-76 item 2): the
/// view's side of the second window.
mod detached;
pub use detached::Detached;

/// The keyboard on the component pages (task-75 phase 2c): Tab over the
/// projection, the arrow keys by widget, Enter and Space press.
pub(in crate::views) mod keyboard;

/// The view's own layer over the page, projected and run through the
/// gallery's dispatcher (task-75 phase 2a): the toasts, the modal, the
/// context menu.
pub(super) mod overlay;

/// What a screen reader is shown (task-75 phase 2a): the view's
/// projection and what only a reader hears.
mod reader;

/// The shell's capabilities (task-76 item 4): the page, the theme, the
/// page in a window of its own and the recording's save, projected and
/// run through the gallery's dispatcher.
mod shell;
pub(crate) use shell::PERSON_ONLY;
pub use shell::{Shell, ShellTarget};
