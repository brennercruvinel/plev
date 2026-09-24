//! Literal hot reload: a saved `.rs` whose only difference is in literals
//! (strings, hex colors) shows up in the open window, state intact.
//!
//! The idea is dioxus' rsx hot reload without the macro. At start the
//! watcher snapshots every source file: the token stream with literals
//! blanked (the "skeleton", hashed) plus the literal list in order. On save
//! the file is tokenized again: same skeleton means only literals moved,
//! so `baseline[i] -> current[i]` for each changed slot is the delta. A
//! different skeleton is a code change, which needs the compiler.
//!
//! The delta is applied where every value passes before it becomes pixels:
//! text at `TextNodeKey` and `TextMeasurer` (so layout follows), colors at
//! `Compositor::push_to_layer`. No macro, no marking, any app, any widget.
//!
//! Limits, by construction: a literal that appears twice in the sources
//! and on screen changes in both places (find and replace); a string that
//! is formatted before display never matches; numbers are ambiguous and
//! are treated as code. The baseline is the source at process start, which
//! is the compiled binary when the app was run after the last edit.
//!
//! Each literal also keeps where it sits (line and column, `proc-macro2`'s
//! `span-locations`), so a text in the window leads back to the literal
//! that holds it ([`sites_of`]). A file saved with only literals, comments
//! or whitespace changed keeps its positions as they are on disk now.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex, RwLock};

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use rustc_hash::FxHasher;

use super::Site;
use crate::color::Color;
use crate::compositor::SceneNode;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Lit {
    Str(String),
    /// `0xRRGGBB` or `0xRRGGBBAA` as written, a color when it reaches the
    /// scene through `Color::hex`.
    Hex(u32),
    /// Every other literal (numbers, chars, bytes): tracked so a change to
    /// one is a code change, never applied.
    Other(String),
}

pub(crate) struct Snapshot {
    skeleton: u64,
    lits: Vec<Lit>,
    /// Where each of `lits` sits, in the same order.
    at: Vec<At>,
}

/// Where a literal starts in its file: line and column from 1, the column
/// counted in chars, as `std::panic::Location` counts them. `unseen` marks
/// a literal the app never draws: the text of a doc comment, or one inside
/// an item under `#[cfg(test)]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct At {
    line: u32,
    column: u32,
    unseen: bool,
}

/// What a saved file turned out to be.
#[derive(Debug, PartialEq)]
pub enum Change {
    /// Nothing the compiler cares about changed (comments, whitespace).
    None,
    /// Only literals changed; `n` slots differ from the baseline.
    Literals(usize),
    /// Code changed: needs a rebuild.
    Code,
}

type Deltas = HashMap<PathBuf, Vec<(Lit, Lit)>>;

static BASELINE: LazyLock<Mutex<HashMap<PathBuf, Snapshot>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static DELTAS: LazyLock<Mutex<Deltas>> = LazyLock::new(|| Mutex::new(HashMap::new()));
/// The files a save left with the baseline's skeleton, as they are on disk
/// now: their literals and positions are the ones a click finds.
static EDITED: LazyLock<Mutex<HashMap<PathBuf, Snapshot>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Default)]
struct Overrides {
    text: HashMap<String, String>,
    /// `(from, to)` as `Color::hex` gives them (srgb floats), plus the same
    /// pair linearized: the window clear color travels linear.
    colors: Vec<([f32; 4], [f32; 4])>,
    colors_linear: Vec<([f32; 4], [f32; 4])>,
}

static OVERRIDES: LazyLock<RwLock<Overrides>> = LazyLock::new(|| RwLock::new(Overrides::default()));
/// Fast path for the hot paths: no override, no lock.
static ACTIVE: AtomicBool = AtomicBool::new(false);

// ── snapshot + diff ──

pub(crate) fn snapshot(src: &str) -> Option<Snapshot> {
    let lexed = TokenStream::from_str(src).ok().map(|stream| {
        let mut hasher = FxHasher::default();
        let (mut lits, mut at) = (Vec::new(), Vec::new());
        walk(stream, &mut hasher, &mut lits, &mut at, false);
        Snapshot {
            skeleton: hasher.finish(),
            lits,
            at,
        }
    });
    // With `span-locations` the lexer keeps a copy of every text it reads,
    // failed or not, in a map of this thread for span lookups, and never
    // frees it. The positions are read by now: let it go, or every save
    // would grow the watcher thread by a copy of the file.
    proc_macro2::extra::invalidate_current_thread_spans();
    lexed
}

