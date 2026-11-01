//! Settings' five sections (`Settings/General`, `Profile`, `Security`,
//! `Notifications`, `Subscription`): their rows and controls, and one
//! walk down a section that places the controls and, given a compositor,
//! draws what is around them.

use accord::SemId;

use crate::drawing::compositor::Compositor;
use crate::styling::theme::Theme;
use crate::widgets::buttons::{Button, ButtonVariant};
use crate::widgets::form::new_field::NewField;
use crate::widgets::form::{Select, Switch};
use crate::widgets::widget::Rect;

use super::{
    Controls, Item, ItemRole, Kind, boxed, child, glyph, hairline, middle, picture, raise, stroke,
    text, text_end, text_w, tokens,
};

/// What ends a row.
#[derive(Clone, Copy)]
enum End {
    /// The control at this index.
    Control(usize),
    /// A plain value.
    Value(&'static str),
}

/// A row: its title, its description, what ends it.
struct Row {
    title: &'static str,
    description: Option<&'static str>,
    end: End,
}

const fn row(title: &'static str, end: End) -> Row {
    Row {
        title,
        description: None,
        end,
    }
}

pub(super) const TITLES: [&str; 5] = [
    "General",
    "Profile",
    "Security",
    "Notifications",
    "Subscription",
];

const FEATURES: [&str; 5] = [
    "Everything in Free",
    "Unlimited 3D scene generation",
    "Premium asset library access",
    "Animations up to 30 seconds",
    "Multiple AI models",
];

/// Section `s`'s rows.
fn rows(s: usize) -> Vec<Row> {
    use End::{Control as C, Value as V};
    match s {
        0 => vec![
            row("Username", C(0)),
            row("Email", V("sophie@ui.live")),
            row("Enable auto-prompt idea suggestion", C(1)),
            row("Auto-play video on Explore", C(2)),
            row("Publish to Explore", C(3)),
            row("Language", C(4)),
        ],
        1 => vec![
            row("Private profile", C(0)),
            Row {
                description: Some("Use 800x800 px (PNG/JPG)"),
                ..row("Avatar", C(1))
            },
            row("Portfolio link", C(2)),
            row("Display name", V("Sophie Bennett ®")),
        ],
        2 => vec![
            row("Password", C(0)),
            row("Multi-factor authentication", C(1)),
            Row {
                description: Some("Sign out from all devices."),
                ..row("Log out of all devices", C(2))
            },
        ],
        3 => vec![
            row("Display while working", C(0)),
            row("Video generated", C(1)),
            row("3D Object generated", C(2)),
            row("Someone mentioned in the comment", C(3)),
        ],
        _ => Vec::new(),
    }
}

fn switch(on: bool, label: &str) -> Kind {
    Kind::Switch(Switch::new(on), label.into())
}

/// Section `s`'s controls under `<id>.<section>`.
pub(super) fn controls(s: usize, id: Option<&SemId>) -> Controls {
    let names = [
        "general",
        "profile",
        "security",
        "notifications",
        "subscription",
    ];
    let base = child(id, names[s]);
    let at = |name: &str| child(base.as_ref(), name);
    let mut c = Controls::default();
    match s {
        0 => {
            c.push(
                Kind::Field(
                    NewField::new("sophie", "Enter new username"),
                    "Username".into(),
                ),
                at("username"),
            );
            c.push(
                switch(true, "Enable auto-prompt idea suggestion"),
                at("auto_prompt"),
            );
            c.push(switch(true, "Auto-play video on Explore"), at("auto_play"));
            c.push(switch(false, "Publish to Explore"), at("publish"));
            let language = Select::new(["Auto detect", "English", "Spanish"], 0);
            c.push(Kind::Select(language, "Language".into()), at("language"));
        }
        1 => {
            c.push(switch(false, "Private profile"), at("private"));
            c.push(
                Kind::Item(Item::new("Upload an avatar", ItemRole::Button)),
                at("avatar"),
            );
            let link = NewField::new("research-app.app/@username", "Enter new link");
            c.push(Kind::Field(link, "Portfolio link".into()), at("link"));
        }
        2 => {
            // The kit shows the password as its mask: no secret is held.
            let password = NewField::new("••••••••••••••", "Enter new password");
            c.push(Kind::Field(password, "Password".into()), at("password"));
            c.push(switch(false, "Multi-factor authentication"), at("mfa"));
            let out = Button::new("Log out all").variant(ButtonVariant::Primary);
            c.push(Kind::Button(out), at("log_out"));
        }
        3 => {
            let display = Select::new(["Push", "Email", "None"], 0);
            c.push(
                Kind::Select(display, "Display while working".into()),
                at("display"),
            );
            c.push(switch(true, "Video generated"), at("video"));
            c.push(switch(false, "3D Object generated"), at("object"));
            c.push(
                switch(true, "Someone mentioned in the comment"),
                at("mention"),
            );
        }
        _ => {
            c.push(
                Kind::Button(Button::new("Cancel").variant(ButtonVariant::Primary)),
                at("cancel"),
            );
            c.push(
                Kind::Button(Button::new("Upgrade").variant(ButtonVariant::Secondary)),
                at("upgrade"),
            );
        }
    }
    c
}

/// A control's size in a room `w` wide.
fn size(controls: &Controls, i: usize, w: f32, theme: &Theme) -> (f32, f32) {
    let o = &tokens(theme).settings.option;
    match controls.0.get(i).map(|c| &c.kind) {
        Some(Kind::Switch(..)) => {
            let [sw, sh] = theme.control.switch_size;
            (sw, sh)
        }
        Some(Kind::Field(f, _)) => f.size(w, theme),
        Some(Kind::Select(s, _)) => (s.width_in(theme).max(o.select_w).min(w), s.height_in(theme)),
        Some(Kind::Button(b)) => b.preferred_size(theme),
        Some(Kind::Item(_)) => (o.upload, o.upload),
        _ => (0.0, 0.0),
    }
}

/// Down section `s` in `area`: the controls' rects into `out`, what is
/// around them drawn when `c` is given. Returns the section's height.
pub(super) fn walk(
    s: usize,
    controls: &Controls,
    area: Rect,
    theme: &Theme,
    out: &mut Vec<Rect>,
    mut c: Option<&mut Compositor>,
) -> f32 {
    let scale = &theme.typography;
    let t = tokens(theme);
    let (st, ink) = (&t.settings, &t.ink);
    let o = &st.option;
    out.clear();
    out.resize(controls.0.len(), Rect::new(0.0, 0.0, 0.0, 0.0));
    let mut y = area.y;
    let title_h = scale.run(st.title).line_height + st.title_pad_y * 2.0;
    if let Some(c) = c.as_deref_mut() {
        let w = area.w - st.title_pad_x * 2.0;
        text(
            c,
            TITLES[s],
            &scale.run(st.title),
            area.x + st.title_pad_x,
            y + st.title_pad_y,
            w,
            ink.ink,
        );
    }
    y += title_h;
    let (x, w) = (area.x + o.pad_x, (area.w - o.pad_x * 2.0).max(0.0));
    // What ends a row never takes more than half of it: the title keeps
    // the rest.
    for r in rows(s) {
        let (end_w, end_h) = match r.end {
            End::Control(i) => size(controls, i, w / 2.0, theme),
            End::Value(v) => (
                text_w(v, &scale.run(o.value)).min(w / 2.0),
                scale.run(o.value).line_height,
            ),
        };
        let text_room = (w - end_w - o.title_gap).max(0.0);
        let mut block = scale.run(o.title).line_height;
        if r.description.is_some() {
            block += o.description_gap + scale.run(o.description).line_height;
        }
        let h = o.min_h.max(o.pad_y * 2.0 + block.max(end_h));
        let end = Rect::new(x + w - end_w, y + (h - end_h) / 2.0, end_w, end_h);
        if let End::Control(i) = r.end {
            out[i] = end;
        }
        if let Some(c) = c.as_deref_mut() {
            hairline(c, area.x, y, area.w, ink.rule_w, ink.rule);
            let ty = y + (h - block) / 2.0;
            text(c, r.title, &scale.run(o.title), x, ty, text_room, ink.ink);
            if let Some(d) = r.description {
                let dy = ty + scale.run(o.title).line_height + o.description_gap;
                text(c, d, &scale.run(o.description), x, dy, text_room, ink.muted);
            }
            match r.end {
                End::Value(v) => {
                    // The email reads faded in the kit (#121212/30); the
                    // secondary text holds AAA here.
                    let color = if s == 0 { ink.muted } else { ink.ink };
                    text_end(
                        c,
                        v,
                        &scale.run(o.value),
                        end.right(),
                        middle(&scale.run(o.value), end.y, end.h),
                        end_w,
                        color,
                    );
                }
                End::Control(i) => {
                    if let Some(item) = controls.item(i) {
                        picture(c, end, end.w / 2.0, t.pictures.avatars[0]);
                        if item.press.hovered || controls.0[i].is_focused() {
                            boxed(c, end, end.w / 2.0, o.upload_scrim, None);
                            glyph(c, "plus", o.upload_icon, o.upload_ink, end);
                        }
                    }
                }
            }
        }
        y += h;
    }
    if s == 4 {
        y = plan(controls, area, y, theme, out, c);
    }
    y - area.y
}

/// The subscription: the renewal, the plan's price and its two buttons,
/// its features. Returns the bottom.
fn plan(
    controls: &Controls,
    area: Rect,
    top: f32,
    theme: &Theme,
    out: &mut [Rect],
    mut c: Option<&mut Compositor>,
) -> f32 {
    let scale = &theme.typography;
    let t = tokens(theme);
    let (p, ink) = (&t.settings.plan, &t.ink);
    if let Some(c) = c.as_deref_mut() {
        hairline(c, area.x, top, area.w, ink.rule_w, ink.rule);
    }
    let (x, w) = (area.x + p.pad, (area.w - p.pad * 2.0).max(0.0));
    let mut y = top + p.pad;
    // The renewal.
    let renew_h = p
        .renew_icon
        .max(scale.run(t.settings.option.title).line_height)
        + p.renew_pad_y * 2.0;
    if let Some(c) = c.as_deref_mut() {
        let style = &scale.run(t.settings.option.title);
        let rx = x + p.renew_pad_x;
        let room = w - p.renew_pad_x * 2.0 - p.renew_icon;
        text(
            c,
            "Your plan auto-renews on Jul 3, 2025",
            style,
            rx,
            middle(style, y, renew_h),
            room,
            ink.ink,
        );
        let icon = Rect::new(
            x + w - p.renew_pad_x - p.renew_icon,
            y,
            p.renew_icon,
            renew_h,
        );
        glyph(c, "user-check", p.renew_icon, ink.icon, icon);
    }
    y += renew_h;
    // The price and the buttons, in the plan's lit head.
    let button_h = size(controls, 0, w, theme).1;
    let head_h = p.head_pad * 2.0 + scale.run(p.price).line_height + p.buttons_top + button_h;
    let head = Rect::new(x, y, w, head_h);
    let bw = ((w - p.head_pad * 2.0 - p.buttons_gap) / 2.0).max(0.0);
    let by = head.bottom() - p.head_pad - button_h;
    out[0] = Rect::new(x + p.head_pad, by, bw, button_h);
    out[1] = Rect::new(out[0].right() + p.buttons_gap, by, bw, button_h);
    let features_h = p.features_pad * 2.0
        + FEATURES.len() as f32 * p.check.max(scale.run(p.feature).line_height)
        + (FEATURES.len() as f32 - 1.0) * p.features_gap;
    let frame = Rect::new(x, y, w, head_h + features_h);
    if let Some(c) = c {
        stroke(c, frame, p.radius, ink.edge, ink.rule_w);
        raise(c, head, p.radius, &p.head_shadow);
        boxed(c, head, p.radius, ink.well, Some((ink.edge, ink.rule_w)));
        let px = head.x + p.head_pad;
        let py = head.y + p.head_pad;
        let cur_x = px + p.currency_pad[0];
        text(
            c,
            "$",
            &scale.run(p.currency),
            cur_x,
            py + p.currency_pad[1],
            w,
            ink.muted,
        );
        let price_x = cur_x + text_w("$", &scale.run(p.currency)) + p.currency_pad[0];
        text(c, "20", &scale.run(p.price), price_x, py, w, ink.ink);
        let period_x = price_x + text_w("20", &scale.run(p.price)) + p.period_pad;
        let room = head.right() - p.head_pad - period_x;
        text(
            c,
            "USD /",
            &scale.run(p.period),
            period_x,
            py + p.period_pad,
            room,
            ink.muted,
        );
        let second = py + p.period_pad + scale.run(p.period).line_height;
        text(
            c,
            "month",
            &scale.run(p.period),
            period_x,
            second,
            room,
            ink.muted,
        );
        let mut fy = head.bottom() + p.features_pad;
        let row_h = p.check.max(scale.run(p.feature).line_height);
        for f in FEATURES {
            let check = Rect::new(x + p.features_pad + p.check_start, fy, p.check, row_h);
            glyph(c, "check", p.check, ink.ink, check);
            let tx = check.right() + p.check_gap;
            let room = frame.right() - p.features_pad - tx;
            text(
                c,
                f,
                &scale.run(p.feature),
                tx,
                middle(&scale.run(p.feature), fy, row_h),
                room,
                ink.ink,
            );
            fy += row_h + p.features_gap;
        }
    }
    frame.bottom() + p.pad
}
