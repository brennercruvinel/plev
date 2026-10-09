//! The recorded demo of the video editor (task-69 phase 4c): the script
//! of the thesis on the Silences page, driven by an agent through the
//! port, with no window, and recorded the way the shell records a
//! session (`showcase::app::record`) when `PLEV_RECORD=1`.
//!
//! ```text
//! PLEV_RECORD=1 cargo run -p showcase --example demo -- [--out <file.monster>]
//!     [--review rules|claude] [--compare rules|claude] [--name <agent>]
//!     [--width <px>] [--height <px>]
//! ```
//!
//! This thread is the shell's (`shell`): it serves the port, ticks the
//! page (the worker's answers land in its wake), draws a frame on demand
//! into a compositor and never a window, and records each. A spawned
//! thread is the agent (`agent`), over `wire::Client`, every act resting
//! on a revision it was shown (`from_revision`). The script:
//!
//! 1. observe the page, with where each control shows;
//! 2. `detect_silence`: ffmpeg generates the test video when it is
//!    missing and detects the silences; the request ends when the worker
//!    answers, and the frames of the instant shown and of every cut
//!    follow;
//! 3. choose the proposer (`select` on the row's proposer), write the
//!    intent in the field and `propose`: the review opens beside the
//!    fixed screen, and its frames arrive as the page decodes them. The
//!    intent is the thesis' sentence, "cut the silences of this video,
//!    then I want to review the cuts", for the Claude; the rules read
//!    only the words of the app ("then" and "review" are not among them,
//!    task-69 phase 4b), so for them it is their sentence for the same
//!    screen, "the list with the video and the counts";
//! 4. apply the second cut through the generated surface (`activate` on
//!    its button, routed to the app's `apply_cut`);
//! 5. the person rejects the first cut on the generated surface too: a
//!    click, played by the shell's thread when the agent asks for it,
//!    recorded with the control under it;
//! 6. "compare the original with the cuts", `propose` again: the review
//!    becomes the comparison in place, the result's frame beside the
//!    original's;
//! 7. `close`.
//!
//! Without `PLEV_RECORD=1` the script runs and nothing is written. With
//! it, the window kept is the whole run (ten minutes, not the shell's
//! twenty seconds) and the file goes to `--out`, else to
//! `showcase::app::record::default_dir()`, whatever the script came to.
//! The `claude` proposer runs the real program (`PLEV_CLAUDE`, or the one
//! on the PATH): a real call, minutes and money, up to three per
//! proposal. Desktop only, like the port and the recording.

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod agent;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod shell;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run::main(&args) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("demo: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod run {
    use std::path::{Path, PathBuf};
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::thread;
    use std::time::{Duration, Instant};

    use engine::drawing::compositor::Compositor;
    use ::record::{Limits, Persisted};
    use uigallery::shell::window::record::{self, Recording};
    use uigallery::views::ShowcaseView;
    use socket::{AgentPort, Config};

    use crate::agent::agent;
    use crate::shell::{Answer, Poke, Shell};

    pub const USAGE: &str = "\
usage: PLEV_RECORD=1 cargo run -p showcase --example demo -- [--out <file.monster>]
           [--review rules|claude] [--compare rules|claude] [--name <agent>]
           [--width <px>] [--height <px>]
the script of the thesis on the Silences page, driven by an agent through
the port with no window: detect the silences (ffmpeg), the intent of the
thesis proposed by --review (the rules by default; claude runs the real
program, minutes and money), a cut applied by the agent and one rejected
by a person on the generated surface, the comparison proposed by
--compare, and close. with PLEV_RECORD=1 the whole run is recorded as the
shell records a session and written to --out (else to $TMPDIR/plev/sessions);
without it nothing is written. --name is the agent's name (demo). the
frame is 1600 by 1000 logical pixels unless --width and --height say
otherwise: at the window's launch size (1200 by 800) a row of the rules'
review is wider than its half and its buttons fall past the window's edge,
out of a click's reach.";

    /// The intent of the thesis, section 4, for the review: what the
    /// Claude is asked.
    const REVIEW: &str = "cut the silences of this video, then I want to review the cuts";
    /// The review as the rules read it: the words of the app alone (the
    /// Silences page has no "then" and no "review").
    const REVIEW_BY_RULES: &str = "the list with the video and the counts";
    /// The comparison, the same sentence for both proposers.
    pub const COMPARE: &str = "compare the original with the cuts";

    /// The frame the demo is drawn at unless asked otherwise: wider than
    /// the window's launch size (1200 by 800), where a row of the rules'
    /// review (a thumbnail, two texts, two buttons) is wider than its
    /// half and its buttons fall past the window's right edge.
    const WIDTH: f32 = 1600.0;
    const HEIGHT: f32 = 1000.0;

    /// How long one message to the port may take before the agent gives
    /// up: a proposal by the Claude runs minutes.
    pub const TIMEOUT_MS: u64 = 600_000;

    /// How often the shell's thread turns while nothing wakes it.
    pub const TURN: Duration = Duration::from_millis(8);

    /// How long the agent waits for the page's worker to go idle (the
    /// frames decoded) before it goes on.
    pub const IDLE_WAIT: Duration = Duration::from_secs(120);

    /// The recording keeps the whole run, not the shell's last twenty
    /// seconds.
    fn limits() -> Limits {
        Limits {
            max_age: 600_000,
            max_bytes: 64 << 20,
            ..Limits::default()
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Proposer {
        Rules,
        Claude,
    }

    impl Proposer {
        fn parse(text: &str) -> Result<Proposer, String> {
            match text {
                "rules" => Ok(Proposer::Rules),
                "claude" => Ok(Proposer::Claude),
                other => Err(format!("a proposer is rules or claude, got {other:?}")),
            }
        }

        /// The word the row's select takes.
        pub fn name(self) -> &'static str {
            match self {
                Proposer::Rules => "rules",
                Proposer::Claude => "claude",
            }
        }

        /// The sentence this proposer is asked for the review.
        pub fn review(self) -> &'static str {
            match self {
                Proposer::Rules => REVIEW_BY_RULES,
                Proposer::Claude => REVIEW,
            }
        }
    }

    pub struct Request {
        pub out: Option<PathBuf>,
        pub review: Proposer,
        pub compare: Proposer,
        pub name: String,
        pub width: f32,
        pub height: f32,
    }

    fn positive(flag: &str, raw: &str) -> Result<f32, String> {
        match raw.parse::<f32>() {
            Ok(v) if v.is_finite() && v > 0.0 => Ok(v),
            _ => Err(format!("{flag} takes a positive number, got {raw:?}")),
        }
    }

    fn parse(args: &[String]) -> Result<Option<Request>, String> {
        let mut req = Request {
            out: None,
            review: Proposer::Rules,
            compare: Proposer::Rules,
            name: "demo".to_owned(),
            width: WIDTH,
            height: HEIGHT,
        };
        let mut it = args.iter();
        while let Some(flag) = it.next() {
            let mut value = || {
                it.next()
                    .cloned()
                    .ok_or_else(|| format!("{flag} needs a value"))
            };
            match flag.as_str() {
                "-h" | "--help" => return Ok(None),
                "--out" => req.out = Some(PathBuf::from(value()?)),
                "--review" => req.review = Proposer::parse(&value()?)?,
                "--compare" => req.compare = Proposer::parse(&value()?)?,
                "--name" => req.name = value()?,
                "--width" => req.width = positive(flag, &value()?)?,
                "--height" => req.height = positive(flag, &value()?)?,
                other => return Err(format!("unknown argument {other:?}")),
            }
        }
        Ok(Some(req))
    }

    pub fn main(args: &[String]) -> Result<(), String> {
        let Some(req) = parse(args)? else {
            println!("{USAGE}");
            return Ok(());
        };
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("warn,showcase=info"),
        )
        .init();

        let mut view = ShowcaseView::new(req.width, req.height);
        view.jump_to_page("silences");
        if view.page_title() != "Silences" {
            return Err("the gallery has no Silences page".to_owned());
        }
        view.resize(req.width, req.height, 1.0);
        let recording = record::enabled().then(|| {
            let dir = req
                .out
                .as_deref()
                .and_then(Path::parent)
                .filter(|p| !p.as_os_str().is_empty())
                .map_or_else(record::default_dir, Path::to_path_buf);
            Recording::new(dir, limits())
        });
        if let Some(recording) = &recording {
            recording.attach(&mut view);
            println!(
                "recording: the session goes to {}",
                recording.next_path().display()
            );
        } else {
            println!("not recording: set PLEV_RECORD=1 to record the run");
        }

        let (tx, woke): (Sender<()>, Receiver<()>) = mpsc::channel();
        let port = AgentPort::open_with(Config::new("showcase"), move || {
            let _ = tx.send(());
        })
        .map_err(|e| format!("the port: {e}"))?;
        println!("agent port: {}", port.path().display());
        let path = port.path().to_owned();
        let mut shell = Shell {
            view,
            frame: Compositor::new(),
            port,
            recording,
            last: Instant::now(),
            frames: 0,
        };
        shell.draw();

        let (pokes, poked) = mpsc::channel::<(Poke, Answer)>();
        let started = Instant::now();
        let out = req.out.clone();
        let agent = thread::spawn(move || agent(&path, pokes, &req));
        while !agent.is_finished() {
            let _ = woke.recv_timeout(TURN);
            shell.turn();
            while let Ok((what, done)) = poked.try_recv() {
                let answer = match what {
                    Poke::Click(id) => shell.click(&id),
                };
                let _ = done.send(answer);
            }
        }
        let story = agent
            .join()
            .map_err(|_| "the agent's thread panicked".to_owned())
            .and_then(|story| story);
        shell.settle(Duration::from_secs(3));
        let took = started.elapsed();

        // saved whatever the script came to: a run that failed midway is
        // the one to inspect
        if let Some(recording) = shell.recording.as_mut() {
            let (path, written) = save(recording, out.as_deref())?;
            println!(
                "session saved: {} ({} bytes: events {}, index {}, chunks {}, assets {}, \
                 bindings {}; {} events, {} frames drawn in {} chunks, {} assets, {} images \
                 without pixels)",
                path.display(),
                written.bytes,
                written.events_bytes,
                written.index_bytes,
                written.chunks_bytes,
                written.assets_bytes,
                written.bindings_bytes,
                written.events,
                shell.frames,
                written.chunks,
                written.assets,
                written.images_without_pixels
            );
        }
        let story = story?;
        println!(
            "the script ran in {:.1} s: {} observations, {} frames drawn",
            took.as_secs_f64(),
            story.observations,
            shell.frames
        );
        println!(
            "  detect_silence: request {}; the cuts {:?}",
            story.detect, story.cuts
        );
        println!(
            "  review: request {} opened the surface {:?}",
            story.review, story.surface
        );
        println!(
            "  apply through the surface: request {} on {}",
            story.applied.0, story.applied.1
        );
        println!(
            "  the person's click: reject on {}",
            story.rejected_by_person
        );
        println!("  comparison: request {}", story.compare);
        println!("  close: request {}", story.close);
        Ok(())
    }

    /// The retained window written where the recording saves, then moved
    /// to `out` when one was asked (the same directory: the recording was
    /// made beside it).
    fn save(recording: &mut Recording, out: Option<&Path>) -> Result<(PathBuf, Persisted), String> {
        let (path, written) = recording
            .save()
            .map_err(|e| format!("the session is not saved: {e}"))?;
        let Some(out) = out else {
            return Ok((path, written));
        };
        std::fs::rename(&path, out)
            .map_err(|e| format!("{} to {}: {e}", path.display(), out.display()))?;
        Ok((out.to_path_buf(), written))
    }
}
