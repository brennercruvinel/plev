//! The gallery's sidebar: its places, its labels, the drawer, the safe
//! area and the current page's marks.

use super::*;

#[test]
fn every_page_has_one_place_and_back() {
    for (i, page) in PAGES.iter().enumerate() {
        let (row, at) = place(i);
        assert_eq!(page_at(row, at), Some(i), "{}", page.title);
    }
    assert_eq!(page_at(Group::ALL.len(), 0), None);
}

/// A finger turns the sidebar into a drawer behind the menu button
/// at any width; a pointer that moves turns it back.
#[test]
fn a_finger_turns_the_sidebar_into_a_drawer_and_a_pointer_back() {
    let mut view = ShowcaseView::new(1280.0, 800.0);
    let mode = |v: &ShowcaseView| v.shell.layout(&v.theme).dock.map(|d| d.mode);
    assert_eq!(mode(&view), Some(DockMode::Peek));
    assert!(view.set_touch(true));
    assert_eq!(mode(&view), Some(DockMode::Drawer));
    assert!(view.shell.layout(&view.theme).menu_button.is_some());
    assert!(!view.set_touch(true), "nothing changes twice");
    assert!(view.set_touch(false));
    assert_eq!(mode(&view), Some(DockMode::Peek));
}

/// The drawer on the keyboard: Enter on the menu button opens it with
/// the focus on its close, Tab stays in the panel (the trap), and
/// Escape shuts it with the focus back on the menu button.
#[test]
fn the_open_drawer_traps_the_focus_and_gives_it_back() {
    let mut view = ShowcaseView::new(360.0, 740.0);
    let open = |v: &ShowcaseView| v.shell.dock.as_ref().is_some_and(|d| d.is_drawer_open());
    let d = view.shell.dock.as_ref().expect("docked");
    let (menu, close) = (d.menu_id().unwrap(), d.close_id().unwrap());
    assert!(view.focus_sidebar(&menu));
    assert!(view.handle_enter());
    assert!(open(&view));
    assert_eq!(view.sidebar_focus(), Some(&close));
    for _ in 0..40 {
        view.handle_edit_key(super::super::EditKey::Tab);
        let at = view.sidebar_focus().cloned().expect("the focus stays in");
        assert!(view.is_sidebar(&at) && at != menu, "{at}");
    }
    assert!(view.close_top_overlay(), "Escape");
    assert!(!open(&view));
    assert_eq!(view.sidebar_focus(), Some(&menu));
}

/// The drawer is front B's edge panel: with motion it slides in from
/// the edge (`sheet::slide`), with reduced motion it is there at once
/// and gone at once, no frame between; drawn, its surface is the
/// theme's drawer panel under the sidebar's parts.
#[test]
fn the_drawer_slides_in_and_with_reduced_motion_cuts() {
    use engine::compose::animation::set_reduced_motion;
    let dock = |v: &ShowcaseView| v.shell.layout(&v.theme).dock.expect("docked");
    for reduced in [false, true] {
        set_reduced_motion(reduced);
        let mut view = ShowcaseView::new(360.0, 740.0);
        // a frame settles the dock in the mode the window gives it
        view.sidebar_wake(1.0 / 60.0);
        let menu = view.shell.dock.as_ref().unwrap().menu_id().unwrap();
        assert!(view.focus_sidebar(&menu));
        assert!(view.handle_enter(), "Enter on the menu opens it");
        view.sidebar_wake(1.0 / 60.0);
        let d = dock(&view);
        assert_eq!(d.mode, DockMode::Drawer);
        let animating = view.shell.dock.as_ref().unwrap().is_animating();
        if reduced {
            assert_eq!(d.shown, 1.0, "a cut");
            assert_eq!(d.panel, Some(d.rest));
            assert!(!animating);
        } else {
            assert!(d.shown < 1.0 && animating, "it slides");
            let p = d.panel.expect("on its way");
            assert!(p.x < d.rest.x && p.w == d.rest.w, "from the edge");
        }
        assert!(d.rest.w <= 360.0 - view.theme.control.min_touch_target);
        assert!(view.close_top_overlay(), "Escape");
        let d = dock(&view);
        if reduced {
            assert_eq!((d.shown, d.panel), (0.0, None), "gone at once");
        }
    }
    set_reduced_motion(false);
}

