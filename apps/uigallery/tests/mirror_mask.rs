//! The secret does not cross (task-79 phase 2): a frame is offered with
//! the places of the bound controls the port's one rule counts as secret
//! masked (`uigallery::shell::window::mirror::masks`, `record::scene::Sender::frame_masked`),
//! the shell's path. A secret drawn for real (`login`: a password field
//! and a pin said by a status, marked secret by the projection) is in no
//! line and no byte of the watch, the mirror draws a box in each secret's
//! place and is the app's frame everywhere else, and the person typing in
//! the secret field sends nothing. Every gallery page in both themes
//! through the same path: the pixels of phase 1 where nothing is masked,
//! and a difference only inside the masked places where something is (the
//! Git page's three panes, which the gallery does not project: the rule
//! fails closed).
//!
//! Desktop only, like the port; skips without a GPU adapter.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use crate::support;

use accord::Request;
use engine::drawing::gpu::RenderConfig;
use engine::runtime::window::WindowHost;
use record::scene::{Head, Mask, SECRET_LABEL, Sender};
use socket::{Outgoing, Reply, encode_server};
use uigallery::shell::window::mirror::{MAIN, Viewer, masks, offer};
use uigallery::views::{PAGES, ShowcaseView};

fn host(w: f32, h: f32) -> Option<WindowHost> {
    let host = WindowHost::headless(w as u32, h as u32, 1.0, RenderConfig::default());
    if host.is_none() {
        eprintln!("no GPU adapter; skipping");
    }
    host
}

fn head(w: f32, h: f32, clear: [f32; 4]) -> Head {
    Head {
        width: w,
        height: h,
        scale: 1.0,
        clear,
    }
}

/// A server whose scenes are shared, with one watch of the main window.
fn watched() -> socket::Server {
    let mut server = socket::Server::new("mirror", 8);
    server.share_scenes(true);
    for ask in [
        socket::Ask::Hello {
            name: "mirror".into(),
        },
        socket::Ask::Scene(socket::SceneAsk {
            window: MAIN.into(),
        }),
    ] {
        let out = server.handle(
            1,
            socket::ClientMsg {
                id: "w".into(),
                ask,
            },
            &mut NoHost,
        );
        assert!(!out.iter().any(|o| matches!(o.msg.reply, Reply::Error(_))));
    }
    server
}

struct NoHost;

impl socket::Host for NoHost {
    fn project(&self) -> Option<accord::Snapshot> {
        None
    }

    fn dispatch(&mut self, _req: Request) -> Option<accord::RequestId> {
        None
    }

    fn add_sink(&mut self, _sink: Box<dyn accord::ReceiptSink>) {}
}

/// Every line of `out` as the port writes it, and every binary part in
/// them decoded.
fn wire_of(out: &[Outgoing]) -> (Vec<String>, Vec<Vec<u8>>) {
    let mut lines = Vec::new();
    let mut parts = Vec::new();
    for o in out {
        lines.push(encode_server(&o.msg));
        match &o.msg.reply {
            Reply::Scene(f) => {
                parts.extend(f.head.clone());
                for l in &f.layers {
                    parts.extend(l.body.clone());
                    parts.extend(l.patch.clone());
                }
            }
            Reply::SceneBlob(b) => parts.push(b.bytes.clone()),
            _ => {}
        }
    }
    (lines, parts)
}

fn carries(lines: &[String], parts: &[Vec<u8>], text: &str) -> bool {
    lines.iter().any(|l| l.contains(text))
        || parts
            .iter()
            .any(|p| p.windows(text.len()).any(|w| w == text.as_bytes()))
}

fn inside([x, y, w, h]: [f32; 4], px: f32, py: f32) -> bool {
    px >= x - 1.0 && px <= x + w + 1.0 && py >= y - 1.0 && py <= y + h + 1.0
}

