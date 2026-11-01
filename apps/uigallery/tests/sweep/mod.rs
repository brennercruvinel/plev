//! The variable-content sweep (ADR 0086): every page of the gallery drawn
//! whole under one condition (a text size, a width), and what of its
//! frame breaks the layout. `text_scale.rs` runs it.

pub mod low;

use accord::SemId;
use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::{ControlSize, Prefs, TextScale};
use uigallery::views::{PAGES, ShowcaseView};

/// WCAG 2.5.8's minimum, in logical px.
const MIN: f32 = 24.0;
/// The widths looked at: a phone held upright and the kit's desktop.
pub const WIDTHS: [f32; 2] = [360.0, 1280.0];
/// The window a page opens in before it is grown to its whole height.
const OPEN_H: f32 = 800.0;
/// The tallest frame drawn: past it, a page is looked at to here.
const MAX_H: f32 = 16000.0;
/// Half a pixel of rounding either way.
const EPS: f32 = 0.5;

/// One condition of the sweep.
pub struct Case {
    pub prefs: Prefs,
    pub widths: &'static [f32],
    /// Every text as a translation 1.4 times as long
    /// (`engine::drawing::text::pseudo`).
    pub pseudo: bool,
    pub label: String,
}

impl Case {
    /// The person's text size `factor`, at both widths.
    pub fn scales(factor: f32) -> Self {
        Self {
            prefs: Prefs::default().with_text_scale(TextScale::new(factor)),
            widths: &WIDTHS,
            pseudo: false,
            label: format!("text x{factor}"),
        }
    }

    /// The same, every text pseudo-localized.
    pub fn pseudo(mut self) -> Self {
        self.pseudo = true;
        self.label.push_str(" pseudo");
        self
    }
}

type Rect4 = [f32; 4];

fn right(r: Rect4) -> f32 {
    r[0] + r[2]
}

fn bottom(r: Rect4) -> f32 {
    r[1] + r[3]
}

fn intersect(a: Rect4, b: Rect4) -> Option<Rect4> {
    let (x, y) = (a[0].max(b[0]), a[1].max(b[1]));
    let (r, btm) = (right(a).min(right(b)), bottom(a).min(bottom(b)));
    (r - x > EPS && btm - y > EPS).then_some([x, y, r - x, btm - y])
}

fn holds(outer: Rect4, inner: Rect4) -> bool {
    inner[0] >= outer[0] - EPS
        && inner[1] >= outer[1] - EPS
        && right(inner) <= right(outer) + EPS
        && bottom(inner) <= bottom(outer) + EPS
}

/// The style a text node was laid out with.
fn style_of(key: &TextNodeKey) -> TextStyle {
    let mut style = TextStyle::new(f32::from_bits(key.font_size_bits))
        .with_line_height(f32::from_bits(key.line_height_bits))
        .with_weight(key.font_weight)
        .with_letter_spacing(f32::from_bits(key.letter_spacing_bits))
        .with_overflow_wrap(key.overflow_wrap);
    if let Some(family) = &key.font_family {
        style = style.with_family(family);
    }
    style
}

/// A text of the frame: what it says, where it lies whole, the width it
/// was given, and the clip it is drawn under.
struct Shown {
    text: String,
    rect: Rect4,
    max_w: Option<f32>,
    clip: Rect4,
}

/// Every text node of the frame, with the clip stack it is drawn under
/// (the window at the bottom of it).
fn texts(c: &Compositor, window: Rect4) -> Vec<Shown> {
    let mut out = Vec::new();
    for layer in c.layers() {
        if !layer.visible || layer.opacity <= 0.0 {
            continue;
        }
        let mut clips = vec![window];
        for node in layer.nodes() {
            match node {
                SceneNode::PushClip { x, y, w, h } => {
                    let top = *clips.last().unwrap_or(&window);
                    clips.push(intersect(top, [*x, *y, *w, *h]).unwrap_or([*x, *y, 0.0, 0.0]));
                }
                SceneNode::PopClip => {
                    if clips.len() > 1 {
                        clips.pop();
                    }
                }
                SceneNode::Text { key, x, y, color } if color[3] > 0.0 => {
                    if key.text.trim().is_empty() {
                        continue;
                    }
                    let max_w = key.max_width_bits.map(f32::from_bits);
                    let (w, h) = TextMeasurer::measure_styled(&key.text, &style_of(key), max_w);
                    out.push(Shown {
                        text: key.text.clone(),
                        rect: [*x, *y, w, h],
                        max_w,
                        clip: *clips.last().unwrap_or(&window),
                    });
                }
                _ => {}
            }
        }
    }
    out
}

