//! The mirror (task-79): the scene a window draws, written by the app,
//! read back by a viewer and drawn there with the same pixels.
//!
//! In process, two compositors: every gallery page in both main themes,
//! staged, drawn, written (`record::scene::Sender`), read back into a second
//! compositor (`uigallery::shell::window::mirror::Viewer` over `record::scene::Mirror`)
//! and drawn by a second headless host; the two frames are the same
//! bytes. The image store is global, so here both sides share it: an
//! image proves nothing in process.
//!
//! Between two processes, over the socket: this test binary runs itself
//! again as the app (`app_child`, a `ShowcaseView` drawn by a headless
//! window host, its agent port open, its frames offered as the shell
//! offers them), and the test is the viewer, headless. The child prints
//! the hash of each frame it draws; the viewer draws what came over the
//! socket and compares. Pages: Attachment (images, whose pixels cross
//! from one image store to the other), Todo (text and bindings, and a hover
//! that sends one layer's patch), Effects (glass, the backdrop blur) and
//! Capture with the fake camera on (a stream image: one stream slot in
//! the viewer, its atlas does not grow). The bytes each frame took are
//! printed: the band.
//!
//! The mirror answers (task-79 phase 2): the viewer finds the control
//! under a point as the app's hit test does, on every page; over the
//! socket, a click at the viewer goes to the app child as an act of the
//! agent "mirror" and reaches the frame and the revision an agent's act
//! reaches. The click's other cases, in process, are `mirror_input.rs`.
//!
//! Desktop only, like the port; skips without a GPU adapter.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use accord::{Actor, Request};
use engine::drawing::gpu::RenderConfig;
use engine::drawing::headless::Frame;
use engine::runtime::platform::capture::FakeCapture;
use engine::runtime::window::WindowHost;
use engine::widgets::prelude::WidgetEvent;
use record::scene::{Head, Sender};
use socket::{AgentPort, Client, Config, Reply, WireError, encode_server};
use uigallery::shell::window::mirror::{Hand, MAIN, Pressed, Scenes, Viewer, offer};
use uigallery::views::{PAGES, ShowcaseView};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 800;
/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

