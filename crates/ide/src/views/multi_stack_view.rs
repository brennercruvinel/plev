//! Center "Stacks" column — HOFF feed container (rgba(40,40,40,.7)) where
//! every commit is a hoff list card (Post/Follower recipe): radius 20,
//! padding 12, 8px gap, bg rgba($n2,.02) -> hover .05 -> selected .10 +
//! a soft edge-light at rest (stronger when hovered/selected) and the inset
//! key-light; 44px avatar circle with the author initial, message (the card
//! headline) in base-2 semibold at rgba($n2,.95), sha + author + time in
//! caption-r at $text-tertiary (.50) — the white/.76/.50 hierarchy.
//! Branch headers show the 8px #55F08B dot when the branch is checked out.

use crate::components::hoff;
use crate::theme::Theme;
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::input::scroll::ScrollState;
use engine::text::TextStyle;

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
    // Panel geometry stays flat like every other render fn (card.rs
    // trade-off); a rect bag would be repacked at the call site.
    #[allow(clippy::too_many_arguments)]
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
                    // Top-lit edge: every card carries a soft rim like the
                    // HOFF post card; selected/hovered cards get the stronger
                    // .10 edge. Plus the inset key-light glint for glass depth.
                    let edge = if is_sel || is_hov {
                        theme.edge_strong
                    } else {
                        theme.edge
                    };
                    hoff::edge_light(
                        compositor,
                        LayerId::DEFAULT,
                        row_x,
                        cy,
                        row_w,
                        COMMIT_H,
                        theme.radius_card,
                        1.0,
                        edge,
                    );
                    hoff::inset_keylight(
                        compositor,
                        LayerId::DEFAULT,
                        row_x,
                        cy,
                        row_w,
                        COMMIT_H,
                        theme.radius_card,
                    );

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
                    // One style measures the initial AND draws it, so it
                    // sits centered in the 44px disc.
                    let initial_style =
                        TextStyle::new(14.0).with_line_height(14.0).with_weight(600);
                    let initial_w = hoff::measure_text(&initial, &initial_style);
                    compositor.push(SceneNode::Text {
                        key: TextNodeKey::from_style(&initial, &initial_style, None),
                        x: avatar_x + (AVATAR_SIZE - initial_w) / 2.0,
                        y: avatar_y + (AVATAR_SIZE - 14.0) / 2.0,
                        color: theme.text_active.to_array(),
                    });

                    // Text column.
                    let text_x = avatar_x + AVATAR_SIZE + PAD;
                    let text_w = row_w - AVATAR_SIZE - PAD * 3.0;

                    // Commit message — the card headline, like the HOFF post
                    // card's name: base-2 semibold (14/600) at text-primary
                    // (.95), the brightest line in the white/.76/.50 ramp.
                    // Truncated to the column width with the SAME style it
                    // is drawn with and rendered WITHOUT a wrap max (None)
                    // so a long message never spills onto a second line and
                    // collides with the meta row below.
                    let msg_style = TextStyle::new(14.0)
                        .with_line_height(14.0 * 1.4)
                        .with_weight(600);
                    let msg = hoff::truncate_to_width(&commit.message, text_w, &msg_style);
                    compositor.push(SceneNode::Text {
                        key: TextNodeKey::from_style(&msg, &msg_style, None),
                        x: text_x,
                        y: cy + PAD + 2.0,
                        color: theme.text_primary.to_array(),
                    });

                    // sha · author · time — caption-r at $text-tertiary.
                    let meta_style = TextStyle::new(12.0).with_line_height(12.0 * 1.33);
                    let sha_display = commit.sha.get(..7).unwrap_or(&commit.sha);
                    let meta = hoff::truncate_to_width(
                        &format!(
                            "{} \u{00B7} {} \u{00B7} {}",
                            sha_display, commit.author, commit.time_ago
                        ),
                        text_w,
                        &meta_style,
                    );
                    compositor.push(SceneNode::Text {
                        key: TextNodeKey::from_style(&meta, &meta_style, None),
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
