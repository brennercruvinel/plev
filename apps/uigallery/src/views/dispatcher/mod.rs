//! Dispatch to the active pane: its natural height, its share of an
//! event (`event.rs`), its visible-only tick and its drawing
//! (`render.rs`). One match per verb over `pages::Pane`, so the rest of
//! the view never names a pane's field. Here too what the page means
//! (its projection), the requests and the focus from outside its own
//! input, and the Capture page's devices.

use accord::{Millis, Receipt, ReceiptSink, Request, SemId, Snapshot, SnapshotBuilder};
use engine::runtime::platform::capture::Capture;
use engine::widgets::prelude::{Rect, Wake};

use super::{Pane, ShowcaseView};

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
                Pane::Typography => self.panes.typography.content_height(content, t),
                Pane::Icons => self.panes.icons.content_height(content, t),
                Pane::Effects => self.panes.effects.content_height(content, t),
                Pane::Avatar => self.panes.avatar.content_height(content, t),
                Pane::Badge => self.panes.badge.content_height(content, t),
                Pane::Breadcrumb => self.panes.breadcrumb.content_height(content, t),
                Pane::Button => self.panes.button.content_height(content, t),
                Pane::Card => self.panes.card.content_height(content, t),
                Pane::Chart => self.panes.chart.content_height(content, t),
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
                Pane::Toolbar => self.panes.toolbar.content_height(content, t),
                Pane::Switch => self.panes.switch.content_height(content, t),
                Pane::Table => self.panes.table.content_height(content, t),
                Pane::Tabs => self.panes.tabs.content_height(content, t),
                Pane::Toast => self.panes.toast.content_height(content, t),
                Pane::Tooltip => self.panes.tooltip.content_height(content, t),
                Pane::Tree => self.panes.tree.content_height(content, t),
                // task-104 front A, fields (#154)
                Pane::Field => self.panes.field.content_height(content, t),
                Pane::Label => self.panes.label.content_height(content, t),
                Pane::InputGroup => self.panes.input_group.content_height(content, t),
                Pane::InputOtp => self.panes.input_otp.content_height(content, t),
                Pane::Textarea => self.panes.textarea.content_height(content, t),
                Pane::Combobox => self.panes.combobox.content_height(content, t),
                Pane::Command => self.panes.command.content_height(content, t),
                // task-104 front B, overlays (#155)
                Pane::AlertDialog => self.panes.alert_dialog.content_height(content, t),
                Pane::Drawer => self.panes.drawer.content_height(content, t),
                Pane::Sheet => self.panes.sheet.content_height(content, t),
                Pane::DropdownMenu => self.panes.dropdown_menu.content_height(content, t),
                Pane::Menubar => self.panes.menubar.content_height(content, t),
                Pane::Modals => self.panes.modals.content_height(content, t),
                Pane::HoverCard => self.panes.hover_card.content_height(content, t),
                Pane::Popover => self.panes.popover.content_height(content, t),
                // task-104 front C, conversation and feedback (#156)
                Pane::Alert => self.panes.alert.content_height(content, t),
                Pane::Attachment => self.panes.attachment.content_height(content, t),
                Pane::Bubble => self.panes.bubble.content_height(content, t),
                Pane::Comments => self.panes.comments.content_height(content, t),
                Pane::Marker => self.panes.marker.content_height(content, t),
                Pane::Message => self.panes.message.content_height(content, t),
                Pane::MessageScroller => self.panes.message_scroller.content_height(content, t),
                Pane::PromptInput => self.panes.prompt_input.content_height(content, t),
                Pane::Notifications => self.panes.notifications.content_height(content, t),
                // task-104 front D, disclosure and display (#157)
                Pane::Accordion => self.panes.accordion.content_height(content, t),
                Pane::AspectRatio => self.panes.aspect_ratio.content_height(content, t),
                Pane::Collapsible => self.panes.collapsible.content_height(content, t),
                Pane::Item => self.panes.item.content_height(content, t),
                Pane::Kbd => self.panes.kbd.content_height(content, t),
                Pane::Carousel => self.panes.carousel.content_height(content, t),
                Pane::Pagination => self.panes.pagination.content_height(content, t),
                Pane::StaticTable => self.panes.table_static.content_height(content, t),
                // task-104 front E, selection and dates (#158)
                Pane::ButtonGroup => self.panes.button_group.content_height(content, t),
                Pane::Calendar => self.panes.calendar.content_height(content, t),
                Pane::DatePicker => self.panes.date_picker.content_height(content, t),
                Pane::Menus => self.panes.menus.content_height(content, t),
                Pane::RadioGroup => self.panes.radio_group.content_height(content, t),
                Pane::Toggle => self.panes.toggle.content_height(content, t),
                Pane::ToggleGroup => self.panes.toggle_group.content_height(content, t),
                // task-104 front F, kit foundation
                Pane::Home => self.panes.home.content_height(content, t),
                Pane::Topbar => self.panes.topbar.content_height(content, t),
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
                Pane::Builder => self.panes.builder.content_height(content, t),
            },
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
            Some(Pane::Label) => self.panes.label.wake(dt),
            Some(Pane::InputGroup) => self.panes.input_group.wake(dt),
            Some(Pane::InputOtp) => self.panes.input_otp.wake(dt),
            Some(Pane::Textarea) => self.panes.textarea.wake(dt),
            Some(Pane::Combobox) => self.panes.combobox.wake(dt),
            Some(Pane::Command) => self.panes.command.wake(dt),
            // task-104 front B, overlays (#155)
            Some(Pane::Modals) => self.panes.modals.wake(dt),
            Some(Pane::Menubar) => {
                let (content, viewport) = (self.page_rect(), self.content_rect());
                let page = &mut self.panes.menubar;
                Wake::moving(page.sync(&mut self.overlay_mgr, content, viewport, &self.theme))
            }
            Some(Pane::DropdownMenu) => {
                let (content, viewport) = (self.page_rect(), self.content_rect());
                let page = &mut self.panes.dropdown_menu;
                Wake::moving(page.sync(&mut self.overlay_mgr, content, viewport, &self.theme))
            }
            Some(Pane::HoverCard) => {
                let (content, viewport) = (self.page_rect(), self.content_rect());
                let page = &mut self.panes.hover_card;
                let mgr = &mut self.overlay_mgr;
                Wake::moving(page.wake(dt, mgr, content, viewport, &self.theme))
            }
            Some(Pane::Popover) => {
                let (content, viewport) = (self.page_rect(), self.content_rect());
                let popover = &mut self.panes.popover;
                Wake::moving(popover.sync(&mut self.overlay_mgr, content, viewport, &self.theme))
            }
            // task-104 front C, conversation and feedback (#156)
            Some(Pane::MessageScroller) => Wake::moving(self.panes.message_scroller.tick(dt)),
            Some(Pane::PromptInput) => self.panes.prompt_input.wake(dt),
            Some(Pane::Comments) => self.panes.comments.wake(dt),
            Some(Pane::Notifications) => Wake::moving(self.panes.notifications.tick(dt)),
            // task-104 front D, disclosure and display (#157)
            Some(Pane::Accordion) => self.panes.accordion.wake(dt),
            Some(Pane::Collapsible) => self.panes.collapsible.wake(dt),
            // task-104 front E, selection and dates (#158)
            Some(Pane::ToggleGroup) => Wake::moving(self.panes.toggle_group.tick(dt)),
            // task-104 front F, kit foundation
            Some(Pane::Topbar) => self.panes.topbar.wake(dt),
            // task-104 front G, navigation and data
            Some(Pane::Sidebar) => self.panes.sidebar.wake(dt),
            Some(Pane::Toolbar) => Wake::moving(self.panes.toolbar.tick(dt)),
            _ => Wake::idle(),
        };
        if let Some(detached) = self.panes.detached.as_mut() {
            detached.wake = detached.wake.merge(wake);
        }
        wake
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
        self.arm_reveal();
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

    /// Keyboard focus from assistive technology (and Tab) to a node of
    /// the active page, of the overlay that holds the pointer, or of the
    /// sidebar (`shell/sidebar.rs`); `false` when none takes it. The
    /// sidebar lets its focus go once the page or the overlay takes it.
    pub fn assist_focus(&mut self, id: &SemId) -> bool {
        self.arm_reveal();
        if self.is_sidebar(id) {
            return self.focus_sidebar(id);
        }
        let took = self.assist_page_focus(id);
        if took {
            self.blur_sidebar();
        }
        took
    }

    fn assist_page_focus(&mut self, id: &SemId) -> bool {
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

/// The active pane's share of a pointer event and of a file drag.
mod event;

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

/// The active pane's drawing, and a placeholder's.
mod render;

/// The shell's capabilities (task-76 item 4): the page, the theme, the
/// page in a window of its own and the recording's save, projected and
/// run through the gallery's dispatcher.
mod shell;
pub(crate) use shell::{PERSON_ONLY, SIDEBAR};
pub use shell::{Shell, ShellTarget};
