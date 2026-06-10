//! Center "Stacks" column — HOFF feed container (rgba(40,40,40,.7)) where
//! every commit is a hoff list card (Post/Follower recipe): radius 20,
//! padding 12, 8px gap, bg rgba($n2,.02) -> hover .05 -> selected .10 +
//! edge-light; 44px avatar circle with the author initial, message in
//! base-2m at rgba($n2,.76), sha + time in caption-r at $text-tertiary.
//! Branch headers show the 8px #55F08B dot when the branch is checked out.

use crate::components::hoff;
use crate::theme::Theme;
use plev::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use plev::scroll::ScrollState;

/// A commit in a stack.
#[derive(Clone, Debug)]
pub struct CommitEntry {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub time_ago: String,
}

/// A branch/stack containing commits.
#[derive(Clone, Debug)]
pub struct Stack {
    pub branch_name: String,
    pub commits: Vec<CommitEntry>,
    pub is_active: bool,
}

/// State for the center "Stacks" panel.
pub struct MultiStackView {
    pub stacks: Vec<Stack>,
    pub selected_commit: Option<(usize, usize)>, // (stack_idx, commit_idx)
    pub scroll: ScrollState,
    /// Cached hit rects from last render (stack_idx, commit_idx, x, y, w, h).
    hit_rects: Vec<(usize, usize, f32, f32, f32, f32)>,
}

const HEADER_H: f32 = 68.0;
const STACK_HEADER_H: f32 = 36.0;
const COMMIT_H: f32 = 68.0;
const CARD_GAP: f32 = 8.0;
const PAD: f32 = 12.0;
const AVATAR_SIZE: f32 = 44.0;

impl MultiStackView {
    /// Starts empty; the app injects real data via [`set_stacks`](Self::set_stacks).
    pub fn new() -> Self {
        Self {
            stacks: Vec::new(),
            selected_commit: None,
            scroll: ScrollState::new(),
            hit_rects: Vec::new(),
        }
    }

    /// Replaces the stacks (e.g. from fresh `git log`/`branches` data),
    /// keeping the selection on the same commit sha when it still exists.
    pub fn set_stacks(&mut self, stacks: Vec<Stack>) {
        let selected_sha = self
            .selected_commit
            .and_then(|(si, ci)| self.stacks.get(si).and_then(|s| s.commits.get(ci)))
            .map(|c| c.sha.clone());
        self.stacks = stacks;
        self.selected_commit = selected_sha.and_then(|sha| {
            self.stacks.iter().enumerate().find_map(|(si, stack)| {
                stack
                    .commits
                    .iter()
                    .position(|c| c.sha == sha)
                    .map(|ci| (si, ci))
            })
        });
    }

    /// Hit-test a screen position against commit rows. Returns (stack_idx, commit_idx) if hit.
    pub fn hit_test(&self, cx: f32, cy: f32) -> Option<(usize, usize)> {
        self.hit_rects.iter().find_map(|(si, ci, rx, ry, rw, rh)| {
            if cx >= *rx && cx <= rx + rw && cy >= *ry && cy <= ry + rh {
                Some((*si, *ci))
            } else {
                None
            }
        })
    }

    /// Select commit by indices. Returns true if selection changed.
    pub fn select(&mut self, sel: Option<(usize, usize)>) -> bool {
        if self.selected_commit == sel {
            return false;
        }
        self.selected_commit = sel;
        true
    }

