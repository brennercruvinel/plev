pub mod suggest;

#[cfg(test)]
mod tests;

use proc_macro2::{Ident, Span};

pub use suggest::suggest_similar;

// ── Custom keywords for block-item parsing ──

pub mod kw {
    syn::custom_keyword!(show);
    syn::custom_keyword!(on);
    syn::custom_keyword!(bind);
    syn::custom_keyword!(when);
    syn::custom_keyword!(each);
    syn::custom_keyword!(otherwise);
    syn::custom_keyword!(keyed);
    syn::custom_keyword!(by);
    syn::custom_keyword!(to);
}

/// All known element names (lowercase built-ins).
pub const ELEMENT_NAMES: &[&str] = &["row", "col", "div", "text", "button", "image", "spacer"];

/// All known modifier names.
pub const MODIFIER_NAMES: &[&str] = &[
    "flex",
    "center",
    "centered",
    "wrap",
    "gap",
    "p",
    "px",
    "py",
    "pt",
    "pb",
    "pl",
    "pr",
    "m",
    "mx",
    "my",
    "w",
    "h",
    "min_w",
    "min_h",
    "max_w",
    "max_h",
    "grow",
    "shrink",
    "basis",
    "align_items",
    "justify",
    "bg",
    "text_color",
    "rounded",
    "shadow",
    "opacity",
    "border",
    "font_size",
    "bold",
    "italic",
];

/// All known event names.
pub const EVENT_NAMES: &[&str] = &["click", "hover", "key", "focus", "blur", "scroll"];

/// All known block-item keywords.
pub const BLOCK_KEYWORDS: &[&str] = &["show", "on", "bind", "when", "each", "otherwise"];

// ── Element kinds ──

#[derive(Debug, Clone)]
pub enum ElementKind {
    Row,
    Col,
    Div,
    Text,
    Button,
    Image,
    Spacer,
    Custom(Ident),
}

impl ElementKind {
    pub fn from_ident(ident: &Ident) -> Option<Self> {
        let s = ident.to_string();
        match s.as_str() {
            "row" => Some(Self::Row),
            "col" => Some(Self::Col),
            "div" => Some(Self::Div),
            "text" => Some(Self::Text),
            "button" => Some(Self::Button),
            "image" => Some(Self::Image),
            "spacer" => Some(Self::Spacer),
            _ if s.starts_with(|c: char| c.is_ascii_uppercase()) => {
                Some(Self::Custom(ident.clone()))
            }
            _ => None,
        }
    }
}

/// Returns true if the ident string is an element keyword or a block-item keyword.
pub fn is_element_or_block_keyword(s: &str) -> bool {
    matches!(
        s,
        "row"
            | "col"
            | "div"
            | "text"
            | "button"
            | "image"
            | "spacer"
            | "show"
            | "on"
            | "bind"
            | "when"
            | "each"
            | "otherwise"
    ) || s.starts_with(|c: char| c.is_ascii_uppercase())
}

// ── Modifier keys ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierKey {
    // Layout
    Flex,
    Center,
    Wrap,
    Gap,
    P,
    Px,
    Py,
    Pt,
    Pb,
    Pl,
    Pr,
    M,
    Mx,
    My,
    W,
    H,
    MinW,
    MinH,
    MaxW,
    MaxH,
    Grow,
    Shrink,
    Basis,
    AlignItems,
    Justify,
    // Style
    Bg,
    TextColor,
    Rounded,
    Shadow,
    Opacity,
    Border,
    FontSize,
    Bold,
    Italic,
}

impl ModifierKey {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "flex" => Some(Self::Flex),
            "center" | "centered" => Some(Self::Center),
            "wrap" => Some(Self::Wrap),
            "gap" => Some(Self::Gap),
            "p" => Some(Self::P),
            "px" => Some(Self::Px),
            "py" => Some(Self::Py),
            "pt" => Some(Self::Pt),
            "pb" => Some(Self::Pb),
            "pl" => Some(Self::Pl),
            "pr" => Some(Self::Pr),
            "m" => Some(Self::M),
            "mx" => Some(Self::Mx),
            "my" => Some(Self::My),
            "w" => Some(Self::W),
            "h" => Some(Self::H),
            "min_w" => Some(Self::MinW),
            "min_h" => Some(Self::MinH),
            "max_w" => Some(Self::MaxW),
            "max_h" => Some(Self::MaxH),
            "grow" => Some(Self::Grow),
            "shrink" => Some(Self::Shrink),
            "basis" => Some(Self::Basis),
            "align_items" => Some(Self::AlignItems),
            "justify" => Some(Self::Justify),
            "bg" => Some(Self::Bg),
            "text_color" => Some(Self::TextColor),
            "rounded" => Some(Self::Rounded),
            "shadow" => Some(Self::Shadow),
            "opacity" => Some(Self::Opacity),
            "border" => Some(Self::Border),
            "font_size" => Some(Self::FontSize),
            "bold" => Some(Self::Bold),
            "italic" => Some(Self::Italic),
            _ => None,
        }
    }

    pub fn is_flag(self) -> bool {
        matches!(
            self,
            Self::Flex | Self::Center | Self::Bold | Self::Italic | Self::Wrap
        )
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Flex => "flex",
            Self::Center => "center",
            Self::Wrap => "wrap",
            Self::Gap => "gap",
            Self::P => "p",
            Self::Px => "px",
            Self::Py => "py",
            Self::Pt => "pt",
            Self::Pb => "pb",
            Self::Pl => "pl",
            Self::Pr => "pr",
            Self::M => "m",
            Self::Mx => "mx",
            Self::My => "my",
            Self::W => "w",
            Self::H => "h",
            Self::MinW => "min_w",
            Self::MinH => "min_h",
            Self::MaxW => "max_w",
            Self::MaxH => "max_h",
            Self::Grow => "grow",
            Self::Shrink => "shrink",
            Self::Basis => "basis",
            Self::AlignItems => "align_items",
            Self::Justify => "justify",
            Self::Bg => "bg",
            Self::TextColor => "text_color",
            Self::Rounded => "rounded",
            Self::Shadow => "shadow",
            Self::Opacity => "opacity",
            Self::Border => "border",
            Self::FontSize => "font_size",
            Self::Bold => "bold",
            Self::Italic => "italic",
        }
    }

    pub fn method_ident(self) -> Ident {
        Ident::new(self.as_str(), Span::call_site())
    }
}

// ── Event kinds ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Click,
    Hover,
    Key,
    Focus,
    Blur,
    Scroll,
}

impl EventKind {
    pub fn from_ident(ident: &Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "click" => Some(Self::Click),
            "hover" => Some(Self::Hover),
            "key" => Some(Self::Key),
            "focus" => Some(Self::Focus),
            "blur" => Some(Self::Blur),
            "scroll" => Some(Self::Scroll),
            _ => None,
        }
    }

    pub fn method_ident(self) -> Ident {
        let name = match self {
            Self::Click => "on_click",
            Self::Hover => "on_hover",
            Self::Key => "on_key",
            Self::Focus => "on_focus",
            Self::Blur => "on_blur",
            Self::Scroll => "on_scroll",
        };
        Ident::new(name, Span::call_site())
    }
}
