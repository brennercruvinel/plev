//! The round of the checkpoint b (task-69): the review open beside the
//! fixed screen at the default window (1200x800), by the rules, and in a
//! narrower column. Every node of the surface stays inside the surface's
//! card, and no text is narrower than its longest word: the row of a cut
//! wraps instead of shrinking its texts to a glyph and pushing its
//! buttons past the edge.

use comps::prelude::Rect;
use engine::compositor::Compositor;
use engine::text::TextMeasurer;
use engine::theme::Theme;
use sem::Snapshot;
use showcase::model::silences::{PROBE_STDERR, parse};

use super::super::super::worker::Worker;
use super::super::{TURN, frame, narrow, setup, sid};
use crate::view::ShowcaseView;

/// The intent of the checkpoint's rules round.
const RULES: &str = "the list with the video and the counts";

/// The gallery at the checkpoint's window on the Silences page, over the
/// fake worker, the review proposed by the rules for `intent` and every
/// frame and transition settled: the view and its last frame.
fn gallery(intent: &str) -> (ShowcaseView, Compositor) {
    let mut view = ShowcaseView::new(1200.0, 800.0);
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
    let style = theme.typography.body();
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

/// The texts of a cut the rules show, by their state's key.
fn rules_texts(cut: u32) -> Vec<String> {
    ["decision", "duration", "end", "range", "start"]
        .iter()
        .map(|k| format!("proposed.silences.cut[{cut}].{k}"))
        .collect()
}

/// The acceptance of the layout: the rules' review at 1200x800 (the
/// surface in the right half, a card 420 wide): every node inside the
/// card, the buttons of a cut included, and every text of a cut whole.
#[test]
fn the_rules_review_fits_its_column_at_the_default_window() {
    let (view, c) = gallery(RULES);
    let card = rect(&c, "proposed.card");
    assert_eq!((card.x, card.w), (740.0, 420.0), "the right half");
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
