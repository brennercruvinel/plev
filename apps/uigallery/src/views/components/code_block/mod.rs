//! Code Block page, on the showcase mold of the kit's ButtonsPage
//! (`showcase`): groups with a title and a bottom rule, cards in rows, a
//! chip naming what each card shows. The widget is
//! `engine::widgets::content::CodeBlock`; the kit has none, so its recipe
//! (`theme.navigation.code`) derives from the kit's pricing well and its
//! type table.
//!
//! The page shows the ranges a block meets: bare and captioned, a diff
//! with every kind of row, a long command that wraps at the card's width
//! and a translated caption. A block takes its card's width and grows the
//! card as its code wraps.
//!
//! Projected: a group per title, `code_block_page.group[<g>]`, and in it
//! each card's block, `code_block_page.group[<g>].card[<c>]`, a label
//! named by its caption, its code as its value. A block is read, not
//! operated: no action.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::{CodeBlock, CodeLine, LineKind};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, root_id, under};

struct Card {
    title: &'static str,
    block: CodeBlock,
}

struct Group {
    title: &'static str,
    span: usize,
    cards: Vec<Card>,
}

pub struct CodeBlockPage {
    root: Option<SemId>,
    groups: Vec<Group>,
}

/// `code_block_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `code_block_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn diff() -> Vec<CodeLine> {
    vec![
        CodeLine::new("src/lib.rs: @@ -1,4 +1,4 @@", LineKind::Header),
        CodeLine::new(" fn main() {", LineKind::Context),
        CodeLine::new("-    let theme = Theme::hoff();", LineKind::Removed),
        CodeLine::new("+    let theme = Theme::hoff_core();", LineKind::Added),
        CodeLine::new(" }", LineKind::Context),
    ]
}

impl CodeBlockPage {
    pub fn new() -> Self {
        let card = |title, block| Card { title, block };
        let mut groups = vec![
            Group {
                title: "Code block",
                span: 2,
                cards: vec![
                    card("Default", CodeBlock::new("cargo test --workspace")),
                    card(
                        "Caption",
                        CodeBlock::new("cargo run -p uigallery -- --render \"Code Block\"")
                            .caption("shell"),
                    ),
                ],
            },
            Group {
                title: "Diff",
                span: 1,
                cards: vec![card(
                    "Context, Added, Removed, Header",
                    CodeBlock::lines(diff()).caption("diff"),
                )],
            },
            Group {
                title: "Long content",
                span: 2,
                cards: vec![
                    card(
                        "Wrapped",
                        CodeBlock::new(
                            "cargo run -p regress -- check --page \"Code Block\" --theme core \
                             --width 1280 --height 800 --scale 1 && cargo test -p engine widgets",
                        )
                        .caption("shell"),
                    ),
                    card(
                        "Translated caption",
                        CodeBlock::new("tema = \"hoff_core\"\nlargura = 1280")
                            .caption("configuração da galeria.toml"),
                    ),
                ],
            },
        ];
        let root = root_id("code_block_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        card.block = card.block.clone().sem(id);
                    }
                }
            }
        }
        Self { root, groups }
    }

    /// The block's size in a card `card_w` wide: the card's width less
    /// its inset on both sides, as tall as its code wraps there.
    fn widget_size(block: &CodeBlock, card_w: f32, theme: &Theme) -> (f32, f32) {
        let w = showcase::room(card_w, theme);
        (w, block.height_for(w, theme))
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().map(|g| {
            let w = showcase::card_width(g.span, content, theme);
            let heights = g
                .cards
                .iter()
                .map(|c| showcase::card_height(Self::widget_size(&c.block, w, theme).1, theme))
                .collect();
            (g.title, g.span, heights)
        });
        showcase::layout_sized(groups, content, theme)
    }

    fn widget_rect(block: &CodeBlock, card: Rect, theme: &Theme) -> Rect {
        showcase::centered(Self::widget_size(block, card.w, theme), card)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    pub fn handle_event(
        &mut self,
        _event: &WidgetEvent,
        _content: Rect,
        _theme: &Theme,
    ) -> EventResult {
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (g, (group, at)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, at.title_y, theme);
            for (card, rect) in group.cards.iter().zip(&at.cards) {
                showcase::card(c, *rect, card.title, theme);
                let r = Self::widget_rect(&card.block, *rect, theme);
                card.block.render(c, r, theme);
            }
            if g != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
    }
}

impl Component for CodeBlockPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(at) = group_id(root, g) else {
                continue;
            };
            b.node(&at, Role::Group, group.title).parent(root);
            for card in &group.cards {
                if let Some(id) = card.block.sem_id() {
                    card.block.project(b, id, Some(&at)).description(card.title);
                }
            }
        }
    }

    /// A block offers nothing to run.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        Err(format!(
            "the code block page has no {} on {}",
            req.action, req.target
        ))
    }

    /// A reader lands on a block to hear it; nothing here draws a focus,
    /// so the page takes it and keeps none.
    fn focus(&mut self, id: &SemId) -> bool {
        self.root.as_ref().is_some_and(|root| under(root, id))
    }
}

#[cfg(test)]
mod tests;