/// FNV-1a 64 over the frame's RGBA8: the hash of a frame on both sides.
fn hash(rgba: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in rgba {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn host() -> Option<WindowHost> {
    let host = WindowHost::headless(WIDTH, HEIGHT, 1.0, RenderConfig::default());
    if host.is_none() {
        eprintln!("no GPU adapter; skipping");
    }
    host
}

fn head(clear: [f32; 4]) -> Head {
    Head {
        width: WIDTH as f32,
        height: HEIGHT as f32,
        scale: 1.0,
        clear,
    }
}

/// The viewer's frame: what it holds, drawn by its host and read back.
fn mirrored(viewer: &mut Viewer, host: &mut WindowHost) -> Frame {
    let clear = viewer.head().expect("a frame came").clear;
    host.begin_frame();
    viewer.draw(host.compositor_mut(), false);
    assert!(host.present(clear, "mirror").is_some(), "the mirror drew");
    host.read().expect("the mirror reads")
}

/// Every page in both main themes, in process: the frame drawn from the
/// scene read back is the frame drawn from the scene written, byte for
/// byte. One view tours the gallery and one viewer follows it through a
/// server, as a watch would: each page after the first arrives as a
/// delta against the one before.
#[test]
fn every_page_mirrors_in_process_with_the_same_pixels() {
    let (Some(mut app), Some(mut seen)) = (host(), host()) else {
        return;
    };
    let mut view = ShowcaseView::new(WIDTH as f32, HEIGHT as f32);
    view.resize(WIDTH as f32, HEIGHT as f32, 1.0);
    let mut server = watched(1);
    let mut sender = Sender::new();
    let mut viewer = Viewer::new();
    let mut pages = 0;
    for theme in ["core", "hoff"] {
        for p in PAGES {
            view.jump_to_page(p.title);
            view.apply_theme(theme);
            settle(&mut view);
            let clear = view.theme.colors.bg.to_linear_array();
            app.begin_frame();
            view.render(app.compositor_mut());
            app.present(clear, "app").expect("the app drew");
            let drawn = app.read().expect("the app reads");
            let frame = sender.frame(app.compositor(), head(clear));
            for out in server.offer_scene(MAIN, &offer(frame, app.compositor())) {
                viewer.take(out.msg.reply).expect("the frame reads");
            }
            let mirror = mirrored(&mut viewer, &mut seen);
            assert!(
                mirror.rgba == drawn.rgba,
                "{} / {theme}: the mirror's pixels differ",
                p.title
            );
            pages += 1;
        }
    }
    assert_eq!(pages, PAGES.len() * 2);
}

/// The bindings over the mirror: the frame with them differs from the
/// app's only where a bound control sits (its outline and its name above
/// it), and at every control the app binds.
#[test]
fn the_bindings_are_drawn_over_the_mirror_where_the_controls_are() {
    let (Some(mut app), Some(mut seen)) = (host(), host()) else {
        return;
    };
    let mut view = ShowcaseView::new(WIDTH as f32, HEIGHT as f32);
    view.jump_to_page("Todo");
    view.resize(WIDTH as f32, HEIGHT as f32, 1.0);
    settle(&mut view);
    let clear = view.theme.colors.bg.to_linear_array();
    app.begin_frame();
    view.render(app.compositor_mut());
    app.present(clear, "app").expect("the app drew");
    let drawn = app.read().expect("the app reads");
    let mut server = watched(1);
    let frame = Sender::new().frame(app.compositor(), head(clear));
    let mut viewer = Viewer::new();
    for out in server.offer_scene(MAIN, &offer(frame, app.compositor())) {
        viewer.take(out.msg.reply).expect("the frame reads");
    }
    let bound: Vec<(String, [f32; 4])> = app
        .compositor()
        .visible_bindings()
        .map(|(id, r)| (id.as_str().to_owned(), r))
        .collect();
    assert!(bound.len() >= 5, "the Todo page binds its controls");
    assert_eq!(
        viewer
            .bounds()
            .iter()
            .map(|b| (b.id.as_str().to_owned(), b.rect))
            .collect::<Vec<_>>(),
        bound
    );
    seen.begin_frame();
    viewer.draw(seen.compositor_mut(), true);
    seen.present(clear, "mirror").expect("the mirror drew");
    let over = seen.read().expect("the mirror reads");
    let near = |x: f32, y: f32| {
        bound.iter().any(|(id, [bx, by, bw, bh])| {
            let label = 6.0 * id.chars().count() as f32 + 6.0;
            x >= bx - 1.0 && x <= bx + bw.max(label) + 1.0 && y >= by - 14.0 && y <= by + bh + 1.0
        })
    };
    let mut changed = 0;
    for (i, (a, b)) in drawn.rgba.chunks(4).zip(over.rgba.chunks(4)).enumerate() {
        if a != b {
            let (x, y) = ((i as u32 % WIDTH) as f32, (i as u32 / WIDTH) as f32);
            assert!(
                near(x, y),
                "a pixel changed at ({x}, {y}), far from every control"
            );
            changed += 1;
        }
    }
    // every control's outline shows: its top edge, a pixel in from the
    // corner, differs from the app's frame
    for (id, [x, y, w, _]) in &bound {
        let (px, py) = ((x + w / 2.0) as u32, *y as u32);
        let at = ((py * WIDTH + px) * 4) as usize;
        assert!(
            drawn.rgba[at..at + 4] != over.rgba[at..at + 4],
            "{id}: no outline at ({px}, {py})"
        );
    }
    assert!(changed > 1000, "{changed} pixels changed");
}

/// The viewer finds under a point the control the app's own hit test
/// finds there (`Compositor::hit`), from the bindings the frame carries
/// alone: every page in both main themes, at the center of each bound
/// control and on a 20 px grid over the window. No GPU: the scene is
/// built into a plain compositor.
#[test]
fn the_viewer_hits_the_control_the_app_hits() {
    let (mut points, mut controls) = (0, 0);
    for theme in ["core", "hoff"] {
        for p in PAGES {
            let mut view = ShowcaseView::new(WIDTH as f32, HEIGHT as f32);
            view.jump_to_page(p.title);
            view.apply_theme(theme);
            view.resize(WIDTH as f32, HEIGHT as f32, 1.0);
            settle(&mut view);
            let mut c = engine::drawing::compositor::Compositor::new();
            view.render(&mut c);
            let mut server = watched(1);
            let frame = Sender::new().frame(&c, head([0.0; 4]));
            let mut viewer = Viewer::new();
            for out in server.offer_scene(MAIN, &offer(frame, &c)) {
                viewer.take(out.msg.reply).expect("the frame reads");
            }
            let mut at: Vec<(f32, f32)> = c
                .visible_bindings()
                .map(|(_, [x, y, w, h])| (x + w / 2.0, y + h / 2.0))
                .collect();
            for y in (0..HEIGHT).step_by(20) {
                for x in (0..WIDTH).step_by(20) {
                    at.push((x as f32 + 0.5, y as f32 + 0.5));
                }
            }
            for (x, y) in at {
                let app = c.hit(x, y);
                assert_eq!(viewer.hit(x, y), app, "{} / {theme} at ({x}, {y})", p.title);
                points += 1;
                controls += usize::from(app.is_some());
            }
        }
    }
    assert!(
        controls > 1000,
        "{controls} of {points} points on a control"
    );
}

/// A server whose scenes the app's person shares (`PLEV_MIRROR=1`).
fn shared_server() -> socket::Server {
    let mut server = socket::Server::new("mirror", 8);
    server.share_scenes(true);
    server
}

/// `conn` names itself and watches the main window.
fn watch_on(server: &mut socket::Server, conn: u64) {
    for ask in [
        socket::Ask::Hello {
            name: format!("viewer{conn}"),
        },
        socket::Ask::Scene(socket::SceneAsk {
            window: MAIN.into(),
        }),
    ] {
        let out = server.handle(
            conn,
            socket::ClientMsg {
                id: format!("w{conn}"),
                ask,
            },
            &mut NoHost,
        );
        assert!(
            !out.iter().any(|o| matches!(o.msg.reply, Reply::Error(_))),
            "{out:?}"
        );
    }
}

/// A shared server with `conn` watching the main window.
fn watched(conn: u64) -> socket::Server {
    let mut server = shared_server();
    watch_on(&mut server, conn);
    server
}

/// A host for a server driven in process: the scene asks nothing of it.
struct NoHost;

impl socket::Host for NoHost {
    fn project(&self) -> Option<accord::Snapshot> {
        None
    }

    fn dispatch(&mut self, _req: Request) -> Option<accord::RequestId> {
        None
    }

    fn add_sink(&mut self, _sink: Box<dyn accord::ReceiptSink>) {}
}

// -- Between two processes ----------------------------------------------------

/// The environment variable that makes this binary the app.
const CHILD: &str = "PLEV_MIRROR_CHILD";

/// What starts a line the app says, wherever the test harness put it
/// (it prints the test's name first, on the same line).
const SAID: &str = "@mirror ";

/// The app in a process of its own: a gallery view drawn by a headless
/// window host, its agent port open in the directory the parent names,
/// each frame offered to the port's watches after it is drawn and after
/// each drain, as the shell does. Commands on stdin, one per line:
/// `page <title>|<theme>`, `hover <x> <y>`, `camera`, `wake`, `redraw`,
/// `quit`; each command that draws prints `drawn <hash>`. A drain that
/// dispatched an act (an agent's, the mirror's click) draws the frame
/// the act left, as the shell does, and prints `acted <hash>`. Does
/// nothing unless run by the parent (`PLEV_MIRROR_CHILD` set).
#[test]
#[ignore = "the app's side of the socket test, run by it"]
fn app_child() {
    let Some(dir) = std::env::var_os(CHILD) else {
        return;
    };
    let Some(mut host) = host() else {
        println!("{SAID}no-gpu");
        return;
    };
    let mut view = ShowcaseView::new(WIDTH as f32, HEIGHT as f32);
    view.resize(WIDTH as f32, HEIGHT as f32, 1.0);
    let fake = Arc::new(Mutex::new(FakeCapture::new()));
    view.set_capture(Box::new(fake.clone()));
    view.set_capture_shown(true);
    // the first frame, drawn before the port opens: the gallery's first
    // page, what a watch that comes finds on screen
    settle(&mut view);
    let first = draw(&mut view, &mut host);
    println!("{SAID}drawn {first:016x}");
    let (wake_tx, woke) = mpsc::channel::<()>();
    let config = Config {
        app: "mirror".into(),
        dir: PathBuf::from(dir),
        ring: 8,
    };
    let mut port = AgentPort::open_with(config, move || {
        let _ = wake_tx.send(());
    })
    .expect("the port opens");
    // the person launched this app with PLEV_MIRROR=1
    port.share_scenes(true);
    let mut scenes = Scenes::default();
    let (cmd_tx, cmds) = mpsc::channel::<String>();
    thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            let Ok(line) = line else { break };
            if cmd_tx.send(line).is_err() {
                break;
            }
        }
    });
    println!("{SAID}socket {}", port.path().display());
    let mut last = host.read().expect("the app reads").rgba;
    loop {
        if woke.recv_timeout(Duration::from_millis(2)).is_ok() {
            if view.serve_agent(&mut port, host.compositor()) {
                settle(&mut view);
                let drawn = draw(&mut view, &mut host);
                last = host.read().expect("the app reads").rgba;
                println!("{SAID}acted {drawn:016x}");
            }
            let clear = view.theme.colors.bg.to_linear_array();
            scenes.offer(&mut port, MAIN, host.compositor(), head(clear), || {
                view.gallery_semantics().ok()
            });
        }
        let Ok(cmd) = cmds.try_recv() else {
            continue;
        };
        let (verb, rest) = cmd.split_once(' ').unwrap_or((cmd.as_str(), ""));
        match verb {
            "quit" => return,
            "page" => {
                let (title, theme) = rest.split_once('|').expect("page <title>|<theme>");
                view.jump_to_page(title);
                view.apply_theme(theme);
                settle(&mut view);
            }
            "hover" => {
                let mut at = rest.split(' ').map(|v| v.parse::<f32>().expect("a number"));
                let (x, y) = (at.next().expect("x"), at.next().expect("y"));
                view.handle_event(&WidgetEvent::MouseMove { x, y });
                settle(&mut view);
            }
            "camera" => {
                let seen = view.semantics().expect("the page projects");
                let req = Request::observe(&seen, Actor::Human, "capture.camera", "toggle", None)
                    .expect("the camera toggles");
                view.act(req).expect("the page takes it");
                settle(&mut view);
            }
            // the next frame of the camera, as the loop's wake takes it
            "wake" => {
                view.wake(TURN);
            }
            // the last frame drawn, its RGBA8 into a file
            "save" => {
                std::fs::write(rest, &last).expect("the frame is written");
                println!("{SAID}saved");
                continue;
            }
            "redraw" => {}
            other => panic!("unknown command {other}"),
        }
        let drawn = draw(&mut view, &mut host);
        last = host.read().expect("the app reads").rgba;
        println!("{SAID}drawn {drawn:016x}");
        let clear = view.theme.colors.bg.to_linear_array();
        let at = Instant::now();
        scenes.offer(&mut port, MAIN, host.compositor(), head(clear), || {
            view.gallery_semantics().ok()
        });
        println!("{SAID}offered {}", at.elapsed().as_micros());
    }
}

