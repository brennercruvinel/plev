//! Structural regression for the showcase (task-67): a frame is compared
//! first by what it means, then by what it draws, and only then by its
//! pixels.
//!
//! ```text
//! cargo run -p showcase --example regress -- check [--page <title>]...
//!     [--theme core] [--width 1280 --height 800 --scale 1] [--dir <dir>]
//!     [--json <file>]
//! cargo run -p showcase --example regress -- bless --page Todo [--theme core]
//!     [--width 1280 --height 800 --scale 1] [--all] [--png | --no-png]
//!     [--dir <dir>]
//! ```
//!
//! A baseline is one (page, theme, viewport) under
//! `crates/showcase/fixtures/regress` (`baseline`): the page's projection,
//! every layer under a stable name with a stable content hash (`scene`),
//! the bound controls. `bless` stages each page without a window
//! (`showcase::app::stage_page`, a fixed clock), or a page in a state
//! named by `--page` (`state`: the Silences page with the review of the
//! checkpoint b open, and adapted by task-70's operations), and writes
//! what it holds;
//! `check` stages it again and compares (`compare`): the projection by
//! `Snapshot::diff`, the layers by hash, the bindings by rect. When
//! nothing moved it ends there, without drawing a frame. When a layer's
//! hash moved and the baseline keeps its image (`bless --png`), it draws
//! the frame through `engine::headless` and compares pixels only inside
//! the regions of the layers that moved (`pixels`), then crosses the
//! changed pixels with the bindings to name the controls they fall in.
//! The report (`report`) goes by `SemId`: meaning changed or not, visual
//! changed or not, layer, pixels, region; `bless` prints the same report
//! for what it accepts.
//!
//! Exit status: 0 every baseline held, 1 something moved, 2 a malformed
//! request, 3 an error (a page or a baseline that cannot be read).

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod baseline;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod capture;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod classify;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod compare;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod determinism;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod hash;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod pixels;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod report;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod requested;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod run;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod scene;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod state;
#[cfg(all(
    test,
    not(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))
))]
mod tests;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(code) = determinism::main(&args) {
        std::process::exit(code);
    }
    match cli::parse(&args) {
        Ok(None) => println!("{}", cli::USAGE),
        Ok(Some(request)) => match run::run(&request) {
            Ok(true) => {}
            Ok(false) => std::process::exit(1),
            Err(e) => {
                eprintln!("regress: {e}");
                std::process::exit(3);
            }
        },
        Err(e) => {
            eprintln!("regress: {e}\n\n{}", cli::USAGE);
            std::process::exit(2);
        }
    }
}

#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod cli {
    use std::path::PathBuf;

    pub const USAGE: &str = "\
usage: regress check [--page <title>]... [--theme <name>]
                     [--width <px>] [--height <px>] [--scale <s>] [--dir <dir>]
                     [--json <file>]
       regress bless [--page <title>]... [--all] [--theme <name>]
                     [--width <px>] [--height <px>] [--scale <s>]
                     [--png | --no-png] [--dir <dir>]
       regress compare <base.png> <other.png> [--scale <s>] [--target <name>]
       regress capture [--page <title>]... [--theme <name>]... [--scale <s>]...
                       [--out <dir>] [--against <dir>]
