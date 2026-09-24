//! The windows' scenes over the agent port (task-79,
//! docs/ADR/scene-travels-as-data.md), desktop only, like the port.
//!
//! The app's side: when a connection watches a window (`main`, or `page`
//! for the page's own window), the shell writes that window's frame
//! (`rec::scene::Sender`) after each frame it draws there, and after each
//! drain of the port, so a watch that just came gets the frame on screen
//! without waiting for the next one; the port sends each watch only what
//! changed. With nobody watching, nothing is written.
//!
//! The viewer's side, [`Viewer`]: a watch's replies put back together
//! (`wire::SceneReader`), applied to a `rec::scene::Mirror`, and drawn
//! into a compositor of its own, with the bindings (where each control
//! shows, task-59) drawn over it on a layer of their own when asked. The
//! `mirror` example and the tests draw through it.

use std::collections::HashMap;

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use rec::scene::{Brought, Frame, Head, Incoming, Mirror, Sender};
use winit::window::WindowId;
use wire::{AgentPort, Bound, LayerBody, Offer, OfferLayer, Reply, SceneReader};

use super::App;

/// The main window's name on the scene channel.
pub const MAIN: &str = wire::MAIN_WINDOW;

/// The page's own window's name on the scene channel.
pub const PAGE: &str = "page";

/// The scene writers of the windows someone watches.
#[derive(Default)]
pub struct Scenes {
    senders: HashMap<String, Sender>,
}

impl Scenes {
    /// Offers `compositor`'s frame of `window`, laid out and cleared as
    /// `head` says, to the port's watches of it. Nothing is written when
    /// none watches (and the writer of that window is let go). True when
    /// a frame was offered.
    pub fn offer(
        &mut self,
        port: &mut AgentPort,
        window: &str,
        compositor: &Compositor,
        head: Head,
    ) -> bool {
        if !port.watching(window) {
            self.senders.remove(window);
            return false;
        }
        let sender = self.senders.entry(window.to_owned()).or_default();
        let frame = sender.frame(compositor, head);
        port.offer_scene(window, &offer(frame, compositor));
        true
    }
}

/// A frame as the port takes it, with where each bound control shows
/// in it (the visible part of each binding, as an observation serves it).
pub fn offer(frame: Frame, compositor: &Compositor) -> Offer {
    Offer {
        head: frame.head,
        layers: frame
            .layers
            .into_iter()
            .map(|l| OfferLayer {
                key: l.key,
                hash: l.hash,
                body: l.body,
                patch: l.patch,
            })
            .collect(),
        assets: frame.assets,
        bounds: compositor
            .visible_bindings()
            .map(|(id, rect)| Bound {
                id: id.clone(),
                rect,
            })
            .collect(),
    }
}

impl App {
    /// Window `id`'s frame to its watches, if any: after it was drawn.
    pub(super) fn offer_scene(&mut self, id: WindowId) {
        let name = if Some(id) == self.main {
            MAIN
        } else if self.is_page_window(id) {
            PAGE
        } else {
            return;
        };
        let clear = self.view.theme.colors.bg.to_linear_array();
        let (Some(port), Some(host)) = (self.access.agent.as_mut(), self.windows.get(id)) else {
            return;
        };
        let (width, height) = host.logical_size();
        let head = Head {
            width,
            height,
            scale: host.scale_factor() as f32,
            clear,
        };
        self.access
            .scenes
            .offer(port, name, host.compositor(), head);
    }

    /// Every window's frame to its watches: after a drain of the port, so
    /// a watch that just came gets the frame on screen.
    pub(super) fn offer_scenes(&mut self) {
        let ids = [self.main, self.page_window_id()];
        for id in ids.into_iter().flatten() {
            self.offer_scene(id);
        }
    }
}

/// The viewer's side of a watch: its replies put back together into
/// the scene the app drew.
#[derive(Default)]
pub struct Viewer {
    reader: SceneReader,
    mirror: Mirror,
    bounds: Vec<Bound>,
    overlay: Option<LayerId>,
}

impl Viewer {
    pub fn new() -> Viewer {
        Viewer::default()
    }

    /// One reply of the watch: the frame's number when it completed one,
    /// `None` for a part of a blob (or a reply that is not the scene's).
    /// An error when the lines do not read as the app writes them: the
    /// watch has to start over, on a new connection.
    pub fn take(&mut self, reply: Reply) -> Result<Option<u64>, String> {
        let Some(frame) = self.reader.take(reply).map_err(|e| e.to_string())? else {
            return Ok(None);
        };
        for (key, bytes) in &frame.assets {
            self.mirror.asset(*key, bytes).map_err(|e| e.to_string())?;
        }
        let layers: Vec<Incoming<'_>> = frame
            .layers
            .iter()
            .map(|l| Incoming {
                key: l.key,
                hash: l.hash,
                brought: match &l.body {
                    LayerBody::Held => Brought::Held,
                    LayerBody::Body(b) => Brought::Body(b),
                    LayerBody::Patch(p) => Brought::Patch(p),
                },
            })
            .collect();
        self.mirror
            .frame(frame.head.as_deref(), &layers)
            .map_err(|e| e.to_string())?;
        if let Some(bounds) = frame.bounds {
            self.bounds = bounds;
        }
        Ok(Some(frame.seq))
    }

    /// The size, scale and clear of the last frame.
    pub fn head(&self) -> Option<Head> {
        self.mirror.head()
    }

    /// Where each bound control shows in the last frame.
    pub fn bounds(&self) -> &[Bound] {
        &self.bounds
    }

    pub fn mirror(&self) -> &Mirror {
        &self.mirror
    }

