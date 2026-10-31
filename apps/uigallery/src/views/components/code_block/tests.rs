//! Code Block page tests: ids and what a reader hears, the run that
//! refuses, the wrap that grows the card, and the width sweep.

use accord::{Request, Role, SemId, SnapshotBuilder, Value};
use engine::styling::theme::Theme;
use engine::widgets::prelude::Rect;

use super::{CodeBlockPage, Component};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("the id parses")
}

fn seen(page: &CodeBlockPage) -> accord::Snapshot {
    let root = page.root.clone().expect("the root parses");
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Code Block");
    page.project(&mut b, &root);
    b.finish().expect("the page projects a valid tree")
}

/// Every block is a label named by its caption ("code" without one),
/// its code as its value, the rows of a diff one per line; no action.
#[test]
fn every_block_is_heard_with_its_code() {
    let page = CodeBlockPage::new();
    let snap = seen(&page);
    let bare = snap.get(&sid("code_block_page.group[0].card[0]")).unwrap();
    assert_eq!((bare.role, bare.label.as_str()), (Role::Label, "code"));
    assert_eq!(
        bare.state.get("value"),
        Some(&Value::Text("cargo test --workspace".into()))
    );
    let diff = snap.get(&sid("code_block_page.group[1].card[0]")).unwrap();
    assert_eq!(diff.label, "diff");
    let Some(Value::Text(text)) = diff.state.get("value") else {
        panic!("the diff has its text");
    };
    assert_eq!(text.lines().count(), 5);
    assert!(text.lines().nth(3).is_some_and(|l| l.starts_with('+')));
    assert!(snap.nodes().values().all(|n| n.actions.is_empty()));
}

/// A request on a block is refused; a reader's focus under the root is
/// the page's, any other is not.
#[test]
fn a_block_runs_nothing() {
    let mut page = CodeBlockPage::new();
    let req = Request {
        actor: accord::Actor::Test {
            name: "code".into(),
        },
        target: sid("code_block_page.group[0].card[0]"),
        action: accord::ActionName::parse("activate").expect("an action name"),
        param: None,
        observed: accord::Observation {
            revision: seen(&page).revision(),
            values: Vec::new(),
        },
        ask: None,
    };
    assert!(page.run(&req).is_err());
    assert!(page.focus(&sid("code_block_page.group[2].card[1]")));
    assert!(!page.focus(&sid("stat_page.group[0]")));
}

/// The long command wraps to more lines in a narrow card than in a wide
/// one, and its card grows past `h-50` with it.
#[test]
fn the_long_command_grows_its_card_when_narrow() {
    let theme = Theme::hoff_core();
    let page = CodeBlockPage::new();
    let card_h = |w: f32| page.layout(Rect::new(0.0, 0.0, w, 800.0), &theme)[2].cards[0].h;
    assert!(card_h(280.0) > card_h(1600.0));
}

/// The width sweep of the epic, 280 to 1600 every 8 px, under the core
/// and the hoff themes: every card inside the content and above its
/// rule, cards apart, every block inside its card and below its chip.
#[test]
fn nothing_cuts_or_overlaps_from_280_to_1600() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let page = CodeBlockPage::new();
        for w in (280..=1600).step_by(8) {
            let content = Rect::new(0.0, 0.0, w as f32, 800.0);
            let mut prev_rule = content.y;
            for (group, at) in page.groups.iter().zip(page.layout(content, &theme)) {
                assert!(at.title_y >= prev_rule, "{w}: groups in order");
                prev_rule = at.rule_y;
                for (i, (card, r)) in group.cards.iter().zip(&at.cards).enumerate() {
                    assert!(
                        r.x >= content.x && r.right() <= content.right() + 0.5,
                        "{w}"
                    );
                    assert!(r.bottom() <= at.rule_y, "{w}: card above its rule");
                    if let Some(next) = at.cards.get(i + 1)
                        && next.y == r.y
                    {
                        assert!(next.x >= r.right(), "{w}: cards overlap");
                    }
                    let b = CodeBlockPage::widget_rect(&card.block, *r, &theme);
                    let chip = theme.spacing.sm * 2.0 + theme.spacing.xs * 2.0;
                    assert!(
                        b.x >= r.x && b.right() <= r.right() + 0.5,
                        "{w}: {}",
                        card.title
                    );
                    assert!(
                        b.y >= r.y + chip && b.bottom() <= r.bottom(),
                        "{w}: {}",
                        card.title
                    );
                }
            }
        }
    }
}
