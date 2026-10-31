//! The text hosts: the component pages whose own text fields take the
//! characters, the editing keys, the input method, the clipboard keys,
//! the field's menu and Escape. A page is one by implementing
//! [`TextHost`] and taking one line of the table below; the routes in
//! `input.rs`, `ime.rs`, `clipboard.rs` and `field_menu.rs` reach a host
//! through the table and nowhere else. The app pages (Todo, Surface,
//! Silences, Git...) keep their own routes: their fields share state
//! with the todo's dispatcher.

use engine::styling::theme::Theme;
use engine::widgets::prelude::Rect;

use super::super::pages::panes::Panes;
use super::super::{EditKey, ImeEvent, Pane};
use super::clipboard::Keys;

/// A page whose text fields take the keys: the focused one, if any.
pub(in crate::views) trait TextHost {
    /// Type `s` into the focused field. `false` when none is focused.
    fn handle_text(&mut self, s: &str) -> bool;

    /// An editing key for the focused field; Tab is the view's (it walks
    /// the projection), never an edit.
    fn handle_edit_key(&mut self, key: EditKey) -> bool;

    /// A step of the input method; `Disabled` drops a composition
    /// wherever one is, focused or not.
    fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool;

    /// Where the candidate window goes: the focused field's caret, laid
    /// out in `content`.
    fn ime_area(&self, content: Rect, theme: &Theme) -> Option<Rect>;

    /// A right-click: the field under the pointer takes the keys for its
    /// menu. `false` off the fields.
    fn right_click(&mut self, x: f32, y: f32, content: Rect, theme: &Theme) -> bool;

    /// Who has the keys (a chord, the field's menu).
    fn keys(&mut self) -> Keys<'_>;

    /// Escape blurs what has the focus. `false` when nothing had it.
    fn handle_escape(&mut self) -> bool;

    /// Enter, before the view's focus walk presses with it: a focused
    /// text area breaks the line. `false` (the default) leaves it to the
    /// walk.
    fn handle_enter(&mut self) -> bool {
        false
    }

    /// Up (`down` false) or Down, before the view's focus walk: a focused
    /// text area moves a line. `false` (the default) leaves it to the
    /// walk.
    fn handle_nav(&mut self, _down: bool) -> bool {
        false
    }
}

/// The table: one line per pane whose page is a [`TextHost`].
macro_rules! text_hosts {
    ($($pane:ident => $field:ident),* $(,)?) => {
        impl Panes {
            /// The page of `pane`, when it hosts text fields.
            pub(in crate::views) fn text_host(&self, pane: Option<Pane>) -> Option<&dyn TextHost> {
                match pane? {
                    $(Pane::$pane => Some(&self.$field),)*
                    _ => None,
                }
            }

            /// The page of `pane`, when it hosts text fields, to route to.
            pub(in crate::views) fn text_host_mut(
                &mut self,
                pane: Option<Pane>,
            ) -> Option<&mut dyn TextHost> {
                match pane? {
                    $(Pane::$pane => Some(&mut self.$field),)*
                    _ => None,
                }
            }

            /// `f` on every host, whichever page shows; `true` when it was
            /// for any.
            pub(in crate::views) fn each_text_host(
                &mut self,
                mut f: impl FnMut(&mut dyn TextHost) -> bool,
            ) -> bool {
                let mut any = false;
                $(any |= f(&mut self.$field);)*
                any
            }
        }
    };
}

text_hosts! {
    Input => input,
    // task-104 front A, fields (#154)
    Field => field,
    Label => label,
    InputGroup => input_group,
    InputOtp => input_otp,
    Textarea => textarea,
}