/// The view's frame, drawn by the host as a window draws it; its hash.
fn draw(view: &mut ShowcaseView, host: &mut WindowHost) -> u64 {
    let clear = view.theme.colors.bg.to_linear_array();
    host.begin_frame();
    view.render(host.compositor_mut());
    host.present(clear, "app").expect("the app drew");
    hash(&host.read().expect("the app reads").rgba)
}

/// Ticks until nothing moves (a caret at rest counts as settled).
fn settle(view: &mut ShowcaseView) {
    let mut dt = 0.0;
    for _ in 0..240 {
        if !view.tick(dt) {
            return;
        }
        dt = TURN;
    }
}

/// The app in a child process, its stdout read line by line.
struct App {
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<String>,
}

impl App {
    fn spawn(dir: &std::path::Path) -> App {
        let exe = std::env::current_exe().expect("this test binary");
        let mut child = Command::new(exe)
            .args([
                "--exact",
                // the test's name in the suite carries this file's module
                "mirror::app_child",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD, dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("the app starts");
        let stdin = child.stdin.take().expect("its stdin");
        let stdout: ChildStdout = child.stdout.take().expect("its stdout");
        let (tx, lines) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        App {
            child,
            stdin,
            lines,
        }
    }

    /// What the app says next that starts with `word`, the rest of it;
    /// `None` when it said it has no GPU.
    fn said(&self, word: &str) -> Option<String> {
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let line = match self.lines.recv_timeout(left) {
                Ok(line) => line,
                Err(e) => panic!("the app did not say {word}: {e}"),
            };
            let Some((_, said)) = line.split_once(SAID) else {
                continue;
            };
            if said == "no-gpu" {
                return None;
            }
            if let Some(rest) = said.strip_prefix(word) {
                return Some(rest.trim().to_owned());
            }
        }
    }

    /// Runs `cmd` in the app: the hash of the frame it drew, and how
    /// long offering it took on its thread (writing the frame, choosing
    /// what each watch needs, the lines as JSON), in microseconds.
    fn tell(&mut self, cmd: &str) -> (u64, u64) {
        writeln!(self.stdin, "{cmd}").expect("the app listens");
        let drawn = self.said("drawn").expect("the app draws");
        let took = self.said("offered").expect("the app offers");
        (
            u64::from_str_radix(&drawn, 16).expect("a hash"),
            took.parse().expect("microseconds"),
        )
    }

    /// The app's last frame, its RGBA8, into `path`.
    fn save(&mut self, path: &std::path::Path) {
        writeln!(self.stdin, "save {}", path.display()).expect("the app listens");
        self.said("saved").expect("the app saves");
    }

    fn quit(&mut self) {
        let _ = writeln!(self.stdin, "quit");
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            match self.child.try_wait() {
                Ok(Some(status)) => {
                    assert!(status.success(), "the app ended with {status}");
                    return;
                }
                Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
                _ => panic!("the app did not quit"),
            }
        }
    }
}

