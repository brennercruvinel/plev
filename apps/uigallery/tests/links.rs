//! The gallery's links on screen (task-76 item 6), without a window and
//! without opening a link: a `ShowcaseView` over a headless host (the
//! window's frame path into a texture), a link text given to
//! `showcase::app::links::follow` as a platform would deliver it. A
//! link to the camera is refused and its line is on screen, the page as
//! it was around it; a link to the Files page draws, byte for byte, the
//! frame of a gallery that went there by its own sidebar name. Desktop
//! only, like the shell; skips without a GPU adapter, like the headless
//! tests.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use engine::drawing::gpu::RenderConfig;
use engine::drawing::headless::Frame;
use engine::runtime::platform::links::read;
use engine::runtime::window::WindowHost;
use accord::{Stage, StateRef, Value};
use uigallery::shell::system::links::{OPEN, SCHEME, follow};
use uigallery::views::ShowcaseView;

const MAIN: (u32, u32) = (1200, 800);
/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;
/// Around a toast, what its shadow may reach.
const HALO: f32 = 48.0;

fn host() -> Option<WindowHost> {
    let host = WindowHost::headless(MAIN.0, MAIN.1, 1.0, RenderConfig::default());
    if host.is_none() {
        eprintln!("no GPU adapter; skipping");
    }
    host
}

/// `page` laid out for the main window, settled.
fn view(page: &str) -> ShowcaseView {
    let mut view = ShowcaseView::new(MAIN.0 as f32, MAIN.1 as f32);
    view.jump_to_page(page);
    view.resize(MAIN.0 as f32, MAIN.1 as f32, 1.0);
    settle(&mut view);
    view
}

/// Ticks until nothing moves (a toast springs in).
fn settle(view: &mut ShowcaseView) {
    for _ in 0..600 {
        if !view.tick(TURN) {
            return;
        }
    }
    panic!("the view never settles");
}

fn frame(view: &mut ShowcaseView, host: &mut WindowHost) -> Frame {
    host.begin_frame();
    view.render(host.compositor_mut());
    let clear = view.theme.colors.bg.to_linear_array();
    assert!(host.present(clear, "main").is_some(), "the frame drew");
    host.read().expect("the frame reads")
}

fn camera(view: &ShowcaseView) -> Option<Value> {
    view.semantics()?
        .value(&StateRef::parse("capture:camera").ok()?)
        .cloned()
}

#[test]
fn a_link_to_the_camera_is_refused_and_its_line_is_on_screen() {
    let Some(mut host) = host() else {
        return;
    };
    let mut view = view("Capture");
    let before = frame(&mut view, &mut host);
    assert_eq!(camera(&view), Some(Value::from("off")));

    let url = "plev-showcase://capture.camera/toggle";
    let followed = follow(&mut view, read(url, SCHEME), OPEN, &mut |_, _| {});
    assert!(followed.receipt.is_none(), "no request was made");
    let line = followed.told.expect("the person is told");
    assert_eq!(
        line,
        "link refused: plev-showcase://capture.camera/toggle: capture.camera toggle is not open \
         to links"
    );
    settle(&mut view);
    let after = frame(&mut view, &mut host);

    let shown: Vec<&str> = view.toasts.visible().map(|t| t.message.as_str()).collect();
    assert_eq!(shown, [line.as_str()]);
    let rects = view
        .toasts
        .visible_rects(&view.theme, MAIN.0 as f32, MAIN.1 as f32);
    assert_eq!(rects.len(), 1);
    let toast = rects[0];
    let (mut inside, mut around) = (0usize, 0usize);
    for y in 0..MAIN.1 {
        for x in 0..MAIN.0 {
            if before.pixel(x, y) == after.pixel(x, y) {
                continue;
            }
            let (px, py) = (x as f32, y as f32);
            if px >= toast.x && px < toast.x + toast.w && py >= toast.y && py < toast.y + toast.h {
                inside += 1;
            } else if px < toast.x - HALO
                || px >= toast.x + toast.w + HALO
                || py < toast.y - HALO
                || py >= toast.y + toast.h + HALO
            {
                around += 1;
            }
        }
    }
    let area = (toast.w * toast.h) as usize;
    assert!(
        inside > area / 2,
        "the toast is drawn where it stands: {inside} of {area}"
    );
    assert_eq!(around, 0, "the page is as it was around it");
    assert_eq!(camera(&view), Some(Value::from("off")), "nothing ran");
}

#[test]
fn a_link_to_the_files_page_draws_that_page_byte_for_byte() {
    let (Some(mut host), Some(mut control_host)) = (host(), host()) else {
        return;
    };
    let mut gallery = view("Todo");
    let url = "plev-showcase://shell.page/select?value=Files";
    let followed = follow(&mut gallery, read(url, SCHEME), OPEN, &mut |_, _| {});
    let receipt = followed.receipt.expect("dispatched");
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    assert!(followed.told.is_none());
    settle(&mut gallery);
    let got = frame(&mut gallery, &mut host);

    // the same page reached by its sidebar name
    let mut control = view("Files");
    let want = frame(&mut control, &mut control_host);
    assert!(
        got.rgba == want.rgba,
        "the Files page, as its own name draws it"
    );
}
