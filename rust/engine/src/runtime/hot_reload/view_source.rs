//! View source for pixels (docs/ADR/view-source-for-pixels.md): a point in
//! the window leads to the code that made what shows there. A bound
//! control (`Compositor::hit`) leads to where its node was projected
//! (`sem::Node::source`); a text with no id over it leads to the string
//! literals that hold it (`literals::sites_of`). The shell asks
//! [`origin_at`] on a cmd+click and opens the one site it names with
//! [`open`]; nothing here draws or changes what the window shows.

use std::fmt;
use std::path::PathBuf;
use std::process::Command;

use accord::{SemId, Snapshot};

use super::{Site, literals, project_root};
use crate::drawing::compositor::{ClipRect, Compositor, SceneNode, TextNodeKey, intersect_rects};
use crate::drawing::text::{TextMeasurer, TextStyle};

/// What made the pixel at a point.
#[derive(Debug, PartialEq)]
pub enum Origin {
    /// A bound control, and where its node was projected.
    Projected { id: SemId, site: Site },
    /// A bound control the snapshot does not hold, or holds with no source
    /// (no snapshot at all: the page projects nothing yet).
    Unsourced(SemId),
    /// A text with no id over it, and every string literal that holds it
    /// (one or more, in file, line and column order).
    Literal { text: String, sites: Vec<Site> },
    /// A text with no id over it that no literal holds: formatted before
    /// it was drawn.
    Formatted(String),
    /// No bound control and no text at the point.
    Nothing,
}

impl Origin {
    /// The one site a click opens: a projected control, or a text exactly
    /// one literal holds. Several literals are listed, never guessed.
    pub fn site(&self) -> Option<&Site> {
        match self {
            Self::Projected { site, .. } => Some(site),
            Self::Literal { sites, .. } if sites.len() == 1 => sites.first(),
            _ => None,
        }
    }
}

/// The line the shell logs: what was found and where it leads.
impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Projected { id, site } => write!(f, "{id}: projected at {site}"),
            Self::Unsourced(id) => write!(f, "{id}: the projection has no source for it"),
            Self::Literal { text, sites } if sites.len() == 1 => {
                write!(f, "{text:?}: the literal at {}", sites[0])
            }
            Self::Literal { text, sites } => {
                write!(
                    f,
                    "{text:?}: {} literals hold it, none opened:",
                    sites.len()
                )?;
                sites.iter().try_for_each(|s| write!(f, "\n  {s}"))
            }
            Self::Formatted(text) => {
                write!(
                    f,
                    "{text:?}: no literal holds it (formatted before it was drawn)"
                )
            }
            Self::Nothing => write!(f, "no bound control and no text here"),
        }
    }
}

/// What made the pixel at `(x, y)`, in logical window pixels, in the frame
/// on screen: the bound control there and its projection site in
/// `snapshot`, else the topmost text there and the literals that hold it.
pub fn origin_at(compositor: &Compositor, snapshot: Option<&Snapshot>, x: f32, y: f32) -> Origin {
    if let Some(id) = compositor.hit(x, y) {
        let node = snapshot.and_then(|s| s.get(id));
        return match node.and_then(|n| n.source.as_ref()) {
            Some(source) => Origin::Projected {
                id: id.clone(),
                site: Site {
                    // relative to the workspace root, as `file!()` is in a
                    // member; an absolute path (a registry crate) stays.
                    file: project_root().join(&*source.file),
                    line: source.line,
                    column: source.column,
                },
            },
            None => Origin::Unsourced(id.clone()),
        };
    }
    let Some(text) = text_at(compositor, x, y) else {
        return Origin::Nothing;
    };
    let sites = literals::sites_of(&text);
    if sites.is_empty() {
        Origin::Formatted(text)
    } else {
        Origin::Literal { text, sites }
    }
}

