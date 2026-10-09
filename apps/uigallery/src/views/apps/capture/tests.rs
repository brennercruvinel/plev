//! The Capture page as a capability (task-76, item 10), over the fake
//! capture (`engine::runtime::platform::capture::FakeCapture`, no process, no
//! device): what it projects at rest, only a person turning a device on
//! (an agent refused before anything starts, and reading the state), the
//! newest frame into the stream image on the wake and not before, the
//! deadline asked only while a device is live, the devices released when
//! the page is not seen (covered, asleep, left) and taken back when it is,
//! the refusals as state, the level meter, and the keys. The gallery's
//! page holds no capture until a shell gives it one, so nothing in the
//! crate's tests reaches a device.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use accord::{Actor, Rejection, Request, Role, SemId, Stage, StateRef, Value};
use engine::drawing::compositor::Compositor;
use engine::runtime::platform::capture::{CaptureState, FakeCapture};
use engine::runtime::platform::menu::locked;
use engine::runtime::window::Background;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{Rect, WidgetEvent};

use super::*;
use crate::views::apps::todo::TodoPage;
use crate::views::dispatcher::PERSON_ONLY;

type Shared = Arc<Mutex<FakeCapture>>;

fn content() -> Rect {
    Rect::new(288.0, 118.0, 900.0, 682.0)
}

/// The active page over a shared fake: the test keeps the other handle.
fn setup(fake: FakeCapture) -> (TodoPage, CapturePage, Theme, Shared) {
    let theme = Theme::hoff_core();
    let shared = Arc::new(Mutex::new(fake));
    let mut page = CapturePage::new(&theme);
    page.set_capture(Box::new(shared.clone()));
    page.wake(0.0, true);
    (TodoPage::new(), page, theme, shared)
}

fn state(page: &CapturePage, at: &str) -> Value {
    page.snapshot()
        .expect("the page projects")
        .value(&StateRef::parse(at).expect("a state ref"))
        .cloned()
        .unwrap_or_else(|| panic!("{at} is projected"))
}

fn line(page: &CapturePage, kind: &str) -> String {
    match state(page, &format!("capture:{kind}")) {
        Value::Text(text) => text,
        other => panic!("a line: {other:?}"),
    }
}

fn toggle(page: &mut CapturePage, todo: &mut TodoPage, actor: Actor, id: &str) -> Stage {
    let seen = page.snapshot().unwrap();
    let req = Request::observe(&seen, actor, id, "toggle", None).unwrap();
    page.act(req, todo, false)
        .expect("a receipt")
        .stage()
        .cloned()
        .expect("a stage")
}

/// Wakes until the switches' knobs rest; what the last wake asked.
fn rest(page: &mut CapturePage, active: bool) -> Option<Duration> {
    for _ in 0..200 {
        let next = page.wake(0.05, active).next;
        if next != Some(Duration::ZERO) {
            return next;
        }
    }
    panic!("the knobs never rest")
}

fn human(page: &mut CapturePage, todo: &mut TodoPage, id: &str) {
    let stage = toggle(page, todo, Actor::Human, id);
    assert!(matches!(stage, Stage::Completed(_)), "{stage:?}");
}

#[test]
fn the_page_at_rest_projects_both_devices_off_and_asks_for_nothing() {
    let theme = Theme::hoff_core();
    let mut page = CapturePage::new(&theme);
    assert_eq!(page.wake(0.0, true).next, None);
    let snap = page.snapshot().unwrap();
    for (id, name) in [
        ("capture.camera", "camera"),
        ("capture.microphone", "microphone"),
    ] {
        let node = snap.get(&SemId::parse(id).unwrap()).expect("the toggle");
        assert_eq!(node.role, Role::Toggle);
        assert_eq!(node.label, name);
        assert_eq!(node.actions.len(), 1);
        assert_eq!(node.actions[0].name.as_str(), "toggle");
        assert_eq!(node.actions[0].policy.actors, PERSON_ONLY);
        assert_eq!(state(&page, &format!("{id}:on")), Value::Bool(false));
        assert_eq!(line(&page, name), "off");
    }
    assert_eq!(state(&page, "capture.frame:showing"), Value::Bool(false));
    assert_eq!(state(&page, "capture.level:level"), Value::Num(0.0));
    assert_eq!(
        state(&page, "capture.status:message"),
        Value::from("nothing is captured until a person turns a device on")
    );
    // drawn at rest: the card, no image
    let mut c = Compositor::new();
    page.render(&mut c, content(), &theme);
    assert!(c.binding(&SemId::parse("capture.frame").unwrap()).is_some());
    assert!(page.showing().is_none());
}

