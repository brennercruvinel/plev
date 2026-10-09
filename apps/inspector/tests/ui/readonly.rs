//! no path of the window creates a request or calls a dispatcher, the
//! honest check on both sides: the crate's sources name nothing that
//! could, and its manifest depends on nothing new that does; and driving
//! every handler over every instant leaves the session and its file
//! exactly as they were, with no recorded control ever bound as a live
//! one in the window's compositor.

use std::collections::BTreeSet;
use std::path::Path;

use inspector::Key;

use crate::support::{Fallible, click, draw, fixture, open};

/// what the crate's sources must never name: the types that run an
/// action, record one, or build one; the calls that do; and every way to
/// send something out of the process (a socket, another process, the
/// agent port, the accessibility bridge).
const NEVER: [&str; 18] = [
    "Dispatcher",
    "Operable",
    "ReceiptSink",
    "Execution",
    "Recorder",
    "Request::",
    "Request {",
    ".dispatch(",
    ".execute(",
    ".confirm(",
    ".decline(",
    "std::net",
    "TcpStream",
    "UnixStream",
    "Command::new",
    "wire::",
    "accessibility",
    "Bridge",
];

fn sources(dir: &Path, out: &mut Vec<(String, String)>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            sources(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push((path.display().to_string(), std::fs::read_to_string(&path)?));
        }
    }
    Ok(())
}

#[test]
fn the_crate_holds_nothing_that_runs_an_action_or_sends_anything() -> Fallible {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    sources(&root.join("src"), &mut files)?;
    assert!(files.len() >= 9, "the sources were read: {}", files.len());
    for (path, body) in &files {
        for word in NEVER {
            assert!(!body.contains(word), "{path} names {word}");
        }
    }
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))?;
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .and_then(|rest| rest.split("\n[").next())
        .ok_or("a [dependencies] table")?;
    let names: Vec<&str> = deps
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split(['.', ' ', '=']).next())
        .collect();
    assert_eq!(names, ["comps", "engine", "inspect", "rec", "sem"]);
    assert!(
        !manifest.contains("[dev-dependencies]"),
        "nothing more for the tests"
    );
    Ok(())
}

#[test]
fn driving_every_handler_leaves_the_session_as_it_was() -> Fallible {
    for name in ["a2ppa", "bug84"] {
        let bytes = std::fs::read(fixture(name))?;
        let mut view = open(name, 1280.0, 800.0)?;
        let recorded: BTreeSet<String> = view
            .session()
            .frame_times()
            .into_iter()
            .flat_map(|t| view.session().bindings_at(t).unwrap_or_default())
            .map(|b| b.id.to_string())
            .collect();
        assert!(!recorded.is_empty(), "{name} binds controls");

        let mut instants: Vec<u64> = view.strip().instants.clone();
        instants.extend(
            view.session()
                .frame_times()
                .iter()
                .map(|t| t - view.span().0),
        );
        for (n, at) in instants.iter().enumerate() {
            view.seek_at(*at);
            // every control on the frame: hovered, clicked.
            for b in view.bound().to_vec() {
                if let Some(r) = view.control_rect(&b.id) {
                    let (x, y) = r.center();
                    click(&mut view, x, y);
                }
            }
            // the whole window, on a grid, every few instants: the list,
            // the strip, the scrub, the panel, the header.
            if n % 4 == 0 {
                let (w, h) = view.size();
                for i in 0..32 {
                    for j in 0..20 {
                        let (x, y) = ((i as f32 + 0.5) * w / 32.0, (j as f32 + 0.5) * h / 20.0);
                        click(&mut view, x, y);
                        view.scroll(x, y, 40.0);
                        view.seek_at(*at);
                    }
                }
            }
            for key in [Key::Next, Key::Previous, Key::First, Key::Last, Key::Close] {
                view.key(key);
            }
            let c = draw(&mut view);
            let bound: Vec<String> = c.visible_bindings().map(|(id, _)| id.to_string()).collect();
            for id in &bound {
                assert!(!recorded.contains(id), "{name}: {id} bound in the window");
            }
        }
        for i in 0..view.requests().len() {
            view.go_to_request(i);
            draw(&mut view);
        }

        let fresh = record::Session::open(fixture(name))?;
        assert!(view.session() == &fresh, "{name}: the session is as it was");
        assert_eq!(std::fs::read(fixture(name))?, bytes, "{name}: the file too");
    }
    Ok(())
}
