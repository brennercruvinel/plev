//! The window's loop without the window: input routed the way the
//! showcase shell routes it (`showcase::app`), a frame drawn only when an
//! event changed something or the last tick was still moving (render on
//! demand), the scene built by the surface and drawn by
//! `engine::headless::Renderer`, which runs the window's frame tail.
//! Every turn is one stage frame (`engine::perf::stages`); the GPU time a
//! turn's timestamps read feeds a `PerfMonitor`, as a window's loop would.

use comps::editor::{EditorTheme, EditorView, MouseEvent};
use comps::prelude::WidgetEvent;
use engine::compositor::{Compositor, SceneNode};
use engine::headless::Renderer;
use engine::layout::ComputedBounds;
use engine::perf::stages::{self, Stage, StageFrame};
use engine::perf::{PerfMonitor, PerfSnapshot};
use engine::theme::Theme;
use showcase::view::ShowcaseView;
use winit::keyboard::{Key, ModifiersState, NamedKey};

/// The fixed step of every tick: a 60 Hz display.
pub const DT: f32 = 1.0 / 60.0;

/// What the window hands the app: pointer events and pressed keys.
#[derive(Clone, Debug)]
pub enum Input {
    Pointer(WidgetEvent),
    Char(char),
    Named(NamedKey),
}

impl Input {
    /// A left click: press and release at one point.
    pub fn click(x: f32, y: f32) -> [Input; 2] {
        [
            Input::Pointer(WidgetEvent::MouseDown { x, y }),
            Input::Pointer(WidgetEvent::MouseUp { x, y }),
        ]
    }
}

/// What the loop drives: the showcase view, or one widget on its own.
pub trait Surface {
    /// Route one input; `true` asks for a frame.
    fn input(&mut self, input: &Input) -> bool;
    /// Advance animations; `true` while something still moves.
    fn tick(&mut self, dt: f32) -> bool;
    /// Build the scene (its `begin_frame` included).
    fn render(&mut self, c: &mut Compositor);
    /// The background in linear space.
    fn clear(&self) -> [f32; 4];
}

impl Surface for ShowcaseView {
    /// The routing of `showcase::app`'s `window_event`: Escape closes the
    /// top overlay, the editor takes a key first, named keys go through
    /// `showcase::keys`, characters through `handle_key`.
    fn input(&mut self, input: &Input) -> bool {
        let none = ModifiersState::empty();
        match input {
            Input::Pointer(event) => self.handle_event(event),
            Input::Named(NamedKey::Escape) => self.close_top_overlay(),
            Input::Named(named) => {
                self.handle_editor_key(&Key::Named(*named), none)
                    || showcase::keys::handle_named(self, named)
            }
            Input::Char(c) => {
                let text = c.to_string();
                self.handle_editor_key(&Key::Character(text.as_str().into()), none)
                    || self.handle_key(&text)
            }
        }
    }

    fn tick(&mut self, dt: f32) -> bool {
        ShowcaseView::tick(self, dt)
    }

    fn render(&mut self, c: &mut Compositor) {
        ShowcaseView::render(self, c);
    }

    fn clear(&self) -> [f32; 4] {
        self.theme.colors.bg.to_linear_array()
    }
}

/// The comps editor filling the viewport, fed the way the showcase Editor
/// page feeds it (`view/apps/editor.rs`), for a document the page does not
/// open.
pub struct EditorSurface {
    pub editor: EditorView,
    theme: EditorTheme,
    bounds: ComputedBounds,
    background: [f32; 4],
    mouse_down: bool,
}

impl EditorSurface {
    pub fn new(editor: EditorView, theme: &Theme, width: f32, height: f32) -> Self {
        Self {
            editor,
            theme: EditorTheme::from_theme(theme),
            bounds: ComputedBounds {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
            background: theme.colors.bg.to_linear_array(),
            mouse_down: false,
        }
    }
}

impl Surface for EditorSurface {
    fn input(&mut self, input: &Input) -> bool {
        let none = ModifiersState::empty();
        self.editor.set_bounds(self.bounds);
        match *input {
            Input::Pointer(WidgetEvent::MouseDown { x, y }) => {
                self.mouse_down = true;
                self.editor.handle_mouse(MouseEvent::Down {
                    x,
                    y,
                    alt: false,
                    shift: false,
                })
            }
            Input::Pointer(WidgetEvent::MouseMove { x, y }) if self.mouse_down => {
                self.editor.handle_mouse(MouseEvent::Drag { x, y })
            }
            Input::Pointer(WidgetEvent::MouseUp { .. }) if self.mouse_down => {
                self.mouse_down = false;
                self.editor.handle_mouse(MouseEvent::Up)
            }
            Input::Pointer(WidgetEvent::Scroll { delta, .. }) => {
                self.editor.handle_mouse(MouseEvent::Wheel { dy: delta });
                true
            }
            Input::Pointer(_) => false,
            Input::Named(named) => self.editor.handle_key(&Key::Named(named), none),
            Input::Char(c) => {
                let text = c.to_string();
                self.editor
                    .handle_key(&Key::Character(text.as_str().into()), none)
            }
        }
    }

    fn tick(&mut self, dt: f32) -> bool {
        self.editor.tick(dt)
    }

    fn render(&mut self, c: &mut Compositor) {
        c.begin_frame();
        self.editor.render(c, self.bounds, &self.theme);
    }