/// An actionable node of the page and the rect it binds.
struct Target {
    id: SemId,
    rect: Rect4,
    /// Bound by its part inside a scroller (a row half scrolled out): it
    /// is short, and its text runs past it, because it is cut.
    cut: bool,
}

/// A rect its parent binds by its part inside the viewport: it touches
/// the parent's edge on the axis it is short on (`hit_targets`).
fn cut_by(r: Rect4, p: Rect4, tall: f32) -> bool {
    let near = |a: f32, b: f32| (a - b).abs() < EPS;
    let (top, btm) = (near(r[1], p[1]), near(bottom(r), bottom(p)));
    let (start, end) = (near(r[0], p[0]), near(right(r), right(p)));
    (r[3] < tall && (top || btm)) || (r[2] < MIN && (start || end))
}

fn sidebar(id: &SemId) -> bool {
    id.as_str().starts_with("shell.sidebar")
}

/// The enabled, visible actionable nodes the frame binds, the sidebar's
/// left out.
fn targets(view: &ShowcaseView, c: &Compositor) -> Vec<Target> {
    let Some(snapshot) = view.reader_semantics() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (id, node) in snapshot.nodes() {
        let actionable = node.role.is_interactive() || !node.actions.is_empty();
        if !actionable || !node.enabled || node.inert || sidebar(id) {
            continue;
        }
        let Some(b) = c.binding(id) else { continue };
        if b.visible_rect().is_none() {
            continue;
        }
        let parent = node.parent.as_ref().and_then(|p| c.binding(p));
        // a row is short of its whole height when cut: of the 24 px a
        // target needs, or of the line its text takes
        let tall = MIN.max(view.theme.control.height(ControlSize::Xs));
        out.push(Target {
            id: id.clone(),
            rect: b.rect,
            cut: parent.is_some_and(|p| cut_by(b.rect, p.rect, tall)),
        });
    }
    out
}

/// The page `title` drawn whole at `width` under `case`.
fn frame(title: &str, width: f32, case: &Case) -> (ShowcaseView, Compositor) {
    engine::drawing::text::pseudo::set_pseudo_locale(case.pseudo);
    let mut view = ShowcaseView::new(width, OPEN_H);
    view.set_prefs(case.prefs);
    view.jump_to_page(title);
    let mut c = Compositor::new();
    view.render(&mut c);
    let tall = view.page_height().min(MAX_H);
    if tall > OPEN_H + EPS {
        view.resize(width, tall, 1.0);
        c = Compositor::new();
        view.render(&mut c);
    }
    (view, c)
}

fn short(text: &str) -> String {
    let mut s: String = text.chars().take(32).collect();
    if s.len() < text.len() {
        s.push('…');
    }
    s.replace('\n', " ")
}

fn fmt(r: Rect4) -> String {
    format!("[{:.0},{:.0} {:.0}x{:.0}]", r[0], r[1], r[2], r[3])
}

