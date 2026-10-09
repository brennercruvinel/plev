//! The builder code the parser emits and the plev IR it emits from the
//! same `ParserNode`, verified and materialized by the `surface`, drawn by
//! the same headless renderer (the window's frame path, task-66) at the
//! same size, scale and theme (`Theme::hoff()`, which the emitted code
//! reads), on the theme's background (task-71, sonda (f)).
//!
//! The two are different on purpose: the IR carries no pixel, so the
//! verdict is a difference, measured as the ADR `what-identical-means`
//! measures one (how many pixels, the largest and the mean channel delta,
//! the rect that holds them) and explained, region by region, by the
//! report's entries: every text run is where the report's rules put it,
//! and the pixels of a run whose style and color were carried exactly are
//! the same pixels, only moved.
//!
//! The builder's root is laid in a host column as wide as the area,
//! stretched, as the surface lays its root. Without a GPU adapter every
//! test skips.

// shared with tests/ir.rs: this binary uses the fixtures, not the census.
#[allow(dead_code)]
mod support;

#[allow(dead_code)]
mod sep {
    include!("fixtures/gpui/expected.rs");
}
#[allow(dead_code)]
mod card {
    include!("fixtures/react/expected.rs");
}

#[allow(dead_code)]
mod html_card {
    include!("fixtures/html/expected.rs");
}
#[allow(dead_code)]
mod settings {
    include!("fixtures/figma/expected.rs");
}

use accord::Snapshot;
use engine::compose::builder::{Align, Element, div};
use engine::compose::view::{View, ViewContext};
use engine::drawing::compositor::{Compositor, SceneNode};
use engine::drawing::gpu::RenderConfig;
use engine::drawing::headless::{Frame, HeadlessError, Renderer, Viewport};
use engine::styling::theme::Theme;
use engine::widgets::Rect;
use screen::Surface;
use support::*;

fn renderer(w: f32, h: f32) -> Option<Renderer> {
    match Renderer::new(Viewport::new(w, h, 1.0), RenderConfig::default()) {
        Ok(r) => Some(r),
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter; skipping");
            None
        }
        Err(e) => panic!("headless renderer: {e}"),
    }
}

/// A text run drawn: its text, where, and its size, line height and color.
#[derive(Debug, Clone, PartialEq)]
struct Run {
    text: String,
    x: f32,
    y: f32,
    size: f32,
    line: f32,
    color: [f32; 4],
}

fn runs<'a>(nodes: impl Iterator<Item = &'a SceneNode>) -> Vec<Run> {
    nodes
        .filter_map(|n| match n {
            SceneNode::Text { key, x, y, color } => Some(Run {
                text: key.text.clone(),
                x: *x,
                y: *y,
                size: f32::from_bits(key.font_size_bits),
                line: f32::from_bits(key.line_height_bits),
                color: *color,
            }),
            _ => None,
        })
        .collect()
}

fn run<'a>(runs: &'a [Run], text: &str) -> &'a Run {
    runs.iter()
        .find(|r| r.text == text)
        .unwrap_or_else(|| panic!("no run `{text}` in {runs:#?}"))
}

/// What a frame drew and where.
struct Drawn {
    frame: Frame,
    runs: Vec<Run>,
    /// Every scene node (the builder's, or the surface's layers').
    nodes: Vec<SceneNode>,
}

/// The emitted builder code's element, in a host column as wide as the
/// area.
fn builder(el: Element, r: &mut Renderer, theme: &Theme) -> Drawn {
    let v = r.viewport();
    let host = div().w(v.width).align_items(Align::Stretch).child(el);
    let mut c = Compositor::new();
    c.begin_frame();
    let mut cx = ViewContext::new(v.width, v.height);
    cx.theme = Some(theme.clone());
    let nodes = host.render(&mut cx);
    for n in &nodes {
        c.push(n.clone());
    }
    let frame = r
        .render(&mut c, theme.colors.bg.to_linear_array())
        .expect("the builder frame");
    Drawn {
        frame,
        runs: runs(nodes.iter()),
        nodes,
    }
}

/// The IR, verified against the app's registry and materialized over the
/// app's state, drawn in the whole area.
fn materialized(
    imported: &import::Imported,
    app: &Snapshot,
    r: &mut Renderer,
    theme: &Theme,
) -> Drawn {
    let v = r.viewport();
    let verified = plevir::verify(
        &imported.ir,
        &plevir::Capabilities::from_snapshot(app),
        &plevir::GenerationPolicy::default(),
    )
    .unwrap_or_else(|e| panic!("{e:#?}"));
    let mut surface = Surface::materialize(verified, app);
    let mut c = Compositor::new();
    c.begin_frame();
    surface.render(&mut c, Rect::new(0.0, 0.0, v.width, v.height), theme);
    let nodes: Vec<SceneNode> = c
        .layers()
        .iter()
        .flat_map(|l| l.nodes().iter().cloned())
        .collect();
    let frame = r
        .render(&mut c, theme.colors.bg.to_linear_array())
        .expect("the IR frame");
    Drawn {
        frame,
        runs: runs(nodes.iter()),
        nodes,
    }
}

