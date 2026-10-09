//! The windows' scenes over the agent port (task-79,
//! docs/adr/recording/0065-scene-travels-as-data.md), desktop only, like the port.
//!
//! The app's side: when a connection watches a window (`main`, or `page`
//! for the page's own window), the shell writes that window's frame
//! (`record::scene::Sender`) after each frame it draws there, and after each
//! drain of the port, so a watch that just came gets the frame on screen
//! without waiting for the next one; the port sends each watch only what
//! changed. With nobody watching, nothing is written. The port takes a
//! watch only when the person launched the app with `PLEV_MIRROR=1`
//! beside `PLEV_AGENT=1`, and only after `hello`; the shell logs who
//! begins and who ends watching which window ("scene: claude watches
//! main"), and a recording keeps a watch that begins as an observation
//! of its agent (the port tells the host, `Host::observed`). A camera's
//! frames never travel: the codec sends a stream image's size and place.
//! Nor does a secret (task-79 phase 2): each frame is written with the
//! places of the bound controls the port's one rule counts as secret, in
//! the gallery's projection at that frame, masked ([`masks`]): what is
//! drawn there stays out of the frame, a box in its stead.
//!
//! The viewer's side, [`Viewer`]: a watch's replies put back together
//! (`socket::SceneReader`), applied to a `record::scene::Mirror`, and drawn
//! into a compositor of its own, with the bindings (where each control
//! shows, task-59) drawn over it on a layer of their own when asked. The
//! app sends the bindings from the bottom of the frame to the top
//! ([`stacked`]), so the viewer finds the control under a point
//! ([`Viewer::hit`]) as the app's own hit test does, and its [`Hand`]
//! presses it: an act through the app's one dispatcher, as the agent the
//! viewer says it is, resting on its last look at the app (task-79 phase
//! 2). The `mirror` example and the tests draw and click through them.

use std::collections::HashMap;
use std::path::Path;

use accord::redactor::is_secret;
use accord::{ActionName, Actor, Change, Rejection, Revision, SemId, Snapshot, Stage};
use engine::control::accessibility::primary;
use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use record::scene::{Brought, Frame, Head, Incoming, Mask, Mirror, Sender};
use socket::{
    Act, AgentPort, Bound, Client, LayerBody, Observe, Offer, OfferLayer, Reply, SceneReader,
    SceneWatch, SnapshotReply, StageReply,
};
use winit::window::WindowId;

use super::super::App;

/// The main window's name on the scene channel.
pub const MAIN: &str = socket::MAIN_WINDOW;

/// The page's own window's name on the scene channel.
pub const PAGE: &str = "page";

/// The scene writers of the windows someone watches.
#[derive(Default)]
pub struct Scenes {
    senders: HashMap<String, Sender>,
}

impl Scenes {
    /// Offers `compositor`'s frame of `window`, laid out and cleared as
    /// `head` says, to the port's watches of it, with what a secret draws
    /// masked ([`masks`]) by `now`, the app's projection at this frame,
    /// asked only when someone watches. Nothing is written when none
    /// watches (and the writer of that window is let go). True when a
    /// frame was offered.
    pub fn offer(
        &mut self,
        port: &mut AgentPort,
        window: &str,
        compositor: &Compositor,
        head: Head,
        now: impl FnOnce() -> Option<Snapshot>,
    ) -> bool {
        tell_watches(port);
        if !port.watching(window) {
            self.senders.remove(window);
            return false;
        }
        let masks = masks(now().as_ref(), compositor);
        let sender = self.senders.entry(window.to_owned()).or_default();
        let frame = sender.frame_masked(compositor, head, &masks);
        port.offer_scene(window, &offer(frame, compositor));
        // a watcher cut off by this offer
        tell_watches(port);
        true
    }
}

/// The places of `compositor`'s frame whose drawing never travels: every
/// bound control the one rule of the port counts as secret in `now`, the
/// app's projection at this frame (`accord::redactor::is_secret`: marked
/// secret, or not held by the projection, or no projection at all; the
/// rule fails closed), at the rect it is bound to on its layer, the part
/// a clip hides included. The codec leaves what is drawn there out of the
/// frame and writes a box in its stead (`record::scene::Sender::frame_masked`).
pub fn masks(now: Option<&Snapshot>, compositor: &Compositor) -> Vec<Mask> {
    compositor
        .bindings()
        .filter(|(id, _)| is_secret(now, id))
        .map(|(_, b)| Mask {
            layer: b.layer.0,
            rect: b.rect,
        })
        .collect()
}

