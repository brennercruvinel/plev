//! Sidebar event routing: pinned footer links, a wheel over the band,
//! group captions that fold, the links, the drawer's close-on-outside.

use engine::theme::{SidebarMode, Theme};

use crate::core::{EventResult, WidgetEvent};

use super::Sidebar;

impl Sidebar {
    /// Route an event. Returns the result and the index of the link that
    /// was activated, if any. A drawer closes on navigation and on a
    /// press outside it.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) -> (EventResult, Option<usize>) {
        let mode = self.mode(theme, vw);
        let Some(rect) = self.rect(theme, vw, vh) else {
            return (EventResult::IGNORED, None);
        };
        self.sync_scroll(rect, theme, mode);
        let band = self.band(rect, theme, mode);

        // Pinned footer links first: they never scroll.
        let footer_rects = self.footer_link_rects(rect, theme);
        let base = self.links.len();
        let mut footer_clicked = None;
        let mut footer_result = EventResult::IGNORED;
        for (i, (link, r)) in self.footer_links.iter_mut().zip(&footer_rects).enumerate() {
            let lr = link.handle_event(event, *r);
            if lr.clicked {
                footer_clicked = Some(base + i);
            }
            footer_result = footer_result.merge(lr);
        }
        if let Some(i) = footer_clicked {
            self.set_active(i);
            if mode == SidebarMode::Drawer {
                self.drawer_open = false;
            }
            return (EventResult::clicked(), Some(i));
        }

        if let WidgetEvent::Scroll { x, y, delta } = *event
            && band.contains(x, y)
        {
            let before = self.scroll.offset();
            self.scroll.scroll_by(delta);
            return if self.scroll.offset() != before {
                (EventResult::changed(), None)
            } else {
                (
                    EventResult {
                        handled: true,
                        ..EventResult::IGNORED
                    },
                    None,
                )
            };
        }

        // A press on a group caption folds or unfolds it.
        if let WidgetEvent::MouseDown { x, y } = *event
            && band.contains(x, y)
            && let Some(g) = self
                .group_rects(rect, theme, mode)
                .iter()
                .position(|r| r.contains(x, y))
        {
            self.toggle_group(g);
            return (EventResult::changed(), None);
        }

        let rects = self.link_rects(rect, theme, mode);
        let mut result = footer_result;
        let mut clicked = None;
        for (i, (link, r)) in self.links.iter_mut().zip(&rects).enumerate() {
            // Links outside the band, or folded away, are clipped: they
            // cannot be hit.
            let (px, py) = event.pos();
            if r.h == 0.0 || (!band.contains(px, py) && !link.is_hovered()) {
                continue;
            }
            let lr = link.handle_event(event, *r);
            if lr.clicked {
                clicked = Some(i);
            }
            result = result.merge(lr);
        }
        if let Some(i) = clicked {
            self.set_active(i);
            if mode == SidebarMode::Drawer {
                self.drawer_open = false;
            }
            return (EventResult::clicked(), Some(i));
        }
        if mode == SidebarMode::Drawer
            && let WidgetEvent::MouseDown { x, y } = *event
            && !rect.contains(x, y)
        {
            self.drawer_open = false;
            return (EventResult::changed(), None);
        }
        // The sidebar surface swallows what lands on it.
        if let (x, y) = event.pos()
            && rect.contains(x, y)
        {
            result.handled = true;
        }
        (result, None)
    }
}
