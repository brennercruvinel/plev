//! The Menus page as a component: its projection (a menu per card, its
//! items and their `picked`), the requests it takes and where the focus
//! goes.

use accord::{Request, Role, SemId, SnapshotBuilder, Value};
use engine::widgets::prelude::MenuEntry;
use engine::widgets::selection::Folders;

use super::super::{After, Component, under};
use super::{Body, MenusPage, Spot, group_id};

impl Component for MenusPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(group) = group_id(root) else {
            return;
        };
        b.node(&group, Role::Group, "Menu").parent(root);
        for card in &self.cards {
            match &card.body {
                Body::Menu(m) => {
                    let Some(id) = m.sem_id() else {
                        continue;
                    };
                    for e in &m.entries {
                        if let MenuEntry::Item {
                            id: item,
                            label,
                            shortcut,
                            checked,
                            ..
                        } = e
                        {
                            let mut node = b.node(id.key(item.to_string()), Role::MenuItem, label);
                            node.parent(id);
                            if let Some(keys) = shortcut {
                                node.description(keys.as_str());
                            }
                            if let Some(on) = checked {
                                node.state("checked", *on);
                            }
                            node.action("activate");
                        }
                    }
                    let mut node = b.node(id, Role::Menu, card.title);
                    node.parent(&group);
                    node.state("picked", card.picked.clone().unwrap_or_default());
                }
                Body::Folders(f) => {
                    if let Some(id) = f.sem_id() {
                        f.project(b, id, card.title, Some(&group));
                        if let Some(mut node) = b.reopen(id) {
                            node.state("picked", card.picked.clone().unwrap_or_default());
                        }
                    }
                }
                Body::Zoom(z) => {
                    if let Some(id) = z.sem_id() {
                        z.project(b, id, Some(&group));
                        if let Some(mut node) = b.reopen(id) {
                            node.state("picked", card.picked.clone().unwrap_or_default());
                        }
                    }
                }
                Body::Models(m) => {
                    if let Some(id) = m.sem_id() {
                        let mut node = m.project(b, id, card.title, Some(&group));
                        node.state("picked", card.picked.clone().unwrap_or_default());
                    }
                }
            }
        }
        let focused = self
            .focus
            .and_then(|(c, spot)| match (&self.cards[c].body, spot) {
                (Body::Menu(m), Spot::Item(id)) => Some(m.sem_id()?.key(id.to_string())),
                (Body::Folders(f), Spot::Row(r)) => f.row_id(r),
                (Body::Zoom(z), Spot::Line(i)) => {
                    Some(z.sem_id()?.child("line").ok()?.key(i.to_string()))
                }
                (Body::Models(m), Spot::Line(i)) => m.option_id(i),
                _ => None,
            });
        let field = self.text_field().and_then(|f| f.sem_id().cloned());
        if let Some(id) = focused.or(field) {
            b.focus(&id);
        }
    }

    /// An item's or a folder's `activate` picks it, as its press does; a
    /// box's `set_value` puts its text; the name's `submit` adds the
    /// folder, as Enter does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let refuse = || format!("the page has no {} on {}", req.action, req.target);
        let f = self.folders();
        let search = f.and_then(|f| f.search.sem_id().cloned());
        let name = f.and_then(|f| f.naming()?.sem_id().cloned());
        let target = Some(&req.target);
        match (req.action.as_str(), &req.param) {
            ("set_value", Some(Value::Text(text))) if target == search.as_ref() => {
                let f = self.folders_mut().ok_or_else(refuse)?;
                f.search.set_text(text);
                Ok(After::Nothing)
            }
            ("set_value", Some(Value::Text(text))) if target == name.as_ref() => {
                let f = self.folders_mut().and_then(Folders::naming_mut);
                f.ok_or_else(refuse)?.set_text(text);
                Ok(After::Nothing)
            }
            ("submit", _) if target == name.as_ref() && self.commit_name() => Ok(After::Nothing),
            ("activate", _) => match self.at(&req.target) {
                Some((c, spot)) if self.pick(c, spot) => Ok(After::Nothing),
                _ => Err(refuse()),
            },
            _ => Err(refuse()),
        }
    }

    /// A row, an item or a box takes the focus (its ring, or the box's
    /// caret); another node of the page takes it from them.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let f = self.folders();
        let search = f.and_then(|f| f.search.sem_id()) == Some(id);
        let name = f.and_then(|f| f.naming()?.sem_id().cloned()).as_ref() == Some(id);
        if let Some(f) = self.folders_mut() {
            if search || name {
                f.focus_field(name);
            } else {
                f.blur_fields();
            }
        }
        if let Some(z) = self.zoom_mut().filter(|z| z.field.is_focused()) {
            z.field.unfocus();
            z.sync_level();
        }
        self.set_focus(self.at(id));
        true
    }

    fn blur(&mut self) -> bool {
        self.handle_escape()
    }
}
