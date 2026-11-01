//! The round of the checkpoint b (task-69): the review open beside the
//! fixed screen at the default window (1200x800), by the rules, and in a
//! narrower column. Every node of the surface stays inside the surface's
//! card, and no text is narrower than its longest word: the row of a cut
//! wraps instead of shrinking its texts to a glyph and pushing its
//! buttons past the edge. And the Claude's review of the first round, as
//! a fixture of IR shown by the page's staging entries (the regression
//! harness's way in, with frames made here and no worker job): the same
//! structure the Claude proposed, its sliders now with their label and
//! value, its durations with two decimals at most, its rows inside.

use crate::domain::silences::{PROBE_STDERR, parse};
use accord::Snapshot;
use engine::drawing::compositor::Compositor;
use engine::drawing::gpu::image::{ImageHandle, load_image_rgba};
use engine::drawing::text::TextMeasurer;
use engine::styling::theme::Theme;
use engine::styling::theme::TypeRole;
use engine::widgets::prelude::Rect;

use super::super::super::fixed_page::worker::Worker;
use super::super::{TURN, frame, images, narrow, setup, sid};
use crate::views::ShowcaseView;

/// The Claude's review of the checkpoint's first round, as IR.
const CLAUDE: &str = "../../rust/plevir/tests/fixtures/silences-review-claude.ir.json";

/// The intent of the checkpoint's rules round.
const RULES: &str = "the list with the video and the counts";

/// The gallery at the checkpoint's window on the Silences page, over the
/// fake worker, the review proposed by the rules for `intent` and every
/// frame and transition settled: the view and its last frame.
fn gallery(intent: &str) -> (ShowcaseView, Compositor) {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    // the window the fixture saw: the sidebar pinned beside the page
    view.pin_sidebar(true);
    view.jump_to_page("silences");
    view.panes
        .silences
        .set_worker(Worker::fake(parse(PROBE_STDERR), false));
    view.panes
        .silences
        .generated
        .proposing
        .intent
        .set_text(intent);
    let panes = &mut view.panes;
    panes.silences.propose(&mut panes.todo).expect("proposed");
    let mut c = Compositor::new();
    view.render(&mut c);
    let mut steps = 0;
    loop {
        let moving = view.tick(TURN);
        view.render(&mut c);
        if !moving && !view.panes.silences.worker.busy() {
            break;
        }
        steps += 1;
        assert!(steps < 600, "the page never settles");
    }
    (view, c)
}

fn rect(c: &Compositor, id: &str) -> Rect {
    let [x, y, w, h] = c
        .binding(&sid(id))
        .unwrap_or_else(|| panic!("{id} is bound"))
        .rect;
    Rect::new(x, y, w, h)
}

/// Every node the surface `root` holds is inside it, left and right.
fn inside(c: &Compositor, root: &str) {
    let card = rect(c, root);
    let prefix = root.split('.').next().unwrap_or(root);
    for (id, b) in c.bindings() {
        if !id.as_str().starts_with(prefix) {
            continue;
        }
        let [x, _, w, _] = b.rect;
        assert!(
            x >= card.x - 0.5 && x + w <= card.x + card.w + 0.5,
            "{id} at {:?} passes the card {card:?}",
            b.rect
        );
    }
}

/// Each text of `ids` at least as wide as its longest word, as the page
/// projects what it shows.
fn whole_words(c: &Compositor, snap: &Snapshot, theme: &Theme, ids: &[String]) {
    let style = theme.typography.role(TypeRole::BodyLg);
    for id in ids {
        let shown = &snap.get(&sid(id)).expect("projected").label;
        let word = shown
            .split_whitespace()
            .map(|w| TextMeasurer::measure_styled(w, &style, None).0)
            .fold(0.0f32, f32::max);
        let r = rect(c, id);
        assert!(r.w + 1.0 >= word, "{id} {shown:?}: {r:?} under {word}");
    }
}

/// The texts of a cut the rules show, by their state's key: its range
/// (its decision is a badge and its numbers stats, each the size its
/// widget measures, task-70 phase 2).
fn rules_texts(cut: u32) -> Vec<String> {
    ["range"]
        .iter()
        .map(|k| format!("proposed.silences.cut[{cut}].{k}"))
        .collect()
}

