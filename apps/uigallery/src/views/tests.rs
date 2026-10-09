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

// -- Sidebar responsiveness (the tree overflows short windows) ----------

/// The sidebar's link rects at the current viewport.
fn link_rects(view: &ShowcaseView) -> Vec<Rect> {
    let sb = &view.shell.sidebar;
    let rect = sb.rect(&view.theme, view.width, view.height).unwrap();
    sb.link_rects(rect, &view.theme, sb.mode(&view.theme, view.width))
}

fn band(view: &ShowcaseView) -> Rect {
    let sb = &view.shell.sidebar;
    let rect = sb.rect(&view.theme, view.width, view.height).unwrap();
    sb.band(rect, &view.theme, sb.mode(&view.theme, view.width))
}

#[test]
fn sidebar_scrolls_in_short_windows_and_stays_put_in_tall_ones() {
    // Short: the links overflow the band, the rail becomes scrollable.
    let mut view = ShowcaseView::new(1280.0, 500.0);
    let mut c = Compositor::new();
    view.render(&mut c);
    let last_before = link_rects(&view).last().unwrap().y;
    let band_bottom = band(&view).bottom();
    assert!(
        last_before + NavLink::height(&view.theme) > band_bottom,
        "500px window: the last link must start below the visible band"
    );

    // Wheel over the rail scrolls the rail, not the page.
    let (bx, by) = band(&view).center();
    let changed = view.handle_event(&WidgetEvent::Scroll {
        x: bx,
        y: by,
        delta: 200.0,
    });
    assert!(changed, "sidebar scroll must request a redraw");
    assert_eq!(view.page_offset(), 0.0, "page must not scroll");
    let last_after = link_rects(&view).last().unwrap().y;
    assert!(last_after < last_before, "links must move up");

    // Tall, with the Components group folded: everything fits, nothing
    // moves.
    let mut tall = ShowcaseView::new(1280.0, 1100.0);
    tall.render(&mut c);
    let sb = &tall.shell.sidebar;
    let rect = sb.rect(&tall.theme, tall.width, tall.height).unwrap();
    let caption = sb.group_rects(rect, &tall.theme, sb.mode(&tall.theme, tall.width))[1];
    let (x, y) = caption.center();
    assert!(tall.handle_event(&WidgetEvent::MouseDown { x, y }));
    let (bx, by) = band(&tall).center();
    let before = link_rects(&tall)[0].y;
    let changed = tall.handle_event(&WidgetEvent::Scroll {
        x: bx,
        y: by,
        delta: 200.0,
    });
    assert!(!changed, "nothing to scroll: no redraw");
    assert_eq!(link_rects(&tall)[0].y, before);
}

#[test]
fn sidebar_links_clip_to_their_band_and_do_not_hit_outside_it() {
    let mut view = ShowcaseView::new(1280.0, 500.0);
    let mut c = Compositor::new();
    view.render(&mut c);
    let band = band(&view);
    let overlay = view.layers.unwrap().overlay;
    let has_band_clip = c.layer(overlay).unwrap().nodes().iter().any(|n| {
        matches!(n, SceneNode::PushClip { x, y, w, h }
                if *x == band.x && *y == band.y && *w == band.w && *h == band.h)
    });
    assert!(has_band_clip, "sidebar links must be clipped to the band");

    // A point below the band (footer zone) must not hover a link that
    // scrolled-space-wise would live there.
    let below = band.bottom() + 2.0;
    assert!(below < view.height, "test point must be inside the window");
    assert!(
        !view.handle_event(&WidgetEvent::MouseMove { x: 100.0, y: below }),
        "footer zone must not hover a clipped link"
    );
    assert!(view.shell.sidebar.links.iter().all(|l| !l.is_hovered()));
}

#[test]
fn sidebar_labels_fit_their_slot_in_the_ui_font() {
    // Every page title fits the full sidebar's link width with the
    // icon slot, the shortcut hint and the paddings taken out.
    let view = ShowcaseView::new(1280.0, 800.0);
    let t = &view.theme;
    let link_w = link_rects(&view)[0].w;
    let style = NavLink::label_style(t);
    let (hint_w, _) =
        engine::drawing::text::TextMeasurer::measure_styled("14", &t.typography.caption_r(), None);
    let slot =
        link_w - t.spacing.sm * 2.0 - NavLink::icon_slot(t) - t.spacing.xs - hint_w - t.spacing.sm;
    for page in PAGES {
        let (w, _) = engine::drawing::text::TextMeasurer::measure_styled(page.title, &style, None);
        assert!(
            w <= slot,
            "{} label measures {w:.1}px, over the {slot:.1}px slot",
            page.title
        );
    }
}

