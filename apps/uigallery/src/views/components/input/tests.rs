//! The Input page's tests.

use super::*;
use engine::drawing::compositor::SceneNode;
use engine::drawing::text::TextMeasurer;

fn content() -> Rect {
    Rect::new(288.0, 80.0, 760.0, 700.0)
}

fn texts(page: &InputPage) -> Vec<String> {
    let theme = Theme::hoff();
    let mut c = Compositor::new();
    c.begin_frame();
    page.render(
        &mut c,
        engine::drawing::compositor::LayerId::DEFAULT,
        content(),
        &theme,
    );
    c.layer(engine::drawing::compositor::LayerId::DEFAULT)
        .unwrap()
        .nodes()
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.text.clone()),
            _ => None,
        })
        .collect()
}

/// A field takes the focus the view's Tab asks for (the projection's
/// order, `dispatch/keyboard.rs`), and Tab itself is no edit; Escape
/// blurs.
#[test]
fn a_field_takes_the_focus_tab_asks_for_and_escape_blurs() {
    let mut p = InputPage::new();
    assert_eq!(p.focused(), None);
    assert!(!p.handle_edit_key(EditKey::Tab), "Tab is the view's");
    let field = p.fields[2].sem_id().cloned().expect("an id");
    assert!(p.focus(&field));
    assert_eq!(p.focused(), Some(2));
    assert!(p.handle_escape());
    assert_eq!(p.focused(), None);
    assert!(!p.handle_escape());
}

/// A click inside a field focuses it and places the caret on the
/// clicked glyph (real shaping via TextMeasurer, not chars * factor).
#[test]
fn click_focuses_field_and_places_caret() {
    let theme = Theme::hoff();
    let mut p = InputPage::new();
    let field = p.rects(content(), &theme)[1];

    let (cx, cy) = field.center();
    let r = p.handle_event(&WidgetEvent::MouseDown { x: cx, y: cy }, content(), &theme);
    assert!(r.changed);
    assert_eq!(p.focused(), Some(1));

    assert!(p.handle_text("Brenner"));
    assert_eq!(p.value(1), "Brenner");
    let style = TextField::text_style(&theme);
    let x = field.x + theme.spacing.md + TextMeasurer::cursor_x_styled("Brenner", &style, None, 4);
    p.handle_event(&WidgetEvent::MouseDown { x, y: cy }, content(), &theme);
    assert_eq!(p.cursor(1), 4);

    // A click outside every field blurs but is not swallowed.
    let r = p.handle_event(
        &WidgetEvent::MouseDown {
            x: content().x + 1.0,
            y: content().y + content().h - 1.0,
        },
        content(),
        &theme,
    );
    assert_eq!(p.focused(), None);
    assert!(r.changed);
}

/// Characters only reach a field while one is focused, and editing
/// keys edit the focused buffer.
#[test]
fn typing_requires_focus_and_editing_keys_edit() {
    let mut p = InputPage::new();
    assert!(!p.handle_text("x"), "no focus: the shell keeps the key");
    p.set_focused(Some(0));
    p.handle_text("abc");
    p.handle_edit_key(EditKey::Left);
    p.handle_edit_key(EditKey::Backspace);
    assert_eq!(p.value(0), "ac");
    p.handle_edit_key(EditKey::End);
    p.handle_edit_key(EditKey::Delete);
    assert_eq!(p.value(0), "ac");
    p.handle_escape();
    assert!(
        !p.handle_edit_key(EditKey::Backspace),
        "blurred: not consumed"
    );
}

/// Each field draws under its label, the placeholder an example
/// while empty, the typed value once typed; nothing else speaks for
/// the value.
#[test]
fn a_field_draws_under_its_label_and_shows_what_is_typed() {
    let theme = Theme::hoff_core();
    let mut p = InputPage::new();
    let before = texts(&p);
    for (label, placeholder) in LABELS.iter().zip(PLACEHOLDERS) {
        assert!(before.iter().any(|t| t == label), "{label}");
        assert!(before.iter().any(|t| t == placeholder), "{placeholder}");
    }
    for (label, field) in p.parts(content(), &theme) {
        assert!(label.bottom() < field.y, "{label:?} over {field:?}");
        assert_eq!(label.x, field.x, "the label starts where its box does");
    }
    p.set_focused(Some(0));
    p.handle_text("Ada");
    let after = texts(&p);
    assert!(after.iter().any(|t| t == "Ada"), "typed value rendered");
    assert!(!after.iter().any(|t| t.contains("live value")));
}

