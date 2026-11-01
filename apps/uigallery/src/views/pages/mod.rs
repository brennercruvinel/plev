//! The gallery's pages: the sidebar tree (three groups) and, per page,
//! the title, icon and blurb the chrome shows and what the page body is.
//! A page is backed by a [`Pane`] (a module with state under
//! `components/`, `foundations/` or `apps/`), shares one with a note on
//! what its own example still lacks, or is a placeholder for a widget
//! `comps` does not have yet. `PAGES` is the one list; the sidebar, the
//! digit hotkeys and the launch argument all index it. The table itself
//! is `table.rs`; this file is the types and the lookups.

pub(super) mod panes;
mod table;
#[cfg(test)]
mod tests;

pub use table::PAGES;

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Foundations,
    Components,
    AppsDemo,
}

impl Group {
    pub const ALL: [Group; 3] = [Group::Foundations, Group::Components, Group::AppsDemo];

    pub fn label(self) -> &'static str {
        match self {
            Group::Foundations => "Foundations",
            Group::Components => "Components",
            Group::AppsDemo => "Apps Demo",
        }
    }

    /// Number of pages in the group (the sidebar's group lengths).
    pub fn pages(self) -> usize {
        PAGES.iter().filter(|p| p.group == self).count()
    }
}

// ---------------------------------------------------------------------------
// Panes
// ---------------------------------------------------------------------------

/// The modules with state: one struct in the view per pane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pane {
    // foundations
    Theme,
    Typography,
    Icons,
    Effects,
    // components
    Avatar,
    Badge,
    Breadcrumb,
    Button,
    Card,
    Chart,
    Checkbox,
    Chip,
    CodeBlock,
    ContextMenu,
    Dialog,
    Empty,
    Graph,
    IconButton,
    Input,
    NavigationMenu,
    Panel,
    PanelHeader,
    Progress,
    Resizable,
    ScrollArea,
    Select,
    Separator,
    Sidebar,
    Skeleton,
    Slider,
    Spinner,
    Stat,
    Switch,
    Table,
    Tabs,
    Toast,
    Toolbar,
    Tooltip,
    Tree,
    // task-104: each front adds its variants under its anchor, with the page
    // task-104 front A, fields (#154)
    Field,
    Label,
    InputGroup,
    InputOtp,
    Textarea,
    Combobox,
    Command,
    // task-104 front B, overlays (#155)
    AlertDialog,
    Drawer,
    DropdownMenu,
    HoverCard,
    Menubar,
    Modals,
    Popover,
    Sheet,
    // task-104 front C, conversation and feedback (#156)
    Alert,
    Attachment,
    Bubble,
    Comments,
    Marker,
    Message,
    MessageScroller,
    Notifications,
    PromptInput,
    // task-104 front D, disclosure and display (#157)
    Accordion,
    AspectRatio,
    Carousel,
    Collapsible,
    Item,
    Kbd,
    Pagination,
    /// The static table; `Table` is the Data Table.
    StaticTable,
    // task-104 front E, selection and dates (#158)
    ButtonGroup,
    Calendar,
    DatePicker,
    Menus,
    RadioGroup,
    Toggle,
    ToggleGroup,
    // task-104 front F, kit foundation
    Home,
    Topbar,
    // apps demo
    Todo,
    Surface,
    Silences,
    Files,
    Capture,
    Dock,
    Editor,
    Git,
    Builder,
}

impl Pane {
    /// Panes that are apps filling the window past the header, with no
    /// page gutter.
    pub fn is_app(self) -> bool {
        matches!(self, Pane::Editor | Pane::Git)
    }
}

// ---------------------------------------------------------------------------
// Pages
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    /// The page is this pane.
    Pane(Pane),
    /// The page shows another pane's example; `gap` says what its own
    /// example still needs (shown as a note above the content).
    Alias { pane: Pane, gap: &'static str },
    /// No widget in `comps` yet: an empty page that keeps the slot.
    Placeholder,
}

#[derive(Debug)]
pub struct Page {
    pub group: Group,
    pub title: &'static str,
    pub icon: &'static str,
    pub blurb: &'static str,
    pub body: Body,
}

impl Page {
    /// The pane behind the page, if any (none for a placeholder).
    pub fn pane(&self) -> Option<Pane> {
        match self.body {
            Body::Pane(p) | Body::Alias { pane: p, .. } => Some(p),
            Body::Placeholder => None,
        }
    }
}

/// Index of the page titled `name` (case-insensitive).
pub fn find(name: &str) -> Option<usize> {
    PAGES
        .iter()
        .position(|p| p.title.eq_ignore_ascii_case(name))
}

/// Index of the page that is exactly this pane (not an alias of it).
pub fn index_of(pane: Pane) -> usize {
    PAGES
        .iter()
        .position(|p| p.body == Body::Pane(pane))
        .expect("every pane has a page")
}
