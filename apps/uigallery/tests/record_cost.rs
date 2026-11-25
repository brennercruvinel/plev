//! what recording costs, measured on a real scene: the showcase todo
//! page, driven by events the way a window drives it (render on demand:
//! a frame after each input and while something animates), recorded
//! frame by frame from the compositor, drawn headless through
//! engine::drawing::headless when the machine has a GPU. then the semantic track
//! under a busy toy app. a measurement, not a check: run it in release,
//!
//!     cargo test -p uigallery --release --test suite record_cost:: -- --ignored --nocapture
//!
//! the quick check that the harness itself works runs with the suite.

use std::time::{Duration, Instant};

use accord::{Actor, Dispatcher, Millis, Request, SemId};
use engine::drawing::compositor::Compositor;
use engine::drawing::gpu::RenderConfig;
use engine::drawing::headless::{Renderer, Viewport};
use engine::widgets::prelude::WidgetEvent;
use record::{InputKind, Limits, Recorder, Stats};
use uigallery::views::ShowcaseView;

// the semantic track runs under the recorder's own toy git client, the one
// its session tests drive; read from there rather than copied.
#[path = "../../../rust/record/tests/session/toys.rs"]
#[allow(dead_code)]
mod toys;

use toys::{Git, look};

/// one simulated window: the view, its compositor, the recorder, an
/// optional headless renderer, and the clock in milliseconds.
struct Window {
    view: ShowcaseView,
    c: Compositor,
    rec: Recorder,
    gpu: Option<Renderer>,
    clear: [f32; 4],
    t: Millis,
    build: Vec<Duration>,
    draw: Vec<Duration>,
    record: Vec<Duration>,
}

impl Window {
    fn new(width: f32, height: f32, limits: Limits, gpu: bool) -> Window {
        let mut view = ShowcaseView::new(width, height);
        view.jump_to_page("Todo");
        view.resize(width, height, 1.0);
        let gpu = gpu
            .then(|| Renderer::new(Viewport::new(width, height, 1.0), RenderConfig::default()).ok())
            .flatten();
        let clear = view.theme.colors.bg.to_linear_array();
        Window {
            view,
            c: Compositor::new(),
            rec: Recorder::new(limits),
            gpu,
            clear,
            t: 0,
            build: Vec::new(),
            draw: Vec::new(),
            record: Vec::new(),
        }
    }

    /// one rendered frame, `dt_ms` after the last: tick, build, draw,
    /// record. true while something still moves.
    fn step(&mut self, dt_ms: Millis) -> bool {
        self.t += dt_ms;
        let at = Instant::now();
        let moving = self.view.tick(dt_ms as f32 / 1_000.0);
        self.view.render(&mut self.c);
        self.build.push(at.elapsed());
        if let Some(gpu) = self.gpu.as_mut() {
            let at = Instant::now();
            gpu.draw(&mut self.c, self.clear);
            self.draw.push(at.elapsed());
        }
        let at = Instant::now();
        let recorded = self.rec.frame(self.t, &self.c);
        self.record.push(at.elapsed());
        assert!(recorded.is_ok(), "{recorded:?}");
        moving
    }

    /// frames at 60 per second while something moves, for at most `ms`.
    fn settle(&mut self, ms: Millis) {
        let until = self.t + ms;
        while self.step(16) && self.t < until {}
    }

    fn idle(&mut self, ms: Millis) {
        self.t += ms;
    }

    /// the rect a control is bound to in the frame on screen.
    fn rect(&self, id: &str) -> Option<[f32; 4]> {
        let id = SemId::parse(id).ok()?;
        self.c.binding(&id).map(|b| b.rect)
    }

    /// the ids bound in the frame on screen that start with `prefix`.
    fn bound(&self, prefix: &str) -> Vec<String> {
        self.c
            .bindings()
            .map(|(id, _)| id.as_str().to_owned())
            .filter(|id| id.starts_with(prefix))
            .collect()
    }

    fn pointer(&mut self, kind: InputKind, x: f32, y: f32) {
        let target = self.c.hit(x, y).cloned();
        self.rec.input(self.t, kind, Some([x, y]), target);
        let event = match kind {
            InputKind::PointerDown => WidgetEvent::MouseDown { x, y },
            InputKind::PointerUp => WidgetEvent::MouseUp { x, y },
            _ => WidgetEvent::MouseMove { x, y },
        };
        self.view.handle_event(&event);
    }

    fn click(&mut self, x: f32, y: f32) {
        self.pointer(InputKind::PointerMove, x, y);
        self.step(16);
        self.pointer(InputKind::PointerDown, x, y);
        self.pointer(InputKind::PointerUp, x, y);
        self.settle(700);
    }

    fn type_text(&mut self, text: &str) {
        for ch in text.chars() {
            self.idle(90);
            self.rec.input(self.t, InputKind::Text, None, None);
            self.view.handle_key(&ch.to_string());
            self.step(0);
        }
    }

