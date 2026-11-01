//! Headless view tests (no GPU: scenes build into a plain compositor):
//! the chrome, page switching, the reactivity probes and the page
//! scroll regressions.

use super::components::scroll_area::ScrollAreaPage;
use super::*;
use engine::drawing::compositor::{Compositor, SceneNode};
use engine::widgets::prelude::WidgetEvent;

#[test]
fn starts_on_the_first_page_with_the_core_theme() {
    let view = ShowcaseView::new(1200.0, 800.0);
    assert_eq!(view.page_title(), "Theme");
    assert_eq!(view.theme_name, "core");
    // The page is the kit's paper, shade 1.
    assert_eq!(
        view.theme.colors.bg.0,
        engine::styling::theme::hoff_core::SHADES[0].0
    );
}

// -- The sidebar (the kit's HomeSidebar where the dock puts it) ----------

/// The panel's rect now.
fn panel(view: &ShowcaseView) -> Rect {
    view.panel_rect().expect("the panel shows")
}

/// Page `i`'s tab in the panel, while its group is open.
fn tab(view: &ShowcaseView, i: usize) -> Option<Rect> {
    view.nav.tab_rect(i, panel(view), &view.theme)
}

/// A press and a release at the middle of `r`.
fn click(view: &mut ShowcaseView, r: Rect) -> bool {
    let (x, y) = r.center();
    view.handle_event(&WidgetEvent::MouseDown { x, y });
    view.handle_event(&WidgetEvent::MouseUp { x, y })
}

/// Runs the view's clock until the motions land.
fn settle(view: &mut ShowcaseView) {
    for _ in 0..120 {
        view.wake(1.0 / 60.0);
    }
}

#[test]
fn sidebar_scrolls_in_short_windows_and_stays_put_in_tall_ones() {
    // Short, on a component: its group open, the tabs overflow the body.
    let mut view = ShowcaseView::new(1280.0, 500.0);
    view.jump_to_page("Button");
    settle(&mut view);
    let mut c = Compositor::new();
    view.render(&mut c);
    let body = engine::widgets::nav::HomeSidebar::body_rect(panel(&view), &view.theme);
    let button = super::pages::find("Button").unwrap();
    let before = tab(&view, button).unwrap().y;
    assert!(view.nav.sidebar.max_scroll(panel(&view), &view.theme) > 0.0);

    // Wheel over the panel scrolls the panel, not the page.
    let (bx, by) = body.center();
    let changed = view.handle_event(&WidgetEvent::Scroll {
        x: bx,
        y: by,
        delta: 200.0,
    });
    assert!(changed, "sidebar scroll must request a redraw");
    assert_eq!(view.page_offset(), 0.0, "page must not scroll");
    assert!(tab(&view, button).unwrap().y < before, "tabs must move up");

    // Tall, on the first page: only Foundations open, everything fits,
    // nothing moves.
    let mut tall = ShowcaseView::new(1280.0, 1100.0);
    tall.render(&mut c);
    let body = engine::widgets::nav::HomeSidebar::body_rect(panel(&tall), &tall.theme);
    let (bx, by) = body.center();
    let before = tab(&tall, 0).unwrap().y;
    let changed = tall.handle_event(&WidgetEvent::Scroll {
        x: bx,
        y: by,
        delta: 200.0,
    });
    assert!(!changed, "nothing to scroll: no redraw");
    assert_eq!(tab(&tall, 0).unwrap().y, before);
}

