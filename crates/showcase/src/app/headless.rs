//! `showcase --render`: one frame of a gallery page without a window,
//! through `engine::headless` (the window's own frame path into an
//! offscreen texture). Desktop only, like the binary that calls it.

use crate::view::ShowcaseView;
use engine::compositor::Compositor;
use engine::headless::{Frame, HeadlessError, Renderer};

/// Animation steps a headless frame may take before it is drawn: the
/// window keeps drawing while anything moves, so the frame it settles
/// on is the one to show. A page that never settles (a spinner, a
/// blinking caret) is drawn after this many steps, deterministically.
const SETTLE_STEPS: usize = 120;

/// One animation step of a headless frame: a 60 Hz display.
const SETTLE_DT: f32 = 1.0 / 60.0;

/// Why [`render_page`] drew nothing.
#[derive(Debug)]
pub enum RenderError {
    /// No page has this title (titles as the sidebar shows them).
    UnknownPage(String),
    /// No built-in theme has this name.
    UnknownTheme(String),
    Headless(HeadlessError),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownPage(p) => write!(f, "no page titled '{p}'"),
            Self::UnknownTheme(t) => write!(f, "no theme named '{t}'"),
            Self::Headless(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for RenderError {}

impl From<HeadlessError> for RenderError {
    fn from(e: HeadlessError) -> Self {
        Self::Headless(e)
    }
}

/// Draw one frame of a gallery page without a window: the launch state
/// the window takes from `[page] [theme] [scroll]`, laid out at the
/// renderer's viewport, animated until it settles, then drawn by the
/// window's own frame path into the renderer's target and read back.
pub fn render_page(
    renderer: &mut Renderer,
    page: &str,
    theme: &str,
    scroll: Option<f32>,
) -> Result<Frame, RenderError> {
    let v = renderer.viewport();
    let mut view = ShowcaseView::new(v.width, v.height);
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
    view.resize(v.width, v.height, v.scale);

    let mut compositor = Compositor::new();
    for step in 0..SETTLE_STEPS {
        let moving = view.tick(if step == 0 { 0.0 } else { SETTLE_DT });
        view.render(&mut compositor);
        if !moving {
            break;
        }
    }
    let clear = view.theme.colors.bg.to_linear_array();
    Ok(renderer.render(&mut compositor, clear)?)
}

/// Every page through the window's frame path, no window. The GPU
/// tests return early when the machine has no adapter.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::PAGES;
    use engine::gpu::RenderConfig;
    use engine::headless::Viewport;

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
        let Some(t) = engine::theme::Theme::named(theme) else {
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
}
