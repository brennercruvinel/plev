//! CUPS as child processes: `lpstat -p -d` for the destinations, `lp` to
//! print, `lpstat -W <which> -o <destination>` for a job's place in its
//! queue. Every call is one child with a deadline past which it is killed
//! by its pid (the form of the showcase's ffmpeg worker, not a dependency
//! on it), never a library: the engine takes no dependency for printing,
//! and the two programs are what every desktop unix with CUPS has (macOS
//! ships them; linux with cups installed). `PLEV_LP` and `PLEV_LPSTAT`
//! name other programs (a test's fakes); without them `lp` and `lpstat`
//! are looked up on the PATH by the spawn.
//!
//! What this machine, with no destination, said on 2026-09-24:
//! `lpstat -p -d` prints "lpstat: No destinations added." on stderr and
//! "no system default destination" on stdout, exit 1, in 0.14 s; `lp
//! <file>` says "lp: Error - No default destination.", exit 1; `lp -d
//! Nowhere <file>` says "lp: No such file or directory", exit 1 (about
//! the destination, the file was there); `lp -d Nowhere /missing.png`
//! says "lp: Error - unable to access ... - No such file or directory";
//! `lpstat -o <job-id>` is refused ("Invalid destination name in list"),
//! so a job's status is read from its destination's queue, where a job is
//! a line starting with its id.

use std::ffi::OsString;
use std::fs;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use super::{
    Destination, JobId, JobStatus, PrintDocument, PrintError, PrintOptions, Printer, Shown,
};

/// The variables that name another `lp` and `lpstat` (a test's fakes).
pub const LP_ENV: &str = "PLEV_LP";
pub const LPSTAT_ENV: &str = "PLEV_LPSTAT";

/// How long a child may run before it is killed. `lp` copies the file
/// into the spool and returns; `lpstat` asks the scheduler. Both take a
/// fraction of a second on a machine whose scheduler answers; a scheduler
/// that does not is what the deadline is for.
const TIMEOUT: Duration = Duration::from_secs(20);

/// How often the child is asked whether it has exited.
const POLL: Duration = Duration::from_millis(10);

/// How long the end of a child's output is waited for once it exited.
const GRACE: Duration = Duration::from_secs(1);

/// The printing system through `lp` and `lpstat`.
pub struct CupsPrinter {
    lp: PathBuf,
    lpstat: PathBuf,
    timeout: Duration,
    /// Where a document is written for `lp` to take; made private to the
    /// user on the first print.
    spool: PathBuf,
    /// Documents written so far: the next file's number.
    spooled: u64,
}

impl Default for CupsPrinter {
    fn default() -> CupsPrinter {
        CupsPrinter::new()
    }
}

impl CupsPrinter {
    /// `lp` and `lpstat`, or what `PLEV_LP` and `PLEV_LPSTAT` name. Runs
    /// nothing.
    pub fn new() -> CupsPrinter {
        let (lp, lpstat) = programs_from(std::env::var_os(LP_ENV), std::env::var_os(LPSTAT_ENV));
        CupsPrinter::with(lp, lpstat)
    }

    /// These two programs (a test's fakes).
    pub fn with(lp: impl Into<PathBuf>, lpstat: impl Into<PathBuf>) -> CupsPrinter {
        CupsPrinter {
            lp: lp.into(),
            lpstat: lpstat.into(),
            timeout: TIMEOUT,
            spool: std::env::temp_dir().join("plev-print"),
            spooled: 0,
        }
    }

    /// The deadline for every child.
    pub fn timeout(mut self, timeout: Duration) -> CupsPrinter {
        self.timeout = timeout;
        self
    }

    /// Where the document is written for `lp` (`$TMPDIR/plev-print` by
    /// default); a test reads what its fake `lp` left beside it.
    pub fn spool_dir(mut self, dir: impl Into<PathBuf>) -> CupsPrinter {
        self.spool = dir.into();
        self
    }

    pub fn lp(&self) -> &Path {
        &self.lp
    }

    pub fn lpstat(&self) -> &Path {
        &self.lpstat
    }

    /// The document as a file `lp` can take, in the spool directory.
    fn write_document(&mut self, document: &PrintDocument) -> Result<PathBuf, PrintError> {
        private_dir(&self.spool)
            .map_err(|e| PrintError::Io(format!("{}: {e}", self.spool.display())))?;
        let path = self
            .spool
            .join(format!("plev-{}-{}.png", std::process::id(), self.spooled));
        self.spooled += 1;
        fs::write(&path, &document.png)
            .map_err(|e| PrintError::Io(format!("{}: {e}", path.display())))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
        Ok(path)
    }
}

