//! plev ide - plev-native git client.
//!
//! GPU-native 3-panel workspace UI (no Tauri, no WebView, no JavaScript)
//! showing a real repository: status, log, branches and diffs come from
//! `git` running on a worker thread; the UI thread never blocks.
//!
//! The window holds one `Workbench`: every click and key that acts on the
//! repository or the commit form becomes a request through its
//! dispatcher, the same path an agent or a test takes.
//!
//! Frames render on demand, paced as the showcase paces them: the
//! workbench's tick answers a [`Wake`], motion (a scrollbar fade, a toast
//! springing in) draws every turn, and a caret or a toast at rest sleeps
//! the loop until its deadline (`ControlFlow::WaitUntil`), or for good
//! when nothing is pending. Input, the git worker and the file watcher end
//! the wait early, as winit documents.
//!
//! Run: `cargo run -p ide [path-to-repo]` (defaults to the cwd).

mod actions;
mod renderer;
mod watcher;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use comps::prelude::Wake;
use engine::actions::{
    Action, ActionRegistry, ContextStack, KeyContext, KeymapMatcher, MatchResult,
    keystroke_from_key_event,
};
use engine::compositor::Compositor;
use engine::gpu::GpuContext;
use engine::gpu::texture_pool::TexturePool;
use engine::text::TextSystem;
use git::{GitClient, GitEvent};
use ide::views::workspace::Side;
use ide::workbench::Workbench;
use winit::application::ApplicationHandler;
use winit::event::{
    ElementState, KeyEvent, MouseButton, MouseScrollDelta, StartCause, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, ModifiersState};
use winit::window::{CursorIcon, Window, WindowAttributes, WindowId};

/// Events injected into the winit loop from background threads.
enum AppEvent {
    Git(GitEvent),
    /// The worktree or `.git` changed on disk (debounced).
    FsChange,
}

// ---------------------------------------------------------------------------
// GPU state
// ---------------------------------------------------------------------------

// Ready is ~2464 B vs 0 for Uninitialized; one instance lives for the
// whole process, so boxing would only add indirection on the render path.
#[allow(clippy::large_enum_variant)]
enum GpuState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: engine::effects::EffectProcessor,
        texture_pool: TexturePool,
    },
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

struct App {
    window: Option<Arc<Window>>,
    state: GpuState,
    compositor: Compositor,
    /// The workspace, the git session and the dispatcher.
    bench: Workbench,
    cursor_pos: (f32, f32),
    scale_factor: f64,

    /// Keeps the OS file watcher alive (`None` when watching failed).
    _fs_watcher: Option<watcher::FsWatcher>,

    // Keymap dispatch (Zed model): keystroke -> matcher -> name -> registry.
    modifiers: ModifiersState,
    matcher: KeymapMatcher,
    registry: ActionRegistry,

    /// Frame clock for the widgets that animate (scrollbar fades, the
    /// commit field caret, the toasts). Ticks only while a frame is being
    /// drawn; its `since_last` is the real time a deadline slept through.
    clock: engine::animation::FrameClock,
}

