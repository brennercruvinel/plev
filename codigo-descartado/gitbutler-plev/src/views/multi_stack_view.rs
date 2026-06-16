use plev::compositor::{Compositor, SceneNode, TextNodeKey};
use plev::scroll::ScrollState;
use crate::theme::Theme;

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
    pub selected_commit: Option<(usize, usize)>,  // (stack_idx, commit_idx)
    pub scroll: ScrollState,
    /// Cached hit rects from last render (stack_idx, commit_idx, x, y, w, h).
    hit_rects: Vec<(usize, usize, f32, f32, f32, f32)>,
}

const HEADER_H: f32 = 44.0;
const STACK_HEADER_H: f32 = 36.0;
const COMMIT_H: f32 = 56.0;
const PAD_X: f32 = 12.0;
const AVATAR_SIZE: f32 = 28.0;
const FONT_SIZE: f32 = 13.0;

impl MultiStackView {
    pub fn new() -> Self {
        Self {
            stacks: mock_stacks(),
            selected_commit: None,
            scroll: ScrollState::new(),
            hit_rects: Vec::new(),
        }
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
        if self.selected_commit == sel { return false; }
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
        let total_h: f32 = self.stacks.iter().map(|s| {
            STACK_HEADER_H + s.commits.len() as f32 * COMMIT_H + 8.0
        }).sum::<f32>();
        self.scroll.set_viewport(h - HEADER_H);
        self.scroll.set_content(total_h);

        // Panel background
        compositor.push(SceneNode::Rect {
            x, y, w, h,
            color: theme.bg_2.to_array(),
        });

        // Top header
        compositor.push(SceneNode::Rect {
            x, y, w, h: HEADER_H,
            color: theme.bg_3.to_array(),
        });
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new("Stacks", 12.0, 16.0, None).with_weight(600),
            x: x + PAD_X,
            y: y + 14.0,
            color: theme.text_2.to_array(),
        });
        compositor.push(SceneNode::Rect {
            x, y: y + HEADER_H, w, h: 1.0,
            color: theme.border.to_array(),
        });

        let list_y = y + HEADER_H + 1.0;
        let scroll_offset = self.scroll.offset();
        let mut hit_rects = Vec::new();
        let mut cursor_y = list_y - scroll_offset;

        for (si, stack) in self.stacks.iter().enumerate() {
            // Stack header bar
            if cursor_y + STACK_HEADER_H > list_y && cursor_y < y + h {
                let header_bg = if stack.is_active { theme.bg_3 } else { theme.bg_2 };
                compositor.push(SceneNode::Rect {
                    x, y: cursor_y, w, h: STACK_HEADER_H,
                    color: header_bg.to_array(),
                });
                // Branch name
                compositor.push(SceneNode::Text {
                    key: TextNodeKey::new(&stack.branch_name, FONT_SIZE, FONT_SIZE * 1.3, Some(w - PAD_X * 2.0)).with_weight(600),
                    x: x + PAD_X,
                    y: cursor_y + (STACK_HEADER_H - FONT_SIZE * 1.3) / 2.0,
                    color: theme.text_1.to_array(),
                });
                // Active indicator dot
                if stack.is_active {
                    compositor.push(SceneNode::RoundedRect {
                        x: x + 4.0,
                        y: cursor_y + STACK_HEADER_H / 2.0 - 3.0,
                        w: 6.0,
                        h: 6.0,
                        color: theme.pop.to_array(),
                        corner_radius: 3.0,
                        border_width: 0.0,
                        border_color: [0.0; 4],
                    });
                }
                compositor.push(SceneNode::Rect {
                    x, y: cursor_y + STACK_HEADER_H - 1.0, w, h: 1.0,
                    color: theme.border.to_array(),
                });
            }
            cursor_y += STACK_HEADER_H;

            // Commits
            for (ci, commit) in stack.commits.iter().enumerate() {
                let cy = cursor_y;
                if cy + COMMIT_H > list_y && cy < y + h {
                    let is_sel = self.selected_commit == Some((si, ci));
                    let is_hov = hover == Some((si, ci));
                    let row_bg = if is_sel { theme.bg_3 }
                        else if is_hov { theme.hover_bg_2 }
                        else { theme.bg_2 };
                    compositor.push(SceneNode::Rect {
                        x, y: cy, w, h: COMMIT_H,
                        color: row_bg.to_array(),
                    });
                    // Avatar circle
                    let avatar_x = x + PAD_X;
                    let avatar_y = cy + (COMMIT_H - AVATAR_SIZE) / 2.0;
                    compositor.push(SceneNode::RoundedRect {
                        x: avatar_x, y: avatar_y, w: AVATAR_SIZE, h: AVATAR_SIZE,
                        color: theme.bg_3.to_array(),
                        corner_radius: AVATAR_SIZE / 2.0,
                        border_width: 0.0,
                        border_color: [0.0; 4],
                    });
                    // Author initial
                    let initial = commit.author.chars().next().map(|c| c.to_string()).unwrap_or_default();
                    compositor.push(SceneNode::Text {
                        key: TextNodeKey::new(&initial, 12.0, 14.0, None).with_weight(600),
                        x: avatar_x + (AVATAR_SIZE - 8.0) / 2.0,
                        y: avatar_y + (AVATAR_SIZE - 14.0) / 2.0,
                        color: theme.text_2.to_array(),
                    });
                    // Commit message (truncated)
                    let msg = truncate(&commit.message, 40);
                    compositor.push(SceneNode::Text {
                        key: TextNodeKey::new(&msg, FONT_SIZE, FONT_SIZE * 1.3, Some(w - AVATAR_SIZE - PAD_X * 3.0)).with_weight(400),
                        x: avatar_x + AVATAR_SIZE + 8.0,
                        y: cy + 10.0,
                        color: theme.text_1.to_array(),
                    });
                    // Author + time
                    let meta = format!("{} \u{00B7} {}", commit.author, commit.time_ago);
                    compositor.push(SceneNode::Text {
                        key: TextNodeKey::new(&meta, 11.0, 14.0, Some(w - AVATAR_SIZE - PAD_X * 3.0)).with_weight(400),
                        x: avatar_x + AVATAR_SIZE + 8.0,
                        y: cy + 10.0 + FONT_SIZE * 1.3 + 2.0,
                        color: theme.text_3.to_array(),
                    });
                    // SHA tag (guard against short hashes)
                    let sha_display = commit.sha.get(..7).unwrap_or(&commit.sha);
                    compositor.push(SceneNode::Text {
                        key: TextNodeKey::new(sha_display, 10.0, 12.0, None).with_weight(500),
                        x: x + w - PAD_X - 40.0,
                        y: cy + (COMMIT_H - 12.0) / 2.0,
                        color: theme.text_3.to_array(),
                    });
                    compositor.push(SceneNode::Rect {
                        x, y: cy + COMMIT_H - 1.0, w, h: 1.0,
                        color: theme.border.to_array(),
                    });
                }
                hit_rects.push((si, ci, x, cy, w, COMMIT_H));
                cursor_y += COMMIT_H;
            }
            cursor_y += 8.0; // gap between stacks
        }

        // Scrollbar
        if self.scroll.is_scrollable() {
            draw_scrollbar(compositor, theme, x + w - 4.0, list_y, h - HEADER_H - 1.0, &self.scroll);
        }

        self.hit_rects = hit_rects.clone();
        hit_rects
    }
}

