//! the app under experiment: the ide's `Workbench` on the experiment's
//! repository, its git worker waking this thread through a channel, and
//! its frame drawn without a window through `engine::drawing::headless` when the
//! machine has a GPU. the plev, scripted, screenshot and review arms start
//! here.

use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::{Duration, Instant};

use engine::drawing::compositor::Compositor;
use engine::drawing::gpu::RenderConfig;
use engine::drawing::headless::{HeadlessError, Renderer, Viewport};
use gitdrv::{GitClient, GitEvent};
use idestudio::capability::Workbench;

use crate::repo::Repo;

/// the window the frames are drawn for: the ide's launch size.
pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 800.0;

/// how long the first status, log and branches may take.
const LOAD: Duration = Duration::from_secs(30);

/// what wakes the laboratory's thread.
pub enum Wake {
    Git(GitEvent),
    Port,
}

pub struct Bench {
    pub repo: Repo,
    pub bench: Workbench,
    pub rx: Receiver<Wake>,
    pub tx: Sender<Wake>,
    pub compositor: Compositor,
    /// the frame path into a texture, when the machine has a GPU adapter.
    pub gpu: Option<Renderer>,
}

impl Bench {
    /// the repository built, the worker spawned, the workbench connected
    /// and loaded (status, log and branches applied), the headless
    /// renderer opened at scale 1 when `draw` and the machine allow.
    pub fn open(draw: bool) -> Result<Bench, String> {
        Bench::open_on(Repo::build()?, draw)
    }

    /// `open`, on `repo` (the repository of the review, task-96).
    pub fn open_on(repo: Repo, draw: bool) -> Result<Bench, String> {
        let (tx, rx) = channel();
        let to_loop = tx.clone();
        let client = GitClient::spawn(repo.path(), move |event| {
            let _ = to_loop.send(Wake::Git(event));
        })
        .map_err(|e| format!("the git worker on the repository: {e}"))?;
        let mut bench = Workbench::new(WIDTH, HEIGHT);
        bench.connect(client);
        let gpu = if draw { headless(1.0)? } else { None };
        let mut out = Bench {
            repo,
            bench,
            rx,
            tx,
            compositor: Compositor::new(),
            gpu,
        };
        out.until(|e| matches!(e, GitEvent::Branches(_)))?;
        Ok(out)
    }

    /// applies worker events, in order, until one matches `last`; port
    /// wakes arriving meanwhile are dropped (no port is served here).
    pub fn until(&mut self, last: impl Fn(&GitEvent) -> bool) -> Result<(), String> {
        let deadline = Instant::now() + LOAD;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match self.rx.recv_timeout(left) {
                Ok(Wake::Git(event)) => {
                    let stop = last(&event);
                    self.bench.apply(event);
                    if stop {
                        return Ok(());
                    }
                }
                Ok(Wake::Port) => {}
                Err(RecvTimeoutError::Timeout) => {
                    return Err("the git worker did not answer in time".to_owned());
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err("the git worker is gone".to_owned());
                }
            }
        }
    }

    /// the background the frame clears to, in linear space.
    pub fn clear(&self) -> [f32; 4] {
        self.bench.workspace().theme().colors.bg.to_linear_array()
    }

    /// builds the frame as the window would (`begin_frame`, the workbench
    /// draws and binds) and, with a GPU, draws it into the texture. the
    /// time of each part.
    pub fn frame(&mut self) -> Timed {
        let at = Instant::now();
        self.compositor.begin_frame();
        self.bench.render(&mut self.compositor);
        let build = at.elapsed();
        let clear = self.clear();
        let draw = self.gpu.as_mut().map(|gpu| {
            let at = Instant::now();
            gpu.draw(&mut self.compositor, clear);
            at.elapsed()
        });
        Timed { build, draw }
    }
}

/// how long a frame took: the scene build, and the GPU draw when there
/// was one.
pub struct Timed {
    pub build: Duration,
    pub draw: Option<Duration>,
}

/// a headless renderer for the ide's window at `scale`, or `None` with a
/// line on stderr when the machine has no GPU adapter.
pub fn headless(scale: f32) -> Result<Option<Renderer>, String> {
    match Renderer::new(Viewport::new(WIDTH, HEIGHT, scale), RenderConfig::default()) {
        Ok(renderer) => Ok(Some(renderer)),
        Err(HeadlessError::NoAdapter) => {
            eprintln!("agenteval: no GPU adapter; the frames are built and recorded, not drawn");
            Ok(None)
        }
        Err(e) => Err(format!("the headless renderer: {e}")),
    }
}