#[test]
fn sidebar_body_clips_under_the_logo_and_does_not_hit_there() {
    let mut view = ShowcaseView::new(1280.0, 500.0);
    view.jump_to_page("Button");
    settle(&mut view);
    let p = panel(&view);
    let body = engine::widgets::nav::HomeSidebar::body_rect(p, &view.theme);
    let (bx, by) = body.center();
    view.handle_event(&WidgetEvent::Scroll {
        x: bx,
        y: by,
        delta: 400.0,
    });
    let mut c = Compositor::new();
    view.render(&mut c);
    let has_body_clip = c.layer(LayerId::DEFAULT).unwrap().nodes().iter().any(|n| {
        matches!(n, SceneNode::PushClip { x, y, w, h }
                if *x == body.x && *y == body.y && *w == body.w && *h == body.h)
    });
    assert!(
        has_body_clip,
        "the pinned sidebar's body clips under the logo"
    );

    // A tab scrolled under the logo is not there: a press on the head,
    // off the logo, picks nothing.
    let under = (0..PAGES.len())
        .find(|i| tab(&view, *i).is_some_and(|r| r.y >= p.y && r.bottom() <= body.y))
        .expect("a tab scrolled under the logo");
    let r = tab(&view, under).unwrap();
    let at = Rect::new(p.right() - 4.0, r.y, 2.0, r.h);
    click(&mut view, at);
    assert_eq!(view.page_title(), "Button");
}

#[test]
fn sidebar_tabs_fit_one_line_in_the_ui_font() {
    // Every tab keeps the kit's height: no title wraps in the rail list.
    let mut view = ShowcaseView::new(1280.0, 4000.0);
    for g in 0..Group::ALL.len() {
        if let Some(r) = view.nav.trigger_rect(g, panel(&view), &view.theme)
            && !view.nav.menus().nth(g).unwrap().0.is_open()
        {
            click(&mut view, r);
        }
    }
    settle(&mut view);
    let rail = view.theme.disclosure.collapsible.rail.unwrap();
    for (i, page) in PAGES.iter().enumerate() {
        let r = tab(&view, i).expect("every group open");
        assert_eq!(r.h, rail.link_h, "{} wraps", page.title);
    }
}

#[test]
fn sidebar_is_pinned_where_it_fits_and_a_drawer_where_it_does_not() {
    let t = Theme::hoff_core();
    let r = t.navigation.dock;
    // Wide: pinned beside the page, at the kit's width.
    let wide = ShowcaseView::new(1400.0, 900.0);
    assert_eq!(wide.page_x(), r.width);
    // Phone: a drawer behind the menu button; the page spans the width.
    let mut phone = ShowcaseView::new(390.0, 844.0);
    assert_eq!(phone.page_x(), 0.0);
    assert!(phone.panel_rect().is_none(), "the drawer starts shut");
    let mb = phone
        .shell
        .layout(&t)
        .menu_button
        .expect("menu button on a phone");
    phone.handle_event(&WidgetEvent::MouseMove {
        x: mb.center().0,
        y: mb.center().1,
    });
    assert!(click(&mut phone, mb));
    assert!(phone.shell.dock.as_ref().unwrap().is_drawer_open());
    settle(&mut phone);
    // Navigating from the drawer switches the page and closes it.
    let icons = super::pages::find("Icons").unwrap();
    let r = tab(&phone, icons).expect("Foundations open");
    click(&mut phone, r);
    assert_eq!(phone.page_title(), "Icons");
    assert!(!phone.shell.dock.as_ref().unwrap().is_drawer_open());
    // Escape closes it too.
    click(&mut phone, mb);
    assert!(phone.close_top_overlay());
    assert!(!phone.shell.dock.as_ref().unwrap().is_drawer_open());
    // Every page renders on the phone.
    for (i, page) in PAGES.iter().enumerate() {
        phone.set_page(i);
        let mut c = Compositor::new();
        phone.render(&mut c);
        assert!(page_nodes(&phone, &c) > 0, "{}", page.title);
    }
}

/// Nodes the page itself emitted on the default layer: what sits between
/// the content clip (the push at the page's left edge) and its pop.
fn page_nodes(view: &ShowcaseView, c: &Compositor) -> usize {
    let page_x = view.page_x();
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes();
    let start = nodes
        .iter()
        .position(|n| matches!(n, SceneNode::PushClip { x, .. } if *x == page_x))
        .expect("page content clip");
    nodes[start + 1..]
        .iter()
        .position(|n| matches!(n, SceneNode::PopClip))
        .expect("page content clip closes")
}