/// An agent asking to turn the camera on is refused before the page runs
/// anything: nothing starts, and it reads the state instead. The person's
/// same request turns it on; assistive technology is a person too.
#[test]
fn only_a_person_turns_a_device_on_and_an_agent_is_refused() {
    let (mut todo, mut page, _theme, fake) = setup(FakeCapture::new());
    let agent = Actor::Agent {
        name: "claude".to_owned(),
    };
    for id in ["capture.camera", "capture.microphone"] {
        let stage = toggle(&mut page, &mut todo, agent.clone(), id);
        assert!(
            matches!(&stage, Stage::Rejected(Rejection::NotAuthorized { actor }) if *actor == agent),
            "{stage:?}"
        );
    }
    assert_eq!(locked(&fake).starts(DeviceKind::Camera), 0, "never started");
    assert_eq!(locked(&fake).starts(DeviceKind::Microphone), 0);
    assert_eq!(line(&page, "camera"), "off");

    human(&mut page, &mut todo, "capture.camera");
    assert_eq!(locked(&fake).starts(DeviceKind::Camera), 1);
    assert_eq!(state(&page, "capture.camera:on"), Value::Bool(true));
    assert_eq!(line(&page, "camera"), "on 640x360 at 15 fps");
    assert_eq!(
        state(&page, "capture.status:message"),
        Value::from("the camera was turned on")
    );
    let stage = toggle(&mut page, &mut todo, Actor::Assistive, "capture.microphone");
    assert!(matches!(stage, Stage::Completed(_)), "{stage:?}");
    assert_eq!(line(&page, "microphone"), "on 16000 Hz");

    // the person turns them off: stopped, released
    human(&mut page, &mut todo, "capture.camera");
    human(&mut page, &mut todo, "capture.microphone");
    assert_eq!(locked(&fake).stops(DeviceKind::Camera), 1);
    assert_eq!(locked(&fake).stops(DeviceKind::Microphone), 1);
    assert_eq!(line(&page, "camera"), "off");
    assert_eq!(rest(&mut page, true), None);
}

/// The newest frame goes into the stream image on the wake after the
/// camera handed it, never before: the frame box shows nothing until
/// then. Every wake puts the next frame in the same slot under a new id.
#[test]
fn the_frame_enters_the_texture_on_the_wake_and_not_before() {
    let (mut todo, mut page, theme, fake) = setup(FakeCapture::new());
    human(&mut page, &mut todo, "capture.camera");
    assert!(page.showing().is_none(), "not before the wake");
    assert_eq!(locked(&fake).frames(), 0, "nothing taken yet");

    let wake = page.wake(0.0, true);
    assert!(wake.now, "a frame came: draw it");
    let first = page.showing().expect("the frame is in");
    assert_eq!((first.width, first.height), (640, 360));
    assert_eq!(locked(&fake).frames(), 1);
    assert_eq!(state(&page, "capture.frame:showing"), Value::Bool(true));

    page.wake(0.0, true);
    let second = page.showing().expect("the next frame");
    assert_eq!(
        (second.atlas_x, second.atlas_y),
        (first.atlas_x, first.atlas_y)
    );
    assert_ne!(second.id, first.id, "a new id, so the layer draws again");

    let mut c = Compositor::new();
    c.begin_frame();
    page.render(&mut c, content(), &theme);
    let drawn = c.layers().iter().flat_map(|l| l.nodes().iter()).any(
        |n| matches!(n, engine::drawing::compositor::SceneNode::Image { image, .. } if image == &second),
    );
    assert!(drawn, "the frame box draws the newest frame");
}

