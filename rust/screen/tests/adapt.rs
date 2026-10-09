//! A surface adapted one operation at a time (task-70): `SurfaceHost::apply`
//! verifies the operation again, reconciles the new form by `SemId` as a
//! whole new form is reconciled, and keeps the change with its inverse and
//! who asked; `undo` takes back the last one. What lives in a widget whose
//! node stays (a drag in flight, a caret, a press on one cut) stays; and an
//! action the app keeps from agents is still kept from them, projected
//! with the app's own policy, whatever the surface became.

// shared with the other surface tests: this binary uses the fixtures, the
// page and the app.
#[allow(dead_code)]
mod support;

use engine::widgets::form::EditKey;
use engine::widgets::{Rect, WidgetEvent};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use accord::{Actor, Actors, Policy, Role, SemId, Snapshot, SnapshotBuilder, StateRef, Value};
use support::{
    App, MIXER_LEFT, REVIEW_SILENCES, SHOWCASE, SILENCES, TODO, id, mixer, page, settle, snapshot,
    verified, verified_local,
};
use screen::{Change, Surface, SurfaceError, SurfaceHost};
use plevir::{Capabilities, Component, GenerationPolicy, Inverse, Op, TransformError, verify_json};

const AREA: Rect = Rect {
    x: 40.0,
    y: 30.0,
    w: 900.0,
    h: 600.0,
};

fn frame(surface: &mut Surface, theme: &Theme) -> Compositor {
    let mut c = Compositor::new();
    surface.render(&mut c, AREA, theme);
    c
}

fn bound(c: &Compositor, at: &SemId) -> [f32; 4] {
    c.binding(at)
        .unwrap_or_else(|| panic!("{at} is bound"))
        .rect
}

fn person() -> Actor {
    Actor::Human
}

fn agent() -> Actor {
    Actor::Agent {
        name: "claude".to_owned(),
    }
}

/// The x at the fraction `t` of a slider's travel drawn at `r`.
fn along(r: [f32; 4], t: f32, theme: &Theme) -> f32 {
    let knob = theme.control.slider_knob;
    r[0] + knob / 2.0 + t * (r[2] - knob)
}

fn volume(app: &App) -> f64 {
    match app
        .snapshot()
        .value(&StateRef::parse("mixer.volume:value").unwrap())
    {
        Some(Value::Num(n)) => *n,
        other => panic!("a number: {other:?}"),
    }
}

/// A drag on the volume goes on while the mixer changes form twice, a
/// hide and a reorder, and while the reorder is undone.
#[test]
fn a_drag_goes_on_through_a_hide_a_reorder_and_an_undo() {
    let theme = Theme::hoff_core();
    let mut app = App::over(&mixer());
    let now = app.snapshot();
    let mut host = SurfaceHost::new();
    let panel = id("mix_panel");
    host.open(verified_local(MIXER_LEFT, &now), &now)
        .expect("opens");
    let slider = id("mix_panel.volume");
    let surface = host.get_mut(&panel).expect("open");
    let c = frame(surface, &theme);
    let at = bound(&c, &slider);
    let y = at[1] + at[3] / 2.0;
    for event in [
        WidgetEvent::MouseMove {
            x: along(at, 0.25, &theme),
            y,
        },
        WidgetEvent::MouseDown {
            x: along(at, 0.25, &theme),
            y,
        },
        WidgetEvent::MouseMove {
            x: along(at, 0.5, &theme),
            y,
        },
    ] {
        surface.handle_event(&event, &theme, &mut app);
    }
    assert!((volume(&app) - 0.5).abs() < 1e-3, "{}", volume(&app));

    let now = app.snapshot();
    let hide = Op::Hide {
        id: id("mix_panel.right"),
    };
    host.apply(&panel, hide, person(), &now)
        .expect("the right panel hides");
    let reorder = Op::Reorder {
        parent: id("mix_panel.left"),
        ids: vec![id("mix_panel.reset")],
    };
    let surface = host
        .apply(&panel, reorder, person(), &now)
        .expect("the reset goes first");
    let seen = page(&now, surface);
    assert!(
        seen.get(&id("mix_panel.level")).is_none(),
        "hidden, not projected"
    );
    let (c, _) = settle(surface, AREA, &theme);
    let moved = bound(&c, &slider);
    assert_ne!(moved, at, "the slider moved down, under the reset");
    // the button is still down: the drag goes on over the new rect
    let y = moved[1] + moved[3] / 2.0;
    let x = along(moved, 0.8, &theme);
    surface.handle_event(&WidgetEvent::MouseMove { x, y }, &theme, &mut app);
    assert!((volume(&app) - 0.8).abs() < 1e-3, "{}", volume(&app));

    // the reorder undone: the same widget, still in the drag
    let now = app.snapshot();
    let undone = host.undo(&panel, &now).expect("undoes");
    assert!(matches!(undone.op, Op::Reorder { .. }));
    let surface = host.get_mut(&panel).expect("open");
    let (c, _) = settle(surface, AREA, &theme);
    let back = bound(&c, &slider);
    let y = back[1] + back[3] / 2.0;
    let x = along(back, 0.3, &theme);
    surface.handle_event(&WidgetEvent::MouseMove { x, y }, &theme, &mut app);
    assert!((volume(&app) - 0.3).abs() < 1e-3, "{}", volume(&app));
    surface.handle_event(&WidgetEvent::MouseUp { x, y }, &theme, &mut app);
    assert_eq!(host.changes(&panel).len(), 1, "the hide is left");
}

