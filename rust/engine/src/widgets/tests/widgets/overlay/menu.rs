//! The kit's three menus (task-104 front B): their recipes on core and
//! elsewhere, the file menu's shortcut caps, the filter menu's checks,
//! and what a reader hears of both.

use super::*;
use crate::widgets::content::kbd::Kbd;
use crate::widgets::overlay::tokens::{MenuStyle, MenuTokens};
use accord::{SnapshotBuilder, Value};

fn drawn(menu: &ContextMenu, theme: &Theme) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    menu.render(&mut c, LayerId::DEFAULT, theme, 0.0, 0.0);
    c.layer(LayerId::DEFAULT)
        .map(|l| l.nodes().to_vec())
        .unwrap_or_default()
}

fn count(nodes: &[SceneNode], pick: fn(&SceneNode) -> bool) -> usize {
    nodes.iter().filter(|n| pick(n)).count()
}

/// The labels' left edges, top to bottom.
fn label_xs(nodes: &[SceneNode]) -> Vec<f32> {
    nodes
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { x, .. } => Some(*x),
            _ => None,
        })
        .collect()
}

#[test]
fn menu_tokens_are_the_kit_s_three_on_core_and_the_theme_s_menu_elsewhere() {
    let core = Theme::hoff_core();
    let o = &core.overlay;
    let geometry = |m: &MenuTokens| (m.min_w, m.item_h, m.radius, m.sep_h);
    // w-55 h-10 rounded-[1.25rem] mb-4; w-51 h-9; w-45 h-9 rounded-2xl my-2.
    assert_eq!(
        geometry(o.menu_for(MenuStyle::Profile)),
        (220.0, 40.0, 20.0, 16.0)
    );
    assert_eq!(
        geometry(o.menu_for(MenuStyle::File)),
        (204.0, 36.0, 20.0, 16.0)
    );
    assert_eq!(
        geometry(o.menu_for(MenuStyle::Filter)),
        (180.0, 36.0, 16.0, 17.0)
    );
    assert_eq!(
        geometry(o.menu_for(MenuStyle::Upload)),
        (204.0, 40.0, 20.0, 16.0)
    );
    let up = &o.upload_menu;
    let label = core.typography.run(up.label);
    assert_eq!((label.font_size, label.font_weight), (13.0, 500));
    assert_eq!(
        up.item_pad_x + up.icon + up.gap,
        8.0 + 24.0 + 10.0,
        "px-2, size-6, gap-2.5"
    );
    assert_eq!(core.typography.run(o.file_menu.label).font_weight, 500);
    assert_eq!(o.filter_menu.icon, 16.0, "the check's size-4");
    for theme in [Theme::hoff(), Theme::dark(), Theme::light()] {
        let o = &theme.overlay;
        // the theme's menu for every style, each style's label its kit role
        let menu = |label| o.menu.clone().labelled(label);
        assert_eq!(o.file_menu, menu(MenuTokens::hoff_core_file().label));
        assert_eq!(o.filter_menu, menu(MenuTokens::hoff_core_filter().label));
        assert_eq!(o.upload_menu, menu(MenuTokens::hoff_core_upload().label));
        assert_eq!(o.menu, MenuTokens::derive(&theme));
    }
}

#[test]
fn a_file_menu_row_draws_its_shortcut_in_a_cap_and_grows_for_it() {
    let theme = Theme::hoff_core();
    let plain =
        ContextMenu::new(vec![MenuEntry::item(1, "Create new scene...")]).style(MenuStyle::File);
    let keyed = ContextMenu::new(vec![
        MenuEntry::item(1, "Create new scene...").shortcut("⌘N"),
        MenuEntry::item(2, "A row whose label is long enough to fill it").shortcut("⇧⌘E"),
    ])
    .style(MenuStyle::File);
    let cap = theme.disclosure.kbd[0].shadow.len();
    let shadows = |n: &SceneNode| matches!(n, SceneNode::Shadow { .. });
    let menu = theme.overlay.file_menu.shadow.len();
    assert_eq!(count(&drawn(&plain, &theme), shadows), menu);
    assert_eq!(count(&drawn(&keyed, &theme), shadows), menu + 2 * cap);
    let m = &theme.overlay.file_menu;
    let long = "A row whose label is long enough to fill it";
    let (tw, _) = crate::drawing::text::TextMeasurer::measure_styled(
        long,
        &theme.typography.run(m.label),
        None,
    );
    let (cw, _) = Kbd::new("⇧⌘E").preferred_size(&theme);
    let (w, _) = keyed.size(&theme);
    assert!(
        w >= tw + m.gap + cw + 2.0 * (m.pad + m.item_pad_x),
        "room for the cap"
    );
    assert_eq!(
        plain.size(&theme).0,
        204.0,
        "the kit's w-51 holds a short row"
    );
}

