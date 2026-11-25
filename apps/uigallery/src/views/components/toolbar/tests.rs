//! Toolbar page tests: the projection a reader hears, the runs that
//! select a tool, open the zoom panel and flip play, the pointer doing
//! the same, every card's widget inside its card, and the contrast the
//! recipe keeps.

use accord::{Request, Role, SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::Compositor;
use engine::styling::color::Color;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{Rect, WidgetEvent};

use super::{Component, EditKey, Shows, TextHost, ToolbarPage};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("the id parses")
}

fn seen(page: &ToolbarPage) -> accord::Snapshot {
    let root = page.root.clone().expect("the root parses");
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Toolbar");
    page.project(&mut b, &root);
    b.finish().expect("the page projects a valid tree")
}

fn activate(page: &mut ToolbarPage, target: &str) {
    let req = Request {
        actor: accord::Actor::Test {
            name: "toolbar".into(),
        },
        target: sid(target),
        action: accord::ActionName::parse("activate").expect("an action name"),
        param: None,
        observed: accord::Observation {
            revision: seen(page).revision(),
            values: Vec::new(),
        },
        ask: None,
    };
    page.run(&req).expect("the page runs it");
}

/// The edit bar is a toolbar whose tools name their key, the select tool
/// on, Undo off; the zoom control is collapsed until it opens, and its
/// menu holds the commands and the toggles, checked as the kit has them.
#[test]
fn the_bar_is_heard_with_its_tools_and_its_zoom_menu() {
    let mut page = ToolbarPage::new();
    let snap = seen(&page);
    let bar = snap.get(&sid("toolbar_page.bar[0]")).expect("the bar");
    assert_eq!(bar.role, Role::Toolbar);
    let select = snap.get(&sid("toolbar_page.bar[0].tool[0]")).unwrap();
    assert_eq!(select.label, "Select");
    assert_eq!(select.description.as_deref(), Some("V"));
    assert_eq!(select.state.get("active"), Some(&Value::Bool(true)));
    let undo = snap.get(&sid("toolbar_page.bar[0].history[0]")).unwrap();
    assert!(!undo.enabled, "undo is disabled");
    let zoom = snap.get(&sid("toolbar_page.bar[0].zoom")).unwrap();
    assert_eq!(zoom.state.get("expanded"), Some(&Value::Bool(false)));
    assert!(snap.get(&sid("toolbar_page.bar[0].zoom.menu")).is_none());

    activate(&mut page, "toolbar_page.bar[0].zoom");
    let snap = seen(&page);
    let menu = snap
        .get(&sid("toolbar_page.bar[0].zoom.menu"))
        .expect("open");
    assert_eq!(menu.role, Role::Menu);
    let checked = |snap: &accord::Snapshot, i: usize| {
        let row = snap
            .get(&sid(&format!("toolbar_page.bar[0].zoom.menu.line[{i}]")))
            .unwrap();
        row.state.get("checked") == Some(&Value::Bool(true))
    };
    assert!(checked(&snap, 2), "zoom to 100% is the last run");
    assert!(checked(&snap, 5) && checked(&snap, 6), "both views on");
    activate(&mut page, "toolbar_page.bar[0].zoom.menu.line[4]");
    activate(&mut page, "toolbar_page.bar[0].zoom.menu.line[5]");
    let snap = seen(&page);
    assert!(checked(&snap, 4) && !checked(&snap, 2));
    assert!(!checked(&snap, 5), "comments off");
}

