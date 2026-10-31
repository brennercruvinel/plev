//! Stat page tests: ids and the projection a reader hears, the run that
//! refuses, the delta that wraps when narrow, and the width sweep.

use accord::{Request, Role, SemId, SnapshotBuilder, Value};
use engine::styling::theme::Theme;
use engine::widgets::prelude::Rect;

use super::{Component, StatPage};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("the id parses")
}

fn seen(page: &StatPage) -> accord::Snapshot {
    let root = page.root.clone().expect("the root parses");
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Stat");
    page.project(&mut b, &root);
    b.finish().expect("the page projects a valid tree")
}

/// Every readout is a label named by its label, its value with the unit
/// as its value, the delta as its description, and nothing to run.
#[test]
fn every_readout_is_heard_as_a_label_with_its_value() {
    let page = StatPage::new();
    let snap = seen(&page);
    let node = snap
        .get(&sid("stat_page.group[0].card[1]"))
        .expect("the medium readout is projected");
    assert_eq!(node.role, Role::Label);
    assert_eq!(node.label, "on disk");
    assert_eq!(node.state.get("value"), Some(&Value::Text("1.3 GB".into())));
    assert_eq!(node.description.as_deref(), Some("+12%"));
    assert!(node.actions.is_empty());
    let small = snap.get(&sid("stat_page.group[0].card[0]")).unwrap();
    assert_eq!(small.description, None, "no delta, no description");
    for group in &page.groups {
        assert!(group.cards.iter().all(|c| c.stat.sem_id().is_some()));
    }
}

/// A request on a readout is refused, and the focus a reader brings is
/// the page's only under its root.
#[test]
fn a_readout_runs_nothing_and_takes_a_reader_s_focus() {
    let mut page = StatPage::new();
    let req = Request {
        actor: accord::Actor::Test {
            name: "stat".into(),
        },
        target: sid("stat_page.group[0].card[1]"),
        action: accord::ActionName::parse("activate").expect("an action name"),
        param: None,
        observed: accord::Observation {
            revision: seen(&page).revision(),
            values: Vec::new(),
        },
        ask: None,
    };
    assert!(page.run(&req).is_err());
    assert!(page.focus(&sid("stat_page.group[2].card[0]")));
    assert!(!page.focus(&sid("toggle_page.group[0]")));
}

/// The translated readout's delta wraps under its value in a narrow
/// card and sits beside it in a wide one: the room decides.
#[test]
fn the_delta_wraps_only_when_its_card_is_narrow() {
    let theme = Theme::hoff_core();
    let page = StatPage::new();
    let stat = &page.groups[2].cards[1].stat;
    let wraps_at = |w: f32| {
        let layout = page.layout(Rect::new(0.0, 0.0, w, 800.0), &theme);
        let r = StatPage::widget_rect(stat, layout[2].cards[1], &theme);
        stat.delta_wraps(r.w, &theme)
    };
    assert!(wraps_at(280.0));
    assert!(!wraps_at(1600.0));
}

/// The width sweep of the epic, 280 to 1600 every 8 px, under the core
/// and the hoff themes: every card inside the content and above its
/// rule, cards apart, every readout inside its card.
#[test]
fn nothing_cuts_or_overlaps_from_280_to_1600() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let page = StatPage::new();
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
                    let s = StatPage::widget_rect(&card.stat, *r, &theme);
                    assert!(
                        s.x >= r.x && s.right() <= r.right() + 0.5,
                        "{w}: {}",
                        card.title
                    );
                    assert!(
                        s.y >= r.y && s.bottom() <= r.bottom(),
                        "{w}: {}",
                        card.title
                    );
                }
            }
        }
    }
}
