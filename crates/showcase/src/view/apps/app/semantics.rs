//! The todo page as a capability: what it means (`sem::Semantics`), what
//! it runs (`sem::Operable`) and the one path every actor takes to run it.
//!
//! A click, the Enter key, an agent, assistive technology and a test all
//! become a `sem::Request` over the page as it is at that instant and go
//! through the page's `Dispatcher`; the model changes only inside
//! `execute`. The page binds the rects it draws and hit-tests with to the
//! same ids (`draw.rs`), beside the scene.
//!
//! The semantic model: `todo` (region, inert while the view holds an
//! overlay up), `todo.input` (text input: `text`, `set_value` and `submit`
//! reading `todo.input:text`), `todo.filter` (tab list: `selected`) with
//! one tab per filter (`select` reading `todo.filter:selected`),
//! `todo.item[<id>]` for each visible item
//! (list item: `completed`, `toggle` reading it, `remove`) and
//! `todo.counter` (status: `active`, `done`).

use comps::prelude::{EventResult, Rect};
use engine::perf::stages::{self, Stage};
use sem::{
    Actor, Execution, Operable, Outcome, Policy, Request, RequestId, Role, SemError, SemErrors,
    SemId, Semantics, Snapshot, SnapshotBuilder, Value,
};
use showcase::model::todo::Filter;

use super::TodoPage;
use super::layout::ROW_H;

/// The ids the page projects and binds, parsed once.
pub(super) struct Ids {
    pub todo: SemId,
    pub input: SemId,
    pub filter: SemId,
    /// One tab per filter, in `Filter::ALL` order.
    pub tabs: [SemId; 3],
    /// `todo.item`, keyed by each item's id.
    item: SemId,
    pub counter: SemId,
}

impl Ids {
    pub(super) fn new() -> Result<Self, SemError> {
        let todo = SemId::name("todo")?;
        let filter = todo.child("filter")?;
        Ok(Self {
            input: todo.child("input")?,
            tabs: Filter::ALL.map(|f| filter.key(filter_key(f))),
            item: todo.child("item")?,
            counter: todo.child("counter")?,
            filter,
            todo,
        })
    }

    /// `todo.item[<id>]`.
    pub(super) fn item(&self, id: u64) -> SemId {
        self.item.key(id.to_string())
    }

    /// The item id behind a `todo.item[<id>]`.
    fn item_of(&self, target: &SemId) -> Option<u64> {
        target
            .as_str()
            .strip_prefix(self.item.as_str())?
            .strip_prefix('[')?
            .strip_suffix(']')?
            .parse()
            .ok()
    }

    fn target(&self, target: Target) -> SemId {
        match target {
            Target::Input => self.input.clone(),
            Target::Tab(filter) => self.tab(filter).clone(),
            Target::Item(id) => self.item(id),
        }
    }

    /// `todo.filter[<filter>]`.
    pub(super) fn tab(&self, filter: Filter) -> &SemId {
        let [all, active, completed] = &self.tabs;
        match filter {
            Filter::All => all,
            Filter::Active => active,
            Filter::Completed => completed,
        }
    }

    /// What a request accepted against the page asks it to do.
    fn op(&self, req: &Request) -> Option<Op> {
        let item = || self.item_of(&req.target);
        match req.action.as_str() {
            "set_value" if req.target == self.input => match &req.param {
                Some(Value::Text(text)) => Some(Op::SetText(text.clone())),
                _ => None,
            },
            "submit" if req.target == self.input => Some(Op::Submit),
            "select" => self
                .tabs
                .iter()
                .zip(Filter::ALL)
                .find(|(tab, _)| **tab == req.target)
                .map(|(_, f)| Op::Select(f)),
            "toggle" => item().map(Op::Toggle),
            "remove" => item().map(Op::Remove),
            _ => None,
        }
    }
}

/// The name of a filter in its tab id: `todo.filter[active]`.
fn filter_key(f: Filter) -> &'static str {
    match f {
        Filter::All => "all",
        Filter::Active => "active",
        Filter::Completed => "completed",
    }
}

/// What the page's own input asks for, named without the ids.
#[derive(Clone, Copy, Debug)]
pub(super) enum Target {
    Input,
    Tab(Filter),
    Item(u64),
}

/// The one handler of each capability.
enum Op {
    SetText(String),
    Submit,
    Select(Filter),
    Toggle(u64),
    Remove(u64),
}

/// `(x, y, w, h)`, the form `Compositor::bind` takes.
pub(super) fn bounds(r: Rect) -> [f32; 4] {
    [r.x, r.y, r.w, r.h]
}

fn count(n: usize) -> Value {
    Value::Int(i64::try_from(n).unwrap_or(i64::MAX))
}

impl Semantics for TodoPage {
    fn project(&self, b: &mut SnapshotBuilder) {
        let Some(ids) = &self.ids else {
            return;
        };
        b.node(&ids.todo, Role::Region, "todo").inert(self.inert);
        self.input
            .project(b, &ids.input, "new todo", Some(&ids.todo))
            .action("submit")
            .enabled(!self.input.text().trim().is_empty())
            .reads(format!("{}:text", ids.input));
        b.node(&ids.filter, Role::TabList, "filter")
            .parent(&ids.todo)
            .state("selected", filter_key(self.model.filter()));
        let selected = format!("{}:selected", ids.filter);
        for (tab, f) in ids.tabs.iter().zip(Filter::ALL) {
            b.node(tab, Role::Tab, f.label())
                .parent(&ids.filter)
                .action("select")
                .reads(selected.as_str());
        }
        for item in self.model.visible_items() {
            let id = ids.item(item.id());
            let mut node = b.node(&id, Role::ListItem, item.text());
            node.parent(&ids.todo).state("completed", item.completed());
            node.action("toggle").reads(format!("{id}:completed"));
            node.action("remove").policy(Policy {
                destructive: true,
                ..Policy::default()
            });
        }
        let counts = self.model.counts();
        b.node(&ids.counter, Role::Status, self.counter_text())
            .parent(&ids.todo)
            .state("active", count(counts.active))
            .state("done", count(counts.completed));
        if self.input.is_focused() {
            b.focus(&ids.input);
        }
    }
}

