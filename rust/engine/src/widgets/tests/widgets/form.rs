//! form widgets: checkbox, switch, slider, select, text field, field.

use super::*;
use crate::drawing::text::TextMeasurer;
use crate::styling::color::Color;
use crate::styling::theme::TypeRole;
use crate::widgets::form::field::{Field, FieldPart};
use crate::widgets::form::text_input::MASK;
use accord::{Role, SemId, SnapshotBuilder, Value};

// Checkbox

#[test]
fn checkbox_click_toggles_checked() {
    let mut cb = Checkbox::new(false);
    let r = click(|e| cb.handle_event(e, B));
    assert!(r.clicked);
    assert!(cb.checked);
    let r = click(|e| cb.handle_event(e, B));
    assert!(r.clicked);
    assert!(!cb.checked);
}

#[test]
fn checkbox_disabled_ignores_click() {
    let mut cb = Checkbox::new(false).disabled(true);
    let r = click(|e| cb.handle_event(e, B));
    assert!(!r.clicked);
    assert!(!cb.checked);
}

#[test]
fn checkbox_release_outside_does_not_toggle() {
    let mut cb = Checkbox::new(false);
    cb.handle_event(&down(20.0, 20.0), B);
    cb.handle_event(&up(500.0, 500.0), B);
    assert!(!cb.checked);
}

// Switch

#[test]
fn switch_click_toggles_and_animates_knob() {
    let theme = Theme::dark();
    let mut sw = Switch::new(false).with_motion(&theme.motion);
    assert_eq!(sw.knob_progress(), 0.0);

    let r = click(|e| sw.handle_event(e, B));
    assert!(r.clicked);
    assert!(sw.on);
    assert!(sw.is_animating());

    sw.tick(1.0 / 60.0);
    let mid = sw.knob_progress();
    assert!(mid > 0.0 && mid < 1.0, "knob mid-flight, got {mid}");

    for _ in 0..300 {
        sw.tick(1.0 / 60.0);
    }
    assert!(!sw.is_animating());
    assert!((sw.knob_progress() - 1.0).abs() < 1e-3);
}

#[test]
fn switch_disabled_ignores_click() {
    let mut sw = Switch::new(false).disabled(true);
    let r = click(|e| sw.handle_event(e, B));
    assert!(!r.clicked);
    assert!(!sw.on);
    assert!(!sw.is_animating());
}

// Slider

#[test]
fn slider_drag_updates_value() {
    let mut s = Slider::new(0.0, 100.0, 0.0);
    let bounds = Rect::new(0.0, 0.0, 200.0, 20.0);
    s.handle_event(&down(100.0, 10.0), bounds, &hoff());
    assert!(s.is_dragging());
    assert!(s.value() > 25.0 && s.value() < 75.0, "value={}", s.value());

    s.handle_event(&move_to(1000.0, 10.0), bounds, &hoff());
    assert_eq!(s.value(), 100.0, "drag clamps at max");
    s.handle_event(&up(1000.0, 10.0), bounds, &hoff());
    assert!(!s.is_dragging());
}

#[test]
fn slider_step_snaps_value() {
    let mut s = Slider::new(0.0, 10.0, 0.0).step(2.0);
    s.set_value(3.2);
    assert_eq!(s.value(), 4.0);
}

#[test]
fn slider_disabled_ignores_drag() {
    let mut s = Slider::new(0.0, 100.0, 50.0).disabled(true);
    let bounds = Rect::new(0.0, 0.0, 200.0, 20.0);
    s.handle_event(&down(180.0, 10.0), bounds, &hoff());
    assert_eq!(s.value(), 50.0);
}

// Select

#[test]
fn select_opens_and_picks_option() {
    let mut s = Select::new(["dark", "light", "dracula"], 0);
    let bounds = Rect::new(10.0, 10.0, 160.0, 30.0);

    s.handle_event(&down(50.0, 20.0), bounds, &hoff());
    assert!(s.is_open());

    // Option 1 ("light"): dropdown starts at bounds bottom + gap;
    // PAD_Y(8) + OPTION_H(44) + 22 centers the second option.
    let dd = s.dropdown_rect(bounds, &hoff());
    let y = dd.y + 8.0 + 44.0 + 22.0;
    let r = s.handle_event(&down(50.0, y), bounds, &hoff());
    assert!(r.clicked);
    assert!(!s.is_open());
    assert_eq!(s.selected, 1);
    assert_eq!(s.selected_label(), Some("light"));
}

#[test]
fn select_click_outside_closes_without_change() {
    let mut s = Select::new(["a", "b"], 0);
    let bounds = Rect::new(10.0, 10.0, 160.0, 30.0);
    s.handle_event(&down(50.0, 20.0), bounds, &hoff());
    assert!(s.is_open());
    let r = s.handle_event(&down(700.0, 500.0), bounds, &hoff());
    assert!(!s.is_open());
    assert!(!r.clicked);
    assert_eq!(s.selected, 0);
}

#[test]
fn select_disabled_never_opens() {
    let mut s = Select::new(["a"], 0).disabled(true);
    let bounds = Rect::new(10.0, 10.0, 160.0, 30.0);
    s.handle_event(&down(50.0, 20.0), bounds, &hoff());
    assert!(!s.is_open());
}

fn twenty() -> Select {
    Select::new((0..20).map(|i| format!("option {i}")), 0)
}

#[test]
fn select_wheel_belongs_to_the_open_panel_only_where_the_panel_is() {
    let theme = hoff();
    let mut s = twenty();
    let (x, y) = B.center();
    assert_eq!(
        s.handle_event(&wheel(x, y, 40.0), B, &theme),
        EventResult::IGNORED,
        "a closed select does not scroll"
    );
    s.open();
    assert_eq!(
        s.handle_event(&wheel(700.0, 500.0, 40.0), B, &theme),
        EventResult::IGNORED,
        "outside the panel the wheel is the page's"
    );
    let (px, py) = s.dropdown_rect(B, &theme).center();
    assert!(s.handle_event(&wheel(px, py, 40.0), B, &theme).changed);
    // Past the end nothing moves, and the wheel still stops at the panel.
    assert!(s.handle_event(&wheel(px, py, 10_000.0), B, &theme).changed);
    let end = s.handle_event(&wheel(px, py, 40.0), B, &theme);
    assert!(end.handled && !end.changed);
    // A broken delta moves nothing and leaves the rows scrollable.
    let broken = s.handle_event(&wheel(px, py, f32::NAN), B, &theme);
    assert!(broken.handled && !broken.changed);
    assert!(s.handle_event(&wheel(px, py, -40.0), B, &theme).changed);

    // A panel whose options all fit does not scroll (the pointer rests
    // there first, so the highlight under it is already set).
    let mut short = Select::new(["a", "b", "c"], 0);
    short.open();
    let (px, py) = short.dropdown_rect(B, &theme).center();
    short.handle_event(&move_to(px, py), B, &theme);
    let r = short.handle_event(&wheel(px, py, 40.0), B, &theme);
    assert!(r.handled && !r.changed);
}

#[test]
fn select_highlight_follows_the_row_under_a_still_pointer_as_the_wheel_scrolls() {
    let theme = hoff();
    let mut s = twenty();
    s.open();
    let panel = s.dropdown_rect(B, &theme);
    let row_h = theme.control.height(crate::styling::theme::ControlSize::Md);
    let (x, y) = (
        panel.x + panel.w / 2.0,
        panel.y + theme.control.menu_pad + row_h / 2.0,
    );
    assert!(s.handle_event(&move_to(x, y), B, &theme).changed);
    assert_eq!(s.highlighted(), Some(0));
    assert!(s.handle_event(&wheel(x, y, row_h), B, &theme).changed);
    assert_eq!(s.highlighted(), Some(1), "the rows moved under the pointer");
}

#[test]
fn select_with_no_options_opens_an_empty_panel_the_keys_leave_alone() {
    let theme = hoff();
    let mut s = Select::new(Vec::<String>::new(), 0);
    s.open();
    assert_eq!(s.highlight_next(&theme), EventResult::IGNORED);
    assert_eq!(s.highlight_prev(&theme), EventResult::IGNORED);
    let mut c = Compositor::new();
    c.begin_frame();
    s.render_dropdown(&mut c, LayerId::DEFAULT, B, &theme);
    let texts = c
        .layer(LayerId::DEFAULT)
        .unwrap()
        .nodes()
        .iter()
        .filter(|n| matches!(n, SceneNode::Text { .. }))
        .count();
    assert_eq!(texts, 0, "no option row is drawn");
    let (px, py) = s.dropdown_rect(B, &theme).center();
    assert!(!s.handle_event(&down(px, py), B, &theme).clicked);
    assert!(!s.is_open(), "a click in the empty panel only closes it");
}

// Text field: the form recipe and a line wider than its box

