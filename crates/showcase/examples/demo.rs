//! The recorded demo of the video editor (task-69 phase 4c): the script
//! of the thesis on the Silences page, driven by an agent through the
//! port, with no window, and recorded the way the shell records a
//! session (`showcase::app::record`) when `PLEV_RECORD=1`.
//!
//! ```text
//! PLEV_RECORD=1 cargo run -p showcase --example demo -- [--out <file.monster>]
//!     [--review rules|claude] [--compare rules|claude] [--name <agent>]
//! ```
//!
//! This thread is the shell's: it serves the port, ticks the page (the
//! worker's answers land in its wake), draws a frame on demand into a
//! compositor and never a window, and records each. A spawned thread is
//! the agent, over `wire::Client`, every act resting on a revision it was
//! shown (`from_revision`). The script:
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
//! `showcase::app::record::default_dir()`. The `claude` proposer runs the
//! real program (`PLEV_CLAUDE`, or the one on the PATH): a real call,
//! minutes and money, up to three per proposal. Desktop only, like the
//! port and the recording.

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

    use comps::prelude::WidgetEvent;
    use engine::compositor::Compositor;
    use rec::{InputKind, Limits, Persisted};
    use sem::{ActionName, RequestId, SemId, Stage, StateKey, Value};
    use showcase::app::record::{self, Recording};
    use showcase::view::ShowcaseView;
    use wire::{Act, AgentPort, Client, Config, Observe, Reply, SnapshotReply, StageReply};

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
    const COMPARE: &str = "compare the original with the cuts";

    /// The frame the demo is drawn at unless asked otherwise: wider than
    /// the window's launch size (1200 by 800), where a row of the rules'
    /// review (a thumbnail, two texts, two buttons) is wider than its
    /// half and its buttons fall past the window's right edge.
    const WIDTH: f32 = 1600.0;
    const HEIGHT: f32 = 1000.0;

    /// How long one message to the port may take before the agent gives
    /// up: a proposal by the Claude runs minutes.
    const TIMEOUT_MS: u64 = 600_000;

    /// How often the shell's thread turns while nothing wakes it.
    const TURN: Duration = Duration::from_millis(8);

    /// How long the agent waits for the page's worker to go idle (the
    /// frames decoded) before it goes on.
    const IDLE_WAIT: Duration = Duration::from_secs(120);

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
    enum Proposer {
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
        fn name(self) -> &'static str {
            match self {
                Proposer::Rules => "rules",
                Proposer::Claude => "claude",
            }
        }

        /// The sentence this proposer is asked for the review.
        fn review(self) -> &'static str {
            match self {
                Proposer::Rules => REVIEW_BY_RULES,
                Proposer::Claude => REVIEW,
            }
        }
    }

    struct Request {
        out: Option<PathBuf>,
        review: Proposer,
        compare: Proposer,
        name: String,
        width: f32,
        height: f32,
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

    /// What the agent asks the shell's thread to do between two of its
    /// messages: the person at the window.
    enum Poke {
        /// A click on the middle of where `id` shows in the frame on
        /// screen.
        Click(SemId),
    }

    type Answer = Sender<Result<(), String>>;
    type Pokes = Sender<(Poke, Answer)>;

    fn poke(pokes: &Pokes, what: Poke) -> Result<(), String> {
        let (done, wait) = mpsc::channel();
        pokes
            .send((what, done))
            .map_err(|_| "the shell went away".to_owned())?;
        wait.recv()
            .map_err(|_| "the shell did not answer".to_owned())?
    }

    /// The shell without a window: the view, the frame on screen, the
    /// port, and the recording when there is one.
    struct Shell {
        view: ShowcaseView,
        frame: Compositor,
        port: AgentPort,
        recording: Option<Recording>,
        last: Instant,
        frames: usize,
    }

    impl Shell {
        /// A frame drawn, and recorded with the state it was drawn from.
        fn draw(&mut self) {
            self.view.render(&mut self.frame);
            if let Some(recording) = &self.recording {
                recording.frame(&mut self.view, &self.frame);
            }
            self.frames += 1;
        }

        /// One turn of the loop: the port served (an act records the
        /// state it left), the page ticked by the real time gone, and a
        /// frame when either changed something.
        fn turn(&mut self) -> bool {
            let dt = self.last.elapsed().as_secs_f32();
            self.last = Instant::now();
            let recorder = self.recording.as_ref().map(Recording::recorder);
            let acted = self
                .view
                .serve_agent_recorded(&mut self.port, &self.frame, recorder);
            let moved = self.view.tick(dt);
            if acted || moved {
                self.draw();
            }
            acted || moved
        }

        /// The person clicks the middle of `id` in the frame on screen:
        /// the press and the release recorded with the control under
        /// them, the state after each when it moved, and a frame.
        /// Where `id` shows in the frame on screen, whole: the person
        /// scrolls the page down with the wheel (over the intent row, a
        /// frame per notch) until the control is inside the window, as
        /// the generated surface's list starts below the fold at the
        /// launch size.
        fn reveal(&mut self, id: &SemId) -> Result<[f32; 4], String> {
            let wheel = self
                .frame
                .binding(&SemId::parse("silences_surface.intent").map_err(|e| e.to_string())?)
                .map(|b| (b.rect[0] + b.rect[2] / 2.0, b.rect[1] + b.rect[3] / 2.0))
                .ok_or_else(|| "the intent row does not show".to_owned())?;
            for _ in 0..60 {
                let bound = self
                    .frame
                    .binding(id)
                    .map(|b| (b.rect, b.visible_rect()))
                    .ok_or_else(|| format!("{id} is not bound in the frame on screen"))?;
                // whole within a pixel: the frame's clip is whole
                // physical pixels, the rect is not
                if let (rect, Some(visible)) = bound
                    && visible[2] >= rect[2] - 1.0
                    && visible[3] >= rect[3] - 1.0
                {
                    return Ok(visible);
                }
                let (x, y) = wheel;
                let scrolled = self
                    .view
                    .handle_event(&WidgetEvent::Scroll { x, y, delta: 48.0 });
                if !scrolled {
                    return Err(format!(
                        "{id} sits outside the window ({:?}, visible {:?}, the page scrolled to \
                         {}) and the page does not scroll further",
                        bound.0,
                        bound.1,
                        self.view.page_offset()
                    ));
                }
                self.draw();
            }
            Err(format!("{id} never came whole into the window"))
        }

        fn click(&mut self, id: &SemId) -> Result<(), String> {
            let [x, y, w, h] = self.reveal(id)?;
            let (x, y) = (x + w / 2.0, y + h / 2.0);
            if self.view.handle_event(&WidgetEvent::MouseMove { x, y }) {
                self.draw();
            }
            for (kind, event) in [
                (InputKind::PointerDown, WidgetEvent::MouseDown { x, y }),
                (InputKind::PointerUp, WidgetEvent::MouseUp { x, y }),
            ] {
                if let Some(recording) = &self.recording {
                    recording.input(&mut self.view, kind, Some([x, y]), &self.frame);
                }
                if self.view.handle_event(&event)
                    && let Some(recording) = &self.recording
                {
                    recording.state(&mut self.view);
                }
            }
            self.draw();
            Ok(())
        }

        /// Turns until the page has asked for nothing over a stretch (the
        /// exit of a closed surface fades out), at most `at_most`.
        fn settle(&mut self, at_most: Duration) {
            let start = Instant::now();
            let mut quiet = 0;
            while quiet < 30 && start.elapsed() < at_most {
                thread::sleep(TURN);
                if self.turn() {
                    quiet = 0;
                } else {
                    quiet += 1;
                }
            }
        }
    }

    /// What the agent asked, in order, for the summary.
    struct Story {
        observations: usize,
        detect: RequestId,
        cuts: Vec<String>,
        review: RequestId,
        surface: String,
        applied: (RequestId, SemId),
        rejected_by_person: SemId,
        compare: RequestId,
        close: RequestId,
    }

    fn look(c: &mut Client, observe: Observe) -> Result<SnapshotReply, String> {
        match c.observe(observe).map_err(|e| format!("observe: {e}"))? {
            Reply::Snapshot(s) => Ok(s),
            other => Err(format!("a snapshot was expected, got {other:?}")),
        }
    }

    /// The whole page as it is now, a new revision to rest the next act
    /// on.
    fn now(c: &mut Client) -> Result<SnapshotReply, String> {
        look(c, Observe::default())
    }

    fn act(
        c: &mut Client,
        target: &str,
        action: &str,
        param: Option<Value>,
        seen: &SnapshotReply,
    ) -> Result<Vec<StageReply>, String> {
        let target = SemId::parse(target).map_err(|e| e.to_string())?;
        let action = ActionName::parse(action).map_err(|e| e.to_string())?;
        let mut act = Act::from_revision(target.clone(), action.clone(), seen.revision);
        if let Some(param) = param {
            act = act.with_param(param);
        }
        c.act(act, |_| {})
            .map_err(|e| format!("{action} on {target}: {e}"))
    }

    /// The request's id when its last stage is `completed`; the stage
    /// otherwise, in words.
    fn done(stages: &[StageReply], what: &str) -> Result<RequestId, String> {
        match stages.last() {
            Some(last) if matches!(last.stage, Stage::Completed(_)) => Ok(last.request),
            Some(last) => Err(format!("{what} ended {:?}", last.stage)),
            None => Err(format!("{what}: no stage came back")),
        }
    }

    /// A text of the outcome of a completed request.
    fn outcome(stages: &[StageReply], key: &str) -> Option<String> {
        match &stages.last()?.stage {
            Stage::Completed(outcome) => match outcome.result.get(key)? {
                Value::Text(text) => Some(text.clone()),
                other => Some(format!("{other:?}")),
            },
            _ => None,
        }
    }

    /// The keys of the cuts, in the order the page shows them.
    fn cuts(seen: &SnapshotReply) -> Vec<String> {
        seen.nodes
            .iter()
            .find(|n| n.id.as_str() == "silences")
            .and_then(|n| n.state.get("cuts"))
            .and_then(|v| match v {
                Value::List(keys) => Some(keys.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    /// The node of the generated surface that runs `action` on the cut
    /// `key`: named by the rules `<surface>.silences.cut[<key>].<action>`;
    /// from the Claude, any node under the surface that carries the cut
    /// and `activate`s something called after the action.
    fn button(seen: &SnapshotReply, surface: &str, key: &str, action: &str) -> Option<SemId> {
        let own = format!("silences.cut[{key}]");
        let word = action.split('_').next().unwrap_or(action);
        seen.nodes
            .iter()
            .filter(|n| n.id.as_str() != own && n.id.as_str().contains(&format!("cut[{key}]")))
            .filter(|n| n.id.as_str().starts_with(surface) || n.id.as_str().starts_with(&own))
            .filter(|n| n.actions.iter().any(|a| a.name.as_str() == "activate"))
            .find(|n| n.id.as_str().ends_with(action) || n.label.to_lowercase().contains(word))
            .map(|n| n.id.clone())
    }

    /// Waits for the page's worker to be idle twice in a row (the frames
    /// the screen and the surface show are decoded), observing the status
    /// alone.
    fn idle(c: &mut Client, observations: &mut usize) -> Result<(), String> {
        let start = Instant::now();
        let mut idle = 0;
        thread::sleep(Duration::from_millis(300));
        while idle < 2 {
            if start.elapsed() > IDLE_WAIT {
                return Err("the page's worker never went idle".to_owned());
            }
            let status = look(
                c,
                Observe {
                    scope: Some(SemId::parse("silences.status").map_err(|e| e.to_string())?),
                    keys: Some(vec![StateKey::parse("busy").map_err(|e| e.to_string())?]),
                    ..Observe::default()
                },
            )?;
            *observations += 1;
            let busy = status
                .nodes
                .iter()
                .find_map(|n| n.state.get("busy"))
                .is_some_and(|v| *v == Value::Bool(true));
            idle = if busy { 0 } else { idle + 1 };
            thread::sleep(Duration::from_millis(200));
        }
        Ok(())
    }

    /// The agent's side of the script.
    fn agent(path: &Path, pokes: Pokes, req: &Request) -> Result<Story, String> {
        let mut c = Client::connect(path).map_err(|e| format!("connect: {e}"))?;
        c.set_timeout(Some(TIMEOUT_MS))
            .map_err(|e| format!("timeout: {e}"))?;
        c.hello(&req.name).map_err(|e| format!("hello: {e}"))?;
        let mut observations = 0;
        let mut seen = look(
            &mut c,
            Observe {
                bounds: true,
                ..Observe::default()
            },
        )?;
        observations += 1;

        // 2. the silences detected by ffmpeg; the frames follow
        let detected = act(&mut c, "silences.detect", "detect_silence", None, &seen)?;
        let detect = done(&detected, "detect_silence")?;
        idle(&mut c, &mut observations)?;
        seen = now(&mut c)?;
        observations += 1;
        let cuts = cuts(&seen);
        let [first, second, ..] = cuts.as_slice() else {
            return Err(format!(
                "the detection found {} cuts; two are needed",
                cuts.len()
            ));
        };

        // 3. the review, by the proposer chosen, for the intent of the thesis
        let chosen = Some(Value::Text(req.review.name().to_owned()));
        let selected = act(&mut c, "silences_surface.proposer", "select", chosen, &seen)?;
        done(&selected, "select the proposer")?;
        seen = now(&mut c)?;
        observations += 1;
        let intent = Some(Value::Text(req.review.review().to_owned()));
        let written = act(
            &mut c,
            "silences_surface.intent",
            "set_value",
            intent,
            &seen,
        )?;
        done(&written, "write the intent")?;
        seen = now(&mut c)?;
        observations += 1;
        let proposed = act(&mut c, "silences_surface.propose", "propose", None, &seen)?;
        let review = done(&proposed, "propose the review")?;
        let surface = outcome(&proposed, "surface").unwrap_or_default();
        idle(&mut c, &mut observations)?;

        // 4. the second cut applied through the generated surface: the agent acts by id, and its card sits below the fold at the launch size
        seen = look(
            &mut c,
            Observe {
                bounds: true,
                ..Observe::default()
            },
        )?;
        observations += 1;
        let apply = button(&seen, &surface, second, "apply_cut")
            .ok_or_else(|| format!("no apply button of cut {second} on the surface {surface}"))?;
        let applied = act(&mut c, apply.as_str(), "activate", None, &seen)?;
        let applied = (done(&applied, "apply the second cut")?, apply);

        // 5. the person rejects the first cut on the generated surface: its card shows
        seen = now(&mut c)?;
        observations += 1;
        let reject = button(&seen, &surface, first, "reject_cut")
            .ok_or_else(|| format!("no reject button of cut {first} on the surface {surface}"))?;
        poke(&pokes, Poke::Click(reject.clone()))?;

        // 6. the comparison, in place
        seen = now(&mut c)?;
        observations += 1;
        let chosen = Some(Value::Text(req.compare.name().to_owned()));
        let selected = act(&mut c, "silences_surface.proposer", "select", chosen, &seen)?;
        done(&selected, "select the proposer")?;
        seen = now(&mut c)?;
        observations += 1;
        let intent = Some(Value::Text(COMPARE.to_owned()));
        let written = act(
            &mut c,
            "silences_surface.intent",
            "set_value",
            intent,
            &seen,
        )?;
        done(&written, "write the intent")?;
        seen = now(&mut c)?;
        observations += 1;
        let proposed = act(&mut c, "silences_surface.propose", "propose", None, &seen)?;
        let compare = done(&proposed, "propose the comparison")?;
        idle(&mut c, &mut observations)?;

        // 7. close
        seen = look(
            &mut c,
            Observe {
                bounds: true,
                ..Observe::default()
            },
        )?;
        observations += 1;
        let closed = act(&mut c, "silences_surface.close", "close", None, &seen)?;
        let close = done(&closed, "close")?;
        now(&mut c)?;
        observations += 1;
        Ok(Story {
            observations,
            detect,
            cuts,
            review,
            surface,
            applied,
            rejected_by_person: reject,
            compare,
            close,
        })
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
