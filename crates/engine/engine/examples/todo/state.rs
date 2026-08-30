//! Domain state, models, and logic for the Todo App example.

use engine::animation::{Easing, Tween};
use engine::text_input::TextInput;

// ---------------------------------------------------------------------------
// Palette (shared across examples)
// ---------------------------------------------------------------------------

pub(crate) const BG: [f32; 4] = [0.06, 0.06, 0.10, 1.0];
pub(crate) const HEADER_BG: [f32; 4] = [0.08, 0.08, 0.14, 1.0];
pub(crate) const SURFACE: [f32; 4] = [0.10, 0.10, 0.16, 1.0];
pub(crate) const ACCENT: [f32; 4] = [0.30, 0.55, 1.0, 1.0];
pub(crate) const TEXT: [f32; 4] = [0.93, 0.93, 0.96, 1.0];
pub(crate) const TEXT_DIM: [f32; 4] = [0.55, 0.55, 0.65, 1.0];
pub(crate) const DIVIDER: [f32; 4] = [0.18, 0.18, 0.25, 1.0];

// App-specific colors
pub(crate) const SURFACE_HOVER: [f32; 4] = [0.13, 0.13, 0.20, 1.0];
pub(crate) const TEXT_COMPLETED: [f32; 4] = [0.40, 0.40, 0.50, 0.6];
pub(crate) const RED: [f32; 4] = [1.0, 0.30, 0.25, 1.0];
pub(crate) const RED_HOVER: [f32; 4] = [1.0, 0.35, 0.30, 1.0];
pub(crate) const FILTER_ACTIVE_BG: [f32; 4] = [0.20, 0.35, 0.60, 0.5];
pub(crate) const CHECKBOX_BORDER: [f32; 4] = [0.35, 0.35, 0.45, 1.0];
pub(crate) const CHECKBOX_FILL: [f32; 4] = [0.25, 0.70, 0.40, 1.0];

// ---------------------------------------------------------------------------
// Layout constants
// ---------------------------------------------------------------------------

pub(crate) const MARGIN: f32 = 32.0;
pub(crate) const HEADER_H: f32 = 70.0;
pub(crate) const INPUT_H: f32 = 44.0;
pub(crate) const ITEM_H: f32 = 48.0;
pub(crate) const ITEM_GAP: f32 = 2.0;
pub(crate) const CHECKBOX_SIZE: f32 = 20.0;
pub(crate) const DELETE_SIZE: f32 = 24.0;

// ---------------------------------------------------------------------------
// Filter
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Filter {
    All,
    Active,
    Completed,
}

impl Filter {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Filter::All => "All",
            Filter::Active => "Active",
            Filter::Completed => "Completed",
        }
    }
}

// ---------------------------------------------------------------------------
// TodoItem
// ---------------------------------------------------------------------------

pub(crate) struct TodoItem {
    pub(crate) id: u64,
    pub(crate) text: String,
    pub(crate) completed: bool,
    pub(crate) opacity: Tween<f32>,
    pub(crate) complete_opacity: Tween<f32>,
}

impl TodoItem {
    pub(crate) fn new(id: u64, text: String) -> Self {
        let mut opacity = Tween::new(0.0_f32, 0.3, Easing::EaseOutCubic);
        opacity.set_target(1.0);
        Self {
            id,
            text,
            completed: false,
            opacity,
            complete_opacity: Tween::new(1.0_f32, 0.2, Easing::EaseInOut),
        }
    }

    pub(crate) fn toggle(&mut self) {
        self.completed = !self.completed;
        if self.completed {
            self.complete_opacity.set_target(0.6);
        } else {
            self.complete_opacity.set_target(1.0);
        }
    }

    pub(crate) fn tick(&mut self, dt: f32) {
        self.opacity.tick(dt);
        self.complete_opacity.tick(dt);
    }

    pub(crate) fn effective_opacity(&self) -> f32 {
        self.opacity.get() * self.complete_opacity.get()
    }
}

// ---------------------------------------------------------------------------
// GPU state enum
// ---------------------------------------------------------------------------

#[allow(clippy::large_enum_variant)]
pub(crate) enum GpuState {
    Uninitialized,
    Ready {
        gpu: engine::gpu::GpuContext,
        text_system: engine::text::TextSystem,
        pool: engine::gpu::texture_pool::TexturePool,
    },
}

// ---------------------------------------------------------------------------
// TodoApp state
// ---------------------------------------------------------------------------

