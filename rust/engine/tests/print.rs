//! Printing as the frame on paper (task-76 item 9), without a printer.
//!
//! The probe first: a text run drawn through the window's frame path
//! without a window (`engine::headless`) at the screen's scale (2.0) and
//! at a paper scale (300 dpi over 72 points per inch, 4.1667), timed
//! piece by piece: the device, the draw and the readback, the png. The
//! glyphs are rasterized at the physical scale (the atlas of task-66), so
//! the ink of the same run is twice as wide at twice the scale, not a
//! blown-up bitmap.
//!
//! Then the document from that render (`PrintDocument::from_headless`:
//! the frame's size, the dpi, a png that decodes to the frame) and the
//! CUPS printer over two fake programs (`tests/fake-lp`, `tests/fake-lpstat`
//! and `tests/fake-lpstat-none`, shell scripts that spool nothing): the
//! destinations parsed with the default marked, the `lp` command as built
//! (the destination, the title, fit to page, the copies, the media), a
//! job's place read from its destination's queue, this machine's "no
//! destination" as an empty list, `lp`'s refusals as errors, and the
//! deadline killing a program that does not return. No real `lp` or
//! `lpstat` runs here. The GPU tests skip without an adapter.

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::gpu::RenderConfig;
use engine::headless::{HeadlessError, Renderer, Viewport};
use engine::platform::print::{
    Destination, JobId, JobStatus, PAPER_DPI, PrintDocument, PrintError, PrintOptions, Printer,
    fit_scale, paper_scale,
};
use engine::text::TextStyle;
use engine::text::probe::Rendered;

/// One line of the Todo page, at the body size.
const TEXT: &str = "Absorb the plev demos into the showcase";
/// The logical viewport of the specimen: wide enough for the run at the
/// body size, one line tall.
const LOGICAL: (f32, f32) = (600.0, 80.0);
/// The scale of a Retina screen, and of paper: 300 dpi in points.
const SCREEN: f32 = 2.0;
const PAPER: f32 = 300.0 / 72.0;
/// Background, linear (the probe's own: the sRGB target encodes it to
/// about 108, well under the ink threshold).
const CLEAR: [f32; 4] = [0.15, 0.15, 0.15, 1.0];

fn style() -> TextStyle {
    TextStyle::new(15.0).with_line_height(21.0).with_weight(400)
}

/// The run on a layer of its own, white on the clear.
fn scene() -> Compositor {
    let mut c = Compositor::new();
    let layer = c.create_layer(1);
    c.begin_frame();
    c.push_to_layer(
        layer,
        SceneNode::Text {
            key: TextNodeKey::from_style(TEXT, &style(), None),
            x: 20.0,
            y: 20.0,
            color: [1.0, 1.0, 1.0, 1.0],
        },
    );
    c
}

fn renderer(scale: f32) -> Option<Renderer> {
    let viewport = Viewport::new(LOGICAL.0, LOGICAL.1, scale);
    match Renderer::new(viewport, RenderConfig::default()) {
        Ok(r) => Some(r),
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter; skipping");
            None
        }
        Err(e) => panic!("{e}"),
    }
}

/// What one scale cost and gave.
struct Measured {
    physical: (u32, u32),
    device_ms: f64,
    draw_read_ms: f64,
    png_ms: f64,
    png_bytes: usize,
    /// The ink of the run in physical pixels, first to last column.
    ink: u32,
}

fn measure(scale: f32) -> Option<Measured> {
    let t = Instant::now();
    let mut renderer = renderer(scale)?;
    let device_ms = t.elapsed().as_secs_f64() * 1000.0;
    let mut c = scene();
    let t = Instant::now();
    let frame = renderer.render(&mut c, CLEAR).expect("the frame reads");
    let draw_read_ms = t.elapsed().as_secs_f64() * 1000.0;
    let t = Instant::now();
    let png = frame.encode_png().expect("the frame encodes");
    let png_ms = t.elapsed().as_secs_f64() * 1000.0;
    let rendered = Rendered {
        width: frame.width,
        height: frame.height,
        rgba: frame.rgba,
    };
    let (first, last) = rendered
        .ink_extent(0, rendered.height)
        .expect("the run left ink");
    Some(Measured {
        physical: (frame.width, frame.height),
        device_ms,
        draw_read_ms,
        png_ms,
        png_bytes: png.len(),
        ink: last + 1 - first,
    })
}

