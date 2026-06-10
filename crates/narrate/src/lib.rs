pub use narrate_macro::plev_narrate;

/// Bridge module: re-exports from the real `plev::builder` API.
///
/// The `plev_narrate!` macro generates code referencing `::narrate::builder::*`.
/// This module provides all symbols the generated code needs, using the real
/// builder implementations instead of the previous stubs.
pub mod builder {
    // Re-export everything from the real builder
    pub use plev::builder::{
        Align, ClickEvent, Direction, Element, EventHandlers, FocusEvent, HoverEvent, IntoF32,
        IntoRadius, IntoView, Justify, KeyEvent, LayoutConfig, Scope, ScrollEvent, SizeConstraint,
        Spacing, Style, div,
    };
    pub use plev::color::{Color, IntoColor};

    /// No-arg `text()` for DSL — creates an empty text element.
    /// Content is set via `.child("content")` which merges into the text node.
    pub fn text() -> Element {
        plev::builder::text("")
    }

    /// No-arg `button()` for DSL — creates a styled container.
    /// Label is set via `.child("label")` inside the DSL body.
    pub fn button() -> Element {
        div()
            .bg(Color::rgba(0.25, 0.25, 0.35, 1.0))
            .rounded(4.0_f32)
            .p(8.0_f32)
    }

    /// Placeholder `image()` constructor for DSL.
    pub fn image() -> Element {
        plev::builder::image()
    }

    /// `spacer()` constructor — grows to fill available space.
    pub fn spacer() -> Element {
        plev::builder::spacer()
    }
}