impl App {
    fn new(bench: Workbench, fs_watcher: Option<watcher::FsWatcher>) -> Self {
        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor: Compositor::new(),
            bench,
            cursor_pos: (0.0, 0.0),
            scale_factor: 1.0,
            _fs_watcher: fs_watcher,
            modifiers: ModifiersState::empty(),
            matcher: KeymapMatcher::new(actions::default_keymap()),
            registry: actions::registry(),
            clock: engine::animation::FrameClock::new(),
        }
    }

    /// Mark the scene as changed and schedule a frame. Frames are rendered
    /// on demand only: without input the event loop stays idle (no
    /// unconditional `request_redraw`).
    fn invalidate(&mut self) {
        self.compositor.invalidate();
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    /// What the frame's [`Wake`] asks of the loop, after the frame that
    /// carries its `now` was drawn: continuous motion draws the next
    /// frame; a deadline sets the wait to that instant; nothing pending
    /// sleeps until an event. Any event ends a wait early.
    fn pace(&mut self, event_loop: &ActiveEventLoop, wake: Wake) {
        match wake.next {
            Some(next) if next.is_zero() => {
                // Something still moves: keep the frames coming.
                self.invalidate();
                event_loop.set_control_flow(ControlFlow::Wait);
            }
            Some(next) => {
                event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + next))
            }
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }

    /// Applies a result from the git worker to the workbench.
    fn apply_git_event(&mut self, event: GitEvent) {
        self.bench.apply(event);
        self.invalidate();
    }

    /// Keyboard handling: text input for the commit form, everything else
    /// through the keymap. Returns true if state changed.
    fn handle_key_event(&mut self, key_event: &KeyEvent, event_loop: &ActiveEventLoop) -> bool {
        // The visible commit form captures plain characters as text before
        // the keymap sees them (named keys, Enter/Escape/Backspace, still
        // dispatch through the CommitForm context bindings). Typing is
        // editing, not a capability: it stays with the field.
        let form = &mut self.bench.workspace_mut().commit_form;
        if form.visible
            && !self.modifiers.control_key()
            && !self.modifiers.super_key()
            && !self.modifiers.alt_key()
            && let Key::Character(text) = &key_event.logical_key
        {
            for ch in text.chars() {
                form.append_char(ch);
            }
            return true;
        }

        let Some(keystroke) = keystroke_from_key_event(key_event, self.modifiers) else {
            return false;
        };
        let stack = self.context_stack();
        match self.matcher.match_keystroke(&keystroke, &stack) {
            MatchResult::Complete(name) => {
                let Some(action) = self.registry.build(&name) else {
                    log::warn!("keymap resolved unknown action `{name}`");
                    return false;
                };
                self.dispatch(action.as_ref(), event_loop)
            }
            MatchResult::Pending | MatchResult::None => false,
        }
    }

    /// Key contexts from outermost to innermost; deeper entries win binding
    /// conflicts (e.g. Escape closes an overlay before quitting the app).
    fn context_stack(&self) -> ContextStack {
        let ws = self.bench.workspace();
        let mut stack = ContextStack::new();
        stack.push(KeyContext::new("Workspace"));
        if ws.commit_form.visible {
            stack.push(KeyContext::new("CommitForm"));
        }
        if !ws.overlay_mgr.is_empty() {
            stack.push(KeyContext::new("Overlay"));
        }
        stack
    }

    /// Executes a resolved action. Returns true if state changed. The
    /// keymap names input actions; the ones that act on a capability
    /// (`commit::Submit`, `file::Stage`, ...) go through the workbench,
    /// which asks the dispatcher as the person.
    fn dispatch(&mut self, action: &dyn Action, event_loop: &ActiveEventLoop) -> bool {
        let bench = &mut self.bench;
        if action.is::<actions::Quit>() {
            event_loop.exit();
            false
        } else if action.is::<actions::Toggle>() {
            bench.workspace_mut().toggle_theme();
            true
        } else if action.is::<actions::Up>() {
            bench.nav_up()
        } else if action.is::<actions::Down>() {
            bench.nav_down()
        } else if action.is::<actions::ShowDiff>() {
            bench.show_selected_diff()
        } else if action.is::<actions::Stage>() {
            bench.stage_selected()
        } else if action.is::<actions::Discard>() {
            bench.discard_selected()
        } else if action.is::<actions::OpenForm>() {
            bench.toggle_commit_form()
        } else if action.is::<actions::Submit>() {
            bench.submit_commit()
        } else if action.is::<actions::Cancel>() {
            bench.cancel_commit_form()
        } else if action.is::<actions::Backspace>() {
            bench.workspace_mut().commit_form.backspace();
            true
        } else if action.is::<actions::Close>() {
            bench.close_top_overlay()
        } else {
            log::warn!("no handler for action {}", action.name());
            false
        }
    }
}