/// The core theme draws the kit's Field box (`components/Field/
/// index.tsx:66`); every other theme keeps the glass field it drew.
#[test]
fn the_core_field_is_the_kit_and_the_others_keep_their_glass() {
    let core = Theme::hoff_core();
    let input = &core.form.input;
    assert_eq!(
        (input.height, input.pad_x, input.radius, input.border_width),
        (48.0, 22.0, 12.0, 1.5)
    );
    assert_eq!(input.rest.border, Color::hex(0xE2E2E2));
    assert!(input.rest.shadow.is_empty(), "no shadow at rest");
    assert_eq!(input.hover.shadow, input.focus.shadow);
    let lift = input.hover.shadow[0];
    assert_eq!(
        (lift.offset, lift.blur, lift.spread),
        ([0.0, 1.0], 4.0, -1.0)
    );
    // the kit's #FE5938, deepened to read as text (AA)
    assert_eq!(
        core.form.field.error_color,
        crate::styling::theme::hoff_core::red_text()
    );
    assert_eq!(TextField::height(&core), 48.0);

    for theme in [Theme::hoff(), Theme::dark(), Theme::light()] {
        let input = &theme.form.input;
        let glass = &theme.glass;
        assert_eq!(input.rest.fill, glass.field);
        assert_eq!(input.hover.fill, glass.surface_active);
        assert_eq!(input.focus.border, glass.field_focus_border);
        assert_eq!(input.rest.border, glass.edge_soft);
        assert_eq!(input.pad_x, theme.spacing.md);
        assert_eq!(input.radius, theme.shape.nav);
        assert_eq!(input.text, TypeRole::BodyLg.run());
        assert_eq!(
            TextField::height(&theme),
            theme.control.height(crate::styling::theme::ControlSize::Md)
        );
    }
}

/// Every theme but the core reads its form recipe from itself, once it
/// is built: a palette spread over another theme (`..Self::dark()`) does
/// not keep that theme's field.
#[test]
fn every_theme_but_the_core_derives_its_form_from_itself() {
    use crate::widgets::form::tokens::FormTokens;
    let names = [
        "hoff",
        "hoff-light",
        "plev",
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
    let named = names.map(|n| (n, Theme::named(n).expect("a theme")));
    let built = [("dark", Theme::dark()), ("light", Theme::light())];
    for (name, theme) in named.into_iter().chain(built) {
        assert_eq!(*theme.form, FormTokens::derive(&theme), "{name}");
    }
    assert_eq!(*Theme::hoff_core().form, FormTokens::hoff_core());
}

fn nodes_of(draw: impl FnOnce(&mut Compositor)) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    draw(&mut c);
    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

/// The texts drawn, each with its x and the width it wraps in.
fn texts(nodes: &[SceneNode]) -> Vec<(String, f32, Option<f32>)> {
    nodes
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, x, .. } => {
                Some((key.text.clone(), *x, key.max_width_bits.map(f32::from_bits)))
            }
            _ => None,
        })
        .collect()
}

const LONG: &str = "a value far wider than the field it was typed into, and then some";

/// A value wider than the field never wraps under it: focused, it draws
/// on one line under a clip of the text's box with the caret inside;
/// blurred, cut with an ellipsis inside the box. Before the line fit the
/// text was laid out in the box's width and wrapped.
#[test]
fn a_long_value_stays_on_one_line_inside_the_field() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let style = TextField::text_style(&theme);
        let bounds = Rect::new(10.0, 10.0, 200.0, TextField::height(&theme));
        let pad = theme.form.input.pad_x;
        let (box_x, box_w) = (bounds.x + pad, bounds.w - pad * 2.0);
        for focused in [true, false] {
            let mut f = TextField::new("name").with_text(LONG);
            if focused {
                f.focus();
                f.edit(EditKey::End);
            }
            let nodes = nodes_of(|c| f.render(c, bounds, &theme));
            for (text, x, width) in texts(&nodes) {
                let (w, h) = TextMeasurer::measure_styled(&text, &style, width);
                assert!(h <= style.line_height + 0.5, "{text:?} wraps: {h}");
                if focused {
                    assert_eq!(text, LONG);
                    let caret_w = theme.control.focus_ring_width;
                    let end = box_x + box_w - caret_w;
                    assert!((x + w - end).abs() < 0.5, "slid to the caret at the end");
                } else {
                    assert!(text.ends_with('\u{2026}'), "cut: {text:?}");
                    assert!(x >= box_x && x + w <= box_x + box_w + 0.5);
                }
            }
            let clip = nodes.iter().find_map(|n| match *n {
                SceneNode::PushClip { x, w, .. } => Some((x, w)),
                _ => None,
            });
            assert_eq!(clip, Some((box_x, box_w)), "the text's box clips it");
            if focused {
                let caret = nodes.iter().rev().find_map(|n| match *n {
                    SceneNode::Rect { x, w, color, .. } if color == theme.form.input.caret.0 => {
                        Some((x, w))
                    }
                    _ => None,
                });
                let (x, w) = caret.expect("the caret draws (fresh blink)");
                assert!(x >= box_x - 0.5 && x + w <= box_x + box_w + 0.5, "caret in");
            }
        }
    }
}

/// A placeholder too long for the box is cut, never wrapped.
#[test]
fn a_long_placeholder_is_cut_to_the_box() {
    let theme = Theme::hoff_core();
    let style = TextField::text_style(&theme);
    let f = TextField::new(LONG);
    let bounds = Rect::new(0.0, 0.0, 160.0, TextField::height(&theme));
    let nodes = nodes_of(|c| f.render(c, bounds, &theme));
    let [(text, _, width)] = texts(&nodes).try_into().expect("one text");
    assert!(text.ends_with('\u{2026}'));
    assert!(TextMeasurer::measure_styled(&text, &style, width).1 <= style.line_height + 0.5);
}

/// A masked field draws one bullet per grapheme and keeps the text.
#[test]
fn a_masked_field_draws_bullets_and_keeps_the_text() {
    let theme = Theme::hoff_core();
    let f = TextField::new("Password").with_text("hunter2").masked(true);
    let bounds = Rect::new(0.0, 0.0, 240.0, TextField::height(&theme));
    let nodes = nodes_of(|c| f.render(c, bounds, &theme));
    let [(text, _, _)] = texts(&nodes).try_into().expect("one text");
    assert_eq!(text, MASK.to_string().repeat(7));
    assert_eq!(f.text(), "hunter2");
}

// Field

fn password(id: &str) -> Field {
    Field::new("Enter password")
        .label("Password")
        .link("Forgot password?")
        .password(true)
        .with_text("hunter2")
        .error("Confirm password is incorrect")
        .sem(SemId::parse(id).unwrap())
}

fn inside(r: Rect, x: f32, w: f32) -> bool {
    r.x >= x - 0.5 && r.right() <= x + w + 0.5
}

fn overlap(a: Rect, b: Rect) -> bool {
    a.x < b.right() && b.x < a.right() && a.y < b.bottom() && b.y < a.bottom()
}

/// From 120 to 600 px, with a label, a link and an error far longer than
/// the kit's: every part stays inside the width, nothing overlaps, the
/// label row sits over the box and the message under it, and the height
/// is the layout's. Wide, the link shares the label's row; narrow, it
/// goes under the label.
#[test]
fn a_field_keeps_its_parts_inside_its_width_from_narrow_to_wide() {
    let long = "The email address the workspace sends its invitations and receipts to";
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let field = Field::new("Enter email")
            .label(long)
            .link("Forgot password?")
            .icon("scan")
            .password(true)
            .error(long);
        let gap = theme.form.field.label_gap;
        for w in (120..=600).step_by(8).map(|w| w as f32) {
            let bounds = Rect::new(20.0, 30.0, w, 0.0);
            let l = field.layout(bounds, &theme);
            let (label, link, message) = (l.label.unwrap(), l.link.unwrap(), l.message.unwrap());
            for r in [
                label,
                link,
                l.input,
                message,
                l.icon.unwrap(),
                l.reveal.unwrap(),
            ] {
                assert!(inside(r, bounds.x, w), "{w}: {r:?} leaves the width");
            }
            assert!(!overlap(label, link), "{w}: label and link overlap");
            assert!(label.bottom().max(link.bottom()) + gap <= l.input.y + 0.01);
            assert!(message.y >= l.input.bottom());
            assert!((field.height_for(w, &theme) + bounds.y - l.bottom()).abs() < 0.01);
        }
        let wide = field
            .clone()
            .label("Password")
            .layout(Rect::new(0.0, 0.0, 600.0, 0.0), &theme);
        assert_eq!(wide.label.unwrap().y, wide.link.unwrap().y, "one row");
        let narrow = field.layout(Rect::new(0.0, 0.0, 160.0, 0.0), &theme);
        assert!(
            narrow.link.unwrap().y >= narrow.label.unwrap().bottom(),
            "link under"
        );
    }
}

/// The eye reveals and hides a password; the box keeps its focus. A
/// validated password shows the check and no eye, as in the kit.
#[test]
fn the_eye_reveals_and_hides_the_password() {
    let theme = Theme::hoff_core();
    let mut f = password("form.password");
    let bounds = Rect::new(0.0, 0.0, 240.0, 0.0);
    f.input.focus();
    let eye = f.layout(bounds, &theme).reveal.expect("an eye");
    let (x, y) = eye.center();
    assert_eq!(f.part_at(x, y, bounds, &theme), Some(FieldPart::Reveal));
    assert!(
        f.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme)
            .clicked
    );
    assert!(f.is_revealed() && !f.input.is_masked());
    assert!(
        f.input.is_focused(),
        "the eye leaves the focus where it was"
    );
    assert!(f.toggle_reveal());
    assert!(f.input.is_masked());

    let mut done = password("form.done").validated(true);
    let l = done.layout(bounds, &theme);
    assert!(l.reveal.is_none() && l.check.is_some());
    assert!(!done.toggle_reveal(), "no eye, nothing to reveal");
    let mut off = password("form.off").disabled(true);
    assert!(!off.toggle_reveal(), "disabled");
}