    /// one pass of a person using the todo page.
    fn session(&mut self, pass: usize) {
        if let Some([x, y, w, h]) = self.rect("todo.input") {
            self.click(x + w / 2.0, y + h / 2.0);
        }
        self.type_text(&format!("record the session, pass {pass}"));
        self.rec.input(self.t, InputKind::Key, None, None);
        self.view.handle_enter();
        self.settle(900);
        self.view.close_top_overlay();
        self.settle(300);
        self.idle(800);
        for id in self.bound("todo.item[") {
            if let Some([x, y, w, h]) = self.rect(&id) {
                self.pointer(InputKind::PointerMove, x + w / 2.0, y + h / 2.0);
                self.step(16);
                self.idle(120);
            }
        }
        if let Some(last) = self.bound("todo.item[").last().cloned() {
            if let Some([x, y, _, h]) = self.rect(&last) {
                self.click(x + 20.0, y + h / 2.0);
            }
        }
        self.idle(600);
        for id in self.bound("todo.filter[") {
            if let Some([x, y, w, h]) = self.rect(&id) {
                self.click(x + w / 2.0, y + h / 2.0);
                self.idle(400);
            }
        }
        if let Some(first) = self.bound("todo.item[").first().cloned() {
            if let Some([x, y, w, h]) = self.rect(&first) {
                self.click(x + w - 20.0, y + h / 2.0);
            }
        }
        self.idle(1_500);
    }
}

fn micros(d: &Duration) -> f64 {
    d.as_secs_f64() * 1e6
}

/// mean, p50, p95 and max in microseconds.
fn spread(samples: &[Duration]) -> (f64, f64, f64, f64) {
    if samples.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let mut us: Vec<f64> = samples.iter().map(micros).collect();
    us.sort_by(f64::total_cmp);
    let at = |q: f64| us[((us.len() - 1) as f64 * q).round() as usize];
    let mean = us.iter().sum::<f64>() / us.len() as f64;
    (mean, at(0.5), at(0.95), at(1.0))
}

fn report(name: &str, samples: &[Duration]) {
    let (mean, p50, p95, max) = spread(samples);
    println!("  {name:<18} mean {mean:8.1} us  p50 {p50:8.1}  p95 {p95:8.1}  max {max:9.1}");
}

fn run(passes: usize, gpu: bool, theme: &str) -> (Window, Stats) {
    let mut w = Window::new(1280.0, 800.0, Limits::default(), gpu);
    w.view.apply_theme(theme);
    w.clear = w.view.theme.colors.bg.to_linear_array();
    w.settle(2_000);
    for pass in 0..passes {
        w.session(pass);
    }
    let stats = w.rec.stats();
    (w, stats)
}

#[test]
fn the_todo_page_records_through_its_compositor() -> Result<(), Box<dyn std::error::Error>> {
    let (w, stats) = run(1, false, "core");
    assert!(stats.frames > 20, "{stats:?}");
    assert_eq!(stats.frame_errors, 0);
    let session = w.rec.snapshot_window();
    let (_, end) = session.frames_span().ok_or("no frames")?;
    let scene = session.scene_at(end)?.ok_or("no scene")?;
    assert!(!scene.is_empty());
    let inputs = session
        .events()
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                record::Kind::Input {
                    target: Some(_),
                    ..
                }
            )
        })
        .count();
    assert!(inputs > 0, "the pointer landed on bound controls");
    // the page binds its controls: every frame read back says where.
    let shown = session.bindings_at(end).ok_or("no frame")?;
    assert!(!shown.is_empty(), "the todo page binds its controls");
    assert!(stats.bindings_whole > 0, "{stats:?}");
    // and a point of the replayed frame is the control the compositor's
    // hit test named in that frame.
    let mut hits = 0;
    for x in (0..1280).step_by(10) {
        for y in (0..800).step_by(10) {
            let (x, y) = (x as f32 + 0.5, y as f32 + 0.5);
            let live = w.c.hit(x, y).cloned();
            hits += usize::from(live.is_some());
            assert_eq!(session.hit_at(end, x, y), live, "at ({x}, {y})");
        }
    }
    assert!(hits > 100, "the grid lands on bound controls: {hits}");
    Ok(())
}

