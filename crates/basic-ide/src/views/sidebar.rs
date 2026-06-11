//! Left rail — the HOFF Sidebar in its collapsed (72px) variant:
//! surface rgba(40,40,40,.8); NavLink items 48px tall, radius 12, with the
//! icon centered in a 32x32 slot; icons rgba($n2,.4) at rest, active item
//! gets bg rgba($n2,.1) + icon .76 + an edge-light 1px rgba(255,255,255,.1)
//! rim; the settings item sits in the 12px foot.

use crate::components::hoff;
use crate::theme::Theme;
use plev::compositor::{Compositor, SceneNode, TextNodeKey};

/// Collapsed HOFF sidebar width.
pub const SIDEBAR_W: f32 = 72.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SidebarTab {
    Workspace,
    Branches,
    History,
    Settings,
}

pub struct Sidebar {
    pub active: SidebarTab,
    hit_rects: Vec<(SidebarTab, f32, f32, f32, f32)>,
}

const PAD: f32 = 12.0;
const ITEM_H: f32 = 48.0;
const ITEM_GAP: f32 = 4.0;
const ICON_SIZE: f32 = 20.0;

impl Sidebar {
    pub fn new() -> Self {
        Self {
            active: SidebarTab::Workspace,
            hit_rects: Vec::new(),
        }
    }

    /// Hit-test a click position. Returns the tab if hit.
    pub fn hit_test(&self, cx: f32, cy: f32) -> Option<SidebarTab> {
        self.hit_rects.iter().find_map(|(tab, rx, ry, rw, rh)| {
            if cx >= *rx && cx <= rx + rw && cy >= *ry && cy <= ry + rh {
                Some(*tab)
            } else {
                None
            }
        })
    }

    fn draw_item(
        &mut self,
        compositor: &mut Compositor,
        theme: &Theme,
        y: f32,
        tab: SidebarTab,
        icon: &str,
    ) {
        let item_x = PAD;
        let item_w = SIDEBAR_W - PAD * 2.0;
        let is_active = self.active == tab;

        if is_active {
            // Active NavLink: bg rgba($n2,.1) + edge-light 1px rim.
            compositor.push(SceneNode::RoundedRect {
                x: item_x,
                y,
                w: item_w,
                h: ITEM_H,
                color: theme.surface_active.to_array(),
                corner_radius: theme.radius_nav,
                border_width: 0.0,
                border_color: [0.0; 4],
            });
            hoff::edge_light(
                compositor,
                plev::compositor::LayerId::DEFAULT,
                item_x,
                y,
                item_w,
                ITEM_H,
                theme.radius_nav,
                1.0,
                theme.edge_strong,
            );
        }

        // Icon (codicons font) centered in the 32x32 slot.
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new(icon, ICON_SIZE, ICON_SIZE, None)
                .with_weight(400)
                .with_family("codicon"),
            x: item_x + (item_w - ICON_SIZE) / 2.0,
            y: y + (ITEM_H - ICON_SIZE) / 2.0,
            color: if is_active {
                theme.text_active
            } else {
                theme.text_muted
            }
            .to_array(),
        });

        self.hit_rects.push((tab, item_x, y, item_w, ITEM_H));
    }

    pub fn render(&mut self, compositor: &mut Compositor, theme: &Theme, vh: f32, top_y: f32) {
        let h = vh - top_y;
        self.hit_rects.clear();

        // Sidebar surface — rgba(40,40,40,.8).
        compositor.push(SceneNode::Rect {
            x: 0.0,
            y: top_y,
            w: SIDEBAR_W,
            h,
            color: theme.bg_sidebar.to_array(),
        });

        // Top items: Workspace, Branches, History (menu gap 4px).
        let top_tabs = [
            (SidebarTab::Workspace, "\u{EB67}"), // codicon: layout
            (SidebarTab::Branches, "\u{EA68}"),  // codicon: git-branch
            (SidebarTab::History, "\u{EA82}"),   // codicon: history
        ];

        let mut y = top_y + PAD;
        for (tab, icon) in top_tabs {
            self.draw_item(compositor, theme, y, tab, icon);
            y += ITEM_H + ITEM_GAP;
        }

        // Foot: Settings pinned to the bottom (foot padding 12px).
        let settings_y = top_y + h - ITEM_H - PAD;
        self.draw_item(
            compositor,
            theme,
            settings_y,
            SidebarTab::Settings,
            "\u{EB52}", // codicon: settings-gear
        );
    }
}
