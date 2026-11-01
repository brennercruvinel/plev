//! the scene codec on hand-built compositors: every node kind back as it
//! was, a delta with one dirty layer, an asset built once, the patch, the
//! refusals. the gallery and the socket are in apps/uigallery/tests/scene_codec.rs
//! and apps/uigallery/tests/mirror.rs.

#[cfg(test)]
mod mask;

use engine::drawing::compositor::{
    Compositor, LayerEffect, LayerId, QuadVertex, SceneNode, TextNodeKey,
};
use engine::drawing::gpu::load_image_rgba;
use engine::drawing::path::TessellatedPath;

use super::*;

const HEAD: Head = Head {
    width: 640.0,
    height: 480.0,
    scale: 2.0,
    clear: [0.1, 0.2, 0.3, 1.0],
};

fn text(s: &str) -> TextNodeKey {
    TextNodeKey {
        text: s.to_owned(),
        font_size_bits: 13.5f32.to_bits(),
        line_height_bits: 1.4f32.to_bits(),
        max_width_bits: Some(200.25f32.to_bits()),
        font_weight: 600,
        letter_spacing_bits: (-0.25f32).to_bits(),
        font_family: Some("Inter".to_owned()),
        features: engine::drawing::text::TextFeatures::NONE,
    }
}

fn path(seed: f32) -> TessellatedPath {
    TessellatedPath {
        vertices: (0..4)
            .map(|i| QuadVertex {
                position: [seed + i as f32, seed * 2.0 - i as f32],
                color: [0.5, 0.25, seed, 1.0],
            })
            .collect(),
        indices: vec![0, 1, 2, 0, 2, 3],
        hash: seed.to_bits() as u64 * 7919,
    }
}

/// every kind of node, a text with every field set, an image, a path.
fn every_node() -> Vec<SceneNode> {
    let image = load_image_rgba(3, 2, [9u8, 8, 7, 255].repeat(6)).expect("an image packs");
    vec![
        SceneNode::Rect {
            x: 1.0,
            y: 2.0,
            w: 3.0,
            h: 4.0,
            color: [0.1, 0.2, 0.3, 0.4],
        },
        SceneNode::PushClip {
            x: 0.5,
            y: 0.5,
            w: 100.0,
            h: 50.0,
        },
        SceneNode::RoundedRect {
            x: -1.5,
            y: 2.0,
            w: 30.0,
            h: 40.0,
            color: [1.0, 0.0, 0.0, 1.0],
            corner_radius: 6.0,
            border_width: 1.0,
            border_color: [0.0, 0.0, 0.0, 0.5],
        },
        SceneNode::GradientRect {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
            color: [0.0, 1.0, 0.0, 1.0],
            color2: [0.0, 0.0, 1.0, 1.0],
            angle_deg: 135.0,
            corner_radius: 2.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        },
        SceneNode::Text {
            key: text("olá, mundo"),
            x: 12.0,
            y: 13.0,
            color: [0.9, 0.9, 0.9, 1.0],
        },
        SceneNode::Text {
            key: TextNodeKey {
                max_width_bits: None,
                font_family: None,
                ..text("")
            },
            x: 0.0,
            y: 0.0,
            color: [1.0; 4],
        },
        SceneNode::Path { data: path(3.0) },
        SceneNode::Image {
            x: 5.0,
            y: 6.0,
            w: 30.0,
            h: 20.0,
            image,
            corner_radius: 4.0,
        },
        SceneNode::BackdropBlur {
            x: 10.0,
            y: 10.0,
            w: 200.0,
            h: 100.0,
            corner_radius: 12.0,
            sigma: 18.0,
        },
        SceneNode::Shadow {
            x: 1.0,
            y: 1.0,
            w: 2.0,
            h: 3.0,
            corner_radius: 4.0,
            blur_radius: 16.0,
            offset: [2.0, 4.0],
            color: [0.97, 0.97, 0.97, 0.06],
            inset: true,
        },
        SceneNode::PopClip,
        // a negative zero and a NaN travel as their bits
        SceneNode::Rect {
            x: -0.0,
            y: f32::NAN,
            w: f32::INFINITY,
            h: 1e-38,
            color: [0.0; 4],
        },
    ]
}

