//! the IR's layout on the engine's `LayoutEngine`, no second engine.
//!
//! - `stack` is a flex row or column with its gap and alignment. a row
//!   first shrinks what can shrink; when its children still pass its
//!   content box, it wraps: the children that do not fit go down a line,
//!   in order, and nothing runs past the row's edge;
//! - `grid` is a wrapping row whose children start at `min_col` and grow:
//!   as many columns as fit, none narrower than `min_col`. unlike a CSS
//!   grid, the last row, when it is not full, stretches its children;
//! - `split` gives its two children the ratio as flex weights on a zero
//!   basis;
//! - `overlay`: the `LayoutStyle` has no absolute position, so the
//!   container lays out in its parent's flow with its first layer, and
//!   every other layer is a pass of its own over the container's rect.
//!
//! `size` bounds a node by `theme.size` tokens (each on the dimension its
//! name says) and weighs it for the free space; `padding` insets a
//! container. a leaf takes the size its widget measures; a text is never
//! narrower than its longest word (the flex minimum of a text would be
//! one glyph: the engine measures its min-content at width zero).

use engine::compose::layout::TextMeasureSpec;
use engine::compose::layout::{
    Align as FlexAlign, Direction, LayoutEngine, LayoutItem, LayoutStyle,
};
use engine::drawing::text::TextMeasurer;
use engine::styling::theme::Theme;
use engine::widgets::Rect;
use plevir::{Align, Axis, Layout};

use crate::expand::Node;
use crate::token::{self, Dim};

/// what a node takes in its parent when it holds no children.
#[derive(Clone, Debug)]
pub(crate) enum Leaf {
    /// a container: its size comes from its children and its padding.
    /// `inset` is added to the padding (top, right, bottom, left): a
    /// dialog's title band above and its buttons below.
    Container { inset: [f32; 4] },
    /// not drawn now (a closed dialog): no size, and its subtree is not
    /// laid out.
    Hidden,
    /// the widget's own size.
    Fixed(f32, f32),
    /// a text run, measured by the engine with the style it is drawn in.
    Text(TextMeasureSpec),
    /// this tall, as wide as the parent gives (at least `min_w`).
    Fill { height: f32, min_w: f32 },
}

/// the rect of every node of `tree` laid out in `rect`: the root is as
/// wide as `rect` and as tall as its content, from `rect`'s origin.
///
/// a row whose children pass its content box once they shrank all they
/// can wraps, and the tree is laid out again, until no row that does not
/// wrap yet overflows: one pass when everything fits, one more per level
/// of rows that did not.
pub(crate) fn compute(
    tree: &[Node],
    rect: Rect,
    theme: &Theme,
    leaf: &dyn Fn(usize) -> Leaf,
) -> Vec<Rect> {
    let mut wrap = vec![false; tree.len()];
    let mut engine = LayoutEngine::new();
    loop {
        let mut pass = Pass {
            tree,
            theme,
            leaf,
            wrap: &wrap,
            engine: &mut engine,
            rects: vec![Rect::default(); tree.len()],
            padding: vec![None; tree.len()],
        };
        pass.run(0, rect, false);
        let over = pass.overflowing();
        if over.is_empty() {
            return pass.rects;
        }
        for at in over {
            if let Some(w) = wrap.get_mut(at) {
                *w = true;
            }
        }
    }
}

/// true for a row: a horizontal stack, the one layout that wraps only
/// when its children do not fit.
fn is_row(node: &Node) -> bool {
    matches!(
        node.layout,
        Some(Layout::Stack {
            axis: Axis::Horizontal,
            ..
        })
    )
}

/// the width of the longest word of a text run: what it takes without
/// breaking a word.
fn longest_word(spec: &TextMeasureSpec) -> f32 {
    spec.content
        .split_whitespace()
        .map(|w| TextMeasurer::measure_styled(w, &spec.style, None).0)
        .fold(0.0, f32::max)
        .ceil()
}

