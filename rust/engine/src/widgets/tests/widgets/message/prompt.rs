//! The prompt input of the kit (task-104 front C, wave 3): its tokens,
//! the model menu, the line a person composes in, and the box over every
//! width it is given, in every state its page shows.

use super::*;
use crate::widgets::message::compose::ComposeLine;
use crate::widgets::message::prompt::{Part, PromptInput};
use crate::widgets::message::select_ai::SelectAi;
use crate::widgets::message::tokens::PromptTokens;
use accord::Value;

fn themes() -> [Theme; 4] {
    [core(), Theme::hoff(), Theme::dark(), Theme::light()]
}

fn inside(inner: Rect, outer: Rect) -> bool {
    let e = 0.5;
    inner.x >= outer.x - e
        && inner.y >= outer.y - e
        && inner.right() <= outer.right() + e
        && inner.bottom() <= outer.bottom() + e
}

/// The core is the kit verbatim; every theme keeps its geometry and takes
/// its text and paper from its own palette.
#[test]
fn the_core_is_the_kit_and_the_others_derive_it() {
    let kit = PromptTokens::hoff_core();
    assert_eq!(core().message.prompt, kit);
    assert_eq!(
        (kit.max_w, kit.pad, kit.radius, kit.control),
        (542.0, 12.0, 24.0, 40.0)
    );
    assert_eq!((kit.panel.w, kit.panel.radius), (204.0, 20.0));
    for theme in themes() {
        let t = &theme.message.prompt;
        assert_eq!(t.max_w, kit.max_w);
        assert_eq!(t.text.line_height, kit.text.line_height);
        if theme.message.prompt != kit {
            assert_eq!(t.text.color, theme.colors.text);
            assert_eq!(t.choice.title.color, theme.colors.text);
            assert!(
                t.shadow.iter().all(|s| !s.inset),
                "no key light off the kit"
            );
        }
    }
}

/// The model menu: the panel's width, as tall as its rows; a click on a
/// row chooses it; on its own it is a menu of items, opened by a select
/// its rows are that select's options.
#[test]
fn the_model_menu_chooses_by_a_click_and_projects_both_ways() {
    let theme = core();
    let id = SemId::parse("models").unwrap();
    let mut m = SelectAi::kit().sem(id.clone());
    let (w, h) = m.size(&theme);
    let c = &theme.message.prompt.choice;
    let row = c.pad_y * 2.0 + c.title.line_height + c.description_gap + c.description.line_height;
    assert_eq!(w, 204.0);
    assert_eq!(h, 2.0 * (1.0 + 8.0) + 4.0 * row);
    let b = Rect::new(0.0, 0.0, w, h);
    let rows = m.rows(b, &theme);
    let (x, y) = rows[2].center();
    m.handle_event(&down(x, y), b, &theme);
    assert!(m.handle_event(&up(x, y), b, &theme).clicked);
    assert_eq!(m.chosen_title(), Some("Kling AI"));

    let mut sb = SnapshotBuilder::new();
    m.project(&mut sb, &id, "AI models", None);
    let seen = sb.finish().expect("the menu validates");
    assert_eq!(seen.get(&id).unwrap().role, Role::Menu);
    let item = seen.get(&id.key("2")).unwrap();
    assert_eq!(item.role, Role::MenuItem);
    assert_eq!(item.state.get("selected"), Some(&Value::Bool(true)));

    let select = SemId::parse("model").unwrap();
    let mut sb = SnapshotBuilder::new();
    sb.node(&select, Role::Select, "AI model")
        .state("selected", "Kling AI");
    m.project_options(&mut sb, &select, true);
    let seen = sb.finish().expect("the options validate");
    assert_eq!(seen.get(&select.key("0")).unwrap().role, Role::Option);
}

