//! Structural regression for the showcase (task-67): a frame is compared
//! first by what it means, then by what it draws, and only then by its
//! pixels.
//!
//! ```text
//! cargo run -p showcase --example regress -- bless --page Todo [--theme core]
//!     [--width 1280 --height 800 --scale 1] [--all] [--dir <dir>]
//! ```
//!
//! A baseline is one (page, theme, viewport) under
//! `crates/showcase/fixtures/regress` (`baseline`): the page's projection,
//! every layer under a stable name with a stable content hash (`scene`),
//! the bound controls. `bless` stages each page without a window
//! (`showcase::app::stage_page`, a fixed clock) and writes what it holds.

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod baseline;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod run;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod scene;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match cli::parse(&args) {
        Ok(None) => println!("{}", cli::USAGE),
        Ok(Some(request)) => match run::run(&request) {
            Ok(()) => {}
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
usage: regress bless [--page <title>]... [--all] [--theme <name>]
                     [--width <px>] [--height <px>] [--scale <s>] [--dir <dir>]
bless stages each page and writes its baseline (semantics, layers,
bindings): the pages named with --page, every page of the gallery with
--all, else every baseline already in the directory. new baselines take
--theme core --width 1280 --height 800 --scale 1 unless given.
--dir defaults to crates/showcase/fixtures/regress.";

    /// The baselines of the repository, wherever the example runs from.
    pub const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/regress");

    pub const DEFAULT_THEME: &str = "core";
    pub const DEFAULT_WIDTH: f32 = 1280.0;
    pub const DEFAULT_HEIGHT: f32 = 800.0;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Command {
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
    }

    /// `Ok(None)` asks for the usage.
    pub fn parse(args: &[String]) -> Result<Option<Request>, String> {
        let mut it = args.iter();
        let command = match it.next().map(String::as_str) {
            None | Some("-h" | "--help") => return Ok(None),
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
                other => return Err(format!("unknown argument '{other}'")),
            }
        }
        if r.all && !r.pages.is_empty() {
            return Err("--all and --page exclude each other".to_string());
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
        }

        #[test]
        fn malformed_requests_are_errors_with_the_reason() {
            for (line, needle) in [
                ("frob", "unknown command"),
                ("bless --page", "needs a value"),
                ("bless --width 0", "--width"),
                ("bless --fast", "unknown argument"),
                ("bless --all --page Todo", "exclude"),
            ] {
                match parse(&args(line)) {
                    Err(e) => assert!(e.contains(needle), "{line}: '{e}' lacks '{needle}'"),
                    Ok(r) => panic!("{line}: expected an error, got {r:?}"),
                }
            }
            assert_eq!(parse(&args("")), Ok(None));
            assert_eq!(parse(&args("bless --help")), Ok(None));
        }
    }
}
