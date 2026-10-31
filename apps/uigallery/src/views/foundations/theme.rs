//! Theme page, the hoff core kit's ColorsPage: groups with a title and a
//! bottom rule, one swatch card per color, its name and its hex under the
//! swatch. The colors are the active theme's tokens (its neutrals, then
//! its intents), so the page reads the same under every theme; the last
//! group is every built-in palette, painted in its own colors: a click
//! applies it (the shell's `shell.theme`, the same capability as the T
//! key and the menu).

use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::color::Color;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent, rounded_rect, rounded_rect_stroke};

use super::super::components::showcase::{self, Grid, GroupLayout};

pub const THEMES: &[&str] = &[
    "core",
    "hoff",
    "dark",
    "light",
    "catppuccin",
    "dracula",
    "tokyo-night",
    "rose-pine",
    "nord",
    "gruvbox",
    "github-dark",
    "one-dark",
    "kanagawa",
    "moonlight",
];

/// Resolve a showcase theme name to a `Theme`.
pub fn resolve(name: &str) -> Option<Theme> {
    match name {
        "dark" => Some(Theme::dark()),
        "light" => Some(Theme::light()),
        other => Theme::named(other),
    }
}

/// Swatches to a row at full width (`w-[calc(20%-1rem)]`).
const SPAN: usize = 5;
/// The swatch's name, 14/1.25 medium, and its hex, 10/16: the ratios of
/// the kit's `text-[0.875rem] leading-[1.25]` and `leading-[1rem]`.
const NAME_LINE: f32 = 1.25;
const NAME_WEIGHT: u16 = 500;
const HEX_LINE_H: f32 = 16.0;

/// One swatch: the names of the tokens that hold it and its color.
struct Swatch {
    name: String,
    color: Color,
}

/// The theme's neutrals, surface to ink, each color once with every
/// token that holds it.
fn neutrals(theme: &Theme) -> Vec<Swatch> {
    let c = &theme.colors;
    let roles = [
        ("bg", c.bg),
        ("surface", c.surface),
        ("bg panel", c.bg_panel),
        ("bg hover", c.bg_hover),
        ("divider", c.divider),
        ("border active", c.border_active),
        ("text dim", c.text_dim),
        ("text mid", c.text_mid),
        ("accent dim", c.accent_dim),
        ("text", c.text),
    ];
    merged(&roles)
}

/// The theme's intents: the accent and the four status colors.
fn intents(theme: &Theme) -> Vec<Swatch> {
    let c = &theme.colors;
    merged(&[
        ("accent", c.accent),
        ("success", c.success),
        ("info", c.info),
        ("warning", c.warning),
        ("danger", c.danger),
    ])
}

/// `roles` with the ones of the same color folded into one swatch.
fn merged(roles: &[(&str, Color)]) -> Vec<Swatch> {
    let mut out: Vec<Swatch> = Vec::new();
    for (name, color) in roles {
        match out.iter_mut().find(|s| s.color == *color) {
            Some(s) => s.name = format!("{} · {name}", s.name),
            None => out.push(Swatch {
                name: (*name).to_string(),
                color: *color,
            }),
        }
    }
    out
}

/// `RRGGBB`, and the opacity after it when the color is not opaque.
pub(crate) fn hex(color: Color) -> String {
    let [r, g, b, a] = color.0.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
    if a == 255 {
        format!("{r:02X}{g:02X}{b:02X}")
    } else {
        format!("{r:02X}{g:02X}{b:02X} · {}%", (a as f32 / 2.55).round())
    }
}

/// The swatch card's measures, from the theme: `p-1.5`, the `h-15`
/// swatch, `mb-1.5`, the `p-2` caption.
struct Metrics {
    pad: f32,
    swatch_h: f32,
    caption_pad: f32,
    name: TextStyle,
    hex: TextStyle,
}

impl Metrics {
    fn of(theme: &Theme) -> Self {
        let t = &theme.typography;
        // The kit's 14 is not a step of the ramp: the nearest, `body_sm`.
        let name_size = t.body_sm;
        Self {
            pad: theme.spacing.xs * 1.5,
            swatch_h: theme.spacing.xxl + theme.spacing.md,
            caption_pad: theme.spacing.sm,
            name: t.exact(name_size, name_size * NAME_LINE, NAME_WEIGHT),
            hex: t.exact(t.small, HEX_LINE_H, 400),
        }
    }

