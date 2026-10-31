//! The contrast of the four themes' text and controls against what they
//! sit on, measured against WCAG 2.2 AA (task-75 phase 1): 4.5 for text
//! (1.4.3), 3 for large text (1.4.3) and for the parts of a control a
//! person must see to use it (1.4.11: the focus ring, an intent color, an
//! active border, an icon).
//!
//! What is measured is what the window shows: a translucent token is
//! composed over the page's `bg` (or over the surface it names) in linear
//! light, as the GPU blends into the sRGB surface (`Color::to_linear_array`,
//! `crate::color`), and the relative luminance is WCAG's, from the linear
//! channels. The pairs are the ones the design draws: every text token on
//! every surface text sits on, the tooltip's own text on the tooltip
//! (`comps` feedback/tooltip.rs draws `text_mid` there), the headline
//! gradient as large text, the intents, the accent (the focus ring,
//! `comps` recipe.rs) and the active border on the page and on a card, and
//! each button's label on its own fill at rest, hovered and pressed. A
//! disabled control is left out, as WCAG leaves it out.
//!
//! A pair that fails is not fixed here: a theme's color is a design
//! decision and moves every baseline. The pairs that fail today are listed
//! in `KNOWN`, with their ratio; the test fails when one of them starts
//! passing (take it off the list) and when a pair that passed starts
//! failing.

use crate::styling::color::Color;
use crate::styling::theme::{ButtonRecipe, Theme};

/// WCAG 2.2 AA: text (1.4.3).
const TEXT: f64 = 4.5;
/// WCAG 2.2 AA: large text (1.4.3) and non-text contrast (1.4.11).
const LARGE: f64 = 3.0;

