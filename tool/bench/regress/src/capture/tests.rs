//! The capture's command line, and a small capture end to end on a GPU:
//! two runs, the second against the first, and a png changed between
//! them. The GPU part skips without an adapter, as the headless tests do.

use super::*;

fn args(s: &str) -> Vec<String> {
    s.split_whitespace().map(String::from).collect()
}

fn options(line: &str) -> Options {
    match parse(&args(line)) {
        Ok(Some(o)) => o,
        other => panic!("{line}: {other:?}"),
    }
}

#[test]
fn the_defaults_are_every_page_the_four_themes_and_the_four_scales() {
    let o = options("");
    assert!(o.pages.is_empty());
    assert_eq!(o.themes, THEMES);
    assert_eq!(o.scales, SCALES);
    assert_eq!((o.width, o.height), (1280.0, 800.0));
    assert!(o.out.ends_with("target/determinism/desktop"));
    assert!(o.png && !o.reverse && o.against.is_none());
    let all = match keys(&o) {
        Ok(k) => k,
        Err(e) => panic!("{e}"),
    };
    assert_eq!(all.len(), PAGES.len() * 4 * 4);
    // scale by scale, theme by theme, page by page
    assert_eq!(all[0].label(), "theme@core@1280x800");
    assert_eq!(all[1].page, PAGES[1].title);
    assert_eq!(all[PAGES.len()].theme, "hoff");
    assert_eq!(all[PAGES.len() * 4].viewport.scale, 2.0);
}

#[test]
fn flags_narrow_the_capture_and_bad_ones_are_errors() {
    let o = options(
        "--page todo --page Button --theme hoff --scale 1.25 --width 390 --height 844 \
         --out o --against a --reverse --no-png",
    );
    assert_eq!(o.pages, ["todo", "Button"]);
    assert_eq!(o.themes, ["hoff"]);
    assert_eq!(o.scales, [1.25]);
    assert_eq!((o.width, o.height), (390.0, 844.0));
    assert_eq!(o.out, PathBuf::from("o"));
    assert_eq!(o.against, Some(PathBuf::from("a")));
    assert!(o.reverse && !o.png);
    let k = match keys(&o) {
        Ok(k) => k,
        Err(e) => panic!("{e}"),
    };
    // the titles as the sidebar has them, in reverse
    let labels: Vec<String> = k.iter().map(Key::label).collect();
    assert_eq!(
        labels,
        ["button@hoff@390x844@1.25x", "todo@hoff@390x844@1.25x"]
    );
    for (line, needle) in [
        ("--scale 0", "positive number"),
        ("--width", "needs a value"),
        ("--frob", "unknown argument"),
    ] {
        match parse(&args(line)) {
            Err(e) => assert!(e.contains(needle), "{line}: '{e}' lacks '{needle}'"),
            Ok(o) => panic!("{line}: {o:?}"),
        }
    }
    assert_eq!(parse(&args("--help")), Ok(None));
    match keys(&options("--page Nothing")) {
        Err(e) => assert!(e.contains("no page titled 'Nothing'"), "{e}"),
        Ok(k) => panic!("{k:?}"),
    }
}

/// Two pages, one theme, scales 1 and 1.25, a small viewport: every
/// frame equal across contexts; a second run against the first equal
/// across processes; a png of the first run changed, that frame silent
/// across processes and measured, the others still equal.
#[test]
fn two_runs_hold_against_each_other_and_a_changed_png_is_silent() {
    let root = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => panic!("{e}"),
    };
    let first = root.path().join("a");
    let line = format!(
        "--page Button --page Card --theme hoff --scale 1 --scale 1.25 \
         --width 320 --height 240 --out {}",
        first.display()
    );
    let o = options(&line);
    match run(&o) {
        Ok(true) => {}
        Ok(false) => panic!("a frame did not hold across two contexts"),
        Err(e) if e.contains("no GPU adapter") => {
            eprintln!("no GPU adapter; skipping");
            return;
        }
        Err(e) => panic!("{e}"),
    }
    let text = match std::fs::read_to_string(first.join("report.txt")) {
        Ok(t) => t,
        Err(e) => panic!("{e}"),
    };
    for needle in [
        "2 pages x 1 themes x 2 scales = 4 frames at 320x240 logical",
        "two GPU contexts of this process: 4 equal, 0 requested, 0 silent, 0 resized",
        "Button hoff   equal    equal",
        "every frame held",
    ] {
        assert!(text.contains(needle), "{needle}\n---\n{text}");
    }
    assert!(first.join("report.json").is_file());
    // the png of Card at 1.25x: 400 x 300, one pixel nudged
    let card = first.join("card@hoff@320x240@1.25x.png");
    let mut frame = match read_png(&card) {
        Ok(f) => f,
        Err(e) => panic!("{e}"),
    };
    assert_eq!((frame.width, frame.height), (400, 300));
    let at = ((10 * 400 + 7) * 4) as usize;
    let red = frame.rgba[at];
    frame.rgba[at] = if red > 250 { red - 3 } else { red + 3 };
    if let Err(e) = frame.write_png(&card) {
        panic!("{e}");
    }
    let second = root.path().join("b");
    let o = options(&format!(
        "{line} --out {} --against {} --reverse --no-png",
        second.display(),
        first.display()
    ));
    match run(&o) {
        Ok(false) => {}
        other => panic!("the changed png held: {other:?}"),
    }
    let text = match std::fs::read_to_string(second.join("report.txt")) {
        Ok(t) => t,
        Err(e) => panic!("{e}"),
    };
    for needle in [
        "4 equal, 0 requested, 0 silent, 0 resized\nthis process against",
        ": 3 equal, 0 requested, 1 silent, 0 resized",
        "equal / silent 1 px, max 3",
        "1 frames did not hold",
        "card@hoff@320x240@1.25x: contexts: equal in every byte; processes: silent: 1 px \
         differ in no declared area, delta max 3 mean 3.00, at 7,10 1x1",
    ] {
        assert!(text.contains(needle), "{needle}\n---\n{text}");
    }
    // --no-png: the second run wrote its report and no frame
    assert!(!second.join("card@hoff@320x240@1.25x.png").exists());
    // against a directory without the pngs: an error that names the file
    let o = options(&format!(
        "{line} --out {} --against {}",
        first.display(),
        second.display()
    ));
    match run(&o) {
        Err(e) => assert!(e.contains("button@hoff@320x240.png"), "{e}"),
        Ok(v) => panic!("ran against missing pngs: {v}"),
    }
}
