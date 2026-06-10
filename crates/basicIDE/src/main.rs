//! basicIDE — plev-native git client.
//!
//! GPU-native 3-panel workspace UI (no Tauri, no WebView, no JavaScript)
//! showing a real repository: status, log, branches and diffs come from
//! `git_backend` running on a worker thread; the UI thread never blocks.
//!
//! Run: `cargo run -p basicIDE [path-to-repo]` (defaults to the cwd).

// Catálogo de componentes (avatar, badge, checkbox, tabs, separator,
// panel_header) ainda não está todo conectado às views — silencia os
// dead_code até o port completar.
#![allow(dead_code)]

mod adapters;
mod components;
mod renderer;
mod theme;
mod views;

use std::path::PathBuf;
use std::sync::Arc;

use git_backend::{GitClient, GitCommand, GitEvent};
use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::texture_pool::TexturePool;
use views::workspace::{Side, UiRequest, WorkspaceView};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{CursorIcon, Window, WindowAttributes, WindowId};

/// How many commits the stacks panel loads per refresh.
const LOG_LIMIT: usize = 100;

/// Events injected into the winit loop from background threads.
enum AppEvent {
    Git(GitEvent),
}

// ---------------------------------------------------------------------------
// GPU state
// ---------------------------------------------------------------------------

enum GpuState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        _pool: TexturePool,
    },
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

/// What the diff panel is currently waiting for; stale diff events (the
/// user clicked elsewhere meanwhile) are dropped.
#[derive(PartialEq)]
enum DiffTarget {
    File(String),
    Commit(String),
}

struct App {
    window: Option<Arc<Window>>,
    state: GpuState,
    compositor: Compositor,
    workspace: WorkspaceView,
    cursor_pos: (f32, f32),
    scale_factor: f64,

    git: GitClient,
    diff_target: Option<DiffTarget>,
    /// Last log/branches payloads — the stacks panel needs both.
    log_cache: Vec<git_backend::Commit>,
    branch_cache: Vec<git_backend::Branch>,
}

