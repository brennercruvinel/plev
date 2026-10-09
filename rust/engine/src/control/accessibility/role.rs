//! `accord::Role` to `accesskit::Role`: what a node is, told to assistive
//! technology. The match is exhaustive, so a role added to the contract
//! does not compile until it has a mapping here.

use accesskit::Role as Ak;
use accord::Role;

/// The AccessKit role of a contract role. `secret` turns a text input
/// into a password field (its value is never exposed either, see
/// `tree`).
///
/// Three roles have no one-to-one AccessKit counterpart and take the
/// nearest: `Toggle` (an on/off control, the comps `Switch` and a
/// selectable `Chip`) is a `Switch`; `Select` (a collapsed choice that
/// opens a list) is a `ComboBox`, as a native select is; `Media` (audio
/// or video, the contract does not say which) is an `EmbeddedObject`.
pub fn of(role: Role, secret: bool) -> Ak {
    match role {
        Role::Window => Ak::Window,
        Role::Group => Ak::Group,
        Role::Region => Ak::Region,
        Role::Toolbar => Ak::Toolbar,
        Role::Button => Ak::Button,
        Role::Toggle => Ak::Switch,
        Role::Checkbox => Ak::CheckBox,
        Role::Radio => Ak::RadioButton,
        Role::TextInput if secret => Ak::PasswordInput,
        Role::TextInput => Ak::TextInput,
        Role::Slider => Ak::Slider,
        Role::Select => Ak::ComboBox,
        Role::Option => Ak::ListBoxOption,
        Role::List => Ak::List,
        Role::ListItem => Ak::ListItem,
        Role::Table => Ak::Table,
        Role::Row => Ak::Row,
        Role::Cell => Ak::Cell,
        Role::TabList => Ak::TabList,
        Role::Tab => Ak::Tab,
        Role::TabPanel => Ak::TabPanel,
        Role::Tree => Ak::Tree,
        Role::TreeItem => Ak::TreeItem,
        Role::Dialog => Ak::Dialog,
        Role::Menu => Ak::Menu,
        Role::MenuItem => Ak::MenuItem,
        Role::Label => Ak::Label,
        Role::Image => Ak::Image,
        Role::Link => Ak::Link,
        Role::ProgressBar => Ak::ProgressIndicator,
        Role::Status => Ak::Status,
        Role::Alert => Ak::Alert,
        Role::Canvas => Ak::Canvas,
        Role::Media => Ak::EmbeddedObject,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole table, row by row: every contract role has one AccessKit
    /// role, and no two contract roles share one.
    #[test]
    fn every_role_maps_and_no_two_share_an_accesskit_role() {
        let expected = [
            (Role::Window, Ak::Window),
            (Role::Group, Ak::Group),
            (Role::Region, Ak::Region),
            (Role::Toolbar, Ak::Toolbar),
            (Role::Button, Ak::Button),
            (Role::Toggle, Ak::Switch),
            (Role::Checkbox, Ak::CheckBox),
            (Role::Radio, Ak::RadioButton),
            (Role::TextInput, Ak::TextInput),
            (Role::Slider, Ak::Slider),
            (Role::Select, Ak::ComboBox),
            (Role::Option, Ak::ListBoxOption),
            (Role::List, Ak::List),
            (Role::ListItem, Ak::ListItem),
            (Role::Table, Ak::Table),
            (Role::Row, Ak::Row),
            (Role::Cell, Ak::Cell),
            (Role::TabList, Ak::TabList),
            (Role::Tab, Ak::Tab),
            (Role::TabPanel, Ak::TabPanel),
            (Role::Tree, Ak::Tree),
            (Role::TreeItem, Ak::TreeItem),
            (Role::Dialog, Ak::Dialog),
            (Role::Menu, Ak::Menu),
            (Role::MenuItem, Ak::MenuItem),
            (Role::Label, Ak::Label),
            (Role::Image, Ak::Image),
            (Role::Link, Ak::Link),
            (Role::ProgressBar, Ak::ProgressIndicator),
            (Role::Status, Ak::Status),
            (Role::Alert, Ak::Alert),
            (Role::Canvas, Ak::Canvas),
            (Role::Media, Ak::EmbeddedObject),
        ];
        assert_eq!(expected.len(), Role::ALL.len());
        for (role, (listed, ak)) in Role::ALL.iter().zip(expected) {
            assert_eq!(*role, listed, "the table follows Role::ALL");
            assert_eq!(of(*role, false), ak, "{}", role.as_str());
        }
        let mapped: rustc_hash::FxHashSet<Ak> = Role::ALL.iter().map(|r| of(*r, false)).collect();
        assert_eq!(mapped.len(), Role::ALL.len(), "two roles share one");
    }

    #[test]
    fn only_a_text_input_changes_when_secret() {
        for role in Role::ALL {
            let expected = match role {
                Role::TextInput => Ak::PasswordInput,
                other => of(other, false),
            };
            assert_eq!(of(role, true), expected, "{}", role.as_str());
        }
    }
}
