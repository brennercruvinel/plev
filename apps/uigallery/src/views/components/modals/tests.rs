use super::*;

/// From 280 to 1600 every 8 px, in the four themes: every modal sits
/// inside its card's body, and the card inside the content.
#[test]
fn every_modal_fits_its_card_across_the_width_range() {
    for theme in [
        Theme::hoff_core(),
        Theme::hoff(),
        Theme::dark(),
        Theme::light(),
    ] {
        let page = ModalsPage::new();
        let mut w = 280.0;
        while w <= 1600.0 {
            let content = Rect::new(0.0, 0.0, w, 10_000.0);
            for (g, cards) in page.rects(content, &theme).iter().enumerate() {
                for (c, (card, modal)) in cards.iter().enumerate() {
                    let body = showcase::body(*card, &theme);
                    let at = format!("{w}: group {g} card {c}");
                    assert!(
                        card.x >= content.x && card.right() <= content.right() + 0.01,
                        "{at}: the card"
                    );
                    assert!(
                        modal.x >= body.x - 0.5 && modal.right() <= body.right() + 0.5,
                        "{at}: wider than its card ({modal:?} in {body:?})"
                    );
                    assert!(
                        modal.y >= body.y - 0.5 && modal.bottom() <= body.bottom() + 0.5,
                        "{at}: taller than its card ({modal:?} in {body:?})"
                    );
                }
            }
            w += 8.0;
        }
    }
}

/// Every modal is projected under its group, named by its title, and
/// every control under it is the page's.
#[test]
fn every_modal_projects_under_its_group() {
    let page = ModalsPage::new();
    let Some(root) = page.root().cloned() else {
        panic!("the root parses");
    };
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Modals");
    page.project(&mut b, &root);
    let Ok(seen) = b.finish() else {
        panic!("the projection holds");
    };
    for (g, group) in page.groups.iter().enumerate() {
        for (c, s) in group.cards.iter().enumerate() {
            let Some(id) = card_id(Some(&root), g, c) else {
                panic!("the card's id parses");
            };
            let node = seen.get(&id).unwrap_or_else(|| panic!("{id} is projected"));
            assert_eq!(node.label, s.body.title(), "{id}");
            assert_eq!(node.role, Role::Group);
        }
    }
    for id in seen.nodes().keys() {
        assert!(under(&root, id), "{id} is the page's");
    }
}