pub(crate) struct TodoApp {
    pub(crate) window: Option<std::sync::Arc<engine::winit::window::Window>>,
    pub(crate) state: GpuState,
    pub(crate) compositor: engine::compositor::Compositor,
    pub(crate) clock: engine::animation::FrameClock,
    pub(crate) items: Vec<TodoItem>,
    pub(crate) input: TextInput,
    pub(crate) filter: Filter,
    pub(crate) next_id: u64,
    pub(crate) cursor_pos: (f32, f32),
    pub(crate) hover_item_id: Option<u64>,
    pub(crate) hover_delete_id: Option<u64>,
}

impl TodoApp {
    pub(crate) fn new() -> Self {
        let mut input = TextInput::new()
            .with_placeholder("What needs to be done?")
            .with_font_size(16.0);
        input.focus();

        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor: engine::compositor::Compositor::new(),
            clock: engine::animation::FrameClock::new(),
            items: Vec::new(),
            input,
            filter: Filter::All,
            next_id: 1,
            cursor_pos: (0.0, 0.0),
            hover_item_id: None,
            hover_delete_id: None,
        }
    }

    pub(crate) fn add_todo(&mut self) {
        let text = self.input.buffer.text().trim().to_string();
        if text.is_empty() {
            return;
        }
        let item = TodoItem::new(self.next_id, text);
        self.next_id += 1;
        self.items.push(item);
        self.input.buffer.set_text("");
        self.input.reset_blink();
    }

    pub(crate) fn toggle_todo(&mut self, id: u64) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.toggle();
        }
    }

    pub(crate) fn remove_todo(&mut self, id: u64) {
        self.items.retain(|i| i.id != id);
    }

    pub(crate) fn visible_items(&self) -> Vec<&TodoItem> {
        self.items
            .iter()
            .filter(|i| match self.filter {
                Filter::All => true,
                Filter::Active => !i.completed,
                Filter::Completed => i.completed,
            })
            .collect()
    }

    pub(crate) fn active_count(&self) -> usize {
        self.items.iter().filter(|i| !i.completed).count()
    }

    pub(crate) fn content_width(&self) -> f32 {
        match &self.state {
            GpuState::Ready { gpu, .. } => {
                (gpu.surface_config.width as f32 - MARGIN * 2.0).min(600.0)
            }
            _ => 500.0,
        }
    }

    pub(crate) fn content_x(&self) -> f32 {
        match &self.state {
            GpuState::Ready { gpu, .. } => {
                let vw = gpu.surface_config.width as f32;
                let cw = self.content_width();
                (vw - cw) / 2.0
            }
            _ => MARGIN,
        }
    }

    pub(crate) fn input_rect(&self) -> (f32, f32, f32, f32) {
        let cx = self.content_x();
        let cw = self.content_width();
        (cx, HEADER_H + 16.0, cw, INPUT_H)
    }

    pub(crate) fn item_rect(&self, visible_idx: usize) -> (f32, f32, f32, f32) {
        let cx = self.content_x();
        let cw = self.content_width();
        let (_, iy, _, ih) = self.input_rect();
        let list_start = iy + ih + 12.0;
        let y = list_start + visible_idx as f32 * (ITEM_H + ITEM_GAP);
        (cx, y, cw, ITEM_H)
    }

    pub(crate) fn checkbox_rect(&self, visible_idx: usize) -> (f32, f32, f32, f32) {
        let (ix, iy, _, _) = self.item_rect(visible_idx);
        let cy = iy + (ITEM_H - CHECKBOX_SIZE) / 2.0;
        (ix + 12.0, cy, CHECKBOX_SIZE, CHECKBOX_SIZE)
    }

    pub(crate) fn delete_rect(&self, visible_idx: usize) -> (f32, f32, f32, f32) {
        let (ix, iy, iw, _) = self.item_rect(visible_idx);
        let dy = iy + (ITEM_H - DELETE_SIZE) / 2.0;
        (ix + iw - DELETE_SIZE - 12.0, dy, DELETE_SIZE, DELETE_SIZE)
    }

    pub(crate) fn footer_y(&self) -> f32 {
        let visible_count = self.visible_items().len();
        let (_, iy, _, ih) = self.input_rect();
        let list_start = iy + ih + 12.0;
        list_start + visible_count as f32 * (ITEM_H + ITEM_GAP) + 16.0
    }

    pub(crate) fn filter_rects(&self) -> Vec<(Filter, f32, f32, f32, f32)> {
        let cx = self.content_x();
        let fy = self.footer_y() + 28.0;
        let filters = [Filter::All, Filter::Active, Filter::Completed];
        let mut rects = Vec::new();
        let mut fx = cx;
        for f in &filters {
            let label = f.label();
            let w = label.len() as f32 * 8.0 + 20.0;
            rects.push((*f, fx, fy, w, 28.0));
            fx += w + 8.0;
        }
        rects
    }
}
