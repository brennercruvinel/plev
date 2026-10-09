//! Builder tests, split by area:
//! - [`styles`]: builder API surface (children composition, style sugar).
//! - [`emit`]: scene-node emission (which `SceneNode`s an element produces).
//! - [`layout`]: layout pipeline (positioning + text measurement parity).

mod emit;
mod layout;
mod styles;

use crate::view::ViewContext;

fn test_cx() -> ViewContext {
    ViewContext::new(800.0, 600.0)
}