/// The same run at the screen's scale and at paper: the physical size
/// follows the scale, and so does the ink, within the rounding of glyph
/// edges. The timings are the probe's measurements (`--nocapture`).
#[test]
fn the_same_run_on_paper_is_as_many_times_wider_as_the_scale() {
    let Some(screen) = measure(SCREEN) else {
        return;
    };
    let Some(paper) = measure(PAPER) else {
        return;
    };
    for (name, m) in [("screen 2.0", &screen), ("paper 4.1667", &paper)] {
        eprintln!(
            "{name}: {}x{} px, device {:.1} ms, draw+read {:.1} ms, png {:.1} ms ({} bytes), ink {} px",
            m.physical.0, m.physical.1, m.device_ms, m.draw_read_ms, m.png_ms, m.png_bytes, m.ink
        );
    }
    assert_eq!(screen.physical, (1200, 160));
    assert_eq!(paper.physical, (2500, 333));
    let ratio = paper.ink as f32 / screen.ink as f32;
    let expected = PAPER / SCREEN;
    assert!(
        (ratio - expected).abs() < expected * 0.03,
        "the ink grew {ratio:.3}x for a scale {expected:.3}x: the glyphs are not rasterized at the physical scale"
    );
}

/// The document is the frame: its size in dots, the dpi it was rendered
/// for, a png that decodes back to the same pixels, the paper it covers.
#[test]
fn the_document_from_the_headless_render_is_the_frame_at_the_papers_dpi() {
    let scale = fit_scale(LOGICAL.0, LOGICAL.1, paper_scale(PAPER_DPI), 8192);
    assert!((scale - PAPER).abs() < 0.001, "{scale} fits as it is");
    let Some(mut renderer) = renderer(scale) else {
        return;
    };
    let mut c = scene();
    let doc = PrintDocument::from_headless(&mut renderer, &mut c, CLEAR, PAPER_DPI)
        .expect("the document renders");
    assert_eq!((doc.width, doc.height), renderer.viewport().physical());
    assert_eq!((doc.width, doc.height), (2500, 333));
    assert_eq!(doc.dpi, PAPER_DPI);
    assert!(doc.svg.is_none());
    let (w, h) = doc.paper_size();
    assert!((w - 8.333).abs() < 0.01, "{w} in wide");
    assert!((h - 1.11).abs() < 0.01, "{h} in tall");

    let decoded = image::load_from_memory(&doc.png).expect("a png").to_rgba8();
    assert_eq!((decoded.width(), decoded.height()), (doc.width, doc.height));
    let frame = renderer.read().expect("the frame reads again");
    assert!(
        decoded.as_raw() == &frame.rgba,
        "the png holds the frame's pixels"
    );
    let rendered = Rendered {
        width: doc.width,
        height: doc.height,
        rgba: decoded.into_raw(),
    };
    assert!(rendered.ink() > 1000, "the run is in the document");

    // the same frame, the svg beside it
    let svg = renderer.svg(&c, None).expect("the svg exports");
    let with_svg = doc.clone().with_svg(svg.svg.clone());
    assert!(
        with_svg
            .svg
            .as_deref()
            .is_some_and(|s| s.contains("<path d="))
    );
    assert_eq!(with_svg.png, doc.png);
}

// -- the CUPS printer over the fakes ------------------------------------------

#[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
mod cups {
    use super::*;
    use engine::platform::print::CupsPrinter;

    fn fake(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join(name)
    }

