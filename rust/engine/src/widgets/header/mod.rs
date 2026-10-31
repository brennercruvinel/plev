//! The top bars of the hoff core kit (task-104 front F): the simple,
//! the minimal and the asset header (`components/HeaderSimple`,
//! `HeaderMinimal`, `HeaderAsset`) and the logo (`components/Logo`).
//! The behavior (the row, the focus, the projection) is [`Header`]; the
//! looks are `theme.header` ([`HeaderTokens`]). The search in them is
//! the form's input group.

mod bar;
mod logo;
mod parts;
pub mod tokens;

pub use bar::{Header, HeaderItem, HeaderLayout, HeaderPart, HeaderSlot};
pub use logo::{Logo, LogoKind};
pub use parts::{Action, ActionStyle, Avatar, Thumbs};
pub use tokens::{ActionLook, AvatarTokens, DotTokens, HeaderTokens, LogoTokens, ThumbTokens};

#[cfg(test)]
mod tests;
