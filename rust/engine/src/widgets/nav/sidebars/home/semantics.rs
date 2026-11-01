//! The HomeSidebar told to a reader and an agent: a region of its own
//! ([`HomeSidebar::project`]), or its parts under a node the app declares
//! ([`HomeSidebar::project_parts`]: the gallery's dock declares the
//! navigation landmark and the sidebar's parts go under it).

use accord::{Role, SemId, SnapshotBuilder};

use super::{FolderRow, HomeRow, HomeSidebar};

impl HomeSidebar {
    /// A `Role::Region` named `label`: the dropdowns (their links while
    /// open), the links, the scenes link, the new folder (`expanded`
    /// while its field is open) and the folders, each a button.
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: &str,
        parent: Option<&SemId>,
    ) {
        let mut node = b.node(id, Role::Region, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        self.project_parts(b, id);
    }

    /// The parts alone, under `id` (a node the caller declared): the
    /// logo, a link home, then what [`HomeSidebar::project`] lists.
    #[track_caller]
    pub fn project_parts(&self, b: &mut SnapshotBuilder, id: &SemId) {
        if let Some(l) = self.logo.sem_id() {
            self.logo.project(b, l, Some(id));
        }
        for row in &self.rows {
            match row {
                HomeRow::Menu(m, links) => {
                    let Some(at) = m.sem_id() else { continue };
                    m.project(b, at, Some(id));
                    let open = m.project_panel(b, at, Some(id)).is_some();
                    if open && let Ok(panel) = at.child("panel") {
                        links.project(b, Some(&panel));
                    }
                }
                HomeRow::Link(link) => {
                    if let Some(at) = link.sem_id() {
                        link.project(b, at, Some(id));
                    }
                }
            }
        }
        if self.library
            && let Some(at) = self.scenes.sem_id()
        {
            self.scenes
                .project(b, at, Some(id))
                .description(self.caption.as_str());
        }
        for row in self.folder_rows() {
            let Some(at) = self.folder_id(row) else {
                continue;
            };
            let name = match row {
                FolderRow::New => "New Folder",
                FolderRow::Folder(i) => self.folders[i].as_str(),
            };
            let mut node = b.node(&at, Role::Button, name);
            node.parent(id);
            if row == FolderRow::New {
                node.state("expanded", self.naming);
            }
            node.action("activate");
        }
        if self.naming
            && let Some(at) = self.name.sem_id()
        {
            self.name.project(b, at, "Folder name", Some(id));
        }
    }
}