/// a compositor of three layers: the default one, a glass layer above
/// with effects and a clip, and a hidden one below.
fn app() -> Compositor {
    let mut c = Compositor::new();
    let glass = c.create_layer(10);
    let below = c.create_layer(-5);
    c.set_layer_opacity(glass, 0.75);
    c.set_layer_effects(
        glass,
        vec![
            LayerEffect::Blur { sigma: 3.0 },
            LayerEffect::Shadow {
                sigma: 2.0,
                color: [0.0, 0.0, 0.0, 0.5],
            },
        ],
    );
    c.set_layer_clip_rect(glass, Some((1, 2, 300, 400)));
    c.set_layer_visible(below, false);
    c.begin_frame();
    for n in every_node() {
        c.push(n);
    }
    c.push_to_layer(
        glass,
        SceneNode::Text {
            key: text("glass"),
            x: 1.0,
            y: 1.0,
            color: [1.0; 4],
        },
    );
    c.push_to_layer(below, SceneNode::Path { data: path(3.0) });
    c.resolve_scene((640.0, 480.0));
    c
}

/// what the layers of `c` hold, in composite order, with their
/// properties, compared by bits (a NaN equals itself).
fn scene(c: &Compositor) -> Vec<String> {
    c.layers()
        .iter()
        .filter(|l| !l.nodes().is_empty())
        .map(|l| {
            format!(
                "z {} visible {} opacity {} clip {:?} effects {:?} nodes {:?}",
                l.z_order,
                l.visible,
                l.opacity,
                l.clip_rect,
                l.effects(),
                l.nodes()
            )
        })
        .collect()
}