impl App {
    fn new(git: GitClient) -> Self {
        let mut workspace = WorkspaceView::new(1280.0, 800.0);
        workspace.repo_label = git
            .workdir()
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| git.workdir().display().to_string());
        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor: Compositor::new(),
            workspace,
            cursor_pos: (0.0, 0.0),
            scale_factor: 1.0,
            git,
            diff_target: None,
            log_cache: Vec::new(),
            branch_cache: Vec::new(),
        }
    }

    /// Mark the scene as changed and schedule a frame. Frames are rendered
    /// on demand only: without input the event loop stays idle (no
    /// unconditional `request_redraw`). State changes are also the only
    /// source of git work, so the request queue is flushed here too.
    fn invalidate(&mut self) {
        self.flush_requests();
        self.compositor.invalidate();
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    /// Forwards git operations queued by the views to the worker thread.
    fn flush_requests(&mut self) {
        for request in self.workspace.take_requests() {
            let command = match request {
                UiRequest::FileDiff { path } => {
                    self.diff_target = Some(DiffTarget::File(path.clone()));
                    GitCommand::DiffWorkdir { path }
                }
                UiRequest::CommitDiff { sha } => {
                    self.diff_target = Some(DiffTarget::Commit(sha.clone()));
                    GitCommand::DiffCommit { sha }
                }
                UiRequest::Stage { path } => GitCommand::Stage { path },
                UiRequest::Unstage { path } => GitCommand::Unstage { path },
                UiRequest::Discard { path } => GitCommand::Discard { path },
                UiRequest::Ignore { path } => GitCommand::Ignore { path },
                UiRequest::Commit { message } => GitCommand::Commit { message },
            };
            self.git.send(command);
        }
    }

    /// Applies a result from the git worker to the views.
    fn apply_git_event(&mut self, event: GitEvent) {
        match event {
            GitEvent::Status(Ok(status)) => {
                self.workspace
                    .unassigned
                    .set_files(adapters::file_entries(&status));
            }
            GitEvent::Log(Ok(log)) => {
                self.log_cache = log;
                self.rebuild_stacks();
            }
            GitEvent::Branches(Ok(branches)) => {
                self.branch_cache = branches;
                self.workspace.branch_label = self
                    .branch_cache
                    .iter()
                    .find(|b| b.is_head)
                    .map(|b| b.name.clone())
                    .unwrap_or_default();
                self.rebuild_stacks();
            }
            GitEvent::DiffWorkdir { path, result } => {
                if self.diff_target == Some(DiffTarget::File(path)) {
                    match result {
                        Ok(hunks) => {
                            self.workspace.diff.set_lines(adapters::diff_lines(&hunks));
                        }
                        Err(e) => log::error!("diff failed: {e}"),
                    }
                }
            }
            GitEvent::DiffCommit { sha, result } => {
                if self.diff_target == Some(DiffTarget::Commit(sha)) {
                    match result {
                        Ok(hunks) => {
                            self.workspace.diff.set_lines(adapters::diff_lines(&hunks));
                        }
                        Err(e) => log::error!("commit diff failed: {e}"),
                    }
                }
            }
            // Mutations: refresh everything afterwards so the optimistic
            // view updates reconcile with what git actually did (also on
            // error, to roll wrong guesses back).
            GitEvent::Staged { path, result }
            | GitEvent::Unstaged { path, result }
            | GitEvent::Discarded { path, result }
            | GitEvent::Ignored { path, result } => {
                if let Err(e) = result {
                    log::error!("git operation on {path} failed: {e}");
                }
                self.git.send(GitCommand::Refresh {
                    log_limit: LOG_LIMIT,
                });
            }
            GitEvent::Committed(result) => {
                match result {
                    Ok(sha) => log::info!("committed {sha}"),
                    Err(e) => log::error!("commit failed: {e}"),
                }
                self.git.send(GitCommand::Refresh {
                    log_limit: LOG_LIMIT,
                });
            }
            GitEvent::Status(Err(e)) => log::error!("status failed: {e}"),
            GitEvent::Log(Err(e)) => log::error!("log failed: {e}"),
            GitEvent::Branches(Err(e)) => log::error!("branches failed: {e}"),
        }
        self.invalidate();
    }

    fn rebuild_stacks(&mut self) {
        self.workspace
            .stacks
            .set_stacks(adapters::stacks(&self.branch_cache, &self.log_cache));
    }

    /// Keyboard handling. Returns true if state changed.
    fn handle_key(&mut self, key: &Key, event_loop: &ActiveEventLoop) -> bool {
        // The visible commit form captures text input first.
        if self.workspace.commit_form.visible {
            return match key {
                Key::Named(NamedKey::Escape) => {
                    self.workspace.commit_form.hide();
                    true
                }
                Key::Named(NamedKey::Backspace) => {
                    self.workspace.commit_form.backspace();
                    true
                }
                Key::Named(NamedKey::Enter) => self.workspace.submit_commit(),
                Key::Character(c) => {
                    for ch in c.chars() {
                        self.workspace.commit_form.append_char(ch);
                    }
                    true
                }
                _ => false,
            };
        }

        match key {
            Key::Named(NamedKey::Escape) => {
                // Overlays close first; quit only with nothing on screen.
                if self.workspace.close_top_overlay() {
                    true
                } else {
                    event_loop.exit();
                    false
                }
            }
            Key::Named(NamedKey::ArrowUp) => self.workspace.nav_up(),
            Key::Named(NamedKey::ArrowDown) => self.workspace.nav_down(),
            Key::Named(NamedKey::Enter) => self.workspace.show_selected_diff(),
            Key::Character(c) if c.eq_ignore_ascii_case("t") => {
                self.workspace.toggle_theme();
                true
            }
            Key::Character(c) if c.eq_ignore_ascii_case("c") => self.workspace.toggle_commit_form(),
            _ => false,
        }
    }
}

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let title = format!("basicIDE — {}", self.workspace.repo_label);
        let attrs = WindowAttributes::default()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(1280u32, 800u32));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());

        self.scale_factor = window.scale_factor();
        let gpu = pollster::block_on(GpuContext::new(window.clone()));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let pool = TexturePool::new();
        self.state = GpuState::Ready {
            gpu,
            text_system,
            _pool: pool,
        };

        let size = window.inner_size();
        let sf = self.scale_factor as f32;
        let lw = size.width as f32 / sf;
        let lh = size.height as f32 / sf;
        self.workspace.resize(lw, lh);
        if let GpuState::Ready { gpu, .. } = &mut self.state {
            gpu.set_projection(lw, lh);
        }
        self.invalidate();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::Git(git_event) => self.apply_git_event(git_event),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if key_event.state == ElementState::Pressed
                    && self.handle_key(&key_event.logical_key, event_loop)
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
                self.workspace.resize(lw, lh);
                self.invalidate();
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
            }

            WindowEvent::CursorMoved { position, .. } => {
                let sf = self.scale_factor as f32;
                let cx = position.x as f32 / sf;
                let cy = position.y as f32 / sf;
                self.cursor_pos = (cx, cy);

                if self.workspace.dragging_left || self.workspace.dragging_right {
                    self.workspace.update_drag(cx);
                    self.invalidate();
                }

                let old_lh = self.workspace.hover_left_handle;
                let old_rh = self.workspace.hover_right_handle;
                match self.workspace.hit_test_handle(cx) {
                    Some(Side::Left) => {
                        self.workspace.hover_left_handle = true;
                        self.workspace.hover_right_handle = false;
                    }
                    Some(Side::Right) => {
                        self.workspace.hover_right_handle = true;
                        self.workspace.hover_left_handle = false;
                    }
                    None => {
                        self.workspace.hover_left_handle = false;
                        self.workspace.hover_right_handle = false;
                    }
                }

                let on_handle = self.workspace.hover_left_handle
                    || self.workspace.hover_right_handle
                    || self.workspace.dragging_left
                    || self.workspace.dragging_right;
                if let Some(w) = &self.window {
                    w.set_cursor(if on_handle {
                        CursorIcon::EwResize
                    } else {
                        CursorIcon::Default
                    });
                }

                let handle_changed = old_lh != self.workspace.hover_left_handle
                    || old_rh != self.workspace.hover_right_handle;
                let hover_changed = if !self.workspace.overlay_mgr.is_empty() {
                    self.workspace.handle_overlay_hover(cx, cy)
                } else {
                    self.workspace.handle_hover(cx, cy)
                };
                if handle_changed || hover_changed {
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
                    ElementState::Pressed => match self.workspace.hit_test_handle(cx) {
                        Some(Side::Left) => self.workspace.begin_drag_left(cx),
                        Some(Side::Right) => self.workspace.begin_drag_right(cx),
                        None => {
                            if self.workspace.handle_click(cx, cy) {
                                self.invalidate();
                            }
                        }
                    },
                    ElementState::Released => {
                        self.workspace.end_drag();
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
                if self.workspace.handle_right_click(cx, cy) {
                    self.invalidate();
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let (cx, _) = self.cursor_pos;
                let scroll_delta = match delta {
                    MouseScrollDelta::LineDelta(_x, y) => -y * 20.0,
                    MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
                };
                self.workspace.scroll(cx, scroll_delta);
                self.invalidate();
            }

            WindowEvent::RedrawRequested => {
                let GpuState::Ready {
                    gpu, text_system, ..
                } = &mut self.state
                else {
                    return;
                };
                renderer::render_frame(gpu, text_system, &mut self.compositor, &mut self.workspace);
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
                "basicIDE: cannot open a git repository at `{}`: {e}",
                repo_path.display()
            );
            std::process::exit(1);
        }
    };
    // Initial data load (results stream in as user events).
    git.send(GitCommand::Refresh {
        log_limit: LOG_LIMIT,
    });

    let mut app = App::new(git);
    event_loop.run_app(&mut app).unwrap();
}
