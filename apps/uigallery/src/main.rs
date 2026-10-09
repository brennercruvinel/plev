//! Desktop binary entry (trunk builds `showcase-web` for the web, see
//! `main_web.rs`): delegates to the showcase library, which owns
//! the winit application shell and every per-platform entry point. Mobile
//! does not use this binary: android loads the cdylib's `android_main` and
//! iOS links the staticlib's `uigallery_ios_main` instead.
//!
//! On desktop the binary also draws a page without a window:
//! `showcase --render <page> ... --out file.png` renders one frame through
//! the window's frame path (`showcase::app::stage_page`), `--svg file.svg`
//! writes the same frame as SVG with every approximation reported, and
//! `--semantics file.json` writes the page's semantic projection in the
//! form `plev observe` serves; then it exits.

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match cli::parse(&args) {
        Ok(cli::Mode::Window) => uigallery::run(),
        Ok(cli::Mode::Help) => println!("{}", cli::USAGE),
        Ok(cli::Mode::Render(request)) => {
            if let Err(e) = cli::render(&request) {
                eprintln!("showcase: {e}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("showcase: {e}\n\n{}", cli::USAGE);
            std::process::exit(2);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    uigallery::run_web();
}

// Keep the binary target compilable when the whole crate is built for a
// mobile target; the real entry points live in the library.
#[cfg(any(target_os = "android", target_os = "ios"))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod cli {
    use std::path::PathBuf;

    use engine::drawing::gpu::RenderConfig;
    use engine::drawing::headless::{Renderer, Viewport};

    pub const USAGE: &str = "\
usage:
  showcase [page] [theme] [scroll]      open the gallery window
  showcase --render <page> [--out <file.png>] [--svg <file.svg>] [--semantics <file.json>]
           [--svg-report <file.txt>] [--source]
           [--theme <name>] [--width <px>] [--height <px>] [--scale <s>] [--scroll <px>]
                                        one frame of a page, no window: as png, as svg (every
                                        approximation reported on stderr, and to --svg-report
                                        when given), and the page's semantic projection as json
                                        (the sem form `plev observe` serves; --source keeps each
                                        node's source). at least one of --out, --svg, --semantics.
defaults: --theme core --width 1200 --height 800 --scale 1 (the window's
launch size); width and height are logical pixels, the PNG is width*scale
by height*scale and the SVG's viewBox is the logical size. page titles are
the sidebar's (\"Data Table\").";

    /// The window's launch size and theme, the defaults of `--render`.
    const DEFAULT_THEME: &str = "core";
    const DEFAULT_WIDTH: f32 = 1200.0;
    const DEFAULT_HEIGHT: f32 = 800.0;

    #[derive(Debug, PartialEq)]
    pub enum Mode {
        /// No flags: the window, which reads its positional launch state.
        Window,
        Help,
        Render(Request),
    }

    #[derive(Debug, PartialEq)]
    pub struct Request {
        pub page: String,
        pub theme: String,
        pub width: f32,
        pub height: f32,
        pub scale: f32,
        pub scroll: Option<f32>,
        pub out: Option<PathBuf>,
        pub svg: Option<PathBuf>,
        pub svg_report: Option<PathBuf>,
        pub semantics: Option<PathBuf>,
        pub source: bool,
    }

    pub fn parse(args: &[String]) -> Result<Mode, String> {
        if !args.iter().any(|a| a.starts_with("--") || a == "-h") {
            return Ok(Mode::Window);
        }
        let mut page = None;
        let (mut out, mut svg, mut svg_report, mut semantics) = (None, None, None, None);
        let mut source = false;
        let mut theme = DEFAULT_THEME.to_string();
        let (mut width, mut height, mut scale) = (DEFAULT_WIDTH, DEFAULT_HEIGHT, 1.0);
        let mut scroll = None;

        let mut it = args.iter();
        while let Some(flag) = it.next() {
            let mut value = || {
                it.next()
                    .cloned()
                    .ok_or_else(|| format!("{flag} needs a value"))
            };
            match flag.as_str() {
                "-h" | "--help" => return Ok(Mode::Help),
                "--render" => page = Some(value()?),
                "--out" => out = Some(PathBuf::from(value()?)),
                "--svg" => svg = Some(PathBuf::from(value()?)),
                "--svg-report" => svg_report = Some(PathBuf::from(value()?)),
                "--semantics" => semantics = Some(PathBuf::from(value()?)),
                "--source" => source = true,
                "--theme" => theme = value()?,
                "--width" => width = positive(flag, &value()?)?,
                "--height" => height = positive(flag, &value()?)?,
                "--scale" => scale = positive(flag, &value()?)?,
                "--scroll" => scroll = Some(offset(flag, &value()?)?),
                other if other.starts_with("--") => return Err(format!("unknown flag {other}")),
                other => {
                    return Err(format!(
                        "unexpected argument '{other}' (positional launch \
                         arguments only open the window)"
                    ));
                }
            }
        }
        let page = page.ok_or("--render <page> is required")?;
        if out.is_none() && svg.is_none() && semantics.is_none() {
            return Err(
                "--render needs an output: --out <file.png>, --svg <file.svg> \
                        or --semantics <file.json>"
                    .to_string(),
            );
        }
        if svg_report.is_some() && svg.is_none() {
            return Err("--svg-report needs --svg <file.svg>".to_string());
        }
        if source && semantics.is_none() {
            return Err("--source needs --semantics <file.json>".to_string());
        }
        Ok(Mode::Render(Request {
            page,
            theme,
            width,
            height,
            scale,
            scroll,
            out,
            svg,
            svg_report,
            semantics,
            source,
        }))
    }

    fn positive(flag: &str, raw: &str) -> Result<f32, String> {
        match raw.parse::<f32>() {
            Ok(v) if v.is_finite() && v > 0.0 => Ok(v),
            _ => Err(format!("{flag} takes a positive number, got '{raw}'")),
        }
    }

    fn offset(flag: &str, raw: &str) -> Result<f32, String> {
        match raw.parse::<f32>() {
            Ok(v) if v.is_finite() && v >= 0.0 => Ok(v),
            _ => Err(format!("{flag} takes a number of px >= 0, got '{raw}'")),
        }
    }

    /// One staged frame, then each output asked for. The GPU is opened
    /// only when pixels or glyph outlines are wanted; the projection alone
    /// needs none.
    pub fn render(r: &Request) -> Result<(), Box<dyn std::error::Error>> {
        let viewport = Viewport::new(r.width, r.height, r.scale);
        let mut staged = uigallery::shell::stage_page(viewport, &r.page, &r.theme, r.scroll)?;
        let where_ = format!(
            "{} / {} at {}x{} scale {}",
            staged.title(),
            r.theme,
            r.width,
            r.height,
            r.scale
        );
        if r.out.is_some() || r.svg.is_some() {
            let mut renderer = Renderer::new(viewport, RenderConfig::default())?;
            let frame = staged.render(&mut renderer)?;
            if let Some(out) = &r.out {
                frame.write_png(out)?;
                println!(
                    "wrote {} ({}x{}, {where_})",
                    out.display(),
                    frame.width,
                    frame.height
                );
            }
            if let Some(path) = &r.svg {
                let export = staged.svg(&mut renderer)?;
                export.write(path)?;
                eprint!("{}", export.report_text());
                if let Some(report) = &r.svg_report {
                    std::fs::write(report, export.report_text())?;
                }
                println!(
                    "wrote {} ({} approximations reported, {where_})",
                    path.display(),
                    export.report.len()
                );
            }
        }
        if let Some(path) = &r.semantics {
            let snapshot = staged.semantics()?;
            std::fs::write(path, uigallery::shell::semantics_json(&snapshot, r.source)?)?;
            println!(
                "wrote {} ({} nodes, revision {}, {where_})",
                path.display(),
                snapshot.len(),
                snapshot.revision()
            );
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn args(s: &str) -> Vec<String> {
            s.split_whitespace().map(String::from).collect()
        }

        fn request(s: &str) -> Request {
            match parse(&args(s)) {
                Ok(Mode::Render(r)) => r,
                other => panic!("{s}: expected a render request, got {other:?}"),
            }
        }

        #[test]
        fn positional_arguments_still_open_the_window() {
            assert_eq!(parse(&[]), Ok(Mode::Window));
            assert_eq!(parse(&args("Button hoff 120")), Ok(Mode::Window));
        }

        #[test]
        fn render_takes_every_flag_in_any_order() {
            let r = request(
                "--out a.png --scale 3 --render Button --theme hoff \
                 --width 390 --height 844 --scroll 120",
            );
            assert_eq!(r.page, "Button");
            assert_eq!(r.theme, "hoff");
            assert_eq!((r.width, r.height, r.scale), (390.0, 844.0, 3.0));
            assert_eq!(r.scroll, Some(120.0));
            assert_eq!(r.out, Some(PathBuf::from("a.png")));
            assert_eq!((r.svg, r.svg_report, r.semantics), (None, None, None));
            assert!(!r.source);
        }

        #[test]
        fn svg_and_semantics_are_outputs_of_the_same_frame() {
            let r = request(
                "--render Todo --out a.png --svg a.svg --svg-report a.txt \
                 --semantics a.json --source",
            );
            assert_eq!(r.out, Some(PathBuf::from("a.png")));
            assert_eq!(r.svg, Some(PathBuf::from("a.svg")));
            assert_eq!(r.svg_report, Some(PathBuf::from("a.txt")));
            assert_eq!(r.semantics, Some(PathBuf::from("a.json")));
            assert!(r.source);
            // each output stands alone
            assert!(request("--render Todo --svg a.svg").out.is_none());
            let only = request("--render Todo --semantics a.json");
            assert!(only.out.is_none() && only.svg.is_none());
            assert!(!only.source);
        }

        #[test]
        fn render_defaults_to_the_window_launch_state() {
            let r = request("--render Badge --out b.png");
            assert_eq!(r.theme, "core");
            assert_eq!((r.width, r.height, r.scale), (1200.0, 800.0, 1.0));
            assert_eq!(r.scroll, None);
        }

        #[test]
        fn a_page_title_with_spaces_is_one_argument() {
            let a = vec![
                "--render".to_string(),
                "Data Table".to_string(),
                "--out".to_string(),
                "t.png".to_string(),
            ];
            match parse(&a) {
                Ok(Mode::Render(r)) => assert_eq!(r.page, "Data Table"),
                other => panic!("{other:?}"),
            }
        }

        #[test]
        fn malformed_requests_are_errors_with_the_reason() {
            for (line, needle) in [
                (
                    "--render Button",
                    "--out <file.png>, --svg <file.svg> or --semantics",
                ),
                ("--out a.png", "--render"),
                ("--semantics out.json", "--render"),
                (
                    "--render Button --out a.png --svg-report r.txt",
                    "--svg-report needs --svg",
                ),
                (
                    "--render Button --out a.png --source",
                    "--source needs --semantics",
                ),
                ("--render Button --svg", "needs a value"),
                ("--render", "needs a value"),
                ("--render Button --out a.png --width wide", "--width"),
                ("--render Button --out a.png --scale 0", "--scale"),
                ("--render Button --out a.png --height -5", "--height"),
                ("--render Button --out a.png --scroll -1", "--scroll"),
                ("--render Button --out a.png --fast", "unknown flag"),
                ("--render Button --out a.png extra", "unexpected"),
            ] {
                match parse(&args(line)) {
                    Err(e) => assert!(e.contains(needle), "{line}: '{e}' lacks '{needle}'"),
                    Ok(m) => panic!("{line}: expected an error, got {m:?}"),
                }
            }
        }

        #[test]
        fn scroll_zero_is_the_top_of_the_page() {
            assert_eq!(
                request("--render Card --out c.png --scroll 0").scroll,
                Some(0.0)
            );
        }

        #[test]
        fn help_wins() {
            assert_eq!(parse(&args("--render Button --help")), Ok(Mode::Help));
            assert_eq!(parse(&args("-h")), Ok(Mode::Help));
        }
    }
}