fn text(app: &App) -> Option<Value> {
    app.snapshot()
        .value(&StateRef::parse("todo.input:text").unwrap())
        .cloned()
}

/// A person types in the todo, the caret three places back; the entry
/// row is emphasized and the counts hidden: the field keeps the keyboard
/// and its caret, and the key lands where it was.
#[test]
fn a_caret_stays_where_it_was_through_the_operations() {
    let theme = Theme::hoff_core();
    let mut app = App::over(&snapshot(SHOWCASE));
    let now = app.snapshot();
    let mut host = SurfaceHost::new();
    let panel = id("todo_panel");
    let field = id("todo_panel.input");
    host.open(verified(TODO, &now), &now).expect("opens");
    let surface = host.get_mut(&panel).expect("open");
    let c = frame(surface, &theme);
    let r = bound(&c, &field);
    let (x, y) = (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0);
    for event in [
        WidgetEvent::MouseMove { x, y },
        WidgetEvent::MouseDown { x, y },
        WidgetEvent::MouseUp { x, y },
    ] {
        surface.handle_event(&event, &theme, &mut app);
    }
    for ch in "buy milk".chars() {
        assert!(surface.handle_text(&ch.to_string(), &mut app));
    }
    for _ in 0..3 {
        assert!(surface.handle_edit_key(EditKey::Left, &mut app));
    }
    let now = app.snapshot();
    for op in [
        Op::Hide {
            id: id("todo_panel.counts"),
        },
        Op::Emphasize {
            id: id("todo_panel.add"),
        },
        Op::Reorder {
            parent: id("todo_panel.card"),
            ids: vec![id("todo_panel.filter")],
        },
    ] {
        host.apply(&panel, op, person(), &now).expect("applies");
    }
    let surface = host.get_mut(&panel).expect("open");
    assert_eq!(surface.focused(), Some(&field));
    assert!(surface.handle_text("X", &mut app));
    assert_eq!(text(&app), Some(Value::Text("buy mXilk".to_owned())));
    let (c, _) = settle(surface, AREA, &theme);
    assert!(
        bound(&c, &field)[1] > r[1],
        "the field went down, under the filter"
    );
}

