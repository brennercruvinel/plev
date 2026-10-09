//! The keyboard on the component pages (task-75 phases 2c and 2d): Tab
//! and Shift-Tab walk the projection in the order a screen reader walks
//! it, the arrow keys move inside a tab list, a tree, a list, a table, a
//! menu and an open select as the WAI-ARIA patterns ask and step a
//! slider, and Enter and Space press what has the focus.
//!
//! A list and a table are one stop of Tab each: their projection makes
//! one row focusable (the focused one, else the selected one in view,
//! else the first in view; the comps decides), so `focus::next` stops
//! once in each, and the arrows move the focus among all the rows,
//! focusable or not (the roving tabindex of WAI-ARIA). A tab list and a
//! tree stay a stop per item: the contract ties a screen reader's focus
//! to the Tab sequence, and an item out of it would lose that focus.
//!
//! There is one order and it is the projection's: `focus::next` over the
//! view's snapshot (`ShowcaseView::semantics`: the page, the toasts and
//! the overlay that holds the pointer) names the next node, and the page
//! takes it through `assist_focus`, the way it takes the focus a screen
//! reader moves; an arrow picks among the nodes of that same order that
//! belong to the widget the focus is in. While a modal holds the pointer
//! the page beneath is inert, so the walk never leaves the modal (the
//! trap); the page keeps its own focus meanwhile, and it is the
//! projection's focus again once the modal closes (the return).
//!
//! What a key asks of a widget (press, select, open, expand, step) is the
//! person's request through the gallery's one dispatcher (`act`), the
//! path a click on the modal, a screen reader's press and an agent's act
//! take; the key adds no capability and no path of its own.

use engine::control::accessibility::focus::{self, Direction};
use engine::control::accessibility::primary;
use accord::{Actor, Node, Request, Role, SemId, Snapshot, Value};

use super::super::ShowcaseView;
use super::super::components::key_under;

/// The steps a continuous slider (one without a step of its own) takes
/// from one end to the other, one per arrow key.
const CONTINUOUS_STEPS: f64 = 100.0;

/// A key the keyboard walks the projection with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::views) enum Walk {
    /// Tab, or Shift-Tab while the shift key is held.
    Tab,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    /// Enter or Space.
    Press,
}

/// Where an arrow moves inside a widget.
#[derive(Clone, Copy)]
enum Move {
    Prev,
    Next,
    First,
    Last,
}

/// The focused node of `seen`, with its id.
fn focused(seen: &Snapshot) -> Option<(SemId, &Node)> {
    let id = seen.focused()?;
    Some((id.clone(), seen.get(id)?))
}

/// The nearest ancestor of `node` with `role`.
fn ancestor<'a>(seen: &'a Snapshot, node: &Node, role: Role) -> Option<&'a Node> {
    let mut at = node.parent.as_ref().and_then(|p| seen.get(p));
    while let Some(n) = at {
        if n.role == role {
            return Some(n);
        }
        at = n.parent.as_ref().and_then(|p| seen.get(p));
    }
    None
}

/// The nodes under `root` that pass `keep` and could have the focus
/// (enabled, not inert), focusable or not, in the order `focus::order`
/// walks: depth first, children in projection order.
fn items<'a>(seen: &'a Snapshot, root: &SemId, keep: impl Fn(&Node) -> bool) -> Vec<&'a SemId> {
    let mut out = Vec::new();
    let mut stack: Vec<&Node> = seen.children(root).collect();
    stack.reverse();
    while let Some(n) = stack.pop() {
        if n.enabled && !n.inert && keep(n) {
            out.push(&n.id);
        }
        let from = stack.len();
        stack.extend(seen.children(&n.id));
        if let Some(pushed) = stack.get_mut(from..) {
            pushed.reverse();
        }
    }
    out
}

/// Where `mv` lands from `at` among `items`: wrapping around the ends, or
/// stopping at them. `None` when it stays.
fn step<'a>(items: &[&'a SemId], at: &SemId, mv: Move, wrap: bool) -> Option<&'a SemId> {
    let last = items.len().checked_sub(1)?;
    let from = items.iter().position(|id| *id == at)?;
    let to = match mv {
        Move::First => 0,
        Move::Last => last,
        Move::Next if from < last => from + 1,
        Move::Next if wrap => 0,
        Move::Prev if from > 0 => from - 1,
        Move::Prev if wrap => last,
        Move::Next | Move::Prev => from,
    };
    (to != from).then(|| items[to])
}

