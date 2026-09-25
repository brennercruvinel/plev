//! the mask (task-79 phase 2) on a hand-built login form: a password
//! field drawn as a field draws itself (its ground and edge, the secret's
//! text scrolled inside a clip, the caret, an icon and a mark in it),
//! a label beside it and a word below it. the frame written with the
//! field's place masked holds no byte of what the secret draws, and the
//! same bytes whatever the secret is; a receiver draws the box there.

use engine::compositor::{Compositor, LayerId, QuadVertex, SceneNode, TextNodeKey};
use engine::gpu::load_image_rgba;
use engine::path::TessellatedPath;

use super::*;

const HEAD: Head = Head {
    width: 640.0,
    height: 480.0,
    scale: 1.0,
    clear: [0.0, 0.0, 0.0, 1.0],
};

/// the password field's place: where it is bound.
const FIELD: [f32; 4] = [100.0, 40.0, 160.0, 32.0];

/// the icon's pixels: a run no other byte of the frame makes.
const ICON: [u8; 4] = [0xAB, 0xCD, 0xEF, 0x77];

fn key(s: &str) -> TextNodeKey {
    TextNodeKey {
        text: s.to_owned(),
        font_size_bits: 14.0f32.to_bits(),
        line_height_bits: 20.0f32.to_bits(),
        max_width_bits: None,
        font_weight: 400,
        letter_spacing_bits: 0.0f32.to_bits(),
        font_family: None,
    }
}

fn text(s: &str, x: f32, y: f32) -> SceneNode {
    SceneNode::Text {
        key: key(s),
        x,
        y,
        color: [0.9, 0.9, 0.9, 1.0],
    }
}

/// the form with `secret` in the field, scrolled so its text starts
/// left of the field (its origin out of the place) and a line of it above
/// the field, out of sight in the field's clip; the caret `caret` pixels
/// into the field.
fn form(secret: &str, caret: f32) -> Compositor {
    let [fx, fy, fw, fh] = FIELD;
    let mut c = Compositor::new();
    c.begin_frame();
    c.push(SceneNode::Rect {
        x: 0.0,
        y: 0.0,
        w: 640.0,
        h: 480.0,
        color: [0.1, 0.1, 0.12, 1.0],
    });
    c.push(text("password", 20.0, 46.0));
    c.push(SceneNode::RoundedRect {
        x: fx,
        y: fy,
        w: fw,
        h: fh,
        color: [0.2, 0.2, 0.22, 1.0],
        corner_radius: 6.0,
        border_width: 1.0,
        border_color: [0.5, 0.5, 0.5, 1.0],
    });
    c.push(SceneNode::PushClip {
        x: fx + 4.0,
        y: fy,
        w: fw - 8.0,
        h: fh,
    });
    c.push(text(&format!("{secret} above"), fx + 4.0, fy - 24.0));
    c.push(text(secret, fx - 60.0, fy + 6.0));
    c.push(SceneNode::Rect {
        x: fx + 4.0 + caret,
        y: fy + 6.0,
        w: 1.5,
        h: 20.0,
        color: [1.0; 4],
    });
    c.push(SceneNode::PopClip);
    let icon = load_image_rgba(4, 4, ICON.repeat(16)).expect("an icon packs");
    c.push(SceneNode::Image {
        x: fx + fw - 24.0,
        y: fy + 8.0,
        w: 16.0,
        h: 16.0,
        image: icon,
        corner_radius: 0.0,
    });
    c.push(SceneNode::Path {
        data: TessellatedPath {
            vertices: [
                [fx + 10.0, fy + 28.0],
                [fx + 60.0, fy + 28.0],
                [fx + 60.0, fy + 30.0],
            ]
            .into_iter()
            .map(|position| QuadVertex {
                position,
                color: [1.0, 0.0, 0.0, 1.0],
            })
            .collect(),
            indices: vec![0, 1, 2],
            hash: 4242,
        },
    });
    c.push(text("sign in", fx, fy + fh + 12.0));
    c.resolve_scene((640.0, 480.0));
    c
}

fn masked() -> [Mask; 1] {
    [Mask {
        layer: LayerId::DEFAULT.0,
        rect: FIELD,
    }]
}

