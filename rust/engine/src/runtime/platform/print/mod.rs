//! Printing as a capability (task-76 item 9): "print this page" behind a
//! trait, so an app spools to the system's printer and a test never does.
//!
//! A [`PrintDocument`] is a frame of the app on paper: the same scene the
//! window draws, rendered through the headless frame path
//! (`engine::headless`, the window's own) at a paper scale, dots per inch
//! over points per inch, since a logical pixel is a point on paper
//! ([`paper_scale`]), and encoded as png, which every CUPS filter chain
//! takes. The frame's svg (`engine::headless::svg`) may ride along
//! ([`PrintDocument::with_svg`]): it is the way toward a vector pdf, which
//! nothing here writes yet.
//!
//! [`Printer`] lists the destinations, prints a document with the few
//! options a page needs (the destination, the copies, the title, fit to
//! page, the media) and asks where a job is. [`CupsPrinter`] is the
//! desktop unix one: `lp` and `lpstat` as child processes with a deadline
//! (`cups.rs`); [`FakePrinter`] answers what a test loaded and keeps every
//! document it was given; [`NoPrinter`] is windows until its bring-up,
//! wasm and mobile: `Unsupported`. The same shape as `alerts.rs` and
//! `notify.rs`, with [`default_printer`] choosing.
//!
//! Printing is a person's: an app that offers it declares the capability
//! for a person alone, as it does for a window or a file dialog, and an
//! agent reads what the app projects about the job.

use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
use crate::drawing::compositor::Compositor;
#[cfg(not(target_arch = "wasm32"))]
use crate::drawing::headless::{Frame, HeadlessError, Renderer};

use super::menu::locked;

#[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
mod cups;
#[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
pub use cups::{CupsPrinter, LP_ENV, LPSTAT_ENV};

/// A point is a seventy-second of an inch: what a logical pixel is on
/// paper.
pub const POINTS_PER_INCH: f32 = 72.0;

/// The paper resolution a page is rendered at unless asked otherwise:
/// what an office printer prints, sharp body text at reading distance.
pub const PAPER_DPI: f32 = 300.0;

/// The render scale that puts `dpi` dots in every point: `300 / 72` is
/// `4.1667`, so a `1200 x 800` window is `5000 x 3333` dots.
pub fn paper_scale(dpi: f32) -> f32 {
    dpi / POINTS_PER_INCH
}

/// The largest scale, `scale` at most, at which `width x height` logical
/// pixels fit a texture of `max_side` pixels per side (the device's
/// `max_texture_dimension_2d`; `wgpu::Limits::default()` asks for 8192).
/// Rounded down to a hundredth so the physical size never rounds past.
pub fn fit_scale(width: f32, height: f32, scale: f32, max_side: u32) -> f32 {
    let side = width.max(height);
    if side <= 0.0 || !side.is_finite() {
        return scale;
    }
    let fits = max_side as f32 / side;
    if scale <= fits {
        scale
    } else {
        (fits * 100.0).floor() / 100.0
    }
}

/// A printer the system knows, by the name `lp -d` takes; `default` for
/// the one a print goes to when none is named.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Destination {
    pub name: String,
    pub default: bool,
}

impl Destination {
    pub fn new(name: impl Into<String>, default: bool) -> Destination {
        Destination {
            name: name.into(),
            default,
        }
    }
}

/// The few options a page asks with: the destination (`None` for the
/// system's default), the copies, the job's title (what the queue shows),
/// whether the document is scaled to the paper, and the media (`a4`,
/// `letter`; `None` for the printer's own).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrintOptions {
    pub destination: Option<String>,
    pub copies: u32,
    pub title: String,
    pub fit_to_page: bool,
    pub media: Option<String>,
}

impl Default for PrintOptions {
    /// One copy, titled "plev", fit to the page, on the printer's media.
    fn default() -> PrintOptions {
        PrintOptions {
            destination: None,
            copies: 1,
            title: "plev".to_owned(),
            fit_to_page: true,
            media: None,
        }
    }
}

