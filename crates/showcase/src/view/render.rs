//! Frame side of the gallery: the animation tick and the scene build
//! (chrome, the clipped section page, the active overlay, the toasts).

use comps::overlay::OverlayManager;
use engine::compositor::{Compositor, LayerId, SceneNode};

use super::{ActiveOverlay, Layers, Section, ShowcaseView};

impl ShowcaseView {
    // -- Animation -------------------------------------------------------------

    /// Advance all animations. Returns `true` while anything is moving.
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut animating = false;
        animating |= self.overlay_mgr.tick(dt);
        animating |= self.toasts.tick(dt);
        animating |= self.forms.tick(dt);
        animating |= self.lists.tick(dt);
        animating |= self.overlays.tick(dt);
        // Dock motion only while its section is visible: an open panel
        // blinks its caret forever, which would busy-loop other sections.
        if self.section == Section::Dock {
            animating |= self.dock.tick(dt);
        }
        // Charts reveal only while visible: replay is click-triggered, so
        // off-screen frames would be wasted work.
        if self.section == Section::Charts {
            animating |= self.charts.tick(dt);
        }
        // Spinners rotate only while the Extras section is visible.
        if self.section == Section::Extras {
            animating |= self.extras.tick(dt);
        }
        if self.section == Section::Chrome {
            animating |= self.chrome.tick(dt);
        }
        // The todo app blinks its caret only while visible.
        if self.section == Section::App {
            animating |= self.app.tick(dt);
        }
        if self.section == Section::Editor {
            animating |= self.editor.tick(dt);
        }
        // The git tab always ticks: worker events land there whichever
        // section is showing, and must not pile up until it is.
        animating |= self.git.tick(dt) && self.section == Section::Git;

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
            let idx = self.section_idx();
            self.page_scroll[idx].scroll_to(offset);
        }
        let content = self.page_rect();
        let viewport = self.content_rect();

        // Clip the virtualized list (overscan rows) to its panel.
        let clip = if self.section == Section::Lists {
            let b = self.lists.list_bounds(content);
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

        // Scrolled section content clips to the viewport below the header
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
        match self.section {
            Section::Cards => self.cards.render(c, content, &theme),
            Section::Buttons => self.buttons.render(c, content, &theme),
            Section::Forms => self.forms.render(c, layers.overlay, content, &theme),
            Section::Overlays => {
                self.overlays.render(c, content, &theme);
                self.overlays
                    .render_tooltip(c, layers.toast, &theme, self.width, self.height);
            }
            Section::Lists => self.lists.render(c, layers.list, content, &theme),
            Section::Icons => self.icons_gallery.render(c, content, &theme),
            Section::Theme => self.themes.render(c, content, &theme, &self.theme_name),
            Section::Charts => self.charts.render(c, content, &theme),
            Section::Dock => self.dock.render(c, content, &theme),
            Section::App => self.app.render(c, content, &theme),
            Section::Builder => self.builder_tour.render(c, content, &theme),
            Section::Extras => self.extras.render(c, content, &theme),
            Section::Typography => self.typography.render(c, content, &theme),
            Section::Effects => self.effects.render(c, content, &theme),
            Section::Chrome => self.chrome.render(c, content, &theme),
            Section::Editor => self.editor.render(c, content),
            Section::Git => self.git.render(c, content, &theme),
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
}