/// The line types while focused and refuses while left out; a reader's
/// value replaces it.
#[test]
fn the_line_types_while_focused_and_not_while_left_out() {
    let mut line = ComposeLine::new("Reply");
    assert!(!line.insert("a"), "unfocused");
    line.set_focused(true);
    assert!(line.insert("fox"));
    assert_eq!(line.text(), "fox");
    line.set_disabled(true);
    assert!(!line.is_focused());
    assert!(!line.insert("!"));
    assert!(!line.set_value("cat"));
    line.set_disabled(false);
    assert!(line.set_value("cat"));
    assert_eq!(line.text(), "cat");
}

/// The width sweep, 200 to 900, in the four themes and every state: the
/// box as wide as the column up to the kit's, every control in the box
/// and none over another, the selects beside the square buttons where
/// they fit and on a row of their own where not, what opens in the room
/// the box reserves.
#[test]
fn the_box_holds_over_every_width_in_every_state() {
    let states = || {
        [
            PromptInput::new(),
            PromptInput::new().voice(true),
            PromptInput::new().files_open(true),
            PromptInput::new().models_open(true),
            PromptInput::new().collapsed(true).premade(true),
            PromptInput::new().view_controller(true),
        ]
    };
    for theme in themes() {
        let t = &theme.message.prompt;
        for p in states() {
            for w in (200..=900).step_by(4) {
                let (bw, bh) = p.size(w as f32, &theme);
                assert_eq!(bw, (w as f32).min(t.max_w));
                let frame = Rect::new(30.0, 400.0, bw, bh);
                let (above, below) = p.room(frame, &theme);
                let block = Rect::new(frame.x, frame.y - above, bw, bh + above + below);
                let l = p.layout(frame, &theme);
                let parts = [l.add, l.setting, l.model, l.voice, l.submit];
                for (i, a) in parts.iter().enumerate() {
                    assert!(inside(*a, frame), "{w}: part {i}");
                    for b in &parts[i + 1..] {
                        let apart = a.right() <= b.x + 0.5
                            || b.right() <= a.x + 0.5
                            || a.bottom() <= b.y + 0.5
                            || b.bottom() <= a.y + 0.5;
                        assert!(apart, "{w}: {a:?} over {b:?}");
                    }
                }
                if let Some((panel, rows)) = &l.files {
                    assert!(inside(*panel, block), "{w}: files");
                    assert_eq!(rows.len(), 3);
                }
                if let Some(panel) = l.models {
                    assert!(inside(panel, block), "{w}: models");
                }
                if let Some(v) = l.view {
                    assert!(inside(v, block), "{w}: the view controller");
                    assert!(v.bottom() <= frame.y, "{w}: over the box");
                }
                if let Some(pl) = &l.premade {
                    assert!(inside(pl.panel, block), "{w}: premade");
                    for r in pl.rows.iter().chain(&pl.categories) {
                        assert!(inside(*r, pl.panel), "{w}: premade row");
                    }
                }
            }
        }
    }
    // the kit's box: 542 wide, its line and its one row of controls
    let p = PromptInput::new();
    assert_eq!(
        p.size(800.0, &core()),
        (542.0, 2.0 + 24.0 + 8.0 + 24.0 + 32.0 + 40.0)
    );
    let narrow = p.size(300.0, &core());
    assert_eq!(narrow.1, 2.0 + 24.0 + 8.0 + 24.0 + 32.0 + 40.0 * 2.0 + 8.0);
}

/// Voice input leaves out the line, the add button, the setting and the
/// model, asks something else and marks the corner; it closes what was
/// open, and comes back as it was.
#[test]
fn voice_input_leaves_out_what_it_does_not_use() {
    let theme = core();
    let mut p = PromptInput::new().models_open(true);
    p.toggle_voice();
    assert!(p.is_voice() && !p.is_models_open());
    assert_eq!(p.line().placeholder(), "Ask me anything...");
    assert!(p.line().disabled());
    let frame = Rect::new(0.0, 0.0, 542.0, 130.0);
    let l = p.layout(frame, &theme);
    assert!(l.mark.is_some());
    assert!(!p.toggle_files(), "the add button is left out");
    assert!(p.act(Part::Model, "open", None).is_err());
    p.toggle_voice();
    assert!(!p.line().disabled());
    assert_eq!(
        p.line().placeholder(),
        "Describe your 3D object or scene..."
    );
    assert!(p.layout(frame, &theme).mark.is_none());
}