impl Printer for CupsPrinter {
    fn destinations(&mut self) -> Shown<Vec<Destination>> {
        let exit = run(&self.lpstat, &["-p".into(), "-d".into()], self.timeout)?;
        parse_destinations(&exit.stdout(), &exit.stderr(), exit.success)
    }

    fn print(&mut self, document: &PrintDocument, options: &PrintOptions) -> Shown<JobId> {
        let file = self.write_document(document)?;
        let ran = run(&self.lp, &lp_args(options, &file), self.timeout);
        // lp copied the file into the spool, or never took it
        let _ = fs::remove_file(&file);
        let exit = ran?;
        if !exit.success {
            return Err(print_refusal(&exit.stderr(), options));
        }
        let stdout = exit.stdout();
        parse_job_id(&stdout)
            .ok_or_else(|| PrintError::Spooler(format!("lp gave no request id: {}", tail(&stdout))))
    }

    fn status(&mut self, job: &JobId) -> Shown<JobStatus> {
        let destination = destination_of(job)
            .ok_or_else(|| PrintError::Spooler(format!("{job} names no destination")))?
            .to_owned();
        for (which, status) in [
            ("not-completed", JobStatus::Queued),
            ("completed", JobStatus::Completed),
        ] {
            let args: [OsString; 4] = [
                "-W".into(),
                which.into(),
                "-o".into(),
                destination.clone().into(),
            ];
            let exit = run(&self.lpstat, &args, self.timeout)?;
            if !exit.success {
                return Err(PrintError::Spooler(tail(&exit.stderr())));
            }
            if lists_job(&exit.stdout(), job) {
                return Ok(status);
            }
        }
        Ok(JobStatus::Gone)
    }
}

/// The programs: what the two variables name, or `lp` and `lpstat` for
/// the PATH.
pub(crate) fn programs_from(lp: Option<OsString>, lpstat: Option<OsString>) -> (PathBuf, PathBuf) {
    (
        lp.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("lp")),
        lpstat
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("lpstat")),
    )
}

/// `lpstat -p -d` read: a destination per `printer <name> ...` line, the
/// default from `system default destination: <name>`. A run that ended
/// saying "No destinations added" is an empty list, not an error: the
/// machine has none. Any other failure is the scheduler's word.
pub(crate) fn parse_destinations(
    stdout: &str,
    stderr: &str,
    success: bool,
) -> Shown<Vec<Destination>> {
    if !success && !stderr.contains("No destinations added") {
        return Err(PrintError::Spooler(tail(stderr)));
    }
    let mut out: Vec<Destination> = Vec::new();
    let mut default = None;
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("printer ") {
            if let Some(name) = rest.split_whitespace().next() {
                out.push(Destination::new(name, false));
            }
        } else if let Some(name) = line.strip_prefix("system default destination: ") {
            default = Some(name.trim().to_owned());
        }
    }
    if let Some(default) = default {
        for destination in &mut out {
            destination.default = destination.name == default;
        }
    }
    Ok(out)
}

/// The arguments of `lp` for `options` and `file`: `-d <destination>`
/// when one is named (without it `lp` prints to the system's default),
/// `-n <copies>` above one, `-t <title>`, `-o fit-to-page` when asked,
/// `-o media=<media>` when named, then `--` and the file.
pub(crate) fn lp_args(options: &PrintOptions, file: &Path) -> Vec<OsString> {
    let mut args: Vec<OsString> = Vec::new();
    if let Some(destination) = &options.destination {
        args.push("-d".into());
        args.push(destination.into());
    }
    if options.copies > 1 {
        args.push("-n".into());
        args.push(options.copies.to_string().into());
    }
    args.push("-t".into());
    args.push(options.title.as_str().into());
    if options.fit_to_page {
        args.push("-o".into());
        args.push("fit-to-page".into());
    }
    if let Some(media) = &options.media {
        args.push("-o".into());
        args.push(format!("media={media}").into());
    }
    args.push("--".into());
    args.push(file.as_os_str().to_owned());
    args
}

/// `request id is Office-42 (1 file(s))` -> `Office-42`.
pub(crate) fn parse_job_id(stdout: &str) -> Option<JobId> {
    let rest = stdout.split("request id is ").nth(1)?;
    let id = rest.split_whitespace().next()?;
    (!id.is_empty()).then(|| JobId(id.to_owned()))
}

