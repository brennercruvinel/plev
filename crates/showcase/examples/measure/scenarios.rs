//! The interactions measured, each driven through real input on a fresh
//! page: the setup puts the page where the interaction starts, a few
//! unmeasured warm-up turns run the interaction once the caches and GPU
//! buffers have grown, then every measured turn is one sample.

use comps::editor::EditorView;
use comps::prelude::WidgetEvent;
use engine::clipboard::LocalClipboard;
use engine::gpu::RenderConfig;
use engine::headless::{Renderer, Viewport};
use engine::theme::Theme;
use rope::Document;
use showcase::view::ShowcaseView;
use winit::keyboard::NamedKey;

use crate::harness::{
    EditorSurface, EmptySurface, Harness, Input, Sample, Surface, bound_center, first_bound_center,
    layer_clip, scene_line, text_origin,
};

/// Unmeasured turns of the interaction before the samples.
pub const WARM_UP: usize = 10;

/// Turns a page gets to come to rest after it opens.
const SETTLE: usize = 600;

/// Pointer travel per turn while hovering the list (logical px).
const HOVER_STEP: f32 = 5.0;

/// Lines of the large document the editor scenario opens.
const LARGE_LINES: usize = 100_000;

/// What is typed, one key per turn (a space is the named key).
const PHRASE: &str = "plev measures what one keystroke costs ";

/// The z-order the showcase gives its virtualized list layer.
const LIST_Z: i32 = 10;

pub struct Config {
    pub viewport: Viewport,
    pub frames: usize,
    pub theme: String,
    /// The frame tail waits for the GPU after submit (`GpuWait`).
    pub gpu_sync: bool,
    /// The renderer asks for GPU timestamp queries
    /// (`RenderConfig::gpu_timestamps`): the GPU stages.
    pub gpu_timestamps: bool,
}

/// One scenario's samples and what the reader needs to read them.
pub struct Run {
    pub key: &'static str,
    pub title: &'static str,
    /// How the page was put in place and what one turn does.
    pub setup: String,
    pub samples: Vec<Sample>,
    /// The scene the last turn left.
    pub scene: String,
    pub notes: Vec<String>,
    /// What the loop's perf monitor held after the last turn: the last
    /// GPU time the timestamps fed it (`PerfSnapshot::gpu_micros`).
    pub monitor_gpu_micros: Option<u64>,
}

/// Every scenario, in the order of the report: key and title.
pub const ALL: [(&str, &str); 9] = [
    ("type", "a. type one character in the todo input"),
    ("toggle", "b. toggle one todo item"),
    ("hover", "c. hover across the rows of a list"),
    ("overlay", "d. one frame in the middle of a modal opening"),
    ("editor", "e. insert one character in a large document"),
    (
        "editor-page",
        "e'. insert one character in the showcase Editor page",
    ),
    ("idle", "f. an idle redraw with nothing changed"),
    ("caret", "g. a focused todo input, no typing"),
    ("floor", "h. the floor: an empty scene redrawn"),
];

pub fn run(key: &str, cfg: &Config) -> Result<Run, String> {
    let (key, title) = ALL
        .iter()
        .copied()
        .find(|(k, _)| *k == key)
        .ok_or_else(|| format!("no scenario '{key}'"))?;
    let mut run = match key {
        "type" => typing(cfg)?,
        "toggle" => toggle(cfg)?,
        "hover" => hover(cfg)?,
        "overlay" => overlay(cfg)?,
        "editor" => editor(cfg)?,
        "editor-page" => editor_page(cfg)?,
        "idle" => idle(cfg)?,
        "caret" => caret(cfg)?,
        _ => floor(cfg)?,
    };
    run.key = key;
    run.title = title;
    Ok(run)
}

fn renderer(cfg: &Config) -> Result<Renderer, String> {
    let config = RenderConfig {
        gpu_timestamps: cfg.gpu_timestamps,
        ..RenderConfig::default()
    };
    Renderer::new(cfg.viewport, config).map_err(|e| e.to_string())
}

/// A gallery page as the window opens it (`showcase <page> <theme>`),
/// come to rest.
fn page(cfg: &Config, title: &str) -> Result<Harness<ShowcaseView>, String> {
    let v = cfg.viewport;
    let mut view = ShowcaseView::new(v.width, v.height);
    view.jump_to_page(title);
    if !view.page_title().eq_ignore_ascii_case(title) {
        return Err(format!("no page titled '{title}'"));
    }
    view.apply_theme(&cfg.theme);
    if view.theme_name != cfg.theme {
        return Err(format!("no theme named '{}'", cfg.theme));
    }
    view.resize(v.width, v.height, v.scale);
    let mut h = Harness::new(renderer(cfg)?, view);
    h.settle(SETTLE);
    Ok(h)
}

