use accord::SemId;

use super::*;
use crate::styling::theme::Theme;
use crate::widgets::content::{Collapsible, DisclosureList};
use crate::widgets::nav::NavLink;
use crate::widgets::selection::FolderRow;
use crate::widgets::widget::Rect;

fn home() -> HomeSidebar {
    HomeSidebar::new(
        vec![
            HomeRow::Menu(
                Collapsible::new("Explore")
                    .icon("layout-grid")
                    .current(true)
                    .open(true),
                DisclosureList::new(["Designs", "Animations"]).current(0),
            ),
            HomeRow::Menu(
                Collapsible::new("Assets").icon("layers").counter("112"),
                DisclosureList::new(["3D Objects", "Materials"]),
            ),
            HomeRow::Link(NavLink::new("Likes").icon("heart")),
        ],
        "My scenes",
        NavLink::new("My Scenes").icon("box"),
    )
    .folders(["Untitled Folder", "3D Icons"])
    .sem(SemId::name("home").unwrap())
}

fn guideline() -> GuidelineSidebar {
    GuidelineSidebar::new()
        .group(
            "Foundations",
            &[("Branding", "venetian-mask"), ("Colors", "swatch-book")],
        )
        .group(
            "Components",
            &[("Buttons", "mouse-pointer-2"), ("Sidebar", "panel-left")],
        )
        .current(3)
        .sem(SemId::name("guide").unwrap())
}

fn themes() -> [Theme; 2] {
    [Theme::hoff_core(), Theme::hoff()]
}

/// The rows stack down the frame in order, inside it, none over another.
#[test]
fn the_home_rows_stack_inside_the_frame() {
    for theme in themes() {
        let s = home().naming(true);
        let (w, h) = s.size(&theme);
        let frame = Rect::new(0.0, 0.0, w, h);
        let l = s.layout(frame, &theme);
        let mut rects = l.rows.clone();
        rects.extend([l.scenes, l.new, l.field.unwrap()]);
        rects.extend(l.folders.iter().copied());
        for pair in rects.windows(2) {
            assert!(pair[0].bottom() <= pair[1].y + 0.01, "{pair:?}");
        }
        assert!(rects.last().unwrap().bottom() <= frame.bottom());
        assert!(
            rects
                .iter()
                .all(|r| r.x >= frame.x && r.right() <= frame.right())
        );
    }
}

/// The kit's `min-h-219` holds in the core; the sidebar grows past it
/// only with more rows than it holds.
#[test]
fn the_core_sidebars_are_the_kits_height() {
    let t = Theme::hoff_core();
    assert_eq!(home().size(&t), (240.0, 876.0));
    assert_eq!(guideline().size(&t), (240.0, 876.0));
}

/// Opening the field moves the folders down by its height and a gap.
#[test]
fn naming_a_folder_opens_its_field_above_the_folders() {
    let t = Theme::hoff_core();
    let frame = Rect::new(0.0, 0.0, 240.0, 876.0);
    let shut = home().layout(frame, &t);
    let open = home().naming(true).layout(frame, &t);
    let r = &t.navigation.sidebars;
    let moved = open.folders[0].y - shut.folders[0].y;
    assert!((moved - (r.field_h + r.row_gap)).abs() < 0.01, "{moved}");
}

/// A reader's activate does what a press does, by id.
#[test]
fn activate_by_id_runs_the_part() {
    let mut s = home();
    let new = s.folder_id(FolderRow::New).unwrap();
    assert!(s.activate(&new) && s.naming);
    let assets = SemId::name("home").unwrap().child("row").unwrap().key("1");
    assert!(s.activate(&assets));
    let HomeRow::Menu(m, _) = &s.rows[1] else {
        panic!()
    };
    assert!(m.is_open());
    assert!(s.focus_id(Some(&new)));
    assert!(!s.focus_id(None));

    let mut g = guideline();
    assert_eq!(g.current_link(), Some(3));
    let first = SemId::name("guide").unwrap().key("0");
    assert_eq!(g.position(&first), Some(0));
    g.set_current(0);
    assert_eq!(g.current_link(), Some(0));
}

/// The guideline's links fall under their captions, in order.
#[test]
fn the_guideline_links_follow_their_captions() {
    for theme in themes() {
        let g = guideline();
        let (w, h) = g.size(&theme);
        let (captions, links) = g.layout(Rect::new(0.0, 0.0, w, h), &theme);
        assert_eq!(links.len(), 4);
        assert!(captions[0] < links[0].y && links[1].bottom() <= captions[1]);
        assert!(captions[1] < links[2].y);
    }
}
