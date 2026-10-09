//! what a node looks like, by token only: `Props` (spacing, typography,
//! color, elevation, control size, shape), `Layout` (how a container
//! arranges its children) and `Size` (how a node sits in its parent).
//!
//! every token names a field or a style of `engine::theme::Theme` by its
//! own name, so the materializer (task-69) reads the theme and the IR never
//! holds a color or a pixel. the only numbers are proportions: a split's
//! `ratio` and a `flex` weight, which the verifier bounds.

use serde::{Deserialize, Serialize};

/// a closed set of token names. each variant writes as its theme name in
/// the JSON form; `ALL` lists them for the schema.
macro_rules! tokens {
    ($(#[$doc:meta])* $name:ident { $($variant:ident = $text:literal,)+ }) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub enum $name {
            $(#[serde(rename = $text)] $variant,)+
        }

        impl $name {
            /// every token, in declaration order.
            pub const ALL: &'static [$name] = &[$($name::$variant,)+];

            /// the name in the JSON form, the same as the theme's.
            pub fn as_str(self) -> &'static str {
                match self {
                    $($name::$variant => $text,)+
                }
            }
        }
    };
}

tokens! {
    /// a gap or a padding: a field of `theme.spacing` (`SpacingScale`).
    SpacingToken {
        Xs = "xs",
        Sm = "sm",
        Md = "md",
        Lg = "lg",
        Xl = "xl",
        Xxl = "xxl",
    }
}

tokens! {
    /// a text style: a method of `theme.typography` (`TypographyScale`),
    /// the HOFF type ramp.
    TypographyRole {
        H4 = "h4",
        Headline = "headline",
        Title = "title",
        BaseR = "base_r",
        BaseM = "base_m",
        Base2r = "base_2r",
        Base2m = "base_2m",
        Base2sm = "base_2sm",
        Base2b = "base_2b",
        CaptionR = "caption_r",
        CaptionSm = "caption_sm",
        SmallR = "small_r",
        SmallSm = "small_sm",
        Body = "body",
        Body2r = "body_2r",
        Body2m = "body_2m",
        Body2b = "body_2b",
        Hairline = "hairline",
        Mono = "mono",
        Readout = "readout",
    }
}

tokens! {
    /// a color by role: a field of `theme.colors` (`ColorTokens`).
    ColorRole {
        Bg = "bg",
        Surface = "surface",
        BgPanel = "bg_panel",
        BgHover = "bg_hover",
        Text = "text",
        TextDim = "text_dim",
        TextMid = "text_mid",
        Accent = "accent",
        AccentDim = "accent_dim",
        Success = "success",
        Danger = "danger",
        Warning = "warning",
        Info = "info",
        Divider = "divider",
        BorderActive = "border_active",
    }
}

tokens! {
    /// a shadow stack: `engine::theme::Elevation`, read from
    /// `theme.shadows`.
    Elevation {
        Base = "base",
        Raised = "raised",
        Floating = "floating",
        Overlay = "overlay",
        Hint = "hint",
    }
}

tokens! {
    /// a control height: `engine::theme::ControlSize`, read from
    /// `theme.control`.
    ControlSize {
        Xs = "xs",
        Sm = "sm",
        Md = "md",
        Lg = "lg",
        Xl = "xl",
    }
}

tokens! {
    /// a corner radius by role: a field of `theme.shape` (`ShapeTokens`).
    ShapeRole {
        Pill = "pill",
        Dropdown = "dropdown",
        Tabs = "tabs",
        Card = "card",
        Block = "block",
        Item = "item",
        Nav = "nav",
        Cluster = "cluster",
        Tooltip = "tooltip",
        Micro = "micro",
    }
}

tokens! {
    /// a container width or height the design fixes, only ever a bound: a
    /// field of `theme.size` (`SizeTokens`).
    SizeToken {
        MenuW = "menu_w",
        ModalMaxW = "modal_max_w",
        SidebarW = "sidebar_w",
        SidebarRailW = "sidebar_rail_w",
        CardW = "card_w",
        CardMinW = "card_min_w",
        ReadableMaxW = "readable_max_w",
        FieldMinW = "field_min_w",
        ToastW = "toast_w",
        TooltipMaxW = "tooltip_max_w",
        DropdownMaxH = "dropdown_max_h",
    }
}

