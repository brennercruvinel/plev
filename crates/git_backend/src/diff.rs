//! Unified-diff parsing.
//!
//! ADR #5 (pragmatic CLI use): gix's tree/worktree diff API requires manual
//! blob resolution, rename tracking setup and intra-file diffing via
//! `gix-diff` building blocks — disproportionate for "give me the hunks of
//! one file". We shell out to `git diff`/`git show` (stable, porcelain-ish
//! output since forever) and parse the unified format here. This is the
//! same route Zed's `git` crate takes for worktree diffs.

use crate::error::{GitError, Result};
use crate::types::{DiffLine, DiffLineKind, Hunk};

/// Parses `git diff` / `git show --patch` output into hunks.
///
/// When the diff spans multiple files (commit diffs), each hunk header is
/// prefixed with `<path>: ` so flat renderers keep file context.
pub fn parse_unified(text: &str) -> Result<Vec<Hunk>> {
    let mut hunks: Vec<Hunk> = Vec::new();
    let mut current_file: Option<String> = None;
    let mut multi_file = false;
    let mut old_no: u32 = 0;
    let mut new_no: u32 = 0;
    let mut in_hunk = false;
    // Prefix columns before each content line: 1 for an ordinary diff, N for a
    // combined diff of a merge commit (`@@@ -a -b +c @@@` has 2 `-` columns and
    // 2-char line prefixes). Recomputed at every hunk header.
    let mut prefix_width: usize = 1;

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            multi_file = multi_file || current_file.is_some();
            // Prefer the `+++ b/` path (handles renames); fall back to the
            // `b/…` half of this line for binary or deleted files.
            current_file = Some(parse_diff_git_path(rest));
            in_hunk = false;
            continue;
        }
        if let Some(path) = line.strip_prefix("+++ b/") {
            current_file = Some(path.to_string());
            continue;
        }
        if line.starts_with("@@") {
            // Leading `@` run is 2 for an ordinary diff, N+1 for a combined
            // diff of N parents; the content prefix is one column per parent.
            let at_run = line.chars().take_while(|&c| c == '@').count();
            prefix_width = at_run.saturating_sub(1).max(1);
            let header = line.to_string();
            let (old_start, new_start) = parse_hunk_header(&header)?;
            old_no = old_start;
            new_no = new_start;
            in_hunk = true;
            hunks.push(Hunk {
                header,
                lines: Vec::new(),
            });
            continue;
        }
        if !in_hunk {
            continue; // file headers, index lines, mode changes…
        }
        let Some(hunk) = hunks.last_mut() else {
            continue;
        };
        // Combined diffs carry one prefix column per parent; classify by the
        // columns (any `+` → addition, any `-` → removal, else context) and
        // strip all prefix columns from the content.
        if prefix_width > 1 && line.len() >= prefix_width {
            let (cols, content) = line.split_at(prefix_width);
            let (kind, old_d, new_d) = if cols.contains('+') {
                (DiffLineKind::Add, None, Some(new_no))
            } else if cols.contains('-') {
                (DiffLineKind::Remove, Some(old_no), None)
            } else {
                (DiffLineKind::Context, Some(old_no), Some(new_no))
            };
            if old_d.is_some() {
                old_no += 1;
            }
            if new_d.is_some() {
                new_no += 1;
            }
            hunk.lines.push(DiffLine {
                kind,
                content: content.to_string(),
                old_no: old_d,
                new_no: new_d,
            });
        } else if let Some(content) = line.strip_prefix('+') {
            hunk.lines.push(DiffLine {
                kind: DiffLineKind::Add,
                content: content.to_string(),
                old_no: None,
                new_no: Some(new_no),
            });
            new_no += 1;
        } else if let Some(content) = line.strip_prefix('-') {
            hunk.lines.push(DiffLine {
                kind: DiffLineKind::Remove,
                content: content.to_string(),
                old_no: Some(old_no),
                new_no: None,
            });
            old_no += 1;
        } else if let Some(content) = line.strip_prefix(' ') {
            hunk.lines.push(DiffLine {
                kind: DiffLineKind::Context,
                content: content.to_string(),
                old_no: Some(old_no),
                new_no: Some(new_no),
            });
            old_no += 1;
            new_no += 1;
        } else if line.starts_with('\\') {
            // "\ No newline at end of file" — not a content line.
        } else if line.is_empty() {
            // Some git versions emit completely empty context lines.
            hunk.lines.push(DiffLine {
                kind: DiffLineKind::Context,
                content: String::new(),
                old_no: Some(old_no),
                new_no: Some(new_no),
            });
            old_no += 1;
            new_no += 1;
        } else {
            in_hunk = false; // next file section started
        }
    }

    // Prefix headers with the file path only when several files are involved.
    if multi_file {
        // Re-walk to attach paths: cheapest is a second pass over the raw
        // text mirroring the state machine above, but tracking which hunk
        // belongs to which file as we created them in order.
        attach_file_prefixes(text, &mut hunks);
    }
    Ok(hunks)
}

