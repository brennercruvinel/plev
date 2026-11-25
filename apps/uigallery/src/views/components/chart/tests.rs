//! Chart page tests: what a reader hears of each chart, the reveal a
//! click replays (and reduced motion settles at once), a mid-reveal
//! scene, and the width sweep with the donut's legend inside its plot.
//! No GPU anywhere.

use accord::{Role, SemId, SnapshotBuilder};
use engine::compose::animation::set_reduced_motion;
use engine::drawing::compositor::{Compositor, LayerId, SceneNode};
use engine::styling::theme::Theme;
use engine::widgets::charts::donut;
use engine::widgets::prelude::{Rect, WidgetEvent};

use super::{ChartPage, Component, Plot};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("the id parses")
}

fn seen(page: &ChartPage) -> accord::Snapshot {
    let root = page.root.clone().expect("the root parses");
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Chart");
    page.project(&mut b, &root);
    b.finish().expect("the page projects a valid tree")
}

fn content() -> Rect {
    Rect::new(288.0, 118.0, 952.0, 682.0)
}

fn first_card(page: &ChartPage, theme: &Theme) -> Rect {
    page.layout(content(), theme)[0].cards[0]
}

/// Every chart is an image named by its title, described by what it
/// shows, over a table with a row per datum; nothing to run.
#[test]
fn every_chart_is_an_image_over_a_table_of_its_data() {
    let page = ChartPage::new();
    let snap = seen(&page);
    let line = snap.get(&sid("chart_page.group[0].card[0]")).unwrap();
    assert_eq!(
        (line.role, line.label.as_str()),
        (Role::Image, "Sessions per week")
    );
    assert_eq!(
        line.description.as_deref(),
        Some("line chart, 12 values from 0.30 to 0.95")
    );
    let table = snap.get(&sid("chart_page.group[0].card[0].data")).unwrap();
    assert_eq!(table.role, Role::Table);
    let rows: Vec<_> = snap.children(&table.id).collect();
    assert_eq!(rows.len(), 12);
    assert_eq!(rows[9].label, "10: 0.95");
    let donut = snap
        .get(&sid("chart_page.group[0].card[3].data[0]"))
        .unwrap();
    assert_eq!(donut.label, "Search: 35%");
    assert!(snap.nodes().values().all(|n| n.actions.is_empty()));
}

/// A click on a card replays the reveal and it settles; a click off
/// every card is ignored.
#[test]
fn a_click_on_a_chart_replays_the_reveal_and_it_settles() {
    let theme = Theme::hoff_core();
    let mut page = ChartPage::new();
    assert!(!page.tick(0.016), "construction must not animate");
    let c = content();
    let outside = WidgetEvent::MouseDown {
        x: c.x - 50.0,
        y: c.y - 50.0,
    };
    let r = page.handle_event(&outside, c, &theme);
    assert!(!r.changed && !r.handled && !page.tick(0.016));

    let (x, y) = first_card(&page, &theme).center();
    let r = page.handle_event(&WidgetEvent::MouseDown { x, y }, c, &theme);
    assert!(r.changed, "replay must request a redraw");
    assert!(page.tick(0.016), "reveal must keep frames flowing");
    let mut frames = 0;
    while page.tick(1.0 / 60.0) {
        frames += 1;
        assert!(frames < 200, "reveal must settle (no busy loop)");
    }
    assert_eq!(page.reveal.get(), 1.0, "reveal completes fully drawn");
}

/// With reduced motion the replay lands drawn on its first frame.
#[test]
fn reduced_motion_settles_the_replay_at_once() {
    let theme = Theme::hoff_core();
    let mut page = ChartPage::new();
    let (x, y) = first_card(&page, &theme).center();
    set_reduced_motion(true);
    page.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
    let moving = page.tick(0.016);
    let at = page.reveal.get();
    set_reduced_motion(false);
    assert!(!moving, "no frames to ask for");
    assert_eq!(at, 1.0);
}

/// Mid-reveal the scene holds paths, labels and the line's reveal clip,
/// under both themes.
#[test]
fn mid_reveal_render_emits_paths_labels_and_a_reveal_clip() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let mut page = ChartPage::new();
        let (x, y) = first_card(&page, &theme).center();
        page.handle_event(&WidgetEvent::MouseDown { x, y }, content(), &theme);
        page.tick(0.4);
        let mut c = Compositor::new();
        page.render(&mut c, content(), &theme);
        let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes();
        let count = |f: fn(&SceneNode) -> bool| nodes.iter().filter(|n| f(n)).count();
        let paths = count(|n| matches!(n, SceneNode::Path { .. }));
        let texts = count(|n| matches!(n, SceneNode::Text { .. }));
        let clips = count(|n| matches!(n, SceneNode::PushClip { .. }));
        assert!(paths >= 10, "line, dots, bands and slices: got {paths}");
        assert!(texts >= 10, "titles, ticks, values and legend: got {texts}");
        assert_eq!(clips, 1, "the line chart reveal clips its plot");
    }
}

/// The width sweep of the epic, 280 to 1600 every 8 px, under the core
/// and the hoff themes: every card inside the content and above its
/// rule, cards apart, every plot inside its card and below its chip, and
/// every legend label of a donut inside its plot.
#[test]
fn nothing_cuts_or_overlaps_from_280_to_1600() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let k = &theme.navigation.chart;
        let ty = &theme.typography;
        let page = ChartPage::new();
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
                    let p = ChartPage::plot_rect(&card.plot, *r, &theme);
                    let chip = theme.spacing.sm * 2.0 + theme.spacing.xs * 2.0;
                    assert!(
                        p.x >= r.x && p.right() <= r.right() + 0.5,
                        "{w}: {}",
                        card.title
                    );
                    assert!(p.y >= r.y + chip && p.bottom() <= r.bottom(), "{w}");
                    if let Plot::Donut(items) = &card.plot {
                        let d = donut(items, p, "100%", k.center.style(ty), k.legend.style(ty));
                        for item in &d.legend {
                            let l = &item.label;
                            assert!(
                                l.x >= p.x - 0.5 && l.x + l.w <= p.right() + 0.5,
                                "{w}: {}",
                                l.text
                            );
                            assert!(
                                l.y >= p.y - 0.5 && l.y + l.h <= p.bottom() + 0.5,
                                "{w}: {}",
                                l.text
                            );
                        }
                    }
                }
            }
        }
    }
}