/// The accent focus ring sits 4px (offset + stroke) outside the
/// focused field, and nowhere when nothing is focused.
#[test]
fn focus_ring_tracks_the_focused_field() {
    let theme = Theme::hoff();
    let mut p = InputPage::new();
    let rings = |p: &InputPage| -> Vec<(f32, f32)> {
        let accent = theme.colors.accent.0;
        let mut c = Compositor::new();
        c.begin_frame();
        p.render(
            &mut c,
            engine::drawing::compositor::LayerId::DEFAULT,
            content(),
            &theme,
        );
        c.layer(engine::drawing::compositor::LayerId::DEFAULT)
            .unwrap()
            .nodes()
            .iter()
            .filter_map(|n| match *n {
                SceneNode::RoundedRect {
                    x,
                    y,
                    color,
                    border_width,
                    border_color,
                    ..
                } if color[3] == 0.0 && border_width == 2.0 && border_color == accent => {
                    Some((x, y))
                }
                _ => None,
            })
            .collect()
    };
    assert!(rings(&p).is_empty(), "no focus, no ring");
    p.set_focused(Some(0));
    let f = p.rects(content(), &theme)[0];
    assert_eq!(rings(&p), vec![(f.x - 4.0, f.y - 4.0)]);
}

/// A value wider than its field stays on one line inside it, focused
/// (clipped to the text's box, slid to the caret) or blurred (cut
/// with an ellipsis). It used to be laid out in the box's width and
/// wrap, a second line drawn under the field.
#[test]
fn a_long_value_stays_on_one_line_inside_its_field() {
    let long = "a value far wider than the field it was typed into, ".repeat(4);
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let style = TextField::text_style(&theme);
        let field = InputPage::new().rects(content(), &theme)[0];
        for focused in [true, false] {
            let mut p = InputPage::new();
            p.set_focused(Some(0));
            assert!(p.handle_text(&long));
            if !focused {
                p.set_focused(None);
            }
            let mut c = Compositor::new();
            c.begin_frame();
            p.render(
                &mut c,
                engine::drawing::compositor::LayerId::DEFAULT,
                content(),
                &theme,
            );
            let nodes = c
                .layer(engine::drawing::compositor::LayerId::DEFAULT)
                .unwrap()
                .nodes()
                .to_vec();
            let mut clip: Option<(f32, f32)> = None;
            let mut seen = 0;
            for n in &nodes {
                match n {
                    SceneNode::PushClip { x, w, .. } => clip = Some((*x, *w)),
                    SceneNode::PopClip => clip = None,
                    SceneNode::Text { key, x, .. } if key.text.starts_with("a value") => {
                        seen += 1;
                        let width = key.max_width_bits.map(f32::from_bits);
                        let (w, h) = TextMeasurer::measure_styled(&key.text, &style, width);
                        assert!(h <= style.line_height + 0.5, "wrapped: {h}");
                        let (left, right) = clip.map_or((*x, *x + w), |(cx, cw)| (cx, cx + cw));
                        assert!(left >= field.x && right <= field.x + field.w, "outside");
                    }
                    _ => {}
                }
            }
            assert_eq!(seen, 1, "the value draws once");
        }
    }
}

/// While a field is focused the page asks for the caret's next flip,
/// not for a frame every tick; idle it asks for nothing.
#[test]
fn focused_field_asks_for_the_next_flip() {
    let mut p = InputPage::new();
    assert_eq!(p.wake(0.016), Wake::idle(), "idle page requests no frames");
    p.set_focused(Some(0));
    let w = p.wake(0.016);
    assert!(!w.now && !w.is_continuous(), "no flip yet, no frame: {w:?}");
    assert!(w.next.is_some(), "the next flip is pending");
}

// The kit's groups

use accord::Role;
use engine::widgets::form::color_picker::{ColorPart, ColorPicker};

/// The picker's card, `input_page.group[1].card[0]`.
const PICKER: (usize, usize) = (1, 0);

fn picker(p: &InputPage) -> &ColorPicker {
    p.kit
        .spec(PICKER)
        .and_then(|s| s.get::<ColorPicker>())
        .expect("the color picker")
}

fn snapshot(p: &InputPage) -> accord::Snapshot {
    let root = p.root().expect("the root parses").clone();
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Input");
    p.project(&mut b, &root);
    b.finish().expect("the page projects")
}