/// The acceptance of the layout: the rules' review at 1200x800 (the
/// surface in the right half, a card 424 wide): every node inside the
/// card, the buttons of a cut included, and every text of a cut whole.
#[test]
fn the_rules_review_fits_its_column_at_the_default_window() {
    let (view, c) = gallery(RULES);
    let card = rect(&c, "proposed.card");
    assert_eq!((card.x, card.w), (736.0, 432.0), "the right half");
    inside(&c, "proposed.card");
    let snap = view.semantics().expect("the page projects");
    for cut in [1, 2] {
        whole_words(&c, &snap, &view.theme, &rules_texts(cut));
        let apply = rect(&c, &format!("proposed.silences.cut[{cut}].apply_cut"));
        let frame = rect(&c, &format!("proposed.silences.cut[{cut}].frame"));
        assert!(apply.y > frame.y, "the buttons went down a line");
    }
}

/// The same review in a narrower column: the page 358 wide, the surface
/// under the fixed screen at the whole width.
#[test]
fn the_rules_review_fits_a_narrower_column() {
    let (mut todo, mut page, theme) = setup(false);
    page.generated.proposing.intent.set_text(RULES);
    page.propose(&mut todo).expect("proposed");
    let c = frame(&mut page, narrow(), &theme);
    let card = rect(&c, "proposed.card");
    assert!(card.w <= narrow().w, "{card:?}");
    inside(&c, "proposed.card");
    let snap = page.projection().expect("the page projects");
    for cut in [1, 2] {
        whole_words(&c, &snap, &theme, &rules_texts(cut));
    }
}

/// A frame made from its instant, a wash whose grey follows it: what the
/// staging entry is given instead of ffmpeg.
fn wash(ms: u64) -> Result<ImageHandle, String> {
    let (w, h) = (32u32, 18u32);
    let v = u8::try_from(40 + (ms / 100) % 200).unwrap_or(120);
    let pixels = std::iter::repeat_n([v, v, v, 255], (w * h) as usize)
        .flatten()
        .collect();
    load_image_rgba(w, h, pixels).map_err(|e| e.to_string())
}

/// The gallery at 1200x800 with the Claude's review shown by the staging
/// entries, as the regression harness stages it: the frames first, then
/// the fixture; the page's worker (in the crate's tests one that looks
/// for nothing) is never asked.
fn claude_review() -> (ShowcaseView, Compositor) {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    // the window the fixture saw: the sidebar pinned beside the page
    view.pin_sidebar(true);
    view.jump_to_page("silences");
    let filled = view.stage_silences_frames(&mut wash);
    assert_eq!(filled, [0, 4017, 9009], "the instant shown and each cut's");
    let json = super::fixture(CLAUDE);
    let id = view.stage_silences_ir(&json).expect("the fixture verifies");
    assert_eq!(id, sid("review"));
    assert!(
        view.stage_silences_frames(&mut wash).is_empty(),
        "all there"
    );
    let mut c = Compositor::new();
    view.render(&mut c);
    let mut steps = 0;
    while view.tick(TURN) {
        view.render(&mut c);
        steps += 1;
        assert!(steps < 600, "the page never settles");
    }
    view.render(&mut c);
    assert!(!view.panes.silences.worker.busy(), "no job for ffmpeg");
    (view, c)
}

