//! context menu, modal and toast.

use super::*;

// ContextMenu

fn menu() -> ContextMenu {
    ContextMenu::new(vec![
        MenuEntry::item(1, "Open").icon("folder"),
        MenuEntry::item(2, "Rename").disabled(true),
        MenuEntry::Separator,
        MenuEntry::item(3, "Delete").intent(Intent::Destructive),
    ])
}

#[test]
fn context_menu_projects_a_menu_with_one_item_per_entry() {
    let file = id("file.menu");
    let m = menu();
    let s = snap(|b| {
        m.project(b, &file, "file actions", None);
    });
    let node = s.get(&file).unwrap();
    assert_eq!(
        (node.role, node.label.as_str()),
        (Role::Menu, "file actions")
    );
    assert_eq!(actions(&s, &file), ["close"]);
    let kids: Vec<(&str, Role, &str, bool)> = s
        .children(&file)
        .map(|n| (n.id.as_str(), n.role, n.label.as_str(), n.enabled))
        .collect();
    assert_eq!(
        kids,
        [
            ("file.menu[1]", Role::MenuItem, "Open", true),
            ("file.menu[2]", Role::MenuItem, "Rename", false),
            ("file.menu[3]", Role::MenuItem, "Delete", true),
        ]
    );
    let policy = |who: &str| s.get(&id(who)).unwrap().actions[0].policy;
    assert!(policy("file.menu[3]").destructive);
    assert!(!policy("file.menu[1]").destructive);
    assert_eq!(actions(&s, &id("file.menu[1]")), ["activate"]);
}

#[test]
fn context_menu_binds_itself_and_each_item_it_hit_tests_with() {
    let theme = hoff();
    let file = id("menu");
    let mut m = menu().sem(file.clone());
    let (x, y) = (40.0, 60.0);
    let mut c = Compositor::new();
    let overlay = c.create_layer(200);
    c.begin_frame();
    m.render(&mut c, overlay, &theme, x, y);
    let (w, h) = m.size(&theme);
    assert_eq!(
        c.binding(&file).map(|b| (b.rect, b.layer)),
        Some(([x, y, w, h], overlay))
    );
    let rows = m.entry_rects(x, y, &theme);
    for (key, row) in [("1", rows[0]), ("2", rows[1]), ("3", rows[3])] {
        assert_bound(&c, &file.key(key), row, overlay);
    }
    // A separator is the menu's, not an item's.
    let (sx, sy) = rows[2].center();
    assert_eq!(c.hit(sx, sy), Some(&file));
    let (ix, iy) = rows[3].center();
    assert_eq!(m.handle_event(&down(ix, iy), x, y, &theme).1, Some(3));
}

#[test]
fn context_menu_hover_is_visual_only() {
    let theme = hoff();
    let file = id("menu");
    let mut m = menu();
    let seen = snap(|b| {
        m.project(b, &file, "file actions", None);
    });
    let req = Request::observe(&seen, test_actor(), id("menu[1]"), "activate", None).unwrap();
    let (ix, iy) = m.entry_rects(0.0, 0.0, &theme)[0].center();
    m.handle_event(&move_to(ix, iy), 0.0, 0.0, &theme);
    assert_eq!(m.hovered(), Some(0));
    let now = snap(|b| {
        m.project(b, &file, "file actions", None);
    });
    assert_visual_only(&req, &seen, &now);
}

// Modal

const VIEW: (f32, f32) = (1024.0, 768.0);

#[test]
fn modal_projects_a_dialog_whose_children_are_its_buttons() {
    let confirm = id("confirm");
    let modal = Modal::new("Delete branch?", "This cannot be undone.", "Delete", "Keep")
        .intent(Intent::Destructive);
    let s = snap(|b| {
        modal.project(b, &confirm, None);
    });
    let node = s.get(&confirm).unwrap();
    assert_eq!(
        (node.role, node.label.as_str()),
        (Role::Dialog, "Delete branch?")
    );
    assert_eq!(node.description.as_deref(), Some("This cannot be undone."));
    assert_eq!(actions(&s, &confirm), ["confirm", "cancel"]);
    let kids: Vec<(&str, Role, &str)> = s
        .children(&confirm)
        .map(|n| (n.id.as_str(), n.role, n.label.as_str()))
        .collect();
    assert_eq!(
        kids,
        [
            ("confirm[confirm]", Role::Button, "Delete"),
            ("confirm[cancel]", Role::Button, "Keep"),
        ]
    );
    // A destructive modal's confirm is destructive, on the dialog and on
    // its button; the cancel never is.
    assert!(node.actions[0].policy.destructive);
    assert!(!node.actions[1].policy.destructive);
    let button = |key: &str| s.get(&confirm.key(key)).unwrap().actions[0].clone();
    assert_eq!(button("confirm").name.as_str(), "activate");
    assert!(button("confirm").policy.destructive);
    assert!(!button("cancel").policy.destructive);

    let calm = Modal::new("Save?", "", "Save", "Cancel");
    let s = snap(|b| {
        calm.project(b, &confirm, None);
    });
    let destructive = |who: &SemId| s.get(who).unwrap().actions[0].policy.destructive;
    assert!(!destructive(&confirm));
    assert!(!destructive(&confirm.key("confirm")));
}