/// What breaks the layout of `title` at `width` under `case`, a line each.
pub fn breaks(title: &str, width: f32, case: &Case) -> Vec<String> {
    let (view, c) = frame(title, width, case);
    let window = [0.0, 0.0, view.width, view.height];
    let at = format!("{title} @{width} {}", case.label);
    let mut out = Vec::new();
    let shown = texts(&c, window);
    let targets = targets(&view, &c);
    engine::drawing::text::pseudo::set_pseudo_locale(false);
    for t in &shown {
        // wholly outside what shows: scrolled away, or the peeking sidebar
        if intersect(t.rect, t.clip).is_none() {
            continue;
        }
        if let Some(max_w) = t.max_w
            && t.rect[2] > max_w + EPS
        {
            out.push(format!(
                "{at}: '{}' {:.0} wide in {max_w:.0}",
                short(&t.text),
                t.rect[2]
            ));
        } else if t.rect[0] < t.clip[0] - EPS || right(t.rect) > right(t.clip) + EPS {
            out.push(format!(
                "{at}: '{}' {} cut across by {}",
                short(&t.text),
                fmt(t.rect),
                fmt(t.clip)
            ));
        }
    }
    for t in targets.iter().filter(|t| !t.cut) {
        let [_, _, w, h] = t.rect;
        if w < MIN - 0.01 || h < MIN - 0.01 {
            out.push(format!("{at}: {} {w:.1}x{h:.1} under {MIN}", t.id));
        }
        for s in &shown {
            let start = [s.rect[0] + EPS, s.rect[1] + s.rect[3] / 2.0];
            let inside = start[0] >= t.rect[0]
                && start[0] <= right(t.rect)
                && start[1] >= t.rect[1]
                && start[1] <= bottom(t.rect);
            let seen = intersect(s.rect, s.clip).unwrap_or(s.rect);
            if inside && !holds(t.rect, seen) {
                out.push(format!(
                    "{at}: {} {} spills '{}' {}",
                    t.id,
                    fmt(t.rect),
                    short(&s.text),
                    fmt(seen)
                ));
            }
        }
    }
    for (i, a) in targets.iter().enumerate() {
        for b in &targets[i + 1..] {
            if intersect(a.rect, b.rect).is_some()
                && !holds(a.rect, b.rect)
                && !holds(b.rect, a.rect)
            {
                out.push(format!(
                    "{at}: {} {} overlaps {} {}",
                    a.id,
                    fmt(a.rect),
                    b.id,
                    fmt(b.rect)
                ));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// What the sweep finds that is known and has an owner, as `(page,
/// part of the line, why)`: a line of that page holding that part is
/// left out of the failure (and printed). Each entry is a fix waiting in
/// its front; a new break, or one of these spreading to another page,
/// still fails.
const KNOWN: &[(&str, &str, &str)] = &[
    // at every size: targets under 24 px (front C, A, B; hit_targets
    // looks at the first 800 px only and does not reach them)
    ("Comments", "under 24", "C: the comment fields and replies"),
    (
        "Field",
        "card[0]",
        "A: the inline edit (`h-5`) and its pencil",
    ),
    (
        "Input",
        "card[0]",
        "A: the inline edit and the color fields",
    ),
    (
        "Sidebar",
        "color.",
        "A: the color fields of the design sidebar",
    ),
    ("Prompt Input", "view.", "C: the view's back and forward"),
    ("Toolbar", "view[", "G: the view's back and forward"),
    ("Skeleton", "skeleton", "the skeleton is a placeholder"),
    // front L's pages (Menus, Modals, Topbar)
    (
        "Modals",
        "",
        "L: share rows, reply field, delete confirm, title",
    ),
    ("Menus", "", "L: menu rows of fixed width and height"),
    ("Topbar", "", "L: the bar at a phone's width"),
    // at twice the size or in a longer language (the fix is the page's
    // or the widget's: wrap, truncate, or grow with the text)
    (
        "Comments",
        "wide in",
        "C: a cut comment measured past its width",
    ),
    (
        "Comments",
        "overlaps",
        "C: the quick tools under the send button",
    ),
    ("Sheet", "", "B: the sheet form's label column is fixed"),
    ("Stat", "", "G: the stat card at a phone's width"),
    (
        "Toolbar",
        "cut across",
        "G: the toolbar page's own group titles",
    ),
    (
        "Chart",
        "cut across",
        "G: the donut's label at a phone's width",
    ),
    ("Context Menu", "⌘", "B: the menu is wider than a phone"),
    ("Dropdown Menu", "⌘", "B: the menu is wider than a phone"),
    (
        "Sidebar",
        "",
        "G: the scene and design sidebars' fixed rows",
    ),
    ("Typography", "wide in", "F: the specimen's sample line"),
    ("Collapsible", "wide in", "D: a row's description"),
    (
        "Input Group",
        "wide in 0",
        "A: the addon leaves the text no room",
    ),
    ("Textarea", "wide in", "A: the placeholder is not wrapped"),
    ("Separator", "cut across", "the separator page's columns"),
    ("Slider", "cut across", "E: the value beside the slider"),
    // apps that fill the window
    ("Editor", "", "the code editor scrolls sideways"),
    ("Surface", "todo", "the todo panel's rows"),
    ("Capture", "", "the capture's note does not wrap"),
    ("Silences", "", "the cut list's rows"),
];

fn known(line: &str) -> Option<&'static str> {
    KNOWN
        .iter()
        .find(|(page, part, _)| line.starts_with(&format!("{page} @")) && line.contains(part))
        .map(|k| k.2)
}

/// Every page at every width of `case`; fails with the full list.
pub fn report(case: &Case) {
    let mut all = Vec::new();
    for page in PAGES {
        for &width in case.widths {
            all.extend(breaks(page.title, width, case));
        }
    }
    let (held, all): (Vec<String>, Vec<String>) = all.into_iter().partition(|l| known(l).is_some());
    eprintln!(
        "{} known breaks under {}:\n{}",
        held.len(),
        case.label,
        held.join("\n")
    );
    assert!(
        all.is_empty(),
        "{} breaks under {}:\n{}",
        all.len(),
        case.label,
        all.join("\n")
    );
}