/// A password projects secret (the snapshot keeps its length), named by
/// its label, described and marked by its error; the eye is a toggle and
/// the link a link, beside it.
#[test]
fn a_password_projects_secret_with_its_eye_and_its_link() {
    let f = password("form.password");
    let id = SemId::parse("form.password").unwrap();
    let form = SemId::parse("form").unwrap();
    let mut b = SnapshotBuilder::new();
    b.node(form.clone(), Role::Group, "form");
    f.project(&mut b, &id, Some(&form));
    let snap = b.finish().expect("a valid projection");
    let n = snap.get(&id).unwrap();
    assert_eq!((n.role, n.label.as_str()), (Role::TextInput, "Password"));
    assert!(n.constraints.secret);
    assert_eq!(n.state.get("text"), Some(&Value::Int(7)), "only its length");
    assert_eq!(n.state.get("invalid"), Some(&Value::Bool(true)));
    assert_eq!(
        n.description.as_deref(),
        Some("Confirm password is incorrect")
    );
    assert!(!format!("{snap:?}").contains("hunter2"));
    let eye = snap
        .get(&SemId::parse("form.password.reveal").unwrap())
        .unwrap();
    assert_eq!(eye.role, Role::Toggle);
    assert_eq!(eye.state.get("on"), Some(&Value::Bool(false)));
    let link = snap
        .get(&SemId::parse("form.password.link").unwrap())
        .unwrap();
    assert_eq!(
        (link.role, link.label.as_str()),
        (Role::Link, "Forgot password?")
    );
}

/// Hovering the link lifts it to full opacity; a press on it is a click
/// the owner reads with `part_at`, and it blurs the box.
#[test]
fn the_link_lifts_on_hover_and_a_press_on_it_blurs_the_box() {
    let theme = Theme::hoff_core();
    let mut f = password("form.password");
    let bounds = Rect::new(0.0, 0.0, 240.0, 0.0);
    f.input.focus();
    let (x, y) = f.layout(bounds, &theme).link.unwrap().center();
    assert!(
        f.handle_event(&WidgetEvent::MouseMove { x, y }, bounds, &theme)
            .changed
    );
    let alpha = |f: &Field| {
        let nodes = nodes_of(|c| f.render(c, bounds, &theme));
        nodes.iter().find_map(|n| match n {
            SceneNode::Text { key, color, .. } if key.text == "Forgot password?" => Some(color[3]),
            _ => None,
        })
    };
    assert_eq!(alpha(&f), Some(theme.form.field.link_hover_alpha));
    assert_eq!(f.part_at(x, y, bounds, &theme), Some(FieldPart::Link));
    assert!(
        f.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme)
            .clicked
    );
    assert!(!f.input.is_focused());
}

// Label

/// A label wraps to its width and draws with the style it measures with;
/// a disabled one fades by the control's alpha.
#[test]
fn a_label_wraps_to_its_width_and_fades_when_disabled() {
    use crate::widgets::form::label::Label;
    let theme = Theme::hoff_core();
    let long = Label::new("A label far longer than the room it is given to stand in");
    let line = theme.typography.run(theme.form.label.text).line_height;
    let (w, h) = long.size(Some(120.0), &theme);
    assert!(w <= 120.0 && h > line, "wrapped: {w}x{h}");
    assert_eq!(
        long.style(&theme),
        theme.typography.run(theme.form.label.text)
    );
    let nodes = nodes_of(|c| long.render(c, Rect::new(0.0, 0.0, 120.0, h), &theme));
    let [(_, _, width)] = texts(&nodes).try_into().expect("one text");
    assert_eq!(width, Some(120.0));
    let alpha = |l: &Label| {
        let nodes = nodes_of(|c| l.render(c, Rect::new(0.0, 0.0, 200.0, 20.0), &theme));
        nodes.iter().find_map(|n| match n {
            SceneNode::Text { color, .. } => Some(color[3]),
            _ => None,
        })
    };
    let rest = alpha(&Label::new("Email")).unwrap();
    let off = alpha(&Label::new("Email").disabled(true)).unwrap();
    assert!((off - rest * theme.form.label.disabled_alpha).abs() < 1e-6);
}

/// The caption is the scale's strong body in every theme, in the theme's
/// face and tracking, in the glass's placeholder tone.
#[test]
fn a_caption_is_the_strong_body_of_every_theme() {
    use crate::widgets::form::label::Label;
    for theme in [
        Theme::hoff_core(),
        Theme::hoff(),
        Theme::dark(),
        Theme::light(),
    ] {
        // In the theme's own face: Inter under the core, not the engine's
        // default sans.
        let head = theme.typography.role(TypeRole::BodyStrong);
        assert_eq!(Label::new("A").caption().style(&theme), head);
        assert_eq!(head.font_family.as_deref(), theme.typography.family);
        assert_eq!(theme.form.label.caption_color, theme.glass.text_placeholder);
    }
}

/// A press on a field's label focuses its box, not a disabled one.
#[test]
fn a_press_on_the_label_focuses_the_box() {
    let theme = Theme::hoff_core();
    let bounds = Rect::new(0.0, 0.0, 240.0, 0.0);
    let mut f = Field::new("Enter email").label("Email");
    let (x, y) = f.layout(bounds, &theme).label.unwrap().center();
    assert_eq!(f.part_at(x, y, bounds, &theme), Some(FieldPart::Label));
    assert!(
        f.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme)
            .clicked
    );
    assert!(f.input.is_focused());
    let mut off = Field::new("Enter email").label("Email").disabled(true);
    off.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme);
    assert!(!off.input.is_focused());
}

/// A label that reads right to left hangs from the end of its row, as
/// its value does, and the link takes the start; its message too.
#[test]
fn a_right_to_left_label_hangs_from_the_end_of_its_row() {
    let theme = Theme::hoff_core();
    let bounds = Rect::new(10.0, 0.0, 240.0, 0.0);
    let f = Field::new("הזינו כתובת דוא״ל")
        .label("דוא״ל")
        .link("שכחתם?")
        .helper("נשלח אליכם קוד");
    let l = f.layout(bounds, &theme);
    let (label, link, message) = (l.label.unwrap(), l.link.unwrap(), l.message.unwrap());
    assert!((label.right() - bounds.right()).abs() < 0.01, "{label:?}");
    assert!((link.x - bounds.x).abs() < 0.01, "{link:?}");
    assert!(
        (message.right() - bounds.right()).abs() < 0.01,
        "{message:?}"
    );
    let ltr = Field::new("Enter email").label("Email").link("Forgot?");
    let l = ltr.layout(bounds, &theme);
    assert_eq!(l.label.unwrap().x, bounds.x);
    assert!((l.link.unwrap().right() - bounds.right()).abs() < 0.01);
}

// Input group

fn search_group() -> crate::widgets::form::input_group::InputGroup {
    use crate::widgets::form::input_group::{Addon, InputGroup};
    InputGroup::new("Search...")
        .start(Addon::Icon("search"))
        .end(Addon::Hint("⌘ K".into()))
}

/// The kit's Search: a 40px box, the icon button 4px in and 32 wide, the
/// hint 10px from the end, and the typed text between them, never under
/// them, from narrow to wide and with a long value.
#[test]
fn a_search_keeps_its_text_between_its_addons() {
    let theme = Theme::hoff_core();
    let style = theme.typography.run(theme.form.search.text);
    for w in (120..=600).step_by(8).map(|w| w as f32) {
        let mut g = search_group().with_text(&LONG.repeat(3));
        g.input.focus();
        let bounds = Rect::new(10.0, 10.0, w, 0.0);
        let l = g.layout(bounds, &theme);
        assert_eq!(l.input.h, 40.0);
        let (icon, hint) = (l.start.unwrap(), l.end.unwrap());
        assert_eq!((icon.x - l.input.x, icon.w), (4.0, 32.0));
        assert!((l.input.right() - hint.right() - 10.0).abs() < 0.01);
        let nodes = nodes_of(|c| g.render(c, bounds, &theme));
        let clip = nodes.iter().find_map(|n| match *n {
            SceneNode::PushClip { x, w, .. } => Some((x, w)),
            _ => None,
        });
        let (x, cw) = clip.expect("a long value is clipped to its box");
        assert!(
            x >= icon.right() && x + cw <= hint.x + 0.5,
            "{w}: under an addon"
        );
        for (text, _, width) in texts(&nodes) {
            let (_, h) = TextMeasurer::measure_styled(&text, &style, width);
            assert!(h <= style.line_height + 0.5, "{text:?} wraps");
        }
    }
}

/// A right-to-left text hangs from the end of the box (laid out in the
/// text's room, aligned to its right), and that room keeps the group's
/// `addon_gap` from the `⌘ K`, in every theme.
#[test]
fn a_right_to_left_search_keeps_clear_of_its_hint() {
    let text = "חיפוש במסמכים";
    for (name, theme) in [
        ("core", Theme::hoff_core()),
        ("hoff", Theme::hoff()),
        ("dark", Theme::dark()),
    ] {
        let g = search_group().with_text(text);
        let bounds = Rect::new(10.0, 10.0, 320.0, 0.0);
        let hint = g.layout(bounds, &theme).end.unwrap();
        let gap = theme.form.group.addon_gap;
        assert!(gap > 0.0, "{name}");
        let nodes = nodes_of(|c| g.render(c, bounds, &theme));
        let drawn = texts(&nodes).into_iter().find(|(t, ..)| t == text);
        let (_, x, width) = drawn.unwrap_or_else(|| panic!("{name}: the text is drawn"));
        let right = x + width.expect("a text that fits hangs from its room");
        assert!(right <= hint.x - gap + 0.5, "{name}: {right} past {hint:?}");
    }
}