/// The acceptance of the mask: a password and a pin drawn for real, and
/// marked secret by the projection, never cross; the mirror draws the box
/// in their places and is the app's frame elsewhere; a keystroke in the
/// password sends nothing.
#[test]
fn a_secret_drawn_never_crosses_and_the_mirror_draws_the_box() {
    let secret = "hunter2-correct-horse";
    let (w, h) = (support::login::WIDTH, support::login::HEIGHT);
    let (Some(mut app), Some(mut seen)) = (host(w, h), host(w, h)) else {
        return;
    };
    let clear = [0.0, 0.0, 0.0, 1.0];
    app.begin_frame();
    let now = support::login::form(app.compositor_mut(), secret);
    app.present(clear, "app").expect("the app drew");
    let drawn = app.read().expect("the app reads");

    // unmasked (the frame of phase 1), both would cross: the search finds
    let plain = Sender::new().frame(app.compositor(), head(w, h, clear));
    let (lines, parts) = wire_of(&watched().offer_scene(MAIN, &offer(plain, app.compositor())));
    assert!(carries(&lines, &parts, secret) && carries(&lines, &parts, support::login::PIN));

    let hidden = masks(Some(&now), app.compositor());
    let places: Vec<[f32; 4]> = hidden.iter().map(|m: &Mask| m.rect).collect();
    assert_eq!(
        places,
        [support::login::PASSWORD_AT, support::login::PIN_AT],
        "the two secrets"
    );
    let mut sender = Sender::new();
    let mut server = watched();
    let frame = sender.frame_masked(app.compositor(), head(w, h, clear), &hidden);
    let out = server.offer_scene(MAIN, &offer(frame, app.compositor()));
    let (lines, parts) = wire_of(&out);
    assert!(!carries(&lines, &parts, secret), "the password went out");
    assert!(
        !carries(&lines, &parts, support::login::PIN),
        "the pin went out"
    );
    assert!(
        carries(&lines, &parts, support::login::USER),
        "the clear field travels"
    );
    assert!(carries(&lines, &parts, SECRET_LABEL), "the box says why");

    // the mirror: the app's frame, but a box in each secret's place
    let mut viewer = Viewer::new();
    for o in out {
        viewer.take(o.msg.reply).expect("the frame reads");
    }
    seen.begin_frame();
    viewer.draw(seen.compositor_mut(), false);
    seen.present(clear, "mirror").expect("the mirror drew");
    let mirror = seen.read().expect("the mirror reads");
    let mut differ = [0usize; 2];
    for (i, (a, b)) in drawn.rgba.chunks(4).zip(mirror.rgba.chunks(4)).enumerate() {
        if a == b {
            continue;
        }
        let (px, py) = (
            (i as u32 % drawn.width) as f32,
            (i as u32 / drawn.width) as f32,
        );
        let at = places.iter().position(|r| inside(*r, px, py));
        let Some(at) = at else {
            panic!("the mirror differs at ({px}, {py}), out of every secret's place");
        };
        differ[at] += 1;
    }
    assert!(
        differ.iter().all(|n| *n > 200),
        "a box drawn in each place: {differ:?}"
    );

    // the person types: the secret and its caret move, nothing travels
    app.begin_frame();
    let now = support::login::form(app.compositor_mut(), "hunter2-correct-horse-battery");
    let hidden = masks(Some(&now), app.compositor());
    let frame = sender.frame_masked(app.compositor(), head(w, h, clear), &hidden);
    let out = server.offer_scene(MAIN, &offer(frame, app.compositor()));
    assert!(out.is_empty(), "a keystroke in a secret field sent {out:?}");
}

/// Every gallery page in both themes through the shell's masked path: the
/// mirror is the app's frame byte for byte where nothing is masked; where
/// something is, it differs only inside the masked places. What is masked
/// today: the Git page's three panes (changes, stacks, diff), bound by
/// the ide's workbench and not in the gallery's projection, so the rule
/// counts them as secret; the Field page's and the Input page's three
/// passwords each, and the Input OTP page's ten codes, secret by their own
/// constraint (task-104 front A).
#[test]
fn every_page_mirrors_through_the_mask_but_what_it_hides() {
    let (w, h) = (1280.0, 800.0);
    let (Some(mut app), Some(mut seen)) = (host(w, h), host(w, h)) else {
        return;
    };
    let mut view = ShowcaseView::new(w, h);
    view.resize(w, h, 1.0);
    let mut sender = Sender::new();
    let mut server = watched();
    let mut viewer = Viewer::new();
    let mut masked = Vec::new();
    for theme in ["core", "hoff"] {
        for p in PAGES {
            view.jump_to_page(p.title);
            view.apply_theme(theme);
            let mut dt = 0.0;
            for _ in 0..240 {
                if !view.tick(dt) {
                    break;
                }
                dt = 1.0 / 60.0;
            }
            let clear = view.theme.colors.bg.to_linear_array();
            app.begin_frame();
            view.render(app.compositor_mut());
            app.present(clear, "app").expect("the app drew");
            let drawn = app.read().expect("the app reads");
            let hidden = masks(view.gallery_semantics().ok().as_ref(), app.compositor());
            let frame = sender.frame_masked(app.compositor(), head(w, h, clear), &hidden);
            for out in server.offer_scene(MAIN, &offer(frame, app.compositor())) {
                viewer.take(out.msg.reply).expect("the frame reads");
            }
            seen.begin_frame();
            viewer.draw(seen.compositor_mut(), false);
            seen.present(clear, "mirror").expect("the mirror drew");
            let mirror = seen.read().expect("the mirror reads");
            if hidden.is_empty() {
                assert!(
                    mirror.rgba == drawn.rgba,
                    "{} / {theme}: nothing masked, and the pixels differ",
                    p.title
                );
                continue;
            }
            masked.push(format!("{} / {theme}: {}", p.title, hidden.len()));
            for (i, (a, b)) in drawn.rgba.chunks(4).zip(mirror.rgba.chunks(4)).enumerate() {
                let (px, py) = (
                    (i as u32 % drawn.width) as f32,
                    (i as u32 / drawn.width) as f32,
                );
                assert!(
                    a == b || hidden.iter().any(|m| inside(m.rect, px, py)),
                    "{} / {theme}: the mirror differs at ({px}, {py}), out of every masked place",
                    p.title
                );
            }
        }
    }
    assert_eq!(
        masked,
        [
            "Field / core: 3",
            "Input / core: 3",
            "Input OTP / core: 10",
            "Git / core: 3",
            "Field / hoff: 3",
            "Input / hoff: 3",
            "Input OTP / hoff: 10",
            "Git / hoff: 3"
        ]
    );
}
