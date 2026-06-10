//! Game state, logic, and constants for the Snake example.

use std::collections::VecDeque;
use web_time::Instant;

// ── Colors ──────────────────────────────────────────────────────────────

pub(crate) const BG: [f32; 4] = [0.04, 0.04, 0.08, 1.0];
pub(crate) const GRID_BG: [f32; 4] = [0.06, 0.06, 0.10, 1.0];
pub(crate) const GRID_LINE: [f32; 4] = [0.08, 0.08, 0.14, 0.4];
pub(crate) const SNAKE_HEAD: [f32; 4] = [0.25, 0.95, 0.85, 1.0];
pub(crate) const SNAKE_BODY: [f32; 4] = [0.20, 0.75, 0.40, 1.0];
pub(crate) const SNAKE_TAIL: [f32; 4] = [0.10, 0.40, 0.22, 1.0];
pub(crate) const FOOD_COLOR: [f32; 4] = [1.0, 0.85, 0.15, 1.0];
pub(crate) const FOOD_GLOW: [f32; 4] = [1.0, 0.85, 0.15, 0.15];
pub(crate) const WALL_COLOR: [f32; 4] = [0.80, 0.15, 0.15, 1.0];
pub(crate) const TEXT_COLOR: [f32; 4] = [0.93, 0.93, 0.96, 1.0];
pub(crate) const TEXT_DIM: [f32; 4] = [0.50, 0.50, 0.60, 1.0];
pub(crate) const ACCENT: [f32; 4] = [0.30, 0.55, 1.0, 1.0];
pub(crate) const DIVIDER: [f32; 4] = [0.15, 0.15, 0.22, 1.0];

// ── Grid & timing constants ─────────────────────────────────────────────

pub(crate) const GRID_W: usize = 32;
pub(crate) const GRID_H: usize = 24;
pub(crate) const TICK_INTERVAL: f64 = 0.07;
pub(crate) const FLASH_DURATION: f32 = 0.5;
pub(crate) const HEADER_H: f32 = 52.0;
pub(crate) const FOOTER_H: f32 = 28.0;
pub(crate) const GRID_PAD: f32 = 16.0;

// ── Cell type ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Cell {
    Empty,
    Snake,
    Head,
    Food,
    Wall,
}

// ── Utility ─────────────────────────────────────────────────────────────

