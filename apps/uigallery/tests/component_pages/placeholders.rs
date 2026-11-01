//! No page draws or projects a note of its making: a card that names
//! the front a widget was waiting for ("Zoom, from front G") instead of
//! the widget. Every page, narrow and wide, tall enough to hold all of
//! it, its drawn text and its projection's labels and descriptions.

use engine::drawing::compositor::{Compositor, SceneNode};
use uigallery::views::{PAGES, ShowcaseView};

/// A note of the task's fronts: "front " and a capital letter that ends
/// a word ("front C, the …", "from front G").
fn names_a_front(text: &str) -> bool {
    text.match_indices("front ").any(|(i, m)| {
        let mut rest = text[i + m.len()..].chars();
        rest.next().is_some_and(|c| c.is_ascii_uppercase())
            && rest.next().is_none_or(|c| !c.is_alphanumeric())
    })
}

fn texts(c: &Compositor) -> Vec<String> {
    c.layers()
        .iter()
        .flat_map(|layer| layer.nodes())
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn the_note_finder_finds_the_notes_and_nothing_else() {
    assert!(names_a_front("Zoom, from front G"));
    assert!(names_a_front("QuickComment: front C, the Comments tab"));
    assert!(!names_a_front("the front Door"));
    assert!(!names_a_front("in front of the card"));
}

#[test]
fn no_page_names_the_front_of_its_making() {
    for w in [360.0, 1280.0] {
        for page in PAGES {
            let mut view = ShowcaseView::new(w, 6000.0);
            view.jump_to_page(page.title);
            let mut c = Compositor::new();
            view.render(&mut c);
            for text in texts(&c) {
                assert!(!names_a_front(&text), "{} at {w}: {text:?}", page.title);
            }
            let seen = view.semantics().unwrap_or_default();
            for node in seen.nodes().values() {
                let said = [Some(&node.label), node.description.as_ref()];
                for text in said.into_iter().flatten() {
                    assert!(!names_a_front(text), "{} at {w}: {text:?}", page.title);
                }
            }
        }
    }
}