/// A press on the icon button is a click that leaves the box's focus; a
/// hovered group darkens its icon.
#[test]
fn the_icon_button_presses_and_darkens_with_the_group() {
    let theme = Theme::hoff_core();
    let bounds = Rect::new(0.0, 0.0, 216.0, 0.0);
    let mut g = search_group();
    let icon = g.layout(bounds, &theme).start.unwrap();
    let (x, y) = icon.center();
    assert!(
        g.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme)
            .clicked
    );
    assert!(!g.input.is_focused(), "the button, not the box");
    let tint = |g: &crate::widgets::form::input_group::InputGroup| {
        let nodes = nodes_of(|c| g.render(c, bounds, &theme));
        nodes.iter().find_map(|n| match n {
            SceneNode::Path { data } => Some(data.vertices[0].color),
            _ => None,
        })
    };
    let drawn = |color: Color| match crate::widgets::icons::icon("search", 16.0, color.0) {
        Some(SceneNode::Path { data }) => Some(data.vertices[0].color),
        _ => None,
    };
    assert_eq!(tint(&g), drawn(theme.form.group.icon_color));
    g.handle_event(&WidgetEvent::MouseMove { x: 100.0, y }, bounds, &theme);
    assert_eq!(tint(&g), drawn(theme.form.group.icon_hover));
}

/// The group projects as its text field described by its hint, with its
/// icon button beside it.
#[test]
fn a_search_projects_its_hint_and_its_button() {
    let g = search_group().sem(SemId::parse("s.search").unwrap());
    let parent = SemId::parse("s").unwrap();
    let mut b = SnapshotBuilder::new();
    b.node(parent.clone(), Role::Group, "s");
    g.project(&mut b, g.sem_id().unwrap(), "Search", Some(&parent));
    let snap = b.finish().expect("a valid projection");
    let field = snap.get(g.sem_id().unwrap()).unwrap();
    assert_eq!(field.description.as_deref(), Some("⌘ K"));
    let action = snap.get(&g.action_id().unwrap()).unwrap();
    assert_eq!(
        (action.role, action.label.as_str()),
        (Role::Button, "search")
    );
}

// Input OTP

fn code() -> crate::widgets::form::otp::InputOtp {
    crate::widgets::form::otp::InputOtp::new(&[3, 3]).sem(SemId::parse("s.code").unwrap())
}

/// Typing fills the cells one by one with digits only, and stops at the
/// last; Backspace empties the last; a paste over a full code takes what
/// it has room for (nothing), over a selected cell its first digit; the
/// full-width digits of an input method count as digits.
#[test]
fn a_code_takes_its_digits_a_cell_at_a_time() {
    use crate::runtime::platform::clipboard::{ClipboardProvider, LocalClipboard};
    let mut otp = code();
    otp.input.focus();
    assert!(otp.input.insert("1a2-3"));
    assert_eq!((otp.input.text(), otp.active()), ("123", 3));
    otp.input.insert("456789");
    assert_eq!(otp.input.text(), "123456");
    assert!(otp.is_complete());
    assert_eq!(otp.active(), 5, "the last cell stays active when full");
    otp.input.edit(crate::widgets::form::EditKey::Backspace);
    assert_eq!((otp.input.text(), otp.active()), ("12345", 5));
    otp.input.compose("６", "", None);
    assert_eq!(otp.input.text(), "123456", "a full-width six");
    let mut board = LocalClipboard::new();
    board.set_text("0000");
    assert!(!otp.input.paste(&mut board), "full: no room");
    otp.input.edit(crate::widgets::form::EditKey::SelectLeft);
    assert!(otp.input.paste(&mut board));
    assert_eq!(
        otp.input.text(),
        "123450",
        "over the selected cell, one digit"
    );
}

/// A press on a filled cell selects its digit, and the next one replaces
/// it; a press on an empty cell puts the caret after the last digit; a
/// press off the cells blurs.
#[test]
fn a_press_on_a_filled_cell_selects_it_and_typing_replaces_it() {
    let theme = Theme::hoff_core();
    let bounds = Rect::new(0.0, 0.0, 400.0, 0.0);
    let mut otp = code().with_text("1234");
    let l = otp.layout(bounds, &theme);
    let (x, y) = l.cells[1].center();
    assert!(
        otp.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme)
            .clicked
    );
    assert!(otp.input.is_focused());
    assert_eq!(otp.active(), 1);
    otp.input.insert("9");
    assert_eq!(otp.input.text(), "1934");
    let (x, y) = l.cells[5].center();
    otp.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme);
    assert_eq!(
        otp.active(),
        4,
        "after the last digit, not the cell pressed"
    );
    let r = otp.handle_event(
        &WidgetEvent::MouseDown { x: 390.0, y: 300.0 },
        bounds,
        &theme,
    );
    assert!(r.changed && !otp.input.is_focused());
    let mut off = code().disabled(true);
    off.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme);
    assert!(!off.input.is_focused(), "disabled");
}

/// The value set whole (an agent's, a reader's) is the code's or is
/// refused, the value as it was.
#[test]
fn a_code_refuses_a_value_that_is_not_one() {
    let mut otp = code();
    assert!(otp.input.set_value("042917"));
    assert!(otp.input.set_value("0429"), "a part of a code");
    assert!(!otp.input.set_value("12345a"), "a letter");
    assert!(!otp.input.set_value("1234567"), "past its cells");
    assert_eq!(otp.input.text(), "0429", "as it was");
    assert!(otp.input.set_value(""), "emptied");
}

/// From narrow to wide the cells stay in the width, in order, apart, and
/// square; at their natural width they are the Field's box, 48 tall, the
/// minus in the gap between the groups; the digit never wraps.
#[test]
fn a_code_keeps_its_cells_inside_its_width_from_narrow_to_wide() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let otp = code().with_text("123456").label("Verification code");
        let natural = otp.natural_width(&theme);
        let least = otp.min_width(&theme).ceil() as usize;
        assert!(least < 140, "a group fits a narrow phone's card");
        for w in (least..=600).step_by(4).map(|w| w as f32) {
            let bounds = Rect::new(20.0, 10.0, w, 0.0);
            let l = otp.layout(bounds, &theme);
            assert_eq!(l.cells.len(), 6);
            let stacked = l.separators.is_empty();
            for (i, pair) in l.cells.windows(2).enumerate() {
                if stacked && i == 2 {
                    assert!(pair[0].bottom() < pair[1].y, "{w}: the rows meet");
                } else {
                    assert!(pair[0].right() <= pair[1].x + 0.01, "{w}: cells meet");
                }
            }
            for c in &l.cells {
                assert!(c.right() <= bounds.right() + 0.01, "{w}: past the width");
                assert!((c.w - c.h).abs() < 0.01, "{w}: square");
            }
            if let Some(s) = l.separators.first() {
                assert!(l.cells[2].right() < s.x && s.right() < l.cells[3].x);
            }
            if w >= natural {
                assert_eq!(l.cells[0].h, TextField::height(&theme));
            }
            let line = theme.typography.run(theme.form.input.text).line_height;
            assert!(l.cells[0].h >= line, "{w}: a cell shorter than its digit");
        }
    }
}

/// A code projects as a secret text field: its length only, at most its
/// cells, named by its label, described and `invalid` with an error.
#[test]
fn a_code_projects_secret_with_its_length_only() {
    let otp = code()
        .label("Verification code")
        .error("The code is incorrect")
        .with_text("4242");
    let parent = SemId::parse("s").unwrap();
    let mut b = SnapshotBuilder::new();
    b.node(parent.clone(), Role::Group, "s");
    otp.project(&mut b, otp.sem_id().unwrap(), "Code", Some(&parent));
    let snap = b.finish().expect("a valid projection");
    let node = snap.get(otp.sem_id().unwrap()).unwrap();
    assert_eq!(
        (node.role, node.label.as_str()),
        (Role::TextInput, "Verification code")
    );
    assert!(node.constraints.secret);
    assert_eq!(node.constraints.max_len, Some(6));
    assert_eq!(node.description.as_deref(), Some("The code is incorrect"));
    assert_eq!(
        node.state.get("text"),
        Some(&Value::Int(4)),
        "its length, never the digits"
    );
    assert_eq!(node.state.get("invalid"), Some(&Value::Bool(true)));
    assert!(!format!("{snap:?}").contains("4242"));
}

/// The focused code draws the focus ring on its active cell and the
/// caret there while it is empty; a masked one draws bullets.
#[test]
fn the_active_cell_has_the_ring_and_the_caret() {
    let theme = Theme::hoff_core();
    let bounds = Rect::new(0.0, 0.0, 400.0, 0.0);
    let mut otp = code().with_text("12").masked(true);
    otp.input.focus();
    let nodes = nodes_of(|c| otp.render(c, bounds, &theme));
    let drawn: Vec<String> = texts(&nodes).into_iter().map(|(t, _, _)| t).collect();
    assert_eq!(drawn, [MASK.to_string(), MASK.to_string()]);
    let cell = otp.layout(bounds, &theme).cells[2];
    let caret = nodes
        .iter()
        .any(|n| matches!(*n, SceneNode::Rect { x, w, .. } if x > cell.x && x + w < cell.right()));
    assert!(caret, "the caret in the third cell");
}

// Textarea

fn area() -> crate::widgets::form::textarea::TextArea {
    crate::widgets::form::textarea::TextArea::new("Write a message")
        .sem(SemId::parse("s.area").unwrap())
}