/// Logs who began and who ended watching which window since the last
/// look, as the port saw it.
fn tell_watches(port: &mut AgentPort) {
    for w in port.take_scene_watches() {
        log::info!("{}", watch_line(&w));
    }
}

/// What the app says of a watch: "scene: claude watches main" when it
/// begins, "scene: claude no longer watches main" when it ends.
pub fn watch_line(w: &SceneWatch) -> String {
    let who = match &w.actor {
        Actor::Agent { name } | Actor::Test { name } => name.clone(),
        other => other.to_string(),
    };
    let verb = if w.began {
        "watches"
    } else {
        "no longer watches"
    };
    format!("scene: {who} {verb} {}", w.window)
}

/// A frame as the port takes it, with where each bound control shows
/// in it (the visible part of each binding, as an observation serves it),
/// in the order a click finds them ([`stacked`]).
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
        bounds: stacked(compositor),
    }
}

/// Where each bound control shows (the visible part of its binding),
/// from the bottom of the frame to the top: by its layer's z-order, then
/// the layer's place among the compositor's, then the bind order, the
/// order `Compositor::hit` ranks them by. A binding on a hidden layer
/// (not visible, or no opacity) shows nowhere and is left out, as `hit`
/// leaves it. So the last one holding a point is the one the app's own
/// hit test finds there ([`Viewer::hit`]).
pub fn stacked(compositor: &Compositor) -> Vec<Bound> {
    let layers = compositor.layers();
    let mut shown: Vec<((i32, usize, u32), Bound)> = compositor
        .bindings()
        .filter_map(|(id, b)| {
            let (rank, layer) = layers.iter().enumerate().find(|(_, l)| l.id == b.layer)?;
            if !layer.visible || layer.opacity <= 0.0 {
                return None;
            }
            let rect = b.visible_rect()?;
            let bound = Bound {
                id: id.clone(),
                rect,
            };
            Some(((layer.z_order, rank, b.order), bound))
        })
        .collect();
    shown.sort_by_key(|(at, _)| *at);
    shown.into_iter().map(|(_, b)| b).collect()
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
        let view = &self.view;
        self.access
            .scenes
            .offer(port, name, host.compositor(), head, || {
                view.gallery_semantics().ok()
            });
    }

    /// Every window's frame to its watches: after a drain of the port, so
    /// a watch that just came gets the frame on screen.
    pub(crate) fn offer_scenes(&mut self) {
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

    /// Where each bound control shows in the last frame, from the bottom
    /// of the frame to the top.
    pub fn bounds(&self) -> &[Bound] {
        &self.bounds
    }

    /// The bound control at `(x, y)` in the last frame, in the scene's
    /// logical pixels: the topmost binding whose visible rect holds the
    /// point (edges included), the one the app's own hit test finds
    /// there. `None` over no control. The bindings the viewer draws over
    /// the scene take no part.
    pub fn hit(&self, x: f32, y: f32) -> Option<&SemId> {
        self.bounds
            .iter()
            .rev()
            .find(|b| {
                let [bx, by, bw, bh] = b.rect;
                x >= bx && x <= bx + bw && y >= by && y <= by + bh
            })
            .map(|b| &b.id)
    }

    pub fn mirror(&self) -> &Mirror {
        &self.mirror
    }

    /// Draws into another compositor from now on (a window made anew):
    /// its layers are made again at the next `draw`.
    pub fn forget_layers(&mut self) {
        self.mirror.forget_layers();
        self.overlay = None;
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

/// The viewer's hand (task-79 phase 2): a click at the viewer goes back
/// to the app as an act on the control under it, through the semantic
/// contract like any agent's, never as an event or a coordinate.
///
/// It is a connection of its own to the app's port under the name the
/// watch said (one actor: what one connection was shown, the other
/// was), so a watch keeps reading its frames while an act waits for its
/// stages, or for a person. It looks at the whole app ([`Hand::look`])
/// after each frame the viewer takes, and a press rests on the last look
/// (`from_revision`): when the app moved on since (the scene was old),
/// the act comes back `stale`, with what moved and who acted, as it does
/// for any agent. The app takes it as `Actor::Agent`: an action kept for
/// a person is refused, and one that asks a person waits for the person
/// at the app. A click presses what a screen reader's click presses
/// (`engine::control::accessibility::primary`), and only where the reader's tree
/// offers one (the action and the node on).
pub struct Hand {
    client: Client,
    seen: Option<SnapshotReply>,
}

/// What a click at the viewer came to.
#[derive(Clone, Debug, PartialEq)]
pub enum Pressed {
    /// No bound control holds the point: nothing went to the app.
    Nothing,
    /// The control has nothing a click presses (no primary action, or
    /// it is off), or the hand was never shown it: nothing went.
    Unpressable { target: SemId, why: String },
    /// The act went: its stages as they came, the last one terminal.
    Acted {
        target: SemId,
        action: ActionName,
        stages: Vec<StageReply>,
    },
    /// The app refused the act before its dispatcher (a revision it no
    /// longer holds, a read never shown), or the connection failed.
    Refused { target: SemId, why: String },
}

impl Hand {
    /// A connection of its own to the port at `path`, said `hello` under
    /// `name`: the name the watch said.
    pub fn connect(path: &Path, name: &str) -> Result<Hand, String> {
        let mut client = Client::connect(path).map_err(|e| e.to_string())?;
        client.hello(name).map_err(|e| e.to_string())?;
        Ok(Hand { client, seen: None })
    }

    /// How long a read waits for the app, in milliseconds; `None` waits
    /// for as long as it takes (a person's confirmation).
    pub fn set_timeout(&mut self, ms: Option<u64>) -> Result<(), String> {
        self.client.set_timeout(ms).map_err(|e| e.to_string())
    }

    /// Observes the whole app (every node, every state: no read of a
    /// press falls outside it); the next press rests on it. Its revision.
    pub fn look(&mut self) -> Result<Revision, String> {
        match self.client.observe(Observe::default()) {
            Ok(Reply::Snapshot(seen)) => {
                let revision = seen.revision;
                self.seen = Some(seen);
                Ok(revision)
            }
            Ok(other) => Err(format!("an observation came back as {other:?}")),
            Err(e) => Err(e.to_string()),
        }
    }

    /// What the last look showed: the observation a press rests on, as
    /// the port served it (a secret as its length).
    pub fn shown(&self) -> Option<&SnapshotReply> {
        self.seen.as_ref()
    }

    /// The control under `(x, y)` in the viewer's last frame, pressed
    /// ([`Hand::press`]); nothing goes when no control is there.
    pub fn click(
        &mut self,
        viewer: &Viewer,
        x: f32,
        y: f32,
        each: impl FnMut(&ActionName, &StageReply),
    ) -> Pressed {
        match viewer.hit(x, y) {
            Some(target) => {
                let target = target.clone();
                self.press(&target, each)
            }
            None => Pressed::Nothing,
        }
    }

    /// `target`'s primary action as the last look showed it, acted on
    /// that look's revision; `each` hears the action and every stage as
    /// it comes (an `awaiting_confirmation` may wait long). Nothing goes
    /// when the look does not hold the control, or holds nothing a click
    /// presses there.
    pub fn press(
        &mut self,
        target: &SemId,
        mut each: impl FnMut(&ActionName, &StageReply),
    ) -> Pressed {
        let unpressable = |why: String| Pressed::Unpressable {
            target: target.clone(),
            why,
        };
        let Some(seen) = &self.seen else {
            return unpressable("the mirror has not looked at the app yet".to_owned());
        };
        let Some(node) = seen.nodes.iter().find(|n| &n.id == target) else {
            return unpressable(format!("{target} is not in what the mirror was shown"));
        };
        let Some(decl) = primary(node) else {
            return unpressable(format!("{target} has nothing a click presses"));
        };
        if !decl.enabled || !node.enabled {
            return unpressable(format!("{target}: {} is off", decl.name));
        }
        let action = decl.name.clone();
        let act = Act::from_revision(target.clone(), action.clone(), seen.revision);
        match self.client.act(act, |s| each(&action, s)) {
            Ok(stages) => Pressed::Acted {
                target: target.clone(),
                action,
                stages,
            },
            Err(e) => Pressed::Refused {
                target: target.clone(),
                why: e.to_string(),
            },
        }
    }
}

impl Pressed {
    /// The terminal stage, when the act went.
    pub fn stage(&self) -> Option<&Stage> {
        match self {
            Pressed::Acted { stages, .. } => stages.last().map(|s| &s.stage),
            _ => None,
        }
    }

    /// What the viewer says of it, in one line.
    pub fn said(&self) -> String {
        match self {
            Pressed::Nothing => "no control here: nothing went to the app".to_owned(),
            Pressed::Unpressable { why, .. } => format!("{why}: nothing went to the app"),
            Pressed::Acted {
                target,
                action,
                stages,
            } => match stages.last() {
                Some(last) => said(target, action, &last.stage),
                None => format!("{target}: {action} went, and no stage came back"),
            },
            Pressed::Refused { target, why } => format!("{target}: {why}"),
        }
    }
}

/// What the viewer says of a stage of its act on `target`: a refusal
/// with its reason, a confirmation the person at the app gives there.
pub fn said(target: &SemId, action: &ActionName, stage: &Stage) -> String {
    let refused = |why: String| format!("{target}: {action} refused: {why}");
    match stage {
        Stage::Completed(_) => format!("{target}: {action} done"),
        Stage::AwaitingConfirmation => format!(
            "{target}: {action} waits for the person at the app, who confirms or declines it there"
        ),
        Stage::Confirmed { by } => format!("{target}: {action} confirmed by {by}"),
        Stage::Failed { error } => format!("{target}: {action} failed: {error}"),
        Stage::Rejected(Rejection::NotAuthorized { actor }) => refused(format!(
            "only a person asks it, and the mirror acts as {actor}"
        )),
        Stage::Rejected(Rejection::Stale { changes, acted }) => {
            let moved: Vec<String> = changes.iter().map(changed).collect();
            let who: Vec<String> = acted.iter().map(|a| a.actor.to_string()).collect();
            let who = if who.is_empty() {
                String::new()
            } else {
                format!(", {} acted since", who.join(", "))
            };
            refused(format!(
                "the mirror was behind the app ({} moved{who}); look again",
                moved.join(", ")
            ))
        }
        Stage::Rejected(Rejection::Declined { by }) => refused(format!("declined by {by}")),
        Stage::Rejected(Rejection::Inert { .. }) => {
            refused("something over it holds the pointer".to_owned())
        }
        Stage::Rejected(Rejection::Disabled { .. }) => refused("it is off".to_owned()),
        Stage::Rejected(other) => refused(format!("{other:?}")),
        other => format!("{target}: {action} {}", other.name()),
    }
}

/// A change as the viewer names it: the state that moved (what a stale
/// refusal carries), or the node that came or went.
fn changed(c: &Change) -> String {
    match c {
        Change::StateChanged { at, .. } => at.to_string(),
        Change::NodeAdded(node) => format!("{} came", node.id),
        Change::NodeRemoved(id) => format!("{id} went"),
        _ => "the app's structure".to_owned(),
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

    use engine::drawing::compositor::Compositor;
    use socket::{AgentPort, Client, Config};

    use super::super::super::App;
    use super::*;
    use crate::views::ShowcaseView;

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
        let mut port = AgentPort::open_with(config, || {}).expect("the port opens");
        // the person launched it with PLEV_MIRROR=1, and PLEV_RECORD=1
        port.share_scenes(true);
        app.access.agent = Some(port);
        app.recording = Some(super::super::record::Recording::new(
            dir.join("sessions"),
            record::Limits::default(),
        ));
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
                Err(socket::WireError::Timeout(_)) => {}
                Err(e) => panic!("{e}"),
            }
        }
    }

    fn sid(text: &str) -> SemId {
        SemId::parse(text).expect("an id")
    }

    /// Bound out of the stacking order: the overlay's control first,
    /// then the one under it, and a third on a layer made invisible. The
    /// viewer finds what the app's hit test finds, never the later bind.
    #[test]
    fn the_viewer_hits_by_stacking_not_by_bind_order() {
        let mut c = Compositor::new();
        let overlay = c.create_layer(10);
        let hidden = c.create_layer(20);
        c.begin_frame();
        c.bind_to_layer(overlay, &sid("dialog.ok"), [100.0, 100.0, 80.0, 40.0]);
        c.bind(&sid("page.button"), [90.0, 90.0, 200.0, 60.0]);
        c.bind_to_layer(hidden, &sid("gone"), [0.0, 0.0, 400.0, 400.0]);
        c.set_layer_visible(hidden, false);
        let mut v = Viewer::new();
        v.bounds = stacked(&c);
        let ids: Vec<&str> = v.bounds().iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, ["page.button", "dialog.ok"], "bottom to top");
        for (x, y) in [(120.0, 120.0), (250.0, 120.0), (10.0, 10.0), (100.0, 100.0)] {
            assert_eq!(v.hit(x, y), c.hit(x, y), "at ({x}, {y})");
        }
        assert_eq!(v.hit(120.0, 120.0), Some(&sid("dialog.ok")));
        assert_eq!(v.hit(250.0, 120.0), Some(&sid("page.button")));
        assert_eq!(v.hit(10.0, 10.0), None, "the hidden layer shows nothing");
    }

    /// Who watches stays in the log and the recording, and now so does
    /// what the mirror does (task-79 phase 2): its look is an
    /// observation of the agent "mirror" beside its watch, and its click
    /// a receipt of the agent "mirror", in the session the shell records,
    /// as any agent's.
    #[test]
    fn the_recording_keeps_the_mirror_s_look_and_click_under_its_name() {
        let tmp = tempfile::tempdir().expect("a temporary directory");
        let mut app = app(tmp.path());
        // the sink on the gallery's dispatcher, as the shell puts it
        if let Some(recording) = &app.recording {
            recording.attach(&mut app.view);
        }
        let path = app
            .access
            .agent
            .as_ref()
            .expect("the port")
            .path()
            .to_owned();
        let side = std::thread::spawn(move || {
            let mut c = Client::connect(&path).expect("the watch connects");
            c.set_timeout(Some(20_000)).expect("a timeout");
            c.hello("mirror").expect("hello");
            c.watch_scene(MAIN).expect("the watch goes");
            let mut v = Viewer::new();
            while v
                .take(c.recv().expect("a frame").reply)
                .expect("reads")
                .is_none()
            {}
            let mut hand = Hand::connect(&path, "mirror").expect("the hand connects");
            hand.set_timeout(Some(20_000)).expect("a timeout");
            hand.look().expect("the hand looks");
            let [x, y, w, h] = v
                .bounds()
                .iter()
                .find(|b| b.id.as_str() == "todo.item[1]")
                .expect("the item")
                .rect;
            hand.click(&v, x + w / 2.0, y + h / 2.0, |_, _| {})
        });
        let deadline = Instant::now() + Duration::from_secs(30);
        while !side.is_finished() {
            assert!(Instant::now() < deadline, "the mirror did not finish");
            app.serve_agent();
            std::thread::sleep(Duration::from_millis(2));
        }
        let pressed = side.join().expect("the mirror's thread");
        assert_eq!(pressed.stage().map(Stage::name), Some("completed"));
        let mirror = Actor::Agent {
            name: "mirror".into(),
        };
        let session = app
            .recording
            .as_ref()
            .expect("a recording")
            .recorder()
            .snapshot_window();
        let looked = session
            .events()
            .iter()
            .filter(|e| matches!(&e.kind, record::Kind::Observed { actor, .. } if *actor == mirror))
            .count();
        assert_eq!(looked, 2, "the watch and the look");
        let acted: Vec<(String, &str)> = session
            .events()
            .iter()
            .filter_map(|e| match &e.kind {
                record::Kind::Stage { request, stage, .. } if request.actor == mirror => Some((
                    format!("{} {}", request.target, request.action),
                    stage.name(),
                )),
                _ => None,
            })
            .collect();
        let toggle = "todo.item[1] toggle".to_owned();
        assert_eq!(
            acted,
            [
                (toggle.clone(), "requested"),
                (toggle.clone(), "accepted"),
                (toggle.clone(), "executing"),
                (toggle, "completed"),
            ]
        );
    }

    #[test]
    fn a_watch_is_said_by_its_agent_and_window() {
        let w = |began| SceneWatch {
            actor: Actor::Agent {
                name: "claude".into(),
            },
            window: "main".into(),
            began,
        };
        assert_eq!(watch_line(&w(true)), "scene: claude watches main");
        assert_eq!(
            watch_line(&w(false)),
            "scene: claude no longer watches main"
        );
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
        c.send(socket::Ask::Hello {
            name: "mirror".into(),
        })
        .expect("the viewer names itself");
        c.watch_scene(MAIN).expect("the watch goes");
        let mut v = Viewer::new();
        assert_eq!(next(&mut app, &mut c, &mut v), 1);
        // the shell took (and logged) the watch that began
        let port = app.access.agent.as_mut().expect("the port");
        assert!(port.take_scene_watches().is_empty(), "told already");
        // and the recording keeps it as the agent's observation
        let session = app
            .recording
            .as_ref()
            .expect("a recording")
            .recorder()
            .snapshot_window();
        let watched: Vec<_> = session
            .events()
            .iter()
            .filter_map(|e| match &e.kind {
                record::Kind::Observed { actor, scope, .. } => Some((actor.clone(), scope.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(
            watched,
            [(
                Actor::Agent {
                    name: "mirror".into()
                },
                None
            )]
        );
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
            .handle_event(&engine::widgets::prelude::WidgetEvent::MouseMove {
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