fn draw_scrollbar(compositor: &mut Compositor, theme: &Theme, x: f32, y: f32, h: f32, scroll: &ScrollState) {
    let thumb_h = (h * scroll.thumb_ratio()).max(24.0);
    let thumb_y = y + (h - thumb_h) * scroll.thumb_position();
    compositor.push(SceneNode::Rect {
        x, y: thumb_y, w: 4.0, h: thumb_h,
        color: [theme.text_3.0[0], theme.text_3.0[1], theme.text_3.0[2], 0.5],
    });
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max - 1).collect();
        format!("{}\u{2026}", t)
    }
}

fn mock_stacks() -> Vec<Stack> {
    vec![
        Stack {
            branch_name: "feat/gitbutler-plev-frontend".into(),
            is_active: true,
            commits: vec![
                CommitEntry {
                    sha: "a3f2d91e".into(),
                    message: "feat: add theme.rs with dark/light design tokens".into(),
                    author: "Brenner".into(),
                    time_ago: "2m ago".into(),
                },
                CommitEntry {
                    sha: "9b1c4e72".into(),
                    message: "feat: Phase 0 — font_weight and truncation in builder".into(),
                    author: "Brenner".into(),
                    time_ago: "15m ago".into(),
                },
                CommitEntry {
                    sha: "7d8a3f10".into(),
                    message: "feat: ScrollState in scroll.rs".into(),
                    author: "Brenner".into(),
                    time_ago: "1h ago".into(),
                },
            ],
        },
        Stack {
            branch_name: "task/TASK-39-scene-memoization".into(),
            is_active: false,
            commits: vec![
                CommitEntry {
                    sha: "e5c2b9a1".into(),
                    message: "feat: PartialEq-based dirty flag bubbling for scene cache".into(),
                    author: "Brenner".into(),
                    time_ago: "2d ago".into(),
                },
                CommitEntry {
                    sha: "f3d7a840".into(),
                    message: "test: 12 memoization tests for Component<L>".into(),
                    author: "Brenner".into(),
                    time_ago: "2d ago".into(),
                },
            ],
        },
        Stack {
            branch_name: "task/TASK-38-event-batching".into(),
            is_active: false,
            commits: vec![
                CommitEntry {
                    sha: "aaea8f03".into(),
                    message: "feat: BufferedEvent drain in about_to_wait — 5-10x GPU reduction".into(),
                    author: "Brenner".into(),
                    time_ago: "3d ago".into(),
                },
            ],
        },
    ]
}
