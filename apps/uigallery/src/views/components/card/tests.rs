//! The Card page: the CardsPage's groups and cards, the ranges, the
//! projection.

use super::*;

/// The kit's seven groups, in its order, with its chips.
#[test]
fn the_page_is_the_kit_s_cards_page() {
    let page = CardPage::new();
    assert!(page.root.is_some());
    let titles: Vec<_> = page.groups.iter().map(|g| g.title).collect();
    assert_eq!(
        titles,
        [
            "Pricing card",
            "Slide item",
            "Asset",
            "Asset",
            "Explore tags",
            "Explore tags",
            "Asset"
        ]
    );
    let chips: Vec<_> = page.groups[0].cards.iter().map(|c| c.title).collect();
    assert_eq!(chips, ["Free", "Studio", "Creator"]);
    assert_eq!(page.groups[3].cards[1].title, "Skeleton");
}

/// From 280 to 1600 every 8 px, under core and hoff: every widget sits in
/// its card, under the chip's band, and the groups follow one another.
#[test]
fn every_card_holds_its_widget_from_280_to_1600() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let page = CardPage::new();
        for w in (280..=1600).step_by(8) {
            let content = Rect::new(0.0, 0.0, w as f32, 800.0);
            let mut y = 0.0;
            for (placed, rects) in page.layout(content, &theme) {
                assert!(placed.title_y >= y, "{w}px: groups overlap");
                y = placed.rule_y;
                for (card, at) in placed.cards.iter().zip(&rects) {
                    let body = showcase::body(*card, &theme);
                    assert!(
                        at.x >= card.x - 0.5 && at.right() <= card.right() + 0.5,
                        "{w}px"
                    );
                    assert!(at.y >= body.y - 0.5, "{w}px: under the chip");
                    assert!(at.bottom() <= card.bottom() + 0.5, "{w}px: in the card");
                }
            }
        }
    }
}

/// The page projects a group per title and a node per card; the plan in
/// use refuses its button, a plan to take is chosen.
#[test]
fn the_cards_project_and_run() {
    let mut page = CardPage::new();
    let root = page.root.clone().unwrap();
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Card");
    page.project(&mut b, &root);
    let seen = b.finish().unwrap();
    let cards: usize = page.groups.iter().map(|g| g.cards.len()).sum();
    let ids = seen
        .nodes()
        .keys()
        .filter(|id| id.as_str().contains(".card["))
        .count();
    // Each card, and each pricing card's plan.
    assert_eq!(ids, cards + 3);
    let free = card_id(&root, 0, 0).unwrap().child("plan").unwrap();
    let revision = seen.revision();
    let req = |target: SemId| Request {
        actor: accord::Actor::Test {
            name: "card".into(),
        },
        target,
        action: accord::ActionName::parse("activate").expect("an action name"),
        param: None,
        observed: accord::Observation {
            revision,
            values: Vec::new(),
        },
        ask: None,
    };
    assert!(page.run(&req(free)).is_err());
    let studio = card_id(&root, 0, 1).unwrap().child("plan").unwrap();
    assert!(page.run(&req(studio)).is_ok());
    let scene = card_id(&root, 3, 0).unwrap();
    assert!(page.run(&req(scene)).is_ok());
}
