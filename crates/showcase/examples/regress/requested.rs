//! The table of the differences the product asks for, per target
//! (`crates/showcase/fixtures/determinism/requested.json`, versioned with
//! the code): what differs between a target's frame and the desktop's by
//! a decision (the platform draws over a band, the device's scale is the
//! frame's), each with its reason in one line and whether it was known
//! on the desktop or waits for the first capture on its target
//! (`to_confirm`). The comparator reads a target's rows as its
//! decisions; a difference no row explains is silent.

use std::path::Path;

use serde::Deserialize;

use crate::classify::{Area, Decision};

/// The table in the repository, wherever the example runs from.
pub const TABLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/fixtures/determinism/requested.json"
);

/// The version of the table's layout.
pub const FORMAT: &str = "determinism.requested.v1";

/// The targets a row may name.
pub const TARGETS: [&str; 4] = ["desktop", "web", "ios", "android"];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    pub format: String,
    pub rows: Vec<Row>,
}

/// One decision of the product on one target.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub target: String,
    /// Short and kebab-case, unique on its target: `status-bar`.
    pub name: String,
    pub area: Area,
    pub status: Status,
    /// Why, in one line.
    pub reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Known on the desktop, where the comparison runs today.
    Known,
    /// Declared from the platform's documented behavior; the first
    /// capture on the target confirms the area or corrects it.
    ToConfirm,
}

impl Table {
    pub fn read(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// The table in `text`, refused when a row is malformed: an unknown
    /// target, a name that is not kebab-case or repeats on its target, a
    /// reason that is empty or spans lines, an empty area, two size rows
    /// on one target.
    pub fn parse(text: &str) -> Result<Self, String> {
        let table: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if table.format != FORMAT {
            return Err(format!(
                "written as {}, this harness reads {FORMAT}",
                table.format
            ));
        }
        for (i, row) in table.rows.iter().enumerate() {
            let at = format!("row {} ({} {})", i + 1, row.target, row.name);
            if !TARGETS.contains(&row.target.as_str()) {
                return Err(format!("{at}: the target is one of {}", TARGETS.join(", ")));
            }
            let kebab = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-';
            if row.name.is_empty() || !row.name.chars().all(kebab) {
                return Err(format!("{at}: the name is kebab-case"));
            }
            if row.reason.trim().is_empty() || row.reason.contains('\n') {
                return Err(format!("{at}: the reason is one line"));
            }
            let positive = |v: f32| v.is_finite() && v > 0.0;
            let area = match row.area {
                Area::Edge { px, .. } => positive(px),
                Area::Rect { rect } => {
                    rect.iter().all(|v| v.is_finite()) && positive(rect[2]) && positive(rect[3])
                }
                Area::Size => true,
            };
            if !area {
                return Err(format!("{at}: the area is empty"));
            }
            let before = &table.rows[..i];
            if before
                .iter()
                .any(|r| r.target == row.target && r.name == row.name)
            {
                return Err(format!("{at}: the name repeats on its target"));
            }
            let size = |r: &Row| r.target == row.target && r.area == Area::Size;
            if size(row) && before.iter().any(size) {
                return Err(format!("{at}: one size row per target"));
            }
        }
        Ok(table)
    }

    /// The rows of `target`.
    pub fn rows(&self, target: &str) -> Result<Vec<&Row>, String> {
        if !TARGETS.contains(&target) {
            return Err(format!(
                "no target '{target}'; the targets are {}",
                TARGETS.join(", ")
            ));
        }
        Ok(self.rows.iter().filter(|r| r.target == target).collect())
    }

    /// The rows of `target` as the comparator's decisions.
    pub fn decisions(&self, target: &str) -> Result<Vec<Decision<'_>>, String> {
        Ok(self.rows(target)?.into_iter().map(Row::decision).collect())
    }
}

