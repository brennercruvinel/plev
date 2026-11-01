//! The kit's headers: the tokens as its classes write them, the row's
//! layout across widths, the projection and what a request does.

use accord::{ActionName, Actor, Observation, Request, Role, SemId, SnapshotBuilder, Value};

use super::*;
use crate::drawing::compositor::Compositor;
use crate::styling::theme::Theme;
use crate::widgets::widget::{Rect, WidgetEvent};

fn id(s: &str) -> SemId {
    SemId::parse(s).expect("an id")
}

fn bar(w: f32, t: &Theme) -> Rect {
    Rect::new(0.0, 0.0, w, t.header.height)
}

/// `h-20 px-5`, `gap-4` / `gap-3` / `gap-2`, the arrows `size-10
/// rounded-[0.625rem]` with a `size-5` icon, the asset's actions `size-4`,
/// the dot `size-1` at `top-1 right-1` in `#fe5938`, the search `w-65`,
/// the thumbnails `size-12.5` and `size-15`, the logo `size-8` + `gap-3`
/// in `p-1`.
#[test]
fn the_tokens_are_the_kit_s() {
    let h = HeaderTokens::hoff_core();
    assert_eq!((h.height, h.pad_x), (80.0, 20.0));
    assert_eq!((h.gap, h.gap_tight, h.pair_gap), (16.0, 12.0, 8.0));
    assert_eq!((h.nav.size, h.nav.radius, h.nav.icon), (40.0, 10.0, 20.0));
    assert_eq!(
        (h.plain.size, h.plain.radius, h.plain.icon),
        (40.0, 12.0, 20.0)
    );
    assert_eq!((h.framed.radius, h.framed.icon), (12.0, 16.0));
    assert_eq!(h.nav.recipe.disabled_alpha, 0.40);
    assert_eq!((h.dot.size, h.dot.inset), (4.0, 4.0));
    // the kit's #FE5938, deepened the least it must to read as a mark on
    // the bar and on the plain action's hover (1.4.11)
    let red = crate::styling::color::Color::hex(0xFE5938);
    assert!(h.dot.color.0[0] <= red.0[0] && h.dot.color.0[1] <= red.0[1]);
    assert!((h.dot.color.to_oklch().h - red.to_oklch().h).abs() < 2.0);
    assert_eq!(h.search_w, 260.0);
    assert_eq!((h.avatar.size, h.avatar.pad), (40.0, 4.0));
    let t = &h.thumbs;
    assert_eq!(
        (t.size, t.current, t.radius, t.gap, t.fade),
        (50.0, 60.0, 12.0, 12.0, 60.0)
    );
    assert_eq!(t.pictures.len(), 5);
    let l = &h.logo;
    assert_eq!((l.symbol_size, l.gap, l.pad), (32.0, 12.0, 4.0));
}

/// Every theme keeps the kit's shapes and takes its own colors.
#[test]
fn every_theme_derives_the_shapes_and_its_own_colors() {
    let kit = HeaderTokens::hoff_core();
    for t in [Theme::hoff(), Theme::dark(), Theme::light()] {
        let h = &t.header;
        assert_eq!(
            (h.height, h.gap, h.search_w),
            (kit.height, kit.gap, kit.search_w)
        );
        assert_eq!(h.nav.size, kit.nav.size);
        assert_eq!(h.fill, t.colors.bg);
        assert_eq!(h.dot.color, t.colors.danger);
        assert_eq!(h.logo.ink, t.colors.text);
        assert_eq!(h.plain.recipe, t.button.toolbar);
    }
    assert_eq!(*Theme::hoff_core().header, kit);
}

/// The symbol alone is the link's square; the full logo adds the gap and
/// the wordmark.
#[test]
fn the_logo_is_its_symbol_its_wordmark_or_both() {
    let t = Theme::hoff_core();
    let l = &t.header.logo;
    let symbol = Logo::new("Plev").kind(LogoKind::Symbol).preferred_size(&t);
    assert_eq!(
        symbol,
        (l.symbol_size + l.pad * 2.0, l.symbol_size + l.pad * 2.0)
    );
    let word = Logo::new("Plev")
        .kind(LogoKind::Wordmark)
        .preferred_size(&t);
    let full = Logo::new("Plev").preferred_size(&t);
    assert_eq!(full.0, word.0 + l.symbol_size + l.gap);
    assert!(word.0 > l.symbol_size, "the wordmark is measured");
}