/// A run selects a tool on the edit bar, not on the play bar (its tools
/// are commands), and flips the player's play button.
#[test]
fn runs_select_a_tool_and_flip_play() {
    let mut page = ToolbarPage::new();
    activate(&mut page, "toolbar_page.bar[0].tool[3]");
    assert_eq!(page.bars[0].active, Some(3));
    activate(&mut page, "toolbar_page.bar[1].tool[1]");
    assert_eq!(page.bars[1].active, None);
    assert!(!page.players[0].playing);
    activate(&mut page, "toolbar_page.player[0].play");
    assert!(page.players[0].playing);
    let snap = seen(&page);
    let play = snap.get(&sid("toolbar_page.player[0].play")).unwrap();
    assert_eq!(play.label, "Pause");
    assert!(snap.get(&sid("toolbar_page.player[1].repeat")).is_some());
    assert!(snap.get(&sid("toolbar_page.view[1].forward")).is_some());
}

/// The pointer does what the runs do: a click on a tool selects it, on
/// the zoom control opens the panel, a click away closes it.
#[test]
fn the_pointer_selects_and_opens() {
    let theme = Theme::hoff_core();
    let content = Rect::new(0.0, 0.0, 1200.0, 800.0);
    let mut page = ToolbarPage::new();
    let (_, _, bar) = page.placed(content, &theme)[0];
    let at = page.bars[0].layout(bar, &theme);
    let click = |page: &mut ToolbarPage, (x, y): (f32, f32)| {
        for e in [
            WidgetEvent::MouseMove { x, y },
            WidgetEvent::MouseDown { x, y },
            WidgetEvent::MouseUp { x, y },
        ] {
            page.handle_event(&e, content, &theme);
        }
    };
    click(&mut page, at.tools[1].center());
    assert_eq!(page.bars[0].active, Some(1));
    click(&mut page, at.zoom.center());
    assert!(page.bars[0].open);
    click(&mut page, (content.x + 1.0, content.y + 1.0));
    assert!(!page.bars[0].open);
}

/// Every widget sits inside its card at the desktop width, in both
/// faces of the kit, and the page draws.
#[test]
fn every_widget_fits_its_card() {
    let content = Rect::new(0.0, 0.0, 1200.0, 800.0);
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let page = ToolbarPage::new();
        for (shows, card, w) in page.placed(content, &theme) {
            assert!(
                w.x >= card.x && w.x + w.w <= card.x + card.w && w.y >= card.y,
                "{shows:?}: {w:?} outside {card:?}"
            );
        }
        let mut c = Compositor::new();
        c.begin_frame();
        page.render(&mut c, content, &theme);
        assert!(page.content_height(content, &theme) > 0.0);
        assert!(matches!(page.placed(content, &theme)[0].0, Shows::Bar(0)));
    }
}

