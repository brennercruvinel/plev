use super::*;
use accord::{Actor, Value};

const EPS: f32 = 0.5;

fn within(inner: Rect, outer: Rect) -> bool {
    inner.x >= outer.x - EPS
        && inner.y >= outer.y - EPS
        && inner.right() <= outer.right() + EPS
        && inner.bottom() <= outer.bottom() + EPS
}

/// The width sweep, 280 to 1600 every 8 px, in the core and the hoff
/// themes: every widget in its card's inside, the groups down the page,
/// the dropdown no taller than the kit's.
#[test]
fn the_page_holds_over_the_whole_width_range() {
    let page = NotificationsPage::new();
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        for w in (280..=1600).step_by(8) {
            let content = Rect::new(24.0, 0.0, w as f32, 800.0);
            let mut prev = content.y;
            for (group, (placed, boxes)) in page.groups.iter().zip(page.layout(content, &theme)) {
                assert!(placed.title_y >= prev, "{w}: {} climbs", group.title);
                prev = placed.rule_y;
                for ((card, rect), at) in group.cards.iter().zip(&placed.cards).zip(&boxes) {
                    let inside = showcase::body(*rect, &theme);
                    assert!(within(*at, *rect), "{w}: {} leaves its card", card.title);
                    assert!(at.w <= inside.w + EPS, "{w}: {} too wide", card.title);
                    assert!(at.h <= theme.message.notifications.max_h + EPS);
                }
            }
        }
    }
}

fn seen(page: &NotificationsPage) -> accord::Snapshot {
    let root = page.root().cloned().expect("the root parses");
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Notifications");
    page.project(&mut b, &root);
    b.finish().expect("the projection validates")
}

fn ask(page: &NotificationsPage, target: &str, action: &str) -> Request {
    let actor = Actor::Test {
        name: "notifications page".to_owned(),
    };
    Request::observe(&seen(page), actor, target, action, None).expect("declared")
}

const DROPDOWN: &str = "notifications_page.group[1].card[0].dropdown";

/// The Unread filter shows the five unread; reading one under it takes it
/// off and the rows after it take its place; answering the invite reads
/// it.
#[test]
fn the_filter_shows_the_unread_and_a_read_one_leaves_it() {
    let mut page = NotificationsPage::new();
    let count = |page: &NotificationsPage| {
        let s = seen(page);
        let list = SemId::parse(&format!("{DROPDOWN}.list")).unwrap();
        s.get(&list).and_then(|n| n.state.get("count").cloned())
    };
    assert_eq!(count(&page), Some(Value::Int(8)));
    let req = ask(&page, &format!("{DROPDOWN}.filter[1]"), "select");
    assert_eq!(page.run(&req), Ok(After::Nothing));
    assert_eq!(count(&page), Some(Value::Int(5)));
    let req = ask(&page, &format!("{DROPDOWN}.list[0]"), "activate");
    assert_eq!(page.run(&req), Ok(After::Nothing));
    assert_eq!(count(&page), Some(Value::Int(4)));
    // the invite is now the second unread
    let req = ask(&page, &format!("{DROPDOWN}.list[1].action[1]"), "activate");
    assert_eq!(page.run(&req), Ok(After::Nothing));
    assert_eq!(count(&page), Some(Value::Int(3)));
}

/// A variation is read by `activate`, and a reader's focus on a row of
/// the dropdown is projected and lets go.
#[test]
fn a_variation_is_read_and_a_row_takes_the_focus() {
    let mut page = NotificationsPage::new();
    let comment = "notifications_page.group[0].card[0].alert";
    let req = ask(&page, comment, "activate");
    assert_eq!(page.run(&req), Ok(After::Nothing));
    let s = seen(&page);
    let node = s.get(&SemId::parse(comment).unwrap()).unwrap();
    assert_eq!(node.state.get("unread"), Some(&Value::Bool(false)));
    let row = SemId::parse(&format!("{DROPDOWN}.list[6]")).unwrap();
    assert!(page.focus(&row));
    assert_eq!(seen(&page).focused(), Some(&row));
    assert!(page.blur());
    assert_eq!(seen(&page).focused(), None);
}
