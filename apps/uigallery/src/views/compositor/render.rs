//! Frame side of the gallery: the animation tick and the scene build
//! (chrome, the clipped page, the alias note or placeholder, the active
//! overlay, the toasts).

use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::widgets::overlay::OverlayManager;
use engine::widgets::prelude::{Rect, Wake};

use super::super::components::scroll_area::ScrollAreaPage;
use super::super::{ActiveOverlay, Body, Layers, PAGES, Pane, ShowcaseView, pages};

impl ShowcaseView {
    // -- Animation -------------------------------------------------------------

    /// Advance all animations. The answer is every widget's ask merged:
    /// a frame now while anything moves, and the earliest deadline (a
    /// focused caret's next flip) when nothing does.
    pub fn wake(&mut self, dt: f32) -> Wake {
        let mut wake = Wake::moving(self.overlay_mgr.tick(dt));
        wake = wake.merge(self.toasts.wake(dt));
        // Widgets that settle on their own tick whichever page shows
        // (a focused field, a knob spring, a scrollbar fade, a tooltip
        // delay): they must not freeze mid-motion when the page changes.
        wake = wake.merge(self.panes.input.wake(dt));
        wake = wake.merge(Wake::moving(self.panes.switch.tick(dt)));
        wake = wake.merge(Wake::moving(self.panes.scroll_area.tick(dt)));
        wake = wake.merge(Wake::moving(self.panes.tooltip.tick(dt)));
        wake = wake.merge(Wake::moving(self.panes.table.tick(dt)));
        wake = wake.merge(self.page_wake(dt));
        // The git tab always ticks (its caret and toasts keep time
        // whichever page shows); only the page on screen asks the loop
        // for frames. Worker events land through `apply_git`.
        let git = self.panes.git.wake(dt);
        if self.pane() == Some(Pane::Git) {
            wake = wake.merge(git);
        }

        // Drop overlay widgets whose exit animation finished.
        if let Some(active) = &self.active_overlay {
            let id = active.id();
            if !self.overlay_mgr.stack.iter().any(|o| o.id == id) {
                self.active_overlay = None;
            }
        }
        wake
    }

    /// What the git worker sent, applied to the embedded workspace
    /// whichever page shows (it must not fall behind while hidden).
    /// `true` when that changed something on screen: the Git page is
    /// showing and there were events.
    pub fn apply_git(&mut self) -> bool {
        let changed = self.panes.git.apply_events();
        changed && self.pane() == Some(Pane::Git)
    }

    /// [`ShowcaseView::wake`] as the `bool` the settle loops read (the
    /// headless `render_page`, the session recorder): `true` while
    /// something moves, a frame now or on the very next one. A caret at
    /// rest between flips is settled, not moving.
    pub fn tick(&mut self, dt: f32) -> bool {
        let wake = self.wake(dt);
        wake.now || wake.is_continuous()
    }

    // -- Rendering -------------------------------------------------------------

    fn ensure_layers(&mut self, c: &mut Compositor) -> Layers {
        *self.layers.get_or_insert_with(|| Layers {
            list: c.create_layer(10),
            overlay: c.create_layer(OverlayManager::BASE_Z),
            anchored: c.create_layer(OverlayManager::BASE_Z + 100),
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
        self.page_render(c, layers, content, &theme);
        c.push(SceneNode::PopClip);

        // Active overlay: fade via layer opacity driven by the manager.
        // An edge panel slides instead: the layer stays opaque and the
        // panel draws its entry's progress.
        let mut overlay_opacity = 1.0;
        if let Some(active) = &self.active_overlay {
            let id = active.id();
            let shown = self
                .overlay_mgr
                .stack
                .iter()
                .find(|o| o.id == id)
                .map_or(1.0, |o| o.progress());
            if !active.slides() {
                overlay_opacity = shown;
            }
            let window = Rect::new(0.0, 0.0, self.width, self.height);
            match active {
                ActiveOverlay::Modal { widget, .. } => {
                    widget.render(c, layers.overlay, &theme, self.width, self.height);
                }
                ActiveOverlay::Menu { widget, x, y, .. } => {
                    widget.render(c, layers.overlay, &theme, *x, *y);
                }
                ActiveOverlay::Sheet { widget, .. } => {
                    widget.render(c, layers.overlay, &theme, window, shown);
                }
                ActiveOverlay::Drawer { widget, .. } => {
                    widget.render(c, layers.overlay, &theme, window, shown);
                }
            }
        }
        c.set_layer_opacity(layers.overlay, overlay_opacity);

        self.toasts
            .render(c, layers.toast, &theme, self.width, self.height);
    }

    /// The note an alias page shows above the shared example: which
    /// page's example it is, and what its own still lacks.
    fn render_note(
        &self,
        c: &mut Compositor,
        content: Rect,
        theme: &engine::styling::theme::Theme,
    ) {
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
}

#[cfg(test)]
mod tests {
    use engine::drawing::compositor::Compositor;
    use engine::widgets::prelude::{Wake, WidgetEvent};

    use super::ShowcaseView;

    /// One turn of a 60 Hz display.
    const TURN: f32 = 1.0 / 60.0;

    /// The Switch page, laid out and at rest; returns a point on its
    /// first switch.
    fn switch_page() -> (ShowcaseView, (f32, f32)) {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.jump_to_page("Switch");
        view.render(&mut Compositor::new());
        for _ in 0..600 {
            if !view.wake(TURN).is_continuous() {
                break;
            }
        }
        let content = view.page_rect();
        (view, (content.x + 4.0, content.y + 4.0))
    }

    fn click(view: &mut ShowcaseView, (x, y): (f32, f32)) {
        view.handle_event(&WidgetEvent::MouseDown { x, y });
        view.handle_event(&WidgetEvent::MouseUp { x, y });
    }

    /// The frames the knob draws after the click, one turn apart.
    fn knob_frames(view: &mut ShowcaseView, first: f32) -> usize {
        let mut wake = view.wake(first);
        let mut frames = 0;
        while wake.is_continuous() && frames < 600 {
            frames += 1;
            wake = view.wake(TURN);
        }
        frames
    }

    /// The shell catches up as the loop wakes (`App::advance` with the
    /// three idle seconds), then applies the click: the knob animates
    /// from where it was, for as long as its spring takes.
    #[test]
    fn a_switch_flipped_after_three_idle_seconds_animates_from_the_start() {
        let (mut view, at) = switch_page();
        assert_eq!(view.wake(3.0), Wake::idle(), "the catch-up: nothing moves");
        click(&mut view, at);
        let frames = knob_frames(&mut view, 0.004);
        assert!(frames > 10, "the knob animated for {frames} frames");
    }

    /// Without the catch-up the click's frame would carry the idle
    /// seconds, and the spring would land at its end in one tick.
    #[test]
    fn without_the_catch_up_the_idle_gap_jumps_the_knob() {
        let (mut view, at) = switch_page();
        click(&mut view, at);
        assert_eq!(knob_frames(&mut view, 3.0), 0, "jumped to the end");
    }
}