fn walk(
    stream: TokenStream,
    hasher: &mut FxHasher,
    lits: &mut Vec<Lit>,
    at: &mut Vec<At>,
    test: bool,
) {
    // 1 after `doc`, 2 after `doc =`: a literal then is a doc comment's text
    // (`/// x` lexes as `#[doc = " x"]`).
    let mut after_doc = 0u8;
    // after `#[cfg(test)]`, the next `{ .. }` here is test code (a module,
    // a fn body); an item that ends with `;` first ends it.
    let mut cfg_test = false;
    for tt in stream {
        let next = match &tt {
            TokenTree::Ident(i) if i == "doc" => 1,
            TokenTree::Punct(p) if p.as_char() == '=' && after_doc == 1 => 2,
            _ => 0,
        };
        match tt {
            TokenTree::Ident(i) => i.to_string().hash(hasher),
            TokenTree::Punct(p) => {
                cfg_test &= p.as_char() != ';';
                p.as_char().hash(hasher);
            }
            TokenTree::Group(g) => {
                (g.delimiter() as u8).hash(hasher);
                let body = cfg_test && g.delimiter() == Delimiter::Brace;
                cfg_test = (cfg_test && !body) || is_cfg_test(&g);
                walk(g.stream(), hasher, lits, at, test || body);
            }
            TokenTree::Literal(l) => {
                0xB1u8.hash(hasher);
                lits.push(classify(&l.to_string()));
                let start = l.span().start();
                at.push(At {
                    line: u32::try_from(start.line).unwrap_or(u32::MAX),
                    column: u32::try_from(start.column + 1).unwrap_or(u32::MAX),
                    unseen: test || after_doc == 2,
                });
            }
        }
        after_doc = next;
    }
}

/// `[cfg(test)]`, the group of the attribute.
fn is_cfg_test(g: &Group) -> bool {
    let mut inner = g.stream().into_iter();
    g.delimiter() == Delimiter::Bracket
        && matches!(
            (inner.next(), inner.next(), inner.next()),
            (Some(TokenTree::Ident(cfg)), Some(TokenTree::Group(args)), None)
                if cfg == "cfg" && args.stream().to_string() == "test"
        )
}

fn classify(text: &str) -> Lit {
    if let Some(s) = unquote(text) {
        return Lit::Str(s);
    }
    if let Some(hex) = text.strip_prefix("0x") {
        let digits: String = hex
            .chars()
            .take_while(|c| c.is_ascii_hexdigit() || *c == '_')
            .filter(|c| *c != '_')
            .collect();
        if (6..=8).contains(&digits.len())
            && let Ok(v) = u32::from_str_radix(&digits, 16)
        {
            return Lit::Hex(v);
        }
    }
    Lit::Other(text.to_string())
}