    /// Returns hit rects: Vec<(stack_idx, commit_idx, x, y, w, h)>
    pub fn render(
        &mut self,
        compositor: &mut Compositor,
        theme: &Theme,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        hover: Option<(usize, usize)>,
    ) -> Vec<(usize, usize, f32, f32, f32, f32)> {
        // Compute total content height
        let total_h: f32 = self
            .stacks
            .iter()
            .map(|s| STACK_HEADER_H + s.commits.len() as f32 * (COMMIT_H + CARD_GAP) + 8.0)
            .sum::<f32>();
        self.scroll.set_viewport(h - HEADER_H);
        self.scroll.set_content(total_h);

        // Feed surface — $bg-surface rgba(40,40,40,.7).
        compositor.push(SceneNode::Rect {
            x,
            y,
            w,
            h,
            color: theme.bg_panel.to_array(),
        });

        // Head — title (20/500) at .56.
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new("Stacks", 20.0, 20.0 * 1.2, None).with_weight(500),
            x: x + PAD,
            y: y + (HEADER_H - 20.0 * 1.2) / 2.0,
            color: theme.text_default.to_array(),
        });

        let list_y = y + HEADER_H;
        let row_x = x + PAD;
        let row_w = w - PAD * 2.0;
        let scroll_offset = self.scroll.offset();
        let mut hit_rects = Vec::new();
        let mut cursor_y = list_y - scroll_offset;
        // Scrolled feed clips to the area below the panel head.
        compositor.push(SceneNode::PushClip {
            x,
            y: list_y,
            w,
            h: h - HEADER_H,
        });

        for (si, stack) in self.stacks.iter().enumerate() {
            // Branch header — base-2sm; checked-out branch gets the green dot.
            if cursor_y + STACK_HEADER_H > list_y && cursor_y < y + h {
                let mut label_x = row_x + PAD;
                if stack.is_active {
                    let dot = 8.0;
                    compositor.push(SceneNode::RoundedRect {
                        x: label_x,
                        y: cursor_y + STACK_HEADER_H / 2.0 - dot / 2.0,
                        w: dot,
                        h: dot,
                        color: theme.accent_green.to_array(),
                        corner_radius: dot / 2.0,
                        border_width: 0.0,
                        border_color: [0.0; 4],
                    });
                    label_x += dot + 8.0;
                }
                compositor.push(SceneNode::Text {
                    key: TextNodeKey::new(
                        &stack.branch_name,
                        14.0,
                        14.0 * 1.4,
                        Some(row_w - PAD * 2.0),
                    )
                    .with_weight(600),
                    x: label_x,
                    y: cursor_y + (STACK_HEADER_H - 14.0 * 1.4) / 2.0,
                    color: if stack.is_active {
                        theme.text_active
                    } else {
                        theme.text_default
                    }
                    .to_array(),
                });
            }
            cursor_y += STACK_HEADER_H;

            // Commit cards.
            for (ci, commit) in stack.commits.iter().enumerate() {
                let cy = cursor_y;
                if cy + COMMIT_H > list_y && cy < y + h {
                    let is_sel = self.selected_commit == Some((si, ci));
                    let is_hov = hover == Some((si, ci));
                    let card_bg = if is_sel {
                        theme.surface_active
                    } else if is_hov {
                        theme.surface_hover
                    } else {
                        theme.surface
                    };
                    compositor.push(SceneNode::RoundedRect {
                        x: row_x,
                        y: cy,
                        w: row_w,
                        h: COMMIT_H,
                        color: card_bg.to_array(),
                        corner_radius: theme.radius_card,
                        border_width: 0.0,
                        border_color: [0.0; 4],
                    });
                    if is_sel {
                        hoff::edge_light(
                            compositor,
                            LayerId::DEFAULT,
                            row_x,
                            cy,
                            row_w,
                            COMMIT_H,
                            theme.radius_card,
                            1.0,
                            theme.edge_strong,
                        );
                    }

                    // Avatar — 44px circle with the author initial.
                    let avatar_x = row_x + PAD;
                    let avatar_y = cy + (COMMIT_H - AVATAR_SIZE) / 2.0;
                    compositor.push(SceneNode::RoundedRect {
                        x: avatar_x,
                        y: avatar_y,
                        w: AVATAR_SIZE,
                        h: AVATAR_SIZE,
                        color: theme.chip.to_array(),
                        corner_radius: AVATAR_SIZE / 2.0,
                        border_width: 0.0,
                        border_color: [0.0; 4],
                    });
                    let initial = commit
                        .author
                        .chars()
                        .next()
                        .map(|c| c.to_uppercase().to_string())
                        .unwrap_or_default();
                    let initial_w = hoff::text_width(&initial, 14.0);
                    compositor.push(SceneNode::Text {
                        key: TextNodeKey::new(&initial, 14.0, 14.0, None).with_weight(600),
                        x: avatar_x + (AVATAR_SIZE - initial_w) / 2.0,
                        y: avatar_y + (AVATAR_SIZE - 14.0) / 2.0,
                        color: theme.text_active.to_array(),
                    });

                    // Text column.
                    let text_x = avatar_x + AVATAR_SIZE + PAD;
                    let text_w = row_w - AVATAR_SIZE - PAD * 3.0;

                    // Commit message — base-2m (14/500) at .76.
                    let msg = truncate(&commit.message, 48);
                    compositor.push(SceneNode::Text {
                        key: TextNodeKey::new(&msg, 14.0, 14.0 * 1.4, Some(text_w))
                            .with_weight(500),
                        x: text_x,
                        y: cy + PAD + 2.0,
                        color: theme.text_active.to_array(),
                    });

                    // sha · author · time — caption-r at $text-tertiary.
                    let sha_display = commit.sha.get(..7).unwrap_or(&commit.sha);
                    let meta = format!(
                        "{} \u{00B7} {} \u{00B7} {}",
                        sha_display, commit.author, commit.time_ago
                    );
                    compositor.push(SceneNode::Text {
                        key: TextNodeKey::new(&meta, 12.0, 12.0 * 1.33, Some(text_w))
                            .with_weight(400),
                        x: text_x,
                        y: cy + PAD + 2.0 + 14.0 * 1.4 + 4.0,
                        color: theme.text_tertiary.to_array(),
                    });
                }
                // Hit rect clamped to the visible part of the card: cards
                // hidden behind the panel head are not hoverable/clickable.
                let top = cy.max(list_y);
                let bottom = (cy + COMMIT_H).min(y + h);
                if bottom > top {
                    hit_rects.push((si, ci, row_x, top, row_w, bottom - top));
                }
                cursor_y += COMMIT_H + CARD_GAP;
            }
            cursor_y += 8.0; // gap between stacks
        }
        compositor.push(SceneNode::PopClip);

        // Scrollbar
        if self.scroll.is_scrollable() {
            hoff::draw_scrollbar(
                compositor,
                theme,
                x + w - 4.0,
                list_y,
                h - HEADER_H,
                &self.scroll,
            );
        }

        self.hit_rects = hit_rects.clone();
        hit_rects
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max - 1).collect();
        format!("{}\u{2026}", t)
    }
}