/// The move of a key along a horizontal widget (a tab list).
fn across(key: Walk) -> Option<Move> {
    match key {
        Walk::Left => Some(Move::Prev),
        Walk::Right => Some(Move::Next),
        Walk::Home => Some(Move::First),
        Walk::End => Some(Move::Last),
        _ => None,
    }
}

/// The move of a key along a vertical widget (a menu, a list of options,
/// a tree's rows).
fn down(key: Walk) -> Option<Move> {
    match key {
        Walk::Up => Some(Move::Prev),
        Walk::Down => Some(Move::Next),
        Walk::Home => Some(Move::First),
        Walk::End => Some(Move::Last),
        _ => None,
    }
}

/// A number the projection states.
fn number(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Num(v) => Some(*v),
        Value::Int(v) => Some(*v as f64),
        _ => None,
    }
}

impl ShowcaseView {
    /// Whether the keyboard walks the projection: on a component page, or
    /// while an overlay holds the pointer (the modal, the menu).
    pub(in crate::views) fn walks(&self) -> bool {
        self.overlay_holds_pointer()
            || self
                .pane()
                .is_some_and(|pane| self.panes.component(pane).is_some())
    }

    /// One key over the projection as it stands: `true` when it moved the
    /// focus or asked for something.
    pub(in crate::views) fn walk(&mut self, key: Walk) -> bool {
        let Some(seen) = self.semantics() else {
            return false;
        };
        match key {
            Walk::Tab => {
                let direction = if self.shift {
                    Direction::Previous
                } else {
                    Direction::Next
                };
                focus::next(&seen, direction).is_some_and(|id| self.assist_focus(&id))
            }
            Walk::Press => self.press(&seen),
            arrow => self.arrow(&seen, arrow),
        }
    }

    /// The person's request `action` on `id`, observed on `seen`, through
    /// the gallery's dispatcher. `true` when it went (a refusal still
    /// counts: the person aimed at it).
    fn ask(&mut self, seen: &Snapshot, id: &SemId, action: &str, param: Option<Value>) -> bool {
        match Request::observe(seen, Actor::Human, id, action, param) {
            Ok(req) => self.act(req).is_some(),
            Err(e) => {
                log::debug!("keyboard: {id} {action}: {e}");
                false
            }
        }
    }

    /// Enter or Space: the focused node's primary action, the one a
    /// screen reader's press asks for.
    fn press(&mut self, seen: &Snapshot) -> bool {
        let Some((id, node)) = focused(seen) else {
            return false;
        };
        let Some(action) = primary(node).map(|a| a.name.to_string()) else {
            return false;
        };
        if !self.ask(seen, &id, &action, None) {
            return false;
        }
        if node.role == Role::Select && action == "open" {
            self.focus_chosen(&id);
        }
        self.show_overlay_focus();
        true
    }

    /// A modal or a menu the key opened takes the focus the projection
    /// gives it (the modal's cancel, the menu's first item) and shows it.
    fn show_overlay_focus(&mut self) {
        if !self.overlay_holds_pointer() || self.overlay_focus.is_some() {
            return;
        }
        let at = self.semantics().and_then(|s| s.focused().cloned());
        if let Some(at) = at.filter(|at| self.is_overlay(at)) {
            self.focus_overlay(&at);
        }
    }

    /// An open select's focus to its chosen option.
    fn focus_chosen(&mut self, select: &SemId) -> bool {
        let Some(now) = self.semantics() else {
            return false;
        };
        let chosen = now.get(select).and_then(|n| n.state.get("selected"));
        let option = items(&now, select, |n| {
            n.role == Role::Option
                && n.parent.as_ref() == Some(select)
                && chosen == Some(&Value::Text(n.label.clone()))
        })
        .first()
        .map(|id| (*id).clone());
        option.is_some_and(|id| self.assist_focus(&id))
    }

