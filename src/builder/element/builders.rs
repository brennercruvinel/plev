use super::super::style::*;
use super::super::traits::{IntoF32, IntoRadius};
use super::{
    ClickEvent, Element, ElementKind, FocusEvent, HoverEvent, IntoView, KeyEvent, ScrollEvent,
};
use crate::color::IntoColor;

impl Element {
    /// Inspect children (Mantra-15: testable without GPU).
    pub fn children_ref(&self) -> &[Element] {
        &self.children
    }

    pub fn flex(mut self) -> Self {
        self.layout.direction = Direction::Row;
        self
    }
    pub fn row(mut self) -> Self {
        self.layout.direction = Direction::Row;
        self
    }
    pub fn col(mut self) -> Self {
        self.layout.direction = Direction::Column;
        self
    }

    pub fn center(mut self) -> Self {
        self.layout.align = Align::Center;
        self.layout.justify = Justify::Center;
        self
    }

    pub fn centered(self) -> Self {
        self.center()
    }

    /// Flush left: all children share the same left edge (align Start).
    /// Column layout, no stretch -- children keep intrinsic width.
    pub fn flush_left(mut self) -> Self {
        self.layout.direction = Direction::Column;
        self.layout.align = Align::Start;
        self
    }

    /// Flush right: all children share the same right edge (align End).
    pub fn flush_right(mut self) -> Self {
        self.layout.direction = Direction::Column;
        self.layout.align = Align::End;
        self
    }

    /// Align top edges: row layout, children share the same top Y.
    pub fn align_top(mut self) -> Self {
        self.layout.direction = Direction::Row;
        self.layout.align = Align::Start;
        self
    }

    /// Align bottom edges: row layout, children share the same bottom Y.
    pub fn align_bottom(mut self) -> Self {
        self.layout.direction = Direction::Row;
        self.layout.align = Align::End;
        self
    }

    /// Align center on cross axis (vertical center in row, horizontal center in col).
    pub fn align_center(mut self) -> Self {
        self.layout.align = Align::Center;
        self
    }

    pub fn wrap(mut self) -> Self {
        self.layout.wrap = true;
        self
    }

    pub fn gap(mut self, g: impl IntoF32) -> Self {
        self.layout.gap = g.into_f32();
        self
    }

    pub fn p(mut self, v: impl IntoF32) -> Self {
        self.layout.padding = Spacing::all(v.into_f32());
        self
    }

    pub fn px(mut self, v: impl IntoF32) -> Self {
        let v = v.into_f32();
        self.layout.padding.left = v;
        self.layout.padding.right = v;
        self
    }

    pub fn py(mut self, v: impl IntoF32) -> Self {
        let v = v.into_f32();
        self.layout.padding.top = v;
        self.layout.padding.bottom = v;
        self
    }

    pub fn pt(mut self, v: impl IntoF32) -> Self {
        self.layout.padding.top = v.into_f32();
        self
    }
    pub fn pb(mut self, v: impl IntoF32) -> Self {
        self.layout.padding.bottom = v.into_f32();
        self
    }
    pub fn pl(mut self, v: impl IntoF32) -> Self {
        self.layout.padding.left = v.into_f32();
        self
    }
    pub fn pr(mut self, v: impl IntoF32) -> Self {
        self.layout.padding.right = v.into_f32();
        self
    }

    pub fn m(mut self, v: impl IntoF32) -> Self {
        self.layout.margin = Spacing::all(v.into_f32());
        self
    }

    pub fn mx(mut self, v: impl IntoF32) -> Self {
        let v = v.into_f32();
        self.layout.margin.left = v;
        self.layout.margin.right = v;
        self
    }

    pub fn my(mut self, v: impl IntoF32) -> Self {
        let v = v.into_f32();
        self.layout.margin.top = v;
        self.layout.margin.bottom = v;
        self
    }

    pub fn w(mut self, w: impl IntoF32) -> Self {
        self.layout.width = SizeConstraint::Fixed(w.into_f32());
        self
    }
    pub fn h(mut self, h: impl IntoF32) -> Self {
        self.layout.height = SizeConstraint::Fixed(h.into_f32());
        self
    }
    pub fn min_w(mut self, v: impl IntoF32) -> Self {
        self.layout.min_width = Some(v.into_f32());
        self
    }
    pub fn min_h(mut self, v: impl IntoF32) -> Self {
        self.layout.min_height = Some(v.into_f32());
        self
    }
    pub fn max_w(mut self, v: impl IntoF32) -> Self {
        self.layout.max_width = Some(v.into_f32());
        self
    }
    pub fn max_h(mut self, v: impl IntoF32) -> Self {
        self.layout.max_height = Some(v.into_f32());
        self
    }
    pub fn grow(mut self, v: impl IntoF32) -> Self {
        self.layout.grow = v.into_f32();
        self
    }
    pub fn shrink(mut self, v: impl IntoF32) -> Self {
        self.layout.shrink = v.into_f32();
        self
    }
    pub fn basis(mut self, v: impl IntoF32) -> Self {
        self.layout.basis = Some(v.into_f32());
        self
    }
    pub fn align_items(mut self, a: Align) -> Self {
        self.layout.align = a;
        self
    }
    pub fn justify(mut self, j: Justify) -> Self {
        self.layout.justify = j;
        self
    }

