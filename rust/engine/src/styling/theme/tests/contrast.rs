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
//! every surface text sits on, the tooltip's own text and its key's on
//! the tooltip (`overlay.tooltip`, what feedback/tooltip.rs draws), the headline
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
use crate::styling::theme::{ButtonRecipe, Intent, Theme};

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
    // The tooltip's text on its fill, and on its key's square over it.
    let tip = &t.overlay.tooltip;
    let tooltip = over(tip.fill, page);
    let key = over(tip.key.fill, tooltip);
    for (bg, under) in [
        ("overlay.tooltip.fill", tooltip),
        ("overlay.tooltip.key.fill", key),
    ] {
        add(
            "overlay.tooltip.text_color",
            bg,
            TEXT,
            over(tip.text_color, under),
            under,
        );
    }
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
    // the color picker on a card: its texts (the bare box's) on its box
    // at rest and hovered, at AAA
    for (bg, surface) in [
        ("form.color.rest.fill", &f.color.rest),
        ("form.color.hover.fill", &f.color.hover),
    ] {
        let under = over(surface.fill, card);
        add(
            "form.bare.text_color",
            bg,
            AAA,
            over(f.bare.text_color, under),
            under,
        );
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
        ("kinds.comment", a.kinds.comment),
        ("kinds.video", a.kinds.video),
        ("kinds.invite", a.kinds.invite),
        ("kinds.like", a.kinds.like),
        ("kinds.image", a.kinds.image),
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
    // the attachment's file tile (its name, detail and icon on its fill)
    // and the remove button's cross on both ends of its gradient.
    let at = &t.message.attachment;
    let tile = over(at.file_fill, card);
    for (fg, need, color) in [
        ("message.attachment.name", TEXT, at.name.color),
        ("message.attachment.detail", TEXT, at.detail.color),
        (
            "message.attachment.file_icon_color",
            LARGE,
            at.file_icon_color,
        ),
    ] {
        add(
            fg,
            "message.attachment.file_fill",
            need,
            over(color, tile),
            tile,
        );
    }
    for (i, fill) in at.remove_fill.iter().enumerate() {
        let under = over(*fill, card);
        add(
            "message.attachment.remove_ink",
            &format!("message.attachment.remove_fill[{i}]"),
            LARGE,
            over(at.remove_ink, under),
            under,
        );
    }
    // the markers: the unread pin's border and the point cursor's on the
    // card, and each cursor's mark on both ends of its fill.
    let m = &t.message.marker;
    add(
        "message.marker.pin_unread",
        "colors.surface",
        LARGE,
        over(m.pin_unread, card),
        card,
    );
    add(
        "message.marker.point_border",
        "colors.surface",
        LARGE,
        over(m.point_border, card),
        card,
    );
    for (name, shape) in [
        ("point", &m.point),
        ("pinned", &m.pinned),
        ("active", &m.active),
    ] {
        for (i, fill) in shape.fill.iter().enumerate() {
            let under = over(*fill, card);
            add(
                &format!("message.marker.{name}.ink"),
                &format!("message.marker.{name}.fill[{i}]"),
                LARGE,
                over(shape.ink, under),
                under,
            );
        }
    }
    // the badge on the card: the chip's label on its fill and on its
    // raised paper, an intent's disc on the chip, the dot on the card, and
    // a count's numeral on its fill.
    let bd = &t.message.badge;
    let well = over(c.bg_panel, card);
    for (name, border) in [
        ("border", bd.tag.border),
        ("raised_border", bd.tag.raised_border),
    ] {
        for (bg, under) in [("colors.surface", card), ("colors.bg_panel", well)] {
            add(
                &format!("message.badge.tag.{name}"),
                bg,
                LARGE,
                over(border, under),
                under,
            );
        }
    }
    let chip = over(bd.tag.fill, card);
    let raised = over(bd.tag.raised_fill, card);
    add(
        "message.badge.tag.text",
        "message.badge.tag.fill",
        TEXT,
        over(bd.tag.text.color, chip),
        chip,
    );
    add(
        "message.badge.tag.text",
        "message.badge.tag.raised_fill",
        TEXT,
        over(bd.tag.text.color, raised),
        raised,
    );
    for (name, intent) in [
        ("neutral", Intent::Neutral),
        ("informational", Intent::Informational),
        ("constructive", Intent::Constructive),
        ("destructive", Intent::Destructive),
    ] {
        if intent != Intent::Neutral {
            let mark = over(bd.marks.of(intent), chip);
            add(
                &format!("message.badge.marks.{name}"),
                "message.badge.tag.fill",
                LARGE,
                mark,
                chip,
            );
        }
        add(
            &format!("message.badge.dots.{name}"),
            "colors.surface",
            LARGE,
            over(bd.dots.of(intent), card),
            card,
        );
        let fill = over(bd.count_fill.of(intent), card);
        add(
            &format!("message.badge.count_ink.{name}"),
            &format!("message.badge.count_fill.{name}"),
            TEXT,
            over(bd.count_ink.of(intent), fill),
            fill,
        );
    }
    // the avatar on a card: the initial on the disc that stands in for a
    // photo, and the online dot on its paper ring.
    let av = &t.message.avatar;
    let disc = over(av.fill, card);
    add(
        "message.avatar.initial",
        "message.avatar.fill",
        TEXT,
        over(av.initial, disc),
        disc,
    );
    let ring = over(av.ring, card);
    add(
        "message.avatar.online_fill",
        "message.avatar.ring",
        LARGE,
        over(av.online_fill, ring),
        ring,
    );
    // wave 3: the prompt input on a card, its text and its hint on its
    // paper, the model's trigger on its hover wash, a model's title and
    // description on the chosen row, the add-files rows and their icons,
    // a premade prompt on its wash, and the voice mark's dot.
    let pr = &t.message.prompt;
    let paper = over(pr.fill, card);
    let wash = over(pr.trigger.hover, paper);
    let chosen = over(pr.choice.selected, over(pr.panel.fill, card));
    let lit = over(pr.files.hover, over(pr.panel.fill, card));
    let row = over(pr.premade.row_hover, over(pr.premade.fill, card));
    for (fg, bg, need, color, under) in [
        (
            "message.prompt.text",
            "message.prompt.fill",
            TEXT,
            pr.text.color,
            paper,
        ),
        (
            "message.prompt.placeholder",
            "message.prompt.fill",
            TEXT,
            pr.placeholder,
            paper,
        ),
        (
            "message.prompt.trigger.text",
            "message.prompt.trigger.hover",
            TEXT,
            pr.trigger.text.color,
            wash,
        ),
        (
            "message.prompt.trigger.chevron_color",
            "message.prompt.fill",
            LARGE,
            pr.trigger.chevron_color,
            paper,
        ),
        (
            "message.prompt.choice.title",
            "message.prompt.choice.selected",
            TEXT,
            pr.choice.title.color,
            chosen,
        ),
        (
            "message.prompt.choice.description",
            "message.prompt.choice.selected",
            TEXT,
            pr.choice.description.color,
            chosen,
        ),
        (
            "message.prompt.files.text",
            "message.prompt.files.hover",
            TEXT,
            pr.files.text.color,
            lit,
        ),
        (
            "message.prompt.files.icon_color",
            "message.prompt.panel.fill",
            LARGE,
            pr.files.icon_color,
            over(pr.panel.fill, card),
        ),
        (
            "message.prompt.files.icon_hover",
            "message.prompt.files.hover",
            LARGE,
            pr.files.icon_hover,
            lit,
        ),
        (
            "message.prompt.premade.row",
            "message.prompt.premade.row_hover",
            TEXT,
            pr.premade.row.color,
            row,
        ),
        (
            "message.prompt.voice.dot_fill[0]",
            "message.prompt.fill",
            LARGE,
            pr.voice.dot_fill[0],
            paper,
        ),
        (
            "message.prompt.voice.dot_fill[1]",
            "message.prompt.fill",
            LARGE,
            pr.voice.dot_fill[1],
            paper,
        ),
    ] {
        add(fg, bg, need, over(color, under), under);
    }
    // the comment popups: the reply on its well, the close disc's cross on
    // both ends of its fill, the new comment's and the quick comment's
    // runs and hints on their paper, their tools at rest and under the
    // pointer.
    let cm = &t.message.comment;
    let well = over(cm.reply.well, over(t.message.scroller.fill, card));
    let new = over(cm.new.fill, card);
    let quick = over(cm.quick.fill, card);
    let new_lit = over(cm.new.tool.hover, new);
    let quick_lit = over(cm.quick.tool.hover, quick);
    let close = cm.close.fill.map(|f| over(f, card));
    for (fg, bg, need, color, under) in [
        (
            "message.comment.reply.text",
            "message.comment.reply.well",
            TEXT,
            cm.reply.text.color,
            well,
        ),
        (
            "message.comment.reply.placeholder",
            "message.comment.reply.well",
            TEXT,
            cm.reply.placeholder,
            well,
        ),
        (
            "message.comment.close.ink",
            "message.comment.close.fill[0]",
            LARGE,
            cm.close.ink,
            close[0],
        ),
        (
            "message.comment.close.ink",
            "message.comment.close.fill[1]",
            LARGE,
            cm.close.ink,
            close[1],
        ),
        (
            "message.comment.new.text",
            "message.comment.new.fill",
            TEXT,
            cm.new.text.color,
            new,
        ),
        (
            "message.comment.new.placeholder",
            "message.comment.new.fill",
            TEXT,
            cm.new.placeholder,
            new,
        ),
        (
            "message.comment.new.tool.color",
            "message.comment.new.fill",
            LARGE,
            cm.new.tool.color,
            new,
        ),
        (
            "message.comment.new.tool.color",
            "message.comment.new.tool.hover",
            LARGE,
            cm.new.tool.color,
            new_lit,
        ),
        (
            "message.comment.quick.title",
            "message.comment.quick.fill",
            TEXT,
            cm.quick.title.color,
            quick,
        ),
        (
            "message.comment.quick.subtitle",
            "message.comment.quick.fill",
            TEXT,
            cm.quick.subtitle.color,
            quick,
        ),
        (
            "message.comment.quick.name",
            "message.comment.quick.fill",
            TEXT,
            cm.quick.name.color,
            quick,
        ),
        (
            "message.comment.quick.text",
            "message.comment.quick.fill",
            TEXT,
            cm.quick.text.color,
            quick,
        ),
        (
            "message.comment.quick.placeholder",
            "message.comment.quick.fill",
            TEXT,
            cm.quick.placeholder,
            quick,
        ),
        (
            "message.comment.quick.tool.color",
            "message.comment.quick.tool.hover",
            LARGE,
            cm.quick.tool.color,
            quick_lit,
        ),
    ] {
        add(fg, bg, need, over(color, under), under);
    }
    // the notifications dropdown: its title and a filter at rest on the
    // card's paper, the chosen filter on its wash.
    let nt = &t.message.notifications;
    let sheet = over(t.message.toast.fill, card);
    let chosen = over(nt.tab_fill, sheet);
    for (fg, bg, color, under) in [
        (
            "message.notifications.title",
            "message.toast.fill",
            nt.title.color,
            sheet,
        ),
        (
            "message.notifications.tab",
            "message.toast.fill",
            nt.tab.color,
            sheet,
        ),
        (
            "message.notifications.tab_active",
            "message.notifications.tab_fill",
            nt.tab_active,
            chosen,
        ),
    ] {
        add(fg, bg, TEXT, over(color, under), under);
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
    // task-104 front E: the segment labels on the well and on the thumb,
    // the radio's label on the page and under the pointer, and the
    // calendar's weekdays on a card, at AAA (a color of the kit or the
    // theme that falls under it gives way, `selection::tokens::legible`),
    // and today's ring, a mark a person must see (1.4.11).
    let aaa = f64::from(crate::widgets::selection::tokens::TEXT_CONTRAST);
    let sel = &t.selection;
    let well = over(sel.segmented.track.fill[0], page_bg.1);
    let thumb = over(sel.segmented.thumb.fill[0], well);
    let row = over(sel.radio.row_hover, page_bg.1);
    for (fg, bg, color, under) in [
        (
            "selection.segmented.text",
            "selection.segmented.track",
            sel.segmented.text,
            well,
        ),
        (
            "selection.segmented.text_hover",
            "selection.segmented.track",
            sel.segmented.text_hover,
            well,
        ),
        (
            "selection.segmented.thumb.text",
            "selection.segmented.thumb",
            sel.segmented.thumb.text,
            thumb,
        ),
        (
            "selection.radio.label",
            "colors.bg",
            sel.radio.label,
            page_bg.1,
        ),
        (
            "selection.radio.label",
            "selection.radio.row_hover",
            sel.radio.label,
            row,
        ),
        (
            "selection.calendar.weekday",
            "colors.surface",
            sel.calendar.weekday,
            card,
        ),
    ] {
        add(fg, bg, aaa, over(color, under), under);
    }
    let chosen = over(sel.calendar.chosen.rest.fill[0], card);
    let day = over(sel.calendar.chosen.rest.text, chosen);
    add(
        "selection.calendar.chosen.text",
        "selection.calendar.chosen",
        aaa,
        day,
        chosen,
    );
    let cb = &sel.checkbox;
    let box_on = over(cb.on.fill[0], page_bg.1);
    add(
        "selection.checkbox.label",
        "colors.bg",
        aaa,
        over(cb.label, page_bg.1),
        page_bg.1,
    );
    add(
        "selection.checkbox.mark",
        "selection.checkbox.on",
        LARGE,
        over(cb.mark, box_on),
        box_on,
    );
    let faded = |c: Color, a: f32| Color::rgba(c.0[0], c.0[1], c.0[2], c.0[3] * a);
    let box_off = over(faded(cb.on.fill[0], cb.disabled_alpha), page_bg.1);
    add(
        "selection.checkbox.mark_disabled",
        "selection.checkbox.on@disabled",
        LARGE,
        over(cb.mark_disabled, box_off),
        box_off,
    );
    let edge = cb.off.shadow.first().map_or(cb.off.fill[0], |s| s.color);
    add(
        "selection.checkbox.off.edge",
        "colors.bg",
        LARGE,
        over(edge, page_bg.1),
        page_bg.1,
    );
    for (name, tag) in [
        ("rest", &sel.chip.rest),
        ("rest_hover", &sel.chip.rest_hover),
        ("on", &sel.chip.on),
        ("on_hover", &sel.chip.on_hover),
    ] {
        let fill = over(tag.fill[0], page_bg.1);
        add(
            &format!("selection.chip.{name}.text"),
            &format!("selection.chip.{name}"),
            aaa,
            over(tag.text, fill),
            fill,
        );
    }
    // the switch's well off (the radio's disc) and the slider's thumb:
    // their rings on all they border (1.4.11)
    let ring_of =
        |s: &crate::styling::theme::ButtonSurface| s.shadow.last().map_or(s.fill[0], |r| r.color);
    let well = ring_of(&sel.switch.off);
    for (bg, under) in [
        ("colors.bg", page_bg.1),
        ("selection.radio.row_hover", row),
        (
            "selection.switch.off",
            over(sel.switch.off.fill[0], page_bg.1),
        ),
    ] {
        add(
            "selection.switch.off.edge",
            bg,
            LARGE,
            over(well, under),
            under,
        );
    }
    let sl = &sel.slider;
    let range = over(sl.track.fill[0], page_bg.1);
    for (bg, under) in [
        ("selection.slider.track", range),
        ("selection.slider.fill", over(sl.fill, range)),
    ] {
        let edge = over(ring_of(&sl.thumb), under);
        add("selection.slider.thumb.edge", bg, LARGE, edge, under);
    }
    // the date picker's calendar on the popover's panel: its weekdays at
    // AAA and today's ring
    let panel = over(t.overlay.popover.panel.fill[0], page_bg.1);
    add(
        "selection.calendar.weekday",
        "overlay.popover.panel",
        aaa,
        over(sel.calendar.weekday, panel),
        panel,
    );
    add(
        "selection.calendar.today",
        "overlay.popover.panel",
        LARGE,
        over(sel.calendar.today, panel),
        panel,
    );
    // the folder picker: its labels and placeholder at AAA, its marks
    // (the search and new-folder icons) at 3:1, on the panel, a row's
    // hover and the search box
    let fo = &sel.folders;
    let fpanel = over(fo.panel.fill[0], page_bg.1);
    let fhover = over(fo.row_hover, fpanel);
    let fsearch = over(fo.search.fill[0], fpanel);
    for (fg, bg, need, color, under) in [
        (
            "selection.folders.label",
            "selection.folders.panel",
            aaa,
            fo.label,
            fpanel,
        ),
        (
            "selection.folders.label",
            "selection.folders.row_hover",
            aaa,
            fo.label,
            fhover,
        ),
        (
            "selection.folders.naming_icon",
            "selection.folders.naming",
            LARGE,
            fo.naming_icon,
            over(fo.naming.fill[0], fpanel),
        ),
        (
            "selection.folders.mark",
            "selection.folders.panel",
            LARGE,
            fo.mark,
            fpanel,
        ),
        (
            "selection.folders.mark",
            "selection.folders.row_hover",
            LARGE,
            fo.mark,
            fhover,
        ),
        (
            "selection.folders.mark",
            "selection.folders.search",
            LARGE,
            fo.mark,
            fsearch,
        ),
    ] {
        add(fg, bg, need, over(color, under), under);
    }
    let today = over(sel.calendar.today, card);
    add(
        "selection.calendar.today",
        "colors.surface",
        LARGE,
        today,
        card,
    );
    // task-104 front F: the headers, on their bar: the logo's ink at AAA,
    // each action's icon at rest and hovered and the dot of news at 3:1
    // (marks), on the bar and on the plain action's hover
    let hd = &t.header;
    let bar = over(hd.fill, page);
    add(
        "header.logo.ink",
        "header.fill",
        AAA,
        over(hd.logo.ink, bar),
        bar,
    );
    for (name, look) in [
        ("header.nav", &hd.nav),
        ("header.plain", &hd.plain),
        ("header.framed", &hd.framed),
    ] {
        let r = &look.recipe;
        let rest = over(r.rest.fill[0], bar);
        let hover = over(r.hover.fill[0], bar);
        add(
            &format!("{name}.rest.text"),
            &format!("{name}.rest.fill"),
            LARGE,
            over(r.rest.text, rest),
            rest,
        );
        add(
            &format!("{name}.hover.text"),
            &format!("{name}.hover.fill"),
            LARGE,
            over(r.hover.text, hover),
            hover,
        );
    }
    let plain_hover = over(hd.plain.recipe.hover.fill[0], bar);
    for (bg, under) in [
        ("header.fill", bar),
        ("header.plain.hover.fill", plain_hover),
    ] {
        add(
            "header.dot.color",
            bg,
            LARGE,
            over(hd.dot.color, under),
            under,
        );
    }
    out
}

/// The pairs that fail today, 110 of the 725 (core 41, hoff 19, dark 13,
/// light 37), by theme, text and background, with the ratio the test
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
        "button.secondary.rest.text",
        "button.secondary.rest.fill",
        1.09,
    ),
    (
        "light",
        "button.secondary.active.text",
        "button.secondary.active.fill",
        1.09,
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
    // task-104 front F: the kit's dot of news (#FE5938) on the hovered
    // action's wash, and hoff's danger as that dot
    ("core", "header.dot.color", "header.plain.hover.fill", 2.77),
    ("hoff", "header.dot.color", "header.fill", 2.27),
    ("hoff", "header.dot.color", "header.plain.hover.fill", 1.45),
    // task-104 front C, wave 3: the kit's prompt placeholder (a hint, as
    // the field's), and the top of its voice dot, the kit's #FF732D (its
    // bottom, #E24D03, reads)
    (
        "core",
        "message.prompt.placeholder",
        "message.prompt.fill",
        4.13,
    ),
    (
        "core",
        "message.prompt.voice.dot_fill[0]",
        "message.prompt.fill",
        2.64,
    ),
    (
        "light",
        "message.prompt.voice.dot_fill[0]",
        "message.prompt.fill",
        2.60,
    ),
    // the comment popups' hints, the kit's #7B7B7B (`placeholder:text-secondary`)
    (
        "core",
        "message.comment.reply.placeholder",
        "message.comment.reply.well",
        3.75,
    ),
    (
        "core",
        "message.comment.new.placeholder",
        "message.comment.new.fill",
        4.13,
    ),
    (
        "core",
        "message.comment.quick.placeholder",
        "message.comment.quick.fill",
        4.13,
    ),
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
        count, 849,
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

/// WCAG 2.2 AAA: text (1.4.6).
const AAA: f64 = 7.0;

/// task-104 front D: the core table's and tree's text meets AAA on every
/// surface under it (the card, a hovered and a selected row). The kit's
/// #7B7B7B, and its header at opacity 50, fail it; they take shade 7.
#[test]
fn the_core_table_and_tree_text_meets_aaa() {
    let t = Theme::hoff_core();
    let (table, tree) = (t.disclosure.table, t.disclosure.tree);
    let page = over(t.colors.surface, [0.0; 3]);
    let rows = &table.rows;
    let unders = [
        page,
        over(rows.hover.fill, page),
        over(rows.selected.fill, page),
        over(tree.hover.fill, page),
        over(tree.selected.fill, page),
    ];
    let texts = [
        table.header_text,
        table.cell,
        table.cell_dim,
        tree.label,
        tree.label_selected,
    ];
    for under in unders {
        for fg in texts {
            let r = ratio(over(fg, under), under);
            assert!(r >= AAA, "{fg:?} at {r:.2}");
        }
    }
}

/// task-104 front D: the core page head (the kit's Layout title) reads at
/// AAA on the page, narrow and wide: the title in the ink, the blurb in
/// shade 7 (the kit's #7B7B7B fails it).
#[test]
fn the_core_page_head_meets_aaa() {
    let t = Theme::hoff_core();
    let page = over(t.colors.bg, [0.0; 3]);
    let p = t.disclosure.page;
    for head in [p.narrow, p.wide] {
        for fg in [head.title, head.blurb] {
            let r = ratio(over(fg, page), page);
            assert!(r >= AAA, "{fg:?} at {r:.2}");
        }
    }
}

/// task-104 front D: the core card's chart and legend marks read at 3:1
/// on the card (WCAG 1.4.11): both bars, both ends of the highlighted
/// one, its cap, the solid dot, the hollow dot's ring and the ticks. The
/// baseline is a rule, the kit's line, and is left out.
#[test]
fn the_core_card_marks_meet_three_to_one() {
    let t = Theme::hoff_core();
    let card = over(t.glass.surface, over(t.colors.bg, [0.0; 3]));
    let m = t.disclosure.card;
    let marks = [
        m.series[0],
        m.series[1],
        m.highlight[0],
        m.highlight[1],
        m.cap,
        m.dot_solid[0],
        m.dot_solid[1],
        m.dot_ring,
        m.tick,
    ];
    for fg in marks {
        let r = ratio(over(fg, card), card);
        assert!(r >= LARGE, "{fg:?} at {r:.2}");
    }
}

/// task-104 front D: the core catalog cards' text reads at AAA: the ink
/// and the muted text on the paper and both washes, the featured plan's
/// title on its image, and the hover's text on a photo where it sits (the
/// top of the explore card's title, 38 px above the foot of a 288 px
/// photo, under the scrim at that height).
#[test]
fn the_core_catalog_text_meets_aaa() {
    let t = Theme::hoff_core();
    let k = t.disclosure.catalog.palette;
    let page = over(t.colors.bg, [0.0; 3]);
    for under in [k.paper, k.panel, k.wash].map(|s| over(s, page)) {
        for fg in [k.ink, k.muted] {
            let r = ratio(over(fg, under), under);
            assert!(r >= AAA, "{fg:?} at {r:.2}");
        }
    }
    for under in k.feature.map(|s| over(s, page)) {
        let r = ratio(over(k.on_image, under), under);
        assert!(r >= AAA, "on the featured plan at {r:.2}");
    }
    let e = &t.disclosure.catalog.explore;
    let photo_h = e.width / e.ratio;
    let from_foot = e.bar_pad + e.button / 2.0 + t.disclosure.catalog.tile.title.line_height / 2.0;
    let at = 1.0 - from_foot / photo_h;
    let [top, foot] = k.image.map(|c| c.0);
    let mix: [f32; 4] = std::array::from_fn(|i| top[i] + (foot[i] - top[i]) * at);
    let scrim_h = t.disclosure.catalog.tile.scrim_h;
    let alpha = e.scrim_alpha * (1.0 - from_foot / scrim_h);
    let mut scrim = k.scrim;
    scrim.0[3] = alpha;
    let under = over(scrim, over(Color(mix), page));
    let r = ratio(over(k.on_image, under), under);
    assert!(r >= AAA, "on a photo at {r:.2}");
}
/// task-104 front C, wave 3: the core prompt input's text meets AAA on
/// every surface under it (its paper, the trigger's and a row's wash, the
/// chosen model). The kit's model description, `#7b7b7b/70`, fails it; it
/// takes shade 7 (`message/tokens/prompt.rs`). The placeholder is a hint,
/// kept as the kit's and known above.
#[test]
fn the_core_prompt_text_meets_aaa() {
    let t = Theme::hoff_core();
    let p = &t.message.prompt;
    let card = over(t.colors.surface, [0.0; 3]);
    let paper = over(p.fill, card);
    let panel = over(p.panel.fill, card);
    let pairs = [
        (p.text.color, paper),
        (p.trigger.text.color, over(p.trigger.hover, paper)),
        (p.choice.title.color, over(p.choice.selected, panel)),
        (p.choice.title.color, over(p.choice.hover, panel)),
        (p.choice.description.color, over(p.choice.selected, panel)),
        (p.choice.description.color, over(p.choice.hover, panel)),
        (p.files.text.color, over(p.files.hover, panel)),
        (p.premade.row.color, over(p.premade.row_hover, paper)),
    ];
    for (fg, under) in pairs {
        let r = ratio(over(fg, under), under);
        assert!(r >= AAA, "{fg:?} at {r:.2}");
    }
}

/// task-104 front C, wave 3: the core comment popups' text meets AAA on
/// what is under it: the reply on its well, the new comment's and the
/// quick comment's runs on their paper. The kit's quick subtitle,
/// `#7B7B7B`, fails it; it takes shade 7 (`message/tokens/comment.rs`).
/// The hints are the kit's and known above.
#[test]
fn the_core_comment_text_meets_aaa() {
    let t = Theme::hoff_core();
    let c = &t.message.comment;
    let card = over(t.colors.surface, [0.0; 3]);
    let well = over(c.reply.well, over(t.message.scroller.fill, card));
    let paper = over(c.quick.fill, card);
    let pairs = [
        (c.reply.text.color, well),
        (c.new.text.color, over(c.new.fill, card)),
        (c.quick.title.color, paper),
        (c.quick.subtitle.color, paper),
        (c.quick.name.color, paper),
        (c.quick.text.color, paper),
    ];
    for (fg, under) in pairs {
        let r = ratio(over(fg, under), under);
        assert!(r >= AAA, "{fg:?} at {r:.2}");
    }
}

/// task-104 front C, wave 3: the core notifications dropdown's header
/// meets AAA: the title and a filter at rest on the card's paper (the
/// kit's `#7B7B7B` filter fails it and takes shade 7), the chosen one on
/// its wash.
#[test]
fn the_core_notifications_header_meets_aaa() {
    let t = Theme::hoff_core();
    let n = &t.message.notifications;
    let sheet = over(t.message.toast.fill, over(t.colors.surface, [0.0; 3]));
    for (fg, under) in [
        (n.title.color, sheet),
        (n.tab.color, sheet),
        (n.tab_active, over(n.tab_fill, sheet)),
    ] {
        let r = ratio(over(fg, under), under);
        assert!(r >= AAA, "{fg:?} at {r:.2}");
    }
}

/// task-104 front G: the core sidebars' text meets AAA on the surfaces
/// under it (the panel, a filled row, a field's well), their marks 3:1.
/// The kit's #7B7B7B text, and its 70% and 50% washes, fail it; that text
/// takes shade 7.
#[test]
fn the_core_sidebars_meet_aaa() {
    let t = Theme::hoff_core();
    let s = &t.navigation.sidebars;
    let k = &s.canvas;
    let panel = over(s.fill, [0.0; 3]);
    let unders = [
        panel,
        over(s.field_fill, panel),
        over(k.row_fill, panel),
        over(k.design.field_fill, panel),
    ];
    let texts = [s.caption_color, s.field_placeholder, k.ink, k.subtle];
    for under in unders {
        for fg in texts {
            let r = ratio(over(fg, under), under);
            assert!(r >= AAA, "{fg:?} at {r:.2}");
        }
        for mark in [k.mark, s.field_mark] {
            let r = ratio(over(mark, under), under);
            assert!(r >= LARGE, "{mark:?} at {r:.2}");
        }
    }
}