/// A press begun on the apply of the first cut, a reorder that puts the
/// list of the cuts above the video: the instance of the cut keeps its
/// widget, so the release completes the click and the app applies the
/// cut through its dispatcher.
#[test]
fn a_press_on_one_cut_completes_after_the_list_moves_up() {
    let theme = Theme::hoff_core();
    let mut app = App::over(&snapshot(SILENCES));
    let now = app.snapshot();
    let mut host = SurfaceHost::new();
    let review = id("review");
    host.open(verified_local(REVIEW_SILENCES, &now), &now)
        .expect("the review opens");
    let surface = host.get_mut(&review).expect("open");
    let c = frame(surface, &theme);
    let apply = bound(&c, &id("review.cut[1].apply"));
    let (x, y) = (apply[0] + apply[2] / 2.0, apply[1] + apply[3] / 2.0);
    for event in [
        WidgetEvent::MouseMove { x, y },
        WidgetEvent::MouseDown { x, y },
    ] {
        surface.handle_event(&event, &theme, &mut app);
    }
    let now = app.snapshot();
    let up = Op::Reorder {
        parent: id("review.card"),
        ids: vec![id("review.cuts")],
    };
    let surface = host
        .apply(&review, up, person(), &now)
        .expect("the cuts go first");
    let (c, _) = settle(surface, AREA, &theme);
    let moved = bound(&c, &id("review.cut[1].apply"));
    assert!(
        moved[1] < apply[1],
        "the cut is higher up: {moved:?} {apply:?}"
    );
    let (x, y) = (moved[0] + moved[2] / 2.0, moved[1] + moved[3] / 2.0);
    for event in [
        WidgetEvent::MouseMove { x, y },
        WidgetEvent::MouseUp { x, y },
    ] {
        surface.handle_event(&event, &theme, &mut app);
    }
    let ran = app.executed.last().expect("the click completed");
    assert_eq!(
        (ran.target.as_str(), ran.action.as_str()),
        ("silences.cut[1]", "apply_cut")
    );
}

/// The history: each change with its operation, its inverse and who
/// asked; undo takes the last back, byte for byte, until none is left; a
/// refused operation and a whole new form change nothing and end the
/// history.
#[test]
fn the_host_keeps_every_change_and_undoes_the_last() {
    let app = App::over(&mixer());
    let now = app.snapshot();
    let mut host = SurfaceHost::new();
    let panel = id("mix_panel");
    let opened = verified_local(MIXER_LEFT, &now);
    host.open(opened.clone(), &now).expect("opens");
    let written = |host: &SurfaceHost| {
        serde_json::to_string(host.get(&panel).expect("open").verified().ir()).unwrap()
    };
    let at_open = written(&host);

    let dense = Op::Density {
        step: plevir::Step::Denser,
    };
    host.apply(&panel, dense.clone(), agent(), &now)
        .expect("denser");
    let mute = Op::SwapView {
        id: id("mix_panel.mute"),
        component: Component::Checkbox,
    };
    host.apply(&panel, mute.clone(), person(), &now)
        .expect("the mute as a checkbox");
    let changes: Vec<(&Op, &Actor)> = host
        .changes(&panel)
        .iter()
        .map(|c| (&c.op, &c.by))
        .collect();
    assert_eq!(changes, [(&dense, &agent()), (&mute, &person())]);
    assert_eq!(
        host.changes(&panel)[1].inverse,
        Inverse::Op(Op::SwapView {
            id: id("mix_panel.mute"),
            component: Component::Switch,
        })
    );
    let seen = page(&now, host.get(&panel).expect("open"));
    assert_eq!(
        seen.get(&id("mix_panel.mute")).map(|n| n.role),
        Some(Role::Checkbox)
    );

    // refused: the surface as it was, the history as it was
    let before = written(&host);
    let refused = host.apply(
        &panel,
        Op::Hide {
            id: id("mix_panel.card"),
        },
        person(),
        &now,
    );
    assert!(matches!(
        refused.err(),
        Some(SurfaceError::Transform(TransformError::Refused(_)))
    ));
    assert_eq!(written(&host), before);
    assert_eq!(host.changes(&panel).len(), 2);

    let undone: Change = host.undo(&panel, &now).expect("undoes");
    assert_eq!(undone.op, mute);
    host.undo(&panel, &now).expect("undoes");
    assert_eq!(written(&host), at_open, "byte for byte");
    assert_eq!(
        host.undo(&panel, &now).err(),
        Some(SurfaceError::NothingToUndo(panel.clone()))
    );

    // a whole new form ends the history
    host.apply(
        &panel,
        Op::Hide {
            id: id("mix_panel.reset"),
        },
        person(),
        &now,
    )
    .expect("hides");
    assert!(host.adapted(&panel).is_some_and(|a| a.aside().count() == 1));
    host.transform(opened, &now).expect("the whole form");
    assert!(host.changes(&panel).is_empty());
    assert!(host.adapted(&panel).is_none());
    assert_eq!(
        host.apply(&id("nope"), Op::Hide { id: id("x") }, person(), &now)
            .err(),
        Some(SurfaceError::NotOpen(id("nope")))
    );
}

/// Who may ask for the vault's actions: a person, at the screen or
/// through assistive technology, and a test; never an agent (the
/// showcase's `PERSON_ONLY`).
const PERSON_ONLY: Actors = Actors {
    human: true,
    agent: false,
    assistive: true,
    test: true,
};