impl PrintOptions {
    pub fn to(mut self, destination: impl Into<String>) -> PrintOptions {
        self.destination = Some(destination.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> PrintOptions {
        self.title = title.into();
        self
    }

    pub fn copies(mut self, copies: u32) -> PrintOptions {
        self.copies = copies.max(1);
        self
    }

    pub fn fit_to_page(mut self, fit: bool) -> PrintOptions {
        self.fit_to_page = fit;
        self
    }

    pub fn media(mut self, media: impl Into<String>) -> PrintOptions {
        self.media = Some(media.into());
        self
    }
}

/// A frame on paper: the png of the rendered frame, its size in dots and
/// the dots per inch it was rendered for, and the frame's svg when it was
/// asked for (the path to a vector pdf, not walked yet).
#[derive(Clone, PartialEq)]
pub struct PrintDocument {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub dpi: f32,
    pub svg: Option<String>,
}

impl fmt::Debug for PrintDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrintDocument")
            .field("png_bytes", &self.png.len())
            .field("width", &self.width)
            .field("height", &self.height)
            .field("dpi", &self.dpi)
            .field("svg_bytes", &self.svg.as_ref().map(String::len))
            .finish()
    }
}

impl PrintDocument {
    /// `frame`, read back from the headless frame path, as a document at
    /// `dpi`: the png encoded here.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_frame(frame: &Frame, dpi: f32) -> Result<PrintDocument, HeadlessError> {
        Ok(PrintDocument {
            png: frame.encode_png()?,
            width: frame.width,
            height: frame.height,
            dpi,
            svg: None,
        })
    }

    /// The scene in `compositor` drawn by `renderer` (built for the paper
    /// viewport: the page's logical size at [`paper_scale`]) and read
    /// back, as a document at `dpi`. `clear_linear` is the page color in
    /// linear space, as `Renderer::render` takes it.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_headless(
        renderer: &mut Renderer,
        compositor: &mut Compositor,
        clear_linear: [f32; 4],
        dpi: f32,
    ) -> Result<PrintDocument, HeadlessError> {
        let frame = renderer.render(compositor, clear_linear)?;
        PrintDocument::from_frame(&frame, dpi)
    }

    /// The frame's svg beside the png.
    pub fn with_svg(mut self, svg: impl Into<String>) -> PrintDocument {
        self.svg = Some(svg.into());
        self
    }

    /// The paper the document covers at its dpi, in inches.
    pub fn paper_size(&self) -> (f32, f32) {
        (self.width as f32 / self.dpi, self.height as f32 / self.dpi)
    }
}

/// A job in the system's queue, by the id it was given (`Office-42` in
/// CUPS: the destination and a number).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JobId(pub String);

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Where a job is: waiting or printing, printed, or no longer in the
/// queue nor in its history (cancelled, or expired from it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobStatus {
    Queued,
    Completed,
    Gone,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            JobStatus::Queued => "queued",
            JobStatus::Completed => "completed",
            JobStatus::Gone => "gone",
        })
    }
}

/// Why nothing was printed, or nothing answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrintError {
    /// This target has no printing system (windows until its bring-up,
    /// wasm, android, ios).
    Unsupported,
    /// No destination was named and the system has no default.
    NoDestination,
    /// The named destination is not one the system knows.
    UnknownDestination(String),
    /// The printing system refused, in its words (the last line it said).
    Spooler(String),
    /// A child did not finish within the deadline and was killed.
    Timeout { program: String, secs: u64 },
    /// The document could not be written, or the program not started.
    Io(String),
}

impl fmt::Display for PrintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrintError::Unsupported => f.write_str("print: no printing system on this target"),
            PrintError::NoDestination => f.write_str("print: no destination to print to"),
            PrintError::UnknownDestination(name) => {
                write!(f, "print: no destination named {name}")
            }
            PrintError::Spooler(why) => write!(f, "the printing system refused: {why}"),
            PrintError::Timeout { program, secs } => {
                write!(f, "{program} did not finish in {secs} s: killed")
            }
            PrintError::Io(why) => write!(f, "print: {why}"),
        }
    }
}

impl std::error::Error for PrintError {}

/// What a printer answers.
pub type Shown<T> = Result<T, PrintError>;

/// The system's printing. `destinations` lists what it knows; `print`
/// hands a document to the queue and returns at once with the job's id;
/// `status` asks where that job is now. Each call may block on the
/// system (a child process, a socket): the app calls from a thread beside
/// its loop, and never from `execute`.
pub trait Printer {
    fn destinations(&mut self) -> Shown<Vec<Destination>>;
    fn print(&mut self, document: &PrintDocument, options: &PrintOptions) -> Shown<JobId>;
    fn status(&mut self, job: &JobId) -> Shown<JobStatus>;
}

