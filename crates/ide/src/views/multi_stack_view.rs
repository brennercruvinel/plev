//! Center "Stacks" column: the page surface where every commit is a
//! list card (the post recipe: card radius, surface wash at rest, hover
//! and selected washes, edge rim, inset key-light); an [`Avatar`] with
//! the author initial, the message in base-2sm at the primary tone,
//! `sha · author · time` in caption-r at the tertiary tone. Branch
//! headers show the success dot when the branch is checked out.

use comps::content::{Avatar, AvatarSize};
use comps::feedback::Scrollbar;
use comps::nav::PanelHeader;
use comps::prelude::{Rect, edge_light, inset_keylight, rounded_rect};
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::input::scroll::ScrollState;
use engine::text::TextMeasurer;
use engine::theme::{ControlSize, Theme};

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
    scrollbar: Scrollbar,
    /// Cached hit rects from last render (stack_idx, commit_idx, x, y, w, h).
    hit_rects: Vec<(usize, usize, f32, f32, f32, f32)>,
}

impl MultiStackView {
    /// Starts empty; the app injects real data via [`set_stacks`](Self::set_stacks).
    pub fn new() -> Self {
        Self {
            stacks: Vec::new(),
            selected_commit: None,
            scroll: ScrollState::new(),
            scrollbar: Scrollbar::new(),
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

    pub fn notify_scroll(&mut self) {
        self.scrollbar.notify_scroll();
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        self.scrollbar.tick(dt)
    }

    /// Branch header row: one `Xs` control.
    fn stack_header_h(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Xs)
    }

    /// Commit card: the avatar plus the card padding above and below.
    fn commit_h(theme: &Theme) -> f32 {
        AvatarSize::Md.px(theme) + theme.spacing.md * 2.0
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
        let header = PanelHeader::new("Stacks");
        let header_h = header.height(theme);
        let stack_header_h = Self::stack_header_h(theme);
        let commit_h = Self::commit_h(theme);
        let card_gap = theme.spacing.sm;
        let pad = theme.spacing.md;
        let glass = &theme.glass;

        let total_h: f32 = self
            .stacks
            .iter()
            .map(|s| stack_header_h + s.commits.len() as f32 * (commit_h + card_gap) + card_gap)
            .sum::<f32>();
        self.scroll.set_viewport(h - header_h);
        self.scroll.set_content(total_h);

        // Feed surface: the page tone.
        compositor.push(SceneNode::Rect {
            x,
            y,
            w,
            h,
            color: theme.colors.bg.0,
        });
        header.render(compositor, Rect::new(x, y, w, header_h), theme);

        let list_y = y + header_h;
        let list = Rect::new(x, list_y, w, h - header_h);
        let row_x = x + pad;
        let row_w = w - pad * 2.0;
        let scroll_offset = self.scroll.offset();
        let mut hit_rects = Vec::new();
        let mut cursor_y = list_y - scroll_offset;
        compositor.push(SceneNode::PushClip {
            x: list.x,
            y: list.y,
            w: list.w,
            h: list.h,
        });

        let branch_style = theme.typography.base_2sm();
        let msg_style = theme.typography.base_2sm();
        let meta_style = theme.typography.caption_r();
        let dot = theme.spacing.sm;
        for (si, stack) in self.stacks.iter().enumerate() {
            // Branch header: checked-out branch gets the success dot.
            if cursor_y + stack_header_h > list_y && cursor_y < y + h {
                let mut label_x = row_x + pad;
                if stack.is_active {
                    compositor.push(rounded_rect(
                        label_x,
                        cursor_y + (stack_header_h - dot) / 2.0,
                        dot,
                        dot,
                        dot / 2.0,
                        theme.colors.success.0,
                    ));
                    label_x += dot + theme.spacing.sm;
                }
                let avail = (row_x + row_w - pad - label_x).max(0.0);
                let name =
                    TextMeasurer::truncate_to_width(&stack.branch_name, &branch_style, avail);
                compositor.push(SceneNode::Text {
                    key: TextNodeKey::from_style(&name, &branch_style, None),
                    x: label_x,
                    y: cursor_y + TextMeasurer::vertical_center(&branch_style, stack_header_h),
                    color: if stack.is_active {
                        glass.text_active.0
                    } else {
                        glass.text_default.0
                    },
                });
            }
            cursor_y += stack_header_h;

            for (ci, commit) in stack.commits.iter().enumerate() {
                let cy = cursor_y;
                if cy + commit_h > list_y && cy < y + h {
                    let is_sel = self.selected_commit == Some((si, ci));
                    let is_hov = hover == Some((si, ci));
                    let card_bg = if is_sel {
                        glass.surface_active
                    } else if is_hov {
                        glass.surface_hover
                    } else {
                        glass.surface
                    };
                    let card = Rect::new(row_x, cy, row_w, commit_h);
                    compositor.push(rounded_rect(
                        card.x,
                        card.y,
                        card.w,
                        card.h,
                        theme.shape.card,
                        card_bg.0,
                    ));
                    edge_light(
                        compositor,
                        LayerId::DEFAULT,
                        card,
                        theme.shape.card,
                        theme.control.edge_width,
                        if is_sel || is_hov {
                            glass.edge.0
                        } else {
                            glass.edge_soft.0
                        },
                    );
                    inset_keylight(compositor, LayerId::DEFAULT, card, theme.shape.card, theme);

                    let avatar = Avatar::new(commit.author.clone());
                    let (av, _) = avatar.preferred_size(theme);
                    avatar.render(
                        compositor,
                        Rect::new(row_x + pad, cy + (commit_h - av) / 2.0, av, av),
                        theme,
                    );

                    // Text column: the message as the card headline, the
                    // meta line under it, both truncated with the style
                    // they are drawn with.
                    let text_x = row_x + pad + av + pad;
                    let text_w = row_w - av - pad * 3.0;
                    let block_h = msg_style.line_height + theme.spacing.xs + meta_style.line_height;
                    let top = cy + (commit_h - block_h) / 2.0;
                    let msg = TextMeasurer::truncate_to_width(&commit.message, &msg_style, text_w);
                    compositor.push(SceneNode::Text {
                        key: TextNodeKey::from_style(&msg, &msg_style, None),
                        x: text_x,
                        y: top,
                        color: theme.colors.text.0,
                    });
                    let sha_display = commit.sha.get(..7).unwrap_or(&commit.sha);
                    let meta = TextMeasurer::truncate_to_width(
                        &format!(
                            "{} \u{00B7} {} \u{00B7} {}",
                            sha_display, commit.author, commit.time_ago
                        ),
                        &meta_style,
                        text_w,
                    );
                    compositor.push(SceneNode::Text {
                        key: TextNodeKey::from_style(&meta, &meta_style, None),
                        x: text_x,
                        y: top + msg_style.line_height + theme.spacing.xs,
                        color: theme.colors.text_dim.0,
                    });
                }
                // Hit rect clamped to the visible part of the card: cards
                // hidden behind the panel head are not hoverable/clickable.
                let top = cy.max(list_y);
                let bottom = (cy + commit_h).min(y + h);
                if bottom > top {
                    hit_rects.push((si, ci, row_x, top, row_w, bottom - top));
                }
                cursor_y += commit_h + card_gap;
            }
            cursor_y += card_gap;
        }
        compositor.push(SceneNode::PopClip);

        self.scrollbar.render(compositor, list, &self.scroll, theme);

        self.hit_rects = hit_rects.clone();
        hit_rects
    }
}
