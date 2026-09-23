//! Keyboard focus order from the semantic contract: Tab and Shift-Tab walk
//! the snapshot, not the geometry. The order is the tree order the
//! accessibility tree presents (each root in projection order, then its
//! descendants depth first, children in projection order), so a screen
//! reader and the keyboard agree on what comes next, and a control keeps
//! its place in the order when the layout moves it.
//!
//! A node takes keyboard focus when it is focusable, enabled and not
//! inert: an open modal leaves the page out of the order, and a disabled
//! control is skipped, as on the web. What focus means for the app (a
//! caret in a field, a ring on a button) stays the app's: it moves its
//! own focus to the id this module names, the same way it answers a
//! screen reader's `Assist::Focus`.

use sem::{Node, SemId, Snapshot};

use super::tree::tree_order;

/// Which way focus moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Tab.
    Next,
    /// Shift-Tab.
    Previous,
}

/// Whether `node` takes keyboard focus.
pub fn takes_focus(node: &Node) -> bool {
    node.focusable && node.enabled && !node.inert
}

/// Every node that takes keyboard focus, in tree order.
pub fn order(snapshot: &Snapshot) -> Vec<&SemId> {
    tree_order(snapshot)
        .into_iter()
        .filter(|n| takes_focus(n))
        .map(|n| &n.id)
        .collect()
}

/// Where focus goes from the snapshot's focused node: the next (or
/// previous) node that takes focus in tree order, wrapping around. With
/// nothing focused, or focus on a node that is gone, Tab starts at the
/// first and Shift-Tab at the last. A focused node that does not take
/// focus itself (it became inert) still counts as the place to move from.
/// `None` when nothing takes focus.
pub fn next(snapshot: &Snapshot, direction: Direction) -> Option<SemId> {
    let all = tree_order(snapshot);
    let from = snapshot
        .focused()
        .and_then(|f| all.iter().position(|n| n.id == *f));
    let len = all.len();
    let pick = |at: usize| all.get(at % len).filter(|n| takes_focus(n));
    let found = match (direction, from) {
        (Direction::Next, None) => all.iter().find(|n| takes_focus(n)),
        (Direction::Previous, None) => all.iter().rev().find(|n| takes_focus(n)),
        (Direction::Next, Some(at)) => (1..=len).find_map(|step| pick(at + step)),
        (Direction::Previous, Some(at)) => (1..=len).find_map(|step| pick(at + len - step)),
    };
    found.map(|n| n.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sem::{Role, SnapshotBuilder};

    fn sid(text: &str) -> SemId {
        match SemId::parse(text) {
            Ok(id) => id,
            Err(e) => panic!("{text}: {e}"),
        }
    }

    /// A form: a field, a tab list whose tabs are projected before their
    /// list (projection order is not tree order), a disabled button, a
    /// label that is not focusable, a submit.
    fn form(focus: Option<&str>, inert: bool) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        b.node("form", Role::Region, "form").inert(inert);
        b.node("form.name", Role::TextInput, "name").parent("form");
        for key in ["a", "b"] {
            b.node(format!("form.tabs[{key}]").as_str(), Role::Tab, key)
                .parent("form.tabs");
        }
        b.node("form.tabs", Role::TabList, "tabs").parent("form");
        b.node("form.hint", Role::Label, "hint").parent("form");
        b.node("form.reset", Role::Button, "reset")
            .parent("form")
            .enabled(false);
        b.node("form.send", Role::Button, "send").parent("form");
        b.node("help", Role::Link, "help");
        if let Some(f) = focus {
            b.focus(f);
        }
        match b.finish() {
            Ok(s) => s,
            Err(e) => panic!("{e}"),
        }
    }

    fn names(ids: &[&SemId]) -> Vec<String> {
        ids.iter().map(|id| id.to_string()).collect()
    }

    #[test]
    fn the_order_is_the_tree_order_of_the_nodes_that_take_focus() {
        let s = form(None, false);
        assert_eq!(
            names(&order(&s)),
            [
                "form.name",
                "form.tabs[a]",
                "form.tabs[b]",
                "form.send",
                "help"
            ]
        );
    }

    #[test]
    fn tab_and_shift_tab_follow_the_order_and_wrap() {
        let tab = |focus: &str| next(&form(Some(focus), false), Direction::Next);
        let back = |focus: &str| next(&form(Some(focus), false), Direction::Previous);
        assert_eq!(tab("form.name"), Some(sid("form.tabs[a]")));
        assert_eq!(tab("form.tabs[b]"), Some(sid("form.send")), "skips reset");
        assert_eq!(tab("help"), Some(sid("form.name")), "wraps");
        assert_eq!(back("form.name"), Some(sid("help")), "wraps back");
        assert_eq!(back("form.send"), Some(sid("form.tabs[b]")));
        // From a node that does not take focus: the neighbours in order.
        assert_eq!(tab("form.hint"), Some(sid("form.send")));
        assert_eq!(back("form.hint"), Some(sid("form.tabs[b]")));
    }

    #[test]
    fn with_nothing_focused_tab_starts_at_either_end() {
        let s = form(None, false);
        assert_eq!(next(&s, Direction::Next), Some(sid("form.name")));
        assert_eq!(next(&s, Direction::Previous), Some(sid("help")));
    }

    #[test]
    fn an_inert_region_leaves_the_order() {
        let s = form(Some("form.name"), true);
        assert_eq!(names(&order(&s)), ["help"]);
        assert_eq!(next(&s, Direction::Next), Some(sid("help")));
        assert_eq!(next(&s, Direction::Previous), Some(sid("help")));
    }

    #[test]
    fn nothing_to_focus_is_none() {
        let mut b = SnapshotBuilder::new();
        b.node("note", Role::Label, "note");
        let s = match b.finish() {
            Ok(s) => s,
            Err(e) => panic!("{e}"),
        };
        assert!(order(&s).is_empty());
        assert_eq!(next(&s, Direction::Next), None);
        assert_eq!(next(&Snapshot::default(), Direction::Previous), None);
    }
}
