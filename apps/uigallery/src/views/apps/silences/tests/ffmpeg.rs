//! The real ffmpeg, ignored (a runner of the gate may not have it):
//! `cargo test -p showcase --lib silences::tests::ffmpeg -- --ignored
//! --nocapture`, with `SILENCES_OUT=<dir>` for the png and the projection
//! of the page after the detection. Without the program the test says so
//! and passes.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use engine::drawing::compositor::Compositor;
use accord::{Stage, Value};

use super::super::SilencesPage;
use super::super::fixed_page::worker::Worker;
use super::{TURN, dispatch_as_test, stage, value, wide};
use crate::views::apps::todo::TodoPage;

/// Wakes the page until `until` holds or `limit` passes.
fn wait(
    page: &mut SilencesPage,
    todo: &mut TodoPage,
    limit: Duration,
    until: impl Fn(&SilencesPage, &TodoPage) -> bool,
) {
    let deadline = Instant::now() + limit;
    while !until(page, todo) {
        assert!(
            Instant::now() < deadline,
            "ffmpeg did not answer in {limit:?}"
        );
        std::thread::sleep(Duration::from_millis(5));
        page.wake(TURN, todo);
    }
}

#[test]
#[ignore = "runs ffmpeg: cargo test -p showcase --lib silences::tests::ffmpeg -- --ignored --nocapture"]
fn the_real_ffmpeg_generates_the_video_detects_the_silences_and_decodes_a_frame() {
    use engine::drawing::gpu::RenderConfig;
    use engine::drawing::headless::{Renderer, Viewport};

    let program = match Worker::find_program() {
        Ok(p) => p,
        Err(why) => {
            eprintln!("skipping: {why}");
            return;
        }
    };
    println!("ffmpeg: {}", program.display());
    let mut todo = TodoPage::new();
    let mut page = SilencesPage::with(Worker::real());
    let video = page.worker.video().to_owned();
    println!(
        "the test video: {} (exists before: {})",
        video.display(),
        video.is_file()
    );

    // the detection: the request starts and ends when ffmpeg answers
    let started = Instant::now();
    let id = dispatch_as_test(
        &mut page,
        &mut todo,
        "silences.detect",
        "detect_silence",
        None,
    );
    assert!(matches!(stage(&todo, id), Stage::Executing));
    wait(&mut page, &mut todo, Duration::from_secs(90), |_, todo| {
        todo.receipt(id).unwrap().is_finished()
    });
    let detect_wall = started.elapsed();
    let Stage::Completed(outcome) = stage(&todo, id) else {
        panic!("the detection failed: {:?}", stage(&todo, id));
    };
    let took = match outcome.result.get("seconds") {
        Some(Value::Num(s)) => *s,
        other => panic!("seconds: {other:?}"),
    };
    println!(
        "detect_silence: {} cuts, ffmpeg took {:.0} ms (video generated in the same job when missing), the request ended after {:?}",
        page.model.cuts().len(),
        took * 1000.0,
        detect_wall
    );
    assert!(
        video.is_file(),
        "the video was generated at {}",
        video.display()
    );
    println!(
        "the video: {} bytes",
        std::fs::metadata(&video).unwrap().len()
    );
    assert_eq!(page.model.duration(), 12.0);
    let cuts = page.model.cuts();
    assert_eq!(cuts.len(), 2, "{cuts:?}");
    for (cut, (start, end)) in cuts.iter().zip([(3.018594, 5.01551), (8.010884, 10.0078)]) {
        let seg = cut.segment();
        println!("cut {}: {:.6} to {:.6}", cut.id(), seg.start, seg.end);
        assert!((seg.start - start).abs() < 0.1, "{seg:?}");
        assert!((seg.end - end).abs() < 0.1, "{seg:?}");
    }

    // the frames follow: the instant shown and the middle of each cut
    wait(&mut page, &mut todo, Duration::from_secs(60), |page, _| {
        !page.worker.busy()
    });
    let mut at: Vec<u64> = page.frames.keys().copied().collect();
    at.sort_unstable();
    println!("frames at {at:?} ms");
    assert_eq!(at.len(), 3);
    for (ms, frame) in &page.frames {
        let handle = frame
            .as_ref()
            .unwrap_or_else(|e| panic!("frame at {ms}: {e}"));
        assert_eq!((handle.width, handle.height), (320, 180), "frame at {ms}");
    }
    let frame_ms: Vec<u128> = page.frame_took.iter().map(|d| d.as_millis()).collect();
    println!("each frame took ffmpeg {frame_ms:?} ms");

    // a second detection, with the video there: the detection alone
    let id = dispatch_as_test(
        &mut page,
        &mut todo,
        "silences.detect",
        "detect_silence",
        None,
    );
    wait(&mut page, &mut todo, Duration::from_secs(90), |_, todo| {
        todo.receipt(id).unwrap().is_finished()
    });
    let Stage::Completed(outcome) = stage(&todo, id) else {
        panic!("the second detection failed: {:?}", stage(&todo, id));
    };
    if let Some(Value::Num(s)) = outcome.result.get("seconds") {
        println!(
            "detect_silence again, the video there: ffmpeg took {:.0} ms",
            s * 1000.0
        );
    }
    assert_eq!(page.model.cuts().len(), 2);
    wait(&mut page, &mut todo, Duration::from_secs(60), |page, _| {
        !page.worker.busy()
    });

    // a preview of a decoded cut completes at once and shows its frame
    let first = format!("silences.cut[{}]", page.model.cuts()[0].id());
    let id = dispatch_as_test(&mut page, &mut todo, &first, "preview_segment", None);
    assert!(matches!(stage(&todo, id), Stage::Completed(_)));
    let shown = value(&page, "silences.video:shown");
    println!("shown: {shown:?}");

    // drawn headless, and with a GPU its pixels
    let content = wide();
    let theme = engine::styling::theme::Theme::hoff_core();
    let mut view = crate::views::ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("silences");
    // the page of the gallery takes this page's state, so the frame is
    // the gallery's frame of it
    view.panes.silences = page;
    let mut c = Compositor::new();
    view.render(&mut c);
    let snap = view.semantics().expect("the page projects");
    let _ = (content, theme);
    let viewport = Viewport {
        width: 1200.0,
        height: 800.0,
        scale: 1.0,
    };
    let Ok(mut r) = Renderer::new(viewport, RenderConfig::default()) else {
        eprintln!("no GPU adapter: the scene and the projection only");
        return;
    };
    let clear = view.theme.colors.bg.to_linear_array();
    let frame = r.render(&mut c, clear).expect("the frame draws");
    if let Some(out) = std::env::var_os("SILENCES_OUT") {
        let path = PathBuf::from(out).join("silences-detected");
        frame
            .write_png(path.with_extension("png"))
            .expect("the png writes");
        let json = crate::shell::semantics_json(&snap, false).expect("the json");
        std::fs::write(path.with_extension("json"), json).expect("the json writes");
        println!("wrote {}.png and .json", path.display());
    }
}