/// A vault whose lock and open are kept from agents.
fn vault() -> Snapshot {
    let kept = Policy {
        actors: PERSON_ONLY,
        ..Policy::default()
    };
    let mut b = SnapshotBuilder::new();
    b.node("vault", Role::Region, "vault");
    let mut lock = b.node("vault.lock", Role::Toggle, "locked");
    lock.parent("vault").state("on", true);
    lock.action("toggle").reads("vault.lock:on").policy(kept);
    let mut open = b.node("vault.open", Role::Button, "open the vault");
    open.parent("vault");
    open.action("open").policy(kept);
    b.node("vault.note", Role::Label, "note")
        .parent("vault")
        .state("text", "the key is in the drawer");
    b.finish().expect("the vault projects")
}

const VAULT_PANEL: &str = r#"{"plev_ir": 1, "surface": {"id": "vault_panel", "title": "vault",
    "root": {"sem": "vault_panel.root", "layout": {"stack": {"axis": "vertical", "gap": "md"}},
      "children": [
        {"sem": "vault_panel.note", "component": "text", "bind": {"state": {"text": "vault.note:text"}}},
        {"sem": "vault_panel.lock", "component": "checkbox", "label": "locked",
         "bind": {"state": {"checked": "vault.lock:on"},
                  "actions": {"toggle": {"target": "vault.lock", "action": "toggle"}}}}
      ]}}}"#;

/// The policy of `action` on the node `at` of the page `seen`.
fn actors(seen: &Snapshot, at: &str, action: &str) -> Option<Actors> {
    seen.get(&id(at))?
        .actions
        .iter()
        .find(|a| a.name.as_str() == action)
        .map(|a| a.policy.actors)
}

/// A surface a person wrote over the vault binds the lock, which the
/// vault keeps from agents. Through a group, a reorder, a swap and a
/// promotion of the open, every action of the surface is projected with
/// the vault's own policy: kept from agents. An agent cannot promote the
/// open at all.
#[test]
fn an_action_kept_from_agents_stays_kept_through_every_operation() {
    let app = vault();
    let policy = GenerationPolicy::by(person());
    let ir = verify_json(VAULT_PANEL, &Capabilities::from_snapshot(&app), &policy)
        .unwrap_or_else(|e| panic!("{e:#?}"));
    let mut host = SurfaceHost::new();
    let panel = id("vault_panel");
    host.open_under(ir, &app, policy.clone()).expect("opens");
    assert_eq!(host.policy(&panel), Some(&policy));
    let seen = page(&app, host.get(&panel).expect("open"));
    assert_eq!(
        actors(&seen, "vault_panel.lock", "toggle"),
        Some(PERSON_ONLY)
    );

    let promote = Op::Promote {
        target: id("vault.open"),
        action: "open".parse().unwrap(),
        into: id("vault_panel.root"),
    };
    let refused = host.apply(&panel, promote.clone(), agent(), &app);
    assert!(
        matches!(&refused, Err(SurfaceError::Transform(TransformError::Refused(why))) if why.contains("from agent:claude")),
        "{refused:?}"
    );
    for op in [
        Op::Group {
            ids: vec![id("vault_panel.lock")],
            into: id("vault_panel.guard"),
        },
        Op::Reorder {
            parent: id("vault_panel.root"),
            ids: vec![id("vault_panel.guard")],
        },
        Op::SwapView {
            id: id("vault_panel.lock"),
            component: Component::Switch,
        },
        promote,
    ] {
        let surface = host
            .apply(&panel, op.clone(), person(), &app)
            .unwrap_or_else(|e| panic!("{op}: {e}"));
        let seen = page(&app, surface);
        assert_eq!(
            actors(&seen, "vault_panel.lock", "toggle"),
            Some(PERSON_ONLY),
            "{op}"
        );
    }
    let seen = page(&app, host.get(&panel).expect("open"));
    assert_eq!(
        seen.get(&id("vault_panel.lock")).map(|n| n.role),
        Some(Role::Toggle)
    );
    assert_eq!(
        actors(&seen, "vault_panel.root.open", "activate"),
        Some(PERSON_ONLY),
        "the promoted open is kept from agents as the vault keeps it"
    );
    assert_eq!(host.changes(&panel).len(), 4);
}