#[test]
fn sidebar_follows_the_breakpoint() {
    let t = Theme::hoff();
    // Desktop: the full sidebar takes its token width from the page.
    let wide = ShowcaseView::new(1400.0, 900.0);
    assert_eq!(wide.page_x(), t.size.sidebar_w);
    // Narrow desktop window: the rail.
    let medium = ShowcaseView::new(800.0, 700.0);
    assert_eq!(medium.page_x(), t.size.sidebar_rail_w);
    // Phone: a drawer behind the menu button; the page spans the width.
    let mut phone = ShowcaseView::new(390.0, 844.0);
    assert_eq!(phone.page_x(), 0.0);
    let mb = phone
        .shell
        .layout(&t)
        .menu_button
        .expect("menu button on a phone");
    let (x, y) = mb.center();
    phone.handle_event(&WidgetEvent::MouseDown { x, y });
    assert!(phone.handle_event(&WidgetEvent::MouseUp { x, y }));
    assert!(phone.shell.sidebar.drawer_open);
    // Navigating from the drawer switches the page and closes it.
    let rects = link_rects(&phone);
    let (x, y) = rects[2].center();
    phone.handle_event(&WidgetEvent::MouseDown { x, y });
    assert!(phone.handle_event(&WidgetEvent::MouseUp { x, y }));
    assert_eq!(phone.page_title(), "Icons");
    assert!(!phone.shell.sidebar.drawer_open);
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
fn sidebar_offers_one_nav_link_per_page_under_three_captions() {
    let view = ShowcaseView::new(1200.0, 800.0);
    let rects = link_rects(&view);
    assert_eq!(rects.len(), PAGES.len());
    assert!(rects.iter().all(|r| r.h == NavLink::height(&view.theme)));
    let sb = &view.shell.sidebar;
    assert_eq!(sb.groups.len(), Group::ALL.len());
    assert_eq!(
        sb.groups.iter().map(|g| g.len).sum::<usize>(),
        PAGES.len(),
        "the groups partition the pages"
    );
}

#[test]
fn folding_the_components_group_hides_its_links_and_navigation_unfolds_it() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    let sb = &view.shell.sidebar;
    let rect = sb.rect(&view.theme, view.width, view.height).unwrap();
    let captions = sb.group_rects(rect, &view.theme, sb.mode(&view.theme, view.width));
    let (x, y) = captions[1].center();
    assert!(view.handle_event(&WidgetEvent::MouseDown { x, y }));
    let first_component = Group::Foundations.pages();
    assert_eq!(link_rects(&view)[first_component].h, 0.0, "folded");
    view.jump_to_page("button");
    assert!(view.shell.sidebar.groups[1].expanded, "navigating unfolds");
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

#[test]
fn alias_pages_show_the_note_and_placeholders_name_their_slot() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("field");
    let mut c = Compositor::new();
    view.render(&mut c);
    let texts: Vec<String> = c
        .layer(LayerId::DEFAULT)
        .unwrap()
        .nodes()
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.text.clone()),
            _ => None,
        })
        .collect();
    assert!(
        texts
            .iter()
            .any(|t| t.starts_with("shares the Input example"))
    );
    assert!(
        texts.iter().any(|t| t == "Full name"),
        "the shared example draws"
    );

    view.jump_to_page("calendar");
    let mut c = Compositor::new();
    view.render(&mut c);
    let texts: Vec<String> = c
        .layer(LayerId::DEFAULT)
        .unwrap()
        .nodes()
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.text.clone()),
            _ => None,
        })
        .collect();
    assert!(
        texts.iter().filter(|t| *t == "Calendar").count() >= 2,
        "header and slot"
    );
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
    let rect = link_rects(&view)[2];
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
    let rect = link_rects(&view)[3];
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
    let y_before = text_node_y(&c, LayerId::DEFAULT, "PALETTES").expect("palettes label rendered");

    let content = view.content_rect();
    let (cx, cy) = content.center();
    assert!(view.handle_event(&WidgetEvent::Scroll {
        x: cx,
        y: cy,
        delta: 100.0,
    }));
    view.render(&mut c);
    let y_after = text_node_y(&c, LayerId::DEFAULT, "PALETTES").expect("still emitted");
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
