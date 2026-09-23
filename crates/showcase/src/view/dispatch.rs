//! Dispatch to the active pane: its natural height, its share of an
//! event, its visible-only tick and its drawing. One match per verb over
//! `pages::Pane`, so the rest of the view never names a pane's field.

use comps::prelude::{EmptyState, EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;

use super::components::toast;
use super::{Layers, Pane, ShowcaseView, panel};

impl ShowcaseView {
    /// Natural (unclipped) height of the active page's content.
    pub(super) fn page_content_height(&self, content: Rect) -> f32 {
        let t = &self.theme;
        let body = match self.pane() {
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
                Pane::Todo => self.panes.todo.content_height(content),
                Pane::Dock => self.panes.dock.content_height(content),
                Pane::Editor => self.panes.editor.content_height(content),
                Pane::Git => self.panes.git.content_height(content),
                Pane::Builder => self.panes.builder.content_height(content),
            },
        };
        body + self.note_h()
    }

    /// The active pane's share of a pointer event, with the view-level
    /// effects a page asks for (a dialog, a toast, a theme).
    pub(super) fn page_event(&mut self, event: &WidgetEvent) -> EventResult {
        let content = self.page_rect();
        let t = &self.theme;
        match self.pane() {
            None => EventResult::IGNORED,
            Some(pane) => match pane {
                Pane::Theme => {
                    let (r, picked) = self.panes.themes.handle_event(event, content);
                    if let Some(name) = picked {
                        self.set_theme(name);
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
                        self.open_modal(destructive);
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
                        self.toasts
                            .push(toast::ToastPage::message(intent), intent, &self.theme);
                    }
                    r
                }
                Pane::Tooltip => self.panes.tooltip.handle_event(event, content, t),
                Pane::Tree => self.panes.tree.handle_event(event, content, t),
                Pane::Todo => {
                    let inert = self.overlay_holds_pointer();
                    self.panes.todo.set_inert(inert);
                    self.panes.todo.handle_event(event, content, t)
                }
                Pane::Dock => self.panes.dock.handle_event(event, content),
                Pane::Editor => self.panes.editor.handle_event(event, content),
                Pane::Git => self.panes.git.handle_event(event, content),
                Pane::Builder => self.panes.builder.handle_event(event, content),
            },
        }
    }

    /// Motion that would run forever ticks only while visible: the
    /// dock's caret, the chart reveal, spinners, breathing skeletons,
    /// the todo caret, the editor.
    pub(super) fn page_tick(&mut self, dt: f32) -> bool {
        match self.pane() {
            Some(Pane::Dock) => self.panes.dock.tick(dt),
            Some(Pane::Chart) => self.panes.chart.tick(dt),
            Some(Pane::Spinner) => self.panes.spinner.tick(dt),
            Some(Pane::Skeleton) => self.panes.skeleton.tick(dt),
            Some(Pane::Todo) => self.panes.todo.tick(dt),
            Some(Pane::Editor) => self.panes.editor.tick(dt),
            _ => false,
        }
    }

    /// Draw the active page into `content` (a placeholder names its slot).
    pub(super) fn page_render(
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
                Pane::Todo => {
                    let inert = self.overlay_holds_pointer();
                    self.panes.todo.set_inert(inert);
                    self.panes.todo.render(c, content, theme);
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