#[test]
fn a_filter_row_keeps_its_check_slot_whether_checked_or_not() {
    let theme = Theme::hoff_core();
    let m = &theme.overlay.filter_menu;
    let menu = ContextMenu::new(vec![
        MenuEntry::item(1, "All scenes").checked(true),
        MenuEntry::item(2, "Designs").checked(false),
    ])
    .style(MenuStyle::Filter);
    let nodes = drawn(&menu, &theme);
    let at = m.pad + m.item_pad_x + m.icon + m.gap;
    assert_eq!(label_xs(&nodes), [at, at], "both labels after the slot");
    let paths = |n: &SceneNode| matches!(n, SceneNode::Path { .. });
    assert_eq!(count(&nodes, paths), 1, "the one check drawn");
}

#[test]
fn a_reader_hears_a_row_s_check_and_its_shortcut() {
    let menu = ContextMenu::new(vec![
        MenuEntry::item(1, "Export...").shortcut("⇧⌘E"),
        MenuEntry::item(2, "Designs").checked(true),
        MenuEntry::item(3, "Animation").checked(false),
    ]);
    let id = SemId::parse("menu").unwrap_or_else(|e| panic!("{e}"));
    let mut b = SnapshotBuilder::new();
    menu.project(&mut b, &id, "file menu", None);
    let s = b.finish().unwrap_or_else(|e| panic!("{e}"));
    let node = |k: &str| s.get(&id.key(k)).cloned().unwrap_or_else(|| panic!("{k}"));
    assert_eq!(node("1").description.as_deref(), Some("⇧⌘E"));
    assert_eq!(node("1").state.get("checked"), None, "no check, no state");
    assert_eq!(node("2").state.get("checked"), Some(&Value::Bool(true)));
    assert_eq!(node("3").state.get("checked"), Some(&Value::Bool(false)));
}

#[test]
fn the_menubar_is_the_kit_s_toolbar_on_core_and_the_theme_s_surface_elsewhere() {
    use crate::widgets::overlay::tokens::MenubarTokens;
    let core = Theme::hoff_core();
    let bar = &core.overlay.menubar;
    // rounded-[1.25rem], p-2 gap-2, two drops and the #ECECEC border.
    assert_eq!((bar.radius, bar.pad, bar.gap), (20.0, 8.0, 8.0));
    assert_eq!(bar.surface.shadow.len(), 3);
    for theme in [Theme::hoff(), Theme::dark(), Theme::light()] {
        assert_eq!(theme.overlay.menubar, MenubarTokens::derive(&theme));
    }
}

/// A low window (task-104p): the placement caps the menu to the room
/// under its trigger; the rows scroll inside the panel, clipped there,
/// the wheel moves them within their range, and a row the keys highlight
/// scrolls in the least that shows it.
#[test]
fn a_capped_menu_scrolls_its_rows_and_reveals_the_highlight() {
    use crate::widgets::overlay::dropdown_menu::DropdownMenu;
    let theme = Theme::hoff_core();
    let entries = (1..=12)
        .map(|i| MenuEntry::item(i, format!("Item {i}")))
        .collect();
    let mut menu = DropdownMenu::new(entries);
    menu.open();
    let (w, h) = menu.menu.size(&theme);
    let trigger = Rect::new(10.0, 10.0, 40.0, 30.0);
    let bounds = Rect::new(0.0, 0.0, 400.0, 200.0);
    let p = menu.placement(trigger, bounds, &theme);
    assert!(p.capped && p.rect.h < h, "the room caps it");
    let panel = p.rect;
    let max = menu.menu.max_offset(panel, &theme);
    assert!(max > 0.0);

    // the wheel scrolls, held to the range
    let (cx, cy) = (panel.x + w / 2.0, panel.y + panel.h / 2.0);
    let wheel = |by| WidgetEvent::Scroll {
        x: cx,
        y: cy,
        delta: by,
    };
    let (r, _) = menu.handle_event(&wheel(1000.0), trigger, panel, &theme);
    assert!(r.changed);
    assert_eq!(menu.offset(), max);
    menu.handle_event(&wheel(-1000.0), trigger, panel, &theme);
    assert_eq!(menu.offset(), 0.0);

    // the last row highlighted by the keys scrolls in, whole
    menu.highlight(Some(12));
    assert!(menu.fit(panel, &theme));
    let view = menu.menu.rows_view(panel, &theme);
    let rows = menu
        .menu
        .entry_rects(panel.x, panel.y - menu.offset(), &theme);
    let last = rows[11];
    assert!(last.y >= view.y - 0.01 && last.bottom() <= view.bottom() + 0.01);

    // the rows draw under a clip of the strip, and bind under it
    let mut c = Compositor::new();
    c.begin_frame();
    let id = SemId::parse("menu").unwrap();
    menu.menu = menu.menu.clone().sem(id.clone());
    menu.render(&mut c, LayerId::DEFAULT, panel, &theme);
    let clipped = c.binding(&id.key("12")).and_then(|b| b.clip);
    assert_eq!(clipped, Some([view.x, view.y, view.w, view.h]));
}
