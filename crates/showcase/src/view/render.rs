//! Frame side of the gallery: the animation tick and the scene build
//! (chrome, the clipped page, the alias note or placeholder, the active
//! overlay, the toasts).

use comps::overlay::OverlayManager;
use comps::prelude::{EmptyState, Rect};
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};

use super::components::scroll_area::ScrollAreaPage;
use super::{ActiveOverlay, Body, Layers, PAGES, Pane, ShowcaseView, pages, panel};

impl ShowcaseView {
    // -- Animation -------------------------------------------------------------

    /// Advance all animations. Returns `true` while anything is moving.
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut animating = false;
        animating |= self.overlay_mgr.tick(dt);
        animating |= self.toasts.tick(dt);
        // Widgets that settle on their own tick whichever page shows
        // (a focused field, a knob spring, a scrollbar fade, a tooltip
        // delay): they must not freeze mid-motion when the page changes.
        animating |= self.input.tick(dt);
        animating |= self.switch.tick(dt);
        animating |= self.scroll_area.tick(dt);
        animating |= self.tooltip.tick(dt);
        animating |= self.table.tick(dt);
        // Motion that would run forever ticks only while visible: the
        // dock's caret, the chart reveal, spinners, breathing skeletons,
        // the todo caret, the editor.
        let visible = match self.pane() {
            Some(Pane::Dock) => self.dock.tick(dt),
            Some(Pane::Chart) => self.chart.tick(dt),
            Some(Pane::Spinner) => self.spinner.tick(dt),
            Some(Pane::Skeleton) => self.skeleton.tick(dt),
            Some(Pane::Todo) => self.todo.tick(dt),
            Some(Pane::Editor) => self.editor.tick(dt),
            _ => false,
        };
        animating |= visible;
        // The git tab always ticks: worker events land there whichever
        // page is showing, and must not pile up until it is.
        animating |= self.git.tick(dt) && self.pane() == Some(Pane::Git);