tokens! {
    /// the direction a stack or a split runs in.
    Axis {
        Horizontal = "horizontal",
        Vertical = "vertical",
    }
}

tokens! {
    /// where a stack places its children across its axis.
    Align {
        Start = "start",
        Center = "center",
        End = "end",
        Stretch = "stretch",
    }
}

/// how a node looks, every field a token and every field optional: what is
/// left out is the component's default in the theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Props {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<SpacingToken>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typography: Option<TypographyRole>,
    /// the foreground: text and icon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorRole>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<ColorRole>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elevation: Option<Elevation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_size: Option<ControlSize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<ShapeRole>,
}

impl Props {
    pub fn is_empty(&self) -> bool {
        *self == Props::default()
    }
}

/// how a container arranges its children. a node without a layout stacks
/// its children vertically with no gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Layout {
    /// one child after the other along `axis`.
    Stack {
        axis: Axis,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gap: Option<SpacingToken>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align: Option<Align>,
    },
    /// as many columns as fit, none narrower than `min_col`.
    Grid {
        min_col: SizeToken,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gap: Option<SpacingToken>,
    },
    /// exactly two children along `axis`, the first taking `ratio`
    /// percent (from `MIN_RATIO` to `MAX_RATIO`).
    Split { axis: Axis, ratio: u8 },
    /// every child over the one before it, in order.
    Overlay,
}

/// the smallest first pane of a split, in percent.
pub const MIN_RATIO: u8 = 10;
/// the largest first pane of a split, in percent.
pub const MAX_RATIO: u8 = 90;
/// the largest `flex` weight, the columns of the widest grid in the theme.
pub const MAX_FLEX: u8 = 12;

/// how a node sits in its parent: bounds by token and a weight for the
/// free space (from 1 to `MAX_FLEX`). all optional: without them the node
/// takes what its content asks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Size {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<SizeToken>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<SizeToken>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flex: Option<u8>,
}

impl Size {
    pub fn is_empty(&self) -> bool {
        *self == Size::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_writes_as_its_theme_name() {
        let props = Props {
            padding: Some(SpacingToken::Xxl),
            typography: Some(TypographyRole::Base2sm),
            color: Some(ColorRole::TextDim),
            ..Props::default()
        };
        let json = serde_json::to_string(&props).unwrap();
        assert_eq!(
            json,
            r#"{"padding":"xxl","typography":"base_2sm","color":"text_dim"}"#
        );
        assert_eq!(serde_json::from_str::<Props>(&json).unwrap(), props);
        for t in TypographyRole::ALL {
            let text = serde_json::to_string(t).unwrap();
            assert_eq!(text, format!("{:?}", t.as_str()));
        }
    }

    #[test]
    fn a_literal_is_not_a_token() {
        for bad in [
            r##"{"color":"#ffffff"}"##,
            r#"{"padding":12}"#,
            r#"{"background":"rgb(0,0,0)"}"#,
            r#"{"margin":"md"}"#,
        ] {
            assert!(serde_json::from_str::<Props>(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_layout_is_one_of_four_primitives() {
        let stack = Layout::Stack {
            axis: Axis::Horizontal,
            gap: Some(SpacingToken::Sm),
            align: None,
        };
        let json = serde_json::to_string(&stack).unwrap();
        assert_eq!(json, r#"{"stack":{"axis":"horizontal","gap":"sm"}}"#);
        assert_eq!(serde_json::from_str::<Layout>(&json).unwrap(), stack);
        assert_eq!(
            serde_json::to_string(&Layout::Overlay).unwrap(),
            r#""overlay""#
        );
        assert!(serde_json::from_str::<Layout>(r#"{"flow":{}}"#).is_err());
        let extra = r#"{"stack":{"axis":"vertical","wrap":true}}"#;
        assert!(serde_json::from_str::<Layout>(extra).is_err());
    }
}
