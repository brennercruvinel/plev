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

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex, RwLock};

use proc_macro2::{TokenStream, TokenTree};
use rustc_hash::FxHasher;

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
    let stream = TokenStream::from_str(src).ok()?;
    let mut hasher = FxHasher::default();
    let mut lits = Vec::new();
    walk(stream, &mut hasher, &mut lits);
    Some(Snapshot {
        skeleton: hasher.finish(),
        lits,
    })
}

fn walk(stream: TokenStream, hasher: &mut FxHasher, lits: &mut Vec<Lit>) {
    for tt in stream {
        match tt {
            TokenTree::Ident(i) => i.to_string().hash(hasher),
            TokenTree::Punct(p) => p.as_char().hash(hasher),
            TokenTree::Group(g) => {
                (g.delimiter() as u8).hash(hasher);
                walk(g.stream(), hasher, lits);
            }
            TokenTree::Literal(l) => {
                0xB1u8.hash(hasher);
                lits.push(classify(&l.to_string()));
            }
        }
    }
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
    if n == 0 && was == 0 {
        Change::None
    } else {
        Change::Literals(n)
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
