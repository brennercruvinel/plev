//! Layout engine tests, split by area:
//! - [`flex`]: stacking, padding, gap, flex-grow, align/justify.
//! - [`percent`]: percentage dimensions.
//! - [`measure`]: text measurement integration (taffy measure function).

mod flex;
mod measure;
mod percent;

use super::*;

fn leaf(style: LayoutStyle) -> LayoutItem {
    LayoutItem {
        style,
        children: vec![],
        text: None,
    }
}

fn container(style: LayoutStyle, children: Vec<usize>) -> LayoutItem {
    LayoutItem {
        style,
        children,
        text: None,
    }
}
