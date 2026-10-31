//! The privacy promises on the component pages (task-75 phase 2b). No page
//! of this phase has a case: no secret value (a password, a code, a field
//! marked secret), no camera, no action for a person alone, no state only
//! a reader hears. This guards that: a component page that starts to show
//! one fails here, and asks for the test of its case (the ones of the
//! Capture, Files and Silences pages are the pattern).

use accord::Role;
use uigallery::views::PAGES;

use super::harness::gallery;

#[test]
fn no_component_page_has_a_secret_a_camera_a_person_s_action_or_a_reader_s_state() {
    let mut pages = 0;
    for page in PAGES {
        let g = gallery(page.title);
        let Some(seen) = g.view.semantics() else {
            continue;
        };
        if !seen.roots().any(|n| g.view.is_component(&n.id)) {
            continue;
        }
        pages += 1;
        for node in seen.nodes().values() {
            let at = format!("{}: {}", page.title, node.id);
            assert!(!node.constraints.secret, "{at} is secret");
            assert_ne!(node.role, Role::Media, "{at} is a camera or a sound");
            for action in &node.actions {
                assert!(
                    action.policy.actors.agent,
                    "{at} {} is refused to an agent",
                    action.name
                );
            }
        }
        assert_eq!(
            g.view.reader_semantics(),
            Some(seen.clone()),
            "{}: nothing only a reader hears",
            page.title
        );
        let gallery = g.view.gallery_semantics().unwrap_or_default();
        for (id, node) in seen.nodes() {
            let served = gallery.get(id).map(|n| (&n.label, &n.state, &n.actions));
            assert_eq!(
                served,
                Some((&node.label, &node.state, &node.actions)),
                "{}: {id} goes out through the port as the page projects it",
                page.title
            );
        }
    }
    assert_eq!(pages, 32, "the component pages that project");
}
