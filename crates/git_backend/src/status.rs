//! `git status --porcelain=v2 -z` parsing.
//!
//! ADR #5 (pragmatic CLI use): gix's `status()` API is still iterator-heavy
//! and rename detection / index-vs-worktree split require nontrivial
//! assembly. Porcelain v2 is an explicitly stable, documented format made
//! for tools, so we parse it instead (the Zed route). `-z` gives
//! NUL-separated records: no quoting issues with spaces or unicode paths.

use crate::error::{GitError, Result};
use crate::types::{FileStatus, StatusKind};

/// Parses NUL-separated `status --porcelain=v2 -z` output.
///
/// A path with both staged (X) and unstaged (Y) changes produces two
/// entries, so callers can show it in both lists like git itself does.
pub fn parse_porcelain_v2(raw: &str) -> Result<Vec<FileStatus>> {
    let mut entries = Vec::new();
    let mut tokens = raw.split('\0').peekable();

    while let Some(record) = tokens.next() {
        if record.is_empty() {
            continue;
        }
        match record.as_bytes()[0] {
            b'1' => {
                // 1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>
                let (xy, path) = split_fields(record, 8)?;
                push_xy(&mut entries, xy, path, path);
            }
            b'2' => {
                // 2 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path>
                // followed by a separate NUL-terminated <origPath> token.
                let (xy, path) = split_fields(record, 9)?;
                let _orig = tokens.next(); // consume original path token
                push_xy(&mut entries, xy, path, path);
            }
            b'?' => {
                let path = record
                    .strip_prefix("? ")
                    .ok_or_else(|| GitError::Parse(format!("bad untracked record: {record}")))?;
                entries.push(FileStatus {
                    path: path.to_string(),
                    status: StatusKind::Untracked,
                    staged: false,
                });
            }
            // 'u' (unmerged) and '!' (ignored) are not surfaced yet; '#'
            // headers only appear with --branch which we don't pass.
            b'u' | b'!' | b'#' => {}
            _ => {
                return Err(GitError::Parse(format!("unknown status record: {record}")));
            }
        }
    }
    Ok(entries)
}

/// Splits a v2 record into its `<XY>` field and the path that starts at
/// field index `path_field` (fields are single-space separated; the path
/// runs to the end of the record).
fn split_fields(record: &str, path_field: usize) -> Result<(&str, &str)> {
    let mut rest = record;
    let mut xy = "";
    for i in 0..path_field {
        let (field, tail) = rest
            .split_once(' ')
            .ok_or_else(|| GitError::Parse(format!("truncated status record: {record}")))?;
        if i == 1 {
            xy = field;
        }
        rest = tail;
    }
    Ok((xy, rest))
}

fn push_xy(entries: &mut Vec<FileStatus>, xy: &str, path: &str, _orig: &str) {
    let mut chars = xy.chars();
    let x = chars.next().unwrap_or('.');
    let y = chars.next().unwrap_or('.');
    if let Some(status) = status_kind(x) {
        entries.push(FileStatus {
            path: path.to_string(),
            status,
            staged: true,
        });
    }
    if let Some(status) = status_kind(y) {
        entries.push(FileStatus {
            path: path.to_string(),
            status,
            staged: false,
        });
    }
}

fn status_kind(c: char) -> Option<StatusKind> {
    match c {
        'M' | 'T' => Some(StatusKind::Modified), // type change shown as modified
        'A' | 'C' => Some(StatusKind::Added),    // copy is a new file for the UI
        'D' => Some(StatusKind::Deleted),
        'R' => Some(StatusKind::Renamed),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_untracked() {
        let raw = "? new file.txt\0";
        let entries = parse_porcelain_v2(raw).unwrap();
        assert_eq!(
            entries,
            vec![FileStatus {
                path: "new file.txt".into(),
                status: StatusKind::Untracked,
                staged: false,
            }]
        );
    }

    #[test]
    fn parses_staged_and_unstaged_sides() {
        // Modified in index AND in worktree -> two entries.
        let raw = "1 MM N... 100644 100644 100644 1111111 2222222 src/a.rs\0";
        let entries = parse_porcelain_v2(raw).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries[0].staged && entries[0].status == StatusKind::Modified);
        assert!(!entries[1].staged && entries[1].status == StatusKind::Modified);
        assert_eq!(entries[0].path, "src/a.rs");
    }

    #[test]
    fn parses_staged_added() {
        let raw = "1 A. N... 000000 100644 100644 0000000 1111111 b.txt\0";
        let entries = parse_porcelain_v2(raw).unwrap();
        assert_eq!(
            entries,
            vec![FileStatus {
                path: "b.txt".into(),
                status: StatusKind::Added,
                staged: true,
            }]
        );
    }

    #[test]
    fn parses_rename_record_and_consumes_orig_path() {
        let raw =
            "2 R. N... 100644 100644 100644 1111111 1111111 R100 new.rs\0old.rs\0? other.txt\0";
        let entries = parse_porcelain_v2(raw).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, "new.rs");
        assert_eq!(entries[0].status, StatusKind::Renamed);
        assert!(entries[0].staged);
        // The orig path token must not be misread as a record.
        assert_eq!(entries[1].path, "other.txt");
    }

    #[test]
    fn empty_output_is_clean() {
        assert!(parse_porcelain_v2("").unwrap().is_empty());
    }
}
