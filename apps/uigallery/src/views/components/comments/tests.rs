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
/// themes: every widget in its card's inside, clear of the chip's band;
/// the groups down the page.
#[test]
fn the_page_holds_over_the_whole_width_range() {
    let page = CommentsPage::new();
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
                }
            }
        }
    }
}

fn seen(page: &CommentsPage) -> accord::Snapshot {
    let root = page.root().cloned().expect("the root parses");
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Comments");
    page.project(&mut b, &root);
    b.finish().expect("the projection validates")
}

fn ask(page: &CommentsPage, target: &str, action: &str, param: Option<Value>) -> Request {
    let actor = Actor::Test {
        name: "comments page".to_owned(),
    };
    Request::observe(&seen(page), actor, target, action, param).expect("declared")
}

const THREAD: &str = "comments_page.group[1].card[1].thread";
const NEW: &str = "comments_page.group[1].card[0].new";
const QUICK: &str = "comments_page.group[3].card[0].quick";

/// A reply sent from the projection ends the thread; the close button
/// closes it to its pin and the pin opens it again.
#[test]
fn a_thread_takes_a_reply_closes_and_opens() {
    let mut page = CommentsPage::new();
    let text = Some(Value::Text("On it".into()));
    let req = ask(&page, &format!("{THREAD}.reply"), "set_value", text);
    assert_eq!(page.run(&req), Ok(After::Nothing));
    let req = ask(&page, &format!("{THREAD}.send"), "activate", None);
    assert_eq!(page.run(&req), Ok(After::Nothing));
    let s = seen(&page);
    let last = SemId::parse(&format!("{THREAD}.messages[1]")).unwrap();
    assert_eq!(s.get(&last).map(|n| n.label.as_str()), Some("Sam, now"));
    let req = ask(&page, &format!("{THREAD}.close"), "activate", None);
    assert_eq!(page.run(&req), Ok(After::Nothing));
    assert!(seen(&page).get(&last).is_none(), "closed to its pin");
    let req = ask(&page, &format!("{THREAD}.pin"), "activate", None);
    assert_eq!(page.run(&req), Ok(After::Nothing));
    assert!(seen(&page).get(&last).is_some(), "open again");
}

/// The new comment opens with its first words, takes a picture from its
/// add tool, and posts.
#[test]
fn a_new_comment_opens_attaches_and_posts() {
    let mut page = CommentsPage::new();
    let text = Some(Value::Text("Nice".into()));
    let req = ask(&page, &format!("{NEW}.text"), "set_value", text);
    assert_eq!(page.run(&req), Ok(After::Nothing));
    let req = ask(&page, &format!("{NEW}.tools[0]"), "activate", None);
    assert_eq!(page.run(&req), Ok(After::Nothing));
    let s = seen(&page);
    let tile = SemId::parse(&format!("{NEW}.files[0]")).unwrap();
    assert!(s.get(&tile).is_some(), "a picture came");
    let req = ask(&page, &format!("{NEW}.send"), "activate", None);
    assert_eq!(page.run(&req), Ok(After::Nothing));
    let s = seen(&page);
    let new = s.get(&SemId::parse(NEW).unwrap()).unwrap();
    assert_eq!(new.state.get("posted"), Some(&Value::Int(1)));
    assert!(s.get(&tile).is_none(), "the box emptied");
}

/// The quick comment's `Send` posts the reply as the replier's comment.
#[test]
fn a_quick_comment_posts_its_reply() {
    let mut page = CommentsPage::new();
    let text = Some(Value::Text("Agreed".into()));
    let req = ask(&page, &format!("{QUICK}.reply"), "set_value", text);
    assert_eq!(page.run(&req), Ok(After::Nothing));
    let req = ask(&page, &format!("{QUICK}.send"), "activate", None);
    assert_eq!(page.run(&req), Ok(After::Nothing));
    let s = seen(&page);
    let list = s
        .get(&SemId::parse(&format!("{QUICK}.comments")).unwrap())
        .unwrap();
    assert_eq!(list.state.get("count"), Some(&Value::Int(2)));
}