// -- the fake ---------------------------------------------------------------

/// One print the fake took: the document as given, the options, and the
/// id it answered with.
#[derive(Clone, Debug, PartialEq)]
pub struct Printed {
    pub document: PrintDocument,
    pub options: PrintOptions,
    pub job: JobId,
}

/// The printer of a test: the destinations it was loaded with, every
/// document it was given, and the refusals and statuses a test lined up.
/// Without a destination it refuses as a machine without one does; a job
/// it printed is `Queued` until a test says otherwise.
#[derive(Debug, Default)]
pub struct FakePrinter {
    destinations: Vec<Destination>,
    printed: Vec<Printed>,
    refusals: VecDeque<PrintError>,
    statuses: VecDeque<JobStatus>,
    next: u64,
}

impl FakePrinter {
    pub fn new() -> FakePrinter {
        FakePrinter::default()
    }

    /// A destination the fake knows; the default when `default`.
    pub fn destination(mut self, name: impl Into<String>, default: bool) -> FakePrinter {
        self.destinations.push(Destination::new(name, default));
        self
    }

    /// The next `print` that reaches the spooler (a destination picked)
    /// is refused with `why`, and nothing is kept.
    pub fn will_refuse(mut self, why: PrintError) -> FakePrinter {
        self.refusals.push_back(why);
        self
    }

    /// The next `status` answers `status`, whatever the job.
    pub fn will_report(mut self, status: JobStatus) -> FakePrinter {
        self.statuses.push_back(status);
        self
    }

    /// Every print taken so far, in order.
    pub fn printed(&self) -> &[Printed] {
        &self.printed
    }

    /// Every print taken so far, leaving none.
    pub fn take_printed(&mut self) -> Vec<Printed> {
        std::mem::take(&mut self.printed)
    }

    /// The destination a print with `wanted` goes to.
    fn pick(&self, wanted: Option<&str>) -> Shown<String> {
        match wanted {
            Some(name) => self
                .destinations
                .iter()
                .find(|d| d.name == name)
                .map(|d| d.name.clone())
                .ok_or_else(|| PrintError::UnknownDestination(name.to_owned())),
            None => self
                .destinations
                .iter()
                .find(|d| d.default)
                .map(|d| d.name.clone())
                .ok_or(PrintError::NoDestination),
        }
    }
}

impl Printer for FakePrinter {
    fn destinations(&mut self) -> Shown<Vec<Destination>> {
        Ok(self.destinations.clone())
    }

    fn print(&mut self, document: &PrintDocument, options: &PrintOptions) -> Shown<JobId> {
        // a print without a destination never reaches the spooler
        let destination = self.pick(options.destination.as_deref())?;
        if let Some(why) = self.refusals.pop_front() {
            return Err(why);
        }
        self.next += 1;
        let job = JobId(format!("{destination}-{}", self.next));
        self.printed.push(Printed {
            document: document.clone(),
            options: options.clone(),
            job: job.clone(),
        });
        Ok(job)
    }

    fn status(&mut self, job: &JobId) -> Shown<JobStatus> {
        if let Some(status) = self.statuses.pop_front() {
            return Ok(status);
        }
        Ok(if self.printed.iter().any(|p| p.job == *job) {
            JobStatus::Queued
        } else {
            JobStatus::Gone
        })
    }
}

// -- no printing at all -------------------------------------------------------

/// The printer of a target without a printing system: windows until its
/// bring-up (no `lp`, and no dependency here), wasm (the browser's print
/// is another capability) and mobile. Every call is `Unsupported`, and
/// the app says so where it would have offered to print.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoPrinter;

impl Printer for NoPrinter {
    fn destinations(&mut self) -> Shown<Vec<Destination>> {
        Err(PrintError::Unsupported)
    }

    fn print(&mut self, _document: &PrintDocument, _options: &PrintOptions) -> Shown<JobId> {
        Err(PrintError::Unsupported)
    }

    fn status(&mut self, _job: &JobId) -> Shown<JobStatus> {
        Err(PrintError::Unsupported)
    }
}

