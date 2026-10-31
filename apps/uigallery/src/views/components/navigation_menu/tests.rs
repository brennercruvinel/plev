//! Navigation Menu page tests: ids, the one active mark a click, the
//! rail and a request move, the focus, and the width sweep. No GPU, no
//! mock.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::styling::theme::Theme;
use engine::widgets::prelude::{Rect, WidgetEvent};

use super::{After, Component, MENU, NavigationMenuPage, Shows};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("the id parses")
}

fn seen(page: &NavigationMenuPage) -> accord::Snapshot {
    let root = page.root.clone().expect("the root parses");
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Navigation Menu");
    page.project(&mut b, &root);
    b.finish().expect("the page projects a valid tree")
}

fn ask(page: &mut NavigationMenuPage, target: &str, action: &str) -> Result<After, String> {
    let req = Request {
        actor: accord::Actor::Test {
            name: "navigation_menu".into(),
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

fn active(page: &NavigationMenuPage) -> Vec<usize> {
    (0..page.links.len())
        .filter(|i| page.links[*i].active)
        .collect()
}

fn click(page: &mut NavigationMenuPage, r: Rect, content: Rect, theme: &Theme) -> bool {
    let (x, y) = r.center();
    page.handle_event(&WidgetEvent::MouseDown { x, y }, content, theme);
    page.handle_event(&WidgetEvent::MouseUp { x, y }, content, theme)
        .clicked
}

/// Every link has its id and is projected once, under its group.
#[test]
fn every_link_projects_once_under_its_group() {
    let page = NavigationMenuPage::new();
    let snap = seen(&page);
    for (i, link) in page.links.iter().enumerate() {
        let id = link.sem_id().expect("an id per link");
        assert_eq!(id, &sid(&format!("navigation_menu_page.link[{i}]")));
        assert_eq!(snap.get(id).map(|n| n.role), Some(Role::Link));
    }
    assert!(
        page.rail.iter().all(|l| l.sem_id().is_none()),
        "no id twice"
    );
}

/// A click on a link of any card, the rail's included, moves the one
/// active mark, and the rail follows the menu.
#[test]
fn a_click_moves_the_one_active_mark_and_the_rail_follows() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let mut page = NavigationMenuPage::new();
        assert_eq!(active(&page), vec![0]);
        let placed = page.placed(content, &theme);
        assert!(click(&mut page, placed[2][1].rows[0], content, &theme));
        assert_eq!(active(&page), vec![7]);
        assert!(page.rail.iter().all(|l| !l.active), "7 is not in the rail");

        assert!(click(&mut page, placed[0][1].rows[2], content, &theme));
        assert_eq!(active(&page), vec![2], "the rail moves the menu's mark");
        assert!(page.rail[2].active);
    }
}

/// `activate` does what the click does; anything else on a link, or a
/// link that is not there, is refused.
#[test]
fn activate_moves_the_mark_and_the_rest_is_refused() {
    let mut page = NavigationMenuPage::new();
    assert!(ask(&mut page, "navigation_menu_page.link[5]", "activate").is_ok());
    assert_eq!(active(&page), vec![5]);
    assert!(ask(&mut page, "navigation_menu_page.link[5]", "select").is_err());
    assert!(ask(&mut page, "navigation_menu_page.link[42]", "activate").is_err());
    assert!(ask(&mut page, "navigation_menu_page.group[0]", "activate").is_err());
    assert_eq!(active(&page), vec![5]);
}

/// The focus rings one link; blur lets it go; an id off the page is
/// not the page's.
#[test]
fn the_focus_rings_one_link() {
    let mut page = NavigationMenuPage::new();
    assert!(page.focus(&sid("navigation_menu_page.link[3]")));
    let rung: Vec<bool> = page.links.iter().map(|l| l.is_focused()).collect();
    assert_eq!(rung.iter().filter(|r| **r).count(), 1);
    assert!(rung[3]);
    assert!(page.blur());
    assert!(page.links.iter().all(|l| !l.is_focused()));
    assert!(!page.focus(&sid("breadcrumb_page.trail[0]")));
}

/// The width sweep of the epic, 280 to 1600 every 8 px, under the core
/// and the hoff themes: every card inside the content and above its
/// rule, cards apart, every row inside its card and below its chip,
/// rows apart, and the rail as many rows as the menu.
#[test]
fn nothing_cuts_or_overlaps_from_280_to_1600() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let page = NavigationMenuPage::new();
        for w in (280..=1600).step_by(8) {
            let content = Rect::new(0.0, 0.0, w as f32, 800.0);
            let layout = page.layout(content, &theme);
            let placed = page.placed(content, &theme);
            let mut prev_rule = content.y;
            for ((group, at), cards) in page.groups.iter().zip(&layout).zip(&placed) {
                assert!(at.title_y >= prev_rule, "{w}: groups in order");
                prev_rule = at.rule_y;
                for (k, (card, p)) in group.cards.iter().zip(cards).enumerate() {
                    let r = p.card;
                    assert!(
                        r.x >= content.x && r.right() <= content.right() + 0.5,
                        "{w}"
                    );
                    assert!(r.bottom() <= at.rule_y, "{w}: card above its rule");
                    if let Some(next) = at.cards.get(k + 1)
                        && next.y == r.y
                    {
                        assert!(next.x >= r.right(), "{w}: cards overlap");
                    }
                    let chip = theme.spacing.sm * 2.0 + theme.spacing.xs * 2.0;
                    let mut prev_bottom = r.y + chip;
                    for row in &p.rows {
                        assert!(row.x >= r.x && row.right() <= r.right() + 0.5, "{w}: row");
                        assert!(row.y >= prev_bottom - 0.5, "{w}: rows overlap");
                        assert!(row.bottom() <= r.bottom(), "{w}: row in its card");
                        prev_bottom = row.bottom();
                    }
                    if let Shows::Rail = card.shows {
                        assert_eq!(p.rows.len(), MENU);
                    }
                }
            }
        }
    }
}