#[test]
fn modal_binds_the_sheet_and_the_buttons_it_hit_tests_with() {
    let theme = hoff();
    let (vw, vh) = VIEW;
    let dialog = id("dialog");
    let mut modal = Modal::new("Title", "Body", "OK", "Cancel").sem(dialog.clone());
    let mut c = Compositor::new();
    let overlay = c.create_layer(300);
    c.begin_frame();
    modal.render(&mut c, overlay, &theme, vw, vh);
    let sheet = modal.dialog_rect(&theme, vw, vh);
    let (ok, cancel) = modal.button_rects(&theme, sheet);
    assert_eq!(
        c.binding(&dialog).map(|b| (b.rect, b.layer)),
        Some((rect_of(sheet), overlay))
    );
    assert_eq!(c.hit(sheet.x + 2.0, sheet.y + 2.0), Some(&dialog));
    assert_bound(&c, &dialog.key("confirm"), ok, overlay);
    assert_bound(&c, &dialog.key("cancel"), cancel, overlay);
    let (x, y) = ok.center();
    modal.handle_event(&down(x, y), &theme, vw, vh);
    assert_eq!(
        modal.handle_event(&up(x, y), &theme, vw, vh).0,
        ModalAction::Confirm
    );
}

#[test]
fn modal_hover_and_press_are_visual_only() {
    let theme = hoff();
    let (vw, vh) = VIEW;
    let dialog = id("dialog");
    let mut modal = Modal::new("Title", "Body", "OK", "Cancel");
    let seen = snap(|b| {
        modal.project(b, &dialog, None);
    });
    let req = Request::observe(&seen, test_actor(), &dialog, "confirm", None).unwrap();
    let sheet = modal.dialog_rect(&theme, vw, vh);
    let (x, y) = modal.button_rects(&theme, sheet).0.center();
    modal.handle_event(&move_to(x, y), &theme, vw, vh);
    modal.handle_event(&down(x, y), &theme, vw, vh);
    let now = snap(|b| {
        modal.project(b, &dialog, None);
    });
    assert_visual_only(&req, &seen, &now);
}

// Toast

fn settle(toasts: &mut ToastManager) {
    for _ in 0..30 {
        toasts.wake(1.0 / 60.0);
    }
}

#[test]
fn toasts_project_a_status_or_an_alert_per_toast_on_screen() {
    let theme = hoff();
    let area = id("toasts");
    let mut toasts = ToastManager::new();
    toasts.max_visible = 2;
    toasts.push("saved", Intent::Constructive, &theme);
    toasts.push("push failed", Intent::Destructive, &theme);
    toasts.push("queued", Intent::Neutral, &theme);
    let s = snap(|b| {
        toasts.project(b, &area, "notifications", None);
    });
    let node = s.get(&area).unwrap();
    assert_eq!(
        (node.role, node.label.as_str()),
        (Role::Region, "notifications")
    );
    let kids: Vec<(&str, Role, &str)> = s
        .children(&area)
        .map(|n| (n.id.as_str(), n.role, n.label.as_str()))
        .collect();
    // The third waits in the queue, off screen.
    assert_eq!(
        kids,
        [
            ("toasts[0]", Role::Status, "saved"),
            ("toasts[1]", Role::Alert, "push failed"),
        ]
    );
    assert_eq!(actions(&s, &id("toasts[1]")), ["dismiss"]);
}

#[test]
fn a_toast_keeps_its_id_when_the_one_before_it_leaves() {
    let theme = hoff();
    let area = id("toasts");
    let mut toasts = ToastManager::new();
    toasts.push("first", Intent::Neutral, &theme);
    toasts.push("second", Intent::Neutral, &theme);
    settle(&mut toasts);
    assert!(toasts.dismiss_id(0));
    assert!(!toasts.dismiss_id(0), "already leaving");
    assert!(!toasts.dismiss_id(9), "no such toast");
    // A leaving toast is already dismissed: it is not projected.
    let s = snap(|b| {
        toasts.project(b, &area, "notifications", None);
    });
    let kids: Vec<&str> = s.children(&area).map(|n| n.id.as_str()).collect();
    assert_eq!(kids, ["toasts[1]"]);
    for _ in 0..240 {
        toasts.wake(1.0 / 60.0);
        if toasts.len() == 1 {
            break;
        }
    }
    assert_eq!(toasts.len(), 1);
    assert_eq!(toasts.visible().next().map(Toast::id), Some(1));
}

#[test]
fn toasts_bind_the_rect_each_one_is_drawn_and_clicked_in() {
    let theme = hoff();
    let (vw, vh) = VIEW;
    let area = id("toasts");
    let mut toasts = ToastManager::new().sem(area.clone());
    toasts.push("one", Intent::Neutral, &theme);
    toasts.push("two", Intent::Informational, &theme);
    settle(&mut toasts);
    let mut c = Compositor::new();
    let overlay = c.create_layer(400);
    c.begin_frame();
    toasts.render(&mut c, overlay, &theme, vw, vh);
    let rects = toasts.visible_rects(&theme, vw, vh);
    for (i, r) in rects.iter().enumerate() {
        assert_bound(&c, &area.key(i.to_string()), *r, overlay);
    }
    assert!(
        c.binding(&area).is_none(),
        "the region draws nothing itself"
    );
    let (x, y) = rects[1].center();
    assert!(toasts.handle_event(&down(x, y), &theme, vw, vh).clicked);
    assert!(toasts.visible().nth(1).is_some_and(Toast::is_closing));
}

#[test]
fn a_toast_sliding_in_is_visual_only() {
    let theme = hoff();
    let area = id("toasts");
    let mut toasts = ToastManager::new();
    toasts.push("saved", Intent::Constructive, &theme);
    let seen = snap(|b| {
        toasts.project(b, &area, "notifications", None);
    });
    let req = Request::observe(&seen, test_actor(), id("toasts[0]"), "dismiss", None).unwrap();
    settle(&mut toasts);
    let now = snap(|b| {
        toasts.project(b, &area, "notifications", None);
    });
    assert_visual_only(&req, &seen, &now);
}
