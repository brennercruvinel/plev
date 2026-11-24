//! The shell's pacing with an injected clock: a change from outside the
//! frame lands after the loop caught up (`App::advance`, what `catch_up`
//! runs with the real elapsed time), so it counts its time from the
//! change, however long the window sat idle before it.

use engine::styling::theme::Intent;
use engine::widgets::prelude::Wake;

use super::App;
use crate::views::ShowcaseView;

/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

/// How long the window sat idle before the event.
const IDLE: f32 = 10.0;

/// How long after the event its frame is drawn.
const LATENCY: f32 = 0.004;

/// The gallery's shell, settled: nothing it starts with still moves.
fn app(page: &str) -> App {
    let mut app = App::with_view(ShowcaseView::new(1200.0, 800.0));
    app.view.jump_to_page(page);
    for _ in 0..600 {
        if !app.frame_wake(TURN).is_continuous() {
            break;
        }
    }
    app
}

/// The frames over `turns` display turns after an event: turn 0 is the
/// event's frame; afterwards a turn is drawn when the last ask was
/// continuous or its deadline has come due, and its tick gets the time
/// since the last drawn turn.
fn paced(app: &mut App, turns: usize) -> Vec<(usize, Wake)> {
    let mut drawn = vec![(0, app.frame_wake(LATENCY))];
    for turn in 1..turns {
        let (last, ask) = *drawn.last().expect("turn 0 is drawn");
        let since = (turn - last) as f32 * TURN;
        if ask.next.is_some_and(|d| since >= d.as_secs_f32()) {
            drawn.push((turn, app.frame_wake(since)));
        }
    }
    drawn
}

#[test]
fn a_toast_pushed_after_an_idle_gap_lives_its_whole_duration() {
    let mut app = app("Button");
    app.advance(IDLE);
    let theme = app.view.theme.clone();
    app.view.toasts.push("saved", Intent::Constructive, &theme);
    let drawn = paced(&mut app, 360);
    let toast = |app: &App| app.view.toasts.visible().next().map(|t| t.is_closing());
    // springs in, rests on the deadline, leaves 4 s after the push
    let rest = drawn
        .iter()
        .position(|(_, ask)| !ask.is_continuous())
        .expect("the entry settles");
    let (rest_turn, rest_ask) = drawn[rest];
    let left = rest_ask.next.expect("a deadline while it rests");
    let due = rest_turn as f32 * TURN + LATENCY + left.as_secs_f32();
    assert!((due - 4.0).abs() < TURN, "leaves {due} s after the push");
    assert_eq!(toast(&app), None, "gone within six seconds");
}

#[test]
fn a_toast_pushed_after_an_idle_gap_springs_in_from_the_start() {
    let mut app = app("Button");
    app.advance(IDLE);
    let theme = app.view.theme.clone();
    app.view.toasts.push("saved", Intent::Constructive, &theme);
    let ask = app.frame_wake(LATENCY);
    assert!(ask.is_continuous(), "springing in");
    let shown = app.view.toasts.visible().next().expect("on screen");
    assert!(!shown.is_closing());
    assert!(shown.progress() < 0.1, "{}", shown.progress());
}

#[test]
fn a_caret_focused_after_an_idle_gap_starts_solid_and_flips_on_time() {
    let mut app = app("Git");
    app.advance(IDLE);
    // `c` opens the commit form, focused
    assert!(app.view.handle_key("c"));
    let drawn = paced(&mut app, 360);
    let (_, first) = drawn[0];
    assert!(
        !first.now || first.is_continuous(),
        "no flip on the key's frame"
    );
    // a whole half-period from the key, none of the idle before it
    let next = first.next.expect("a flip is pending").as_secs_f32();
    assert!(
        (0.53 - LATENCY - 1e-3..0.53 + 1e-3).contains(&next),
        "{next}"
    );
    let flips: Vec<usize> = drawn[1..]
        .iter()
        .filter(|(_, ask)| ask.now)
        .map(|(turn, _)| *turn)
        .collect();
    // 0.53 s, 1.06 s ... after the key: the first on turn 32
    assert_eq!(flips.len(), 11, "{drawn:?}");
    assert_eq!(flips[0], 32);
}

#[test]
fn a_catch_up_that_changes_nothing_asks_for_no_frame() {
    let mut app = app("Button");
    app.advance(IDLE);
    assert!(!app.caught_up);
    assert_eq!(app.frame_wake(TURN), Wake::idle());
}

/// The dev mode in the window, without a platform window: the shell's own
/// pointer and frame paths (`App::pointer`, `App::main_frame`) against a
/// host that draws into a texture. Each test skips where no GPU adapter
/// answers.
mod dev {
    use engine::drawing::gpu::RenderConfig;
    use engine::runtime::dev::DevMode;
    use engine::runtime::window::WindowHost;
    use engine::widgets::prelude::{Button, ButtonSize, WidgetEvent};
    use winit::window::WindowId;

    use super::{App, TURN};
    use crate::views::ShowcaseView;

    const WINDOW: (u32, u32) = (1280, 800);