impl Operable for TodoPage {
    /// Everything here ends now, so the request id is not kept.
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        let Some(op) = self.ids.as_ref().and_then(|ids| ids.op(req)) else {
            return Execution::Failed(format!(
                "the todo page has no {} on {}",
                req.action, req.target
            ));
        };
        let mut outcome = Outcome::default();
        match op {
            Op::SetText(text) => self.input.set_text(&text),
            Op::Submit => {
                let Some(id) = self.model.add(self.input.text()) else {
                    return Execution::Failed("the text is blank".to_owned());
                };
                self.input.set_text("");
                self.input.input.reset_blink();
                self.scroll
                    .set_content(self.model.visible_items().len() as f32 * ROW_H);
                self.scroll.scroll_to(f32::MAX);
                if let Some(ids) = &self.ids {
                    let item = Value::Text(ids.item(id).to_string());
                    outcome.result.insert("item".to_owned(), item);
                }
            }
            Op::Select(filter) => {
                if self.model.set_filter(filter) {
                    self.scroll.scroll_to(0.0);
                }
            }
            Op::Toggle(id) => {
                if !self.model.toggle(id) {
                    return Execution::Failed(format!("no item {id}"));
                }
            }
            Op::Remove(id) => {
                if !self.model.delete(id) {
                    return Execution::Failed(format!("no item {id}"));
                }
            }
        }
        Execution::Done(outcome)
    }
}

impl TodoPage {
    /// The page projected now.
    pub(crate) fn snapshot(&self) -> Result<Snapshot, SemErrors> {
        let _probe = stages::span(Stage::Projection);
        let mut b = SnapshotBuilder::new();
        self.project(&mut b);
        b.finish()
    }

    /// Runs `req` through the page's dispatcher at the page clock's now.
    pub(crate) fn dispatch(&mut self, req: Request) -> RequestId {
        let now = u64::try_from(self.clock.elapsed().as_millis()).unwrap_or(u64::MAX);
        // `execute` needs the page itself, so the dispatcher is lent out
        // for the call (taking it leaves an empty one behind, no
        // allocation) and comes back with the receipt it kept.
        let mut dispatcher = std::mem::take(&mut self.dispatcher);
        let id = dispatcher.dispatch(self, req, now);
        self.dispatcher = dispatcher;
        id
    }

    /// The receipt the page's dispatcher keeps for `id` (the shell logs
    /// what a screen reader's request came to).
    pub(crate) fn receipt(&self, id: RequestId) -> Option<&sem::Receipt> {
        self.dispatcher.receipt(id)
    }

    /// Keyboard focus asked from outside the page's pointer handling: a
    /// screen reader's focus, or a Tab resolved in the semantic order.
    /// Focus is editing, not a capability, so it does not go through the
    /// dispatcher, the same as a click into the field. The field takes it;
    /// another node of the page takes it from the field (the pills and
    /// the rows hold no focus of their own). `false` for an id the page
    /// does not project, and while inert.
    pub(crate) fn focus(&mut self, id: &SemId) -> bool {
        let Some(ids) = &self.ids else {
            return false;
        };
        if self.inert {
            return false;
        }
        if *id == ids.input {
            self.input.focus();
            return true;
        }
        let ours = *id == ids.todo
            || *id == ids.filter
            || *id == ids.counter
            || ids.tabs.contains(id)
            || ids.item_of(id).is_some();
        if ours {
            self.input.unfocus();
        }
        ours
    }

    /// A person's click or key as a request: observe the page as it is
    /// now, ask through the dispatcher, and ask for a frame when the
    /// projected state moved. A refused request (inert, disabled) is
    /// still handled: the person aimed at the page.
    pub(super) fn ask(
        &mut self,
        target: Target,
        action: &str,
        param: Option<Value>,
    ) -> EventResult {
        let Some(target) = self.ids.as_ref().map(|ids| ids.target(target)) else {
            return EventResult::IGNORED;
        };
        let seen = match self.snapshot() {
            Ok(seen) => seen,
            Err(errors) => {
                log::warn!("todo: the projection does not validate: {errors}");
                return EventResult::IGNORED;
            }
        };
        let req = match Request::observe(&seen, Actor::Human, &target, action, param) {
            Ok(req) => req,
            Err(e) => {
                log::warn!("todo: {e}");
                return EventResult::IGNORED;
            }
        };
        let id = self.dispatch(req);
        log::debug!("todo: {target} {action}: {:?}", self.dispatcher.receipt(id));
        let moved = self
            .snapshot()
            .map_or(true, |now| now.revision() != seen.revision());
        if moved {
            EventResult::clicked()
        } else {
            EventResult {
                handled: true,
                ..EventResult::IGNORED
            }
        }
    }
}

/// The page through a screen reader (task-60): the tree the showcase
/// shows, and AccessKit requests reaching the page's one dispatcher.
#[cfg(test)]
mod assistive;