/// `WARM_UP` unmeasured turns, then `frames` measured ones; `turn(i)` is
/// the i-th turn counting the warm-up.
fn sample<S: Surface>(
    h: &mut Harness<S>,
    frames: usize,
    mut turn: impl FnMut(&mut Harness<S>, usize) -> Sample,
) -> Vec<Sample> {
    for i in 0..WARM_UP {
        turn(h, i);
    }
    (WARM_UP..WARM_UP + frames).map(|i| turn(h, i)).collect()
}

fn finish<S: Surface>(h: &Harness<S>, setup: String, samples: Vec<Sample>) -> Run {
    Run {
        key: "",
        title: "",
        setup,
        samples,
        scene: scene_line(h.compositor()),
        notes: Vec::new(),
        monitor_gpu_micros: h.perf().gpu_micros,
    }
}

fn phrase_key(i: usize) -> Input {
    let chars: Vec<char> = PHRASE.chars().collect();
    match chars[i % chars.len()] {
        ' ' => Input::Named(NamedKey::Space),
        c => Input::Char(c),
    }
}

fn focus_todo_input(h: &mut Harness<ShowcaseView>) -> Result<(), String> {
    let (x, y) = bound_center(h.compositor(), "todo.input").ok_or("todo.input is not bound")?;
    h.frame(&Input::click(x, y));
    Ok(())
}

fn typing(cfg: &Config) -> Result<Run, String> {
    let mut h = page(cfg, "Todo")?;
    focus_todo_input(&mut h)?;
    let samples = sample(&mut h, cfg.frames, |h, i| h.frame(&[phrase_key(i)]));
    let setup = format!(
        "Todo page; a click on the center of `todo.input` focuses it. each turn is one \
         key of \"{PHRASE}\" (a space is the named key), so the field grows by one \
         character per frame."
    );
    Ok(finish(&h, setup, samples))
}

fn toggle(cfg: &Config) -> Result<Run, String> {
    let mut h = page(cfg, "Todo")?;
    let (x, y) = first_bound_center(h.compositor(), "todo.item[").ok_or("no todo item is bound")?;
    let samples = sample(&mut h, cfg.frames, |h, _| h.frame(&Input::click(x, y)));
    let setup = "Todo page with its three seed items; each turn is a click (press and \
                 release) on the center of the first bound `todo.item[..]` row, which lies \
                 on its checkbox: the item toggles through the dispatcher every frame."
        .to_string();
    Ok(finish(&h, setup, samples))
}

fn hover(cfg: &Config) -> Result<Run, String> {
    let mut h = page(cfg, "Scroll Area")?;
    let [lx, ly, lw, lh] = layer_clip(h.compositor(), LIST_Z, cfg.viewport.scale)
        .ok_or("the list layer has no clip")?;
    let x = lx + lw / 2.0;
    let (top, span) = (ly + 2.0, (lh - 4.0).max(HOVER_STEP));
    // down the list and back up, HOVER_STEP px per turn
    let at = move |i: usize| {
        let travel = i as f32 * HOVER_STEP % (2.0 * span);
        top + if travel < span {
            travel
        } else {
            2.0 * span - travel
        }
    };
    let samples = sample(&mut h, cfg.frames, |h, i| {
        h.frame(&[Input::Pointer(WidgetEvent::MouseMove { x, y: at(i) })])
    });
    let setup = format!(
        "Scroll Area page (the 10,000-row virtualized list, its rows on their own \
         layer); each turn moves the pointer {HOVER_STEP} px down the middle of the list \
         and back, no button pressed."
    );
    Ok(finish(&h, setup, samples))
}

fn overlay(cfg: &Config) -> Result<Run, String> {
    let mut h = page(cfg, "Dialog")?;
    let (tx, ty) = text_origin(h.compositor(), "Open dialog").ok_or("no 'Open dialog' label")?;
    let open = Input::click(tx + 2.0, ty + 2.0);

    // a dry run counts the turns the opening takes
    h.frame(&open);
    let opening = h.settle(SETTLE);
    if opening == 0 {
        return Err("the click did not open the dialog".to_string());
    }
    h.frame(&[Input::Named(NamedKey::Escape)]);
    h.settle(SETTLE);
    let mid = opening / 2;

    let mut scene = String::new();
    let samples = sample(&mut h, cfg.frames, |h, _| {
        h.frame(&open);
        for _ in 1..mid {
            h.frame(&[]);
        }
        let middle = h.frame(&[]);
        scene = scene_line(h.compositor());
        h.settle(SETTLE);
        h.frame(&[Input::Named(NamedKey::Escape)]);
        h.settle(SETTLE);
        middle
    });
    let mut run = finish(&h, String::new(), samples);
    run.scene = scene;
    run.setup = format!(
        "Dialog page; a click on the 'Open dialog' button opens the modal, whose spring \
         takes {opening} turns to rest. each sample is turn {mid} after the click, with no \
         input; between samples the modal settles, Escape closes it and the page \
         settles again."
    );
    Ok(run)
}

