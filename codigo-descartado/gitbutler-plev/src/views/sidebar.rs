use plev::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::theme::Theme;

pub const SIDEBAR_W: f32 = 48.0;

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

const ICON_SIZE: f32 = 20.0;
const ITEM_H: f32 = 44.0;

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

    pub fn render(&mut self, compositor: &mut Compositor, theme: &Theme, vh: f32, top_y: f32) {
        let h = vh - top_y;
        self.hit_rects.clear();

        // Sidebar background
        compositor.push(SceneNode::Rect {
            x: 0.0, y: top_y, w: SIDEBAR_W, h,
            color: theme.bg_2.to_array(),
        });

        // Right border
        compositor.push(SceneNode::Rect {
            x: SIDEBAR_W - 1.0, y: top_y, w: 1.0, h,
            color: theme.border.to_array(),
        });

        // Top icons: Workspace, Branches, History
        let top_tabs = [
            (SidebarTab::Workspace, "\u{EB67}"),  // codicon: layout
            (SidebarTab::Branches,  "\u{EA68}"),   // codicon: git-branch
            (SidebarTab::History,   "\u{EA82}"),   // codicon: history
        ];

        let mut y = top_y + 8.0;
        for (tab, icon) in &top_tabs {
            let is_active = self.active == *tab;

            // Active indicator (12x18 rounded pill, like real GitButler)
            if is_active {
                let ind_w = 4.0;
                let ind_h = 18.0;
                compositor.push(SceneNode::RoundedRect {
                    x: 0.0,
                    y: y + (ITEM_H - ind_h) / 2.0,
                    w: ind_w,
                    h: ind_h,
                    color: theme.pop.to_array(),
                    corner_radius: 2.0,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                });
            }

            // Icon (codicons font)
            compositor.push(SceneNode::Text {
                key: TextNodeKey::new(icon, ICON_SIZE, ICON_SIZE, None)
                    .with_weight(400)
                    .with_family("codicon"),
                x: (SIDEBAR_W - ICON_SIZE) / 2.0,
                y: y + (ITEM_H - ICON_SIZE) / 2.0,
                color: if is_active { theme.text_1 } else { theme.text_3 }.to_array(),
            });

            self.hit_rects.push((*tab, 0.0, y, SIDEBAR_W, ITEM_H));
            y += ITEM_H;
        }

        // Bottom icon: Settings (pushed to bottom)
        let settings_y = top_y + h - ITEM_H - 8.0;
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new("\u{EB52}", ICON_SIZE, ICON_SIZE, None)  // codicon: settings-gear
                .with_weight(400)
                .with_family("codicon"),
            x: (SIDEBAR_W - ICON_SIZE) / 2.0,
            y: settings_y + (ITEM_H - ICON_SIZE) / 2.0,
            color: theme.text_3.to_array(),
        });
        self.hit_rects.push((SidebarTab::Settings, 0.0, settings_y, SIDEBAR_W, ITEM_H));
    }
}