struct Pass<'a> {
    tree: &'a [Node],
    theme: &'a Theme,
    leaf: &'a dyn Fn(usize) -> Leaf,
    /// the rows that wrap in this pass.
    wrap: &'a [bool],
    engine: &'a mut LayoutEngine,
    rects: Vec<Rect>,
    /// the padding of each container laid out, none for a leaf or a node
    /// not laid out.
    padding: Vec<Option<[f32; 4]>>,
}

impl Pass<'_> {
    /// lays out the subtree of `root` in `rect` (filling it when `fill`),
    /// then each overlay layer the subtree holds in a pass of its own.
    fn run(&mut self, root: usize, rect: Rect, fill: bool) {
        let mut items = Vec::new();
        let mut map = Vec::new();
        let mut layers = Vec::new();
        self.item(root, None, 0, &mut items, &mut map, &mut layers);
        if let Some(first) = items.first_mut() {
            first.style.width = Some(rect.w);
            first.style.height = fill.then_some(rect.h);
            first.style.margin = [0.0; 4];
        }
        let bounds = self.engine.compute(&items, rect.w, rect.h.max(1.0));
        for (b, &at) in bounds.iter().zip(&map) {
            if let Some(r) = self.rects.get_mut(at) {
                *r = Rect::new(rect.x + b.x, rect.y + b.y, b.width, b.height);
            }
        }
        for (container, layer) in layers {
            let over = self.rects.get(container).copied().unwrap_or_default();
            self.run(layer, over, true);
        }
    }

    /// the rows that do not wrap yet and whose children pass their
    /// content box, by more than half a pixel on either side.
    fn overflowing(&self) -> Vec<usize> {
        let mut out = Vec::new();
        for (at, node) in self.tree.iter().enumerate() {
            let wraps = self.wrap.get(at).copied().unwrap_or(true);
            let (Some(Some(pad)), Some(r)) = (self.padding.get(at), self.rects.get(at)) else {
                continue;
            };
            if wraps || !is_row(node) {
                continue;
            }
            let (left, right) = (r.x + pad[3] - 0.5, r.x + r.w - pad[1] + 0.5);
            let mut children = node.children.iter().filter_map(|&c| self.rects.get(c));
            if children.any(|c| c.x < left || c.x + c.w > right) {
                out.push(at);
            }
        }
        out
    }

    /// pushes the item of `at` (and of its subtree) and returns its index.
    /// `nth` is its place among its parent's children.
    fn item(
        &mut self,
        at: usize,
        parent: Option<&Node>,
        nth: usize,
        items: &mut Vec<LayoutItem>,
        map: &mut Vec<usize>,
        layers: &mut Vec<(usize, usize)>,
    ) -> usize {
        let index = items.len();
        let Some(node) = self.tree.get(at) else {
            return index;
        };
        let mut style = LayoutStyle::default();
        let mut text = None;
        let mut word = None;
        match (self.leaf)(at) {
            Leaf::Container { inset } => {
                self.container(at, node, &mut style);
                for (p, extra) in style.padding.iter_mut().zip(inset) {
                    *p += extra;
                }
                if let Some(p) = self.padding.get_mut(at) {
                    *p = Some(style.padding);
                }
            }
            Leaf::Hidden => {
                style.width = Some(0.0);
                style.height = Some(0.0);
                items.push(LayoutItem {
                    style,
                    children: Vec::new(),
                    text: None,
                });
                map.push(at);
                return index;
            }
            Leaf::Fixed(w, h) => {
                style.width = Some(w);
                style.height = Some(h);
                style.flex_shrink = 0.0;
            }
            Leaf::Text(spec) => {
                word = Some(longest_word(&spec));
                text = Some(spec);
            }
            Leaf::Fill { height, min_w } => {
                style.height = Some(height);
                style.min_width = Some(min_w);
            }
        }
        self.sized(node, &mut style);
        if let Some(w) = word {
            style.min_width = Some(style.min_width.map_or(w, |m| m.max(w)));
        }
        if let Some(parent) = parent {
            self.in_parent(parent, nth, &mut style);
        }
        items.push(LayoutItem {
            style,
            children: Vec::new(),
            text,
        });
        map.push(at);
        let overlay = matches!(node.layout, Some(Layout::Overlay));
        let mut children = Vec::new();
        for (i, &child) in node.children.iter().enumerate() {
            if overlay && i > 0 {
                layers.push((at, child));
                continue;
            }
            children.push(self.item(child, Some(node), i, items, map, layers));
        }
        if let Some(item) = items.get_mut(index) {
            item.children = children;
        }
        index
    }

    /// how a container arranges its children, and its padding. a row
    /// wraps when an earlier pass found it overflowing.
    fn container(&self, at: usize, node: &Node, style: &mut LayoutStyle) {
        let theme = self.theme;
        let gap = |g: Option<plevir::SpacingToken>| g.map_or(0.0, |g| token::spacing(g, theme));
        match node.layout {
            None | Some(Layout::Overlay) => {}
            Some(Layout::Stack {
                axis,
                gap: g,
                align,
            }) => {
                style.direction = direction(axis);
                style.gap = gap(g);
                style.align = align.map_or(FlexAlign::Stretch, flex_align);
                style.flex_wrap = self.wrap.get(at).copied().unwrap_or(false);
            }
            Some(Layout::Grid { gap: g, .. }) => {
                style.direction = Direction::Row;
                style.flex_wrap = true;
                style.gap = gap(g);
            }
            Some(Layout::Split { axis, .. }) => style.direction = direction(axis),
        }
        if let Some(p) = node.props.padding {
            style.padding = [token::spacing(p, theme); 4];
        }
    }

    /// `size`: the bounds by token, and the weight for the free space.
    fn sized(&self, node: &Node, style: &mut LayoutStyle) {
        if let Some(t) = node.size.min {
            match token::size(t, self.theme) {
                (v, Dim::Width) => style.min_width = Some(v),
                (v, Dim::Height) => style.min_height = Some(v),
            }
        }
        if let Some(t) = node.size.max {
            match token::size(t, self.theme) {
                (v, Dim::Width) => style.max_width = Some(v),
                (v, Dim::Height) => style.max_height = Some(v),
            }
        }
        if let Some(flex) = node.size.flex {
            style.flex_grow = f32::from(flex);
        }
    }

    /// what the parent's layout asks of its `nth` child: a grid column,
    /// a split pane.
    fn in_parent(&self, parent: &Node, nth: usize, style: &mut LayoutStyle) {
        match parent.layout {
            Some(Layout::Grid { min_col, .. }) => {
                let (col, _) = token::size(min_col, self.theme);
                style.flex_basis = Some(col);
                style.min_width = Some(style.min_width.map_or(col, |m| m.max(col)));
                style.flex_grow = style.flex_grow.max(1.0);
                style.width = None;
            }
            Some(Layout::Split { axis, ratio }) => {
                let share = if nth == 0 {
                    f32::from(ratio)
                } else {
                    f32::from(100u8.saturating_sub(ratio))
                };
                style.flex_basis = Some(0.0);
                style.flex_grow = share;
                style.flex_shrink = 1.0;
                match axis {
                    Axis::Horizontal => {
                        style.width = None;
                        style.min_width = Some(0.0);
                    }
                    Axis::Vertical => {
                        style.height = None;
                        style.min_height = Some(0.0);
                    }
                }
            }
            _ => {}
        }
    }
}

