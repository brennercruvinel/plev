//! A literal written back (task-65 phase c,
//! docs/ADR/view-source-for-pixels.md): the text the person typed over a
//! literal in the window replaces exactly that literal in its file, the
//! bytes of its token and nothing else, escaped for the form it had.
//!
//! Only while the file on disk still has the baseline's code (the skeleton
//! and every literal that is not a string or a color): the running binary
//! was built from that code, and a literal slot means the same literal only
//! while it holds. The slot is found where the click found the literal (the
//! baseline, or the last save that kept the skeleton) and read again on
//! disk; a slot that no longer holds the text is refused too. The file is
//! replaced at once (a dot file beside it, then a rename over it), and the
//! write is applied as the watcher applies a save, so the next frame shows
//! it; the watcher's own event finds the same delta later.
//!
//! The person's alone: an agent's request is refused before anything is
//! read. Every refusal leaves the file untouched.

use std::fmt;
use std::fs;
use std::io::{self, Write as _};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use accord::Actor;
use proc_macro2::{TokenStream, TokenTree};

use super::{BASELINE, Change, EDITED, Lit, Site, apply_change, same_code, snapshot, unquote};

/// Why a literal was not written. The file is as it was in every case.
#[derive(Debug, PartialEq)]
pub enum Refused {
    /// An agent asked: the sources are written by the person alone.
    Agent,
    /// No baseline holds the file: not a workspace source, or the watcher
    /// has not read the sources yet.
    NoBaseline(PathBuf),
    /// The file's code changed since the baseline (the build that runs):
    /// a literal slot may no longer be the same literal. Rebuild first.
    Code(PathBuf),
    /// No string literal holding `text` starts at `site` any more, in the
    /// sources the click read or on disk now.
    Moved { site: Site, text: String },
    /// Reading or replacing the file failed.
    Io(String),
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Agent => write!(f, "an agent may not write the sources"),
            Self::NoBaseline(file) => {
                write!(f, "{}: the hot reload has not read this file", rel(file))
            }
            Self::Code(file) => write!(
                f,
                "{}: its code changed since this build; rebuild, then edit",
                rel(file)
            ),
            Self::Moved { site, text } => {
                write!(f, "{}: no literal there holds {text:?} now", rel_site(site))
            }
            Self::Io(e) => write!(f, "the file could not be written: {e}"),
        }
    }
}

/// `file` from the workspace root, as a person reads it.
fn rel(file: &Path) -> String {
    file.strip_prefix(crate::runtime::hot_reload::project_root())
        .unwrap_or(file)
        .display()
        .to_string()
}

fn rel_site(site: &Site) -> String {
    format!("{}:{}:{}", rel(&site.file), site.line, site.column)
}

/// Write `to` over the string literal holding `from` that starts at
/// `site` (a site [`super::sites_of`] gave), asked `by` an actor. The
/// file's code must still be the baseline's and the literal still there;
/// then exactly its token is replaced, `to` escaped for its form (a raw
/// string stays raw), the file is replaced at once, and the change is
/// applied as a save is. `Ok` carries what the hot reload made of it.
pub fn write(site: &Site, from: &str, to: &str, by: &Actor) -> Result<Change, Refused> {
    if matches!(by, Actor::Agent { .. }) {
        return Err(Refused::Agent);
    }
    let path = site.file.as_path();
    let slot = slot_of(site, from)?;
    let src = fs::read_to_string(path).map_err(|e| Refused::Io(e.to_string()))?;
    let code = || Refused::Code(path.to_path_buf());
    let now = snapshot(&src).ok_or_else(code)?;
    {
        let base = BASELINE.lock().unwrap_or_else(|e| e.into_inner());
        let before = base
            .get(path)
            .ok_or_else(|| Refused::NoBaseline(path.to_path_buf()))?;
        if !same_code(before, &now) {
            return Err(code());
        }
    }
    let moved = || Refused::Moved {
        site: site.clone(),
        text: from.to_owned(),
    };
    if !matches!(now.lits.get(slot), Some(Lit::Str(s)) if s == from) {
        return Err(moved());
    }
    // the token's bytes, checked against the text it must hold
    let range = ranges(&src)
        .and_then(|r| r.get(slot).cloned())
        .filter(|r| src.get(r.clone()).and_then(unquote).as_deref() == Some(from))
        .ok_or_else(moved)?;
    let token = quoted(&src[range.clone()], to);
    let mut out = String::with_capacity(src.len() + token.len());
    out.push_str(&src[..range.start]);
    out.push_str(&token);
    out.push_str(&src[range.end..]);
    replace(path, out.as_bytes()).map_err(|e| Refused::Io(e.to_string()))?;
    Ok(apply_change(path))
}