impl Drop for App {
    /// A test that failed leaves no app behind: its own process, by its
    /// handle (the pid it was given), and nothing else.
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

/// The watch's side: the viewer, the lines of the frame being put
/// together and what they weighed.
struct Watch {
    c: Client,
    viewer: Viewer,
    host: WindowHost,
    /// every line that came, as the app wrote it, and every binary part
    /// in them decoded: the whole output of the watch.
    wire: Vec<String>,
    parts: Vec<Vec<u8>>,
}

/// One frame as it came: its number, the bytes of its lines (as the
/// app wrote them, each with its newline), the lines, the layers that
/// carried a body or a patch.
#[derive(Debug)]
struct Came {
    seq: u64,
    bytes: usize,
    lines: usize,
    sent: usize,
}

impl Watch {
    fn next(&mut self) -> Came {
        let (mut bytes, mut lines, mut sent) = (0, 0, 0);
        loop {
            let msg = match self.c.recv() {
                Ok(msg) => msg,
                Err(e) => panic!("the watch: {e}"),
            };
            let line = encode_server(&msg);
            bytes += line.len() + 1;
            lines += 1;
            self.wire.push(line);
            match &msg.reply {
                Reply::Scene(f) => {
                    sent = f
                        .layers
                        .iter()
                        .filter(|l| l.body.is_some() || l.patch.is_some())
                        .count();
                    self.parts.extend(f.head.clone());
                    for l in &f.layers {
                        self.parts.extend(l.body.clone());
                        self.parts.extend(l.patch.clone());
                    }
                }
                Reply::SceneBlob(b) => self.parts.push(b.bytes.clone()),
                _ => {}
            }
            if let Some(seq) = self.viewer.take(msg.reply).expect("the frame reads") {
                return Came {
                    seq,
                    bytes,
                    lines,
                    sent,
                };
            }
        }
    }