/// The value of a `"..."` or `r#"..."#` literal, `None` for anything else.
fn unquote(text: &str) -> Option<String> {
    if let Some(body) = text.strip_prefix('"') {
        return Some(unescape(body.strip_suffix('"')?));
    }
    let raw = text.strip_prefix('r')?;
    let hashes = raw.chars().take_while(|c| *c == '#').count();
    let body = raw[hashes..].strip_suffix(&"#".repeat(hashes))?;
    let body = body.strip_prefix('"')?.strip_suffix('"')?;
    Some(body.to_string())
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('0') => out.push('\0'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('u') => {
                let hex: String = chars
                    .by_ref()
                    .skip_while(|c| *c == '{')
                    .take_while(|c| *c != '}')
                    .collect();
                if let Some(ch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                    out.push(ch);
                }
            }
            // `\` + newline: the line continuation swallows leading blanks.
            Some('\n') => {
                while chars.peek().is_some_and(|c| c.is_whitespace()) {
                    chars.next();
                }
            }
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Take the baseline of every `.rs` under `dirs`. Runs once, on the
/// watcher thread, before the first change can arrive in practice.
pub fn take_baseline(dirs: &[PathBuf]) {
    let mut files = Vec::new();
    for dir in dirs {
        collect_rs(dir, &mut files);
    }
    let mut base = BASELINE.lock().unwrap_or_else(|e| e.into_inner());
    for path in files {
        if let Ok(src) = std::fs::read_to_string(&path)
            && let Some(snap) = snapshot(&src)
        {
            base.insert(path, snap);
        }
    }
    log::info!("literal hot reload on: {} files", base.len());
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Classify a saved file against its baseline and, for a literal-only
/// change, refresh the overrides.
pub fn apply_change(path: &Path) -> Change {
    let Ok(src) = std::fs::read_to_string(path) else {
        return Change::Code;
    };
    let Some(now) = snapshot(&src) else {
        return Change::Code;
    };
    let mut base = BASELINE.lock().unwrap_or_else(|e| e.into_inner());
    let Some(before) = base.get(path) else {
        // A new file: it becomes its own baseline, and the compiler owns it.
        base.insert(path.to_path_buf(), now);
        return Change::Code;
    };
    if before.skeleton != now.skeleton || before.lits.len() != now.lits.len() {
        return Change::Code;
    }
    let delta: Vec<(Lit, Lit)> = before
        .lits
        .iter()
        .zip(&now.lits)
        .filter(|(a, b)| a != b)
        .map(|(a, b)| (a.clone(), b.clone()))
        .collect();
    if delta.iter().any(|(a, _)| matches!(a, Lit::Other(_))) {
        return Change::Code;
    }
    let n = delta.len();
    let mut deltas = DELTAS.lock().unwrap_or_else(|e| e.into_inner());
    let was = deltas.get(path).map_or(0, Vec::len);
    if n == 0 {
        deltas.remove(path);
    } else {
        deltas.insert(path.to_path_buf(), delta);
    }
    rebuild_overrides(&deltas);
    EDITED
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(path.to_path_buf(), now);
    if n == 0 && was == 0 {
        Change::None
    } else {
        Change::Literals(n)
    }
}

/// Every string literal whose value is `text`, ordered by file, line and
/// column: the sources as the baseline read them, or as the last save that
/// kept a file's skeleton left it. Literals the app never draws are left
/// out: doc comments, items under `#[cfg(test)]`, and test files (the
/// file hygiene guard's rule: `tests.rs`, `tests_*.rs`, a `tests` or
/// `benches` directory). A test module declared out of line under another
/// name still counts. Empty when no literal holds the text (formatted
/// before display) or before the baseline is taken.
pub fn sites_of(text: &str) -> Vec<Site> {
    let base = BASELINE.lock().unwrap_or_else(|e| e.into_inner());
    let edited = EDITED.lock().unwrap_or_else(|e| e.into_inner());
    let mut sites = Vec::new();
    for (path, snap) in base.iter() {
        edited
            .get(path)
            .unwrap_or(snap)
            .find(path, text, &mut sites);
    }
    sites.sort_by(|a, b| (&a.file, a.line, a.column).cmp(&(&b.file, b.line, b.column)));
    sites
}

impl Snapshot {
    /// The string literals of this file whose value is `text`, none in a
    /// test file.
    fn find(&self, path: &Path, text: &str, out: &mut Vec<Site>) {
        if is_test_file(path) {
            return;
        }
        for (lit, at) in self.lits.iter().zip(&self.at) {
            if !at.unseen && matches!(lit, Lit::Str(s) if s == text) {
                out.push(Site {
                    file: path.to_path_buf(),
                    line: at.line,
                    column: at.column,
                });
            }
        }
    }
}

fn rebuild_overrides(deltas: &Deltas) {
    let mut over = Overrides::default();
    for (from, to) in deltas.values().flatten() {
        match (from, to) {
            (Lit::Str(a), Lit::Str(b)) => {
                over.text.insert(a.clone(), b.clone());
            }
            (Lit::Hex(a), Lit::Hex(b)) => {
                let (a, b) = (Color::hex(*a), Color::hex(*b));
                over.colors.push((a.0, b.0));
                over.colors_linear
                    .push((a.to_linear_array(), b.to_linear_array()));
            }
            _ => {}
        }
    }
    let active = !over.text.is_empty() || !over.colors.is_empty();
    *OVERRIDES.write().unwrap_or_else(|e| e.into_inner()) = over;
    ACTIVE.store(active, Ordering::Release);
}

/// Test code by where it lives, under the workspace root.
fn is_test_file(path: &Path) -> bool {
    let rel = path.strip_prefix(super::project_root()).unwrap_or(path);
    let stem = rel.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    stem == "tests"
        || stem.starts_with("tests_")
        || rel
            .components()
            .any(|c| matches!(c.as_os_str().to_str(), Some("tests" | "benches")))
}

// ── apply ──

/// The current value of a displayed string, or itself.
pub fn text(s: &str) -> Option<String> {
    if !ACTIVE.load(Ordering::Acquire) {
        return None;
    }
    OVERRIDES
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .text
        .get(s)
        .cloned()
}

/// rgb from the literal, alpha as the widget set it (`with_alpha` on a
/// token is still that token).
fn swap_in(table: &[([f32; 4], [f32; 4])], c: &mut [f32; 4]) {
    if let Some((_, to)) = table.iter().find(|(from, _)| same_rgb(from, c)) {
        c[..3].copy_from_slice(&to[..3]);
    }
}

/// A color that already left the scene (the window clear), linear.
pub fn linear(mut c: [f32; 4]) -> [f32; 4] {
    if ACTIVE.load(Ordering::Acquire) {
        let over = OVERRIDES.read().unwrap_or_else(|e| e.into_inner());
        swap_in(&over.colors_linear, &mut c);
    }
    c
}

/// A scene node with every overridden color swapped.
pub fn recolor(mut node: SceneNode) -> SceneNode {
    if !ACTIVE.load(Ordering::Acquire) {
        return node;
    }
    let over = OVERRIDES.read().unwrap_or_else(|e| e.into_inner());
    if over.colors.is_empty() {
        return node;
    }
    let swap = |c: &mut [f32; 4]| swap_in(&over.colors, c);
    match &mut node {
        SceneNode::Rect { color, .. } | SceneNode::Text { color, .. } => swap(color),
        // icons: the color is baked per vertex at tessellation
        SceneNode::Path { data } => {
            for v in &mut data.vertices {
                swap(&mut v.color);
            }
        }
        SceneNode::RoundedRect {
            color,
            border_color,
            ..
        } => {
            swap(color);
            swap(border_color);
        }
        SceneNode::GradientRect {
            color,
            color2,
            border_color,
            ..
        } => {
            swap(color);
            swap(color2);
            swap(border_color);
        }
        _ => {}
    }
    node
}

fn same_rgb(a: &[f32; 4], b: &[f32; 4]) -> bool {
    // half a step of 8-bit, the precision a hex literal carries
    a.iter()
        .zip(b)
        .take(3)
        .all(|(x, y)| (x - y).abs() < 1.0 / 512.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const OLD: &str = r#"
        /// a page
        pub const PAGE: Page = Page::new("Theme", "sun", "Click a card to apply it.");
        pub const N2: Color = Color::hex(0xf8f8f8);
        fn size() -> f32 { 16.0 }
    "#;

    fn delta(old: &str, new: &str) -> Result<Vec<(Lit, Lit)>, &'static str> {
        let (a, b) = (snapshot(old).unwrap(), snapshot(new).unwrap());
        if a.skeleton != b.skeleton || a.lits.len() != b.lits.len() {
            return Err("code");
        }
        Ok(a.lits
            .into_iter()
            .zip(b.lits)
            .filter(|(x, y)| x != y)
            .collect())
    }

    #[test]
    fn string_and_hex_are_literals_numbers_are_other() {
        let snap = snapshot(OLD).unwrap();
        assert_eq!(
            snap.lits[0],
            Lit::Str(" a page".into()),
            "doc comment is a string literal"
        );
        assert_eq!(snap.lits[1], Lit::Str("Theme".into()));
        assert_eq!(snap.lits[3], Lit::Str("Click a card to apply it.".into()));
        assert_eq!(snap.lits[4], Lit::Hex(0xf8f8f8));
        assert_eq!(snap.lits[5], Lit::Other("16.0".into()));
    }

    #[test]
    fn a_changed_string_is_one_literal_delta() {
        let new = OLD.replace("Click a card to apply it.", "testeeeeee");
        let d = delta(OLD, &new).unwrap();
        assert_eq!(
            d,
            vec![(
                Lit::Str("Click a card to apply it.".into()),
                Lit::Str("testeeeeee".into())
            )]
        );
    }

    #[test]
    fn a_changed_color_is_a_hex_delta() {
        let new = OLD.replace("0xf8f8f8", "0xff0000");
        let d = delta(OLD, &new).unwrap();
        assert_eq!(d, vec![(Lit::Hex(0xf8f8f8), Lit::Hex(0xff0000))]);
    }

    #[test]
    fn whitespace_and_comments_are_no_change() {
        let new = OLD.replace("fn size()", "fn size(  ) /* spaced */");
        assert!(delta(OLD, &new).unwrap().is_empty());
    }

    #[test]
    fn code_changes_are_code() {
        assert_eq!(
            delta(OLD, &OLD.replace("fn size()", "fn width()")),
            Err("code")
        );
        assert_eq!(
            delta(OLD, &OLD.replace("16.0 }", "16.0 + 1.0 }")),
            Err("code")
        );
        assert_eq!(
            delta(OLD, &OLD.replace("Color::hex(0xf8f8f8);", "")),
            Err("code")
        );
    }

    #[test]
    fn escapes_and_raw_strings_unquote() {
        assert_eq!(unquote(r#""a\"b\n\u{e9}""#).unwrap(), "a\"b\né");
        assert_eq!(unquote(r##"r#"raw "x""#"##).unwrap(), r#"raw "x""#);
        assert_eq!(unquote("'c'"), None);
    }

    /// Line and column from 1, in chars, of the first `needle` in `src`.
    fn position(src: &str, needle: &str) -> (u32, u32) {
        let (i, line) = src
            .lines()
            .enumerate()
            .find(|(_, l)| l.contains(needle))
            .unwrap();
        let byte = line.find(needle).unwrap();
        let column = line[..byte].chars().count() + 1;
        (i as u32 + 1, column as u32)
    }

    #[test]
    fn each_literal_keeps_its_line_and_column() {
        let snap = snapshot(OLD).unwrap();
        assert_eq!(snap.at.len(), snap.lits.len());
        let (line, column) = position(OLD, "\"Theme\"");
        assert_eq!((snap.at[1].line, snap.at[1].column), (line, column));
        let (line, column) = position(OLD, "0xf8f8f8");
        assert_eq!((snap.at[4].line, snap.at[4].column), (line, column));
        assert!(snap.at[0].unseen, "the doc comment is marked");
        assert!(!snap.at[1].unseen && !snap.at[4].unseen);
    }

    #[test]
    fn literals_in_test_code_are_unseen() {
        let src = r#"
            const A: &str = "shown";
            #[cfg(test)]
            mod tests {
                const B: &str = "in a test module";
            }
            #[cfg(test)]
            pub(crate) fn helper(x: u32) -> Vec<&'static str> { vec!["in a test fn"] }
            #[cfg(test)]
            mod out_of_line;
            #[cfg(not(test))]
            fn g() -> &'static str { "not test" }
            fn f() -> &'static str { "in a fn" }
        "#;
        let snap = snapshot(src).unwrap();
        let marks: Vec<(String, bool)> = snap
            .lits
            .iter()
            .zip(&snap.at)
            .map(|(lit, at)| match lit {
                Lit::Str(s) => (s.clone(), at.unseen),
                other => panic!("{other:?}"),
            })
            .collect();
        let want = [
            ("shown", false),
            ("in a test module", true),
            ("in a test fn", true),
            ("not test", false),
            ("in a fn", false),
        ];
        assert_eq!(marks, want.map(|(s, u)| (s.to_owned(), u)));

        let root = super::super::project_root();
        for (rel, test) in [
            ("crates/a/src/tests.rs", true),
            ("crates/a/src/b/tests_scales.rs", true),
            ("crates/a/src/tests/widgets/mod.rs", true),
            ("crates/a/src/view/apps/dock/tests.rs", true),
            ("crates/a/src/view/latest.rs", false),
            ("crates/a/src/contests.rs", false),
        ] {
            assert_eq!(is_test_file(&root.join(rel)), test, "{rel}");
        }
    }

    #[test]
    fn the_column_counts_chars_and_a_second_file_starts_over() {
        snapshot(OLD).unwrap();
        let src = "fn f() { let a = \"é→\"; let b = \"x\"; }";
        let snap = snapshot(src).unwrap();
        assert_eq!(snap.lits[1], Lit::Str("x".into()));
        assert_eq!(
            (snap.at[1].line, snap.at[1].column),
            (1, 32),
            "chars, not bytes (é and → are 5 bytes, 2 chars)"
        );
    }

    #[test]
    fn a_text_finds_its_literals_and_follows_a_save() {
        let dir = std::env::temp_dir().join("plev-literals-sites-test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("page.rs");
        let old = "/// sites-probe-title\n\
                   const A: &str = \"sites-probe-title\"; const B: &str = \"sites-probe-b\";\n\
                   const C: &str = \"sites-probe-title\";\n";
        std::fs::write(&file, old).unwrap();
        take_baseline(std::slice::from_ref(&dir));

        let found = sites_of("sites-probe-title");
        assert_eq!(
            found.iter().map(|s| (s.line, s.column)).collect::<Vec<_>>(),
            vec![(2, 17), (3, 17)],
            "both literals, in order; the doc comment with the same text is left out"
        );
        assert!(found.iter().all(|s| s.file == file));
        let b = |src: &str| position(src, "\"sites-probe-b\"").1;
        assert_eq!(sites_of("sites-probe-b")[0].column, b(old));
        assert!(sites_of("sites-probe-nowhere").is_empty());

        // a longer literal on the same line moves the one after it
        let new = old.replacen(
            "= \"sites-probe-title\";",
            "= \"sites-probe-title, longer\";",
            1,
        );
        std::fs::write(&file, &new).unwrap();
        assert_eq!(apply_change(&file), Change::Literals(1));
        assert_eq!(sites_of("sites-probe-b")[0].column, b(old) + 8);
        assert_eq!(sites_of("sites-probe-b")[0].column, b(&new));
        let now = sites_of("sites-probe-title, longer");
        assert_eq!((now.len(), now[0].line, now[0].column), (1, 2, 17));
        assert_eq!(
            sites_of("sites-probe-title").len(),
            1,
            "only the third line"
        );

        std::fs::write(&file, old).unwrap();
        assert_eq!(apply_change(&file), Change::Literals(0));
        assert_eq!(sites_of("sites-probe-b")[0].column, b(old));
    }

    #[test]
    fn overrides_apply_to_text_and_color_and_revert() {
        let dir = std::env::temp_dir().join("plev-literals-test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("page.rs");
        std::fs::write(&file, OLD).unwrap();
        take_baseline(std::slice::from_ref(&dir));

        std::fs::write(
            &file,
            OLD.replace("Click a card to apply it.", "testeeeeee")
                .replace("0xf8f8f8", "0xff0000"),
        )
        .unwrap();
        assert_eq!(apply_change(&file), Change::Literals(2));
        assert_eq!(
            text("Click a card to apply it.").as_deref(),
            Some("testeeeeee")
        );
        let node = recolor(SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            color: [
                0xf8 as f32 / 255.0,
                0xf8 as f32 / 255.0,
                0xf8 as f32 / 255.0,
                0.5,
            ],
        });
        let SceneNode::Rect { color, .. } = node else {
            panic!()
        };
        assert_eq!(
            color,
            [1.0, 0.0, 0.0, 0.5],
            "rgb from the literal, alpha kept"
        );

        std::fs::write(&file, OLD).unwrap();
        assert_eq!(apply_change(&file), Change::Literals(0));
        assert_eq!(text("Click a card to apply it."), None);
        assert_eq!(apply_change(&file), Change::None);
    }
}