fn lum(c: Color) -> f32 {
    let [r, g, b, _] = c.to_linear_array();
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn ratio(a: Color, b: Color) -> f32 {
    let (x, y) = (lum(a), lum(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// The kit's text reads at 7:1 on what it sits on, its marks at 3:1: the
/// player's time and the zoom's labels, the track, the minor ticks.
#[test]
fn the_core_recipe_keeps_the_bars() {
    let t = Theme::hoff_core();
    let tool = &t.navigation.tool;
    let p = &tool.player;
    assert!(ratio(p.solid.time, p.fill) >= 7.0);
    assert!(ratio(tool.zoom.label_color, tool.zoom.fill) >= 7.0);
    assert!(ratio(tool.toolbar.zoom_text_color, tool.toolbar.zoom_fill) >= 7.0);
    assert!(ratio(p.solid.track, p.fill) >= 3.0);
    assert!(ratio(tool.view.tick, tool.view.fill) >= 3.0);
    assert!(ratio(p.on_image.time, p.image[0]) >= 7.0);
}

/// At a phone's width every bar wraps inside its card: no part passes
/// the card's end, and the bar is as wide as the room it was given.
#[test]
fn at_360_every_bar_wraps_inside_its_card() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let page = ToolbarPage::new();
        let content = Rect::new(0.0, 0.0, 330.0, 800.0);
        for (shows, card, rect) in page.placed(content, &theme) {
            let Shows::Bar(b) = shows else { continue };
            assert!(
                rect.right() <= card.right() + 0.01,
                "{rect:?} past {card:?}"
            );
            let l = page.bars[b].layout(rect, &theme);
            let parts = l.tools.iter().chain(&l.history).chain([&l.zoom, &l.export]);
            for r in parts {
                assert!(
                    r.right() <= rect.right() + 0.01 && r.bottom() <= rect.bottom() + 0.01,
                    "{r:?} {rect:?}"
                );
            }
        }
    }
}

/// A view controller draws at its own width (the kit's pill is as wide
/// as its arrows and ruler): a card never gets narrower than it, the row
/// drops a column first, so at every width the pill stays inside its
/// card's insets, even a pill wider than the theme's least card.
#[test]
fn a_view_controller_never_passes_its_card() {
    let mut wide = Theme::hoff_core();
    wide.navigation.tool.view.pad = wide.size.card_min_w;
    for theme in [Theme::hoff_core(), wide] {
        let page = ToolbarPage::new();
        let inset = super::showcase::inset(&theme);
        for w in (280..=1600).step_by(8) {
            let content = Rect::new(0.0, 0.0, w as f32, 800.0);
            for (shows, card, r) in page.placed(content, &theme) {
                let Shows::View(_) = shows else { continue };
                if r.w + inset * 2.0 > content.w {
                    continue;
                }
                assert!(
                    r.x >= card.x + inset - 0.01 && r.right() <= card.right() - inset + 0.01,
                    "{w}: {r:?} past {card:?}"
                );
            }
        }
    }
}

/// A hovered tool's tooltip waits out the Tooltip's delay; the
/// keyboard's focus shows it at once and rings the tool.
#[test]
fn the_tooltip_waits_for_the_pointer_and_not_for_the_focus() {
    let theme = Theme::hoff_core();
    let mut page = ToolbarPage::new();
    let content = Rect::new(0.0, 0.0, 1200.0, 800.0);
    let (_, _, rect) = page.placed(content, &theme)[0];
    let tool = page.bars[0].layout(rect, &theme).tools[1];
    let (x, y) = (tool.x + tool.w / 2.0, tool.y + tool.h / 2.0);
    page.handle_event(&WidgetEvent::MouseMove { x, y }, content, &theme);
    let shown = |p: &ToolbarPage| p.bars[0].tooltip_shown();
    assert!(!shown(&page), "no tooltip before the delay");
    for _ in 0..60 {
        page.tick(1.0 / 60.0);
    }
    assert!(shown(&page), "the tooltip after the delay");
    page.handle_event(
        &WidgetEvent::MouseMove { x: -9.0, y: -9.0 },
        content,
        &theme,
    );
    assert!(!shown(&page));
    assert!(page.focus(&sid("toolbar_page.bar[0].tool[2]")));
    assert!(shown(&page), "the focus shows it at once");
    assert!(page.blur());
    assert!(!shown(&page));
}

/// A press on the open panel's level focuses its field; typing there is
/// the level, Escape leaves it.
#[test]
fn the_zoom_level_takes_typing() {
    let theme = Theme::hoff_core();
    let mut page = ToolbarPage::new();
    let content = Rect::new(0.0, 0.0, 1200.0, 800.0);
    let (_, _, rect) = page.placed(content, &theme)[0];
    page.bars[0].open = true;
    let panel = page.bars[0].panel_rect(rect, &theme);
    let at = page.bars[0].zoom.text_rect(panel, &theme);
    let (x, y) = (at.right() - 1.0, at.y + at.h / 2.0);
    page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
    page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme);
    assert!(page.bars[0].zoom.field.is_focused());
    assert!(page.handle_edit_key(EditKey::Backspace));
    assert!(page.handle_text("5%"));
    assert_eq!(page.bars[0].zoom.level, "1005%");
    assert!(page.handle_escape());
    assert!(!page.bars[0].zoom.field.is_focused());
}