/// the visual track on the todo page in one theme, printed.
fn visual(theme: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (w, stats) = run(6, true, theme);
    let session = w.rec.snapshot_window();
    let (start, end) = session.frames_span().ok_or("no frames")?;
    let held_s = (end - start) as f64 / 1_000.0;
    println!(
        "\nvisual track, showcase todo page 1280x800, theme {theme}, {} ms simulated",
        w.t
    );
    println!(
        "  frames {} ({} unchanged, not stored), chunks closed {}, held {}",
        stats.frames, stats.frames_unchanged, stats.chunks_closed, stats.chunks
    );
    println!(
        "  not carried by the IR (clip push, clip pop, blur, shadow): {:?}; layers approximated {}",
        stats.skipped, stats.layers_approximated
    );
    println!(
        "  bindings recorded whole in {} frames, as moved rects in {}, unchanged in the other {}",
        stats.bindings_whole,
        stats.bindings_moved,
        stats.frames - stats.bindings_whole - stats.bindings_moved
    );
    report("build (tick+scene)", &w.build);
    if w.draw.is_empty() {
        println!("  draw               no GPU adapter: skipped");
    } else {
        report("draw (headless)", &w.draw);
    }
    report("record (frame)", &w.record);
    let (b, _, _, _) = spread(&w.build);
    let (d, _, _, _) = spread(&w.draw);
    let (r, _, _, _) = spread(&w.record);
    println!("  record / (build + draw) = {:.1} %", 100.0 * r / (b + d));
    let chunk_bytes: usize = session.chunk_sizes().iter().map(|(b, _)| b).sum();
    println!(
        "  window held {held_s:.1} s: visual {} B (encoded chunks {} B, assets {} B, the rest the open chunk)",
        stats.visual_bytes, chunk_bytes, stats.asset_bytes
    );
    println!(
        "  per second held: visual {:.0} B/s, encoded chunks alone {:.0} B/s",
        stats.visual_bytes as f64 / held_s,
        chunk_bytes as f64 / held_s
    );
    persisted(&w.rec, &format!("rec-measure-todo-{theme}.monster"))
}

/// persists a recorder's window next to the other temporary files, times
/// it, reads it back, and prints what the file holds.
fn persisted(rec: &Recorder, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join(name);
    let at = Instant::now();
    let written = rec.persist(&path)?;
    let wrote = at.elapsed();
    let at = Instant::now();
    let back = record::Session::open(&path)?;
    let read = at.elapsed();
    assert_eq!(back.events().len(), written.events);
    println!(
        "  persisted {} B in {:.1} ms (S {} B, I {} B, C {} B, A {} B, B {} B; {} events, {} chunks, {} assets, {} dropped), read back in {:.1} ms: {}",
        written.bytes,
        micros(&wrote) / 1_000.0,
        written.events_bytes,
        written.index_bytes,
        written.chunks_bytes,
        written.assets_bytes,
        written.bindings_bytes,
        written.events,
        written.chunks,
        written.assets,
        written.dropped,
        micros(&read) / 1_000.0,
        path.display()
    );
    Ok(())
}

#[test]
#[ignore = "a measurement: run in release with --ignored --nocapture"]
fn measure_the_cost_of_recording() -> Result<(), Box<dyn std::error::Error>> {
    visual("core")?;
    visual("hoff")?;

    // the semantic track under load: an app projected at 60 per second
    // while its text changes on every keystroke, one commit a second.
    let rec = Recorder::default();
    let mut dispatcher = Dispatcher::new();
    dispatcher.add_sink(Box::new(rec.sink()));
    let mut git = Git::new(&["a.rs", "b.rs", "c.rs", "d.rs"], "");
    let mut snap = Vec::new();
    for tick in 0..3_600u64 {
        let t = tick * 16;
        if tick % 6 == 0 {
            git.message.push('x');
        }
        if tick % 60 == 59 {
            let seen = look(&git);
            let req = Request::observe(&seen, Actor::Human, "commit.submit", "submit", None)?;
            rec.observed(t, Actor::Human, None, seen.revision());
            dispatcher.dispatch(&mut git, req, t);
            git.staged = ["a.rs", "b.rs"].map(String::from).into();
        }
        let projected = look(&git);
        let at = Instant::now();
        rec.snapshot(t, &projected);
        snap.push(at.elapsed());
    }
    let s = rec.stats();
    let held = rec.snapshot_window();
    let (a, b) = held.span().ok_or("no events")?;
    let secs = (b - a) as f64 / 1_000.0;
    let json = serde_json::to_vec(held.events())?.len();
    println!("\nsemantic track, toy git app, 60 projections/s for 57.6 s");
    report("snapshot()", &snap);
    println!(
        "  events {} (full {}, changes {}, unchanged {}, rekeys {})",
        s.events, s.snapshots_full, s.snapshots_changed, s.snapshots_unchanged, s.rekeys
    );
    println!(
        "  held {} events over {secs:.1} s: estimate {} B = {:.0} B/s; as json {} B = {:.0} B/s",
        held.events().len(),
        s.semantic_bytes,
        s.semantic_bytes as f64 / secs,
        json,
        json as f64 / secs
    );
    println!(
        "  inline sizes: Event {} B, Change {} B, Node {} B, Request {} B",
        std::mem::size_of::<record::Event>(),
        std::mem::size_of::<accord::Change>(),
        std::mem::size_of::<accord::Node>(),
        std::mem::size_of::<Request>()
    );
    persisted(&rec, "rec-measure-semantic.monster")
}