/// The fixture is the structure the Claude proposed in the first round
/// (the ids, roles, labels and state of `review*` in the round's
/// observation, in order) and, above the sliders, where it drew it: the
/// geometry the observation recorded at 1200x800, to the pixel.
#[test]
fn the_claudes_review_fixture_is_the_first_rounds_structure() {
    let (view, c) = claude_review();
    let snap = view.semantics().expect("the page projects");
    let review: Vec<(String, &str, String)> = snap
        .nodes()
        .values()
        .filter(|n| n.id.as_str().starts_with("review"))
        .map(|n| (n.id.to_string(), n.role.as_str(), n.label.clone()))
        .collect();
    let ids: Vec<&str> = review.iter().map(|(id, _, _)| id.as_str()).collect();
    assert_eq!(
        &ids[..21],
        [
            "review",
            "review.root",
            "review.title",
            "review.video_card",
            "review.video_file",
            "review.video_duration",
            "review.video_original",
            "review.video_result",
            "review.controls",
            "review.threshold",
            "review.min_duration",
            "review.detect",
            "review.counts_card",
            "review.status_message",
            "review.count_total",
            "review.count_applied",
            "review.count_rejected",
            "review.count_pending",
            "review.count_cut_secs",
            "review.apply_all",
            "review.cuts_list",
        ]
    );
    let cut: Vec<&str> = ids[21..29].to_vec();
    assert_eq!(
        cut,
        [
            "review.cut_item[1]",
            "review.cut_range[1]",
            "review.cut_decision[1]",
            "review.cut_duration[1]",
            "review.cut_frame[1]",
            "review.cut_preview[1]",
            "review.cut_apply[1]",
            "review.cut_reject[1]",
        ]
    );
    let label = |id: &str| {
        review
            .iter()
            .find(|(i, _, _)| i == id)
            .map(|(_, role, label)| (*role, label.as_str()))
    };
    assert_eq!(label("review"), Some(("region", "Review Silence Cuts")));
    assert_eq!(
        label("review.threshold"),
        Some(("slider", "threshold (dB)"))
    );
    assert_eq!(label("review.cut_preview[1]"), Some(("button", "preview")));
    assert_eq!(
        label("review.cut_duration[1]"),
        Some(("label", "duration (s)"))
    );
    // the observation's rects of the round, above the sliders (the
    // sliders are taller now: their label and value), as the gallery
    // draws them today: the page's head the core kit's Layout title, as
    // tall as its blurb wraps, and the content lg below it (task-104d), and the duration the core kit's stat (task-104g,
    // the medium readout, 32 + 4 + 16 px tall, where the round saw 58 by
    // 43), which grows its card by 9; the page past the kit's sidebar,
    // `w-60` (task-104h), 8 px wider than the old 248, the half 4 more;
    // the text without a style in the scale's body role, 12/16 in the
    // core (task-104f), so the title and the file name are one 16 line;
    // the page's gutter fluid with the page's width (32 here, the half 8
    // wider) and no lg under a head that bleeds, the head itself fluid
    // (task-104j): a pixel lower
    for (id, [x, y, w, h]) in [
        ("review.root", [736.0, 222.0, 432.0, f32::NAN]),
        ("review.title", [736.0, 222.0, 432.0, 16.0]),
        ("review.video_card", [736.0, 250.0, 432.0, 372.0]),
        ("review.video_file", [736.0, 250.0, 432.0, 16.0]),
        ("review.video_duration", [736.0, 274.0, 63.0, 52.0]),
        ("review.video_original", [736.0, 334.0, 432.0, 140.0]),
        ("review.video_result", [736.0, 482.0, 432.0, 140.0]),
    ] {
        let r = rect(&c, id);
        let near = |a: f32, b: f32| b.is_nan() || (a - b).abs() < 0.5;
        assert!(
            near(r.x, x) && near(r.y, y) && near(r.w, w) && near(r.h, h),
            "{id}: {r:?}"
        );
    }
}

/// The acceptance of the three defects on the Claude's review: every
/// node inside its column, the cuts' rows included; each slider with its
/// label and its value; each duration as "2", never "1.996916"; the
/// frames drawn, the fixed screen's three and the surface's four.
#[test]
fn the_claudes_review_is_usable_at_the_default_window() {
    let (view, c) = claude_review();
    inside(&c, "review.root");
    let snap = view.semantics().expect("the page projects");
    let ids: Vec<String> = [
        "review.video_file",
        "review.cut_range[1]",
        "review.cut_range[2]",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect();
    whole_words(&c, &snap, &view.theme, &ids);
    let drawn: Vec<(String, f32, f32)> = c
        .layer(engine::drawing::compositor::LayerId::DEFAULT)
        .map(|l| {
            l.nodes()
                .iter()
                .filter_map(|n| match n {
                    engine::drawing::compositor::SceneNode::Text { key, x, y, .. } => {
                        Some((key.text.clone(), *x, *y))
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    let at = |s: &str, inside: Rect| {
        drawn
            .iter()
            .any(|(t, x, y)| t == s && inside.contains(*x + 0.5, *y + 0.5))
    };
    for (slider, name, value) in [
        ("review.threshold", "threshold (dB)", "-30"),
        ("review.min_duration", "minimum silence (s)", "0.5"),
    ] {
        let track = rect(&c, slider);
        let band = Rect::new(track.x, track.y - 30.0, track.w, 30.0);
        assert!(at(name, band), "{slider}: {name} above its track");
        assert!(at(value, band), "{slider}: {value} above its track");
    }
    for cut in [1, 2] {
        let stat = rect(&c, &format!("review.cut_duration[{cut}]"));
        assert!(at("2", stat), "the duration of cut {cut}");
    }
    assert!(
        !drawn.iter().any(|(t, _, _)| t.contains("1.996916")),
        "a raw duration"
    );
    assert_eq!(images(&c), 3 + 4, "every frame drawn");
}
