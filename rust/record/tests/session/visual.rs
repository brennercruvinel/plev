//! the visual track: every frame reads back at its time, the screen holds
//! the last frame between two, what the IR cannot carry is counted, assets
//! enter once and leave with the last chunk that uses them, the window
//! opens at a visual keyframe, and one byte budget covers both tracks.

use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use motion::{AssetBank, NodeKind, Prop, RaiseError, Value, quantize_value, raise_scene};
use record::{FrameError, InputKind, Limits, Recorder};

use crate::toys::look;

type Fallible = Result<(), Box<dyn std::error::Error>>;

fn label(text: &str) -> TextNodeKey {
    TextNodeKey::new(text, 14.0, 20.0, None)
}

/// a todo-like frame: a card, a row per item sliding in from `slide`, a
/// counter text.
fn frame(items: &[&str], slide: f32, hover: Option<usize>) -> Vec<SceneNode> {
    let mut scene = vec![SceneNode::RoundedRect {
        x: 20.0,
        y: 20.0,
        w: 360.0,
        h: 400.0,
        color: [0.1, 0.1, 0.12, 1.0],
        corner_radius: 12.0,
        border_width: 1.0,
        border_color: [1.0, 1.0, 1.0, 0.1],
    }];
    for (i, item) in items.iter().enumerate() {
        let y = 60.0 + 40.0 * i as f32;
        let lit = hover == Some(i);
        scene.push(SceneNode::Rect {
            x: 30.0 + slide,
            y,
            w: 340.0,
            h: 36.0,
            color: if lit {
                [0.3, 0.3, 0.35, 1.0]
            } else {
                [0.15, 0.15, 0.18, 1.0]
            },
        });
        scene.push(SceneNode::Text {
            key: label(item),
            x: 44.0 + slide,
            y: y + 8.0,
            color: [0.9, 0.9, 0.9, 1.0],
        });
    }
    scene.push(SceneNode::Text {
        key: label(&format!("{} active", items.len())),
        x: 30.0,
        y: 380.0,
        color: [0.6, 0.6, 0.6, 1.0],
    });
    scene
}

/// one raised node on the wire grid: depth, kind, quantized props.
type GridNode = (u16, NodeKind, Vec<(Prop, Value)>);

/// a scene as the wire carries it: raised, every prop quantized.
fn on_grid(scene: &[SceneNode], bank: &mut AssetBank) -> Vec<GridNode> {
    let Ok(raised) = raise_scene(scene, bank) else {
        panic!("the scene does not raise");
    };
    raised
        .nodes
        .into_iter()
        .map(|n| {
            let props = n
                .props
                .iter()
                .map(|(p, v)| (*p, quantize_value(*p, *v)))
                .collect();
            (n.depth, n.kind, props)
        })
        .collect()
}

#[test]
fn every_frame_reads_back_at_its_time_and_holds_until_the_next() -> Fallible {
    let rec = Recorder::new(Limits {
        chunk_every: 300,
        ..Limits::default()
    });
    // render on demand: a burst of animation, a pause, a hover, an add.
    let mut taken: Vec<(u64, Vec<SceneNode>)> = Vec::new();
    for i in 0..12u64 {
        let slide = 40.0 - 40.0 * (i as f32 / 11.0);
        taken.push((100 + 16 * i, frame(&["milk", "mail"], slide, None)));
    }
    taken.push((900, frame(&["milk", "mail"], 0.0, Some(1))));
    taken.push((1_450, frame(&["milk", "mail", "bread"], 0.0, Some(1))));
    taken.push((1_451, frame(&["milk", "mail", "bread"], 0.0, None)));
    for (t, scene) in &taken {
        rec.frame_scene(*t, scene)?;
    }
    let session = rec.snapshot_window();
    assert_eq!(
        session.frame_times(),
        taken.iter().map(|(t, _)| *t).collect::<Vec<_>>()
    );
    assert_eq!(session.frames_span(), Some((100, 1_451)));
    assert!(session.frame_keys().len() > 1, "the cadence cut chunks");
    assert!(session.scene_at(99)?.is_none(), "before the first frame");

    let mut bank = AssetBank::new();
    for (i, (t, scene)) in taken.iter().enumerate() {
        let want = on_grid(scene, &mut bank);
        let until = taken.get(i + 1).map_or(*t + 500, |(next, _)| *next);
        for at in [*t, (*t + until) / 2, until - 1] {
            let shown = session.scene_at(at)?.ok_or("no frame")?;
            assert_eq!(
                on_grid(&shown, &mut bank),
                want,
                "at {at}, the frame of {t}"
            );
            assert_eq!(session.frame_at(at), Some(*t));
        }
    }
    let stats = rec.stats();
    assert_eq!(stats.frames, taken.len() as u64);
    assert_eq!(stats.frame_errors, 0);
    Ok(())
}

