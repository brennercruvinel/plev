//! The Menus page's tests.
use super::*;
use crate::views::text_input::host::TextHost;

/// The width sweep, 280 to 1600 every 8 px under the core and the
/// hoff themes: every card in the content and apart, every menu inside
/// its card and under its chip.
#[test]
fn every_menu_fits_its_card_from_narrow_to_wide() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let page = MenusPage::new();
        for w in (280..=1600).step_by(8) {
            let content = Rect::new(0.0, 0.0, w as f32, 800.0);
            let at = &page.layout(content, &theme)[0];
            for (n, (card, r)) in page.cards.iter().zip(&at.cards).enumerate() {
                assert!(
                    r.x >= content.x && r.right() <= content.right() + 0.5,
                    "{w}"
                );
                if let Some(next) = at.cards.get(n + 1)
                    && next.y == r.y
                {
                    assert!(next.x >= r.right(), "{w}: cards overlap");
                }
                let inset = showcase::inset(&theme);
                let chip = showcase::chip_rect(r.x + inset, r.y + inset, card.title, &theme);
                let b = MenusPage::widget_rect(&card.body, *r, &theme);
                assert!(
                    b.x >= r.x && b.right() <= r.right() + 0.5,
                    "{w}: {}",
                    card.title
                );
                assert!(b.y >= chip.bottom() && b.bottom() <= r.bottom(), "{w}");
            }
        }
    }
}

/// A press on a filter moves its check within its section and the
/// menu says it; a press on a folder picks it; the focus rings an
/// item and Escape lets it go.
#[test]
fn a_press_picks_and_a_filter_moves_its_check() {
    let theme = Theme::hoff_core();
    let mut page = MenusPage::new();
    let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
    let layout = page.layout(content, &theme);
    let press = |page: &mut MenusPage, (x, y): (f32, f32)| {
        page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme)
    };
    let Body::Menu(m) = &page.cards[5].body else {
        panic!("the filter menu")
    };
    let at = MenusPage::widget_rect(&page.cards[5].body, layout[0].cards[5], &theme);
    let rows = m.entry_rects(at.x, at.y, &theme);
    assert!(press(&mut page, rows[2].center()).clicked);
    let Body::Menu(m) = &page.cards[5].body else {
        unreachable!()
    };
    let checks: Vec<_> = m
        .entries
        .iter()
        .filter_map(|e| match e {
            MenuEntry::Item { checked, .. } => *checked,
            _ => None,
        })
        .collect();
    assert_eq!(checks, [false, false, true, true, false]);
    assert_eq!(page.cards[5].picked.as_deref(), Some("Animation"));

    let Body::Folders(f) = &page.cards[2].body else {
        panic!("the folders")
    };
    let at = MenusPage::widget_rect(&page.cards[2].body, layout[0].cards[2], &theme);
    let row = f.layout(at, &theme).folders[1];
    press(&mut page, row.center());
    assert_eq!(page.cards[2].picked.as_deref(), Some("3D Icons"));

    let id = card_id(page.root.as_ref().unwrap(), 1).unwrap().key("2");
    assert!(page.focus(&id));
    assert_eq!(page.focus, Some((1, Spot::Item(2))));
    assert!(page.handle_escape());
    assert_eq!(page.focus, None);
}

/// New Folder opens the name's box with the keys in it; typing goes
/// there, Enter adds the folder, and Escape closes an open box with
/// nothing added.
#[test]
fn the_name_box_takes_typing_enter_adds_and_escape_cancels() {
    let mut page = MenusPage::new();
    let c = page.folders_card().unwrap();
    assert!(page.pick(c, Spot::Row(FolderRow::New)));
    assert!(page.handle_text("Moodboard"));
    assert!(TextHost::handle_enter(&mut page));
    let f = page.folders().unwrap();
    assert_eq!(f.folders.last().map(String::as_str), Some("Moodboard"));
    assert!(f.naming().is_none());

    page.pick(c, Spot::Row(FolderRow::New));
    assert!(page.handle_text("draft"));
    assert!(MenusPage::handle_escape(&mut page), "Escape cancels");
    let f = page.folders().unwrap();
    assert_eq!((f.folders.len(), f.naming().is_none()), (5, true));
    assert!(!page.handle_text("x"), "no box has the keys");
}

/// `activate` on `target`, seen at the page's projection (which must be
/// a valid tree).
fn activate(page: &MenusPage, target: SemId) -> Request {
    let root = page.root.clone().unwrap();
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Menus");
    page.project(&mut b, &root);
    let snap = b.finish().expect("the page projects a valid tree");
    Request {
        actor: accord::Actor::Test {
            name: "menus".into(),
        },
        target,
        action: accord::ActionName::parse("activate").expect("an action name"),
        param: None,
        observed: accord::Observation {
            revision: snap.revision(),
            values: Vec::new(),
        },
        ask: None,
    }
}

/// The zoom options and the AI model picker are the toolbar's zoom panel
/// and the prompt's model menu: a press on a zoom row takes the check and
/// the card says it, a model's `activate` chooses it, and no card draws a
/// placeholder.
#[test]
fn the_zoom_and_the_models_are_the_real_widgets() {
    let theme = Theme::hoff_core();
    let mut page = MenusPage::new();
    let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
    let layout = page.layout(content, &theme);
    let Body::Zoom(z) = &page.cards[0].body else {
        panic!("the zoom panel")
    };
    let at = MenusPage::widget_rect(&page.cards[0].body, layout[0].cards[0], &theme);
    let (x, y) = z.line_rects(at, &theme)[4].center();
    page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
    page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme);
    let Body::Zoom(z) = &page.cards[0].body else {
        unreachable!()
    };
    assert!(z.checked(4));
    assert_eq!(page.cards[0].picked.as_deref(), Some("Zoom to fit"));

    let Body::Models(_) = &page.cards[3].body else {
        panic!("the model picker")
    };
    let root = page.root.clone().unwrap();
    let model = card_id(&root, 3).unwrap().key("2");
    assert!(page.focus(&model));
    assert_eq!(page.focus, Some((3, Spot::Line(2))));
    assert!(page.run(&activate(&page, model)).is_ok());
    assert_eq!(page.cards[3].picked.as_deref(), Some("Kling AI"));
    let line = card_id(&root, 0).unwrap().child("line").unwrap().key("5");
    assert!(page.run(&activate(&page, line)).is_ok());
    assert_eq!(page.cards[0].picked.as_deref(), Some("Comments"));
}