    // -- Style builders -------------------------------------------------------

    pub fn bg(mut self, color: impl IntoColor) -> Self {
        self.style.bg = Some(color.into_color());
        self
    }

    /// Fill the background with a 2-stop linear gradient. `angle_deg` follows
    /// the CSS convention: 0 puts `from` at the bottom, 90 puts it at the
    /// left, measured clockwise. Takes precedence over `bg`.
    pub fn bg_linear(
        mut self,
        from: impl IntoColor,
        to: impl IntoColor,
        angle_deg: impl IntoF32,
    ) -> Self {
        self.style.bg_gradient = Some(LinearGradient {
            from: from.into_color(),
            to: to.into_color(),
            angle_deg: angle_deg.into_f32(),
        });
        self
    }

    /// Analytic drop shadow under this element: `blur` is the CSS-like blur
    /// radius, `offset_y` shifts the shadow down. Follows the element's
    /// corner radius.
    pub fn shadow_drop(
        mut self,
        blur: impl IntoF32,
        offset_y: impl IntoF32,
        color: impl IntoColor,
    ) -> Self {
        self.style.drop_shadow = Some(DropShadow {
            blur: blur.into_f32(),
            offset: [0.0, offset_y.into_f32()],
            color: color.into_color(),
        });
        self
    }

    /// CSS `box-shadow: inset`: analytic shadow falling INSIDE the element,
    /// clipped to its rounded bounds and drawn over the background fill —
    /// the HOFF glass relief (`inset 2px 4px 16px rgba(248,248,248,.06)`).
    /// `offset` is the CSS (x, y) offset: positive values pool the light
    /// at the top/left inside edges. Combine with `shadow_drop` (or extra
    /// `SceneNode::Shadow` pushes) for multi-shadow stacks.
    pub fn shadow_inset(
        mut self,
        blur: impl IntoF32,
        offset: [f32; 2],
        color: impl IntoColor,
    ) -> Self {
        self.style.inset_shadow = Some(DropShadow {
            blur: blur.into_f32(),
            offset,
            color: color.into_color(),
        });
        self
    }

    /// Clip children to this element's bounds (scissor). Needed so scrolled
    /// or oversized content does not leak outside panels and lists.
    pub fn clip_children(mut self) -> Self {
        self.style.clip_children = true;
        self
    }

    /// Set the encoded image bytes (png/jpeg) for an `image()` element.
    /// Decoding and atlas packing are memoized by content, so passing the
    /// same bytes every frame (e.g. `include_bytes!`) is cheap.
    pub fn src_bytes(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        if let ElementKind::Image {
            bytes: ref mut slot,
        } = self.kind
        {
            *slot = Some(std::sync::Arc::new(bytes.into()));
        }
        self
    }
    pub fn text_color(mut self, color: impl IntoColor) -> Self {
        self.style.text_color = color.into_color();
        self
    }
    pub fn rounded(mut self, r: impl IntoRadius) -> Self {
        self.style.corner_radius = r.into_radius();
        self
    }
    pub fn shadow(mut self, s: impl IntoF32) -> Self {
        self.style.shadow = s.into_f32();
        self
    }
    pub fn opacity(mut self, v: impl IntoF32) -> Self {
        self.style.opacity = v.into_f32();
        self
    }
    pub fn border(mut self, v: impl IntoF32) -> Self {
        self.style.border = v.into_f32();
        self
    }

    pub fn border_color(mut self, color: impl IntoColor) -> Self {
        self.style.border_color = color.into_color();
        self
    }

    pub fn border_top(mut self, v: impl IntoF32, color: impl IntoColor) -> Self {
        self.style.border_sides.top = v.into_f32();
        self.style.border_sides.color = color.into_color();
        self
    }

    pub fn border_bottom(mut self, v: impl IntoF32, color: impl IntoColor) -> Self {
        self.style.border_sides.bottom = v.into_f32();
        self.style.border_sides.color = color.into_color();
        self
    }

    pub fn border_left(mut self, v: impl IntoF32, color: impl IntoColor) -> Self {
        self.style.border_sides.left = v.into_f32();
        self.style.border_sides.color = color.into_color();
        self
    }

    pub fn border_right(mut self, v: impl IntoF32, color: impl IntoColor) -> Self {
        self.style.border_sides.right = v.into_f32();
        self.style.border_sides.color = color.into_color();
        self
    }

