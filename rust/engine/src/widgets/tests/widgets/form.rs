//! form widgets: checkbox, switch, slider, select, text field, field.

use super::*;
use crate::drawing::text::TextMeasurer;
use crate::styling::color::Color;
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
    assert_eq!(core.form.field.error_color, Color::hex(0xFE5938));
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
        assert_eq!(input.text, theme.typography.body());
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
        assert_eq!(theme.form, FormTokens::derive(&theme), "{name}");
    }
    assert_eq!(Theme::hoff_core().form, FormTokens::hoff_core());
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
    let line = theme.form.label.text.line_height;
    let (w, h) = long.size(Some(120.0), &theme);
    assert!(w <= 120.0 && h > line, "wrapped: {w}x{h}");
    assert_eq!(long.style(&theme), theme.form.label.text);
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

/// The caption is the HOFF accordion head the gallery's groups were
/// drawn with, in every theme: 12/600 on a line of 1.4, the engine's
/// face, the glass's placeholder tone.
#[test]
fn a_caption_is_the_accordion_head_in_every_theme() {
    use crate::drawing::text::TextStyle;
    use crate::widgets::form::label::Label;
    let head = TextStyle::new(12.0)
        .with_line_height(12.0 * 1.4)
        .with_weight(600);
    for theme in [
        Theme::hoff_core(),
        Theme::hoff(),
        Theme::dark(),
        Theme::light(),
    ] {
        assert_eq!(Label::new("A").caption().style(&theme), head);
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
