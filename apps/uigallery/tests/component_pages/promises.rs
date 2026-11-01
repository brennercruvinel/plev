//! The privacy promises on the component pages (task-75 phase 2b). A
//! component page has no case unless it is listed in [`SECRETS`] with the
//! test that keeps its promise: no secret value (a password, a code, a
//! field marked secret), no camera, no action for a person alone, no state
//! only a reader hears. This guards that: a component page that starts to
//! show one fails here, and asks for the test of its case (the ones of the
//! Capture, Files and Silences pages are the pattern).

use accord::Role;
use uigallery::views::PAGES;

use super::fields;
use super::harness::gallery;

/// The component pages that project: the 31 before task-104, and each
/// front's own, on a line of its own so the fronts merge without a
/// conflict.
const PROJECTING: usize = 31
    // task-104 front A, fields (#154): Input Group, Input OTP, Textarea,
    // Command (Field, Label and Combobox projected before, through the pages
    // they borrowed)
    + 4
    // task-104 front E, selection (#158): Radio Group, Calendar, Date
    // Picker (Toggle Group projected before, through the Todo it borrowed)
    + 3
    // task-104 front B, overlays (#155): Dropdown Menu, Menubar, Sheet, Drawer,
    // Context Menu
    + 5
    // task-104 front C, conversation (#156): Message, Bubble, Message
    // Scroller, Alert, Attachment, Marker, Badge, Avatar, Skeleton, Progress;
    // wave 3: Prompt Input, Comments, Notifications
    + 13
    // task-104 front D, disclosure and display (#157): Accordion,
    // Collapsible, Aspect Ratio, Carousel, Pagination (Item, Kbd and Table
    // projected before, through the pages they borrowed), Panel, Panel
    // Header
    + 7
    // task-104 front G, navigation and data: Chart, Code Block, Graph, Stat
    + 4;

/// The pages that show a secret value on purpose, each with the test that
/// proves the person, the reader and the agent see no more of it than its
/// length.
const SECRETS: &[(&str, fn())] = &[
    // the password fields
    (
        "Field",
        fields::a_password_takes_the_same_secret_from_the_three_and_shows_its_length_only,
    ),
    // the one-time codes
    (
        "Input OTP",
        fields::a_code_takes_the_same_digits_from_the_three_and_shows_its_length_only,
    ),
];

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
        let keeps_secrets = SECRETS.iter().any(|(title, _)| *title == page.title);
        let mut secrets = 0;
        for node in seen.nodes().values() {
            let at = format!("{}: {}", page.title, node.id);
            secrets += usize::from(node.constraints.secret);
            assert!(
                keeps_secrets || !node.constraints.secret,
                "{at} is secret, and the page is not in SECRETS"
            );
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
            keeps_secrets,
            secrets > 0,
            "{}: in SECRETS exactly when it shows a secret",
            page.title
        );
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
    assert_eq!(pages, PROJECTING, "the component pages that project");
}
