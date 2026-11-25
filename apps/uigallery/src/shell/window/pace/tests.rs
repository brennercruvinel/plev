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
    use engine::widgets::prelude::WidgetEvent;
    use winit::window::WindowId;

    use super::{App, TURN};
    use crate::views::ShowcaseView;

    const WINDOW: (u32, u32) = (1280, 800);

    /// A pixel of the window's right edge just under its middle: the
    /// page's ground while the panel is closed, the panel's strip, which
    /// has another ground, while it is open at the bottom.
    const CORNER: (u32, u32) = (WINDOW.0 - 2, WINDOW.1 / 2 + 6);

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

    /// A click in the middle of `at`.
    fn press(app: &mut App, id: WindowId, (x, y): (f32, f32)) {
        for event in [
            WidgetEvent::MouseMove { x, y },
            WidgetEvent::MouseDown { x, y },
            WidgetEvent::MouseUp { x, y },
        ] {
            app.pointer(id, &event);
        }
    }

    /// A click on the gear where the dev mode puts it: the page's lower
    /// right corner.
    fn press_gear(app: &mut App, id: WindowId) {
        let gear = app.dev.gear(WINDOW.0 as f32, WINDOW.1 as f32);
        press(app, id, gear.center());
    }

    /// A click on the close of the open panel: the right end of its strip.
    fn press_close(app: &mut App, id: WindowId) {
        let (_, page_h) = app.dev.page_size(WINDOW.0 as f32, WINDOW.1 as f32);
        press(app, id, (WINDOW.0 as f32 - 16.0, page_h + 16.0));
    }

    fn corner(app: &App, id: WindowId) -> Option<[u8; 4]> {
        let host = app.windows.get(id).expect("the window is there");
        let shot = host.read().expect("the frame reads back");
        shot.pixel(CORNER.0, CORNER.1)
    }

    #[test]
    fn the_gear_opens_the_panel_in_the_window_and_the_panel_reads_its_frames() {
        let Some((mut app, id)) = windowed(true) else {
            return;
        };
        frame(&mut app, id);
        let page = corner(&app, id);
        assert!(!app.dev.is_open());
        assert!(!app.windows.get(id).unwrap().dev().is_on());

        press_gear(&mut app, id);
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
        assert_ne!(corner(&app, id), page, "the panel's strip is drawn there");
        let whole = recorder.settled().expect("a frame to show");
        assert!(
            !recorder.gpu_available() || whole.gpu.is_some(),
            "the frame the panel shows has its GPU time"
        );

        press_close(&mut app, id);
        assert!(!app.dev.is_open(), "the press closed the panel");
        assert_eq!(
            app.view.height, WINDOW.1 as f32,
            "the page has the window again"
        );
        frame(&mut app, id);
        assert!(!app.windows.get(id).unwrap().dev().is_on());
        assert_eq!(corner(&app, id), page, "the page is back there");
    }

    #[test]
    fn the_picker_chooses_the_control_under_the_pointer_and_the_page_keeps_its_state() {
        let Some((mut app, id)) = windowed(true) else {
            return;
        };
        frame(&mut app, id);
        press_gear(&mut app, id);
        frame(&mut app, id);
        let (_, page_h) = app.dev.page_size(WINDOW.0 as f32, WINDOW.1 as f32);
        // the picker is the first place of the strip
        press(&mut app, id, (16.0, page_h + 16.0));
        assert_eq!(app.dev.tab(), engine::runtime::dev::Tab::Inspect);
        frame(&mut app, id);

        // a control of the page that shows in its half of the window
        let host = app.windows.get(id).unwrap();
        let (target, [x, y, w, h]) = host
            .compositor()
            .visible_bindings()
            .filter(|(_, [_, y, _, h])| y + h < page_h)
            .map(|(id, rect)| (id.clone(), rect))
            .last()
            .expect("the Todo page binds its controls");
        let before = app.view.semantics();
        press(&mut app, id, (x + w / 2.0, y + h / 2.0));
        // the control itself, or what it draws under the pointer (its
        // label, its mark), whichever is the more particular: either way
        // the control the choice is, or sits in, is the one pressed
        let chosen = app.dev.picked_kind().expect("the press chose something");
        assert_eq!(
            app.dev.picked_owner(),
            Some(&target),
            "the control under the press, or a {chosen} node of it"
        );
        assert_eq!(
            app.view.semantics(),
            before,
            "the press did not reach the page"
        );
    }

    #[test]
    fn the_picker_chooses_an_icon_of_a_page_that_gives_it_no_id() {
        let Some((mut app, id)) = windowed(true) else {
            return;
        };
        app.view.jump_to_page("Icons");
        frame(&mut app, id);
        press_gear(&mut app, id);
        frame(&mut app, id);
        let (_, page_h) = app.dev.page_size(WINDOW.0 as f32, WINDOW.1 as f32);
        press(&mut app, id, (16.0, page_h + 16.0));
        frame(&mut app, id);

        // an icon the page draws in its half of the window, at no id:
        // nothing is bound where it is
        let host = app.windows.get(id).unwrap();
        let c = host.compositor();
        let [x, y, w, h] = c
            .layers()
            .iter()
            .filter(|layer| layer.z_order == 0)
            .flat_map(|layer| layer.nodes())
            .filter(|node| node.kind() == "Path")
            .filter_map(|node| node.bounds())
            .find(|[x, y, w, h]| {
                let at = (x + w / 2.0, y + h / 2.0);
                *y > 160.0 && y + h < page_h - 8.0 && c.hit(at.0, at.1).is_none()
            })
            .expect("the Icons page draws its icons as paths");
        press(&mut app, id, (x + w / 2.0, y + h / 2.0));
        assert_eq!(
            app.dev.picked_kind(),
            Some("Path"),
            "the icon under the press is what was chosen"
        );
        assert_eq!(app.dev.picked(), None, "it is no control");
    }

    #[test]
    fn the_dev_key_opens_and_closes_the_panel_and_is_the_pages_without_a_dev_mode() {
        let (Some((mut app, id)), Some((mut none, none_id))) = (windowed(true), windowed(false))
        else {
            return;
        };
        frame(&mut app, id);
        assert!(app.dev_key(id), "the key is the dev mode's");
        assert!(app.dev.is_open());
        assert_eq!(app.view.height, WINDOW.1 as f32 / 2.0, "the page gave half");
        assert!(app.dev_key(id));
        assert!(!app.dev.is_open());
        assert_eq!(app.view.height, WINDOW.1 as f32);

        frame(&mut none, none_id);
        assert!(!none.dev_key(none_id), "no dev mode: the key goes on");
        assert!(!none.dev.is_open());
    }

    #[test]
    fn a_launch_without_the_dev_mode_draws_no_gear_and_takes_no_press() {
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

        press_gear(&mut app, id);
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