/// The destination a CUPS job id names: everything before its last dash
/// (`Office-42` is job 42 of `Office`; a name may hold dashes itself).
pub(crate) fn destination_of(job: &JobId) -> Option<&str> {
    let (destination, number) = job.0.rsplit_once('-')?;
    (!destination.is_empty() && number.chars().all(|c| c.is_ascii_digit()) && !number.is_empty())
        .then_some(destination)
}

/// Whether an `lpstat -o` listing has `job`: a line that starts with the
/// id, followed by a space.
pub(crate) fn lists_job(stdout: &str, job: &JobId) -> bool {
    stdout.lines().any(|line| {
        line.strip_prefix(job.0.as_str())
            .is_some_and(|rest| rest.starts_with(char::is_whitespace))
    })
}

/// What `lp`'s refusal means for `options`: no default destination, a
/// destination it does not know (CUPS says "No such file or directory"
/// about it, and "unable to access" about a file), or its own last word.
pub(crate) fn print_refusal(stderr: &str, options: &PrintOptions) -> PrintError {
    if stderr.contains("No default destination") {
        PrintError::NoDestination
    } else if stderr.contains("No such file or directory") && !stderr.contains("unable to access") {
        PrintError::UnknownDestination(options.destination.clone().unwrap_or_default())
    } else {
        PrintError::Spooler(tail(stderr))
    }
}

/// The last line of an output that says something, for a message.
fn tail(text: &str) -> String {
    text.lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("no output")
        .to_owned()
}

/// `dir`, made when missing, private to the user on unix.
fn private_dir(dir: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)?;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))
    }
    #[cfg(not(unix))]
    {
        fs::create_dir_all(dir)
    }
}

