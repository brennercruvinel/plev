//! Drawing the active pane, in the main window or the page's own, and
//! the placeholder of a slot not in comps yet.

use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{EmptyState, Rect};

use super::super::{Layers, Pane, ShowcaseView, panel};

impl ShowcaseView {
    /// Draw the active page into `content` of the main window (a
    /// placeholder names its slot).
    pub(in crate::views) fn page_render(
        &mut self,
        c: &mut Compositor,
        layers: Layers,
        content: Rect,
        theme: &Theme,
    ) {
        self.pane_render(c, layers, content, theme);
    }

    /// Draw the active pane into `content`, whichever window's.
    pub(super) fn pane_render(
        &mut self,
        c: &mut Compositor,
        layers: Layers,
        content: Rect,
        theme: &Theme,
    ) {
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
                Pane::Input => self.panes.input.render(c, layers.overlay, content, theme),
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
                Pane::Toolbar => self.panes.toolbar.render(c, content, theme),
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
                Pane::Field => self.panes.field.render(c, layers.overlay, content, theme),
                Pane::Label => self.panes.label.render(c, layers.overlay, content, theme),
                Pane::InputGroup => {
                    self.panes
                        .input_group
                        .render(c, layers.overlay, content, theme)
                }
                Pane::InputOtp => self
                    .panes
                    .input_otp
                    .render(c, layers.overlay, content, theme),
                Pane::Textarea => self
                    .panes
                    .textarea
                    .render(c, layers.overlay, content, theme),
                Pane::Combobox => self
                    .panes
                    .combobox
                    .render(c, layers.overlay, content, theme),
                Pane::Command => self.panes.command.render(c, layers.overlay, content, theme),
                // task-104 front B, overlays (#155)
                Pane::AlertDialog => self.panes.alert_dialog.render(c, content, theme),
                Pane::Drawer => self.panes.drawer.render(c, content, theme),
                Pane::Sheet => self.panes.sheet.render(c, content, theme),
                Pane::Modals => self.panes.modals.render(c, layers.overlay, content, theme),
                Pane::Menubar => {
                    let bounds = self.anchored_bounds(content);
                    let page = &mut self.panes.menubar;
                    page.render(
                        c,
                        layers.anchored,
                        content,
                        bounds,
                        theme,
                        &self.overlay_mgr,
                    );
                }
                Pane::DropdownMenu => {
                    let bounds = self.anchored_bounds(content);
                    let page = &mut self.panes.dropdown_menu;
                    page.render(
                        c,
                        layers.anchored,
                        content,
                        bounds,
                        theme,
                        &self.overlay_mgr,
                    );
                }
                Pane::HoverCard => {
                    let bounds = self.anchored_bounds(content);
                    let page = &self.panes.hover_card;
                    page.render(
                        c,
                        layers.anchored,
                        content,
                        bounds,
                        theme,
                        &self.overlay_mgr,
                    );
                }
                Pane::Popover => {
                    let bounds = self.anchored_bounds(content);
                    let popover = &mut self.panes.popover;
                    popover.render(
                        c,
                        layers.anchored,
                        content,
                        bounds,
                        theme,
                        &self.overlay_mgr,
                    );
                }
                // task-104 front C, conversation and feedback (#156)
                Pane::Alert => self.panes.alert.render(c, content, theme),
                Pane::Attachment => self.panes.attachment.render(c, content, theme),
                Pane::Bubble => self.panes.bubble.render(c, content, theme),
                Pane::Comments => self.panes.comments.render(c, content, theme),
                Pane::Notifications => self.panes.notifications.render(c, content, theme),
                Pane::Marker => self.panes.marker.render(c, content, theme),
                Pane::Message => self.panes.message.render(c, content, theme),
                Pane::MessageScroller => self.panes.message_scroller.render(c, content, theme),
                Pane::PromptInput => {
                    self.panes
                        .prompt_input
                        .render(c, layers.overlay, content, theme)
                }
                // task-104 front D, disclosure and display (#157)
                Pane::Accordion => self.panes.accordion.render(c, content, theme),
                Pane::AspectRatio => self.panes.aspect_ratio.render(c, content, theme),
                Pane::Collapsible => self.panes.collapsible.render(c, content, theme),
                Pane::Item => self.panes.item.render(c, content, theme),
                Pane::Kbd => self.panes.kbd.render(c, content, theme),
                Pane::Carousel => self.panes.carousel.render(c, content, theme),
                Pane::Pagination => self.panes.pagination.render(c, content, theme),
                Pane::StaticTable => self.panes.table_static.render(c, content, theme),
                // task-104 front E, selection and dates (#158)
                Pane::ButtonGroup => self.panes.button_group.render(c, content, theme),
                Pane::Calendar => self.panes.calendar.render(c, content, theme),
                Pane::DatePicker => self.panes.date_picker.render(c, content, theme),
                Pane::Menus => self.panes.menus.render(c, content, theme),
                Pane::RadioGroup => self.panes.radio_group.render(c, content, theme),
                Pane::Toggle => self.panes.toggle.render(c, content, theme),
                Pane::ToggleGroup => self.panes.toggle_group.render(c, content, theme),
                // task-104 front F, kit foundation
                Pane::Home => self.panes.home.render(c, content, theme),
                Pane::Topbar => self.panes.topbar.render(c, content, theme),
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