#[test]
fn what_the_ir_cannot_carry_is_counted_and_the_rest_replays() -> Fallible {
    let rec = Recorder::default();
    let mut scene = frame(&["milk"], 0.0, None);
    scene.insert(
        1,
        SceneNode::PushClip {
            x: 20.0,
            y: 20.0,
            w: 360.0,
            h: 400.0,
        },
    );
    scene.push(SceneNode::PopClip);
    scene.push(SceneNode::Shadow {
        x: 0.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
        corner_radius: 2.0,
        blur_radius: 4.0,
        offset: [0.0, 2.0],
        color: [0.0, 0.0, 0.0, 0.5],
        inset: false,
    });
    rec.frame_scene(0, &scene)?;
    rec.frame_scene(10, &scene)?;
    let stats = rec.stats();
    assert_eq!(
        stats.skipped,
        [2, 2, 0, 2],
        "clip push, clip pop, blur, shadow"
    );
    assert_eq!(
        stats.frames_unchanged, 1,
        "the second frame is the first again"
    );

    let shown = rec.snapshot_window().scene_at(5)?.ok_or("no frame")?;
    let mut bank = AssetBank::new();
    assert_eq!(
        on_grid(&shown, &mut bank),
        on_grid(&frame(&["milk"], 0.0, None), &mut bank)
    );
    Ok(())
}

#[test]
fn an_asset_enters_once_and_leaves_with_the_last_chunk_using_it() -> Fallible {
    let rec = Recorder::new(Limits {
        max_age: 1_000,
        chunk_every: 200,
        ..Limits::default()
    });
    let long = "a label long enough to weigh more than the others ".repeat(8);
    for t in (0..600).step_by(50) {
        rec.frame_scene(t, &frame(&[long.as_str()], (t % 100) as f32, None))?;
    }
    let with_long = rec.stats().asset_bytes;
    for t in (650..3_000).step_by(50) {
        rec.frame_scene(t, &frame(&["short"], (t % 100) as f32, None))?;
    }
    let stats = rec.stats();
    assert!(stats.evicted_chunks > 0);
    assert!(
        stats.asset_bytes < with_long / 2,
        "the long label left with its chunks: {} then {}",
        with_long,
        stats.asset_bytes
    );
    let session = rec.snapshot_window();
    for size in session.chunk_sizes() {
        assert_eq!(size.1, 2, "one label and one counter per chunk");
    }
    Ok(())
}

#[test]
fn the_visual_window_opens_at_a_chunk_and_keeps_max_age() -> Fallible {
    let limits = Limits {
        max_age: 2_000,
        chunk_every: 500,
        ..Limits::default()
    };
    let rec = Recorder::new(limits);
    let step = 16;
    for now in (0..10_000u64).step_by(step) {
        let slide = (now % 1_000) as f32 / 10.0;
        rec.frame_scene(now, &frame(&["milk", "mail"], slide, None))?;
        let session = rec.snapshot_window();
        let (start, end) = session.frames_span().ok_or("no frames")?;
        assert_eq!(end, now);
        assert_eq!(session.frame_keys().first(), Some(&start));
        assert!(start <= now.saturating_sub(limits.max_age), "t={now}");
        let most = limits.max_age + limits.chunk_every + step as u64;
        assert!(now - start < most, "t={now}: spans {}", now - start);
    }
    assert!(rec.stats().evicted_chunks > 0);
    Ok(())
}

#[test]
fn one_byte_budget_covers_both_tracks() -> Fallible {
    let limits = Limits {
        max_age: u64::MAX,
        max_bytes: 64 << 10,
        full_every: 300,
        chunk_every: 300,
        ..Limits::default()
    };
    let rec = Recorder::new(limits);
    let mut counter = crate::toys::Git::new(&["A"], "wip");
    for now in (0..20_000u64).step_by(20) {
        counter.message = format!("wip {now}");
        rec.snapshot(now, &look(&counter));
        rec.input(now, InputKind::PointerMove, Some([1.0, 1.0]), None);
        rec.frame_scene(now, &frame(&["milk", "mail"], (now % 300) as f32, None))?;
        let stats = rec.stats();
        assert!(stats.bytes <= limits.max_bytes, "t={now}: {}", stats.bytes);
        assert_eq!(stats.bytes, stats.semantic_bytes + stats.visual_bytes);
    }
    let stats = rec.stats();
    assert!(stats.evicted_segments > 0 && stats.evicted_chunks > 0);
    let session = rec.snapshot_window();
    assert!(session.state_at(19_980)?.is_some());
    assert!(session.scene_at(19_980)?.is_some());
    Ok(())
}

