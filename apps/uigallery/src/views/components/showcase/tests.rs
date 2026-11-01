//! The showcase mold's tests.

use super::*;

/// `count` cards of a 36 px widget in each group: the button's.
fn buttons(groups: &[(usize, usize)], content: Rect, theme: &Theme) -> Vec<GroupLayout> {
    let groups = groups.iter().map(|&(span, n)| ("Group", span, n));
    layout_measured(groups, content, theme, |_, _, _| 36.0)
}

/// Narrow content: rows drop to one column instead of squeezing the
/// cards under the theme's least card width.
#[test]
fn cards_stack_in_narrow_content() {
    let theme = Theme::hoff_core();
    let content = Rect::new(0.0, 0.0, 500.0, 900.0);
    let layout = buttons(&[(2, 2), (3, 3)], content, &theme);
    for group in &layout {
        for r in &group.cards {
            assert_eq!(r.x, content.x);
            assert!(r.w >= card_min_w(&theme));
        }
    }
}

/// Wide content: a two-card span sits side by side, a three-card span
/// in threes, and the groups follow each other down the page.
#[test]
fn cards_fill_rows_in_wide_content() {
    let theme = Theme::hoff_core();
    let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
    let layout = buttons(&[(2, 2), (3, 3)], content, &theme);
    let first = &layout[0].cards;
    assert_eq!(first[0].y, first[1].y);
    assert!(first[1].x > first[0].x);
    let second = &layout[1].cards;
    assert_eq!(second[0].y, second[2].y);
    assert!(layout[1].title_y > layout[0].rule_y);
    assert!(content_height(&layout, content) > 2.0 * card_min_h(&theme));
}

/// A card is as tall as what it holds: alone in its row, a 36 px
/// widget's card is the chip's band twice and the widget, nothing
/// more; in a row of several the row keeps the theme's least card
/// height so the stages line up; a tall card grows its row only.
#[test]
fn a_card_is_as_tall_as_what_it_holds() {
    let theme = Theme::hoff_core();
    let floor = card_min_h(&theme);
    let alone = buttons(&[(3, 2)], Rect::new(0.0, 0.0, 320.0, 900.0), &theme);
    let snug = card_height(36.0, &theme);
    assert!(snug < floor, "a button's card needs less than h-50");
    assert!(alone[0].cards.iter().all(|r| r.h == snug));
    let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
    let wide = buttons(&[(3, 3)], content, &theme);
    assert!(wide[0].cards.iter().all(|r| r.h == floor));
    let tall = card_height(400.0, &theme);
    let sized = layout_sized(
        [("Group", 3, vec![snug, tall, snug, snug])],
        content,
        &theme,
    );
    let cards = &sized[0].cards;
    assert_eq!((cards[0].h, cards[1].h), (tall, tall), "the tallest card");
    assert_eq!(cards[3].h, floor, "the next row keeps the floor");
    assert_eq!(cards[3].y, cards[0].y + tall + theme.spacing.sm);
    assert_eq!(card_width(3, content, &theme), cards[0].w);
    // A body holds its content under the chip's band, its inset clear
    // below: the body of a card `body_height(h)` tall is `h` tall.
    let card = Rect::new(0.0, 0.0, 300.0, body_height(120.0, &theme));
    assert_eq!(body(card, &theme).h, 120.0);
}

/// A chip longer than its card is cut with an ellipsis inside it.
#[test]
fn a_long_chip_is_cut_to_its_card() {
    let theme = Theme::hoff_core();
    let inset = inset(&theme);
    let card = Rect::new(0.0, 0.0, 160.0, card_min_h(&theme));
    let text = chip_fit(
        "Default, Click on, Focus, Hover, Typing",
        card.w - inset * 2.0,
        &theme,
    );
    assert!(text.ends_with('\u{2026}'));
    let at = chip_rect(card.x + inset, card.y + inset, &text, &theme);
    assert!(at.right() <= card.right() - inset + 0.5);
    assert_eq!(chip_fit("Typed", card.w - inset * 2.0, &theme), "Typed");
}

/// A widget sits in the middle of its card, on whole pixels.
#[test]
fn a_widget_is_centered_in_its_card() {
    let card = Rect::new(10.0, 20.0, 301.0, 200.0);
    let r = centered((80.0, 36.0), card);
    assert_eq!((r.x, r.y), (121.0, 102.0));
    assert_eq!((r.w, r.h), (80.0, 36.0));
}