/// Second pass for multi-file diffs: prepend `path: ` to each hunk header
/// in creation order.
fn attach_file_prefixes(text: &str, hunks: &mut [Hunk]) {
    let mut current_file: Option<String> = None;
    let mut idx = 0;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            current_file = Some(parse_diff_git_path(rest));
        } else if let Some(path) = line.strip_prefix("+++ b/") {
            current_file = Some(path.to_string());
        } else if line.starts_with("@@") {
            if let (Some(hunk), Some(file)) = (hunks.get_mut(idx), &current_file) {
                hunk.header = format!("{file}: {}", hunk.header);
            }
            idx += 1;
        }
    }
}

/// Extracts the new-side path from a `diff --git a/old b/new` line.
fn parse_diff_git_path(rest: &str) -> String {
    rest.rsplit(" b/").next().unwrap_or(rest).to_string()
}

/// Parses the start lines from a hunk header into `(old_start, new_start)`.
///
/// Handles both ordinary headers (`@@ -a,b +c,d @@ …`) and combined headers of
/// merge commits (`@@@ -a,b -e,f +c,d @@@ …`): the old start is the first `-`
/// column and the new start is the `+` column, found by scanning tokens rather
/// than fixed positions.
fn parse_hunk_header(header: &str) -> Result<(u32, u32)> {
    let start_of = |token: &str| -> Result<u32> {
        let digits = &token[1..]; // drop the leading sign
        let start = digits.split(',').next().unwrap_or(digits);
        start
            .parse::<u32>()
            .map_err(|_| GitError::Parse(format!("bad hunk header: {header}")))
    };
    let mut old = None;
    let mut new = None;
    for token in header.split_whitespace() {
        // Stop at the closing `@@`/`@@@` so trailing context (e.g. a function
        // signature containing `+`/`-`) is never read as a range.
        if token.starts_with('@') && (old.is_some() || new.is_some()) {
            break;
        }
        if old.is_none() && token.starts_with('-') && token.len() > 1 {
            old = Some(start_of(token)?);
        } else if new.is_none() && token.starts_with('+') && token.len() > 1 {
            new = Some(start_of(token)?);
        }
    }
    match (old, new) {
        (Some(o), Some(n)) => Ok((o, n)),
        _ => Err(GitError::Parse(format!("bad hunk header: {header}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SINGLE_FILE: &str = "\
diff --git a/src/lib.rs b/src/lib.rs
index 1111111..2222222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -10,3 +10,4 @@ fn context_fn() {
 line a
-removed line
+added line
+second added
 line b
";

    #[test]
    fn parses_single_file_hunk() {
        let hunks = parse_unified(SINGLE_FILE).unwrap();
        assert_eq!(hunks.len(), 1);
        let hunk = &hunks[0];
        assert_eq!(hunk.header, "@@ -10,3 +10,4 @@ fn context_fn() {");
        assert_eq!(hunk.lines.len(), 5);

        assert_eq!(hunk.lines[0].kind, DiffLineKind::Context);
        assert_eq!(hunk.lines[0].old_no, Some(10));
        assert_eq!(hunk.lines[0].new_no, Some(10));

        assert_eq!(hunk.lines[1].kind, DiffLineKind::Remove);
        assert_eq!(hunk.lines[1].content, "removed line");
        assert_eq!(hunk.lines[1].old_no, Some(11));
        assert_eq!(hunk.lines[1].new_no, None);

        assert_eq!(hunk.lines[2].kind, DiffLineKind::Add);
        assert_eq!(hunk.lines[2].content, "added line");
        assert_eq!(hunk.lines[2].old_no, None);
        assert_eq!(hunk.lines[2].new_no, Some(11));

        assert_eq!(hunk.lines[3].kind, DiffLineKind::Add);
        assert_eq!(hunk.lines[3].new_no, Some(12));

        assert_eq!(hunk.lines[4].kind, DiffLineKind::Context);
        assert_eq!(hunk.lines[4].old_no, Some(12));
        assert_eq!(hunk.lines[4].new_no, Some(13));
    }

    #[test]
    fn multi_file_headers_get_path_prefix() {
        let text = format!(
            "{SINGLE_FILE}\
diff --git a/README.md b/README.md
index 3333333..4444444 100644
--- a/README.md
+++ b/README.md
@@ -1,1 +1,2 @@
 # title
+new line
"
        );
        let hunks = parse_unified(&text).unwrap();
        assert_eq!(hunks.len(), 2);
        assert!(hunks[0].header.starts_with("src/lib.rs: @@"));
        assert!(hunks[1].header.starts_with("README.md: @@"));
    }

    #[test]
    fn no_newline_marker_is_skipped() {
        let text = "\
diff --git a/f b/f
--- a/f
+++ b/f
@@ -1,1 +1,1 @@
-old
\\ No newline at end of file
+new
\\ No newline at end of file
";
        let hunks = parse_unified(text).unwrap();
        assert_eq!(hunks[0].lines.len(), 2);
        assert_eq!(hunks[0].lines[0].kind, DiffLineKind::Remove);
        assert_eq!(hunks[0].lines[1].kind, DiffLineKind::Add);
    }

    #[test]
    fn empty_input_yields_no_hunks() {
        assert!(parse_unified("").unwrap().is_empty());
    }

    // Combined diff of a merge commit: `git show <merge>` emits `@@@` headers
    // with two `-` columns and 2-char line prefixes. The old parser read the
    // second `-` column as the `+` start and panicked ("bad hunk header"),
    // crashing basicIDE when a merge commit was opened.
    const COMBINED_MERGE: &str = "\
diff --cc src/ui/menu.rs
index aaa,bbb..ccc
--- a/src/ui/menu.rs
+++ b/src/ui/menu.rs
@@@ -202,10 -206,10 +206,10 @@@ impl ContextMenu
  context line
 -removed from parent 1
- removed from parent 2
++added in merge
  trailing context
";

    #[test]
    fn parses_combined_merge_diff_without_panicking() {
        let hunks = parse_unified(COMBINED_MERGE).unwrap();
        assert_eq!(hunks.len(), 1);
        let h = &hunks[0];
        assert!(h.header.contains("@@@ -202,10 -206,10 +206,10 @@@"));
        // old start = first `-` column (202), new start = `+` column (206).
        assert_eq!(h.lines[0].old_no, Some(202));
        assert_eq!(h.lines[0].new_no, Some(206));
        assert_eq!(h.lines[0].kind, DiffLineKind::Context);
        assert_eq!(h.lines[0].content, "context line");
        assert_eq!(h.lines[1].kind, DiffLineKind::Remove);
        assert_eq!(h.lines[2].kind, DiffLineKind::Remove);
        assert_eq!(h.lines[3].kind, DiffLineKind::Add);
        assert_eq!(h.lines[3].content, "added in merge");
        assert_eq!(h.lines[4].kind, DiffLineKind::Context);
    }

    #[test]
    fn hunk_header_with_signs_in_context_is_safe() {
        // The trailing function context contains `+`/`-`; must not be parsed.
        let text = "\
diff --git a/m.rs b/m.rs
--- a/m.rs
+++ b/m.rs
@@ -5,2 +5,2 @@ fn f(a: i32) -> i32 { a + 1 }
 keep
+changed
";
        let hunks = parse_unified(text).unwrap();
        assert_eq!(hunks[0].lines[0].old_no, Some(5));
        assert_eq!(hunks[0].lines[0].new_no, Some(5));
    }
}