/// Wide, the simple header keeps every part on the bar's midline, the
/// free width after the search, the profile at the end; narrow, back and
/// forward leave first and the search gives way last.
#[test]
fn the_simple_row_pushes_its_end_and_drops_the_arrows_when_narrow() {
    let t = Theme::hoff_core();
    let h = Header::simple();
    let wide = bar(1100.0, &t);
    let l = h.layout(wide, &t);
    let rects: Vec<Rect> = l.slots.iter().map(|r| r.expect("all shown")).collect();
    let mid = wide.h / 2.0;
    for r in &rects {
        assert!((r.y + r.h / 2.0 - mid).abs() < 0.01);
    }
    assert_eq!(rects[0].x, t.header.pad_x);
    assert_eq!(rects[1].x, rects[0].right() + t.header.pair_gap);
    assert_eq!(rects[2].w, t.header.search_w);
    let last = rects[rects.len() - 1];
    assert!((last.right() - (wide.right() - t.header.pad_x)).abs() < 0.01);
    assert!(
        rects[3].x - rects[2].right() > t.header.gap,
        "the free width"
    );
    let narrow = h.layout(bar(480.0, &t), &t);
    assert_eq!((narrow.slots[0], narrow.slots[1]), (None, None));
    assert!(narrow.slots[2].is_some());
    let tiny = h.layout(bar(320.0, &t), &t);
    assert!(tiny.slots[2].expect("the search stays").w < t.header.search_w);
}

/// The thumbnails sit at the bar's middle while they clear both sides,
/// the current one the larger, and leave when they do not.
#[test]
fn the_asset_strip_floats_in_the_middle_while_there_is_room() {
    let t = Theme::hoff_core();
    let h = Header::asset(5, 2);
    let wide = bar(1100.0, &t);
    let r = h.layout(wide, &t).thumbs.expect("room");
    assert!((r.x + r.w / 2.0 - wide.w / 2.0).abs() < 0.01);
    let thumbs = h.thumbs.as_ref().expect("thumbs");
    let rects = thumbs.rects(r, &t);
    assert_eq!(rects[2].w, t.header.thumbs.current);
    assert_eq!(rects[0].w, t.header.thumbs.size);
    assert!(h.layout(bar(600.0, &t), &t).thumbs.is_none());
}

fn seen(h: &Header) -> accord::Snapshot {
    let root = id("page");
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Page");
    h.project(&mut b, "Asset", &root);
    b.finish().expect("a valid tree")
}

fn ask(h: &mut Header, target: &str, action: &str, param: Option<Value>) -> Result<(), String> {
    let snap = seen(h);
    h.run(&Request {
        actor: Actor::Test {
            name: "header".into(),
        },
        target: id(target),
        action: ActionName::parse(action).expect("an action"),
        param,
        observed: Observation {
            revision: snap.revision(),
            values: Vec::new(),
        },
        ask: None,
    })
}

/// A toolbar of its parts; the thumbnails a group of radios, the current
/// one checked; `select` moves it as a click does; a disabled arrow
/// refuses `activate`.
#[test]
fn the_header_projects_its_parts_and_runs_what_a_click_does() {
    let t = Theme::hoff_core();
    let mut h = Header::asset(5, 2).sem(id("page.asset"));
    let snap = seen(&h);
    assert_eq!(
        snap.get(&id("page.asset")).map(|n| n.role),
        Some(Role::Toolbar)
    );
    for key in ["close", "download", "share", "add"] {
        let node = snap.get(&id(&format!("page.asset.{key}"))).expect(key);
        assert_eq!(node.role, Role::Button, "{key}");
    }
    let checked = |s: &accord::Snapshot, i: usize| {
        s.get(&id(&format!("page.asset.assets[{i}]")))
            .and_then(|n| n.state.get("checked").cloned())
    };
    assert_eq!(checked(&snap, 2), Some(Value::Bool(true)));
    ask(&mut h, "page.asset.assets[4]", "select", None).expect("selects");
    assert_eq!(checked(&seen(&h), 4), Some(Value::Bool(true)));
    // a click on the first thumbnail does the same
    let r = h.layout(bar(1100.0, &t), &t).thumbs.expect("room");
    let first = h.thumbs.as_ref().expect("thumbs").rects(r, &t)[0];
    let (x, y) = (first.x + 1.0, first.y + 1.0);
    let got = h.handle_event(&WidgetEvent::MouseDown { x, y }, bar(1100.0, &t), &t);
    assert!(got.clicked);
    assert_eq!(checked(&seen(&h), 0), Some(Value::Bool(true)));

    let mut s = Header::simple().sem(id("page.asset"));
    assert!(ask(&mut s, "page.asset.back", "activate", None).is_err());
    assert!(ask(&mut s, "page.asset.forward", "activate", None).is_ok());
    let text = Some(Value::Text("chairs".into()));
    ask(&mut s, "page.asset.search", "set_value", text).expect("typed");
    assert_eq!(s.search().map(|g| g.input.text()), Some("chairs"));
}

