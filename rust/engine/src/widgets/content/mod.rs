//! Content: the things that show data.

mod avatar;
pub(crate) mod badge;
mod card;
pub(crate) mod chip;
mod code_block;
mod empty_state;
mod list;
mod panel;
mod separator;
mod skeleton;
mod stat;
mod table;
mod tree;

// task-104 front D, disclosure and display (#157): one module per widget,
// and the front's token slot (`Theme::disclosure`).
pub mod accordion;
pub mod aspect_ratio;
pub mod carousel;
pub mod collapsible;
pub mod item;
pub mod kbd;
pub mod pagination;
pub mod tokens;

pub use accordion::{Accordion, AccordionMode};
pub use aspect_ratio::{AspectFrame, AspectRatio};
pub use carousel::{Carousel, CarouselLayout, CarouselPart};
pub use collapsible::{Collapsible, DisclosureList, DisclosureLook};
pub use item::{Item, ItemShape};
pub use kbd::{Kbd, KbdSize};
pub use pagination::{PageSlot, Pagination};
pub use tokens::{
    AspectRecipe, CapRecipe, CarouselRecipe, DisclosureMotion, DisclosureRail, DisclosureRecipe,
    DisclosureSurface, DisclosureTile, DisclosureTokens, GridRecipe, ItemRecipe, PagerRecipe,
    RowRecipe, TableRecipe, TreeRecipe, TreeTile,
};

pub use avatar::{Avatar, AvatarSize};
pub use card::{Card, CardListRow, CardVariant};
pub use code_block::{CodeBlock, CodeLine, LineKind};
pub use empty_state::EmptyState;
pub use list::VirtualList;
pub use panel::Panel;
pub use separator::Separator;
pub use skeleton::{Skeleton, SkeletonShape};
pub use stat::{Stat, StatSize};
pub use table::{Align, Column, ColumnLayout, Table};
pub use tree::{Tree, TreeNode};
