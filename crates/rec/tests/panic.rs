//! the panic hook, alone in its own test binary because a hook is global
//! to the process: a panic persists the window to the directory, the hook
//! installed before still runs, and a hook that cannot write never makes
//! things worse.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use engine::compositor::SceneNode;
use rec::{InputKind, Kind, Recorder, Session, install_panic_hook};
use sem::{Role, SnapshotBuilder};

fn sessions_in(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out: Vec<_> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "monster"))
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out
}

#[test]
fn a_panic_persists_the_window_and_chains_the_previous_hook()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dir = tmp.path().join("crashes");
    let rec = Recorder::default();
    let mut b = SnapshotBuilder::new();
    b.node("app", Role::Group, "app").state("n", 1i64);
    rec.snapshot(10, &b.finish()?);
    rec.input(20, InputKind::Key, None, None);
    rec.frame_scene(
        30,
        &[SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: 4.0,
            h: 4.0,
            color: [1.0; 4],
        }],
    )?;

    let before = Arc::new(AtomicUsize::new(0));
    let seen = before.clone();
    std::panic::set_hook(Box::new(move |_| {
        seen.fetch_add(1, Ordering::SeqCst);
    }));
    install_panic_hook(rec.clone(), &dir);

    let crashed = std::thread::spawn(|| panic!("the app fell over")).join();
    assert!(crashed.is_err());
    assert_eq!(before.load(Ordering::SeqCst), 1, "the previous hook ran");
    let files = sessions_in(&dir);
    let [file] = files.as_slice() else {
        return Err(format!("one session in {dir:?}, got {files:?}").into());
    };
    let name = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    assert!(name.starts_with("session-"), "{name}");
    let session = Session::open(file)?;
    assert_eq!(session, rec.snapshot_window());
    let kinds: Vec<&str> = session.events().iter().map(|e| e.kind.name()).collect();
    assert_eq!(kinds, ["snapshot", "input"]);
    assert!(matches!(session.events()[0].kind, Kind::Snapshot { .. }));
    assert!(session.scene_at(30)?.is_some());

    // a second panic writes a second file, never over the first.
    let again = std::thread::spawn(|| panic!("and again")).join();
    assert!(again.is_err());
    assert_eq!(sessions_in(&dir).len(), 2);
    assert_eq!(before.load(Ordering::SeqCst), 2);

    // a directory that cannot be made: the hook writes nothing, and the
    // panic goes on as before, through the previous hook.
    let blocked = tmp.path().join("a-file");
    std::fs::write(&blocked, b"not a directory")?;
    install_panic_hook(rec.clone(), blocked.join("crashes"));
    let third = std::thread::spawn(|| panic!("nowhere to write")).join();
    assert!(third.is_err());
    assert_eq!(before.load(Ordering::SeqCst), 3);
    assert_eq!(sessions_in(&dir).len(), 3, "the hook before it still wrote");
    Ok(())
}