/// a frame as a receiver gets it: every asset, then every layer whole.
fn whole(m: &mut Mirror, f: &Frame) -> Result<(), SceneError> {
    for (key, bytes) in &f.assets {
        m.asset(*key, bytes)?;
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
    m.frame(Some(&f.head), &layers)
}

/// a frame as a receiver that holds the frame before gets it: the
/// patch when there is one, nothing for a layer that did not change.
fn delta(m: &mut Mirror, f: &Frame, held: &[(u32, u64, usize)]) -> Result<usize, SceneError> {
    let mut sent = 0;
    let layers: Vec<Incoming<'_>> = f
        .layers
        .iter()
        .map(|l| {
            let had = held
                .iter()
                .find(|(k, _, _)| *k == l.key)
                .map(|(_, h, _)| *h);
            let brought = match (&l.patch, had) {
                (_, Some(h)) if h == l.hash => Brought::Held,
                (Some((base, patch)), Some(h)) if *base == h => {
                    sent += patch.len();
                    Brought::Patch(patch)
                }
                _ => {
                    sent += l.body.len();
                    Brought::Body(&l.body)
                }
            };
            Incoming {
                key: l.key,
                hash: l.hash,
                brought,
            }
        })
        .collect();
    m.frame(None, &layers)?;
    Ok(sent)
}

#[test]
fn a_whole_frame_comes_back_node_for_node() {
    let a = app();
    let f = Sender::new().frame(&a, HEAD);
    let mut m = Mirror::new();
    whole(&mut m, &f).unwrap();
    assert_eq!(m.head(), Some(HEAD));
    let mut b = Compositor::new();
    m.draw(&mut b);
    b.resolve_scene((640.0, 480.0));
    assert_eq!(scene(&b), scene(&a));
    // the same image content is the same handle in one store: the load
    // on the receiver's side was answered by the dedupe
    assert_eq!(m.image_counts().loads, 1);
    assert_eq!(m.image_counts().live, 0);
}

#[test]
fn a_text_comes_back_with_every_field_of_its_key() {
    let a = app();
    let f = Sender::new().frame(&a, HEAD);
    let mut m = Mirror::new();
    whole(&mut m, &f).unwrap();
    let mut b = Compositor::new();
    m.draw(&mut b);
    let keys = |c: &Compositor| -> Vec<TextNodeKey> {
        c.layers()
            .iter()
            .flat_map(|l| l.nodes())
            .filter_map(|n| match n {
                SceneNode::Text { key, .. } => Some(key.clone()),
                _ => None,
            })
            .collect()
    };
    assert_eq!(keys(&b), keys(&a));
    assert_eq!(keys(&a).len(), 3);
}

#[test]
fn one_dirty_layer_travels_as_a_patch_and_the_others_not_at_all() {
    let mut a = app();
    let mut sender = Sender::new();
    let first = sender.frame(&a, HEAD);
    let mut m = Mirror::new();
    whole(&mut m, &first).unwrap();
    let held = m.layers();

    // one node of the default layer changes color
    let mut nodes: Vec<SceneNode> = a.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    if let SceneNode::RoundedRect { color, .. } = &mut nodes[2] {
        *color = [0.0, 1.0, 1.0, 1.0];
    }
    let glass: Vec<SceneNode> = a.layers()[2].nodes().to_vec();
    let glass_id = a.layers()[2].id;
    let below_id = a.layers()[0].id;
    let below: Vec<SceneNode> = a.layers()[0].nodes().to_vec();
    a.begin_frame();
    for n in nodes {
        a.push(n);
    }
    for n in glass {
        a.push_to_layer(glass_id, n);
    }
    for n in below {
        a.push_to_layer(below_id, n);
    }
    a.resolve_scene((640.0, 480.0));

    let second = sender.frame(&a, HEAD);
    let moved: Vec<u32> = second
        .layers
        .iter()
        .zip(&first.layers)
        .filter(|(n, b)| n.hash != b.hash)
        .map(|(n, _)| n.key)
        .collect();
    assert_eq!(moved, [LayerId::DEFAULT.0], "one layer is dirty");
    let dirty = &second.layers.iter().find(|l| l.key == 0).unwrap();
    let (base, patch) = dirty
        .patch
        .as_ref()
        .expect("a patch against the frame before");
    assert_eq!(
        *base,
        first.layers.iter().find(|l| l.key == 0).unwrap().hash
    );
    assert!(
        patch.len() * 4 < dirty.body.len(),
        "{} vs {}",
        patch.len(),
        dirty.body.len()
    );

    let sent = delta(&mut m, &second, &held).unwrap();
    assert_eq!(
        sent,
        patch.len(),
        "only the patch of the dirty layer travels"
    );
    let mut b = Compositor::new();
    m.draw(&mut b);
    b.resolve_scene((640.0, 480.0));
    assert_eq!(scene(&b), scene(&a));
}

#[test]
fn an_asset_is_built_once_while_frames_reference_it() {
    let a = app();
    let mut sender = Sender::new();
    let one = sender.frame(&a, HEAD);
    let two = sender.frame(&a, HEAD);
    assert_eq!(
        one.assets.len(),
        2,
        "one image and one path, the path drawn twice"
    );
    for ((k1, b1), (k2, b2)) in one.assets.iter().zip(&two.assets) {
        assert_eq!(k1, k2);
        assert!(std::sync::Arc::ptr_eq(b1, b2), "built once, then kept");
    }
    // a frame with no image lets it go; the next that draws it builds it
    // again
    let empty = Compositor::new();
    assert!(sender.frame(&empty, HEAD).assets.is_empty());
    let three = sender.frame(&a, HEAD);
    assert!(!std::sync::Arc::ptr_eq(
        &one.assets[0].1,
        &three.assets[0].1
    ));
    assert_eq!(one.assets[0].1, three.assets[0].1);
}

#[test]
fn the_receiver_lets_go_what_no_layer_draws() {
    let a = app();
    let mut sender = Sender::new();
    let mut m = Mirror::new();
    whole(&mut m, &sender.frame(&a, HEAD)).unwrap();
    assert_eq!(m.assets(), (1, 1));
    let mut plain = Compositor::new();
    plain.begin_frame();
    plain.push(SceneNode::PopClip);
    whole(&mut m, &sender.frame(&plain, HEAD)).unwrap();
    assert_eq!(m.assets(), (0, 0));
}

#[test]
fn a_frame_that_names_what_the_receiver_does_not_hold_is_refused() {
    let a = app();
    let f = Sender::new().frame(&a, HEAD);
    let mut m = Mirror::new();
    let held: Vec<Incoming<'_>> = f
        .layers
        .iter()
        .map(|l| Incoming {
            key: l.key,
            hash: l.hash,
            brought: Brought::Held,
        })
        .collect();
    assert!(matches!(m.frame(None, &held), Err(SceneError::NotHeld(_))));
    let wrong = [Incoming {
        key: f.layers[0].key,
        hash: f.layers[0].hash ^ 1,
        brought: Brought::Body(&f.layers[0].body),
    }];
    assert_eq!(
        m.frame(None, &wrong),
        Err(SceneError::Hash(f.layers[0].key))
    );
}

#[test]
fn a_cut_or_broken_body_is_an_error_never_a_panic() {
    let a = app();
    let f = Sender::new().frame(&a, HEAD);
    for l in &f.layers {
        for end in 0..l.body.len() {
            let cut = &l.body[..end];
            let r = Mirror::new().frame(
                None,
                &[Incoming {
                    key: l.key,
                    hash: super::bytes::fnv(&[cut]),
                    brought: Brought::Body(cut),
                }],
            );
            assert!(r.is_err(), "a body cut at {end} of {} read", l.body.len());
        }
        let mut flipped = l.body.to_vec();
        for i in 0..flipped.len() {
            flipped[i] ^= 0xff;
            let _ = Mirror::new().frame(
                None,
                &[Incoming {
                    key: l.key,
                    hash: super::bytes::fnv(&[&flipped]),
                    brought: Brought::Body(&flipped),
                }],
            );
            flipped[i] ^= 0xff;
        }
    }
    for (_, bytes) in &f.assets {
        for end in 0..bytes.len().min(200) {
            assert!(Mirror::new().asset(1, &bytes[..end]).is_err());
        }
    }
}

/// a camera's frames: one stream slot in the app's store, a new handle
/// per frame. none of their pixels travels: the stream goes once, as its
/// size and place, under a key its frames share, so a new frame moves no
/// layer; the receiver draws the box that says it is not mirrored, and
/// loads nothing.
#[test]
fn a_stream_travels_without_its_pixels() {
    use engine::drawing::gpu::{replace_image_rgba, stream_image_rgba};
    let slot = stream_image_rgba(64, 32).expect("a slot");
    let mut shown = slot;
    let mut sender = Sender::new();
    let mut m = Mirror::new();
    let mut hashes = Vec::new();
    let mut sent: Vec<u8> = Vec::new();
    for n in 0..5u8 {
        // a frame no other bytes of the scene look like
        let pixel = [0xa0 + n, 0x0b, 0xc0 + n, 0xfe];
        shown = replace_image_rgba(&shown, &pixel.repeat(64 * 32)).expect("the frame replaces");
        let mut c = Compositor::new();
        c.begin_frame();
        c.push(SceneNode::Image {
            x: 10.0,
            y: 20.0,
            w: 320.0,
            h: 160.0,
            image: shown,
            corner_radius: 6.0,
        });
        let f = sender.frame(&c, HEAD);
        for (_, bytes) in &f.assets {
            sent.extend_from_slice(bytes);
        }
        for l in &f.layers {
            sent.extend_from_slice(&l.body);
        }
        hashes.push(f.layers[0].hash);
        whole(&mut m, &f).unwrap();
        let run = pixel.repeat(4);
        assert!(
            !sent.windows(run.len()).any(|w| w == run),
            "frame {n}: the camera's pixels travel"
        );
    }
    assert!(
        hashes.windows(2).all(|w| w[0] == w[1]),
        "no frame moved the layer"
    );
    let mut b = Compositor::new();
    m.draw(&mut b);
    let nodes = b.layer(LayerId::DEFAULT).unwrap().nodes();
    assert!(matches!(
        nodes[0],
        SceneNode::RoundedRect {
            x: 10.0,
            y: 20.0,
            w: 320.0,
            h: 160.0,
            ..
        }
    ));
    assert!(
        nodes
            .iter()
            .any(|n| matches!(n, SceneNode::Text { key, .. } if key.text == LIVE_LABEL))
    );
    assert!(!nodes.iter().any(|n| matches!(n, SceneNode::Image { .. })));
    let counts = m.image_counts();
    assert_eq!((counts.loads, counts.live, counts.missing), (0, 5, 0));
    assert_eq!(m.assets(), (1, 0), "held as long as it is drawn");
}
