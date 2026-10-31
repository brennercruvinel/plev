use super::*;
use accord::{Actor, Value};

const EPS: f32 = 0.5;

fn within(inner: Rect, outer: Rect) -> bool {
    inner.x >= outer.x - EPS
        && inner.y >= outer.y - EPS
        && inner.right() <= outer.right() + EPS
        && inner.bottom() <= outer.bottom() + EPS
}

/// The width sweep, 280 to 1600 every 8 px, in the core and the hoff
/// themes: every box in its card's inside, and what opens over and under
/// it (the menus, the premade prompts) in its card too, clear of the
/// chip's band; the controls inside the box, side by side or the selects
/// on a row of their own; the groups down the page.
#[test]
fn the_page_holds_over_the_whole_width_range() {
    let page = PromptInputPage::new();
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        for w in (280..=1600).step_by(8) {
            let content = Rect::new(24.0, 0.0, w as f32, 800.0);
            let mut prev = content.y;
            for (group, (placed, boxes)) in page.groups.iter().zip(page.layout(content, &theme)) {
                assert!(placed.title_y >= prev, "{w}: {} climbs", group.title);
                prev = placed.rule_y;
                for ((card, rect), at) in group.cards.iter().zip(&placed.cards).zip(&boxes) {
                    let inside = showcase::body(*rect, &theme);
                    let name = card.title;
                    match &card.sample {
                        Sample::Prompt(p) => {
                            let (above, below) = p.room(*at, &theme);
                            let block = Rect::new(at.x, at.y - above, at.w, at.h + above + below);
                            assert!(within(block, *rect), "{w}: {name} leaves its card");
                            assert!(at.w <= inside.w + EPS, "{w}: {name} too wide");
                            let l = p.layout(*at, &theme);
                            for part in [l.add, l.setting, l.model, l.voice, l.submit] {
                                assert!(within(part, *at), "{w}: {name}: {part:?}");
                            }
                            for (a, b) in [(l.add, l.setting), (l.setting, l.model)] {
                                let apart = a.right() <= b.x + EPS || a.bottom() <= b.y + EPS;
                                assert!(apart, "{w}: {name}: {a:?} over {b:?}");
                            }
                            if let Some((panel, _)) = l.files {
                                assert!(within(panel, *rect), "{w}: {name}: files");
                            }
                            if let Some(panel) = l.models {
                                assert!(within(panel, *rect), "{w}: {name}: models");
                            }
                        }
                        Sample::Models(_) => assert!(within(*at, inside), "{w}: {name}"),
                    }
                }
            }
        }
    }
}

fn seen(page: &PromptInputPage) -> accord::Snapshot {
    let root = page.root().cloned().expect("the root parses");
    let mut b = SnapshotBuilder::new();
    b.node(&root, Role::Region, "Prompt Input");
    page.project(&mut b, &root);
    b.finish().expect("the projection validates")
}

fn ask(seen: &accord::Snapshot, target: &str, action: &str, param: Option<Value>) -> Request {
    let actor = Actor::Test {
        name: "prompt input page".to_owned(),
    };
    Request::observe(seen, actor, target, action, param).expect("declared")
}

const LIVE: &str = "prompt_input_page.group[0].card[0].prompt";

/// What a reader and an agent ask runs on the prompt: the line set,
/// sent (kept, and cleared), the model chosen from its open listbox,
/// voice input that leaves the line out.
#[test]
fn the_prompt_runs_from_the_projection() {
    let mut page = PromptInputPage::new();
    let text = format!("{LIVE}.text");
    let req = ask(
        &seen(&page),
        &text,
        "set_value",
        Some(Value::Text("a fox".into())),
    );
    assert_eq!(page.run(&req), Ok(After::Nothing));
    let req = ask(&seen(&page), &format!("{LIVE}.submit"), "activate", None);
    assert_eq!(page.run(&req), Ok(After::Nothing));
    let Sample::Prompt(p) = &page.groups[0].cards[0].sample else {
        panic!("a prompt");
    };
    assert_eq!((p.sent(), p.text()), (Some("a fox"), ""));

    let model = format!("{LIVE}.model");
    let req = ask(&seen(&page), &model, "open", None);
    assert_eq!(page.run(&req), Ok(After::Nothing));
    let s = seen(&page);
    assert!(
        s.get(&SemId::parse(&format!("{model}[2]")).unwrap())
            .is_some()
    );
    let req = ask(&s, &format!("{model}[2]"), "select", None);
    assert_eq!(page.run(&req), Ok(After::Nothing));
    let Sample::Prompt(p) = &page.groups[0].cards[0].sample else {
        panic!("a prompt");
    };
    assert_eq!(p.model(), Some("Kling AI"));
    assert!(!p.is_models_open());

    let req = ask(&seen(&page), &format!("{LIVE}.voice"), "toggle", None);
    assert_eq!(page.run(&req), Ok(After::Nothing));
    let s = seen(&page);
    let line = s
        .get(&SemId::parse(&text).unwrap())
        .expect("the line shows");
    assert!(!line.enabled, "voice input leaves the line out");
}

/// A reader's focus lands on a part and on an open menu's row, and
/// Escape closes the menu before it lets the focus go.
#[test]
fn escape_closes_what_is_open_then_lets_the_focus_go() {
    let mut page = PromptInputPage::new();
    let add = SemId::parse(&format!("{LIVE}.add")).unwrap();
    let req = ask(&seen(&page), add.as_str(), "activate", None);
    assert_eq!(page.run(&req), Ok(After::Nothing));
    let row = SemId::parse(&format!("{LIVE}.files[1]")).unwrap();
    assert!(page.focus(&row));
    assert_eq!(seen(&page).focused(), Some(&row));
    assert!(page.blur(), "Escape closes the menu");
    let Sample::Prompt(p) = &page.groups[0].cards[0].sample else {
        panic!("a prompt");
    };
    assert!(!p.is_files_open());
    assert!(page.focus(&add));
    assert!(page.blur(), "then the focus goes");
    assert_eq!(seen(&page).focused(), None);
}
