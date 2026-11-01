//! Icon data: Lucide v0.469.0 primitives, verbatim, in the 24x24 viewBox,
//! and the hoff core kit's inline glyphs (`kit-*`, the paths its
//! components write in their own `<svg>`, verbatim, filled in their own
//! viewBox). The entries live in three tables by meaning and name range:
//! the lucide icons before the kit's glyphs, the glyphs, and the lucide
//! icons after them. Each table is kept in alphabetical order, no name is
//! in two of them, and [`icons`] reads them in that order, so the whole
//! set comes out sorted (tests check all three).

mod kit;
mod lucide_a_k;
mod lucide_l_z;

/// One SVG primitive of an icon, in the 24x24 viewBox.
pub(super) enum Shape {
    /// SVG path `d` data.
    Path(&'static str),
    /// `<circle cx cy r>`.
    Circle(f32, f32, f32),
    /// `<line x1 y1 x2 y2>`.
    Line(f32, f32, f32, f32),
    /// `<polyline points>`.
    Polyline(&'static str),
    /// `<polygon points>` (closed).
    Polygon(&'static str),
    /// `<rect x y w h rx>`.
    Rect(f32, f32, f32, f32, f32),
    /// A filled `<path d>` in its own square viewBox: the kit's inline
    /// glyphs (`kit-*`), drawn as the kit fills them, not stroked.
    Fill(f32, &'static str),
    /// [`Shape::Fill`] at a share of the icon's color (an SVG
    /// `opacity`).
    Faint(f32, f32, &'static str),
}

/// The tables in name order: lucide `a`-`k`, the kit's `kit-*` glyphs,
/// lucide `l`-`z`. A new icon goes in the table its name sorts into.
pub(super) fn tables() -> [&'static [(&'static str, &'static [Shape])]; 3] {
    [lucide_a_k::ICONS, kit::ICONS, lucide_l_z::ICONS]
}

/// Every icon, name and shapes, in alphabetical order.
pub(super) fn icons() -> impl Iterator<Item = &'static (&'static str, &'static [Shape])> {
    tables().into_iter().flatten()
}