    /// A pixel of the window's corner, where the page draws nothing and
    /// the open panel does.
    const CORNER: (u32, u32) = (WINDOW.0 - 4, WINDOW.1 - 4);

    /// The shell around the Todo page, launched with a dev mode or
    /// without (`dev`), in a window that draws into a texture and times
    /// its passes on the GPU.
    fn windowed(dev: bool) -> Option<(App, WindowId)> {
        let config = RenderConfig {
            gpu_timestamps: true,
            ..RenderConfig::default()
        };
        let host = WindowHost::headless(WINDOW.0, WINDOW.1, 1.0, config)?;
        let mut app = App::with_view(ShowcaseView::new(WINDOW.0 as f32, WINDOW.1 as f32));
        app.view.jump_to_page("Todo");
        app.dev = DevMode::new(dev);
        let id = WindowId::dummy();
        app.main = Some(id);
        app.windows.insert(id, host);
        Some((app, id))
    }

    /// One frame of the main window, as the loop draws it.
    fn frame(app: &mut App, id: WindowId) {
        let wake = app.frame_wake(TURN);
        let clear = app.view.theme.colors.bg.to_linear_array();
        app.main_frame(id, wake, clear)
            .expect("the window is there");
    }

    /// A click on the "Dev" button where the dev mode puts it: the bottom
    /// center of the page, a large space above its edge.
    fn press_dev(app: &mut App, id: WindowId) {
        let theme = &app.view.theme;
        let button = Button::new("Dev").size(ButtonSize::Sm);
        let x = app.view.width / 2.0;
        let y = app.view.height - theme.spacing.lg - button.preferred_size(theme).1 / 2.0;
        for event in [
            WidgetEvent::MouseMove { x, y },
            WidgetEvent::MouseDown { x, y },
            WidgetEvent::MouseUp { x, y },
        ] {
            app.pointer(id, &event);
        }
    }

    fn corner(app: &App, id: WindowId) -> Option<[u8; 4]> {
        let host = app.windows.get(id).expect("the window is there");
        let shot = host.read().expect("the frame reads back");
        shot.pixel(CORNER.0, CORNER.1)
    }

    #[test]
    fn the_dev_button_opens_the_panel_in_the_window_and_the_panel_reads_its_frames() {
        let Some((mut app, id)) = windowed(true) else {
            return;
        };
        frame(&mut app, id);
        let page = corner(&app, id);
        assert!(!app.dev.is_open());
        assert!(!app.windows.get(id).unwrap().dev().is_on());

        press_dev(&mut app, id);
        assert!(app.dev.is_open(), "the press opened the panel");
        let half = (WINDOW.0 as f32, WINDOW.1 as f32 / 2.0);
        assert_eq!(
            (app.view.width, app.view.height),
            half,
            "the page keeps half"
        );

        for _ in 0..6 {
            frame(&mut app, id);
        }
        let recorder = app.windows.get(id).unwrap().dev();
        assert!(recorder.is_on(), "an open panel records");
        assert_eq!(recorder.len(), 6, "one report per frame drawn");
        assert!(
            recorder.history().any(|f| f.tool),
            "the panel asked for a frame of its own"
        );
        let latest = recorder.latest().expect("a frame of the app's own");
        assert!(latest.pipeline.cpu_nanos > 0, "the stages were recorded");
        let zones = latest.zones.stats();
        for name in ["scene build", "dev mode", "draw_frame"] {
            assert!(zones.iter().any(|z| z.name == name), "no zone {name}");
        }
        let bound = latest.plots.iter().find(|p| p.name == "controls bound");
        assert!(bound.is_some_and(|p| p.value > 0.0), "the page's plot");
        if recorder.gpu_available() {
            assert!(
                recorder.history().any(|f| f.gpu.is_some()),
                "no frame got its GPU time back"
            );
        }
        assert_ne!(corner(&app, id), page, "the panel is drawn over the corner");

        press_dev(&mut app, id);
        assert!(!app.dev.is_open(), "the press closed the panel");
        assert_eq!(
            app.view.height, WINDOW.1 as f32,
            "the page has the window again"
        );
        frame(&mut app, id);
        assert!(!app.windows.get(id).unwrap().dev().is_on());
        assert_eq!(corner(&app, id), page, "the page is back in the corner");
    }

    #[test]
    fn a_launch_without_the_dev_mode_draws_no_button_and_takes_no_press() {
        let (Some((mut with, with_id)), Some((mut app, id))) = (windowed(true), windowed(false))
        else {
            return;
        };
        frame(&mut with, with_id);
        frame(&mut app, id);
        let layers = |app: &App, id| app.windows.get(id).unwrap().compositor().layers().len();
        assert_eq!(
            layers(&with, with_id),
            layers(&app, id) + 1,
            "no layer of its own"
        );

        press_dev(&mut app, id);
        frame(&mut app, id);
        assert!(!app.dev.is_open());
        assert_eq!(app.view.height, WINDOW.1 as f32);
        let recorder = app.windows.get(id).unwrap().dev();
        assert!(
            !recorder.is_on() && recorder.is_empty(),
            "nothing is recorded"
        );
    }
}
