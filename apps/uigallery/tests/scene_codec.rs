//! the scene codec over the whole gallery (task-79): every page of the
//! showcase in both main themes, written by `record::scene::Sender`, read
//! back by a `Mirror` and drawn into a second compositor, holds the same
//! layers with the same nodes, bit for bit. no GPU: the pixels are
//! compared in apps/uigallery/tests/mirror.rs.

use std::hash::{DefaultHasher, Hash, Hasher};

use engine::drawing::compositor::{Compositor, SceneNode};
use engine::drawing::gpu::image_pixels;
use engine::drawing::headless::Viewport;
use record::scene::{Brought, Head, Incoming, Mirror, Sender};
use uigallery::shell::stage_page;
use uigallery::views::PAGES;

const VIEWPORT: Viewport = Viewport {
    width: 1280.0,
    height: 800.0,
    scale: 1.0,
};

/// a node as Debug text (a NaN reads as itself). an image reads as its
/// size and its pixels, not as its handle: the handle is where a store
/// holds it, and the mirror packs what it receives (`load_image_rgba`),
/// which is the sender's own slot only for an image the sender packed
/// from raw pixels too. the Dialog's illustration is decoded from a PNG
/// (`load_image_bytes`), so it comes back in a slot of its own.
fn node(n: &SceneNode) -> String {
    let SceneNode::Image {
        x,
        y,
        w,
        h,
        image,
        corner_radius,
    } = n
    else {
        return format!("{n:?}");
    };
    let pixels = image_pixels(image).expect("an image drawn is in the store");
    let mut hasher = DefaultHasher::new();
    pixels.hash(&mut hasher);
    format!(
        "Image {{ x: {x:?}, y: {y:?}, w: {w:?}, h: {h:?}, {}x{} pixels {:x}, \
         corner_radius: {corner_radius:?} }}",
        image.width,
        image.height,
        hasher.finish()
    )
}

/// every layer that draws, in composite order, with its properties and
/// its nodes.
fn scene(c: &Compositor) -> Vec<String> {
    c.layers()
        .iter()
        .filter(|l| !l.nodes().is_empty())
        .map(|l| {
            format!(
                "z {} visible {} opacity {} clip {:?} effects {:?} nodes [{}]",
                l.z_order,
                l.visible,
                l.opacity,
                l.clip_rect,
                l.effects(),
                l.nodes().iter().map(node).collect::<Vec<_>>().join(", ")
            )
        })
        .collect()
}

#[test]
fn every_gallery_page_comes_back_node_for_node() {
    let mut pages = 0;
    for theme in ["core", "hoff"] {
        for p in PAGES {
            let staged = stage_page(VIEWPORT, p.title, theme, None).expect("the page stages");
            let head = Head {
                width: VIEWPORT.width,
                height: VIEWPORT.height,
                scale: VIEWPORT.scale,
                clear: [0.0, 0.0, 0.0, 1.0],
            };
            let frame = Sender::new().frame(staged.compositor(), head);
            let mut m = Mirror::new();
            for (key, bytes) in &frame.assets {
                m.asset(*key, bytes).expect("an asset reads");
            }
            let layers: Vec<Incoming<'_>> = frame
                .layers
                .iter()
                .map(|l| Incoming {
                    key: l.key,
                    hash: l.hash,
                    brought: Brought::Body(&l.body),
                })
                .collect();
            m.frame(Some(&frame.head), &layers)
                .unwrap_or_else(|e| panic!("{} / {theme}: {e}", p.title));
            let mut b = Compositor::new();
            m.draw(&mut b);
            b.resolve_scene((VIEWPORT.width, VIEWPORT.height));
            assert_eq!(
                scene(&b),
                scene(staged.compositor()),
                "{} / {theme}",
                p.title
            );
            assert_eq!(m.image_counts().missing, 0, "{} / {theme}", p.title);
            pages += 1;
        }
    }
    assert_eq!(pages, PAGES.len() * 2);
}