/// The text drawn topmost at `(x, y)`: the last text run whose box (where
/// it is drawn, its measured size, inside its clip) holds the point, on a
/// visible layer, layers stacked as `Compositor::hit` stacks them (z-order,
/// then creation). The text as drawn, a hot reloaded literal included.
pub fn text_at(compositor: &Compositor, x: f32, y: f32) -> Option<String> {
    text_run_at(compositor, x, y).map(|(text, _)| text)
}

/// [`text_at`] with the run's box, `[x, y, w, h]` in logical window
/// pixels: where a field that edits the text goes.
pub fn text_run_at(compositor: &Compositor, x: f32, y: f32) -> Option<(String, [f32; 4])> {
    let mut layers: Vec<_> = compositor
        .layers()
        .iter()
        .enumerate()
        .filter(|(_, l)| l.visible && l.opacity > 0.0)
        .collect();
    layers.sort_by_key(|(rank, l)| (l.z_order, *rank));
    let mut top = None;
    for (_, layer) in layers {
        let mut clips: Vec<ClipRect> = Vec::new();
        for node in layer.nodes() {
            match node {
                SceneNode::PushClip { x, y, w, h } => {
                    let rect = [*x, *y, *w, *h];
                    clips.push(clips.last().map_or(rect, |c| intersect_rects(*c, rect)));
                }
                SceneNode::PopClip => {
                    clips.pop();
                }
                SceneNode::Text {
                    key, x: tx, y: ty, ..
                } => {
                    let shows = clips.last().is_none_or(|c| contains(*c, x, y));
                    let run = text_box(key, *tx, *ty);
                    if shows && contains(run, x, y) {
                        top = Some((&key.text, run));
                    }
                }
                _ => {}
            }
        }
    }
    top.map(|(text, run)| (text.clone(), run))
}

/// Where a text run sits: drawn from `(x, y)` down and right, as wide and
/// tall as it measures with the style its key carries.
fn text_box(key: &TextNodeKey, x: f32, y: f32) -> [f32; 4] {
    let style = TextStyle {
        font_size: f32::from_bits(key.font_size_bits),
        line_height: f32::from_bits(key.line_height_bits),
        font_weight: key.font_weight,
        letter_spacing: f32::from_bits(key.letter_spacing_bits),
        font_family: key.font_family.clone(),
    };
    let max_width = key.max_width_bits.map(f32::from_bits);
    let (w, h) = TextMeasurer::measure_styled(&key.text, &style, max_width);
    [x, y, w, h]
}

fn contains([rx, ry, rw, rh]: [f32; 4], x: f32, y: f32) -> bool {
    x >= rx && x < rx + rw && y >= ry && y < ry + rh
}

/// The command that opens `site` in the developer's editor, the file as
/// `file:line:column` in its last argument:
///
/// - `plev_editor` (`$PLEV_EDITOR`) when set: its words are the program and
///   its first arguments (`PLEV_EDITOR="subl -n"` runs `subl -n
///   file:line:column`); a program path with a space in it is not supported;
/// - else `zed file:line:column`, when `zed` is on `path`;
/// - else `code -g file:line:column`, when `code` is on `path`.
///
/// `None` when none of them is there.
pub fn editor_command(
    site: &Site,
    plev_editor: Option<&str>,
    path: Option<&str>,
) -> Option<Command> {
    let at = site.to_string();
    if let Some(mut words) = plev_editor.map(str::split_whitespace)
        && let Some(program) = words.next()
    {
        let mut cmd = Command::new(program);
        cmd.args(words).arg(at);
        return Some(cmd);
    }
    let path = path?;
    if let Some(zed) = on_path("zed", path) {
        let mut cmd = Command::new(zed);
        cmd.arg(at);
        return Some(cmd);
    }
    let code = on_path("code", path)?;
    let mut cmd = Command::new(code);
    cmd.arg("-g").arg(at);
    Some(cmd)
}

