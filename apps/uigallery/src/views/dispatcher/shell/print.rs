//! Printing the page (task-76 item 9), the view's side. `shell.print` is
//! a button a person asks for (cmd-p, the menu item), and the shell
//! prints the frame of the main window as it stands: the same scene the
//! window draws, at a paper scale, through the system's printer
//! (`engine::runtime::platform::print`). The scene is built on the loop's thread
//! (the view lives here; it costs a frame's scene build); everything after
//! runs on a thread of its own: a headless renderer of its own draws the
//! scene through the window's frame path and reads it back, the png is
//! encoded, the document handed to the printer, the job's place asked
//! once. The request stays open (`Execution::Started`) until the answer
//! lands, read by `settle_print` on the loop, as the Silences page reads
//! its ffmpeg. Under `engine::runtime::window::Background`: a print asked while
//! the app sleeps waits for the wake (the scene is built then, from the
//! state then), and an answer that lands asleep waits on the channel.
//!
//! The destinations are looked for once, off the loop, when the printer
//! is given (`set_printer`), and projected when they land; without one
//! the button is disabled and the projection says so, instead of a
//! failed request. What is projected: `shell.print` (`destinations`, and
//! the last job's `destination`, `job` and `status`) and `shell:print`,
//! one line for an agent: "no destination", "ready on Office", "printing
//! on Office", "Office-7 on Office: queued", "failed on Office: ...".

use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::Duration;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
use std::time::Instant;

use accord::{Policy, RequestId, Role, SnapshotBuilder, Value};
use engine::runtime::platform::print::{Destination, Printer};
use engine::runtime::window::Background;
use engine::widgets::prelude::Wake;

use super::super::super::ShowcaseView;
use super::{Ids, PERSON_ONLY};

/// How often the loop looks at a thread's channel while one is out:
/// nothing else wakes it when the thread answers (the Surface's `POLL`).
const POLL: Duration = Duration::from_millis(100);

/// The printer, shared with the job's thread.
type Shared = Arc<Mutex<Box<dyn Printer + Send>>>;

/// The shell's printing: the printer, what it knows, the job in course
/// and the last one.
pub(super) struct Printing {
    // off the desktop no printer is ever given (`set_printer` is the
    // desktop's), so nothing reads it there
    #[cfg_attr(
        any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
        allow(dead_code)
    )]
    printer: Option<Shared>,
    /// What the printer knows, once it answered; `None` until then.
    destinations: Option<Vec<Destination>>,
    /// The request `execute` started and the view has not taken up yet.
    pub(super) asked: Option<RequestId>,
    /// A request asked while the app slept: starts on the first look
    /// after the wake.
    held: Option<RequestId>,
    running: Option<Running>,
    last: Option<Last>,
}

/// A job on its thread: the request that waits for it (`None` for the
/// look for destinations), where it prints, and where its answer comes.
struct Running {
    id: Option<RequestId>,
    destination: String,
    rx: Receiver<Done>,
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    started: Instant,
}

/// The last job, as the projection tells it: the id the queue gave it
/// (`None` when nothing was queued), where, and its status ("queued",
/// "completed", "gone", or "failed: ...").
struct Last {
    pub job: Option<String>,
    pub destination: String,
    pub status: String,
}

/// What a thread sends back.
enum Done {
    Destinations(Result<Vec<Destination>, String>),
    Printed(Result<Printed, String>),
}

/// A document queued: the job, its status when asked right after, and
/// the document's measures for the receipt.
#[cfg_attr(
    any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
    allow(dead_code)
)]
struct Printed {
    job: String,
    status: String,
    width: u32,
    height: u32,
    dpi: f32,
    bytes: usize,
}

impl Printing {
    pub fn new() -> Printing {
        Printing {
            printer: None,
            destinations: None,
            asked: None,
            held: None,
            running: None,
            last: None,
        }
    }

    /// The destinations known, none until the printer answered.
    fn known(&self) -> &[Destination] {
        self.destinations.as_deref().unwrap_or(&[])
    }

    /// Where a print goes: the default destination, or the first.
    fn destination(&self) -> Option<&str> {
        let known = self.known();
        known
            .iter()
            .find(|d| d.default)
            .or_else(|| known.first())
            .map(|d| d.name.as_str())
    }

    /// A job is in course, or waits for the wake.
    fn busy(&self) -> bool {
        self.held.is_some() || self.running.as_ref().is_some_and(|r| r.id.is_some())
    }