        // Drop overlay widgets whose exit animation finished.
        if let Some(active) = &self.active_overlay {
            let id = active.id();
            if !self.overlay_mgr.stack.iter().any(|o| o.id == id) {
                self.active_overlay = None;
            }
        }
        animating
    }

    // -- Rendering -------------------------------------------------------------

    fn ensure_layers(&mut self, c: &mut Compositor) -> Layers {
        *self.layers.get_or_insert_with(|| Layers {
            list: c.create_layer(10),
            overlay: c.create_layer(OverlayManager::BASE_Z),
            toast: c.create_layer(OverlayManager::BASE_Z + 200),
        })
    }

    pub fn render(&mut self, c: &mut Compositor) {
        c.begin_frame();
        let layers = self.ensure_layers(c);
        let theme = self.theme.clone();

        // The chrome: header on the default layer, the sidebar on the
        // overlay layer so a drawer sits above the content.
        self.shell
            .render(c, LayerId::DEFAULT, layers.overlay, &theme);

        // Keep the page scroll clamped to the current viewport/content
        // (resize can shrink content; the offset must follow).
        self.sync_page_scroll();
        if let Some(offset) = self.pending_scroll.take() {
            self.page_scroll[self.page].scroll_to(offset);
        }
        let content = self.page_rect();
        let viewport = self.content_rect();

        // Clip the virtualized list (overscan rows) to its panel.
        let clip = if self.pane() == Some(Pane::ScrollArea) {
            let b = ScrollAreaPage::bounds(content, &theme);
            let sf = self.scale_factor;
            Some((
                (b.x * sf) as u32,
                (b.y * sf) as u32,
                (b.w * sf).ceil() as u32,
                (b.h * sf).ceil() as u32,
            ))
        } else {
            None
        };
        c.set_layer_clip_rect(layers.list, clip);

        // Scrolled page content clips to the viewport below the header
        // (PushClip rects are logical; the encoder scales them to physical).
        let page_x = self.page_x();
        c.push(SceneNode::PushClip {
            x: page_x,
            y: viewport.y,
            w: self.width - page_x,
            h: if self.is_app() {
                self.height - viewport.y
            } else {
                viewport.h
            },
        });
        self.render_note(c, content, &theme);
        match self.pane() {
            None => self.render_placeholder(c, content, &theme),
            Some(pane) => match pane {
                Pane::Theme => self.themes.render(c, content, &theme, &self.theme_name),
                Pane::Typography => self.typography.render(c, content, &theme),
                Pane::Icons => self.icons.render(c, content, &theme),
                Pane::Effects => self.effects.render(c, content, &theme),
                Pane::Avatar => self.avatar.render(c, content, &theme),
                Pane::Badge => self.badge.render(c, content, &theme),
                Pane::Breadcrumb => self.breadcrumb.render(c, content, &theme),
                Pane::Button => self.button.render(c, content, &theme),
                Pane::Card => self.card.render(c, content, &theme),
                Pane::Chart => self.chart.render(c, content, &theme),
                Pane::Checkbox => self.checkbox.render(c, content, &theme),
                Pane::Chip => self.chip.render(c, content, &theme),
                Pane::CodeBlock => self.code_block.render(c, content, &theme),
                Pane::ContextMenu => self.context_menu.render(c, content, &theme),
                Pane::Dialog => self.dialog.render(c, content, &theme),
                Pane::Empty => self.empty.render(c, content, &theme),
                Pane::Graph => self.graph.render(c, content, &theme),
                Pane::IconButton => self.icon_button.render(c, content, &theme),
                Pane::Input => self.input.render(c, content, &theme),
                Pane::NavigationMenu => self.navigation_menu.render(c, content, &theme),
                Pane::Panel => self.panel.render(c, content, &theme),
                Pane::PanelHeader => self.panel_header.render(c, content, &theme),
                Pane::Progress => self.progress.render(c, content, &theme),
                Pane::Resizable => self.resizable.render(c, content, &theme),
                Pane::ScrollArea => self.scroll_area.render(c, layers.list, content, &theme),
                Pane::Select => self.select.render(c, layers.overlay, content, &theme),
                Pane::Separator => self.separator.render(c, content, &theme),
                Pane::Sidebar => self.sidebar.render(c, content, &theme),
                Pane::Skeleton => self.skeleton.render(c, content, &theme),
                Pane::Slider => self.slider.render(c, content, &theme),
                Pane::Spinner => self.spinner.render(c, content, &theme),
                Pane::Stat => self.stat.render(c, content, &theme),
                Pane::Switch => self.switch.render(c, content, &theme),
                Pane::Table => self.table.render(c, content, &theme),
                Pane::Tabs => self.tabs.render(c, content, &theme),
                Pane::Toast => self.toast.render(c, content, &theme),
                Pane::Tooltip => {
                    self.tooltip.render(c, content, &theme);
                    self.tooltip
                        .render_tooltip(c, layers.toast, &theme, self.width, self.height);
                }
                Pane::Tree => self.tree.render(c, content, &theme),
                Pane::Todo => self.todo.render(c, content, &theme),
                Pane::Dock => self.dock.render(c, content, &theme),
                Pane::Editor => self.editor.render(c, content),
                Pane::Git => self.git.render(c, content, &theme),
                Pane::Builder => self.builder.render(c, content, &theme),
            },
        }
        c.push(SceneNode::PopClip);

        // Active overlay: fade via layer opacity driven by the manager.
        let mut overlay_opacity = 1.0;
        if let Some(active) = &self.active_overlay {
            let id = active.id();
            if let Some(overlay) = self.overlay_mgr.stack.iter().find(|o| o.id == id) {
                overlay_opacity = overlay.opacity();
            }
            match active {
                ActiveOverlay::Modal { widget, .. } => {
                    widget.render(c, layers.overlay, &theme, self.width, self.height);
                }
                ActiveOverlay::Menu { widget, x, y, .. } => {
                    widget.render(c, layers.overlay, &theme, *x, *y);
                }
            }
        }
        c.set_layer_opacity(layers.overlay, overlay_opacity);

        self.toasts
            .render(c, layers.toast, &theme, self.width, self.height);
    }

    /// The note an alias page shows above the shared example: which
    /// page's example it is, and what its own still lacks.
    fn render_note(&self, c: &mut Compositor, content: Rect, theme: &engine::theme::Theme) {
        let Body::Alias { pane, gap } = self.current().body else {
            return;
        };
        let owner = PAGES[pages::index_of(pane)].title;
        let note = format!("shares the {owner} example. still missing: {gap}.");
        let style = theme.typography.caption_r();
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&note, &style, Some(content.w)),
            x: content.x,
            y: content.y - self.note_h(),
            color: theme.glass.text_placeholder.0,
        });
    }

    /// A placeholder page: an empty state on a panel, naming the slot.
    fn render_placeholder(&self, c: &mut Compositor, content: Rect, theme: &engine::theme::Theme) {
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