/// The kit's box is three lines tall at rest (`h-20`, 80 in the core);
/// one that grows takes its lines up to its most and no more.
#[test]
fn a_text_area_is_the_kits_box_and_grows_to_its_most_lines() {
    let theme = Theme::hoff_core();
    let w = 300.0;
    let rest = area().layout(Rect::new(0.0, 0.0, w, 0.0), &theme).input.h;
    assert_eq!(rest, 80.0);
    let full = area().with_text("1\n2\n3\n4\n5\n6\n7\n8");
    assert_eq!(
        full.layout(Rect::new(0.0, 0.0, w, 0.0), &theme).input.h,
        80.0
    );
    let lh = theme.typography.run(theme.form.input.text).line_height;
    let grown = |n: usize| {
        let text = vec!["line"; n].join("\n");
        area()
            .grows(5)
            .with_text(&text)
            .layout(Rect::new(0.0, 0.0, w, 0.0), &theme)
            .input
            .h
    };
    assert_eq!(grown(1), rest, "never under the box at rest");
    assert_eq!(grown(4), rest + lh);
    assert_eq!(grown(5), rest + 2.0 * lh);
    assert_eq!(grown(9), rest + 2.0 * lh, "five lines at most");
}

/// Enter breaks the line; Up and Down move a line keeping the column,
/// past the ends to the text's ends; Home and End go to the line's ends;
/// Left at a line's start goes to the end of the one before.
#[test]
fn the_keys_move_over_the_lines() {
    use crate::widgets::form::EditKey;
    let theme = Theme::hoff_core();
    let w = 400.0;
    let mut a = area().with_text("first line\nfirst more");
    a.input.focus();
    nodes_of(|c| a.render(c, Rect::new(0.0, 0.0, w, 0.0), &theme));
    let cursor = |a: &crate::widgets::form::textarea::TextArea| a.input.input.buffer.cursor();
    assert_eq!(cursor(&a), 21);
    assert!(a.edit(EditKey::Home));
    assert_eq!(cursor(&a), 11);
    assert!(a.edit(EditKey::Left));
    assert_eq!(cursor(&a), 10, "the end of the first line");
    a.edit(EditKey::Home);
    for _ in 0..3 {
        a.edit(EditKey::Right);
    }
    assert!(a.vertical(true, false));
    assert_eq!(cursor(&a), 14, "the same column a line down");
    assert!(a.vertical(true, false));
    assert_eq!(cursor(&a), 21, "past the last line, the end");
    a.vertical(false, false);
    a.vertical(false, false);
    assert_eq!(cursor(&a), 0, "past the first, the start");
    a.edit(EditKey::End);
    assert!(a.newline());
    assert_eq!(a.input.text(), "first line\n\nfirst more");
    assert!(a.vertical(true, true), "Shift+Down");
    assert!(a.input.has_selection());
}

/// A paste into a text area keeps its line breaks; into a field it joins
/// them.
#[test]
fn a_paste_keeps_its_lines_in_a_text_area_only() {
    use crate::runtime::platform::clipboard::{ClipboardProvider, LocalClipboard};
    let mut board = LocalClipboard::new();
    board.set_text("one\r\ntwo\tthree\n");
    let mut a = area();
    a.input.focus();
    assert!(a.input.paste(&mut board));
    assert_eq!(a.input.text(), "one\ntwo three\n");
    let mut f = TextField::new("x");
    f.focus();
    f.paste(&mut board);
    assert_eq!(f.text(), "one two three");
}

/// A focused area longer than its box scrolls its lines so the caret
/// stays in the text box, and a long word wraps inside it, from narrow
/// to wide.
#[test]
fn the_caret_stays_in_the_box_from_narrow_to_wide() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        for w in (120..=600).step_by(8).map(|w| w as f32) {
            let text = format!("{LONG}\n{LONG}\n{LONG}");
            let mut a = area()
                .label("Message")
                .helper("Markdown is fine.")
                .with_text(&text);
            a.input.focus();
            let bounds = Rect::new(10.0, 10.0, w, 0.0);
            let l = a.layout(bounds, &theme);
            assert!(l.input.right() <= bounds.right() + 0.01 && l.text.right() <= l.input.right());
            assert!(
                l.message.unwrap().y >= l.input.bottom(),
                "{w}: the message under"
            );
            let nodes = nodes_of(|c| a.render(c, bounds, &theme));
            let ime = a.ime_area(bounds, &theme);
            assert!(
                ime.y >= l.text.y - 0.01 && ime.bottom() <= l.text.bottom() + 0.01,
                "{w}: the caret out of the box"
            );
            assert!(
                ime.x <= l.text.right() + 0.5,
                "{w}: past the box, {ime:?} in {:?}",
                l.text
            );
            let clipped = nodes
                .iter()
                .any(|n| matches!(n, SceneNode::PushClip { .. }));
            assert!(clipped);
        }
    }
}

/// A press puts the caret on the line pressed; a text area projects as
/// a multiline text field named by its label.
#[test]
fn a_press_places_the_caret_and_the_area_projects_multiline() {
    let theme = Theme::hoff_core();
    let bounds = Rect::new(0.0, 0.0, 300.0, 0.0);
    let mut a = area().label("Notes").with_text("ab\ncd\nef");
    let l = a.layout(bounds, &theme);
    let lh = theme.typography.run(theme.form.input.text).line_height;
    let (x, y) = (l.text.x + 200.0, l.text.y + lh * 1.5);
    assert!(
        a.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme)
            .clicked
    );
    assert_eq!(
        a.input.input.buffer.cursor(),
        5,
        "the end of the second line"
    );
    let parent = SemId::parse("s").unwrap();
    let mut b = SnapshotBuilder::new();
    b.node(parent.clone(), Role::Group, "s");
    a.project(&mut b, a.sem_id().unwrap(), Some(&parent));
    let snap = b.finish().expect("a valid projection");
    let node = snap.get(a.sem_id().unwrap()).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::TextInput, "Notes"));
    assert_eq!(node.state.get("multiline"), Some(&Value::Bool(true)));
    assert_eq!(node.state.get("text"), Some(&Value::from("ab\ncd\nef")));
}

// Select recipe and Combobox

/// The core select is the kit's (`h-9`, a flat panel, rows `p-2`, no
/// dot); every other theme keeps the glass select it drew: one `Md`
/// control tall, the glass popover, a dot on the chosen row.
#[test]
fn the_core_select_is_the_kit_and_the_others_keep_their_glass() {
    use crate::styling::theme::ControlSize;
    use crate::widgets::form::tokens::PanelBody;
    let core = Theme::hoff_core();
    let s = &core.form.select;
    assert_eq!(
        (s.height, s.radius, s.panel.row_h, s.panel.pad),
        (36.0, 10.0, 32.0, 4.0)
    );
    assert!(matches!(s.panel.body, PanelBody::Flat(_)));
    assert_eq!(s.panel.marker, None);
    assert_eq!(
        core.form.combo.height, 36.0,
        "the combobox's box is the control"
    );
    for theme in [Theme::hoff(), Theme::dark(), Theme::light()] {
        let s = &theme.form.select;
        assert_eq!(s.height, theme.control.height(ControlSize::Md));
        assert_eq!(s.panel.body, PanelBody::Glass);
        assert_eq!(s.panel.marker, Some(theme.spacing.sm));
        assert_eq!(s.panel.max_h, theme.size.dropdown_max_h);
    }
}

fn frameworks() -> crate::widgets::form::combobox::Combobox {
    crate::widgets::form::combobox::Combobox::new(
        ["Next.js", "SvelteKit", "Nuxt", "Remix", "Astro"],
        "Select framework...",
    )
    .sem(SemId::parse("s.combo").unwrap())
}

/// Focused, the text is selected whole and the list opens on `open` or
/// a key with every option; typing opens it and filters (any case), Down
/// and Up move over the matches, Enter picks and closes; blurred, the
/// box shows the chosen option.
#[test]
fn a_combobox_filters_highlights_and_picks() {
    let mut c = frameworks();
    c.set_focused(true);
    assert!(!c.is_open(), "focus alone does not open it");
    assert!(c.nav(true) && c.is_open(), "Down opens it");
    c.set_focused(false);
    c.open();
    assert!(c.is_open());
    assert_eq!(c.matches().len(), 5);
    assert!(c.input.insert("X"));
    assert_eq!(c.matches(), [0, 2, 3], "Next.js, Nuxt, Remix");
    assert_eq!(c.highlighted(), Some(0));
    assert!(c.nav(true) && c.nav(true) && c.nav(true));
    assert_eq!(c.highlighted(), Some(3), "stops at the last match");
    assert!(c.nav(false));
    assert!(c.enter());
    assert_eq!((c.selected_label(), c.is_open()), (Some("Nuxt"), false));
    assert_eq!(c.input.text(), "Nuxt");
    c.open();
    assert_eq!(c.matches().len(), 5, "the chosen label filters nothing");
    c.input.insert("qq");
    assert!(c.matches().is_empty());
    assert!(c.enter() && c.is_open(), "nothing to pick: it stays open");
    c.set_focused(false);
    assert_eq!(c.input.text(), "Nuxt", "back to the chosen");
    let mut off = frameworks().disabled(true);
    off.set_focused(true);
    assert!(!off.is_open() && !off.pick(1));
}

