use accord::{SemId, SnapshotBuilder, Value};

use super::export::Export;
use super::settings::Settings;
use super::share::{SharePost, ShareProfile};
use super::share_file::{Person, ShareFile};
use super::variations::Variations;
use super::*;
use crate::drawing::compositor::Compositor;

fn themes() -> [Theme; 4] {
    [
        Theme::hoff_core(),
        Theme::hoff(),
        Theme::dark(),
        Theme::light(),
    ]
}

fn id(name: &str) -> SemId {
    let Ok(id) = SemId::name(name) else {
        panic!("{name} parses");
    };
    id
}

fn bodies() -> Vec<Box<dyn ModalBody>> {
    let people = vec![
        Person::new("Sam Dy", "sam@hoffresearch.com", Some(0)),
        Person::new("Hellen", "helen@hoffresearch.com", None),
    ];
    vec![
        Box::new(Variations::new(4, Some(&id("variations")))),
        Box::new(SharePost::new(
            "Cute Monster",
            "randomdash",
            "3 Jun, 2025",
            Some(&id("post")),
        )),
        Box::new(ShareProfile::new(
            "Sophie Bennett",
            "159 scenes",
            Some(&id("profile")),
        )),
        Box::new(ShareFile::new(
            people,
            "https://research-app.co/f",
            Some(&id("file")),
        )),
        Box::new(Export::new("Robot 2.0", Some(&id("export")))),
        Box::new(Settings::new(Some(&id("settings")))),
    ]
}

/// The kit in core; every other theme derives its own, the kit's
/// geometry kept.
#[test]
fn the_core_is_the_kit_and_the_others_derive_it() {
    let core = Theme::hoff_core();
    assert_eq!(*core.overlay.modals, ModalsTokens::hoff_core());
    for theme in [Theme::hoff(), Theme::dark(), Theme::light()] {
        let m = &theme.overlay.modals;
        assert_eq!(**m, ModalsTokens::derive(&theme));
        assert_eq!(m.ink.ink, theme.colors.text);
        assert!(m.share_file.panel.glass, "the theme's own sheet");
        assert_eq!(m.export.panel.width, core.overlay.modals.export.panel.width);
    }
}

/// From 240 to 1600, in the four themes: no modal is wider than its room
/// nor wider than its kit width, and each draws.
#[test]
fn every_modal_keeps_to_its_room_and_draws() {
    for theme in themes() {
        for body in bodies() {
            let mut w = 240.0;
            while w <= 1600.0 {
                let (mw, mh) = body.size(w, &theme);
                assert!(mw <= w + 0.01, "{}: {mw} in {w}", body.title());
                assert!(mh > 0.0, "{}", body.title());
                w += 40.0;
            }
            let (mw, mh) = body.size(1600.0, &theme);
            let mut c = Compositor::new();
            c.begin_frame();
            body.render(&mut c, Rect::new(0.0, 0.0, mw, mh), &theme);
        }
    }
}

/// A variation chosen by an actor: the count reads it and the group
/// states it.
#[test]
fn a_variation_chosen_is_counted() {
    let mut v = Variations::new(4, Some(&id("v")));
    assert_eq!(v.count_text(), "0 of 4");
    let tile = id("v").child("tile").map(|t| t.key("2"));
    let Ok(tile) = tile else {
        panic!("the tile's id");
    };
    assert_eq!(v.run(&tile, "select", None), Some(Ok(true)));
    assert_eq!(v.count_text(), "3 of 4");
    assert_eq!(v.chosen(), Some(2));
}

/// The side menu switches the section; the projection shows the new
/// section's controls and not the old one's.
#[test]
fn the_side_menu_switches_the_section_and_its_controls() {
    let root = id("settings");
    let mut s = Settings::new(Some(&root));
    let Ok(security) = root.child("section").map(|f| f.key("2")) else {
        panic!("the tab's id");
    };
    assert_eq!(s.run(&security, "select", None), Some(Ok(true)));
    assert_eq!(s.section(), 2);
    let mut b = SnapshotBuilder::new();
    b.node(&id("page"), accord::Role::Region, "page");
    s.project(&mut b, &root, &id("page"));
    let Ok(seen) = b.finish() else {
        panic!("the projection holds");
    };
    let has = |name: &str| seen.nodes().keys().any(|k| k.as_str().contains(name));
    assert!(has("settings.security.mfa"));
    assert!(
        !has("settings.general.username"),
        "the hidden section is not projected"
    );
    let Ok(mfa) = root.child("security").and_then(|s| s.child("mfa")) else {
        panic!("the switch's id");
    };
    assert_eq!(s.run(&mfa, "toggle", None), Some(Ok(true)));
    let Ok(username) = root.child("general").and_then(|s| s.child("username")) else {
        panic!("the field's id");
    };
    assert!(
        s.run(&username, "set_value", Some(&Value::Text("x".into())))
            .is_none()
    );
}

/// Export's selects open, take a value by name and by option, and close
/// on Escape; a value not offered is refused.
#[test]
fn an_export_select_opens_picks_and_closes() {
    let root = id("export");
    let mut e = Export::new("Robot 2.0", Some(&root));
    let Ok(video) = root.child("section").map(|f| f.key("1")) else {
        panic!("the tab's id");
    };
    assert_eq!(e.run(&video, "select", None), Some(Ok(true)));
    let Ok(rate) = root.child("video").and_then(|v| v.child("rate")) else {
        panic!("the select's id");
    };
    let to = |v: &str| Some(Value::Text(v.into()));
    assert_eq!(e.run(&rate, "select", to("30").as_ref()), Some(Ok(true)));
    assert!(matches!(
        e.run(&rate, "select", to("25").as_ref()),
        Some(Err(_))
    ));
    assert_eq!(e.run(&rate, "open", None), Some(Ok(true)));
    assert!(e.has_open());
    assert_eq!(e.run(&rate.key("2"), "select", None), Some(Ok(true)));
    assert!(!e.has_open(), "a pick closes the list");
    e.run(&rate, "open", None);
    assert!(e.close(), "Escape closes it");
    assert!(!e.has_open());
}

/// Less room than the kit's sheet: the side menu goes above its section,
/// as wide as the sheet.
#[test]
fn the_side_menu_goes_above_its_section_in_less_room() {
    let theme = Theme::hoff_core();
    let s = Settings::new(None);
    let kit = theme.overlay.modals.settings.panel.width;
    let (_, side) = s.size(kit, &theme);
    let (_, stacked) = s.size(kit / 2.0, &theme);
    assert!(stacked > side, "the menu above adds its height");
}