/// A printer shared by handle: the app's thread beside the loop prints
/// through one `Arc` while a test reads what was printed through the
/// other. A poisoned lock still answers: the printer holds no invariant
/// a panic elsewhere could break.
impl<P: Printer> Printer for Arc<Mutex<P>> {
    fn destinations(&mut self) -> Shown<Vec<Destination>> {
        locked(self).destinations()
    }

    fn print(&mut self, document: &PrintDocument, options: &PrintOptions) -> Shown<JobId> {
        locked(self).print(document, options)
    }

    fn status(&mut self, job: &JobId) -> Shown<JobStatus> {
        locked(self).status(job)
    }
}

/// A boxed printer is one too, so `Arc<Mutex<Box<dyn Printer + Send>>>`
/// shares whatever `default_printer` chose.
impl<P: Printer + ?Sized> Printer for Box<P> {
    fn destinations(&mut self) -> Shown<Vec<Destination>> {
        (**self).destinations()
    }

    fn print(&mut self, document: &PrintDocument, options: &PrintOptions) -> Shown<JobId> {
        (**self).print(document, options)
    }

    fn status(&mut self, job: &JobId) -> Shown<JobStatus> {
        (**self).status(job)
    }
}

/// The printer of the current target: CUPS through `lp` and `lpstat` on
/// the desktop unixes (macOS, linux), none elsewhere. Building it runs no
/// process; the first call does.
pub fn default_printer() -> Box<dyn Printer + Send> {
    #[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
    {
        Box::new(CupsPrinter::new())
    }
    #[cfg(not(all(unix, not(any(target_os = "android", target_os = "ios")))))]
    {
        Box::new(NoPrinter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page() -> PrintDocument {
        PrintDocument {
            png: vec![0x89, b'P', b'N', b'G'],
            width: 5000,
            height: 3333,
            dpi: PAPER_DPI,
            svg: None,
        }
    }

    #[test]
    fn a_point_on_paper_is_a_logical_pixel() {
        assert!((paper_scale(300.0) - 4.1667).abs() < 0.001);
        assert_eq!(paper_scale(72.0), 1.0);
        assert_eq!(paper_scale(144.0), 2.0);
        let (w, h) = page().paper_size();
        assert!((w - 16.667).abs() < 0.01, "{w}");
        assert!((h - 11.11).abs() < 0.01, "{h}");
    }

    #[test]
    fn the_scale_that_fits_the_device_is_the_wanted_one_or_the_largest_under_it() {
        let paper = paper_scale(PAPER_DPI);
        assert_eq!(fit_scale(1200.0, 800.0, paper, 8192), paper, "5000 fits");
        let fitted = fit_scale(2000.0, 800.0, paper, 8192);
        assert!((fitted - 4.09).abs() < 0.001, "{fitted}");
        assert!((2000.0 * fitted).round() as u32 <= 8192);
        assert_eq!(fit_scale(0.0, 0.0, paper, 8192), paper, "nothing to fit");
        assert_eq!(fit_scale(8192.0, 10.0, 3.0, 8192), 1.0);
    }

    #[test]
    fn the_options_default_to_one_fitted_copy_titled_plev() {
        let options = PrintOptions::default();
        assert_eq!(options.destination, None);
        assert_eq!(options.copies, 1);
        assert_eq!(options.title, "plev");
        assert!(options.fit_to_page);
        assert_eq!(options.media, None);
        let options = PrintOptions::default()
            .to("Office")
            .title("plev, Todo")
            .copies(0)
            .fit_to_page(false)
            .media("a4");
        assert_eq!(options.destination.as_deref(), Some("Office"));
        assert_eq!(options.copies, 1, "never fewer than one");
        assert!(!options.fit_to_page);
        assert_eq!(options.media.as_deref(), Some("a4"));
    }

    #[test]
    fn the_document_debugs_by_its_sizes_and_carries_an_svg() {
        let doc = page().with_svg("<svg/>");
        let text = format!("{doc:?}");
        assert!(text.contains("png_bytes: 4"), "{text}");
        assert!(text.contains("svg_bytes: Some(6)"), "{text}");
        assert!(
            !text.contains("137"),
            "the bytes themselves stay out: {text}"
        );
    }

    #[test]
    fn the_fake_prints_to_the_default_or_the_named_destination_and_keeps_the_document() {
        let mut fake = FakePrinter::new()
            .destination("Office", true)
            .destination("Home", false);
        assert_eq!(
            fake.destinations().unwrap(),
            [
                Destination::new("Office", true),
                Destination::new("Home", false)
            ]
        );
        let first = fake.print(&page(), &PrintOptions::default()).unwrap();
        assert_eq!(first, JobId("Office-1".to_owned()));
        let second = fake
            .print(&page(), &PrintOptions::default().to("Home").copies(2))
            .unwrap();
        assert_eq!(second.to_string(), "Home-2");
        assert_eq!(fake.status(&first).unwrap(), JobStatus::Queued);
        assert_eq!(
            fake.status(&JobId("Office-9".to_owned())).unwrap(),
            JobStatus::Gone
        );
        assert_eq!(fake.printed().len(), 2);
        assert_eq!(fake.printed()[0].document, page());
        assert_eq!(fake.printed()[1].options.copies, 2);
        assert_eq!(fake.printed()[1].job, second);
        assert_eq!(fake.take_printed().len(), 2);
        assert!(fake.printed().is_empty(), "taken");
    }

    #[test]
    fn the_fake_refuses_without_a_destination_an_unknown_one_and_what_was_lined_up() {
        let mut none = FakePrinter::new();
        assert_eq!(none.destinations().unwrap(), []);
        assert_eq!(
            none.print(&page(), &PrintOptions::default()),
            Err(PrintError::NoDestination)
        );
        assert!(none.printed().is_empty(), "nothing kept");
        let mut fake = FakePrinter::new()
            .destination("Office", false)
            .will_refuse(PrintError::Spooler("out of paper".to_owned()))
            .will_report(JobStatus::Completed);
        assert_eq!(
            fake.print(&page(), &PrintOptions::default()),
            Err(PrintError::NoDestination),
            "a destination that is not the default is not picked for it"
        );
        assert_eq!(
            fake.print(&page(), &PrintOptions::default().to("Nowhere")),
            Err(PrintError::UnknownDestination("Nowhere".to_owned()))
        );
        assert_eq!(
            fake.print(&page(), &PrintOptions::default().to("Office")),
            Err(PrintError::Spooler("out of paper".to_owned())),
            "the lined-up refusal is spent on the first print that reaches it"
        );
        let job = fake
            .print(&page(), &PrintOptions::default().to("Office"))
            .unwrap();
        assert_eq!(fake.status(&job).unwrap(), JobStatus::Completed, "lined up");
        assert_eq!(fake.status(&job).unwrap(), JobStatus::Queued, "spent");
        assert_eq!(
            PrintError::UnknownDestination("Nowhere".to_owned()).to_string(),
            "print: no destination named Nowhere"
        );
        assert_eq!(
            PrintError::Timeout {
                program: "lp".to_owned(),
                secs: 20
            }
            .to_string(),
            "lp did not finish in 20 s: killed"
        );
        assert_eq!(JobStatus::Gone.to_string(), "gone");
    }

    #[test]
    fn a_shared_fake_prints_through_one_handle_and_reports_through_the_other() {
        let fake = Arc::new(Mutex::new(FakePrinter::new().destination("Office", true)));
        let mut app: Box<dyn Printer + Send> = Box::new(fake.clone());
        let job = app.print(&page(), &PrintOptions::default()).unwrap();
        assert_eq!(locked(&fake).printed()[0].job, job);
        let shared = Arc::new(Mutex::new(Box::new(NoPrinter) as Box<dyn Printer + Send>));
        let mut through: Box<dyn Printer + Send> = Box::new(shared.clone());
        assert_eq!(through.destinations(), Err(PrintError::Unsupported));
    }

    #[test]
    fn no_printer_is_unsupported_for_every_call() {
        let mut none = NoPrinter;
        assert_eq!(none.destinations(), Err(PrintError::Unsupported));
        assert_eq!(
            none.print(&page(), &PrintOptions::default()),
            Err(PrintError::Unsupported)
        );
        assert_eq!(
            none.status(&JobId("x-1".to_owned())),
            Err(PrintError::Unsupported)
        );
        assert_eq!(
            PrintError::Unsupported.to_string(),
            "print: no printing system on this target"
        );
        // built, never asked: building runs no process
        let _printer = default_printer();
    }
}