/// How two regions of `w` by `h` pixels differ: how many pixels, the
/// largest and the mean channel delta (0 to 255, the largest of the four
/// channels of a pixel), and the rect that holds them in `a`'s place.
#[derive(Debug, Default, PartialEq)]
struct Delta {
    count: usize,
    max: u8,
    mean: f32,
    /// x, y, width, height.
    rect: Option<[u32; 4]>,
    /// The differing pixels, in `a`'s place.
    at: Vec<(u32, u32)>,
}

fn delta(
    a: &Frame,
    (ax, ay): (u32, u32),
    b: &Frame,
    (bx, by): (u32, u32),
    (w, h): (u32, u32),
) -> Delta {
    let mut d = Delta::default();
    let mut sum = 0u64;
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for y in 0..h {
        for x in 0..w {
            let (p, q) = (a.pixel(ax + x, ay + y), b.pixel(bx + x, by + y));
            let (Some(p), Some(q)) = (p, q) else {
                continue;
            };
            let m = p
                .iter()
                .zip(q)
                .map(|(s, t)| s.abs_diff(t))
                .max()
                .unwrap_or(0);
            if m == 0 {
                continue;
            }
            d.count += 1;
            d.max = d.max.max(m);
            sum += u64::from(m);
            let (px, py) = (ax + x, ay + y);
            (x0, y0, x1, y1) = (x0.min(px), y0.min(py), x1.max(px), y1.max(py));
            d.at.push((px, py));
        }
    }
    if d.count > 0 {
        d.mean = sum as f32 / d.count as f32;
        d.rect = Some([x0, y0, x1 - x0 + 1, y1 - y0 + 1]);
    }
    d
}

fn whole(a: &Frame, b: &Frame) -> Delta {
    delta(a, (0, 0), b, (0, 0), (a.width, a.height))
}

/// The separator: the IR's row is the separator's own height, 36 px
/// (`theme.control.height(Xs)`, where the surface draws a separator in a
/// row), not the chip's 24: everything sits 6 px lower. Moved back by those
/// 6 px, the frames differ only on the line's two pixel rows: the surface's
/// separator draws its 1 px line centered in its 36 px box, at 17.5, half on
/// each of two rows, where the builder's line sits whole on one row (the
/// builder's layout lands it on 12); and beside the label the IR's lines
/// run 4 px further (the chip's padding is `xs`, 4 px, where the source has
/// 8 on the sides). The label's pixels are the same, though its line box is
/// 15.96 px (`caption_r`) where the source's is 16.
#[test]
fn the_separator_by_pixel() {
    let theme = Theme::hoff();
    let Some(mut r) = renderer(400.0, 60.0) else {
        return;
    };
    let a = builder(sep::separator("OR"), &mut r, &theme);
    let b = materialized(&import_gpui(), &separator_app("OR"), &mut r, &theme);
    let all = whole(&a.frame, &b.frame);
    eprintln!(
        "separator: {} pixels differ, delta max {} mean {:.1}, in {:?}",
        all.count, all.max, all.mean, all.rect
    );
    assert!(all.count > 0, "the IR carries no pixel: the frames differ");

    let (ra, rb) = (run(&a.runs, "OR"), run(&b.runs, "OR"));
    assert_eq!((ra.x, ra.y), (192.0, 4.0));
    assert_eq!((rb.x, rb.y), (192.0, 10.0), "6 px lower, centered the same");
    assert_eq!((ra.size, rb.size), (12.0, 12.0));
    assert_eq!(ra.color, rb.color, "text_mid, supported");
    assert!((ra.line - 16.0).abs() < 1e-4 && (rb.line - 15.96).abs() < 1e-3);

    let dy = 6;
    let moved = delta(&a.frame, (0, 0), &b.frame, (0, dy), (400, 60 - dy));
    eprintln!(
        "separator moved back 6 px: {} pixels differ, delta max {} mean {:.1}, in {:?}",
        moved.count, moved.max, moved.mean, moved.rect
    );
    let line_y = 12;
    for &(x, y) in &moved.at {
        assert!(
            y == line_y || y == line_y - 1,
            "({x}, {y}) differs off the line's rows: no entry explains it"
        );
    }
    let label = delta(&a.frame, (192, 4), &b.frame, (192, 4 + dy), (18, 17));
    assert_eq!(label.count, 0, "the label: the same pixels, moved");
    // no difference is larger than the whole line against the background
    let line = a.frame.pixel(100, line_y).expect("on the line");
    let ground = a.frame.pixel(100, 40).expect("under it");
    let contrast = line.iter().zip(ground).map(|(p, q)| p.abs_diff(q)).max();
    assert!(moved.max <= contrast.unwrap_or(0), "{moved:?}");
    // the builder's chip spans 184..216 (8 + 16 + 8), the IR's 188..213
    // (4 + 17 + 4, the label measured to 17): the lines run 4 px further
    let beside = |x: u32| (184..188).contains(&x) || (213..216).contains(&x);
    assert!(moved.at.iter().any(|(x, y)| *y == line_y && beside(*x)));
    for x in [184, 187, 213, 215] {
        assert_eq!(
            a.frame.pixel(x, line_y),
            a.frame.pixel(100, 40),
            "the builder draws no line at {x}"
        );
    }
}

