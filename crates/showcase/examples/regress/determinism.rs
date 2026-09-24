//! The determinism commands (task-77): two frames classified by the
//! comparator (`classify`).
//!
//! ```text
//! cargo run -p showcase --example regress -- compare <base.png> <other.png>
//!     [--scale <s>]
//! ```
//!
//! Exit status as the other commands: 0 equal, or different only as
//! asked; 1 a silent difference or two sizes; 2 a malformed request; 3
//! an error (a png that cannot be read).

use std::path::PathBuf;

use crate::baseline::read_png;
use crate::classify::{Verdict, classify};

pub const USAGE: &str = "\
usage: regress compare <base.png> <other.png> [--scale <s>]
compare classifies other against base: equal in every byte; requested
(every differing pixel in an area a decision of the product declares);
silent (a differing pixel no decision explains, measured: pixels, the
largest and the mean channel delta, the rect that holds them); resized
(two sizes, not compared by pixel). --scale is the frames' scale, 1
unless given: it maps the declared areas, in logical px, onto pixels.
exit 0 when equal or requested, 1 when silent or resized.";

/// `compare`: two pngs and the scale they were drawn at.
#[derive(Debug, PartialEq)]
pub struct Compare {
    pub base: PathBuf,
    pub other: PathBuf,
    pub scale: f32,
}

/// What a determinism command line asks.
#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Compare(Compare),
}

/// The exit status of `args` (the command first) when they name a
/// determinism command, `None` for any other.
pub fn main(args: &[String]) -> Option<i32> {
    if args.first().map(String::as_str) != Some("compare") {
        return None;
    }
    let code = match parse(args) {
        Ok(Command::Help) => {
            println!("{USAGE}");
            0
        }
        Ok(Command::Compare(c)) => match compare(&c) {
            Ok(v) if v.held() => 0,
            Ok(_) => 1,
            Err(e) => {
                eprintln!("regress: {e}");
                3
            }
        },
        Err(e) => {
            eprintln!("regress: {e}\n\n{USAGE}");
            2
        }
    };
    Some(code)
}

/// `args` with the command first.
pub fn parse(args: &[String]) -> Result<Command, String> {
    let mut files = Vec::new();
    let mut scale = 1.0;
    let mut it = args.iter().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "--scale" => {
                let raw = it.next().ok_or("--scale needs a value")?;
                scale = match raw.parse::<f32>() {
                    Ok(v) if v.is_finite() && v > 0.0 => v,
                    _ => return Err(format!("--scale takes a positive number, got '{raw}'")),
                };
            }
            flag if flag.starts_with("--") => return Err(format!("unknown argument '{flag}'")),
            file => files.push(PathBuf::from(file)),
        }
    }
    let [base, other] = <[PathBuf; 2]>::try_from(files)
        .map_err(|f| format!("compare takes two pngs, got {}", f.len()))?;
    Ok(Command::Compare(Compare { base, other, scale }))
}

/// Read both pngs, classify, print the verdict.
fn compare(c: &Compare) -> Result<Verdict, String> {
    let base = read_png(&c.base)?;
    let other = read_png(&c.other)?;
    let verdict = classify(&base, &other, c.scale, &[]);
    println!(
        "{} against {} at {}x: {}",
        c.other.display(),
        c.base.display(),
        c.scale,
        verdict.line()
    );
    Ok(verdict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::headless::Frame;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn compare_takes_two_pngs_and_a_scale() {
        assert_eq!(
            parse(&args("compare a.png b.png --scale 1.5")),
            Ok(Command::Compare(Compare {
                base: PathBuf::from("a.png"),
                other: PathBuf::from("b.png"),
                scale: 1.5,
            }))
        );
        match parse(&args("compare a.png b.png")) {
            Ok(Command::Compare(c)) => assert_eq!(c.scale, 1.0),
            other => panic!("{other:?}"),
        }
        assert_eq!(parse(&args("compare --help")), Ok(Command::Help));
        for (line, needle) in [
            ("compare a.png", "two pngs, got 1"),
            ("compare a.png b.png c.png", "two pngs, got 3"),
            ("compare a.png b.png --scale 0", "--scale"),
            ("compare a.png b.png --scale", "needs a value"),
            ("compare a.png b.png --fast", "unknown argument"),
        ] {
            match parse(&args(line)) {
                Err(e) => assert!(e.contains(needle), "{line}: '{e}' lacks '{needle}'"),
                Ok(c) => panic!("{line}: expected an error, got {c:?}"),
            }
        }
        assert_eq!(main(&args("check")), None);
    }

    #[test]
    fn two_pngs_on_disk_are_read_and_classified() {
        let dir = match tempfile::tempdir() {
            Ok(d) => d,
            Err(e) => panic!("{e}"),
        };
        let gray = Frame {
            width: 4,
            height: 2,
            rgba: [48, 48, 48, 255].repeat(8),
        };
        let mut lit = gray.clone();
        lit.rgba[4 * 5] = 50;
        for (name, f) in [("a.png", &gray), ("b.png", &gray), ("c.png", &lit)] {
            if let Err(e) = f.write_png(dir.path().join(name)) {
                panic!("{e}");
            }
        }
        let run = |other: &str| {
            compare(&Compare {
                base: dir.path().join("a.png"),
                other: dir.path().join(other),
                scale: 1.0,
            })
        };
        assert_eq!(run("b.png"), Ok(Verdict::Equal));
        match run("c.png") {
            Ok(Verdict::Silent(s)) => {
                assert_eq!((s.pixels, s.max_delta, s.bbox), (1, 2, [1, 1, 1, 1]))
            }
            other => panic!("{other:?}"),
        }
        match run("missing.png") {
            Err(e) => assert!(e.contains("missing.png"), "{e}"),
            Ok(v) => panic!("read a missing png: {v:?}"),
        }
    }
}
