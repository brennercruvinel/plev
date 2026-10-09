//! Reduced motion on the widgets the gallery shows (task-75 phase 1):
//! with the system's "reduce motion" on (`engine::animation`, set on the
//! event loop's thread by the accessibility bridge), a switch's knob, a
//! toast's slide, a modal's fade and scale, the todo rows and the whole
//! gallery reach their end on the first frame, and ask for no more.
//! Without it, the same first frame is mid-motion. The ambient loops,
//! the spinner's turn and the skeleton's pulse, stand still too (task-75
//! phase 2a, tested with their pages).
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use engine::compose::animation::{reduced_motion, set_reduced_motion};
use engine::widgets::overlay::{OverlayKind, OverlayManager};
use engine::widgets::prelude::{Intent, Switch, Theme, ToastManager};
use uigallery::views::ShowcaseView;

const FRAME: f32 = 1.0 / 60.0;

/// The preference on for the test that holds it, off again at its end.
struct Reduced;

impl Reduced {
    fn on() -> Self {
        set_reduced_motion(true);
        Reduced
    }
}

impl Drop for Reduced {
    fn drop(&mut self) {
        set_reduced_motion(false);
    }
}

/// A switch flipped, a toast pushed and a modal opened, one frame
/// ticked: whether each is still moving.
fn first_frame() -> [bool; 3] {
    let theme = Theme::hoff_core();
    let mut switch = Switch::new(false);
    switch.toggle();
    let knob = switch.tick(FRAME);

    let mut toasts = ToastManager::new();
    toasts.push("saved", Intent::Constructive, &theme);
    let toast = toasts.wake(FRAME).is_continuous();

    let mut overlays = OverlayManager::new();
    let modal = OverlayKind::Modal {
        title: "Apply settings?".into(),
        body: String::new(),
        confirm: "Apply".into(),
        cancel: "Cancel".into(),
    };
    overlays.push_animated(modal, 0.0, 0.0, 400.0, 200.0, &theme.motion);
    let fade = overlays.tick(FRAME);
    [knob, toast, fade]
}

#[test]
fn with_motion_the_widgets_are_mid_motion_after_one_frame() {
    assert!(!reduced_motion(), "a test thread starts with motion");
    assert_eq!(first_frame(), [true, true, true]);
}

#[test]
fn reduced_the_widgets_end_on_the_first_frame() {
    let _reduced = Reduced::on();
    assert_eq!(first_frame(), [false, false, false]);

    let theme = Theme::hoff_core();
    let mut switch = Switch::new(false);
    switch.toggle();
    switch.tick(FRAME);
    assert_eq!(switch.knob_progress(), 1.0, "the knob is across");
    let mut toasts = ToastManager::new();
    toasts.push("saved", Intent::Constructive, &theme);
    toasts.wake(FRAME);
    let shown: Vec<f32> = toasts.visible().map(|t| t.progress()).collect();
    assert_eq!(shown, [1.0], "the toast is in");
    let mut overlays = OverlayManager::new();
    let modal = OverlayKind::Tooltip { text: "hi".into() };
    overlays.push_animated(modal, 0.0, 0.0, 80.0, 24.0, &theme.motion);
    overlays.tick(FRAME);
    let open: Vec<(f32, f32)> = overlays
        .stack
        .iter()
        .map(|o| (o.opacity(), o.scale()))
        .collect();
    assert_eq!(open, [(1.0, 1.0)], "the overlay is open and full size");
}

/// The gallery as the window drives it: a toast arrives on the todo page
/// and the view is ticked once. Reduced, nothing moves after that frame
/// (the loop may sleep); with motion, the next frame is asked for.
#[test]
fn the_gallery_settles_on_the_first_frame_when_reduced() {
    let moving_after_one_frame = || {
        let mut view = ShowcaseView::new(1280.0, 800.0);
        view.jump_to_page("todo");
        let theme = view.theme.clone();
        view.toasts.push("saved", Intent::Constructive, &theme);
        view.tick(FRAME)
    };
    assert!(moving_after_one_frame(), "with motion the toast slides in");
    let _reduced = Reduced::on();
    assert!(!moving_after_one_frame(), "reduced: it is in at once");
}
