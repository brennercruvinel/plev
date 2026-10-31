//! Breadcrumb page tests: ids, the click and its echo, the current place
//! that is no target, the focus, and the width sweep. No GPU, no mock.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::styling::theme::Theme;
use engine::widgets::prelude::{Rect, WidgetEvent};

use super::{After, BreadcrumbPage, Component};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("the id parses")
}

/// The page under its region, as the view projects it; the tree must be
/// valid.
fn seen(page: &BreadcrumbPage) -> accord::Snapshot {
    let root = page.root.clone().expect("the root parses");
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Breadcrumb");
    page.project(&mut b, &root);
    b.finish().expect("the page projects a valid tree")
}

/// `action` on `target`, asked by a test having seen the page now.
fn ask(page: &mut BreadcrumbPage, target: &str, action: &str) -> Result<After, String> {
    let req = Request {
        actor: accord::Actor::Test {
            name: "breadcrumb".into(),
        },
        target: sid(target),
        action: accord::ActionName::parse(action).expect("an action name"),
        param: None,
        observed: accord::Observation {
            revision: seen(page).revision(),
            values: Vec::new(),
        },
        ask: None,
    };
    page.run(&req)
}

/// Every id parses, so the page projects.
#[test]
fn the_page_has_its_root_and_an_id_per_trail() {
    let page = BreadcrumbPage::new();
    assert!(page.root.is_some());
    for group in &page.groups {
        assert!(group.cards.iter().all(|c| c.trail.sem_id().is_some()));
    }
    assert_eq!(
        page.groups[0].cards[0].trail.sem_id(),
        Some(&sid("breadcrumb_page.trail"))
    );
}

/// A click on a crumb echoes it; a press that leaves the crumb before
/// the release does not.
#[test]
fn a_crumb_click_is_echoed_and_a_slide_off_is_not() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let content = Rect::new(40.0, 80.0, 1272.0, 900.0);
        let mut page = BreadcrumbPage::new();
        let card = page.layout(content, &theme)[0].cards[0];
        let trail = &page.groups[0].cards[0].trail;
        let bounds = BreadcrumbPage::widget_rect(trail, card, &theme);
        let rects = trail.crumb_rects(bounds, &theme);
        let (x, y) = rects[1].1.center();
        page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        assert_eq!(page.groups[0].cards[0].trail.pressed(), Some(1));
        let r = page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme);
        assert!(r.clicked);
        assert_eq!(page.clicked, Some((0, 0, 1)));
        assert_eq!(page.echo((0, 0, 1)), "clicked crumb 1");

        let (x2, y2) = rects[2].1.center();
        page.handle_event(&WidgetEvent::MouseDown { x: x2, y: y2 }, content, &theme);
        let r = page.handle_event(&WidgetEvent::MouseUp { x: 0.0, y: 0.0 }, content, &theme);
        assert!(!r.clicked, "released off the crumb");
        assert_eq!(page.clicked, Some((0, 0, 1)), "the echo stays");
    }
}

/// The current place is where the user is: no click, no `activate`, no
/// focus. A crumb that leads somewhere takes all three.
#[test]
fn the_current_place_is_not_a_target() {
    let theme = Theme::hoff_core();
    let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
    let mut page = BreadcrumbPage::new();
    let card = page.layout(content, &theme)[0].cards[0];
    let trail = &page.groups[0].cards[0].trail;
    let bounds = BreadcrumbPage::widget_rect(trail, card, &theme);
    let (x, y) = trail.crumb_rects(bounds, &theme)[3].1.center();
    page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
    let r = page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme);
    assert!(!r.clicked);
    assert!(ask(&mut page, "breadcrumb_page.trail[3]", "activate").is_err());
    assert!(ask(&mut page, "breadcrumb_page.single[0]", "activate").is_err());
    assert!(ask(&mut page, "breadcrumb_page.trail[9]", "activate").is_err());
    assert!(ask(&mut page, "breadcrumb_page.trail[1]", "drag").is_err());
    assert!(page.focus(&sid("breadcrumb_page.trail[3]")));
    assert_eq!(page.focus, None, "the current place takes no focus");

    assert!(ask(&mut page, "breadcrumb_page.folded[2]", "activate").is_ok());
    assert_eq!(page.clicked, Some((1, 0, 2)));
    assert_eq!(page.echo((1, 0, 2)), "clicked crumb 2 in folded");
}