#[test]
fn every_page_renders_in_short_viewports() {
    // Width-only probes missed the sidebar overflow: pin short heights.
    for (w, h) in [(1280.0, 700.0), (1024.0, 600.0)] {
        let mut view = ShowcaseView::new(w, h);
        for (i, page) in PAGES.iter().enumerate() {
            view.set_page(i);
            let mut c = Compositor::new();
            view.render(&mut c);
            let nodes = page_nodes(&view, &c);
            assert!(nodes > 0, "{} at {w}x{h} emitted {nodes} nodes", page.title);
            // Page scroll limits track the real viewport height.
            assert_eq!(view.page_offset(), 0.0);
        }
    }
}

#[test]
fn jump_to_page_matches_case_insensitively() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("effects");
    assert_eq!(view.page_title(), "Effects");
    view.jump_to_page("CARD");
    assert_eq!(view.page_title(), "Card");
    view.jump_to_page("nope");
    assert_eq!(view.page_title(), "Card", "unknown names are ignored");
    // A title with a space, as the launch argument passes it.
    view.jump_to_page("data table");
    assert_eq!(view.page_title(), "Data Table");
}

#[test]
fn digit_keys_select_pages_and_t_cycles_themes() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    assert!(view.handle_key("2"));
    assert_eq!(view.page_title(), PAGES[1].title);
    assert!(view.handle_key("9"));
    assert_eq!(view.page_title(), PAGES[8].title);
    assert!(view.handle_key("0"), "0 reads as 10");
    assert_eq!(view.page_title(), PAGES[9].title);
    assert!(view.handle_key("1"));
    assert_eq!(view.page_title(), PAGES[0].title);

    assert!(view.handle_key("t"));
    assert_eq!(view.theme_name, "hoff");
    assert!(view.handle_key("t"));
    assert_eq!(view.theme_name, "dark");
    assert!(view.handle_key("t"));
    assert_eq!(view.theme_name, "light");
    assert!(view.handle_key("t"));
    assert_eq!(view.theme_name, "core");
}

#[test]
fn todo_add_field_takes_typing_enter_and_escape_through_the_chrome() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("todo");
    let mut c = Compositor::new();
    view.render(&mut c);
    // Click the add field (top of the app card), then type through
    // the chrome's key routes: "t" must type, not switch the theme.
    let content = view.page_rect();
    let (x, y) = view.panes.todo.input_rect(content, &view.theme).center();
    assert!(view.handle_event(&WidgetEvent::MouseDown { x, y }));
    assert!(view.handle_key("t"));
    assert_eq!(view.theme_name, "core", "typing must not cycle the theme");
    assert!(view.handle_key("x"));
    assert!(view.handle_edit_key(EditKey::Backspace));
    assert!(view.handle_enter(), "Enter adds the todo");
    // The new row's enter tween runs its course; what is left pending is
    // the focused caret, which asks for its next flip, not for every frame.
    let mut wake = view.wake(1.0 / 60.0);
    for _ in 0..600 {
        if !wake.is_continuous() {
            break;
        }
        wake = view.wake(1.0 / 60.0);
    }
    assert!(
        wake.next.is_some() && !wake.is_continuous(),
        "a focused field asks for its next flip, not for every frame: {wake:?}"
    );
    assert!(view.close_top_overlay(), "Escape blurs the field first");
    assert!(!view.handle_key("x"), "blurred: characters fall through");
}