    fn clear(&self) -> [f32; 4] {
        self.background
    }
}

/// Nothing at all: one empty default layer, the least a frame can draw.
pub struct EmptySurface {
    pub background: [f32; 4],
}

impl Surface for EmptySurface {
    fn input(&mut self, _input: &Input) -> bool {
        false
    }

    fn tick(&mut self, _dt: f32) -> bool {
        false
    }

    fn render(&mut self, c: &mut Compositor) {
        c.begin_frame();
    }

    fn clear(&self) -> [f32; 4] {
        self.background
    }
}

/// One turn of the loop.
pub struct Sample {
    pub stages: StageFrame,
    /// A frame was built and drawn this turn.
    pub drawn: bool,
}

/// The loop: a surface, its compositor and the headless frame tail.
pub struct Harness<S: Surface> {
    renderer: Renderer,
    compositor: Compositor,
    pub surface: S,
    /// A frame is asked for: an event changed something, or the last tick
    /// was still moving.
    pending: bool,
    /// Fed as a window's loop would feed it: the GPU time each turn's
    /// timestamps read.
    perf: PerfMonitor,
}

impl<S: Surface> Harness<S> {
    pub fn new(renderer: Renderer, surface: S) -> Self {
        Self {
            renderer,
            compositor: Compositor::new(),
            surface,
            pending: true,
            perf: PerfMonitor::new(),
        }
    }

    pub fn compositor(&self) -> &Compositor {
        &self.compositor
    }

    pub fn perf(&self) -> PerfSnapshot {
        self.perf.snapshot()
    }

    /// One turn: deliver `inputs`, then draw if a frame is asked for.
    pub fn frame(&mut self, inputs: &[Input]) -> Sample {
        self.turn(inputs, false)
    }

    /// One turn in which the platform asks for a frame with nothing
    /// changed (an expose, a resize to the same size).
    pub fn redraw(&mut self) -> Sample {
        self.turn(&[], true)
    }

    /// Turns with no input until nothing asks for a frame; the count of
    /// turns taken, at most `max`.
    pub fn settle(&mut self, max: usize) -> usize {
        for turn in 0..max {
            if !self.pending {
                return turn;
            }
            self.frame(&[]);
        }
        max
    }

    fn turn(&mut self, inputs: &[Input], forced: bool) -> Sample {
        // what setup recorded between turns is not this frame's
        let _ = stages::take();
        let frame = stages::span(Stage::Frame);
        for input in inputs {
            let _input = stages::span(Stage::Input).items(1);
            self.pending |= self.surface.input(input);
        }
        let drawn = self.pending || forced;
        if drawn {
            let animating = {
                let _tick = stages::span(Stage::Tick);
                self.surface.tick(DT)
            };
            let build = stages::span(Stage::SceneBuild);
            self.surface.render(&mut self.compositor);
            build.end(|| scene_nodes(&self.compositor));
            let clear = self.surface.clear();
            self.renderer.draw(&mut self.compositor, clear);
            self.pending = animating;
        }
        drop(frame);
        let stages = stages::take();
        if let Some(micros) = stages.gpu_micros() {
            self.perf.record_gpu_micros(micros);
        }
        Sample { stages, drawn }
    }
}

/// Scene nodes in every layer.
pub fn scene_nodes(c: &Compositor) -> u64 {
    c.layers().iter().map(|l| l.nodes().len() as u64).sum()
}

/// The center of the rect `id` is bound to this frame.
pub fn bound_center(c: &Compositor, id: &str) -> Option<(f32, f32)> {
    c.bindings()
        .find(|(bound, _)| bound.as_str() == id)
        .map(|(_, b)| center(b.rect))
}

/// The center of the first id bound this frame that starts with `prefix`.
pub fn first_bound_center(c: &Compositor, prefix: &str) -> Option<(f32, f32)> {
    c.bindings()
        .find(|(bound, _)| bound.as_str().starts_with(prefix))
        .map(|(_, b)| center(b.rect))
}

fn center(r: [f32; 4]) -> (f32, f32) {
    (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0)
}

/// Where a text run reading exactly `text` was drawn (its origin).
pub fn text_origin(c: &Compositor, text: &str) -> Option<(f32, f32)> {
    c.layers()
        .iter()
        .flat_map(|l| l.nodes())
        .find_map(|n| match n {
            SceneNode::Text { key, x, y, .. } if key.text == text => Some((*x, *y)),
            _ => None,
        })
}

/// The layer at `z_order`'s scissor in logical pixels `(x, y, w, h)`.
pub fn layer_clip(c: &Compositor, z_order: i32, scale: f32) -> Option<[f32; 4]> {
    let layer = c.layers().iter().find(|l| l.z_order == z_order)?;
    let (x, y, w, h) = layer.clip_rect?;
    Some([x, y, w, h].map(|v| v as f32 / scale))
}

/// The scene the last drawn frame left: nodes per layer (by z-order) and
/// the ids bound.
pub fn scene_line(c: &Compositor) -> String {
    let layers: Vec<String> = c
        .layers()
        .iter()
        .map(|l| format!("z {} {} nodes", l.z_order, l.nodes().len()))
        .collect();
    format!(
        "{} layers ({}), {} ids bound",
        layers.len(),
        layers.join(", "),
        c.bindings().count()
    )
}