/// While a device is live the wake asks for the next look a frame period
/// away; with none live it asks for nothing, and a device refused is not
/// live.
#[test]
fn the_wake_asks_for_a_deadline_only_while_a_device_is_live() {
    let (mut todo, mut page, _theme, _fake) = setup(FakeCapture::new());
    assert_eq!(page.wake(0.0, true).next, None, "off");
    human(&mut page, &mut todo, "capture.microphone");
    let next = rest(&mut page, true).expect("live: a look is asked");
    let period = Duration::from_secs_f64(1.0 / 15.0);
    assert!(next.abs_diff(period) < Duration::from_millis(1), "{next:?}");
    human(&mut page, &mut todo, "capture.microphone");
    assert_eq!(rest(&mut page, true), None, "off again");

    let (mut todo, mut page, _theme, _fake) =
        setup(FakeCapture::new().will_come_to(DeviceKind::Camera, CaptureState::Denied));
    human(&mut page, &mut todo, "capture.camera");
    assert_eq!(rest(&mut page, true), None, "denied is not live");
}

/// Not seen (the window covered, the app asleep, the page left), a device
/// the person turned on is stopped and released, the frame box empties
/// and no frame is taken; seen again, it starts again.
#[test]
fn a_page_not_seen_releases_its_devices_and_takes_them_back() {
    let (mut todo, mut page, _theme, fake) = setup(FakeCapture::new());
    human(&mut page, &mut todo, "capture.camera");
    human(&mut page, &mut todo, "capture.microphone");
    page.wake(0.0, true);
    assert!(page.showing().is_some());

    // covered
    assert!(page.set_shown(false));
    assert_eq!(locked(&fake).stops(DeviceKind::Camera), 1);
    assert_eq!(locked(&fake).stops(DeviceKind::Microphone), 1);
    assert!(page.showing().is_none(), "the box waits for a new frame");
    assert_eq!(line(&page, "camera"), "paused while the page is not seen");
    assert_eq!(
        state(&page, "capture.camera:on"),
        Value::Bool(true),
        "still wanted"
    );
    let taken = locked(&fake).frames();
    assert_eq!(rest(&mut page, true), None, "nothing live, nothing asked");
    assert_eq!(
        locked(&fake).frames(),
        taken,
        "no frame taken while covered"
    );
    assert!(page.set_shown(true));
    assert_eq!(locked(&fake).starts(DeviceKind::Camera), 2);
    assert!(page.showing().is_none(), "nothing before the wake");
    page.wake(0.0, true);
    assert!(page.showing().is_some(), "a new frame");

    // asleep
    Background::sleeping(true);
    page.wake(0.1, true);
    assert_eq!(locked(&fake).stops(DeviceKind::Camera), 2);
    assert_eq!(locked(&fake).frames(), taken + 1, "no frame taken asleep");
    Background::sleeping(false);
    page.wake(0.0, true);
    assert_eq!(locked(&fake).starts(DeviceKind::Camera), 3);

    // left for another page, then back
    page.wake(0.0, false);
    assert_eq!(locked(&fake).stops(DeviceKind::Camera), 3);
    assert_eq!(locked(&fake).stops(DeviceKind::Microphone), 3);
    page.wake(0.0, true);
    assert_eq!(locked(&fake).starts(DeviceKind::Camera), 4);
    assert_eq!(locked(&fake).starts(DeviceKind::Microphone), 4);

    // a page that goes away stops what it started
    drop(page);
    assert_eq!(locked(&fake).stops(DeviceKind::Camera), 4);
}

/// What a start came to is the line the page projects, and a device
/// that ended on its own is not started again at every wake.
#[test]
fn refusals_are_projected_as_state() {
    for (lined, want) in [
        (CaptureState::Denied, "permission denied"),
        (CaptureState::NoDevice, "no camera"),
        (
            CaptureState::Failed("the camera stopped: gone".to_owned()),
            "failed: the camera stopped: gone",
        ),
    ] {
        let (mut todo, mut page, _theme, fake) =
            setup(FakeCapture::new().will_come_to(DeviceKind::Camera, lined));
        human(&mut page, &mut todo, "capture.camera");
        for _ in 0..5 {
            page.wake(0.0, true);
        }
        assert_eq!(line(&page, "camera"), want);
        assert_eq!(locked(&fake).starts(DeviceKind::Camera), 1, "{want}");
        assert!(page.showing().is_none());
    }
    // no capture at all (the page before the shell gives it one)
    let theme = Theme::hoff_core();
    let mut page = CapturePage::new(&theme);
    let mut todo = TodoPage::new();
    page.wake(0.0, true);
    human(&mut page, &mut todo, "capture.camera");
    assert_eq!(
        line(&page, "camera"),
        "failed: capture: no camera or microphone on this target"
    );
}

