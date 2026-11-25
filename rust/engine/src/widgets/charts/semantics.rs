//! A chart in a semantic projection: an image named by its title and
//! described by a sentence that says what it shows, over a table of its
//! data, one row per datum, so a reader and an agent get the numbers the
//! picture draws.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

/// Declares the chart `id`: a `Role::Image` named `title`, described by
/// `summary`, with a `Role::Table` child `<id>.data` named "data" and a
/// `Role::Row` `<id>.data[<i>]` per datum, labeled `<name>: <value>`.
/// Returns the image's node for the app to add to.
#[track_caller]
pub fn project<'b>(
    b: &'b mut SnapshotBuilder,
    id: &SemId,
    parent: Option<&SemId>,
    title: &str,
    summary: &str,
    rows: &[(String, String)],
) -> NodeBuilder<'b> {
    if let Ok(table) = id.child("data") {
        for (i, (name, value)) in rows.iter().enumerate() {
            b.node(
                table.key(i.to_string()),
                Role::Row,
                format!("{name}: {value}"),
            )
            .parent(&table);
        }
        b.node(&table, Role::Table, "data").parent(id);
    }
    let mut node = b.node(id, Role::Image, title);
    if let Some(parent) = parent {
        node.parent(parent);
    }
    node.description(summary);
    node
}
