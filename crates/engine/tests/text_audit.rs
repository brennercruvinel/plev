//! Text coverage audit by script (task-74, phase 1).
//!
//! The corpus in `tests/text_corpus/` has one short, readable phrase per
//! script. Each phrase is shaped with every embedded family that makes
//! sense for text (Inclusive Sans and Inter), against the embedded faces
//! alone (what wasm and mobile ship) and, for the record, against this
//! machine's system fonts too (what the desktop path resolves).
//!
//! `shaping_matches_the_golden` is the measure phase 3 onward compares
//! against: glyphs, `.notdef` count, measured width, line breaks at a
//! wrap width and the visual order of the bidi runs, embedded faces only,
//! so the table is the same on every machine. It fails when the table
//! moves; `TEXT_AUDIT_UPDATE=1` rewrites `tests/text_audit.txt` after a
//! change that was meant.
//!
//! `pixels_match_the_measured_width` renders every phrase through the
//! frame path and compares the ink with the width the measurer promised.
//! It needs a GPU adapter (skips without one), writes one PNG per family
//! to `TEXT_AUDIT_OUT` (default: the OS temp dir under `plev-text-audit`)
//! and records the engine's once-per-face fallback warning.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use engine::text::probe::{Faces, Rendered, ShapeReport, Specimen, render, shape_report};
use engine::text::{TextMeasurer, TextStyle};

const FONT_SIZE: f32 = 16.0;
const LINE_HEIGHT: f32 = 22.0;
/// Wrap width for the line-break column: narrow enough that every phrase
/// of more than one word breaks at least once.
const WRAP_WIDTH: f32 = 80.0;
/// `(label, TextStyle::font_family)` of the embedded families used for
/// text. Codicons is an icon font and shapes no text.
const FAMILIES: [(&str, Option<&str>); 2] = [("Inclusive Sans", None), ("Inter", Some("Inter"))];

const SCALE: f32 = 2.0;
const ORIGIN_X: f32 = 20.0;
/// Logical pixels between phrase baselines in the rendered sheet; a band
/// this tall holds one phrase and nothing of its neighbours.
const ROW: f32 = 40.0;
const SHEET_W: u32 = 700;

// ---------------------------------------------------------------------------
// corpus
// ---------------------------------------------------------------------------

struct Phrase {
    /// File stem, `01-latin-diacritics`.
    script: String,
    /// The `# cobre:` line.
    covers: String,
    text: String,
}

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/text_corpus")
}

