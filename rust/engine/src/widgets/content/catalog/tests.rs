//! The catalog cards: the kit's widths, the heights from what they hold,
//! the ranges, the hover, the choice and the projection.

use accord::{Role, SemId, SnapshotBuilder, Value};

use super::*;
use crate::drawing::compositor::LayerId;

fn core() -> Theme {
    Theme::hoff_core()
}

fn price(plan: Plan) -> CatalogCard {
    CatalogCard::new(CatalogKind::Price {
        plan,
        title: "Studio".into(),
        description: "For teams and studios that need power, speed.".into(),
        price: "40".into(),
        features: vec![
            "Everything in Creator".into(),
            "Unlimited 3D scene generation".into(),
            "Full access to premium asset library".into(),
        ],
    })
}

fn all() -> Vec<CatalogCard> {
    vec![
        price(Plan::Studio),
        CatalogCard::new(CatalogKind::Asset),
        CatalogCard::new(CatalogKind::Scene {
            title: "Futuristic Humanoid Robot".into(),
            category: "Untitled Folder".into(),
            video: false,
        }),
        CatalogCard::new(CatalogKind::SceneSkeleton),
        CatalogCard::new(CatalogKind::Explore {
            title: "Tran Mau Tri Tam".into(),
        }),
        CatalogCard::new(CatalogKind::ExploreSm {
            title: "Product Design".into(),
            description: "Minimalist, soft".into(),
        }),
        CatalogCard::new(CatalogKind::Slide),
    ]
}

fn nodes(card: &CatalogCard, b: Rect, theme: &Theme) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    card.render(&mut c, b, theme);
    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

/// At their ease the cards are the kit's widths: `w-80`, `w-65.5`
/// square, `w-65`, 360 by 288, `w-59`, the 52 px slide.
#[test]
fn the_cards_take_the_kit_s_widths() {
    let t = core();
    let w: Vec<f32> = all().iter().map(|c| c.size(1000.0, &t).0).collect();
    assert_eq!(w, [320.0, 262.0, 260.0, 260.0, 360.0, 236.0, 52.0]);
    let asset = CatalogCard::new(CatalogKind::Asset).size(1000.0, &t);
    assert_eq!(asset, (262.0, 262.0));
    let explore = all()[4].size(1000.0, &t);
    assert_eq!(explore, (360.0, 288.0));
    assert_eq!(all()[5].size(1000.0, &t).1, 82.0);
    // The skeleton holds the scene's place.
    assert_eq!(all()[2].size(1000.0, &t), all()[3].size(1000.0, &t));
}

/// From 120 to 1600 every card fits the room it is given, and a wider
/// pricing card is never taller (its text wraps less).
#[test]
fn every_card_fits_its_room_from_120_to_1600() {
    for theme in [core(), Theme::hoff()] {
        for card in all() {
            let mut last_h = f32::INFINITY;
            for w in (120..=1600).step_by(8) {
                let (cw, ch) = card.size(w as f32, &theme);
                assert!(cw <= w as f32 && ch > 0.0, "{} at {w}", card.label());
                if matches!(card.kind, CatalogKind::Price { .. }) {
                    assert!(ch <= last_h + 0.01, "wider is never taller");
                }
                last_h = ch;
                let b = Rect::new(0.0, 0.0, cw, ch);
                for n in nodes(&card, b, &theme) {
                    if let SceneNode::Text { x, .. } = n {
                        assert!(x >= b.x && x <= b.right(), "{} at {w}", card.label());
                    }
                }
            }
        }
    }
}

/// The hover shows the actions the kit shows under the pointer.
#[test]
fn the_hover_shows_the_actions() {
    let t = core();
    for i in [1, 2, 4, 5] {
        let mut card = all().remove(i);
        let (w, h) = card.size(1000.0, &t);
        let b = Rect::new(0.0, 0.0, w, h);
        let rest = nodes(&card, b, &t).len();
        card.handle_event(&WidgetEvent::MouseMove { x: 10.0, y: 10.0 }, b, &t);
        assert!(card.is_hovered());
        assert!(nodes(&card, b, &t).len() > rest, "{}", card.label());
    }
}

/// A click chooses a card; on a pricing card its plan's button does, and
/// the plan in use takes no click.
#[test]
fn a_click_chooses_a_card_and_a_plan() {
    let t = core();
    let mut asset = CatalogCard::new(CatalogKind::Asset);
    let b = Rect::new(0.0, 0.0, 262.0, 262.0);
    asset.handle_event(&WidgetEvent::MouseDown { x: 50.0, y: 50.0 }, b, &t);
    let r = asset.handle_event(&WidgetEvent::MouseUp { x: 50.0, y: 50.0 }, b, &t);
    assert!(r.clicked && asset.is_selected());
    for (plan, takes) in [(Plan::Studio, true), (Plan::Free, false)] {
        let mut card = price(plan);
        let (w, h) = card.size(1000.0, &t);
        let b = Rect::new(0.0, 0.0, w, h);
        // A click on the panel is not a choice: the button is.
        card.handle_event(&WidgetEvent::MouseDown { x: 30.0, y: 70.0 }, b, &t);
        card.handle_event(&WidgetEvent::MouseUp { x: 30.0, y: 70.0 }, b, &t);
        assert!(!card.is_selected());
        let (x, y) = card.button_rect(b, &t).unwrap().center();
        card.handle_event(&WidgetEvent::MouseDown { x, y }, b, &t);
        card.handle_event(&WidgetEvent::MouseUp { x, y }, b, &t);
        assert_eq!(card.is_selected(), takes, "{plan:?}");
    }
}

/// A card projects a button with `selected` and `activate`; a pricing
/// card a group around its plan's button, disabled for the plan in use.
#[test]
fn a_card_projects_a_button_and_a_plan_a_group() {
    let id = SemId::parse("cards.asset").unwrap();
    let mut b = SnapshotBuilder::new();
    CatalogCard::new(CatalogKind::Asset)
        .selected(true)
        .project(&mut b, &id, None);
    let seen = b.finish().unwrap();
    let node = seen.get(&id).unwrap();
    assert_eq!(node.role, Role::Button);
    assert_eq!(node.state.get("selected"), Some(&Value::Bool(true)));
    let id = SemId::parse("cards.free").unwrap();
    let mut b = SnapshotBuilder::new();
    price(Plan::Free).project(&mut b, &id, None);
    let seen = b.finish().unwrap();
    assert_eq!(seen.get(&id).unwrap().role, Role::Group);
    let plan = seen.get(&id.child("plan").unwrap()).unwrap();
    assert_eq!(plan.role, Role::Button);
    assert!(!plan.enabled);
}

/// Every theme keeps the kit's geometry and its photos; the palette is
/// the theme's.
#[test]
fn the_other_themes_keep_the_geometry_in_their_palette() {
    let kit = CatalogTokens::hoff_core();
    for theme in [Theme::hoff(), Theme::dark(), Theme::light()] {
        let t = &theme.disclosure.catalog;
        assert_eq!(
            (&t.price, &t.tile, &t.explore),
            (&kit.price, &kit.tile, &kit.explore)
        );
        assert_eq!(t.palette.paper, theme.colors.surface);
        assert_eq!(t.palette.ink, theme.colors.text);
        assert_eq!(t.palette.image, kit.palette.image);
    }
}