fn direction(axis: Axis) -> Direction {
    match axis {
        Axis::Horizontal => Direction::Row,
        Axis::Vertical => Direction::Column,
    }
}

fn flex_align(align: Align) -> FlexAlign {
    match align {
        Align::Start => FlexAlign::Start,
        Align::Center => FlexAlign::Center,
        Align::End => FlexAlign::End,
        Align::Stretch => FlexAlign::Stretch,
    }
}

/// the axis `node` lays its children along (a node without a layout is a
/// column, as the IR says).
pub(crate) fn axis_of(node: &Node) -> Axis {
    match node.layout {
        Some(Layout::Stack { axis, .. } | Layout::Split { axis, .. }) => axis,
        Some(Layout::Grid { .. }) => Axis::Horizontal,
        None | Some(Layout::Overlay) => Axis::Vertical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accord::SemId;
    use engine::styling::theme::TypeRole;
    use plevir::{Bind, Props, Size, SizeToken, SpacingToken};

    fn node(id: &str, layout: Option<Layout>, children: Vec<usize>) -> Node {
        Node {
            id: SemId::parse(id).unwrap(),
            component: None,
            label: None,
            description: None,
            props: Props::default(),
            layout,
            size: Size::default(),
            bind: Bind::default(),
            item: None,
            parent: None,
            children,
        }
    }

    fn lay(tree: &[Node], w: f32) -> Vec<Rect> {
        let theme = Theme::hoff_core();
        let leaf = |at: usize| match tree.get(at).map(|n| n.children.is_empty()) {
            Some(true) => Leaf::Fixed(100.0, 20.0),
            _ => Leaf::Container { inset: [0.0; 4] },
        };
        compute(tree, Rect::new(10.0, 20.0, w, 600.0), &theme, &leaf)
    }

    #[test]
    fn a_stack_places_its_children_one_after_the_other_with_its_gap() {
        let theme = Theme::hoff_core();
        let row = Layout::Stack {
            axis: Axis::Horizontal,
            gap: Some(SpacingToken::Md),
            align: None,
        };
        let tree = [
            node("a", Some(row), vec![1, 2]),
            node("a.b", None, vec![]),
            node("a.c", None, vec![]),
        ];
        let r = lay(&tree, 400.0);
        assert_eq!((r[0].x, r[0].y, r[0].w), (10.0, 20.0, 400.0));
        assert_eq!(r[1].x, 10.0);
        assert_eq!(r[2].x, 10.0 + 100.0 + theme.spacing.md);
    }

    #[test]
    fn a_split_gives_its_panes_the_ratio() {
        let split = Layout::Split {
            axis: Axis::Horizontal,
            ratio: 30,
        };
        let tree = [
            node("a", Some(split), vec![1, 2]),
            node("a.b", None, vec![]),
            node("a.c", None, vec![]),
        ];
        let r = lay(&tree, 1000.0);
        assert!((r[1].w - 300.0).abs() < 0.5, "{:?}", r[1]);
        assert!((r[2].w - 700.0).abs() < 0.5, "{:?}", r[2]);
    }

    #[test]
    fn a_grid_fits_as_many_columns_as_min_col_allows() {
        let theme = Theme::hoff_core();
        let grid = Layout::Grid {
            min_col: SizeToken::CardMinW,
            gap: None,
        };
        let tree = [
            node("a", Some(grid), vec![1, 2, 3]),
            node("a.b", None, vec![]),
            node("a.c", None, vec![]),
            node("a.d", None, vec![]),
        ];
        let min = theme.size.card_min_w;
        let r = lay(&tree, min * 2.5);
        // two columns fit: the first row holds two, the third wraps
        assert_eq!(r[1].y, r[2].y);
        assert!(r[3].y > r[1].y);
        assert!(r[1].w >= min && r[2].w >= min);
    }

    /// the row of a cut as the rules propose it (task-69 checkpoint b):
    /// a thumbnail, five texts that grow, three buttons, in a card.
    fn cut_row() -> (Vec<Node>, Vec<Leaf>) {
        let theme = Theme::hoff_core();
        let row = Layout::Stack {
            axis: Axis::Horizontal,
            gap: Some(SpacingToken::Sm),
            align: Some(Align::Center),
        };
        let mut card = node("a", None, vec![1]);
        card.props.padding = Some(SpacingToken::Lg);
        let mut tree = vec![
            card,
            node("a.row", Some(row), (2..11).collect()),
            node("a.row.frame", None, vec![]),
        ];
        let mut leaves = vec![
            Leaf::Container { inset: [0.0; 4] },
            Leaf::Container { inset: [0.0; 4] },
            Leaf::Fixed(92.0, 52.0),
        ];
        for text in [
            "pending",
            "1.996916",
            "5.01551",
            "0:03.0 to 0:05.0",
            "3.018594",
        ] {
            let mut n = node("a.row.text", None, vec![]);
            n.size.flex = Some(1);
            tree.push(n);
            leaves.push(Leaf::Text(TextMeasureSpec {
                content: text.to_owned(),
                style: theme.typography.role(TypeRole::BodyLg),
                max_width: None,
            }));
        }
        for w in [161.0, 109.0, 109.0] {
            tree.push(node("a.row.button", None, vec![]));
            leaves.push(Leaf::Fixed(w, 40.0));
        }
        (tree, leaves)
    }

    fn lay_leaves(tree: &[Node], leaves: &[Leaf], w: f32) -> Vec<Rect> {
        let theme = Theme::hoff_core();
        let leaf = |at: usize| leaves.get(at).cloned().unwrap_or(Leaf::Hidden);
        compute(tree, Rect::new(10.0, 20.0, w, 600.0), &theme, &leaf)
    }

    /// what a text's run measures without breaking a word, and each text
    /// of `leaves` with its rect.
    fn texts<'a>(leaves: &'a [Leaf], r: &'a [Rect]) -> impl Iterator<Item = (f32, Rect)> + 'a {
        leaves.iter().zip(r).filter_map(|(l, r)| match l {
            Leaf::Text(spec) => Some((longest_word(spec), *r)),
            _ => None,
        })
    }

    /// The row of a cut in the column the review has at 1200x800 (a card
    /// 420 wide) and in a narrower one: nothing passes the row's box, and
    /// no text is narrower than its longest word; the row wraps (the
    /// buttons go down a line) instead of shrinking the texts to a glyph.
    #[test]
    fn a_row_that_does_not_fit_wraps_and_keeps_every_word_whole() {
        let theme = Theme::hoff_core();
        let (tree, leaves) = cut_row();
        for width in [420.0, 280.0] {
            let r = lay_leaves(&tree, &leaves, width);
            let row = r[1];
            assert_eq!(row.w, width - theme.spacing.lg * 2.0);
            for (at, c) in r.iter().enumerate().skip(2) {
                assert!(
                    c.x >= row.x - 0.5 && c.x + c.w <= row.x + row.w + 0.5,
                    "{width}: child {at} {c:?} passes the row {row:?}"
                );
            }
            for (word, c) in texts(&leaves, &r) {
                assert!(c.w + 0.5 >= word, "{width}: {c:?} under its word {word}");
            }
            let lines: std::collections::BTreeSet<i32> =
                r[2..].iter().map(|c| (c.y + c.h / 2.0) as i32).collect();
            assert!(lines.len() > 1, "{width}: the row wrapped: {lines:?}");
        }
    }

    /// A row shrinks before it wraps: a long text beside a button keeps
    /// the button on its line and breaks between its own words.
    #[test]
    fn a_row_shrinks_its_text_before_it_wraps() {
        let theme = Theme::hoff_core();
        let row = Layout::Stack {
            axis: Axis::Horizontal,
            gap: Some(SpacingToken::Sm),
            align: None,
        };
        let mut text = node("a.text", None, vec![]);
        text.size.flex = Some(1);
        let tree = [
            node("a", Some(row), vec![1, 2]),
            text,
            node("a.button", None, vec![]),
        ];
        let leaves = [
            Leaf::Container { inset: [0.0; 4] },
            Leaf::Text(TextMeasureSpec {
                content: "a sentence that is much longer than the room it has".to_owned(),
                style: theme.typography.role(TypeRole::BodyLg),
                max_width: None,
            }),
            Leaf::Fixed(100.0, 40.0),
        ];
        let r = lay_leaves(&tree, &leaves, 300.0);
        assert_eq!(r[1].y, r[2].y, "one line: {r:?}");
        assert!(r[2].x + r[2].w <= 310.5, "{r:?}");
        assert!(
            r[1].h > theme.typography.role(TypeRole::BodyLg).line_height,
            "the text wraps"
        );
    }

    #[test]
    fn an_overlay_puts_every_layer_over_the_container() {
        let tree = [
            node("a", Some(Layout::Overlay), vec![1, 2]),
            node("a.b", None, vec![]),
            node("a.c", None, vec![]),
        ];
        let r = lay(&tree, 400.0);
        assert_eq!((r[1].x, r[1].y), (r[0].x, r[0].y));
        assert_eq!((r[2].x, r[2].y), (r[0].x, r[0].y));
    }
}