    /// A person may ask now: a destination, and no job in course.
    fn printable(&self) -> bool {
        self.destination().is_some() && !self.busy()
    }

    /// The one line an agent reads (`shell:print`).
    pub fn summary(&self) -> String {
        if self.held.is_some()
            && let Some(destination) = self.destination()
        {
            return format!("printing on {destination}");
        }
        if let Some(running) = self.running.as_ref().filter(|r| r.id.is_some()) {
            return format!("printing on {}", running.destination);
        }
        match (&self.last, self.destination()) {
            (
                Some(Last {
                    job: Some(job),
                    destination,
                    status,
                }),
                _,
            ) => format!("{job} on {destination}: {status}"),
            (
                Some(Last {
                    job: None,
                    destination,
                    status,
                }),
                _,
            ) => format!("{status} on {destination}"),
            (None, Some(destination)) => format!("ready on {destination}"),
            (None, None) => "no destination".to_owned(),
        }
    }
}

/// `shell.print` beside the shell's other capabilities: a button for a
/// person alone, enabled with a destination and no job in course, with
/// the destinations and the last job as its state.
pub(super) fn project(printing: &Printing, ids: &Ids, b: &mut SnapshotBuilder) {
    let names: Vec<String> = printing.known().iter().map(|d| d.name.clone()).collect();
    let last = printing.last.as_ref();
    let mut node = b.node(&ids.print, Role::Button, "print this page");
    node.parent(&ids.shell)
        .enabled(printing.printable())
        .focusable(false)
        .state("destinations", Value::List(names))
        .state(
            "destination",
            Value::Text(printing.destination().unwrap_or_default().to_owned()),
        )
        .state(
            "job",
            Value::Text(last.and_then(|l| l.job.clone()).unwrap_or_default()),
        )
        .state(
            "status",
            Value::Text(last.map(|l| l.status.clone()).unwrap_or_default()),
        );
    node.action("activate").policy(Policy {
        actors: PERSON_ONLY,
        ..Policy::default()
    });
}

impl ShowcaseView {
    /// The printer the shell prints through (the platform's, or a fake):
    /// its destinations are looked for at once, on a thread, and
    /// projected when they land (`settle_print`). Desktop only: elsewhere
    /// there is no thread and no printer, and the button stays disabled.
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    pub fn set_printer(&mut self, printer: Box<dyn Printer + Send>) {
        let shared: Shared = Arc::new(Mutex::new(printer));
        let printing = &mut self.panes.shell.print;
        printing.printer = Some(shared.clone());
        printing.destinations = None;
        printing.running = Some(Running {
            id: None,
            destination: String::new(),
            rx: desktop::spawn_lookup(shared),
            started: Instant::now(),
        });
    }

    /// cmd-p, or the menu: the person asks `shell.print`. `true` when a
    /// job was taken up: started, or held for the wake.
    pub fn request_print(&mut self) -> bool {
        let receipt = self.ask_shell_receipt(super::ShellTarget::Print, "activate", None);
        receipt.is_some_and(|r| matches!(r.stage(), Some(accord::Stage::Executing)))
            && self.panes.shell.print.busy()
    }

    /// `execute` started request `id` for a print: the scene is built now
    /// and the job starts on its thread; asleep, it waits for the wake.
    /// Off the desktop the request fails at once, saying so.
    pub(super) fn start_print(&mut self, id: RequestId) {
        if Background::asleep() {
            self.panes.shell.print.held = Some(id);
            return;
        }
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        {
            self.start_print_job(id);
        }
        #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
        {
            self.end_print(id, Err("printing needs the desktop".to_owned()));
        }
    }

    /// While a thread is out (the look for destinations, a job) or a job
    /// waits for the wake, a look at the channel every `POLL`: the answer
    /// is read by `settle_print` on the catch-up the deadline brings, not
    /// on whatever input comes next.
    pub fn print_wake(&self) -> Wake {
        let printing = &self.panes.shell.print;
        if printing.running.is_some() || printing.held.is_some() {
            Wake::after(POLL)
        } else {
            Wake::idle()
        }
    }