    /// Nothing comes within `ms`.
    fn quiet(&mut self, ms: u64) -> bool {
        self.c.set_timeout(Some(ms)).expect("a timeout");
        let quiet = matches!(self.c.recv(), Err(WireError::Timeout(_)));
        self.c.set_timeout(Some(60_000)).expect("a timeout");
        quiet
    }

    fn hash(&mut self) -> u64 {
        hash(&mirrored(&mut self.viewer, &mut self.host).rgba)
    }

    /// Whether `pixel`, repeated, is anywhere in what came: in a binary
    /// part decoded, or in a line as base64 (at any of the four phases a
    /// run of 4-byte pixels takes in base64's 3-byte groups).
    fn carried(&self, pixel: [u8; 4]) -> bool {
        let run = pixel.repeat(16);
        let in_parts = self
            .parts
            .iter()
            .any(|p| p.windows(run.len()).any(|w| w == run));
        let long = pixel.repeat(64);
        let texts: Vec<String> = (0..4)
            .map(|shift| socket::encode_base64(&long[shift..])[8..72].to_owned())
            .collect();
        let in_lines = self
            .wire
            .iter()
            .any(|line| texts.iter().any(|t| line.contains(t.as_str())));
        in_parts || in_lines
    }
}

/// The acceptance: the app in another process, the viewer here, over
/// the socket; every frame drawn here has the hash of the one the app
/// drew, and a fixed image's pixels crossed; a camera's never do.
#[test]
fn the_app_mirrors_over_the_socket_with_the_same_pixels() {
    let Some(view_host) = host() else {
        return;
    };
    let tmp = tempfile::tempdir().expect("a temporary directory");
    let dir = tmp.path().join("plev");
    let mut app = App::spawn(&dir);
    let Some(on_screen) = app.said("drawn") else {
        app.quit();
        return;
    };
    let path = app.said("socket").expect("the port opens");
    let mut c = Client::connect(std::path::Path::new(&path)).expect("the viewer connects");
    c.set_timeout(Some(60_000)).expect("a timeout");
    c.hello("mirror").expect("the viewer names itself");
    c.watch_scene(MAIN).expect("the watch goes");
    let mut w = Watch {
        c,
        viewer: Viewer::new(),
        host: view_host,
        wire: Vec::new(),
        parts: Vec::new(),
    };
    let first = w.next();
    assert_eq!(first.seq, 1, "the frame the app holds, at once");
    let on_screen = u64::from_str_radix(&on_screen, 16).expect("a hash");
    assert_eq!(
        w.hash(),
        on_screen,
        "the frame on screen when the watch came"
    );

    let mut band = Vec::new();
    for (title, theme) in [
        ("Attachment", "hoff"),
        ("Todo", "core"),
        ("Effects", "hoff"),
        ("Capture", "core"),
    ] {
        let (drawn, took) = app.tell(&format!("page {title}|{theme}"));
        let came = w.next();
        assert_eq!(
            w.hash(),
            drawn,
            "{title} / {theme}: the mirror's pixels differ"
        );
        band.push(format!(
            "{title} / {theme}, the page arrives: {} bytes in {} lines, {} layers sent; \
             offered in {took} us",
            came.bytes, came.lines, came.sent
        ));
        // drawn again with nothing changed: nothing travels
        let (again, took) = app.tell("redraw");
        assert_eq!(again, drawn);
        assert!(w.quiet(300), "{title}: an unchanged frame sent something");
        band.push(format!(
            "{title} / {theme}, drawn again unchanged: 0 bytes; offered in {took} us"
        ));
    }
    let counts = w.viewer.mirror().image_counts();
    assert!(counts.loads >= 2, "Attachment's images crossed: {counts:?}");
    assert_eq!(counts.missing, 0);

    // Todo: a hover over one control, one layer's patch
    let (drawn, _) = app.tell("page Todo|core");
    w.next();
    assert_eq!(w.hash(), drawn);
    let [x, y, bw, bh] = w
        .viewer
        .bounds()
        .iter()
        .find(|b| b.id.as_str() == "todo.filter[completed]")
        .expect("the filter")
        .rect;
    let (drawn, took) = app.tell(&format!("hover {} {}", x + bw / 2.0, y + bh / 2.0));
    let came = w.next();
    assert_eq!(w.hash(), drawn, "the hover's frame");
    assert_eq!(came.sent, 1, "one layer moved");
    band.push(format!(
        "Todo / core, a hover: {} bytes in {} lines, {} layer sent; offered in {took} us",
        came.bytes, came.lines, came.sent
    ));

    // Capture with the fake camera on: a stream image. none of its
    // pixels crosses: the viewer draws a box in its place, and a new
    // frame of the camera moves nothing that travels
    let (drawn, _) = app.tell("page Capture|core");
    w.next();
    assert_eq!(w.hash(), drawn);
    let (drawn, took) = app.tell("camera");
    let came = w.next();
    band.push(format!(
        "Capture / core, the camera turns on: {} bytes in {} lines, {} layer sent; \
         offered in {took} us",
        came.bytes, came.lines, came.sent
    ));
    let mut shown = vec![drawn];
    for n in 0..4 {
        let (drawn, took) = app.tell("wake");
        shown.push(drawn);
        assert!(w.quiet(300), "camera frame {n} sent something");
        band.push(format!(
            "Capture / core, camera frame {n}: 0 bytes; offered in {took} us"
        ));
    }
    let frames: std::collections::HashSet<u64> = shown.iter().copied().collect();
    assert!(
        frames.len() >= 3,
        "the app drew the camera's frames: {shown:x?}"
    );
    // every color a frame of the fake takes (its frame number mod 200,
    // which the clock sets, not only the first frames)
    for n in 0..200 {
        assert!(
            !w.carried(FakeCapture::pixel(n)),
            "the bytes of camera frame {n} went out on the watch"
        );
    }
    // the mirror is the app's frame but for the camera's box
    let tmp_frame = tmp.path().join("app.rgba");
    app.save(&tmp_frame);
    let app_frame = std::fs::read(&tmp_frame).expect("the app's frame");
    let mirror = mirrored(&mut w.viewer, &mut w.host);
    let [bx, by, bw, bh] = w
        .viewer
        .bounds()
        .iter()
        .find(|b| b.id.as_str() == "capture.frame")
        .expect("the camera's box is bound")
        .rect;
    let mut inside = 0;
    for (i, (a, b)) in app_frame.chunks(4).zip(mirror.rgba.chunks(4)).enumerate() {
        if a != b {
            let (x, y) = ((i as u32 % WIDTH) as f32, (i as u32 / WIDTH) as f32);
            assert!(
                x >= bx - 1.0 && x <= bx + bw + 1.0 && y >= by - 1.0 && y <= by + bh + 1.0,
                "the mirror differs from the app at ({x}, {y}), out of the camera's box"
            );
            inside += 1;
        }
    }
    assert!(
        inside > 1000,
        "the box drawn in the camera's place: {inside} pixels"
    );
    let counts = w.viewer.mirror().image_counts();
    assert_eq!(
        counts.live, 1,
        "the camera came once, as its size and place"
    );
    app.quit();
    println!("\nthe band, 1280x800 @1, over the socket:");
    for line in band {
        println!("  {line}");
    }
}

/// The watch and the hand of a viewer on the app child's socket, both
/// under the name "mirror".
fn viewer_on(path: &str, host: WindowHost) -> (Watch, Hand) {
    let path = std::path::Path::new(path);
    let mut c = Client::connect(path).expect("the viewer connects");
    c.set_timeout(Some(60_000)).expect("a timeout");
    c.hello("mirror").expect("the viewer names itself");
    c.watch_scene(MAIN).expect("the watch goes");
    let mut hand = Hand::connect(path, "mirror").expect("the hand connects");
    hand.set_timeout(Some(60_000)).expect("a timeout");
    let w = Watch {
        c,
        viewer: Viewer::new(),
        host,
        wire: Vec::new(),
        parts: Vec::new(),
    };
    (w, hand)
}

/// `name` acts `action` on `target` over its own connection, from what
/// it observes first; the terminal stage.
fn agent_act(path: &str, name: &str, target: &str, action: &str) -> socket::StageReply {
    let mut c = Client::connect(std::path::Path::new(path)).expect("the agent connects");
    c.set_timeout(Some(60_000)).expect("a timeout");
    c.hello(name).expect("hello");
    let Reply::Snapshot(seen) = c.observe(socket::Observe::default()).expect("observed") else {
        panic!("a snapshot");
    };
    let target = accord::SemId::parse(target).expect("an id");
    let act = socket::Act::from_revision(target, action.parse().expect("an action"), seen.revision);
    let mut stages = c.act(act, |_| {}).expect("its stages");
    stages.pop().expect("a terminal stage")
}

/// The input's acceptance between two processes: the viewer here, the
/// app in a child, over the socket. A click on a Todo item at the mirror
/// goes to the app as the agent "mirror" and toggles it; the frame it
/// left comes back to the mirror with the app's hash. An agent then
/// toggles the item back (the frame of before) and again: it lands on
/// the frame and the revision the mirror's click reached. A click on the
/// camera's switch, kept for a person, is refused and said, and the app
/// draws nothing new.
#[test]
fn the_app_takes_the_mirror_s_click_over_the_socket() {
    let Some(view_host) = host() else {
        return;
    };
    let tmp = tempfile::tempdir().expect("a temporary directory");
    let mut app = App::spawn(&tmp.path().join("plev"));
    if app.said("drawn").is_none() {
        app.quit();
        return;
    }
    let path = app.said("socket").expect("the port opens");
    let (mut w, mut hand) = viewer_on(&path, view_host);
    w.next();
    let (before, _) = app.tell("page Todo|core");
    w.next();
    assert_eq!(w.hash(), before);
    hand.look().expect("the hand looks");

    let [x, y, bw, bh] = w
        .viewer
        .bounds()
        .iter()
        .find(|b| b.id.as_str() == "todo.item[1]")
        .expect("the first item")
        .rect;
    let pressed = hand.click(&w.viewer, x + bw / 2.0, y + bh / 2.0, |_, _| {});
    assert_eq!(pressed.said(), "todo.item[1]: toggle done");
    let clicked =
        u64::from_str_radix(&app.said("acted").expect("the app acted"), 16).expect("a hash");
    assert_ne!(clicked, before, "the item drawn done");
    w.next();
    assert_eq!(
        w.hash(),
        clicked,
        "the mirror shows the frame the click left"
    );
    let by_mirror = match &pressed {
        Pressed::Acted { stages, .. } => stages.last().and_then(|s| s.revision),
        other => panic!("the act went: {other:?}"),
    };

    let back = agent_act(&path, "claude", "todo.item[1]", "toggle");
    assert_eq!(back.stage.name(), "completed");
    let drawn = u64::from_str_radix(&app.said("acted").expect("acted"), 16).expect("a hash");
    assert_eq!(drawn, before, "toggled back: the frame of before");
    w.next();
    let again = agent_act(&path, "claude", "todo.item[1]", "toggle");
    let drawn = u64::from_str_radix(&app.said("acted").expect("acted"), 16).expect("a hash");
    assert_eq!(drawn, clicked, "the agent's act lands where the click did");
    assert_eq!(again.revision, by_mirror, "on the same revision");
    w.next();
    assert_eq!(w.hash(), clicked);

    // an action kept for a person: refused, said, nothing drawn anew
    let (capture, _) = app.tell("page Capture|core");
    w.next();
    hand.look().expect("the hand looks");
    let [x, y, bw, bh] = w
        .viewer
        .bounds()
        .iter()
        .find(|b| b.id.as_str() == "capture.camera")
        .expect("the camera's switch")
        .rect;
    let refused = hand.click(&w.viewer, x + bw / 2.0, y + bh / 2.0, |_, _| {});
    assert_eq!(
        refused.stage(),
        Some(&accord::Stage::Rejected(accord::Rejection::NotAuthorized {
            actor: Actor::Agent {
                name: "mirror".into()
            }
        }))
    );
    assert!(refused.said().contains("refused: only a person asks it"));
    let drawn = u64::from_str_radix(&app.said("acted").expect("acted"), 16).expect("a hash");
    assert_eq!(drawn, capture, "the camera stayed off");
    assert!(w.quiet(300), "an unchanged frame sent nothing");
    app.quit();
}

// -- The band -------------------------------------------------------------------

/// A page staged for the band: the view, its compositor, a watch.
struct Band {
    view: ShowcaseView,
    c: engine::drawing::compositor::Compositor,
    sender: Sender,
    server: socket::Server,
    clear: [f32; 4],
}

impl Band {
    fn new(page: &str, theme: &str) -> Band {
        let mut view = ShowcaseView::new(WIDTH as f32, HEIGHT as f32);
        view.jump_to_page(page);
        view.apply_theme(theme);
        view.resize(WIDTH as f32, HEIGHT as f32, 1.0);
        settle(&mut view);
        let clear = view.theme.colors.bg.to_linear_array();
        let mut band = Band {
            view,
            c: engine::drawing::compositor::Compositor::new(),
            sender: Sender::new(),
            server: shared_server(),
            clear,
        };
        band.view.render(&mut band.c);
        band
    }