impl Row {
    pub fn decision(&self) -> Decision<'_> {
        Decision {
            name: &self.name,
            reason: &self.reason,
            area: &self.area,
        }
    }

    /// `status-bar (to confirm)`.
    pub fn label(&self) -> String {
        match self.status {
            Status::Known => self.name.clone(),
            Status::ToConfirm => format!("{} (to confirm)", self.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::Edge;

    fn table() -> Table {
        match Table::read(Path::new(TABLE)) {
            Ok(t) => t,
            Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn the_committed_table_reads_and_every_target_declares_its_size() {
        let t = table();
        for target in TARGETS {
            let rows = match t.rows(target) {
                Ok(r) => r,
                Err(e) => panic!("{e}"),
            };
            let sizes = rows.iter().filter(|r| r.area == Area::Size).count();
            assert_eq!(sizes, 1, "{target}");
        }
        // the desktop knows what it declares; a row of another target is
        // known once a capture confirmed it (2026-09-25: web, the iOS
        // simulator and the android emulator), the rest waits for one
        let confirmed = [
            ("web", "size"),
            ("ios", "status-bar"),
            ("ios", "home-indicator"),
            ("ios", "size"),
            ("android", "navigation-bar"),
            ("android", "size"),
        ];
        for row in &t.rows {
            let known = row.status == Status::Known;
            let expected = row.target == "desktop"
                || confirmed.contains(&(row.target.as_str(), row.name.as_str()));
            assert_eq!(known, expected, "{} {}", row.target, row.name);
        }
        let ios = match t.decisions("ios") {
            Ok(d) => d,
            Err(e) => panic!("{e}"),
        };
        assert!(ios.iter().any(|d| d.name == "status-bar"
            && *d.area
                == Area::Edge {
                    edge: Edge::Top,
                    px: 62.0
                }));
        match t.rows("linux") {
            Err(e) => assert!(e.contains("no target 'linux'"), "{e}"),
            Ok(r) => panic!("a row for linux: {r:?}"),
        }
    }

    fn row(target: &str, name: &str, area: &str, reason: &str) -> String {
        format!(
            r#"{{"target":"{target}","name":"{name}","area":{area},"status":"to_confirm","reason":"{reason}"}}"#
        )
    }

    fn parse(rows: &[String]) -> Result<Table, String> {
        Table::parse(&format!(
            r#"{{"format":"{FORMAT}","rows":[{}]}}"#,
            rows.join(",")
        ))
    }

    #[test]
    fn a_malformed_row_is_refused_with_the_reason() {
        let size = r#"{"kind":"size"}"#;
        let edge = r#"{"kind":"edge","edge":"top","px":20}"#;
        assert!(parse(&[row("ios", "size", size, "the device")]).is_ok());
        for (rows, needle) in [
            (
                vec![row("linux", "size", size, "x")],
                "the target is one of",
            ),
            (vec![row("ios", "Status Bar", edge, "x")], "kebab-case"),
            (vec![row("ios", "bar", edge, "one\\ntwo")], "one line"),
            (vec![row("ios", "bar", edge, " ")], "one line"),
            (
                vec![row(
                    "ios",
                    "bar",
                    r#"{"kind":"edge","edge":"top","px":0}"#,
                    "x",
                )],
                "empty",
            ),
            (
                vec![row(
                    "ios",
                    "bar",
                    r#"{"kind":"rect","rect":[0,0,10,-1]}"#,
                    "x",
                )],
                "empty",
            ),
            (
                vec![row("ios", "bar", edge, "x"), row("ios", "bar", edge, "y")],
                "repeats",
            ),
            (
                vec![
                    row("ios", "size", size, "x"),
                    row("ios", "scale", size, "y"),
                ],
                "one size row",
            ),
        ] {
            match parse(&rows) {
                Err(e) => assert!(e.contains(needle), "'{e}' lacks '{needle}'"),
                Ok(t) => panic!("accepted {:?}", t.rows),
            }
        }
        // the same name on two targets is two rows
        assert!(
            parse(&[
                row("ios", "bar", edge, "x"),
                row("android", "bar", edge, "y")
            ])
            .is_ok()
        );
        match Table::parse(r#"{"format":"determinism.requested.v0","rows":[]}"#) {
            Err(e) => assert!(e.contains("this harness reads"), "{e}"),
            Ok(_) => panic!("another format was read"),
        }
    }
}