/// The note card: every text run is where the report's rules put it. The
/// preview's 160 px height is impossible (the IR has no height token near
/// it), so everything under it is 160 px higher; the details' side padding
/// (`0 16px`) is impossible, so the runs are 16 px further left. The title
/// and the content keep their style and color exactly (`title`, `base_r`,
/// `text`, `text_mid`): their pixels are the same pixels, moved. The meta
/// line's color is approximated (`n2(0.66)` read as `text_mid`, 10.2
/// levels of alpha) and its line box (`caption_r`, 15.96 px), so its pixels
/// differ, by little. The field and the button are the comps widgets, in
/// the theme's own style: the source's box is approximated, the field
/// shows its placeholder (an attribute the code emitter does not read).
#[test]
fn the_note_card_by_pixel() {
    let theme = Theme::hoff();
    let Some(mut r) = renderer(400.0, 460.0) else {
        return;
    };
    let a = builder(
        card::note_card(div(), "A title", "Some content"),
        &mut r,
        &theme,
    );
    let app = note_card_app("A title", "Some content");
    let b = materialized(&import_react(), &app, &mut r, &theme);
    let all = whole(&a.frame, &b.frame);
    eprintln!(
        "note card: {} pixels differ, delta max {} mean {:.1}, in {:?}",
        all.count, all.max, all.mean, all.rect
    );
    assert!(all.count > 0, "the IR carries no pixel: the frames differ");

    let (preview, side) = (160.0, 16.0);
    for text in ["Field note", "A title", "Some content"] {
        let (ra, rb) = (run(&a.runs, text), run(&b.runs, text));
        assert_eq!(
            (rb.x, rb.y),
            (ra.x - side, ra.y - preview),
            "{text}: moved by the preview's height and the details' side padding"
        );
        assert_eq!(ra.size, rb.size, "{text}");
    }
    for text in ["A title", "Some content"] {
        let (ra, rb) = (run(&a.runs, text), run(&b.runs, text));
        assert_eq!(
            (ra.line, ra.color),
            (rb.line, rb.color),
            "{text}: carried exactly"
        );
        let same = delta(
            &a.frame,
            (ra.x as u32, ra.y as u32),
            &b.frame,
            (rb.x as u32, rb.y as u32),
            (200, ra.line as u32),
        );
        assert_eq!(same.count, 0, "{text}: the same pixels, moved: {same:?}");
    }
    let (ma, mb) = (run(&a.runs, "Field note"), run(&b.runs, "Field note"));
    assert!((ma.color[3] - 0.66).abs() < 1e-4 && (mb.color[3] - 0.70).abs() < 1e-4);
    let meta = delta(
        &a.frame,
        (ma.x as u32, ma.y as u32),
        &b.frame,
        (mb.x as u32, mb.y as u32),
        (200, 16),
    );
    eprintln!(
        "meta line: {} pixels differ, delta max {} mean {:.1}",
        meta.count, meta.max, meta.mean
    );
    // 0.04 of alpha over the card's fill: at most 0.04 x (248 - 46) = 8.1
    // levels on a glyph's full coverage, and the 0.04 px of line height
    assert!(meta.count > 0 && meta.max <= 9, "{meta:?}");

    // the controls are the comps widgets: the field shows its placeholder,
    // the button its label in the theme's button style
    assert!(!a.runs.iter().any(|r| r.text == "Reply"));
    let reply = run(&b.runs, "Reply");
    let open = (run(&a.runs, "Open"), run(&b.runs, "Open"));
    assert!(reply.y > mb.y);
    assert_ne!(open.0.size, open.1.size, "the button's own label style");

    // the card: the builder's is 408 px tall, the IR's 228: 160 of preview,
    // 16 of bottom padding (32 read as the smallest side, 16), the field's
    // own height (44 against the source's 24) and the button's (40 against
    // 48), and the stack's one gap (8) where the source had 16 twice
    let card_h = |nodes: &[SceneNode]| {
        nodes
            .iter()
            .find_map(|n| match n {
                SceneNode::RoundedRect { x, y, w, h, .. }
                    if *x == 0.0 && *y == 0.0 && *w == 368.0 =>
                {
                    Some(*h)
                }
                _ => None,
            })
            .expect("the card's fill")
    };
    let (ha, hb) = (card_h(&a.nodes), card_h(&b.nodes));
    assert_eq!(ha, 408.0);
    assert_eq!(
        hb,
        ha - 160.0 - 16.0 + (44.0 - 24.0) - (48.0 - 40.0) - 8.0 - 8.0
    );
}