fn theme(name: &str) -> Result<Theme, String> {
    match name {
        "dark" => Ok(Theme::dark()),
        "light" => Ok(Theme::light()),
        other => Theme::named(other).ok_or_else(|| format!("no theme named '{other}'")),
    }
}

/// Code-like text, five lines a block, like the Editor page's sample.
fn large_document(lines: usize) -> String {
    let mut s = String::with_capacity(lines * 36);
    for i in 0..lines.div_ceil(5) {
        s.push_str(&format!("/// Block {i}: five lines of sample content.\n"));
        s.push_str(&format!("fn sample_{i}(input: &str) -> usize {{\n"));
        s.push_str(&format!("    let value = input.len() + {i};\n"));
        s.push_str("    value * 2\n");
        s.push_str("}\n");
    }
    s
}

fn editor(cfg: &Config) -> Result<Run, String> {
    let v = cfg.viewport;
    let text = large_document(LARGE_LINES);
    let bytes = text.len();
    let editor =
        EditorView::new(Document::load(&text)).with_clipboard(Box::new(LocalClipboard::new()));
    let surface = EditorSurface::new(editor, &theme(&cfg.theme)?, v.width, v.height);
    let mut h = Harness::new(renderer(cfg)?, surface);
    h.settle(SETTLE);
    // the wheel to the middle of the document, a click on a line there
    let (x, y) = (v.width / 2.0, v.height / 2.0);
    let middle = h.surface.editor.scroll.max_offset() / 2.0;
    h.frame(&[Input::Pointer(WidgetEvent::Scroll {
        x,
        y,
        delta: middle,
    })]);
    h.frame(&Input::click(x, y));
    let line = h.surface.editor.visible_lines().start;
    let samples = sample(&mut h, cfg.frames, |h, i| h.frame(&[phrase_key(i)]));
    let setup = format!(
        "comps::editor::EditorView, the widget the Editor page embeds, filling the \
         viewport on its own (no gallery chrome) with a {LARGE_LINES}-line document \
         ({:.1} MB); the wheel scrolls to the middle and a click puts the caret near \
         line {line}. each turn is one key of \"{PHRASE}\".",
        bytes as f64 / 1e6
    );
    Ok(finish(&h, setup, samples))
}

fn editor_page(cfg: &Config) -> Result<Run, String> {
    let mut h = page(cfg, "Editor")?;
    let samples = sample(&mut h, cfg.frames, |h, i| h.frame(&[phrase_key(i)]));
    let setup = format!(
        "the gallery's Editor page with its own sample (about 200 lines), inside the \
         chrome; each turn is one key of \"{PHRASE}\" at the caret (the start of the \
         document)."
    );
    Ok(finish(&h, setup, samples))
}

fn idle(cfg: &Config) -> Result<Run, String> {
    let mut h = page(cfg, "Todo")?;
    let left_alone = (0..cfg.frames).filter(|_| h.frame(&[]).drawn).count();
    let samples = sample(&mut h, cfg.frames, |h, _| h.redraw());
    let mut run = finish(
        &h,
        "Todo page at rest, nothing focused; each turn the platform asks for a frame \
         with nothing changed (an expose), so the whole frame runs on an unchanged scene."
            .to_string(),
        samples,
    );
    run.notes.push(format!(
        "left alone for {} turns, the loop drew {left_alone} frames: with nothing asked, \
         render on demand runs no stage at all.",
        cfg.frames
    ));
    Ok(run)
}

fn caret(cfg: &Config) -> Result<Run, String> {
    let mut h = page(cfg, "Todo")?;
    focus_todo_input(&mut h)?;
    let samples = sample(&mut h, cfg.frames, |h, _| h.frame(&[]));
    let setup = "Todo page with `todo.input` focused by a click; no input afterwards. \
                 the todo page asks for frames while its field is focused (the caret \
                 blinks), so every turn draws."
        .to_string();
    Ok(finish(&h, setup, samples))
}

fn floor(cfg: &Config) -> Result<Run, String> {
    let background = theme(&cfg.theme)?.colors.bg.to_linear_array();
    let mut h = Harness::new(renderer(cfg)?, EmptySurface { background });
    h.settle(SETTLE);
    let samples = sample(&mut h, cfg.frames, |h, _| h.redraw());
    let setup = "no page: a compositor with only its empty default layer, redrawn every \
                 turn. what is left is the frame tail's fixed cost, the clear and the \
                 composite of one layer, and on the gpu the latency of a submit."
        .to_string();
    Ok(finish(&h, setup, samples))
}