/// The open list sits under the control, a row per match, and a press on
/// a row picks it; the projection is a select with its query and, open,
/// its matches as options out of the Tab order.
#[test]
fn a_combobox_list_picks_on_a_press_and_projects_its_matches() {
    let theme = Theme::hoff_core();
    let bounds = Rect::new(0.0, 0.0, 176.0, 36.0);
    let mut c = frameworks();
    let (x, y) = bounds.center();
    assert!(
        c.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme)
            .clicked
    );
    c.input.insert("r");
    let dd = c.dropdown_rect(bounds, &theme).expect("open");
    assert!(dd.y >= bounds.bottom());
    let p = &theme.form.select.panel;
    assert_eq!(dd.h, p.pad * 2.0 + p.row_h * 2.0, "Remix, Astro");
    let parent = SemId::parse("s").unwrap();
    let mut b = SnapshotBuilder::new();
    b.node(parent.clone(), Role::Group, "s");
    c.project(&mut b, c.sem_id().unwrap(), "Framework", Some(&parent));
    let snap = b.finish().expect("a valid projection");
    let node = snap.get(c.sem_id().unwrap()).unwrap();
    assert_eq!(node.role, Role::Select);
    assert_eq!(node.state.get("query"), Some(&Value::from("r")));
    assert_eq!(node.state.get("expanded"), Some(&Value::Bool(true)));
    let astro = snap.get(&SemId::parse("s.combo[4]").unwrap()).unwrap();
    assert!(astro.focusable && astro.role == Role::Option);
    assert!(
        snap.get(&SemId::parse("s.combo[0]").unwrap()).is_none(),
        "not a match"
    );
    let (rx, ry) = (dd.x + 10.0, dd.y + p.pad + p.row_h * 1.5);
    let r = c.handle_event(&WidgetEvent::MouseDown { x: rx, y: ry }, bounds, &theme);
    assert!(r.clicked);
    assert_eq!(c.selected_label(), Some("Astro"));
}

/// A palette's rows never draw out of its list: every row the draw binds
/// is inside the list's strip, in every theme, as the list scrolls.
#[test]
fn a_palette_s_rows_stay_in_its_list() {
    use crate::widgets::form::command::Command;
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let cmd = Command::new("Type a command...")
            .group(
                "File",
                [
                    ("New file", Some("file"), Some("⌘ N")),
                    ("Open folder", None, None),
                    ("Save", None, None),
                ],
            )
            .group(
                "Go to",
                [
                    ("Home", None, None),
                    ("Settings", None, None),
                    ("Terminal", None, None),
                ],
            )
            .sem(SemId::parse("s.cmd").unwrap());
        let bounds = Rect::new(0.0, 0.0, 288.0, 0.0);
        let l = cmd.layout(bounds, &theme);
        let mut c = Compositor::new();
        c.begin_frame();
        cmd.render(&mut c, bounds, &theme);
        let p = &theme.form.select.panel;
        let strip = Rect::new(l.list.x, l.list.y + p.pad, l.list.w, l.list.h - p.pad * 2.0);
        let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
        let clip = nodes.iter().find_map(|n| match *n {
            SceneNode::PushClip { x, y, w, h } => Some(Rect::new(x, y, w, h)),
            _ => None,
        });
        assert_eq!(
            clip,
            Some(strip),
            "the rows are clipped to the list's strip"
        );
    }
}

// NewField

fn username() -> crate::widgets::form::new_field::NewField {
    crate::widgets::form::new_field::NewField::new("sophie", "Enter new username")
        .sem(SemId::parse("s.user").unwrap())
}

/// A press edits: the box out, empty and focused, the value kept; Enter
/// (save) keeps what was typed, an empty save keeps the value, cancel
/// leaves it; a press off it saves.
#[test]
fn a_new_field_edits_in_place_and_saves_or_keeps_its_value() {
    let theme = Theme::hoff_core();
    let bounds = Rect::new(0.0, 0.0, 300.0, 0.0);
    let mut n = username();
    let l = n.layout(bounds, &theme);
    assert_eq!(l.whole.h, 20.0, "h-5");
    let (x, y) = l.whole.center();
    assert!(
        n.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme)
            .clicked
    );
    assert!(n.is_editing() && n.input.is_focused() && n.input.is_empty());
    assert!(n.layout(bounds, &theme).text.w >= 120.0, "w-30");
    assert!(n.save());
    assert_eq!(n.value(), "sophie", "nothing typed: the value stays");
    n.edit();
    n.input.insert("ada");
    assert!(n.cancel());
    assert_eq!(n.value(), "sophie");
    n.edit();
    n.input.insert("ada");
    let r = n.handle_event(
        &WidgetEvent::MouseDown { x: 290.0, y: 200.0 },
        bounds,
        &theme,
    );
    assert!(r.changed && !n.is_editing());
    assert_eq!(n.value(), "ada");
    let mut off = username().disabled(true);
    assert!(!off.edit() && !off.set_value("x"));
}

/// It projects as a text field whose text is the value saved, never the
/// draft, with the pencil at rest and the confirm while editing; the
/// confirm is green with something to save.
#[test]
fn a_new_field_projects_its_value_and_its_icon() {
    let theme = Theme::hoff_core();
    let mut n = username();
    n.edit();
    n.input.insert("draft");
    let parent = SemId::parse("s").unwrap();
    let mut b = SnapshotBuilder::new();
    b.node(parent.clone(), Role::Group, "s");
    n.project(&mut b, n.sem_id().unwrap(), "Username", Some(&parent));
    let snap = b.finish().expect("a valid projection");
    let node = snap.get(n.sem_id().unwrap()).unwrap();
    assert_eq!(node.state.get("text"), Some(&Value::from("sophie")));
    assert_eq!(node.state.get("editing"), Some(&Value::Bool(true)));
    assert!(snap.get(&SemId::parse("s.user.save").unwrap()).is_some());
    let bounds = Rect::new(0.0, 0.0, 300.0, 0.0);
    let green = |n: &crate::widgets::form::new_field::NewField| {
        let nodes = nodes_of(|c| n.render(c, bounds, &theme));
        let drawn = crate::widgets::icons::icon("circle-check", 16.0, theme.form.edit.confirm.0);
        let want = match drawn {
            Some(SceneNode::Path { data }) => data.vertices[0].color,
            _ => unreachable!(),
        };
        nodes
            .iter()
            .any(|x| matches!(x, SceneNode::Path { data } if data.vertices[0].color == want))
    };
    assert!(green(&n));
    n.input.set_text("");
    assert!(!green(&n), "nothing to save: gray");
    assert!(n.set_value("grace"));
    assert_eq!((n.value(), n.is_editing()), ("grace", false));
}

// Command dialog

fn command_dialog() -> crate::widgets::form::command_dialog::CommandDialog {
    use crate::widgets::form::command::Command;
    let cmd = Command::new("Type a command...").group(
        "File",
        [
            ("New file", Some("file"), Some("⌘ N")),
            ("Save", Some("save"), Some("⌘ S")),
        ],
    );
    crate::widgets::form::command_dialog::CommandDialog::new(cmd)
        .sem(SemId::parse("s.palette").unwrap())
}

/// A press on the trigger opens the palette with its caret, ⌘K toggles
/// it, Escape and a press off the sheet close it, and a command run
/// closes it with the command for the owner.
#[test]
fn a_command_dialog_opens_runs_and_closes() {
    let theme = Theme::hoff_core();
    let window = Rect::new(0.0, 0.0, 1024.0, 768.0);
    let trigger = Rect::new(20.0, 20.0, 216.0, 40.0);
    let mut d = command_dialog();
    let (tx, ty) = trigger.center();
    let r = d.handle_event(
        &WidgetEvent::MouseDown { x: tx, y: ty },
        window,
        trigger,
        &theme,
    );
    assert!(r.clicked && d.is_open());
    assert!(d.command.search.input.is_focused(), "the caret in the box");
    assert!(d.escape() && !d.is_open());
    assert!(!d.escape(), "closed: Escape is the page's");
    d.toggle();
    assert!(d.is_open());
    let off = WidgetEvent::MouseDown { x: 2.0, y: 700.0 };
    assert!(d.handle_event(&off, window, trigger, &theme).changed);
    assert!(!d.is_open(), "a press off the sheet closes");
    d.toggle();
    assert!(d.command.nav(true), "Down from the first command");
    assert!(d.enter(), "Enter runs the highlighted command");
    assert!(!d.is_open());
    assert_eq!(d.command.ran().map(|c| c.label.as_str()), Some("Save"));
}

/// The sheet is a modal's width, inside the window from a phone to a
/// desktop, the palette inside it by the modal's padding; open, the
/// scrim covers the window under the sheet.
#[test]
fn a_command_dialog_sits_on_the_modal_sheet_in_every_window() {
    for theme in [Theme::hoff_core(), Theme::hoff(), Theme::dark()] {
        let d = command_dialog().opened();
        for w in [320.0, 768.0, 1440.0] {
            let window = Rect::new(0.0, 0.0, w, 800.0);
            let sheet = d.sheet_rect(&theme, window);
            let modal = theme.overlay.modal.max_w;
            assert!(sheet.w <= modal + 0.01 && sheet.x >= 0.0 && sheet.right() <= w + 0.01);
            let palette = d.palette_rect(&theme, sheet);
            let pad = theme.overlay.modal.pad;
            assert!((palette.x - sheet.x - pad).abs() < 0.01);
            assert!(palette.bottom() <= sheet.bottom() + 0.01);
            let nodes = nodes_of(|c| d.render(c, window, Rect::new(0.0, 0.0, 0.0, 0.0), &theme));
            let scrim = theme.overlay.modal.scrim.0;
            assert!(matches!(
                nodes.first(),
                Some(SceneNode::Rect { w: sw, color, .. }) if *sw == w && *color == scrim
            ));
        }
    }
}