    /// What a thread answered, applied: the destinations projected, a
    /// job's end completing or failing its request and told as the last
    /// print; a job held for the wake started. `true` when the projection
    /// moved. Nothing while the app sleeps.
    pub fn settle_print(&mut self) -> bool {
        if Background::asleep() {
            return false;
        }
        let mut moved = false;
        if let Some(id) = self.panes.shell.print.held.take() {
            self.start_print(id);
            moved = true;
        }
        let Some(running) = self.panes.shell.print.running.as_ref() else {
            return moved;
        };
        let done = match running.rx.try_recv() {
            Ok(done) => done,
            Err(TryRecvError::Empty) => return moved,
            Err(TryRecvError::Disconnected) => {
                let why = "the print worker ended without an answer".to_owned();
                if running.id.is_some() {
                    Done::Printed(Err(why))
                } else {
                    Done::Destinations(Err(why))
                }
            }
        };
        let Some(running) = self.panes.shell.print.running.take() else {
            return moved;
        };
        match done {
            Done::Destinations(result) => {
                let printing = &mut self.panes.shell.print;
                printing.destinations = Some(match result {
                    Ok(list) => list,
                    Err(why) => {
                        log::info!("print: no destinations: {why}");
                        Vec::new()
                    }
                });
                log::info!(
                    "print: {} destination(s), {}",
                    printing.known().len(),
                    printing.summary()
                );
            }
            Done::Printed(result) => {
                let Running {
                    id, destination, ..
                } = running;
                #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
                let seconds = running.started.elapsed().as_secs_f64();
                #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
                let seconds = 0.0;
                let ended = match result {
                    Ok(printed) => {
                        self.panes.shell.print.last = Some(Last {
                            job: Some(printed.job.clone()),
                            destination: destination.clone(),
                            status: printed.status.clone(),
                        });
                        let mut outcome = accord::Outcome::default();
                        let result = &mut outcome.result;
                        result.insert("job".to_owned(), Value::Text(printed.job));
                        result.insert("destination".to_owned(), Value::Text(destination));
                        result.insert("status".to_owned(), Value::Text(printed.status));
                        result.insert("width".to_owned(), Value::Int(i64::from(printed.width)));
                        result.insert("height".to_owned(), Value::Int(i64::from(printed.height)));
                        result.insert("dpi".to_owned(), Value::Num(f64::from(printed.dpi)));
                        result.insert(
                            "bytes".to_owned(),
                            Value::Int(i64::try_from(printed.bytes).unwrap_or(i64::MAX)),
                        );
                        result.insert("seconds".to_owned(), Value::Num(seconds));
                        Ok(outcome)
                    }
                    Err(why) => {
                        self.panes.shell.print.last = Some(Last {
                            job: None,
                            destination,
                            status: format!("failed: {why}"),
                        });
                        Err(why)
                    }
                };
                log::info!("print: {}", self.panes.shell.print.summary());
                if let Some(id) = id {
                    self.end_print(id, ended);
                }
            }
        }
        true
    }

    /// Ends request `id` through the gallery's dispatcher, at its clock.
    fn end_print(&mut self, id: RequestId, ended: Result<accord::Outcome, String>) {
        let closed = self.panes.todo.with_dispatcher(|d, _, now| match ended {
            Ok(outcome) => d.complete(id, outcome, now),
            Err(why) => d.fail(id, why, now),
        });
        if let Err(e) = closed {
            log::warn!("print: the request did not end: {e}");
        }
    }
}