/// A reader's focus rings the part it names, and goes; the search's
/// caret is its focus.
#[test]
fn the_focus_goes_to_the_part_it_names() {
    let t = Theme::hoff_core();
    let mut h = Header::simple().sem(id("page.simple"));
    let alerts = h.part(&id("page.simple.alerts")).expect("a part");
    h.set_focus(Some(alerts));
    assert_eq!(h.focused_id(), Some(id("page.simple.alerts")));
    let search = h.part(&id("page.simple.search")).expect("the search");
    h.set_focus(Some(search));
    assert!(h.search().is_some_and(|g| g.input.is_focused()));
    let action = h.part(&id("page.simple.search.action"));
    assert_eq!(action, Some(HeaderPart::SearchAction));
    h.set_focus(None);
    assert_eq!(h.focused_id(), None);
    let mut c = Compositor::new();
    h.render(&mut c, bar(1100.0, &t), &t);
}

/// The headers' search is the form's `Look::Header`: the search box with
/// `!bg-[#F8F7F7] !border-[#E2E2E2]`, the same at rest, hovered and
/// focused, the focus well kept; every other theme pins its panel tone
/// and its active border.
#[test]
fn the_header_search_is_the_pinned_search_box() {
    use crate::styling::color::Color;
    use crate::widgets::form::tokens::Look;
    let t = Theme::hoff_core();
    let (search, header) = (t.form.recipe(Look::Search), t.form.recipe(Look::Header));
    for s in [&header.rest, &header.hover, &header.focus] {
        assert_eq!(
            (s.fill, s.border),
            (Color::hex(0xF8F7F7), Color::hex(0xE2E2E2))
        );
    }
    assert_eq!(header.focus.shadow, search.focus.shadow);
    assert!(!header.focus.shadow.is_empty(), "the focus well");
    assert_eq!((header.height, header.pad_x), (search.height, search.pad_x));
    for theme in [Theme::hoff(), Theme::dark(), Theme::light()] {
        let h = theme.form.recipe(Look::Header);
        assert_eq!(h.rest.fill, theme.colors.bg_panel);
        assert_eq!(h.focus.border, theme.colors.border_active);
    }
}

/// Another product's mark: its name and its symbol, on one logo or on a
/// header's, which keeps its kind; the theme's symbol otherwise.
#[test]
fn a_logo_takes_another_name_and_symbol() {
    let t = Theme::hoff_core();
    let logo = Logo::new("Hoff").symbol("box");
    assert_eq!((logo.name.as_str(), logo.symbol), ("Hoff", Some("box")));
    assert_eq!(Logo::new("Hoff").symbol, None);
    let mut c = Compositor::new();
    logo.render(&mut c, Rect::new(0.0, 0.0, 200.0, 40.0), &t);
    let h = Header::minimal("Plev")
        .sem(SemId::parse("page.minimal").expect("an id"))
        .with_logo(logo);
    let swapped = h.slots.iter().find_map(|s| match &s.item {
        HeaderItem::Logo(l) => Some((l.name.clone(), l.symbol, l.kind, l.sem_id().cloned())),
        _ => None,
    });
    let kept = Some(id("page.minimal.logo"));
    assert_eq!(
        swapped,
        Some(("Hoff".into(), Some("box"), LogoKind::Symbol, kept))
    );
}
