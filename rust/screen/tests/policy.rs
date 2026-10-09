//! What the host keeps for each surface (task-70 phase 2): the policy it
//! was verified under, so its operations are verified under it again
//! whoever asks; and the changes of one ask together, applied all or
//! none and undone whole.

// shared with the other surface tests: this binary uses the fixtures and
// the app.
#[allow(dead_code)]
mod support;

use sem::{Actor, Actors, Policy, Role, Snapshot, SnapshotBuilder};
use support::{MIXER_LEFT, id, mixer, verified_local};
use surface::{SurfaceError, SurfaceHost};
use uir::{Capabilities, GenerationPolicy, Op, Step, TransformError, verify_json};

fn agent() -> Actor {
    Actor::Agent {
        name: "claude".to_owned(),
    }
}

/// A lock only a person may toggle.
fn lock() -> Snapshot {
    let kept = Policy {
        actors: Actors {
            human: true,
            agent: false,
            assistive: true,
            test: true,
        },
        ..Policy::default()
    };
    let mut b = SnapshotBuilder::new();
    b.node("vault", Role::Region, "vault");
    let mut lock = b.node("vault.lock", Role::Toggle, "locked");
    lock.parent("vault").state("on", true);
    lock.action("toggle").reads("vault.lock:on").policy(kept);
    b.finish().expect("the vault projects")
}

const LOCK_PANEL: &str = r#"{"plev_ir": 1, "surface": {"id": "vault_panel", "title": "vault",
    "root": {"sem": "vault_panel.root", "layout": {"stack": {"axis": "vertical", "gap": "md"}},
      "children": [
        {"sem": "vault_panel.lock", "component": "checkbox", "label": "locked",
         "bind": {"state": {"checked": "vault.lock:on"},
                  "actions": {"toggle": {"target": "vault.lock", "action": "toggle"}}}}
      ]}}}"#;

/// A person wrote the panel (it binds the lock, which only a person may
/// ask for): opened under the person's policy, an agent's density
/// applies, verified again under the person's policy the surface keeps.
/// The same panel under the default policy (an agent wrote it) would not
/// verify at all; the host does not verify it under that.
#[test]
fn an_operation_is_verified_under_the_surfaces_own_policy() {
    let app = lock();
    let caps = Capabilities::from_snapshot(&app);
    let person = GenerationPolicy::by(Actor::Human);
    let ir = verify_json(LOCK_PANEL, &caps, &person).expect("a person may bind the lock");
    assert!(
        verify_json(LOCK_PANEL, &caps, &GenerationPolicy::default()).is_err(),
        "an agent may not"
    );
    let mut host = SurfaceHost::new();
    let panel = id("vault_panel");
    host.open_under(ir.clone(), &app, person.clone())
        .expect("opens");
    assert_eq!(host.policy(&panel), Some(&person));
    host.apply(&panel, Op::Density { step: Step::Denser }, agent(), &app)
        .expect("verified under the person's policy");
    assert_eq!(host.changes(&panel)[0].by, agent());

    // a new form under another policy replaces it; closing lets it go
    host.transform_under(ir, &app, GenerationPolicy::by(Actor::Assistive))
        .expect("the whole form");
    assert_eq!(
        host.policy(&panel),
        Some(&GenerationPolicy::by(Actor::Assistive))
    );
    assert!(host.close(&panel));
    assert_eq!(host.policy(&panel), None);
}

/// Three operations asked together: one form taken, three changes kept
/// as one ask; the ask undone whole gives the form before, byte for
/// byte. A list with one refusal changes nothing.
#[test]
fn the_changes_of_one_ask_are_all_or_none_and_undone_together() {
    let now = mixer();
    let mut host = SurfaceHost::new();
    let panel = id("mix_panel");
    host.open(verified_local(MIXER_LEFT, &now), &now)
        .expect("opens");
    assert_eq!(host.policy(&panel), Some(&GenerationPolicy::default()));
    let written = |host: &SurfaceHost| {
        serde_json::to_string(host.get(&panel).expect("open").verified().ir()).unwrap()
    };
    let at_open = written(&host);
    host.apply(
        &panel,
        Op::Hide {
            id: id("mix_panel.reset"),
        },
        Actor::Human,
        &now,
    )
    .expect("one first");
    let after_one = written(&host);

    let refused = host.apply_all(
        &panel,
        vec![
            Op::Density { step: Step::Denser },
            Op::Hide {
                id: id("mix_panel.card"),
            },
        ],
        agent(),
        &now,
    );
    assert!(matches!(
        refused.err(),
        Some(SurfaceError::Transform(TransformError::Refused(_)))
    ));
    assert_eq!(written(&host), after_one, "none applied");
    assert_eq!(host.changes(&panel).len(), 1);

    let asked = vec![
        Op::Density { step: Step::Denser },
        Op::Emphasize {
            id: id("mix_panel.left"),
        },
        Op::Density { step: Step::Denser },
    ];
    host.apply_all(&panel, asked.clone(), agent(), &now)
        .expect("all three");
    assert_eq!(host.changes(&panel).len(), 4);
    let current = host.current(&panel).expect("open");
    assert_eq!(current.aside().count(), 1, "the reset is still aside");

    let undone = host.undo_asked(&panel, &now).expect("the ask undone");
    let ops: Vec<Op> = undone.into_iter().map(|c| c.op).collect();
    assert_eq!(ops, asked);
    assert_eq!(written(&host), after_one, "byte for byte");
    let undone = host.undo_asked(&panel, &now).expect("the first ask");
    assert_eq!(undone.len(), 1);
    assert_eq!(written(&host), at_open);
    assert_eq!(
        host.undo_asked(&panel, &now).err(),
        Some(SurfaceError::NothingToUndo(panel.clone()))
    );
}