    /// The scene into `c`, as the app's compositor held it; with
    /// `bindings`, each bound control outlined and named on a layer above
    /// every other (the frame without them is the app's, pixel for pixel).
    pub fn draw(&mut self, c: &mut Compositor, bindings: bool) {
        self.mirror.draw(c);
        if !bindings {
            return;
        }
        let layer = *self.overlay.get_or_insert_with(|| c.create_layer(i32::MAX));
        for b in &self.bounds {
            let [x, y, w, h] = b.rect;
            c.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x,
                    y,
                    w,
                    h,
                    color: [0.0; 4],
                    corner_radius: 2.0,
                    border_width: 1.0,
                    border_color: OUTLINE,
                },
            );
            let label = TextNodeKey {
                text: b.id.as_str().to_owned(),
                font_size_bits: 10.0f32.to_bits(),
                line_height_bits: 12.0f32.to_bits(),
                max_width_bits: None,
                font_weight: 500,
                letter_spacing_bits: 0.0f32.to_bits(),
                font_family: None,
            };
            let width = 6.0 * label.text.chars().count() as f32 + 6.0;
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x,
                    y: y - 13.0,
                    w: width,
                    h: 13.0,
                    color: LABEL,
                },
            );
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: label,
                    x: x + 3.0,
                    y: y - 12.5,
                    color: [1.0; 4],
                },
            );
        }
    }
}

/// The outline of a bound control and the ground of its name.
const OUTLINE: [f32; 4] = [1.0, 0.18, 0.55, 0.95];
const LABEL: [f32; 4] = [0.35, 0.02, 0.16, 0.85];

/// The shell's side: a watch of the main window gets the frame on screen
/// after the drain that took it, then only what a new frame changed;
/// with nobody watching, nothing is written. A bare main window (no
/// platform window, no device); the frame is built into its compositor.
#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use engine::compositor::Compositor;
    use wire::{AgentPort, Client, Config};

    use super::super::App;
    use super::*;
    use crate::view::ShowcaseView;

    /// The shell on the Todo page with its port in `dir`, and the main
    /// window's frame built.
    fn app(dir: &std::path::Path) -> App {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.jump_to_page("Todo");
        view.resize(1200.0, 800.0, 1.0);
        let mut app = App::with_bare_window(view);
        let config = Config {
            app: "showcase".into(),
            dir: dir.to_owned(),
            ring: 8,
        };
        app.access.agent = Some(AgentPort::open_with(config, || {}).expect("the port opens"));
        app.draw_main();
        app
    }

    impl App {
        fn draw_main(&mut self) {
            let id = self.main.expect("a main window");
            let host = self.windows.get_mut(id).expect("its host");
            self.view.render(host.compositor_mut());
        }
    }

    /// The watch's frames until one completes, draining the port between
    /// reads as the shell does on its wake.
    fn next(app: &mut App, c: &mut Client, v: &mut Viewer) -> u64 {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            assert!(Instant::now() < deadline, "no frame came");
            app.serve_agent();
            match c.recv() {
                Ok(msg) => {
                    if let Some(seq) = v.take(msg.reply).expect("the frame reads") {
                        return seq;
                    }
                }
                Err(wire::WireError::Timeout(_)) => {}
                Err(e) => panic!("{e}"),
            }
        }
    }

    #[test]
    fn the_shell_offers_the_main_window_to_its_watch() {
        let tmp = tempfile::tempdir().expect("a temporary directory");
        let mut app = app(tmp.path());
        let path = app
            .access
            .agent
            .as_ref()
            .expect("the port")
            .path()
            .to_owned();
        app.serve_agent();
        assert!(app.access.scenes.senders.is_empty(), "nobody watches");

        let mut c = Client::connect(&path).expect("the client connects");
        c.set_timeout(Some(50)).expect("a timeout");
        c.watch_scene(MAIN).expect("the watch goes");
        let mut v = Viewer::new();
        assert_eq!(next(&mut app, &mut c, &mut v), 1);
        let mut drawn = Compositor::new();
        v.draw(&mut drawn, false);
        let main = app.main.expect("a main window");
        let shown = app.windows.get(main).expect("its host").compositor();
        let nodes = |c: &Compositor| -> Vec<String> {
            c.layers()
                .iter()
                .filter(|l| !l.nodes().is_empty())
                .map(|l| format!("{} {:?}", l.z_order, l.nodes()))
                .collect()
        };
        assert_eq!(nodes(&drawn), nodes(shown), "the frame on screen");
        assert!(
            v.bounds().iter().any(|b| b.id.as_str() == "todo.input"),
            "with where each control shows"
        );

        // a frame drawn again with nothing changed sends nothing; a
        // hover sends the layer it changed
        app.draw_main();
        app.offer_scene(main);
        let rect = v
            .bounds()
            .iter()
            .find(|b| b.id.as_str() == "todo.filter[completed]")
            .expect("the filter of the items done")
            .rect;
        app.view
            .handle_event(&comps::prelude::WidgetEvent::MouseMove {
                x: rect[0] + rect[2] / 2.0,
                y: rect[1] + rect[3] / 2.0,
            });
        for _ in 0..60 {
            if !app.view.tick(1.0 / 60.0) {
                break;
            }
        }
        app.draw_main();
        app.offer_scene(main);
        assert_eq!(
            next(&mut app, &mut c, &mut v),
            2,
            "the unchanged frame sent nothing"
        );
        v.draw(&mut drawn, false);
        let shown = app.windows.get(main).expect("its host").compositor();
        assert_eq!(nodes(&drawn), nodes(shown));
    }
}