/// The four themes the gallery cycles with `T`.
fn themes() -> [(&'static str, Theme); 4] {
    [
        ("core", Theme::hoff_core()),
        ("hoff", Theme::hoff()),
        ("dark", Theme::dark()),
        ("light", Theme::light()),
    ]
}

/// One channel from sRGB to linear light (IEC 61966-2-1, the curve
/// `Color::to_linear_array` applies).
fn linear(c: f32) -> f64 {
    let c = f64::from(c);
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// `fg` composed over the opaque linear color `under`, in linear light.
fn over(fg: Color, under: [f64; 3]) -> [f64; 3] {
    let a = f64::from(fg.0[3]);
    let [r, g, b] = [fg.0[0], fg.0[1], fg.0[2]].map(linear);
    [
        r * a + under[0] * (1.0 - a),
        g * a + under[1] * (1.0 - a),
        b * a + under[2] * (1.0 - a),
    ]
}

/// WCAG relative luminance of a linear color.
fn luminance([r, g, b]: [f64; 3]) -> f64 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// WCAG contrast ratio, lighter over darker, 1 to 21.
fn ratio(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// One pair as the window shows it.
#[derive(Debug)]
struct Pair {
    theme: &'static str,
    fg: String,
    bg: String,
    need: f64,
    ratio: f64,
}

/// Every pair of one theme (see the module docs for which).
fn pairs(theme: &'static str, t: &Theme) -> Vec<Pair> {
    let c = &t.colors;
    let g = &t.glass;
    let page = over(c.bg, [0.0; 3]);
    let surfaces: [(&str, [f64; 3]); 5] = [
        ("colors.bg", page),
        ("colors.surface", over(c.surface, page)),
        ("colors.bg_panel", over(c.bg_panel, page)),
        ("colors.bg_hover", over(c.bg_hover, page)),
        ("glass.popover", over(g.popover, page)),
    ];
    let texts = [
        ("colors.text", c.text),
        ("colors.text_mid", c.text_mid),
        ("colors.text_dim", c.text_dim),
        ("glass.text_active", g.text_active),
        ("glass.text_default", g.text_default),
        ("glass.text_faint", g.text_faint),
        ("glass.text_placeholder", g.text_placeholder),
    ];
    let mut out = Vec::new();
    let mut add = |fg: &str, bg: &str, need: f64, fg_rgb: [f64; 3], bg_rgb: [f64; 3]| {
        out.push(Pair {
            theme,
            fg: fg.to_owned(),
            bg: bg.to_owned(),
            need,
            ratio: ratio(fg_rgb, bg_rgb),
        });
    };
    for (bg, under) in surfaces {
        for (fg, color) in texts {
            add(fg, bg, TEXT, over(color, under), under);
        }
    }
    let tooltip = over(g.tooltip, page);
    add(
        "colors.text_mid",
        "glass.tooltip",
        TEXT,
        over(c.text_mid, tooltip),
        tooltip,
    );
    let [page_bg, card] = [surfaces[0], surfaces[1]];
    for (bg, under) in [page_bg, card] {
        for (i, stop) in g.text_gradient.iter().enumerate() {
            add(
                &format!("glass.text_gradient[{i}]"),
                bg,
                LARGE,
                over(*stop, under),
                under,
            );
        }
        for (fg, color) in [
            ("colors.accent", c.accent),
            ("colors.success", c.success),
            ("colors.danger", c.danger),
            ("colors.warning", c.warning),
            ("colors.info", c.info),
            ("colors.border_active", c.border_active),
        ] {
            add(fg, bg, LARGE, over(color, under), under);
        }
    }
    let b = &t.button;
    let recipes: [(&str, &ButtonRecipe, f64); 7] = [
        ("primary", &b.primary, TEXT),
        ("secondary", &b.secondary, TEXT),
        ("warning", &b.warning, TEXT),
        ("outline", &b.outline, TEXT),
        // the plain icon buttons: what they draw is an icon
        ("ghost", &b.ghost, LARGE),
        ("toolbar", &b.toolbar, TEXT),
        ("toolbar_active", &b.toolbar_active, TEXT),
    ];
    for (name, recipe, need) in recipes {
        for (state, s) in [
            ("rest", &recipe.rest),
            ("hover", &recipe.hover),
            ("active", &recipe.active),
        ] {
            // a flat fill is one background; a gradient, its two ends
            let stops: &[(Color, &str)] = if s.fill[0] == s.fill[1] {
                &[(s.fill[0], "fill")]
            } else {
                &[(s.fill[0], "fill[0]"), (s.fill[1], "fill[1]")]
            };
            for (fill, at) in stops {
                let under = over(s.overlay, over(*fill, page));
                add(
                    &format!("button.{name}.{state}.text"),
                    &format!("button.{name}.{state}.{at}"),
                    need,
                    over(s.text, under),
                    under,
                );
            }
        }
    }
    // task-104 front A: the field on a card, its box at rest (the
    // text, the placeholder and the icons inside, the border around)
    // and the label row and the message line around it.
    let f = &t.form;
    let card = surfaces[1].1;
    let field = over(f.input.rest.fill, card);
    let link = Color([
        f.field.link_color.0[0],
        f.field.link_color.0[1],
        f.field.link_color.0[2],
        f.field.link_color.0[3] * f.field.link_alpha,
    ]);
    for (fg, bg, need, color, under) in [
        (
            "form.input.text_color",
            "form.input.rest.fill",
            TEXT,
            f.input.text_color,
            field,
        ),
        (
            "form.input.placeholder",
            "form.input.rest.fill",
            TEXT,
            f.input.placeholder,
            field,
        ),
        (
            "form.adorn.icon_color",
            "form.input.rest.fill",
            LARGE,
            f.adorn.icon_color,
            field,
        ),
        (
            "form.input.rest.border",
            "colors.surface",
            LARGE,
            f.input.rest.border,
            card,
        ),
        (
            "form.label.color",
            "colors.surface",
            TEXT,
            f.label.color,
            card,
        ),
        ("form.field.link_color", "colors.surface", TEXT, link, card),
        (
            "form.field.error_color",
            "colors.surface",
            TEXT,
            f.field.error_color,
            card,
        ),
        (
            "form.field.helper_color",
            "colors.surface",
            TEXT,
            f.field.helper_color,
            card,
        ),
    ] {
        add(fg, bg, need, over(color, under), under);
    }
    // task-104 front C: the inline alert on a card, its runs and its
    // unread dot, and the intent's icon on its badge (ink or paper,
    // whichever reads on the fill, `contrast_text`).
    let a = &t.message.alert;
    for (fg, need, color) in [
        ("message.alert.title", TEXT, a.title.color),
        ("message.alert.time", TEXT, a.time.color),
        ("message.alert.detail", TEXT, a.detail.color),
        ("message.alert.content", TEXT, a.content.color),
        ("message.alert.unread", LARGE, a.unread),
    ] {
        add(fg, "colors.surface", need, over(color, card), card);
    }
    for (name, fill) in [
        ("neutral", a.intents.neutral),
        ("informational", a.intents.informational),
        ("constructive", a.intents.constructive),
        ("destructive", a.intents.destructive),
    ] {
        let under = over(fill, card);
        let icon = Color(crate::widgets::widget::contrast_text(fill.0));
        add(
            "message.alert.icon",
            &format!("message.alert.intents.{name}"),
            LARGE,
            over(icon, under),
            under,
        );
    }
    // task-104 front B: the sheet's and the drawer's text and icons on
    // their panel over the page, a drawer link's also on its lit fill. On
    // core the kit's #7B7B7B hint and group label fail AA; they take
    // shade 7 (`overlay/sheet/tokens.rs`), which these pairs hold.
    let o = &t.overlay;
    for (name, sheet_or_drawer, fill) in [
        ("sheet", true, o.sheet.panel.fill),
        ("drawer", false, o.drawer.panel.fill),
    ] {
        let under = over(fill, page);
        let bg = format!("overlay.{name}.panel.fill");
        let texts: Vec<(&str, f64, Color)> = if sheet_or_drawer {
            let s = &o.sheet;
            vec![
                ("title_color", TEXT, s.title_color),
                ("description_color", TEXT, s.description_color),
                ("label_color", TEXT, s.label_color),
                ("hint_color", TEXT, s.hint_color),
                ("value_color", TEXT, s.value_color),
            ]
        } else {
            let d = &o.drawer;
            vec![
                ("brand_color", TEXT, d.brand_color),
                ("group_color", TEXT, d.group_color),
                ("item_color", TEXT, d.item_color),
                ("icon_color", LARGE, d.icon_color),
            ]
        };
        for (fg, need, color) in texts {
            add(
                &format!("overlay.{name}.{fg}"),
                &bg,
                need,
                over(color, under),
                under,
            );
        }
    }
    // The modal's sheet: its title and its body on its fill.
    let m = &o.modal;
    let sheet = over(m.fill, page);
    for (fg, color) in [("title_color", m.title_color), ("body_color", m.body_color)] {
        let fg = format!("overlay.modal.{fg}");
        add(&fg, "overlay.modal.fill", TEXT, over(color, sheet), sheet);
    }
    let d = &o.drawer;
    let lit = over(d.item_fill, over(d.panel.fill, page));
    for (fg, need, color) in [
        ("item_color", TEXT, d.item_color),
        ("icon_active", LARGE, d.icon_active),
    ] {
        let fg = format!("overlay.drawer.{fg}");
        add(&fg, "overlay.drawer.item_fill", need, over(color, lit), lit);
    }
    out
}

/// The pairs that fail today, 116 of the 369 (core 41, hoff 18, dark 13,
/// light 44), by theme, text and background, with the ratio the test
/// measured (for the record; the test compares the list, not the
/// numbers). Changing a theme's color is the design's decision (task-75
/// phase 1 reports them); this list is how the test keeps counting until
/// then.
const KNOWN: &[(&str, &str, &str, f64)] = &[
    ("core", "colors.text_mid", "colors.bg", 4.13),
    ("core", "colors.text_dim", "colors.bg", 4.13),
    ("core", "glass.text_default", "colors.bg", 4.13),
    ("core", "glass.text_faint", "colors.bg", 4.13),
    ("core", "glass.text_placeholder", "colors.bg", 1.61),
    ("core", "colors.text_mid", "colors.surface", 4.13),
    ("core", "colors.text_dim", "colors.surface", 4.13),
    ("core", "glass.text_default", "colors.surface", 4.13),
    ("core", "glass.text_faint", "colors.surface", 4.13),
    ("core", "glass.text_placeholder", "colors.surface", 1.61),
    ("core", "colors.text_mid", "colors.bg_panel", 3.96),
    ("core", "colors.text_dim", "colors.bg_panel", 3.96),
    ("core", "glass.text_default", "colors.bg_panel", 3.96),
    ("core", "glass.text_faint", "colors.bg_panel", 3.96),
    ("core", "glass.text_placeholder", "colors.bg_panel", 1.61),
    ("core", "colors.text_mid", "colors.bg_hover", 3.75),
    ("core", "colors.text_dim", "colors.bg_hover", 3.75),
    ("core", "glass.text_default", "colors.bg_hover", 3.75),
    ("core", "glass.text_faint", "colors.bg_hover", 3.75),
    ("core", "glass.text_placeholder", "colors.bg_hover", 1.60),
    ("core", "colors.text_mid", "glass.popover", 4.13),
    ("core", "colors.text_dim", "glass.popover", 4.13),
    ("core", "glass.text_default", "glass.popover", 4.13),
    ("core", "glass.text_faint", "glass.popover", 4.13),
    ("core", "glass.text_placeholder", "glass.popover", 1.61),
    ("core", "colors.text_mid", "glass.tooltip", 4.43),
    ("core", "colors.success", "colors.bg", 2.41),
    ("core", "colors.border_active", "colors.bg", 1.26),
    ("core", "colors.success", "colors.surface", 2.41),
    ("core", "colors.border_active", "colors.surface", 1.26),
    (
        "core",
        "button.warning.rest.text",
        "button.warning.rest.fill[0]",
        3.37,
    ),
    (
        "core",
        "button.warning.rest.text",
        "button.warning.rest.fill[1]",
        3.65,
    ),
    (
        "core",
        "button.warning.hover.text",
        "button.warning.hover.fill[0]",
        2.70,
    ),
    (
        "core",
        "button.warning.hover.text",
        "button.warning.hover.fill[1]",
        2.86,
    ),
    (
        "core",
        "button.warning.active.text",
        "button.warning.active.fill[0]",
        3.37,
    ),
    (
        "core",
        "button.warning.active.text",
        "button.warning.active.fill[1]",
        3.65,
    ),
    ("hoff", "glass.text_placeholder", "colors.bg", 3.86),
    ("hoff", "glass.text_placeholder", "colors.surface", 3.95),
    ("hoff", "glass.text_placeholder", "colors.bg_panel", 3.39),
    ("hoff", "colors.text_dim", "colors.bg_hover", 4.45),
    ("hoff", "glass.text_faint", "colors.bg_hover", 3.76),
    ("hoff", "glass.text_placeholder", "colors.bg_hover", 2.73),
    ("hoff", "glass.text_placeholder", "glass.popover", 3.39),
    ("hoff", "colors.danger", "colors.bg", 2.27),
    ("hoff", "colors.border_active", "colors.bg", 2.22),
    ("hoff", "colors.danger", "colors.surface", 2.34),
    ("hoff", "colors.border_active", "colors.surface", 2.26),
    (
        "hoff",
        "button.secondary.hover.text",
        "button.secondary.hover.fill",
        1.57,
    ),
    (
        "hoff",
        "button.warning.rest.text",
        "button.warning.rest.fill",
        3.27,
    ),
    (
        "hoff",
        "button.warning.hover.text",
        "button.warning.hover.fill",
        3.40,
    ),
    (
        "hoff",
        "button.warning.active.text",
        "button.warning.active.fill",
        3.27,
    ),
    ("dark", "colors.text_dim", "colors.bg", 2.82),
    ("dark", "colors.text_dim", "colors.surface", 2.66),
    ("dark", "glass.text_placeholder", "colors.surface", 4.50),
    ("dark", "colors.text_dim", "colors.bg_panel", 2.53),
    ("dark", "glass.text_placeholder", "colors.bg_panel", 4.33),
    ("dark", "colors.text_dim", "colors.bg_hover", 2.33),
    ("dark", "glass.text_placeholder", "colors.bg_hover", 4.05),
    ("dark", "colors.text_dim", "glass.popover", 2.53),
    ("dark", "glass.text_placeholder", "glass.popover", 4.33),
    ("dark", "colors.border_active", "colors.bg", 2.16),
    ("dark", "colors.border_active", "colors.surface", 2.03),
    ("light", "colors.text_dim", "colors.bg", 4.50),
    ("light", "glass.text_active", "colors.bg", 3.50),
    ("light", "glass.text_default", "colors.bg", 2.11),
    ("light", "glass.text_faint", "colors.bg", 1.60),
    ("light", "glass.text_placeholder", "colors.bg", 1.31),
    ("light", "glass.text_active", "colors.surface", 3.53),
    ("light", "glass.text_default", "colors.surface", 2.12),
    ("light", "glass.text_faint", "colors.surface", 1.61),
    ("light", "glass.text_placeholder", "colors.surface", 1.31),
    ("light", "colors.text_dim", "colors.bg_panel", 4.12),
    ("light", "glass.text_active", "colors.bg_panel", 3.45),
    ("light", "glass.text_default", "colors.bg_panel", 2.10),
    ("light", "glass.text_faint", "colors.bg_panel", 1.60),
    ("light", "glass.text_placeholder", "colors.bg_panel", 1.30),
    ("light", "colors.text_dim", "colors.bg_hover", 3.84),
    ("light", "glass.text_active", "colors.bg_hover", 3.41),
    ("light", "glass.text_default", "colors.bg_hover", 2.09),
    ("light", "glass.text_faint", "colors.bg_hover", 1.59),
    ("light", "glass.text_placeholder", "colors.bg_hover", 1.30),
    ("light", "colors.text_dim", "glass.popover", 4.12),
    ("light", "glass.text_active", "glass.popover", 3.45),
    ("light", "glass.text_default", "glass.popover", 2.10),
    ("light", "glass.text_faint", "glass.popover", 1.60),
    ("light", "glass.text_placeholder", "glass.popover", 1.30),
    ("light", "glass.text_gradient[1]", "colors.bg", 1.89),
    ("light", "colors.warning", "colors.bg", 2.15),
    ("light", "colors.info", "colors.bg", 2.76),
    ("light", "colors.border_active", "colors.bg", 2.73),
    ("light", "glass.text_gradient[1]", "colors.surface", 1.89),
    ("light", "colors.warning", "colors.surface", 2.24),
    ("light", "colors.info", "colors.surface", 2.87),
    ("light", "colors.border_active", "colors.surface", 2.84),
    (
        "light",
        "button.primary.rest.text",
        "button.primary.rest.fill",
        4.06,
    ),
    (
        "light",
        "button.primary.active.text",
        "button.primary.active.fill",
        4.06,
    ),
    (
        "light",
        "button.secondary.rest.text",
        "button.secondary.rest.fill",
        1.09,
    ),
    (
        "light",
        "button.secondary.hover.text",
        "button.secondary.hover.fill",
        1.17,
    ),
    (
        "light",
        "button.secondary.active.text",
        "button.secondary.active.fill",
        1.09,
    ),
    (
        "light",
        "button.warning.rest.text",
        "button.warning.rest.fill",
        2.15,
    ),
    (
        "light",
        "button.warning.hover.text",
        "button.warning.hover.fill",
        1.83,
    ),
    (
        "light",
        "button.warning.active.text",
        "button.warning.active.fill",
        2.15,
    ),
    // task-104 front A: the field. The core pairs are the kit's Field as
    // written (`components/Field/index.tsx`: the #E2E2E2 border, the
    // #7B7B7B placeholder, the #FE5938 error, the link at opacity-70) and
    // the helper in the kit's secondary gray; the others are the glass
    // field every theme drew before the kit.
    (
        "core",
        "form.input.placeholder",
        "form.input.rest.fill",
        4.13,
    ),
    ("core", "form.input.rest.border", "colors.surface", 1.26),
    ("core", "form.field.link_color", "colors.surface", 2.13),
    ("core", "form.field.error_color", "colors.surface", 3.05),
    ("core", "form.field.helper_color", "colors.surface", 4.13),
    (
        "hoff",
        "form.input.placeholder",
        "form.input.rest.fill",
        2.76,
    ),
    ("hoff", "form.input.rest.border", "colors.surface", 1.63),
    ("hoff", "form.field.error_color", "colors.surface", 2.34),
    (
        "dark",
        "form.input.placeholder",
        "form.input.rest.fill",
        2.96,
    ),
    ("dark", "form.input.rest.border", "colors.surface", 1.70),
    (
        "light",
        "form.input.placeholder",
        "form.input.rest.fill",
        1.31,
    ),
    ("light", "form.input.rest.border", "colors.surface", 1.05),
    ("light", "form.field.error_color", "colors.surface", 4.24),
    // task-104 front C: the alert's unread dot takes the info accent,
    // which the light theme already draws under 3:1.
    ("light", "message.alert.unread", "colors.surface", 2.87),
];

#[test]
fn the_ratio_is_wcags() {
    let white = over(Color::WHITE, [0.0; 3]);
    let black = over(Color::BLACK, [0.0; 3]);
    assert!((ratio(white, black) - 21.0).abs() < 1e-9);
    assert!((ratio(black, white) - 21.0).abs() < 1e-9, "either order");
    assert!((ratio(white, white) - 1.0).abs() < 1e-9);
    // #767676 on white is the classic 4.54, #777777 the classic 4.48
    let on_white = |hex: u32| ratio(over(Color::hex(hex), [0.0; 3]), white);
    assert!(
        (on_white(0x767676) - 4.54).abs() < 0.01,
        "{}",
        on_white(0x767676)
    );
    assert!(
        (on_white(0x777777) - 4.48).abs() < 0.01,
        "{}",
        on_white(0x777777)
    );
    // half white over black, in linear light: luminance 0.5
    let half = over(Color::rgba(1.0, 1.0, 1.0, 0.5), black);
    assert!((luminance(half) - 0.5).abs() < 1e-9);
}

#[test]
fn every_pair_of_the_four_themes_meets_aa_but_the_known_ones() {
    let mut failing: Vec<Pair> = Vec::new();
    let mut count = 0;
    for (name, theme) in themes() {
        for pair in pairs(name, &theme) {
            assert!(pair.ratio.is_finite() && pair.ratio >= 1.0, "{pair:?}");
            count += 1;
            if pair.ratio < pair.need {
                failing.push(pair);
            }
        }
    }
    assert_eq!(
        count, 421,
        "the pairs of the four themes (a new one is counted here)"
    );
    let listed = |p: &Pair| {
        KNOWN
            .iter()
            .any(|(t, fg, bg, _)| *t == p.theme && *fg == p.fg && *bg == p.bg)
    };
    let new: Vec<String> = failing
        .iter()
        .filter(|p| !listed(p))
        .map(|p| {
            format!(
                "(\"{}\", \"{}\", \"{}\", {:.2}),",
                p.theme, p.fg, p.bg, p.ratio
            )
        })
        .collect();
    assert!(
        new.is_empty(),
        "{} pairs fail WCAG AA and are not known:\n{}",
        new.len(),
        new.join("\n")
    );
    let fixed: Vec<String> = KNOWN
        .iter()
        .filter(|(t, fg, bg, _)| {
            !failing
                .iter()
                .any(|p| p.theme == *t && p.fg == *fg && p.bg == *bg)
        })
        .map(|(t, fg, bg, _)| format!("{t} {fg} on {bg}"))
        .collect();
    assert!(
        fixed.is_empty(),
        "these pairs pass now, take them off KNOWN: {fixed:?}"
    );
}