/// The microphone's last chunk sets the meter; the projection carries the
/// level to a tenth.
#[test]
fn the_microphone_sets_the_level_meter() {
    let (mut todo, mut page, _theme, _fake) = setup(FakeCapture::new());
    human(&mut page, &mut todo, "capture.microphone");
    assert!(page.wake(0.0, true).now);
    let want = meter(FakeCapture::RMS);
    assert!((want - 0.649).abs() < 0.01, "{want}");
    assert_eq!(state(&page, "capture.level:level"), Value::Num(0.6));
    // the bar's value, what a screen reader reads (task-75 phase 2a)
    assert_eq!(state(&page, "capture.level:value"), Value::Num(0.6));
    assert_eq!(meter(0.0), 0.0);
    assert_eq!(meter(1.0), 1.0);
    assert_eq!(meter(0.0001), 0.0, "under the floor");
    human(&mut page, &mut todo, "capture.microphone");
    assert_eq!(state(&page, "capture.level:level"), Value::Num(0.0));
}

/// The person's click on a switch and the keys: a click asks `toggle`
/// through the dispatcher (the widget's own flip put back first, so the
/// switch ends on), Tab walks the two switches, Enter flips the focused
/// one, Escape lets go.
#[test]
fn a_click_and_the_keys_are_the_persons_toggle() {
    let (mut todo, mut page, theme, fake) = setup(FakeCapture::new());
    let rect = page.switch_rect(DeviceKind::Camera, content(), &theme);
    let (x, y) = (rect.x + rect.w / 2.0, rect.y + rect.h / 2.0);
    for event in [
        WidgetEvent::MouseMove { x, y },
        WidgetEvent::MouseDown { x, y },
        WidgetEvent::MouseUp { x, y },
    ] {
        page.handle_event(&event, content(), &theme, &mut todo, false);
    }
    assert!(page.wanted(DeviceKind::Camera));
    assert_eq!(locked(&fake).starts(DeviceKind::Camera), 1);

    assert!(page.handle_edit_key(EditKey::Tab));
    assert!(page.handle_edit_key(EditKey::Tab));
    assert!(page.handle_enter(&mut todo, false), "the microphone");
    assert!(page.wanted(DeviceKind::Microphone));
    assert!(!page.handle_enter(&mut todo, true), "inert");
    assert!(page.handle_escape());
    assert!(!page.handle_escape());
    assert!(!page.handle_enter(&mut todo, false), "nothing focused");
    assert!(page.focus(&SemId::parse("capture.camera").unwrap()));
    assert!(!page.focus(&SemId::parse("files.open").unwrap()));
}

/// The card fits a phone and a wide window: inside the content, the frame
/// box at the camera's aspect and never wider than the card's inside or
/// the camera's own width, the switches and the meter inside the card.
#[test]
fn the_card_fits_a_phone_and_a_wide_window() {
    let theme = Theme::hoff_core();
    let mut page = CapturePage::new(&theme);
    for content in [content(), Rect::new(16.0, 118.0, 358.0, 682.0)] {
        let mut c = Compositor::new();
        c.begin_frame();
        page.render(&mut c, content, &theme);
        let rect = |id: &str| {
            let [x, y, w, h] = c
                .binding(&SemId::parse(id).unwrap())
                .unwrap_or_else(|| panic!("{id} is bound"))
                .rect;
            Rect::new(x, y, w, h)
        };
        let card = rect("capture");
        let frame = rect("capture.frame");
        assert!(card.x >= content.x && card.right() <= content.right() + 0.5);
        assert!(frame.x >= card.x && frame.right() <= card.right());
        assert!(frame.w <= 640.0);
        assert!(
            (frame.h - (frame.w * 360.0 / 640.0).floor()).abs() < 0.5,
            "{frame:?}"
        );
        for id in ["capture.camera", "capture.microphone", "capture.level"] {
            let r = rect(id);
            assert!(r.x >= card.x && r.right() <= card.right(), "{id} {r:?}");
            assert!(r.y >= card.y && r.y + r.h <= card.y + card.h, "{id} {r:?}");
        }
        assert!(
            (page.content_height(content, &theme) - card.h).abs() < 0.5,
            "the height is the card's"
        );
    }
}