/// every byte the frame carries: the head, each body and patch, each
/// asset.
fn bytes(f: &Frame) -> Vec<u8> {
    let mut all = f.head.to_vec();
    for l in &f.layers {
        all.extend_from_slice(&l.body);
        if let Some((_, patch)) = &l.patch {
            all.extend_from_slice(patch);
        }
    }
    for (_, a) in &f.assets {
        all.extend_from_slice(a);
    }
    all
}

fn holds(all: &[u8], part: &[u8]) -> bool {
    all.windows(part.len()).any(|w| w == part)
}

/// the texts a compositor draws, in order.
fn texts(c: &Compositor) -> Vec<String> {
    c.layers()
        .iter()
        .flat_map(|l| l.nodes())
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn what_a_secret_draws_is_in_no_byte_of_the_frame() {
    let secret = "hunter2-correct-horse";
    let c = form(secret, 30.0);
    let plain = Sender::new().frame(&c, HEAD);
    assert!(
        holds(&bytes(&plain), secret.as_bytes()),
        "unmasked, it would"
    );

    let f = Sender::new().frame_masked(&c, HEAD, &masked());
    let all = bytes(&f);
    assert!(!holds(&all, secret.as_bytes()), "the secret's text");
    assert!(!holds(&all, &ICON.repeat(4)), "the icon's pixels");
    assert!(
        f.assets.is_empty(),
        "no asset of what it draws: {:?}",
        f.assets.len()
    );
    assert!(holds(&all, b"password") && holds(&all, b"sign in"));

    // the receiver draws the box in the field's place, over its ground
    let mut m = Mirror::new();
    for (k, a) in &f.assets {
        m.asset(*k, a).expect("an asset");
    }
    let layers: Vec<Incoming<'_>> = f
        .layers
        .iter()
        .map(|l| Incoming {
            key: l.key,
            hash: l.hash,
            brought: Brought::Body(&l.body),
        })
        .collect();
    m.frame(Some(&f.head), &layers).expect("the frame reads");
    let mut drawn = Compositor::new();
    m.draw(&mut drawn);
    assert_eq!(texts(&drawn), ["password", SECRET_LABEL, "sign in"]);
    let nodes = drawn.layers()[0].nodes();
    let at = |n: &SceneNode| nodes.iter().position(|m| m == n);
    let ground = at(&c.layers()[0].nodes()[2]).expect("the field's ground travels");
    let [x, y, w, h] = FIELD;
    let boxed = nodes
        .iter()
        .position(|n| {
            matches!(n, SceneNode::RoundedRect { x: bx, y: by, w: bw, h: bh, .. }
            if [*bx, *by, *bw, *bh] == [x, y, w, h] && Some(n) != nodes.get(ground))
        })
        .expect("the box in the field's place");
    assert!(ground < boxed, "the box over the ground");
    let clips = nodes
        .iter()
        .filter(|n| matches!(n, SceneNode::PushClip { .. }))
        .count();
    assert_eq!(clips, 2, "the field's clip and the box's");
}

/// the person types in the field: the secret and the caret move, and the
/// masked layer is the same bytes, so nothing of it travels.
#[test]
fn the_secret_changing_moves_no_layer() {
    let mut s = Sender::new();
    let first = s.frame_masked(&form("a", 8.0), HEAD, &masked());
    let typed = s.frame_masked(&form("ab-and-then-some", 90.0), HEAD, &masked());
    let hashes = |f: &Frame| f.layers.iter().map(|l| l.hash).collect::<Vec<_>>();
    assert_eq!(hashes(&first), hashes(&typed));
    assert!(typed.layers.iter().all(|l| l.patch.is_none()));

    let mut plain = Sender::new();
    let a = plain.frame(&form("a", 8.0), HEAD);
    let b = plain.frame(&form("ab-and-then-some", 90.0), HEAD);
    assert_ne!(hashes(&a), hashes(&b), "unmasked, it moves");
}

/// with no mask, and with a mask on another layer, a layer is written as
/// it always was.
#[test]
fn a_layer_without_a_mask_is_written_as_before() {
    let c = form("secret", 10.0);
    let plain = Sender::new().frame(&c, HEAD);
    let none = Sender::new().frame_masked(&c, HEAD, &[]);
    let elsewhere = Sender::new().frame_masked(
        &c,
        HEAD,
        &[Mask {
            layer: 77,
            rect: FIELD,
        }],
    );
    for f in [&none, &elsewhere] {
        assert_eq!(bytes(f), bytes(&plain));
    }
}