/// Closed, a reader finds the trigger, a button with its `⌘ K`; open, a
/// dialog with the palette in it.
#[test]
fn a_command_dialog_projects_its_trigger_or_its_palette() {
    let parent = SemId::parse("s").unwrap();
    let project = |d: &crate::widgets::form::command_dialog::CommandDialog| {
        let mut b = SnapshotBuilder::new();
        b.node(parent.clone(), Role::Group, "s");
        d.project(&mut b, Some(&parent));
        b.finish().expect("a valid projection")
    };
    let mut d = command_dialog();
    let snap = project(&d);
    let trigger = snap
        .get(&SemId::parse("s.palette.trigger").unwrap())
        .unwrap();
    assert_eq!(trigger.role, Role::Button);
    assert!(snap.get(&SemId::parse("s.palette").unwrap()).is_none());
    d.open();
    let snap = project(&d);
    let dialog = snap.get(&SemId::parse("s.palette").unwrap()).unwrap();
    assert_eq!(dialog.role, Role::Dialog);
    assert!(
        snap.get(&SemId::parse("s.palette.palette").unwrap())
            .is_some()
    );
}

/// On the dialog's sheet the rows stand on the sheet: the palette draws
/// the same box, captions, rows and keycaps as inline, and less (no
/// panel of its own under them).
#[test]
fn a_command_dialog_draws_no_panel_over_its_sheet() {
    for theme in [Theme::hoff_core(), Theme::hoff(), Theme::dark()] {
        let d = command_dialog();
        let inline = crate::widgets::form::command::Command::new("Type a command...").group(
            "File",
            [
                ("New file", Some("file"), Some("⌘ N")),
                ("Save", Some("save"), Some("⌘ S")),
            ],
        );
        let at = Rect::new(0.0, 0.0, 400.0, 0.0);
        let bare = nodes_of(|c| d.command.render(c, at, &theme));
        let full = nodes_of(|c| inline.render(c, at, &theme));
        assert_eq!(texts(&bare), texts(&full));
        assert!(bare.len() < full.len(), "the panel goes");
    }
}

// Color picker

fn picker() -> crate::widgets::form::color_picker::ColorPicker {
    crate::widgets::form::color_picker::ColorPicker::new("F4F4F4", 100)
        .sem(SemId::parse("s.color").unwrap())
}

/// A hex of six digits is the color as it is typed; a box left with
/// something else shows the color again; the opacity takes a number up
/// to 100.
#[test]
fn a_color_picker_takes_what_is_a_color() {
    use crate::widgets::form::color_picker::ColorPart;
    let mut p = picker();
    assert_eq!(p.value(), "#F4F4F4FF");
    assert!(p.focus(ColorPart::Hex));
    p.hex.select_all();
    assert!(p.hex.insert("#00ff7a-zz"));
    assert_eq!(p.hex.text(), "00ff7a", "only hex digits, six at most");
    assert_eq!(p.rgb(), [0x00, 0xFF, 0x7A], "a color as soon as typed");
    p.hex.edit(EditKey::Backspace);
    assert_eq!(p.rgb(), [0xF4; 3], "five digits: the color kept");
    p.hex.insert("A");
    assert!(p.tab(true));
    assert_eq!(p.hex.text(), "00FF7A", "left, it shows the color");
    p.alpha.select_all();
    p.alpha.insert("5a0");
    assert_eq!(p.opacity(), 50);
    assert_eq!(p.value(), "#00FF7A80");
    p.alpha.select_all();
    p.alpha.insert("250");
    assert_eq!(p.opacity(), 100, "over 100 is no opacity: the one kept");
    assert!(p.blur());
    assert_eq!(p.alpha.text(), "100");
    assert!(p.set_hex("#ABC") && p.rgb() == [0xAA, 0xBB, 0xCC]);
    assert!(!p.set_hex("12345") && !p.set_opacity("101"));
    assert!(p.set_opacity("0%") && p.opacity() == 0);
}

/// Tab goes from the hex to the opacity and Shift+Tab back, out at
/// either end; Escape puts back the color the caret came in to; Enter
/// keeps what was typed.
#[test]
fn a_color_picker_walks_its_texts_and_escape_puts_the_color_back() {
    use crate::widgets::form::color_picker::ColorPart;
    let mut p = picker();
    assert!(!p.tab(true), "no caret: the page's");
    p.focus(ColorPart::Hex);
    assert!(!p.tab(false), "Shift+Tab out of the hex");
    assert!(p.tab(true));
    assert_eq!(p.focused(), Some(ColorPart::Alpha));
    assert!(!p.tab(true), "Tab out of the opacity");
    assert!(p.tab(false));
    assert_eq!(p.focused(), Some(ColorPart::Hex));
    p.hex.select_all();
    p.hex.insert("112233");
    assert!(p.escape());
    assert_eq!((p.focused(), p.hex.text()), (None, "F4F4F4"));
    assert!(!p.escape());
    p.focus(ColorPart::Alpha);
    p.alpha.select_all();
    p.alpha.insert("040");
    assert!(p.enter());
    assert_eq!(
        (p.focused(), p.alpha.text()),
        (Some(ColorPart::Alpha), "40")
    );
    assert!(p.escape());
    assert_eq!(p.opacity(), 100, "Escape goes back past Enter");
}

/// A press on the swatch or the hex puts the caret in the hex, on the
/// opacity's part in the opacity, off the box lets it go; hover lights
/// the box.
#[test]
fn a_color_picker_takes_a_press_in_the_part_pressed() {
    use crate::widgets::form::color_picker::ColorPart;
    let theme = Theme::hoff_core();
    let mut p = picker();
    let at = Rect::new(10.0, 10.0, theme.form.color.width, 0.0);
    let l = p.layout(at, &theme);
    for (r, part) in [
        (l.swatch, ColorPart::Hex),
        (l.hex, ColorPart::Hex),
        (l.suffix, ColorPart::Alpha),
        (l.alpha, ColorPart::Alpha),
    ] {
        let (x, y) = r.center();
        assert!(p.handle_event(&down(x, y), at, &theme).clicked);
        assert_eq!(p.focused(), Some(part), "{r:?}");
    }
    assert!(p.handle_event(&down(500.0, 500.0), at, &theme).changed);
    assert_eq!(p.focused(), None);
    let (x, y) = l.whole.center();
    let before = nodes_of(|c| p.render(c, at, &theme));
    p.handle_event(&WidgetEvent::MouseMove { x, y }, at, &theme);
    assert_ne!(nodes_of(|c| p.render(c, at, &theme)), before, "hovered");
    let mut off = picker().disabled(true);
    assert!(!off.handle_event(&down(x, y), at, &theme).clicked);
    assert!(!off.focus(ColorPart::Hex) && !off.set_hex("000000"));
}

/// The parts sit in the box in order, swatch, hex, divider, opacity and
/// its `%`, from a narrow box to a wide one in every theme; the kit's
/// box is 38 tall with a 28 swatch, its opacity part at least 64 wide.
#[test]
fn a_color_picker_lays_its_parts_in_the_box() {
    for theme in [
        Theme::hoff_core(),
        Theme::hoff(),
        Theme::dark(),
        Theme::light(),
    ] {
        let p = picker();
        for w in [96.0, 160.0, theme.form.color.width, 480.0] {
            let l = p.layout(Rect::new(0.0, 0.0, w, 0.0), &theme);
            let inside = |r: Rect| {
                r.x >= l.whole.x - 0.01
                    && r.right() <= l.whole.right() + 0.01
                    && r.y >= l.whole.y - 0.01
                    && r.bottom() <= l.whole.bottom() + 0.01
            };
            for r in [l.swatch, l.hex, l.divider, l.alpha, l.suffix] {
                assert!(inside(r), "{w}: {r:?} in {:?}", l.whole);
            }
            assert!(l.swatch.right() <= l.hex.x + 0.01);
            assert!(l.hex.right() <= l.divider.x + 0.01);
            assert!(l.divider.right() <= l.alpha.x + 0.01);
            assert!(l.alpha.right() <= l.suffix.x + 0.01);
        }
    }
    let core = Theme::hoff_core();
    let l = picker().layout(Rect::new(0.0, 0.0, 218.0, 0.0), &core);
    assert_eq!((l.whole.h, l.swatch.w), (38.0, 28.0));
    assert!(l.alpha_part.w >= 64.0, "the kit's w-16: {}", l.alpha_part.w);
}

/// From 200 to 400 px across (208, the Background group of the kit's
/// Design sidebar), in every theme, the opacity shows `100 %` whole, and
/// its number keeps room for the caret after it.
#[test]
fn a_color_picker_shows_its_whole_opacity_from_200_to_400() {
    for theme in [
        Theme::hoff_core(),
        Theme::hoff(),
        Theme::dark(),
        Theme::light(),
    ] {
        let style = &theme.typography.run(theme.form.bare.text);
        let measure = |t: &str| TextMeasurer::measure_styled(t, style, None).0;
        let number = measure("100") + theme.control.focus_ring_width;
        let p = picker();
        for w in 200u16..=400 {
            let at = Rect::new(0.0, 0.0, f32::from(w), 0.0);
            let drawn: Vec<String> = texts(&nodes_of(|c| p.render(c, at, &theme)))
                .into_iter()
                .map(|(t, ..)| t)
                .collect();
            assert!(
                drawn.iter().any(|t| t == "100") && drawn.iter().any(|t| t == "%"),
                "{w}: {drawn:?}"
            );
            let l = p.layout(at, &theme);
            assert!(l.alpha.w >= number, "{w}: {} < {number}", l.alpha.w);
            assert!(l.suffix.w >= measure("%"), "{w}: {:?}", l.suffix);
        }
    }
}