check stages every baseline in the directory (narrowed by the flags) and
compares meaning, then layers and bindings; nothing moved, no frame is
drawn. a layer moved and the baseline keeps an image: the frame is drawn
and compared inside the regions that moved. the report goes by control
(semantics changed or not, visual changed or not, layer, pixels, region),
then by layer; --json writes it for a tool. exit 0 when every baseline
held, 1 when something moved.
bless stages each page and writes its baseline (semantics, layers,
bindings), printing what it accepts: the pages named with --page, every
page of the gallery with --all, else every baseline already in the
directory. --page also takes a page in a state: \"Silences, review by the
rules\", \"Silences, review by the Claude (fixture)\", \"Silences, review
by the Claude (fixture), adapted\", \"Silences, review by the rules, too
much information\", \"Silences, review by the Claude (fixture), too
much, in portuguese\", \"Silences, review by the rules, can't compare\",
\"Silences, review by the rules, larger text and more contrast\".
new baselines take
--theme core --width 1280 --height 800 --scale 1 unless given. --png
keeps the frame's image too (needs a GPU), --no-png drops it; without
either a baseline keeps what it had.
--dir defaults to crates/showcase/fixtures/regress.
compare classifies two frames, capture draws the desktop against itself
(task-77): regress compare --help, regress capture --help.";

    /// The baselines of the repository, wherever the example runs from.
    pub const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/baselines");

    pub const DEFAULT_THEME: &str = "core";
    pub const DEFAULT_WIDTH: f32 = 1280.0;
    pub const DEFAULT_HEIGHT: f32 = 800.0;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Command {
        Check,
        Bless,
    }

    #[derive(Debug, PartialEq)]
    pub struct Request {
        pub command: Command,
        pub dir: PathBuf,
        /// Page titles (`--page`, repeatable), matched as the sidebar
        /// shows them, whatever the case.
        pub pages: Vec<String>,
        /// Every page of the gallery.
        pub all: bool,
        pub theme: Option<String>,
        pub width: Option<f32>,
        pub height: Option<f32>,
        pub scale: Option<f32>,
        /// `bless --png` / `--no-png`; `None` keeps what the baseline had.
        pub png: Option<bool>,
        /// `check --json <file>`: every report as JSON.
        pub json: Option<PathBuf>,
    }

    /// `Ok(None)` asks for the usage.
    pub fn parse(args: &[String]) -> Result<Option<Request>, String> {
        let mut it = args.iter();
        let command = match it.next().map(String::as_str) {
            None | Some("-h" | "--help") => return Ok(None),
            Some("check") => Command::Check,
            Some("bless") => Command::Bless,
            Some(other) => return Err(format!("unknown command '{other}'")),
        };
        let mut r = Request {
            command,
            dir: PathBuf::from(FIXTURES),
            pages: Vec::new(),
            all: false,
            theme: None,
            width: None,
            height: None,
            scale: None,
            png: None,
            json: None,
        };
        while let Some(flag) = it.next() {
            let mut value = || {
                it.next()
                    .cloned()
                    .ok_or_else(|| format!("{flag} needs a value"))
            };
            match flag.as_str() {
                "-h" | "--help" => return Ok(None),
                "--dir" => r.dir = PathBuf::from(value()?),
                "--page" => r.pages.push(value()?),
                "--all" => r.all = true,
                "--theme" => r.theme = Some(value()?),
                "--width" => r.width = Some(positive(flag, &value()?)?),
                "--height" => r.height = Some(positive(flag, &value()?)?),
                "--scale" => r.scale = Some(positive(flag, &value()?)?),
                "--png" => r.png = Some(true),
                "--no-png" => r.png = Some(false),
                "--json" => r.json = Some(PathBuf::from(value()?)),
                other => return Err(format!("unknown argument '{other}'")),
            }
        }
        if r.all && !r.pages.is_empty() {
            return Err("--all and --page exclude each other".to_string());
        }
        if r.all && r.command == Command::Check {
            return Err("--all is for bless; check reads every baseline already".to_string());
        }
        if r.png.is_some() && r.command == Command::Check {
            return Err("--png and --no-png are for bless".to_string());
        }
        if r.json.is_some() && r.command == Command::Bless {
            return Err("--json is for check".to_string());
        }
        Ok(Some(r))
    }

    fn positive(flag: &str, raw: &str) -> Result<f32, String> {
        match raw.parse::<f32>() {
            Ok(v) if v.is_finite() && v > 0.0 => Ok(v),
            _ => Err(format!("{flag} takes a positive number, got '{raw}'")),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn args(s: &str) -> Vec<String> {
            s.split_whitespace().map(String::from).collect()
        }

        #[test]
        fn bless_takes_pages_a_theme_and_a_viewport() {
            let r = match parse(&args(
                "bless --page Todo --page Button --theme hoff --width 390 --height 844 --scale 3",
            )) {
                Ok(Some(r)) => r,
                other => panic!("{other:?}"),
            };
            assert_eq!(r.command, Command::Bless);
            assert_eq!(r.pages, ["Todo", "Button"]);
            assert_eq!(r.theme.as_deref(), Some("hoff"));
            assert_eq!(
                (r.width, r.height, r.scale),
                (Some(390.0), Some(844.0), Some(3.0))
            );
            assert_eq!(r.dir, PathBuf::from(FIXTURES));
            assert!(!r.all);
            assert_eq!(r.png, None);
            let png = |line: &str| match parse(&args(line)) {
                Ok(Some(r)) => r.png,
                other => panic!("{other:?}"),
            };
            assert_eq!(png("bless --png"), Some(true));
            assert_eq!(png("bless --png --no-png"), Some(false));
        }

        #[test]
        fn malformed_requests_are_errors_with_the_reason() {
            for (line, needle) in [
                ("frob", "unknown command"),
                ("bless --page", "needs a value"),
                ("bless --width 0", "--width"),
                ("bless --fast", "unknown argument"),
                ("bless --all --page Todo", "exclude"),
                ("check --all", "for bless"),
                ("check --png", "for bless"),
                ("bless --json r.json", "for check"),
            ] {
                match parse(&args(line)) {
                    Err(e) => assert!(e.contains(needle), "{line}: '{e}' lacks '{needle}'"),
                    Ok(r) => panic!("{line}: expected an error, got {r:?}"),
                }
            }
            assert_eq!(parse(&args("")), Ok(None));
            match parse(&args("check --page Todo --dir d")) {
                Ok(Some(r)) => {
                    assert_eq!(r.command, Command::Check);
                    assert_eq!(
                        (r.pages, r.dir),
                        (vec!["Todo".to_string()], PathBuf::from("d"))
                    );
                }
                other => panic!("{other:?}"),
            }
            assert_eq!(parse(&args("bless --help")), Ok(None));
        }
    }
}
