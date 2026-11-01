//! The constructors the page table is written with: one per group, so
//! an entry names its title, icon, blurb and pane and the section it sits
//! in says its group, and the placeholder for a component slot `comps`
//! does not fill yet.

use super::{Body, Group, Page, Pane};

const fn page(
    group: Group,
    title: &'static str,
    icon: &'static str,
    blurb: &'static str,
    pane: Pane,
) -> Page {
    Page {
        group,
        title,
        icon,
        blurb,
        body: Body::Pane(pane),
    }
}

/// A page of the Foundations group.
pub(super) const fn foundation(
    title: &'static str,
    icon: &'static str,
    blurb: &'static str,
    pane: Pane,
) -> Page {
    page(Group::Foundations, title, icon, blurb, pane)
}

/// A page of the Components group.
pub(super) const fn component(
    title: &'static str,
    icon: &'static str,
    blurb: &'static str,
    pane: Pane,
) -> Page {
    page(Group::Components, title, icon, blurb, pane)
}

/// A page of the Apps Demo group.
pub(super) const fn app(
    title: &'static str,
    icon: &'static str,
    blurb: &'static str,
    pane: Pane,
) -> Page {
    page(Group::AppsDemo, title, icon, blurb, pane)
}

/// A Components slot with no widget in `comps` yet.
pub(super) const fn placeholder(title: &'static str) -> Page {
    Page {
        group: Group::Components,
        title,
        icon: "circle",
        blurb: "Not in comps yet. The slot is here so the tree reads complete.",
        body: Body::Placeholder,
    }
}
