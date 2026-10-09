//! The Claude's review adapted (task-70 phase 1): over the fixture of the
//! first round, the one that put the list of the cuts last, a `Reorder`
//! puts the cuts first and a `Hide` takes out the detection the fixed
//! screen already offers; the page shows it at the default window, the
//! host keeps both changes with who asked, and undoing them gives the
//! Claude's review back, byte for byte. The rules' review cannot swap its
//! list for a chart: a list repeats its cuts' template, and no state of
//! the app is a list of durations.

use accord::{Actor, SemId};
use engine::drawing::compositor::Compositor;
use engine::drawing::gpu::image::{ImageHandle, load_image_rgba};
use engine::widgets::prelude::Rect;
use plevir::{ChartKind, Component, Op};

use super::super::TURN;
use super::super::sid;
use crate::views::ShowcaseView;

/// The Claude's review of the checkpoint's first round, as IR.
const CLAUDE: &str = "../../rust/plevir/tests/fixtures/silences-review-claude.ir.json";

fn wash(ms: u64) -> Result<ImageHandle, String> {
    let v = u8::try_from(40 + (ms / 100) % 200).unwrap_or(120);
    let pixels = std::iter::repeat_n([v, v, v, 255], 32 * 18)
        .flatten()
        .collect();
    load_image_rgba(32, 18, pixels).map_err(|e| e.to_string())
}

/// Every frame drawn until the page rests.
fn settle(view: &mut ShowcaseView) -> Compositor {
    let mut c = Compositor::new();
    view.render(&mut c);
    let mut steps = 0;
    while view.tick(TURN) {
        view.render(&mut c);
        steps += 1;
        assert!(steps < 600, "the page never settles");
    }
    view.render(&mut c);
    c
}

/// The gallery at `w` by 800 with the Claude's review shown as the
/// regression harness shows it.
fn claude_review(w: f32) -> ShowcaseView {
    let mut view = ShowcaseView::new(w, 800.0);
    view.jump_to_page("silences");
    view.stage_silences_frames(&mut wash);
    let json = super::fixture(CLAUDE);
    view.stage_silences_ir(&json).expect("the fixture verifies");
    view
}

fn rect(c: &Compositor, id: &str) -> Rect {
    let [x, y, w, h] = c
        .binding(&sid(id))
        .unwrap_or_else(|| panic!("{id} is bound"))
        .rect;
    Rect::new(x, y, w, h)
}

fn regress() -> Actor {
    Actor::Test {
        name: "regress".to_owned(),
    }
}

/// The two operations of the demo: the cuts after the title, then the
/// detection out.
fn demo() -> [Op; 2] {
    [
        Op::Reorder {
            parent: sid("review.root"),
            ids: vec![sid("review.title"), sid("review.cuts_list")],
        },
        Op::Hide {
            id: sid("review.controls"),
        },
    ]
}

#[test]
fn the_claudes_review_takes_the_cuts_first_and_drops_the_repeated_detection() {
    let mut view = claude_review(1280.0);
    let c = settle(&mut view);
    // before: the cuts under everything else, below the fold
    let first = rect(&c, "review.cut_item[1]");
    assert!(
        first.y > 800.0,
        "the cuts start below the window: {first:?}"
    );
    let had = view.semantics().expect("the page projects");
    assert!(had.get(&sid("review.threshold")).is_some());

    for op in demo() {
        let line = view
            .stage_silences_op(op.clone(), regress())
            .unwrap_or_else(|e| panic!("{op}: {e}"));
        assert_eq!(line, op.to_string());
    }
    let c = settle(&mut view);
    let snap = view.semantics().expect("the page projects");
    // the cuts right after the title, both inside the window
    let title = rect(&c, "review.title");
    for cut in [1, 2] {
        let item = rect(&c, &format!("review.cut_item[{cut}]"));
        assert!(
            item.y > title.y && item.y + item.h <= 800.0,
            "cut {cut}: {item:?}"
        );
        let video = rect(&c, "review.video_card");
        assert!(item.y < video.y, "cut {cut} above the video");
    }
    // the detection the fixed screen offers is gone from the surface,
    // not from the page
    for gone in [
        "review.controls",
        "review.threshold",
        "review.min_duration",
        "review.detect",
    ] {
        assert!(snap.get(&sid(gone)).is_none(), "{gone} is hidden");
    }
    for fixed in [
        "silences.threshold",
        "silences.min_duration",
        "silences.detect",
    ] {
        assert!(
            snap.get(&sid(fixed)).is_some(),
            "{fixed} is the fixed screen's"
        );
    }
    // the history: both changes, by the harness, with their way back
    let review = sid("review");
    let host = &view.panes.silences.generated.host;
    let changes: Vec<(String, &Actor)> = host
        .changes(&review)
        .iter()
        .map(|c| (c.op.to_string(), &c.by))
        .collect();
    assert_eq!(
        changes,
        [
            (
                "reorder review.root: review.title, review.cuts_list".to_owned(),
                &regress()
            ),
            ("hide review.controls".to_owned(), &regress()),
        ]
    );
    let aside: Vec<&SemId> = host
        .adapted(&review)
        .map(|a| a.aside().map(|kept| &kept.node.sem).collect())
        .unwrap_or_default();
    assert_eq!(aside, [&sid("review.controls")]);
}

/// Both undone, the Claude's review is back as the fixture wrote it.
#[test]
fn undoing_the_demo_gives_the_claudes_review_back_byte_for_byte() {
    let mut view = claude_review(1280.0);
    let review = sid("review");
    let written = |view: &ShowcaseView| {
        let host = &view.panes.silences.generated.host;
        let surface = host.get(&review).expect("open");
        serde_json::to_string(surface.verified().ir()).expect("an IR writes")
    };
    let at_start = written(&view);
    for op in demo() {
        view.stage_silences_op(op, regress()).expect("applies");
    }
    assert_ne!(written(&view), at_start);
    let page = &mut view.panes.silences;
    let app = page.snapshot().expect("the app projects");
    for _ in 0..2 {
        page.generated.host.undo(&review, &app).expect("undoes");
    }
    assert_eq!(written(&view), at_start);
    let _ = settle(&mut view);
    let snap = view.semantics().expect("the page projects");
    assert!(snap.get(&sid("review.threshold")).is_some(), "shown again");
}

/// The chart of the durations does not fit the Silences: the list of the
/// cuts repeats a template (its items are the cuts' ids), so a swap would
/// drop every row's id, and the one list the app declares is those ids.
#[test]
fn the_list_of_the_cuts_does_not_swap_for_a_chart() {
    let mut view = claude_review(1280.0);
    let why = view
        .stage_silences_op(
            Op::SwapView {
                id: sid("review.cuts_list"),
                component: Component::Chart(ChartKind::Bar),
            },
            regress(),
        )
        .expect_err("refused");
    assert!(why.contains("repeats a template"), "{why}");
    let app = view.panes.silences.snapshot().expect("the app projects");
    let lists: Vec<String> = app
        .nodes()
        .values()
        .flat_map(|n| {
            n.state
                .iter()
                .filter(|(_, v)| v.kind() == accord::ValueKind::List)
                .map(move |(k, _)| format!("{}:{k}", n.id))
        })
        .collect();
    assert_eq!(lists, ["silences:cuts"], "the only list: the cuts' ids");
}
