//! `uigallery --render`: one frame of a gallery page without a window,
//! through `engine::drawing::headless` (the window's own frame path into an
//! offscreen texture), the same frame as SVG with every approximation
//! reported (`engine::drawing::headless::svg`), and the page's semantic projection
//! in the form `plev observe` serves. Desktop only, like the binary that
//! calls it.

use crate::views::ShowcaseView;
use accord::Snapshot;
use engine::drawing::compositor::Compositor;
use engine::drawing::headless::{Frame, HeadlessError, Renderer, SvgExport, Viewport};

/// Animation steps a headless frame may take before it is drawn: the
/// window keeps drawing while anything moves, so the frame it settles
/// on is the one to show. A page that never settles (a spinner, a
/// blinking caret) is drawn after this many steps, deterministically.
const SETTLE_STEPS: usize = 120;

/// One animation step of a headless frame: a 60 Hz display.
const SETTLE_DT: f32 = 1.0 / 60.0;

/// Why a page could not be staged, drawn, exported or projected.
#[derive(Debug)]
pub enum RenderError {
    /// No page has this title (titles as the sidebar shows them).
    UnknownPage(String),
    /// No built-in theme has this name.
    UnknownTheme(String),
    /// The page projects no semantics yet (the todo and the surface pages do).
    NoSemantics(String),
    Headless(HeadlessError),
    Json(serde_json::Error),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownPage(p) => write!(f, "no page titled '{p}'"),
            Self::UnknownTheme(t) => write!(f, "no theme named '{t}'"),
            Self::NoSemantics(p) => write!(
                f,
                "the page '{p}' has no semantic projection yet (the todo page has one)"
            ),
            Self::Headless(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "semantics as json: {e}"),
        }
    }
}

impl std::error::Error for RenderError {}

impl From<HeadlessError> for RenderError {
    fn from(e: HeadlessError) -> Self {
        Self::Headless(e)
    }
}

impl From<serde_json::Error> for RenderError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// A gallery page staged for one headless frame: the launch state the
/// window takes from `[page] [theme] [scroll]`, laid out at a viewport
/// and animated until it settles, its scene built. The png, the svg and
/// the semantics asked of one staged page are of that one frame.
pub struct StagedPage {
    view: ShowcaseView,
    compositor: Compositor,
    steps: usize,
    settled: bool,
}

/// Stage `page` under `theme` at `viewport`, scrolled to `scroll` when
/// given. Needs no GPU: the scene is built into a plain compositor. The
/// page is the same on every machine: it offers no Claude found on the
/// host (`stage_without_host`).
pub fn stage_page(
    viewport: Viewport,
    page: &str,
    theme: &str,
    scroll: Option<f32>,
) -> Result<StagedPage, RenderError> {
    crate::views::stage_without_host();
    let mut view = ShowcaseView::new(viewport.width, viewport.height);
    view.jump_to_page(page);
    if !view.page_title().eq_ignore_ascii_case(page) {
        return Err(RenderError::UnknownPage(page.to_string()));
    }
    view.apply_theme(theme);
    if view.theme_name != theme {
        return Err(RenderError::UnknownTheme(theme.to_string()));
    }
    if let Some(offset) = scroll {
        view.scroll_page_to(offset);
    }
    view.resize(viewport.width, viewport.height, viewport.scale);

    let mut compositor = Compositor::new();
    let (mut steps, mut settled) = (0, false);
    while steps < SETTLE_STEPS && !settled {
        let moving = view.tick(if steps == 0 { 0.0 } else { SETTLE_DT });
        view.render(&mut compositor);
        steps += 1;
        settled = !moving;
    }
    Ok(StagedPage {
        view,
        compositor,
        steps,
        settled,
    })
}

