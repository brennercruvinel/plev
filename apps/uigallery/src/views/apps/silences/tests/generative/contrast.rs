//! The contrast of the generated review measured as the window shows it
//! (task-70 phase 4), the way task-75 measures the four themes
//! (`rust/engine/src/styling/theme/tests/contrast.rs`): every text run the
//! surface draws, its ink composed in linear light over what it sits on
//! (the fills drawn under it, in order, over the page's `bg`), the
//! relative luminance of WCAG, and the ratio against 4.5 (3 for large
//! text: 24 px, or 18.66 px bold). Before and with "more contrast", pair
//! by pair, in the core theme. No theme's color changes; what no token of
//! the IR reaches is listed with why.

use std::collections::BTreeSet;

use engine::drawing::compositor::{Compositor, SceneNode};
use engine::styling::theme::Theme;
use engine::widgets::prelude::Rect;
use plevir::{Component, Preference, Preferences};
use screen::NoFrames;

use super::super::super::SilencesPage;
use super::super::{setup, tester};
use super::access::review;
use super::show;

const CLAUDE: &str = "../../rust/plevir/tests/fixtures/silences-review-claude.ir.json";

/// WCAG 2.2 AA: text (1.4.3).
const TEXT: f64 = 4.5;
/// WCAG 2.2 AA: large text (1.4.3).
const LARGE: f64 = 3.0;