/// The first executable file named `name` in the directories of `path`.
fn on_path(name: &str, path: &str) -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    std::env::split_paths(path)
        .map(|dir| dir.join(name))
        .find(|p| {
            std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
}

/// Open `site` in the editor [`editor_command`] picks from the environment
/// (`PLEV_EDITOR`, `PATH`), without waiting for it: a thread reaps the
/// child. The command line that ran, for the log.
pub fn open(site: &Site) -> std::io::Result<String> {
    let editor = std::env::var("PLEV_EDITOR").ok();
    let path = std::env::var("PATH").ok();
    open_with(site, editor.as_deref(), path.as_deref())
}

/// [`open`] with `PLEV_EDITOR` and `PATH` given, not read.
pub fn open_with(
    site: &Site,
    plev_editor: Option<&str>,
    path: Option<&str>,
) -> std::io::Result<String> {
    let Some(mut cmd) = editor_command(site, plev_editor, path) else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "no editor: set PLEV_EDITOR, or put zed or code on the PATH",
        ));
    };
    let line = command_line(&cmd);
    let mut child = cmd.spawn()?;
    std::thread::spawn(move || child.wait());
    Ok(line)
}

/// `program arg arg`, as the log shows a command.
fn command_line(cmd: &Command) -> String {
    std::iter::once(cmd.get_program())
        .chain(cmd.get_args())
        .map(|a| a.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use accord::{Role, SnapshotBuilder};

    use super::*;
    use crate::drawing::compositor::LayerId;

    /// The sources the text tests find their literals in.
    const PAGE: &str = "const A: &str = \"view-source-probe-once\";\n\
                        const B: &str = \"view-source-probe-twice\";\n\
                        const C: &str = \"view-source-probe-twice\";\n";

    fn baseline() -> PathBuf {
        static TAKEN: Once = Once::new();
        let dir = std::env::temp_dir().join("plev-view-source-test");
        TAKEN.call_once(|| {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("page.rs"), PAGE).unwrap();
            literals::take_baseline(std::slice::from_ref(&dir));
        });
        dir.join("page.rs")
    }

    fn id(text: &str) -> SemId {
        SemId::parse(text).unwrap()
    }

    fn text(c: &mut Compositor, layer: LayerId, s: &str, x: f32, y: f32) {
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::new(s, 14.0, 18.0, None),
                x,
                y,
                color: [0.0, 0.0, 0.0, 1.0],
            },
        );
    }

    /// The text of `site`'s file from its line and column on.
    fn at(site: &Site) -> String {
        let src = std::fs::read_to_string(&site.file).unwrap();
        let line = src.lines().nth(site.line as usize - 1).unwrap();
        line.chars().skip(site.column as usize - 1).collect()
    }

    #[test]
    fn a_bound_control_leads_to_where_it_was_projected() {
        let add = id("todo.add");
        let mut c = Compositor::new();
        c.begin_frame();
        c.bind(&add, [10.0, 10.0, 80.0, 30.0]);
        let mut b = SnapshotBuilder::new();
        let line = line!() + 1;
        b.node(&add, Role::Button, "Add");
        let snap = b.finish().unwrap();

        let origin = origin_at(&c, Some(&snap), 50.0, 25.0);
        let site = origin.site().expect("a projected control opens").clone();
        assert_eq!(
            origin,
            Origin::Projected {
                id: add,
                site: site.clone()
            }
        );
        assert_eq!(site.file, project_root().join(file!()));
        assert!(site.file.is_absolute() && site.file.is_file());
        assert_eq!(site.line, line);
        assert!(
            at(&site).starts_with("node(&add, Role::Button"),
            "{}",
            at(&site)
        );
        assert!(
            origin
                .to_string()
                .ends_with(&format!("view_source.rs:{line}:{}", site.column))
        );
    }

    #[test]
    fn a_bound_control_the_snapshot_does_not_source_is_said_so() {
        let add = id("todo.add");
        let mut c = Compositor::new();
        c.begin_frame();
        c.bind(&add, [10.0, 10.0, 80.0, 30.0]);
        let other = SnapshotBuilder::new().finish().unwrap();
        for snap in [None, Some(&other)] {
            let origin = origin_at(&c, snap, 50.0, 25.0);
            assert_eq!(origin, Origin::Unsourced(add.clone()));
            assert_eq!(origin.site(), None);
        }
        assert_eq!(origin_at(&c, None, 200.0, 200.0), Origin::Nothing);
        assert_eq!(origin_at(&c, None, 200.0, 200.0).site(), None);
    }

    #[test]
    fn a_text_leads_to_the_one_literal_that_holds_it() {
        let file = baseline();
        let mut c = Compositor::new();
        c.begin_frame();
        text(
            &mut c,
            LayerId::DEFAULT,
            "view-source-probe-once",
            10.0,
            10.0,
        );
        let origin = origin_at(&c, None, 14.0, 16.0);
        let Origin::Literal { text, sites } = &origin else {
            panic!("{origin:?}");
        };
        assert_eq!(text, "view-source-probe-once");
        assert_eq!(sites.len(), 1);
        assert_eq!(
            (&sites[0].file, sites[0].line, sites[0].column),
            (&file, 1, 17)
        );
        assert!(at(&sites[0]).starts_with("\"view-source-probe-once\""));
        assert_eq!(origin.site(), Some(&sites[0]));
    }

    #[test]
    fn a_text_several_literals_hold_is_listed_and_not_opened() {
        let file = baseline();
        let mut c = Compositor::new();
        c.begin_frame();
        text(
            &mut c,
            LayerId::DEFAULT,
            "view-source-probe-twice",
            10.0,
            10.0,
        );
        let origin = origin_at(&c, None, 14.0, 16.0);
        let Origin::Literal { sites, .. } = &origin else {
            panic!("{origin:?}");
        };
        assert_eq!(
            sites.iter().map(|s| (s.line, s.column)).collect::<Vec<_>>(),
            vec![(2, 17), (3, 17)]
        );
        assert!(sites.iter().all(|s| s.file == file));
        assert_eq!(origin.site(), None, "several literals: nothing is guessed");
        let log = origin.to_string();
        assert!(log.contains("2 literals hold it, none opened"), "{log}");
        assert!(log.contains(&format!("{}:3:17", file.display())), "{log}");
    }

    #[test]
    fn a_formatted_text_has_no_literal() {
        baseline();
        let mut c = Compositor::new();
        c.begin_frame();
        let shown = format!("{} view-source-probe items", 7);
        text(&mut c, LayerId::DEFAULT, &shown, 10.0, 10.0);
        let origin = origin_at(&c, None, 14.0, 16.0);
        assert_eq!(origin, Origin::Formatted(shown));
        assert_eq!(origin.site(), None);
    }

    #[test]
    fn the_text_is_the_topmost_one_that_shows_at_the_point() {
        let mut c = Compositor::new();
        c.begin_frame();
        text(&mut c, LayerId::DEFAULT, "below", 10.0, 10.0);
        let over = c.create_layer(5);
        text(&mut c, over, "above", 10.0, 10.0);
        assert_eq!(text_at(&c, 14.0, 16.0).as_deref(), Some("above"));
        // the run's box: where it is drawn, as big as it measures
        let (_, [x, y, w, h]) = text_run_at(&c, 14.0, 16.0).unwrap();
        let key = TextNodeKey::new("above", 14.0, 18.0, None);
        assert_eq!([x, y, w, h], text_box(&key, 10.0, 10.0));
        assert!(w > 0.0 && h > 0.0);
        // a text whose box does not reach the point is not there
        assert_eq!(text_at(&c, 14.0, 60.0), None);

        c.set_layer_visible(over, false);
        assert_eq!(text_at(&c, 14.0, 16.0).as_deref(), Some("below"));

        // clipped away where the point is
        let mut c = Compositor::new();
        c.begin_frame();
        c.push(SceneNode::PushClip {
            x: 0.0,
            y: 0.0,
            w: 12.0,
            h: 100.0,
        });
        text(&mut c, LayerId::DEFAULT, "clipped", 10.0, 10.0);
        c.push(SceneNode::PopClip);
        assert_eq!(text_at(&c, 11.0, 16.0).as_deref(), Some("clipped"));
        assert_eq!(text_at(&c, 14.0, 16.0), None);
    }

    #[test]
    fn the_editor_is_plev_editor_then_zed_then_code() {
        let site = Site {
            file: PathBuf::from("/w/crates/a/src/lib.rs"),
            line: 12,
            column: 5,
        };
        let args = |cmd: &Command| {
            cmd.get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        };
        let cmd = editor_command(&site, Some("/tmp/rec.sh --flag"), None).unwrap();
        assert_eq!(cmd.get_program(), "/tmp/rec.sh");
        assert_eq!(args(&cmd), ["--flag", "/w/crates/a/src/lib.rs:12:5"]);

        let root = std::env::temp_dir().join("plev-view-source-editors");
        let (both, code_only, neither) = (root.join("both"), root.join("code"), root.join("none"));
        for (dir, names) in [
            (&both, &["zed", "code"][..]),
            (&code_only, &["code"][..]),
            (&neither, &[][..]),
        ] {
            std::fs::create_dir_all(dir).unwrap();
            for name in names {
                exe(&dir.join(name), 0o755);
            }
        }
        // a `zed` that is not executable is not an editor
        exe(&neither.join("zed"), 0o644);
        let path = |dirs: &[&PathBuf]| std::env::join_paths(dirs).unwrap().into_string().unwrap();

        let cmd = editor_command(&site, Some("  "), Some(&path(&[&neither, &both]))).unwrap();
        assert_eq!(cmd.get_program(), both.join("zed").as_os_str());
        assert_eq!(args(&cmd), ["/w/crates/a/src/lib.rs:12:5"]);

        let cmd = editor_command(&site, None, Some(&path(&[&code_only]))).unwrap();
        assert_eq!(cmd.get_program(), code_only.join("code").as_os_str());
        assert_eq!(args(&cmd), ["-g", "/w/crates/a/src/lib.rs:12:5"]);
        assert_eq!(
            command_line(&cmd),
            format!(
                "{} -g /w/crates/a/src/lib.rs:12:5",
                code_only.join("code").display()
            )
        );

        assert!(editor_command(&site, None, Some(&path(&[&neither]))).is_none());
        assert!(editor_command(&site, None, None).is_none());
    }

    #[test]
    fn open_runs_the_editor_with_the_site_and_says_when_there_is_none() {
        let dir = std::env::temp_dir().join("plev-view-source-open");
        std::fs::create_dir_all(&dir).unwrap();
        let (script, got) = (dir.join("rec.sh"), dir.join("args.txt"));
        let _ = std::fs::remove_file(&got);
        std::fs::write(
            &script,
            format!("#!/bin/sh\nprintf '%s\\n' \"$*\" > '{}'\n", got.display()),
        )
        .unwrap();
        mode(&script, 0o755);
        let site = Site {
            file: PathBuf::from("/w/crates/a/src/lib.rs"),
            line: 3,
            column: 9,
        };

        let editor = format!("{} --wait", script.display());
        let ran = open_with(&site, Some(&editor), None).unwrap();
        assert_eq!(ran, format!("{editor} /w/crates/a/src/lib.rs:3:9"));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !got.exists() && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert_eq!(
            std::fs::read_to_string(&got).unwrap(),
            "--wait /w/crates/a/src/lib.rs:3:9\n"
        );

        let none = open_with(&site, None, Some(dir.to_str().unwrap())).unwrap_err();
        assert_eq!(none.kind(), std::io::ErrorKind::NotFound);
        assert!(none.to_string().contains("PLEV_EDITOR"), "{none}");
    }

    /// A script at `p` that does nothing, with `bits` as its mode.
    fn exe(p: &std::path::Path, bits: u32) {
        std::fs::write(p, "#!/bin/sh\n").unwrap();
        mode(p, bits);
    }

    fn mode(p: &std::path::Path, bits: u32) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(p, std::fs::Permissions::from_mode(bits)).unwrap();
    }
}