    /// A spool directory of this test alone (the fake `lp` writes beside
    /// the document), under the temp dir, named by the process and the
    /// test.
    fn spool(test: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("plev-print-tests")
            .join(format!("{}-{test}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn printer(lpstat: &str, test: &str) -> CupsPrinter {
        CupsPrinter::with(fake("fake-lp"), fake(lpstat))
            .spool_dir(spool(test))
            .timeout(Duration::from_secs(10))
    }

    fn page() -> PrintDocument {
        PrintDocument {
            png: b"\x89PNG\r\n\x1a\n".to_vec(),
            width: 5000,
            height: 3333,
            dpi: PAPER_DPI,
            svg: None,
        }
    }

    fn lp_args(spool: &std::path::Path) -> Vec<String> {
        std::fs::read_to_string(spool.join("lp.args"))
            .expect("the fake lp left its arguments")
            .lines()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn the_destinations_are_what_lpstat_lists_with_the_default_marked() {
        let mut printer = printer("fake-lpstat", "destinations");
        assert_eq!(
            printer.destinations().unwrap(),
            [
                Destination::new("Office", true),
                Destination::new("Home", false)
            ]
        );
    }

    #[test]
    fn this_machine_has_no_destination_and_that_is_an_empty_list() {
        let mut printer = printer("fake-lpstat-none", "none");
        assert_eq!(printer.destinations().unwrap(), []);
        // and lp, asked without one, says so
        let err = printer
            .print(&page(), &PrintOptions::default())
            .unwrap_err();
        assert_eq!(err, PrintError::NoDestination);
    }

    #[test]
    fn a_print_runs_lp_with_the_destination_the_title_and_the_options_and_reads_the_job() {
        let dir = spool("print");
        let mut printer = CupsPrinter::with(fake("fake-lp"), fake("fake-lpstat"))
            .spool_dir(&dir)
            .timeout(Duration::from_secs(10));
        let options = PrintOptions::default()
            .to("Office")
            .title("plev, Todo")
            .copies(2)
            .media("a4");
        let job = printer.print(&page(), &options).unwrap();
        assert_eq!(job, JobId("Office-7".to_owned()));
        let args = lp_args(&dir);
        assert_eq!(args.len(), 12, "{args:?}");
        assert_eq!(
            &args[..11],
            [
                "-d",
                "Office",
                "-n",
                "2",
                "-t",
                "plev, Todo",
                "-o",
                "fit-to-page",
                "-o",
                "media=a4",
                "--"
            ]
        );
        let file = PathBuf::from(&args[11]);
        assert_eq!(file.parent(), Some(dir.as_path()));
        assert!(
            file.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("plev-") && n.ends_with(".png")),
            "{}",
            file.display()
        );
        assert!(
            !file.exists(),
            "the document leaves the spool dir once lp took it"
        );

        // where the job is: the queue of its destination
        assert_eq!(printer.status(&job).unwrap(), JobStatus::Queued);
        assert_eq!(
            printer.status(&JobId("Home-7".to_owned())).unwrap(),
            JobStatus::Completed
        );
        assert_eq!(
            printer.status(&JobId("Office-8".to_owned())).unwrap(),
            JobStatus::Gone
        );
        assert!(matches!(
            printer.status(&JobId("Nowhere-1".to_owned())),
            Err(PrintError::Spooler(why)) if why.contains("Invalid destination")
        ));
        assert!(matches!(
            printer.status(&JobId("Office".to_owned())),
            Err(PrintError::Spooler(_))
        ));

        // a second print, without the extras, to the default
        let job = printer
            .print(&page(), &PrintOptions::default().to("Home"))
            .unwrap();
        assert_eq!(job.to_string(), "Home-7");
        let args = lp_args(&dir);
        assert_eq!(
            &args[..6],
            ["-d", "Home", "-t", "plev", "-o", "fit-to-page"]
        );
    }

    #[test]
    fn what_lp_refuses_is_refused_in_its_words() {
        let mut printer = printer("fake-lpstat", "refuse");
        let err = printer
            .print(
                &page(),
                &PrintOptions::default().to("Nowhere").title("refuse"),
            )
            .unwrap_err();
        assert_eq!(err, PrintError::UnknownDestination("Nowhere".to_owned()));
        let mut missing =
            CupsPrinter::with("/nonexistent/lp", fake("fake-lpstat")).spool_dir(spool("missing"));
        assert!(matches!(
            missing.print(&page(), &PrintOptions::default().to("Office")),
            Err(PrintError::Io(why)) if why.contains("could not start")
        ));
    }

    /// A program that does not return is killed at the deadline, by its
    /// pid: the sleep the fake became is gone right after. The deadline
    /// is two seconds (the fake sleeps thirty): beside the GPU tests of
    /// this binary a child takes its time to start.
    #[test]
    fn a_program_past_the_deadline_is_killed() {
        let dir = spool("slow");
        let mut printer = CupsPrinter::with(fake("fake-lp"), fake("fake-lpstat"))
            .spool_dir(&dir)
            .timeout(Duration::from_secs(2));
        let started = Instant::now();
        let err = printer
            .print(&page(), &PrintOptions::default().to("Office").title("slow"))
            .unwrap_err();
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "killed, not waited for"
        );
        assert_eq!(
            err,
            PrintError::Timeout {
                program: "fake-lp".to_owned(),
                secs: 2
            }
        );
        let pid = std::fs::read_to_string(dir.join("lp.pid"))
            .expect("the fake wrote its pid")
            .trim()
            .to_owned();
        let alive = Command::new("kill")
            .args(["-0", &pid])
            .status()
            .expect("kill runs")
            .success();
        assert!(!alive, "pid {pid} still runs");
        assert!(
            std::fs::read_dir(&dir)
                .expect("the spool dir")
                .flatten()
                .all(|e| e.path().extension().is_none_or(|x| x != "png")),
            "the document leaves the spool dir after a kill too"
        );
    }
}