impl ApplicationHandler<AppEvent> for App {
    /// The deadline a tick asked for came due: the frame whose tick flips
    /// the caret or lets a toast go. The wait goes back to `Wait` first,
    /// since a `WaitUntil` in the past would fire on every iteration if
    /// the redraw does not come (an occluded window); the redraw's own
    /// wake sets the next.
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            event_loop.set_control_flow(ControlFlow::Wait);
            self.invalidate();
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let title = format!("plev ide - {}", self.bench.workspace().repo_label);
        let attrs = WindowAttributes::default()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(1280u32, 800u32));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());

        self.scale_factor = window.scale_factor();
        let gpu = pollster::block_on(GpuContext::new(window.clone()));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let effects = engine::effects::EffectProcessor::new(&gpu.device, gpu.surface_format());
        self.state = GpuState::Ready {
            gpu,
            text_system,
            effects,
            texture_pool: TexturePool::new(),
        };

        let size = window.inner_size();
        let sf = self.scale_factor as f32;
        let lw = size.width as f32 / sf;
        let lh = size.height as f32 / sf;
        self.bench.workspace_mut().resize(lw, lh);
        if let GpuState::Ready { gpu, .. } = &mut self.state {
            gpu.set_projection(lw, lh);
        }
        self.invalidate();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::Git(git_event) => self.apply_git_event(git_event),
            AppEvent::FsChange => {
                // Something changed on disk: reload status/log/branches.
                // The resulting events invalidate the scene when applied.
                self.bench.refresh();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }

            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if key_event.state == ElementState::Pressed
                    && self.handle_key_event(&key_event, event_loop)
                {
                    self.invalidate();
                }
            }

            WindowEvent::Resized(size) => {
                let sf = self.scale_factor as f32;
                let lw = size.width as f32 / sf;
                let lh = size.height as f32 / sf;
                if let GpuState::Ready { gpu, .. } = &mut self.state {
                    gpu.resize(size.width, size.height);
                    gpu.set_projection(lw, lh);
                }
                self.bench.workspace_mut().resize(lw, lh);
                self.invalidate();
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                // A Resized follows on most platforms, but invalidating here
                // is cheap and guarantees a frame on DPI changes.
                self.scale_factor = scale_factor;
                self.invalidate();
            }

            WindowEvent::CursorMoved { position, .. } => {
                let sf = self.scale_factor as f32;
                let cx = position.x as f32 / sf;
                let cy = position.y as f32 / sf;
                self.cursor_pos = (cx, cy);
                let ws = self.bench.workspace_mut();

                let dragged = ws.dragging_left || ws.dragging_right;
                if dragged {
                    ws.update_drag(cx);
                }

                let old_lh = ws.hover_left_handle;
                let old_rh = ws.hover_right_handle;
                match ws.hit_test_handle(cx) {
                    Some(Side::Left) => {
                        ws.hover_left_handle = true;
                        ws.hover_right_handle = false;
                    }
                    Some(Side::Right) => {
                        ws.hover_right_handle = true;
                        ws.hover_left_handle = false;
                    }
                    None => {
                        ws.hover_left_handle = false;
                        ws.hover_right_handle = false;
                    }
                }

                let on_handle = ws.hover_left_handle
                    || ws.hover_right_handle
                    || ws.dragging_left
                    || ws.dragging_right;
                let handle_changed =
                    old_lh != ws.hover_left_handle || old_rh != ws.hover_right_handle;
                let hover_changed = if !ws.overlay_mgr.is_empty() {
                    ws.handle_overlay_hover(cx, cy)
                } else {
                    ws.handle_hover(cx, cy)
                };
                if let Some(w) = &self.window {
                    w.set_cursor(if on_handle {
                        CursorIcon::EwResize
                    } else {
                        CursorIcon::Default
                    });
                }
                if dragged || handle_changed || hover_changed {
                    self.invalidate();
                }
            }

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                let (cx, cy) = self.cursor_pos;
                match state {
                    ElementState::Pressed => match self.bench.workspace().hit_test_handle(cx) {
                        Some(Side::Left) => self.bench.workspace_mut().begin_drag_left(cx),
                        Some(Side::Right) => self.bench.workspace_mut().begin_drag_right(cx),
                        None => {
                            if self.bench.click(cx, cy) {
                                self.invalidate();
                            }
                        }
                    },
                    ElementState::Released => {
                        self.bench.workspace_mut().end_drag();
                        self.invalidate();
                    }
                }
            }

            WindowEvent::MouseInput {
                button: MouseButton::Right,
                state: ElementState::Pressed,
                ..
            } => {
                let (cx, cy) = self.cursor_pos;
                if self.bench.right_click(cx, cy) {
                    self.invalidate();
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let (cx, _) = self.cursor_pos;
                let scroll_delta = match delta {
                    MouseScrollDelta::LineDelta(_x, y) => -y * 20.0,
                    MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
                };
                // Scroll invalidates unconditionally: ScrollState clamps, so
                // a no-op wheel costs one cheap identical-hash frame at most.
                self.bench.workspace_mut().scroll(cx, scroll_delta);
                self.invalidate();
            }

            WindowEvent::RedrawRequested => {
                let GpuState::Ready {
                    gpu,
                    text_system,
                    effects,
                    texture_pool,
                } = &mut self.state
                else {
                    return;
                };
                // The real time since the last frame: a deadline the loop
                // slept through arrives whole (`dt` is clamped for motion).
                let tick = self.clock.tick();
                let wake = self.bench.wake(tick.since_last);
                log::debug!(
                    "frame: {:.3} s since the last, wake now {} next {:?}",
                    tick.since_last,
                    wake.now,
                    wake.next
                );
                renderer::render_frame(
                    gpu,
                    text_system,
                    effects,
                    texture_pool,
                    &mut self.compositor,
                    &mut self.bench,
                );
                // `wake.now` is carried by this frame: the scene was built
                // after the tick, so a flipped caret is already in it.
                self.pace(event_loop, wake);
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    // Repository: argv[1] or the current directory (any subdir works).
    let repo_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("cwd is accessible"));

    let event_loop = EventLoop::<AppEvent>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();
    let git = match GitClient::spawn(&repo_path, move |event| {
        // Wake the UI thread; send_event only fails after loop shutdown.
        let _ = proxy.send_event(AppEvent::Git(event));
    }) {
        Ok(git) => git,
        Err(e) => {
            eprintln!(
                "plev ide: cannot open a git repository at `{}`: {e}",
                repo_path.display()
            );
            std::process::exit(1);
        }
    };
    let workdir = git.workdir().to_path_buf();
    let mut bench = Workbench::new(1280.0, 800.0);
    // Initial data load (results stream in as user events).
    bench.connect(git);

    // Watch worktree + .git so external changes (editor saves, CLI
    // commits) refresh the UI automatically. The app still works if the
    // platform watcher fails; it just won't auto-refresh.
    let fs_proxy = event_loop.create_proxy();
    let fs_watcher = watcher::spawn(&workdir, move || {
        let _ = fs_proxy.send_event(AppEvent::FsChange);
    })
    .inspect_err(|e| log::warn!("file watcher disabled: {e}"))
    .ok();

    let mut app = App::new(bench, fs_watcher);
    event_loop.run_app(&mut app).unwrap();
}