    pub fn tracking(mut self, v: impl IntoF32) -> Self {
        self.style.letter_spacing = v.into_f32();
        self
    }

    pub fn letter_spacing(mut self, v: impl IntoF32) -> Self {
        self.style.letter_spacing = v.into_f32();
        self
    }

    pub fn uppercase(mut self) -> Self {
        self.style.uppercase = true;
        self
    }

    pub fn font_size(mut self, s: impl IntoF32) -> Self {
        let s = s.into_f32();
        if let ElementKind::Text {
            ref mut font_size,
            ref mut line_height,
            ..
        } = self.kind
        {
            *font_size = s;
            *line_height = s * 1.3;
        }
        self
    }

    pub fn line_height(mut self, lh: impl IntoF32) -> Self {
        if let ElementKind::Text {
            ref mut line_height,
            ..
        } = self.kind
        {
            *line_height = lh.into_f32();
        }
        self
    }

    pub fn max_width(mut self, mw: impl IntoF32) -> Self {
        if let ElementKind::Text {
            ref mut max_width, ..
        } = self.kind
        {
            *max_width = Some(mw.into_f32());
        }
        self
    }

    pub fn bold(mut self) -> Self {
        self.style.bold = true;
        self.style.font_weight = 700;
        self
    }

    pub fn italic(mut self) -> Self {
        self.style.italic = true;
        self
    }

    pub fn font_weight(mut self, weight: u16) -> Self {
        self.style.font_weight = weight;
        self.style.bold = weight >= 700;
        self
    }

    pub fn intent(mut self, intent: crate::theme::Intent) -> Self {
        self.intent = Some(intent);
        self
    }

    pub fn truncate(mut self, max_chars: usize) -> Self {
        if let ElementKind::Text {
            ref mut truncate_chars,
            ..
        } = self.kind
        {
            *truncate_chars = Some(max_chars);
        }
        self
    }

    // -- Children builders ----------------------------------------------------

    pub fn child(mut self, child: impl IntoView) -> Self {
        let child_elem = child.into_view();
        if let ElementKind::Text {
            ref mut content, ..
        } = self.kind
            && content.is_empty()
            && let ElementKind::Text {
                content: ref child_content,
                ..
            } = child_elem.kind
            && !child_content.is_empty()
        {
            *content = child_content.clone();
            return self;
        }
        self.children.push(child_elem);
        self
    }

    pub fn children<I, V>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: IntoView,
    {
        for item in iter {
            self.children.push(item.into_view());
        }
        self
    }

    pub fn child_if<C, F>(self, cond: C, then: F) -> Self
    where
        C: Fn() -> bool,
        F: Fn() -> Element,
    {
        if cond() { self.child(then()) } else { self }
    }

    pub fn child_if_else<C, T, E>(self, cond: C, then: T, otherwise: E) -> Self
    where
        C: Fn() -> bool,
        T: Fn() -> Element,
        E: Fn() -> Element,
    {
        if cond() {
            self.child(then())
        } else {
            self.child(otherwise())
        }
    }

    pub fn children_each<T, I, F>(self, items: I, render: F) -> Self
    where
        I: Fn() -> Vec<T>,
        F: Fn(T) -> Element,
    {
        let mut s = self;
        for item in items() {
            s = s.child(render(item));
        }
        s
    }

    pub fn children_each_keyed<T, K, I, KF, F>(self, items: I, _key: KF, render: F) -> Self
    where
        I: Fn() -> Vec<T>,
        KF: Fn(&T) -> K,
        F: Fn(T) -> Element,
    {
        let mut s = self;
        for item in items() {
            s = s.child(render(item));
        }
        s
    }

    pub fn bind<F, V>(self, _target: &str, _value: F) -> Self
    where
        F: Fn() -> V,
    {
        self
    }

    // -- Event builders -------------------------------------------------------

    pub fn on_click(mut self, f: impl FnMut(&ClickEvent) + 'static) -> Self {
        self.events.on_click = Some(Box::new(f));
        self
    }
    pub fn on_hover(mut self, f: impl FnMut(&HoverEvent) + 'static) -> Self {
        self.events.on_hover = Some(Box::new(f));
        self
    }
    pub fn on_key(mut self, f: impl FnMut(&KeyEvent) + 'static) -> Self {
        self.events.on_key = Some(Box::new(f));
        self
    }
    pub fn on_focus(mut self, f: impl FnMut(&FocusEvent) + 'static) -> Self {
        self.events.on_focus = Some(Box::new(f));
        self
    }
    pub fn on_blur(mut self, f: impl FnMut(&FocusEvent) + 'static) -> Self {
        self.events.on_blur = Some(Box::new(f));
        self
    }
    pub fn on_scroll(mut self, f: impl FnMut(&ScrollEvent) + 'static) -> Self {
        self.events.on_scroll = Some(Box::new(f));
        self
    }
}