/// The focus a reader asks for rings that crumb alone, on its trail
/// alone; blur lets it go; an id off the page is not the page's.
#[test]
fn the_focus_rings_one_crumb_of_one_trail() {
    let mut page = BreadcrumbPage::new();
    assert!(page.focus(&sid("breadcrumb_page.translated[1]")));
    assert_eq!(page.focus, Some((1, 1, 1)));
    assert_eq!(page.groups[1].cards[1].trail.focused(), Some(1));
    assert_eq!(page.groups[0].cards[0].trail.focused(), None);
    assert!(page.focus(&sid("breadcrumb_page.trail[0]")));
    assert_eq!(page.groups[1].cards[1].trail.focused(), None, "moved");
    assert!(page.blur());
    assert!(!page.blur());
    assert!(!page.focus(&sid("toggle_page.group[0]")));
}

/// The width sweep of the epic, 280 to 1600 every 8 px, under the core
/// and the hoff themes: every card inside the content and above its
/// rule, the cards of a row apart, every trail inside its card, every
/// drawn crumb inside its trail and after the one before it, and the
/// current place always drawn.
#[test]
fn nothing_cuts_or_overlaps_from_280_to_1600() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let page = BreadcrumbPage::new();
        for w in (280..=1600).step_by(8) {
            let content = Rect::new(0.0, 0.0, w as f32, 800.0);
            let mut prev_rule = content.y;
            for (group, at) in page.groups.iter().zip(page.layout(content, &theme)) {
                assert!(at.title_y >= prev_rule, "{w}: groups in order");
                prev_rule = at.rule_y;
                for (i, (card, r)) in group.cards.iter().zip(&at.cards).enumerate() {
                    assert!(
                        r.x >= content.x && r.right() <= content.right() + 0.5,
                        "{w}: card in the content"
                    );
                    assert!(r.bottom() <= at.rule_y, "{w}: card above its rule");
                    if let Some(next) = at.cards.get(i + 1)
                        && next.y == r.y
                    {
                        assert!(next.x >= r.right(), "{w}: cards overlap");
                    }
                    let t = BreadcrumbPage::widget_rect(&card.trail, *r, &theme);
                    assert!(t.x >= r.x && t.right() <= r.right() + 0.5, "{w}: trail");
                    assert!(t.y >= r.y && t.bottom() <= r.bottom(), "{w}");
                    let crumbs = card.trail.crumb_rects(t, &theme);
                    let mut prev_right = t.x;
                    for (_, c) in &crumbs {
                        assert!(c.x >= prev_right - 0.5, "{w}: crumbs overlap");
                        assert!(c.right() <= t.right() + 0.5, "{w}: crumb out");
                        assert!(c.y >= t.y && c.bottom() <= t.bottom() + 0.5, "{w}");
                        prev_right = c.right();
                    }
                    let last = card.trail.crumbs.len() - 1;
                    assert_eq!(crumbs.last().and_then(|c| c.0), Some(last), "{w}");
                }
            }
        }
    }
}

/// The deep trail folds where its card is narrow and the short one
/// never does: the room, not a breakpoint, decides.
#[test]
fn the_deep_trail_folds_only_when_its_card_is_narrow() {
    let theme = Theme::hoff_core();
    let page = BreadcrumbPage::new();
    let folded_at = |g: usize, c: usize, w: f32| {
        let layout = page.layout(Rect::new(0.0, 0.0, w, 800.0), &theme);
        let trail = &page.groups[g].cards[c].trail;
        let t = BreadcrumbPage::widget_rect(trail, layout[g].cards[c], &theme);
        trail.visible(t.w, &theme).contains(&None)
    };
    assert!(folded_at(1, 0, 280.0));
    assert!(folded_at(1, 0, 1600.0), "ten crumbs never fit half a row");
    assert!(!folded_at(0, 0, 1600.0));
    assert!(!folded_at(0, 1, 280.0), "one crumb never folds");
}