/// What a child came to.
struct Exit {
    success: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Exit {
    fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

/// A pipe read on a thread of its own, into a buffer shared with the
/// caller, until its end: a child that leaves a grandchild holding the
/// pipe must not hold the caller with it.
struct Drain {
    bytes: Arc<Mutex<Vec<u8>>>,
    end: Receiver<()>,
}

impl Drain {
    fn new(from: Option<impl Read + Send + 'static>) -> Drain {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let (tx, end) = mpsc::channel();
        if let Some(mut from) = from {
            let into = Arc::clone(&bytes);
            thread::spawn(move || {
                let mut chunk = [0u8; 4096];
                loop {
                    match from.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(n) => {
                            if let Ok(mut all) = into.lock() {
                                all.extend_from_slice(&chunk[..n]);
                            }
                        }
                        Err(e) if e.kind() == ErrorKind::Interrupted => {}
                        Err(_) => break,
                    }
                }
                let _ = tx.send(());
            });
        }
        Drain { bytes, end }
    }

    /// All of it once the pipe ends, or what came before `until`.
    fn bytes(self, until: Instant) -> Vec<u8> {
        let _ = self
            .end
            .recv_timeout(until.saturating_duration_since(Instant::now()));
        self.bytes.lock().map(|b| b.clone()).unwrap_or_default()
    }
}

/// Kills `child` by its pid and reaps it.
fn kill(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Runs `program` with `args` and waits at most `timeout`: past it the
/// child is killed and the error says so.
fn run(program: &Path, args: &[OsString], timeout: Duration) -> Result<Exit, PrintError> {
    let name = program
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| program.display().to_string());
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| PrintError::Io(format!("could not start {}: {e}", program.display())))?;
    let stdout = Drain::new(child.stdout.take());
    let stderr = Drain::new(child.stderr.take());
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                kill(&mut child);
                return Err(PrintError::Timeout {
                    program: name,
                    secs: timeout.as_secs(),
                });
            }
            Ok(None) => thread::sleep(POLL),
            Err(e) => {
                kill(&mut child);
                return Err(PrintError::Io(format!("could not wait for {name}: {e}")));
            }
        }
    };
    let until = Instant::now() + GRACE;
    Ok(Exit {
        success: status.success(),
        stdout: stdout.bytes(until),
        stderr: stderr.bytes(until),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The listing of a machine with two printers, as `lpstat -p -d`
    /// prints it.
    const TWO: &str = "printer Office is idle.  enabled since Wed Sep 24 10:00:00 2026\n\
                       printer Home disabled since Wed Sep 24 09:00:00 2026 -\n\
                       \tPaused\n\
                       system default destination: Office\n";

    #[test]
    fn the_programs_come_from_the_variables_or_the_path() {
        let (lp, lpstat) = programs_from(None, None);
        assert_eq!(lp, PathBuf::from("lp"));
        assert_eq!(lpstat, PathBuf::from("lpstat"));
        let (lp, lpstat) = programs_from(Some("/x/fake-lp".into()), Some("/x/fake-lpstat".into()));
        assert_eq!(lp, PathBuf::from("/x/fake-lp"));
        assert_eq!(lpstat, PathBuf::from("/x/fake-lpstat"));
        let printer = CupsPrinter::with("/x/lp", "/x/lpstat").timeout(Duration::from_secs(3));
        assert_eq!(printer.lp(), Path::new("/x/lp"));
        assert_eq!(printer.lpstat(), Path::new("/x/lpstat"));
        assert_eq!(printer.timeout, Duration::from_secs(3));
    }

    #[test]
    fn the_destinations_are_read_off_the_listing_with_the_default_marked() {
        let found = parse_destinations(TWO, "", true).unwrap();
        assert_eq!(
            found,
            [
                Destination::new("Office", true),
                Destination::new("Home", false)
            ]
        );
        // this machine, 2026-09-24: no destination is an empty list
        let none = parse_destinations(
            "no system default destination\n",
            "lpstat: No destinations added.\n",
            false,
        )
        .unwrap();
        assert!(none.is_empty());
        // without a default nothing is marked
        let unmarked = parse_destinations("printer Office is idle.\n", "", true).unwrap();
        assert_eq!(unmarked, [Destination::new("Office", false)]);
        // the scheduler's own failure is an error in its words
        assert_eq!(
            parse_destinations("", "lpstat: Unable to connect to server\n", false),
            Err(PrintError::Spooler(
                "lpstat: Unable to connect to server".to_owned()
            ))
        );
    }

    #[test]
    fn the_lp_command_carries_the_destination_the_title_and_the_options() {
        let file = Path::new("/tmp/plev-print/plev-1-0.png");
        let full = PrintOptions::default()
            .to("Office")
            .title("plev, Todo")
            .copies(2)
            .media("a4");
        let args: Vec<String> = lp_args(&full, file)
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            [
                "-d",
                "Office",
                "-n",
                "2",
                "-t",
                "plev, Todo",
                "-o",
                "fit-to-page",
                "-o",
                "media=a4",
                "--",
                "/tmp/plev-print/plev-1-0.png"
            ]
        );
        let bare: Vec<String> = lp_args(&PrintOptions::default().fit_to_page(false), file)
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(bare, ["-t", "plev", "--", "/tmp/plev-print/plev-1-0.png"]);
    }

    #[test]
    fn the_job_id_is_read_off_lps_answer_and_names_its_destination() {
        let job = parse_job_id("request id is Office-42 (1 file(s))\n").unwrap();
        assert_eq!(job, JobId("Office-42".to_owned()));
        assert_eq!(destination_of(&job), Some("Office"));
        assert_eq!(
            destination_of(&JobId("Brother-HL-2270DW-7".to_owned())),
            Some("Brother-HL-2270DW")
        );
        assert_eq!(destination_of(&JobId("Office".to_owned())), None);
        assert_eq!(destination_of(&JobId("-42".to_owned())), None);
        assert_eq!(destination_of(&JobId("Office-x".to_owned())), None);
        assert_eq!(parse_job_id(""), None);
        assert_eq!(parse_job_id("request id is "), None);
    }

    #[test]
    fn a_job_is_listed_when_a_line_starts_with_its_id() {
        let listing = "Office-42               nn               1024   Wed Sep 24 10:00:00 2026\n\
                       Office-420              nn               2048   Wed Sep 24 10:01:00 2026\n";
        assert!(lists_job(listing, &JobId("Office-42".to_owned())));
        assert!(lists_job(listing, &JobId("Office-420".to_owned())));
        assert!(!lists_job(listing, &JobId("Office-4".to_owned())));
        assert!(!lists_job("", &JobId("Office-42".to_owned())));
    }

    #[test]
    fn lps_refusals_are_read_as_this_machine_gave_them() {
        let to_office = PrintOptions::default().to("Nowhere");
        assert_eq!(
            print_refusal(
                "lp: Error - No default destination.\n",
                &PrintOptions::default()
            ),
            PrintError::NoDestination
        );
        assert_eq!(
            print_refusal("lp: No such file or directory\n", &to_office),
            PrintError::UnknownDestination("Nowhere".to_owned())
        );
        assert_eq!(
            print_refusal(
                "lp: Error - unable to access \u{201c}/x.png\u{201d} - No such file or directory\n",
                &to_office
            ),
            PrintError::Spooler(
                "lp: Error - unable to access \u{201c}/x.png\u{201d} - No such file or directory"
                    .to_owned()
            )
        );
        assert_eq!(
            print_refusal("", &to_office),
            PrintError::Spooler("no output".to_owned())
        );
    }
}