    fn card_h(&self) -> f32 {
        self.pad * 2.0
            + self.swatch_h
            + self.pad
            + self.caption_pad * 2.0
            + self.name.line_height
            + self.hex.line_height
    }
}

pub struct ThemePage {
    hovered: Option<usize>,
}

impl ThemePage {
    pub fn new() -> Self {
        Self { hovered: None }
    }

    /// The narrowest a swatch card gets: half the theme's narrowest card.
    fn min_w(theme: &Theme) -> f32 {
        theme.size.card_min_w / 2.0
    }

    /// The three groups: neutrals, intents, palettes.
    fn layout(content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let h = Metrics::of(theme).card_h();
        let counts = [neutrals(theme).len(), intents(theme).len(), THEMES.len()];
        let grids = counts.map(|n| Grid {
            span: SPAN,
            min_w: Self::min_w(theme),
            heights: vec![h; n],
        });
        showcase::layout_grids(grids, content, theme)
    }

    /// Natural height of the page (page scrolling needs it).
    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&Self::layout(content, theme), content)
    }

    /// Returns the picked theme name, if a palette card was clicked.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> (EventResult, Option<&'static str>) {
        let layout = Self::layout(content, theme);
        let Some(palettes) = layout.last() else {
            return (EventResult::IGNORED, None);
        };
        let hit = |x: f32, y: f32| palettes.cards.iter().position(|r| r.contains(x, y));
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let at = hit(x, y);
                if at == self.hovered {
                    return (EventResult::IGNORED, None);
                }
                self.hovered = at;
                (EventResult::changed(), None)
            }
            WidgetEvent::MouseDown { x, y } => match hit(x, y) {
                Some(i) => (EventResult::clicked(), Some(THEMES[i])),
                None => (EventResult::IGNORED, None),
            },
            _ => (EventResult::IGNORED, None),
        }
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme, current: &str) {
        let layout = Self::layout(content, theme);
        let m = Metrics::of(theme);
        let titles = ["Neutrals", "Colors", "Palettes"];
        let last = layout.len().saturating_sub(1);
        for (g, (group, title)) in layout.iter().zip(titles).enumerate() {
            showcase::title(c, title, content, group.title_y, theme);
            match g {
                0 | 1 => {
                    let swatches = if g == 0 {
                        neutrals(theme)
                    } else {
                        intents(theme)
                    };
                    for (s, rect) in swatches.iter().zip(&group.cards) {
                        let caption = hex(s.color);
                        swatch_card(c, *rect, &[s.color], &s.name, &caption, false, &m, theme);
                    }
                }
                _ => {
                    for (i, (name, rect)) in THEMES.iter().zip(&group.cards).enumerate() {
                        let Some(t) = resolve(name) else { continue };
                        let colors = [
                            t.colors.bg,
                            t.colors.bg_panel,
                            t.colors.accent,
                            t.colors.success,
                            t.colors.danger,
                            t.colors.text,
                        ];
                        let state = if *name == current {
                            "current"
                        } else if self.hovered == Some(i) {
                            "apply"
                        } else {
                            ""
                        };
                        let lit = *name == current || self.hovered == Some(i);
                        swatch_card(c, *rect, &colors, name, state, lit, &m, theme);
                    }
                }
            }
            if g < last {
                showcase::rule(c, content, group.rule_y, theme);
            }
        }
    }
}

/// A ColorsPage card: the rounded shell, the swatch (one color, or a strip
/// of a palette's), the name and the caption under it. `lit` draws the
/// shell in the theme's active border (the current or hovered palette).
#[allow(clippy::too_many_arguments)]
fn swatch_card(
    c: &mut Compositor,
    rect: Rect,
    colors: &[Color],
    name: &str,
    caption: &str,
    lit: bool,
    m: &Metrics,
    theme: &Theme,
) {
    let edge = theme.control.edge_width;
    let border = if lit {
        theme.colors.border_active
    } else {
        theme.colors.divider
    };
    c.push(rounded_rect_stroke(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        theme.shape.nav,
        border.0,
        edge,
    ));
    let well = Rect::new(
        rect.x + m.pad,
        rect.y + m.pad,
        rect.w - m.pad * 2.0,
        m.swatch_h,
    );
    let radius = theme.shape.cluster;
    swatch_strip(c, well, radius, colors);
    c.push(rounded_rect_stroke(
        well.x,
        well.y,
        well.w,
        well.h,
        radius,
        theme.colors.divider.0,
        edge,
    ));
    let x = well.x + m.caption_pad;
    let room = well.w - m.caption_pad * 2.0;
    let y = well.bottom() + m.pad + m.caption_pad;
    let name = TextMeasurer::truncate_to_width(name, &m.name, room);
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(&name, &m.name, None),
        x,
        y,
        color: theme.colors.text.0,
    });
    let caption = TextMeasurer::truncate_to_width(caption, &m.hex, room);
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(&caption, &m.hex, None),
        x,
        y: y + m.name.line_height,
        color: theme.colors.text_mid.0,
    });
}

