//! What the checkpoint b of task-69 found in every generated surface,
//! pinned: a slider shows its label and its value, as the fixed screens'
//! sliders do.

// shared with the other surface tests: this binary uses the fixtures.
#[allow(dead_code)]
mod support;

use comps::Rect;
use engine::compositor::{Compositor, LayerId, SceneNode};
use engine::theme::Theme;
use support::{App, MIXER_LEFT, id, mixer, verified_local};
use surface::Surface;

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