pub(crate) fn lerp_color(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

// ── SnakeGame ───────────────────────────────────────────────────────────

pub(crate) struct SnakeGame {
    pub(crate) field: Vec<Cell>,
    pub(crate) body: VecDeque<(usize, usize)>,
    pub(crate) head: (usize, usize),
    pub(crate) dir: (isize, isize),
    pub(crate) score: u32,
    pub(crate) high_score: u32,
    pub(crate) game_over: bool,
    pub(crate) ai_mode: bool,
    pub(crate) last_tick: Instant,
    pub(crate) last_food_eaten: Instant,
    pub(crate) food_flash: f32,
    pub(crate) rng: u64,
    pub(crate) frame: u64,
    pub(crate) fps_time: Instant,
    pub(crate) fps_count: u32,
    pub(crate) fps_display: f32,
}

impl SnakeGame {
    pub(crate) fn new() -> Self {
        let mut game = Self {
            field: vec![Cell::Empty; GRID_W * GRID_H],
            body: VecDeque::new(),
            head: (GRID_W / 2, GRID_H / 2),
            dir: (1, 0),
            score: 0,
            high_score: 0,
            game_over: false,
            ai_mode: true,
            last_tick: Instant::now(),
            last_food_eaten: Instant::now(),
            food_flash: 0.0,
            rng: 42,
            frame: 0,
            fps_time: Instant::now(),
            fps_count: 0,
            fps_display: 0.0,
        };
        game.restart();
        game
    }

    pub(crate) fn restart(&mut self) {
        self.field.fill(Cell::Empty);
        self.body.clear();
        self.head = (GRID_W / 2, GRID_H / 2);
        self.body.push_front(self.head);
        self.set_cell(self.head, Cell::Head);
        self.dir = (1, 0);
        self.score = 0;
        self.game_over = false;
        self.food_flash = 0.0;
        self.last_tick = Instant::now();
        self.place_food();
    }

    pub(crate) fn idx(&self, pos: (usize, usize)) -> usize {
        pos.1 * GRID_W + pos.0
    }

    pub(crate) fn set_cell(&mut self, pos: (usize, usize), cell: Cell) {
        let i = self.idx(pos);
        self.field[i] = cell;
    }

    pub(crate) fn cell_at(&self, pos: (usize, usize)) -> Cell {
        self.field[self.idx(pos)]
    }

    fn rng_next(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0xdeadbeefdeadbeef);
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn place_food(&mut self) {
        for _ in 0..(GRID_W * GRID_H) {
            let r = self.rng_next();
            let x = (r % GRID_W as u64) as usize;
            let y = ((r / GRID_W as u64) % GRID_H as u64) as usize;
            if self.cell_at((x, y)) == Cell::Empty {
                self.set_cell((x, y), Cell::Food);
                return;
            }
        }
    }

    fn find_food(&self) -> Option<(usize, usize)> {
        self.field.iter().position(|c| *c == Cell::Food)
            .map(|i| (i % GRID_W, i / GRID_W))
    }

    fn ai_decide(&mut self) {
        let Some(food) = self.find_food() else { return };
        let dirs: [(isize, isize); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        let mut best: Option<((isize, isize), usize)> = None;

        for d in dirs {
            if d.0 == -self.dir.0 && d.1 == -self.dir.1 && self.body.len() > 1 {
                continue;
            }
            let nx = ((self.head.0 as isize + d.0 + GRID_W as isize) as usize) % GRID_W;
            let ny = ((self.head.1 as isize + d.1 + GRID_H as isize) as usize) % GRID_H;
            match self.cell_at((nx, ny)) {
                Cell::Empty | Cell::Food => {
                    let dist = (nx as isize - food.0 as isize).unsigned_abs()
                        + (ny as isize - food.1 as isize).unsigned_abs();
                    if best.is_none() || dist < best.unwrap().1 {
                        best = Some((d, dist));
                    }
                }
                _ => {}
            }
        }
        if let Some((d, _)) = best {
            self.dir = d;
        }
    }

    pub(crate) fn tick(&mut self) {
        if self.game_over { return; }
        if self.ai_mode { self.ai_decide(); }

        let nx = ((self.head.0 as isize + self.dir.0 + GRID_W as isize) as usize) % GRID_W;
        let ny = ((self.head.1 as isize + self.dir.1 + GRID_H as isize) as usize) % GRID_H;

        let mut ate = false;
        match self.cell_at((nx, ny)) {
            Cell::Wall | Cell::Snake => {
                self.game_over = true;
                if self.score > self.high_score { self.high_score = self.score; }
                return;
            }
            Cell::Food => {
                ate = true;
                self.score += 1;
                self.food_flash = 1.0;
                self.last_food_eaten = Instant::now();
                self.place_food();
            }
            _ => {}
        }

        self.set_cell(self.head, Cell::Snake);
        self.head = (nx, ny);
        self.body.push_front(self.head);
        self.set_cell(self.head, Cell::Head);

        if !ate {
            if let Some(tail) = self.body.pop_back() {
                if tail != self.head { self.set_cell(tail, Cell::Empty); }
            }
        }
    }

    pub(crate) fn update_flash(&mut self) {
        if self.food_flash > 0.0 {
            let elapsed = Instant::now().duration_since(self.last_food_eaten).as_secs_f32();
            if elapsed < FLASH_DURATION {
                self.food_flash = (1.0 - elapsed / FLASH_DURATION).powi(2);
            } else {
                self.food_flash = 0.0;
            }
        }
    }

    pub(crate) fn try_set_dir(&mut self, d: (isize, isize)) {
        if self.ai_mode { return; }
        if d.0 == -self.dir.0 && d.1 == -self.dir.1 && self.body.len() > 1 { return; }
        self.dir = d;
    }
}
