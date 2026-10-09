//! what the window shows of a session never goes past what the session
//! kept, over everything the view draws, not a sample:
//!
//! - a camera's pixels never reach it: the view draws no image at all, not
//!   even from a session read in the process that recorded it, where the
//!   image's handle still resolves (a file carries no pixel either);
//! - a secret is shown only as the session kept it: its length. the panel
//!   is the model's text over the recorded (redacted) values;
//! - the accessibility preference, and anything else, never leaves by the
//!   port or into a prompt: the view sends nothing anywhere (its sources
//!   name no socket, no process, no port, no bridge: `readonly.rs`).

use engine::drawing::compositor::SceneNode;
use engine::drawing::gpu::ImageHandle;
use inspector::{InspectorView, Key};
use record::{Limits, Recorder};
use accord::{
    Actor, Constraints, Dispatcher, Execution, Operable, Outcome, ParamSpec, Request, RequestId,
    Role, SemId, Semantics, Snapshot, SnapshotBuilder, Value, ValueKind,
};

use crate::support::{Fallible, draw, id, nodes, open, texts};

fn images(view: &mut InspectorView) -> usize {
    nodes(&draw(view))
        .iter()
        .filter(|n| matches!(n, SceneNode::Image { .. }))
        .count()
}

#[test]
fn the_window_draws_no_image_at_any_instant() -> Fallible {
    // a frame with an image, read in the process that recorded it: the
    // session's own replay still has the image, the window does not.
    let rec = Recorder::new(Limits::default());
    let camera = ImageHandle {
        id: 7,
        atlas_x: 0,
        atlas_y: 0,
        width: 64,
        height: 48,
    };
    let frame = [
        SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: 640.0,
            h: 400.0,
            color: [0.1, 0.1, 0.1, 1.0],
        },
        SceneNode::Image {
            x: 100.0,
            y: 80.0,
            w: 320.0,
            h: 240.0,
            image: camera,
            corner_radius: 0.0,
        },
    ];
    rec.frame_scene(10, &frame)?;
    rec.frame_scene(20, &frame)?;
    let session = rec.snapshot_window();
    let replayed = session.scene_at(20)?.ok_or("a frame")?;
    assert!(
        replayed
            .iter()
            .any(|n| matches!(n, SceneNode::Image { .. })),
        "the replay in process still holds the image"
    );
    let mut view = InspectorView::new(session, "camera", 1280.0, 800.0)?;
    for t in [10, 15, 20] {
        view.seek(t);
        assert!(view.fit().is_some(), "the frame is shown");
        assert_eq!(images(&mut view), 0, "no image at {t}");
    }

    // the sessions the repository keeps, at every frame.
    for name in ["a2ppa", "bug84"] {
        let mut view = open(name, 1280.0, 800.0)?;
        for t in view.session().frame_times() {
            view.seek(t);
            assert_eq!(images(&mut view), 0, "{name} at {t}");
        }
    }
    Ok(())
}

const SECRET: &str = "hunter2-correct-horse";

struct Login {
    user: String,
    password: String,
}

impl Semantics for Login {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node("login.user", Role::TextInput, "user")
            .state("text", self.user.as_str());
        b.node("login.password", Role::TextInput, "password")
            .state("text", self.password.as_str())
            .constraints(Constraints {
                secret: true,
                editable: true,
                ..Constraints::default()
            })
            .action("set")
            .param(ParamSpec {
                kind: ValueKind::Text,
                constrained: false,
            });
        b.node("login.submit", Role::Button, "sign in")
            .action("submit")
            .reads("login.user:text")
            .reads("login.password:text");
    }
}

impl Operable for Login {
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        match (req.action.as_str(), &req.param) {
            ("set", Some(Value::Text(secret))) => {
                self.password = secret.clone();
                // the app echoes it back: the recorder keeps its length.
                Execution::Done(Outcome {
                    result: [("echo".to_owned(), Value::Text(secret.clone()))]
                        .into_iter()
                        .collect(),
                })
            }
            _ => Execution::Done(Outcome::default()),
        }
    }
}

fn look(app: &impl Semantics) -> Result<Snapshot, Box<dyn std::error::Error>> {
    let mut b = SnapshotBuilder::new();
    app.project(&mut b);
    Ok(b.finish()?)
}

fn agent() -> Actor {
    Actor::Agent {
        name: "reviewer".to_owned(),
    }
}

#[test]
fn a_secret_shows_only_as_the_session_kept_it() -> Fallible {
    let rec = Recorder::new(Limits::default());
    let mut dispatcher = Dispatcher::new();
    dispatcher.add_sink(Box::new(rec.sink()));
    let mut login = Login {
        user: "ana".to_owned(),
        password: String::new(),
    };
    rec.snapshot(0, &look(&login)?);
    let seen = look(&login)?;
    rec.observed(5, agent(), None, seen.revision());
    let set = Request::observe(
        &seen,
        Actor::Human,
        "login.password",
        "set",
        Some(SECRET.into()),
    )?;
    dispatcher.dispatch(&mut login, set, 10);
    assert_eq!(login.password, SECRET, "the app got the secret");
    rec.snapshot(10, &look(&login)?);
    // the agent submits on what it saw before: refused stale, its delta on
    // the password.
    let submit = Request::observe(&seen, agent(), "login.submit", "submit", None)?;
    dispatcher.dispatch(&mut login, submit, 20);
    rec.snapshot(20, &look(&login)?);

    let mut view = InspectorView::new(rec.snapshot_window(), "login", 1280.0, 800.0)?;
    let ids: Vec<SemId> = ["login.user", "login.password", "login.submit"]
        .into_iter()
        .map(id)
        .collect();
    let mut shown: Vec<String> = Vec::new();
    let mut instants: Vec<u64> = view.strip().instants.clone();
    instants.push(view.strip().span);
    for at in instants {
        view.seek_at(at);
        for id in &ids {
            view.pick(Some(id.clone()));
            shown.extend(view.rows().iter().map(|r| r.text.clone()));
            shown.extend(texts(&draw(&mut view)));
        }
        view.key(Key::Close);
    }
    for i in 0..view.requests().len() {
        view.go_to_request(i);
        shown.extend(view.rows().iter().map(|r| r.text.clone()));
        shown.extend(texts(&draw(&mut view)));
    }
    shown.extend(view.strip().lanes.iter().cloned());

    let all = shown.join("\n");
    assert!(!all.contains(SECRET), "the secret is shown");
    assert!(!all.contains("hunter2"), "a piece of it is shown");
    // not an empty check: the panel did answer about the password, its
    // request and the refusal on it.
    assert!(all.contains("set on login.password by human"), "{all}");
    assert!(
        all.contains("submit on login.submit by agent:reviewer"),
        "{all}"
    );
    assert!(all.contains("login.password:text"), "{all}");
    Ok(())
}