/// One channel from sRGB to linear light, as `Color::to_linear_array`.
fn linear(c: f32) -> f64 {
    let c = f64::from(c);
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// `fg` composed over the opaque linear color `under`, in linear light.
fn over(fg: [f32; 4], under: [f64; 3]) -> [f64; 3] {
    let a = f64::from(fg[3]);
    let [r, g, b] = [fg[0], fg[1], fg[2]].map(linear);
    [
        r * a + under[0] * (1.0 - a),
        g * a + under[1] * (1.0 - a),
        b * a + under[2] * (1.0 - a),
    ]
}

/// WCAG relative luminance of a linear color.
fn luminance([r, g, b]: [f64; 3]) -> f64 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// WCAG contrast ratio, lighter over darker.
fn ratio(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// sRGB hex of a linear color, for the record.
fn hex([r, g, b]: [f64; 3]) -> String {
    let back = |c: f64| {
        let s = if c <= 0.003_130_8 {
            c * 12.92
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (s * 255.0).round().clamp(0.0, 255.0) as u8
    };
    format!("#{:02X}{:02X}{:02X}", back(r), back(g), back(b))
}

/// One text run as the window shows it.
#[derive(Clone, Debug)]
struct Pair {
    text: String,
    size: f32,
    fg: String,
    bg: String,
    need: f64,
    ratio: f64,
}

impl Pair {
    fn passes(&self) -> bool {
        self.ratio >= self.need
    }

    /// The pair in a line of the record: text, size, ink over fill, ratio.
    fn line(&self) -> String {
        format!(
            "{:<28} {:>4.1} px  {} on {}  {:>5.2} (need {})",
            format!("{:?}", self.text),
            self.size,
            self.fg,
            self.bg,
            self.ratio,
            self.need
        )
    }
}

/// A fill drawn: where, and its color (a gradient by both ends).
struct Fill {
    rect: [f32; 4],
    colors: Vec<[f32; 4]>,
}

fn inside([x, y, w, h]: [f32; 4], (px, py): (f32, f32)) -> bool {
    px >= x && px <= x + w && py >= y && py <= y + h
}

/// Every text run of `c` over the page's `bg` of `theme`, in draw order:
/// its ink over every fill drawn before it under its first line, the
/// worse end of a gradient.
fn pairs(c: &Compositor, theme: &Theme) -> Vec<Pair> {
    let page = over(theme.colors.bg.0, [0.0; 3]);
    let mut fills: Vec<Fill> = Vec::new();
    let mut out = Vec::new();
    for layer in c.layers() {
        for node in layer.nodes() {
            match node {
                SceneNode::Rect { x, y, w, h, color } => fills.push(Fill {
                    rect: [*x, *y, *w, *h],
                    colors: vec![*color],
                }),
                SceneNode::RoundedRect {
                    x, y, w, h, color, ..
                } => fills.push(Fill {
                    rect: [*x, *y, *w, *h],
                    colors: vec![*color],
                }),
                SceneNode::GradientRect {
                    x,
                    y,
                    w,
                    h,
                    color,
                    color2,
                    ..
                } => fills.push(Fill {
                    rect: [*x, *y, *w, *h],
                    colors: vec![*color, *color2],
                }),
                SceneNode::Text { key, x, y, color } => {
                    let size = f32::from_bits(key.font_size_bits);
                    let line = f32::from_bits(key.line_height_bits);
                    let at = (x + 1.0, y + line / 2.0);
                    let mut worst: Option<([f64; 3], [f64; 3], f64)> = None;
                    let under = fills.iter().filter(|f| inside(f.rect, at));
                    let mut grounds = vec![page];
                    for fill in under {
                        grounds = grounds
                            .iter()
                            .flat_map(|g| fill.colors.iter().map(move |c| over(*c, *g)))
                            .collect();
                    }
                    for ground in grounds {
                        let ink = over(*color, ground);
                        let r = ratio(ink, ground);
                        if worst.is_none_or(|(_, _, w)| r < w) {
                            worst = Some((ink, ground, r));
                        }
                    }
                    let Some((ink, ground, r)) = worst else {
                        continue;
                    };
                    let bold = key.font_weight >= 700;
                    let large = size >= 24.0 || (bold && size >= 18.66);
                    out.push(Pair {
                        text: key.text.clone(),
                        size,
                        fg: hex(ink),
                        bg: hex(ground),
                        need: if large { LARGE } else { TEXT },
                        ratio: r,
                    });
                }
                _ => {}
            }
        }
    }
    out
}

#[test]
fn the_measure_is_task_75_s() {
    let white = over([1.0; 4], [0.0; 3]);
    let black = over([0.0, 0.0, 0.0, 1.0], [0.0; 3]);
    assert!((ratio(white, black) - 21.0).abs() < 1e-9);
    // the core's secondary ink, #7B7B7B, on its paper, #FCFCFC: 4.13, as
    // the engine's contrast test records it
    let theme = Theme::hoff_core();
    let paper = over(theme.colors.bg.0, [0.0; 3]);
    let mid = over(theme.colors.text_mid.0, paper);
    assert!(
        (ratio(mid, paper) - 4.13).abs() < 0.01,
        "{}",
        ratio(mid, paper)
    );
    assert_eq!(hex(paper), "#FCFCFC");
}

/// The surface open, drawn alone in `theme`, its images as their shells:
/// every text run it draws.
fn measured(page: &mut SilencesPage, theme: &Theme) -> Vec<Pair> {
    let id = page.generated.id.clone().expect("open");
    let surface = page.generated.host.get_mut(&id).expect("open");
    let mut c = Compositor::new();
    let area = Rect::new(0.0, 0.0, 620.0, 2000.0);
    surface.render_with(&mut c, area, theme, &NoFrames);
    pairs(&c, theme)
}

/// The labels of the stats of the surface open: comps draws them.
fn stat_labels(page: &SilencesPage) -> BTreeSet<String> {
    let id = page.generated.id.clone().expect("open");
    let surface = page.generated.host.get(&id).expect("open");
    let mut out = BTreeSet::new();
    let mut stack = vec![&surface.verified().ir().surface.root];
    while let Some(n) = stack.pop() {
        if n.component == Some(Component::Stat) {
            out.extend(n.label.clone());
        }
        stack.extend(n.children.iter());
        stack.extend(n.repeat.as_ref().map(|r| &r.item));
    }
    out
}

/// The contrast of the review by the rules and of the Claude's, pair by
/// pair in the core theme: before, the badges, the images' labels and
/// the sliders' labels fail AA; with "more contrast", every pair passes
/// but the stats' labels, which comps draws in `text_dim` and no role of
/// the IR reaches (4.13 on the paper). Every ratio is in the record.
#[test]
fn more_contrast_measured_pair_by_pair_in_core() {
    let theme = Theme::hoff_core();
    for claude in [false, true] {
        let (_, mut page) = if claude {
            let (todo, mut page, _) = setup(false);
            show(&mut page, CLAUDE);
            (todo, page)
        } else {
            review()
        };
        let name = if claude { "the Claude's" } else { "the rules'" };
        let stats = stat_labels(&page);
        let before = measured(&mut page, &theme);
        let app = page.snapshot().unwrap();
        let on = Preferences::default().with(Preference::MoreContrast, true);
        page.generated.host.prefer(on, tester(), &app).unwrap();
        let after = measured(&mut page, &theme);
        eprintln!("{name} review, core, before and with more contrast:");
        for (b, a) in before.iter().zip(&after) {
            eprintln!("  {}\n  -> {:.2}", b.line(), a.ratio);
        }
        assert_eq!(before.len(), after.len(), "{name}: the same runs");
        let failing = |pairs: &[Pair]| -> Vec<Pair> {
            pairs.iter().filter(|p| !p.passes()).cloned().collect()
        };
        // what no token reaches: a stat's label, in `text_dim`
        for p in failing(&after) {
            assert!(stats.contains(&p.text), "{name}: {}", p.line());
            assert_eq!(p.fg, "#7B7B7B", "{name}: {}", p.line());
            assert!((p.ratio - 4.13).abs() < 0.01, "{}", p.line());
        }
        // what it made pass, with the ratio before and after
        let fixed: Vec<(&Pair, &Pair)> = before
            .iter()
            .zip(&after)
            .filter(|(b, a)| !b.passes() && a.passes())
            .collect();
        assert!(!fixed.is_empty(), "{name}");
        for (b, a) in &fixed {
            let was = [3.96, 4.13].iter().any(|r| (b.ratio - r).abs() < 0.01);
            assert!(was, "{name}: {}", b.line());
            let is = [17.96, 18.26].iter().any(|r| (a.ratio - r).abs() < 0.01);
            assert!(is, "{name}: {}", a.line());
        }
        // the badge filled: its text in contrast to the accent
        let filled = after
            .iter()
            .find(|p| p.bg == "#121212")
            .expect("the filled badge");
        assert_eq!(
            (filled.text.as_str(), filled.fg.as_str()),
            ("pending", "#FAFAFC")
        );
        // nothing that passed stops passing
        for (b, a) in before.iter().zip(&after) {
            assert!(!b.passes() || a.passes(), "{name}: {}", a.line());
        }
    }
}
