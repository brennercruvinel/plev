//! The mirror (task-79): watches the scene of a window of a plev app
//! running with its agent port on, over the port's socket, and draws what
//! it receives with the same engine, the same embedded fonts and the
//! same pixels, the bindings (where each control shows) outlined and
//! named over it.
//!
//! ```text
//! PLEV_AGENT=1 cargo run -p showcase                      # the app, in a debug build
//! cargo run -p showcase --example mirror -- --show        # its main window, mirrored in a window
//! cargo run -p showcase --example mirror -- --frames 5 --png /tmp/mirror.png
//! ```
//!
//! Without `--show` it draws headless (`engine::window::WindowHost::headless`,
//! the host a window has, into a texture) and prints one line per frame:
//! what came (bytes, layers sent of the frame's, assets) and the hash of
//! the pixels drawn (FNV-1a 64 over the RGBA8), the hash the app's own
//! frame has. With `--show` it opens a window the size of the app's and
//! draws each frame there as it comes; `b` turns the bindings on and off,
//! Escape closes it.
//!
//! A click answers (task-79 phase 2): in the window a left click, and
//! headless `--click x,y` after the first frame, finds the control under
//! the point in the frame shown (`Viewer::hit`) and presses it through
//! the app's semantic contract (`Hand`): an act on that control, as an
//! agent named "mirror", resting on the mirror's last look at the app;
//! never an event, never a coordinate. What the app answers is said: done,
//! refused (an action kept for a person, a scene the app moved on from),
//! or waiting for the person at the app to confirm. A point on no control
//! sends nothing.
//!
//! The socket is local: `$TMPDIR/plev/<app>-<pid>.sock`, mode 0600 in a
//! directory of this user; nothing leaves the machine.

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod show;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let opts = match cli::parse(&args) {
        Ok(Some(opts)) => opts,
        Ok(None) => {
            println!("{}", cli::USAGE);
            return;
        }
        Err(e) => {
            eprintln!("mirror: {e}\n\n{}", cli::USAGE);
            std::process::exit(2);
        }
    };
    let run = if opts.show {
        show::run(&opts)
    } else {
        headless::run(&opts)
    };
    if let Err(e) = run {
        eprintln!("mirror: {e}");
        std::process::exit(3);
    }
}

#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod cli {
    use std::path::PathBuf;

    pub const USAGE: &str = "\
usage: mirror [--app <name | socket path>] [--window main|page]
              [--frames <n>] [--png <file>] [--no-bindings] [--show]
              [--click <x>,<y>]
