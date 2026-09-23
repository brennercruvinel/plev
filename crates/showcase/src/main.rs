//! Desktop/web binary entry: delegates to the showcase library, which owns
//! the winit application shell and every per-platform entry point. Mobile
//! does not use this binary: android loads the cdylib's `android_main` and
//! iOS links the staticlib's `showcase_ios_main` instead.
//!
//! On desktop the binary also draws a page without a window:
//! `showcase --render <page> ... --out file.png` renders one frame through
//! the window's frame path (`showcase::app::render_page`) and exits.

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match cli::parse(&args) {
        Ok(cli::Mode::Window) => showcase::run(),
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
    showcase::run_web();
}

// Keep the binary target compilable when the whole crate is built for a
// mobile target; the real entry points live in the library.
#[cfg(any(target_os = "android", target_os = "ios"))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod cli {
    use std::path::PathBuf;

    use engine::gpu::RenderConfig;
    use engine::headless::{Renderer, Viewport};

    pub const USAGE: &str = "\
usage:
  showcase [page] [theme] [scroll]      open the gallery window
  showcase --render <page> --out <file.png>
           [--theme <name>] [--width <px>] [--height <px>] [--scale <s>] [--scroll <px>]
                                        draw one frame of a page, no window
defaults: --theme core --width 1200 --height 800 --scale 1 (the window's
launch size); width and height are logical pixels, the PNG is width*scale
by height*scale. page titles are the sidebar's (\"Data Table\").";

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
        pub out: PathBuf,
    }

    pub fn parse(args: &[String]) -> Result<Mode, String> {
        if !args.iter().any(|a| a.starts_with("--") || a == "-h") {
            return Ok(Mode::Window);
        }
        let mut page = None;
        let mut out = None;
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
                "--semantics" => {
                    return Err("--semantics is not available yet: the semantic \
                                projection of a page arrives with task-59"
                        .to_string());
                }
                "--render" => page = Some(value()?),
                "--out" => out = Some(PathBuf::from(value()?)),
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
        let out = out.ok_or("--out <file.png> is required")?;
        Ok(Mode::Render(Request {
            page,
            theme,
            width,
            height,
            scale,
            scroll,
            out,
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

    pub fn render(r: &Request) -> Result<(), Box<dyn std::error::Error>> {
        let viewport = Viewport::new(r.width, r.height, r.scale);
        let mut renderer = Renderer::new(viewport, RenderConfig::default())?;
        let frame = showcase::app::render_page(&mut renderer, &r.page, &r.theme, r.scroll)?;
        frame.write_png(&r.out)?;
        println!(
            "wrote {} ({}x{}, {} / {} at {}x{} scale {})",
            r.out.display(),
            frame.width,
            frame.height,
            r.page,
            r.theme,
            r.width,
            r.height,
            r.scale
        );
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
            assert_eq!(r.out, PathBuf::from("a.png"));
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
                ("--render Button", "--out"),
                ("--out a.png", "--render"),
                ("--render", "needs a value"),
                ("--render Button --out a.png --width wide", "--width"),
                ("--render Button --out a.png --scale 0", "--scale"),
                ("--render Button --out a.png --height -5", "--height"),
                ("--render Button --out a.png --scroll -1", "--scroll"),
                ("--render Button --out a.png --fast", "unknown flag"),
                ("--render Button --out a.png extra", "unexpected"),
                ("--semantics out.json", "not available yet"),
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