fn picker_id(p: &InputPage, part: ColorPart) -> SemId {
    picker(p).text_id(part).expect("an id").clone()
}

/// The Input group comes first (Tab reaches its fields first) and the
/// kit's groups under its rule, in the kit's order; every group's title
/// is drawn, and the page is as tall as both.
#[test]
fn the_kit_s_groups_come_under_the_input_group() {
    let theme = Theme::hoff_core();
    let p = InputPage::new();
    let titles: Vec<&str> = p.kit.groups.iter().map(|g| g.title).collect();
    assert_eq!(titles, ["Replace avatar", "Color picker"]);
    let drawn = texts(&p);
    for title in titles.iter().chain([&TITLE]) {
        assert!(drawn.iter().any(|t| t == title), "{title}");
    }
    let area = p.kit_area(content(), &theme);
    for field in p.rects(content(), &theme) {
        assert!(field.bottom() < area.y, "{field:?} over the kit's groups");
    }
    let card = p.kit.widget_rect(PICKER, area, &theme).unwrap();
    assert!(card.y > area.y);
    let bottom = content().y + p.content_height(content(), &theme);
    assert!(card.bottom() < bottom);
}

/// The picker sits inside its card at every width from a phone to a
/// desktop, the kit's `w-54.5` where there is room.
#[test]
fn the_color_picker_fits_its_card_at_every_width() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let p = InputPage::new();
        let mut w = 240.0;
        while w <= 1600.0 {
            let content = Rect::new(0.0, 0.0, w, 2000.0);
            let laid = p.kit.layout(content, &theme);
            let (card, at) = laid[PICKER.0].cards[PICKER.1];
            assert!(at.x >= card.x && at.right() <= card.right() + 0.01, "{w}");
            assert!(at.bottom() <= card.bottom(), "{w}");
            assert!(at.w <= theme.form.color.width + 0.01);
            w += 37.0;
        }
    }
}

/// The page's keys reach the picker: a reader's focus on the hex, the
/// characters typed into it, the opacity's focus, and Escape putting the
/// color back; the Input group's fields lose the caret to it.
#[test]
fn the_page_s_keys_reach_the_color_picker() {
    let mut p = InputPage::new();
    p.set_focused(Some(0));
    assert!(p.focus(&picker_id(&p, ColorPart::Hex)));
    assert_eq!(p.focused(), None, "the Input group lets the caret go");
    for _ in 0..6 {
        assert!(p.handle_edit_key(EditKey::Backspace));
    }
    assert!(p.handle_text("336699"));
    assert_eq!(picker(&p).rgb(), [0x33, 0x66, 0x99]);
    assert!(p.focus(&picker_id(&p, ColorPart::Alpha)));
    assert_eq!(picker(&p).focused(), Some(ColorPart::Alpha));
    assert!(p.handle_edit_key(EditKey::Backspace));
    assert_eq!(picker(&p).opacity(), 10, "100 less a digit");
    assert!(p.handle_escape());
    assert_eq!(picker(&p).focused(), None);
    assert_eq!(picker(&p).opacity(), 100, "Escape puts it back");
    assert!(!p.handle_escape());
}

/// A reader finds the picker in its group, its value the color; an
/// agent's `set_value` on the hex sets it, a value that is no color is
/// refused.
#[test]
fn the_color_picker_projects_and_takes_a_request() {
    let mut p = InputPage::new();
    let snap = snapshot(&p);
    let group = snap
        .get(&SemId::parse("input_page.group[1]").unwrap())
        .unwrap();
    assert_eq!(group.label, "Color picker");
    let at = accord::StateRef::parse("input_page.group[1].card[0]:value").unwrap();
    assert_eq!(snap.value(&at), Some(&Value::from("#F4F4F4FF")));
    let hex = picker_id(&p, ColorPart::Hex);
    let ask = |p: &InputPage, v: &str| {
        let actor = accord::Actor::Test {
            name: "input".into(),
        };
        Request::observe(&snapshot(p), actor, &hex, "set_value", Some(Value::from(v)))
            .expect("a request")
    };
    let good = ask(&p, "00FF7A");
    assert!(p.run(&good).is_ok());
    assert_eq!(picker(&p).value(), "#00FF7AFF");
    let bad = ask(&p, "nope");
    assert!(p.run(&bad).is_err());
}