impl StagedPage {
    /// The page's title as the sidebar shows it.
    pub fn title(&self) -> &'static str {
        self.view.page_title()
    }

    /// The scene of the frame, as built (read only: the regression
    /// harness names and hashes its layers and reads its bindings).
    pub fn compositor(&self) -> &Compositor {
        &self.compositor
    }

    /// Fixed animation steps taken before the frame was built; the clock
    /// never reads the wall, so the same page stages the same frame.
    pub fn steps(&self) -> usize {
        self.steps
    }

    /// `false` when the page still moved after the last step (a spinner,
    /// a blinking caret): its frame is the one at that fixed step.
    pub fn settled(&self) -> bool {
        self.settled
    }

    /// Draw the frame by the window's own frame path into the renderer's
    /// target and read it back.
    pub fn render(&mut self, renderer: &mut Renderer) -> Result<Frame, RenderError> {
        let clear = self.view.theme.colors.bg.to_linear_array();
        Ok(renderer.render(&mut self.compositor, clear)?)
    }

    /// The frame as SVG, every approximation on the report. The glyphs
    /// are the renderer's shaping, the same [`render`](Self::render)
    /// draws from, so the svg of a drawn frame holds the glyphs the png
    /// shows.
    pub fn svg(&self, renderer: &mut Renderer) -> Result<SvgExport, RenderError> {
        let background = self.view.theme.colors.bg.to_array();
        Ok(renderer.svg(&self.compositor, Some(background))?)
    }

    /// What the page means, projected as it stands in this frame; an
    /// error for a page without a projection.
    pub fn semantics(&self) -> Result<Snapshot, RenderError> {
        self.view
            .semantics()
            .ok_or_else(|| RenderError::NoSemantics(self.title().to_string()))
    }
}

/// The lean serde form of `sem` (docs/adr/semantics/0040-lean-semantic-form-and-one-redaction.md):
/// the nodes in projection order, and the focus when there is one.
#[derive(serde::Serialize)]
struct SnapshotJson<'a> {
    nodes: Vec<accord::Node>,
    #[serde(skip_serializing_if = "Option::is_none")]
    focused: Option<&'a accord::SemId>,
}

/// `snapshot` as pretty JSON in the serde form of `sem`, the form an agent
/// receives from `plev observe`: each node's `source` (the code that
/// projected it) leaves unless `source` asks for it, as the agent port
/// does. With `source` the text is the snapshot's own serialization.
pub fn semantics_json(snapshot: &Snapshot, source: bool) -> Result<String, serde_json::Error> {
    if source {
        return serde_json::to_string_pretty(snapshot);
    }
    let nodes = snapshot
        .nodes()
        .values()
        .map(|node| {
            let mut node = node.clone();
            node.source = None;
            node
        })
        .collect();
    serde_json::to_string_pretty(&SnapshotJson {
        nodes,
        focused: snapshot.focused(),
    })
}

/// Draw one frame of a gallery page without a window: [`stage_page`] at
/// the renderer's viewport, then [`StagedPage::render`].
pub fn render_page(
    renderer: &mut Renderer,
    page: &str,
    theme: &str,
    scroll: Option<f32>,
) -> Result<Frame, RenderError> {
    stage_page(renderer.viewport(), page, theme, scroll)?.render(renderer)
}