/// A finger turns the sidebar into a drawer; open, a tap off it (on
/// the scrim) closes it, as winit's touch reaches the view (the
/// press and the release at the lift), and a tap on it does not.
#[test]
fn a_tap_off_the_open_drawer_closes_it() {
    use engine::compose::animation::set_reduced_motion;
    set_reduced_motion(true);
    let mut view = ShowcaseView::new(1280.0, 800.0);
    assert!(view.set_touch(true));
    let open = |v: &ShowcaseView| v.shell.dock.as_ref().is_some_and(|d| d.is_drawer_open());
    let tap = |v: &mut ShowcaseView, x: f32, y: f32| {
        v.handle_event(&WidgetEvent::MouseMove { x, y });
        v.handle_event(&WidgetEvent::MouseDown { x, y });
        v.handle_event(&WidgetEvent::MouseUp { x, y });
    };
    let mb = view
        .shell
        .layout(&view.theme)
        .menu_button
        .expect("the menu");
    let (x, y) = mb.center();
    tap(&mut view, x, y);
    assert!(open(&view), "the menu button opens it");
    let panel = view.panel_rect().expect("shown");
    // on the panel, below its rows: it stays
    tap(&mut view, panel.x + panel.w / 2.0, panel.bottom() - 4.0);
    assert!(open(&view), "a tap on it");
    tap(&mut view, (panel.right() + 1280.0) / 2.0, 400.0);
    assert!(!open(&view), "a tap off it closes it");
    set_reduced_motion(false);
}

/// The window's safe area holds the pinned panel and the page off
/// the notch's side.
#[test]
fn the_sidebar_and_the_page_stand_inside_the_safe_area() {
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.pin_sidebar(true);
    let inset = view.theme.spacing.xl;
    view.set_safe_area(SafeArea {
        left: inset,
        ..SafeArea::default()
    });
    let l = view.shell.layout(&view.theme);
    let dock = l.dock.expect("docked");
    assert_eq!(dock.mode, DockMode::Pinned);
    assert_eq!(dock.rest.x, inset);
    assert!(l.header.x >= dock.rest.right());
}

/// A sidebar 200 wide (narrower than its 240) keeps every group's
/// label on one line, cut with an ellipsis, never broken inside the
/// word: each trigger is as tall as in a wide panel and each tab a
/// link high; a reader and an agent still hear the whole name.
#[test]
fn a_narrow_sidebar_cuts_its_labels_on_one_line() {
    use accord::SnapshotBuilder;
    use engine::drawing::compositor::SceneNode;
    let theme = Theme::hoff_core();
    let mut nav = Navigation::new(0);
    for row in &mut nav.sidebar.rows {
        if let HomeRow::Menu(m, _) = row {
            *m = m.clone().open(true);
        }
    }
    let tall = 1e4;
    let narrow = Rect::new(0.0, 0.0, 200.0, tall);
    let wide = Rect::new(0.0, 0.0, 1000.0, tall);
    for (r, g) in Group::ALL.iter().enumerate() {
        let at = |b| nav.trigger_rect(r, b, &theme).unwrap().h;
        assert_eq!(at(narrow), at(wide), "{} takes one line", g.label());
    }
    let link_h = theme.disclosure.collapsible.rail.as_ref().unwrap().link_h;
    for (i, page) in PAGES.iter().enumerate() {
        let h = nav.tab_rect(i, narrow, &theme).unwrap().h;
        assert_eq!(h, link_h, "{} takes one line", page.title);
    }
    // drawn: each group's label one line (no wrap width), some cut
    let mut c = Compositor::new();
    nav.render(&mut c, narrow, &theme, true);
    let texts: Vec<_> = c
        .layer(LayerId::DEFAULT)
        .unwrap()
        .nodes()
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.clone()),
            _ => None,
        })
        .collect();
    let mut cut = 0;
    for g in Group::ALL {
        let label = g.label();
        let key = texts
            .iter()
            .find(|k| {
                let stem = k.text.trim_end_matches('…');
                k.text == label || (k.text.ends_with('…') && label.starts_with(stem))
            })
            .unwrap_or_else(|| panic!("{label} is drawn"));
        assert_eq!(key.max_width_bits, None, "{label} does not wrap");
        cut += usize::from(key.text != label);
    }
    assert!(cut > 0, "at 200 a label is cut");
    // projected: the whole names
    let id = SemId::name("nav").unwrap();
    nav.sidebar = nav.sidebar.clone().sem(id.clone());
    let mut b = SnapshotBuilder::new();
    nav.sidebar.project(&mut b, &id, "Pages", None);
    let snap = b.finish().unwrap();
    let rows = id.child("row").unwrap();
    for (r, g) in Group::ALL.iter().enumerate() {
        let node = snap.get(&rows.key(r.to_string())).unwrap();
        assert_eq!(node.label, g.label());
    }
}

#[test]
fn the_current_page_marks_its_tab_and_its_group_alone() {
    let last = PAGES.len() - 1;
    let mut nav = Navigation::new(0);
    nav.set_current(last);
    let (row, at) = place(last);
    for (r, (m, l)) in nav.menus().enumerate() {
        assert_eq!(l.current_link(), (r == row).then_some(at));
        if r == row {
            assert!(m.is_open(), "navigation opens the group");
        }
    }
    assert_eq!(nav.menus().count(), Group::ALL.len());
}