/// The swatch paints the color at its opacity over its base.
#[test]
fn a_color_picker_paints_its_swatch() {
    let theme = Theme::hoff_core();
    let mut p = picker();
    assert!(p.set_hex("336699") && p.set_opacity("50"));
    let at = Rect::new(0.0, 0.0, 218.0, 0.0);
    let s = p.layout(at, &theme).swatch;
    let want = p.color().0;
    assert!((want[3] - 0.5).abs() < 1e-6);
    let painted = nodes_of(|c| p.render(c, at, &theme)).iter().any(|n| {
        matches!(n, SceneNode::RoundedRect { x, w, color, .. }
            if *x == s.x && *w == s.w && *color == want)
    });
    assert!(painted);
}

/// A reader finds a group with the color as its value and the two texts
/// in it.
#[test]
fn a_color_picker_projects_its_value_and_texts() {
    let parent = SemId::parse("s").unwrap();
    let p = picker();
    let mut b = SnapshotBuilder::new();
    b.node(parent.clone(), Role::Group, "s");
    p.project(&mut b, p.sem_id().unwrap(), "Fill", Some(&parent));
    let snap = b.finish().expect("a valid projection");
    let group = snap.get(&SemId::parse("s.color").unwrap()).unwrap();
    assert_eq!(group.role, Role::Group);
    let value = snap.value(&accord::StateRef::parse("s.color:value").unwrap());
    assert_eq!(value, Some(&Value::Text("#F4F4F4FF".into())));
    for (id, name) in [("s.color.hex", "Hex"), ("s.color.alpha", "Opacity")] {
        let node = snap.get(&SemId::parse(id).unwrap()).unwrap();
        assert_eq!((node.role, node.label.as_str()), (Role::TextInput, name));
    }
}

// Upload avatar

fn upload(photo: bool) -> crate::widgets::form::upload_avatar::UploadAvatar {
    let a = crate::widgets::form::upload_avatar::UploadAvatar::new();
    let a = if photo { a.with_photo(None) } else { a };
    a.sem(SemId::parse("s.avatar").unwrap())
}

/// A press on the disc asks for a file, once for the owner to take; a
/// press on a corner of its square is off the disc; the owner's answer
/// puts a photo in.
#[test]
fn an_upload_avatar_asks_on_a_press_and_takes_the_choice() {
    let theme = Theme::hoff_core();
    let at = Rect::new(10.0, 10.0, 200.0, 0.0);
    let mut a = upload(false);
    let d = a.disc(at, &theme);
    assert_eq!((d.w, d.h), (48.0, 48.0));
    assert!(
        !a.handle_event(&down(d.x + 1.0, d.y + 1.0), at, &theme)
            .clicked,
        "a corner"
    );
    let (x, y) = d.center();
    assert!(a.handle_event(&down(x, y), at, &theme).clicked);
    assert!(a.is_asking());
    assert!(a.take_ask() && !a.take_ask(), "once");
    assert!(!a.has_photo());
    a.chosen(None);
    assert!(a.has_photo());
    let mut off = upload(true).disabled(true);
    assert!(!off.activate());
    assert!(!off.handle_event(&down(x, y), at, &theme).clicked);
}

/// Hovered or focused, the scrim and the plus show over the photo; the
/// focus adds its ring; at rest neither.
#[test]
fn an_upload_avatar_shows_its_plus_hovered_or_focused() {
    let theme = Theme::hoff_core();
    let at = Rect::new(0.0, 0.0, 48.0, 48.0);
    let scrim = theme.form.upload.scrim.0;
    let shows = |a: &crate::widgets::form::upload_avatar::UploadAvatar| {
        let nodes = nodes_of(|c| a.render(c, at, &theme));
        let scrim = nodes
            .iter()
            .any(|n| matches!(n, SceneNode::RoundedRect { color, .. } if *color == scrim));
        let plus = nodes.iter().any(|n| matches!(n, SceneNode::Path { .. }));
        (scrim, plus)
    };
    let mut a = upload(true);
    assert_eq!(shows(&a), (false, false));
    a.handle_event(&WidgetEvent::MouseMove { x: 24.0, y: 24.0 }, at, &theme);
    assert_eq!(shows(&a), (true, true));
    a.handle_event(&WidgetEvent::MouseMove { x: 400.0, y: 24.0 }, at, &theme);
    a.set_focused(true);
    assert_eq!(shows(&a), (true, true));
}

/// A photo stands as the theme's gradient, none as the stand-in, in
/// every theme; the disc keeps its side down to a narrow room.
#[test]
fn an_upload_avatar_paints_its_photo_or_the_stand_in() {
    for theme in [
        Theme::hoff_core(),
        Theme::hoff(),
        Theme::dark(),
        Theme::light(),
    ] {
        let t = &theme.form.upload;
        for (photo, want) in [(true, t.photo), (false, t.empty)] {
            let a = upload(photo);
            let at = Rect::new(0.0, 0.0, 300.0, 0.0);
            let nodes = nodes_of(|c| a.render(c, at, &theme));
            assert!(nodes.iter().any(|n| matches!(n,
                SceneNode::GradientRect { color, color2, .. }
                    if *color == want[0].0 && *color2 == want[1].0)));
        }
        let a = upload(true);
        assert_eq!(a.disc(Rect::new(0.0, 0.0, 20.0, 0.0), &theme).w, 20.0);
    }
}

/// A reader finds a button named for what it does, its photo state and
/// its `activate`.
#[test]
fn an_upload_avatar_projects_a_button() {
    let parent = SemId::parse("s").unwrap();
    for (photo, name) in [(true, "Replace avatar"), (false, "Upload avatar")] {
        let a = upload(photo);
        let mut b = SnapshotBuilder::new();
        b.node(parent.clone(), Role::Group, "s");
        a.project(&mut b, a.sem_id().unwrap(), Some(&parent));
        let snap = b.finish().expect("a valid projection");
        let node = snap.get(&SemId::parse("s.avatar").unwrap()).unwrap();
        assert_eq!((node.role, node.label.as_str()), (Role::Button, name));
        let at = accord::StateRef::parse("s.avatar:photo").unwrap();
        assert_eq!(snap.value(&at), Some(&Value::Bool(photo)));
    }
}

/// The dialog's sheet follows the window's height: as tall as its
/// commands need up to the theme's share of the window, the list taking
/// what the box leaves; Down down the list keeps the highlighted command
/// in view (the least scroll), and Up back to the first shows its
/// group's caption again.
#[test]
fn a_command_dialog_list_follows_the_window_and_the_arrows() {
    use crate::widgets::form::command::Command;
    use crate::widgets::form::command_dialog::CommandDialog;
    let theme = Theme::hoff_core();
    let names: Vec<&'static str> = (0..60)
        .map(|i| &*Box::leak(format!("Command {i}").into_boxed_str()))
        .collect();
    let cmd =
        Command::new("Type a command...").group("All", names.iter().map(|n| (*n, None, None)));
    let mut d = CommandDialog::new(cmd);
    d.open();
    let share = theme.form.group.palette_share;
    let mut last = 0.0;
    for h in [360.0, 600.0, 900.0, 1400.0] {
        let window = Rect::new(0.0, 0.0, 1024.0, h);
        let sheet = d.sheet_rect(&theme, window);
        assert!(
            sheet.h <= h * share + 0.01,
            "{h}: {} past the share",
            sheet.h
        );
        assert!(sheet.h > last, "{h}: the sheet grows with the window");
        assert!(sheet.y >= 0.0 && sheet.bottom() <= h);
        let palette = d.palette_rect(&theme, sheet);
        let l = d.command.layout(palette, &theme);
        assert!(
            (l.list.bottom() - palette.bottom()).abs() < 0.01,
            "the list fills it"
        );
        last = sheet.h;
    }
    // a few commands: the sheet is as tall as they need
    let few = CommandDialog::new(Command::new("x").group("G", [("One", None, None)])).opened();
    let window = Rect::new(0.0, 0.0, 1024.0, 1400.0);
    let need = few.command.full_height(
        few.sheet_rect(&theme, window).w - theme.overlay.modal.pad * 2.0,
        &theme,
    );
    assert!((few.sheet_rect(&theme, window).h - need - theme.overlay.modal.pad * 2.0).abs() < 0.01);

    let window = Rect::new(0.0, 0.0, 1024.0, 600.0);
    let palette = d.palette_rect(&theme, d.sheet_rect(&theme, window));
    let list = d.command.layout(palette, &theme).list;
    let shown = |d: &CommandDialog, i: usize| {
        let x = list.x + list.w / 2.0;
        let mut y = list.y;
        let mut rows = 0;
        while y < list.bottom() {
            if d.command.item_at(x, y, palette, &theme) == Some(i) {
                rows += 1;
            }
            y += 1.0;
        }
        rows as f32 >= theme.form.select.panel.row_h - 1.0
    };
    for k in 1..names.len() {
        assert!(d.command.nav(true));
        d.command.settle_scroll(palette, &theme);
        assert!(shown(&d, k), "Down to {k}: in view, whole");
    }
    for k in (0..names.len() - 1).rev() {
        assert!(d.command.nav(false));
        d.command.settle_scroll(palette, &theme);
        assert!(shown(&d, k), "Up to {k}: in view, whole");
    }
    // the first command's caption shows with it: the list is at its top
    let first_row = list.y + theme.form.select.panel.pad + theme.form.select.panel.row_h * 1.5;
    assert_eq!(
        d.command.item_at(list.x + 4.0, first_row, palette, &theme),
        Some(0)
    );
}
