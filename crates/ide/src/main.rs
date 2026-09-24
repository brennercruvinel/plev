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
//! With `PLEV_AGENT=1` in a debug build the window also opens the agent
//! port (`ide::agent`, `$TMPDIR/plev/ide-<pid>.sock`), so a person and an
//! agent operate the same workbench at once. The port wakes the loop; the
//! shell serves it on this thread, and again after every git result it
//! applies, since a stage, an unstage or a commit ends on the worker and
//! its last stage goes out only then. A frame is drawn only when an act
//! ran: an observation changes nothing.
//!
//! Frames render on demand, paced as the showcase paces them (`pace.rs`):
//! the workbench's tick answers a `Wake`, motion (a scrollbar fade, a toast
//! springing in) draws every turn, and a caret or a toast at rest sleeps
//! the loop until its deadline (`ControlFlow::WaitUntil`), or for good
//! when nothing is pending. Input, the git worker, the file watcher and
//! the agent port end the wait early, as winit documents, and each loop iteration first
//! advances what animates to the instant it woke, so what the events then
//! change counts its time from the change, not from the last frame. Keys
//! go through the keymap (`keys.rs`).
//!
//! With `PLEV_RECORD=1` in a debug build the shell records the session
//! for the inspector (`ide::record`): the receipts, who observed what
//! through the port, the state, a person's clicks and keys, and every
//! frame drawn with where each bound control shows. It saves the retained
//! window to `$TMPDIR/plev/sessions/ide-<pid>-<n>.monster` on cmd-shift-s
//! (ctrl-shift-s), when the window closes, and on a panic, and logs where.
//!
//! With `PLEV_BACKGROUND=1` on macOS the window comes up without taking the
//! focus, whatever app is in front, so it can open beside someone at work
//! (an agent's run the person watches, not one they use): the app never
//! activates (its activation policy is prohibited, so no Dock icon and no
//! menu bar either), the window is ordered front without becoming key, and
//! it sits below every other window, so it covers nothing and takes no
//! click meant for another app. Keys reach it only posted to its pid.
//! Nothing changes without it, or elsewhere.
//!
//! Run: `cargo run -p ide [path-to-repo]` (defaults to the cwd).

mod actions;
mod keys;
mod pace;
mod recording;
mod renderer;
mod watcher;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use engine::actions::{ActionRegistry, KeymapMatcher};
use engine::compositor::Compositor;
use engine::gpu::GpuContext;
use engine::gpu::texture_pool::TexturePool;
use engine::text::TextSystem;
use git::{GitClient, GitEvent};
use ide::record::Recording;
use ide::views::workspace::Side;
use ide::workbench::Workbench;
use rec::InputKind;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::{CursorIcon, Window, WindowAttributes, WindowId};

/// Events injected into the winit loop from background threads.
enum AppEvent {
    Git(GitEvent),
    /// The worktree or `.git` changed on disk (debounced).
    FsChange,
    /// An agent asked through the port: serve it, draw if an act ran.
    #[cfg_attr(not(unix), allow(dead_code))] // the port is a unix socket
    Agent,
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

    /// The agent port, when `PLEV_AGENT=1` in a debug build.
    #[cfg(unix)]
    port: Option<wire::AgentPort>,

    /// The window comes up without taking the focus (`PLEV_BACKGROUND=1`
    /// on macOS).
    background: bool,

    /// The session's recording, when `PLEV_RECORD=1` in a debug build.
    recording: Option<Recording>,
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
            #[cfg(unix)]
            port: None,
            background: false,
            recording: None,
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

    /// Applies a result from the git worker to the workbench.
    fn apply_git_event(&mut self, event: GitEvent) {
        self.bench.apply(event);
        self.invalidate();
    }

