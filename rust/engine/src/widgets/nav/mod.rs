//! Navigation: the things that move the user between places.

mod breadcrumb;
mod nav_link;
mod panel_header;
mod sidebar;
mod split_pane;
mod tabs;

// task-104 front G, navigation and data: the front's token slot
// (`Theme::navigation`).
pub mod tokens;

pub use breadcrumb::Breadcrumb;
pub use nav_link::NavLink;
pub use panel_header::PanelHeader;
pub use sidebar::{NavGroup, Sidebar};
pub use split_pane::{SplitDirection, SplitPane};
pub use tabs::Tabs;
pub use tokens::{BreadcrumbRecipe, LinkSurface, NavLinkRecipe, NavText, NavigationTokens};