#[test]
fn sidebar_offers_one_tab_per_page_under_three_dropdowns() {
    let view = ShowcaseView::new(1200.0, 800.0);
    assert_eq!(view.nav.menus().count(), Group::ALL.len());
    let lens: Vec<usize> = view.nav.menus().map(|(_, l)| l.len()).collect();
    let groups: Vec<usize> = Group::ALL.iter().map(|g| g.pages()).collect();
    assert_eq!(lens, groups);
    assert_eq!(
        lens.iter().sum::<usize>(),
        PAGES.len(),
        "the groups partition the pages"
    );
}

#[test]
fn folding_a_group_hides_its_tabs_and_navigation_opens_it() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    assert!(tab(&view, 0).is_some(), "the current group starts open");
    let foundations = view.nav.trigger_rect(0, panel(&view), &view.theme).unwrap();
    assert!(click(&mut view, foundations));
    assert!(tab(&view, 0).is_none(), "folded");
    view.jump_to_page("button");
    let (m, l) = view.nav.menus().nth(1).unwrap();
    assert!(m.is_open(), "navigating opens");
    let button = super::pages::find("button").unwrap();
    assert_eq!(l.current_link(), Some(super::navigation::place(button).1));
    // a tab picked asks the page
    settle(&mut view);
    let checkbox = super::pages::find("checkbox").unwrap();
    let r = tab(&view, checkbox).unwrap();
    assert!(click(&mut view, r));
    assert_eq!(view.page_title(), PAGES[checkbox].title);
}

#[test]
fn every_page_renders_a_scene() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    for (i, page) in PAGES.iter().enumerate() {
        view.set_page(i);
        let mut c = Compositor::new();
        view.render(&mut c);
        let nodes = page_nodes(&view, &c);
        assert!(nodes > 0, "{} emitted no nodes", page.title);
    }
}

/// Every text a page draws on the default layer.
fn page_texts(view: &mut ShowcaseView, title: &str) -> Vec<String> {
    view.jump_to_page(title);
    let mut c = Compositor::new();
    view.render(&mut c);
    c.layer(LayerId::DEFAULT)
        .unwrap()
        .nodes()
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.text.clone()),
            _ => None,
        })
        .collect()
}

/// Every alias left in the table says whose example it shares and draws
/// that example; every placeholder left names its slot. The pages come
/// from the table, so a page that becomes its own drops out of the check
/// and the check holds with none left.
#[test]
fn alias_pages_show_the_note_and_placeholders_name_their_slot() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    for page in PAGES {
        match page.body {
            Body::Alias { pane, .. } => {
                let owner = PAGES[pages::index_of(pane)].title;
                let texts = page_texts(&mut view, page.title);
                let note = format!("shares the {owner} example");
                assert!(
                    texts.iter().any(|t| t.starts_with(&note)),
                    "{}: {note}",
                    page.title
                );
                assert!(texts.len() > 2, "{}: the shared example draws", page.title);
            }
            Body::Placeholder => {
                let texts = page_texts(&mut view, page.title);
                assert!(
                    texts.iter().filter(|t| *t == page.title).count() >= 2,
                    "{}: header and slot",
                    page.title
                );
            }
            Body::Pane(_) => {}
        }
    }
}

// -- Reactivity regression probes (event chain must report changes) ----

/// Find the y position of a text node on a layer (probe helper).
fn text_node_y(c: &Compositor, layer: LayerId, needle: &str) -> Option<f32> {
    c.layer(layer).unwrap().nodes().iter().find_map(|n| {
        if let SceneNode::Text { key, y, .. } = n {
            (key.text == needle).then_some(*y)
        } else {
            None
        }
    })
}

#[test]
fn probe_scroll_over_virtual_list_reports_change_and_shifts_rows() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("scroll area");
    let mut c = Compositor::new();
    view.render(&mut c);

    let bounds = ScrollAreaPage::bounds(view.content_rect(), &view.theme);
    let (cx, cy) = bounds.center();
    let y_before = text_node_y(&c, view.layers.unwrap().list, "Item 0")
        .expect("Item 0 rendered before scroll");

    let changed = view.handle_event(&WidgetEvent::Scroll {
        x: cx,
        y: cy,
        delta: 48.0,
    });
    assert!(changed, "scroll over the virtual list must request redraw");
    assert!(
        view.panes.scroll_area.list.scroll.offset() > 0.0,
        "offset must move"
    );

    view.render(&mut c);
    let y_after = text_node_y(&c, view.layers.unwrap().list, "Item 0")
        .expect("Item 0 still in overscan after 48px scroll");
    assert!(
        y_after < y_before,
        "rows must shift up after scrolling down (before={y_before}, after={y_after})"
    );
}

