//! `plev-inspector <session.monster>`: the session inspector's window.
//! the session comes by its path, never by a dialog. exit 2 for a bad
//! invocation, 3 for a file that does not open as a session (the codes of
//! `plev-inspect`).

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn main() -> std::process::ExitCode {
    use std::path::PathBuf;
    use std::process::ExitCode;

    const USAGE: &str = "usage: plev-inspector <session.monster>";
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = match args.as_slice() {
        [flag] if flag == "--help" || flag == "-h" => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        [path] => PathBuf::from(path),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let view = match inspector::InspectorView::open(&path, 1280.0, 800.0) {
        Ok(view) => view,
        Err(e) => {
            eprintln!("plev-inspector: {}: {e}", path.display());
            return ExitCode::from(3);
        }
    };
    let title = format!("plev inspector: {}", path.display());
    match inspector::window::run(view, title) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("plev-inspector: {e}");
            ExitCode::FAILURE
        }
    }
}

// the window is desktop only; the binary stays buildable when the crate is
// built for another target.
#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
fn main() {}
