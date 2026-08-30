//! Declarative UI tree for the builder demo.

use engine::builder::*;
use engine::color::*;

pub fn build_ui() -> Element {
    div()
        .col()
        .gap(0.0)
        .child(build_header())
        .child(header_divider())
        .child(build_content())
        .child(footer_divider())
        .child(build_footer())
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

fn build_header() -> Element {
    div()
        .bg(Color::hex(0x14141f))
        .w(900.0)
        .h(70.0)
        .col()
        .p(0.0)
        .pl(32.0)
        .pt(14.0)
        .child(
            text("BUILDER API")
                .font_size(24.0)
                .text_color([0.93, 0.93, 0.96]),
        )
        .child(
            text("Declarative UI: div().col().bg(...).child(...)")
                .font_size(12.0)
                .text_color([0.55, 0.55, 0.65]),
        )
}

fn header_divider() -> Element {
    div().bg([0.18, 0.18, 0.25]).w(900.0).h(1.0)
}

// ---------------------------------------------------------------------------
// Content
// ---------------------------------------------------------------------------

fn build_content() -> Element {
    div()
        .col()
        .gap(16.0)
        .p(32.0)
        .child(feature_cards_row())
        .child(button_row())
        .child(code_snippet())
}

fn feature_cards_row() -> Element {
    div()
        .row()
        .gap(16.0)
        .child(color_card())
        .child(layout_card())
        .child(typography_card())
}

fn color_card() -> Element {
    div().col().gap(0.0).w(262.0).h(160.0).child(
        div()
            .bg([0.10, 0.10, 0.16])
            .w(262.0)
            .h(160.0)
            .col()
            .gap(0.0)
            .child(div().bg([0.0, 0.85, 0.95]).w(260.0).h(2.0))
            .child(
                div()
                    .col()
                    .gap(8.0)
                    .p(16.0)
                    .child(text("Colors").font_size(16.0).text_color([0.0, 0.85, 0.95]))
                    .child(
                        text("Named, hex, rgba")
                            .font_size(12.0)
                            .text_color([0.55, 0.55, 0.65]),
                    )
                    .child(color_swatches())
                    .child(
                        text("Color::hex(0xff3040)")
                            .font_size(11.0)
                            .text_color([0.55, 0.55, 0.65]),
                    ),
            ),
    )
}

fn color_swatches() -> Element {
    div()
        .row()
        .gap(6.0)
        .child(div().bg([0.30, 0.55, 1.0]).w(24.0).h(24.0))
        .child(div().bg([0.20, 0.80, 0.45]).w(24.0).h(24.0))
        .child(div().bg([1.0, 0.30, 0.25]).w(24.0).h(24.0))
        .child(div().bg([1.0, 0.85, 0.20]).w(24.0).h(24.0))
        .child(div().bg([0.60, 0.30, 0.90]).w(24.0).h(24.0))
        .child(div().bg([0.0, 0.85, 0.95]).w(24.0).h(24.0))
        .child(div().bg([1.0, 0.55, 0.10]).w(24.0).h(24.0))
}

fn layout_card() -> Element {
    div().col().gap(0.0).w(262.0).h(160.0).child(
        div()
            .bg([0.10, 0.10, 0.16])
            .w(262.0)
            .h(160.0)
            .col()
            .gap(0.0)
            .child(div().bg([1.0, 0.85, 0.20]).w(260.0).h(2.0))
            .child(
                div()
                    .col()
                    .gap(8.0)
                    .p(16.0)
                    .child(text("Layout").font_size(16.0).text_color([1.0, 0.85, 0.20]))
                    .child(
                        text("Row, col, gap, padding")
                            .font_size(12.0)
                            .text_color([0.55, 0.55, 0.65]),
                    )
                    .child(layout_boxes()),
            ),
    )
}

fn layout_boxes() -> Element {
    div()
        .row()
        .gap(4.0)
        .child(layout_box_item(".row()"))
        .child(layout_box_item(".col()"))
        .child(layout_box_item(".gap()"))
}

fn layout_box_item(label: &str) -> Element {
    div()
        .bg([0.15, 0.15, 0.22])
        .w(70.0)
        .h(40.0)
        .p(6.0)
        .child(text(label).font_size(10.0).text_color([0.75, 0.75, 0.85]))
}

fn typography_card() -> Element {
    div().col().gap(0.0).w(262.0).h(160.0).child(
        div()
            .bg([0.10, 0.10, 0.16])
            .w(262.0)
            .h(160.0)
            .col()
            .gap(0.0)
            .child(div().bg([0.60, 0.30, 0.90]).w(260.0).h(2.0))
            .child(
                div()
                    .col()
                    .gap(6.0)
                    .p(16.0)
                    .child(
                        text("Typography")
                            .font_size(16.0)
                            .text_color([0.60, 0.30, 0.90]),
                    )
                    .child(
                        text("font_size, bold, italic")
                            .font_size(12.0)
                            .text_color([0.55, 0.55, 0.65]),
                    )
                    .child(
                        text("Large Title")
                            .font_size(20.0)
                            .text_color([0.93, 0.93, 0.96]),
                    )
                    .child(
                        text("Body text at 14px")
                            .font_size(14.0)
                            .text_color([0.75, 0.75, 0.85]),
                    )
                    .child(
                        text("Caption at 11px")
                            .font_size(11.0)
                            .text_color([0.55, 0.55, 0.65]),
                    ),
            ),
    )
}

// ---------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------

fn button_row() -> Element {
    div()
        .row()
        .gap(10.0)
        .child(
            button("Primary")
                .bg([0.18, 0.38, 0.85])
                .rounded(4.0)
                .px(16.0)
                .py(10.0)
                .on_click(|_| {}),
        )
        .child(
            button("Success")
                .bg([0.12, 0.55, 0.30])
                .rounded(4.0)
                .px(16.0)
                .py(10.0),
        )
        .child(
            button("Danger")
                .bg([0.75, 0.18, 0.15])
                .rounded(4.0)
                .px(16.0)
                .py(10.0),
        )
}

// ---------------------------------------------------------------------------
// Code snippet + Footer
// ---------------------------------------------------------------------------

fn code_snippet() -> Element {
    div()
        .bg([0.08, 0.08, 0.13])
        .w(836.0)
        .h(100.0)
        .col()
        .gap(4.0)
        .p(16.0)
        .child(
            text("Builder Pattern")
                .font_size(11.0)
                .text_color([0.55, 0.55, 0.65]),
        )
        .child(
            text("div().col().gap(16).p(32)")
                .font_size(13.0)
                .text_color([0.0, 0.85, 0.95]),
        )
        .child(
            text("    .child(text(\"Hello\").font_size(24))")
                .font_size(13.0)
                .text_color([0.75, 0.75, 0.85]),
        )
        .child(
            text("    .child(button(\"OK\").bg(\"blue\").on_click(|_| {}))")
                .font_size(13.0)
                .text_color([0.75, 0.75, 0.85]),
        )
}

fn footer_divider() -> Element {
    div().bg([0.18, 0.18, 0.25]).w(900.0).h(1.0)
}

fn build_footer() -> Element {
    div()
        .bg([0.07, 0.07, 0.12])
        .w(900.0)
        .h(32.0)
        .pl(32.0)
        .pt(9.0)
        .child(
            text("Element tree  |  View trait  |  SceneNode flatten  |  Zero-alloc render")
                .font_size(11.0)
                .text_color([0.55, 0.55, 0.65]),
        )
}