    /// What agents asked through the port, served on this thread with the
    /// frame on screen, and every stage that ended since sent; a frame
    /// only when an act ran, since an observation changes nothing. The
    /// recording keeps who observed what, and the state an act left.
    fn serve_agent(&mut self) {
        #[cfg(unix)]
        if let Some(port) = self.port.as_mut()
            && ide::agent::serve_recorded(
                port,
                &mut self.bench,
                &self.compositor,
                self.recording.as_ref(),
            )
        {
            self.invalidate();
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
            .with_inner_size(winit::dpi::LogicalSize::new(1280u32, 800u32))
            .with_active(!self.background);
        let attrs = if self.background {
            attrs.with_window_level(winit::window::WindowLevel::AlwaysOnBottom)
        } else {
            attrs
        };
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
            AppEvent::Git(git_event) => {
                self.apply_git_event(git_event);
                // The result may end an agent's request (a stage, an
                // unstage, a commit): its last stage goes out now.
                self.serve_agent();
                self.record_state();
            }
            AppEvent::Agent => self.serve_agent(),
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
                    self.changed();
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
                            self.record_input(InputKind::PointerDown, Some([cx, cy]));
                            if self.bench.click(cx, cy) {
                                self.changed();
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
                self.record_input(InputKind::PointerDown, Some([cx, cy]));
                if self.bench.right_click(cx, cy) {
                    self.changed();
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
                let drawing = Instant::now();
                renderer::render_frame(
                    gpu,
                    text_system,
                    effects,
                    texture_pool,
                    &mut self.compositor,
                    &mut self.bench,
                );
                let drawn = drawing.elapsed().as_secs_f64() * 1e3;
                // The frame just drawn, where each bound control shows in
                // it, and the state it was drawn from.
                if let Some(recording) = &self.recording {
                    let recording_at = Instant::now();
                    recording.frame(&self.bench, &self.compositor);
                    let recorded = recording_at.elapsed().as_secs_f64() * 1e3;
                    log::debug!("drawn: {drawn:.3} ms, recorded: {recorded:.3} ms");
                } else {
                    log::debug!("drawn: {drawn:.3} ms");
                }
                // `wake.now` is carried by this frame: the scene was built
                // after the tick, so a flipped caret is already in it.
                self.pace(event_loop, wake);
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}

    /// The loop ends (the window closed, or Escape quit): the recording's
    /// window is saved.
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.save_session("close");
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// `PLEV_BACKGROUND=1` on macOS: the window comes up without taking the
/// focus. Off by default and on every other platform.
///
/// Asking not to activate over another app is not enough: since macOS 14
/// activation is cooperative, and a process started from the terminal of
/// the app in front is let in over it. An app whose activation policy is
/// prohibited may not be activated at all, and still shows its window.
fn background() -> bool {
    cfg!(target_os = "macos")
        && std::env::var_os("PLEV_BACKGROUND").is_some_and(|v| v == std::ffi::OsStr::new("1"))
}

fn main() {
    // A recording says where its files go: the ide's own info lines show
    // by default when it records.
    let recording = Recording::from_env();
    let filter = if recording.is_some() {
        "warn,ide=info"
    } else {
        "warn"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(filter)).init();

    // Repository: argv[1] or the current directory (any subdir works).
    let repo_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("cwd is accessible"));

    let background = background();
    let mut builder = EventLoop::<AppEvent>::with_user_event();
    #[cfg(target_os = "macos")]
    if background {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        builder
            .with_activation_policy(ActivationPolicy::Prohibited)
            .with_activate_ignoring_other_apps(false);
    }
    let event_loop = builder.build().unwrap();
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
    // Off unless `PLEV_RECORD=1` in a debug build; nothing is created then.
    if let Some(recording) = &recording {
        recording::start(recording, &mut bench);
    }

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
    app.background = background;
    app.record(recording);
    // Off unless `PLEV_AGENT=1` in a debug build; nothing is created then.
    #[cfg(unix)]
    {
        let agent = event_loop.create_proxy();
        app.port = ide::agent::port(move || {
            let _ = agent.send_event(AppEvent::Agent);
        });
        if let Some(port) = &app.port {
            log::info!("agent port: {}", port.path().display());
        }
    }
    event_loop.run_app(&mut app).unwrap();
}