    /// A watch that comes now: the bytes and lines of its first frame.
    fn watch(&mut self, conn: u64) -> (usize, usize) {
        watch_on(&mut self.server, conn);
        self.offer()
    }

    /// The frame drawn again and offered: the bytes and lines that went.
    fn offer(&mut self) -> (usize, usize) {
        self.view.render(&mut self.c);
        let frame = self.sender.frame(&self.c, head(self.clear));
        let out = self.server.offer_scene(MAIN, &offer(frame, &self.c));
        let bytes = out.iter().map(|o| encode_server(&o.msg).len() + 1).sum();
        (bytes, out.len())
    }

    /// The pointer over each bound control, then over a 40 px grid of the
    /// page, until a frame changes; else a 40 px scroll. What it was and
    /// the bytes that went.
    fn one_change(&mut self) -> (String, usize) {
        let mut at: Vec<(String, f32, f32)> = self
            .c
            .visible_bindings()
            .map(|(id, [x, y, w, h])| (id.as_str().to_owned(), x + w / 2.0, y + h / 2.0))
            .collect();
        at.reverse();
        for y in (100..780).step_by(40) {
            for x in (300..1260).step_by(40) {
                at.push((format!("({x}, {y})"), x as f32, y as f32));
            }
        }
        for (what, x, y) in at {
            self.view.handle_event(&WidgetEvent::MouseMove { x, y });
            settle(&mut self.view);
            let (bytes, _) = self.offer();
            if bytes > 0 {
                return (format!("hover {what}"), bytes);
            }
        }
        self.view.handle_event(&WidgetEvent::Scroll {
            x: 800.0,
            y: 400.0,
            delta: 40.0,
        });
        settle(&mut self.view);
        ("scroll 40 px".to_owned(), self.offer().0)
    }
}

/// The band per page, as a watch pays it on the wire (every line as the
/// port writes it): the page's first frame to a watch that just came
/// (its assets among it: the icons' paths, the images' pixels), a frame
/// with one change, a frame with none. A measurement: it prints, run
///
///     cargo test -p uigallery --test suite mirror::band -- --ignored --nocapture
#[test]
#[ignore = "a measurement: run with --ignored --nocapture"]
fn band() {
    println!(
        "\n{:<22} {:<5} {:>9} {:>6} {:>9} {:>9}  one change",
        "page", "theme", "first", "lines", "unchanged", "changed"
    );
    let (mut whole, mut changed, mut n) = (Vec::new(), Vec::new(), 0usize);
    for theme in ["core", "hoff"] {
        for p in PAGES {
            let mut b = Band::new(p.title, theme);
            let (first, lines) = b.watch(1);
            let (same, _) = b.offer();
            let (what, delta) = b.one_change();
            println!(
                "{:<22} {:<5} {:>9} {:>6} {:>9} {:>9}  {what}",
                p.title, theme, first, lines, same, delta
            );
            whole.push(first);
            changed.push(delta);
            n += 1;
        }
    }
    let stats = |v: &mut Vec<usize>| {
        v.sort_unstable();
        (
            v[0],
            v.iter().sum::<usize>() / v.len(),
            v[v.len() / 2],
            v[v.len() - 1],
        )
    };
    let (a, b, c, d) = stats(&mut whole);
    println!("\n{n} frames. first frame: min {a}, mean {b}, median {c}, max {d} bytes");
    let still = changed.iter().filter(|b| **b == 0).count();
    changed.retain(|b| *b > 0);
    let (a, b, c, d) = stats(&mut changed);
    println!(
        "one change: min {a}, mean {b}, median {c}, max {d} bytes over {} frames \
         ({still} change with neither a hover nor a scroll); unchanged: 0",
        changed.len()
    );
}