/// The threads: the look for destinations, and the job (the render on a
/// device of its own, the png, the spool). Desktop only.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod desktop {
    use std::sync::mpsc::{self, Receiver};
    use std::thread;
    use std::time::Instant;

    use accord::RequestId;
    use engine::drawing::compositor::Compositor;
    use engine::drawing::gpu::RenderConfig;
    use engine::drawing::headless::{Renderer, Viewport};
    use engine::runtime::platform::menu::locked;
    use engine::runtime::platform::print::{
        PAPER_DPI, POINTS_PER_INCH, PrintDocument, PrintError, PrintOptions, fit_scale, paper_scale,
    };

    use super::super::super::super::ShowcaseView;
    use super::{Done, Printed, Running, Shared};

    /// The scene of the main window as built for paper: the compositor,
    /// its viewport at the paper scale, and the clear.
    pub(super) struct Scene {
        compositor: Compositor,
        viewport: Viewport,
        clear_linear: [f32; 4],
        dpi: f32,
    }

    /// The destinations, asked on a thread of their own; a printer
    /// without a system (`Unsupported`) knows none.
    pub(super) fn spawn_lookup(printer: Shared) -> Receiver<Done> {
        let (tx, rx) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("print-destinations".to_owned())
            .spawn(move || {
                let found = match locked(&printer).destinations() {
                    Ok(list) => Ok(list),
                    Err(PrintError::Unsupported) => Ok(Vec::new()),
                    Err(e) => Err(e.to_string()),
                };
                let _ = tx.send(Done::Destinations(found));
            });
        if spawned.is_err() {
            return answered(Done::Destinations(Err(
                "could not start a thread to look for destinations".to_owned(),
            )));
        }
        rx
    }

    /// The job on a thread of its own: a headless renderer over a device
    /// of its own draws `scene` through the window's frame path and reads
    /// it back, the png is encoded, the document handed to `printer`, the
    /// job's place asked once.
    fn spawn_print(printer: Shared, scene: Scene, options: PrintOptions) -> Receiver<Done> {
        let (tx, rx) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("print-job".to_owned())
            .spawn(move || {
                let _ = tx.send(Done::Printed(print(printer, scene, &options)));
            });
        if spawned.is_err() {
            return answered(Done::Printed(Err(
                "could not start a thread to print".to_owned()
            )));
        }
        rx
    }

    fn print(printer: Shared, mut scene: Scene, options: &PrintOptions) -> Result<Printed, String> {
        let t = Instant::now();
        let mut renderer =
            Renderer::new(scene.viewport, RenderConfig::default()).map_err(|e| e.to_string())?;
        let device = t.elapsed();
        let t = Instant::now();
        let doc = PrintDocument::from_headless(
            &mut renderer,
            &mut scene.compositor,
            scene.clear_linear,
            scene.dpi,
        )
        .map_err(|e| e.to_string())?;
        let rendered = t.elapsed();
        let t = Instant::now();
        let job = locked(&printer)
            .print(&doc, options)
            .map_err(|e| e.to_string())?;
        let status = match locked(&printer).status(&job) {
            Ok(status) => status.to_string(),
            Err(e) => {
                log::info!("print: the status of {job} is not known: {e}");
                "unknown".to_owned()
            }
        };
        log::info!(
            "print: {job} queued: {}x{} at {} dpi, {} bytes; device {:?}, render and png {:?}, spool {:?}",
            doc.width,
            doc.height,
            doc.dpi,
            doc.png.len(),
            device,
            rendered,
            t.elapsed()
        );
        Ok(Printed {
            job: job.to_string(),
            status,
            width: doc.width,
            height: doc.height,
            dpi: doc.dpi,
            bytes: doc.png.len(),
        })
    }

    /// A channel that already holds `done`.
    fn answered(done: Done) -> Receiver<Done> {
        let (tx, rx) = mpsc::channel();
        let _ = tx.send(done);
        rx
    }

    impl ShowcaseView {
        /// The frame of the main window as it stands, built for paper:
        /// the same scene `render` builds, into a compositor of its own
        /// at `scale`, the main window's layers and scale put back after.
        fn render_for_print(&mut self, scale: f32) -> Compositor {
            let layers = self.layers.take();
            let scale_factor = self.scale_factor;
            self.scale_factor = scale;
            let mut compositor = Compositor::new();
            self.render(&mut compositor);
            self.layers = layers;
            self.scale_factor = scale_factor;
            compositor
        }

        /// The scene built now and the job started, for request `id`; a
        /// request with nothing to print to fails at once.
        pub(super) fn start_print_job(&mut self, id: RequestId) {
            let printing = &self.panes.shell.print;
            let (Some(printer), Some(destination)) = (
                printing.printer.clone(),
                printing.destination().map(str::to_owned),
            ) else {
                self.end_print(id, Err(PrintError::NoDestination.to_string()));
                return;
            };
            let max_side = engine::wgpu::Limits::default().max_texture_dimension_2d;
            let scale = fit_scale(self.width, self.height, paper_scale(PAPER_DPI), max_side);
            let dpi = scale * POINTS_PER_INCH;
            let t = Instant::now();
            let scene = Scene {
                compositor: self.render_for_print(scale),
                viewport: Viewport::new(self.width, self.height, scale),
                clear_linear: self.theme.colors.bg.to_linear_array(),
                dpi,
            };
            log::info!(
                "print: the {} page's scene built for paper at scale {scale:.3} ({} dpi) in {:?}",
                self.page_title(),
                dpi,
                t.elapsed()
            );
            let options = PrintOptions::default()
                .to(&destination)
                .title(format!("plev, {}", self.page_title()));
            let rx = spawn_print(printer, scene, options);
            self.panes.shell.print.running = Some(Running {
                id: Some(id),
                destination,
                rx,
                started: Instant::now(),
            });
        }
    }
}