watches the scene of a window of a plev app whose agent port is on
(PLEV_AGENT=1 PLEV_MIRROR=1 in a debug build) and draws it: headless by
default, one line per frame with the bytes that came and the hash of the
pixels drawn; --show in a window the size of the app's (b: bindings on
and off, Escape: close, a left click presses the control under it).
--app names the app (its socket in $TMPDIR/plev) or gives a socket's
path; alone, the one app listening. --window: main (default) or page (the
page's own window). --frames stops after that many frames, and --png then
writes the last one drawn (headless). --no-bindings draws the app's frame
alone. --click presses the control at x,y (logical pixels of the app's
window) after the first frame (headless), as a click in the window does:
an act through the app's contract, as the agent \"mirror\".";

    /// The name the mirror says `hello` with, on the watch and the hand.
    pub const NAME: &str = "mirror";

    pub struct Opts {
        pub app: Option<String>,
        pub window: String,
        pub frames: Option<u64>,
        pub png: Option<PathBuf>,
        pub bindings: bool,
        pub show: bool,
        pub click: Option<(f32, f32)>,
    }

    pub fn parse(args: &[String]) -> Result<Option<Opts>, String> {
        let mut opts = Opts {
            app: None,
            window: wire::MAIN_WINDOW.to_owned(),
            frames: None,
            png: None,
            bindings: true,
            show: false,
            click: None,
        };
        let mut it = args.iter();
        while let Some(arg) = it.next() {
            let mut value = |name: &str| {
                it.next()
                    .cloned()
                    .ok_or_else(|| format!("{name} needs a value"))
            };
            match arg.as_str() {
                "-h" | "--help" => return Ok(None),
                "--app" => opts.app = Some(value("--app")?),
                "--window" => opts.window = value("--window")?,
                "--frames" => {
                    let n = value("--frames")?;
                    opts.frames = Some(n.parse().map_err(|_| format!("--frames {n}"))?);
                }
                "--png" => opts.png = Some(PathBuf::from(value("--png")?)),
                "--no-bindings" => opts.bindings = false,
                "--show" => opts.show = true,
                "--click" => {
                    let at = value("--click")?;
                    let point = at
                        .split_once(',')
                        .and_then(|(x, y)| Some((x.trim().parse().ok()?, y.trim().parse().ok()?)));
                    opts.click = Some(point.ok_or_else(|| format!("--click {at}: x,y"))?);
                }
                other => return Err(format!("unknown option {other}")),
            }
        }
        Ok(Some(opts))
    }

    /// The socket of the app `--app` names, or of the one app listening.
    pub fn socket(app: Option<&str>) -> Result<PathBuf, String> {
        if let Some(path) = app.map(PathBuf::from).filter(|p| p.exists()) {
            return Ok(path);
        }
        wire::find(&wire::socket_dir(), app).map_err(|e| e.to_string())
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod headless {
    use engine::gpu::RenderConfig;
    use engine::window::WindowHost;
    use showcase::app::mirror::{Hand, Pressed, Viewer, said};
    use wire::{Client, Reply, WireError, encode_server};

    use crate::cli::{self, NAME, Opts};

    /// FNV-1a 64 over the frame's RGBA8.
    pub fn hash(rgba: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in rgba {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }

    /// What came for one frame.
    #[derive(Default)]
    struct Came {
        bytes: usize,
        lines: usize,
        assets: usize,
        sent: usize,
        layers: usize,
    }

    pub fn run(opts: &Opts) -> Result<(), String> {
        let path = cli::socket(opts.app.as_deref())?;
        let mut c = Client::connect(&path).map_err(|e| e.to_string())?;
        c.hello(NAME).map_err(|e| e.to_string())?;
        c.watch_scene(&opts.window).map_err(|e| e.to_string())?;
        println!("watching {} of {}", opts.window, path.display());
        let mut click = opts.click;
        let mut viewer = Viewer::new();
        let mut host: Option<(WindowHost, (u32, u32, u32))> = None;
        let mut came = Came::default();
        let mut frames = 0u64;
        loop {
            let msg = match c.recv() {
                Ok(msg) => msg,
                Err(WireError::Closed) => {
                    println!("the app closed the watch after {frames} frames");
                    return Ok(());
                }
                Err(e) => return Err(e.to_string()),
            };
            came.bytes += encode_server(&msg).len() + 1;
            came.lines += 1;
            match &msg.reply {
                Reply::SceneBlob(b) if b.at == 0 => came.assets += 1,
                Reply::Scene(f) => {
                    came.layers = f.layers.len();
                    came.sent = f
                        .layers
                        .iter()
                        .filter(|l| l.body.is_some() || l.patch.is_some())
                        .count();
                }
                Reply::Error(e) => return Err(e.to_string()),
                _ => {}
            }
            let Some(seq) = viewer.take(msg.reply)? else {
                continue;
            };
            let head = viewer.head().ok_or("a frame without a head")?;
            let size = (
                (head.width * head.scale).round().max(1.0) as u32,
                (head.height * head.scale).round().max(1.0) as u32,
                head.scale.to_bits(),
            );
            if host.as_ref().is_none_or(|(_, s)| *s != size) {
                let made = WindowHost::headless(
                    size.0,
                    size.1,
                    f64::from(head.scale),
                    RenderConfig::default(),
                )
                .ok_or("no GPU adapter")?;
                host = Some((made, size));
                viewer.forget_layers();
            }
            let (h, _) = host.as_mut().ok_or("no host")?;
            h.begin_frame();
            viewer.draw(h.compositor_mut(), opts.bindings);
            h.present(head.clear, "mirror")
                .ok_or("the frame was not drawn")?;
            let frame = h.read().map_err(|e| e.to_string())?;
            println!(
                "frame {seq}: {} bytes in {} lines, {} of {} layers sent, {} assets; \
                 {}x{} hash {:016x}",
                came.bytes,
                came.lines,
                came.sent,
                came.layers,
                came.assets,
                frame.width,
                frame.height,
                hash(&frame.rgba)
            );
            came = Came::default();
            frames += 1;
            if let Some((x, y)) = click.take() {
                press(&path, &viewer, x, y)?;
            }
            if opts.frames.is_some_and(|n| frames >= n) {
                if let Some(png) = &opts.png {
                    frame.write_png(png).map_err(|e| e.to_string())?;
                    println!("wrote {}", png.display());
                }
                return Ok(());
            }
        }
    }

    /// The control at `(x, y)` in the frame drawn, pressed by a hand of
    /// its own that looks at the app first; each stage that comes, and
    /// what came of it, said. A point on no control sends nothing.
    fn press(path: &std::path::Path, viewer: &Viewer, x: f32, y: f32) -> Result<(), String> {
        let Some(target) = viewer.hit(x, y).cloned() else {
            println!("click at ({x}, {y}): {}", Pressed::Nothing.said());
            return Ok(());
        };
        let mut hand = Hand::connect(path, NAME)?;
        hand.look()?;
        let pressed = hand.press(&target, |action, reply| {
            if !reply.is_terminal() {
                println!("  {}", said(&target, action, &reply.stage));
            }
        });
        println!("click at ({x}, {y}): {}", pressed.said());
        Ok(())
    }
}
