//! The active pane's share of the input that reaches the page: a pointer
//! event, in the main window or the page's own, with the view-level
//! effects a page asks for (a dialog, a toast, a theme), and a file drag
//! over the window.

use accord::Value;
use engine::control::input::FileDrop;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::super::{Pane, ShowcaseView};
use super::ShellTarget;
use super::component::After;

impl ShowcaseView {
    /// The active pane's share of a pointer event in the main window,
    /// with the view-level effects a page asks for (a dialog, a toast, a
    /// theme). The events in the page's own window come through
    /// `detached_event`, laid out for that window.
    pub(in crate::views) fn page_event(&mut self, event: &WidgetEvent) -> EventResult {
        let content = self.page_rect();
        self.pane_event(event, content)
    }

    /// The active pane's share of a pointer event, laid out in `content`
    /// (the main window's page rect, or the detached window's).
    pub(super) fn pane_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        let t = &self.theme;
        match self.pane() {
            None => EventResult::IGNORED,
            Some(pane) => match pane {
                // A card picked is the shell's `shell.theme`, the same
                // capability as the T key and the menu, through the
                // dispatcher.
                Pane::Theme => {
                    let (r, picked) = self.panes.themes.handle_event(event, content, t);
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
                Pane::Chart => self.panes.chart.handle_event(event, content, t),
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
                Pane::Toolbar => self.panes.toolbar.handle_event(event, content, t),
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
                Pane::Label => self.panes.label.handle_event(event, content, t),
                Pane::InputGroup => self.panes.input_group.handle_event(event, content, t),
                Pane::InputOtp => self.panes.input_otp.handle_event(event, content, t),
                Pane::Textarea => self.panes.textarea.handle_event(event, content, t),
                Pane::Combobox => self.panes.combobox.handle_event(event, content, t),
                Pane::Command => self.panes.command.handle_event(event, content, t),
                // task-104 front B, overlays (#155)
                Pane::AlertDialog => {
                    let (r, after) = self.panes.alert_dialog.handle_event(event, content, t);
                    if let Some(after) = after {
                        self.after_component(after);
                    }
                    r
                }
                Pane::Drawer => {
                    let (r, after) = self.panes.drawer.handle_event(event, content, t);
                    if let Some(after) = after {
                        self.after_component(after);
                    }
                    r
                }
                Pane::Modals => self.panes.modals.handle_event(event, content, t),
                Pane::Sheet => {
                    let (r, after) = self.panes.sheet.handle_event(event, content, t);
                    if let Some(after) = after {
                        self.after_component(after);
                    }
                    r
                }
                Pane::DropdownMenu => {
                    let bounds = self.anchored_bounds(content);
                    let page = &mut self.panes.dropdown_menu;
                    page.handle_event(event, content, bounds, t)
                }
                Pane::Menubar => {
                    let bounds = self.anchored_bounds(content);
                    let page = &mut self.panes.menubar;
                    page.handle_event(event, content, bounds, t)
                }
                Pane::HoverCard => {
                    let bounds = self.anchored_bounds(content);
                    self.panes
                        .hover_card
                        .handle_event(event, content, bounds, t)
                }
                Pane::Popover => {
                    let bounds = self.anchored_bounds(content);
                    self.panes.popover.handle_event(event, content, bounds, t)
                }
                // task-104 front C, conversation and feedback (#156)
                Pane::Alert => self.panes.alert.handle_event(event, content, t),
                Pane::Attachment => self.panes.attachment.handle_event(event, content, t),
                Pane::Bubble => self.panes.bubble.handle_event(event, content, t),
                Pane::Comments => self.panes.comments.handle_event(event, content, t),
                Pane::Marker => self.panes.marker.handle_event(event, content, t),
                Pane::Message => self.panes.message.handle_event(event, content, t),
                Pane::MessageScroller => {
                    self.panes.message_scroller.handle_event(event, content, t)
                }
                Pane::PromptInput => self.panes.prompt_input.handle_event(event, content, t),
                Pane::Notifications => self.panes.notifications.handle_event(event, content, t),
                // task-104 front D, disclosure and display (#157)
                Pane::Accordion => self.panes.accordion.handle_event(event, content, t),
                Pane::AspectRatio => self.panes.aspect_ratio.handle_event(event, content, t),
                Pane::Collapsible => self.panes.collapsible.handle_event(event, content, t),
                Pane::Item => self.panes.item.handle_event(event, content, t),
                Pane::Kbd => self.panes.kbd.handle_event(event, content, t),
                Pane::Carousel => self.panes.carousel.handle_event(event, content, t),
                Pane::Pagination => self.panes.pagination.handle_event(event, content, t),
                Pane::StaticTable => self.panes.table_static.handle_event(event, content, t),
                // task-104 front E, selection and dates (#158)
                Pane::ButtonGroup => self.panes.button_group.handle_event(event, content, t),
                Pane::Calendar => self.panes.calendar.handle_event(event, content, t),
                Pane::DatePicker => self.panes.date_picker.handle_event(event, content, t),
                Pane::Menus => self.panes.menus.handle_event(event, content, t),
                Pane::RadioGroup => self.panes.radio_group.handle_event(event, content, t),
                Pane::Toggle => self.panes.toggle.handle_event(event, content, t),
                Pane::ToggleGroup => self.panes.toggle_group.handle_event(event, content, t),
                // task-104 front F, kit foundation
                Pane::Home => self.panes.home.handle_event(event, content, t),
                Pane::Topbar => self.panes.topbar.handle_event(event, content, t),
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
                Pane::Builder => self.panes.builder.handle_event(event, content, t),
            },
        }
    }

    /// The active page's share of a file drag over the window (the
    /// engine's event, forwarded whole by the shell): the Files page
    /// receives it; the other pages take no files.
    pub(in crate::views) fn page_file_drop(&mut self, drop: &FileDrop) -> EventResult {
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
}