/// Submit sends the line and clears it; a premade prompt goes into the
/// line, `Surprise me` the next one, a collapsed box shows its line once
/// it holds text; an added file closes the menu.
#[test]
fn the_box_sends_takes_a_premade_prompt_and_adds_a_file() {
    let theme = core();
    let id = SemId::parse("p").unwrap();
    let mut p = PromptInput::new().collapsed(true).premade(true).sem(id);
    let frame = Rect::new(0.0, 400.0, 542.0, 200.0);
    assert!(p.layout(frame, &theme).text.is_none(), "collapsed");
    assert_eq!(p.act(Part::Prompt(1), "activate", None), Ok(()));
    assert!(p.text().starts_with("Design a 3D medieval sword"));
    assert!(p.layout(frame, &theme).text.is_some(), "it opens with text");
    assert_eq!(p.act(Part::Submit, "activate", None), Ok(()));
    assert_eq!(
        (p.text(), p.sent().map(|s| !s.is_empty())),
        ("", Some(true))
    );
    assert_eq!(p.act(Part::Surprise, "activate", None), Ok(()));
    assert!(p.text().starts_with("Create a 3D model"));
    assert_eq!(p.act(Part::Add, "activate", None), Ok(()));
    assert!(p.is_files_open());
    assert_eq!(p.act(Part::File(2), "activate", None), Ok(()));
    assert_eq!((p.added(), p.is_files_open()), (1, false));
    let set = Value::Text("Describe".into());
    assert_eq!(p.act(Part::Setting, "select", Some(&set)), Ok(()));
    assert_eq!(p.setting(), Some("Describe"));
}

/// Every part's id names that part back, and the projection validates in
/// every state, the open menus' rows under their triggers.
#[test]
fn every_part_is_named_back_and_the_projection_validates() {
    let id = SemId::parse("page.prompt").unwrap();
    let parts = [
        Part::Text,
        Part::Add,
        Part::File(1),
        Part::Setting,
        Part::SettingOption(0),
        Part::Model,
        Part::Choice(3),
        Part::Voice,
        Part::Submit,
        Part::Prompt(2),
        Part::Surprise,
        Part::Category(4),
        Part::Orbit(false),
        Part::Orbit(true),
    ];
    let p = PromptInput::new()
        .premade(true)
        .view_controller(true)
        .sem(id.clone());
    for part in parts {
        let pid = p.part_id(part).expect("an id");
        assert_eq!(p.part_of(&pid), Some(part), "{pid}");
    }
    // the view controller's arrows take the focus a reader gives them
    for forward in [false, true] {
        let mut q = p.clone();
        q.focus(Some(Part::Orbit(forward)));
        assert_eq!(q.focused(), Some(Part::Orbit(forward)));
        q.focus(None);
        assert_eq!(q.focused(), None);
    }
    let mut q = PromptInput::new();
    q.focus(Some(Part::Orbit(true)));
    assert_eq!(q.focused(), None, "no controller, no focus on it");
    for p in [
        PromptInput::new().files_open(true),
        PromptInput::new().models_open(true),
        PromptInput::new().collapsed(true).premade(true),
        PromptInput::new().voice(true).premade(true),
        PromptInput::new().view_controller(true),
    ] {
        let p = p.sem(id.clone());
        let mut b = SnapshotBuilder::new();
        p.project(&mut b, &id, "Prompt input", None);
        let seen = b.finish().expect("the projection validates");
        let model = seen.get(&id.child("model").unwrap()).unwrap();
        assert_eq!(model.role, Role::Select);
        if p.is_files_open() {
            let row = id.child("files").unwrap().key("0");
            assert_eq!(seen.get(&row).unwrap().role, Role::MenuItem);
        }
        if p.is_models_open() {
            let row = id.child("model").unwrap().key("0");
            assert_eq!(seen.get(&row).unwrap().role, Role::Option);
        }
    }
}