/// Target size (WCAG 2.5.8, AA): every control of the sidebar, its dock
/// and the palette is hit in a rect at least 24 px on each side, as it
/// is bound in a drawn frame: pinned, as an open drawer at 360, and the
/// palette open. Each measure prints (`--nocapture`) for the record.
#[test]
fn every_sidebar_and_palette_control_is_at_least_24_px() {
    use engine::drawing::compositor::Compositor;
    const AA: f32 = 24.0;
    // one compositor per view: its overlay layer lives in the first
    fn check(v: &mut ShowcaseView, c: &mut Compositor, what: &str, roots: &[&str]) -> usize {
        v.render(c);
        let snap = v.gallery_semantics().expect("a projection");
        let mut seen = 0;
        for (id, node) in snap.nodes() {
            let under = roots.iter().any(|r| id.as_str().starts_with(r));
            if !under || node.actions.is_empty() {
                continue;
            }
            let Some(b) = c.binding(id) else { continue };
            let [_, _, w, h] = b.rect;
            eprintln!("target {what} {id} {:?} {w:.1}x{h:.1}", node.role);
            assert!(w >= AA && h >= AA, "{what} {id}: {w}x{h} under {AA}");
            seen += 1;
        }
        seen
    }
    let sidebar = ["shell.sidebar"];
    let mut pinned = ShowcaseView::new(1280.0, 800.0);
    pinned.pin_sidebar(true);
    assert!(check(&mut pinned, &mut Compositor::new(), "pinned", &sidebar) > 10);

    let mut narrow = ShowcaseView::new(360.0, 740.0);
    let mut c = Compositor::new();
    assert!(
        check(&mut narrow, &mut c, "closed", &sidebar) >= 1,
        "the menu button"
    );
    let menu = narrow.shell.dock.as_ref().unwrap().menu_id().unwrap();
    assert!(narrow.ask_sidebar(&menu, "activate"));
    narrow.sidebar_wake(10.0);
    assert!(check(&mut narrow, &mut c, "drawer", &sidebar) > 10);

    let mut palette = ShowcaseView::new(1280.0, 800.0);
    assert!(palette.handle_chord("k", true));
    assert!(
        check(
            &mut palette,
            &mut Compositor::new(),
            "palette",
            &["palette"]
        ) > 5
    );
}

/// The width sweep (#163's acceptance): from 280 to 1600 every 8 px, in
/// core and hoff, pinned or not, the chrome cuts nothing. The panel, its
/// rows and their labels (one line each) stand inside the window; the
/// page's column starts past a pinned panel; the menu button and the
/// palette's sheet are inside the window, and a drawer leaves a touch
/// target of scrim to close it by.
#[test]
fn nothing_in_the_chrome_cuts_from_280_to_1600() {
    const E: f32 = 0.5;
    let inside = |r: Rect, o: Rect| {
        r.x >= o.x - E
            && r.y >= o.y - E
            && r.right() <= o.right() + E
            && r.bottom() <= o.bottom() + E
    };
    for name in ["core", "hoff"] {
        let mut v = ShowcaseView::new(1280.0, 800.0);
        v.apply_theme(name);
        let mut palette = ShowcaseView::new(1280.0, 800.0);
        palette.apply_theme(name);
        assert!(palette.handle_chord("k", true));
        let wide = Rect::new(0.0, 0.0, 1e3, 1e4);
        let one_line = v.nav.trigger_rect(0, wide, &v.theme).unwrap().h;
        for w in (280..=1600).step_by(8) {
            let w = w as f32;
            let window = Rect::new(0.0, 0.0, w, 800.0);
            v.resize(w, 800.0, 1.0);
            for pinned in [false, true] {
                v.pin_sidebar(pinned);
                let l = v.shell.layout(&v.theme);
                let d = l.dock.expect("docked");
                assert!(inside(d.rest, window), "{name} {w}: {:?}", d.rest);
                assert!(
                    inside(l.content, window) && inside(l.header, window),
                    "{name} {w}"
                );
                assert!(
                    l.content.x >= d.page_x - E,
                    "{name} {w}: the page past the panel"
                );
                if let Some(mb) = l.menu_button {
                    assert!(inside(mb, window), "{name} {w}: the menu");
                }
                if d.mode == DockMode::Drawer {
                    let scrim = w - d.rest.right();
                    assert!(scrim >= v.theme.control.min_touch_target - E, "{name} {w}");
                }
                let rows = v.nav.sidebar.layout(d.rest, &v.theme).rows;
                for (r, row) in rows.iter().enumerate() {
                    assert!(
                        row.x >= d.rest.x - E && row.right() <= d.rest.right() + E,
                        "{name} {w}"
                    );
                    let t = v.nav.trigger_rect(r, d.rest, &v.theme).unwrap();
                    assert_eq!(t.h, one_line, "{name} {w}: row {r} on one line");
                }
            }
            palette.resize(w, 800.0, 1.0);
            let theme = palette.theme.clone();
            let p = palette.palette_mut().expect("open");
            let sheet = p.sheet_rect(&theme, window);
            assert!(inside(sheet, window), "{name} {w}: the palette {sheet:?}");
            let box_ = p
                .command
                .layout(p.palette_rect(&theme, sheet), &theme)
                .search;
            assert!(inside(box_, sheet), "{name} {w}: the palette's box");
        }
    }
}