    /// An arrow, Home or End on the focused node, by the pattern of the
    /// widget it is in.
    fn arrow(&mut self, seen: &Snapshot, key: Walk) -> bool {
        let Some((id, node)) = focused(seen) else {
            return false;
        };
        let (mv, wrap) = match node.role {
            Role::Tab => (across(key), true),
            Role::MenuItem => (down(key), true),
            Role::Option => (down(key), false),
            Role::TreeItem => return self.tree_key(seen, &id, node, key),
            Role::ListItem | Role::Row => return self.row_key(seen, &id, node, key),
            Role::Slider => return self.slide(seen, &id, node, key),
            Role::Select if matches!(key, Walk::Up | Walk::Down) => {
                let open = node.state.get("expanded") == Some(&Value::Bool(true));
                return if open {
                    self.focus_chosen(&id)
                } else {
                    self.press(seen)
                };
            }
            _ => return false,
        };
        let Some(mv) = mv else {
            return false;
        };
        let Some(parent) = node.parent.as_ref() else {
            return false;
        };
        let siblings = items(seen, parent, |n| {
            n.role == node.role && n.parent.as_ref() == Some(parent)
        });
        let to = step(&siblings, &id, mv, wrap).cloned();
        to.is_some_and(|to| self.assist_focus(&to))
    }

    /// A list or a table: up and down along its rows, Home and End to its
    /// first and last, whether the widget holds them now or not. A row's
    /// key is its index (`<list>[<index>]`, as the comps projects a list's
    /// and a table's rows) and the widget states its `count`; the page
    /// scrolls the row it takes into view.
    fn row_key(&mut self, seen: &Snapshot, id: &SemId, node: &Node, key: Walk) -> bool {
        let Some(mv) = down(key) else {
            return false;
        };
        let Some(list) = node.parent.as_ref() else {
            return false;
        };
        let count = seen
            .get(list)
            .and_then(|l| number(l.state.get("count")))
            .map_or(0, |c| c as usize);
        let at = key_under(list, id).and_then(|k| k.parse::<usize>().ok());
        let (Some(at), Some(last)) = (at, count.checked_sub(1)) else {
            return false;
        };
        let to = match mv {
            Move::First => 0,
            Move::Last => last,
            Move::Prev => at.saturating_sub(1),
            Move::Next => (at + 1).min(last),
        };
        to != at && self.assist_focus(&list.key(to.to_string()))
    }

    /// A tree: up and down along the rows it shows, Home and End to the
    /// first and the last; right opens a closed branch, or goes to the
    /// first child of an open one; left closes an open branch, or goes to
    /// the parent row.
    fn tree_key(&mut self, seen: &Snapshot, id: &SemId, node: &Node, key: Walk) -> bool {
        let Some(tree) = ancestor(seen, node, Role::Tree).map(|t| &t.id) else {
            return false;
        };
        let rows = items(seen, tree, |n| n.role == Role::TreeItem);
        let expanded = match node.state.get("expanded") {
            Some(Value::Bool(on)) => Some(*on),
            _ => None,
        };
        let to = match (key, expanded) {
            (Walk::Right, Some(false)) | (Walk::Left, Some(true)) => {
                return self.ask(seen, id, "toggle_expanded", None);
            }
            (Walk::Right, Some(true)) => rows
                .iter()
                .find(|r| seen.get(r).and_then(|n| n.parent.as_ref()) == Some(id))
                .copied(),
            (Walk::Left, _) => node
                .parent
                .as_ref()
                .filter(|p| seen.get(p).is_some_and(|n| n.role == Role::TreeItem)),
            (key, _) => down(key).and_then(|mv| step(&rows, id, mv, false)),
        };
        let to = to.cloned();
        to.is_some_and(|to| self.assist_focus(&to))
    }

    /// A slider: right and up raise it a step, left and down lower it,
    /// Home and End take it to its ends. A stepped slider takes its own
    /// `increment` and `decrement`; a continuous one is set, in
    /// `CONTINUOUS_STEPS` steps of its range.
    fn slide(&mut self, seen: &Snapshot, id: &SemId, node: &Node, key: Walk) -> bool {
        let Some((min, max)) = node.constraints.range else {
            return false;
        };
        let Some(value) = number(node.state.get("value")) else {
            return false;
        };
        let has = |name: &str| node.actions.iter().any(|a| a.name.as_str() == name);
        let nudge = (max - min) / CONTINUOUS_STEPS;
        let to = match key {
            Walk::Right | Walk::Up if has("increment") => {
                return self.ask(seen, id, "increment", None);
            }
            Walk::Left | Walk::Down if has("decrement") => {
                return self.ask(seen, id, "decrement", None);
            }
            Walk::Right | Walk::Up => (value + nudge).min(max),
            Walk::Left | Walk::Down => (value - nudge).max(min),
            Walk::Home => min,
            Walk::End => max,
            Walk::Tab | Walk::Press => return false,
        };
        self.ask(seen, id, "set_value", Some(Value::Num(to)))
    }
}

#[cfg(test)]
mod tests;
