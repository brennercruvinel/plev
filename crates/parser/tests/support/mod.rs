//! What the IR tests share (task-71): the fixtures, the theme's values as
//! the IR emitter measures them, and the registries of the apps the IR is
//! verified against.
//!
//! The registries are the test playing the app: they declare the state and
//! the actions the report says the IR binds (`bindings`). The importer
//! declares none.

use std::fs;
use std::path::PathBuf;

use engine::theme::Theme;
use parser::{Imported, Tokens, TypeStyle, Value};
use sem::{ParamSpec, Role, Snapshot, SnapshotBuilder, ValueKind};
use uir::{ColorRole, ShapeRole, SizeToken, SpacingToken, TypographyRole};

pub fn fixture_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(rel)
}

pub fn fixture(rel: &str) -> String {
    fs::read_to_string(fixture_path(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// Compares `actual` with the golden `rel`; `UPDATE_GOLDEN=1` writes it.
pub fn check_golden(rel: &str, actual: &str) {
    let path = fixture_path(rel);
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        fs::write(&path, actual).unwrap_or_else(|e| panic!("write {rel}: {e}"));
        return;
    }
    let expected = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing golden {rel} (run with UPDATE_GOLDEN=1): {e}"));
    assert_eq!(expected, actual, "golden mismatch for {rel}");
}

/// The react fixture written for task-71 (`fixtures/react`).
pub const TSX: &str = "index.tsx";
pub const SASS: &str = "NoteCard.module.sass";

pub fn react_sources() -> (String, String, String) {
    (
        fixture("react/index.tsx"),
        fixture("react/NoteCard.module.sass"),
        fixture("react/note-card-variables.sass"),
    )
}

pub fn gpui_source() -> String {
    fixture("gpui/separator.rs")
}

/// The card of the external corpus (`tests/golden.rs` reads it too), when
/// this checkout has it.
pub fn corpus() -> Option<(String, String, String)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../refs/parsecomponentes/UIvisualREFs/cards/hoff-research-cards/react");
    if !root.exists() {
        eprintln!("SKIP: corpus refs/parsecomponentes ausente neste checkout");
        return None;
    }
    let read = |rel: &str| {
        fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
    };
    Some((
        read("components/HoffResearchCard/index.tsx"),
        read("components/HoffResearchCard/HoffResearchCard.module.sass"),
        read("styles/hoff-research-card-variables.sass"),
    ))
}

fn style(s: engine::text::TextStyle) -> TypeStyle {
    TypeStyle {
        size: s.font_size,
        line_height: s.line_height,
        weight: s.font_weight,
    }
}

/// The values of `theme` the IR emitter measures against, field by field
/// by the name each IR token has in the theme.
pub fn tokens(theme: &Theme) -> Tokens {
    let s = &theme.spacing;
    let c = &theme.colors;
    let ty = &theme.typography;
    let sh = &theme.shape;
    let sz = &theme.size;
    use ColorRole as C;
    use TypographyRole as T;
    Tokens {
        spacing: vec![
            (SpacingToken::Xs, s.xs),
            (SpacingToken::Sm, s.sm),
            (SpacingToken::Md, s.md),
            (SpacingToken::Lg, s.lg),
            (SpacingToken::Xl, s.xl),
            (SpacingToken::Xxl, s.xxl),
        ],
        colors: vec![
            (C::Bg, c.bg.0),
            (C::Surface, c.surface.0),
            (C::BgPanel, c.bg_panel.0),
            (C::BgHover, c.bg_hover.0),
            (C::Text, c.text.0),
            (C::TextDim, c.text_dim.0),
            (C::TextMid, c.text_mid.0),
            (C::Accent, c.accent.0),
            (C::AccentDim, c.accent_dim.0),
            (C::Success, c.success.0),
            (C::Danger, c.danger.0),
            (C::Warning, c.warning.0),
            (C::Info, c.info.0),
            (C::Divider, c.divider.0),
            (C::BorderActive, c.border_active.0),
        ],
        typography: vec![
            (T::H4, style(ty.h4())),
            (T::Headline, style(ty.headline())),
            (T::Title, style(ty.title())),
            (T::BaseR, style(ty.base_r())),
            (T::BaseM, style(ty.base_m())),
            (T::Base2r, style(ty.base_2r())),
            (T::Base2m, style(ty.base_2m())),
            (T::Base2sm, style(ty.base_2sm())),
            (T::Base2b, style(ty.base_2b())),
            (T::CaptionR, style(ty.caption_r())),
            (T::CaptionSm, style(ty.caption_sm())),
            (T::SmallR, style(ty.small_r())),
            (T::SmallSm, style(ty.small_sm())),
            (T::Body, style(ty.body())),
            (T::Body2r, style(ty.body_2r())),
            (T::Body2m, style(ty.body_2m())),
            (T::Body2b, style(ty.body_2b())),
            (T::Hairline, style(ty.hairline())),
            (T::Mono, style(ty.mono())),
            (T::Readout, style(ty.readout())),
        ],
        shapes: vec![
            (ShapeRole::Pill, sh.pill),
            (ShapeRole::Dropdown, sh.dropdown),
            (ShapeRole::Tabs, sh.tabs),
            (ShapeRole::Card, sh.card),
            (ShapeRole::Block, sh.block),
            (ShapeRole::Item, sh.item),
            (ShapeRole::Nav, sh.nav),
            (ShapeRole::Cluster, sh.cluster),
            (ShapeRole::Tooltip, sh.tooltip),
            (ShapeRole::Micro, sh.micro),
        ],
        sizes: vec![
            (SizeToken::MenuW, sz.menu_w),
            (SizeToken::ModalMaxW, sz.modal_max_w),
            (SizeToken::SidebarW, sz.sidebar_w),
            (SizeToken::SidebarRailW, sz.sidebar_rail_w),
            (SizeToken::CardW, sz.card_w),
            (SizeToken::CardMinW, sz.card_min_w),
            (SizeToken::ReadableMaxW, sz.readable_max_w),
            (SizeToken::FieldMinW, sz.field_min_w),
            (SizeToken::ToastW, sz.toast_w),
            (SizeToken::TooltipMaxW, sz.tooltip_max_w),
            (SizeToken::DropdownMaxH, sz.dropdown_max_h),
        ],
        edge_width: theme.control.edge_width,
        exprs: vec![
            (
                "theme.glass.inset_highlight".into(),
                Value::Color(theme.glass.inset_highlight.0),
            ),
            (
                "theme.glass.edge_soft".into(),
                Value::Color(theme.glass.edge_soft.0),
            ),
            ("theme.glass.edge".into(), Value::Color(theme.glass.edge.0)),
            (
                "theme.glass.button".into(),
                Value::Color(theme.glass.button.0),
            ),
            (
                "engine::theme::hoff::CARD_OVERLAY".into(),
                Value::Color(engine::theme::hoff::CARD_OVERLAY.0),
            ),
            ("theme.radius.xl".into(), Value::Length(theme.radius.xl)),
            (
                "theme.effects.blur_sigma".into(),
                Value::Length(theme.effects.blur_sigma),
            ),
        ],
    }
}

/// The theme the emitted builder code reads (`let theme = Theme::hoff();`),
/// and so the one its IR is measured against.
pub fn hoff() -> Tokens {
    tokens(&Theme::hoff())
}

pub fn import_react() -> Imported {
    let (tsx, sass, vars) = react_sources();
    parser::import_react((TSX, &tsx), (SASS, &sass), &vars, &hoff()).expect("react import")
}

pub fn import_gpui() -> Imported {
    parser::import_gpui(("separator.rs", &gpui_source()), &hoff()).expect("gpui import")
}

/// The app the note card is written for: its data and its two actions,
/// as the report's bindings ask.
pub fn note_card_app(title: &str, content: &str) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    let mut props = b.node("note_card.props", Role::Group, "note card");
    props.state("title", title).state("content", content);
    props.action("reply").param(ParamSpec {
        kind: ValueKind::Text,
        constrained: false,
    });
    props.action("open");
    b.finish().expect("the note card app projects")
}

/// The app the separator is written for: its label.
pub fn separator_app(label: &str) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    b.node("separator.props", Role::Group, "separator")
        .state("label", label);
    b.finish().expect("the separator app projects")
}

/// The app the corpus card is written for.
pub fn corpus_app(title: &str, content: &str) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    let mut props = b.node("hoff_research_card.props", Role::Group, "card");
    props.state("title", title).state("content", content);
    props.action("discover");
    b.finish().expect("the corpus card app projects")
}