#[test]
fn probe_hover_over_sidebar_reports_change() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    let rect = tab(&view, 2).unwrap();
    let (cx, cy) = rect.center();
    assert!(
        view.handle_event(&WidgetEvent::MouseMove { x: cx, y: cy }),
        "hover entering a sidebar item must request redraw"
    );
    assert!(
        !view.handle_event(&WidgetEvent::MouseMove { x: cx, y: cy }),
        "unchanged hover must not request redraw"
    );
    assert!(
        view.handle_event(&WidgetEvent::MouseMove { x: cx, y: 4.0 }),
        "hover leaving the sidebar item must request redraw"
    );
}

#[test]
fn probe_hover_over_buttons_reports_change() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("button");
    let content = view.content_rect();
    // The first card's widget sits at the card's center.
    let (cx, cy) = view.panes.button.first_widget_center(content, &view.theme);
    assert!(
        view.handle_event(&WidgetEvent::MouseMove { x: cx, y: cy }),
        "hover entering a button must request redraw"
    );
    assert!(
        view.handle_event(&WidgetEvent::MouseMove {
            x: cx,
            y: cy - 200.0
        }),
        "hover leaving a button must request redraw"
    );
}

#[test]
fn probe_click_sidebar_switches_page_and_reports_change() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    let rect = tab(&view, 3).unwrap();
    let (cx, cy) = rect.center();
    view.handle_event(&WidgetEvent::MouseDown { x: cx, y: cy });
    assert!(view.handle_event(&WidgetEvent::MouseUp { x: cx, y: cy }));
    assert_eq!(view.page_title(), PAGES[3].title);
}

#[test]
fn probe_ticks_settle_with_no_input() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    for (i, page) in PAGES.iter().enumerate() {
        view.set_page(i);
        let mut c = Compositor::new();
        view.render(&mut c);
        let mut animating = true;
        for _ in 0..600 {
            animating = view.tick(1.0 / 60.0);
            if !animating {
                break;
            }
        }
        assert!(!animating, "{}: tick never settles (busy loop)", page.title);
    }
}

#[test]
fn probe_idle_rerender_is_stable() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    for (i, page) in PAGES.iter().enumerate() {
        view.set_page(i);
        let mut c = Compositor::new();
        view.render(&mut c);
        view.tick(1.0 / 60.0);
        c.resolve_scene((1200.0, 800.0));
        for l in c.layers().iter().map(|l| l.id).collect::<Vec<_>>() {
            c.mark_layer_clean(l);
        }
        // No input: the next frame must hash identically.
        view.tick(1.0 / 60.0);
        view.render(&mut c);
        assert!(
            !c.needs_render(),
            "{}: idle re-render produced a different scene",
            page.title
        );
    }
}

// -- Page scroll regressions (HOFF pages overflow the 800px window) ----

#[test]
fn wheel_scrolls_overflowing_pages_and_requests_redraw() {
    for title in ["Card", "Theme"] {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.jump_to_page(title);
        let mut c = Compositor::new();
        view.render(&mut c);

        let content = view.content_rect();
        assert!(
            view.page_content_height(content) > content.h,
            "{title} fits in 800px now; pick a taller page for this test"
        );

        let (cx, cy) = content.center();
        let changed = view.handle_event(&WidgetEvent::Scroll {
            x: cx,
            y: cy,
            delta: 120.0,
        });
        assert!(changed, "{title}: page scroll must request a redraw");
        assert!(view.page_offset() > 0.0, "{title}: page offset must move");

        // Scrolling back past the top clamps at zero and still redraws.
        let changed = view.handle_event(&WidgetEvent::Scroll {
            x: cx,
            y: cy,
            delta: -10_000.0,
        });
        assert!(changed);
        assert_eq!(view.page_offset(), 0.0, "clamped at the top");
    }
}

