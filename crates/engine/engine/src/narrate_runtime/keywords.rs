//! Keyword classification predicates for the narrate DSL.
//!
//! Three disjoint sets: element keywords (start a new element),
//! block keywords (body-level statements like `show`, `on`, `when`),
//! and modifier keywords (chainable style/layout attributes).

pub(crate) fn is_element_keyword(s: &str) -> bool {
    matches!(
        s,
        "div" | "row" | "col" | "text" | "button" | "image" | "spacer"
    ) || s.starts_with(|c: char| c.is_ascii_uppercase())
}

pub(crate) fn is_block_keyword(s: &str) -> bool {
    matches!(s, "show" | "on" | "when" | "each" | "bind" | "otherwise")
}

pub(crate) fn is_modifier_keyword(s: &str) -> bool {
    matches!(
        s,
        "flex"
            | "center"
            | "centered"
            | "wrap"
            | "gap"
            | "p"
            | "px"
            | "py"
            | "pt"
            | "pb"
            | "pl"
            | "pr"
            | "m"
            | "mx"
            | "my"
            | "w"
            | "h"
            | "min_w"
            | "min_h"
            | "max_w"
            | "max_h"
            | "grow"
            | "shrink"
            | "basis"
            | "align_items"
            | "justify"
            | "bg"
            | "text_color"
            | "rounded"
            | "shadow"
            | "opacity"
            | "border"
            | "border_color"
            | "font_size"
            | "bold"
            | "italic"
            | "font_weight"
            | "tracking"
            | "letter_spacing"
            | "uppercase"
    )
}