/// Every page through the window's frame path, no window. The GPU
/// tests return early when the machine has no adapter.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::PAGES;
    use engine::drawing::gpu::RenderConfig;
    use engine::drawing::headless::Viewport;

    /// The launch theme and the dark glass theme whose page background
    /// is the documented 48,48,48.
    const THEMES: [&str; 2] = ["core", "hoff"];

    const PHONE: Viewport = Viewport {
        width: 390.0,
        height: 844.0,
        scale: 2.0,
    };
    const DESKTOP: Viewport = Viewport {
        width: 1280.0,
        height: 800.0,
        scale: 1.0,
    };

    fn renderer(viewport: Viewport) -> Option<Renderer> {
        match Renderer::new(viewport, RenderConfig::default()) {
            Ok(r) => Some(r),
            Err(HeadlessError::NoAdapter) => {
                eprintln!("no GPU adapter; skipping");
                None
            }
            Err(e) => panic!("{e}"),
        }
    }

    fn page(r: &mut Renderer, title: &str, theme: &str) -> Frame {
        match render_page(r, title, theme, None) {
            Ok(frame) => frame,
            Err(e) => panic!("{title} / {theme}: {e}"),
        }
    }

    /// The theme background as the 8-bit sRGB a frame holds.
    fn background(theme: &str) -> [u8; 3] {
        let Some(t) = engine::styling::theme::Theme::named(theme) else {
            panic!("no theme {theme}");
        };
        let [r, g, b, _] = t.colors.bg.to_array();
        [r, g, b].map(|c| (c * 255.0).round() as u8)
    }

    #[test]
    fn every_page_renders_in_both_main_themes_at_phone_and_desktop_width() {
        for viewport in [PHONE, DESKTOP] {
            let Some(mut r) = renderer(viewport) else {
                return;
            };
            for theme in THEMES {
                let bg = background(theme);
                for p in PAGES {
                    let frame = page(&mut r, p.title, theme);
                    assert_eq!((frame.width, frame.height), viewport.physical());
                    // the chrome alone (the page header, the sidebar or
                    // its menu button) covers more than a quarter of a
                    // percent of the frame; the sparsest page, a
                    // breadcrumb on a light phone, is at 0.9
                    // every fourth pixel: a debug build counts slowly
                    let drawn = frame
                        .rgba
                        .chunks_exact(4)
                        .step_by(4)
                        .filter(|px| (0..3).any(|i| px[i].abs_diff(bg[i]) > 2))
                        .count();
                    let total = (frame.width * frame.height) as usize / 4;
                    assert!(
                        drawn * 400 > total,
                        "{} / {theme} at {}px: only {drawn} of {total} pixels \
                         differ from the background",
                        p.title,
                        viewport.width
                    );
                }
            }
        }
    }

    /// The documented sample: the hoff page background reads 48,48,48,
    /// in the page column at desktop width.
    #[test]
    fn the_hoff_page_background_measures_48() {
        let desktop = Viewport::new(1280.0, 800.0, 2.0);
        let Some(mut r) = renderer(desktop) else {
            return;
        };
        let frame = page(&mut r, "Card", "hoff");
        for (x, y) in [(2559, 0), (2559, 1599), (1280, 5)] {
            assert_eq!(frame.pixel(x, y), Some([48, 48, 48, 255]), "({x}, {y})");
        }
    }

    /// Glass cards, backdrop blur and text: two fresh renderers give the
    /// same bytes, and a renderer that drew other pages first draws
    /// this one identically too.
    #[test]
    fn a_page_renders_to_the_same_bytes_every_time() {
        let Some(mut a) = renderer(DESKTOP) else {
            return;
        };
        let Some(mut b) = renderer(DESKTOP) else {
            return;
        };
        let first = page(&mut a, "Card", "hoff");
        let second = page(&mut b, "Card", "hoff");
        assert!(first.rgba == second.rgba, "two fresh renders differ");
        match (first.encode_png(), second.encode_png()) {
            (Ok(x), Ok(y)) => assert!(x == y, "the PNG bytes differ"),
            (Err(e), _) | (_, Err(e)) => panic!("{e}"),
        }
        page(&mut a, "Typography", "core");
        page(&mut a, "Effects", "hoff");
        let again = page(&mut a, "Card", "hoff");
        assert!(
            first.rgba == again.rgba,
            "a renderer that drew other pages first drew Card differently"
        );
    }

    #[test]
    fn an_unknown_page_or_theme_is_an_error() {
        let Some(mut r) = renderer(Viewport::new(320.0, 240.0, 1.0)) else {
            return;
        };
        assert!(matches!(
            render_page(&mut r, "No Such Page", "core", None),
            Err(RenderError::UnknownPage(_))
        ));
        assert!(matches!(
            render_page(&mut r, "Button", "neon", None),
            Err(RenderError::UnknownTheme(_))
        ));
    }

    /// Titles match as the window's launch argument does, whatever
    /// the case.
    #[test]
    fn a_page_title_matches_case_insensitively() {
        let Some(mut r) = renderer(Viewport::new(320.0, 240.0, 1.0)) else {
            return;
        };
        let upper = page(&mut r, "DATA TABLE", "core");
        let exact = page(&mut r, "Data Table", "core");
        assert!(upper.rgba == exact.rgba);
    }

    fn staged(title: &str, theme: &str, viewport: Viewport) -> StagedPage {
        match stage_page(viewport, title, theme, None) {
            Ok(staged) => staged,
            Err(e) => panic!("{title} / {theme}: {e}"),
        }
    }

    /// Three pages as SVG, from the same frame as their png: the file is
    /// the viewport's size, holds text as labelled glyph groups, and the
    /// svg importer reads it back as drawable content.
    #[test]
    fn three_pages_export_svg_the_importer_reads_back() {
        let Some(mut r) = renderer(DESKTOP) else {
            return;
        };
        for (title, theme) in [("Todo", "hoff"), ("Button", "core"), ("Card", "hoff")] {
            let mut staged = staged(title, theme, DESKTOP);
            let before = match staged.svg(&mut r) {
                Ok(e) => e,
                Err(e) => panic!("{title}: {e}"),
            };
            let frame = match staged.render(&mut r) {
                Ok(f) => f,
                Err(e) => panic!("{title}: {e}"),
            };
            assert_eq!((frame.width, frame.height), DESKTOP.physical());
            let export = match staged.svg(&mut r) {
                Ok(e) => e,
                Err(e) => panic!("{title}: {e}"),
            };
            assert_eq!(before, export, "{title}: the svg changed with the draw");
            let svg = &export.svg;
            assert!(
                svg.contains(" width=\"1280\" height=\"800\" viewBox=\"0 0 1280 800\">"),
                "{title}: viewBox"
            );
            let labels = svg.matches("<g aria-label=\"").count();
            assert!(labels > 3, "{title}: {labels} text runs");
            assert!(
                svg.contains(&format!("aria-label=\"{title}\"")),
                "{title}: header"
            );
            assert!(svg.matches("<rect ").count() > 3, "{title}: rects");
            assert_eq!(
                svg.matches("<g").count(),
                svg.matches("</g>").count(),
                "{title}"
            );
            match ::import::formats::svg::convert(svg, title) {
                Ok((_, stats)) => {
                    assert_eq!((stats.width, stats.height), (1280.0, 800.0), "{title}");
                    assert!(stats.paths > labels, "{title}: {stats:?}");
                }
                Err(e) => panic!("{title}: the importer refused the export: {e}"),
            }
        }
    }

    /// The effects page shows shadows, gradients and a backdrop blur; each
    /// one is on the report, and a report line names its node.
    #[test]
    fn the_effects_page_reports_its_shadows_gradients_and_blur() {
        // Tall enough for the compositor group, under the depths.
        let tall = Viewport {
            height: 1600.0,
            ..DESKTOP
        };
        let Some(mut r) = renderer(tall) else {
            return;
        };
        let export = match staged("Effects", "hoff", tall).svg(&mut r) {
            Ok(e) => e,
            Err(e) => panic!("{e}"),
        };
        let text = export.report_text();
        for needle in ["drop shadow", "inset shadow", "gradient", "backdrop blur"] {
            assert!(text.contains(needle), "the report lacks {needle}:\n{text}");
        }
        assert!(text.lines().all(|l| l.starts_with("layer ")), "{text}");
        assert_eq!(text.lines().count(), export.report.len());
    }

    /// `--semantics` of the todo is its projection in process: with
    /// `--source` the snapshot's own serialization, byte for byte; without,
    /// the same nodes and the same revision, no `source` on any node.
    #[test]
    fn the_todo_semantics_json_is_the_projection_in_process() {
        let staged = staged("Todo", "core", DESKTOP);
        let snapshot = match staged.semantics() {
            Ok(s) => s,
            Err(e) => panic!("{e}"),
        };
        assert!(!snapshot.is_empty());
        let full = match serde_json::to_string_pretty(&snapshot) {
            Ok(s) => s,
            Err(e) => panic!("{e}"),
        };
        assert_eq!(semantics_json(&snapshot, true).ok(), Some(full.clone()));
        assert!(
            full.contains("\"source\""),
            "the projection records no source"
        );

        let lean = match semantics_json(&snapshot, false) {
            Ok(s) => s,
            Err(e) => panic!("{e}"),
        };
        assert!(!lean.contains("\"source\""), "{lean}");
        assert!(lean.len() < full.len());
        let back: Snapshot = match serde_json::from_str(&lean) {
            Ok(s) => s,
            Err(e) => panic!("{e}\n{lean}"),
        };
        assert_eq!(back.revision(), snapshot.revision());
        assert_eq!(back.len(), snapshot.len());
        assert!(
            back.nodes()
                .keys()
                .zip(snapshot.nodes().keys())
                .all(|(a, b)| a == b),
            "the node order changed"
        );
        assert_eq!(back.focused(), snapshot.focused());
        // the same page staged again projects the same revision
        let again = staged.semantics().map(|s| s.revision()).ok();
        assert_eq!(again, Some(snapshot.revision()));
    }

    /// The sidebar is the same widget on every page, so its labels export
    /// the same glyph outlines on every page. A shared swash scale context
    /// once handed the Card page's sidebar the wrong glyphs (stale per-font
    /// state across sizes); this pins the fix.
    #[test]
    fn the_same_text_run_exports_the_same_outlines_on_every_page() {
        let Some(mut r) = renderer(DESKTOP) else {
            return;
        };
        let group = |svg: &str, label: &str| -> String {
            let start = svg
                .find(&format!("<g aria-label=\"{label}\""))
                .unwrap_or_else(|| panic!("no run labelled {label}"));
            let end = svg[start..].find("</g>").map_or(svg.len(), |i| start + i);
            svg[start..end].to_string()
        };
        let mut runs: Vec<(String, String)> = Vec::new();
        for title in ["Card", "Effects", "Typography"] {
            let export = match staged(title, "hoff", DESKTOP).svg(&mut r) {
                Ok(e) => e,
                Err(e) => panic!("{title}: {e}"),
            };
            runs.push((
                group(&export.svg, "Foundations"),
                group(&export.svg, "Accordion"),
            ));
        }
        for (label, at) in [("Foundations", 0usize), ("Accordion", 1)] {
            let first = if at == 0 { &runs[0].0 } else { &runs[0].1 };
            assert!(first.matches("<path d=").count() >= 9, "{label}: {first}");
            for (title, run) in ["Effects", "Typography"].iter().zip(&runs[1..]) {
                let other = if at == 0 { &run.0 } else { &run.1 };
                assert_eq!(first, other, "{label} differs between Card and {title}");
            }
        }
    }

    /// The staging clock is fixed steps, never the wall: the same page
    /// staged twice holds the same scene node for node, after the same
    /// steps. The regression harness hashes that scene (task-67).
    #[test]
    fn a_page_stages_the_same_scene_every_time() {
        for (title, theme) in [("Todo", "core"), ("Card", "hoff"), ("Spinner", "core")] {
            let first = staged(title, theme, DESKTOP);
            let second = staged(title, theme, DESKTOP);
            assert!(
                first.settled(),
                "{title} still moves after {} steps",
                first.steps()
            );
            assert_eq!(first.steps(), second.steps(), "{title}");
            let layers = |s: &StagedPage| {
                s.compositor()
                    .layers()
                    .iter()
                    .map(|l| (l.id, l.z_order, l.nodes().to_vec()))
                    .collect::<Vec<_>>()
            };
            assert!(
                layers(&first) == layers(&second),
                "{title}: the scene moved"
            );
            assert!(!layers(&first).is_empty());
        }
    }

    #[test]
    fn a_page_without_a_projection_is_an_error_not_an_empty_file() {
        let staged = staged("Separator", "core", DESKTOP);
        match staged.semantics() {
            Err(RenderError::NoSemantics(page)) => assert_eq!(page, "Separator"),
            other => panic!("expected NoSemantics, got {:?}", other.map(|s| s.len())),
        }
        assert!(
            RenderError::NoSemantics("Button".to_string())
                .to_string()
                .contains("no semantic projection")
        );
    }
}
