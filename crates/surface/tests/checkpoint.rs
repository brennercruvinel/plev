//! What the checkpoint b of task-69 found in every generated surface,
//! pinned: a slider shows its label and its value, as the fixed screens'
//! sliders do; a number shows with two decimals at most, on the screen
//! and in what the surface projects.

// shared with the other surface tests: this binary uses the fixtures.
#[allow(dead_code)]
mod support;

use comps::Rect;
use engine::compositor::{Compositor, LayerId, SceneNode};
use engine::theme::Theme;
use sem::StateRef;
use support::{App, MIXER_LEFT, SILENCES, id, mixer, page, snapshot, verified_local};
use surface::Surface;
use uir::{Capabilities, GenerationPolicy, verify_json};

const AREA: Rect = Rect {
    x: 16.0,
    y: 24.0,
    w: 480.0,
    h: 600.0,
};

/// Every text run of the frame, with where it starts.
fn texts(c: &Compositor) -> Vec<(String, f32, f32)> {
    c.layer(LayerId::DEFAULT)
        .map(|l| {
            l.nodes()
                .iter()
                .filter_map(|n| match n {
                    SceneNode::Text { key, x, y, .. } => Some((key.text.clone(), *x, *y)),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The volume slider of the mixer: its label ("volume") at the left
/// edge and its value ("0.5") at the right edge of the slider, on a line
/// above the track the slider binds itself to; a drag moves the value
/// shown.
#[test]
fn a_slider_draws_its_label_and_its_value_above_its_track() {
    let theme = Theme::hoff_core();
    let mut app = App::over(&mixer());
    let now = app.snapshot();
    let mut surface = Surface::materialize(verified_local(MIXER_LEFT, &now), &now);
    let mut c = Compositor::new();
    surface.render(&mut c, AREA, &theme);
    let track = c.binding(&id("mix_panel.volume")).expect("bound").rect;
    let runs = texts(&c);
    let find = |s: &str| {
        runs.iter()
            .find(|(t, _, _)| t == s)
            .unwrap_or_else(|| panic!("{s:?} is not drawn: {runs:?}"))
            .clone()
    };
    let (_, lx, ly) = find("volume");
    let (_, vx, vy) = find("0.5");
    assert_eq!(lx, track[0], "the label at the left edge");
    assert!(ly < track[1] && vy < track[1], "above the track");
    assert_eq!(ly, vy, "on one line");
    assert!(
        vx > lx && vx < track[0] + track[2],
        "the value at the right"
    );
    assert_eq!(
        track[3],
        theme.control.height(engine::theme::ControlSize::Xs),
        "the track is the control's height"
    );

    // a drag to three quarters of the travel shows 0.75
    let knob = theme.control.slider_knob;
    let y = track[1] + track[3] / 2.0;
    let x = |t: f32| track[0] + knob / 2.0 + t * (track[2] - knob);
    for event in [
        comps::WidgetEvent::MouseMove { x: x(0.5), y },
        comps::WidgetEvent::MouseDown { x: x(0.5), y },
        comps::WidgetEvent::MouseMove { x: x(0.75), y },
    ] {
        surface.handle_event(&event, &theme, &mut app);
    }
    let mut c = Compositor::new();
    surface.render(&mut c, AREA, &theme);
    assert!(
        texts(&c).iter().any(|(t, _, _)| t == "0.75"),
        "{:?}",
        texts(&c)
    );
}

/// The numbers of a cut over the silences app, as the checkpoint saw
/// them raw: the duration in a stat ("1.996916") and the start in a text
/// ("3.018594") show "2" and "3.02", drawn and projected; the app's state
/// keeps every decimal.
#[test]
fn a_number_shows_two_decimals_at_most_on_screen_and_in_the_projection() {
    let theme = Theme::hoff_core();
    let app = snapshot(SILENCES);
    let json = r#"{
      "plev_ir": 1,
      "surface": {
        "id": "numbers",
        "title": "numbers",
        "root": {
          "sem": "numbers.card",
          "component": "card",
          "label": "the first cut",
          "layout": {"stack": {"axis": "vertical", "gap": "sm"}},
          "children": [
            {
              "sem": "numbers.duration",
              "component": "stat",
              "label": "duration (s)",
              "bind": {"state": {"value": "silences.cut[1]:duration"}}
            },
            {
              "sem": "numbers.start",
              "component": "text",
              "bind": {"state": {"text": "silences.cut[1]:start"}}
            }
          ]
        }
      }
    }"#;
    let ir = verify_json(
        json,
        &Capabilities::from_snapshot(&app),
        &GenerationPolicy::default(),
    )
    .unwrap_or_else(|e| panic!("{e:#?}"));
    let mut surface = Surface::materialize(ir, &app);
    let mut c = Compositor::new();
    surface.render(&mut c, AREA, &theme);
    let drawn: Vec<String> = texts(&c).into_iter().map(|(t, _, _)| t).collect();
    for shown in ["2", "3.02"] {
        assert!(drawn.iter().any(|t| t == shown), "{shown}: {drawn:?}");
    }
    for raw in ["1.996916", "3.018594"] {
        assert!(!drawn.iter().any(|t| t.contains(raw)), "{raw}: {drawn:?}");
    }
    let projected = page(&app, &surface);
    let start = projected.get(&id("numbers.start")).expect("projected");
    assert_eq!(start.label, "3.02", "a text is named by what it shows");
    let kept = StateRef::parse("silences.cut[1]:start").unwrap();
    assert_eq!(app.value(&kept), Some(&sem::Value::Num(3.018594)));
}
