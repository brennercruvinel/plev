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
//! the wait early, as winit documents, and each loop iteration first
//! advances what animates to the instant it woke, so what the events then
//! change counts its time from the change, not from the last frame.
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
    /// commit field caret, the toasts). Ticks when the loop wakes and when
    /// a frame is drawn; its `since_last` is the real time since the last
    /// tick, a slept-through deadline included.
    clock: engine::animation::FrameClock,
    /// A catch-up tick since the last frame changed something visible
    /// (a deadline came due as the loop woke): the next frame shows it.
    caught_up: bool,
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
            caught_up: false,
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

    /// Advances what animates to now, before anything from outside the
    /// frame lands (a key, a click, a git result, a file change). The
    /// tick gets the real time since the last one, so a change applied
    /// between two frames would otherwise take the whole gap on the next:
    /// a refusal toast pushed after ten idle seconds would start ten
    /// seconds old and leave at once, a caret focused then would start in
    /// any phase. This tick draws nothing; if it changed something
    /// visible, it asks for the frame.
    fn catch_up(&mut self) {
        let since_last = self.clock.tick().since_last;
        self.advance(since_last);
    }

    /// [`App::catch_up`] with the elapsed time given.
    fn advance(&mut self, since_last: f32) {
        if self.bench.wake(since_last).now {
            self.caught_up = true;
            self.invalidate();
        }
    }

    /// The frame's tick: what animates, advanced by the time since the
    /// last tick (a catch-up or a frame), and what that asks of the loop.
    /// `now` also carries a visible change a catch-up made since the last
    /// frame.
    fn frame_wake(&mut self, since_last: f32) -> Wake {
        let mut wake = self.bench.wake(since_last);
        wake.now |= std::mem::take(&mut self.caught_up);
        wake
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
    /// The loop woke: time catches up first, so whatever woke it lands at
    /// this instant. When a deadline a tick asked for came due (the caret
    /// flips, a toast leaves), the wait goes back to `Wait` and the frame
    /// is asked for, since a `WaitUntil` in the past would fire on every
    /// iteration if the redraw does not come (an occluded window); the
    /// redraw's own wake sets the next.
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.catch_up();
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
                if !matches!(self.state, GpuState::Ready { .. }) {
                    return;
                }
                // The real time since the last tick: a deadline the loop
                // slept through arrives whole (`dt` is clamped for motion).
                let since_last = self.clock.tick().since_last;
                let wake = self.frame_wake(since_last);
                log::debug!(
                    "frame: {:.3} s since the last tick, wake now {} next {:?}",
                    since_last,
                    wake.now,
                    wake.next
                );
                let GpuState::Ready {
                    gpu,
                    text_system,
                    effects,
                    texture_pool,
                } = &mut self.state
                else {
                    return;
                };
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

#[cfg(test)]
mod tests {
    //! The shell's pacing with an injected clock: a change from outside
    //! the frame lands after the loop caught up (`App::advance`, what
    //! `catch_up` runs with the real elapsed time), so it counts its time
    //! from the change, however long the window sat idle before it.

    use super::*;

    /// One turn of a 60 Hz display.
    const TURN: f32 = 1.0 / 60.0;

    /// How long the window sat idle before the event.
    const IDLE: f32 = 10.0;

    /// How long after the event its frame is drawn.
    const LATENCY: f32 = 0.004;

    /// The shell around a workbench with no repository, settled.
    fn app() -> App {
        let mut app = App::new(Workbench::new(1280.0, 800.0), None);
        for _ in 0..600 {
            if !app.frame_wake(TURN).is_continuous() {
                break;
            }
        }
        app
    }

    /// The frames over `turns` display turns after an event: turn 0 is
    /// the event's frame; afterwards a turn is drawn when the last ask was
    /// continuous or its deadline has come due, and its tick gets the time
    /// since the last drawn turn.
    fn paced(app: &mut App, turns: usize) -> Vec<(usize, Wake)> {
        let mut drawn = vec![(0, app.frame_wake(LATENCY))];
        for turn in 1..turns {
            let (last, ask) = *drawn.last().expect("turn 0 is drawn");
            let since = (turn - last) as f32 * TURN;
            if ask.next.is_some_and(|d| since >= d.as_secs_f32()) {
                drawn.push((turn, app.frame_wake(since)));
            }
        }
        drawn
    }

    /// The toasts the workbench projects.
    fn notices(app: &App) -> usize {
        let snap = app.bench.snapshot().expect("snapshot");
        let id: sem::SemId = "notices".parse().expect("id");
        snap.children(&id).count()
    }

    #[test]
    fn a_refusal_toast_after_an_idle_gap_lives_its_whole_duration() {
        let mut app = app();
        app.advance(IDLE);
        // Enter on an empty message: refused, and the refusal is a toast
        assert!(app.bench.toggle_commit_form());
        assert!(app.bench.submit_commit());
        assert!(app.bench.cancel_commit_form());
        assert_eq!(notices(&app), 1);
        let drawn = paced(&mut app, 360);
        let rest = drawn
            .iter()
            .position(|(_, ask)| !ask.is_continuous())
            .expect("the entry settles");
        let (rest_turn, rest_ask) = drawn[rest];
        let left = rest_ask.next.expect("a deadline while it rests");
        let due = rest_turn as f32 * TURN + LATENCY + left.as_secs_f32();
        assert!((due - 4.0).abs() < TURN, "leaves {due} s after the refusal");
        assert_eq!(notices(&app), 0, "gone within six seconds");
    }

    #[test]
    fn a_caret_focused_after_an_idle_gap_starts_solid_and_flips_on_time() {
        let mut app = app();
        app.advance(IDLE);
        assert!(
            app.bench.toggle_commit_form(),
            "`c` opens the form, focused"
        );
        let drawn = paced(&mut app, 360);
        let (_, first) = drawn[0];
        assert!(!first.now, "no flip on the key's frame");
        // a whole half-period from the key, none of the idle before it
        let next = first.next.expect("a flip is pending").as_secs_f32();
        assert!(
            (0.53 - LATENCY - 1e-3..0.53 + 1e-3).contains(&next),
            "{next}"
        );
        let flips: Vec<usize> = drawn[1..].iter().map(|(turn, _)| *turn).collect();
        assert!(drawn[1..].iter().all(|(_, ask)| ask.now), "{drawn:?}");
        assert_eq!(flips.len(), 11, "{drawn:?}");
        assert_eq!(flips[0], 32, "the first flip 0.53 s after the key");
    }
}
