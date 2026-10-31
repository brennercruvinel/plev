//! Navigation: the things that move the user between places.

mod breadcrumb;
mod nav_link;
mod panel_header;
mod sidebar;
mod sidebars;
mod split_pane;
mod tabs;
mod toolbar;

// task-104 front G, navigation and data: the front's token slot
// (`Theme::navigation`).
pub mod tokens;

pub use breadcrumb::Breadcrumb;
pub use nav_link::NavLink;
pub use panel_header::PanelHeader;
pub use sidebar::{NavGroup, Sidebar};
pub use sidebars::{GuideGroup, GuidelineSidebar, HomeLayout, HomeRow, HomeSidebar};
pub use split_pane::{SplitDirection, SplitPane};
pub use tabs::Tabs;
pub use tokens::{
    BreadcrumbRecipe, ChartRecipe, CodeRecipe, GraphRecipe, LinkSurface, NavLinkRecipe, NavText,
    NavigationTokens, PlayerRecipe, PlayerTones, SidebarsRecipe, StatLevel, StatRecipe, ToolRecipe,
    ToolTokens, ToolbarRecipe, ViewRecipe, ZoomRecipe,
};
pub use toolbar::{
    PlayerLayout, Tool, Toolbar, ToolbarLayout, VideoPlayer, ViewController, Zoom, ZoomLine,
};