/// `\u{XXXX}` escapes to characters, so invisible marks stay visible in
/// the corpus file. Anything malformed is kept verbatim.
fn decode_escapes(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(at) = rest.find("\\u{") {
        out.push_str(&rest[..at]);
        let after = &rest[at + 3..];
        match after.find('}') {
            Some(close) => {
                let hex = &after[..close];
                match u32::from_str_radix(hex, 16).ok().and_then(char::from_u32) {
                    Some(c) => out.push(c),
                    None => out.push_str(&rest[at..at + 4 + close]),
                }
                rest = &after[close + 1..];
            }
            None => {
                out.push_str(&rest[at..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

fn corpus() -> Vec<Phrase> {
    let dir = corpus_dir();
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "txt"))
        .filter(|p| p.file_stem().is_some_and(|s| s != "README"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "empty corpus in {}", dir.display());

    let mut phrases = Vec::new();
    for path in files {
        let script = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let source =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let covers = source
            .lines()
            .find_map(|l| l.strip_prefix("# cobre:"))
            .map(str::trim)
            .unwrap_or("")
            .to_string();
        for line in source.lines() {
            let line = line.trim_end();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            phrases.push(Phrase {
                script: script.clone(),
                covers: covers.clone(),
                text: decode_escapes(line),
            });
        }
    }
    phrases
}

// ---------------------------------------------------------------------------
// shaping table
// ---------------------------------------------------------------------------

fn style(family: Option<&str>) -> TextStyle {
    let st = TextStyle::new(FONT_SIZE).with_line_height(LINE_HEIGHT);
    match family {
        Some(name) => st.with_family(name),
        None => st,
    }
}

/// The bidi runs of a one-line shape in visual order: direction, embedding
/// level and byte range in the paragraph, `L0:0..6 L2:17..20 R1:6..17`.
fn visual_runs(shape: &ShapeReport) -> String {
    let mut runs: Vec<(u8, usize, usize)> = Vec::new();
    for g in &shape.glyphs {
        match runs.last_mut() {
            Some((level, start, end)) if *level == g.level => {
                *start = (*start).min(g.start);
                *end = (*end).max(g.end);
            }
            _ => runs.push((g.level, g.start, g.end)),
        }
    }
    runs.iter()
        .map(|(level, s, e)| {
            let dir = if level % 2 == 1 { 'R' } else { 'L' };
            format!("{dir}{level}:{s}..{e}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Every glyph of a one-line shape as `glyph_id/advance`, in visual
/// order; `0/9.7` is a tofu.
fn glyph_detail(shape: &ShapeReport) -> String {
    shape
        .glyphs
        .iter()
        .map(|g| format!("{}/{:.1}", g.glyph_id, g.advance))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Where the wrapped lines start, as byte offsets in the paragraph.
fn line_starts(shape: &ShapeReport) -> String {
    let mut starts = vec![usize::MAX; shape.lines];
    for g in &shape.glyphs {
        starts[g.line] = starts[g.line].min(g.start);
    }
    starts
        .iter()
        .map(|s| {
            if *s == usize::MAX {
                "-".to_string()
            } else {
                s.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("|")
}

/// Faces the glyphs came from; a face outside the embedded set is marked
/// with `*`.
fn faces_used(shape: &ShapeReport) -> String {
    let mut out: Vec<String> = Vec::new();
    for g in &shape.glyphs {
        let name = if g.embedded {
            g.family.clone()
        } else {
            format!("*{}", g.family)
        };
        if !out.contains(&name) {
            out.push(name);
        }
    }
    out.join(", ")
}

/// One table row per phrase and family.
fn table(faces: Faces) -> String {
    let mut out = String::new();
    out.push_str(
        "script | family | chars | glyphs | notdef | width | lines@80 | starts | runs | faces\n",
    );
    for p in corpus() {
        for (label, family) in FAMILIES {
            let st = style(family);
            let one = shape_report(&p.text, &st, None, faces);
            let wrapped = shape_report(&p.text, &st, Some(WRAP_WIDTH), faces);
            out.push_str(&format!(
                "{} | {} | {} | {} | {} | {:.1} | {} | {} | {} | {}\n",
                p.script,
                label,
                p.text.chars().count(),
                one.glyphs.len(),
                one.notdef(),
                one.width,
                wrapped.lines,
                line_starts(&wrapped),
                visual_runs(&one),
                faces_used(&one),
            ));
        }
    }
    out
}

/// Kerning and ligatures show as the difference between a word's width and
/// the sum of its characters shaped alone: negative is kerning (or a
/// ligature narrower than its parts), zero is neither.
fn kerning_table(faces: Faces) -> String {
    let mut out = String::new();
    out.push_str("phrase | family | width | solo sum | delta | glyphs/chars\n");
    for p in corpus().iter().filter(|p| p.script.contains("ligatures")) {
        for (label, family) in FAMILIES {
            let st = style(family);
            let word = shape_report(&p.text, &st, None, faces);
            let solo: f32 = p
                .text
                .chars()
                .map(|c| shape_report(&c.to_string(), &st, None, faces).width)
                .sum();
            out.push_str(&format!(
                "{} | {} | {:.1} | {:.1} | {:+.1} | {}/{}\n",
                p.text,
                label,
                word.width,
                solo,
                word.width - solo,
                word.glyphs.len(),
                p.text.chars().count(),
            ));
        }
    }
    out
}

fn embedded_font_bytes() -> u64 {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().extension().is_some_and(|x| x == "ttf"))
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .sum()
        })
        .unwrap_or(0)
}

/// Glyph ids and advances of every phrase in the first family, the
/// detail behind the table's counts.
fn detail_table(faces: Faces) -> String {
    let (label, family) = FAMILIES[0];
    let st = style(family);
    let mut out = format!("glyph/advance per phrase, {label}\n");
    for p in corpus() {
        let one = shape_report(&p.text, &st, None, faces);
        out.push_str(&format!(
            "{} | {} | {}\n",
            p.script,
            p.text,
            glyph_detail(&one)
        ));
    }
    out
}

fn golden_text() -> String {
    format!(
        "text audit: shaping with the embedded faces only ({} bytes of fonts), \
         {}px, wrap {}px\n\n{}\nkerning and ligatures\n{}\n{}",
        embedded_font_bytes(),
        FONT_SIZE,
        WRAP_WIDTH,
        table(Faces::Embedded),
        kerning_table(Faces::Embedded),
        detail_table(Faces::Embedded),
    )
}

/// The embedded-only table is deterministic (the faces are in the repo, the
/// shaper is pinned by Cargo.lock), so it is a golden.
#[test]
fn shaping_matches_the_golden() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/text_audit.txt");
    let actual = golden_text();
    if std::env::var_os("TEXT_AUDIT_UPDATE").is_some() {
        fs::write(&path, &actual).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    }
    let expected = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        expected == actual,
        "the text audit moved; run with TEXT_AUDIT_UPDATE=1 if the change was \
         meant.\n--- expected ({})\n{expected}\n--- actual\n{actual}",
        path.display()
    );
}

/// What this machine's desktop path resolves, for the phase 1 report.
/// Machine-dependent (system fonts), so printed, never asserted.
#[test]
fn desktop_shaping_is_reported() {
    let report = format!(
        "text audit: shaping with the embedded faces over this machine's system \
         fonts\n(a face marked * is not embedded: the desktop fallback filled it)\n\n\
         {}\nkerning and ligatures\n{}\n{}",
        table(Faces::Desktop),
        kerning_table(Faces::Desktop),
        detail_table(Faces::Desktop),
    );
    print!("\n{report}");
    let path = out_dir().join("desktop.txt");
    fs::write(&path, &report).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

// ---------------------------------------------------------------------------
// pixels
// ---------------------------------------------------------------------------

/// Collects the engine's log lines so the test can count the fallback
/// warnings the render path emits.
struct Recorder(Mutex<Vec<String>>);

impl log::Log for Recorder {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::Level::Debug
    }
    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!("{} {}: {}", record.level(), record.target(), record.args());
        if let Ok(mut lines) = self.0.lock() {
            lines.push(line);
        }
    }
    fn flush(&self) {}
}

fn recorder() -> &'static Recorder {
    static RECORDER: OnceLock<&'static Recorder> = OnceLock::new();
    RECORDER.get_or_init(|| {
        let r: &'static Recorder = Box::leak(Box::new(Recorder(Mutex::new(Vec::new()))));
        if log::set_logger(r).is_ok() {
            log::set_max_level(log::LevelFilter::Debug);
        }
        r
    })
}

fn recorded(contains: &str) -> Vec<String> {
    recorder()
        .0
        .lock()
        .map(|lines| {
            lines
                .iter()
                .filter(|l| l.contains(contains))
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

fn out_dir() -> PathBuf {
    let dir = std::env::var_os("TEXT_AUDIT_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("plev-text-audit"));
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Ink in the band of phrase `i`: the inked pixel count and the ink's
/// `(left, right)` edges in logical px from the specimen origin.
fn band_ink(img: &Rendered, i: usize) -> (usize, Option<(f32, f32)>) {
    let y = (i as f32 * ROW * SCALE) as u32;
    let h = (ROW * SCALE) as u32;
    let band = Rendered {
        width: img.width,
        height: h,
        rgba: img.band(y, h).to_vec(),
    };
    let edges = img.ink_extent(y, h).map(|(first, last)| {
        (
            first as f32 / SCALE - ORIGIN_X,
            (last + 1) as f32 / SCALE - ORIGIN_X,
        )
    });
    (band.ink(), edges)
}

/// One sheet per family: every phrase on its own row, rendered through the
/// frame path. Ink is compared with the width the measurer promised for the
/// same style; the sheet is written as PNG; the render is repeated to show
/// it is deterministic; the once-per-face fallback warnings are counted.
#[test]
fn pixels_match_the_measured_width() {
    let _ = recorder();
    let phrases = corpus();
    let height = (phrases.len() as f32 * ROW + ROW) as u32;
    let mut report = String::new();
    report.push_str("script | family | text | measured | ink px | ink left..right | verdict\n");

    for (label, family) in FAMILIES {
        let st = style(family);
        let scene: Vec<Specimen> = phrases
            .iter()
            .enumerate()
            .map(|(i, p)| {
                Specimen::new(p.text.clone(), st.clone(), ORIGIN_X, 10.0 + i as f32 * ROW)
            })
            .collect();
        let Some(img) = render(&scene, SHEET_W, height, SCALE) else {
            eprintln!("no GPU adapter; skipping");
            return;
        };
        let Some(again) = render(&scene, SHEET_W, height, SCALE) else {
            return;
        };
        let png = img.write_png(out_dir().join(format!("{}.png", label.replace(' ', "-"))));
        assert!(png.is_ok(), "cannot write the {label} sheet: {png:?}");
        assert_eq!(
            img.rgba, again.rgba,
            "{label}: two renders of the same corpus differ"
        );

        for (i, p) in phrases.iter().enumerate() {
            let measured = TextMeasurer::measure_styled(&p.text, &st, None).0;
            let (ink, edges) = band_ink(&img, i);
            let verdict = match edges {
                None if measured > 0.0 => "measured width without ink",
                None => "empty",
                Some((left, _)) if left < -1.0 => "ink before the origin",
                Some((_, right)) if right > measured + 1.0 => "ink past the measured width",
                Some((_, right)) if right < measured - FONT_SIZE => "ink well short of the width",
                Some(_) => "ok",
            };
            report.push_str(&format!(
                "{} | {} | {} | {:.1} | {} | {} | {}\n",
                p.script,
                label,
                p.text,
                measured,
                ink,
                edges.map_or("-".to_string(), |(l, r)| format!("{l:.1}..{r:.1}")),
                verdict
            ));
            assert!(
                edges.is_none_or(|(left, right)| left >= -1.0 && right <= measured + 1.0),
                "{label} {}: ink spans {edges:?} logical px from the origin but the \
                 measurer promised {measured:.1}px (layout sizes shapes around this \
                 text too small)",
                p.script
            );
        }
    }

    let warnings = recorded("text fallback:");
    report.push_str(&format!(
        "\nfallback warnings from the render path ({}):\n",
        warnings.len()
    ));
    for w in &warnings {
        report.push_str(w);
        report.push('\n');
    }
    let cosmic = recorded("Failed to find");
    report.push_str(&format!(
        "\ncosmic-text fallback notes ({}, debug level):\n",
        cosmic.len()
    ));
    for c in &cosmic {
        report.push_str(c);
        report.push('\n');
    }
    print!("\n{report}");
    let path = out_dir().join("pixels.txt");
    fs::write(&path, &report).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    println!("sheets and report in {}", out_dir().display());
    for p in &phrases {
        println!("{}: {}", p.script, p.covers);
    }
}
