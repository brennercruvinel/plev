//! where a port listens: `$TMPDIR/plev/<app>-<pid>.sock`, in a directory
//! only this user can enter (0700), the socket itself 0600. and how a
//! client finds it by app name.

use std::fs::{self, DirBuilder, Permissions};
use std::io::ErrorKind;
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, PermissionsExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use crate::error::WireError;

/// the longest app name, in characters.
pub const MAX_APP: usize = 32;

/// `$TMPDIR/plev` (the system temporary directory when `TMPDIR` is unset).
pub fn socket_dir() -> PathBuf {
    std::env::temp_dir().join("plev")
}

/// the socket of `app` in process `pid`, under `dir`.
pub fn socket_path(dir: &Path, app: &str, pid: u32) -> PathBuf {
    dir.join(format!("{app}-{pid}.sock"))
}

/// an app name names a file: 1 to 32 characters of `[A-Za-z0-9_-]`.
pub fn check_app(app: &str) -> Result<(), WireError> {
    let fits = (1..=MAX_APP).contains(&app.chars().count())
        && app
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if fits {
        Ok(())
    } else {
        Err(WireError::BadApp(app.to_owned()))
    }
}

/// makes `dir` a directory only this user enters: created 0700 when
/// missing, tightened to 0700 when it exists. a symlink, a file, or a
/// directory this user cannot chmod (another user's) is refused.
pub(crate) fn private_dir(dir: &Path) -> Result<(), WireError> {
    match fs::symlink_metadata(dir) {
        Ok(meta) if meta.file_type().is_symlink() || !meta.is_dir() => {
            return Err(WireError::NotPrivate(dir.to_owned()));
        }
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::NotFound => {
            DirBuilder::new()
                .mode(0o700)
                .recursive(true)
                .create(dir)
                .map_err(|e| io(format!("create {}", dir.display()), e))?;
        }
        Err(e) => return Err(io(format!("inspect {}", dir.display()), e)),
    }
    // only the owner may chmod: a directory someone else planted fails
    // here.
    fs::set_permissions(dir, Permissions::from_mode(0o700))
        .map_err(|_| WireError::NotPrivate(dir.to_owned()))?;
    let meta =
        fs::symlink_metadata(dir).map_err(|e| io(format!("inspect {}", dir.display()), e))?;
    if meta.file_type().is_symlink() || meta.permissions().mode() & 0o777 != 0o700 {
        return Err(WireError::NotPrivate(dir.to_owned()));
    }
    Ok(())
}

/// clears a socket left at `path` by a process that ended (its pid is
/// ours now, so it cannot be alive). anything else there is left alone.
pub(crate) fn clear_stale(path: &Path) -> Result<(), WireError> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_socket() => {
            fs::remove_file(path).map_err(|e| io(format!("remove {}", path.display()), e))
        }
        Ok(_) => Err(WireError::NotASocket(path.to_owned())),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(io(format!("inspect {}", path.display()), e)),
    }
}

/// the socket of the one live app in `dir`, named `app` when given. a
/// socket nobody answers on (its process ended) does not count.
pub fn find(dir: &Path, app: Option<&str>) -> Result<PathBuf, WireError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Err(WireError::NoApp(app.map(str::to_owned)));
        }
        Err(e) => return Err(io(format!("read {}", dir.display()), e)),
    };
    let mut found: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| match (parse(path), app) {
            (Some((name, _)), Some(want)) => name == want,
            (Some(_), None) => true,
            (None, _) => false,
        })
        .filter(|path| UnixStream::connect(path).is_ok())
        .collect();
    found.sort();
    match found.len() {
        0 => Err(WireError::NoApp(app.map(str::to_owned))),
        1 => Ok(found.remove(0)),
        _ => Err(WireError::Ambiguous {
            name: app.map(str::to_owned),
            found,
        }),
    }
}

/// `<app>-<pid>.sock` read back.
fn parse(path: &Path) -> Option<(String, u32)> {
    let name = path.file_name()?.to_str()?.strip_suffix(".sock")?;
    let (app, pid) = name.rsplit_once('-')?;
    check_app(app).ok()?;
    Some((app.to_owned(), pid.parse().ok()?))
}

pub(crate) fn io(what: impl Into<String>, source: std::io::Error) -> WireError {
    WireError::Io {
        what: what.into(),
        source,
    }
}