/// The slot (the index in the file's literal order) of the string literal
/// holding `text` at `site`, in the sources a click reads: the last save
/// that kept the skeleton, else the baseline. Never a literal the app does
/// not draw.
fn slot_of(site: &Site, text: &str) -> Result<usize, Refused> {
    let base = BASELINE.lock().unwrap_or_else(|e| e.into_inner());
    let edited = EDITED.lock().unwrap_or_else(|e| e.into_inner());
    let path = site.file.as_path();
    let snap = edited
        .get(path)
        .or_else(|| base.get(path))
        .ok_or_else(|| Refused::NoBaseline(path.to_path_buf()))?;
    snap.lits
        .iter()
        .zip(&snap.at)
        .position(|(lit, at)| {
            !at.unseen
                && (at.line, at.column) == (site.line, site.column)
                && matches!(lit, Lit::Str(s) if s == text)
        })
        .ok_or_else(|| Refused::Moved {
            site: site.clone(),
            text: text.to_owned(),
        })
}

/// Where each literal of `src` sits, in bytes, in the order the snapshot
/// lists them (the same walk: a group's literals where the group is).
fn ranges(src: &str) -> Option<Vec<Range<usize>>> {
    let lexed = TokenStream::from_str(src).ok().map(|stream| {
        let mut out = Vec::new();
        collect(stream, &mut out);
        out
    });
    // the lexer's copy of the text, as in `snapshot`
    proc_macro2::extra::invalidate_current_thread_spans();
    lexed
}

fn collect(stream: TokenStream, out: &mut Vec<Range<usize>>) {
    for tt in stream {
        match tt {
            TokenTree::Group(g) => collect(g.stream(), out),
            TokenTree::Literal(l) => out.push(l.span().byte_range()),
            TokenTree::Ident(_) | TokenTree::Punct(_) => {}
        }
    }
}

/// `to` as a literal of the form `token` has. A raw string stays raw, with
/// as many `#` as it needs and never fewer than it had (a bare carriage
/// return cannot be raw: it goes plain). A plain string escapes the quote,
/// the backslash and the control characters; every other character,
/// unicode included, is written as it is.
fn quoted(token: &str, to: &str) -> String {
    if let Some(raw) = token.strip_prefix('r')
        && !to.contains('\r')
    {
        let mut hashes = raw.chars().take_while(|c| *c == '#').count();
        while to.contains(&format!("\"{}", "#".repeat(hashes))) {
            hashes += 1;
        }
        let h = "#".repeat(hashes);
        return format!("r{h}\"{to}\"{h}");
    }
    let mut out = String::with_capacity(to.len() + 2);
    out.push('"');
    for c in to.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c if c.is_control() => out.push_str(&format!("\\u{{{:x}}}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// `path` holds `bytes`, at once: written beside it under a dot name (the
/// watcher skips dot files), with the file's permissions, synced, then
/// renamed over it. On a failure the file is as it was and the dot file
/// is gone.
fn replace(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| io::Error::other("the path names no file"))?;
    let beside = path.with_file_name(format!(".{name}.plev-edit"));
    let permissions = fs::metadata(path)?.permissions();
    let written =
        write_beside(&beside, bytes, permissions).and_then(|()| fs::rename(&beside, path));
    if written.is_err() {
        let _ = fs::remove_file(&beside);
    }
    written
}

fn write_beside(beside: &Path, bytes: &[u8], permissions: fs::Permissions) -> io::Result<()> {
    let mut file = fs::File::create(beside)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::set_permissions(beside, permissions)
}

#[cfg(test)]
mod tests;
