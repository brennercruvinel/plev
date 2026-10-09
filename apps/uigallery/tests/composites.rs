//! The composite detector (task-70 phase 3) over real sessions of the
//! gallery: live, as a sink on its one dispatcher, and recorded, from the
//! `.monster` the recorder of task-63 writes, the same stage events read
//! back in the order of the file. One detector, fed the same way, gives
//! the same answer: here an agent looks at each cut of the Silences page
//! and applies it, twice, through `ShowcaseView::act` (the port's path),
//! with the frames staged so no ffmpeg runs. And the recorded demo of
//! task-69 (`fixtures/demo/silences-demo.monster`), where the agent asked
//! for a surface twice (choose the proposer, write the intent, propose):
//! nothing is found there, since two of those steps carry a parameter.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use std::sync::{Arc, Mutex};

use engine::drawing::gpu::image::{ImageHandle, load_image_rgba};
use offers::compose::{Detector, Found, Listener, Target};
use record::{Kind, Limits, Recorder, Session};
use accord::{Actor, Request, SemId};
use uigallery::views::ShowcaseView;

fn agent() -> Actor {
    Actor::Agent {
        name: "reviewer".to_owned(),
    }
}

/// A frame of the test video, flat: the page shows it, nothing decodes.
fn flat(_ms: u64) -> Result<ImageHandle, String> {
    load_image_rgba(4, 4, vec![128; 64]).map_err(|e| e.to_string())
}

fn act(view: &mut ShowcaseView, target: &str, action: &str) {
    let seen = view.semantics().expect("the page projects");
    let req = Request::observe(&seen, agent(), target, action, None).expect("observed");
    let receipt = view.act(req).expect("taken");
    assert_eq!(receipt.stage().map(accord::Stage::name), Some("completed"));
}

/// Every stage event of the session, in the order of the file, into a
/// fresh detector: the recorded half of the one path.
fn read_back(session: &Session, repeats: usize) -> Option<Found> {
    let mut detector = Detector::new(repeats);
    for event in session.events() {
        if let Kind::Stage {
            id, request, stage, ..
        } = &event.kind
        {
            detector.stage(*id, request, stage);
        }
    }
    detector.found()
}

#[test]
fn live_and_recorded_the_detector_finds_the_same_sequence() {
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.jump_to_page("Silences");
    view.stage_silences_frames(&mut flat);
    let recorder = Recorder::new(Limits::default());
    view.add_sink(Box::new(recorder.sink()));
    let live = Arc::new(Mutex::new(Detector::new(2)));
    view.add_sink(Box::new(Listener::new(live.clone())));

    act(&mut view, "silences.cut[1]", "preview_segment");
    act(&mut view, "silences.cut[1]", "apply_cut");
    assert_eq!(live.lock().unwrap().found(), None, "once is no repetition");
    act(&mut view, "silences.cut[2]", "preview_segment");
    act(&mut view, "silences.cut[2]", "apply_cut");

    let heard = live.lock().unwrap().found().expect("found live");
    assert_eq!(heard.actor, agent());
    assert_eq!(heard.repeats, 2);
    let steps: Vec<(&str, &Target)> = heard
        .shape
        .steps
        .iter()
        .map(|s| (s.action.as_str(), &s.target))
        .collect();
    let cut = Target::Item(SemId::parse("silences.cut").unwrap());
    assert_eq!(steps, [("preview_segment", &cut), ("apply_cut", &cut)]);

    let dir = tempfile::tempdir().expect("a temporary directory");
    let path = dir.path().join("composites.monster");
    recorder.persist(&path).expect("the session is written");
    let session = Session::open(&path).expect("the session opens");
    assert_eq!(read_back(&session, 2), Some(heard), "one answer");
}

#[test]
fn the_recorded_demo_repeats_a_proposal_with_parameters_and_nothing_is_found() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/demo/silences-demo.monster"
    );
    let session = Session::open(path).expect("the demo opens");
    // the agent chose the proposer, wrote an intent and proposed, twice
    let asked: Vec<(String, bool)> = session
        .receipts(u64::MAX)
        .iter()
        .map(|r| {
            let req = &r.receipt.request;
            (
                format!("{} {}", req.target, req.action),
                req.param.is_some(),
            )
        })
        .collect();
    let row = [
        ("silences_surface.proposer select".to_owned(), true),
        ("silences_surface.intent set_value".to_owned(), true),
        ("silences_surface.propose propose".to_owned(), false),
    ];
    assert_eq!(asked[1..4], row);
    assert_eq!(asked[6..9], row);
    assert_eq!(read_back(&session, 2), None);
}
