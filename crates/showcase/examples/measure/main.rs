//! What each stage of a frame costs for a few real interactions of the
//! gallery (task-73 phase 1): typing in the todo input, toggling an item,
//! hovering a list, a modal mid-opening, typing in a large document, an
//! idle redraw, a focused field at rest, and the floor of an empty frame.
//! `--probe-cost` prints what one probe costs with stages off and on.
//! With GPU timestamps (on unless `--no-gpu-timestamps`, phase 1b) the
//! report adds what the GPU itself spent on each frame next to the wait.
//!
//! ```text
//! cargo run --release -p showcase --example measure -- --out measurements.md
//!     [--frames 120] [--scenario <key>]... [--theme core]
//!     [--width 1200 --height 800 --scale 2] [--no-gpu-sync]
//!     [--no-gpu-timestamps]
//! cargo run --release -p showcase --example measure -- --probe-cost
//! ```
//!
//! Each scenario opens a page (or the editor) headless, drives it with the
//! input the window would deliver (`harness`), and records every turn of
//! the loop with `engine::perf::stages`. The tables go to stdout and, with
//! `--out`, to a markdown file. Native only: the headless renderer blocks
//! on the adapter.

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod harness;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod probe_cost;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod report;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod scenarios;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--probe-cost") {
        print!("{}", probe_cost::markdown());
        return;
    }
    match cli::parse(&args) {
        Ok(None) => println!("{}", cli::USAGE),
        Ok(Some(request)) => {
            if let Err(e) = cli::measure(&request) {
                eprintln!("measure: {e}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("measure: {e}\n\n{}", cli::USAGE);
            std::process::exit(2);
        }
    }
}

#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod cli {
    use std::path::PathBuf;

    use engine::headless::Viewport;
    use engine::perf::stages;

    use crate::report;
    use crate::scenarios::{self, Config};

    pub const USAGE: &str = "\
usage: measure [--out <file.md>] [--frames <n>] [--scenario <key>]...
               [--theme <name>] [--width <px>] [--height <px>] [--scale <s>]
               [--no-gpu-sync] [--no-gpu-timestamps]
       measure --probe-cost
defaults: every scenario, 120 frames, theme core, 1200x800 at scale 2
(the window's launch size on a retina display), the gpu wait on, gpu
timestamp queries on where the adapter has them.
scenarios: type toggle hover overlay editor editor-page idle caret floor";

    pub struct Request {
        pub out: Option<PathBuf>,
        pub scenarios: Vec<String>,
        pub config: Config,
    }

    /// `Ok(None)` asks for the usage.
    pub fn parse(args: &[String]) -> Result<Option<Request>, String> {
        let mut out = None;
        let mut picked = Vec::new();
        let mut frames = 120;
        let mut theme = "core".to_string();
        let (mut width, mut height, mut scale) = (1200.0, 800.0, 2.0);
        let mut gpu_sync = true;
        let mut gpu_timestamps = true;
        let mut it = args.iter();
        while let Some(flag) = it.next() {
            let mut value = || {
                it.next()
                    .cloned()
                    .ok_or_else(|| format!("{flag} needs a value"))
            };
            match flag.as_str() {
                "-h" | "--help" => return Ok(None),
                "--out" => out = Some(PathBuf::from(value()?)),
                "--scenario" => picked.push(value()?),
                "--frames" => {
                    frames = value()?
                        .parse::<usize>()
                        .ok()
                        .filter(|n| *n > 0)
                        .ok_or("--frames takes a count above zero")?;
                }
                "--theme" => theme = value()?,
                "--width" => width = positive(flag, &value()?)?,
                "--height" => height = positive(flag, &value()?)?,
                "--scale" => scale = positive(flag, &value()?)?,
                "--no-gpu-sync" => gpu_sync = false,
                "--no-gpu-timestamps" => gpu_timestamps = false,
                other => return Err(format!("unknown argument '{other}'")),
            }
        }
        for key in &picked {
            if !scenarios::ALL.iter().any(|(k, _)| k == key) {
                return Err(format!("no scenario '{key}'"));
            }
        }
        if picked.is_empty() {
            picked = scenarios::ALL.iter().map(|(k, _)| k.to_string()).collect();
        }
        Ok(Some(Request {
            out,
            scenarios: picked,
            config: Config {
                viewport: Viewport::new(width, height, scale),
                frames,
                theme,
                gpu_sync,
                gpu_timestamps,
            },
        }))
    }

    fn positive(flag: &str, raw: &str) -> Result<f32, String> {
        match raw.parse::<f32>() {
            Ok(v) if v.is_finite() && v > 0.0 => Ok(v),
            _ => Err(format!("{flag} takes a positive number, got '{raw}'")),
        }
    }

    pub fn measure(r: &Request) -> Result<(), String> {
        stages::enable(true);
        stages::set_gpu_sync(r.config.gpu_sync);
        let mut runs = Vec::new();
        for key in &r.scenarios {
            eprintln!("measure: {key}");
            runs.push(scenarios::run(key, &r.config)?);
        }
        stages::enable(false);
        let build = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        let md = report::markdown(&r.config, &runs, build);
        print!("{md}");
        if let Some(path) = &r.out {
            std::fs::write(path, &md).map_err(|e| format!("{}: {e}", path.display()))?;
            eprintln!("measure: wrote {}", path.display());
        }
        Ok(())
    }
}