#[test]
fn page_scroll_shifts_rendered_nodes_and_clips_to_viewport() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("theme");
    let mut c = Compositor::new();
    view.render(&mut c);
    let y_before = text_node_y(&c, LayerId::DEFAULT, "Palettes").expect("palettes label rendered");

    let content = view.content_rect();
    let (cx, cy) = content.center();
    assert!(view.handle_event(&WidgetEvent::Scroll {
        x: cx,
        y: cy,
        delta: 100.0,
    }));
    view.render(&mut c);
    let y_after = text_node_y(&c, LayerId::DEFAULT, "Palettes").expect("still emitted");
    assert!(
        (y_before - y_after - 100.0).abs() < 0.5,
        "nodes must shift up by the scrolled amount (before={y_before}, after={y_after})"
    );

    // The scrolled content is wrapped in a clip to the content viewport
    // (the sidebar band has its own clip at x=0: find the content one).
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes();
    let page_x = view.page_x();
    let clip = nodes.iter().find_map(|n| match n {
        SceneNode::PushClip { x, y, w, h } if *x == page_x => Some((*x, *y, *w, *h)),
        _ => None,
    });
    let clip = clip.expect("page content must be clipped while scrolled");
    assert_eq!(clip.0, page_x);
    assert_eq!(clip.1, content.y);
}

#[test]
fn scroll_over_virtual_list_keeps_priority_over_page_scroll() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("scroll area");
    let mut c = Compositor::new();
    view.render(&mut c);

    let bounds = ScrollAreaPage::bounds(view.content_rect(), &view.theme);
    let (cx, cy) = bounds.center();
    assert!(view.handle_event(&WidgetEvent::Scroll {
        x: cx,
        y: cy,
        delta: 48.0,
    }));
    assert!(view.panes.scroll_area.list.scroll.offset() > 0.0);
    assert_eq!(
        view.page_offset(),
        0.0,
        "the scroll area page itself fits the viewport and must not scroll"
    );
}

#[test]
fn clicks_on_the_header_band_do_not_reach_scrolled_widgets() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("theme");
    let mut c = Compositor::new();
    view.render(&mut c);

    let content = view.content_rect();
    let (cx, cy) = content.center();
    // Scroll until palette cards sit underneath the header band.
    view.handle_event(&WidgetEvent::Scroll {
        x: cx,
        y: cy,
        delta: 160.0,
    });
    let before = view.theme_name.clone();
    // Click in the header band: must not pick a (hidden) theme card.
    view.handle_event(&WidgetEvent::MouseDown { x: cx, y: 60.0 });
    assert_eq!(view.theme_name, before, "header clicks must hit nothing");
}

#[test]
fn probe_event_change_marks_compositor_needs_render() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("scroll area");
    let mut c = Compositor::new();
    view.render(&mut c);
    c.resolve_scene((1200.0, 800.0));
    for l in c.layers().iter().map(|l| l.id).collect::<Vec<_>>() {
        c.mark_layer_clean(l);
    }
    assert!(!c.needs_render(), "clean after render+resolve");

    let bounds = ScrollAreaPage::bounds(view.content_rect(), &view.theme);
    let (cx, cy) = bounds.center();
    let changed = view.handle_event(&WidgetEvent::Scroll {
        x: cx,
        y: cy,
        delta: 48.0,
    });
    assert!(changed);
    view.render(&mut c);
    assert!(
        c.needs_render(),
        "scrolled scene must be detected as needing render"
    );
}