/// A phase 2 source (task-71): its IR, the app it is verified against,
/// the builder code emitted from the same `ParserNode`, the area, and the
/// texts it holds in the source's order.
struct Case {
    name: &'static str,
    imported: import::Imported,
    app: Snapshot,
    code: Element,
    size: [f32; 2],
    texts: &'static [&'static str],
}

fn phase2() -> [Case; 2] {
    [
        Case {
            name: "html-note-card",
            imported: import_html(),
            app: note_card_app("", ""),
            code: html_card::note_card(),
            size: [400.0, 460.0],
            texts: &["Field note", "Tokens over literals", "New", "ui", "ir"],
        },
        Case {
            name: "figma-settings",
            imported: import_figma(),
            app: accord::SnapshotBuilder::new()
                .finish()
                .expect("an empty app"),
            code: settings::settings(),
            size: [393.0, 852.0],
            texts: &[
                "9:41",
                "Settings",
                "UrAI Pro",
                "Upgrade Now",
                "Privacy Policy",
            ],
        },
    ]
}

/// The IR each phase 2 importer writes, verified and materialized, draws
/// every text the source holds, in the source's order; the builder code
/// draws them too. No comparison by pixel: `write_the_frames` writes the
/// frames for a person to look at.
#[test]
fn the_html_and_figma_ir_materialize_and_draw() {
    let theme = Theme::hoff();
    for c in phase2() {
        let Some(mut r) = renderer(c.size[0], c.size[1]) else {
            return;
        };
        let ir = materialized(&c.imported, &c.app, &mut r, &theme);
        let code = builder(c.code, &mut r, &theme);
        for drawn in [&ir, &code] {
            let ys: Vec<f32> = c.texts.iter().map(|t| run(&drawn.runs, t).y).collect();
            assert!(ys.windows(2).all(|p| p[0] <= p[1]), "{}: {ys:?}", c.name);
        }
    }
}

/// The frames, written for a person to look at:
/// `cargo test -p parser --test pixels -- --ignored`, then the pngs in the
/// test's target tmp dir (the separator, the react note card, and the two
/// phase 2 sources: the html note card and the figma screen).
#[test]
#[ignore = "diagnostic: writes the frames as png"]
fn write_the_frames() {
    let theme = Theme::hoff();
    let out = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    if let Some(mut r) = renderer(400.0, 60.0) {
        let a = builder(sep::separator("OR"), &mut r, &theme);
        let b = materialized(&import_gpui(), &separator_app("OR"), &mut r, &theme);
        a.frame
            .write_png(out.join("separator-builder.png"))
            .expect("png");
        b.frame
            .write_png(out.join("separator-ir.png"))
            .expect("png");
    }
    if let Some(mut r) = renderer(400.0, 460.0) {
        let a = builder(
            card::note_card(div(), "A title", "Some content"),
            &mut r,
            &theme,
        );
        let app = note_card_app("A title", "Some content");
        let b = materialized(&import_react(), &app, &mut r, &theme);
        a.frame
            .write_png(out.join("note-card-builder.png"))
            .expect("png");
        b.frame
            .write_png(out.join("note-card-ir.png"))
            .expect("png");
    }
    for c in phase2() {
        if let Some(mut r) = renderer(c.size[0], c.size[1]) {
            let ir = materialized(&c.imported, &c.app, &mut r, &theme);
            let code = builder(c.code, &mut r, &theme);
            ir.frame
                .write_png(out.join(format!("{}-ir.png", c.name)))
                .expect("png");
            code.frame
                .write_png(out.join(format!("{}-builder.png", c.name)))
                .expect("png");
        }
    }
    eprintln!("frames in {}", out.display());
}
