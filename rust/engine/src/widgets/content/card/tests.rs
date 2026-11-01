//! Every text of every card family stays inside its card at any width:
//! a block (a value, a label, a title, a caption, a body) wraps in the
//! width it has there and the card grows by what it wraps
//! ([`Card::height_for`]); a line beside something (a name before its
//! action, a row's label before its value, a chip's or a button's label)
//! is cut with an ellipsis.

use super::{Card, CardListRow, CardVariant};
use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use crate::widgets::widget::Rect;

const LONG: &str = "a label the translation made three times longer than the one it had";

fn long(n: usize) -> String {
    vec![LONG; n].join(" ")
}

fn families() -> Vec<CardVariant> {
    vec![
        CardVariant::Stat {
            value: long(1),
            label: long(2),
            delta: Some((long(1), true)),
        },
        CardVariant::Profile {
            name: long(1),
            username: long(1),
            bio: long(2),
            action: long(1),
            online: true,
            avatar: None,
        },
        CardVariant::Media {
            title: long(1),
            caption: long(2),
            badge: Some(long(1)),
            image: None,
        },
        CardVariant::List {
            title: long(1),
            rows: vec![
                CardListRow::new(long(1), long(1)).active(true),
                CardListRow::new(long(1), long(1)).progress(0.4),
            ],
        },
        CardVariant::Chart {
            value: long(1),
            label: long(1),
            groups: vec![(0.4, 0.8), (0.6, 0.2)],
            highlight: 0,
        },
        CardVariant::Cta {
            title: long(1),
            body: long(2),
            button: long(1),
        },
    ]
}

/// The style a text node was shaped with.
fn style(key: &crate::drawing::compositor::TextNodeKey) -> TextStyle {
    TextStyle {
        font_size: f32::from_bits(key.font_size_bits),
        line_height: f32::from_bits(key.line_height_bits),
        font_weight: key.font_weight,
        letter_spacing: f32::from_bits(key.letter_spacing_bits),
        font_family: key.font_family.clone(),
        overflow_wrap: key.overflow_wrap,
    }
}

#[test]
fn every_card_text_stays_inside_its_card_at_any_width() {
    const E: f32 = 0.5;
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        for variant in families() {
            let card = Card::new(variant);
            for w in (280..=640).step_by(40) {
                let w = w as f32;
                let b = Rect::new(0.0, 0.0, w, card.height_for(w, &theme));
                let mut c = Compositor::new();
                c.begin_frame();
                card.render(&mut c, b, &theme);
                let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
                for n in nodes {
                    let SceneNode::Text { key, x, y, .. } = n else {
                        continue;
                    };
                    let max_w = key.max_width_bits.map(f32::from_bits);
                    let (tw, th) = TextMeasurer::measure_styled(&key.text, &style(&key), max_w);
                    let what = format!("{w}: {:?} {:?}", card.variant_name(), key.text);
                    assert!(x >= b.x - E && x + tw <= b.right() + E, "{what}: across");
                    assert!(y >= b.y - E && y + th <= b.bottom() + E, "{what}: down");
                }
            }
        }
    }
}

/// A block that wraps grows the card: the long stat card is taller than
/// the short one at the same width, and taller at 280 than at 640.
#[test]
fn a_wrapping_text_grows_its_card() {
    let theme = Theme::hoff_core();
    let stat = |value: &str| {
        Card::new(CardVariant::Stat {
            value: value.into(),
            label: "Visitors".into(),
            delta: None,
        })
    };
    let (short, tall) = (stat("12k"), stat(&long(1)));
    assert!(tall.height_for(320.0, &theme) > short.height_for(320.0, &theme));
    assert!(tall.height_for(280.0, &theme) > tall.height_for(640.0, &theme));
    assert_eq!(
        short.height_for(280.0, &theme),
        short.height_for(640.0, &theme),
        "a short one keeps its line"
    );
}

impl Card {
    fn variant_name(&self) -> &'static str {
        match self.variant {
            CardVariant::Stat { .. } => "stat",
            CardVariant::Profile { .. } => "profile",
            CardVariant::Media { .. } => "media",
            CardVariant::List { .. } => "list",
            CardVariant::Chart { .. } => "chart",
            CardVariant::Cta { .. } => "cta",
        }
    }
}