#[test]
fn a_frame_from_a_compositor_records_its_visible_layers_in_order() -> Fallible {
    let mut c = Compositor::new();
    let top = c.create_layer(10);
    let hidden = c.create_layer(5);
    let faded = c.create_layer(-1);
    c.begin_frame();
    let rect = |x: f32| SceneNode::Rect {
        x,
        y: 0.0,
        w: 10.0,
        h: 10.0,
        color: [1.0, 0.0, 0.0, 1.0],
    };
    c.push(rect(1.0));
    c.push_to_layer(top, rect(2.0));
    c.push_to_layer(hidden, rect(3.0));
    c.push_to_layer(faded, rect(4.0));
    c.set_layer_visible(hidden, false);
    c.set_layer_opacity(faded, 0.5);

    let rec = Recorder::default();
    rec.frame(0, &c)?;
    let shown = rec.snapshot_window().scene_at(0)?.ok_or("no frame")?;
    assert_eq!(
        shown,
        vec![rect(4.0), rect(1.0), rect(2.0)],
        "z order, hidden left out"
    );
    assert_eq!(rec.stats().layers_approximated, 1, "the faded layer");
    Ok(())
}

#[test]
fn a_chunk_the_codec_refuses_is_split_and_still_reads_back() -> Fallible {
    // 600 nodes that change kind on every frame: 600 replace ops per
    // frame, past the u16 op count of one delta block over 120 frames.
    let flip = |i: u64| -> Vec<SceneNode> {
        (0..600)
            .map(|n| {
                let (x, y) = ((n % 30) as f32 * 10.0, (n / 30) as f32 * 10.0);
                if (i + n).is_multiple_of(2) {
                    SceneNode::Rect {
                        x,
                        y,
                        w: 8.0,
                        h: 8.0,
                        color: [1.0, 0.0, 0.0, 1.0],
                    }
                } else {
                    SceneNode::RoundedRect {
                        x,
                        y,
                        w: 8.0,
                        h: 8.0,
                        color: [0.0, 0.0, 1.0, 1.0],
                        corner_radius: 2.0,
                        border_width: 0.0,
                        border_color: [0.0; 4],
                    }
                }
            })
            .collect()
    };
    let rec = Recorder::new(Limits {
        chunk_every: 100_000,
        chunk_frames: 120,
        ..Limits::default()
    });
    for i in 0..121u64 {
        rec.frame_scene(i * 10, &flip(i))?;
    }
    let stats = rec.stats();
    assert_eq!(stats.chunks_closed, 2, "the 120 frames went as two halves");
    assert_eq!(stats.frame_errors, 0);
    let session = rec.snapshot_window();
    assert_eq!(session.frame_keys(), [0, 600, 1_200]);
    assert_eq!(session.frame_times().len(), 121);
    let mut bank = AssetBank::new();
    for i in [0u64, 59, 60, 119, 120] {
        let shown = session.scene_at(i * 10)?.ok_or("no frame")?;
        assert_eq!(
            on_grid(&shown, &mut bank),
            on_grid(&flip(i), &mut bank),
            "frame {i}"
        );
    }
    Ok(())
}

#[test]
fn a_frame_over_the_ir_limits_is_refused_and_counted() -> Fallible {
    let rec = Recorder::default();
    let rect = SceneNode::Rect {
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 1.0,
        color: [1.0; 4],
    };
    let huge = vec![rect; motion::MAX_RAISED + 1];
    assert_eq!(
        rec.frame_scene(0, &huge),
        Err(FrameError::Raise(RaiseError::TooManyNodes))
    );
    rec.frame_scene(1, &frame(&["milk"], 0.0, None))?;
    let session = rec.snapshot_window();
    assert_eq!(session.frame_times(), [1]);
    assert_eq!(rec.stats().frame_errors, 1);
    Ok(())
}
