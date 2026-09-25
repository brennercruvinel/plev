//! A secret drawn, for the mirror's mask (task-79 phase 2). No comp draws
//! a secret today (no field of the comps has a password mode), so the
//! tests draw one as a field draws itself, with no new widget: a login
//! form on the gallery's ground, a clear user field, a password field
//! (its ground and edge, the secret's text inside its clip, the caret), a
//! pin said by a status (a live region), and the submit button, each
//! bound to the id the projection gives it; the projection marks the
//! password and the pin secret, as a page would.

use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use sem::{Constraints, Role, Snapshot, SnapshotBuilder};

/// What the user field shows: it travels.
pub const USER: &str = "brenner-at-the-desk";
/// The pin the status says: secret.
pub const PIN: &str = "pin 4711-0815";

pub const WIDTH: f32 = 640.0;
pub const HEIGHT: f32 = 400.0;

/// Where each control is bound: `(x, y, w, h)`.
pub const USER_AT: [f32; 4] = [120.0, 80.0, 280.0, 36.0];
pub const PASSWORD_AT: [f32; 4] = [120.0, 140.0, 280.0, 36.0];
pub const PIN_AT: [f32; 4] = [120.0, 196.0, 280.0, 24.0];
pub const SUBMIT_AT: [f32; 4] = [120.0, 240.0, 120.0, 36.0];

fn key(text: &str, size: f32) -> TextNodeKey {
    TextNodeKey {
        text: text.to_owned(),
        font_size_bits: size.to_bits(),
        line_height_bits: (size * 1.4).to_bits(),
        max_width_bits: None,
        font_weight: 400,
        letter_spacing_bits: 0.0f32.to_bits(),
        font_family: None,
    }
}

fn text(c: &mut Compositor, s: &str, x: f32, y: f32) {
    c.push(SceneNode::Text {
        key: key(s, 14.0),
        x,
        y,
        color: [0.92, 0.92, 0.94, 1.0],
    });
}

/// A field as a field draws itself: its ground and edge, then `shown`
/// inside its clip, and the caret after `caret` pixels when focused.
fn field(c: &mut Compositor, [x, y, w, h]: [f32; 4], shown: &str, caret: Option<f32>) {
    c.push(SceneNode::RoundedRect {
        x,
        y,
        w,
        h,
        color: [0.16, 0.16, 0.19, 1.0],
        corner_radius: 6.0,
        border_width: 1.0,
        border_color: [0.45, 0.45, 0.5, 1.0],
    });
    c.push(SceneNode::PushClip {
        x: x + 6.0,
        y,
        w: w - 12.0,
        h,
    });
    text(c, shown, x + 10.0, y + 8.0);
    if let Some(at) = caret {
        c.push(SceneNode::Rect {
            x: x + 10.0 + at,
            y: y + 8.0,
            w: 1.5,
            h: 20.0,
            color: [1.0; 4],
        });
    }
    c.push(SceneNode::PopClip);
}

/// The form with `secret` in the password field (the caret after it),
/// drawn and bound into `c` as a frame of its own; the projection that
/// marks what is secret.
pub fn form(c: &mut Compositor, secret: &str) -> Snapshot {
    c.begin_frame();
    c.push(SceneNode::Rect {
        x: 0.0,
        y: 0.0,
        w: WIDTH,
        h: HEIGHT,
        color: [0.08, 0.08, 0.1, 1.0],
    });
    text(c, "user", 40.0, 88.0);
    field(c, USER_AT, USER, None);
    c.bind(&"login.user".parse().expect("an id"), USER_AT);
    text(c, "password", 40.0, 148.0);
    let caret = 7.5 * secret.chars().count() as f32;
    field(c, PASSWORD_AT, secret, Some(caret));
    c.bind(&"login.password".parse().expect("an id"), PASSWORD_AT);
    let [px, py, _, _] = PIN_AT;
    text(c, PIN, px + 2.0, py + 2.0);
    c.bind(&"login.pin".parse().expect("an id"), PIN_AT);
    let [bx, by, bw, bh] = SUBMIT_AT;
    c.push(SceneNode::RoundedRect {
        x: bx,
        y: by,
        w: bw,
        h: bh,
        color: [0.25, 0.4, 0.9, 1.0],
        corner_radius: 8.0,
        border_width: 0.0,
        border_color: [0.0; 4],
    });
    text(c, "sign in", bx + 30.0, by + 8.0);
    c.bind(&"login.submit".parse().expect("an id"), SUBMIT_AT);
    c.resolve_scene((WIDTH, HEIGHT));

    let secret_field = Constraints {
        secret: true,
        editable: true,
        ..Constraints::default()
    };
    let mut b = SnapshotBuilder::new();
    b.node("login", Role::Region, "login");
    b.node("login.user", Role::TextInput, "user")
        .parent("login")
        .state("text", USER)
        .action("set_value")
        .param(sem::ParamSpec {
            kind: sem::ValueKind::Text,
            constrained: false,
        });
    b.node("login.password", Role::TextInput, "password")
        .parent("login")
        .state("text", secret)
        .constraints(secret_field)
        .action("set_value")
        .param(sem::ParamSpec {
            kind: sem::ValueKind::Text,
            constrained: false,
        });
    b.node("login.pin", Role::Status, "pin")
        .parent("login")
        .state("message", PIN)
        .constraints(Constraints {
            secret: true,
            ..Constraints::default()
        });
    b.node("login.submit", Role::Button, "sign in")
        .parent("login")
        .action("submit")
        .reads("login.user:text")
        .reads("login.password:text");
    b.finish().expect("the form projects")
}