/// From 280 to 1600 the group's padding and title grow with the
/// content's width without a step: from `lg` and the narrow title to
/// `xxl` and h6, never more between two widths 8 px apart than the
/// line allows (rounded up to a pixel), and never back down.
#[test]
fn the_pad_and_the_title_are_fluid_with_the_width() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let [from, to] = theme.typography.roles.range;
        let slope = (theme.spacing.xxl - theme.spacing.lg) / (to - from);
        assert_eq!(pad(280.0, &theme), theme.spacing.lg);
        assert_eq!(pad(1600.0, &theme), theme.spacing.xxl);
        let h6 = theme.typography.role(TypeRole::H6);
        assert_eq!(title_style(1600.0, &theme).font_size, h6.font_size);
        let mut prev = (pad(272.0, &theme), title_style(272.0, &theme));
        for w in (280..=1600).step_by(8) {
            let w = w as f32;
            let (p, t) = (pad(w, &theme), title_style(w, &theme));
            assert!(
                p >= prev.0 && p - prev.0 <= (slope * 8.0).ceil(),
                "{w}: pad"
            );
            assert!(t.font_size >= prev.1.font_size, "{w}: title");
            assert!(t.font_size - prev.1.font_size <= 1.0, "{w}: title step");
            assert!(t.line_height - prev.1.line_height <= 1.0, "{w}: title line");
            // Every card of a group stays in the content, whatever
            // the width: nothing cut at either side.
            let content = Rect::new(0.0, 0.0, w, 900.0);
            for g in buttons(&[(3, 5), (2, 3), (1, 1)], content, &theme) {
                for r in &g.cards {
                    assert!(r.x >= 0.0 && r.right() <= w + 0.5, "{w}: cut");
                }
                assert!(g.cards.iter().all(|r| r.bottom() <= g.rule_y), "{w}");
            }
            prev = (p, t);
        }
    }
}

/// A group without a title starts its cards at its padding: no room
/// kept for a title it does not draw.
#[test]
fn an_untitled_group_keeps_no_room_for_a_title() {
    let theme = Theme::hoff_core();
    let content = Rect::new(0.0, 0.0, 800.0, 900.0);
    let grid = || Grid::new("Group", 3, card_min_w(&theme), vec![120.0; 3]);
    let titled = &layout_grids([grid()], content, &theme)[0];
    let bare = &layout_grids([grid().untitled()], content, &theme)[0];
    assert_eq!(bare.cards[0].y, bare.title_y);
    assert_eq!(bare.title_y, pad(content.w, &theme));
    let title_h = title_band_of("Group", content.w, &theme);
    assert_eq!(title_h.fract(), 0.0, "the cards on a whole pixel");
    assert_eq!(titled.cards[0].y - bare.cards[0].y, title_h);
    assert_eq!(titled.rule_y - bare.rule_y, title_h);
}

/// A title longer than its group's line wraps, whole, into as many
/// lines as it needs (a phone, twice the text size, a longer
/// language), and pushes its cards and the groups below down by them;
/// it is drawn whole, wrapped to the content.
#[test]
fn a_long_title_wraps_and_pushes_its_cards() {
    use engine::drawing::compositor::LayerId;
    use engine::styling::theme::{Prefs, TextScale};
    let long = "Notification Dropdown, its filters and a list that scrolls";
    for scale in [1.0, 2.0] {
        let prefs = Prefs::default().with_text_scale(TextScale::new(scale));
        let theme = Theme::hoff_core().adapted(&prefs);
        let content = Rect::new(0.0, 0.0, 300.0, 900.0);
        let grid = |t: &str| Grid::cards(t, 2, vec![120.0; 2], &theme);
        let short = layout_grids([grid("Menu"), grid("Menu")], content, &theme);
        let wrapped = layout_grids([grid(long), grid("Menu")], content, &theme);
        let line = title_style(content.w, &theme).line_height;
        let push = wrapped[0].cards[0].y - short[0].cards[0].y;
        assert!(
            push >= line - 0.5,
            "x{scale}: pushed by a line at least ({push})"
        );
        assert_eq!(push.fract(), 0.0, "on a whole pixel");
        assert_eq!(
            wrapped[1].title_y - short[1].title_y,
            push,
            "the next group too"
        );
        let mut c = Compositor::new();
        c.begin_frame();
        title(&mut c, long, content, wrapped[0].title_y, &theme);
        let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
        let Some(SceneNode::Text { key, .. }) = nodes.first() else {
            panic!("the title is drawn");
        };
        assert_eq!(key.text, long, "whole, never cut");
        assert_eq!(key.max_width_bits.map(f32::from_bits), Some(content.w));
        let (w, h) =
            TextMeasurer::measure_styled(long, &title_style(content.w, &theme), Some(content.w));
        assert!(w <= content.w + 0.5, "x{scale}: {w} in {}", content.w);
        assert!(
            wrapped[0].title_y + h <= wrapped[0].cards[0].y + 0.5,
            "clear of its cards"
        );
    }
}

/// The gray card is the plain one on the panel's tone, its chip on
/// the page's tone in a hairline: the same frame, chip and label at
/// the same places, two more nodes under and around them.
#[test]
fn the_gray_card_is_the_plain_one_on_the_panel_tone() {
    use engine::drawing::compositor::LayerId;
    let theme = Theme::hoff_core();
    let rect = Rect::new(10.0, 20.0, 400.0, 240.0);
    let nodes = |tone| {
        let mut c = Compositor::new();
        c.begin_frame();
        card_in(&mut c, rect, "Simple", tone, &theme);
        c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
    };
    let (plain, gray) = (nodes(Tone::Plain), nodes(Tone::Gray));
    assert_eq!(gray.len(), plain.len() + 2);
    let fill = |n: &SceneNode| match n {
        SceneNode::RoundedRect { color, .. } => Some(*color),
        _ => None,
    };
    assert_eq!(fill(&gray[0]), Some(theme.colors.bg_panel.0), "the card");
    assert_eq!(gray[1], plain[0], "its frame");
    assert_eq!(fill(&gray[2]), Some(theme.colors.bg.0), "the chip");
    assert_eq!(gray.last(), plain.last(), "its label");
}
