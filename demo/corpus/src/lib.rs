//! Graphs of the gramorph corpus drawn live by plev. Two pages: `lesmis`
//! (d3-force's Les Miserables: a live simulation, drag a node and the rest
//! follow) and `compound` (cytoscape's compound nodes: drag a group and its
//! children come along, a child under the pointer is the child, not the
//! group). Pan with a drag on empty space, zoom with the wheel, click to
//! select. The engine's `App` hosts the page (`HostedView`); on the web the
//! page is `?page=`, and every few frames the timings and the screen
//! position of each node land in `<body data-plev>` for a script to read.

pub mod read;

use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::graph::live_graph::hit::Hit;
use engine::widgets::graph::live_graph::net::NetSpec;
use engine::widgets::graph::live_graph::{FrameStats, LiveGraph};
use engine::widgets::widget::{EventResult, Rect, WidgetEvent};

pub use engine::runtime::window::HostedView;

// the corpus inputs live with the repository's data, by source format
// (data/graphs/<format>/), and are embedded so the web build needs no fetch.
const LES_MIS: &str = include_str!("../../../data/graphs/d3/les-mis.json");
const COMPOUND: &str = include_str!("../../../data/graphs/cytoscape/compound.json");

/// Publish the numbers every this many frames.
const PUBLISH_EVERY: u64 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    LesMis,
    Compound,
}

impl Page {
    pub fn from_name(name: &str) -> Page {
        if name.eq_ignore_ascii_case("compound") {
            Page::Compound
        } else {
            Page::LesMis
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Page::LesMis => "lesmis",
            Page::Compound => "compound",
        }
    }

    pub fn spec(self) -> Result<NetSpec, String> {
        match self {
            Page::LesMis => read::d3(LES_MIS),
            Page::Compound => read::cytoscape(COMPOUND),
        }
    }
}

pub struct CorpusView {
    page: Page,
    pub graph: LiveGraph,
    frame: u64,
}

impl CorpusView {
    pub fn new(page: Page) -> Result<CorpusView, String> {
        Ok(CorpusView {
            page,
            graph: LiveGraph::new(page.spec()?),
            frame: 0,
        })
    }

    fn label(&self, hit: Option<Hit>) -> String {
        let spec = self.graph.spec();
        match hit {
            Some(Hit::Node(i)) => spec.nodes[i].label.clone(),
            Some(Hit::Group(g)) => spec.groups[g].label.clone(),
            Some(Hit::Edge(k)) => format!("edge {k}"),
            None => String::new(),
        }
    }

    /// The numbers a measuring script reads: timings in ms, hover and
    /// selection, each node's center and each group's box in screen px.
    pub fn report(&self) -> String {
        let summary = |list: &[f32]| {
            let (p50, p95, max) = FrameStats::summary(list);
            serde_json::json!([p50, p95, max])
        };
        let spec = self.graph.spec();
        let nodes: serde_json::Map<String, serde_json::Value> = spec
            .nodes
            .iter()
            .zip(self.graph.positions())
            .map(|(n, p)| {
                let (x, y) = self.graph.to_screen(*p);
                (n.label.clone(), serde_json::json!([x, y]))
            })
            .collect();
        let groups: serde_json::Map<String, serde_json::Value> = spec
            .groups
            .iter()
            .enumerate()
            .filter_map(|(g, group)| {
                let [x, y, w, h] = self.graph.group_box(g)?;
                let (sx, sy) = self.graph.to_screen((x, y));
                let s = self.graph.scale();
                Some((
                    group.label.clone(),
                    serde_json::json!([sx, sy, w * s, h * s]),
                ))
            })
            .collect();
        serde_json::json!({
            "page": self.page.name(),
            "frames": self.graph.stats.frames,
            "moving": self.graph.moving(),
            "tick_ms": summary(&self.graph.stats.tick_ms),
            "build_ms": summary(&self.graph.stats.build_ms),
            "hovered": self.label(self.graph.hovered()),
            "selected": self.label(self.graph.selected()),
            "nodes": nodes,
            "groups": groups,
        })
        .to_string()
    }

    fn publish(&self) {
        #[cfg(target_arch = "wasm32")]
        if let Some(body) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.body())
        {
            let _ = body.dataset().set("plev", &self.report());
        }
    }
}

impl HostedView for CorpusView {
    fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        let result = self.graph.handle_event(event, bounds);
        if result.clicked || matches!(event, WidgetEvent::MouseUp { .. }) {
            self.publish();
        }
        result
    }

    fn tick(&mut self, dt: f32) -> bool {
        self.graph.tick(dt)
    }

    fn render(&mut self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.graph.render(c, bounds, theme);
        self.frame += 1;
        if self.frame % PUBLISH_EVERY == 1 || !self.graph.moving() {
            self.publish();
        }
    }
}

/// The browser entry: the page from `?page=`, hosted by the engine's `App`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    use engine::runtime::window::{App, AppEvent, launch_args};
    use winit::platform::web::EventLoopExtWebSys;

    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);
    let page = Page::from_name(
        launch_args(&["page"])
            .first()
            .map_or("lesmis", String::as_str),
    );
    let view = match CorpusView::new(page) {
        Ok(view) => view,
        Err(e) => {
            log::error!("corpus: {e}");
            return;
        }
    };
    let event_loop = winit::event_loop::EventLoop::<AppEvent>::with_user_event()
        .build()
        .expect("event loop");
    let app = App::new_with_proxy(event_loop.create_proxy()).with_view(Box::new(view));
    event_loop.spawn_app(app);
}
