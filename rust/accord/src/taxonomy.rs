//! `Role`, what a node is. a closed set: a new role needs the
//! orchestrator's approval, and each maps to an accessibility role later
//! without this crate depending on an accessibility library.

/// what a node is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Role {
    Window,
    Group,
    Region,
    Navigation,
    Toolbar,
    Button,
    Toggle,
    Checkbox,
    Radio,
    TextInput,
    Slider,
    Select,
    Option,
    List,
    ListItem,
    Table,
    Row,
    Cell,
    TabList,
    Tab,
    TabPanel,
    Tree,
    TreeItem,
    Dialog,
    Menu,
    MenuItem,
    Label,
    Image,
    Link,
    ProgressBar,
    Status,
    Alert,
    Canvas,
    Media,
}

impl Role {
    /// every role, in declaration order.
    pub const ALL: [Role; 34] = [
        Role::Window,
        Role::Group,
        Role::Region,
        Role::Navigation,
        Role::Toolbar,
        Role::Button,
        Role::Toggle,
        Role::Checkbox,
        Role::Radio,
        Role::TextInput,
        Role::Slider,
        Role::Select,
        Role::Option,
        Role::List,
        Role::ListItem,
        Role::Table,
        Role::Row,
        Role::Cell,
        Role::TabList,
        Role::Tab,
        Role::TabPanel,
        Role::Tree,
        Role::TreeItem,
        Role::Dialog,
        Role::Menu,
        Role::MenuItem,
        Role::Label,
        Role::Image,
        Role::Link,
        Role::ProgressBar,
        Role::Status,
        Role::Alert,
        Role::Canvas,
        Role::Media,
    ];

    /// true for a control a person or an agent operates directly. these
    /// roles need a non empty label: a control nobody can name cannot be
    /// asked for by voice, by a screen reader or by an agent. containers
    /// and content (list items, rows, cells) may go without one.
    pub fn is_interactive(self) -> bool {
        matches!(
            self,
            Role::Button
                | Role::Toggle
                | Role::Checkbox
                | Role::Radio
                | Role::TextInput
                | Role::Slider
                | Role::Select
                | Role::Option
                | Role::Tab
                | Role::TreeItem
                | Role::MenuItem
                | Role::Link
        )
    }

    /// the stable name, the same one serde uses.
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Window => "window",
            Role::Group => "group",
            Role::Region => "region",
            Role::Navigation => "navigation",
            Role::Toolbar => "toolbar",
            Role::Button => "button",
            Role::Toggle => "toggle",
            Role::Checkbox => "checkbox",
            Role::Radio => "radio",
            Role::TextInput => "text_input",
            Role::Slider => "slider",
            Role::Select => "select",
            Role::Option => "option",
            Role::List => "list",
            Role::ListItem => "list_item",
            Role::Table => "table",
            Role::Row => "row",
            Role::Cell => "cell",
            Role::TabList => "tab_list",
            Role::Tab => "tab",
            Role::TabPanel => "tab_panel",
            Role::Tree => "tree",
            Role::TreeItem => "tree_item",
            Role::Dialog => "dialog",
            Role::Menu => "menu",
            Role::MenuItem => "menu_item",
            Role::Label => "label",
            Role::Image => "image",
            Role::Link => "link",
            Role::ProgressBar => "progress_bar",
            Role::Status => "status",
            Role::Alert => "alert",
            Role::Canvas => "canvas",
            Role::Media => "media",
        }
    }
}