/// `colors` side by side across `well`, its outer corners at `radius`:
/// the first fills the whole rounded well, the middle ones are square
/// bands over it, the last a rounded band whose left round a square band
/// covers.
fn swatch_strip(c: &mut Compositor, well: Rect, radius: f32, colors: &[Color]) {
    let Some((first, rest)) = colors.split_first() else {
        return;
    };
    c.push(rounded_rect(
        well.x, well.y, well.w, well.h, radius, first.0,
    ));
    let band = well.w / colors.len() as f32;
    for (i, color) in rest.iter().enumerate() {
        let x = well.x + (i + 1) as f32 * band;
        if i + 1 == rest.len() {
            let w = well.right() - x;
            c.push(rounded_rect(x, well.y, w, well.h, radius, color.0));
            c.push(rounded_rect(x, well.y, radius.min(w), well.h, 0.0, color.0));
        } else {
            c.push(rounded_rect(x, well.y, band, well.h, 0.0, color.0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_core_neutrals_are_the_kit_shades_but_shade_8() {
        let theme = Theme::hoff_core();
        let hexes: Vec<String> = neutrals(&theme).iter().map(|s| hex(s.color)).collect();
        // Shade 1..9 of the ColorsPage, less shade 8 (222222), which no
        // token holds.
        assert_eq!(
            hexes,
            [
                "FCFCFC", "F8F7F7", "F1F1F1", "ECECEC", "E2E2E2", "7B7B7B", "323232", "121212"
            ]
        );
        assert!(neutrals(&theme)[0].name.contains("surface"));
    }

    #[test]
    fn a_hex_names_its_opacity_when_it_has_one() {
        assert_eq!(hex(Color::hex(0x49BA61)), "49BA61");
        assert_eq!(hex(Color::rgba(0.0, 0.0, 0.0, 0.5)), "000000 · 50%");
    }

    /// The swatches stay inside the content, one column at the narrowest,
    /// five at full width, under every theme.
    #[test]
    fn swatches_fit_from_narrow_to_wide() {
        for theme in [Theme::hoff_core(), Theme::hoff(), Theme::dark()] {
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                for group in ThemePage::layout(content, &theme) {
                    for r in &group.cards {
                        assert!(r.right() <= content.right() + 0.5, "{w}: {r:?}");
                        assert!(r.w >= ThemePage::min_w(&theme).min(content.w) - 0.5);
                    }
                }
            }
            let wide = Rect::new(0.0, 0.0, 1200.0, 800.0);
            let group = &ThemePage::layout(wide, &theme)[2];
            assert_eq!(group.cards[0].y, group.cards[SPAN - 1].y);
        }
    }

    /// A palette card picks its theme; a swatch picks nothing.
    #[test]
    fn a_palette_card_picks_its_theme() {
        let theme = Theme::hoff_core();
        let content = Rect::new(0.0, 0.0, 1000.0, 800.0);
        let layout = ThemePage::layout(content, &theme);
        let mut page = ThemePage::new();
        let at = |r: Rect| WidgetEvent::MouseDown {
            x: r.x + r.w / 2.0,
            y: r.y + r.h / 2.0,
        };
        let (_, picked) = page.handle_event(&at(layout[2].cards[2]), content, &theme);
        assert_eq!(picked, Some(THEMES[2]));
        let (r, picked) = page.handle_event(&at(layout[0].cards[0]), content, &theme);
        assert_eq!(picked, None);
        assert_eq!(r, EventResult::IGNORED);
    }
}
