//! todo domain: items, filtering, counts and per-item animation progress.
//!
//! pure state, no gpu, no window, no text measurement. absorbed from
//! examples/todo_app/state.rs minus its layout heuristics: the demo
//! estimated label width as chars * 8, which is a defect class here
//! (kdb/how-to/code-against-the-plev-engine.md). any text measure the
//! ui needs comes from TextMeasurer at draw time, never from this model.

use engine::animation::{Easing, Tween};

/// Seconds for the enter fade-in of a freshly added item.
const ENTER_SECS: f32 = 0.3;
/// Seconds for the strike animation when completion flips.
const STRIKE_SECS: f32 = 0.2;

// ---------------------------------------------------------------------------
// Filter
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    All,
    Active,
    Completed,
}

impl Filter {
    /// Display order for the filter bar.
    pub const ALL: [Filter; 3] = [Filter::All, Filter::Active, Filter::Completed];

    pub fn label(self) -> &'static str {
        match self {
            Filter::All => "All",
            Filter::Active => "Active",
            Filter::Completed => "Completed",
        }
    }

    fn admits(self, completed: bool) -> bool {
        match self {
            Filter::All => true,
            Filter::Active => !completed,
            Filter::Completed => completed,
        }
    }
}

// ---------------------------------------------------------------------------
// TodoItem
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct TodoItem {
    id: u64,
    text: String,
    completed: bool,
    enter: Tween<f32>,
    strike: Tween<f32>,
}

impl TodoItem {
    fn new(id: u64, text: String) -> Self {
        let mut enter = Tween::new(0.0_f32, ENTER_SECS, Easing::EaseOutCubic);
        enter.set_target(1.0);
        Self {
            id,
            text,
            completed: false,
            enter,
            strike: Tween::new(0.0_f32, STRIKE_SECS, Easing::EaseInOut),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn completed(&self) -> bool {
        self.completed
    }

    /// Fade-in progress: 0 at insertion, 1 once settled.
    pub fn enter_progress(&self) -> f32 {
        self.enter.get()
    }

    /// Strike progress: 0 fully active, 1 fully struck. Runs both ways
    /// as completion toggles, from wherever the previous run stopped.
    pub fn strike_progress(&self) -> f32 {
        self.strike.get()
    }

    fn toggle(&mut self) {
        self.completed = !self.completed;
        self.strike
            .set_target(if self.completed { 1.0 } else { 0.0 });
    }

    fn tick(&mut self, dt: f32) {
        self.enter.tick(dt);
        self.strike.tick(dt);
    }

    fn animating(&self) -> bool {
        self.enter.is_animating() || self.strike.is_animating()
    }
}

// ---------------------------------------------------------------------------
// Counts
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub total: usize,
    pub active: usize,
    pub completed: usize,
}

// ---------------------------------------------------------------------------
// TodoModel
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct TodoModel {
    items: Vec<TodoItem>,
    filter: Filter,
    next_id: u64,
}

impl Default for TodoModel {
    fn default() -> Self {
        Self::new()
    }
}

impl TodoModel {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            filter: Filter::All,
            next_id: 1,
        }
    }

    /// Adds a trimmed item and returns its id; whitespace-only input adds
    /// nothing and returns None.
    pub fn add(&mut self, text: &str) -> Option<u64> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(TodoItem::new(id, text.to_string()));
        Some(id)
    }

    /// Flips completion and starts the strike animation. Returns false
    /// when the id does not exist.
    pub fn toggle(&mut self, id: u64) -> bool {
        match self.items.iter_mut().find(|i| i.id == id) {
            Some(item) => {
                item.toggle();
                true
            }
            None => false,
        }
    }

    /// Removes the item. Returns false when the id does not exist.
    pub fn delete(&mut self, id: u64) -> bool {
        let before = self.items.len();
        self.items.retain(|i| i.id != id);
        self.items.len() != before
    }

    /// Returns true when the visible set may have changed, so the caller
    /// knows to invalidate (render on demand).
    pub fn set_filter(&mut self, filter: Filter) -> bool {
        let changed = self.filter != filter;
        self.filter = filter;
        changed
    }

    pub fn filter(&self) -> Filter {
        self.filter
    }

    /// Items admitted by the current filter, insertion order preserved.
    pub fn visible_items(&self) -> Vec<&TodoItem> {
        self.items
            .iter()
            .filter(|i| self.filter.admits(i.completed))
            .collect()
    }

    pub fn counts(&self) -> Counts {
        let total = self.items.len();
        let completed = self.items.iter().filter(|i| i.completed).count();
        Counts {
            total,
            active: total - completed,
            completed,
        }
    }

    /// Advances every per-item tween by dt seconds. Returns true while
    /// any animation is live, including the frame on which one settles,
    /// so the ui keeps requesting frames until the final values are drawn.
    pub fn update(&mut self, dt: f32) -> bool {
        let mut animating = false;
        for item in &mut self.items {
            let was = item.animating();
            item.tick(dt);
            animating |= was || item.animating();
        }
        animating
    }
}
